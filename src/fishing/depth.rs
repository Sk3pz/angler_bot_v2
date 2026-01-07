use rand::Rng;
use serde::{Deserialize, Serialize};

/// Represents the depth categories
/// max depth of each category is not inclusive
/// absolute max is 10k feet
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Depth {
    /// 0..60 feet
    Shallow,
    /// 60..200 feet
    MidWater,
    /// 200..1000 feet
    Deep,
    /// 1000..4000 feet
    Abyssal,
    /// 4000+ feet (4000..=10000 feet)
    Hadal,
}

impl Depth {
    /// Get the depth range in feet for the given depth category
    pub fn get_range(&self) -> (f32, f32) {
        match self {
            Depth::Shallow => (0.0, 60.0),
            Depth::MidWater => (60.0, 200.0),
            Depth::Deep => (200.0, 1000.0),
            Depth::Abyssal => (1000.0, 4000.0),
            Depth::Hadal => (4000.0, 10000.0),
        }
    }

    /// Get the depth category from a given depth in feet
    pub fn from_depth(depth: f32) -> Self {
        match depth {
            d if d >= 0.0 && d < 60.0 => Depth::Shallow,
            d if d >= 60.0 && d < 200.0 => Depth::MidWater,
            d if d >= 200.0 && d < 1000.0 => Depth::Deep,
            d if d >= 1000.0 && d < 4000.0 => Depth::Abyssal,
            _ => Depth::Hadal,
        }
    }

    /// Get a random depth in feet within the range of the given depth category
    /// Will give a random depth in feet within 2 decimal places of precision
    pub fn random_depth(&self) -> f32 {
        let (min, max) = self.get_range();
        let mut rng = rand::rng();
        let depth = rng.random_range(min..=max);
        (depth * 100.0).round() / 100.0
    }

    pub fn iter() -> impl Iterator<Item = Depth> {
        [
            Depth::Shallow,
            Depth::MidWater,
            Depth::Deep,
            Depth::Abyssal,
            Depth::Hadal,
        ]
        .iter()
        .cloned()
    }
}
