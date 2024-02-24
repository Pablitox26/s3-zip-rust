use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ResponseBody<T> {
    pub message: String,
    pub data: T,
}