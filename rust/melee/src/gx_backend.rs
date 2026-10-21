//! melee_runtime's HSD draw path (`tu::displayfunc` .. `tu::pobj`) on melee_gx's `GxShim`: the
//! `GxBackend` trait the display functions call, implemented over the gw_GX* shim (the C build's
//! game TUs call `gw_GX*` the same way). DAT memory stays big-endian; vertex arrays, textures and
//! TLUTs are handed to the shim as shared `GxData` (one `Arc` per distinct slice, cached).
use melee_gx::api::{GxColor, GxLightObj};
use melee_gx::shim::{get_tex_buffer_size, GxShim};
use melee_gx::state::{TexObj, TlutObj};
use melee_gx::GxData;
use melee_runtime::mtx::Mtx;
use melee_runtime::tu::state::{GxBackend, LightLoad, Rgba, TexLoad};
use std::collections::HashMap;

pub struct ShimBackend<'a> {
    pub shim: &'a mut GxShim,
    pub cache: &'a mut DataCache,
}

/// Slices of DAT memory already wrapped as `GxData` (key: address and length).
#[derive(Default)]
pub struct DataCache {
    data: HashMap<(usize, usize), GxData>,
    tluts: HashMap<u32, TlutObj>,
}

impl DataCache {
    fn get(&mut self, s: &[u8]) -> GxData {
        self.data.entry((s.as_ptr() as usize, s.len())).or_insert_with(|| GxData::from(s)).clone()
    }
}

fn col(c: Rgba) -> GxColor {
    GxColor::new(c[0], c[1], c[2], c[3])
}

impl GxBackend for ShimBackend<'_> {
    fn set_cull_mode(&mut self, mode: u32) {
        self.shim.set_cull_mode(mode)
    }
    fn set_blend_mode(&mut self, ty: u32, src: u32, dst: u32, op: u32) {
        self.shim.set_blend_mode(ty, src, dst, op)
    }
    fn set_z_mode(&mut self, enable: bool, func: u32, update: bool) {
        self.shim.set_z_mode(enable, func, update)
    }
    fn set_alpha_compare(&mut self, c0: u32, r0: u8, op: u32, c1: u32, r1: u8) {
        self.shim.set_alpha_compare(c0, r0, op, c1, r1)
    }
    fn set_color_update(&mut self, on: bool) {
        self.shim.set_color_update(on)
    }
    fn set_alpha_update(&mut self, on: bool) {
        self.shim.set_alpha_update(on)
    }
    fn set_dst_alpha(&mut self, on: bool, a: u8) {
        self.shim.set_dst_alpha(on, a)
    }
    fn set_z_comp_loc(&mut self, before_tex: bool) {
        self.shim.set_z_comp_loc(before_tex)
    }
    fn set_dither(&mut self, on: bool) {
        self.shim.set_dither(on)
    }
    fn set_num_chans(&mut self, n: u8) {
        self.shim.set_num_chans(n)
    }
    fn set_chan_amb_color(&mut self, chan: u32, c: Rgba) {
        self.shim.set_chan_amb_color(chan, col(c))
    }
    fn set_chan_mat_color(&mut self, chan: u32, c: Rgba) {
        self.shim.set_chan_mat_color(chan, col(c))
    }
    fn set_chan_ctrl(&mut self, chan: u32, enable: bool, amb: u32, mat: u32, mask: u32, diff: u32, attn: u32) {
        self.shim.set_chan_ctrl(chan, enable, amb, mat, mask, diff, attn)
    }
    fn set_num_tex_gens(&mut self, n: u8) {
        self.shim.set_num_tex_gens(n)
    }
    fn set_num_tev_stages(&mut self, n: u8) {
        self.shim.set_num_tev_stages(n)
    }
    fn set_tev_order(&mut self, stage: u32, coord: u32, map: u32, color: u32) {
        self.shim.set_tev_order(stage, coord, map, color)
    }
    fn set_tev_op(&mut self, stage: u32, mode: u32) {
        self.shim.set_tev_op(stage, mode)
    }
    fn set_tev_swap_mode(&mut self, stage: u32, ras: u32, tex: u32) {
        self.shim.set_tev_swap_mode(stage, ras, tex)
    }
    fn set_tev_color_in(&mut self, stage: u32, a: u32, b: u32, c: u32, d: u32) {
        self.shim.set_tev_color_in(stage, a, b, c, d)
    }
    fn set_tev_alpha_in(&mut self, stage: u32, a: u32, b: u32, c: u32, d: u32) {
        self.shim.set_tev_alpha_in(stage, a, b, c, d)
    }
    fn set_tev_color_op(&mut self, stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32) {
        self.shim.set_tev_color_op(stage, op, bias, scale, clamp, out)
    }
    fn set_tev_alpha_op(&mut self, stage: u32, op: u32, bias: u32, scale: u32, clamp: bool, out: u32) {
        self.shim.set_tev_alpha_op(stage, op, bias, scale, clamp, out)
    }
    fn set_tev_kcolor(&mut self, id: u32, c: Rgba) {
        self.shim.set_tev_kcolor(id, col(c))
    }
    fn set_tev_kcolor_sel(&mut self, stage: u32, sel: u32) {
        self.shim.set_tev_kcolor_sel(stage, sel)
    }
    fn set_tev_kalpha_sel(&mut self, stage: u32, sel: u32) {
        self.shim.set_tev_kalpha_sel(stage, sel)
    }
    fn set_tex_coord_gen2(&mut self, coord: u32, func: u32, src: u32, mtx: u32, normalize: bool, pt: u32) {
        self.shim.set_tex_coord_gen2(coord, func, src, mtx, normalize, pt)
    }
    fn load_tex_mtx_imm(&mut self, m: &Mtx, id: u32, ty: u32) {
        self.shim.load_tex_mtx_imm(m, id, ty)
    }
    fn load_pos_mtx_imm(&mut self, m: &Mtx, id: u32) {
        self.shim.load_pos_mtx_imm(m, id)
    }
    fn load_nrm_mtx_imm(&mut self, m: &Mtx, id: u32) {
        self.shim.load_nrm_mtx_imm(m, id)
    }
    fn set_current_mtx(&mut self, id: u32) {
        self.shim.set_current_mtx(id)
    }
    fn clear_vtx_desc(&mut self) {
        self.shim.clear_vtx_desc()
    }
    fn set_vtx_desc(&mut self, attr: u32, ty: u32) {
        self.shim.set_vtx_desc(attr, ty)
    }
    fn set_vtx_attr_fmt(&mut self, fmt: u32, attr: u32, cnt: u32, ty: u32, frac: u8) {
        self.shim.set_vtx_attr_fmt(fmt, attr, cnt, ty, frac)
    }
    fn set_array(&mut self, attr: u32, data: &[u8], stride: u8) {
        let d = self.cache.get(data);
        self.shim.set_array(attr, Some(&d), stride)
    }
    fn call_display_list(&mut self, list: &[u8]) {
        self.shim.call_display_list(list)
    }
    fn load_tlut(&mut self, key: u32, lut: &[u8], fmt: u32, n: u16, name: u32) {
        let data = self.cache.get(&lut[..lut.len().min(n as usize * 2)]);
        let mut o = TlutObj::default();
        self.shim.init_tlut_obj(&mut o, Some(data), fmt, n);
        self.shim.load_tlut(&o, name);
        self.cache.tluts.insert(key, o);
    }
    fn load_texture(&mut self, t: &TexLoad) {
        let max_lod = if t.mipmap { t.max_lod.max(1.0) as u8 } else { 0 };
        let size = get_tex_buffer_size(t.width, t.height, t.format, t.mipmap, max_lod).max(32) as usize;
        let data = self.cache.get(&t.image[..t.image.len().min(size)]);
        let mut o = TexObj::default();
        match t.tlut_name {
            Some(name) => self.shim.init_tex_obj_ci(&mut o, Some(data), t.width, t.height, t.format, t.wrap_s, t.wrap_t, t.mipmap, name),
            None => self.shim.init_tex_obj(&mut o, Some(data), t.width, t.height, t.format, t.wrap_s, t.wrap_t, t.mipmap),
        }
        GxShim::init_tex_obj_lod(&mut o, t.min_filt, t.mag_filt, t.min_lod, t.max_lod, t.lod_bias, t.bias_clamp, t.edge_lod, t.max_aniso);
        self.shim.load_tex_obj(&mut o, t.map);
    }
    fn load_light(&mut self, id: u32, l: &LightLoad) {
        let mut o = GxLightObj::default();
        GxShim::init_light_color(&mut o, col(l.color));
        GxShim::init_light_pos(&mut o, l.pos[0], l.pos[1], l.pos[2]);
        GxShim::init_light_dir(&mut o, l.dir[0], l.dir[1], l.dir[2]);
        let a = l.attn;
        GxShim::init_light_attn(&mut o, a[0], a[1], a[2], a[3], a[4], a[5]);
        self.shim.load_light_obj_imm(&o, id);
    }
}
