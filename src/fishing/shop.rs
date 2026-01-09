use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::fishing::rod_data::{
    bait::{Bait, BaitPotency},
    lines::Line,
    reels::Reel,
    rods::RodBase,
    sinkers::Sinker,
};
use crate::nay;

const SHOP_STATE_PATH: &str = "./data/gamedata/shop.ron";
const RODS_PATH: &str = "./data/gamedata/rods.ron";
const LINES_PATH: &str = "./data/gamedata/lines.ron";
const REELS_PATH: &str = "./data/gamedata/reels.ron";
const SINKERS_PATH: &str = "./data/gamedata/sinkers.ron";

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Represents the dynamic state of the shop that changes daily.
pub struct ShopState {
    pub last_refresh: NaiveDate,
    pub daily_baits: Vec<Bait>,
}

impl Default for ShopState {
    fn default() -> Self {
        Self {
            last_refresh: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            daily_baits: Vec::new(),
        }
    }
}

/// The main Shop structure holding both dynamic and static inventory.
pub struct Shop {
    pub state: ShopState,
    pub rods: Vec<RodBase>,
    pub lines: Vec<Line>,
    pub reels: Vec<Reel>,
    pub sinkers: Vec<Sinker>,
}

impl Shop {
    /// Loads the shop. If the date has changed, it refreshes the stock.
    pub fn load() -> Self {
        let mut state = Self::load_state();
        let today = Local::now().date_naive();

        // Check if we need to restock (New Day)
        if state.last_refresh < today {
            state = Self::refresh_stock(today);
        }

        // Load static catalog
        let rods = Self::load_static_data(RODS_PATH).unwrap_or_default();
        let lines = Self::load_static_data(LINES_PATH).unwrap_or_default();
        let reels = Self::load_static_data(REELS_PATH).unwrap_or_default();
        let sinkers = Self::load_static_data(SINKERS_PATH).unwrap_or_default();

        Self {
            state,
            rods,
            lines,
            reels,
            sinkers,
        }
    }

    /// Generates new baits, updates the date, and saves to file.
    fn refresh_stock(date: NaiveDate) -> ShopState {
        let mut baits = Vec::new();

        // Generate Daily Stock:
        // 4 Low Potency (Common/Cheap)
        baits.push(Bait::generate(BaitPotency::Low, false));
        baits.push(Bait::generate(BaitPotency::Low, false));
        baits.push(Bait::generate(BaitPotency::Low, false));
        baits.push(Bait::generate(BaitPotency::Low, false));

        // 2 Medium Potency (Decent)
        baits.push(Bait::generate(BaitPotency::Medium, false));
        baits.push(Bait::generate(BaitPotency::Medium, false));
        baits.push(Bait::generate(BaitPotency::Medium, false));

        // 1 High Potency (Rare/Expensive)
        baits.push(Bait::generate(BaitPotency::High, false));

        // forced lure
        baits.push(Bait::generate(BaitPotency::High, true));

        let state = ShopState {
            last_refresh: date,
            daily_baits: baits,
        };

        Self::save_state(&state);
        state
    }

    /// internal helper to load the state file
    fn load_state() -> ShopState {
        let path = Path::new(SHOP_STATE_PATH);
        if !path.exists() {
            return ShopState::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => ron::from_str(&content).unwrap_or_else(|e| {
                nay!("Failed to parse shop state: {}", e);
                ShopState::default()
            }),
            Err(e) => {
                nay!("Failed to read shop state file: {}", e);
                ShopState::default()
            }
        }
    }

    /// internal helper to save the state file
    fn save_state(state: &ShopState) {
        let path = Path::new(SHOP_STATE_PATH);
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match ron::to_string(state) {
            Ok(content) => {
                if let Err(e) = fs::write(path, content) {
                    nay!("Failed to write shop state: {}", e);
                }
            }
            Err(e) => nay!("Failed to serialize shop state: {}", e),
        }
    }

    /// generic helper to load static RON lists (rods, lines, etc.)
    fn load_static_data<T: for<'a> Deserialize<'a>>(path_str: &str) -> Option<Vec<T>> {
        let path = Path::new(path_str);
        if !path.exists() {
            nay!("Static data file missing: {}", path_str);
            return None;
        }

        match fs::read_to_string(path) {
            Ok(content) => match ron::from_str(&content) {
                Ok(data) => Some(data),
                Err(e) => {
                    nay!("Failed to parse {}: {}", path_str, e);
                    None
                }
            },
            Err(e) => {
                nay!("Failed to read {}: {}", path_str, e);
                None
            }
        }
    }
}