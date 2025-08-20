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
    pub bf_results: Vec<(usize, std::string::String, std::string::String, std::string::String, std::string::String, std::time::Duration)>,
    pub bf_request: String,
}