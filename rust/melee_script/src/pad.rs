//! Script control of the four physical pads (`gw_script_pad.c` section 3): `gd.input` overrides, what the game
//! saw (`gd.pad`), the menu mask and chord a script can keep from the game, and `gd.mirror_pad`.
//!
//! [`ScriptPads::apply`] is `gw_Script_PadApply`: the platform calls it with the four samples it just read, after its
//! own sources (txt pad script, live file, bot), and the game then sees the result.

/// One controller sample: the fields of `PADStatus` a script can reach (buttons in the PAD bit layout).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct PadSample {
    pub buttons: u16,
    pub stick_x: i8,
    pub stick_y: i8,
    pub substick_x: i8,
    pub substick_y: i8,
    pub trigger_l: u8,
    pub trigger_r: u8,
    /// 0 connected; `PAD_ERR_NO_CONTROLLER` (-1) otherwise
    pub err: i8,
}

#[derive(Clone, Copy, Default)]
struct Override {
    samples: i32,
    owner: i32,
    buttons: u32,
    sx: i32,
    sy: i32,
    cx: i32,
    cy: i32,
    tl: i32,
    tr: i32,
}

#[derive(Clone, Copy, Default)]
struct Seen {
    buttons: u32,
    sx: i32,
    sy: i32,
    cx: i32,
    cy: i32,
    tl: i32,
    tr: i32,
}

#[derive(Clone, Copy, Default)]
struct Mask {
    owner: i32,
    buttons: u32,
}

/// D-pad (bits 0-3) and START (0x1000): what a script may keep from the game
const MASKABLE: u32 = 15 | 0x1000;

pub struct ScriptPads {
    ovr: [Override; 4],
    seen: [Seen; 4],
    raw: [u32; 4],
    menu_mask: [Mask; 4],
    chord: [Mask; 4],
    mirror_from: i32,
    mirror_to: i32,
    mirror_take: bool,
    /// log lines `gw_pad_log` would write ("gw: pad: P1 script claim (owner 3)")
    pub log: Vec<String>,
}

impl Default for ScriptPads {
    fn default() -> Self {
        ScriptPads {
            ovr: [Override::default(); 4],
            seen: [Seen::default(); 4],
            raw: [0; 4],
            menu_mask: [Mask::default(); 4],
            chord: [Mask::default(); 4],
            mirror_from: -1,
            mirror_to: -1,
            mirror_take: false,
            log: Vec::new(),
        }
    }
}

impl ScriptPads {
    /// `gw_script_pad_mask`
    pub fn mask(&mut self, ch: i32, owner: i32, buttons: u32) -> bool {
        if !(0..=3).contains(&ch) || owner <= 0 || buttons & !MASKABLE != 0 {
            return false;
        }
        let m = &mut self.menu_mask[ch as usize];
        if m.owner != 0 && m.owner != owner {
            return false;
        }
        m.owner = if buttons != 0 { owner } else { 0 };
        m.buttons = buttons;
        true
    }

    /// `gw_script_pad_chord`: two or more buttons held together are hidden from the game; 0 releases.
    pub fn chord(&mut self, ch: i32, owner: i32, buttons: u32) -> bool {
        if !(0..=3).contains(&ch) || owner <= 0 || buttons & !0x1F7F != 0 {
            return false;
        }
        if buttons != 0 && buttons & (buttons - 1) == 0 {
            return false;
        }
        let c = &mut self.chord[ch as usize];
        if c.owner != 0 && c.owner != owner {
            return false;
        }
        c.owner = if buttons != 0 { owner } else { 0 };
        c.buttons = buttons;
        true
    }

    /// `gw_script_pad_masks_clear`
    pub fn masks_clear(&mut self) {
        self.menu_mask = [Mask::default(); 4];
        self.chord = [Mask::default(); 4];
    }

    pub fn raw_buttons(&self, ch: i32) -> u32 {
        if (0..=3).contains(&ch) { self.raw[ch as usize] } else { 0 }
    }

    /// `gw_script_pad_override`
    #[allow(clippy::too_many_arguments)]
    pub fn override_pad(&mut self, ch: i32, owner: i32, buttons: u32, sx: i32, sy: i32, cx: i32, cy: i32, l: i32, r: i32, samples: i32) {
        if !(0..=3).contains(&ch) || owner <= 0 {
            return;
        }
        let o = &mut self.ovr[ch as usize];
        if o.owner != owner {
            if o.owner != 0 {
                self.log.push(format!("gw: pad: P{} script release (owner {})", ch + 1, o.owner));
            }
            self.log.push(format!("gw: pad: P{} script claim (owner {})", ch + 1, owner));
        }
        *o = Override { samples, owner, buttons, sx, sy, cx, cy, tl: l, tr: r };
    }

    /// `gw_Script_PadFrameConsumed`: a logic frame that read the pads has passed; each held claim covered one more frame.
    pub fn frame_consumed(&mut self) {
        for o in self.ovr.iter_mut() {
            if o.owner != 0 && o.samples > 0 {
                o.samples -= 1;
            }
        }
    }

    /// `gw_script_pad_release` for the four physical pads (slots 5-6 are the game's virtual pads).
    pub fn release(&mut self, ch: i32, owner: i32) {
        if !(0..=3).contains(&ch) || owner <= 0 {
            return;
        }
        let c = ch as usize;
        if self.menu_mask[c].owner == owner {
            self.menu_mask[c] = Mask::default();
        }
        if self.chord[c].owner == owner {
            self.chord[c] = Mask::default();
        }
        if self.ovr[c].owner == owner {
            self.ovr[c].samples = 0;
            self.ovr[c].owner = 0;
            self.log.push(format!("gw: pad: P{} script release (owner {})", ch + 1, owner));
        }
    }

    pub fn release_owner(&mut self, owner: i32) {
        for ch in 0..4 {
            self.release(ch, owner);
        }
    }

    /// `gw_script_pad_state`: what the game saw on the last read.
    pub fn state(&self, ch: i32) -> (u32, i32, i32, i32, i32, i32, i32) {
        if !(0..=3).contains(&ch) {
            return (0, 0, 0, 0, 0, 0, 0);
        }
        let s = &self.seen[ch as usize];
        (s.buttons, s.sx, s.sy, s.cx, s.cy, s.tl, s.tr)
    }

    pub fn mirror(&mut self, from: i32, to: i32) {
        self.mirror_from = from;
        self.mirror_to = to;
        self.mirror_take = false;
    }
    pub fn take(&mut self, on: bool) {
        self.mirror_take = on;
    }

    /// `gw_Script_PadApply` (the override, mirror, mask and bookkeeping half). `driven` is what the platform's own
    /// script sources already drove; the returned mask adds the ports this layer drives.
    pub fn apply(&mut self, st: &mut [PadSample; 4], mut driven: u32, session: bool) -> u32 {
        for ch in 0..4 {
            let o = self.ovr[ch];
            if o.owner != 0 {
                // a claim stays connected and neutral after its queued samples run out: clear every physical field
                st[ch] = PadSample::default();
                if o.samples > 0 {
                    st[ch].stick_x = o.sx as i8;
                    st[ch].stick_y = o.sy as i8;
                    st[ch].substick_x = o.cx as i8;
                    st[ch].substick_y = o.cy as i8;
                    st[ch].trigger_l = o.tl as u8;
                    st[ch].trigger_r = o.tr as u8;
                    st[ch].buttons = o.buttons as u16;
                }
                driven |= 1 << ch;
            }
        }
        let (f, t) = (self.mirror_from, self.mirror_to);
        if (0..4).contains(&f) && (0..4).contains(&t) && !session {
            st[t as usize] = st[f as usize];
            driven |= 1 << t;
            if self.mirror_take {
                st[f as usize] = PadSample::default();
                driven |= 1 << f;
            }
        }
        for ch in 0..4 {
            self.raw[ch] = st[ch].buttons as u32;
            if !session {
                let mut hide = self.menu_mask[ch].buttons;
                let c = self.chord[ch].buttons;
                if c != 0 && self.raw[ch] & c == c {
                    hide |= c;
                }
                st[ch].buttons = (self.raw[ch] & !hide) as u16;
            }
            self.seen[ch] = Seen {
                buttons: st[ch].buttons as u32,
                sx: st[ch].stick_x as i32,
                sy: st[ch].stick_y as i32,
                cx: st[ch].substick_x as i32,
                cy: st[ch].substick_y as i32,
                tl: st[ch].trigger_l as i32,
                tr: st[ch].trigger_r as i32,
            };
        }
        driven
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_holds_for_samples_then_stays_neutral() {
        let mut p = ScriptPads::default();
        p.override_pad(0, 3, 0x0100, 10, -20, 0, 0, 0, 0, 2);
        let mut st = [PadSample { buttons: 0x4000, stick_x: 99, ..Default::default() }; 4];
        let d = p.apply(&mut st, 0, false);
        assert_eq!(d, 1);
        assert_eq!((st[0].buttons, st[0].stick_x, st[0].stick_y), (0x0100, 10, -20));
        p.frame_consumed();
        p.frame_consumed();
        let mut st = [PadSample { buttons: 0x4000, stick_x: 99, ..Default::default() }; 4];
        p.apply(&mut st, 0, false);
        assert_eq!(st[0], PadSample::default(), "claimed and neutral after the hold ran out");
        assert_eq!(st[1].stick_x, 99, "other ports untouched");
        p.release(0, 4);
        assert_eq!(p.apply(&mut [PadSample::default(); 4], 0, false), 1, "another owner cannot release it");
        p.release(0, 3);
        assert_eq!(p.apply(&mut [PadSample::default(); 4], 0, false), 0);
    }

    #[test]
    fn mask_chord_and_raw() {
        let mut p = ScriptPads::default();
        assert!(p.mask(0, 1, 0x0008));
        assert!(!p.mask(0, 2, 0x0004), "owned by another script");
        assert!(!p.mask(0, 1, 0x0100), "only d-pad and start");
        assert!(!p.chord(1, 1, 0x0100), "a chord is two or more buttons");
        assert!(p.chord(1, 1, 0x0110));
        let mut st = [PadSample::default(); 4];
        st[0].buttons = 0x0108;
        st[1].buttons = 0x0110;
        st[2].buttons = 0x0100;
        p.apply(&mut st, 0, false);
        assert_eq!(st[0].buttons, 0x0100);
        assert_eq!(st[1].buttons, 0);
        assert_eq!(p.raw_buttons(1), 0x0110);
        assert_eq!(p.state(2).0, 0x0100);
        // a session never hides buttons
        let mut st = [PadSample { buttons: 0x0008, ..Default::default() }; 4];
        p.apply(&mut st, 0, true);
        assert_eq!(st[0].buttons, 0x0008);
    }

    #[test]
    fn mirror_and_take() {
        let mut p = ScriptPads::default();
        p.mirror(0, 1);
        p.take(true);
        let mut st = [PadSample::default(); 4];
        st[0].buttons = 0x0100;
        let d = p.apply(&mut st, 0, false);
        assert_eq!((st[1].buttons, st[0].buttons, d), (0x0100, 0, 3));
    }
}
