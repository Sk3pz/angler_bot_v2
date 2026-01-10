// this folder is where all fishing related code goes (except for commands)

use rand_distr::{Distribution, Triangular};
use serde::{Deserialize, Serialize};

use crate::error::ReelError;

pub mod depth;
pub mod fish_data;
pub mod rod_data;
pub mod shop;
pub mod bait_bucket;
pub mod inventory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub min: f32,
    pub max: f32,
    pub average: f32,
}

impl Attribute {
    pub fn triangular_rand(&self) -> Result<f32, ReelError> {
        let mut rng = rand::rng();

        let dist: Triangular<f32> = Triangular::new(self.min, self.max, self.average)?;

        Ok(dist.sample(&mut rng))
    }
}
