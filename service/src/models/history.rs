use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HistoryEntry {
    pub id: i64,
    pub remote_addr: String,
    pub uri: String,
    pub method: String,
    pub params: bool,
    pub status: i64,
    pub size: i64,
    pub timestamp: String,
    pub raw: Vec<u8>,
    pub ssl: bool,
    pub response: Vec<u8>,
    pub response_time: String,
    pub content_length: i64,
}