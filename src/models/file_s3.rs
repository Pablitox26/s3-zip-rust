use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct FileS3 {
    pub key: String,
    pub name: String,
}