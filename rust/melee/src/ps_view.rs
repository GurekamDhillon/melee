//! HSD particle display (`psdisp.c`, `psdisptev.c`) for `melee view-golden`: the simulation's native
//! particles (`melee_runtime::particle::PsSystem`) drawn through melee_gx's `GxShim`.
//!
//! Ported: `psDispParticles` (per-link sort by TexEdge/blend bucket, per-particle blend, alpha
//! compare, TEV by kind, tex-edge z mode, TEV/channel colours, texture + TLUT loads), `psDispSub`
//! (camera-facing quads, rotation, trail/direction quads), `psDispSubPoint` (unbatched).
//! AppSRT particles (`psDispSubAppSRT` quads / points, view-space frame) are ported but OFF by default
//! (`MELEE_VIEW_APPSRT=1`): the KO / blast-zone effects they draw render as oversized blocky quads next to
//! the C build's thin beams (texture or extent still wrong), which scores worse than skipping them.
//! NOT ported: form-bank polygons (no form bank
//! is loaded natively), point-trail batches, fog (`DispFog` draws unfogged), light-mask diffuse
//! (lit particles use their ambient colour only).
use crate::gx_backend::DataCache;
use melee_gx::api::GxColor;
use melee_gx::consts::*;
use melee_gx::shim::{get_tex_buffer_size, GxShim};
use melee_gx::state::{TexObj, TlutObj};
use melee_runtime::mtx::{self, Mtx, Vec3};
use melee_runtime::particle::{Particle, PsSystem, Rgba};

const TORNADO: u32 = 1 << 2;
const TEX_EDGE: u32 = 1 << 3;
const COM_TLUT: u32 = 1 << 4;
const MIRROR_S: u32 = 1 << 5;
const MIRROR_T: u32 = 1 << 6;
const PRIM_ENV: u32 = 1 << 7;
const TEX_INTERP_NEAR: u32 = 1 << 9;
const DISP_TEXTURE: u32 = 1 << 10;
const TRAIL: u32 = 1 << 20;
const DIR_VEC: u32 = 1 << 21;
const DISP_POINT: u32 = 1 << 30;
const DISP_LIGHTING: u32 = 1 << 31;

/// `HSD_PSDisp_8040C340`: the quad's texture coordinates, 2 bytes per index (4 per flip variant).
const TEX_TABLE: [u8; 32] = [0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 0];

fn bright(c: Rgba) -> GxColor {
    GxColor::new(c.r, c.g, c.b, c.a)
}

fn lerp_col(cur: Rgba, target: Rgba, count: u16, remain: u16) -> Rgba {
    let scale = 65536i32.wrapping_mul(remain as i32) / count as i32;
    let f = |t: u8, c: u8| ((((t as i32) << 16).wrapping_add((c as i32 - t as i32).wrapping_mul(scale))) >> 16) as u8;
    Rgba { r: f(target.r, cur.r), g: f(target.g, cur.g), b: f(target.b, cur.b), a: f(target.a, cur.a) }
}

/// `getColorPrimEnv`.
fn prim_env(p: &Particle) -> (Rgba, Rgba) {
    let prim = if p.prim_col_count != 0 { lerp_col(p.prim_col, p.prim_col_target, p.prim_col_count, p.prim_col_remain) } else { p.prim_col };
    let env = if p.env_col_count != 0 { lerp_col(p.env_col, p.env_col_target, p.env_col_count, p.env_col_remain) } else { p.env_col };
    (prim, env)
}

/// `getColorMatAmb`.
fn mat_amb(p: &Particle) -> (Rgba, Rgba) {
    let f = |t: u8, c: u8, scale: i32| ((((t as i32) << 16).wrapping_add((c as i32 - t as i32).wrapping_mul(scale))) >> 16) as u8;
    let mat = if p.mat_col_count != 0 {
        let s = 65536i32.wrapping_mul(p.mat_col_remain as i32) / p.mat_col_count as i32;
        let v = f(p.mat_rgb_target, p.mat_rgb, s);
        Rgba { r: v, g: v, b: v, a: f(p.mat_a_target, p.mat_a, s) }
    } else {
        Rgba { r: p.mat_rgb, g: p.mat_rgb, b: p.mat_rgb, a: p.mat_a }
    };
    let amb = if p.amb_col_count != 0 {
        let s = 65536i32.wrapping_mul(p.amb_col_remain as i32) / p.amb_col_count as i32;
        let v = f(p.amb_rgb_target, p.amb_rgb, s);
        Rgba { r: v, g: v, b: v, a: f(p.amb_a_target, p.amb_a, s) }
    } else {
        Rgba { r: p.amb_rgb, g: p.amb_rgb, b: p.amb_rgb, a: p.amb_a }
    };
    (mat, amb)
}

/// `psSetupTev`: `kind & 0x80100480` picks the TEV program (the C also clears kind bit 7 in the
/// particle for the "no texture" cases; the key here carries the same masking).
fn setup_tev(g: &mut GxShim, key: u32) {
    let k = match key {
        0x8000_0080 => 0x8000_0000,
        0x80 => 0,
        0x10_0080 => 0x10_0000,
        0x8010_0080 => 0x8010_0000,
        k => k,
    };
    let s0 = GX_TEVSTAGE0;
    let s1 = GX_TEVSTAGE0 + 1;
    let s2 = GX_TEVSTAGE2;
    let order_tex = |g: &mut GxShim| g.set_tev_order(s0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
    let order_none = |g: &mut GxShim| g.set_tev_order(s0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
    match k {
        0x8000_0000 => {
            g.set_num_tev_stages(2);
            g.set_num_tex_gens(0);
            order_none(g);
            g.set_tev_color_in(s0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_A2, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO);
            g.set_tev_color_in(s1, GX_CC_ZERO, GX_CC_C0, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s1, GX_CA_ZERO, GX_CA_A0, GX_CA_APREV, GX_CA_ZERO);
        }
        0x0 => {
            g.set_num_tev_stages(1);
            g.set_num_tex_gens(0);
            order_none(g);
            g.set_tev_color_in(s0, GX_CC_C0, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_A0, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO);
        }
        0x8000_0480 => {
            g.set_num_tev_stages(3);
            g.set_num_tex_gens(1);
            order_tex(g);
            g.set_tev_color_in(s0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA, GX_CA_ZERO);
            g.set_tev_color_in(s1, GX_CC_ZERO, GX_CC_C2, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s1, GX_CA_ZERO, GX_CA_A2, GX_CA_APREV, GX_CA_ZERO);
            g.set_tev_color_in(s2, GX_CC_ZERO, GX_CC_RASC, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s2, GX_CA_APREV, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO);
        }
        0x8000_0400 => {
            g.set_num_tev_stages(2);
            g.set_num_tex_gens(1);
            order_tex(g);
            g.set_tev_color_in(s0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_A2, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO);
            g.set_tev_color_in(s1, GX_CC_ZERO, GX_CC_TEXC, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s1, GX_CA_ZERO, GX_CA_TEXA, GX_CA_APREV, GX_CA_ZERO);
        }
        0x400 | 0x480 => {
            g.set_num_tev_stages(1);
            g.set_num_tex_gens(1);
            order_tex(g);
            g.set_tev_color_in(s0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA, GX_CA_ZERO);
        }
        0x10_0400 => {
            g.set_num_tev_stages(1);
            g.set_num_tex_gens(1);
            order_tex(g);
            g.set_tev_op(s0, GX_MODULATE);
        }
        0x10_0480 => {
            g.set_num_tev_stages(2);
            g.set_num_tex_gens(1);
            order_tex(g);
            g.set_tev_color_in(s0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA, GX_CA_ZERO);
            g.set_tev_color_in(s1, GX_CC_CPREV, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO);
            g.set_tev_alpha_in(s1, GX_CA_ZERO, GX_CA_RASA, GX_CA_APREV, GX_CA_ZERO);
        }
        0x8010_0400 => {
            g.set_num_tev_stages(2);
            g.set_num_tex_gens(1);
            order_tex(g);
            g.set_tev_color_in(s0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_ZERO, GX_CA_A2, GX_CA_RASA, GX_CA_ZERO);
            g.set_tev_color_in(s1, GX_CC_ZERO, GX_CC_TEXC, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s1, GX_CA_ZERO, GX_CA_TEXA, GX_CA_APREV, GX_CA_ZERO);
        }
        0x8010_0480 => {
            g.set_num_tev_stages(3);
            g.set_num_tex_gens(1);
            order_tex(g);
            g.set_tev_color_in(s0, GX_CC_C1, GX_CC_C0, GX_CC_TEXC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_A1, GX_CA_A0, GX_CA_TEXA, GX_CA_ZERO);
            g.set_tev_color_in(s1, GX_CC_ZERO, GX_CC_C2, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s1, GX_CA_ZERO, GX_CA_A2, GX_CA_APREV, GX_CA_ZERO);
            g.set_tev_color_in(s2, GX_CC_ZERO, GX_CC_RASC, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s2, GX_CA_ZERO, GX_CA_RASA, GX_CA_APREV, GX_CA_ZERO);
        }
        0x10_0000 => {
            g.set_num_tev_stages(1);
            g.set_num_tex_gens(0);
            order_none(g);
            g.set_tev_color_in(s0, GX_CC_RASC, GX_CC_ZERO, GX_CC_ZERO, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_RASA, GX_CA_ZERO, GX_CA_ZERO, GX_CA_ZERO);
        }
        0x8010_0000 => {
            g.set_num_tev_stages(2);
            g.set_num_tex_gens(0);
            order_none(g);
            g.set_tev_color_in(s0, GX_CC_ZERO, GX_CC_C2, GX_CC_RASC, GX_CC_ZERO);
            g.set_tev_alpha_in(s0, GX_CA_ZERO, GX_CA_A2, GX_CA_RASA, GX_CA_ZERO);
            g.set_tev_color_in(s1, GX_CC_ZERO, GX_CC_C0, GX_CC_CPREV, GX_CC_ZERO);
            g.set_tev_alpha_in(s1, GX_CA_ZERO, GX_CA_A0, GX_CA_APREV, GX_CA_ZERO);
        }
        _ => {}
    }
}

/// `psSetupTevCommon`.
fn setup_tev_common(g: &mut GxShim) {
    for st in 0..3u32 {
        g.set_tev_order(st, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        g.set_tev_color_op(st, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
        g.set_tev_alpha_op(st, GX_TEV_ADD, GX_TB_ZERO, GX_CS_SCALE_1, true, GX_TEVPREV);
        g.set_tev_swap_mode(st, GX_TEV_SWAP0, GX_TEV_SWAP0);
    }
}

/// Render-side state `psDispParticles` keeps in locals and file statics.
struct State {
    vmtx: Mtx,
    rvmtx: Mtx,
    pvmtx: [[f32; 4]; 2],
    prj: [f32; 7],
    persp: bool,
    axes: [f32; 6],
    prev_tev: u32,
    blend: i32,
    a_mode: i32,
    a0: u8,
    a1: u8,
    chan_ctrl: i32,
    tex_edge: i32,
    mirror: i32,
    tex_image: usize,
    tex_near: i32,
    tlut: usize,
    point_size: i32,
    prim: Rgba,
    env: Rgba,
    mat: Rgba,
    chan_mat: Rgba,
    chan_amb: Rgba,
    skipped_appsrt: u32,
}

const WHITE: Rgba = Rgba { r: 255, g: 255, b: 255, a: 255 };

fn vtx_desc(g: &mut GxShim, fmt: u32) {
    g.clear_vtx_desc();
    g.set_vtx_desc(GX_VA_POS, GX_DIRECT);
    match fmt {
        0 => g.set_vtx_desc(GX_VA_TEX0, GX_INDEX8),
        1 => {}
        2 => {
            g.set_vtx_desc(GX_VA_CLR0, GX_DIRECT);
            g.set_vtx_desc(GX_VA_TEX0, GX_INDEX8);
        }
        3 => g.set_vtx_desc(GX_VA_CLR0, GX_DIRECT),
        _ => {}
    }
}

fn mtx_mul_vec(m: &Mtx, v: [f32; 3]) -> [f32; 3] {
    [m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2], m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2], m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2]]
}

/// `psDispParticles(links, sw)`: `sw` 1 draws the TexEdge particles (sorted), 2 the others, 0 only
/// advances the C's frame counter.
pub fn disp_particles(shim: &mut GxShim, cache: &mut DataCache, ps: &PsSystem, links: u32, sw: u32, vmtx: &Mtx, tex_cache_epoch: &mut u64) {
    if sw == 0 {
        return;
    }
    let mut st: Option<State> = None;
    for link in 0..16usize {
        if links & (1 << link) == 0 {
            continue;
        }
        // particleSort: stable partition into 16 buckets by (!TexEdge)*8 + ((kind >> 25) & 7).
        let mut ids: Vec<(u32, melee_runtime::particle::PId)> = ps
            .particles(link)
            .into_iter()
            .map(|id| {
                let k = ps.part(id).kind;
                (if k & TEX_EDGE != 0 { 0 } else { 8 } + ((k >> 25) & 7), id)
            })
            .collect();
        ids.sort_by_key(|x| x.0);
        for (_, id) in ids {
            let p = ps.part(id);
            let edge = p.kind & TEX_EDGE != 0;
            if sw == 1 && !edge {
                break;
            }
            if sw != 1 && edge {
                continue;
            }
            if p.size < 1.192_092_9e-7 {
                continue;
            }
            if p.appsrt.is_some() && std::env::var_os("MELEE_VIEW_APPLOG").is_some() {
                eprintln!("APPSRT particle link {link} sw {sw} bank {} tg {} size {} kind {:#x}", p.bank, p.tex_group, p.size, p.kind);
            }
            if p.appsrt.is_some() && std::env::var_os("MELEE_VIEW_APPSRT").is_none() {
                continue;
            }
            if st.is_none() {
                st = Some(setup(shim, vmtx));
            }
            let s = st.as_mut().unwrap();
            draw_one(shim, cache, ps, p, s);
        }
    }
    if let Some(s) = &st {
        *tex_cache_epoch += 1;
        if std::env::var_os("MELEE_VIEW_PSLOG").is_some() {
            eprintln!("PSDISP links {links:#x} sw {sw}: appsrt-skipped {}", s.skipped_appsrt);
        }
    }
}

fn setup(g: &mut GxShim, vmtx: &Mtx) -> State {
    g.set_fog(GX_FOG_NONE, 0.0, 0.0, 0.0, 0.0, GxColor::new(0, 0, 0, 255));
    g.set_chan_mat_color(GX_COLOR0A0, GxColor::new(255, 255, 255, 255));
    g.set_chan_amb_color(GX_COLOR0A0, GxColor::new(255, 255, 255, 255));
    setup_tev_common(g);
    g.set_tev_color(GX_TEVREG0, GxColor::new(255, 255, 255, 255));
    g.set_tev_color(GX_TEVREG1, GxColor::new(0, 0, 0, 0));
    g.set_tev_color(GX_TEVREG2, GxColor::new(255, 255, 255, 255));
    g.set_z_comp_loc(false);
    g.set_color_update(true);
    let rv = mtx::inverse(vmtx).unwrap_or_else(mtx::identity);
    let prj = g.get_projectionv();
    let persp = prj[0] == 0.0;
    let mut pv = [[0.0f32; 4]; 2];
    if persp {
        for c in 0..4 {
            pv[0][c] = prj[1] * vmtx[0][c] + prj[2] * vmtx[2][c];
            pv[1][c] = prj[3] * vmtx[1][c] + prj[4] * vmtx[2][c];
        }
    } else {
        for c in 0..4 {
            pv[0][c] = prj[1] * vmtx[0][c] + prj[2];
            pv[1][c] = prj[3] * vmtx[1][c] + prj[4];
        }
    }
    // psUpdateBillboardAxes
    let (rx, ux, ry, uy, rz, uz) = (rv[0][0], rv[0][1], rv[1][0], rv[1][1], rv[2][0], rv[2][1]);
    let axes = [rx + ux, rx - ux, ry + uy, ry - uy, rz + uz, rz - uz];
    let mut v = *vmtx;
    g.load_pos_mtx_imm(&mut v, GX_PNMTX0);
    // HSD_PSDisp_803B9628 (the AppSRT particles' matrix: their positions are already in view space).
    let mut ident = mtx::identity();
    g.load_pos_mtx_imm(&mut ident, GX_PNMTX1);
    g.set_current_mtx(GX_PNMTX0);
    g.enable_tex_offsets(GX_TEXCOORD0, true, true);
    g.set_cull_mode(GX_CULL_BACK);
    let table = melee_gx::GxData::from(&TEX_TABLE[..]);
    g.set_array(GX_VA_TEX0, Some(&table), 2);
    let fmts: [(u32, bool, bool, u32); 6] = [(0, false, true, 0), (1, false, false, 0), (2, true, true, 0), (3, true, false, 0), (4, false, true, 4), (5, true, true, 4)];
    for (f, clr, tex, tt) in fmts {
        g.set_vtx_attr_fmt(f, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
        if clr {
            g.set_vtx_attr_fmt(f, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
        }
        if tex {
            g.set_vtx_attr_fmt(f, GX_VA_TEX0, GX_TEX_ST, tt, 0);
        }
    }
    State {
        vmtx: *vmtx,
        rvmtx: rv,
        pvmtx: pv,
        prj: [prj[0], prj[1], prj[2], prj[3], prj[4], prj[5], prj[6]],
        persp,
        axes,
        prev_tev: u32::MAX,
        blend: -1,
        a_mode: 0,
        a0: 0,
        a1: 0xFF,
        chan_ctrl: -1,
        tex_edge: -1,
        mirror: -1,
        tex_image: 0,
        tex_near: 0,
        tlut: 0,
        point_size: -1,
        prim: WHITE,
        env: Rgba { r: 0, g: 0, b: 0, a: 0 },
        mat: WHITE,
        chan_mat: WHITE,
        chan_amb: WHITE,
        skipped_appsrt: 0,
    }
}

fn draw_one(g: &mut GxShim, cache: &mut DataCache, ps: &PsSystem, p: &Particle, s: &mut State) {
    let k = p.kind;
    // setBlendMode
    let bm = ((k >> 22) & 3) as i32;
    if s.blend != bm {
        s.blend = bm;
        match bm {
            0 => g.set_blend_mode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_INVSRCALPHA, GX_LO_CLEAR),
            1 => g.set_blend_mode(GX_BM_BLEND, GX_BL_SRCALPHA, GX_BL_ONE, GX_LO_CLEAR),
            _ => {}
        }
    }
    // Alpha compare.
    let (a0, a1) = if p.a_cmp_count != 0 {
        let sc = 65536i32.wrapping_mul(p.a_cmp_remain as i32) / p.a_cmp_count as i32;
        let f = |t: u8, c: u8| ((((t as i32) << 16).wrapping_add((c as i32 - t as i32).wrapping_mul(sc))) >> 16) as u8;
        (f(p.a_cmp_param1_target, p.a_cmp_param1), f(p.a_cmp_param2_target, p.a_cmp_param2))
    } else {
        (p.a_cmp_param1, p.a_cmp_param2)
    };
    let am = p.a_cmp_mode as i32;
    if s.a_mode != am || s.a0 != a0 || s.a1 != a1 || s.prev_tev == u32::MAX {
        s.a_mode = am;
        s.a0 = a0;
        s.a1 = a1;
        g.set_alpha_compare(((am >> 3) & 7) as u32, a0, ((am >> 6) & 3) as u32, (am & 7) as u32, a1);
    }
    // psSetupTev
    let key = k & 0x8010_0480;
    if key != s.prev_tev {
        s.prev_tev = key;
        setup_tev(g, key);
    }
    // setupChanCtrl
    let cs = (k & (DISP_LIGHTING | TRAIL)) as i32;
    if cs != s.chan_ctrl {
        s.chan_ctrl = cs;
        g.set_num_chans(1);
        match (k & DISP_LIGHTING != 0, k & TRAIL != 0) {
            (false, true) => g.set_chan_ctrl(GX_COLOR0A0, false, GX_SRC_VTX, GX_SRC_VTX, 0, GX_DF_NONE, GX_AF_NONE),
            (true, trail) => {
                g.set_chan_ctrl(GX_COLOR0, true, GX_SRC_REG, GX_SRC_REG, 0, GX_DF_NONE, GX_AF_NONE);
                let src = if trail { GX_SRC_VTX } else { GX_SRC_REG };
                g.set_chan_ctrl(GX_ALPHA0, false, src, src, 0, GX_DF_NONE, GX_AF_NONE);
            }
            _ => g.set_chan_ctrl(GX_COLOR0A0, false, GX_SRC_REG, GX_SRC_REG, 0, GX_DF_NONE, GX_AF_NONE),
        }
    }
    let (prim, env) = prim_env(p);
    // setupChanReg
    if k & DISP_LIGHTING != 0 {
        let (mat, amb) = mat_amb(p);
        let (chan_m, mut amb) = if k & PRIM_ENV != 0 {
            (Rgba { r: 255, g: 255, b: 255, a: 0 }, amb)
        } else {
            let a = |x: u8, y: u8| ((x as u32 * y as u32) >> 8) as u8;
            (prim, Rgba { r: a(amb.r, prim.r), g: a(amb.g, prim.g), b: a(amb.b, prim.b), a: amb.a })
        };
        if (chan_m.r, chan_m.g, chan_m.b) != (s.chan_mat.r, s.chan_mat.g, s.chan_mat.b) {
            s.chan_mat = chan_m;
            g.set_chan_mat_color(GX_COLOR0, bright(chan_m));
        }
        amb.a = 255;
        if (amb.r, amb.g, amb.b) != (s.chan_amb.r, s.chan_amb.g, s.chan_amb.b) {
            s.chan_amb = amb;
            g.set_chan_amb_color(GX_COLOR0, bright(amb));
        }
        let _ = mat;
    }
    // setupTevReg
    if k & PRIM_ENV != 0 || (k & DISP_LIGHTING == 0 && k & TRAIL == 0) {
        if s.prim != prim {
            s.prim = prim;
            g.set_tev_color(GX_TEVREG0, bright(prim));
        }
        if k & PRIM_ENV != 0 {
            if s.env != env {
                s.env = env;
                g.set_tev_color(GX_TEVREG1, bright(env));
            }
        } else if s.env != (Rgba { r: 0, g: 0, b: 0, a: 0 }) {
            s.env = Rgba { r: 0, g: 0, b: 0, a: 0 };
            g.set_tev_color(GX_TEVREG1, bright(s.env));
        }
    }
    if k & DISP_LIGHTING != 0 {
        let (mut mat, _) = mat_amb(p);
        if k & PRIM_ENV == 0 {
            mat.a = ((mat.a as u32 * prim.a as u32) >> 8) as u8;
        }
        if s.mat != mat {
            s.mat = mat;
            g.set_tev_color(GX_TEVREG2, bright(mat));
        }
    }
    let edge = (k & TEX_EDGE) as i32;
    if s.tex_edge != edge {
        s.tex_edge = edge;
        g.set_z_mode(true, GX_LEQUAL, edge != 0);
    }

    // Texture.
    if k & DISP_TEXTURE != 0 {
        let (scale_s, wrap_s) = if k & MIRROR_S != 0 { (2.0, GX_MIRROR) } else { (1.0, GX_CLAMP) };
        let (scale_t, wrap_t) = if k & MIRROR_T != 0 { (2.0, GX_MIRROR) } else { (1.0, GX_CLAMP) };
        let mm = (k & (MIRROR_S | MIRROR_T)) as i32;
        if s.mirror != mm {
            s.mirror = mm;
            s.tex_image = 0;
            let mut m = mtx::scale(scale_s, scale_t, 1.0);
            if k & MIRROR_T != 0 {
                m[1][3] = 1.0;
            }
            g.load_tex_mtx_imm(&m, GX_TEXMTX0, GX_MTX2x4);
            g.set_tex_coord_gen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_TEXMTX0, false, GX_PTIDENTITY);
        }
        if let Some(bank) = ps.banks.get(p.bank as usize).and_then(|b| b.as_ref()) {
            let tex = &bank.data.tex;
            if let Some(info) = tex.group_info(p.tex_group as u32) {
                let data = tex.data();
                let image = tex.image_off(p.tex_group as u32, p.pose_num as u32);
                let ci = info.fmt == GX_TF_C4 || info.fmt == GX_TF_C8;
                let mut tlut_name = None;
                if ci {
                    let pal = if p.pal_num != 0xFF {
                        tex.pal_off(p.tex_group as u32, p.pal_num as u32)
                    } else if k & COM_TLUT == 0 {
                        tex.pal_off(p.tex_group as u32, p.pose_num as u32)
                    } else {
                        tex.pal_off(p.tex_group as u32, 0)
                    };
                    if let Some(po) = pal {
                        let n = if info.fmt == GX_TF_C4 { 0x10u16 } else { 0x100 };
                        let key = po as usize + 1;
                        if s.tlut != key {
                            s.tlut = key;
                            let end = (po as usize + n as usize * 2).min(data.len());
                            let d = cache.get(&data[po as usize..end]);
                            let mut o = TlutObj::default();
                            g.init_tlut_obj(&mut o, Some(d), info.tlutfmt & 0xFF, n);
                            g.load_tlut(&o, GX_TLUT0);
                        }
                        s.tex_image = 0;
                        tlut_name = Some(GX_TLUT0);
                    }
                }
                let near = (k & TEX_INTERP_NEAR) as i32;
                if let Some(io) = image {
                    let key = io as usize + 1 + ((p.bank as usize) << 28);
                    if s.tex_image != key || s.tex_near != near {
                        s.tex_image = key;
                        s.tex_near = near;
                        let size = get_tex_buffer_size(info.width as u16, info.height as u16, info.fmt, false, 0).max(32) as usize;
                        let end = (io as usize + size).min(data.len());
                        let d = cache.get(&data[io as usize..end]);
                        let mut o = TexObj::default();
                        match tlut_name {
                            Some(n) => g.init_tex_obj_ci(&mut o, Some(d), info.width as u16, info.height as u16, info.fmt, wrap_s, wrap_t, false, n),
                            None => g.init_tex_obj(&mut o, Some(d), info.width as u16, info.height as u16, info.fmt, wrap_s, wrap_t, false),
                        }
                        let f = if near != 0 { GX_NEAR } else { GX_LINEAR };
                        GxShim::init_tex_obj_lod(&mut o, f, f, 0.0, 0.0, 0.0, false, false, GX_ANISO_1);
                        g.load_tex_obj(&mut o, GX_TEXMAP0);
                    }
                }
            }
        }
    }

    if p.appsrt.is_some() && std::env::var_os("MELEE_VIEW_APPTEX").is_some() {
        if let Some(bank) = ps.banks.get(0).and_then(|b| b.as_ref()) {
            for t in 0..32u32 {
                if let Some(i) = bank.data.tex.group_info(t) {
                    eprintln!("BANK0 tg {t}: fmt {} {}x{} tlutfmt {:#x} img0 {:?}", i.fmt, i.width, i.height, i.tlutfmt, bank.data.tex.image_off(t, 0));
                }
            }
        }
        let bsel: usize = std::env::var("MELEE_VIEW_APPBANK").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        if let Some(bank) = ps.banks.get(bsel).and_then(|b| b.as_ref()) {
            let tex = &bank.data.tex;
            let mut out = String::new();
            for t in [0u32, 1, 2, 23, 24, 25, 11, 4] {
                if let (Some(info), Some(io)) = (tex.group_info(t), tex.image_off(t, 0)) {
                    let size = get_tex_buffer_size(info.width as u16, info.height as u16, info.fmt, false, 0).max(32) as usize;
                    let data = tex.data();
                    let end = (io as usize + size).min(data.len());
                    let c = melee_gx::texture::convert_texture(info.fmt, info.width as u32, info.height as u32, 1, &data[io as usize..end]);
                    // PPM (RGBA -> RGB over black)
                    let mut ppm = format!("P6 {} {} 255\n", info.width, info.height).into_bytes();
                    for px in c.data.chunks(4).take((info.width as usize) * (info.height as usize)) {
                        let a = px.get(3).copied().unwrap_or(255) as u32;
                        ppm.push(px[0]);
                        ppm.push(((px[1] as u32 * 1) as u8).min(255));
                        ppm.push(px[2]);
                        let _ = a;
                    }
                    let path = format!("{}/tg{t}.ppm", std::env::var("MELEE_VIEW_APPTEX").unwrap());
                    let _ = std::fs::write(&path, ppm);
                    out += &format!(" {t}:{path}");
                }
            }
            eprintln!("APPTEX wrote{out}");
        }
        std::process::exit(0);
    }
    if p.appsrt.is_some() && std::env::var_os("MELEE_VIEW_APPLOG").is_some() {
        if let Some(bank) = ps.banks.get(p.bank as usize).and_then(|b| b.as_ref()) {
            if let Some(info) = bank.data.tex.group_info(p.tex_group as u32) {
                eprintln!("APPSRT tex tg {} fmt {} {}x{} tlutfmt {:#x} pose {} pal {} kind {:#x} a_cmp {} {} {}", p.tex_group, info.fmt, info.width, info.height, info.tlutfmt, p.pose_num, p.pal_num, k, p.a_cmp_mode, p.a_cmp_param1, p.a_cmp_param2);
            }
        }
    }
    if let Some(sid) = p.appsrt {
        let m = appsrt_matrix(ps.srt(sid), &s.vmtx);
        if k & DISP_POINT != 0 {
            appsrt_point(g, p, s, &m);
        } else {
            appsrt_quad(g, p, s, &m, prim);
        }
    } else if k & DISP_POINT != 0 {
        point(g, p, s);
    } else {
        quad(g, p, s, prim);
    }
}

/// The matrix `psDispSubAppSRT` leaves in the AppSRT record (`ssx .. x98`): the view-space 3x4 and the
/// column lengths `x94` / `x98`.
struct AppM {
    m: Mtx,
    ex: f32,
    ey: f32,
}

/// `HSD_MtxSRT(mmtx, scale, rot, translate)` (unless STILL) and `PSMTXConcat(vmtx, mmtx)`; with `xA2` the
/// translation-only variant (`HSD_MtxGetScale` of the translated view matrix on the diagonal).
fn appsrt_matrix(srt: &melee_runtime::particle::AppSrt, vmtx: &Mtx) -> AppM {
    const STILL: u8 = 2;
    let mm = if srt.status != STILL {
        melee_runtime::hsd_mtx::srt(&srt.scale, &Vec3 { x: srt.rot.x, y: srt.rot.y, z: srt.rot.z }, &srt.translate, None)
    } else {
        srt.mmtx
    };
    let c = mtx::concat(vmtx, &mm);
    let ex = (c[0][0] * c[0][0] + c[1][0] * c[1][0] + c[2][0] * c[2][0]).sqrt();
    let ey = (c[0][1] * c[0][1] + c[1][1] * c[1][1] + c[2][1] * c[2][1]).sqrt();
    let mut m = c;
    if srt.x_a2 != 0 {
        let mut t = mtx::identity();
        t[0][3] = srt.translate.x;
        t[1][3] = srt.translate.y;
        t[2][3] = srt.translate.z;
        let t = mtx::concat(vmtx, &t);
        let len = |c: usize| (t[0][c] * t[0][c] + t[1][c] * t[1][c] + t[2][c] * t[2][c]).sqrt();
        m = [[len(0), 0.0, 0.0, t[0][3]], [0.0, len(1), 0.0, t[1][3]], [0.0, 0.0, len(2), t[2][3]]];
    }
    AppM { m, ex, ey }
}

fn m_apply(m: &Mtx, x: f32, y: f32, z: f32) -> [f32; 3] {
    [m[0][3] + (m[0][2] * z + (m[0][0] * x + m[0][1] * y)), m[1][3] + (m[1][2] * z + (m[1][0] * x + m[1][1] * y)), m[2][3] + (m[2][2] * z + (m[2][0] * x + m[2][1] * y))]
}

/// `psDispSubAPPSRTPoint` (points; the Trail line variant is not drawn).
fn appsrt_point(g: &mut GxShim, p: &Particle, s: &mut State, a: &AppM) {
    g.set_current_mtx(GX_PNMTX1);
    let c = m_apply(&a.m, p.pos.x, p.pos.y, p.pos.z);
    if p.kind & TRAIL != 0 {
        return;
    }
    let w = if p.size > 42.5 { 255.0f32 } else { 6.0 * p.size } as u8;
    if s.point_size != w as i32 {
        s.point_size = w as i32;
        g.set_point_size(w, GX_TO_ONE);
    }
    let tex = p.kind & DISP_TEXTURE != 0;
    vtx_desc(g, if tex { 0 } else { 1 });
    g.begin(GX_POINTS, if tex { 0 } else { 1 }, 1);
    g.gx.position3f32(c[0], c[1], c[2]);
    if tex {
        g.gx.tex_coord1x8(1);
    }
    g.end();
}

/// `psDispSubAppSRT` with no form table: the camera-facing quad in the AppSRT's view-space frame.
fn appsrt_quad(g: &mut GxShim, p: &Particle, s: &mut State, a: &AppM, prim: Rgba) {
    let k = p.kind;
    let m = &a.m;
    let (x, y, z) = (p.pos.x, p.pos.y, p.pos.z);
    let cur = m_apply(m, x, y, z);
    let (px, py, pz) = (x - p.vel.x, y - p.vel.y, z - p.vel.z);
    let prev = m_apply(m, px, py, pz);
    let x_extent = a.ex * p.size;
    let y_extent = a.ey * p.size;
    if std::env::var_os("MELEE_VIEW_APPLOG").is_some() {
        eprintln!("APPSRT quad bank {} tg {} pos {:.1} {:.1} {:.1} size {:.2} cur {:.1} {:.1} {:.1} ex {:.3} ey {:.3} kind {:#x}", p.bank, p.tex_group, x, y, z, p.size, cur[0], cur[1], cur[2], a.ex, a.ey, k);
    }
    let (mut ax, mut ay, mut bx, mut by) = (x_extent, y_extent, x_extent, -y_extent);
    let mut angle = p.rotate;
    if k & (TRAIL | DIR_VEC) != 0 {
        let prj = &s.prj;
        let (vf1, vf2);
        if prj[0] == 0.0 {
            let w0 = m[2][3] + (m[2][2] * z + (m[2][0] * x + m[2][1] * y));
            let s808 = prj[1] * m[0][0] + prj[2] * m[2][0];
            let s804 = prj[1] * m[0][1] + prj[2] * m[2][1];
            let f16 = prj[1] * m[0][2] + prj[2] * m[2][2];
            let f20 = prj[1] * m[0][3] + prj[2] * m[2][3];
            let f12 = prj[3] * m[1][0] + prj[4] * m[2][0];
            let f8 = prj[3] * m[1][1] + prj[4] * m[2][1];
            let f11 = prj[3] * m[1][2] + prj[4] * m[2][2];
            let f13 = prj[3] * m[1][3] + prj[4] * m[2][3];
            if w0 == 0.0 {
                return;
            }
            let w0inv = -1.0 / w0;
            let w1 = m[2][3] + (m[2][2] * pz + (m[2][0] * px + m[2][1] * py));
            if w1 == 0.0 {
                return;
            }
            let w1inv = -1.0 / w1;
            vf1 = w0inv * (f20 + (f16 * z + (s808 * x + s804 * y))) - w1inv * (f20 + (f16 * pz + (s808 * px + s804 * py)));
            vf2 = w0inv * (f13 + (f11 * z + (f12 * x + f8 * y))) - w1inv * (f13 + (f11 * pz + (f12 * px + f8 * py)));
        } else {
            let s800 = prj[1] * m[0][0] + prj[2];
            let s7fc = prj[1] * m[0][1] + prj[2];
            let s7f8 = prj[1] * m[0][2] + prj[2];
            let f17 = prj[3] * m[1][0] + prj[4];
            let f18 = prj[3] * m[1][1] + prj[4];
            let f20b = prj[3] * m[1][2] + prj[4];
            vf1 = s7f8 * p.vel.z + (s800 * p.vel.x + s7fc * p.vel.y);
            vf2 = f20b * p.vel.z + (f17 * p.vel.x + f18 * p.vel.y);
        }
        angle = if vf2.abs() < f32::MIN_POSITIVE { if -vf1 >= 0.0 { 1.570_796_4 } else { -1.570_796_4 } } else { (-vf1).atan2(vf2) };
        if k & DIR_VEC != 0 {
            angle += p.rotate;
        }
    }
    if (angle.abs() as f64) > 0.01 {
        let (sn, cs) = angle.sin_cos();
        let o = ax;
        ax = cs * ax - sn * ay;
        ay = sn * o + cs * ay;
        let o = bx;
        bx = cs * bx - sn * by;
        by = sn * o + cs * by;
    }
    g.set_current_mtx(GX_PNMTX1);
    let tex = k & DISP_TEXTURE != 0;
    let base = ((k >> 16) & 0xC) as u8;
    if k & TRAIL != 0 {
        let col = if k & (DISP_LIGHTING | PRIM_ENV) == PRIM_ENV || k & (DISP_LIGHTING | PRIM_ENV) == (DISP_LIGHTING | PRIM_ENV) { WHITE } else { prim };
        vtx_desc(g, if tex { 2 } else { 3 });
        g.begin(GX_QUADS, if tex { 2 } else { 3 }, 4);
        let tail_a = (col.a as f32 * p.trail) as u8;
        let verts = [([-ax + prev[0], -ay + prev[1], prev[2]], tail_a), ([-bx + cur[0], -by + cur[1], cur[2]], col.a), ([ax + cur[0], ay + cur[1], cur[2]], col.a), ([bx + prev[0], by + prev[1], prev[2]], tail_a)];
        for (i, (v, al)) in verts.iter().enumerate() {
            g.gx.position3f32(v[0], v[1], v[2]);
            g.gx.color4u8(col.r, col.g, col.b, *al);
            if tex {
                g.gx.tex_coord1x8(base + i as u8);
            }
        }
        g.end();
        return;
    }
    vtx_desc(g, if tex { 0 } else { 1 });
    g.begin(GX_QUADS, if tex { 0 } else { 1 }, 4);
    let verts = [[-ax + cur[0], -ay + cur[1], cur[2]], [-bx + cur[0], -by + cur[1], cur[2]], [ax + cur[0], ay + cur[1], cur[2]], [bx + cur[0], by + cur[1], cur[2]]];
    for (i, v) in verts.iter().enumerate() {
        g.gx.position3f32(v[0], v[1], v[2]);
        if tex {
            g.gx.tex_coord1x8(base + i as u8);
        }
    }
    g.end();
}

fn point(g: &mut GxShim, p: &Particle, s: &mut State) {
    g.set_current_mtx(GX_PNMTX0);
    let w = if p.size > 42.5 { 255.0f32 } else { 6.0 * p.size } as u8;
    if s.point_size != w as i32 {
        s.point_size = w as i32;
        g.set_point_size(w, GX_TO_ONE);
    }
    let tex = p.kind & DISP_TEXTURE != 0;
    vtx_desc(g, if tex { 0 } else { 1 });
    g.begin(GX_POINTS, if tex { 0 } else { 1 }, 1);
    g.gx.position3f32(p.pos.x, p.pos.y, p.pos.z);
    if tex {
        g.gx.tex_coord1x8(1);
    }
    g.end();
}

/// `psDispSub` with no form table (the camera-facing quad) and `psDispSubMakePolygon`'s quad paths.
fn quad(g: &mut GxShim, p: &Particle, s: &mut State, prim: Rgba) {
    let k = p.kind;
    let (x, y, z) = (p.pos.x, p.pos.y, p.pos.z);
    let a = &s.axes;
    let mut right = [a[0] * p.size, a[2] * p.size, a[4] * p.size];
    let mut up = [a[1] * p.size, a[3] * p.size, a[5] * p.size];
    let mut angle = p.rotate;
    if k & (TRAIL | DIR_VEC) != 0 {
        let pv = &s.pvmtx;
        let (px, py, pz);
        let (vx, vy);
        if s.persp {
            if k & TORNADO != 0 {
                // calcTornadoLastPos needs the generator's aux data: approximate with the velocity.
                px = x - p.vel.x;
                py = y - p.vel.y;
                pz = z - p.vel.z;
            } else {
                px = x - p.vel.x;
                py = y - p.vel.y;
                pz = z - p.vel.z;
            }
            let v = &s.vmtx;
            let w0 = v[2][3] + (v[2][2] * z + (v[2][0] * x + v[2][1] * y));
            let w1 = v[2][3] + (v[2][2] * pz + (v[2][0] * px + v[2][1] * py));
            if w0 == 0.0 || w1 == 0.0 {
                return;
            }
            let (i0, i1) = (-1.0 / w0, -1.0 / w1);
            vx = i0 * (pv[0][3] + (pv[0][2] * z + (pv[0][0] * x + pv[0][1] * y))) - i1 * (pv[0][3] + (pv[0][2] * pz + (pv[0][0] * px + pv[0][1] * py)));
            vy = i0 * (pv[1][3] + (pv[1][2] * z + (pv[1][0] * x + pv[1][1] * y))) - i1 * (pv[1][3] + (pv[1][2] * pz + (pv[1][0] * px + pv[1][1] * py)));
        } else {
            vx = pv[0][2] * p.vel.z + (pv[0][0] * p.vel.x + pv[0][1] * p.vel.y);
            vy = pv[1][2] * p.vel.z + (pv[1][0] * p.vel.x + pv[1][1] * p.vel.y);
        }
        angle = if vy.abs() < f32::MIN_POSITIVE { if vx >= 0.0 { 1.570_796_4 } else { -1.570_796_4 } } else { melee_runtime::lbtrigf::atan2f(vx, vy) };
        if k & DIR_VEC != 0 {
            angle += p.rotate;
        }
    }
    if (angle.abs() as f64) > 0.01 {
        let axis = Vec3 { x: right[1] * up[2] - right[2] * up[1], y: right[2] * up[0] - right[0] * up[2], z: right[0] * up[1] - right[1] * up[0] };
        let m = mtx::rot_axis_rad(&axis, angle);
        right = mtx_mul_vec(&m, right);
        up = mtx_mul_vec(&m, up);
    }
    g.set_current_mtx(GX_PNMTX0);
    let tex = k & DISP_TEXTURE != 0;
    let base = ((k >> 16) & 0xC) as u8;
    if k & TRAIL != 0 {
        let (px, py, pz) = (x - p.vel.x, y - p.vel.y, z - p.vel.z);
        let col = if k & (DISP_LIGHTING | PRIM_ENV) == PRIM_ENV || k & (DISP_LIGHTING | PRIM_ENV) == (DISP_LIGHTING | PRIM_ENV) { WHITE } else { prim };
        vtx_desc(g, if tex { 2 } else { 3 });
        g.begin(GX_QUADS, if tex { 2 } else { 3 }, 4);
        let tail_a = (col.a as f32 * p.trail) as u8;
        let verts = [
            ([px - right[0], py - right[1], pz - right[2]], tail_a),
            ([x - up[0], y - up[1], z - up[2]], col.a),
            ([x + right[0], y + right[1], z + right[2]], col.a),
            ([px + up[0], py + up[1], pz + up[2]], tail_a),
        ];
        for (i, (v, al)) in verts.iter().enumerate() {
            g.gx.position3f32(v[0], v[1], v[2]);
            g.gx.color4u8(col.r, col.g, col.b, *al);
            if tex {
                g.gx.tex_coord1x8(base + i as u8);
            }
        }
        g.end();
        return;
    }
    vtx_desc(g, if tex { 0 } else { 1 });
    g.begin(GX_QUADS, if tex { 0 } else { 1 }, 4);
    let verts = [[x - right[0], y - right[1], z - right[2]], [x - up[0], y - up[1], z - up[2]], [x + right[0], y + right[1], z + right[2]], [x + up[0], y + up[1], z + up[2]]];
    for (i, v) in verts.iter().enumerate() {
        g.gx.position3f32(v[0], v[1], v[2]);
        if tex {
            g.gx.tex_coord1x8(base + i as u8);
        }
    }
    g.end();
}
