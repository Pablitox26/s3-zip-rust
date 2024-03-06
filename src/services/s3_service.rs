use std::{env, error::Error, sync::mpsc, time::Duration};
use actix_web::{rt::spawn, web::Json};
use aws_sdk_s3::primitives::ByteStream;

use crate::models::file_s3::FileS3;

pub struct S3Service {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3Service {
    pub fn new(client: aws_sdk_s3::Client) -> Self {
        let bucket = env_var_or_err("AWS_S3_BUCKET_NAME");
        S3Service {
            client,
            bucket,
        }
    }

    pub async fn download_object(&self, key: &str) -> Result<Vec<u8>, Box<dyn Error>> {
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
        files:  Json<Vec<FileS3>>
    ) -> Result<String, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel();

        let download_task = spawn(async move {
            for file in files.iter() {
    
                println!("Thread nro 1 sends: {:?}", file);
    
                // Send the message to the second thread
                tx.send((file.name.to_string(), file.key.to_string())).unwrap();
                
                 // Sleep to simulate some work.
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            drop(tx);
        });
    
        let compress_task = spawn(async move {
            for file in rx.iter() {
                // Receive the message from the first thread.
                println!("Thread nro 2 receives name: {}, key: {}", file.0, file.1);
    
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