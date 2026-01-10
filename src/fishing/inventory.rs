use serde::{Deserialize, Serialize};
use crate::fishing::bait_bucket::BaitBucket;
use crate::fishing::rod_data::bait::Bait;
use crate::fishing::rod_data::lines::Line;
use crate::fishing::rod_data::reels::Reel;
use crate::fishing::rod_data::RodLoadout;
use crate::fishing::rod_data::rods::RodBase;
use crate::fishing::rod_data::sinkers::Sinker;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Inventory {
    pub rods: Vec<RodBase>,
    pub selected_rod: usize,
    pub lines: Vec<Line>,
    pub selected_line: usize,
    pub reels: Vec<Reel>,
    pub selected_reel: usize,
    pub sinkers: Vec<Sinker>,
    pub selected_sinker: usize,

    pub bait_bucket: BaitBucket,
    pub selected_bait: Option<usize>,

    pub underwater_cam: bool,
    pub depth_finder: bool,
}

impl Inventory {

    fn get_selected_bait(&self) -> Option<Bait> {
        if let Some(index) = self.selected_bait {
            self.bait_bucket.get(index).cloned()
        } else {
            None
        }
    }

    pub fn get_loadout(&self) -> RodLoadout {
        RodLoadout {
            rod: self.rods[self.selected_rod].clone(),
            line: self.lines[self.selected_line].clone(),
            reel: self.reels[self.selected_reel].clone(),
            sinker: self.sinkers[self.selected_sinker].clone(),
            bait: self.get_selected_bait(),

            has_underwater_camera: self.underwater_cam,
            has_depth_finder: self.depth_finder,
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        let loadout = RodLoadout::default();
        Self {
            rods: vec![loadout.rod],
            selected_rod: 0,
            lines: vec![loadout.line],
            selected_line: 0,
            reels: vec![loadout.reel],
            selected_reel: 0,
            sinkers: vec![loadout.sinker],
            selected_sinker: 0,

            bait_bucket: BaitBucket::new(),
            selected_bait: None,

            underwater_cam: false,
            depth_finder: false,
        }
    }
}