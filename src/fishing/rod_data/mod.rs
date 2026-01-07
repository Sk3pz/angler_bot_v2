use serde::{Deserialize, Serialize};

use crate::fishing::rod_data::{
    bait::Bait, lines::Line, reels::Reel, rods::RodBase, sinkers::Sinker,
};

pub mod bait;
pub mod lines;
pub mod reels;
pub mod rods;
pub mod sinkers;

// 5 modules:
// - Rod: The base
// - Line: Better line = more weight it can support and less likely to snap.
// - Reel: Better reel = faster fishing speed
// - Sinker: No "Better" sinker, different sinker types for different depths
// - Bait: each bait will have something it's better at: Heavier fish, Bigger fish, rarer fish, etc.
//     Bait is consumable
//     Lure baits can be reused but will be lost upon a line snap

#[derive(Debug, Clone, Serialize, Deserialize)]
/// These multipliers are applied to the cast, not the fish itself on generation. Only bait and sinker directly affect fish generation
pub struct RodLoadout {
    /// can add strength and speed (adding to both line and reel)
    pub rod: RodBase,
    /// strength?
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
