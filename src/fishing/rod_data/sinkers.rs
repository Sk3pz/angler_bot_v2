use serde::{Deserialize, Serialize};

use crate::{
    error::ReelError,
    fishing::{Attribute, depth::Depth},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sinker {
    pub name: String,
    /// weight of the sinker will add to the strain of the fish on the line and rod
    pub weight: f32,
    /// The depth category that the sinker is effective for.
    /// This will affect the types of fish that can be caught with it.
    pub depth_range: Attribute,
}

impl Sinker {
    pub fn generate_depth(&self) -> Result<f32, ReelError> {
        // generate a random depth within the sinker's depth range
        self.depth_range.triangular_rand()
    }

    /// All depth categories encompassed by the sinker's depth range
    pub fn get_effective_depth_categories(&self) -> Vec<Depth> {
        let mut categories = Vec::new();
        for depth in Depth::iter() {
            let (min, max) = depth.get_range();
            if self.depth_range.min <= max && self.depth_range.max >= min {
                categories.push(depth);
            }
        }
        categories
    }
}
