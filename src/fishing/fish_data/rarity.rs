use rand_distr::{Distribution, weighted::WeightedIndex};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

use crate::fishing::rod_data::bait::Bait;

// TODO: When fishing, if there are no fish of the generated rarity or lower, then the player should
//   not catch anything. (i.e. loch ness monster is mythical, but also at the deepest depth where no other fish are.
//   so if the player does not pull a mythical fish, they will not catch anything.
/// Represents the rarity of a fish.
/// Yes, this is from V1. why not?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FishRarity {
    Common,    // 40%
    Uncommon,  // 30%
    Rare,      // 20%
    Elusive,   // 8.9%
    Legendary, // 1%
    Mythical,  // 0.1%
}

impl FishRarity {
    /// Get the weight of the rarity.
    pub fn get_weight(&self) -> u16 {
        match self {
            FishRarity::Common => 400,
            FishRarity::Uncommon => 300,
            FishRarity::Rare => 200,
            FishRarity::Elusive => 89,
            FishRarity::Legendary => 10,
            FishRarity::Mythical => 1,
        }
    }

    /// Iterates all rarities
    pub fn iter() -> impl Iterator<Item = FishRarity> {
        [
            FishRarity::Common,
            FishRarity::Uncommon,
            FishRarity::Rare,
            FishRarity::Elusive,
            FishRarity::Legendary,
            FishRarity::Mythical,
        ]
        .into_iter()
    }

    /// Get a random rarity weighted properly based on bait and rarity weights.
    pub fn weighted_random(bait: Option<&Bait>) -> Self {
        let mut rng = rand::rng();

        let mut items = Vec::new();
        let mut weights = Vec::new();

        // Resolve the modifier once
        let rarity_modifier = bait.and_then(|b| b.get_rarity_modifier());

        for rarity in Self::iter() {
            let mut weight = rarity.get_weight() as f32;

            // Apply Bait Bias
            if let Some((target, multiplier)) = &rarity_modifier {
                if *target == rarity {
                    weight *= *multiplier; // Uses Config value (e.g., 5.0)
                }
            }

            items.push(rarity);
            weights.push(weight);
        }

        let dist = WeightedIndex::new(&weights).unwrap();
        items[dist.sample(&mut rng)].clone()
    }

    /// Get all rarities that are less than or equal to the given rarity.
    pub fn get_possible(&self) -> Vec<Self> {
        match self {
            FishRarity::Common => vec![Self::Common],
            FishRarity::Uncommon => vec![Self::Common, Self::Uncommon],
            FishRarity::Rare => vec![Self::Common, Self::Uncommon, Self::Rare],
            FishRarity::Elusive => vec![Self::Common, Self::Uncommon, Self::Rare, Self::Elusive],
            FishRarity::Legendary => vec![
                Self::Common,
                Self::Uncommon,
                Self::Rare,
                Self::Elusive,
                Self::Legendary,
            ],
            FishRarity::Mythical => vec![
                Self::Common,
                Self::Uncommon,
                Self::Rare,
                Self::Elusive,
                Self::Legendary,
                Self::Mythical,
            ],
        }
    }
}

impl FromStr for FishRarity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Common" => Ok(Self::Common),
            "Uncommon" => Ok(Self::Uncommon),
            "Rare" => Ok(Self::Rare),
            "Elusive" => Ok(Self::Elusive),
            "Legendary" => Ok(Self::Legendary),
            "Mythical" => Ok(Self::Mythical),
            _ => Err("Unknown Rarity".into()),
        }
    }
}

impl Display for FishRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FishRarity::Common => write!(f, "Common"),
            FishRarity::Uncommon => write!(f, "Uncommon"),
            FishRarity::Rare => write!(f, "Rare"),
            FishRarity::Elusive => write!(f, "Elusive"),
            FishRarity::Legendary => write!(f, "Legendary"),
            FishRarity::Mythical => write!(f, "Mythical"),
        }
    }
}
