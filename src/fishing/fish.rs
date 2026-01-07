// fish data will be loaded from a data file

use crate::data_management::config::{Config, ValueCalculationType};
use crate::fishing::rarity::FishRarity;
use crate::{data_management::monetary::MonetaryAmount, error::ReelError, fishing::depth::Depth};
use rand::prelude::IndexedRandom;
use rand_distr::Distribution;
use rand_distr::Triangular;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishAttribute {
    pub min: f32,
    pub max: f32,
    pub average: f32,
}

impl FishAttribute {
    pub fn normal_rand(&self) -> Result<f32, ReelError> {
        let mut rng = rand::rng();

        let dist: Triangular<f32> = Triangular::new(self.min, self.max, self.average)?;

        Ok(dist.sample(&mut rng))
    }
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

impl FishType {
    fn averaged_value(&self, size: f32, weight: f32) -> f32 {
        // Calculate a multiplier based on how the stats compare to the average
        // Average the two ratios so size and weight contribute equally
        let stat_multiplier =
            ((size / self.size_range.average) + (weight / self.weight_range.average)) / 2.0;

        // clamp value so it can't go below 1
        let value = self.base_value * stat_multiplier;

        value
    }

    fn multiplicative_value(&self, size: f32, weight: f32) -> f32 {
        // If a fish is small AND light, the penalties stack.
        // If a fish is long AND heavy, the bonuses stack.
        let value = self.base_value
            * (size / self.size_range.average)
            * (weight / self.weight_range.average);

        value
    }

    pub fn generate_fish(&self, caught_depth: f32) -> Result<Fish, ReelError> {
        // normal distribution around the average size and weight, with a max and min value
        let size = self.size_range.normal_rand()?;
        let weight = self.weight_range.normal_rand()?;

        // get the config to see if we are using averaged or multiplicative
        let config = Config::load();
        let value = match config.fishing.fish_value_calculation {
            ValueCalculationType::Averaged => self.averaged_value(size, weight),
            ValueCalculationType::Multiplicitive => self.multiplicative_value(size, weight),
        };

        // round value to two decimal places:
        let value = (value.max(1.0) * 100.0).round() / 100.0;

        let value = MonetaryAmount::new(value);

        Ok(Fish {
            name: self.name.clone(),
            size,
            weight,
            depth: caught_depth,
            value,
        })
    }
}

// Fish caught does not store rarity, as rarity only affects likelihood of the fish
//  being caught. This is different from V1 of Angler Bot

/// Represents an individual fish that can be caught.
/// TODO: look into storing this in a user's file so they can see their collection of catches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fish {
    pub name: String,
    /// size of the fish in inches
    pub size: f32,
    /// the weight of the fish in pounds
    pub weight: f32,
    /// the depth in feet the fish was caught at
    pub depth: f32,
    /// the value of the fish in $
    pub value: MonetaryAmount,
}

/// Represents a collection of catchable fish types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pond {
    pub fish_types: Vec<FishType>,
}

impl Pond {
    /// Get all fish that have depth ranges that overlap the given depth and are of a given rarity or lower.
    /// Will return a Pond containing only the fish that meet the requirements
    /// The list can be empty if there are no fish that meet the requirements
    fn get_available_fish(&self, depth: Depth, rarity: FishRarity) -> Pond {
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
    pub fn generate_fish(&self, raw_depth: f32) -> Result<Option<Fish>, ReelError> {
        // generate a weighted rarity
        let rarity = FishRarity::weighted_random();

        // get the respective depth category for the given depth
        let depth = Depth::from_depth(raw_depth);

        // collect all fish that can spawn at the given depth and rarity
        let available_fish = self.get_available_fish(depth, rarity);

        // generate a fish from the new pond
        // if there are no available fish types, return None
        if available_fish.fish_types.is_empty() {
            return Ok(None);
        }

        // randomly select a fish type from the available fish types
        let fish_type = available_fish.fish_types.choose(&mut rand::rng()).unwrap();

        Ok(Some(fish_type.generate_fish(raw_depth)?))
    }
}
