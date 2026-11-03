//! Offline input delay (practice parity with netplay): the local pads reach the game `n` frames late, as with a
//! netplay input delay of `n`. `--input-delay N`, else `input_delay=N` in settings.cfg / `MELEE_INPUT_DELAY=N`
//! (0..=8, default 0 = off). Only during a match; the menus are not delayed. The game sees ordinary (late) pads:
//! a recorded replay holds what the game saw and replays identically. Off (0), the pads pass through untouched.
use melee_types::PADStatus;
use std::collections::VecDeque;

pub struct DelayLine {
    pub frames: usize,
    q: VecDeque<[PADStatus; 4]>,
}

impl DelayLine {
    pub fn new(frames: usize) -> Self {
        DelayLine { frames: frames.min(8), q: VecDeque::new() }
    }

    /// From the command line (`--input-delay N`), else the settings / environment.
    pub fn from_args(args: &[String]) -> Self {
        let a = args.iter().position(|a| a == "--input-delay").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<i64>().ok());
        let n = a.unwrap_or_else(|| crate::cfg::open().get_int("input_delay", 0) as i64);
        DelayLine::new(n.clamp(0, 8) as usize)
    }

    /// This frame's pads: `pads` from `frames` calls ago while `active` (neutral, same connection state, until the
    /// line fills); `pads` themselves when inactive (the line is emptied, so the next match starts from neutral).
    pub fn apply(&mut self, pads: [PADStatus; 4], active: bool) -> [PADStatus; 4] {
        if self.frames == 0 || !active {
            self.q.clear();
            return pads;
        }
        if self.q.is_empty() {
            let mut n = pads;
            for p in n.iter_mut() {
                let err = p.err;
                *p = PADStatus::zeroed();
                p.err = err;
            }
            for _ in 0..self.frames {
                self.q.push_back(n);
            }
        }
        self.q.push_back(pads);
        self.q.pop_front().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_by_n() {
        let mut d = DelayLine::new(2);
        let mk = |b: u16| {
            let mut p = [PADStatus::zeroed(); 4];
            p[0].button = b;
            p
        };
        assert_eq!(d.apply(mk(1), true)[0].button, 0);
        assert_eq!(d.apply(mk(2), true)[0].button, 0);
        assert_eq!(d.apply(mk(3), true)[0].button, 1);
        assert_eq!(d.apply(mk(4), false)[0].button, 4);
        let mut z = DelayLine::new(0);
        assert_eq!(z.apply(mk(5), true)[0].button, 5);
    }
}
