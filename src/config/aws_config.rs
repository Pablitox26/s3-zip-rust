use std::{env, error::Error};

use aws_config::{Region, SdkConfig};
use aws_credential_types::Credentials;

pub struct AwsConfig {
    pub sdk_config: SdkConfig,
}

impl AwsConfig {
    pub async fn init() -> Result<Self, Box<dyn Error>> {

        let region = env_var_or_err("AWS_REGION");
        let access_key = env_var_or_err("AWS_ACCESS_KEY_ID");
        let secret_key = env_var_or_err("AWS_SECRET_ACCESS_KEY");

        let sdk_config = aws_config::from_env()
            .region(Region::new(region))
            .credentials_provider(
                Credentials::new(
                    &access_key, 
                    &secret_key, 
                    None, 
                    None, 
                    "example"))
            .load()
            .await;

        Ok(AwsConfig {sdk_config})
    }

    pub fn client_s3(&self) -> aws_sdk_s3::Client {
        aws_sdk_s3::Client::new(&self.sdk_config)
    }
}

fn env_var_or_err(name: &str) -> String {
    match env::var(name) {
        Ok(name) => name.to_string(),
        Err(_) => format!("Error loading env variable {}", name),
    }
}