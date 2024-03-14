use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct FileS3 {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZipFileS3 {
    pub key_zip: String,
    pub files: Vec<FileS3>,
}