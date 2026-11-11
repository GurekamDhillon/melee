// V3 Heat arithmetic (envoy-synergy-v3.md 3, 12.2): the one place the sim's integer rules live, so the split test
// below checks exactly what envoy_sim runs. No floats, no division on the gain path: n records that add up to one
// record's damage give bit-identical Heat, charge, tally, coins and pips.

use crate::codes::*;

/// The cap in ticks.
pub const CAP_TICKS: i64 = HEAT_CAP * HEAT_TICK;

/// Heat ticks -> Heat x1000 (the view's unit).
pub const fn milli(ticks: i64) -> i32 {
    (ticks / (HEAT_TICK / 1000)) as i32
}

/// The state (0 Cold, 1 Warm, 2 Hot, 3 Blazing) of a Heat in ticks.
pub fn state_of(ticks: i64) -> usize {
    let mut s = 0;
    for (i, m) in HEAT_MARKS.iter().enumerate() {
        if ticks >= m * HEAT_TICK {
            s = i + 1;
        }
    }
    s
}

/// A record's damage split at the 150% line on the foe's own percent before it (`below`, `above`), x1000.
pub fn split150(foe_m: i32, dmg_m: i32, no_line: bool) -> (i64, i64) {
    let d = dmg_m.max(0) as i64;
    if no_line {
        return (d, 0);
    }
    let room = (OVERKILL_M as i64 - foe_m as i64).max(0);
    let below = d.min(room);
    (below, d - below)
}

/// Heat gained by one record (ticks): `(2 * below + above) * coef * gain`.
pub fn heat_gain(dmg_m: i32, foe_m: i32, coef_m: i32, gain_m: i32, no_line: bool) -> i64 {
    let (b, a) = split150(foe_m, dmg_m, no_line);
    (2 * b + a) * coef_m.max(0) as i64 * gain_m.max(0) as i64
}

/// Charge one record gives a damage-charged drive (units: 1% = 1e9).
pub fn charge_gain(dmg_m: i32, coef_m: i32, rate_m: i32) -> i64 {
    dmg_m.max(0) as i64 * coef_m.max(0) as i64 * rate_m.max(0) as i64
}

/// Heat lost to damage taken (ticks): `paid * loss * 2000` (loss_m 1000 = 1 Heat per 1%).
pub fn loss(paid_m: i32, loss_m: i32) -> i64 {
    paid_m.max(0) as i64 * loss_m.max(0) as i64 * 2000
}

/// A flat +Heat (Heat x1000) in ticks.
pub fn heat_add(amount_m: i32) -> i64 {
    amount_m as i64 * (HEAT_TICK / 1000)
}

/// One gain step: `g * mul / 1000`, rounded down, capped.
pub fn gain_step(g: i32, mul_m: i32) -> i32 {
    ((g as i64 * mul_m as i64 / 1000) as i32).min(GAIN_CAP)
}

/// A fire's damage before any cap (x1000): number x level scale x state scale, rounded down once each. The sim caps
/// it on what lands (after the foe's toughness).
pub fn fire_damage_raw(number_m: i32, level: i32, state: usize) -> i32 {
    let lv = level_scale(level);
    (number_m as i64 * lv as i64 / 1000 * STATE_SCALE[state.min(3)] as i64 / 1000).min(i32::MAX as i64) as i32
}

/// A fire's damage (x1000): number x level scale x state scale, rounded down once each, clamped.
pub fn fire_damage(number_m: i32, level: i32, state: usize, launch: bool) -> i32 {
    let lv = level_scale(level);
    let n = number_m as i64 * lv as i64 / 1000 * STATE_SCALE[state.min(3)] as i64 / 1000;
    (n as i32).min(if launch { LAUNCH_CAP } else { FIRE_CAP })
}

/// A chain coefficient: the drive's times its parent's, rounded down once per link.
pub fn link_coef(drive_m: i32, parent_m: i32) -> i32 {
    drive_m * parent_m / 1000
}

/// The flare a cash-out of `ticks` at `pay_m` sends to every other foe (x1000 percent).
pub fn flare(ticks: i64, pay_m: i32) -> i32 {
    ((ticks.max(0) as i128 * pay_m as i128 / FLARE_DIV as i128) as i64).min(FLARE_MAX as i64) as i32
}

/// A cash-out's worth (ticks x pay) for the purse and the pips (both carry their remainders).
pub fn cash(ticks: i64, pay_m: i32) -> i64 {
    ticks.max(0) * pay_m as i64
}

/// The split test's model of one player's record path (12.10 A): Heat, a row of damage-charged drives, the tally on
/// one foe, the purse and the pips; damage taken through Insulate's allowance.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Ledger {
    pub heat: i64,
    pub charge: [i64; 5],
    pub tally: i32,
    pub foe_m: i32,
    pub purse: i64,
    pub pips: i64,
    pub free: i32,
}

impl Ledger {
    /// One record dealt to the foe: `rows` says which slots charge from it (a bit per slot).
    pub fn deal(&mut self, dmg_m: i32, coef_m: i32, gain_m: i32, rows: u32) {
        self.heat = (self.heat + heat_gain(dmg_m, self.foe_m, coef_m, gain_m, false)).min(CAP_TICKS);
        for k in 0..5 {
            if rows >> k & 1 != 0 {
                self.charge[k] += charge_gain(dmg_m, coef_m, 1000);
            }
        }
        self.tally += dmg_m;
        self.foe_m += dmg_m;
    }
    /// One record taken: Insulate's allowance first, then 1 Heat per 1%.
    pub fn take(&mut self, dmg_m: i32) {
        let free = self.free.min(dmg_m);
        self.free -= free;
        self.heat = (self.heat - loss(dmg_m - free, 1000)).max(0);
    }
    /// The KO: pay, then keep half.
    pub fn cash_out(&mut self, pay_m: i32) {
        let c = cash(self.heat, pay_m);
        self.purse += c;
        self.pips += c;
        self.heat /= 2;
        self.tally = 0;
        self.foe_m = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 12.10 A, the split test: one record of N against the same total as 2, 7 and 30 records (and uneven pieces).
    #[test]
    fn split_is_exact() {
        let rows_list: [u32; 4] = [0b1, 0b10101, 0b11111, 0b01010];
        for &n in &[2_000, 14_000, 30_000, 160_000] {
            for &rows in &rows_list {
                for &gain in &[1000, 1500, 2250, 4000] {
                    for &coef in &[1000, 500, 300, 120] {
                        for &start_foe in &[0, 100_000, 149_500] {
                            let mut one = Ledger { foe_m: start_foe, ..Default::default() };
                            one.deal(n, coef, gain, rows);
                            for pieces in [2, 7, 30] {
                                let mut many = Ledger { foe_m: start_foe, ..Default::default() };
                                let mut left = n;
                                for i in 0..pieces {
                                    // uneven pieces that add to n exactly
                                    let d = if i == pieces - 1 { left } else { (n / pieces) + (i % 3) - 1 };
                                    many.deal(d, coef, gain, rows);
                                    left -= d;
                                }
                                assert_eq!(left, 0);
                                // the cap can only cut the last piece, which is the same total either way
                                assert_eq!(one, many, "n {n} rows {rows:b} gain {gain} coef {coef} foe {start_foe} pieces {pieces}");
                                let (mut a, mut b) = (one, many);
                                a.cash_out(1500);
                                b.cash_out(1500);
                                assert_eq!(a, b);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn split_taken_with_and_without_insulate() {
        for &free in &[0, INSULATE_FREE] {
            for &n in &[2_000, 14_000, 30_000, 160_000] {
                let start = Ledger { heat: 200 * HEAT_TICK, free, ..Default::default() };
                let mut one = start;
                one.take(n);
                for pieces in [2, 7, 30] {
                    let mut many = start;
                    let mut left = n;
                    for i in 0..pieces {
                        let d = if i == pieces - 1 { left } else { n / pieces };
                        many.take(d);
                        left -= d;
                    }
                    assert_eq!(one, many, "taken {n} free {free} pieces {pieces}");
                }
            }
        }
    }

    #[test]
    fn units() {
        // 1% at x1 is 1 Heat; the overkill half; the marks; 15 Heat a coin at pay x1; a full meter's flare 15%
        assert_eq!(heat_gain(1000, 0, 1000, 1000, false), HEAT_TICK);
        assert_eq!(heat_gain(10_000, 145_000, 1000, 1000, false), 5 * HEAT_TICK + 5 * HEAT_TICK / 2);
        assert_eq!(state_of(39 * HEAT_TICK), 0);
        assert_eq!(state_of(40 * HEAT_TICK), 1);
        assert_eq!(state_of(165 * HEAT_TICK), 3);
        assert_eq!(cash(15 * HEAT_TICK, 1000), COIN_UNIT);
        assert_eq!(cash(150 * HEAT_TICK, 1000), PIP_UNIT);
        assert_eq!(flare(CAP_TICKS, 1000), FLARE_MAX);
        assert_eq!(charge_gain(1000, 1000, 1000), 1_000 * CHARGE_UNIT);
        assert_eq!(loss(1000, 1000), HEAT_TICK);
        assert_eq!(fire_damage(4000, 1, 0, false), 4000);
        assert_eq!(fire_damage(4000, 3, 3, false), 19_200);
        assert_eq!(milli(90 * HEAT_TICK), 90_000);
    }

    /// 12.10 B7 by the sim's own arithmetic: the reference stock (deal 30, take 10) ends at 90 Heat at the first KO.
    #[test]
    fn pace_reference() {
        let mut l = Ledger::default();
        for i in 0..4 {
            l.deal(30_000, 1000, 1000, 0);
            if i < 3 {
                l.take(10_000);
            }
        }
        assert_eq!(l.heat, 90 * HEAT_TICK);
        assert_eq!(state_of(l.heat), 2);
    }
}
