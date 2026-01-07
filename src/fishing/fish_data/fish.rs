// fish data will be loaded from a data file

use crate::data_management::config::{Config, ValueCalculationType};
use crate::fishing::Attribute;
use crate::fishing::fish_data::rarity::FishRarity;
use crate::fishing::rod_data::bait::Bait;
use crate::{data_management::monetary::MonetaryAmount, error::ReelError, fishing::depth::Depth};
use rand_distr::{Distribution, weighted::WeightedIndex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FishCategory {
    /// Small, Common fish
    BaitFish,
    /// Fish that are more likely to be found and are generally easier to catch
    Schooling,
    /// Agressive fish that are harder to catch (higher line snap chance, requires stronger lines and rods)
    Predatory,
    /// Fish that are more likely to be caught if your sinker is near the bottom of the depth category
    BottomFeeder,
    /// Trophy fish that are rare and valuable, but harder to catch
    Ornamental,
    /// Fish that don't fit other niches
    Forager,
    /// i.e. Shark, Marlin, etc.
    Apex,
    /// Rare fish found at the bottom of the ocean, often of high value but also very difficult to catch
    Abyssal,
}

impl FishCategory {
    pub fn fight_multiplier(&self) -> f32 {
        match self {
            FishCategory::Apex => 2.5,                              // Boss fight!
            FishCategory::Predatory | FishCategory::Abyssal => 1.5, // Tough fight
            FishCategory::BottomFeeder => 1.2,                      // Moderate fight
            FishCategory::Schooling => 0.8,                         // Weak individually
            FishCategory::BaitFish => 0.1,                          // No fight
            FishCategory::Ornamental => 0.2,                        // Gentle
            _ => 1.0,                                               // Standard
        }
    }
}

// TODO: Fish generation should also take into account the Rod Loadout used.

/// Represents a type of fish that can be caught.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishType {
    /// The name of the fish, e.g. "Salmon", "Trout", etc.
    pub name: String,
    /// the rarity of the fish, or how likely it is to show up when fishing
    pub rarity: FishRarity,
    /// the category of the fish, which can affect how it behaves and what it is attracted to
    pub category: FishCategory,
    /// The size range of the fish and the average in inches
    pub size_range: Attribute,
    /// The weight range the fish can be and the average in pounds
    pub weight_range: Attribute,
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

    fn apply_bias(attr: &mut Attribute, bias: f32) {
        if bias == 0.0 {
            return;
        }

        let shift = if bias > 0.0 {
            // Positive Bias: Calculate distance to MAX and add percentage
            (attr.max - attr.average) * bias
        } else {
            // Negative Bias: Calculate distance to MIN and subtract percentage
            // (bias is negative here, so adding it actually subtracts)
            (attr.average - attr.min) * bias
        };

        attr.average += shift;
    }

    pub fn generate_fish(&self, caught_depth: f32, bait: Option<&Bait>) -> Result<Fish, ReelError> {
        // Clone the ranges so we can modify them temporarily for this generation
        let mut size_range = self.size_range.clone();
        let mut weight_range = self.weight_range.clone();

        if let Some(bait) = bait {
            // The helper function makes this much more readable
            Self::apply_bias(&mut size_range, bait.get_size_bias());
            Self::apply_bias(&mut weight_range, bait.get_weight_bias());
        }

        // Generate values using the skewed ranges
        // Note: randomness is preserved by the triangular distribution
        let size = size_range.triangular_rand()?;
        let weight = weight_range.triangular_rand()?;

        // ... Value Calculation ...
        let config = Config::load();
        let value = match config.fishing.fish_value_calculation {
            ValueCalculationType::Averaged => self.averaged_value(size, weight),
            ValueCalculationType::Multiplicitive => self.multiplicative_value(size, weight),
        };

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
    pub fn load() -> Self {
        let raw_path = "./data/gamedata/fish_types.ron".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        if !path.exists() {
            panic!("Failed to load fish types: file does not exist");
            // program should not continue if the file does not exist, as it is required for the game to function properly
        }

        let contents = std::fs::read_to_string(path).unwrap();

        ron::from_str(contents.as_str()).unwrap()
    }

    pub fn save(&self) -> Result<(), ReelError> {
        let raw_path = "./data/gamedata/fish_types.ron".to_string();
        let path = std::path::Path::new(raw_path.as_str());

        let contents = ron::to_string(self).unwrap();

        std::fs::write(path, contents)?;

        Ok(())
    }

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
    pub fn generate_fish(
        &self,
        raw_depth: f32,
        bait: Option<&Bait>,
    ) -> Result<Option<Fish>, ReelError> {
        // generate a weighted rarity
        let rarity = FishRarity::weighted_random(bait);

        let depth = Depth::from_depth(raw_depth);
        let available_fish = self.get_available_fish(depth, rarity);

        if available_fish.fish_types.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::rng();

        // Resolve Bait Modifiers ONCE (Optimization)
        // We don't want to load Config inside the loop for every single fish type
        let category_mod = bait.and_then(|b| b.get_category_modifier());
        let specific_mod = bait.and_then(|b| b.get_specific_fish_modifier());

        let weights: Vec<f32> = available_fish
            .fish_types
            .iter()
            .map(|f| {
                let mut weight = 1.0;

                // Apply Category Bias
                if let Some((target_cat, mult)) = &category_mod {
                    if f.category == *target_cat {
                        weight *= *mult; // e.g., x3.0
                    }
                }

                // Apply Specific Fish Bias (Stacks with Category)
                if let Some((target_name, mult)) = specific_mod {
                    if f.name == *target_name {
                        weight *= mult; // e.g., x5.0
                    }
                }

                weight
            })
            .collect();

        let dist = WeightedIndex::new(&weights)
            .map_err(|_| ReelError::RandomError("Failed to create weighted index".into()))?;

        let fish_type = &available_fish.fish_types[dist.sample(&mut rng)];

        Ok(Some(fish_type.generate_fish(raw_depth, bait)?))
    }
}
