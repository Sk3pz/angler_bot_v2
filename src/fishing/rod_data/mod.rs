use serde::{Deserialize, Serialize};

use crate::fishing::{
    Attribute,
    rod_data::{bait::Bait, lines::Line, reels::Reel, rods::RodBase, sinkers::Sinker},
};

pub mod bait;
pub mod lines;
pub mod reels;
pub mod rods;
pub mod sinkers;

// 6 modules:
// - Rod: The base
// - Line: Determines weight limit
// - Reel: Determines fishing speed
// - Sinker: Determines the depth range you can fish at
// - Bait: "Consumable Boosters" - Multipliers for different aspects of fish
//     Lure baits can be reused but will be lost upon a line snap
// - Depth Finder: Tells the user the exact depth they cast to

#[derive(Debug, Clone, Serialize, Deserialize)]
/// These multipliers are applied to the cast, not the fish itself on generation. Only bait and sinker directly affect fish generation
pub struct RodLoadout {
    /// can add strength and speed (adding to both line and reel)
    pub rod: RodBase,
    /// strength (weight limit)
    pub line: Line,
    /// fishing speed multiplier
    pub reel: Reel,
    /// depth range
    pub sinker: Sinker,
    /// You can fish with no bait
    pub bait: Option<Bait>,
    /// purchasable item to tell you what depth you hit while casting.
    pub has_depth_finder: bool,
}

impl RodLoadout {
    pub fn total_strength(&self) -> f32 {
        self.line.strength as f32 * self.rod.strength_bonus
    }

    pub fn total_speed_multiplier(&self) -> f32 {
        self.reel.speed_multiplier * self.rod.efficiency_multiplier
    }

    pub fn catch_chance_multiplier(&self) -> f32 {
        self.rod.sensitivity
    }
}

impl Default for RodLoadout {
    fn default() -> Self {
        Self {
            rod: RodBase {
                name: "Random Branch".to_string(),
                // actually worse to fish with
                sensitivity: 0.8,
                strength_bonus: 0.8,
                efficiency_multiplier: 0.8,
            },
            line: Line {
                name: "Dirty String".to_string(),
                strength: 10, // with the Random Branch this will be 8, 7.8 with the Rock Sinker
            },
            reel: Reel {
                name: "Basic Reel".to_string(),
                speed_multiplier: 0.8,
            },
            sinker: Sinker {
                name: "Rock".to_string(),
                weight: 0.2,
                depth_range: Attribute {
                    min: 0.0,
                    max: 20.0,
                    average: 5.0,
                },
            },
            bait: None,
            has_depth_finder: false,
        }
    }
}
