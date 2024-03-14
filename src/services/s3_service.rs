use actix_web::{
    rt::spawn,
    web::{Bytes, Json},
};
use aws_sdk_s3::{
    operation::{
        complete_multipart_upload::CompleteMultipartUploadOutput,
        create_multipart_upload::CreateMultipartUploadOutput,
    },
    primitives::{ByteStream, SdkBody},
    types::{CompletedMultipartUpload, CompletedPart},
};
use std::io::Write;
use std::{env, error::Error, io::Cursor, sync::Arc};
use tokio::sync::mpsc;

use crate::models::file_s3::ZipFileS3;

#[derive(Debug, Clone)]
pub struct S3Service {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3Service {
    pub fn new(client: aws_sdk_s3::Client) -> Self {
        let bucket = env_var_or_err("AWS_S3_BUCKET_NAME");
        S3Service { client, bucket }
    }

    pub async fn download_object(&self, key: String) -> Result<Bytes, Box<dyn Error>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .unwrap();

        let body_stream = response.body.into_inner();
        let stream = ByteStream::new(body_stream);
        let data = stream
            .collect()
            .await
            .map(|data| data.into_bytes())
            .unwrap();

        Ok(data)
    }

    async fn create_multipart_upload(&self, key: String) -> Result<String, Box<dyn Error>> {
        let multipart_upload_res: CreateMultipartUploadOutput = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(&key)
            .send()
            .await
            .unwrap();

        let upload_id = multipart_upload_res.upload_id().unwrap();
        Ok(upload_id.to_string())
    }

    async fn upload_part(
        &self,
        key: String,
        stream: ByteStream,
        part_number: i32,
        upload_id: String,
    ) -> Result<CompletedPart, Box<dyn Error>> {
        let upload_part_res = self
            .client
            .upload_part()
            .key(&key)
            .bucket(&self.bucket)
            .upload_id(upload_id)
            .body(stream)
            .part_number(part_number)
            .send()
            .await?;

        Ok(CompletedPart::builder()
            .e_tag(upload_part_res.e_tag.unwrap_or_default())
            .part_number(part_number)
            .build())
    }

    async fn complete_multipart_upload(
        &self,
        key: String,
        upload_id: String,
        upload_parts: Vec<CompletedPart>,
    ) -> Result<CompleteMultipartUploadOutput, Box<dyn Error>> {
        let completed_multipart_upload: CompletedMultipartUpload =
            CompletedMultipartUpload::builder()
                .set_parts(Some(upload_parts))
                .build();

        let complete_multipart_upload_res = self
            .client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(&key)
            .multipart_upload(completed_multipart_upload)
            .upload_id(upload_id)
            .send()
            .await
            .unwrap();
        Ok(complete_multipart_upload_res)
    }

    pub async fn compress_objects(
        &self,
        zip_file_s3: Json<ZipFileS3>,
    ) -> Result<String, Box<dyn Error>> {
        let zip_file_s3 = zip_file_s3.into_inner();
        let files = zip_file_s3.files;
        let key_zip = zip_file_s3.key_zip;

        let (tx, mut rx) = mpsc::channel(1);

        let s3_service = Arc::new(self.clone());

        let download_task = spawn({
            let s3_service = Arc::clone(&s3_service);
            async move {
                for file in files {
                    println!("Thread nro 1 sends: {:?}", file);

                    match s3_service.download_object(file.key.to_string()).await {
                        Ok(object_data) => {
                            // Send the message to the second thread
                            tx.send((file.name.to_string(), object_data)).await.unwrap();
                        }
                        Err(err) => eprintln!("Error downloading object {}: {}", &file.key, err),
                    }
                }
                drop(tx);
            }
        });

        let compress_task = spawn({
            let s3_service = Arc::clone(&s3_service);
            async move {
                let upload_id = s3_service
                    .create_multipart_upload(key_zip.clone())
                    .await
                    .unwrap();
                let mut upload_parts: Vec<CompletedPart> = Vec::new();
                let mut part_number = 1;

                while let Some((name, object_data)) = rx.recv().await {
                    // Receive the message from the first thread.
                    println!("Thread nro 2 receives name: {}", name);

                    let mut data = Vec::new();

                    {
                        let buff = Cursor::new(&mut data);
                        let mut zip = zip::ZipWriter::new(buff);
                        let options = zip::write::FileOptions::default()
                            .compression_method(zip::CompressionMethod::Stored)
                            .unix_permissions(0o755);

                        // Start file in zip
                        if let Err(err) = zip.start_file(&name, options) {
                            eprintln!("Error starting file in zip: {}", err);
                        }

                        // Write compressed data to zip
                        if let Err(err) = zip.write_all(&object_data) {
                            eprintln!("Error writing to zip: {}", err);
                        }
                    }

                    let stream = ByteStream::new(SdkBody::from(data));

                    let completed_part = s3_service
                        .upload_part(key_zip.clone(), stream, part_number, upload_id.clone())
                        .await
                        .unwrap();

                    upload_parts.push(completed_part);
                    part_number += 1;
                }

                let complete_multipart_upload = s3_service
                    .complete_multipart_upload(key_zip, upload_id, upload_parts)
                    .await
                    .unwrap();

                println!("Complete multipart upload: {:?}", complete_multipart_upload);
            }
        });

        // We wait for both threads to finish.
        tokio::try_join!(download_task, compress_task).unwrap();

        Ok("Finish".to_string())
    }
}

fn env_var_or_err(name: &str) -> String {
    match env::var(name) {
        Ok(name) => name.to_string(),
        Err(_) => format!("Error loading env variable {}", name),
    }
}
