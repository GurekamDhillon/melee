//! A minimal 2D layer on the GX shim for the native frontend screens: filled rectangles and an
//! 8x8 bitmap font, all in one alpha-blended `GX_QUADS` draw in 640x480 EFB coordinates.
//!
//! This is the port's own UI drawing (a stand-in for the HSD text/menu models of the retail
//! screens, which are not ported); it adds nothing to the simulation.
use melee_gx::consts::*;
use melee_gx::shim::GxShim;

pub const W: f32 = 640.0;
pub const H: f32 = 480.0;

pub type Rgba = [u8; 4];

pub const WHITE: Rgba = [255, 255, 255, 255];
pub const GREY: Rgba = [150, 155, 170, 255];
pub const DIM: Rgba = [90, 95, 110, 255];
pub const YELLOW: Rgba = [255, 224, 80, 255];
pub const RED: Rgba = [235, 70, 60, 255];
pub const GREEN: Rgba = [90, 220, 120, 255];
pub const CYAN: Rgba = [90, 200, 240, 255];

#[derive(Default)]
pub struct Ui {
    quads: Vec<(f32, f32, f32, f32, Rgba)>,
}

impl Ui {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.quads.clear();
    }

    pub fn len(&self) -> usize {
        self.quads.len()
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        if c[3] != 0 && w > 0.0 && h > 0.0 {
            self.quads.push((x, y, w, h, c));
        }
    }

    /// A rectangle outline `t` pixels thick.
    pub fn frame(&mut self, x: f32, y: f32, w: f32, h: f32, t: f32, c: Rgba) {
        self.rect(x, y, w, t, c);
        self.rect(x, y + h - t, w, t, c);
        self.rect(x, y + t, t, h - 2.0 * t, c);
        self.rect(x + w - t, y + t, t, h - 2.0 * t, c);
    }

    /// A vertical two-colour gradient in `bands` strips.
    pub fn gradient(&mut self, x: f32, y: f32, w: f32, h: f32, top: Rgba, bottom: Rgba, bands: u32) {
        for i in 0..bands {
            let t = (i as f32 + 0.5) / bands as f32;
            let mut c = [0u8; 4];
            for k in 0..4 {
                c[k] = (top[k] as f32 + (bottom[k] as f32 - top[k] as f32) * t).round() as u8;
            }
            let y0 = y + h * i as f32 / bands as f32;
            let y1 = y + h * (i + 1) as f32 / bands as f32;
            self.rect(x, y0, w, y1 - y0 + 0.5, c);
        }
    }

    /// Width of `s` at `scale` (each glyph cell is 8 * scale wide).
    pub fn text_width(s: &str, scale: f32) -> f32 {
        s.chars().count() as f32 * 8.0 * scale
    }

    pub fn text(&mut self, x: f32, y: f32, scale: f32, s: &str, c: Rgba) {
        let mut cx = x;
        for ch in s.chars() {
            let glyph = font8x8::UnicodeFonts::get(&font8x8::BASIC_FONTS, if ch.is_ascii() { ch } else { '?' }).unwrap_or([0; 8]);
            for (row, bits) in glyph.iter().enumerate() {
                // Merge horizontal runs of lit pixels (bit 0 is the leftmost column).
                let mut col = 0;
                while col < 8 {
                    if bits >> col & 1 == 1 {
                        let start = col;
                        while col < 8 && bits >> col & 1 == 1 {
                            col += 1;
                        }
                        self.rect(cx + start as f32 * scale, y + row as f32 * scale, (col - start) as f32 * scale, scale, c);
                    } else {
                        col += 1;
                    }
                }
            }
            cx += 8.0 * scale;
        }
    }

    pub fn text_centered(&mut self, cx: f32, y: f32, scale: f32, s: &str, c: Rgba) {
        self.text(cx - Self::text_width(s, scale) / 2.0, y, scale, s, c);
    }

    /// Text with a one-pixel-step drop shadow.
    pub fn text_shadow(&mut self, x: f32, y: f32, scale: f32, s: &str, c: Rgba) {
        self.text(x + scale, y + scale, scale, s, [0, 0, 0, c[3]]);
        self.text(x, y, scale, s, c);
    }

    pub fn text_centered_shadow(&mut self, cx: f32, y: f32, scale: f32, s: &str, c: Rgba) {
        self.text_shadow(cx - Self::text_width(s, scale) / 2.0, y, scale, s, c);
    }

    /// Emit everything queued as one draw over whatever is in the EFB, then clear the queue.
    pub fn flush(&mut self, shim: &mut GxShim) {
        if self.quads.is_empty() {
            return;
        }
        let mut proj = [[0.0f32; 4]; 4];
        proj[0][0] = 2.0 / W;
        proj[0][3] = -1.0;
        proj[1][1] = -2.0 / H;
        proj[1][3] = 1.0;
        proj[2][2] = -1.0 / 100.0;
        proj[2][3] = -1.0;
        proj[3][3] = 1.0;
        shim.set_viewport(0.0, 0.0, W, H, 0.0, 1.0);
        shim.set_scissor(0, 0, W as u32, H as u32);
        shim.set_projection(&proj, GX_ORTHOGRAPHIC);
        let ident = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]];
        shim.load_pos_mtx_imm(&ident, GX_PNMTX0);
        shim.load_nrm_mtx_imm(&ident, GX_PNMTX0);
        shim.set_current_mtx(GX_PNMTX0);
        shim.set_cull_mode(GX_CULL_NONE);
        shim.set_z_mode(false, GX_ALWAYS, false);
        shim.set_alpha_compare(GX_ALWAYS, 0, GX_AOP_AND, GX_ALWAYS, 0);
        shim.set_blend_mode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
        shim.set_num_ind_stages(0);
        shim.set_tev_direct(GX_TEVSTAGE0);
        shim.set_num_tex_gens(0);
        shim.set_num_chans(1);
        shim.set_chan_ctrl(GX_COLOR0A0, false, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE, GX_AF_NONE);
        shim.set_num_tev_stages(1);
        shim.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
        shim.set_tev_op(GX_TEVSTAGE0, GX_PASSCLR);
        shim.clear_vtx_desc();
        shim.set_vtx_desc(GX_VA_POS, GX_DIRECT);
        shim.set_vtx_desc(GX_VA_CLR0, GX_DIRECT);
        shim.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XY, GX_F32, 0);
        shim.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
        // A draw holds at most 65535 vertices.
        for chunk in self.quads.chunks(16000) {
            shim.begin(GX_QUADS, GX_VTXFMT0, (chunk.len() * 4) as u16);
            for &(x, y, w, h, c) in chunk {
                for (px, py) in [(x, y), (x + w, y), (x + w, y + h), (x, y + h)] {
                    shim.gx.position2f32(px, py);
                    shim.gx.color4u8(c[0], c[1], c[2], c[3]);
                }
            }
            shim.end();
        }
        self.quads.clear();
    }
}
