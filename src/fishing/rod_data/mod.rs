use serde::{Deserialize, Serialize};

use crate::{
    data_management::config::Config,
    fishing::{
        Attribute,
        rod_data::{bait::Bait, lines::Line, reels::Reel, rods::RodBase, sinkers::Sinker},
    },
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
    /// fishing speed multiplier (higher = faster, 2.0 = 2x)
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
        self.line.strength as f32 * self.rod.strength_bonus - self.sinker.weight
    }

    pub fn total_speed_multiplier(&self) -> f32 {
        self.reel.speed_multiplier * self.rod.efficiency_multiplier
    }

    pub fn catch_chance_multiplier(&self) -> f32 {
        self.rod.sensitivity
    }

    /// Generate a catch time in seconds, bound by the config
    pub fn generate_catch_time(&self) -> f32 {
        let config = Config::load();
        let base_time = config.fishing.base_cast_wait;

        let multiplier = self.total_speed_multiplier();
        let standard_time = base_time / multiplier;

        // random variation - Replaced by fish weight calculations in the command
        // let mut rng = rand::rng();
        // let allowed_variance = config.fishing.max_cast_time_variation;
        // let variance_seconds: f32 = rng.random_range(-allowed_variance..=allowed_variance);

        // let final_time = standard_time + variance_seconds;

        // clamp
        // final_time.max(config.fishing.min_cast_wait)
        standard_time.max(config.fishing.min_cast_wait)
    }

    pub fn get_catch_chance_display(&self) -> String {
        let multiplier = self.catch_chance_multiplier();
        let config = Config::load();
        let base = config.fishing.base_catch_chance;

        let final_chance = base * multiplier;

        // scale to percentage
        let percentage = (final_chance * 100.0).round() as u32;
        // format!(
        //     "{}% (Base: {}% * Multiplier: {:.2})",
        //     percentage,
        //     (base * 100.0).round() as u32,
        //     multiplier
        // )
        format!("{}%", percentage)
    }

    pub fn get_depth_range_display(&self) -> String {
        let min = self.sinker.depth_range.min;
        let max = self.sinker.depth_range.max;
        let average = self.sinker.depth_range.average;
        format!("{}m - {}m (Average: {}m)", min, max, average)
    }

    pub fn get_total_strength_display(&self) -> String {
        let total_strength = self.total_strength();
        // format!(
        //     "{}lbs (Line Strength: {}, Rod Bonus: {:.2}, Sinker Weight: {:.2})",
        //     total_strength, self.line.strength, self.rod.strength_bonus, self.sinker.weight
        // )
        format!("{}lbs", total_strength)
    }

    pub fn get_speed_multiplier_display(&self) -> String {
        let multiplier = self.total_speed_multiplier();
        let config = Config::load();
        let base_speed = config.fishing.base_cast_wait;
        let final_speed = base_speed / multiplier;
        // format!(
        //     "{}s (Base: {}s / (Multiplier: {:.2} * Rod Efficiency: {:.2}))",
        //     final_speed.round() as u32,
        //     base_speed,
        //     multiplier,
        //     self.rod.efficiency_multiplier
        // )
        format!("{}s", final_speed.round() as u32, )
    }
}

impl Default for RodLoadout {
    fn default() -> Self {
        Self {
            rod: RodBase {
                name: "Willow Branch".to_string(),
                description: "A flexible stick found on the ground. Better than using your bare hands, but not by much.".to_string(),
                price: 0.0,
                sensitivity: 0.8,
                strength_bonus: 0.8,
                efficiency_multiplier: 0.8,
            },
            line: Line {
                name: "Old Thread".to_string(),
                description: "Cotton thread borrowed from a sewing kit. Snaps if a fish looks at it wrong.".to_string(),
                price: 0.0,
                strength: 5,
            },
            reel: Reel {
                name: "Rusty Can".to_string(),
                description: "Line wrapped around a rusty tin can. It takes ages to pull anything in.".to_string(),
                price: 0.0,
                speed_multiplier: 0.7,
            },
            sinker: Sinker {
                name: "River Stone".to_string(),
                description: "A smooth rock tied on with a clumsy knot. Keeps bait just under the surface.".to_string(),
                price: 0.0,
                weight: 0.1,
                depth_range: Attribute {
                    min: 0.0,
                    max: 15.0,
                    average: 5.0,
                },
            },
            bait: None,
            has_depth_finder: false,
        }
    }
}
