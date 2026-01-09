use rand::Rng;

pub const GAME_TIPS: &[&str] = &[
    "🎣 Tip: Use the `/info` command to view your stats and loadout",
    "🎣 Tip: Cast your rod into the Pond with `/cast`",
    "🎣 Tip: Select and view your bait with /bait",
    "🎣 Tip: You can customize your rod loadout with different rods, reels, lines, and sinkers to improve your chances of catching fish.",
    "🎣 Tip: Open the shop with `/shop` to see the available upgrades and items you can buy!",
    "🎣 Tip: Experiment with different combinations of rods, reels, lines, and sinkers to find the best setup for you!",
    "🎣 Tip: Don't stare into the pond, it may stare back...",
    "🎣 Tip: The old angler you are with sure does have some interesting things to say. Maybe you should listen!",
    "🎣 Tip: Jerrie cheats!",
];

pub fn random_tip() -> String {
    GAME_TIPS[rand::rng().random_range(0..GAME_TIPS.len())].to_string()
}