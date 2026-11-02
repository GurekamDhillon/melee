//! TEV -> WGSL: a port of Aurora's `lib/gx/shader.cpp` (`build_shader_source`), `shader_info.cpp`
//! (`build_shader_info`, `fill_uniform`), `attr_fmt.cpp` and `gx.cpp::populate_pipeline_config`.
//!
//! The generated text is meant to be byte-identical to Aurora's for the same [`ShaderConfig`]
//! (tests/oracle.rs, `shader`: FNV-1a of the WGSL for hundreds of random configs), so the same
//! vertex-pulling layout is used: raw vertex bytes in `vbuf`, attribute arrays in `abuf` (both
//! storage `array<u32>`), per-draw immediates, one uniform block. GD surface programs
//! (`surfaceProgram`) and the PC matrix palette's runtime are not ported (config fields kept).
use crate::consts::*;
use crate::state::*;
use std::fmt::Write as _;

/// Aurora `AttrConfig`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AttrConfig {
    pub attr_type: u8,
    pub cnt: u8,
    pub comp_type: u8,
    pub offset: u8,
    pub stride: u8,
    pub frac: u8,
    pub le: bool,
    pub nbt3: bool,
}
impl Default for AttrConfig {
    fn default() -> Self {
        Self { attr_type: GX_NONE as u8, cnt: 0xFF, comp_type: 0xFF, offset: 0, stride: 0, frac: 0, le: true, nbt3: false }
    }
}

/// Aurora `ShaderConfig`: everything the generated WGSL depends on.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderConfig {
    pub fog_type: u8,
    pub vtx_stride: u8,
    /// 1 = GX_LINES, 2 = GX_LINESTRIP, 3 = GX_POINTS.
    pub line_mode: u8,
    pub fog_range_enabled: bool,
    pub pn_palette: bool,
    pub attrs: [AttrConfig; MAX_VTX_ATTR],
    pub tev_swap_table: [TevSwap; MAX_TEV_SWAP],
    pub tev_stages: [TevStage; MAX_TEV_STAGES],
    pub tev_stage_count: u32,
    pub color_channels: [ColorChannelConfig; MAX_COLOR_CHANNELS],
    pub tcgs: [TcgConfig; MAX_TEX_COORD],
    pub alpha_compare: AlphaCompare,
    pub ind_stages: [IndStage; MAX_IND_STAGES],
    pub num_ind_stages: u32,
}
impl Default for ShaderConfig {
    fn default() -> Self {
        Self {
            fog_type: GX_FOG_NONE as u8,
            vtx_stride: 0,
            line_mode: 0,
            fog_range_enabled: false,
            pn_palette: false,
            attrs: [AttrConfig::default(); MAX_VTX_ATTR],
            tev_swap_table: [TevSwap::default(); MAX_TEV_SWAP],
            tev_stages: [TevStage::default(); MAX_TEV_STAGES],
            tev_stage_count: 0,
            color_channels: [ColorChannelConfig::default(); MAX_COLOR_CHANNELS],
            tcgs: [TcgConfig::default(); MAX_TEX_COORD],
            alpha_compare: AlphaCompare::default(),
            ind_stages: [IndStage::default(); MAX_IND_STAGES],
            num_ind_stages: 0,
        }
    }
}

/// Aurora `PipelineConfig` (fixed-function state + the shader config).
#[derive(Clone, Debug, PartialEq)]
pub struct PipelineConfig {
    pub shader: ShaderConfig,
    pub depth_func: u32,
    pub cull_mode: u32,
    pub blend_mode: u32,
    pub blend_fac_src: u32,
    pub blend_fac_dst: u32,
    pub blend_op: u32,
    pub dst_alpha: u32,
    pub polygon_offset: f32,
    pub polygon_offset_scale: f32,
    pub polygon_offset_clamp: f32,
    pub depth_compare: bool,
    pub depth_update: bool,
    pub alpha_update: bool,
    pub color_update: bool,
}

/// Aurora `ShaderInfo`; bitsets are masks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderInfo {
    pub sampled_tex_coords: u32,
    pub sampled_textures: u32,
    pub sampled_kcolors: u32,
    pub sampled_color_channels: u32,
    pub loads_tev_reg: u32,
    pub writes_tev_reg: u32,
    pub uses_pt_tex_mtx: u32,
    pub index_attr: u32,
    pub used_ind_stages: u32,
    pub sampled_ind_textures: u32,
    pub used_ind_tex_mtxs: u32,
    /// Bytes `fill_uniform` writes, aligned to the uniform offset alignment (256).
    pub uniform_size: u32,
    pub uses_fog: bool,
    pub lighting_enabled: bool,
    pub line_mode: u8,
}

pub const MAX_UNIFORM_SIZE: u32 = 3840;
pub const UNIFORM_ALIGN: u32 = 256;

#[inline]
fn bit(m: u32, i: u32) -> bool {
    m & (1 << i) != 0
}

// ----------------------------------------------------------------------------- attr_fmt.cpp

/// Aurora `comp_type_size`.
pub fn comp_type_size(attr: u32, ty: u32) -> u8 {
    match attr {
        a if a <= GX_VA_TEX7MTXIDX => 1,
        GX_VA_CLR0 | GX_VA_CLR1 => match ty {
            GX_RGB565 | GX_RGBA4 => 2,
            GX_RGB8 | GX_RGBA6 => 3,
            GX_RGBX8 | GX_RGBA8 => 4,
            _ => panic!("comp_type_size: bad color type {ty}"),
        },
        _ => match ty {
            GX_U8 | GX_S8 => 1,
            GX_U16 | GX_S16 => 2,
            GX_F32 => 4,
            _ => panic!("comp_type_size: Unsupported component type {ty}"),
        },
    }
}

/// Aurora `comp_cnt_count`.
pub fn comp_cnt_count(attr: u32, cnt: u32) -> u8 {
    match attr {
        a if a <= GX_VA_TEX7MTXIDX => 1,
        GX_VA_POS => match cnt {
            GX_POS_XY => 2,
            GX_POS_XYZ => 3,
            _ => panic!("comp_cnt_count: Unsupported attr/cnt {attr} {cnt}"),
        },
        GX_VA_NRM => match cnt {
            GX_NRM_XYZ => 3,
            GX_NRM_NBT | GX_NRM_NBT3 => 9,
            _ => panic!("comp_cnt_count: Unsupported attr/cnt {attr} {cnt}"),
        },
        GX_VA_CLR0 | GX_VA_CLR1 => 1,
        a if (GX_VA_TEX0..=GX_VA_TEX7).contains(&a) => match cnt {
            GX_TEX_S => 1,
            GX_TEX_ST => 2,
            _ => panic!("comp_cnt_count: Unsupported attr/cnt {attr} {cnt}"),
        },
        _ => panic!("comp_cnt_count: Unsupported attr/cnt {attr} {cnt}"),
    }
}

/// Aurora `color_channel`.
pub fn color_channel(id: u32) -> u32 {
    match id {
        GX_COLOR0 | GX_ALPHA0 | GX_COLOR0A0 => 0,
        GX_COLOR1 | GX_ALPHA1 | GX_COLOR1A1 => 1,
        _ => panic!("unimplemented color channel {id}"),
    }
}

// ----------------------------------------------------------------------------- populate_pipeline_config

/// Aurora `populate_pipeline_config(config, primitive, fmt)` (without the PC matrix palette, whose
/// `g_palette` is never active here).
pub fn populate_pipeline_config(st: &GxState, primitive: u32, fmt: u32) -> PipelineConfig {
    let vf = &st.vtx_fmts[fmt as usize];
    let mut sc = ShaderConfig { fog_type: st.fog.ty as u8, fog_range_enabled: st.fog.range_enabled, ..Default::default() };
    let mut off: u32 = 0;
    for i in GX_VA_PNMTXIDX..=GX_VA_TEX7 {
        let ty = st.vtx_desc[i as usize];
        if ty == GX_NONE {
            sc.attrs[i as usize] = AttrConfig::default();
            continue;
        }
        let af = vf[i as usize];
        let cnt = comp_cnt_count(i, af.cnt);
        let nbt3 = i == GX_VA_NRM && af.cnt == GX_NRM_NBT3;
        let mut m = AttrConfig {
            attr_type: ty as u8,
            cnt,
            comp_type: af.ty as u8,
            offset: off as u8,
            stride: 0,
            frac: af.frac,
            le: false,
            nbt3,
        };
        match ty {
            GX_DIRECT => off += comp_type_size(i, af.ty) as u32 * cnt as u32,
            GX_INDEX8 => {
                m.stride = st.arrays[i as usize].stride;
                m.le = st.arrays[i as usize].le;
                off += if nbt3 { 3 } else { 1 };
            }
            GX_INDEX16 => {
                m.stride = st.arrays[i as usize].stride;
                m.le = st.arrays[i as usize].le;
                off += if nbt3 { 6 } else { 2 };
            }
            _ => panic!("populate_pipeline_config: Invalid vertex type {ty}"),
        }
        sc.attrs[i as usize] = m;
    }
    sc.vtx_stride = off as u8;
    sc.line_mode = match primitive {
        GX_LINES => 1,
        GX_LINESTRIP => 2,
        GX_POINTS => 3,
        _ => 0,
    };
    sc.pn_palette = false;
    sc.tev_swap_table = st.tev_swap_table;
    for i in 0..st.num_tev_stages as usize {
        sc.tev_stages[i] = st.tev_stages[i];
    }
    sc.tev_stage_count = st.num_tev_stages as u32;
    for i in 0..st.num_ind_stages as usize {
        sc.ind_stages[i] = st.ind_stages[i];
    }
    sc.num_ind_stages = st.num_ind_stages as u32;
    for i in 0..MAX_COLOR_CHANNELS {
        let cc = st.color_channel_config[i];
        sc.color_channels[i] =
            if cc.lighting_enabled { cc } else { ColorChannelConfig { mat_src: cc.mat_src, ..Default::default() } };
    }
    for i in 0..st.num_tex_gens as usize {
        sc.tcgs[i] = st.tcgs[i];
    }
    if st.alpha_compare.enabled() {
        sc.alpha_compare = st.alpha_compare;
    }
    let cull = if sc.line_mode == 0 { st.cull_mode } else { GX_CULL_NONE };
    let (po, ps) = if cull == GX_CULL_FRONT { (st.back_offset, st.back_scale) } else { (st.front_offset, st.front_scale) };
    PipelineConfig {
        shader: sc,
        depth_func: st.depth_func,
        cull_mode: cull,
        blend_mode: st.blend_mode,
        blend_fac_src: st.blend_fac_src,
        blend_fac_dst: st.blend_fac_dst,
        blend_op: st.blend_op,
        dst_alpha: st.dst_alpha,
        polygon_offset: po,
        polygon_offset_scale: ps,
        polygon_offset_clamp: st.clamp,
        depth_compare: st.depth_compare,
        depth_update: st.depth_update,
        alpha_update: st.alpha_update,
        color_update: st.color_update,
    }
}

// ----------------------------------------------------------------------------- shader_info.cpp

fn is_alpha_bump_channel(id: u32) -> bool {
    id == GX_ALPHA_BUMP || id == GX_ALPHA_BUMPN
}

fn color_arg_reg_info(arg: u32, s: &TevStage, info: &mut ShaderInfo) {
    let load = |info: &mut ShaderInfo, r: u32| {
        if !bit(info.writes_tev_reg, r) {
            info.loads_tev_reg |= 1 << r;
        }
    };
    match arg {
        GX_CC_CPREV | GX_CC_APREV => load(info, GX_TEVPREV),
        GX_CC_C0 | GX_CC_A0 => load(info, GX_TEVREG0),
        GX_CC_C1 | GX_CC_A1 => load(info, GX_TEVREG1),
        GX_CC_C2 | GX_CC_A2 => load(info, GX_TEVREG2),
        GX_CC_TEXC | GX_CC_TEXA => {
            if s.tex_map_id != GX_TEXMAP_NULL {
                assert!(s.tex_coord_id != GX_TEXCOORD_NULL, "tex coord not bound");
                info.sampled_tex_coords |= 1 << s.tex_coord_id;
                info.sampled_textures |= 1 << s.tex_map_id;
            }
        }
        GX_CC_RASC | GX_CC_RASA => {
            if s.channel_id != GX_COLOR_NULL && s.channel_id != GX_COLOR_ZERO && !is_alpha_bump_channel(s.channel_id) {
                info.sampled_color_channels |= 1 << color_channel(s.channel_id);
            }
        }
        GX_CC_KONST => {
            let k = match s.kc_sel {
                GX_TEV_KCSEL_K0 | GX_TEV_KCSEL_K0_R | GX_TEV_KCSEL_K0_G | GX_TEV_KCSEL_K0_B | GX_TEV_KCSEL_K0_A => Some(0),
                GX_TEV_KCSEL_K1 | GX_TEV_KCSEL_K1_R | GX_TEV_KCSEL_K1_G | GX_TEV_KCSEL_K1_B | GX_TEV_KCSEL_K1_A => Some(1),
                GX_TEV_KCSEL_K2 | GX_TEV_KCSEL_K2_R | GX_TEV_KCSEL_K2_G | GX_TEV_KCSEL_K2_B | GX_TEV_KCSEL_K2_A => Some(2),
                GX_TEV_KCSEL_K3 | GX_TEV_KCSEL_K3_R | GX_TEV_KCSEL_K3_G | GX_TEV_KCSEL_K3_B | GX_TEV_KCSEL_K3_A => Some(3),
                _ => None,
            };
            if let Some(k) = k {
                info.sampled_kcolors |= 1 << k;
            }
        }
        _ => {}
    }
}

fn alpha_arg_reg_info(arg: u32, s: &TevStage, info: &mut ShaderInfo) {
    let load = |info: &mut ShaderInfo, r: u32| {
        if !bit(info.writes_tev_reg, r) {
            info.loads_tev_reg |= 1 << r;
        }
    };
    match arg {
        GX_CA_APREV => load(info, GX_TEVPREV),
        GX_CA_A0 => load(info, GX_TEVREG0),
        GX_CA_A1 => load(info, GX_TEVREG1),
        GX_CA_A2 => load(info, GX_TEVREG2),
        GX_CA_TEXA => {
            if s.tex_map_id != GX_TEXMAP_NULL {
                assert!(s.tex_coord_id != GX_TEXCOORD_NULL, "tex coord not bound");
                info.sampled_tex_coords |= 1 << s.tex_coord_id;
                info.sampled_textures |= 1 << s.tex_map_id;
            }
        }
        GX_CA_RASA => {
            if s.channel_id != GX_COLOR_NULL && s.channel_id != GX_COLOR_ZERO && !is_alpha_bump_channel(s.channel_id) {
                info.sampled_color_channels |= 1 << color_channel(s.channel_id);
            }
        }
        GX_CA_KONST => {
            let k = match s.ka_sel {
                GX_TEV_KASEL_K0_R | GX_TEV_KASEL_K0_G | GX_TEV_KASEL_K0_B | GX_TEV_KASEL_K0_A => Some(0),
                GX_TEV_KASEL_K1_R | GX_TEV_KASEL_K1_G | GX_TEV_KASEL_K1_B | GX_TEV_KASEL_K1_A => Some(1),
                GX_TEV_KASEL_K2_R | GX_TEV_KASEL_K2_G | GX_TEV_KASEL_K2_B | GX_TEV_KASEL_K2_A => Some(2),
                GX_TEV_KASEL_K3_R | GX_TEV_KASEL_K3_G | GX_TEV_KASEL_K3_B | GX_TEV_KASEL_K3_A => Some(3),
                _ => None,
            };
            if let Some(k) = k {
                info.sampled_kcolors |= 1 << k;
            }
        }
        _ => {}
    }
}

/// Aurora `build_shader_info` (uniform size aligned to [`UNIFORM_ALIGN`]).
pub fn build_shader_info(c: &ShaderConfig) -> ShaderInfo {
    let mut info = ShaderInfo { uniform_size: 8 + 8 + 64, ..Default::default() };
    if c.line_mode != 0 {
        info.uniform_size += 16;
        info.line_mode = c.line_mode;
    }
    for (a, m) in c.attrs.iter().enumerate() {
        let a = a as u32;
        let t = m.attr_type as u32;
        if (a == GX_VA_PNMTXIDX || (GX_VA_TEX0MTXIDX..=GX_VA_TEX7MTXIDX).contains(&a)) && t == GX_DIRECT {
            info.index_attr |= 1 << a;
        } else if t == GX_INDEX8 || t == GX_INDEX16 {
            info.index_attr |= 1 << a;
        }
    }
    info.uniform_size += 48 * 30;
    for s in &c.tev_stages[..c.tev_stage_count as usize] {
        for a in [s.color_pass.a, s.color_pass.b, s.color_pass.c, s.color_pass.d] {
            color_arg_reg_info(a, s, &mut info);
        }
        info.writes_tev_reg |= 1 << s.color_op.out_reg;
        for a in [s.alpha_pass.a, s.alpha_pass.b, s.alpha_pass.c, s.alpha_pass.d] {
            alpha_arg_reg_info(a, s, &mut info);
        }
        if !bit(info.writes_tev_reg, s.alpha_op.out_reg) {
            info.loads_tev_reg |= 1 << s.alpha_op.out_reg;
            info.writes_tev_reg |= 1 << s.alpha_op.out_reg;
        }
    }
    for s in &c.tev_stages[..c.tev_stage_count as usize] {
        if s.ind_tex_stage >= c.num_ind_stages {
            continue;
        }
        let uses_ind = s.ind_tex_mtx_id != GX_ITM_OFF || is_alpha_bump_channel(s.channel_id);
        let uses_tc = s.ind_tex_mtx_id != GX_ITM_OFF
            || s.ind_tex_wrap_s != GX_ITW_OFF
            || s.ind_tex_wrap_t != GX_ITW_OFF
            || s.ind_tex_add_prev;
        if uses_tc && s.tex_coord_id != GX_TEXCOORD_NULL {
            info.sampled_tex_coords |= 1 << s.tex_coord_id;
        }
        if uses_tc && s.tex_map_id != GX_TEXMAP_NULL && (s.ind_tex_mtx_id != GX_ITM_OFF || s.ind_tex_add_prev) {
            info.sampled_textures |= 1 << s.tex_map_id;
        }
        if !uses_ind {
            continue;
        }
        info.used_ind_stages |= 1 << s.ind_tex_stage;
        let is = c.ind_stages[s.ind_tex_stage as usize];
        info.sampled_textures |= 1 << is.tex_map_id;
        info.sampled_tex_coords |= 1 << is.tex_coord_id;
        info.sampled_ind_textures |= 1 << is.tex_map_id;
        let m = s.ind_tex_mtx_id;
        if (GX_ITM_0..=GX_ITM_2).contains(&m) {
            info.used_ind_tex_mtxs |= 1 << (m - GX_ITM_0);
        } else if (GX_ITM_S0..=GX_ITM_S2).contains(&m) {
            info.used_ind_tex_mtxs |= 1 << (m - GX_ITM_S0);
        } else if (GX_ITM_T0..=GX_ITM_T2).contains(&m) {
            info.used_ind_tex_mtxs |= 1 << (m - GX_ITM_T0);
        }
    }
    for i in 0..MAX_TEX_COORD as u32 {
        if !bit(info.sampled_tex_coords, i) {
            continue;
        }
        let t = c.tcgs[i as usize];
        if (GX_TG_BUMP0..=GX_TG_BUMP7).contains(&t.ty) {
            info.sampled_tex_coords |= 1 << t.emboss_src;
            info.lighting_enabled = true;
        }
    }
    info.uniform_size += info.loads_tev_reg.count_ones() * 16;
    for i in 0..2u32 {
        if bit(info.sampled_color_channels, i) {
            let cc = c.color_channels[i as usize];
            let cca = c.color_channels[(i + GX_ALPHA0) as usize];
            if cc.lighting_enabled || cca.lighting_enabled {
                info.lighting_enabled = true;
            }
        }
    }
    if info.lighting_enabled {
        info.uniform_size += 16 + 80 * MAX_LIGHTS as u32;
    }
    for i in 0..2u32 {
        if bit(info.sampled_color_channels, i) {
            let cc = c.color_channels[i as usize];
            if cc.lighting_enabled && cc.amb_src == GX_SRC_REG {
                info.uniform_size += 16;
            }
            if cc.mat_src == GX_SRC_REG {
                info.uniform_size += 16;
            }
            let cca = c.color_channels[(i + GX_ALPHA0) as usize];
            if cca.lighting_enabled && cca.amb_src == GX_SRC_REG {
                info.uniform_size += 16;
            }
            if cca.mat_src == GX_SRC_REG {
                info.uniform_size += 16;
            }
        }
    }
    info.uniform_size += info.sampled_kcolors.count_ones() * 16;
    for i in 0..MAX_TEX_COORD as u32 {
        if bit(info.sampled_tex_coords, i) {
            let t = c.tcgs[i as usize];
            if t.post_mtx != GX_PTIDENTITY {
                info.uses_pt_tex_mtx |= 1 << ((t.post_mtx - GX_PTTEXMTX0) / 3);
            }
        }
    }
    if info.uses_pt_tex_mtx != 0 {
        info.uniform_size += 48 * MAX_PT_TEX_MTX as u32;
    }
    if c.fog_type as u32 != GX_FOG_NONE {
        info.uses_fog = true;
        info.uniform_size += 80;
    }
    info.uniform_size += MAX_TEX_COORD as u32 * 16;
    if info.used_ind_tex_mtxs != 0 {
        info.uniform_size += MAX_IND_TEX_MTXS as u32 * 32;
    }
    info.uniform_size += info.sampled_textures.count_ones() * 16;
    info.uniform_size = info.uniform_size.div_ceil(UNIFORM_ALIGN) * UNIFORM_ALIGN;
    assert!(info.uniform_size <= MAX_UNIFORM_SIZE, "Uniform size exceeds maximum: {}", info.uniform_size);
    info
}

/// Per-texture uniform: (width, height, lod bias, 0). Aurora's `texture_size_bias` without the
/// replacement-texture LOD term (no texture replacements here).
pub fn texture_size_bias(obj: &TexObj) -> Vec4 {
    [obj.width() as f32, obj.height() as f32, obj.lod_bias(), 0.0]
}

fn tex_offset(o: u32) -> f32 {
    match o {
        GX_TO_SIXTEENTH => 1.0 / 16.0,
        GX_TO_EIGHTH => 1.0 / 8.0,
        GX_TO_FOURTH => 1.0 / 4.0,
        GX_TO_HALF => 1.0 / 2.0,
        GX_TO_ONE => 1.0,
        _ => 0.0,
    }
}

/// Aurora `fill_uniform`: the uniform block for `info` from the current state. `textures[i]` is the
/// object bound to map i (for `texture_size_bias`).
pub fn fill_uniform(st: &GxState, info: &ShaderInfo, textures: &[TexObj; MAX_TEXTURES]) -> Vec<u8> {
    let mut b: Vec<u8> = Vec::with_capacity(info.uniform_size as usize);
    let f = |b: &mut Vec<u8>, v: f32| b.extend_from_slice(&v.to_le_bytes());
    let v4 = |b: &mut Vec<u8>, v: &Vec4| v.iter().for_each(|x| b.extend_from_slice(&x.to_le_bytes()));
    let m34 = |b: &mut Vec<u8>, m: &Mat3x4| m.iter().for_each(|r| r.iter().for_each(|x| b.extend_from_slice(&x.to_le_bytes())));
    f(&mut b, st.render_viewport.width);
    f(&mut b, st.render_viewport.height);
    f(&mut b, st.logical_viewport.width);
    f(&mut b, st.logical_viewport.height);
    if info.line_mode != 0 {
        if info.line_mode == 3 {
            f(&mut b, st.point_size as f32 / 6.0);
            f(&mut b, 1.0);
            f(&mut b, tex_offset(st.point_tex_offset));
            let mask = (0..8).filter(|&i| st.tex_coord_scales[i].point_offset).fold(0u32, |m, i| m | 1 << i);
            b.extend_from_slice(&mask.to_le_bytes());
        } else {
            f(&mut b, st.line_width as f32 / 6.0);
            f(&mut b, if st.line_half_aspect { 0.5 } else { 1.0 });
            f(&mut b, tex_offset(st.line_tex_offset));
            let mask = (0..8).filter(|&i| st.tex_coord_scales[i].line_offset).fold(0u32, |m, i| m | 1 << i);
            b.extend_from_slice(&mask.to_le_bytes());
        }
    }
    let mut proj = st.proj;
    if USE_REVERSED_Z {
        for x in proj[2].iter_mut() {
            *x *= -1.0;
        }
    }
    proj.iter().for_each(|r| v4(&mut b, r));
    for m in &st.pn_mtx {
        m34(&mut b, &m.pos);
    }
    for m in &st.tex_mtxs {
        m34(&mut b, m);
    }
    for m in &st.pn_mtx {
        m34(&mut b, &m.nrm);
    }
    for i in 0..MAX_TEV_REGS as u32 {
        if bit(info.loads_tev_reg, i) {
            v4(&mut b, &st.color_regs[i as usize]);
        }
    }
    if info.lighting_enabled {
        for l in &st.lights {
            v4(&mut b, &l.pos);
            v4(&mut b, &l.dir);
            v4(&mut b, &l.color);
            v4(&mut b, &l.cos_att);
            v4(&mut b, &l.dist_att);
        }
        for i in 0..4 {
            b.extend_from_slice(&st.color_channel_state[i].light_mask.to_le_bytes());
        }
    }
    for i in 0..2usize {
        if !bit(info.sampled_color_channels, i as u32) {
            continue;
        }
        let ccc = st.color_channel_config[i];
        let ccs = st.color_channel_state[i];
        if ccc.lighting_enabled && ccc.amb_src == GX_SRC_REG {
            v4(&mut b, &ccs.amb_color);
        }
        if ccc.mat_src == GX_SRC_REG {
            v4(&mut b, &ccs.mat_color);
        }
        let ccca = st.color_channel_config[i + GX_ALPHA0 as usize];
        let ccsa = st.color_channel_state[i + GX_ALPHA0 as usize];
        if ccca.lighting_enabled && ccca.amb_src == GX_SRC_REG {
            v4(&mut b, &ccsa.amb_color);
        }
        if ccca.mat_src == GX_SRC_REG {
            v4(&mut b, &ccsa.mat_color);
        }
    }
    for i in 0..MAX_KCOLORS as u32 {
        if bit(info.sampled_kcolors, i) {
            v4(&mut b, &st.kcolors[i as usize]);
        }
    }
    if info.uses_pt_tex_mtx != 0 {
        // Aurora loops over the bitset's size (20) and appends every post matrix.
        for m in &st.pt_tex_mtxs {
            m34(&mut b, m);
        }
    }
    if info.uses_fog {
        let fs = &st.fog;
        let lw = st.logical_viewport.width.max(1.0);
        let rw = st.render_viewport.width.max(1.0);
        v4(&mut b, &fs.color);
        f(&mut b, fs.a);
        f(&mut b, fs.b);
        f(&mut b, fs.c);
        let center =
            ((fs.range_center as f32 - st.logical_viewport.left) / lw) * 2.0 - 1.0 + (st.render_viewport.left / rw) * 2.0;
        f(&mut b, center);
        let mut rk = [[0.0f32; 4]; 3];
        for i in 0..10usize {
            let src = (i & !1) | (1 - (i & 1));
            rk[i / 4][i % 4] = fs.range_k[src] as f32 / 64.0;
        }
        rk[2][2] = rk[2][1];
        rk[2][3] = rk[2][1];
        rk.iter().for_each(|r| v4(&mut b, r));
    }
    for s in &st.tex_coord_scales {
        v4(&mut b, &[s.scale_s as f32 + 1.0, s.scale_t as f32 + 1.0, 0.0, 0.0]);
    }
    if info.used_ind_tex_mtxs != 0 {
        for m in &st.ind_tex_mtxs {
            v4(&mut b, &[m.mtx[0][0], m.mtx[0][1], m.mtx[1][0], m.mtx[1][1]]);
            v4(&mut b, &[m.mtx[2][0], m.mtx[2][1], (m.scale_exp as f32).exp2(), 0.0]);
        }
    }
    for i in 0..MAX_TEXTURES as u32 {
        if bit(info.sampled_textures, i) {
            v4(&mut b, &texture_size_bias(&textures[i as usize]));
        }
    }
    b
}

// ----------------------------------------------------------------------------- shader.cpp

fn chan_comp(c: u32) -> &'static str {
    match c {
        GX_CH_RED => "r",
        GX_CH_GREEN => "g",
        GX_CH_BLUE => "b",
        GX_CH_ALPHA => "a",
        _ => "?",
    }
}

fn tev_mask_expr(value: &str, mask: u32) -> String {
    format!("(f32(u32({value}) & 0x{mask:X}u) / 255.0)")
}

fn alpha_bump_sel(c: &ShaderConfig, s: &TevStage) -> String {
    if s.ind_tex_stage >= c.num_ind_stages || s.ind_tex_alpha_sel == GX_ITBA_OFF {
        return "0.0".into();
    }
    let base = match s.ind_tex_alpha_sel {
        GX_ITBA_S => format!("t_IndTexCoord{}.x", s.ind_tex_stage),
        GX_ITBA_T => format!("t_IndTexCoord{}.y", s.ind_tex_stage),
        GX_ITBA_U => format!("t_IndTexCoord{}.z", s.ind_tex_stage),
        x => panic!("invalid indTexAlphaSel {x}"),
    };
    match s.ind_tex_format {
        GX_ITF_8 => tev_mask_expr(&base, 0xF8),
        GX_ITF_5 => tev_mask_expr(&base, 0xE0),
        GX_ITF_4 => tev_mask_expr(&base, 0xF0),
        GX_ITF_3 => tev_mask_expr(&base, 0xF8),
        x => panic!("invalid indirect format {x}"),
    }
}

fn uses_texture_sample(s: &TevStage) -> bool {
    if s.tex_map_id == GX_TEXMAP_NULL {
        return false;
    }
    let c = s.color_pass;
    let a = s.alpha_pass;
    [c.a, c.b, c.c, c.d].iter().any(|&x| x == GX_CC_TEXC || x == GX_CC_TEXA)
        || [a.a, a.b, a.c, a.d].iter().any(|&x| x == GX_CA_TEXA)
}

fn bump_alpha(c: &ShaderConfig, s: &TevStage) -> String {
    let mut a = alpha_bump_sel(c, s);
    if s.channel_id == GX_ALPHA_BUMPN {
        a = format!("({a} * (255.0 / 248.0))");
    }
    a
}

fn color_arg_reg(arg: u32, idx: usize, c: &ShaderConfig, s: &TevStage) -> String {
    match arg {
        GX_CC_CPREV => "prev.rgb".into(),
        GX_CC_APREV => "vec3f(prev.a)".into(),
        GX_CC_C0 => "tevreg0.rgb".into(),
        GX_CC_A0 => "vec3f(tevreg0.a)".into(),
        GX_CC_C1 => "tevreg1.rgb".into(),
        GX_CC_A1 => "vec3f(tevreg1.a)".into(),
        GX_CC_C2 => "tevreg2.rgb".into(),
        GX_CC_A2 => "vec3f(tevreg2.a)".into(),
        GX_CC_TEXC => {
            if s.tex_map_id == GX_TEXMAP_NULL {
                return "vec3f(1.0)".into();
            }
            assert!(s.tex_map_id <= GX_TEXMAP7, "invalid texture {} for stage {idx}", s.tex_map_id);
            let sw = c.tev_swap_table[s.tev_swap_tex as usize];
            format!("sampled{idx}.{}{}{}", chan_comp(sw.red), chan_comp(sw.green), chan_comp(sw.blue))
        }
        GX_CC_TEXA => {
            if s.tex_map_id == GX_TEXMAP_NULL {
                return "vec3f(1.0)".into();
            }
            assert!(s.tex_map_id <= GX_TEXMAP7, "invalid texture {} for stage {idx}", s.tex_map_id);
            let sw = c.tev_swap_table[s.tev_swap_tex as usize];
            format!("vec3f(sampled{idx}.{})", chan_comp(sw.alpha))
        }
        GX_CC_RASC => {
            if s.channel_id == GX_COLOR_ZERO || s.channel_id == GX_COLOR_NULL {
                return "vec3f(0.0)".into();
            }
            if is_alpha_bump_channel(s.channel_id) {
                return format!("vec3f({})", bump_alpha(c, s));
            }
            let ci = color_channel(s.channel_id);
            let sw = c.tev_swap_table[s.tev_swap_ras as usize];
            format!("rast{ci}.{}{}{}", chan_comp(sw.red), chan_comp(sw.green), chan_comp(sw.blue))
        }
        GX_CC_RASA => {
            if s.channel_id == GX_COLOR_ZERO || s.channel_id == GX_COLOR_NULL {
                return "vec3f(0.0)".into();
            }
            if is_alpha_bump_channel(s.channel_id) {
                return format!("vec3f({})", bump_alpha(c, s));
            }
            let ci = color_channel(s.channel_id);
            let sw = c.tev_swap_table[s.tev_swap_ras as usize];
            format!("vec3f(rast{ci}.{})", chan_comp(sw.alpha))
        }
        GX_CC_ONE => "vec3f(1.0)".into(),
        GX_CC_HALF => "vec3f(0.5)".into(),
        GX_CC_KONST => match s.kc_sel {
            GX_TEV_KCSEL_8_8 => "vec3f(1.0)".into(),
            GX_TEV_KCSEL_7_8 => "vec3f(7.0/8.0)".into(),
            GX_TEV_KCSEL_6_8 => "vec3f(6.0/8.0)".into(),
            GX_TEV_KCSEL_5_8 => "vec3f(5.0/8.0)".into(),
            GX_TEV_KCSEL_4_8 => "vec3f(4.0/8.0)".into(),
            GX_TEV_KCSEL_3_8 => "vec3f(3.0/8.0)".into(),
            GX_TEV_KCSEL_2_8 => "vec3f(2.0/8.0)".into(),
            GX_TEV_KCSEL_1_8 => "vec3f(1.0/8.0)".into(),
            k @ GX_TEV_KCSEL_K0..=GX_TEV_KCSEL_K3 => format!("ubuf.kcolor{}.rgb", k - GX_TEV_KCSEL_K0),
            k @ GX_TEV_KCSEL_K0_R..=GX_TEV_KCSEL_K3_A => {
                let rel = k - GX_TEV_KCSEL_K0_R;
                format!("vec3f(ubuf.kcolor{}.{})", rel % 4, ["r", "g", "b", "a"][(rel / 4) as usize])
            }
            k => panic!("invalid kcSel {k}"),
        },
        GX_CC_ZERO => "vec3f(0.0)".into(),
        _ => panic!("invalid color arg {arg}"),
    }
}

fn alpha_arg_reg(arg: u32, idx: usize, c: &ShaderConfig, s: &TevStage) -> String {
    match arg {
        GX_CA_APREV => "prev.a".into(),
        GX_CA_A0 => "tevreg0.a".into(),
        GX_CA_A1 => "tevreg1.a".into(),
        GX_CA_A2 => "tevreg2.a".into(),
        GX_CA_TEXA => {
            if s.tex_map_id == GX_TEXMAP_NULL {
                return "1.0".into();
            }
            assert!(s.tex_map_id <= GX_TEXMAP7, "invalid texture {} for stage {idx}", s.tex_map_id);
            let sw = c.tev_swap_table[s.tev_swap_tex as usize];
            format!("sampled{idx}.{}", chan_comp(sw.alpha))
        }
        GX_CA_RASA => {
            if s.channel_id == GX_COLOR_ZERO || s.channel_id == GX_COLOR_NULL {
                return "0.0".into();
            }
            if is_alpha_bump_channel(s.channel_id) {
                return bump_alpha(c, s);
            }
            let ci = color_channel(s.channel_id);
            let sw = c.tev_swap_table[s.tev_swap_ras as usize];
            format!("rast{ci}.{}", chan_comp(sw.alpha))
        }
        GX_CA_KONST => match s.ka_sel {
            GX_TEV_KASEL_8_8 => "1.0".into(),
            GX_TEV_KASEL_7_8 => "(7.0/8.0)".into(),
            GX_TEV_KASEL_6_8 => "(6.0/8.0)".into(),
            GX_TEV_KASEL_5_8 => "(5.0/8.0)".into(),
            GX_TEV_KASEL_4_8 => "(4.0/8.0)".into(),
            GX_TEV_KASEL_3_8 => "(3.0/8.0)".into(),
            GX_TEV_KASEL_2_8 => "(2.0/8.0)".into(),
            GX_TEV_KASEL_1_8 => "(1.0/8.0)".into(),
            k @ GX_TEV_KASEL_K0_R..=GX_TEV_KASEL_K3_A => {
                let rel = k - GX_TEV_KASEL_K0_R;
                format!("ubuf.kcolor{}.{}", rel % 4, ["r", "g", "b", "a"][(rel / 4) as usize])
            }
            k => panic!("invalid kaSel {k}"),
        },
        GX_CA_ZERO => "0.0".into(),
        _ => panic!("invalid alpha arg {arg}"),
    }
}

fn color_arg_normalized(arg: u32, cn: &[bool; 4], an: &[bool; 4]) -> bool {
    match arg {
        GX_CC_CPREV => cn[0],
        GX_CC_APREV => an[0],
        GX_CC_C0 => cn[1],
        GX_CC_A0 => an[1],
        GX_CC_C1 => cn[2],
        GX_CC_A1 => an[2],
        GX_CC_C2 => cn[3],
        GX_CC_A2 => an[3],
        _ => true,
    }
}

fn alpha_arg_normalized(arg: u32, an: &[bool; 4]) -> bool {
    match arg {
        GX_CA_APREV => an[0],
        GX_CA_A0 => an[1],
        GX_CA_A1 => an[2],
        GX_CA_A2 => an[3],
        _ => true,
    }
}

#[allow(clippy::too_many_arguments)]
fn tev_op(op: u32, bias: &str, scale: &str, a: &str, b: &str, c: &str, d: &str, zero: &str) -> String {
    match op {
        GX_TEV_ADD | GX_TEV_SUB => {
            let neg = if op == GX_TEV_SUB { "-" } else { "" };
            format!("(({neg}mix({a}, {b}, {c}) + {d}){bias}){scale}")
        }
        GX_TEV_COMP_R8_GT => format!("select({zero}, {c}, round({a}.r * 255.0) > round({b}.r * 255.0)) + {d}"),
        GX_TEV_COMP_R8_EQ => format!("select({zero}, {c}, round({a}.r * 255.0) == round({b}.r * 255.0)) + {d}"),
        GX_TEV_COMP_GR16_GT => format!(
            "select({zero}, {c}, round(dot({a}.rg * 255.0, vec2(1.0, 256.0))) > round(dot({b}.rg * 255.0, vec2(1.0, 256.0)))) + {d}"
        ),
        GX_TEV_COMP_GR16_EQ => format!(
            "select({zero}, {c}, round(dot({a}.rg * 255.0, vec2(1.0, 256.0))) == round(dot({b}.rg * 255.0, vec2(1.0, 256.0)))) + {d}"
        ),
        GX_TEV_COMP_BGR24_GT => format!(
            "select({zero}, {c}, round(dot({a}.rgb * 255.0, vec3(1.0, 256.0, 65536.0))) > round(dot({b}.rgb * 255.0, vec3(1.0, 256.0, 65536.0)))) + {d}"
        ),
        GX_TEV_COMP_BGR24_EQ => format!(
            "select({zero}, {c}, round(dot({a}.rgb * 255.0, vec3(1.0, 256.0, 65536.0))) == round(dot({b}.rgb * 255.0, vec3(1.0, 256.0, 65536.0)))) + {d}"
        ),
        GX_TEV_COMP_RGB8_GT => format!("select({zero}, {c}, round({a} * 255.0) > round({b} * 255.0)) + {d}"),
        GX_TEV_COMP_RGB8_EQ => format!("select({zero}, {c}, round({a} * 255.0) == round({b} * 255.0)) + {d}"),
        _ => panic!("unimplemented tev op {op}"),
    }
}

fn tev_bias(b: u32) -> &'static str {
    match b {
        GX_TB_ZERO => "",
        GX_TB_ADDHALF => " + 0.5",
        GX_TB_SUBHALF => " - 0.5",
        _ => panic!("invalid tev bias {b}"),
    }
}

fn tev_scale(s: u32) -> &'static str {
    match s {
        GX_CS_SCALE_1 => "",
        GX_CS_SCALE_2 => " * 2.0",
        GX_CS_SCALE_4 => " * 4.0",
        GX_CS_DIVIDE_2 => " / 2.0",
        _ => panic!("invalid tev scale {s}"),
    }
}

/// Aurora `AlphaCompareExpr`: an expression, or a known constant (0 / 1).
#[derive(Clone)]
struct Ace {
    expr: String,
    constant: i32,
}
fn ace_const(v: bool) -> Ace {
    Ace { expr: if v { "true" } else { "false" }.into(), constant: v as i32 }
}
fn ace_not(e: &Ace) -> Ace {
    if e.constant != -1 {
        return ace_const(e.constant == 0);
    }
    Ace { expr: format!("!{}", e.expr), constant: -1 }
}
fn ace_and(l: &Ace, r: &Ace) -> Ace {
    if l.constant == 0 || r.constant == 0 {
        return ace_const(false);
    }
    if l.constant == 1 {
        return r.clone();
    }
    if r.constant == 1 {
        return l.clone();
    }
    Ace { expr: format!("({} && {})", l.expr, r.expr), constant: -1 }
}
fn ace_or(l: &Ace, r: &Ace) -> Ace {
    if l.constant == 1 || r.constant == 1 {
        return ace_const(true);
    }
    if l.constant == 0 {
        return r.clone();
    }
    if r.constant == 0 {
        return l.clone();
    }
    Ace { expr: format!("({} || {})", l.expr, r.expr), constant: -1 }
}
fn ace_xor(l: &Ace, r: &Ace) -> Ace {
    if l.constant != -1 && r.constant != -1 {
        return ace_const(l.constant != r.constant);
    }
    if l.constant == 0 {
        return r.clone();
    }
    if r.constant == 0 {
        return l.clone();
    }
    if l.constant == 1 {
        return ace_not(r);
    }
    if r.constant == 1 {
        return ace_not(l);
    }
    Ace { expr: format!("({} != {})", l.expr, r.expr), constant: -1 }
}
fn ace_xnor(l: &Ace, r: &Ace) -> Ace {
    if l.constant != -1 && r.constant != -1 {
        return ace_const(l.constant == r.constant);
    }
    if l.constant == 0 {
        return ace_not(r);
    }
    if r.constant == 0 {
        return ace_not(l);
    }
    if l.constant == 1 {
        return r.clone();
    }
    if r.constant == 1 {
        return l.clone();
    }
    Ace { expr: format!("({} == {})", l.expr, r.expr), constant: -1 }
}
fn alpha_compare(comp: u32, r: u32) -> Ace {
    let r8 = r as u8;
    let iref = r8 as u32;
    match comp {
        GX_NEVER => ace_const(false),
        GX_LESS => {
            if r8 == 0 {
                return ace_const(false);
            }
            Ace { expr: format!("(alphaCompare < {iref}u)"), constant: -1 }
        }
        GX_LEQUAL => {
            if r8 == 255 {
                return ace_const(true);
            }
            Ace { expr: format!("(alphaCompare <= {iref}u)"), constant: -1 }
        }
        GX_EQUAL => Ace { expr: format!("(alphaCompare == {iref}u)"), constant: -1 },
        GX_NEQUAL => Ace { expr: format!("(alphaCompare != {iref}u)"), constant: -1 },
        GX_GEQUAL => {
            if r8 == 0 {
                return ace_const(true);
            }
            Ace { expr: format!("(alphaCompare >= {iref}u)"), constant: -1 }
        }
        GX_GREATER => {
            if r8 == 255 {
                return ace_const(false);
            }
            Ace { expr: format!("(alphaCompare > {iref}u)"), constant: -1 }
        }
        GX_ALWAYS => ace_const(true),
        _ => panic!("invalid alpha comp {comp}"),
    }
}

fn vtx_attr(c: &ShaderConfig, attr: u32) -> String {
    let t = c.attrs[attr as usize].attr_type as u32;
    if t == GX_NONE {
        if attr == GX_VA_PNMTXIDX {
            return "imm.current_pnmtx".into();
        }
        if attr == GX_VA_NRM {
            return "vec3f(1.0, 0.0, 0.0)".into();
        }
        if attr == GX_VA_CLR0 || attr == GX_VA_CLR1 {
            return "vec4f(0.0, 0.0, 0.0, 0.0)".into();
        }
        if (GX_VA_TEX0..=GX_VA_TEX7).contains(&attr) {
            return "vec2f(0.0, 0.0)".into();
        }
        panic!("unmapped vtx attr {attr}");
    }
    match attr {
        GX_VA_POS => "in_pos".into(),
        GX_VA_NRM => "in_nrm".into(),
        GX_VA_CLR0 | GX_VA_CLR1 => format!("in_clr{}", attr - GX_VA_CLR0),
        a if (GX_VA_TEX0..=GX_VA_TEX7).contains(&a) => format!("in_tex{}_uv", a - GX_VA_TEX0),
        GX_VA_PNMTXIDX => "in_pnmtxidx".into(),
        a if (GX_VA_TEX0MTXIDX..=GX_VA_TEX7MTXIDX).contains(&a) => format!("in_texmtxidx{}", a - GX_VA_TEX0MTXIDX),
        _ => panic!("unhandled vtx attr {attr}"),
    }
}

fn fetch_fixed16_attr(func: &str, m: &AttrConfig, buf: &str, offs: &str, le: bool) -> String {
    let comp = |o: u32| format!("{func}_1(&{buf}, {offs} + {o}u, {}u, {le})", m.frac);
    match m.cnt {
        2 => format!("vec2f({}, {})", comp(0), comp(2)),
        3 => format!("vec3f({}, {}, {})", comp(0), comp(2), comp(4)),
        n => format!("{func}_{n}(&{buf}, {offs}, {}u, {le})", m.frac),
    }
}

fn fetch_attr(m: &AttrConfig, buf: &str, offs: &str, le: bool) -> String {
    match m.comp_type as u32 {
        GX_U8 => format!("fetch_u8_{}(&{buf}, {offs}, {}, {le})", m.cnt, m.frac),
        GX_S8 => format!("fetch_s8_{}(&{buf}, {offs}, {}, {le})", m.cnt, m.frac),
        GX_U16 => fetch_fixed16_attr("fetch_u16", m, buf, offs, le),
        GX_S16 => fetch_fixed16_attr("fetch_s16", m, buf, offs, le),
        GX_F32 => format!("fetch_f32_{}(&{buf}, {offs}, {le})", m.cnt),
        GX_RGBA8 => format!("unpack4x8unorm(load_u32_raw(&{buf}, {offs}))"),
        t => panic!("fetch_attr: Unimplemented {t}"),
    }
}

fn fetch_color_attr(m: &AttrConfig, buf: &str, offs: &str, le: bool) -> String {
    let f = match m.comp_type as u32 {
        GX_RGB565 => "fetch_rgb565",
        GX_RGB8 => "fetch_rgb8",
        GX_RGBX8 => "fetch_rgbx8",
        GX_RGBA4 => "fetch_rgba4",
        GX_RGBA6 => "fetch_rgba6",
        GX_RGBA8 => "fetch_rgba8",
        t => panic!("fetch_color_attr: Unimplemented {t}"),
    };
    format!("{f}(&{buf}, {offs}, {le})")
}

fn imm_array_start(attr: u32) -> String {
    let idx = attr - GX_VA_POS;
    format!("imm.array_start{}.{}", idx / 4, ['x', 'y', 'z', 'w'][(idx % 4) as usize])
}

fn attr_address(m: &AttrConfig, attr: u32, vidx: &str, stride: u32, dl_extra: u32, within: u32) -> (String, &'static str, bool) {
    let dl_off = m.offset as u32 + dl_extra;
    match m.attr_type as u32 {
        GX_INDEX8 => (
            format!(
                "{} + raw_fetch_u8_1(&vbuf, imm.vtx_start + {vidx} * {stride}u + {dl_off}u) * {}u + {within}u",
                imm_array_start(attr),
                m.stride
            ),
            "abuf",
            m.le,
        ),
        GX_INDEX16 => (
            format!(
                "{} + raw_fetch_u16_1(&vbuf, imm.vtx_start + {vidx} * {stride}u + {dl_off}u, false) * {}u + {within}u",
                imm_array_start(attr),
                m.stride
            ),
            "abuf",
            m.le,
        ),
        _ => (format!("imm.vtx_start + {vidx} * {stride}u + {}u", dl_off + within), "vbuf", false),
    }
}

fn attr_load(c: &ShaderConfig, attr: u32, vidx: &str) -> String {
    let m = c.attrs[attr as usize];
    if m.attr_type as u32 == GX_NONE {
        return vtx_attr(c, attr);
    }
    let (offs, buf, le) = attr_address(&m, attr, vidx, c.vtx_stride as u32, 0, 0);
    match attr {
        GX_VA_PNMTXIDX => {
            if c.pn_palette {
                format!("raw_fetch_u8_1(&{buf}, {offs})")
            } else {
                format!("(raw_fetch_u8_1(&{buf}, {offs}) / 3u)")
            }
        }
        a if (GX_VA_TEX0MTXIDX..=GX_VA_TEX7MTXIDX).contains(&a) => format!("raw_fetch_u8_1(&{buf}, {offs})"),
        GX_VA_POS => {
            let p = fetch_attr(&m, buf, &offs, le);
            if m.cnt == 2 {
                format!("vec3f({p}, 0.0)")
            } else {
                p
            }
        }
        GX_VA_NRM => {
            if m.cnt > 3 {
                let mut n = m;
                n.cnt = 3;
                fetch_attr(&n, buf, &offs, le)
            } else {
                fetch_attr(&m, buf, &offs, le)
            }
        }
        GX_VA_CLR0 | GX_VA_CLR1 => fetch_color_attr(&m, buf, &offs, le),
        a if (GX_VA_TEX0..=GX_VA_TEX7).contains(&a) => {
            let t = fetch_attr(&m, buf, &offs, le);
            if m.cnt == 1 {
                format!("vec2f({t}, 0.0)")
            } else {
                t
            }
        }
        _ => panic!("attr_load: Unimplemented {attr}"),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Nbt {
    N,
    B,
    T,
}

fn attr_load_nbt_slice(c: &ShaderConfig, slice: Nbt, vidx: &str) -> String {
    let m = c.attrs[GX_VA_NRM as usize];
    if m.attr_type as u32 == GX_NONE || m.cnt != 9 {
        if slice == Nbt::N {
            return attr_load(c, GX_VA_NRM, vidx);
        }
        return "vec3f(0.0)".into();
    }
    let si = slice as u32;
    let cs = comp_type_size(GX_VA_NRM, m.comp_type as u32) as u32;
    let mut dl_extra = 0;
    if m.nbt3 {
        if m.attr_type as u32 == GX_INDEX8 {
            dl_extra = si;
        } else if m.attr_type as u32 == GX_INDEX16 {
            dl_extra = si * 2;
        }
    }
    let within = si * 3 * cs;
    let (offs, buf, le) = attr_address(&m, GX_VA_NRM, vidx, c.vtx_stride as u32, dl_extra, within);
    let mut sm = m;
    sm.cnt = 3;
    fetch_attr(&sm, buf, &offs, le)
}

fn nbt_slice_local(s: Nbt) -> &'static str {
    if s == Nbt::B {
        "in_binrm"
    } else {
        "in_tangent"
    }
}

fn is_emboss(t: u32) -> bool {
    (GX_TG_BUMP0..=GX_TG_BUMP7).contains(&t)
}

fn lighting_func(c: &ShaderConfig, cc: &ColorChannelConfig, i: u32, alpha: bool) -> String {
    let swizzle = if alpha { ".a" } else { "" };
    let out_var = format!("out.cc{i}");
    let pos_var = "mv_pos";
    let amb_src = if cc.amb_src == GX_SRC_VTX {
        vtx_attr(c, GX_VA_CLR0 + i)
    } else if cc.amb_src == GX_SRC_REG {
        format!("ubuf.cc{i}{}_amb", if alpha { "a" } else { "" })
    } else {
        String::new()
    };
    let mat_src = if cc.mat_src == GX_SRC_VTX {
        vtx_attr(c, GX_VA_CLR0 + i)
    } else if cc.mat_src == GX_SRC_REG {
        format!("ubuf.cc{i}{}_mat", if alpha { "a" } else { "" })
    } else {
        String::new()
    };
    if !cc.lighting_enabled {
        return format!("\n    {out_var}{swizzle} = {mat_src}{swizzle};");
    }
    let diff_fn = cc.diff_fn;
    let attn = if cc.attn_fn == GX_AF_NONE {
        "attn = 1.0;".to_string()
    } else if cc.attn_fn == GX_AF_SPOT {
        "\n          var cosine = max(0.0, dot(ldir, light.dir));\n          var cos_attn = dot(light.cos_att, vec3f(1.0, cosine, cosine * cosine));\n          var dist_attn = dot(light.dist_att, vec3f(1.0, dist, dist2));\n          attn = max(0.0, cos_attn / dist_attn);".to_string()
    } else if cc.attn_fn == GX_AF_SPEC {
        let normal = "mv_nrm";
        let dist_attn = if diff_fn != GX_DF_NONE {
            "max(0.0, dot(normalize(light.dist_att), vec3f(1.0, attn, attn * attn)));"
        } else {
            "max(0.0, dot(light.dist_att, vec3f(1.0, attn, attn * attn)));"
        };
        format!(
            "\n          attn = select(0.0, max(0.0, dot({normal}, light.dir)), dot({normal}, ldir) >= 0.0);\n          var cos_attn = dot(light.cos_att, vec3f(1.0, attn, attn * attn));\n          var dist_attn = {dist_attn};\n          attn = max(0.0, cos_attn / dist_attn);"
        )
    } else {
        String::new()
    };
    let diff = match diff_fn {
        GX_DF_NONE => "1.0",
        GX_DF_SIGN => "dot(ldir, mv_nrm)",
        GX_DF_CLAMP => "max(0.0, dot(ldir, mv_nrm))",
        _ => "",
    };
    let a = if alpha { "a" } else { "" };
    let n = MAX_LIGHTS;
    format!(
        "\n    {{\n      var lighting = {amb_src};\n      for (var i = 0u; i < {n}u; i++) {{\n          if ((ubuf.lightState{i}{a} & (1u << i)) == 0u) {{ continue; }}\n          var light = ubuf.lights[i];\n          var ldir = light.pos - {pos_var};\n          var dist2 = dot(ldir, ldir);\n          var dist = sqrt(dist2);\n          ldir = ldir / dist;\n          var attn: f32;{attn}\n          var diff = {diff};\n          lighting = lighting + (attn * diff * light.color);\n      }}\n      {out_var}{swizzle} = ({mat_src} * clamp(lighting, vec4f(0.0), vec4f(1.0))){swizzle};\n    }}"
    )
}

/// Aurora `build_shader_source(config, normalAttachment)`; `normal_attachment` is `u32::MAX`
/// when the render target has no normal attachment.
pub fn build_shader_source(c: &ShaderConfig, normal_attachment: u32) -> String {
    let info = build_shader_info(c);
    let mut uniform_pre = String::new();
    let mut uni = String::new();
    let mut tex_bindings = String::new();
    let mut vtx_out = String::new();
    let mut vtx_in = String::new();
    let mut xfr_pre = String::new();
    let mut xfr = String::new();
    let mut out_idx = 0usize;

    let mut vidx_attr = "vidx";
    if c.line_mode != 0 {
        vtx_in += ",\n    @builtin(instance_index) iidx: u32";
        uni += "\n    line_width: f32,\n    line_aspect_y: f32,\n    line_tex_offset: f32,\n    line_texcoord_mask: u32,";
        if c.line_mode == 3 {
            write!(
                xfr_pre,
                "\n    let in_vidx = iidx;\n    let in_pos = {};\n    let in_pnmtxidx = {};\n    let mv_pos = vec4f(in_pos, 1.0) * ubuf.postex_mtx[in_pnmtxidx];",
                attr_load(c, GX_VA_POS, "in_vidx"),
                attr_load(c, GX_VA_PNMTXIDX, "in_vidx")
            )
            .unwrap();
        } else {
            write!(
                xfr_pre,
                "\n    let use_b = vidx >= 2u;\n    let vidx_a = iidx * {}u;\n    let vidx_b = vidx_a + 1u;\n    let in_vidx = select(vidx_a, vidx_b, use_b);\n    let pos_a = {};\n    let pos_b = {};\n    let in_pos = select(pos_a, pos_b, use_b);\n    let pnmtxidx_a = {};\n    let pnmtxidx_b = {};\n    let in_pnmtxidx = select(pnmtxidx_a, pnmtxidx_b, use_b);\n    let mv_pos_a = vec4f(pos_a, 1.0) * ubuf.postex_mtx[pnmtxidx_a];\n    let mv_pos_b = vec4f(pos_b, 1.0) * ubuf.postex_mtx[pnmtxidx_b];\n    let mv_pos = select(mv_pos_a, mv_pos_b, use_b);",
                if c.line_mode == 1 { 2 } else { 1 },
                attr_load(c, GX_VA_POS, "vidx_a"),
                attr_load(c, GX_VA_POS, "vidx_b"),
                attr_load(c, GX_VA_PNMTXIDX, "vidx_a"),
                attr_load(c, GX_VA_PNMTXIDX, "vidx_b")
            )
            .unwrap();
        }
        vidx_attr = "in_vidx";
    } else if c.attrs[GX_VA_PNMTXIDX as usize].attr_type as u32 == GX_NONE {
        xfr_pre += "\n    let in_pnmtxidx = imm.current_pnmtx;";
    }

    for attr in GX_VA_PNMTXIDX..=GX_VA_TEX7 {
        if c.attrs[attr as usize].attr_type as u32 == GX_NONE {
            continue;
        }
        if (attr != GX_VA_PNMTXIDX && attr != GX_VA_POS) || c.line_mode == 0 {
            write!(xfr_pre, "\n    let {} = {};", vtx_attr(c, attr), attr_load(c, attr, vidx_attr)).unwrap();
        }
    }
    let mut needs_binrm = false;
    let mut needs_tangent = false;
    for i in 0..MAX_TEX_COORD {
        if !bit(info.sampled_tex_coords, i as u32) {
            continue;
        }
        let emboss = is_emboss(c.tcgs[i].ty);
        needs_binrm = needs_binrm || c.tcgs[i].src == GX_TG_BINRM || emboss;
        needs_tangent = needs_tangent || c.tcgs[i].src == GX_TG_TANGENT || emboss;
    }
    if needs_binrm {
        write!(xfr_pre, "\n    let {} = {};", nbt_slice_local(Nbt::B), attr_load_nbt_slice(c, Nbt::B, vidx_attr)).unwrap();
    }
    if needs_tangent {
        write!(xfr_pre, "\n    let {} = {};", nbt_slice_local(Nbt::T), attr_load_nbt_slice(c, Nbt::T, vidx_attr)).unwrap();
    }

    if c.line_mode == 0 && c.pn_palette {
        let row = |w: u32| {
            format!(
                "bitcast<vec4f>(vec4u(abuf[pal_b + {}u], abuf[pal_b + {}u], abuf[pal_b + {}u], abuf[pal_b + {}u]))",
                w,
                w + 1,
                w + 2,
                w + 3
            )
        };
        xfr_pre += "\n    let pal_b = imm._pad + in_pnmtxidx * 24u;";
        write!(xfr_pre, "\n    let pal_pos = mat3x4f({}, {}, {});", row(0), row(4), row(8)).unwrap();
        write!(xfr_pre, "\n    let pal_nrm = mat3x4f({}, {}, {});", row(12), row(16), row(20)).unwrap();
        write!(
            xfr_pre,
            "\n    let mv_pos = vec4f({}, 1.0) * pal_pos;\n    out.pos = vec4f(mv_pos, 1.0) * ubuf.proj;",
            vtx_attr(c, GX_VA_POS)
        )
        .unwrap();
    } else if c.line_mode == 0 {
        write!(
            xfr_pre,
            "\n    let mv_pos = vec4f({}, 1.0) * ubuf.postex_mtx[in_pnmtxidx];\n    out.pos = vec4f(mv_pos, 1.0) * ubuf.proj;",
            vtx_attr(c, GX_VA_POS)
        )
        .unwrap();
    } else if c.line_mode == 3 {
        xfr_pre += "\n    let clip = vec4f(mv_pos, 1.0) * ubuf.proj;\n    let viewport_scale = ubuf.render_viewport_size / max(ubuf.logical_viewport_size, vec2f(1.0));\n    let point_size = ubuf.line_width * min(viewport_scale.x, viewport_scale.y);\n    let x_sign = select(-1.0, 1.0, (vidx & 1u) != 0u);\n    let y_sign = select(-1.0, 1.0, vidx >= 2u);\n    let offset_px = vec2f(x_sign, y_sign) * (point_size / 2.0);\n    let offset_ndc = (offset_px * 2.0) / ubuf.render_viewport_size;\n    out.pos = vec4f(clip.xy + offset_ndc * clip.w, clip.zw);";
    } else {
        xfr_pre += "\n    let clip_a = vec4f(mv_pos_a, 1.0) * ubuf.proj;\n    let clip_b = vec4f(mv_pos_b, 1.0) * ubuf.proj;\n    let ndc_a = clip_a.xy / clip_a.w;\n    let ndc_b = clip_b.xy / clip_b.w;\n    let viewport_scale = ubuf.render_viewport_size / max(ubuf.logical_viewport_size, vec2f(1.0));\n    let delta_px = (ndc_b - ndc_a) / 2.0 * ubuf.render_viewport_size;\n    let dir_px = select(vec2f(1.0, 0.0), normalize(delta_px), dot(delta_px, delta_px) > 1e-10);\n    let perp_px = vec2f(-dir_px.y, dir_px.x);\n    let line_width = ubuf.line_width * min(viewport_scale.x, viewport_scale.y);\n    let offset_px = perp_px * (line_width / 2.0) * select(-1.0, 1.0, (vidx & 1u) != 0u);\n    let offset_ndc = (offset_px * 2.0) / ubuf.render_viewport_size;\n    let clip_base = select(clip_a, clip_b, use_b);\n    out.pos = vec4f(clip_base.xy + offset_ndc * clip_base.w, clip_base.zw);";
    }
    if c.pn_palette && c.line_mode == 0 {
        write!(
            xfr_pre,
            "\n    let nrm_tmp = vec4f({}, 0.0) * pal_nrm;\n    let mv_nrm = select(nrm_tmp, normalize(nrm_tmp), dot(nrm_tmp, nrm_tmp) > 1e-10);",
            vtx_attr(c, GX_VA_NRM)
        )
        .unwrap();
    } else {
        write!(
            xfr_pre,
            "\n    let nrm_tmp = vec4f({}, 0.0) * ubuf.nrm_mtx[in_pnmtxidx];\n    let mv_nrm = select(nrm_tmp, normalize(nrm_tmp), dot(nrm_tmp, nrm_tmp) > 1e-10);",
            vtx_attr(c, GX_VA_NRM)
        )
        .unwrap();
    }
    let use_normal_target = normal_attachment != u32::MAX && c.attrs[GX_VA_NRM as usize].attr_type as u32 != GX_NONE;
    if use_normal_target {
        write!(vtx_out, "\n    @location({out_idx}) mv_nrm: vec3f,").unwrap();
        out_idx += 1;
        xfr_pre += "\n    out.mv_nrm = mv_nrm;";
    }

    uni += "\n    proj: mat4x4f,";
    write!(uni, "\n    postex_mtx: array<mat3x4f, {}>,", MAX_PN_MTX + MAX_TEX_MTX).unwrap();
    write!(uni, "\n    nrm_mtx: array<mat3x4f, {}>,", MAX_PN_MTX).unwrap();
    let mut frag_pre = String::new();
    let mut frag = String::new();

    const REG_NAME: [&str; 4] = ["prev", "tevreg0", "tevreg1", "tevreg2"];
    let mut cn = [false; 4];
    let mut an = [false; 4];
    for idx in 0..c.tev_stage_count as usize {
        let s = &c.tev_stages[idx];
        {
            let carg = |a: u32| {
                let v = color_arg_reg(a, idx, c, s);
                if color_arg_normalized(a, &cn, &an) {
                    format!("vec3f({v})")
                } else {
                    format!("tev_overflow_vec3f({v})")
                }
            };
            let out = REG_NAME[s.color_op.out_reg as usize];
            let e = tev_op(
                s.color_op.op,
                tev_bias(s.color_op.bias),
                tev_scale(s.color_op.scale),
                &carg(s.color_pass.a),
                &carg(s.color_pass.b),
                &carg(s.color_pass.c),
                &color_arg_reg(s.color_pass.d, idx, c, s),
                "vec3(0)",
            );
            let op = if s.color_op.clamp {
                format!("clamp({e}, vec3f(0.0), vec3f(1.0))")
            } else {
                format!("clamp({e}, vec3f(-4.0), vec3f(4.0))")
            };
            write!(frag, "\n    // TEV stage {idx}\n    {out} = vec4f({op}, {out}.a);").unwrap();
            cn[s.color_op.out_reg as usize] = s.color_op.clamp;
        }
        {
            let aarg = |a: u32| {
                let v = alpha_arg_reg(a, idx, c, s);
                if alpha_arg_normalized(a, &an) {
                    v
                } else {
                    format!("tev_overflow_f32({v})")
                }
            };
            let out = REG_NAME[s.alpha_op.out_reg as usize];
            let e = tev_op(
                s.alpha_op.op,
                tev_bias(s.alpha_op.bias),
                tev_scale(s.alpha_op.scale),
                &aarg(s.alpha_pass.a),
                &aarg(s.alpha_pass.b),
                &aarg(s.alpha_pass.c),
                &alpha_arg_reg(s.alpha_pass.d, idx, c, s),
                "0.0",
            );
            let op = if s.alpha_op.clamp { format!("clamp({e}, 0.0, 1.0)") } else { format!("clamp({e}, -4.0, 4.0)") };
            write!(frag, "\n    {out}.a = {op};").unwrap();
            an[s.alpha_op.out_reg as usize] = s.alpha_op.clamp;
        }
    }
    let last = &c.tev_stages[c.tev_stage_count as usize - 1];
    let prev_cn = cn[last.color_op.out_reg as usize];
    let prev_an = an[last.alpha_op.out_reg as usize];
    if last.color_op.out_reg != 0 {
        write!(frag, "\n    prev = vec4f({}.rgb, prev.a);", REG_NAME[last.color_op.out_reg as usize]).unwrap();
    }
    if last.alpha_op.out_reg != 0 {
        write!(frag, "\n    prev.a = {}.a;", REG_NAME[last.alpha_op.out_reg as usize]).unwrap();
    }
    if bit(info.loads_tev_reg, 0) {
        uni += "\n    tevprev: vec4f,";
        frag_pre += "\n    var prev = ubuf.tevprev;";
    } else {
        frag_pre += "\n    var prev: vec4f;";
    }
    for i in 1..MAX_TEV_REGS as u32 {
        if bit(info.loads_tev_reg, i) {
            write!(uni, "\n    tevreg{}: vec4f,", i - 1).unwrap();
            write!(frag_pre, "\n    var tevreg{0} = ubuf.tevreg{0};", i - 1).unwrap();
        } else if bit(info.writes_tev_reg, i) {
            write!(frag_pre, "\n    var tevreg{}: vec4f;", i - 1).unwrap();
        }
    }
    if info.lighting_enabled {
        write!(
            uni,
            "\n    lights: array<Light, {}>,\n    lightState0: u32,\n    lightState1: u32,\n    lightState0a: u32,\n    lightState1a: u32,",
            MAX_LIGHTS
        )
        .unwrap();
        uniform_pre += "\nstruct Light {\n    pos: vec3f,\n    dir: vec3f,\n    color: vec4f,\n    cos_att: vec3f,\n    dist_att: vec3f,\n};";
    }
    for i in 0..2u32 {
        if !bit(info.sampled_color_channels, i) {
            continue;
        }
        let cc = c.color_channels[i as usize];
        let cca = c.color_channels[(i + GX_ALPHA0) as usize];
        if cc.lighting_enabled && cc.amb_src == GX_SRC_REG {
            write!(uni, "\n    cc{i}_amb: vec4f,").unwrap();
        }
        if cc.mat_src == GX_SRC_REG {
            write!(uni, "\n    cc{i}_mat: vec4f,").unwrap();
        }
        if cca.lighting_enabled && cca.amb_src == GX_SRC_REG {
            write!(uni, "\n    cc{i}a_amb: vec4f,").unwrap();
        }
        if cca.mat_src == GX_SRC_REG {
            write!(uni, "\n    cc{i}a_mat: vec4f,").unwrap();
        }
        write!(vtx_out, "\n    @location({out_idx}) cc{i}: vec4f,").unwrap();
        out_idx += 1;
        xfr += &lighting_func(c, &cc, i, false);
        xfr += &lighting_func(c, &cca, i, true);
        write!(frag_pre, "\n    var rast{i} = in.cc{i};").unwrap();
    }
    for i in 0..MAX_KCOLORS as u32 {
        if bit(info.sampled_kcolors, i) {
            write!(uni, "\n    kcolor{i}: vec4f,").unwrap();
        }
    }
    for i in 0..MAX_TEX_COORD {
        if !bit(info.sampled_tex_coords, i as u32) {
            continue;
        }
        let t = c.tcgs[i];
        if t.ty == GX_TG_MTX3x4 {
            write!(vtx_out, "\n    @location({out_idx}) tex{i}_uvw: vec3f,").unwrap();
        } else {
            write!(vtx_out, "\n    @location({out_idx}) tex{i}_uv: vec2f,").unwrap();
        }
        out_idx += 1;
        if is_emboss(t.ty) {
            let light = t.ty - GX_TG_BUMP0;
            let nrm = if c.pn_palette && c.line_mode == 0 { "pal_nrm" } else { "ubuf.nrm_mtx[in_pnmtxidx]" };
            write!(
                xfr,
                "\n    let bump_ldir{i} = normalize(ubuf.lights[{light}].pos - mv_pos);\n    let bump_tan{i} = vec4f(in_tangent, 0.0) * {nrm};\n    let bump_bin{i} = vec4f(in_binrm, 0.0) * {nrm};\n    out.tex{i}_uv = tc{}_proj.xy + vec2f(dot(bump_ldir{i}, bump_tan{i}), dot(bump_ldir{i}, bump_bin{i}));",
                t.emboss_src
            )
            .unwrap();
            write!(frag_pre, "\n    var tex{i}_uv = in.tex{i}_uv.xy;").unwrap();
            continue;
        }
        if (GX_TG_TEX0..=GX_TG_TEX7).contains(&t.src) {
            write!(xfr, "\n    var tc{i} = vec4f({}, 1.0, 1.0);", vtx_attr(c, GX_VA_TEX0 + (t.src - GX_TG_TEX0))).unwrap();
        } else if t.src == GX_TG_POS {
            write!(xfr, "\n    var tc{i} = vec4f({}, 1.0);", vtx_attr(c, GX_VA_POS)).unwrap();
        } else if t.src == GX_TG_NRM {
            write!(xfr, "\n    var tc{i} = vec4f({}, 1.0);", vtx_attr(c, GX_VA_NRM)).unwrap();
        } else if t.src == GX_TG_COLOR0 {
            write!(xfr, "\n    var tc{i} = {};", vtx_attr(c, GX_VA_CLR0)).unwrap();
        } else if t.src == GX_TG_COLOR1 {
            write!(xfr, "\n    var tc{i} = {};", vtx_attr(c, GX_VA_CLR1)).unwrap();
        } else if t.src == GX_TG_BINRM {
            write!(xfr, "\n    var tc{i} = vec4f({}, 1.0);", nbt_slice_local(Nbt::B)).unwrap();
        } else if t.src == GX_TG_TANGENT {
            write!(xfr, "\n    var tc{i} = vec4f({}, 1.0);", nbt_slice_local(Nbt::T)).unwrap();
        } else {
            write!(xfr, "\n    var tc{i} = vec4f(0.0, 0.0, 1.0, 1.0);").unwrap();
        }
        if t.ty == GX_TG_MTX2x4 || t.ty == GX_TG_MTX3x4 {
            if bit(info.index_attr, GX_VA_TEX0MTXIDX + i as u32) {
                write!(xfr, "\n    var tc{i}_tmp = tc{i} * ubuf.postex_mtx[in_texmtxidx{i} / 3u];").unwrap();
            } else if t.mtx == GX_IDENTITY {
                write!(xfr, "\n    var tc{i}_tmp = tc{i}.xyz;").unwrap();
            } else {
                write!(xfr, "\n    var tc{i}_tmp = tc{i} * ubuf.postex_mtx[{}];", t.mtx / 3).unwrap();
            }
            if t.ty == GX_TG_MTX2x4 {
                write!(xfr, "\n    tc{i}_tmp.z = 1.0f;").unwrap();
            }
        } else if t.ty == GX_TG_SRTG {
            write!(xfr, "\n    var tc{i}_tmp = vec3f(tc{i}.xy, 1.0f);").unwrap();
        }
        if t.normalize {
            write!(xfr, "\n    tc{i}_tmp = normalize(tc{i}_tmp);").unwrap();
        }
        if t.post_mtx == GX_PTIDENTITY {
            write!(xfr, "\n    var tc{i}_proj = tc{i}_tmp;").unwrap();
        } else {
            write!(xfr, "\n    var tc{i}_proj = vec4f(tc{i}_tmp.xyz, 1.0) * ubuf.postmtx[{}];", (t.post_mtx - GX_PTTEXMTX0) / 3)
                .unwrap();
        }
        if c.line_mode == 3 {
            write!(
                xfr,
                "\n    if ((ubuf.line_texcoord_mask & (1u << {i})) != 0u) {{\n        if ((vidx & 1u) != 0u) {{ tc{i}_proj.x += ubuf.line_tex_offset; }}\n        if (vidx >= 2u) {{ tc{i}_proj.y += ubuf.line_tex_offset; }}\n    }}"
            )
            .unwrap();
        } else if c.line_mode != 0 {
            write!(
                xfr,
                "\n    if ((ubuf.line_texcoord_mask & (1u << {i})) != 0u && (vidx & 1u) != 0u) {{\n        tc{i}_proj.y += ubuf.line_tex_offset;\n    }}"
            )
            .unwrap();
        }
        if t.ty == GX_TG_MTX3x4 {
            write!(xfr, "\n    out.tex{i}_uvw = tc{i}_proj.xyz;").unwrap();
            write!(frag_pre, "\n    var tex{i}_uv = in.tex{i}_uvw.xy / in.tex{i}_uvw.z;").unwrap();
        } else {
            write!(xfr, "\n    out.tex{i}_uv = tc{i}_proj.xy;").unwrap();
            write!(frag_pre, "\n    var tex{i}_uv = in.tex{i}_uv.xy;").unwrap();
        }
    }
    let ind_scale = |s: u32| -> &'static str {
        match s {
            GX_ITS_1 => "1.0",
            GX_ITS_2 => "(1.0 / 2.0)",
            GX_ITS_4 => "(1.0 / 4.0)",
            GX_ITS_8 => "(1.0 / 8.0)",
            GX_ITS_16 => "(1.0 / 16.0)",
            GX_ITS_32 => "(1.0 / 32.0)",
            GX_ITS_64 => "(1.0 / 64.0)",
            GX_ITS_128 => "(1.0 / 128.0)",
            GX_ITS_256 => "(1.0 / 256.0)",
            _ => panic!("unhandled indirect scale {s}"),
        }
    };
    for i in 0..MAX_IND_STAGES {
        if !bit(info.used_ind_stages, i as u32) {
            continue;
        }
        let is = c.ind_stages[i];
        let tc = is.tex_coord_id;
        let tm = is.tex_map_id;
        let scale = format!(
            "tex{tc}_uv * ubuf.texcoord_scale[{tc}].xy * vec2f({}, {}) / ubuf.tex{tm}_size_bias.xy",
            ind_scale(is.scale_s),
            ind_scale(is.scale_t)
        );
        write!(
            frag_pre,
            "\n    // Indirect stage {i}\n    var t_IndTexCoord{i} = 255.0 * textureSampleBias(tex{tm}, tex{tm}_samp, {scale}, ubuf.tex{tm}_size_bias.z).abg;"
        )
        .unwrap();
    }
    if info.used_ind_stages != 0 {
        frag_pre += "\n    var t_TexCoord = vec2f(0.0);";
    }
    for i in 0..c.tev_stage_count as usize {
        let s = &c.tev_stages[i];
        let needs_ind_coord = s.ind_tex_mtx_id != GX_ITM_OFF;
        let has_ind_stage = s.ind_tex_stage < c.num_ind_stages;
        let needs_tev_tc =
            needs_ind_coord || s.ind_tex_wrap_s != GX_ITW_OFF || s.ind_tex_wrap_t != GX_ITW_OFF || s.ind_tex_add_prev;
        let needs_sample = uses_texture_sample(s);
        if !needs_tev_tc && !needs_sample {
            continue;
        }
        let has_base_tc = s.tex_coord_id != GX_TEXCOORD_NULL;
        let has_base_tex = s.tex_map_id != GX_TEXMAP_NULL;
        let has_base = has_base_tc && has_base_tex;
        let mut uv_in = String::new();
        if needs_tev_tc {
            write!(frag_pre, "\n    // TEV stage {i} indirect").unwrap();
            let mut offset_texel = String::new();
            if needs_ind_coord && has_ind_stage {
                let shift = match s.ind_tex_format {
                    GX_ITF_8 => "",
                    GX_ITF_5 => " / 8.0",
                    GX_ITF_4 => " / 16.0",
                    GX_ITF_3 => " / 32.0",
                    f => panic!("unhandled indirect format {f}"),
                };
                if shift.is_empty() {
                    write!(frag_pre, "\n    var ind{i}_coord = t_IndTexCoord{};", s.ind_tex_stage).unwrap();
                } else {
                    write!(frag_pre, "\n    var ind{i}_coord = floor(t_IndTexCoord{}{shift});", s.ind_tex_stage).unwrap();
                }
                if s.ind_tex_bias_sel != GX_ITB_NONE {
                    let bias = if s.ind_tex_format == GX_ITF_8 { "-128.0" } else { "1.0" };
                    let b = s.ind_tex_bias_sel;
                    let bs = if matches!(b, GX_ITB_S | GX_ITB_ST | GX_ITB_SU | GX_ITB_STU) { "1.0" } else { "0.0" };
                    let bt = if matches!(b, GX_ITB_T | GX_ITB_ST | GX_ITB_TU | GX_ITB_STU) { "1.0" } else { "0.0" };
                    let bu = if matches!(b, GX_ITB_U | GX_ITB_SU | GX_ITB_TU | GX_ITB_STU) { "1.0" } else { "0.0" };
                    write!(frag_pre, "\n    ind{i}_coord = ind{i}_coord + vec3f({bs}, {bt}, {bu}) * {bias};").unwrap();
                }
                let m = s.ind_tex_mtx_id;
                if (GX_ITM_0..=GX_ITM_2).contains(&m) {
                    let mi = m - GX_ITM_0;
                    write!(frag_pre, "\n    let ind{i}_c0 = ubuf.ind_mtx[{mi}][0];\n    let ind{i}_c1 = ubuf.ind_mtx[{mi}][1];").unwrap();
                    offset_texel = format!(
                        "vec2f(dot(vec3f(ind{i}_c0.xz, ind{i}_c1.x), ind{i}_coord), dot(vec3f(ind{i}_c0.yw, ind{i}_c1.y), ind{i}_coord)) * ind{i}_c1.z"
                    );
                } else if (GX_ITM_S0..=GX_ITM_S2).contains(&m) && has_base {
                    let mi = m - GX_ITM_S0;
                    let rt = s.tex_coord_id;
                    offset_texel = format!(
                        "tex{rt}_uv * ubuf.texcoord_scale[{rt}].xy * ind{i}_coord.x * ubuf.ind_mtx[{mi}][1][2] / 256.0"
                    );
                } else if (GX_ITM_T0..=GX_ITM_T2).contains(&m) && has_base {
                    let mi = m - GX_ITM_T0;
                    let rt = s.tex_coord_id;
                    offset_texel = format!(
                        "tex{rt}_uv * ubuf.texcoord_scale[{rt}].xy * ind{i}_coord.y * ubuf.ind_mtx[{mi}][1][2] / 256.0"
                    );
                }
            }
            let simple = s.ind_tex_mtx_id == GX_ITM_OFF && !s.ind_tex_add_prev;
            let wrap_comp = |w: u32, coord: String| -> String {
                match w {
                    GX_ITW_OFF => coord,
                    GX_ITW_256 => format!("({coord} % 256.0)"),
                    GX_ITW_128 => format!("({coord} % 128.0)"),
                    GX_ITW_64 => format!("({coord} % 64.0)"),
                    GX_ITW_32 => format!("({coord} % 32.0)"),
                    GX_ITW_16 => format!("({coord} % 16.0)"),
                    GX_ITW_0 => "0.0".into(),
                    _ => panic!("unhandled indirect wrap {w}"),
                }
            };
            let mut base_expr = String::new();
            if has_base {
                let tc = s.tex_coord_id;
                if simple {
                    base_expr = format!("tex{tc}_uv");
                } else {
                    write!(frag_pre, "\n    var ind{i}_texel = tex{tc}_uv * ubuf.texcoord_scale[{tc}].xy;").unwrap();
                    base_expr = format!("ind{i}_texel");
                }
            }
            let mut wrapped = base_expr.clone();
            if !base_expr.is_empty() && (s.ind_tex_wrap_s != GX_ITW_OFF || s.ind_tex_wrap_t != GX_ITW_OFF) {
                wrapped = format!(
                    "vec2f({}, {})",
                    wrap_comp(s.ind_tex_wrap_s, format!("{base_expr}.x")),
                    wrap_comp(s.ind_tex_wrap_t, format!("{base_expr}.y"))
                );
            }
            let final_coord = if !wrapped.is_empty() && !offset_texel.is_empty() {
                format!("{wrapped} + ({offset_texel})")
            } else if !wrapped.is_empty() {
                wrapped
            } else {
                offset_texel
            };
            if info.used_ind_stages != 0 && !final_coord.is_empty() {
                if s.ind_tex_add_prev {
                    write!(frag_pre, "\n    t_TexCoord += {final_coord};").unwrap();
                } else {
                    write!(frag_pre, "\n    t_TexCoord = {final_coord};").unwrap();
                }
                if needs_sample && has_base_tex {
                    let tm = s.tex_map_id;
                    if simple {
                        write!(frag_pre, "\n    var ind{i}_uv = t_TexCoord;").unwrap();
                    } else {
                        write!(frag_pre, "\n    var ind{i}_uv = t_TexCoord / ubuf.tex{tm}_size_bias.xy;").unwrap();
                    }
                    uv_in = format!("ind{i}_uv");
                }
            }
        }
        if !needs_sample {
            continue;
        }
        assert!(s.tex_map_id != GX_TEXMAP_NULL, "unmapped texture for stage {i}");
        assert!(s.tex_coord_id != GX_TEXCOORD_NULL, "unmapped texcoord for stage {i}");
        if uv_in.is_empty() {
            uv_in = format!("tex{}_uv", s.tex_coord_id);
        }
        let tm = s.tex_map_id;
        write!(frag_pre, "\n    var sampled{i} = textureSampleBias(tex{tm}, tex{tm}_samp, {uv_in}, ubuf.tex{tm}_size_bias.z);")
            .unwrap();
    }
    if info.uses_pt_tex_mtx != 0 {
        write!(uni, "\n    postmtx: array<mat3x4f, {MAX_PT_TEX_MTX}>,").unwrap();
    }
    if info.uses_fog {
        uniform_pre += "\nstruct Fog {\n    color: vec4f,\n    a: f32,\n    b: f32,\n    c: f32,\n    range_center: f32,\n    range_k: array<vec4f, 3>,\n}";
        uni += "\n    fog: Fog,";
        let depth = if USE_REVERSED_Z { "(1.0 - in.pos.z)" } else { "in.pos.z" };
        if c.fog_type & 0x08 != 0 {
            write!(frag, "\n    // Orthographic fog\n    var fogBase = ubuf.fog.a * {depth};").unwrap();
        } else {
            write!(frag, "\n    // Perspective fog\n    var fogBase = ubuf.fog.a / (ubuf.fog.b - {depth});").unwrap();
        }
        if c.fog_range_enabled {
            frag += "\n        fogBase *= bitcast<f32>(abuf[imm.fog_range_base + u32(in.pos.x)]);";
        }
        frag += "\n    var fogF = clamp(fogBase - ubuf.fog.c, 0.0, 1.0);";
        match c.fog_type as u32 {
            GX_FOG_PERSP_LIN | GX_FOG_ORTHO_LIN => frag += "\n    var fogZ = fogF;",
            GX_FOG_PERSP_EXP | GX_FOG_ORTHO_EXP => frag += "\n    var fogZ = 1.0 - exp2(-8.0 * fogF);",
            GX_FOG_PERSP_EXP2 | GX_FOG_ORTHO_EXP2 => frag += "\n    var fogZ = 1.0 - exp2(-8.0 * fogF * fogF);",
            GX_FOG_PERSP_REVEXP | GX_FOG_ORTHO_REVEXP => frag += "\n    var fogZ = exp2(-8.0 * (1.0 - fogF));",
            GX_FOG_PERSP_REVEXP2 | GX_FOG_ORTHO_REVEXP2 => {
                frag += "\n    fogF = 1.0 - fogF;\n    var fogZ = exp2(-8.0 * fogF * fogF);"
            }
            t => panic!("invalid fog type {t}"),
        }
        frag += "\n    prev = vec4f(mix(prev.rgb, ubuf.fog.color.rgb, clamp(fogZ, 0.0, 1.0)), prev.a);";
    }
    write!(uni, "\n    texcoord_scale: array<vec4f, {MAX_TEX_COORD}>,").unwrap();
    if info.used_ind_tex_mtxs != 0 {
        write!(uni, "\n    ind_mtx: array<mat2x4f, {MAX_IND_TEX_MTXS}>,").unwrap();
    }
    for i in 0..MAX_TEXTURES {
        if !bit(info.sampled_textures, i as u32) {
            continue;
        }
        write!(uni, "\n    tex{i}_size_bias: vec4f,").unwrap();
        write!(
            tex_bindings,
            "\n@group(2) @binding({})\nvar tex{i}: texture_2d<f32>;\n@group(2) @binding({})\nvar tex{i}_samp: sampler;",
            i * 2,
            i * 2 + 1
        )
        .unwrap();
    }
    if !prev_cn && !prev_an {
        frag += "\n    prev = tev_overflow_vec4f(prev);";
    } else if !prev_cn {
        frag += "\n    prev = vec4f(tev_overflow_vec3f(prev.rgb), prev.a);";
    } else if !prev_an {
        frag += "\n    prev.a = tev_overflow_f32(prev.a);";
    }
    if c.alpha_compare.enabled() {
        let ac = c.alpha_compare;
        let c0 = alpha_compare(ac.comp0, ac.ref0);
        let c1 = alpha_compare(ac.comp1, ac.ref1);
        let pass = match ac.op {
            GX_AOP_AND => ace_and(&c0, &c1),
            GX_AOP_OR => ace_or(&c0, &c1),
            GX_AOP_XOR => ace_xor(&c0, &c1),
            GX_AOP_XNOR => ace_xnor(&c0, &c1),
            o => panic!("invalid alpha compare op {o}"),
        };
        let discard = ace_not(&pass);
        if discard.constant == 1 {
            frag += "\n    // Alpha compare\n    discard;";
        } else if discard.constant != 0 {
            frag += "\n    // Alpha compare\n    let alphaCompare = u32(round(clamp(prev.a, 0.0, 1.0) * 255.0));";
            write!(frag, "\n    if ({}) {{ discard; }}", discard.expr).unwrap();
        }
    }

    let mut frag_output = String::new();
    let mut frag_out_type = "@location(0) vec4f";
    let mut frag_ret = "\n    return prev;".to_string();
    if normal_attachment != u32::MAX {
        frag_output = format!(
            "\nstruct FragmentOutput {{\n    @location(0) color: vec4f,\n    @location({normal_attachment}) normal: vec4f,\n}};\n"
        );
        frag_out_type = "FragmentOutput";
        frag_ret = "\n    var out: FragmentOutput;\n    out.color = prev;".into();
        if use_normal_target {
            frag_ret += "\n    let nrm_len_sq = dot(in.mv_nrm, in.mv_nrm);\n    let unit_nrm = select(vec3f(0.0), normalize(in.mv_nrm), nrm_len_sq > 1e-10);\n    out.normal = vec4f(unit_nrm * 0.5 + 0.5, select(0.0, 1.0, nrm_len_sq > 1e-10));";
        } else {
            frag_ret += "\n    out.normal = vec4f(0.5, 0.5, 0.5, 0.0);";
        }
        frag_ret += "\n    return out;";
    }

    let mut s = String::with_capacity(16384);
    s += SHADER_PRELUDE;
    s += &uniform_pre;
    s += "\n\nstruct Immediate {\n    vtx_start: u32,\n    current_pnmtx: u32,\n    fog_range_base: u32,\n    _pad: u32,\n    array_start0: vec4u,\n    array_start1: vec4u,\n    array_start2: vec4u,\n};\nvar<immediate> imm: Immediate;\n\nstruct Uniform {\n    render_viewport_size: vec2f,\n    logical_viewport_size: vec2f,";
    s += &uni;
    s += "\n};\n@group(0) @binding(0)\nvar<storage, read> vbuf: array<u32>;\n@group(0) @binding(1)\nvar<storage, read> abuf: array<u32>;\n@group(1) @binding(0)\nvar<uniform> ubuf: Uniform;";
    s += &tex_bindings;
    s += "\n\n// Matching GX position calculations must agree across material shader variants.\n// Otherwise compiler reassociation can change a coplanar layer's depth by an ULP.\nstruct VertexOutput {\n    @invariant @builtin(position) pos: vec4f,";
    s += &vtx_out;
    s += "\n};\n\n@vertex\nfn vs_main(\n    @builtin(vertex_index) vidx: u32";
    s += &vtx_in;
    s += "\n) -> VertexOutput {\n    var out: VertexOutput;";
    s += &xfr_pre;
    s += &xfr;
    s += "\n    return out;\n}\n\n";
    s += &frag_output;
    s += "\n@fragment\nfn fs_main(in: VertexOutput) -> ";
    s += frag_out_type;
    s += " {";
    s += &frag_pre;
    s += &frag;
    s += &frag_ret;
    s += "\n}\n";
    s
}

/// The fixed head of every generated shader (Aurora's format string up to `{8}`).
const SHADER_PRELUDE: &str = include_str!("prelude.wgsl");
