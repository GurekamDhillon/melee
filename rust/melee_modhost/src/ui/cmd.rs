//! gd-ui-v1 A1: the per-frame command buffer (`gd_ui_submit`) and the batch it becomes.
//!
//! One linear scan validates and converts a mod's buffer straight into [`UiQuad`]s; the first bad record refuses the
//! whole buffer (the batch is rolled back to where it stood). Records are `u16 op, u16 words` + `words` payload words,
//! little-endian, 4-byte aligned (docs/specs/gd-ui-lab-v1.h). Quads are grouped in runs of one (layer, z); at the end of
//! the frame the runs are sorted on (layer, z, mod order, submission index), a total order, and copied into the frame.
//! Every vector here is reused frame to frame: nothing allocates once the high-water mark is reached.

use melee_gx::ui::{self as gu, Atlas, AtlasRect, UiFrame, UiQuad, NO_CLIP, Q_SDF, Q_WINDOW};
use melee_gx::ui_font::{self as gu_font, TextStyle};

pub const OP_LAYER: u16 = 1;
pub const OP_RECT: u16 = 2;
pub const OP_TEX: u16 = 3;
pub const OP_TEXT: u16 = 4;
pub const OP_KIT: u16 = 5;
pub const OP_CLIP: u16 = 6;
pub const OP_UNCLIP: u16 = 7;
pub const OP_VECTOR: u16 = 8;
pub const OP_SLICE: u16 = 9;
pub const OP_SPACE: u16 = 10;
/// envoy-v3-ui.md: rotate what follows by `a` radians about (x, y); 0 ends it. Payload a, x, y.
pub const OP_ROT: u16 = 11;

pub const LAYER_SCENE: i32 = 0;
pub const LAYER_HUD_UNDER: i32 = 1;
pub const LAYER_HUD: i32 = 2;
pub const LAYER_OVERLAY: i32 = 3;
pub const LAYER_MENU: i32 = 4;
pub const LAYER_MODAL: i32 = 5;
pub const LAYER_DEBUG: i32 = 6;

pub const SPACE_BAND: i32 = 0;
pub const SPACE_WINDOW: i32 = 1;

pub const ALIGN_LEFT: i32 = 0;
pub const ALIGN_CENTER: i32 = 1;
pub const ALIGN_RIGHT: i32 = 2;

/// Text roles: pixel size at scale 1 (base size 13: label fs-2, value fs-1, body fs, big fs+5, title, toast) and bold.
pub const ROLES: [(f32, bool); 6] = [(11.0, false), (12.0, false), (13.0, false), (18.0, true), (14.0, true), (13.0, true)];
pub const ROLE_LABEL: i32 = 0;

/// gd-ui-v1.1 text style bits in the `role` word (GD_UI_TEXT_* in gd-ui-lab-v1.h): bits 0-7 role, 8-9 weight
/// (0 role default, 1 regular, 2 bold), bit 10 monospace, bits 16-23 pixel size (0: the role's).
pub const TEXT_REGULAR: i32 = 1 << 8;
pub const TEXT_BOLD: i32 = 2 << 8;
pub const TEXT_MONO: i32 = 1 << 10;
/// Envoy V3 faces: bit 11 Bricolage Grotesque ExtraBold, bit 12 Source Sans 3 Bold (they win over weight / mono).
pub const TEXT_DISPLAY: i32 = 1 << 11;
pub const TEXT_BODY: i32 = 1 << 12;

/// The resolved style of a `role` word. v1.0 roles (0..5) resolve as before; out-of-range roles clamp.
pub fn style(role: i32) -> TextStyle {
    let r = (role & 0xFF).min(ROLES.len() as i32 - 1);
    let (mut size, mut bold) = ROLES[r as usize];
    match (role >> 8) & 3 {
        1 => bold = false,
        2 => bold = true,
        _ => {}
    }
    let px = (role >> 16) & 0xFF;
    if px > 0 {
        size = px as f32;
    }
    let mut st = TextStyle::new(size, bold, role & TEXT_MONO != 0);
    st.upper = r == ROLE_LABEL;
    if role & TEXT_DISPLAY != 0 {
        st.face = gu_font::FACE_DISPLAY;
        st.upper = false;
    } else if role & TEXT_BODY != 0 {
        st.face = gu_font::FACE_BODY;
        st.upper = false;
    }
    st
}

pub const CLIP_DEPTH: usize = 8;

/// Why a buffer was refused: the byte offset of the record and the check.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdError {
    pub at: usize,
    pub what: &'static str,
    pub code: i32,
}

#[derive(Clone, Copy, Debug)]
struct Run {
    key: u64,
    seq: u32,
    start: u32,
    len: u32,
}

/// Drawing state while one mod's records (or the host's own node drawing) are encoded.
#[derive(Clone, Copy)]
pub struct Pen {
    pub module: u16,
    pub layer: i32,
    pub z: i32,
    pub window: bool,
    pub scale: f32,
    /// OP_ROT: rotation (radians) about `piv` for the quads that follow
    pub rot: f32,
    pub piv: [f32; 2],
    clips: [[f32; 4]; CLIP_DEPTH],
    depth: usize,
}

impl Pen {
    pub fn new(module: u16) -> Pen {
        Pen { module, layer: LAYER_HUD, z: 0, window: false, scale: 1.0, rot: 0.0, piv: [0.0; 2], clips: [NO_CLIP; CLIP_DEPTH], depth: 0 }
    }
    fn clip(&self) -> [f32; 4] {
        if self.depth == 0 { NO_CLIP } else { self.clips[self.depth - 1] }
    }
    fn flags(&self) -> u32 {
        if self.window { Q_WINDOW } else { 0 }
    }
}

/// The frame's quads, in runs, and the sorted output.
#[derive(Default)]
pub struct Batch {
    staging: Vec<UiQuad>,
    runs: Vec<Run>,
    seq: u32,
    /// Instance flags OR-ed onto every pushed quad (Q_SDF while a text run is emitted).
    extra_flags: u32,
    pub frame: UiFrame,
}

fn f32_ok(v: f32) -> bool {
    v.is_finite() && v.abs() < 1.0e7
}

impl Batch {
    pub fn with_capacity(quads: usize) -> Batch {
        Batch { staging: Vec::with_capacity(quads), runs: Vec::with_capacity(256), seq: 0, extra_flags: 0, frame: UiFrame { quads: Vec::with_capacity(quads), hide_gx: false } }
    }

    pub fn begin(&mut self) {
        self.staging.clear();
        self.runs.clear();
        self.seq = 0;
    }

    /// Quads encoded so far this frame.
    pub fn len(&self) -> usize {
        self.staging.len()
    }
    pub fn is_empty(&self) -> bool {
        self.staging.is_empty()
    }

    fn key(p: &Pen) -> u64 {
        ((p.layer as u64) << 56) | (((p.z as u32 ^ 0x8000_0000) as u64) << 16) | p.module as u64
    }

    /// Start a run for the pen's (layer, z) unless the open run already is one.
    fn open(&mut self, p: &Pen) {
        let key = Batch::key(p);
        if let Some(r) = self.runs.last() {
            if r.key == key && r.start + r.len == self.staging.len() as u32 {
                return;
            }
        }
        self.runs.push(Run { key, seq: self.seq, start: self.staging.len() as u32, len: 0 });
        self.seq += 1;
    }

    fn push(&mut self, p: &Pen, rect: [f32; 4], uv: [f32; 4], rgba: u32, shear: f32) {
        self.open(p);
        let s = p.scale;
        self.staging.push(UiQuad { rect: [rect[0] * s, rect[1] * s, rect[2] * s, rect[3] * s], uv, clip: p.clip(), rgba, shear, flags: p.flags() | self.extra_flags, rot: p.rot, piv: [p.piv[0] * s, p.piv[1] * s], _pad: [0; 2] });
        self.runs.last_mut().unwrap().len += 1;
    }

    pub fn rect(&mut self, p: &Pen, a: &Atlas, x: f32, y: f32, w: f32, h: f32, rgba: u32, shear: f32) {
        if w <= 0.0 || h <= 0.0 || rgba & 0xFF == 0 {
            return;
        }
        self.push(p, [x, y, w, h], a.white.uv(), rgba, shear);
    }

    pub fn tex(&mut self, p: &Pen, r: AtlasRect, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], rgba: u32) {
        let (tw, th) = (r.w as f32, r.h as f32);
        self.push(p, [x, y, w, h], r.sub_uv(uv[0] * tw, uv[1] * th, (uv[2] - uv[0]) * tw, (uv[3] - uv[1]) * th), rgba, 0.0);
    }

    /// A 9-slice piece over (x, y, w, h): 4 corners at their texel size, 4 stretched edges, the centre.
    pub fn slice(&mut self, p: &Pen, a: &Atlas, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32) {
        let (t, c0) = a.slices[id];
        let glow = gu::SLICES[id].glow;
        // a glow tile reaches `glow` outside the shape
        let (x, y, w, h) = (x - glow, y - glow, w + 2.0 * glow, h + 2.0 * glow);
        let c = c0.min(w * 0.5).min(h * 0.5);
        if c <= 0.0 || rgba & 0xFF == 0 {
            return;
        }
        let n = t.w as f32; // 2*c0+1
        let xs = [x, x + c, x + w - c, x + w];
        let ys = [y, y + c, y + h - c, y + h];
        let us = [0.0, c, n - c, n];
        let uc = [0.0, c0, c0 + 1.0, n];
        let _ = us;
        for j in 0..3 {
            for i in 0..3 {
                let (qw, qh) = (xs[i + 1] - xs[i], ys[j + 1] - ys[j]);
                if qw <= 0.0 || qh <= 0.0 {
                    continue;
                }
                // corner texels taken from the tile's corners even when the piece is smaller than 2c0
                let (u0, u1) = match i { 0 => (0.0, c), 1 => (uc[1], uc[2]), _ => (n - c, n) };
                let (v0, v1) = match j { 0 => (0.0, c), 1 => (uc[1], uc[2]), _ => (n - c, n) };
                // the 1-texel middle strip is sampled at its centre so nearest filtering never bleeds
                let (u0, u1) = if i == 1 { (c0 + 0.25, c0 + 0.75) } else { (u0, u1) };
                let (v0, v1) = if j == 1 { (c0 + 0.25, c0 + 0.75) } else { (v0, v1) };
                self.push(p, [xs[i], ys[j], qw, qh], t.sub_uv(u0, v0, u1 - u0, v1 - v0), rgba, 0.0);
            }
        }
    }

    pub fn kit(&mut self, p: &Pen, a: &Atlas, id: usize, x: f32, y: f32, scale: f32, rgba: u32) {
        let r = a.kit[id];
        self.push(p, [x, y, r.w as f32 * scale, r.h as f32 * scale], r.uv(), rgba, 0.0);
    }

    /// Text at (x, y) (top of the line; `align` anchors x), cut at `max_w` (0: no limit). `role` carries the
    /// gd-ui-v1.1 style bits ([`style`]).
    pub fn text(&mut self, p: &Pen, a: &Atlas, x: f32, y: f32, role: i32, rgba: u32, align: i32, max_w: f32, s: &str) {
        let st = style(role);
        let (n, w) = gu_font::fit(&a.faces, st, s, max_w);
        let x0 = match align { ALIGN_CENTER => x - w * 0.5, ALIGN_RIGHT => x - w, _ => x };
        let (save, sdf) = (self.extra_flags, Q_SDF);
        self.extra_flags |= sdf;
        gu_font::emit(&a.faces, st, s, n, x0, y, |r, uv| self.push(p, r, uv, rgba, 0.0));
        self.extra_flags = save;
    }

    /// Sort the runs and lay the quads out in draw order in `frame.quads`.
    pub fn finish(&mut self, hide_gx: bool) {
        self.runs.sort_unstable_by_key(|r| (r.key, r.seq));
        let out = &mut self.frame.quads;
        out.clear();
        for r in &self.runs {
            out.extend_from_slice(&self.staging[r.start as usize..(r.start + r.len) as usize]);
        }
        self.frame.hide_gx = hide_gx;
    }

    /// Validate and encode one mod's buffer. `texs`: the mod's loaded textures (handle h is `texs[h - 1]`).
    pub fn submit(&mut self, buf: &[u8], module: u16, atlas: &Atlas, texs: &[AtlasRect], vector_ok: bool) -> Result<(), CmdError> {
        let (q0, r0, s0) = (self.staging.len(), self.runs.len(), self.seq);
        let r = self.submit_inner(buf, module, atlas, texs, vector_ok);
        if r.is_err() {
            self.staging.truncate(q0);
            self.runs.truncate(r0);
            self.seq = s0;
        }
        r
    }

    fn submit_inner(&mut self, buf: &[u8], module: u16, atlas: &Atlas, texs: &[AtlasRect], vector_ok: bool) -> Result<(), CmdError> {
        let err = |at: usize, what: &'static str| Err(CmdError { at, what, code: crate::pres::GD_EINVAL });
        if buf.len() % 4 != 0 {
            return err(0, "length not a multiple of 4");
        }
        let w = |o: usize| u32::from_le_bytes([buf[o], buf[o + 1], buf[o + 2], buf[o + 3]]);
        let f = |o: usize| f32::from_bits(w(o));
        let mut pen = Pen::new(module);
        let mut o = 0usize;
        while o < buf.len() {
            let head = w(o);
            let (op, words) = ((head & 0xFFFF) as u16, (head >> 16) as usize);
            let at = o;
            let p = o + 4;
            let end = p + words * 4;
            if end > buf.len() {
                return err(at, "record runs past the buffer");
            }
            let need = |n: usize| if words == n { Ok(()) } else { Err(CmdError { at, what: "payload size", code: crate::pres::GD_EINVAL }) };
            let fs = |n: usize| (0..n).all(|i| f32_ok(f(p + i * 4)));
            match op {
                OP_LAYER => {
                    need(2)?;
                    let layer = w(p) as i32;
                    if !(LAYER_SCENE..=LAYER_DEBUG).contains(&layer) {
                        return err(at, "layer id");
                    }
                    pen.layer = layer;
                    pen.z = w(p + 4) as i32;
                }
                OP_SPACE => {
                    need(2)?;
                    let sp = w(p) as i32;
                    let sc = f(p + 4);
                    if !(SPACE_BAND..=SPACE_WINDOW).contains(&sp) || !(sc > 0.0 && sc < 64.0) {
                        return err(at, "space id or scale");
                    }
                    if pen.depth != 0 {
                        return err(at, "space change inside a clip");
                    }
                    pen.window = sp == SPACE_WINDOW;
                    pen.scale = sc;
                }
                OP_ROT => {
                    need(3)?;
                    if !fs(3) {
                        return err(at, "rot: non-finite");
                    }
                    pen.rot = f(p);
                    pen.piv = [f(p + 4), f(p + 8)];
                }
                OP_RECT => {
                    need(6)?;
                    if !fs(4) || !f32_ok(f(p + 20)) {
                        return err(at, "rect: non-finite");
                    }
                    self.rect(&pen, atlas, f(p), f(p + 4), f(p + 8), f(p + 12), w(p + 16), f(p + 20));
                }
                OP_TEX => {
                    need(10)?;
                    let h = w(p) as usize;
                    if h == 0 || h > texs.len() {
                        return err(at, "tex: bad handle");
                    }
                    if !(1..9).all(|i| f32_ok(f(p + i * 4))) {
                        return err(at, "tex: non-finite");
                    }
                    let uv = [f(p + 20), f(p + 24), f(p + 28), f(p + 32)];
                    self.tex(&pen, texs[h - 1], f(p + 4), f(p + 8), f(p + 12), f(p + 16), uv, w(p + 36));
                }
                OP_TEXT => {
                    if words < 7 {
                        return err(at, "payload size");
                    }
                    let len = w(p + 24) as usize;
                    need(7 + len.div_ceil(4))?;
                    if !f32_ok(f(p)) || !f32_ok(f(p + 4)) || !f32_ok(f(p + 20)) {
                        return err(at, "text: non-finite");
                    }
                    let role = w(p + 8) as i32;
                    // v1.1: the role in bits 0-7, style bits 8-10 and the size in 16-23 (GD_UI_TEXT_*); nothing else
                    if !(0..ROLES.len() as i32).contains(&(role & 0xFF)) || role as u32 & !0x00FF_1FFF != 0 || (role >> 8) & 3 == 3 {
                        return err(at, "text: role");
                    }
                    let Ok(s) = core::str::from_utf8(&buf[p + 28..p + 28 + len]) else { return err(at, "text: not UTF-8") };
                    self.text(&pen, atlas, f(p), f(p + 4), role, w(p + 12), w(p + 16) as i32, f(p + 20), s);
                }
                OP_KIT => {
                    need(5)?;
                    let id = w(p) as usize;
                    if id == 0 || id >= gu::KIT_PIECES.len() {
                        return err(at, "kit: bad id");
                    }
                    if !fs(4) {
                        return err(at, "kit: non-finite");
                    }
                    self.kit(&pen, atlas, id, f(p + 4), f(p + 8), f(p + 12), w(p + 16));
                }
                OP_SLICE => {
                    need(6)?;
                    let id = w(p) as usize;
                    if id == 0 || id >= gu::SLICES.len() {
                        return err(at, "slice: bad id");
                    }
                    if !(1..5).all(|i| f32_ok(f(p + i * 4))) {
                        return err(at, "slice: non-finite");
                    }
                    self.slice(&pen, atlas, id, f(p + 4), f(p + 8), f(p + 12), f(p + 16), w(p + 20));
                }
                OP_CLIP => {
                    need(4)?;
                    if !fs(4) {
                        return err(at, "clip: non-finite");
                    }
                    if pen.depth == CLIP_DEPTH {
                        return err(at, "clip: stack depth 8");
                    }
                    let s = pen.scale;
                    let (x, y, cw, ch) = (f(p) * s, f(p + 4) * s, f(p + 8) * s, f(p + 12) * s);
                    let c = pen.clip();
                    pen.clips[pen.depth] = [x.max(c[0]), y.max(c[1]), (x + cw).min(c[2]), (y + ch).min(c[3])];
                    pen.depth += 1;
                }
                OP_UNCLIP => {
                    need(0)?;
                    if pen.depth == 0 {
                        return err(at, "unclip: stack empty");
                    }
                    pen.depth -= 1;
                }
                OP_VECTOR => {
                    need(8)?;
                    if !vector_ok {
                        return Err(CmdError { at, what: "vector: UI_VECTOR not declared (Vello is not in this build)", code: crate::pres::GD_ECAP });
                    }
                }
                _ => return err(at, "unknown op"),
            }
            o = end;
        }
        Ok(())
    }
}
