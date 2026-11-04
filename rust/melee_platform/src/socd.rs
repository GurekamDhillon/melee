//! SOCD (simultaneous opposite cardinal directions) resolution for box controllers: what one axis
//! reads when both of its opposite buttons are held. One [`Socd`] per axis keeps the press order.
//!
//! Runs on the input side, before the value becomes a `PADStatus`, so whatever it decides is what
//! gets recorded, scripted and sent over netplay: the sim only ever sees the resolved PAD bytes.

/// How opposite directions held together resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SocdMode {
    /// Both held reads neutral.
    Neutral,
    /// Second input priority: the newer press wins; releasing it gives the older one back.
    LastWins,
    /// Second input priority without reactivation: the newer press wins, and the older one stays
    /// off until it is released and pressed again (the usual Melee box default).
    #[default]
    LastWinsNoReactivation,
    /// The press held longer wins.
    FirstWins,
}

impl SocdMode {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_ascii_lowercase().replace('-', "_").as_str() {
            "neutral" | "none" => SocdMode::Neutral,
            "2ip" | "last" | "last_wins" => SocdMode::LastWins,
            "2ip_noreact" | "2ip_no_reactivation" | "last_no_reactivation" => SocdMode::LastWinsNoReactivation,
            "first" | "first_wins" => SocdMode::FirstWins,
            _ => return None,
        })
    }
    pub fn name(self) -> &'static str {
        match self {
            SocdMode::Neutral => "neutral",
            SocdMode::LastWins => "2ip",
            SocdMode::LastWinsNoReactivation => "2ip_noreact",
            SocdMode::FirstWins => "first",
        }
    }
}

/// One axis' press history.
#[derive(Clone, Copy, Debug, Default)]
pub struct Socd {
    prev_neg: bool,
    prev_pos: bool,
    /// The most recent press: true = positive.
    last_pos: bool,
    lock_neg: bool,
    lock_pos: bool,
}

impl Socd {
    /// Feed this frame's two buttons; returns -1, 0 or 1.
    pub fn resolve(&mut self, mode: SocdMode, neg: bool, pos: bool) -> i8 {
        if neg && !self.prev_neg {
            self.last_pos = false;
        }
        if pos && !self.prev_pos {
            self.last_pos = true;
        }
        // a press made in the same frame as its opposite: positive was assigned last, keep it
        if neg && pos && !self.prev_neg && !self.prev_pos {
            self.last_pos = true;
        }
        self.prev_neg = neg;
        self.prev_pos = pos;
        if !neg {
            self.lock_neg = false;
        }
        if !pos {
            self.lock_pos = false;
        }
        let dir = |p: bool| if p { 1 } else { -1 };
        match (neg, pos) {
            (false, false) => 0,
            (true, false) => if self.lock_neg { 0 } else { -1 },
            (false, true) => if self.lock_pos { 0 } else { 1 },
            (true, true) => match mode {
                SocdMode::Neutral => 0,
                SocdMode::LastWins => dir(self.last_pos),
                SocdMode::FirstWins => dir(!self.last_pos),
                SocdMode::LastWinsNoReactivation => {
                    if self.last_pos {
                        self.lock_neg = true;
                    } else {
                        self.lock_pos = true;
                    }
                    dir(self.last_pos)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(mode: SocdMode, frames: &[(bool, bool)]) -> Vec<i8> {
        let mut s = Socd::default();
        frames.iter().map(|&(n, p)| s.resolve(mode, n, p)).collect()
    }

    // left held, right added, right released, left released
    const SEQ: [(bool, bool); 4] = [(true, false), (true, true), (true, false), (false, false)];

    #[test]
    fn neutral() {
        assert_eq!(run(SocdMode::Neutral, &SEQ), [-1, 0, -1, 0]);
    }
    #[test]
    fn last_wins_reactivates() {
        assert_eq!(run(SocdMode::LastWins, &SEQ), [-1, 1, -1, 0]);
    }
    #[test]
    fn last_wins_no_reactivation() {
        assert_eq!(run(SocdMode::LastWinsNoReactivation, &SEQ), [-1, 1, 0, 0]);
        // re-pressing the locked side brings it back
        let f = [(true, false), (true, true), (true, false), (false, false), (true, false)];
        assert_eq!(run(SocdMode::LastWinsNoReactivation, &f), [-1, 1, 0, 0, -1]);
        // releasing the older side while both are held leaves the newer one
        let f = [(true, false), (true, true), (false, true), (true, true)];
        assert_eq!(run(SocdMode::LastWinsNoReactivation, &f), [-1, 1, 1, -1]);
    }
    #[test]
    fn first_wins() {
        assert_eq!(run(SocdMode::FirstWins, &SEQ), [-1, -1, -1, 0]);
        assert_eq!(run(SocdMode::FirstWins, &[(false, true), (true, true)]), [1, 1]);
    }
    #[test]
    fn same_frame_press_is_deterministic() {
        assert_eq!(run(SocdMode::LastWins, &[(true, true)]), [1]);
        assert_eq!(run(SocdMode::Neutral, &[(true, true)]), [0]);
    }
    #[test]
    fn parse_names() {
        for m in [SocdMode::Neutral, SocdMode::LastWins, SocdMode::LastWinsNoReactivation, SocdMode::FirstWins] {
            assert_eq!(SocdMode::parse(m.name()), Some(m));
        }
        assert_eq!(SocdMode::parse("bogus"), None);
    }
}
