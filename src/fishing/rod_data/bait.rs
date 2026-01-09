use rand::prelude::IndexedRandom;
use rand::Rng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{data_management::config::Config, fishing::fish_data::{fish::FishCategory, rarity::FishRarity}, nay};

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
    pub price: f32,
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

#[derive(Debug, Clone, Copy)]
pub enum BaitPotency {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaitData {
    pub base_names: Vec<String>,
}

impl BaitData {
    pub fn load() -> Self {
        let path = "./data/gamedata/bait.ron";
        match std::fs::read_to_string(path) {
            Ok(content) => ron::from_str(&content).unwrap_or_else(|e| {
                nay!("Failed to parse bait.ron: {}", e);
                Self { base_names: vec!["Worm".to_string()] }
            }),
            Err(_) => {
                nay!("Could not find bait.ron");
                Self { base_names: vec!["Worm".to_string()] }
            }
        }
    }
}

impl Bait {
    /// Generates a random bait based on the provided potency.
    pub fn generate(potency: BaitPotency) -> Self {
        let mut rng = rand::rng();
        let data = BaitData::load();
        let base_name = data.base_names.choose(&mut rng).cloned().unwrap_or_else(|| "Bait".to_string());

        let mut attractions = Vec::new();

        // Define generation rules based on user request
        match potency {
            BaitPotency::Low => {
                // 1-2 Low bias values
                let count = rng.random_range(1..=2);
                for _ in 0..count {
                    attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Low));
                }
            },
            BaitPotency::Medium => {
                // 1 Medium, and maybe 1 Low
                attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Medium));
                if rng.random_bool(0.5) {
                    attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Low));
                }
            },
            BaitPotency::High => {
                // 1-2 High, 1 Medium, 1-2 Small (Low)
                let high_count = rng.random_range(1..=2);
                for _ in 0..high_count {
                    attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::High));
                }
                attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Medium));

                let low_count = rng.random_range(1..=2);
                for _ in 0..low_count {
                    attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Low));
                }
            },
        }

        let name = Self::generate_name(&base_name, &attractions);
        let description = Self::generate_description(&attractions);

        // Price calculation based on potency roughly
        let price = match potency {
            BaitPotency::Low => rng.random_range(5.0..20.0),
            BaitPotency::Medium => rng.random_range(50.0..150.0),
            BaitPotency::High => rng.random_range(300.0..800.0),
        };

        Bait {
            name,
            description,
            price: ((price * 100.0) as f32).round() / 100.0,
            use_chance: 1.0, // Standard baits are single use
            attraction: attractions,
        }
    }

    fn generate_random_attraction(rng: &mut impl Rng, bias: BaitBias) -> BaitAttraction {
        // Randomly decide what type of attraction this is
        // 0: Size (Large/Small)
        // 1: Weight (Heavy/Light)
        // 2: Category
        // 3: Rarity
        let roll = rng.random_range(0..100);

        match roll {
            0..=29 => {
                if rng.random_bool(0.5) {
                    BaitAttraction::Large { bias }
                } else {
                    BaitAttraction::Small { bias }
                }
            },
            30..=59 => {
                if rng.random_bool(0.5) {
                    BaitAttraction::Heavy { bias }
                } else {
                    BaitAttraction::Light { bias }
                }
            },
            60..=84 => {
                // Pick a random category
                let categories = [
                    FishCategory::BaitFish, FishCategory::Schooling, FishCategory::Predatory,
                    FishCategory::BottomFeeder, FishCategory::Ornamental, FishCategory::Forager,
                    FishCategory::Apex, FishCategory::Abyssal, FishCategory::Mythological
                ];
                let cat = categories.choose(rng).unwrap().clone();
                BaitAttraction::Category(cat, bias)
            },
            _ => {
                // Pick a random rarity
                // Weighted slightly towards common/uncommon for generation sanity,
                // but High bias might allow higher rarities if we wanted more complex logic.
                let rarities = [
                    FishRarity::Common, FishRarity::Uncommon, FishRarity::Rare,
                    FishRarity::Elusive, FishRarity::Legendary
                ];
                let rar = rarities.choose(rng).unwrap().clone();
                BaitAttraction::Rarity(rar, bias)
            }
        }
    }

    fn generate_name(base: &str, attractions: &[BaitAttraction]) -> String {
        // Find the most "potent" attraction to name the bait after
        // Priority: Category > Rarity > Size/Weight

        let mut prefix = "";

        // 1. Check for Category
        if let Some(cat_attr) = attractions.iter().find(|a| matches!(a, BaitAttraction::Category(_, _))) {
            if let BaitAttraction::Category(cat, _) = cat_attr {
                prefix = match cat {
                    FishCategory::BaitFish => "Feeder",
                    FishCategory::Schooling => "Swarming",
                    FishCategory::Predatory => "Hunter's",
                    FishCategory::BottomFeeder => "Muddy",
                    FishCategory::Ornamental => "Shiny",
                    FishCategory::Forager => "Forager's",
                    FishCategory::Apex => "Apex",
                    FishCategory::Abyssal => "Abyssal",
                    FishCategory::Mythological => "Mystic",
                };
            }
        }
        // 2. Check for Rarity if no Category found
        else if let Some(rar_attr) = attractions.iter().find(|a| matches!(a, BaitAttraction::Rarity(_, _))) {
            if let BaitAttraction::Rarity(rar, _) = rar_attr {
                prefix = match rar {
                    FishRarity::Common => "Common",
                    FishRarity::Uncommon => "Uncommon",
                    FishRarity::Rare => "Rare",
                    FishRarity::Elusive => "Elusive",
                    FishRarity::Legendary => "Legendary",
                    FishRarity::Mythical => "Godly",
                };
            }
        }
        // 3. Check for Stats
        else if let Some(stat_attr) = attractions.iter().find(|a| matches!(a, BaitAttraction::Large{..} | BaitAttraction::Heavy{..} | BaitAttraction::Small{..} | BaitAttraction::Light{..})) {
            match stat_attr {
                BaitAttraction::Large { .. } => prefix = "Big",
                BaitAttraction::Small { .. } => prefix = "Tiny",
                BaitAttraction::Heavy { .. } => prefix = "Heavy",
                BaitAttraction::Light { .. } => prefix = "Light",
                _ => {}
            }
        }

        if prefix.is_empty() {
            // Fallback for standard baits
            format!("Standard {}", base)
        } else {
            format!("{} {}", prefix, base)
        }
    }

    fn generate_description(attractions: &[BaitAttraction]) -> String {
        // Simple description generation
        let attr_desc: Vec<String> = attractions.iter().map(|a| match a {
            BaitAttraction::Heavy { .. } => "heavier fish".to_string(),
            BaitAttraction::Light { .. } => "lighter fish".to_string(),
            BaitAttraction::Large { .. } => "larger fish".to_string(),
            BaitAttraction::Small { .. } => "smaller fish".to_string(),
            BaitAttraction::Category(c, _) => format!("{:?} fish", c),
            BaitAttraction::Rarity(r, _) => format!("{} fish", r),
            BaitAttraction::SpecificFish { name, .. } => format!("{}", name),
            _ => "fish".to_string(),
        }).collect();

        format!("A bait that attracts {}.", attr_desc.join(", "))
    }
}