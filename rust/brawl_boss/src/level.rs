//! The level L (format spec 5.3). Code-side, not in any file: `L = TABLE[strength - 1][difficulty]`; out of range
//! gives 0. Values are exact in 1/60000 units.

use crate::fx::Fx;

/// Default-mode table in tenths (rows strength 1-5, columns difficulty 0-14).
const TENTHS: [[i16; 15]; 5] = [
    [0, 3, 5, 17, 21, 25, 32, 36, 40, 55, 60, 65, 80, 85, 90],
    [6, 9, 11, 25, 29, 33, 38, 42, 48, 61, 66, 71, 85, 90, 95],
    [12, 15, 17, 33, 37, 41, 47, 51, 55, 67, 72, 77, 90, 95, 100],
    [18, 21, 23, 39, 43, 47, 55, 59, 63, 73, 78, 83, 95, 100, 105],
    [24, 27, 29, 45, 50, 55, 62, 66, 70, 80, 85, 90, 100, 105, 110],
];

pub fn level(strength: i32, difficulty: i32) -> Fx {
    if !(1..=5).contains(&strength) || !(0..=14).contains(&difficulty) {
        return Fx::ZERO;
    }
    Fx::ratio(TENTHS[(strength - 1) as usize][difficulty as usize] as i64, 10)
}

/// Difficulty = clamp(3 tier + 2 - min(retries, 5), 0, 14). A fresh Normal game is 5.
pub fn difficulty(tier: i32, retries: i32) -> i32 {
    (3 * tier + 2 - retries.clamp(0, 5)).clamp(0, 14)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn table() {
        assert_eq!(level(1, 5), Fx::ratio(25, 10));
        assert_eq!(level(2, 5), Fx::ratio(33, 10));
        assert_eq!(level(3, 5), Fx::ratio(41, 10));
        assert_eq!(level(5, 14), Fx::from_int(11));
        assert_eq!(level(0, 5), Fx::ZERO);
        assert_eq!(level(1, 15), Fx::ZERO);
        assert_eq!(difficulty(1, 0), 5);
    }
}
