//! gd_ui_enc: the gd-ui-v1 command-buffer encoder for Rust wasm mods (docs/specs/gd-ui-lab-v1.h, A1).
//!
//! A mod writes `ui.rect(..)`, `ui.text(..)` into a fixed buffer in its own memory with no host call, then hands the
//! whole frame over once with [`Enc::submit`] (`gd_ui_submit`). No allocation: the buffer is a fixed array, and a
//! record that does not fit is dropped (and counted in `overflow`).
#![no_std]

pub const OP_LAYER: u16 = 1;
pub const OP_RECT: u16 = 2;
pub const OP_TEX: u16 = 3;
pub const OP_TEXT: u16 = 4;
pub const OP_KIT: u16 = 5;
pub const OP_CLIP: u16 = 6;
pub const OP_UNCLIP: u16 = 7;
pub const OP_SLICE: u16 = 9;
pub const OP_SPACE: u16 = 10;
/// Rotate the quads that follow by `a` radians (clockwise) about (x, y); `rot(0.0, ..)` ends it (host v3, envoy-v3-ui.md).
pub const OP_ROT: u16 = 11;
/// A built-in effect for the quads that follow (host fx, envoy-fx.md): `effect(0, 0)` ends it.
pub const OP_EFFECT: u16 = 12;
/// gd-ui-v1.4: an analytic shape (exact at every size and resolution, one quad) and a text outline.
pub const OP_SHAPE: u16 = 13;
pub const OP_OUTLINE: u16 = 14;
pub const SHAPE_BOX: u32 = 1;
pub const SHAPE_DIAMOND: u32 = 2;
pub const SHAPE_GLOW: u32 = 3;
pub const FX_NONE: u32 = 0;
pub const FX_FOIL: u32 = 1;
pub const FX_HOLO: u32 = 2;
pub const FX_GLOW: u32 = 3;

pub const LAYER_SCENE: i32 = 0;
pub const LAYER_HUD_UNDER: i32 = 1;
pub const LAYER_HUD: i32 = 2;
pub const LAYER_OVERLAY: i32 = 3;
pub const LAYER_MENU: i32 = 4;
pub const LAYER_MODAL: i32 = 5;
pub const LAYER_DEBUG: i32 = 6;

pub const SPACE_BAND: i32 = 0;
pub const SPACE_WINDOW: i32 = 1;

pub const ROLE_LABEL: i32 = 0;
pub const ROLE_VALUE: i32 = 1;
pub const ROLE_BODY: i32 = 2;
pub const ROLE_BIG: i32 = 3;
pub const ROLE_TITLE: i32 = 4;
pub const ROLE_TOAST: i32 = 5;
/// gd-ui-v1.1 text style bits, OR-ed onto a role (GD_UI_TEXT_* in gd-ui-lab-v1.h); a mod using them should export
/// `gdmod_ui_abi` = [`UI_ABI_1_1`].
pub const TEXT_REGULAR: i32 = 1 << 8;
pub const TEXT_BOLD: i32 = 2 << 8;
pub const TEXT_MONO: i32 = 1 << 10;
/// Host v3 faces (envoy-v3-ui.md): Bricolage Grotesque ExtraBold / Source Sans 3 Bold.
pub const TEXT_DISPLAY: i32 = 1 << 11;
pub const TEXT_BODY: i32 = 1 << 12;
pub const fn text_size(px: u32) -> i32 {
    ((px & 0xFF) << 16) as i32
}

pub const ALIGN_LEFT: i32 = 0;
pub const ALIGN_CENTER: i32 = 1;
pub const ALIGN_RIGHT: i32 = 2;

pub const SLICE_FILL_R4: u32 = 1;
pub const SLICE_FILL_R6: u32 = 2;
pub const SLICE_FILL_R14: u32 = 3;
pub const SLICE_LINE_R4: u32 = 4;
pub const SLICE_LINE_R6: u32 = 5;
pub const SLICE_LINE_R14_3: u32 = 6;
pub const SLICE_GLOW_R6: u32 = 7;
pub const SLICE_FILL_R1: u32 = 8;

pub const FEATURE_UI_DRAW: u32 = 1 << 4;
pub const FEATURE_UI_MENU: u32 = 1 << 5;
pub const FEATURE_UI_INPUT: u32 = 1 << 6;
pub const FEATURE_UI_SCENE: u32 = 1 << 7;
pub const UI_ABI: u32 = 1 << 16;
pub const UI_ABI_1_1: u32 = (1 << 16) | 1;

pub const CAP_EXCLUSIVE: u32 = 1;
pub const CAP_SHARED: u32 = 2;
pub const CAP_KEYBOARD: u32 = 4;

pub const NODE_PANEL: i32 = 0;
pub const NODE_LIST: i32 = 1;
pub const NODE_TABS: i32 = 2;
pub const NODE_BUTTON: i32 = 3;
pub const NODE_TOGGLE: i32 = 4;
pub const NODE_SLIDER: i32 = 5;
pub const NODE_CHOICE: i32 = 6;
pub const NODE_LABEL: i32 = 7;
pub const EV_ACTIVATE: i32 = 0;
pub const EV_CHANGE: i32 = 1;
pub const EV_BACK: i32 = 2;
pub const EV_FOCUS: i32 = 3;

#[link(wasm_import_module = "gd_ui")]
extern "C" {
    pub fn gd_ui_submit(ptr: *const u8, len: u32) -> i32;
    pub fn gd_ui_measure(s: *const u8, len: u32, role: i32) -> f32;
    pub fn gd_ui_kit_id(s: *const u8, len: u32) -> u32;
    pub fn gd_ui_log(s: *const u8, len: u32) -> i32;
    pub fn gd_ui_band_rect(out: *mut f32) -> i32;
    pub fn gd_ui_theme(id: i32) -> i32;
    pub fn gd_ui_token(tok: i32) -> u32;
    pub fn gd_ui_session() -> i32;
    pub fn gd_ui_node(parent: u32, kind: i32, id: u32) -> u32;
    pub fn gd_ui_node_label(n: u32, s: *const u8, len: u32) -> i32;
    pub fn gd_ui_node_value(n: u32, v: i32) -> i32;
    pub fn gd_ui_node_range(n: u32, lo: i32, hi: i32, step: i32) -> i32;
    pub fn gd_ui_node_choices(n: u32, s: *const u8, len: u32) -> i32;
    pub fn gd_ui_node_rect(n: u32, x: f32, y: f32, w: f32, h: f32) -> i32;
    pub fn gd_ui_show(root: u32, layer: i32, z: i32) -> i32;
    pub fn gd_ui_hide(root: u32);
    pub fn gd_ui_capture(ports: u32, flags: u32) -> u32;
    pub fn gd_ui_release(token: u32);
    pub fn gd_ui_pad(port: u32, out: *mut u8) -> i32;
}

/// The frame's command buffer: `N` 32-bit words.
pub struct Enc<const N: usize> {
    pub w: [u32; N],
    pub len: usize,
    pub overflow: u32,
}

impl<const N: usize> Enc<N> {
    pub const fn new() -> Self {
        Enc { w: [0; N], len: 0, overflow: 0 }
    }
    pub fn clear(&mut self) {
        self.len = 0;
    }
    fn head(&mut self, op: u16, words: usize) -> bool {
        if self.len + 1 + words > N {
            self.overflow += 1;
            return false;
        }
        self.w[self.len] = op as u32 | (words as u32) << 16;
        self.len += 1;
        true
    }
    fn put(&mut self, v: u32) {
        self.w[self.len] = v;
        self.len += 1;
    }
    fn f(&mut self, v: f32) {
        self.put(v.to_bits());
    }
    pub fn layer(&mut self, layer: i32, z: i32) -> &mut Self {
        if self.head(OP_LAYER, 2) {
            self.put(layer as u32);
            self.put(z as u32);
        }
        self
    }
    /// GAP-0: following records in band units (SPACE_BAND) or window pixels (SPACE_WINDOW), all times `scale`.
    pub fn space(&mut self, space: i32, scale: f32) -> &mut Self {
        if self.head(OP_SPACE, 2) {
            self.put(space as u32);
            self.f(scale);
        }
        self
    }
    pub fn rot(&mut self, a: f32, x: f32, y: f32) -> &mut Self {
        if self.head(OP_ROT, 3) {
            self.f(a);
            self.f(x);
            self.f(y);
        }
        self
    }
    /// Effect `id` (FX_*) in colour `rgba` for the records that follow (text is never affected).
    pub fn effect(&mut self, id: u32, rgba: u32) -> &mut Self {
        if self.head(OP_EFFECT, 2) {
            self.put(id);
            self.put(rgba);
        }
        self
    }
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, rgba: u32) -> &mut Self {
        self.rect_shear(x, y, w, h, rgba, 0.0)
    }
    pub fn rect_shear(&mut self, x: f32, y: f32, w: f32, h: f32, rgba: u32, shear: f32) -> &mut Self {
        if self.head(OP_RECT, 6) {
            for v in [x, y, w, h] {
                self.f(v);
            }
            self.put(rgba);
            self.f(shear);
        }
        self
    }
    /// A rounded box (a pill or a circle once `radius` reaches half the short side): `fill`, and a border of
    /// `border` units in `line` drawn inside the outline (0: none). A clear fill with a border is a ring.
    pub fn shape(&mut self, kind: u32, x: f32, y: f32, w: f32, h: f32, radius: f32, border: f32, spread: f32, fill: u32, line: u32) -> &mut Self {
        if self.head(OP_SHAPE, 10) {
            self.put(kind);
            for v in [x, y, w, h, radius, border, spread] {
                self.f(v);
            }
            self.put(fill);
            self.put(line);
        }
        self
    }
    pub fn round_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, fill: u32) -> &mut Self {
        self.shape(SHAPE_BOX, x, y, w, h, radius, 0.0, 0.0, fill, 0)
    }
    /// A disc of diameter `d` centred on (cx, cy).
    pub fn circle(&mut self, cx: f32, cy: f32, d: f32, fill: u32) -> &mut Self {
        self.shape(SHAPE_BOX, cx - d * 0.5, cy - d * 0.5, d, d, d, 0.0, 0.0, fill, 0)
    }
    /// The text that follows is outlined `width` units in `rgba` (0 ends it).
    pub fn outline(&mut self, width: f32, rgba: u32) -> &mut Self {
        if self.head(OP_OUTLINE, 2) {
            self.f(width);
            self.put(rgba);
        }
        self
    }
    pub fn slice(&mut self, id: u32, x: f32, y: f32, w: f32, h: f32, rgba: u32) -> &mut Self {
        if self.head(OP_SLICE, 6) {
            self.put(id);
            for v in [x, y, w, h] {
                self.f(v);
            }
            self.put(rgba);
        }
        self
    }
    pub fn kit(&mut self, id: u32, x: f32, y: f32, scale: f32, rgba: u32) -> &mut Self {
        if self.head(OP_KIT, 5) {
            self.put(id);
            for v in [x, y, scale] {
                self.f(v);
            }
            self.put(rgba);
        }
        self
    }
    pub fn tex(&mut self, tex: u32, x: f32, y: f32, w: f32, h: f32, uv: [f32; 4], rgba: u32) -> &mut Self {
        if self.head(OP_TEX, 10) {
            self.put(tex);
            for v in [x, y, w, h, uv[0], uv[1], uv[2], uv[3]] {
                self.f(v);
            }
            self.put(rgba);
        }
        self
    }
    pub fn text_ex(&mut self, x: f32, y: f32, role: i32, rgba: u32, align: i32, max_w: f32, s: &str) -> &mut Self {
        let b = s.as_bytes();
        let words = b.len().div_ceil(4);
        if self.head(OP_TEXT, 7 + words) {
            self.f(x);
            self.f(y);
            self.put(role as u32);
            self.put(rgba);
            self.put(align as u32);
            self.f(max_w);
            self.put(b.len() as u32);
            for c in b.chunks(4) {
                let mut q = [0u8; 4];
                q[..c.len()].copy_from_slice(c);
                self.put(u32::from_le_bytes(q));
            }
        }
        self
    }
    pub fn text(&mut self, x: f32, y: f32, role: i32, rgba: u32, s: &str) -> &mut Self {
        self.text_ex(x, y, role, rgba, ALIGN_LEFT, 0.0, s)
    }
    pub fn clip(&mut self, x: f32, y: f32, w: f32, h: f32) -> &mut Self {
        if self.head(OP_CLIP, 4) {
            for v in [x, y, w, h] {
                self.f(v);
            }
        }
        self
    }
    pub fn unclip(&mut self) -> &mut Self {
        self.head(OP_UNCLIP, 0);
        self
    }
    /// Hand the frame to the host (the one boundary crossing per present).
    pub fn submit(&self) -> i32 {
        unsafe { gd_ui_submit(self.w.as_ptr() as *const u8, (self.len * 4) as u32) }
    }
}

impl<const N: usize> Default for Enc<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// The text width the host will draw `s` with (cached on the host side by role).
pub fn measure(s: &str, role: i32) -> f32 {
    unsafe { gd_ui_measure(s.as_ptr(), s.len() as u32, role) }
}

/// A line in the game's log (`modhost: ui: <mod>: log: ..`): for headless checks, not per frame.
pub fn log(s: &str) {
    unsafe { gd_ui_log(s.as_ptr(), s.len() as u32) };
}
