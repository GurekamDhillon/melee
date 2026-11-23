//! The input exchange of one game (plan section 2.9): our committed pads out in COMMITs until acknowledged, the
//! peer's pads in with every receive rule, ACK / PING / PONG round trips, the time-offset samples, the checksum pair,
//! and the input transcript (section 2.11).
//!
//! Frame numbers here are the game's own (1 = the first match frame); the wire numbers carry the game's base
//! ((k - 1) x 2^24 in game k of a session), added and removed at this boundary only.

use super::blake2b::Blake2b;
use super::crypto::Bytes32;
use super::desc::GAME_FRAME_SPAN;
use super::wire::{Commit, Msg, Pad, MAX_COMMIT_FRAMES, SAMPLE_VERSION};
use std::collections::BTreeMap;

/// Frames the peer may commit past our finalized frame.
pub const MAX_REMOTE_LEAD: i32 = 200;
/// Offset samples kept (the estimate is the mean of their middle third).
pub const OFFSET_SAMPLES: usize = 30;
/// One frame at 59.94 Hz, in microseconds.
pub const FRAME_US: i64 = 16683;
/// Round trips above this are discarded.
pub const MAX_RTT_US: u32 = 2_000_000;
/// Pads kept each way behind the newest.
const KEEP: i32 = 512;
/// Early-input samples kept per peer.
pub const MAX_SAMPLES: usize = 8;

#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub commits_sent: u64,
    pub commits_received: u64,
    pub acks_sent: u64,
    pub acks_received: u64,
    pub malformed: u64,
    pub other_game: u64,
    pub gaps: u64,
    pub conflicts: u64,
    pub too_far_ahead: u64,
    pub samples_received: u64,
    pub samples_ignored: u64,
}

pub struct Input {
    base: i32,
    local: BTreeMap<i32, Pad>,
    local_latest: i32,
    /// highest frame F with every one of our frames 1..=F held by the peer
    peer_ack: i32,
    remote: BTreeMap<i32, Pad>,
    remote_latest: i32,
    /// newly taken remote frames, for the game (drained by `take_fresh`)
    fresh: Vec<(i32, Pad)>,
    /// our finalized frame (the 200-frame lead limit counts from it)
    floor: i32,
    ck_local: (i32, u32),
    /// the peer's newest (checksum frame, checksum)
    pub ck_remote: (i32, u32),
    newest_remote_seen: i32,
    /// frame and send time (us) of our latest COMMIT
    clock: Option<(i32, u64)>,
    ping_us: Option<u32>,
    offsets: Vec<i32>,
    offset_next: usize,
    /// the last time a COMMIT brought a new frame (us)
    pub last_progress_us: u64,
    /// early inputs: target frame (game numbering) -> pad, at most MAX_SAMPLES; only kept when `samples_on`
    pub samples: BTreeMap<i32, Pad>,
    pub samples_on: bool,
    /// samples taken since the last `take_samples` (target in game numbering, pad), for the game's predictor
    fresh_samples: Vec<(i32, Pad)>,
    pub stats: Stats,
}

impl Input {
    /// The exchange of game `game` (1-based) of a session.
    pub fn new(game: u32, now_us: u64) -> Input {
        Input {
            base: (game.max(1) as i32 - 1) * GAME_FRAME_SPAN,
            local: BTreeMap::new(),
            local_latest: 0,
            peer_ack: 0,
            remote: BTreeMap::new(),
            remote_latest: 0,
            fresh: Vec::new(),
            floor: 0,
            ck_local: (0, 0),
            ck_remote: (0, 0),
            newest_remote_seen: 0,
            clock: None,
            ping_us: None,
            offsets: Vec::new(),
            offset_next: 0,
            last_progress_us: now_us,
            samples: BTreeMap::new(),
            samples_on: false,
            fresh_samples: Vec::new(),
            stats: Stats::default(),
        }
    }

    pub fn base(&self) -> i32 {
        self.base
    }
    pub fn local_latest(&self) -> i32 {
        self.local_latest
    }
    pub fn remote_latest(&self) -> i32 {
        self.remote_latest
    }
    pub fn peer_ack(&self) -> i32 {
        self.peer_ack
    }
    pub fn local_pad(&self, f: i32) -> Option<Pad> {
        self.local.get(&f).copied()
    }
    pub fn remote_pad(&self, f: i32) -> Option<Pad> {
        self.remote.get(&f).copied()
    }
    pub fn ping_us(&self) -> Option<u32> {
        self.ping_us
    }

    /// Commits our pad of frame `f`; frames are committed once, in order from 1.
    pub fn commit(&mut self, f: i32, pad: Pad) -> bool {
        if f != self.local_latest + 1 {
            return false;
        }
        self.local.insert(f, pad);
        self.local_latest = f;
        let keep_from = self.peer_ack.min(f - KEEP);
        while self.local.first_key_value().is_some_and(|(k, _)| *k < keep_from) {
            self.local.pop_first();
        }
        true
    }

    /// Our newest finalized frame and its state checksum (sent in every COMMIT).
    pub fn set_checksum(&mut self, f: i32, ck: u32) {
        if f > self.ck_local.0 {
            self.ck_local = (f, ck);
        }
    }

    /// Our finalized frame, for the lead limit.
    pub fn set_floor(&mut self, f: i32) {
        self.floor = self.floor.max(f);
    }

    /// The COMMIT for this tick: every frame the peer has not acknowledged (at most 128, newest first), or the newest
    /// alone. None before frame 1 is committed.
    pub fn build_commit(&mut self, now_us: u64) -> Option<Msg> {
        if self.local_latest < 1 {
            return None;
        }
        let oldest = if self.peer_ack >= self.local_latest { self.local_latest } else { (self.peer_ack + 1).max(self.local_latest - MAX_COMMIT_FRAMES as i32 + 1) };
        let pads: Vec<Pad> = (oldest..=self.local_latest).rev().map(|f| self.local.get(&f).copied().unwrap_or_default()).collect();
        self.clock = Some((self.local_latest, now_us));
        self.stats.commits_sent += 1;
        let b = self.base;
        Some(Msg::Commit(Commit {
            frame: b + self.local_latest,
            checksum_frame: if self.ck_local.0 > 0 { b + self.ck_local.0 } else { 0 },
            checksum: self.ck_local.1,
            ack: b + self.remote_latest,
            stamp: now_us as u32,
            pads,
        }))
    }

    pub fn build_ping(&self, now_us: u64) -> Msg {
        Msg::Ping { stamp: now_us as u32 }
    }

    /// The early-input samples taken since the last call (target frame, pad): predictions only.
    pub fn take_samples(&mut self) -> Vec<(i32, Pad)> {
        std::mem::take(&mut self.fresh_samples)
    }

    /// Our SAMPLE (revision 2): `target` (game numbering) is the pad frame the change first appears in.
    pub fn build_sample(&self, target: i32, slot: u8, pad: Pad) -> Msg {
        Msg::Sample { version: SAMPLE_VERSION, target: self.base + target, slot, pad }
    }

    /// The peer frames taken since the last call, in order.
    pub fn take_fresh(&mut self) -> Vec<(i32, Pad)> {
        std::mem::take(&mut self.fresh)
    }

    fn rtt_sample(&mut self, now_us: u64, echo: u32) {
        let rtt = (now_us as u32).wrapping_sub(echo);
        if rtt <= MAX_RTT_US {
            self.ping_us = Some(match self.ping_us {
                None => rtt,
                Some(p) => ((p as u64 * 7 + rtt as u64) / 8) as u32,
            });
        }
    }

    fn in_game(&self, wire: i32, low: i32) -> bool {
        wire >= self.base + low && wire - self.base < GAME_FRAME_SPAN
    }

    /// An input-channel message from the peer; the reply to send at once, if any.
    pub fn on_msg(&mut self, m: &Msg, now_us: u64) -> Option<Msg> {
        match m {
            Msg::Commit(c) => self.on_commit(c, now_us),
            Msg::Ack { ack, echo } => {
                if !self.in_game(*ack, 0) {
                    self.stats.other_game += 1;
                    return None;
                }
                self.stats.acks_received += 1;
                let a = (*ack - self.base).min(self.local_latest);
                self.peer_ack = self.peer_ack.max(a);
                self.rtt_sample(now_us, *echo);
                None
            }
            Msg::Ping { stamp } => Some(Msg::Pong { stamp: *stamp }),
            Msg::Pong { stamp } => {
                self.rtt_sample(now_us, *stamp);
                None
            }
            Msg::Sample { version, target, pad, .. } => {
                self.stats.samples_received += 1;
                if !self.samples_on || *version != SAMPLE_VERSION || !self.in_game(*target, 1) {
                    self.stats.samples_ignored += 1;
                    return None;
                }
                let t = *target - self.base;
                if t > self.remote_latest {
                    self.samples.insert(t, *pad);
                    self.fresh_samples.push((t, *pad));
                    while self.samples.len() > MAX_SAMPLES {
                        self.samples.pop_first();
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn on_commit(&mut self, c: &Commit, now_us: u64) -> Option<Msg> {
        let count = c.pads.len() as i32;
        if count < 1 || count > MAX_COMMIT_FRAMES as i32 || c.frame < count || c.checksum_frame < 0 || c.ack < 0 {
            self.stats.malformed += 1;
            return None;
        }
        // another game's numbers: nothing taken, nothing answered
        if !self.in_game(c.frame, 1) || !self.in_game(c.ack, 0) {
            self.stats.other_game += 1;
            return None;
        }
        let frame = c.frame - self.base;
        let ack = c.ack - self.base;
        let ckf = if c.checksum_frame > self.base { c.checksum_frame - self.base } else { 0 };
        if frame < count || ack > self.local_latest {
            self.stats.malformed += 1;
            return None;
        }
        self.stats.commits_received += 1;
        self.peer_ack = self.peer_ack.max(ack);
        let oldest = frame - count + 1;
        let limit = self.floor + MAX_REMOTE_LEAD;
        let mut fresh = false;
        if oldest > self.remote_latest + 1 {
            self.stats.gaps += 1;
        } else {
            for f in oldest..=frame {
                let pad = c.pads[(frame - f) as usize];
                if f <= self.remote_latest {
                    if self.remote.get(&f).is_some_and(|h| *h != pad) {
                        self.stats.conflicts += 1;
                    }
                    continue;
                }
                if f > limit {
                    self.stats.too_far_ahead += 1;
                    break;
                }
                self.remote.insert(f, pad);
                self.fresh.push((f, pad));
                self.remote_latest = f;
                fresh = true;
            }
            let keep_from = self.remote_latest - KEEP;
            while self.remote.first_key_value().is_some_and(|(k, _)| *k < keep_from) {
                self.remote.pop_first();
            }
            while self.samples.first_key_value().is_some_and(|(k, _)| *k <= self.remote_latest) {
                self.samples.pop_first();
            }
        }
        if frame >= self.newest_remote_seen && frame <= limit {
            self.ck_remote = (ckf, c.checksum);
        }
        if frame > self.newest_remote_seen && frame <= limit {
            self.newest_remote_seen = frame;
            // one offset sample per new peer frame, on its first arrival
            if let Some((lf, lt)) = self.clock {
                let off = now_us as i64 - self.ping_us.unwrap_or(0) as i64 / 2 - lt as i64 + FRAME_US * (lf - frame) as i64;
                let off = off.clamp(-(1 << 30), 1 << 30) as i32;
                if self.offsets.len() < OFFSET_SAMPLES {
                    self.offsets.push(off);
                } else {
                    self.offsets[self.offset_next] = off;
                }
                self.offset_next = (self.offset_next + 1) % OFFSET_SAMPLES;
            }
        }
        if fresh {
            self.last_progress_us = now_us;
            self.stats.acks_sent += 1;
            return Some(Msg::Ack { ack: self.base + self.remote_latest, echo: c.stamp });
        }
        None
    }

    /// Our time offset against the peer in microseconds (positive: we are ahead): the mean of the middle third of the
    /// samples. 0 with no samples.
    pub fn time_offset_us(&self) -> i32 {
        if self.offsets.is_empty() {
            return 0;
        }
        let mut s = self.offsets.clone();
        s.sort_unstable();
        let n = s.len();
        let (lo, hi) = if n >= 3 { (n / 3, n - n / 3) } else { (0, n) };
        let mid = &s[lo..hi];
        (mid.iter().map(|&v| v as i64).sum::<i64>() / mid.len() as i64) as i32
    }
}

/// The running input transcript of one game: BLAKE2b over `"MeleeUnlockedNet1 inputs" || descriptor digest`, then
/// for every frame in order `f` (u32) and both slots' pads. The digest after every frame is kept (32 bytes a frame),
/// so the peer's RESULT can be checked at whatever count it names.
#[derive(Clone)]
pub struct Transcript {
    h: Blake2b,
    pub frames: i32,
    digests: Vec<Bytes32>,
}

impl Transcript {
    pub fn new(descriptor_digest: &Bytes32) -> Transcript {
        let mut h = Blake2b::new(32);
        h.update(b"MeleeUnlockedNet1 inputs").update(descriptor_digest);
        let d0 = h.clone().finalize().try_into().unwrap();
        Transcript { h, frames: 0, digests: vec![d0] }
    }
    /// Appends frame `self.frames + 1`.
    pub fn push(&mut self, slot0: &Pad, slot1: &Pad) {
        self.frames += 1;
        self.h.update(&(self.frames as u32).to_le_bytes()).update(slot0).update(slot1);
        let d = self.h.clone().finalize().try_into().unwrap();
        self.digests.push(d);
    }
    pub fn digest(&self) -> Bytes32 {
        self.digests[self.frames as usize]
    }
    /// Our digest after exactly `frames` frames, once we got that far.
    pub fn digest_at(&self, frames: i32) -> Option<Bytes32> {
        usize::try_from(frames).ok().and_then(|f| self.digests.get(f)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mu_net::blake2b::hash32;

    fn p(v: u8) -> Pad {
        [v; 8]
    }

    /// Delivers a's COMMIT to b and b's ACK back.
    fn hop(a: &mut Input, b: &mut Input, t: u64) -> Option<Msg> {
        let c = a.build_commit(t)?;
        let r = b.on_msg(&c, t + 1000);
        if let Some(ack) = &r {
            a.on_msg(ack, t + 2000);
        }
        r
    }

    #[test]
    fn commits_carry_unacked_history_newest_first() {
        let mut a = Input::new(1, 0);
        let mut b = Input::new(1, 0);
        for f in 1..=3 {
            assert!(a.commit(f, p(f as u8)));
        }
        assert!(!a.commit(5, p(5)), "in order only");
        let Some(Msg::Commit(c)) = a.build_commit(10) else { panic!() };
        assert_eq!((c.frame, c.pads.len(), c.pads[0], c.pads[2]), (3, 3, p(3), p(1)));
        assert_eq!(b.on_msg(&Msg::Commit(c), 20), Some(Msg::Ack { ack: 3, echo: 10 }));
        assert_eq!(b.take_fresh(), vec![(1, p(1)), (2, p(2)), (3, p(3))]);
        a.on_msg(&Msg::Ack { ack: 3, echo: 10 }, 5010);
        assert_eq!(a.peer_ack(), 3);
        assert_eq!(a.ping_us(), Some(5000));
        // everything acknowledged: the newest alone; a repeat delivers nothing and is not answered
        let Some(Msg::Commit(c)) = a.build_commit(30) else { panic!() };
        assert_eq!(c.pads.len(), 1);
        assert_eq!(b.on_msg(&Msg::Commit(c), 40), None);
    }

    #[test]
    fn receive_rules() {
        let mut b = Input::new(1, 0);
        b.commit(1, p(0));
        let mk = |frame: i32, n: usize, ack: i32| Msg::Commit(Commit { frame, checksum_frame: 0, checksum: 0, ack, stamp: 0, pads: vec![p(frame as u8); n] });
        // frame < count, ack above what we committed, negative fields
        b.on_msg(&mk(1, 2, 0), 0);
        b.on_msg(&mk(2, 1, 5), 0);
        b.on_msg(&Msg::Commit(Commit { frame: 2, checksum_frame: -1, checksum: 0, ack: 0, stamp: 0, pads: vec![p(1)] }), 0);
        assert_eq!(b.stats.malformed, 3);
        // a gap: frame 3 alone while we hold nothing
        assert_eq!(b.on_msg(&mk(3, 1, 0), 0), None);
        assert_eq!(b.stats.gaps, 1);
        // another game's frame numbers: dropped whole
        b.on_msg(&mk(GAME_FRAME_SPAN + 1, 1, 0), 0);
        assert_eq!(b.stats.other_game, 1);
        // a conflict: frame 1 again with other bytes
        b.on_msg(&mk(1, 1, 0), 0);
        let c = Commit { frame: 1, checksum_frame: 0, checksum: 0, ack: 0, stamp: 0, pads: vec![p(9)] };
        b.on_msg(&Msg::Commit(c), 0);
        assert_eq!((b.stats.conflicts, b.remote_pad(1)), (1, Some(p(1))));
        // the lead limit: 200 past our finalized frame (0)
        let mut z = Input::new(1, 0);
        z.commit(1, p(0));
        let big = Commit { frame: 128, checksum_frame: 0, checksum: 0, ack: 0, stamp: 0, pads: vec![p(1); 128] };
        z.on_msg(&Msg::Commit(big), 0);
        let more = Commit { frame: 250, checksum_frame: 0, checksum: 0, ack: 0, stamp: 0, pads: vec![p(2); 122] };
        z.on_msg(&Msg::Commit(more), 0);
        assert_eq!((z.remote_latest(), z.stats.too_far_ahead), (200, 1));
    }

    #[test]
    fn game_two_numbers_carry_the_base() {
        let mut a = Input::new(2, 0);
        let mut b = Input::new(2, 0);
        a.commit(1, p(1));
        a.set_checksum(1, 77);
        let Some(Msg::Commit(c)) = a.build_commit(0) else { panic!() };
        assert_eq!((c.frame, c.checksum_frame, c.ack), (GAME_FRAME_SPAN + 1, GAME_FRAME_SPAN + 1, GAME_FRAME_SPAN));
        let mut g1 = Input::new(1, 0);
        assert_eq!(g1.on_msg(&Msg::Commit(c.clone()), 0), None);
        assert_eq!(g1.stats.other_game, 1);
        assert!(hop(&mut a, &mut b, 0).is_some());
        assert_eq!(b.ck_remote, (1, 77));
        assert_eq!(a.peer_ack(), 1);
    }

    #[test]
    fn lossy_reordered_exchange_converges() {
        // a sends 300 frames; every third COMMIT is lost and pairs arrive swapped; b holds them all in the end
        let mut a = Input::new(1, 0);
        let mut b = Input::new(1, 0);
        b.commit(1, p(0));
        b.set_floor(400);
        let mut held: Option<Msg> = None;
        for f in 1..=300 {
            a.commit(f, p((f * 7) as u8));
            let c = a.build_commit(f as u64 * 16683).unwrap();
            if f % 3 == 0 {
                continue;
            }
            if f % 2 == 0 {
                held = Some(c);
                continue;
            }
            for m in [Some(c), held.take()].into_iter().flatten() {
                if let Some(ack) = b.on_msg(&m, f as u64 * 16683 + 3000) {
                    if f % 5 != 0 {
                        a.on_msg(&ack, f as u64 * 16683 + 6000);
                    }
                }
            }
        }
        for _ in 0..3 {
            hop(&mut a, &mut b, 6_000_000);
        }
        assert_eq!(b.remote_latest(), 300);
        assert!((1..=300).all(|f| b.remote_pad(f) == Some(p((f * 7) as u8))));
        assert_eq!(b.stats.conflicts, 0);
    }

    #[test]
    fn offsets_and_ping() {
        let mut a = Input::new(1, 0);
        let mut b = Input::new(1, 0);
        a.commit(1, p(0));
        b.on_msg(&a.build_ping(1000), 0).map(|pong| a.on_msg(&pong, 9000));
        assert_eq!(a.ping_us(), Some(8000));
        // b is two frames behind a in time: its frame F is sent when a sends F + 2
        for f in 1..=40 {
            a.commit(f + 1, p(0));
            let _ = a.build_commit(f as u64 * 16683 + 1_000_000);
            b.commit(f, p(1));
            let c = b.build_commit(0).unwrap();
            a.on_msg(&c, f as u64 * 16683 + 1_000_000 + 4000);
        }
        // now - rtt/2 - T + 16683 * (L - F) = 4000 - 4000 + 16683 * 1 (L = F + 1 here)
        assert_eq!(a.time_offset_us(), 16683);
    }

    #[test]
    fn transcript_is_blake2b_of_the_frames() {
        let d = [3u8; 32];
        let mut t = Transcript::new(&d);
        t.push(&p(1), &p(2));
        t.push(&p(3), &p(4));
        let mut raw = Vec::new();
        raw.extend_from_slice(b"MeleeUnlockedNet1 inputs");
        raw.extend_from_slice(&d);
        for (f, a, b) in [(1u32, 1u8, 2u8), (2, 3, 4)] {
            raw.extend_from_slice(&f.to_le_bytes());
            raw.extend_from_slice(&p(a));
            raw.extend_from_slice(&p(b));
        }
        assert_eq!(t.digest(), hash32(&[&raw]));
        assert_eq!(t.digest_at(2), Some(t.digest()));
        assert_eq!(t.digest_at(0), Some(hash32(&[b"MeleeUnlockedNet1 inputs", &d])));
    }
}
