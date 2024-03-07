use actix_web::{rt::spawn, web::Json};
use aws_sdk_s3::primitives::ByteStream;
use std::{env, error::Error, sync::Arc, time::Duration};
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

    pub async fn download_object(&self, key: String) -> Result<Vec<u8>, Box<dyn Error>> {
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
        let data = stream.collect().await.map(|data| data.to_vec()).unwrap();

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

                    // Sleep to simulate some work.
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                drop(tx);
            }
        });

        let compress_task = spawn(async move {
            while let Some((name, object_data)) = rx.recv().await {
                // Receive the message from the first thread.
                println!("Thread nro 2 receives name: {}", name);

                // Sleep to simulate some work.
                tokio::time::sleep(Duration::from_millis(200)).await;
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
