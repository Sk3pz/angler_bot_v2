use serde::{Deserialize, Serialize};

use crate::{
    data_management::config::Config,
    fishing::fish_data::{fish::FishCategory, rarity::FishRarity},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Affects how much the bait affects the weights
pub enum BaitBias {
    Low,    // x1.5
    Medium, // x3.0
    High,   // x5
}

impl BaitBias {
    pub fn get_multiplier(&self) -> f32 {
        let config = Config::load();

        match self {
            BaitBias::Low => config.bait.low_bait_weight,
            BaitBias::Medium => config.bait.medium_bait_weight,
            BaitBias::High => config.bait.high_bait_weight,
        }
    }

    /// Returns a normalized value between 0.0 and 1.0 based on the highest configured weight.
    /// Used for Size/Weight biasing where we need a percentage shift rather than a raw multiplier.
    pub fn get_normalized_strength(&self) -> f32 {
        let config = Config::load();

        let val = match self {
            BaitBias::Low => config.bait.low_bait_weight,
            BaitBias::Medium => config.bait.medium_bait_weight,
            BaitBias::High => config.bait.high_bait_weight,
        };

        // Normalize against the highest possible value so High always equals 1.0 (100% bias)
        // If config is messed up and High is 0, default to 1.0 to avoid NaN.
        let max = config.bait.high_bait_weight.max(1.0);

        (val / max).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents what a given bait is good at attracting.
pub enum BaitAttraction {
    // --- Physical Attributes (Now using BaitBias) ---
    Heavy {
        bias: BaitBias,
    },
    Light {
        bias: BaitBias,
    },

    Large {
        bias: BaitBias,
    },
    Small {
        bias: BaitBias,
    },

    /// Attracts fish of a specific type
    /// RARITY MUST MATCH THIS FISH TYPE!
    SpecificFish {
        name: String, // the name of the fish type
        bias: BaitBias,
    },

    /// Will make it significantly more likely to catch fish of that rarity
    /// Some baits may have really good weight and size attraction but only attract common fish,
    ///  while others may be less effective at weight and size but attract rarer fish.
    Rarity(FishRarity, BaitBias),

    /// Will make it more likely to catch fish of that category
    Category(FishCategory, BaitBias),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bait {
    pub name: String,
    pub description: String,
    pub price: String,
    /// Chance the bait will be used up after a catch.
    /// 0.0 means it will never be used up, 1.0 means it will always be used up.
    /// some bait is one time use, some like lures can be used multiple times
    pub use_chance: f32,
    pub attraction: Vec<BaitAttraction>,
}

impl Bait {
    /// Returns: (Target Name, Multiplier)
    pub fn get_specific_fish_modifier(&self) -> Option<(&String, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::SpecificFish { name, bias } => Some((name, bias.get_multiplier())),
            _ => None,
        })
    }

    /// Returns: (Target Rarity, Multiplier)
    pub fn get_rarity_modifier(&self) -> Option<(FishRarity, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::Rarity(r, bias) => Some((r.clone(), bias.get_multiplier())),
            _ => None,
        })
    }

    /// Returns: (Target Category, Multiplier)
    pub fn get_category_modifier(&self) -> Option<(FishCategory, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::Category(c, bias) => Some((c.clone(), bias.get_multiplier())),
            _ => None,
        })
    }

    /// Calculates the shift for weight.
    /// Returns a value between -1.0 (Light) and 1.0 (Heavy).
    pub fn get_weight_bias(&self) -> f32 {
        let mut total_bias = 0.0;

        for attr in &self.attraction {
            match attr {
                // Add the normalized strength (0.0 to 1.0)
                BaitAttraction::Heavy { bias } => total_bias += bias.get_normalized_strength(),
                // Subtract the normalized strength
                BaitAttraction::Light { bias } => total_bias -= bias.get_normalized_strength(),
                _ => {}
            }
        }
        total_bias.clamp(-1.0, 1.0)
    }

    /// Calculates the shift for size.
    /// Returns a value between -1.0 (Small) and 1.0 (Large).
    pub fn get_size_bias(&self) -> f32 {
        let mut total_bias = 0.0;

        for attr in &self.attraction {
            match attr {
                BaitAttraction::Large { bias } => total_bias += bias.get_normalized_strength(),
                BaitAttraction::Small { bias } => total_bias -= bias.get_normalized_strength(),
                _ => {}
            }
        }
        total_bias.clamp(-1.0, 1.0)
    }
}
