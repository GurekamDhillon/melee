//! Gecko codes applied to the retail image, so the referee runs "retail + a code list" (notes/referee.md section 9).
//!
//! The code bytes are never in this repo: they come at run time from a Dolphin game INI (Slippi's own
//! `Output/Netplay/GALE01r2.ini` in a local ssbm-asm checkout, `[Gecko_Enabled]` names + `[Gecko]` bodies) or from a
//! replay's recorded list. Semantics as Slippi's list walker (`Common/Gecko/ProcessCodeList.asm`) sizes the code types
//! and the Gecko code handler applies them once at boot (Dolphin `GeckoCode.cpp` installs the handler at 0x80001800
//! and the list after it; the handler does the patching):
//! - `00`/`02`/`04` (ba-relative, bit 0x10 of the type clear): write a byte (with a repeat count in the high half),
//!   a halfword (repeat count) or a word at 0x80000000 | (addr & 0x01FFFFFF);
//! - `06`: write the N bytes that follow (padded to 8);
//! - `C2`: insert assembly. The N lines are copied to a code area; the hooked word becomes `b area`; the block's
//!   last word (the 00000000 the assembler leaves, or after a 60000000 nop) becomes `b hook + 4`;
//! - `C0` (run every frame), `08` (serial write) and the po-relative / conditional types are not applied (listed in
//!   the report): the netplay list has no `08` and no conditionals.
//!
//! Code blocks are placed at [`AREA_BASE`] (outside every DOL section and the referee's stack), so a `b` from the
//! game's text reaches them (both within +-32 MB).

use crate::dol::{Dol, Section};

/// Where the inserted blocks go in the guest image.
pub const AREA_BASE: u32 = 0x8160_0000;

#[derive(Clone, Debug, PartialEq)]
pub struct Code {
    pub name: String,
    /// (word0, word1) pairs as written.
    pub lines: Vec<(u32, u32)>,
}

/// One code line group as the list walker sees it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Write { addr: u32, bytes: Vec<u8> },
    Insert { addr: u32, body: Vec<u32> },
    Skipped { kind: u8, addr: u32 },
}

/// The `$Name` blocks of `[Gecko]` that `[Gecko_Enabled]` turns on (or all of them with `all`).
pub fn parse_ini(text: &str, all: bool) -> Vec<Code> {
    let mut section = "";
    let mut enabled: Vec<String> = vec![];
    let mut codes: Vec<Code> = vec![];
    for raw in text.lines() {
        let l = raw.trim();
        if l.starts_with('[') {
            section = if l.starts_with("[Gecko_Enabled]") { "en" } else if l.starts_with("[Gecko]") { "g" } else { "" };
            continue;
        }
        match section {
            "en" if l.starts_with('$') => enabled.push(l[1..].trim().to_string()),
            "g" if l.starts_with('$') => codes.push(Code { name: l[1..].trim().to_string(), lines: vec![] }),
            "g" => {
                let mut it = l.split_whitespace();
                let (Some(a), Some(b)) = (it.next(), it.next()) else { continue };
                let (Ok(a), Ok(b)) = (u32::from_str_radix(a, 16), u32::from_str_radix(b, 16)) else { continue };
                if let Some(c) = codes.last_mut() {
                    c.lines.push((a, b));
                }
            }
            _ => {}
        }
    }
    // a code's INI name carries its authors in brackets: match the enabled name as a prefix
    codes.retain(|c| all || enabled.iter().any(|e| c.name == *e || c.name.starts_with(&format!("{e} ")) || c.name.starts_with(&format!("{e}["))));
    codes
}

/// Walk one code's lines into entries.
pub fn entries(lines: &[(u32, u32)]) -> Vec<Entry> {
    let mut out = vec![];
    let mut i = 0;
    while i < lines.len() {
        let (w0, w1) = lines[i];
        let kind = (w0 >> 24) as u8;
        let addr = 0x8000_0000 | (w0 & 0x01FF_FFFF);
        let po = kind & 0x10 != 0;
        let t = kind & 0xEE; // type without the po bit and the address-high bit
        if (w0 >> 28) == 0xF && w1 == 0 {
            break;
        }
        match t {
            0x00 | 0x02 | 0x04 if !po => {
                let bytes = match t {
                    0x00 => (0..=(w1 >> 16)).map(|_| w1 as u8).collect(),
                    0x02 => (0..=(w1 >> 16)).flat_map(|_| (w1 as u16).to_be_bytes()).collect(),
                    _ => w1.to_be_bytes().to_vec(),
                };
                out.push(Entry::Write { addr, bytes });
                i += 1;
            }
            0x06 if !po => {
                let n = w1 as usize;
                let nl = n.div_ceil(8);
                let mut b = vec![];
                for &(x, y) in &lines[i + 1..(i + 1 + nl).min(lines.len())] {
                    b.extend_from_slice(&x.to_be_bytes());
                    b.extend_from_slice(&y.to_be_bytes());
                }
                b.truncate(n);
                out.push(Entry::Write { addr, bytes: b });
                i += 1 + nl;
            }
            0xC2 if !po => {
                let n = w1 as usize;
                let mut body = vec![];
                for &(x, y) in &lines[i + 1..(i + 1 + n).min(lines.len())] {
                    body.push(x);
                    body.push(y);
                }
                out.push(Entry::Insert { addr, body });
                i += 1 + n;
            }
            0xC0 => {
                out.push(Entry::Skipped { kind, addr });
                i += 1 + w1 as usize;
            }
            0x08 => {
                out.push(Entry::Skipped { kind, addr });
                i += 2;
            }
            _ => {
                out.push(Entry::Skipped { kind, addr });
                i += 1;
            }
        }
    }
    out
}

/// What [`apply`] did.
#[derive(Default, Debug)]
pub struct Report {
    pub writes: usize,
    pub inserts: Vec<(u32, u32, u32)>, // (hook, block start, block words)
    /// writes / hooks outside every DOL section (e.g. into the Gecko area or the scene heap): applied only when they
    /// fall in the code area
    pub outside: Vec<u32>,
    pub skipped: Vec<(u8, u32)>,
}

fn b_to(from: u32, to: u32) -> u32 {
    0x4800_0000 | (to.wrapping_sub(from) & 0x03FF_FFFC)
}

fn poke(dol: &mut Dol, addr: u32, bytes: &[u8]) -> bool {
    for (k, &v) in bytes.iter().enumerate() {
        let a = addr + k as u32;
        let Some(s) = dol.sections.iter_mut().find(|s| a >= s.addr && a < s.addr + s.bytes.len() as u32) else { return false };
        s.bytes[(a - s.addr) as usize] = v;
    }
    true
}

/// Apply `codes` (in list order) to `dol`: writes into its sections, inserts into a new text section at
/// [`AREA_BASE`].
pub fn apply(dol: &mut Dol, codes: &[Code]) -> Report {
    let mut rep = Report::default();
    let mut area: Vec<u8> = vec![];
    for c in codes {
        for e in entries(&c.lines) {
            match e {
                Entry::Write { addr, bytes } => {
                    if poke(dol, addr, &bytes) {
                        rep.writes += 1;
                    } else {
                        rep.outside.push(addr);
                    }
                }
                Entry::Insert { addr, mut body } => {
                    let start = AREA_BASE + area.len() as u32;
                    // the block's last word is the return: `b hook + 4` (Gecko's handler writes it over the 00000000)
                    let len = body.len() as u32;
                    if let Some(last) = body.last_mut() {
                        *last = b_to(start + 4 * (len - 1), addr + 4);
                    }
                    let n = body.len() as u32;
                    for w in &body {
                        area.extend_from_slice(&w.to_be_bytes());
                    }
                    if poke(dol, addr, &b_to(addr, start).to_be_bytes()) {
                        rep.inserts.push((addr, start, n));
                    } else {
                        rep.outside.push(addr);
                    }
                }
                Entry::Skipped { kind, addr } => rep.skipped.push((kind, addr)),
            }
        }
    }
    if !area.is_empty() {
        while area.len() % 32 != 0 {
            area.extend_from_slice(&0u32.to_be_bytes());
        }
        dol.sections.push(Section { addr: AREA_BASE, text: true, bytes: area });
    }
    rep
}
