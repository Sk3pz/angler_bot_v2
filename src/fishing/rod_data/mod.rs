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
}

impl Default for RodLoadout {
    fn default() -> Self {
        Self {
            rod: RodBase {
                name: "Basic Rod".to_string(),
                strength_bonus: 1.0,
                efficiency_multiplier: 1.0,
            },
            line: Line {
                name: "Basic Line".to_string(),
                strength: 10,
            },
            reel: Reel {
                name: "Basic Reel".to_string(),
                speed_multiplier: 1.0,
            },
            sinker: Sinker {
                name: "Basic Sinker".to_string(),
                weight: 0.0,
                depth_range: Attribute {
                    min: 0.0,
                    max: 20.0,
                    average: 10.0,
                },
            },
            bait: None,
            has_depth_finder: false,
        }
    }
}
