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
    let (bits, held, _) = seed_bits_rolling(pads, prev_b_held);
    (bits, held)
}

/// [`seed_bits`] plus the mask of the ports that rolled a random special this frame (0: no pad's bits were read).
pub fn seed_bits_rolling(pads: &[PadBits; 4], prev_b_held: u8) -> (u8, u8, u8) {
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
        return (0, b_held, 0);
    }
    // everyone's bits except the one rolling port, or everyone's when several roll
    let used = if rolling & (rolling - 1) != 0 { 0xF } else { 0xF & !rolling };
    let mut bits = 0u32;
    for p in 0..4 {
        if used & (1 << p) != 0 {
            bits ^= all_bits >> (8 * p);
        }
    }
    ((bits & 0xFF) as u8, b_held, rolling as u8)
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

/// One fighter's part of Slippi's desync-recovery record (`DFRE`: stocks u8, percent u16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncedFighter {
    pub stocks: u8,
    pub percent: u16,
}

/// The state Slippi keeps per checksum frame for a desync recovery (`DDLE_RECOVERY_*` in StartEngineLoop.asm: the
/// timer's seconds, then stocks and percent of all four slots) and sends to the peer in `SLIPPI_SYNCED_STATE`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncedState {
    pub seconds: u32,
    pub fighters: [SyncedFighter; 4],
}

impl Default for SyncedState {
    /// `SlippiDesyncRecoveryResp`'s default (4 stocks, 0 percent, the 8:00 timer): what a game that is not a recovery
    /// starts from.
    fn default() -> Self {
        SyncedState { seconds: 8 * 60, fighters: [SyncedFighter { stocks: 4, percent: 0 }; 4] }
    }
}

impl SyncedState {
    /// `seconds;stocks,percent;...` (four slots): how a recovery state travels between processes and into a test.
    pub fn to_arg(&self) -> String {
        let mut t = self.seconds.to_string();
        for f in &self.fighters {
            t += &format!(";{},{}", f.stocks, f.percent);
        }
        t
    }
    pub fn from_arg(a: &str) -> Option<SyncedState> {
        let mut it = a.trim().split(';');
        let mut s = SyncedState { seconds: it.next()?.parse().ok()?, ..Default::default() };
        for f in &mut s.fighters {
            let (st, pc) = it.next()?.split_once(',')?;
            *f = SyncedFighter { stocks: st.parse().ok()?, percent: pc.parse().ok()? };
        }
        it.next().is_none().then_some(s)
    }
}

/// `SlippiNetplayClient::GetDesyncRecoveryState`'s reconciliation of the local and the remote synced states (their
/// last synced frames can differ by a few frames): the timers must be within 1 s (the larger is used), every slot's
/// stocks must be equal, percents within 25 (the lower is used). Err = "is_error": Slippi drops the connection.
pub fn merge_recovery(local: &SyncedState, remotes: &[SyncedState]) -> Result<SyncedState, &'static str> {
    let mut m = *local;
    for r in remotes {
        if (m.seconds as i64 - r.seconds as i64).abs() > 1 {
            return Err("timer values for desync recovery too different");
        }
        m.seconds = m.seconds.max(r.seconds);
        for j in 0..4 {
            if m.fighters[j].stocks != r.fighters[j].stocks {
                return Err("stocks remaining for desync recovery do not match");
            }
            if (m.fighters[j].percent as i32 - r.fighters[j].percent as i32).abs() > 25 {
                return Err("current health for desync recovery too different");
            }
            m.fighters[j].percent = m.fighters[j].percent.min(r.fighters[j].percent);
        }
    }
    Ok(m)
}

/// StartEngineLoop.asm's per-frame desync check, for one peer: which compares count, when the game ends, and which
/// state the recovery keeps.
///
/// * a compare is only made for a frame the peer reported that is finalized here (`<= ODB_STABLE_FINALIZED_FRAME`; the
///   caller passes only such frames) and `> UNFREEZE_INPUTS_FRAME`;
/// * a Hard (`compare`) sets `ODB_IS_DESYNC_STATE_DISPLAYED`: the game ends (`FN_END_GAME`) and no frame is compared
///   after it; the recovery state is not updated by the Hard pass;
/// * a Soft shows "Desync Risk" (once) and the check goes on;
/// * every other pass copies the compared frame's timer, stocks and percent into `ODB_DESYNC_RECOVERY_*`.
#[derive(Clone, Debug, Default)]
pub struct DesyncWatch {
    /// the Hard happened (the game is ending)
    pub hard_at: Option<i32>,
    /// the first Soft's frame
    pub soft_at: Option<i32>,
    /// the state of the last frame compared without a Hard (or the frame-0 state until one is)
    pub synced: Option<SyncedState>,
}

impl DesyncWatch {
    /// The frame-0 fallback (`SKIP_TAKE_CHECKSUM`: a game whose peer never reports still has a state that is not zeros).
    pub fn set_initial(&mut self, s: SyncedState) {
        if self.synced.is_none() {
            self.synced = Some(s);
        }
    }

    /// One compare of online frame `fr`; `state` is this side's state at that frame. `None`: the frame is not
    /// compared (the countdown, or after a Hard).
    pub fn check(&mut self, fr: i32, local: u32, remote: u32, state: Option<SyncedState>) -> Option<Compare> {
        if self.hard_at.is_some() || fr <= UNFREEZE_INPUTS_FRAME {
            return None;
        }
        let c = compare(local, remote);
        match c {
            Compare::Hard => self.hard_at = Some(fr),
            Compare::Soft => {
                self.soft_at.get_or_insert(fr);
                self.synced = state.or(self.synced);
            }
            Compare::Same => self.synced = state.or(self.synced),
        }
        Some(c)
    }

    /// What Slippi sends at the end of a game that ended on the Hard (ranked only).
    pub fn recovery(&self) -> SyncedState {
        self.synced.unwrap_or_default()
    }
}

/// The match struct writes of `FN_END_GAME` (StartEngineLoop.asm, `ssbm-asm` 9817968 + the later 55 -> 90 change):
/// `0x1` the "pauser" (the local player's index), `0x8` the exit type 7 (LRAS: `OUTCOME_NO_CONTEST`), `0x24D5` the
/// GAME! think's maximum time, 90 frames. The game's own end-of-match check does the rest.
pub const END_GAME_EXIT_LRAS: u8 = 7;
pub const END_GAME_THINK_FRAMES: u8 = 90;

/// Per-game state (the ODB fields the seed uses).
#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    pub rng_offset: u32,
    pub b_held_hist: [u8; RNG_HISTORY],
    pub seed_bits_hist: [u8; RNG_HISTORY],
    /// The pads of the frame about to run, as the driver hands them to the game; SyncRNG mixes them
    /// (`note_pads`) at its own frame number, so the bits and the seed always use the same index.
    pub pending: [PadBits; 4],
    /// The ports that rolled in the last mixed frame (their seed bits came from the other ports' pad bytes).
    pub last_rolling: u8,
}

impl State {
    /// Section 11 for frame `frame`: record its seed bits and B mask.
    pub fn note_pads(&mut self, frame: u32, pads: &[PadBits; 4]) -> u8 {
        let prev = self.b_held_hist[(frame.wrapping_sub(1) as usize) & (RNG_HISTORY - 1)];
        let (bits, held, rolling) = seed_bits_rolling(pads, prev);
        self.last_rolling = rolling;
        self.b_held_hist[frame as usize & (RNG_HISTORY - 1)] = held;
        self.seed_bits_hist[frame as usize & (RNG_HISTORY - 1)] = bits;
        bits
    }
    /// FN_SyncRNG with TriggerSendInput section 11 folded in: mix `pending` for `frame`, then the seed.
    pub fn sync(&mut self, frame: u32) -> u32 {
        let p = self.pending;
        self.note_pads(frame, &p);
        self.seed(frame)
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

    fn synced(sec: u32, st: [u8; 4], pc: [u16; 4]) -> SyncedState {
        let mut s = SyncedState { seconds: sec, ..Default::default() };
        for j in 0..4 {
            s.fighters[j] = SyncedFighter { stocks: st[j], percent: pc[j] };
        }
        s
    }

    #[test]
    fn recovery_merge_is_slippis() {
        let a = synced(300, [3, 2, 0, 0], [40, 90, 0, 0]);
        // timer: the larger within 1 s; percent: the lower within 25; stocks equal
        let b = synced(301, [3, 2, 0, 0], [60, 80, 0, 0]);
        let m = merge_recovery(&a, &[b]).unwrap();
        assert_eq!((m.seconds, m.fighters[0].percent, m.fighters[1].percent), (301, 40, 80));
        assert!(merge_recovery(&a, &[synced(303, [3, 2, 0, 0], [40, 90, 0, 0])]).is_err());
        assert!(merge_recovery(&a, &[synced(300, [3, 1, 0, 0], [40, 90, 0, 0])]).is_err());
        assert!(merge_recovery(&a, &[synced(300, [3, 2, 0, 0], [66, 90, 0, 0])]).is_err());
        assert!(merge_recovery(&a, &[synced(300, [3, 2, 0, 0], [65, 90, 0, 0])]).is_ok());
    }

    #[test]
    fn recovery_arg_round_trips() {
        let a = synced(301, [3, 2, 0, 0], [40, 80, 0, 0]);
        assert_eq!(SyncedState::from_arg(&a.to_arg()), Some(a));
        assert_eq!(SyncedState::from_arg("1;2"), None);
    }

    #[test]
    fn watch_follows_the_asm() {
        let mut w = DesyncWatch::default();
        w.set_initial(SyncedState::default());
        let s1 = synced(470, [4, 4, 0, 0], [10, 0, 0, 0]);
        let s2 = synced(469, [4, 4, 0, 0], [20, 0, 0, 0]);
        // the countdown is never compared
        assert_eq!(w.check(84, 1, 0x10000, Some(s1)), None);
        assert_eq!(w.recovery(), SyncedState::default());
        assert_eq!(w.check(85, 7, 7, Some(s1)), Some(Compare::Same));
        // a Soft (high halves differ) keeps going and keeps the state
        assert_eq!(w.check(86, 0x0001_0005, 0x0002_0005, Some(s2)), Some(Compare::Soft));
        assert_eq!(w.soft_at, Some(86));
        assert_eq!(w.recovery(), s2);
        // a Hard ends it: its own frame's state is not kept, and nothing is compared after it
        let s3 = synced(468, [4, 3, 0, 0], [30, 0, 0, 0]);
        assert_eq!(w.check(87, 0x0000_0005, 0x0000_0009, Some(s3)), Some(Compare::Hard));
        assert_eq!(w.hard_at, Some(87));
        assert_eq!(w.recovery(), s2);
        assert_eq!(w.check(88, 1, 1, Some(s3)), None);
        assert_eq!(w.recovery(), s2);
    }

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
