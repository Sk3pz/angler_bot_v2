use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub name: String,
    pub strength: u32,
}
