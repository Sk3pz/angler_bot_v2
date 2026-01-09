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

    pub fn get_normalized_strength(&self) -> f32 {
        let config = Config::load();

        let val = match self {
            BaitBias::Low => config.bait.low_bait_weight,
            BaitBias::Medium => config.bait.medium_bait_weight,
            BaitBias::High => config.bait.high_bait_weight,
        };

        let max = config.bait.high_bait_weight.max(1.0);
        (val / max).clamp(0.0, 1.0)
    }

    // Helper to score the bias for pricing
    fn score(&self) -> f32 {
        match self {
            BaitBias::Low => 1.0,
            BaitBias::Medium => 2.5,
            BaitBias::High => 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents what a given bait is good at attracting.
pub enum BaitAttraction {
    Heavy { bias: BaitBias },
    Light { bias: BaitBias },
    Large { bias: BaitBias },
    Small { bias: BaitBias },
    SpecificFish { name: String, bias: BaitBias },
    Rarity(FishRarity, BaitBias),
    Category(FishCategory, BaitBias),
}

impl BaitAttraction {
    // Calculate a "value score" for this specific attraction
    fn get_value_score(&self) -> f32 {
        match self {
            // Stats are basic
            BaitAttraction::Heavy { bias } |
            BaitAttraction::Light { bias } |
            BaitAttraction::Large { bias } |
            BaitAttraction::Small { bias } => 10.0 * bias.score(),

            // Categories are specialized
            BaitAttraction::Category(_, bias) => 25.0 * bias.score(),

            // Specific fish are very specialized
            BaitAttraction::SpecificFish { .. } => 35.0 * 2.0, // Usually high impact

            // Rarities are the most valuable
            BaitAttraction::Rarity(rarity, bias) => {
                let rarity_mult = match rarity {
                    FishRarity::Common => 1.0,
                    FishRarity::Uncommon => 1.5,
                    FishRarity::Rare => 3.0,
                    FishRarity::Elusive => 5.0,
                    FishRarity::Legendary => 10.0,
                    FishRarity::Mythical => 25.0,
                };
                20.0 * rarity_mult * bias.score()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bait {
    pub name: String,
    pub description: String,
    pub price: f32,
    pub use_chance: f32,
    pub attraction: Vec<BaitAttraction>,
}

impl Bait {
    pub fn get_specific_fish_modifier(&self) -> Option<(&String, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::SpecificFish { name, bias } => Some((name, bias.get_multiplier())),
            _ => None,
        })
    }

    pub fn get_rarity_modifier(&self) -> Option<(FishRarity, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::Rarity(r, bias) => Some((r.clone(), bias.get_multiplier())),
            _ => None,
        })
    }

    pub fn get_category_modifier(&self) -> Option<(FishCategory, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::Category(c, bias) => Some((c.clone(), bias.get_multiplier())),
            _ => None,
        })
    }

    pub fn get_weight_bias(&self) -> f32 {
        let mut total_bias = 0.0;
        for attr in &self.attraction {
            match attr {
                BaitAttraction::Heavy { bias } => total_bias += bias.get_normalized_strength(),
                BaitAttraction::Light { bias } => total_bias -= bias.get_normalized_strength(),
                _ => {}
            }
        }
        total_bias.clamp(-1.0, 1.0)
    }

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
                let count = rng.random_range(1..=2);
                for _ in 0..count {
                    attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Low));
                }
            },
            BaitPotency::Medium => {
                attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Medium));
                if rng.random_bool(0.5) {
                    attractions.push(Self::generate_random_attraction(&mut rng, BaitBias::Low));
                }
            },
            BaitPotency::High => {
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

        // --- NEW PRICING LOGIC ---
        // 1. Calculate raw score based on stats
        let mut score: f32 = attractions.iter().map(|a| a.get_value_score()).sum();

        // 2. Add random variance (+/- 15%) so players can't perfectly math it out
        let variance = rng.random_range(0.85..1.15);
        score *= variance;

        // 3. Ensure minimum prices based on Potency Tier
        // (Prevents a "High" tier bait that rolled unlucky stats from being $5)
        let min_price = match potency {
            BaitPotency::Low => 5.0,
            BaitPotency::Medium => 50.0,
            BaitPotency::High => 250.0,
        };

        let calculated_price = score.max(min_price);

        Bait {
            name,
            description,
            price: (calculated_price * 100.0).round() / 100.0,
            use_chance: 1.0,
            attraction: attractions,
        }
    }

    fn generate_random_attraction(rng: &mut impl Rng, bias: BaitBias) -> BaitAttraction {
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
                let categories = [
                    FishCategory::BaitFish, FishCategory::Schooling, FishCategory::Predatory,
                    FishCategory::BottomFeeder, FishCategory::Ornamental, FishCategory::Forager,
                    FishCategory::Apex, FishCategory::Abyssal, FishCategory::Mythological
                ];
                let cat = categories.choose(rng).unwrap().clone();
                BaitAttraction::Category(cat, bias)
            },
            _ => {
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
        let mut prefix = "";

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
            format!("Standard {}", base)
        } else {
            format!("{} {}", prefix, base)
        }
    }

    fn generate_description(attractions: &[BaitAttraction]) -> String {
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