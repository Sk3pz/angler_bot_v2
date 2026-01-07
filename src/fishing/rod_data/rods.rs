use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RodBase {
    pub name: String,
    /// Multiplies to Line strength
    pub strength_bonus: f32,
    /// Multiplies Reel speed
    pub efficiency_multiplier: f32,
}
