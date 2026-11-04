//! One decoded draw: what Aurora's `push_gx_draw` hands the renderer (`DrawData`), resolved
//! against the state at the moment of the draw. Port of `command_processor.cpp`'s
//! `prepare_idx_buffer`, `calc_vtx_size`, `max_index_for_attr` and the parts of `push_gx_draw`
//! that are not caching.
use crate::consts::*;
use crate::resources::GxData;
use crate::shader::{self, PipelineConfig, ShaderInfo};
use crate::state::*;

/// A vertex array as a draw sees it: the bytes `[0, needed)` of the array are what the shader may
/// read (Aurora uploads exactly that much, or more).
#[derive(Clone, Debug)]
pub struct ArraySnap {
    pub data: GxData,
    pub needed: u32,
}

#[derive(Clone, Debug)]
pub struct DrawCmd {
    pub prim: u32,
    pub fmt: u32,
    /// Shared with the previous draw when equal (the record is ~2.8 KB; most consecutive draws share one).
    pub config: std::sync::Arc<PipelineConfig>,
    pub info: ShaderInfo,
    pub uniform: Vec<u8>,
    /// Raw vertex bytes from the command stream (big-endian GC layout).
    pub verts: Vec<u8>,
    pub vtx_count: u32,
    /// 16-bit indices into this draw's vertices (Aurora's index buffer); empty = plain triangles.
    pub indices: Vec<u16>,
    pub instance_count: u32,
    /// Indexed attribute arrays, by attribute (POS..TEX7 -> 0..11).
    pub arrays: [Option<ArraySnap>; MAX_INDEX_ATTR],
    pub current_pn_mtx: u32,
    /// Texture objects bound to each map at draw time (only sampled ones matter).
    pub textures: [TexObj; MAX_TEXTURES],
    pub tluts: Vec<TlutObj>,
    pub render_viewport: Viewport,
    pub render_scissor: ClipRect,
    /// Not a draw: GXCopyTex at this point of the frame (everything else in the record is a placeholder).
    pub copy: Option<Box<EfbCopy>>,
    /// The PC matrix palette this draw reads (`config.shader.pn_palette`), 24 floats a slot.
    pub palette: Option<std::sync::Arc<Vec<f32>>>,
}

/// One GXCopyTex: the EFB rectangle `src` (x, y, w, h in EFB pixels) to a `dst_w` x `dst_h` texture of copy format
/// `fmt` at guest address `dest` (`half`: the 2:1 box-filtered mip copy), then, with `clear`, the EFB is cleared to the
/// copy-clear colour and depth. On the GPU the copy stays a texture keyed by `dest`; later draws whose texture object
/// points at `dest` sample it (the GC reads the bytes the copy wrote to main memory).
#[derive(Clone, Debug)]
pub struct EfbCopy {
    pub dest: u64,
    pub src: [u32; 4],
    pub dst_w: u32,
    pub dst_h: u32,
    pub fmt: u32,
    pub half: bool,
    pub clear: bool,
    pub clear_color: Vec4,
    pub clear_depth: f32,
    /// The EFB keeps alpha (RGBA6_Z24); otherwise copies read alpha as 1.
    pub efb_alpha: bool,
}

impl DrawCmd {
    /// The draw-list entry of an EFB copy.
    pub fn copy_marker(config: std::sync::Arc<PipelineConfig>, c: EfbCopy) -> Self {
        DrawCmd {
            prim: 0,
            fmt: 0,
            config,
            info: ShaderInfo::default(),
            uniform: Vec::new(),
            verts: Vec::new(),
            vtx_count: 0,
            indices: Vec::new(),
            instance_count: 0,
            arrays: Default::default(),
            current_pn_mtx: 0,
            textures: Default::default(),
            tluts: Vec::new(),
            render_viewport: Viewport::default(),
            render_scissor: ClipRect::default(),
            copy: Some(Box::new(c)),
            palette: None,
        }
    }
}

/// Aurora `calc_vtx_size`.
pub fn calc_vtx_size(st: &GxState, fmt: u32) -> u32 {
    let vf = &st.vtx_fmts[fmt as usize];
    let mut n = 0u32;
    for i in GX_VA_PNMTXIDX..=GX_VA_TEX7 {
        let af = vf[i as usize];
        match st.vtx_desc[i as usize] {
            GX_DIRECT => n += shader::comp_type_size(i, af.ty) as u32 * shader::comp_cnt_count(i, af.cnt) as u32,
            GX_INDEX8 => n += if i == GX_VA_NRM && af.cnt == GX_NRM_NBT3 { 3 } else { 1 },
            GX_INDEX16 => n += if i == GX_VA_NRM && af.cnt == GX_NRM_NBT3 { 6 } else { 2 },
            _ => {}
        }
    }
    n
}

fn attr_stream_offset(st: &GxState, attr: u32, fmt: u32) -> u32 {
    let vf = &st.vtx_fmts[fmt as usize];
    let mut off = 0;
    for i in GX_VA_PNMTXIDX..attr {
        let af = vf[i as usize];
        match st.vtx_desc[i as usize] {
            GX_DIRECT => off += shader::comp_type_size(i, af.ty) as u32 * shader::comp_cnt_count(i, af.cnt) as u32,
            GX_INDEX8 => off += if i == GX_VA_NRM && af.cnt == GX_NRM_NBT3 { 3 } else { 1 },
            GX_INDEX16 => off += if i == GX_VA_NRM && af.cnt == GX_NRM_NBT3 { 6 } else { 2 },
            _ => {}
        }
    }
    off
}

fn max_index_for_attr(st: &GxState, attr: u32, fmt: u32, verts: &[u8], n: u16) -> u32 {
    let vs = calc_vtx_size(st, fmt) as usize;
    let off = attr_stream_offset(st, attr, fmt) as usize;
    let af = st.vtx_fmts[fmt as usize][attr as usize];
    let nbt3 = attr == GX_VA_NRM && af.cnt == GX_NRM_NBT3;
    let idx16 = st.vtx_desc[attr as usize] == GX_INDEX16;
    let sub = if nbt3 { 3 } else { 1 };
    let mut max = 0u32;
    for v in 0..n as usize {
        let p = v * vs + off;
        for c in 0..sub {
            let idx = if idx16 {
                u16::from_be_bytes([verts[p + c * 2], verts[p + c * 2 + 1]]) as u32
            } else {
                verts[p + c] as u32
            };
            max = max.max(idx);
        }
    }
    max
}

/// Aurora `prepare_idx_buffer` (vtxStart = 0).
pub fn prepare_idx_buffer(prim: u32, vtx_count: u16) -> Vec<u16> {
    let n = vtx_count as usize;
    // sized up front: growing it index by index was ~40% of all allocations in a 4-CPU Temple tick
    let cap = match prim {
        GX_QUADS => n.div_ceil(4) * 6,
        GX_TRIANGLEFAN | GX_TRIANGLESTRIP => n.min(3) + 3 * n.saturating_sub(3),
        GX_LINES | GX_LINESTRIP | GX_POINTS => 6,
        _ => n,
    };
    let mut b = Vec::with_capacity(cap);
    match prim {
        GX_QUADS => {
            let mut v = 0u16;
            while v < vtx_count {
                b.extend_from_slice(&[v, v + 1, v + 2, v + 2, v + 3, v]);
                v += 4;
            }
        }
        GX_TRIANGLES => b.extend(0..vtx_count),
        GX_TRIANGLEFAN => {
            for v in 0..vtx_count {
                if v < 3 {
                    b.push(v);
                } else {
                    b.extend_from_slice(&[0, v - 1, v]);
                }
            }
        }
        GX_TRIANGLESTRIP => {
            for v in 0..vtx_count {
                if v < 3 {
                    b.push(v);
                } else if v & 1 == 0 {
                    b.extend_from_slice(&[v - 2, v - 1, v]);
                } else {
                    b.extend_from_slice(&[v - 1, v - 2, v]);
                }
            }
        }
        GX_LINES | GX_LINESTRIP | GX_POINTS => b.extend_from_slice(&[0, 1, 3, 3, 2, 0]),
        _ => panic!("unsupported primitive type {prim}"),
    }
    b
}

/// Build a draw from the current state (Aurora `push_gx_draw` + `handle_draw_unmerged`).
pub fn build_draw(
    st: &mut GxState,
    prim: u32,
    fmt: u32,
    vtx_count: u16,
    verts: Vec<u8>,
    indices: Option<Vec<u16>>,
) -> Option<DrawCmd> {
    if vtx_count == 0 {
        return None;
    }
    let mut arrays: [Option<ArraySnap>; MAX_INDEX_ATTR] = Default::default();
    for i in GX_VA_POS..=GX_VA_TEX7 {
        let t = st.vtx_desc[i as usize];
        if t != GX_INDEX8 && t != GX_INDEX16 {
            continue;
        }
        let af = st.vtx_fmts[fmt as usize][i as usize];
        let a = &st.arrays[i as usize];
        let needed = max_index_for_attr(st, i, fmt, &verts, vtx_count) * a.stride as u32
            + shader::comp_type_size(i, af.ty) as u32 * shader::comp_cnt_count(i, af.cnt) as u32;
        let Some(data) = a.data.clone() else {
            st.warnings.push(format!("indexed attr {i} has no array"));
            return None;
        };
        assert!(needed <= a.size, "indexed attr {i} references {needed} bytes, array is {} bytes", a.size);
        arrays[(i - GX_VA_POS) as usize] = Some(ArraySnap { data, needed });
    }
    let config = shader::populate_pipeline_config(st, prim, fmt);
    let (config, info) = match st.last_config.as_ref() {
        Some((c, info)) if **c == config => (c.clone(), *info),
        _ => {
            let info = shader::build_shader_info(&config.shader);
            let c = std::sync::Arc::new(config);
            st.last_config = Some((c.clone(), info));
            (c, info)
        }
    };
    let palette = if config.shader.pn_palette { st.palette.clone() } else { None };
    let textures: [TexObj; MAX_TEXTURES] = st.loaded_textures.clone();
    let uniform = shader::fill_uniform(st, &info, &textures);
    let indices = match indices {
        Some(i) => i,
        None if prim != GX_TRIANGLES => prepare_idx_buffer(prim, vtx_count),
        None => Vec::new(),
    };
    let instance_count = match prim {
        GX_LINES => vtx_count as u32 / 2,
        GX_LINESTRIP => vtx_count as u32 - 1,
        GX_POINTS => vtx_count as u32,
        _ => 1,
    };
    Some(DrawCmd {
        prim,
        fmt,
        config,
        info,
        uniform,
        verts,
        vtx_count: vtx_count as u32,
        indices,
        instance_count,
        arrays,
        current_pn_mtx: st.current_pn_mtx,
        textures,
        tluts: st.loaded_tluts.to_vec(),
        render_viewport: st.render_viewport,
        render_scissor: st.render_scissor,
        copy: None,
        palette,
    })
}
