// fish data will be loaded from a data file

use crate::fishing::depth::Depth;
use crate::fishing::rarity::FishRarity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishAttribute {
    pub min: f32,
    pub max: f32,
    pub average: f32,
}

/// Represents a type of fish that can be caught.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishType {
    /// The name of the fish, e.g. "Salmon", "Trout", etc.
    pub name: String,
    /// the rarity of the fish, or how likely it is to show up when fishing
    pub rarity: FishRarity,
    /// The size range of the fish and the average in inches
    pub size_range: FishAttribute,
    /// The weight range the fish can be and the average in pounds
    pub weight_range: FishAttribute,
    /// The Depth range the fish can be found at in feet
    pub depth_range: (f32, f32),
    /// the base value of the fish in $ (can be higher or lower depending on the size and weight of the fish)
    pub base_value: f32,
}

/// Represents all fish that can be caught in the world.
/// This will be loaded in from ./data/fish.data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishTank {
    pub fish_types: Vec<FishType>,
}

impl FishTank {
    /// Get all fish that have depth ranges that overlap the given depth and are of a given rarity or lower.
    /// Will return a FishTank containing only the fish that meet the requirements
    fn get_available_fish(&self, depth: Depth, rarity: FishRarity) -> FishTank {
        Self {
            fish_types: self
                .fish_types
                .iter()
                .filter(|fish| {
                    let (min_depth, max_depth) = fish.depth_range;
                    let (depth_min, depth_max) = depth.get_range();
                    let possible_rarities = rarity.get_possible();
                    // use OR instead of AND because we want to include fish that have depth ranges that overlap the given depth
                    ((min_depth <= depth_max) || (max_depth >= depth_min))
                    // check if the fish's rarity is in the possible rarities
                        && possible_rarities.contains(&fish.rarity)
                })
                .cloned()
                .collect(),
        }
    }

    /// Generate a random fish within the depth.
    /// Will return None if there are no fish of that rarity that can be caught at the given depth.
    pub fn generate_fish(&self, depth: Depth) -> Option<Fish> {
        // generate a weighted rarity
        let rarity = FishRarity::weighted_random();

        // collect all fish that can spawn at the given depth
        let available_fish = self.get_available_fish(depth, rarity);

        // get all fish of the generated rarities at the depth

        todo!()
    }
}

// Fish caught does not store rarity, as rarity only affects likelihood of the fish
//  being caught. This is different from V1 of Angler Bot

/// Represents an individual fish that can be caught.
/// TODO: look into storing this in a user's file so they can see their collection of catches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fish {
    pub name: String,
    pub size: f32,   // the size of the fish in inches
    pub weight: f32, // the weight of the fish in pounds
    pub depth: f32,  // the depth in feet the fish was caught at
    pub value: f32,  // the value of the fish in $
}
