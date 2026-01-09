use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RodBase {
    pub name: String,
    pub price: f32,
    /// How likely it is to hook a fish (affects catch chance)
    /// 1.0 = Standard chance
    /// 1.15 = 15% higher chance to hook
    pub sensitivity: f32,
    /// Multiplies to Line strength
    /// 1.0 is no bonus, 1.5 is 50% stronger, etc.
    pub strength_bonus: f32,
    /// Multiplies Reel speed
    /// 1.0 is no bonus, 1.5 is 50% faster, etc.
    pub efficiency_multiplier: f32,
}
