//! Slippi online: the per-frame RNG seed and the desync checksum of Slippi's online codeset
//! (slippi-ssbm-asm `e03af7f`+, as shipped in Slippi Dolphin `0653df819`). Specified in
//! docs/rust-port/notes/slippi-online-patches.md sections 1 and 2; this is our own implementation of that spec.
//!
//! **Off by default.** Nothing here runs unless a caller turns on [`enable`] for a game: the default game is unchanged.
//! The engine side (the SyncRNG proc, reading the fighters for the checksum) lives in the crates that own those
//! structures; this module is the pure arithmetic and the per-game state.

/// `RNG_HISTORY_COUNT`: the ring the seed bits and the B-held masks are kept in, indexed by `frame & 15`.
pub const RNG_HISTORY: usize = 16;
/// `UNFREEZE_INPUTS_FRAME`: checksums of frames up to here are never compared.
pub const UNFREEZE_INPUTS_FRAME: i32 = 84;

/// External character ids that can roll a random special (`PlayerBlock_LoadExternalCharID`).
pub const EXT_GAME_AND_WATCH: u32 = 0x03;
pub const EXT_LUIGI: u32 = 0x07;
pub const EXT_PEACH: u32 = 0x0C;

/// murmur3's 32-bit finalizer, as FN_SyncRNG applies it.
pub fn fmix32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2_AE35);
    x ^= x >> 16;
    x
}

/// The seed FN_SyncRNG writes for global frame `frame` (`gm_80479D58.unk_8`).
pub fn seed_for_frame(frame: u32, rng_offset: u32, seed_bits: u8) -> u32 {
    fmix32(frame.rotate_left(16).wrapping_add(rng_offset)) ^ ((seed_bits as u32) << 24)
}

/// One port's pad as the ASM sees it in section 11 of TriggerSendInput: the PADStatus fields it reads, the
/// 9th wire byte (the random bits the owner sent with it) and the port's external character id.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadBits {
    /// PADStatus.button (B = 0x0200).
    pub buttons: u16,
    pub stick_x: i8,
    pub stick_y: i8,
    /// Byte 8 of the pad report; 0 for an empty slot.
    pub rng_bits: u8,
    /// `PlayerBlock_LoadExternalCharID` of the port.
    pub ext_char: u32,
}

/// Section 11 for one frame: returns `(seed_bits, b_held_mask)`. `prev_b_held` is the mask stored for the previous frame.
pub fn seed_bits(pads: &[PadBits; 4], prev_b_held: u8) -> (u8, u8) {
    let mut b_held = 0u8;
    let mut rolling = 0u32;
    let mut all_bits = 0u32;
    for (p, pad) in pads.iter().enumerate() {
        all_bits |= (pad.rng_bits as u32) << (8 * p);
        if pad.buttons & 0x0200 == 0 {
            continue;
        }
        b_held |= 1 << p;
        if prev_b_held & (1 << p) != 0 {
            continue; // not a fresh press
        }
        let rolls = match pad.ext_char {
            EXT_PEACH => (pad.stick_y as i32) <= -44,
            EXT_GAME_AND_WATCH | EXT_LUIGI => (pad.stick_x as i32).abs() >= 48,
            _ => false,
        };
        if rolls {
            rolling |= 1 << p;
        }
    }
    if rolling == 0 {
        return (0, b_held);
    }
    // everyone's bits except the one rolling port, or everyone's when several roll
    let used = if rolling & (rolling - 1) != 0 { 0xF } else { 0xF & !rolling };
    let mut bits = 0u32;
    for p in 0..4 {
        if used & (1 << p) != 0 {
            bits ^= all_bits >> (8 * p);
        }
    }
    ((bits & 0xFF) as u8, b_held)
}

/// One fighter (main or sub entity of a player slot) as the checksum reads it.
#[derive(Clone, Copy, Debug, Default)]
pub struct FighterSample {
    pub motion_id: u32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub percent: f32,
    pub spawn_num: u32,
}

/// PowerPC `fctiwz`: f64 -> s32 toward zero, saturating; NaN gives 0x80000000.
pub fn fctiwz(v: f64) -> i32 {
    if v.is_nan() {
        i32::MIN
    } else if v >= 2147483647.0 {
        i32::MAX
    } else if v <= -2147483648.0 {
        i32::MIN
    } else {
        v.trunc() as i32
    }
}

/// Accumulates FN_COMPUTE_CHECKSUM over the four player slots, in slot order.
#[derive(Clone, Copy, Debug, Default)]
pub struct Checksum {
    cs: u32,
    fsum: f32,
}

impl Checksum {
    pub fn new() -> Self {
        Self::default()
    }
    /// One present fighter entity (call for the main entity, then the sub entity, of each slot).
    pub fn fighter(&mut self, f: &FighterSample) {
        self.cs ^= f.motion_id;
        self.cs ^= f.pos_x.to_bits();
        self.cs ^= f.pos_y.to_bits();
        self.cs ^= f.percent.to_bits();
        self.cs ^= f.spawn_num;
        self.fsum += f.pos_x;
        self.fsum += f.pos_y;
        self.fsum += f.percent;
    }
    /// The end of a slot: its stocks byte (StaticPlayer +0x8E, zero-extended), for every slot even an empty one.
    pub fn slot_end(&mut self, stocks: u8) {
        self.cs ^= stocks as u32;
    }
    pub fn finish(&self) -> u32 {
        let hi = (self.cs >> 16) ^ (self.cs & 0xFFFF);
        let lo = fctiwz(self.fsum as f64) as u32 & 0xFFFF;
        (hi << 16) | lo
    }
}

/// The ASM's comparison of a local and a remote checksum for the same frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compare {
    Same,
    /// The high halves differ: Slippi shows "Desync Risk" only.
    Soft,
    /// The float sums (low halves, as s16) differ by more than 1: "DESYNC DETECTED", the game ends.
    Hard,
}

pub fn compare(local: u32, remote: u32) -> Compare {
    let d = (local as u16 as i16 as i32) - (remote as u16 as i16 as i32);
    if !(-1..=1).contains(&d) {
        Compare::Hard
    } else if (local >> 16) != (remote >> 16) {
        Compare::Soft
    } else {
        Compare::Same
    }
}

/// Per-game state (the ODB fields the seed uses).
#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    pub rng_offset: u32,
    pub b_held_hist: [u8; RNG_HISTORY],
    pub seed_bits_hist: [u8; RNG_HISTORY],
}

impl State {
    /// Section 11 for frame `frame`: record its seed bits and B mask.
    pub fn note_pads(&mut self, frame: u32, pads: &[PadBits; 4]) -> u8 {
        let prev = self.b_held_hist[(frame.wrapping_sub(1) as usize) & (RNG_HISTORY - 1)];
        let (bits, held) = seed_bits(pads, prev);
        self.b_held_hist[frame as usize & (RNG_HISTORY - 1)] = held;
        self.seed_bits_hist[frame as usize & (RNG_HISTORY - 1)] = bits;
        bits
    }
    /// FN_SyncRNG for frame `frame`.
    pub fn seed(&self, frame: u32) -> u32 {
        seed_for_frame(frame, self.rng_offset, self.seed_bits_hist[frame as usize & (RNG_HISTORY - 1)])
    }
}

static mut STATE: Option<State> = None;

/// Turns Slippi online mode on for the next game with this `rng_offset` (the decider's MATCH_SELECTIONS value).
pub fn enable(rng_offset: u32) {
    unsafe { STATE = Some(State { rng_offset, ..State::default() }) };
}

pub fn disable() {
    unsafe { STATE = None };
}

/// The online state when the mode is on (it is part of the game state: snapshot it with the rollback state).
pub fn state() -> Option<&'static mut State> {
    unsafe { (*core::ptr::addr_of_mut!(STATE)).as_mut() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmix_reference() {
        // murmur3 fmix32 reference values
        assert_eq!(fmix32(0), 0);
        assert_eq!(fmix32(1), 0x514E_28B7);
    }

    #[test]
    fn seed_rotates_frame() {
        assert_eq!(seed_for_frame(1, 0, 0), fmix32(0x0001_0000));
        assert_eq!(seed_for_frame(0x0001_0002, 5, 0), fmix32(0x0002_0001 + 5));
        assert_eq!(seed_for_frame(3, 7, 0xAB) ^ seed_for_frame(3, 7, 0), 0xAB00_0000);
    }

    fn pad(ch: u32, b: bool, x: i8, y: i8, bits: u8) -> PadBits {
        PadBits { buttons: if b { 0x200 } else { 0 }, stick_x: x, stick_y: y, rng_bits: bits, ext_char: ch }
    }

    #[test]
    fn bits_only_on_fresh_roll() {
        let mut p = [pad(0x02, false, 0, 0, 0x11), pad(EXT_PEACH, true, 0, -50, 0x22), PadBits::default(), PadBits::default()];
        // Peach down-B fresh: the other ports' bits (0x11 ^ 0 ^ 0)
        assert_eq!(seed_bits(&p, 0), (0x11, 0b10));
        // held from last frame: nothing
        assert_eq!(seed_bits(&p, 0b10), (0, 0b10));
        // stick not down enough
        p[1].stick_y = -43;
        assert_eq!(seed_bits(&p, 0).0, 0);
        // Luigi side-B on port 0 too: two rolling ports -> everyone
        p[1].stick_y = -44;
        p[0] = pad(EXT_LUIGI, true, -48, 0, 0x11);
        assert_eq!(seed_bits(&p, 0), (0x33, 0b11));
        // a non-rolling character pressing B
        p[0] = pad(0x02, true, 100, 0, 0x11);
        p[1] = pad(0x02, true, 0, -100, 0x22);
        assert_eq!(seed_bits(&p, 0).0, 0);
    }

    #[test]
    fn checksum_layout() {
        let mut c = Checksum::new();
        let f = FighterSample { motion_id: 14, pos_x: -60.0, pos_y: 10.0, percent: 0.0, spawn_num: 1 };
        c.fighter(&f);
        c.slot_end(4);
        let cs = 14 ^ (-60.0f32).to_bits() ^ 10.0f32.to_bits() ^ 1 ^ 4;
        let want = (((cs >> 16) ^ (cs & 0xFFFF)) << 16) | ((-50i32) as u32 & 0xFFFF);
        assert_eq!(c.finish(), want);
        assert_eq!(compare(want, want), Compare::Same);
        assert_eq!(compare(want, want ^ 0x1_0000), Compare::Soft);
        assert_eq!(compare(want, (want & 0xFFFF_0000) | 0xFFD0), Compare::Hard);
        assert_eq!(fctiwz(f64::NAN), i32::MIN);
        assert_eq!(fctiwz(-1.9), -1);
    }
}
