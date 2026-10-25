//! HSD_SisLib text (src/sysdolphin/baselib/sislib.c, hsd_3A64.c, hsd_3A76.c): the menus' text engine.
//! A text object points at a SIS string (a byte stream in a "Sd*.usd" archive: control opcodes below
//! 0x20, two-byte glyph codes from 0x20xx up) and draws it as one textured quad per glyph (I4 32x32
//! glyph textures; kerning pairs per glyph) in the scene's camera space.
//!
//! Ported here: `HSD_SisLib_803A6368` (set string), `803A7684` / `803A7F0C` (the push / pop state stack in
//! `string_buffer`), `803A8134` (line measure) and the draw `803A84BC`. The SIS bytes are the archive's
//! big-endian data; "pointers" (the call opcodes 8 / 9) are data offsets. NOT ported: formatted text
//! (`803A6B98` and friends build SIS streams from printf formats), the m-ex opt-in text patches.
use super::sis_tables::KERNING;
use super::{be32, DatArchive};
use melee_gx::api::GxColor;
use melee_gx::consts::*;
use melee_gx::shim::GxShim;
use melee_gx::state::TexObj;
use melee_gx::GxData;
use melee_runtime::mtx::Mtx;
use std::collections::HashMap;
use std::sync::Arc;

fn be16(d: &[u8], at: u32) -> u16 {
    d.get(at as usize..at as usize + 2).map_or(0, |b| u16::from_be_bytes(b.try_into().unwrap()))
}
fn s16(d: &[u8], at: u32) -> i16 {
    be16(d, at) as i16
}

/// The default 287-glyph atlas (`HSD_SisLib_FontAtlas`, main.dol 0x8040CD40, 0x23E00 bytes).
pub const ATLAS_ADDR: u32 = 0x8040_CD40;
pub const ATLAS_BYTES: usize = 0x23E00;

/// A loaded `Sd*.usd` string archive and its glyph block (`HSD_SisLib_804D1124[font]`).
pub struct SisArchive {
    dat: Arc<Vec<u8>>,
    table: u32,
}

impl SisArchive {
    pub fn load(archive: &[u8], symbol: &str) -> Result<Self, String> {
        let a = DatArchive::parse(archive).map_err(|e| format!("sis archive: {e}"))?;
        let table = a.public_offset(symbol).ok_or(format!("sis archive: no {symbol}"))?;
        Ok(Self { dat: Arc::new(a.data), table })
    }
    /// `sis->kerning`: the base of the archive's own glyph pixels.
    fn glyph_base(&self) -> u32 {
        be32(&self.dat, self.table)
    }
    /// `sis->textures->data`: the archive's kerning pairs.
    fn kern_base(&self) -> u32 {
        be32(&self.dat, self.table + 4)
    }
    /// `sis_table[idx]`: the string stream.
    pub fn string(&self, idx: u32) -> u32 {
        be32(&self.dat, self.table + 4 * idx)
    }
}

/// Everything a draw needs besides the text.
pub struct SisEnv<'a> {
    pub sis: &'a SisArchive,
    pub atlas: &'a [u8],
}

#[derive(Clone)]
pub struct Text {
    pub pos: [f32; 3],
    pub box_size: [f32; 2],
    /// `x14`
    pub clip: [f32; 4],
    pub font_size: [f32; 2],
    pub bg_color: [u8; 4],
    pub text_color: [u8; 4],
    /// `x34` default scale, `x3C` default spacing
    pub x34: [f32; 2],
    pub x3c: [f32; 2],
    pub x44: u16,
    pub x46: u16,
    pub default_fitting: u8,
    pub default_kerning: u8,
    pub default_alignment: u8,
    x4b: u8,
    /// z-test the glyphs
    pub x4c: u8,
    pub hidden: bool,
    /// clip to the box inset by `clip`
    pub x4e: u8,
    sis_buffer: Option<u32>,
    x60: Option<u32>,
    pub alloc_data: bool,
    sbuf: Vec<u8>,
    x6c: usize,
    current_width: f32,
    current_height: f32,
    x78: [f32; 2],
    x80: [f32; 2],
    x88: f32,
    active_color: [u8; 4],
    x90: u16,
    x92: u16,
    x94: u32,
    x98: u32,
    fitting: u8,
    kerning: u8,
    alignment: u8,
}

impl Text {
    /// `HSD_SisLib_803A5ACC`'s initial state.
    pub fn new(pos: [f32; 3], box_size: [f32; 2]) -> Self {
        Self {
            pos,
            box_size,
            clip: [0.0; 4],
            font_size: [1.0, 1.0],
            bg_color: [0; 4],
            text_color: [0xFF; 4],
            x34: [1.0, 1.0],
            x3c: [0.0, 0.0],
            x44: 0,
            x46: 0,
            default_fitting: 0,
            default_kerning: 0,
            default_alignment: 0,
            x4b: 0,
            x4c: 0,
            hidden: false,
            x4e: 0,
            sis_buffer: None,
            x60: None,
            alloc_data: false,
            sbuf: vec![0; 0x10],
            x6c: 0,
            current_width: 0.0,
            current_height: 0.0,
            x78: [0.0; 2],
            x80: [1.0; 2],
            x88: 0.0,
            active_color: [0xFF; 4],
            x90: 0,
            x92: 0,
            x94: 0,
            x98: 0,
            fitting: 0,
            kerning: 0,
            alignment: 0,
        }
    }

    /// `HSD_SisLib_803A6368(text, sis_idx)`.
    pub fn set_string(&mut self, sis: &SisArchive, idx: u32) {
        self.sis_buffer = Some(sis.string(idx));
        self.x60 = None;
        self.current_height = 0.0;
        self.current_width = 0.0;
        self.active_color = self.text_color;
        self.x80 = self.x34;
        self.x78 = self.x3c;
        self.x90 = self.x44;
        self.x92 = self.x46;
        self.alignment = self.default_alignment;
        self.kerning = self.default_kerning;
        self.fitting = self.default_fitting;
        self.x6c = 0;
        self.x98 = 0;
        self.x94 = 0;
        self.x4b = 0;
        self.sbuf = vec![0; 0x10];
    }

    fn sb_room(&mut self, n: usize) {
        while self.sbuf.len() < self.x6c + n + 1 {
            self.sbuf.resize(self.sbuf.len() + 0x10, 0);
        }
    }

    fn put(&mut self, b: u8) {
        self.sb_room(1);
        self.sbuf[self.x6c] = b;
        self.x6c += 1;
    }

    /// `HSD_SisLib_803A7684(text, cursor, flags)`: push the state `flags & 0x7F` names.
    fn push(&mut self, cursor: u32, flags: u8) {
        let f8 = |v: f32| -> [u8; 2] {
            let s = (256.0f32 * v) as i32;
            [(s >> 8) as u8, (256.0f32 * v) as i32 as u8]
        };
        match flags & 0x7F {
            1 => {
                let (x, y) = (f8(self.x78[0]), f8(self.x78[1]));
                for b in [x[0], x[1], y[0], y[1], flags] {
                    self.put(b);
                }
            }
            2 => {
                let c = self.active_color;
                for b in [c[0], c[1], c[2], flags] {
                    self.put(b);
                }
            }
            3 => {
                let (x, y) = (f8(self.x80[0]), f8(self.x80[1]));
                for b in [x[0], x[1], y[0], y[1], flags] {
                    self.put(b);
                }
            }
            4 => {
                let a = self.alignment;
                for b in [a, flags] {
                    self.put(b);
                }
            }
            5 => {
                for b in cursor.to_be_bytes() {
                    self.put(b);
                }
                self.put(flags);
            }
            _ => {}
        }
    }

    /// `HSD_SisLib_803A7F0C(text, flags)`: pop the newest state of kind `flags & 0x7F` (restoring it into
    /// the text; with the high bit set the entry is removed too). Returns the pointer for kind 5.
    fn pop(&mut self, flags: u8) -> u32 {
        let flag_hi = (flags & 0x80) as i32;
        let target = (flags & 0x7F) as i32;
        let mut pos = self.x6c as i32;
        let mut result = 0u32;
        let mut remove = 0i32;
        let g = |s: &Self, p: i32| -> u8 { s.sbuf.get(p as usize).copied().unwrap_or(0) };
        while pos >= 0 {
            let entry = g(self, pos);
            let ehi = (entry & 0x80) as i32;
            match entry & 0x7F {
                1 => {
                    pos -= 4;
                    if target == 1 {
                        let p = pos as usize;
                        self.x78[0] = i16::from_be_bytes([self.sbuf[p], self.sbuf[p + 1]]) as f32 / 256.0;
                        self.x78[1] = i16::from_be_bytes([self.sbuf[p + 2], self.sbuf[p + 3]]) as f32 / 256.0;
                        if flag_hi == ehi {
                            remove = 5;
                        }
                        break;
                    }
                }
                2 => {
                    pos -= 3;
                    if target == 2 {
                        let p = pos as usize;
                        self.active_color[0] = self.sbuf[p];
                        self.active_color[1] = self.sbuf[p + 1];
                        self.active_color[2] = self.sbuf[p + 2];
                        if flag_hi == ehi {
                            remove = 4;
                        }
                        break;
                    }
                }
                3 => {
                    pos -= 4;
                    if target == 3 {
                        let p = pos as usize;
                        self.x80[0] = u16::from_be_bytes([self.sbuf[p], self.sbuf[p + 1]]) as f32 / 256.0;
                        self.x80[1] = u16::from_be_bytes([self.sbuf[p + 2], self.sbuf[p + 3]]) as f32 / 256.0;
                        if flag_hi == ehi {
                            remove = 5;
                        }
                        break;
                    }
                }
                4 => {
                    pos -= 1;
                    if target == 4 {
                        self.alignment = self.sbuf[pos as usize];
                        if flag_hi == ehi {
                            remove = 2;
                        }
                        break;
                    }
                }
                5 => {
                    pos -= 4;
                    if target == 5 {
                        let p = pos as usize;
                        result = u32::from_be_bytes([self.sbuf[p], self.sbuf[p + 1], self.sbuf[p + 2], self.sbuf[p + 3]]);
                        if flag_hi == ehi {
                            remove = 5;
                        }
                        break;
                    }
                }
                _ => {}
            }
            pos -= 1;
        }
        if remove != 0 {
            let len = self.x6c as i32;
            while pos + remove < len {
                self.sbuf[pos as usize] = self.sbuf[(pos + remove) as usize];
                pos += 1;
            }
            while pos < len {
                self.sbuf[pos as usize] = 0;
                pos += 1;
            }
            self.x6c = (len - remove) as usize;
        }
        result
    }

    fn clear_stack_tail(&mut self) {
        for i in self.x6c..self.sbuf.len() {
            self.sbuf[i] = 0;
        }
    }

    fn kern_pair(env: &SisEnv, glyph: u16) -> (u8, u8) {
        let idx = ((glyph as usize).wrapping_sub(if glyph < 0x4000 { 0x2000 } else { 0x4000 }) * 2) & 0x1FFFE;
        if glyph < 0x4000 {
            (KERNING.get(idx).copied().unwrap_or(0), KERNING.get(idx + 1).copied().unwrap_or(0))
        } else {
            let b = env.sis.kern_base();
            (env.sis.dat.get(b as usize + idx).copied().unwrap_or(0), env.sis.dat.get(b as usize + idx + 1).copied().unwrap_or(0))
        }
    }

    /// `HSD_SisLib_803A8134`: the width / height of the line starting at `cursor`.
    fn measure(&mut self, env: &SisEnv, mut cursor: u32) -> (f32, f32) {
        let d = &env.sis.dat;
        let (saved_sx, saved_sy, saved_sp) = (self.x80[0], self.x80[1], self.x78[0]);
        let mut kern_enabled = self.kerning;
        let saved_x6c = self.x6c;
        let mut w = 0.0f32;
        let mut h = 32.0 * self.x80[1];
        loop {
            let op = d.get(cursor as usize).copied().unwrap_or(0);
            match op {
                0 => {
                    let r = self.pop(0x85);
                    if r != 0 {
                        cursor = r + 4;
                    } else {
                        break;
                    }
                }
                1 | 2 | 3 | 7 => break,
                9 | 8 => {
                    if op == 9 {
                        self.push(cursor, 0x85);
                    }
                    cursor = be32(d, cursor + 1).wrapping_sub(1);
                }
                14 => {
                    self.push(cursor, 0x83);
                    self.x80[0] = be16(d, cursor + 1) as f32 / 256.0;
                    self.x80[1] = be16(d, cursor + 3) as f32 / 256.0;
                    cursor += 4;
                }
                15 => {
                    self.pop(0x83);
                }
                6 => cursor += 4,
                10 => {
                    if !self.alloc_data || kern_enabled == 0 {
                        self.push(cursor, 0x81);
                        self.x78[0] = s16(d, cursor + 1) as f32 / 256.0;
                    }
                    cursor += 4;
                }
                11 => {
                    if !self.alloc_data || kern_enabled == 0 {
                        self.pop(0x81);
                    }
                }
                12 => cursor += 3,
                5 => cursor += 2,
                22 => kern_enabled = 1,
                23 => kern_enabled = 0,
                26 => {
                    w += self.x80[0] * (16.0 + self.x78[0]);
                    if h < 32.0 * self.x80[1] {
                        h = 32.0 * self.x80[1];
                    }
                }
                _ => {
                    if op >= 0x20 {
                        w += self.x80[0] * (32.0 + self.x78[0]);
                        if kern_enabled != 0 {
                            let glyph = be16(d, cursor);
                            let (l, r) = Self::kern_pair(env, glyph);
                            let kw = (r as i32 - 2) + l as i32;
                            w = -((self.x80[0] * kw as f32) - w);
                        }
                        if h < 32.0 * self.x80[1] {
                            h = 32.0 * self.x80[1];
                        }
                        cursor += 1;
                    }
                }
            }
            cursor += 1;
        }
        w -= self.x78[0];
        self.x80 = [saved_sx, saved_sy];
        self.x78[0] = saved_sp;
        self.x6c = saved_x6c;
        self.clear_stack_tail();
        (w, h)
    }

    fn fit_line_to_box(&mut self, measured: f32) {
        if self.fitting == 1 && self.box_size[0] < measured {
            self.current_width = 0.0;
            self.x88 = self.box_size[0] / measured;
        } else {
            self.x88 = 1.0;
            self.current_width = match self.alignment {
                1 => 0.5 * (self.box_size[0] - measured),
                2 => self.box_size[0] - measured,
                _ => 0.0,
            };
        }
    }

    /// `HSD_SisLib_803A84BC` for a gobj-drawn text (3D: `vmtx` is the camera's viewing matrix). The GX
    /// projection must already be the scene camera's.
    pub fn draw(&mut self, shim: &mut GxShim, env: &SisEnv, vmtx: &Mtx, cache: &mut GlyphCache) {
        if self.hidden {
            return;
        }
        let Some(mut cursor) = self.sis_buffer else { return };
        let d = env.sis.dat.clone();
        if self.x4c != 0 {
            shim.set_z_mode(true, GX_LEQUAL, false);
        } else {
            shim.set_z_mode(false, GX_NEVER, false);
        }
        shim.set_current_mtx(GX_PNMTX0);
        shim.load_pos_mtx_imm(vmtx, GX_PNMTX0);
        shim.set_tex_coord_gen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY, false, GX_PTIDENTITY);
        shim.set_num_chans(0);
        shim.clear_vtx_desc();
        shim.set_cull_mode(GX_CULL_NONE);
        shim.set_num_tex_gens(1);
        shim.set_num_tev_stages(1);
        shim.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
        shim.set_vtx_desc(GX_VA_POS, GX_DIRECT);
        shim.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_F32, 0);
        shim.set_vtx_desc(GX_VA_TEX0, GX_DIRECT);
        shim.set_tev_order(GX_TEVSTAGE0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR_NULL);
        shim.set_blend_mode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR);
        shim.set_alpha_compare(GX_GREATER, 0, GX_AOP_OR, GX_GREATER, 0);
        shim.set_color_update(true);
        shim.set_alpha_update(false);
        shim.gx.set_tev_color_in(GX_TEVSTAGE0, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO, GX_CC_C0);
        shim.gx.set_tev_color_op(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, false, GX_TEVPREV);
        shim.gx.set_tev_alpha_op(GX_TEVSTAGE0, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, false, GX_TEVPREV);
        let quad = |shim: &mut GxShim, x0: f32, y0: f32, x1: f32, y1: f32, z: f32, u0: f32, v0: f32, u1: f32, v1: f32| {
            shim.begin(GX_QUADS, GX_VTXFMT0, 4);
            for (x, y, u, v) in [(x0, -y0, u0, v0), (x1, -y0, u1, v0), (x1, -y1, u1, v1), (x0, -y1, u0, v1)] {
                shim.gx.position3f32(x, y, z);
                shim.gx.tex_coord2f32(u, v);
            }
            shim.end();
        };
        if self.bg_color[3] != 0 {
            let (min_x, min_y) = (self.pos[0], self.pos[1]);
            let max_x = self.box_size[0] * self.font_size[0] + self.pos[0];
            let max_y = self.box_size[1] * self.font_size[1] + self.pos[1];
            shim.gx.set_tev_alpha_in(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO, GX_CA_A0);
            let c = self.bg_color;
            shim.gx.set_tev_color(GX_TEVREG0, GxColor::new(c[0], c[1], c[2], c[3]));
            quad(shim, min_x, min_y, max_x, max_y, self.pos[2], 0.0, 0.0, 1.0, 1.0);
        }
        shim.gx.set_tev_alpha_in(GX_TEVSTAGE0, GX_CA_ZERO, GX_CA_TEXA, GX_CA_A0, GX_CA_ZERO);
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        if self.x4e != 0 {
            min_x = self.clip[2] * self.font_size[0] + self.pos[0];
            max_x = self.font_size[0] * (self.box_size[0] - self.clip[3]) + self.pos[0];
            min_y = self.clip[0] * self.font_size[1] + self.pos[1];
            max_y = self.font_size[1] * (self.box_size[1] - self.clip[1]) + self.pos[1];
        }
        if self.alloc_data {
            self.x78 = self.x3c;
            self.alignment = self.default_alignment;
            self.kerning = self.default_kerning;
            self.fitting = self.default_fitting;
        }
        let half_glyph = 16.0f64;
        let mut line_w;
        let mut line_h = 0.0f32;
        'restart: loop {
            self.current_height = 0.0;
            let mut line_started = false;
            let saved_font_size = self.x80;
            let saved_color = [self.active_color[0], self.active_color[1], self.active_color[2]];
            let saved_scale = self.x78;
            let saved_alignment = self.alignment;
            let saved_fitting = self.fitting;
            let saved_kerning = self.kerning;
            let line_delay = self.x90;
            let char_delay = self.x92;
            let mut skip_count = self.x98;
            let saved_x6c = self.x6c;
            let mut line_delay_v = line_delay;
            let mut char_delay_v = char_delay;
            loop {
                if Some(cursor) == self.x60 && self.x4b != 0 {
                    break;
                }
                if Some(cursor) == self.x60 && self.x94 != 0 {
                    self.x94 -= 1;
                    break;
                }
                let op = d.get(cursor as usize).copied().unwrap_or(0);
                match op {
                    0 => {
                        let r = self.pop(5);
                        if r != 0 {
                            cursor = r + 4;
                        } else {
                            break;
                        }
                    }
                    1 => {
                        self.x60 = None;
                        self.active_color = self.text_color;
                        self.x80 = self.x34;
                        self.x78 = self.x3c;
                        self.x90 = self.x44;
                        self.x92 = self.x46;
                        self.alignment = self.default_alignment;
                        self.kerning = self.default_kerning;
                        self.fitting = self.default_fitting;
                        self.x94 = 0;
                        self.x4b = 0;
                        self.sbuf.iter_mut().for_each(|b| *b = 0);
                        self.x6c = 0;
                        self.x98 = 0;
                        cursor += 1;
                        self.sis_buffer = Some(cursor);
                        continue 'restart;
                    }
                    2 => {
                        self.x98 = 0;
                        cursor += 1;
                        self.sis_buffer = Some(cursor);
                        continue 'restart;
                    }
                    3 => {
                        if !line_started {
                            line_h = 32.0 * self.x80[1];
                        }
                        line_started = false;
                        self.current_height = self.font_size[1] * (self.x80[1] * self.x78[1] + line_h) + self.current_height;
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            self.x98 += 1;
                            self.x94 = char_delay_v as u32;
                            self.x60 = Some(cursor + 1);
                        }
                    }
                    4 => {
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            self.x98 += 1;
                            self.x4b = 1;
                            self.x60 = Some(cursor + 1);
                        }
                    }
                    5 => {
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            self.x98 += 1;
                            self.x94 = be16(&d, cursor + 1) as u32;
                            self.x60 = Some(cursor + 3);
                        }
                        cursor += 2;
                    }
                    6 => {
                        line_delay_v = be16(&d, cursor + 1);
                        char_delay_v = be16(&d, cursor + 3);
                        cursor += 4;
                    }
                    7 => {
                        line_started = true;
                        let (w, h) = self.measure(env, cursor + 5);
                        line_w = w;
                        line_h = h;
                        let x_origin = s16(&d, cursor + 1) as f32;
                        if self.fitting == 1 && self.box_size[0] < line_w {
                            self.x88 = self.box_size[0] / line_w;
                        } else {
                            self.x88 = 1.0;
                        }
                        self.current_width = match self.alignment {
                            1 => -((0.5 * (line_w * self.x88)) - x_origin),
                            2 => -((line_w * self.x88) - x_origin),
                            _ => x_origin,
                        };
                        let y_off = s16(&d, cursor + 3);
                        cursor += 4;
                        self.current_height = y_off as f32 * self.font_size[1];
                    }
                    9 | 8 => {
                        if op == 9 {
                            self.push(cursor, 5);
                        }
                        cursor = be32(&d, cursor + 1).wrapping_sub(1);
                    }
                    10 => {
                        if !self.alloc_data || saved_kerning == 0 {
                            self.push(cursor, 1);
                            self.x78[0] = s16(&d, cursor + 1) as f32 / 256.0;
                            self.x78[1] = s16(&d, cursor + 3) as f32 / 256.0;
                        }
                        cursor += 4;
                    }
                    11 => {
                        if !self.alloc_data || saved_kerning == 0 {
                            self.pop(1);
                        }
                    }
                    12 => {
                        self.push(cursor, 2);
                        self.active_color[0] = d[cursor as usize + 1];
                        self.active_color[1] = d[cursor as usize + 2];
                        self.active_color[2] = d[cursor as usize + 3];
                        cursor += 3;
                    }
                    13 => {
                        self.pop(2);
                    }
                    14 => {
                        self.push(cursor, 3);
                        self.x80[0] = be16(&d, cursor + 1) as f32 / 256.0;
                        self.x80[1] = be16(&d, cursor + 3) as f32 / 256.0;
                        cursor += 4;
                    }
                    15 => {
                        self.pop(3);
                    }
                    16 => {
                        self.push(cursor, 4);
                        self.alignment = 1;
                    }
                    18 => {
                        self.push(cursor, 4);
                        self.alignment = 0;
                    }
                    20 => {
                        self.push(cursor, 4);
                        self.alignment = 2;
                    }
                    17 | 19 | 21 => {
                        self.pop(4);
                    }
                    22 => self.kerning = 1,
                    23 => self.kerning = 0,
                    24 => self.fitting = 1,
                    25 => self.fitting = 0,
                    26 => {
                        if !line_started {
                            line_started = true;
                            let (w, h) = self.measure(env, cursor);
                            line_w = w;
                            line_h = h;
                            self.fit_line_to_box(w);
                        }
                        self.current_width = ((self.x88 as f64) * ((self.x80[0] as f64) * (half_glyph + self.x78[0] as f64)) + self.current_width as f64) as f32;
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            self.x98 += 1;
                            self.x94 = line_delay_v as u32;
                            self.x60 = Some(cursor + 1);
                        }
                    }
                    _ if op >= 0x20 => {
                        if !line_started {
                            line_started = true;
                            let (w, h) = self.measure(env, cursor);
                            line_w = w;
                            line_h = h;
                            self.fit_line_to_box(w);
                        }
                        let glyph = be16(&d, cursor);
                        let tex_offset = (glyph as i32) - if glyph < 0x4000 { 0x2000 } else { 0x4000 };
                        let mut glyph_x = self.current_width * self.font_size[0] + self.pos[0];
                        let scale_x = self.font_size[0];
                        if self.kerning != 0 {
                            let (l, _) = Self::kern_pair(env, glyph);
                            glyph_x = -((scale_x * (self.x80[0] * (l as i32 - 1) as f32)) - glyph_x);
                        }
                        let scale_y = self.font_size[1];
                        let glyph_size = 32.0 * self.x80[1];
                        let (mut uv_top, mut uv_bottom, mut uv_left, mut uv_right) = (0.0f32, 1.0f32, 0.0f32, 1.0f32);
                        let glyph_w = 32.0 * self.x80[0] * scale_x;
                        let mut quad_right = self.x88 * glyph_w + glyph_x;
                        let glyph_y = scale_y * (line_h - glyph_size) + (self.pos[1] + self.current_height);
                        let glyph_h = glyph_size * scale_y;
                        let mut quad_top = glyph_y;
                        let mut quad_bottom = glyph_y + glyph_h;
                        let mut skip_draw = false;
                        if self.x4e != 0 {
                            if min_x > quad_right || max_x < glyph_x || min_y > quad_bottom || max_y < quad_top {
                                skip_draw = true;
                            } else {
                                if min_x > glyph_x {
                                    let c = min_x - glyph_x;
                                    uv_left = c / glyph_w;
                                    glyph_x += c;
                                }
                                if max_x < quad_right {
                                    let c = quad_right - max_x;
                                    uv_right = 1.0 - (c / glyph_w);
                                    quad_right -= c;
                                }
                                if min_y > quad_top {
                                    let c = min_y - quad_top;
                                    uv_top = c / glyph_h;
                                    quad_top += c;
                                }
                                if max_y < quad_bottom {
                                    let c = quad_bottom - max_y;
                                    uv_bottom = 1.0 - (c / glyph_h);
                                    quad_bottom -= c;
                                }
                            }
                        }
                        if !skip_draw {
                            let (src, base): (&[u8], usize) = if glyph < 0x4000 { (env.atlas, 0) } else { (&env.sis.dat[..], env.sis.glyph_base() as usize) };
                            let at = base + (((tex_offset as usize) << 9) & 0x01FF_FE00);
                            let mut tex = cache.tex(shim, glyph < 0x4000, tex_offset as u32, src, at);
                            shim.load_tex_obj(&mut tex, GX_TEXMAP0);
                            let c = self.active_color;
                            shim.gx.set_tev_color(GX_TEVREG0, GxColor::new(c[0], c[1], c[2], c[3]));
                            quad(shim, glyph_x, quad_top, quad_right, quad_bottom, self.pos[2], uv_left, uv_top, uv_right, uv_bottom);
                        }
                        self.current_width = self.x88 * (self.x80[0] * (32.0 + self.x78[0])) + self.current_width;
                        if self.kerning != 0 {
                            let (l, r) = Self::kern_pair(env, glyph);
                            let t = r as i32 - 2;
                            self.current_width = -((self.x88 * (self.x80[0] * (l as i32 + t) as f32)) - self.current_width);
                        }
                        if skip_count != 0 {
                            skip_count -= 1;
                        } else {
                            self.x98 += 1;
                            self.x94 = line_delay_v as u32;
                            self.x60 = Some(cursor + 2);
                        }
                        cursor += 1;
                    }
                    _ => {}
                }
                cursor += 1;
            }
            // render_done
            self.active_color[0] = saved_color[0];
            self.active_color[1] = saved_color[1];
            self.active_color[2] = saved_color[2];
            self.x80 = saved_font_size;
            self.x78 = saved_scale;
            self.alignment = saved_alignment;
            self.fitting = saved_fitting;
            self.kerning = saved_kerning;
            self.x6c = saved_x6c;
            self.clear_stack_tail();
            let _ = (line_delay, char_delay);
            break;
        }
    }
}

/// Glyph texture objects by (archive glyph?, index).
#[derive(Default)]
pub struct GlyphCache {
    objs: HashMap<(bool, u32), TexObj>,
}

impl GlyphCache {
    fn tex(&mut self, shim: &mut GxShim, default_atlas: bool, idx: u32, src: &[u8], at: usize) -> TexObj {
        if !self.objs.contains_key(&(default_atlas, idx)) {
            let mut o = TexObj::default();
            let end = (at + 512).min(src.len());
            let data = GxData::from(&src[at.min(end)..end]);
            shim.init_tex_obj(&mut o, Some(data), 32, 32, GX_TF_I4, GX_CLAMP, GX_CLAMP, false);
            self.objs.insert((default_atlas, idx), o);
        }
        self.objs[&(default_atlas, idx)].clone()
    }
}
