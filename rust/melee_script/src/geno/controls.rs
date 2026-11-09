//! A Geno define's `controls` block and the button mapping it builds (docs/geno.md section 26). Ported from
//! pc/platform/geno_controls_schema.inc (the parse) and gw_fighter_controls_model.h (the pure mapping: normalize,
//! build, apply). No profile, registry or game-memory access here.
use super::json::{Doc, Ty};

/// GW_FC_BUTTONS
pub const BUTTONS: usize = 12;
/// GW_FC_MAX_LAYOUTS / GW_FC_MAX_EXTRAS
pub const MAX_LAYOUTS: usize = 16;
pub const MAX_EXTRAS: usize = 8;
/// GW_FC_NAME (31 bytes and the terminator)
pub const NAME: usize = 32;
/// GW_FC_BUTTON_MASK: the PAD bits a map rewrites; the rest pass through
pub const BUTTON_MASK: u32 = 0x1f7f;
/// The choice word: bit 0 Tap jump off, bits 2-5 the layout, bits 8-15 the extras
pub const TAP_MASK: u32 = 1;
pub const TAP_ON: u32 = 0;
pub const TAP_OFF: u32 = 1;
pub const LAYOUT_MASK: u32 = 0x3c;
pub const EXTRA_MASK: u32 = 0xff00;
pub const CHOICE_MASK: u32 = TAP_MASK | LAYOUT_MASK | EXTRA_MASK;
/// GW_FC_LAYOUTS / GW_FC_EXTRAS (the `field` of Count / Name)
pub const LAYOUTS: i32 = 0;
pub const EXTRAS: i32 = 1;

/// gw_fc_bits: the PAD bit of each button, in label order
pub const BITS: [u16; BUTTONS] = [0x100, 0x200, 0x400, 0x800, 0x10, 0x1000, 0x40, 0x20, 8, 4, 1, 2];
/// gw_fc_labels
pub const LABELS: [&str; BUTTONS] = ["A", "B", "X", "Y", "Z", "START", "L", "R", "Up", "Down", "Left", "Right"];

/// GwFighterControlsMap: one layout or extra
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Map {
    pub name: String,
    pub specified: u32,
    pub dest: [u16; BUTTONS],
}

/// GwFighterControlsSchema
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Schema {
    pub layout: Vec<Map>,
    pub extra: Vec<Map>,
}

impl Schema {
    /// Layouts or extras declared: the fighter has options beyond Tap jump.
    pub fn any(&self) -> bool {
        !self.layout.is_empty() || !self.extra.is_empty()
    }
}

/// The string as C's `strcmp` / `strlen` see it: up to the first NUL.
fn cstr(b: &[u8]) -> &[u8] {
    &b[..b.iter().position(|&c| c == 0).unwrap_or(b.len())]
}

/// gn_controls_button
fn button(name: &[u8]) -> Option<usize> {
    let name = cstr(name);
    LABELS.iter().position(|l| l.as_bytes() == name)
}

/// gn_controls_dest: one name, "none" (consume the source) or a nonempty array of distinct names
fn dest(d: &Doc, node: i32) -> Option<u16> {
    let n = &d.n[node as usize];
    if n.ty == Ty::Str {
        if cstr(&n.str) == b"none" {
            return Some(0);
        }
        return button(&n.str).map(|s| BITS[s]);
    }
    if n.ty != Ty::Arr || n.count == 0 || n.count as usize > BUTTONS {
        return None;
    }
    let mut value = 0u16;
    for c in d.kids(node) {
        let k = &d.n[c as usize];
        let s = if k.ty == Ty::Str { button(&k.str)? } else { return None };
        if value & BITS[s] != 0 {
            return None;
        }
        value |= BITS[s];
    }
    Some(value)
}

/// gn_controls_row: exactly `name` (1-31 printable bytes, no edge spaces) and a nonempty `buttons` object
fn row(d: &Doc, node: i32) -> Option<Map> {
    let n = &d.n[node as usize];
    if n.ty != Ty::Obj || n.count != 2 {
        return None;
    }
    let (name, buttons) = (d.get(node, "name"), d.get(node, "buttons"));
    if !d.is(name, Ty::Str) || !d.is(buttons, Ty::Obj) {
        return None;
    }
    let bn = &d.n[buttons as usize];
    if bn.count == 0 || bn.count as usize > BUTTONS {
        return None;
    }
    let s = cstr(&d.n[name as usize].str);
    if s.is_empty() || s.len() >= NAME || s[0] == b' ' || s[s.len() - 1] == b' ' || s.iter().any(|&c| c < 32 || c == 127) {
        return None;
    }
    let mut m = Map { name: String::from_utf8_lossy(s).into_owned(), ..Default::default() };
    for c in d.kids(buttons) {
        let src = button(&d.n[c as usize].key)?;
        if m.specified & (1 << src) != 0 {
            return None;
        }
        m.dest[src] = dest(d, c)?;
        m.specified |= 1 << src;
    }
    Some(m)
}

/// gn_controls_parse: the whole block or nothing (a malformed block falls back to no layouts / no extras, never
/// shifting row indexes). Unknown fields, a repeated field, bad types, exceeded limits and duplicate names refuse it.
pub fn parse(d: &Doc, node: i32) -> Option<Schema> {
    if !d.is(node, Ty::Obj) {
        return None;
    }
    let mut s = Schema::default();
    let mut seen = 0u32;
    for field in d.kids(node) {
        let key = d.n[field as usize].key.as_slice();
        let (layouts, extras) = (key == b"layouts", key == b"extras");
        let (limit, bit) = if layouts { (MAX_LAYOUTS, 1) } else { (MAX_EXTRAS, 2) };
        if (!layouts && !extras) || !d.is(field, Ty::Arr) || d.n[field as usize].count as usize > limit || seen & bit != 0 {
            return None;
        }
        seen |= bit;
        let rows = if layouts { &mut s.layout } else { &mut s.extra };
        for c in d.kids(field) {
            let m = row(d, c)?;
            if rows.iter().any(|r| r.name == m.name) {
                return None;
            }
            rows.push(m);
        }
    }
    Some(s)
}

/// gw_fc_normalize: an unknown layout is layout 0; extras beyond the declared ones and bits outside the word drop
pub fn normalize(s: Option<&Schema>, choice: u32) -> u32 {
    let tap = choice & TAP_MASK;
    let mut layout = (choice & LAYOUT_MASK) >> 2;
    let extras = s.map_or(0, |s| if s.extra.is_empty() { 0 } else { (1u32 << s.extra.len()) - 1 });
    if s.is_none_or(|s| layout as usize >= s.layout.len()) {
        layout = 0;
    }
    tap | (layout << 2) | (choice & (extras << 8))
}

/// gw_fc_overlay
fn overlay(out: &mut [u16; BUTTONS], m: &Map) {
    for i in 0..BUTTONS {
        if m.specified & (1 << i) != 0 {
            out[i] = m.dest[i];
        }
    }
}

/// gw_fc_build: the identity, then the chosen layout, then each enabled extra in declaration order (the last one
/// binding a source wins). Tap jump is not a button map: the game-side jump gate reads it.
pub fn build(s: Option<&Schema>, choice: u32) -> [u16; BUTTONS] {
    let choice = normalize(s, choice);
    let layout = ((choice & LAYOUT_MASK) >> 2) as usize;
    let mut out = BITS;
    if let Some(s) = s {
        if !s.layout.is_empty() {
            overlay(&mut out, &s.layout[layout]);
        }
        for (i, e) in s.extra.iter().enumerate() {
            if choice & (1 << (8 + i)) != 0 {
                overlay(&mut out, e);
            }
        }
    }
    out
}

/// gw_fc_apply: every mapping reads the raw input once (X -> Y, Y -> A cannot emit A from X)
pub fn apply(map: &[u16; BUTTONS], buttons: i32) -> i32 {
    let raw = buttons as u32;
    let mut out = raw & !BUTTON_MASK;
    for i in 0..BUTTONS {
        if raw & BITS[i] as u32 != 0 {
            out |= map[i] as u32;
        }
    }
    out as i32
}
