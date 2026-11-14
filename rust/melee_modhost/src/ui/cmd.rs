//! gd-ui-v1 A1: the per-frame command buffer (`gd_ui_submit`) and the batch it becomes.
//!
//! One linear scan validates and converts a mod's buffer straight into [`UiQuad`]s; the first bad record refuses the
//! whole buffer (the batch is rolled back to where it stood). Records are `u16 op, u16 words` + `words` payload words,
//! little-endian, 4-byte aligned (docs/specs/gd-ui-lab-v1.h). Quads are grouped in runs of one (layer, z); at the end of
//! the frame the runs are sorted on (layer, z, mod order, submission index), a total order, and copied into the frame.
//! Every vector here is reused frame to frame: nothing allocates once the high-water mark is reached.

use melee_gx::ui::{self as gu, Atlas, AtlasRect, UiFrame, UiQuad, NO_CLIP, Q_IN_B, Q_IN_L, Q_IN_R, Q_IN_T, Q_SDF, Q_SHAPE_MASK, Q_SHAPE_SHIFT, Q_WINDOW};
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
/// Effect for the quads that follow (envoy-fx.md): effect id (`melee_gx::ui::FX_*`), colour RGBA; `0, 0` ends it.
pub const OP_EFFECT: u16 = 12;
/// gd-ui-v1.4 (uiquality): one analytic shape, exact at every size, resolution, rotation and tilt. Payload: kind
/// (`melee_gx::ui::SHAPE_*`), x, y, w, h, corner radius, border width, glow spread, fill RGBA, border RGBA.
pub const OP_SHAPE: u16 = 13;
/// gd-ui-v1.4: the text that follows gets an outline of `width` canvas units in `rgba` (drawn from the glyphs' own
/// distance field, under the fill); width 0 ends it. Payload width, rgba.
pub const OP_OUTLINE: u16 = 14;
/// gd-ui-v1.4: the BOX shapes that follow carry a hard shadow in the same quad: their outline shrunk by `inset`,
/// moved by (dx, dy), in `rgba` (its alpha is the shape's: they fade as one). Payload dx, dy, inset, rgba; dx = dy = 0
/// ends it.
pub const OP_SHADOW: u16 = 15;

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

/// How many earlier quads a slanted or rotated quad is checked against for shared sides (a 9-slice is 9 pieces).
const JOIN_LOOKBACK: usize = 12;

/// Bits 8..15 of a quad's flags: how many instances back the first piece of its shape is (0: it is the first). The
/// menu tilt puts each instance at its own parallax depth; the pieces of one shape take their first piece's depth so
/// the shared sides stay shared under the perspective (uiseams2).
const GROUP_SHIFT: u32 = 8;
const GROUP_MASK: u32 = 0xFF << GROUP_SHIFT;

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
    /// OP_EFFECT / a node's `gd_ui_node_effect`: [effect id, colour] for the quads that follow
    pub fx: [u32; 2],
    /// OP_OUTLINE: (width in canvas units, colour) for the text that follows; width 0: none
    pub outline: (f32, u32),
    /// OP_SHADOW: (dx, dy, inset, colour) for the BOX shapes that follow; dx = dy = 0: none
    pub shadow: (f32, f32, f32, u32),
    clips: [[f32; 4]; CLIP_DEPTH],
    depth: usize,
}

impl Pen {
    pub fn new(module: u16) -> Pen {
        Pen { module, layer: LAYER_HUD, z: 0, window: false, scale: 1.0, rot: 0.0, piv: [0.0; 2], fx: [0; 2], outline: (0.0, 0), shadow: (0.0, 0.0, 0.0, 0), clips: [NO_CLIP; CLIP_DEPTH], depth: 0 }
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
    /// `shape` / `rgba2` of the quads pushed next (a shape's parameters; a glyph outline pass's widening).
    extra_shape: ([f32; 3], u32),
    pub frame: UiFrame,
    /// ui_aspect (MELEE_UI_ASPECT=1): what was drawn out of proportion, for the headless aspect check.
    pub audit: Option<Box<Audit>>,
}

/// The aspect audit (tools/rust-port/envoy/ui_aspect.sh; notes/ui-remake.md). The band's pixels are square and the
/// host never scales text sideways, so a piece can only come out squished when a mod asks for it: a sprite drawn
/// into a box of another shape than its art, a round shape that is not quite round, a line cut at its width. With
/// MELEE_UI_ASPECT=1 every such draw is logged once (`ui_aspect: ..`), the bad ones marked `BAD`.
#[derive(Default)]
pub struct Audit {
    /// atlas origin of a loaded texture -> the name the mod loaded it by
    pub names: std::collections::HashMap<(u16, u16), String>,
    seen: std::collections::HashSet<u64>,
    pub out: Vec<String>,
    /// the boxes of this buffer so far (x, y, w, h, rotation), newest last: what a text may sit on
    boxes: Vec<[f32; 5]>,
    /// the window's left and right edges in band units (set before a buffer is encoded); None: unknown
    pub view: Option<(f32, f32)>,
    /// the lines of this buffer so far (x, y, w, h, rotation, layer key) and their words
    texts: Vec<([f32; 5], u64, String)>,
    /// the sprites of this buffer so far (x, y, w, h, rotation), their layer key and atlas origin
    sprites: Vec<([f32; 5], u64, (u16, u16))>,
}

/// A sprite's drawn width / height may differ from its art's by this much (the check's 2%).
pub const ASPECT_TOL: f32 = 0.02;

impl Audit {
    pub fn from_env() -> Option<Box<Audit>> {
        std::env::var("MELEE_UI_ASPECT").is_ok_and(|v| v == "1").then(Box::default)
    }
    fn once(&mut self, k: impl std::hash::Hash) -> bool {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        k.hash(&mut h);
        self.seen.insert(h.finish())
    }
    /// How far a (w, h) draw of the `uv` part of art `r` is from the art's own proportions (0: exact).
    pub fn tex_off(r: AtlasRect, w: f32, h: f32, uv: [f32; 4]) -> Option<(f32, f32, f32)> {
        let (aw, ah) = ((uv[2] - uv[0]).abs() * r.w as f32, (uv[3] - uv[1]).abs() * r.h as f32);
        if aw <= 0.0 || ah <= 0.0 || w <= 0.0 || h <= 0.0 {
            return None;
        }
        Some(((w / h) / (aw / ah) - 1.0, aw, ah))
    }
    /// Two sprites of one layer that cross (neither inside the other) over more than an eighth of the smaller one:
    /// icons packed closer than they are wide. Sprites larger than 120 units (backdrops, rays) are not looked at.
    fn tex_over(&mut self, r: AtlasRect, x: f32, y: f32, w: f32, h: f32, rot: f32, key: u64) {
        if w > 120.0 || h > 120.0 || w < 4.0 || h < 4.0 {
            return;
        }
        let me = [x, y, w, h, rot];
        let mut hit = None;
        for (o, k, at) in self.sprites.iter() {
            if *k != key || o[4] != rot {
                continue;
            }
            let (ix, iy) = ((x + w).min(o[0] + o[2]) - x.max(o[0]), (y + h).min(o[1] + o[3]) - y.max(o[1]));
            if ix <= 0.0 || iy <= 0.0 {
                continue;
            }
            let inside = |a: &[f32; 5], b: &[f32; 5]| a[0] >= b[0] - 1.0 && a[1] >= b[1] - 1.0 && a[0] + a[2] <= b[0] + b[2] + 1.0 && a[1] + a[3] <= b[1] + b[3] + 1.0;
            if inside(&me, o) || inside(o, &me) {
                continue;
            }
            let small = (w * h).min(o[2] * o[3]);
            if ix * iy > small * 0.125 {
                hit = Some((*at, ix, iy, o[2], o[3]));
                break;
            }
        }
        if let Some((at, ix, iy, ow, oh)) = hit {
            if self.once((6u8, r.x, r.y, at, (ix * 2.0) as i32, (iy * 2.0) as i32)) {
                let name = |k: (u16, u16)| self.names.get(&k).cloned().unwrap_or_else(|| "?".into());
                let line = format!("ui_aspect: sprite over sprite by {ix:.1}x{iy:.1}: {} ({w:.1}x{h:.1}) over {} ({ow:.1}x{oh:.1}) BAD", name((r.x, r.y)), name(at));
                self.out.push(line);
            }
        }
        if self.sprites.len() < 2048 {
            self.sprites.push((me, key, (r.x, r.y)));
        }
    }
    fn tex(&mut self, r: AtlasRect, w: f32, h: f32, uv: [f32; 4]) {
        let Some((off, aw, ah)) = Audit::tex_off(r, w, h, uv) else { return };
        if !self.once((0u8, r.x, r.y, uv.map(f32::to_bits), (off * 400.0).round() as i32)) {
            return;
        }
        let name = self.names.get(&(r.x, r.y)).map_or("?", |s| s.as_str());
        let bad = if off.abs() > ASPECT_TOL { " BAD" } else { "" };
        self.out.push(format!("ui_aspect: tex {name} art {aw:.0}x{ah:.0} drawn {w:.1}x{h:.1} off {:+.1}%{bad}", off * 100.0));
    }
    /// A box whose corner radius makes its short side fully round and whose sides differ by 2% to 25% is a circle
    /// that is not round (or a pill too short to read as one).
    pub fn oval(w: f32, h: f32, radius: f32) -> bool {
        let (lo, hi) = if w < h { (w, h) } else { (h, w) };
        radius * 2.0 >= lo - 0.01 && hi / lo > 1.0 + ASPECT_TOL && hi / lo < 1.25
    }
    fn shape(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, rot: f32, border: f32) {
        if self.boxes.len() >= 4096 {
            self.boxes.clear();
        }
        // a sticker (a box with an edge) is something words sit on; a plain fill (a bar, a meter's fill) is not
        if border > 0.0 {
            self.boxes.push([x, y, w, h, rot]);
        }
        if Audit::oval(w, h, radius) && self.once((1u8, (w * 4.0) as i32, (h * 4.0) as i32)) {
            self.out.push(format!("ui_aspect: shape round {w:.1}x{h:.1} off {:+.1}% BAD", (w / h - 1.0) * 100.0));
        }
    }
    /// A line that starts on a box (the newest one under its first glyph, in the same rotation) and runs past that
    /// box's side: words spilling out of their sticker. `x0`, `w`: the line's left end and width; `size` its height.
    #[allow(clippy::too_many_arguments)]
    fn text_fit(&mut self, s: &str, x0: f32, y: f32, w: f32, size: f32, rot: f32, band: bool, key: u64, alpha: u32) {
        if w <= 0.0 || s.trim().is_empty() {
            return;
        }
        // a line set over an earlier line of the same layer (their ink boxes cross): words on words
        let me = [x0, y + size * 0.3, w, size * 0.62, rot];
        if alpha > 40 {
            let hit = self.texts.iter().find_map(|(o, k, os)| {
                let (ix, iy) = ((me[0] + me[2]).min(o[0] + o[2]) - me[0].max(o[0]), (me[1] + me[3]).min(o[1] + o[3]) - me[1].max(o[1]));
                (*k == key && o[4] == rot && ix > 2.0 && iy > size.min(o[3] / 0.62) * 0.3 && os != s).then(|| (ix, iy, os.clone()))
            });
            if let Some((ix, iy, os)) = hit {
                if self.once((5u8, s, os.as_str())) {
                    self.out.push(format!("ui_aspect: text over text by {ix:.1}x{iy:.1}: \"{s}\" (size {size:.0}) over \"{os}\" BAD"));
                }
            }
            if self.texts.len() < 2048 {
                self.texts.push((me, key, s.to_string()));
            }
        }
        // a line that leaves the window (band space, unrotated): words nobody can read
        if let Some((l, r)) = self.view.filter(|_| band && rot == 0.0) {
            let over = (l - x0).max(x0 + w - r);
            if over > 0.5 && self.once((4u8, s)) {
                self.out.push(format!("ui_aspect: text runs off the screen by {over:.1}: \"{s}\" (size {size:.0}, from {x0:.1} to {:.1}, the screen {l:.1} to {r:.1}) BAD", x0 + w));
            }
        }
        let (px, py) = (x0 + size.min(w) * 0.3, y + size * 0.6);
        let Some(b) = self.boxes.iter().rev().find(|b| b[4] == rot && px >= b[0] && px <= b[0] + b[2] && py >= b[1] && py <= b[1] + b[3]).copied() else { return };
        let over = (x0 + w) - (b[0] + b[2]);
        if over > 0.5 && self.once((3u8, s, (b[2] * 2.0) as i32)) {
            self.out.push(format!("ui_aspect: text overflows its box by {over:.1}: \"{s}\" (size {size:.0}, width {w:.1}) on a {:.0}x{:.0} box BAD", b[2], b[3]));
        }
    }
    fn text_cut(&mut self, s: &str, n: usize, max_w: f32, size: f32) {
        if n < s.chars().count() && self.once((2u8, s, max_w.to_bits())) {
            self.out.push(format!("ui_aspect: text cut at {n} of {} chars, width {max_w:.0}, size {size:.0}: \"{s}\" BAD", s.chars().count()));
        }
    }
}

fn f32_ok(v: f32) -> bool {
    v.is_finite() && v.abs() < 1.0e7
}

impl Batch {
    pub fn with_capacity(quads: usize) -> Batch {
        Batch { staging: Vec::with_capacity(quads), runs: Vec::with_capacity(256), seq: 0, extra_flags: 0, extra_shape: ([0.0; 3], 0), frame: UiFrame { quads: Vec::with_capacity(quads), hide_gx: false }, audit: Audit::from_env() }
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
        let (shape, rgba2) = self.extra_shape;
        self.staging.push(UiQuad { rect: [rect[0] * s, rect[1] * s, rect[2] * s, rect[3] * s], uv, clip: p.clip(), rgba, shear, flags: p.flags() | self.extra_flags, rot: p.rot, piv: [p.piv[0] * s, p.piv[1] * s], fx: if self.extra_flags & Q_SDF != 0 { [0; 2] } else { p.fx }, shape, rgba2 });
        self.runs.last_mut().unwrap().len += 1;
        // glyphs and shapes are whole on their own: only flat and textured pieces are joined
        if self.extra_flags & (Q_SDF | Q_SHAPE_MASK | gu::Q_FULL) == 0 {
            self.join_last();
        }
    }

    /// uiseams: a slanted or rotated quad gets edge AA on its sides, so the pieces of one shape (a 9-slice, a tiled
    /// bar, a mod's own sprite slices) would each fade at the shared side and leave a see-through seam. Mark every
    /// side the new quad shares exactly with a recent quad of the same transform as inner on both, for the shader to
    /// leave alone. Only the last few quads of the open run are looked at: the pieces of one shape are pushed together.
    fn join_last(&mut self) {
        let n = self.staging.len();
        let start = self.runs.last().map_or(n, |r| r.start as usize).max(n.saturating_sub(JOIN_LOOKBACK + 1));
        let (head, tail) = self.staging.split_at_mut(n - 1);
        let q = &mut tail[0];
        let mut root = 0usize;
        for (k, o) in head[start..].iter_mut().enumerate().rev() {
            let back = n - 1 - (start + k);
            if o.rot != q.rot || o.piv != q.piv || o.shear != q.shear || o.clip != q.clip || (o.flags ^ q.flags) & (Q_WINDOW | Q_SDF) != 0 || (o.flags | q.flags) & (Q_SDF | Q_SHAPE_MASK | gu::Q_FULL) != 0 {
                continue;
            }
            let (a, b) = (o.rect, q.rect);
            let near = |x: f32, y: f32| (x - y).abs() <= 1.0e-3 * (1.0 + x.abs().max(y.abs()));
            let same_rows = near(a[1], b[1]) && near(a[3], b[3]);
            // the top side of a sheared quad starts shear * h to the right of its bottom side
            let same_cols = |top: [f32; 4], bot: [f32; 4]| near(top[0] + q.shear * top[3], bot[0]) && near(top[2], bot[2]);
            if same_rows && near(a[0] + a[2], b[0]) {
                o.flags |= Q_IN_R;
                q.flags |= Q_IN_L;
                root = root.max(back + ((o.flags & GROUP_MASK) >> GROUP_SHIFT) as usize);
            } else if same_rows && near(b[0] + b[2], a[0]) {
                o.flags |= Q_IN_L;
                q.flags |= Q_IN_R;
                root = root.max(back + ((o.flags & GROUP_MASK) >> GROUP_SHIFT) as usize);
            } else if near(a[1] + a[3], b[1]) && same_cols(b, a) {
                o.flags |= Q_IN_B;
                q.flags |= Q_IN_T;
                root = root.max(back + ((o.flags & GROUP_MASK) >> GROUP_SHIFT) as usize);
            } else if near(b[1] + b[3], a[1]) && same_cols(a, b) {
                o.flags |= Q_IN_T;
                q.flags |= Q_IN_B;
                root = root.max(back + ((o.flags & GROUP_MASK) >> GROUP_SHIFT) as usize);
            }
        }
        if root > 0 && root <= 0xFF {
            q.flags = (q.flags & !GROUP_MASK) | ((root as u32) << GROUP_SHIFT);
        }
    }

    pub fn rect(&mut self, p: &Pen, a: &Atlas, x: f32, y: f32, w: f32, h: f32, rgba: u32, shear: f32) {
        if w <= 0.0 || h <= 0.0 || rgba & 0xFF == 0 {
            return;
        }
        self.push(p, [x, y, w, h], a.white.uv(), rgba, shear);
    }

    pub fn tex(&mut self, p: &Pen, r: AtlasRect, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], rgba: u32) {
        let (tw, th) = (r.w as f32, r.h as f32);
        if let Some(a) = self.audit.as_deref_mut() {
            a.tex(r, w, h, uv);
            if rgba & 0xFF > 40 {
                a.tex_over(r, x, y, w, h, p.rot, Batch::key(p));
            }
        }
        // a whole entry (not a cell of a sheet or a slice of it): its edge may fade into the atlas gutter
        let save = self.extra_flags;
        if uv == [0.0, 0.0, 1.0, 1.0] {
            self.extra_flags |= gu::Q_FULL;
        }
        self.push(p, [x, y, w, h], r.sub_uv(uv[0] * tw, uv[1] * th, (uv[2] - uv[0]) * tw, (uv[3] - uv[1]) * th), rgba, 0.0);
        self.extra_flags = save;
    }

    /// One analytic shape (`melee_gx::ui::SHAPE_*`) over (x, y, w, h): `radius`, `border` and `spread` in the pen's
    /// units, `rgba` the fill, `rgba2` the border. One quad, whatever its size: there are no pieces to come apart.
    pub fn shape(&mut self, p: &Pen, kind: u32, x: f32, y: f32, w: f32, h: f32, radius: f32, border: f32, spread: f32, rgba: u32, rgba2: u32) {
        if w <= 0.0 || h <= 0.0 || (rgba & 0xFF == 0 && (border <= 0.0 || rgba2 & 0xFF == 0)) {
            return;
        }
        let s = p.scale;
        if kind == gu::SHAPE_BOX {
            if let Some(a) = self.audit.as_deref_mut() {
                a.shape(x, y, w, h, radius, p.rot, border);
            }
        }
        // a glow reaches `spread` outside the shape
        let g = if kind == gu::SHAPE_GLOW { spread.max(0.0) } else { 0.0 };
        let save = (self.extra_flags, self.extra_shape);
        self.extra_flags |= kind << Q_SHAPE_SHIFT;
        // shape[2]: a glow's spread, a box's dash length
        self.extra_shape = ([radius.max(0.0) * s, border.max(0.0) * s, spread.max(0.0) * s], rgba2);
        // a box's hard shadow rides in the quad's unused uv: dx, dy, inset, 0xRRGGBB as a number (exact in an f32)
        let sh = p.shadow;
        let uv = if kind == gu::SHAPE_BOX && (sh.0 != 0.0 || sh.1 != 0.0) { [sh.0 * s, sh.1 * s, sh.2.max(0.0) * s, (sh.3 >> 8) as f32] } else { [0.0; 4] };
        self.push(p, [x - g, y - g, w + 2.0 * g, h + 2.0 * g], uv, rgba, 0.0);
        (self.extra_flags, self.extra_shape) = save;
    }

    /// A slice piece over (x, y, w, h) (`GD_UI_SLICE_*`): a rounded fill, a rounded border or the glow, as one shape.
    pub fn slice(&mut self, p: &Pen, _a: &Atlas, id: usize, x: f32, y: f32, w: f32, h: f32, rgba: u32) {
        let d = gu::SLICES[id];
        if d.glow > 0.0 {
            self.shape(p, gu::SHAPE_GLOW, x, y, w, h, d.radius, 0.0, d.glow, rgba, 0);
        } else if d.border > 0.0 {
            self.shape(p, gu::SHAPE_BOX, x, y, w, h, d.radius, d.border, 0.0, rgba & !0xFF, rgba);
        } else {
            self.shape(p, gu::SHAPE_BOX, x, y, w, h, d.radius, 0.0, 0.0, rgba, 0);
        }
    }

    /// A built-in kit piece (`gd_ui_kit_id`): the round ones are shapes, the hatch a bitmap.
    pub fn kit(&mut self, p: &Pen, a: &Atlas, id: usize, x: f32, y: f32, scale: f32, rgba: u32) {
        let (name, pw, ph) = gu::KIT_PIECES[id];
        let (w, h) = (pw as f32 * scale, ph as f32 * scale);
        match name {
            "ui.dot" | "ui.knob" => self.shape(p, gu::SHAPE_BOX, x, y, w, h, w, 0.0, 0.0, rgba, 0),
            "ui.ring" => self.shape(p, gu::SHAPE_BOX, x, y, w, h, w, 2.5 * scale, 0.0, rgba & !0xFF, rgba),
            "ui.diamond" => self.shape(p, gu::SHAPE_DIAMOND, x, y, w, h, 0.0, 0.0, 0.0, rgba, 0),
            _ => self.push(p, [x, y, w, h], a.kit[id].uv(), rgba, 0.0),
        }
    }

    /// Text at (x, y) (top of the line; `align` anchors x), cut at `max_w` (0: no limit). `role` carries the
    /// gd-ui-v1.1 style bits ([`style`]).
    pub fn text(&mut self, p: &Pen, a: &Atlas, x: f32, y: f32, role: i32, rgba: u32, align: i32, max_w: f32, s: &str) {
        let st = style(role);
        let (n, w) = gu_font::fit(&a.faces, st, s, max_w);
        let x0 = match align { ALIGN_CENTER => x - w * 0.5, ALIGN_RIGHT => x - w, _ => x };
        if let Some(au) = self.audit.as_deref_mut() {
            au.text_cut(s, n, max_w, st.size);
            au.text_fit(s, x0, y, w, st.size, p.rot, !p.window && p.scale == 1.0, Batch::key(p), rgba & 0xFF);
        }
        let save = (self.extra_flags, self.extra_shape);
        self.extra_flags |= Q_SDF;
        if p.outline.0 > 0.0 && p.outline.1 & 0xFF != 0 {
            // the outline pass: the same glyphs, widened in their distance field by the outline width (capped where
            // the field ends), all under the fill pass so a neighbour's outline never covers a letter
            let widen = (p.outline.0 * gu_font::SDF_EM / st.size.max(1.0) / (2.0 * gu_font::SDF_RANGE)).min(0.42);
            self.extra_shape = ([0.0, widen, 0.0], 0);
            let col = (p.outline.1 & !0xFF) | ((p.outline.1 & 0xFF) * (rgba & 0xFF) / 255);
            gu_font::emit(&a.faces, st, s, n, x0, y, |r, uv| self.push(p, r, uv, col, 0.0));
        }
        self.extra_shape = ([0.0; 3], 0);
        gu_font::emit(&a.faces, st, s, n, x0, y, |r, uv| self.push(p, r, uv, rgba, 0.0));
        (self.extra_flags, self.extra_shape) = save;
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
        if let Some(au) = self.audit.as_deref_mut() {
            au.boxes.clear();
            au.texts.clear();
            au.sprites.clear();
        }
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
                OP_EFFECT => {
                    need(2)?;
                    if w(p) >= gu::FX_COUNT {
                        return err(at, "effect id");
                    }
                    pen.fx = [w(p), w(p + 4)];
                }
                OP_SHAPE => {
                    need(10)?;
                    let kind = w(p);
                    if !(gu::SHAPE_BOX..=gu::SHAPE_GLOW).contains(&kind) {
                        return err(at, "shape: kind");
                    }
                    if !(1..8).all(|i| f32_ok(f(p + i * 4))) {
                        return err(at, "shape: non-finite");
                    }
                    self.shape(&pen, kind, f(p + 4), f(p + 8), f(p + 12), f(p + 16), f(p + 20), f(p + 24), f(p + 28), w(p + 32), w(p + 36));
                }
                OP_SHADOW => {
                    need(4)?;
                    if !fs(3) {
                        return err(at, "shadow: non-finite");
                    }
                    pen.shadow = (f(p), f(p + 4), f(p + 8), w(p + 12));
                }
                OP_OUTLINE => {
                    need(2)?;
                    if !f32_ok(f(p)) || f(p) < 0.0 {
                        return err(at, "outline: width");
                    }
                    pen.outline = (f(p), w(p + 4));
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
