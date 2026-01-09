use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reel {
    pub name: String,
    pub price: f32,
    /// The multiplier to the speed of reeling in the line. Higher means faster reeling.
    /// 1.0 means normal speed, 2.0 means twice as fast, 0.5 means half as fast.
    /// note: weight also affects speed
    pub speed_multiplier: f32,
}
