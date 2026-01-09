use rand::prelude::IndexedRandom;
use rand::Rng;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AttractionQuality {
    Good,
    Bad,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents what a given bait is good at attracting.
pub enum BaitAttraction {
    Heavy { bias: BaitBias, quality: AttractionQuality },
    Light { bias: BaitBias, quality: AttractionQuality },
    Large { bias: BaitBias, quality: AttractionQuality },
    Small { bias: BaitBias, quality: AttractionQuality },
    SpecificFish { name: String, bias: BaitBias, quality: AttractionQuality },
    Rarity(FishRarity, BaitBias, AttractionQuality),
    Category(FishCategory, BaitBias, AttractionQuality),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bait {
    pub name: String,
    pub description: String,
    pub price: f32,
    /// Chance the bait will be used up after a catch.
    /// 0.0 means it will never be used up, 1.0 means it will always be used up.
    pub reusable: bool,
    pub attraction: Vec<BaitAttraction>,
}

impl Bait {
    pub fn get_specific_fish_modifier(&self) -> Option<(&String, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::SpecificFish { name, bias, .. } => Some((name, bias.get_multiplier())),
            _ => None,
        })
    }

    pub fn get_rarity_modifier(&self) -> Option<(FishRarity, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::Rarity(r, bias, _) => Some((r.clone(), bias.get_multiplier())),
            _ => None,
        })
    }

    pub fn get_category_modifier(&self) -> Option<(FishCategory, f32)> {
        self.attraction.iter().find_map(|attr| match attr {
            BaitAttraction::Category(c, bias, _) => Some((c.clone(), bias.get_multiplier())),
            _ => None,
        })
    }

    pub fn get_weight_bias(&self) -> f32 {
        let mut total_bias = 0.0;
        for attr in &self.attraction {
            match attr {
                BaitAttraction::Heavy { bias, .. } => total_bias += bias.get_normalized_strength(),
                BaitAttraction::Light { bias, .. } => total_bias -= bias.get_normalized_strength(),
                _ => {}
            }
        }
        total_bias.clamp(-1.0, 1.0)
    }

    pub fn get_size_bias(&self) -> f32 {
        let mut total_bias = 0.0;
        for attr in &self.attraction {
            match attr {
                BaitAttraction::Large { bias, .. } => total_bias += bias.get_normalized_strength(),
                BaitAttraction::Small { bias, .. } => total_bias -= bias.get_normalized_strength(),
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
    /// Generates a random bait or lure based on the provided potency.
    pub fn generate(potency: BaitPotency, force_lure: bool) -> Self {
        let mut rng = rand::rng();

        // 10% chance to be a Lure (Infinite use, higher cost)
        // Lures are more common in higher potencies
        let is_lure = if force_lure {
            true
        } else {
            match potency {
                BaitPotency::Low => rng.random_bool(0.05),
                BaitPotency::Medium => rng.random_bool(0.15),
                BaitPotency::High => rng.random_bool(0.25),
            }
        };

        let mut attractions = Vec::new();

        // Helper closure to add attractions while avoiding logic conflicts (e.g., Heavy + Light)
        let mut add_attraction = |target_bias: BaitBias| {
            let mut local_rng = rand::rng();
            let mut attempts = 0;
            // Try up to 5 times to generate a non-conflicting attraction
            while attempts < 5 {
                // Determine Quality based on Potency
                // Low Potency -> High chance of Bad traits
                // High Potency -> High chance of Good traits
                let quality_roll = local_rng.random_range(0..100);
                let good_threshold = match potency {
                    BaitPotency::Low => 30, // 30% Chance for Good
                    BaitPotency::Medium => 60, // 60% Chance for Good
                    BaitPotency::High => 90, // 90% Chance for Good
                };

                let quality = if quality_roll < good_threshold {
                    AttractionQuality::Good
                } else {
                    AttractionQuality::Bad
                };

                let candidate = Self::generate_random_attraction(&mut local_rng, target_bias.clone(), quality);

                let is_conflicting = attractions.iter().any(|existing| {
                    match (existing, &candidate) {
                        // Mutually exclusive physical traits
                        (BaitAttraction::Heavy{..}, BaitAttraction::Light{..}) => true,
                        (BaitAttraction::Light{..}, BaitAttraction::Heavy{..}) => true,
                        (BaitAttraction::Large{..}, BaitAttraction::Small{..}) => true,
                        (BaitAttraction::Small{..}, BaitAttraction::Large{..}) => true,

                        // Prevent duplicate Categories
                        (BaitAttraction::Category(c1, _, _), BaitAttraction::Category(c2, _, _)) if c1 == c2 => true,

                        // Prevent duplicate Rarities
                        (BaitAttraction::Rarity(r1, _, _), BaitAttraction::Rarity(r2, _, _)) if r1 == r2 => true,

                        _ => false
                    }
                });

                if !is_conflicting {
                    attractions.push(candidate);
                    break;
                }
                attempts += 1;
            }
        };

        // Define generation rules
        match potency {
            BaitPotency::Low => {
                let count = rng.random_range(1..=2);
                for _ in 0..count {
                    add_attraction(BaitBias::Low);
                }
            },
            BaitPotency::Medium => {
                add_attraction(BaitBias::Medium);
                let count = rng.random_range(1..=2);
                for _ in 0..count {
                    add_attraction(BaitBias::Low);
                }
            },
            BaitPotency::High => {
                let high_count = rng.random_range(1..=2);
                for _ in 0..high_count {
                    add_attraction(BaitBias::High);
                }
                add_attraction(BaitBias::Medium);

                let low_count = rng.random_range(1..=2);
                for _ in 0..low_count {
                    add_attraction(BaitBias::Low);
                }
            },
        }

        let name: String;
        let price_multiplier: f32;

        if is_lure {
            // Lure Logic
            let base = "Lure";
            name = Self::generate_name(base, &attractions);
            price_multiplier = 10.0; // Much more expensive
        } else {
            // Organic Bait Logic
            let data = BaitData::load();
            let base = data.base_names.choose(&mut rng).cloned().unwrap_or_else(|| "Worm".to_string());
            name = Self::generate_name(&base, &attractions);
            price_multiplier = 1.0;
        }

        let description = Self::generate_description(&attractions, is_lure);

        // Price calculation
        let base_price = match potency {
            BaitPotency::Low => rng.random_range(5.0..20.0),
            BaitPotency::Medium => rng.random_range(50.0..150.0),
            BaitPotency::High => rng.random_range(300.0..800.0),
        };

        let final_price = base_price * price_multiplier;

        Bait {
            name,
            description,
            price: (final_price * 100.0).round() / 100.0,
            reusable: is_lure,
            attraction: attractions,
        }
    }

    fn generate_random_attraction(rng: &mut impl Rng, bias: BaitBias, quality: AttractionQuality) -> BaitAttraction {
        let roll = rng.random_range(0..100);

        match quality {
            AttractionQuality::Good => {
                match roll {
                    0..=29 => BaitAttraction::Large { bias, quality },
                    30..=59 => BaitAttraction::Heavy { bias, quality },
                    60..=84 => {
                        let categories = [
                            FishCategory::Predatory, FishCategory::Apex,
                            FishCategory::Abyssal, FishCategory::Mythological,
                            FishCategory::Ornamental
                        ];
                        let cat = categories.choose(rng).unwrap().clone();
                        BaitAttraction::Category(cat, bias, quality)
                    },
                    _ => {
                        let rarities = [
                            FishRarity::Rare, FishRarity::Elusive, FishRarity::Legendary
                        ];
                        let rar = rarities.choose(rng).unwrap().clone();
                        BaitAttraction::Rarity(rar, bias, quality)
                    }
                }
            },
            AttractionQuality::Bad => {
                match roll {
                    0..=29 => BaitAttraction::Small { bias, quality },
                    30..=59 => BaitAttraction::Light { bias, quality },
                    60..=84 => {
                        let categories = [
                            FishCategory::BaitFish, FishCategory::Schooling,
                            FishCategory::BottomFeeder, FishCategory::Forager
                        ];
                        let cat = categories.choose(rng).unwrap().clone();
                        BaitAttraction::Category(cat, bias, quality)
                    },
                    _ => {
                        let rarities = [
                            FishRarity::Common, FishRarity::Uncommon
                        ];
                        let rar = rarities.choose(rng).unwrap().clone();
                        BaitAttraction::Rarity(rar, bias, quality)
                    }
                }
            }
        }
    }

    fn generate_name(base: &str, attractions: &[BaitAttraction]) -> String {
        let mut prefix = "";

        if let Some(cat_attr) = attractions.iter().find(|a| matches!(a, BaitAttraction::Category(_, _, _))) {
            if let BaitAttraction::Category(cat, _, _) = cat_attr {
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
        } else if let Some(rar_attr) = attractions.iter().find(|a| matches!(a, BaitAttraction::Rarity(_, _, _))) {
            if let BaitAttraction::Rarity(rar, _, _) = rar_attr {
                prefix = match rar {
                    FishRarity::Common => "Common",
                    FishRarity::Uncommon => "Uncommon",
                    FishRarity::Rare => "Rare",
                    FishRarity::Elusive => "Elusive",
                    FishRarity::Legendary => "Legendary",
                    FishRarity::Mythical => "Mythical",
                };
            }
        } else if let Some(stat_attr) = attractions.iter().find(|a| matches!(a, BaitAttraction::Large{..} | BaitAttraction::Heavy{..} | BaitAttraction::Small{..} | BaitAttraction::Light{..})) {
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

    fn generate_description(attractions: &[BaitAttraction], is_lure: bool) -> String {
        let attr_desc: Vec<String> = attractions.iter().map(|a| match a {
            BaitAttraction::Heavy { .. } => "heavier fish".to_string(),
            BaitAttraction::Light { .. } => "lighter fish".to_string(),
            BaitAttraction::Large { .. } => "larger fish".to_string(),
            BaitAttraction::Small { .. } => "smaller fish".to_string(),
            BaitAttraction::Category(c, _, _) => format!("{:?} fish", c),
            BaitAttraction::Rarity(r, _, _) => format!("{} fish", r),
            BaitAttraction::SpecificFish { name, .. } => format!("{}", name),
        }).collect();

        let type_str = if is_lure { "A reusable lure" } else { "A bait" };
        format!("{} that attracts {}.", type_str, attr_desc.join(", "))
    }
}