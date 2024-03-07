use actix_web::{
    rt::spawn,
    web::{Bytes, Json},
};
use aws_sdk_s3::primitives::ByteStream;
use std::io::Write;
use std::{env, error::Error, fs::File, sync::Arc};
use tokio::sync::mpsc;

use crate::models::file_s3::FileS3;

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

    pub async fn compress_objects(
        &self,
        files: Json<Vec<FileS3>>,
    ) -> Result<String, Box<dyn Error>> {
        let (tx, mut rx) = mpsc::channel(1);

        let s3_service = Arc::new(self.clone());

        let download_task = spawn({
            let s3_service = Arc::clone(&s3_service);
            async move {
                for file in files.iter() {
                    println!("Thread nro 1 sends: {:?}", file);

                    match s3_service.download_object(file.key.to_string()).await {
                        Ok(object_data) => {
                            // Send the message to the second thread
                            tx.send((file.name.to_string(), object_data)).await.unwrap();
                        }
                        Err(err) => eprintln!("Error al descargar objeto {}: {}", &file.key, err),
                    }
                }
                drop(tx);
            }
        });

        let zip_file = File::create("./test-zip")?;
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);

        let compress_task = spawn(async move {
            let mut zip = zip::ZipWriter::new(zip_file);

            while let Some((name, object_data)) = rx.recv().await {
                // Receive the message from the first thread.
                println!("Thread nro 2 receives name: {}", name);

                // Start file in zip
                if let Err(err) = zip.start_file(&name, options.clone()) {
                    eprintln!("Error starting file in zip: {}", err);
                }

                // Write compressed data to zip
                if let Err(err) = zip.write_all(&object_data) {
                    eprintln!("Error writing to zip: {}", err);
                }
            }

            if let Err(err) = zip.finish() {
                eprintln!("Error finishing zip file: {}", err);
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
