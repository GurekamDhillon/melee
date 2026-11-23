//! Time sync against a Slippi-wire peer: local pacing only. Nothing here changes what goes on the wire; it decides
//! when this side holds a frame (it is ahead of the peer) or runs one extra frame in a tick (it is behind).
//!
//! The rules follow the behaviour of Slippi Dolphin and Hero's Source Port (read for behaviour only):
//! * the offset (`Client::time_offset_us`, positive = this side ahead) is checked every 30 online frames;
//! * the side that is behind catches up itself: more than a frame plus 10 ms behind, after online frame 120, it runs
//!   up to 3 extra frames, one on every 5th frame;
//! * the side that is ahead holds frames only while the match starts (online frames up to 120: any offset over
//!   10 ms, up to 5 frames). After that it holds one frame only for a gap the other side's capped catch-up cannot
//!   cover (more than 3 + 1 frames plus the slack): holding as well as the other side catching up corrects one gap
//!   twice and leaves the held side closer to its next stall.
//!
//! `MELEE_TIMESYNC_OLD=1` keeps the previous rule (hold up to 5 frames on any offset over 10 ms all match, no
//! catch-up, no start hold) for A/B runs.
//!
//! Against a Slippi-wire peer (real Slippi Dolphin) the previous rule is the DEFAULT ([`slippi_old_rule`]): the
//! staging build of 86dbd5424 had an instant HARD desync against a real Slippi player (Dream Land, online frame 125,
//! the remote checksum constant while ours moved) and the new rule was only ever tested engine vs engine and vs the
//! local mainline Dolphin. `MELEE_TIMESYNC_NEW=1` opts a Slippi-wire match into the new rule for testing.

/// One console frame at 59.94 Hz.
pub const FRAME_US: i32 = 16683;
/// An offset this small is left alone.
pub const SLACK_US: i32 = 10_000;
/// Online frames counted as the match start (strict holds, no catch-up).
pub const OPENING: i32 = 120;
/// The most frames the behind side runs extra per check.
pub const MAX_ADVANCE: i32 = 3;
/// Online frames between offset checks.
pub const CHECK_EVERY: i32 = 30;
/// An extra frame runs only on online frames that are multiples of this.
pub const ADVANCE_EVERY: i32 = 5;
/// Before the peer's first pad: how long the start hold waits for a peer still loading.
pub const FIRST_PAD_S: u64 = 30;
/// After it: silence that ends the match.
pub const SILENCE_S: u64 = 7;

/// Frames to hold at an offset check on online frame `frame` (`offset_us` > 0: this side is ahead).
pub fn hold_frames(offset_us: i32, frame: i32) -> i32 {
    if frame <= OPENING {
        if offset_us <= SLACK_US {
            return 0;
        }
        return ((offset_us - SLACK_US) / FRAME_US + 1).min(5);
    }
    (offset_us > SLACK_US + (MAX_ADVANCE + 1) * FRAME_US) as i32
}

/// Extra frames to run after an offset check on online frame `frame` (`offset_us` < 0: this side is behind).
pub fn advance_frames(offset_us: i32, frame: i32) -> i32 {
    if offset_us >= -(FRAME_US + SLACK_US) || frame <= OPENING {
        return 0;
    }
    ((-(offset_us as i64) - SLACK_US as i64) / FRAME_US as i64 + 1).min(MAX_ADVANCE as i64) as i32
}

/// The previous rule (before port/timesync): hold on any offset over the slack, up to 5 frames, all match.
pub fn hold_frames_old(offset_us: i32) -> i32 {
    if offset_us > SLACK_US {
        ((offset_us - SLACK_US) / FRAME_US + 1).min(5)
    } else {
        0
    }
}

/// Whether the A/B switch asks for the previous behaviour.
pub fn old_rule() -> bool {
    std::env::var_os("MELEE_TIMESYNC_OLD").is_some_and(|v| v == "1")
}

/// The rule for a Slippi-wire match (`slippi-netplay`, `play --slippi-*`): the previous rule unless
/// `MELEE_TIMESYNC_NEW=1` (and never with `MELEE_TIMESYNC_OLD=1`).
pub fn slippi_old_rule() -> bool {
    old_rule() || !std::env::var_os("MELEE_TIMESYNC_NEW").is_some_and(|v| v == "1")
}

/// `MELEE_TIMESYNC_FORCE_CATCHUP=N[:K]` -> (N, K); unset or malformed: (0, 0).
fn force_catchup_env() -> (i32, i32) {
    let Some(v) = std::env::var("MELEE_TIMESYNC_FORCE_CATCHUP").ok() else { return (0, 0) };
    let mut it = v.split(':');
    let n = it.next().and_then(|x| x.trim().parse::<i32>().ok()).filter(|&n| n > 0).unwrap_or(0);
    let k = it.next().and_then(|x| x.trim().parse::<i32>().ok()).filter(|&k| k > 0).unwrap_or(MAX_ADVANCE);
    if n > 0 { (n, k) } else { (0, 0) }
}

/// What a tick should do.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Step {
    /// frames to hold (`RbSession::request_wait`)
    pub hold: i32,
    /// run one extra frame this tick
    pub advance: bool,
}

/// The per-match time-sync state.
#[derive(Clone, Debug, Default)]
pub struct TimeSync {
    pub old: bool,
    checked: i32,
    advanced_on: i32,
    advance_left: i32,
    /// totals this match: frames held, frames run extra, checks that found an offset over the slack
    pub held: i32,
    pub advanced: i32,
    pub offsets_seen: i32,
    /// test hook `MELEE_TIMESYNC_FORCE_CATCHUP=N[:K]` (new rule only, read once per match in `new`): on every Nth
    /// offset check after the opening, start a catch-up of K frames (default MAX_ADVANCE) whatever the offset, so
    /// the extra-frame path runs against a peer on the same clock. 0: off.
    pub force_every: i32,
    pub force_frames: i32,
    checks_after_opening: i32,
}

impl TimeSync {
    pub fn new(old: bool) -> Self {
        let (force_every, force_frames) = if old { (0, 0) } else { force_catchup_env() };
        TimeSync { old, checked: i32::MIN, advanced_on: i32::MIN, force_every, force_frames, ..Default::default() }
    }

    /// Call once per tick with the online frame about to run and the measured offset (read lazily: `offset` is
    /// called on check frames only).
    pub fn step(&mut self, frame: i32, offset: impl FnOnce() -> i32) -> Step {
        let mut st = Step::default();
        if frame <= 0 {
            return st;
        }
        if frame % CHECK_EVERY == 0 && frame != self.checked {
            self.checked = frame;
            let off = offset();
            if off.abs() > SLACK_US {
                self.offsets_seen += 1;
            }
            st.hold = if self.old { hold_frames_old(off) } else { hold_frames(off, frame) };
            if !self.old && st.hold == 0 && self.advance_left == 0 {
                self.advance_left = advance_frames(off, frame);
            }
            if self.force_every > 0 && frame > OPENING {
                self.checks_after_opening += 1;
                if st.hold == 0 && self.advance_left == 0 && self.checks_after_opening % self.force_every == 0 {
                    self.advance_left = self.force_frames;
                }
            }
            self.held += st.hold;
        }
        if st.hold == 0 && self.advance_left > 0 && frame % ADVANCE_EVERY == 0 && frame != self.advanced_on {
            self.advanced_on = frame;
            self.advance_left -= 1;
            self.advanced += 1;
            st.advance = true;
        }
        st
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_only_as_a_last_resort_after_the_opening() {
        assert_eq!(hold_frames(9_000, 60), 0);
        assert_eq!(hold_frames(11_000, 60), 1);
        assert_eq!(hold_frames(200_000, 90), 5);
        assert_eq!(hold_frames(11_000, 150), 0);
        assert_eq!(hold_frames(SLACK_US + 4 * FRAME_US, 150), 0);
        assert_eq!(hold_frames(SLACK_US + 4 * FRAME_US + 1, 150), 1);
        assert_eq!(hold_frames(1_000_000, 150), 1);
    }

    #[test]
    fn the_behind_side_catches_up_after_the_opening() {
        assert_eq!(advance_frames(-30_000, 90), 0);
        assert_eq!(advance_frames(-(FRAME_US + SLACK_US), 150), 0);
        assert_eq!(advance_frames(-(FRAME_US + SLACK_US) - 1, 150), 2);
        assert_eq!(advance_frames(-1_000_000, 150), 3);
        assert_eq!(advance_frames(i32::MIN, 150), 3);
    }

    #[test]
    fn extra_frames_run_on_every_fifth_frame_once() {
        let mut t = TimeSync::new(false);
        assert_eq!(t.step(150, || -60_000), Step { hold: 0, advance: true });
        assert!(!t.step(150, || 0).advance, "the same frame does not advance twice");
        assert!(!t.step(152, || 0).advance);
        assert!(t.step(155, || 0).advance);
        assert!(t.step(160, || 0).advance);
        assert!(!t.step(165, || 0).advance);
        assert_eq!(t.advanced, 3);
    }

    #[test]
    fn slippi_wire_defaults_to_the_old_rule() {
        if std::env::var_os("MELEE_TIMESYNC_NEW").is_none() {
            assert!(slippi_old_rule());
        }
    }

    #[test]
    fn a_forced_catch_up_runs_on_every_nth_check_after_the_opening() {
        let mut t = TimeSync::new(false);
        t.force_every = 2;
        t.force_frames = 3;
        let mut adv = vec![];
        for f in 1..=300 {
            if t.step(f, || 0).advance {
                adv.push(f);
            }
        }
        // checks at 150 (1st), 180 (2nd: forced), 210, 240 (forced), 270, 300 (forced): 3 frames each on multiples of 5
        assert_eq!(adv, vec![180, 185, 190, 240, 245, 250, 300]);
    }

    #[test]
    fn the_old_rule_holds_all_match_and_never_advances() {
        let mut t = TimeSync::new(true);
        assert_eq!(t.step(300, || 30_000).hold, 2);
        assert_eq!(t.step(330, || -90_000), Step::default());
    }
}
