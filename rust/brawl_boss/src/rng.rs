//! Randomness comes only from a caller-supplied source, so the game can feed its own deterministic generator (and
//! rollback can replay it). Every draw takes exactly one `next_u32`.
//!
//! Draw order within one frame (format spec 4.3 step 6), all from the same source:
//! 1. the rule engine's target-slot draw (only when the lowest-priority true rules span two or more slots);
//! 2. any weighted-switch draw inside the chosen rule's action (one per switch executed, zero when every weight is
//!    zero);
//! 3. any range draw (6507) in the next status's enter script (always one per 6507, even when the range is empty);
//! then, later in the frame, the draws of the status loop thread and the timelines, in thread order.

/// A deterministic 32-bit source.
pub trait RandomSource {
    fn next_u32(&mut self) -> u32;
}

/// Uniform in `[0, total)` raw units: `(u32 * total) >> 32`. One draw. `total <= 0` gives 0 (and still draws, so the
/// order stays fixed; callers that must not draw check first).
pub fn below(r: &mut dyn RandomSource, total: i64) -> i64 {
    let u = r.next_u32() as u128;
    if total <= 0 {
        return 0;
    }
    ((u * total as u128) >> 32) as i64
}

/// SplitMix32-style generator for tests and tools (not the game's generator).
#[derive(Clone, Debug)]
pub struct TestRng(pub u64);
impl RandomSource for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        (z ^ (z >> 31)) as u32
    }
}

/// A source that replays fixed values (cycling), for tests that pin a draw.
#[derive(Clone, Debug)]
pub struct FixedRng {
    pub values: alloc::vec::Vec<u32>,
    pub at: usize,
    /// How many draws were taken.
    pub draws: usize,
}
impl FixedRng {
    pub fn new(values: &[u32]) -> FixedRng {
        FixedRng { values: values.to_vec(), at: 0, draws: 0 }
    }
}
impl RandomSource for FixedRng {
    fn next_u32(&mut self) -> u32 {
        self.draws += 1;
        if self.values.is_empty() {
            return 0;
        }
        let v = self.values[self.at % self.values.len()];
        self.at += 1;
        v
    }
}
