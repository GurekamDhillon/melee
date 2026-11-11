//! Port of the fixed-frame txt pad script (`gw_script_pad.c` part 1, `MELEE_PAD_SCRIPT=<file>.txt`):
//! one segment per line, `<frames> <buttons_hex> [stickX stickY [trigL trigR [cstickX cstickY]]]`, `#` comments;
//! each PADRead consumes one frame, the last segment holds; broadcast to the channels
//! `MELEE_PAD_CHANNELS` names (default "0"). Lua scripts, MELEE_PAD_LIVE, gd.input claims and
//! the pad bot are not ported.
use crate::pad::*;

pub const GW_SCRIPT_MAX: usize = 8192;

/// Set by the host's scene flow each tick: a live match (VS / sudden death) is on.
pub static IN_MATCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn no_pause() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("MELEE_PAD_NO_PAUSE").is_ok_and(|v| v == "1"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    pub frames: i32,
    pub buttons: u16,
    pub sx: i8,
    pub sy: i8,
    pub tl: u8,
    pub tr: u8,
    /// The C-stick (port extension: absent = centred, as the C script).
    pub cx: i8,
    pub cy: i8,
}

#[derive(Clone, Debug)]
pub struct PadScript {
    pub segments: Vec<Segment>,
    pub chan_mask: u32,
    cursor: usize,
    left: i32,
}

/// `sscanf("%d")`: optional sign + digits, stops at the first non-digit. None if no digits.
fn scan_int(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    let mut i = 0;
    let neg = match b.first() {
        Some(b'-') => { i = 1; true }
        Some(b'+') => { i = 1; false }
        _ => false,
    };
    let start = i;
    let mut v: i64 = 0;
    while i < b.len() && b[i].is_ascii_digit() {
        v = v.wrapping_mul(10).wrapping_add((b[i] - b'0') as i64);
        i += 1;
    }
    (i > start).then_some(if neg { -v } else { v })
}

/// `sscanf("%x")`: optional sign, optional 0x, hex digits.
fn scan_hex(s: &str) -> Option<u32> {
    let mut t = s;
    let neg = if let Some(r) = t.strip_prefix('-') { t = r; true } else { t = t.strip_prefix('+').unwrap_or(t); false };
    if let Some(r) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        if r.bytes().next().is_some_and(|c| c.is_ascii_hexdigit()) {
            t = r;
        }
    }
    let n = t.bytes().take_while(|c| c.is_ascii_hexdigit()).count();
    if n == 0 {
        return None;
    }
    let v = t[..n].bytes().fold(0u32, |a, c| a.wrapping_mul(16).wrapping_add((c as char).to_digit(16).unwrap()));
    Some(if neg { v.wrapping_neg() } else { v })
}

impl PadScript {
    /// `MELEE_PAD_CHANNELS` parse: digits 0-3; none valid -> channel 0.
    pub fn parse_channels(s: &str) -> u32 {
        let m = s.bytes().filter(|c| (b'0'..=b'3').contains(c)).fold(0u32, |m, c| m | 1 << (c - b'0'));
        if m == 0 { 1 } else { m }
    }

    /// Parse script text (`gw_pad_script_load`'s loop).
    pub fn parse(text: &str, chan_mask: u32) -> Self {
        let mut segments = Vec::new();
        for line in text.lines() {
            if segments.len() >= GW_SCRIPT_MAX {
                break;
            }
            let p = line.trim_start_matches([' ', '\t']);
            if p.is_empty() || p.starts_with('#') || p.starts_with('\r') {
                continue;
            }
            let mut toks = p.split_ascii_whitespace();
            let Some(frames) = toks.next().and_then(scan_int) else { continue };
            let Some(buttons) = toks.next().and_then(scan_hex) else { continue };
            // sscanf stops at the first failed conversion: later fields keep 0.
            let mut rest = [0i64; 6];
            for slot in rest.iter_mut() {
                match toks.next().and_then(scan_int) {
                    Some(v) => *slot = v,
                    None => break,
                }
            }
            let frames = frames as i32;
            if frames <= 0 {
                continue;
            }
            segments.push(Segment {
                frames,
                buttons: buttons as u16,
                sx: rest[0] as i8,
                sy: rest[1] as i8,
                tl: rest[2] as u8,
                tr: rest[3] as u8,
                cx: rest[4] as i8,
                cy: rest[5] as i8,
            });
        }
        Self { segments, chan_mask, cursor: 0, left: 0 }
    }

    /// From `MELEE_PAD_SCRIPT` (a `.txt`) and `MELEE_PAD_CHANNELS`. None if unset, `.lua`, or unreadable.
    pub fn from_env() -> Option<Self> {
        let path = std::env::var("MELEE_PAD_SCRIPT").ok().filter(|p| !p.is_empty())?;
        if path.to_ascii_lowercase().ends_with(".lua") {
            crate::plog!("gw: pad script: .lua scripts are not supported by the Rust platform yet ({path})");
            return None;
        }
        let mask = std::env::var("MELEE_PAD_CHANNELS").ok().filter(|s| !s.is_empty()).map(|s| Self::parse_channels(&s)).unwrap_or(1);
        match std::fs::read_to_string(&path) {
            Ok(t) => {
                let s = Self::parse(&t, mask);
                crate::plog!("gw: pad script: {} segments from {path}", s.segments.len());
                Some(s)
            }
            Err(_) => {
                crate::plog!("gw: pad script: cannot open {path}");
                None
            }
        }
    }

    /// The segment the next `apply` will deliver (None: no script), without consuming it.
    pub fn peek_next(&self) -> Option<Segment> {
        if self.segments.is_empty() {
            return None;
        }
        Some(self.segments[self.cursor.min(self.segments.len() - 1)])
    }

    /// `gw_pad_script_apply`: returns the mask of channels driven.
    pub fn apply(&mut self, st: &mut [PADStatus; PAD_CHANMAX]) -> u32 {
        if self.segments.is_empty() {
            return 0;
        }
        if self.cursor >= self.segments.len() {
            self.cursor = self.segments.len() - 1;
        }
        if self.left == 0 {
            self.left = self.segments[self.cursor].frames;
        }
        let g = self.segments[self.cursor];
        let mut driven = 0;
        for (ch, s) in st.iter_mut().enumerate() {
            if self.chan_mask & (1 << ch) == 0 {
                continue;
            }
            s.stickX = g.sx;
            s.stickY = g.sy;
            s.substickX = g.cx;
            s.substickY = g.cy;
            s.triggerLeft = g.tl;
            s.triggerRight = g.tr;
            s.button = g.buttons;
            // MELEE_PAD_NO_PAUSE=1: a script that mashes Start through the menus does not pause a live match
            if IN_MATCH.load(std::sync::atomic::Ordering::Relaxed) && no_pause() {
                s.button &= !PAD_BUTTON_START;
            }
            s.err = 0;
            driven |= 1 << ch;
        }
        self.left -= 1;
        if self.left == 0 {
            self.cursor += 1;
        }
        driven
    }
}
