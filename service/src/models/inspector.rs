use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InspectorEntry {
    pub id: usize,
    pub request: String,
    pub response: String,
    pub modified_request: String,
    pub new_response: String,
    pub ssl: bool,
    pub target: String,
}