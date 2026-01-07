use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub name: String,
    /// Maximum weight (in lbs) that the line can support before snapping.
    pub strength: u32,
}
