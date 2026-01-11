use core::fmt;
use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};

/// Avoids using floats and their weird bugs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonetaryAmount {
    pub amount_x100: u32,
}

impl MonetaryAmount {
    pub fn new(amount: f32) -> Self {
        Self {
            amount_x100: (amount * 100.0).round() as u32,
        }
    }

    pub fn get(&self) -> f32 {
        self.amount_x100 as f32 / 100.0
    }
}

// 1. Allows you to do: println!("You have {}", wallet); -> "You have $10.50"
impl fmt::Display for MonetaryAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dollars = self.amount_x100 / 100;
        let cents = self.amount_x100 % 100;
        // Convert to string to access characters
        let dollars_str = dollars.to_string();
        let len = dollars_str.len();
        let mut dollars_display = String::with_capacity(len + (len / 3));

        // Iterate through characters and insert commas
        for (i, c) in dollars_str.chars().enumerate() {
            // Insert a comma if we are not at the start
            // and the remaining digits are a multiple of 3
            if i > 0 && (len - i) % 3 == 0 {
                dollars_display.push(',');
            }
            dollars_display.push(c);
        }
        // {:02} ensures 5 cents prints as "05" not "5"
        write!(f, "${}.{:02}", dollars_display, cents)
    }
}

// 2. Allows you to do: let total = wallet + fish_value;
impl Add for MonetaryAmount {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            amount_x100: self.amount_x100 + other.amount_x100,
        }
    }
}

// 3. Allows you to do: wallet += fish_value;
impl AddAssign for MonetaryAmount {
    fn add_assign(&mut self, other: Self) {
        self.amount_x100 += other.amount_x100;
    }
}
