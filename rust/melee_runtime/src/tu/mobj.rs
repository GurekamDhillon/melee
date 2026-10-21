//! `src/sysdolphin/baselib/mobj.c`, the display half: `HSD_MObjSetup` / `HSD_MObjUnset`, and
//! `MObjMakeTExp` + tobj.c's `TObjMakeTExp` / `MakeColorGenTExp` with the TEV built directly.
//!
//! APPROXIMATION (TODO texp.c): HSD compiles the TExp graph with `HSD_TExpCompile` (texp.c, not
//! ported), which folds constants into TEV registers and merges expressions into as few stages
//! as possible. Here every TExp TEV node becomes one GX stage in order, writing PREV (both colour
//! and alpha; an unset half passes PREV through), constants go to KONST colour registers, and a
//! TObj colour-generator node writes REG0. The arithmetic per pixel is the expression's; the
//! stage layout (and so 8-bit rounding between stages) can differ from the compiled TEV. The
//! specular-lightmap chain is reduced to one `PREV + K(specular) * RAS1` stage, toon and shadow
//! TObjs are not appended.
use super::state::gx::*;
use super::state::{Dat, GxBackend, HsdRender, Rgba};
use super::tobj::{self as tt, TObjRt};

pub const RENDER_VERTEX: u32 = 1 << 1;
pub const RENDER_DIFFUSE: u32 = 1 << 2;
pub const RENDER_SPECULAR: u32 = 1 << 3;
pub const RENDER_TOON: u32 = 1 << 12;
pub const RENDER_SHADOW: u32 = 1 << 26;
pub const RENDER_XLU: u32 = 1 << 30;

const TEX_COLORMAP_MASK: u32 = 0xF << 16;
const TEX_ALPHAMAP_MASK: u32 = 0xF << 20;
const TOBJ_TEVREG_ACTIVE_COLOR_TEV: u32 = 1 << 30;
const TOBJ_TEVREG_ACTIVE_ALPHA_TEV: u32 = 1 << 31;

/// A TEV input (`HSD_TEInput` + its expression, resolved to a GX source).
#[derive(Clone, Copy, Debug, PartialEq)]
enum In {
    Zero,
    One,
    Half,
    PrevC,
    PrevA,
    Reg0C,
    Reg0A,
    RasC,
    RasA,
    TexC,
    TexA,
    /// A constant: RGB triple (HSD_TE_RGB) or one value replicated (HSD_TE_X).
    K([u8; 3]),
}

#[derive(Clone, Copy, Debug)]
struct Half {
    ins: [In; 4],
    op: u32,
    bias: u32,
    scale: u32,
    clamp: bool,
}

const PASS_C: Half = Half { ins: [In::Zero, In::Zero, In::Zero, In::PrevC], op: GX_TEV_ADD, bias: 0, scale: 0, clamp: true };
const PASS_A: Half = Half { ins: [In::Zero, In::Zero, In::Zero, In::PrevA], op: GX_TEV_ADD, bias: 0, scale: 0, clamp: true };

fn add(ins: [In; 4]) -> Half {
    Half { ins, op: GX_TEV_ADD, bias: GX_TB_ZERO, scale: GX_CS_SCALE_1, clamp: true }
}

#[derive(Clone, Copy, Debug)]
struct Stage {
    /// (texcoord, texmap) or none.
    tex: Option<(u32, u32)>,
    chan: u32,
    c: Half,
    a: Half,
    out: u32,
}

/// The stage list `MObjMakeTExp` describes.
struct Tev {
    stages: Vec<Stage>,
}

impl Tev {
    fn push(&mut self, tex: Option<(u32, u32)>, chan: u32, c: Half, a: Half) {
        self.stages.push(Stage { tex, chan, c, a, out: GX_TEVPREV });
    }
}

fn k_of(c: Rgba) -> In {
    In::K([c[0], c[1], c[2]])
}
fn kx(v: u8) -> In {
    In::K([v, v, v])
}
/// `HSD_TE_F32` constants are scaled to 0..255 (texp.c's `HSD_TExpSetReg`: `(u8)(255 * x)`).
fn kf(v: f32) -> In {
    kx((255.0 * v.clamp(0.0, 1.0)) as u8)
}

/// tobj.c `MakeColorGenTExp`: the TObj's own TEV (`HSD_TObjTevDesc`) as one stage into REG0.
/// Returns (colour source, alpha source) for the TObj's main stage.
fn color_gen(d: &Dat, tev: u32, o: &TObjRt, out: &mut Tev) -> (In, In) {
    let active = d.u32_(tev + 0x1C);
    let konst = d.color(tev + 0x10);
    let tev0 = d.color(tev + 0x14);
    let tev1 = d.color(tev + 0x18);
    let cin = |v: u8| -> In {
        match v as u32 {
            15 => In::Zero, // GX_CC_ZERO
            12 => In::One,
            13 => In::Half,
            8 => In::TexC,
            9 => In::TexA,
            0x80 => k_of(konst),
            0x81 => kx(konst[0]),
            0x82 => kx(konst[1]),
            0x83 => kx(konst[2]),
            0x84 => kx(konst[3]),
            0x85 => k_of(tev0),
            0x86 => kx(tev0[3]),
            0x87 => k_of(tev1),
            0x88 => kx(tev1[3]),
            _ => In::Zero, // tobj.c:0x4A7 asserts
        }
    };
    let ain = |v: u8| -> In {
        match v as u32 {
            7 => In::Zero, // GX_CA_ZERO
            4 => In::TexA,
            0x40 => kx(konst[0]),
            0x41 => kx(konst[1]),
            0x42 => kx(konst[2]),
            0x43 => kx(konst[3]),
            0x44 => kx(tev0[3]),
            0x45 => kx(tev1[3]),
            _ => In::Zero, // tobj.c:0x4F0 asserts
        }
    };
    let half = |base: u32, ins: [In; 4]| Half {
        ins,
        op: d.u8_(tev + base) as u32,
        bias: d.u8_(tev + base + 2) as u32,
        scale: d.u8_(tev + base + 4) as u32,
        clamp: d.u8_(tev + base + 6) != 0,
    };
    let mut c = Half { ins: [In::Zero, In::Zero, In::Zero, In::TexC], ..PASS_C };
    let mut a = Half { ins: [In::Zero, In::Zero, In::Zero, In::TexA], ..PASS_A };
    let (mut cs, mut asrc) = (In::TexC, In::TexA);
    if active & TOBJ_TEVREG_ACTIVE_COLOR_TEV != 0 {
        c = half(0, [cin(d.u8_(tev + 8)), cin(d.u8_(tev + 9)), cin(d.u8_(tev + 0xA)), cin(d.u8_(tev + 0xB))]);
        cs = In::Reg0C;
    }
    if active & TOBJ_TEVREG_ACTIVE_ALPHA_TEV != 0 {
        a = half(1, [ain(d.u8_(tev + 0xC)), ain(d.u8_(tev + 0xD)), ain(d.u8_(tev + 0xE)), ain(d.u8_(tev + 0xF))]);
        asrc = In::Reg0A;
    }
    if cs == In::TexC && asrc == In::TexA {
        return (cs, asrc);
    }
    out.stages.push(Stage { tex: Some((o.coord, o.id)), chan: GX_COLOR_NULL, c, a, out: GX_TEVREG0 });
    (if cs == In::Reg0C { In::Reg0C } else { In::TexC }, if asrc == In::Reg0A { In::Reg0A } else { In::TexA })
}

/// tobj.c `TObjMakeTExp`: the TObj's blend into the running colour/alpha (PREV).
fn tobj_make_texp(d: &Dat, o: &TObjRt, lightmap_done: u32, out: &mut Tev) {
    let repeat = lightmap_done & o.lightmap() != 0;
    let (mut c_src, mut a_src) = (In::TexC, In::TexA);
    if let Some(tev) = o.tev {
        if d.u32_(tev + 0x1C) & (TOBJ_TEVREG_ACTIVE_COLOR_TEV | TOBJ_TEVREG_ACTIVE_ALPHA_TEV) != 0 {
            (c_src, a_src) = color_gen(d, tev, o, out);
        }
    }
    // a_sel / a_src as a colour input (HSD_TE_A of the texture or REG0).
    let a_as_c = if a_src == In::Reg0A { In::Reg0A } else { In::TexA };
    let flags = o.flags();
    let c = match flags & TEX_COLORMAP_MASK {
        0x1_0000 => add([In::PrevC, c_src, a_as_c, In::Zero]),           // ALPHA_MASK
        0x2_0000 => add([In::PrevC, c_src, c_src, In::Zero]),            // RGB_MASK
        0x3_0000 => add([In::PrevC, c_src, kf(o.obj.blending), In::Zero]), // BLEND
        0x4_0000 => add([In::Zero, In::PrevC, c_src, In::Zero]),         // MODULATE
        0x5_0000 => add([In::Zero, In::Zero, In::Zero, c_src]),          // REPLACE
        0x7_0000 => add([c_src, In::Zero, In::Zero, In::PrevC]),         // ADD
        0x8_0000 => Half { op: GX_TEV_SUB, ..add([c_src, In::Zero, In::Zero, In::PrevC]) }, // SUB
        _ => PASS_C,                                                     // NONE / PASS
    };
    let a = if repeat {
        PASS_A
    } else {
        match flags & TEX_ALPHAMAP_MASK {
            0x10_0000 => add([In::PrevA, a_src, a_src, In::Zero]),           // ALPHA_MASK
            0x20_0000 => add([In::PrevA, a_src, kf(o.obj.blending), In::Zero]), // BLEND
            0x30_0000 => add([In::Zero, In::PrevA, a_src, In::Zero]),        // MODULATE
            0x40_0000 => add([In::Zero, In::Zero, In::Zero, a_src]),         // REPLACE
            0x60_0000 => add([a_src, In::Zero, In::Zero, In::PrevA]),        // ADD
            0x70_0000 => Half { op: GX_TEV_SUB, ..add([a_src, In::Zero, In::Zero, In::PrevA]) },
            _ => PASS_A,
        }
    };
    out.push(Some((o.coord, o.id)), GX_COLOR_NULL, c, a);
}

/// `MObjMakeTExp` as a stage list (see the module note).
fn make_texp(d: &Dat, rendermode: u32, mat: &MatDesc, tobjs: &[TObjRt]) -> Tev {
    let mut t = Tev { stages: Vec::new() };
    if rendermode & RENDER_VERTEX != 0 {
        t.push(None, GX_COLOR0A0, add([In::Zero, In::Zero, In::Zero, In::RasC]), add([In::Zero, In::Zero, In::Zero, In::RasA]));
    } else {
        t.push(None, GX_COLOR_NULL, add([In::Zero, In::Zero, In::Zero, k_of(mat.diffuse)]), add([In::Zero, In::Zero, In::Zero, kf(mat.alpha)]));
    }
    let mut done = 0u32;
    for o in tobjs {
        if o.lightmap() & (tt::TEX_LIGHTMAP_DIFFUSE | tt::TEX_LIGHTMAP_AMBIENT) != 0 && o.id != GX_TEXMAP_NULL {
            tobj_make_texp(d, o, done, &mut t);
        }
    }
    done |= tt::TEX_LIGHTMAP_DIFFUSE | tt::TEX_LIGHTMAP_AMBIENT;
    if rendermode & RENDER_DIFFUSE != 0 {
        t.push(None, GX_COLOR0A0, add([In::Zero, In::PrevC, In::RasC, In::Zero]), add([In::Zero, In::PrevA, In::RasA, In::Zero]));
    }
    if rendermode & RENDER_SPECULAR != 0 {
        t.push(None, GX_COLOR1A1, add([In::Zero, k_of(mat.specular), In::RasC, In::PrevC]), PASS_A);
        done |= tt::TEX_LIGHTMAP_SPECULAR;
    }
    for o in tobjs {
        if o.lightmap() & tt::TEX_LIGHTMAP_EXT != 0 && o.id != GX_TEXMAP_NULL {
            tobj_make_texp(d, o, done, &mut t);
        }
    }
    t
}

/// The `HSD_Material` + render mode + PE of an `HSD_MObjDesc`.
#[derive(Clone, Copy, Debug)]
pub struct MatDesc {
    pub ambient: Rgba,
    pub diffuse: Rgba,
    pub specular: Rgba,
    pub alpha: f32,
    pub shininess: f32,
}

/// `MObjLoad`: the rendermode (|= RENDER_TOON), material and PE of a descriptor.
pub fn mobj_load(d: &Dat, mobj: u32) -> (u32, MatDesc, Option<[u8; 12]>) {
    let rm = d.u32_(mobj + 4) | RENDER_TOON;
    let mat = match d.ptr(mobj + 0xC) {
        Some(m) => MatDesc { ambient: d.color(m), diffuse: d.color(m + 4), specular: d.color(m + 8), alpha: d.f32_(m + 0xC), shininess: d.f32_(m + 0x10) },
        None => MatDesc { ambient: [0x80, 0x80, 0x80, 0xFF], diffuse: [0xFF; 4], specular: [0xFF; 4], alpha: 1.0, shininess: 50.0 },
    };
    let pe = d.ptr(mobj + 0x14).map(|p| {
        let mut b = [0u8; 12];
        for (i, v) in b.iter_mut().enumerate() {
            *v = d.u8_(p + i as u32);
        }
        b
    });
    (rm, mat, pe)
}

fn cin(i: In) -> (u32, Option<[u8; 3]>) {
    match i {
        In::Zero => (GX_CC_ZERO, None),
        In::One => (GX_CC_ONE, None),
        In::Half => (GX_CC_HALF, None),
        In::PrevC => (GX_CC_CPREV, None),
        In::PrevA => (GX_CC_APREV, None),
        In::Reg0C => (GX_CC_C0, None),
        In::Reg0A => (GX_CC_A0, None),
        In::RasC => (GX_CC_RASC, None),
        In::RasA => (GX_CC_RASA, None),
        In::TexC => (GX_CC_TEXC, None),
        In::TexA => (GX_CC_TEXA, None),
        In::K(k) => (GX_CC_KONST, Some(k)),
    }
}

fn ain(i: In) -> (u32, Option<u8>) {
    match i {
        In::Zero => (GX_CA_ZERO, None),
        In::One => (GX_CA_KONST, Some(255)),
        In::Half => (GX_CA_KONST, Some(128)),
        In::PrevA | In::PrevC => (GX_CA_APREV, None),
        In::Reg0A | In::Reg0C => (GX_CA_A0, None),
        In::RasA | In::RasC => (GX_CA_RASA, None),
        In::TexA | In::TexC => (GX_CA_TEXA, None),
        In::K(k) => (GX_CA_KONST, Some(k[0])),
    }
}

impl HsdRender {
    /// Emit the stage list: TevOrder / ColorIn / AlphaIn / ops, KONST registers per stage.
    fn setup_tev<G: GxBackend>(&mut self, g: &mut G, t: &Tev) {
        let mut kregs: Vec<Rgba> = Vec::new();
        for st in &t.stages {
            if self.current_tev >= 16 {
                break;
            }
            let stage = self.state_assign_tev();
            let (coord, map) = st.tex.unwrap_or((GX_TEXCOORD_NULL, GX_TEXMAP_NULL));
            g.set_tev_order(stage, coord, map, st.chan);
            let c: Vec<(u32, Option<[u8; 3]>)> = st.c.ins.iter().map(|&i| cin(i)).collect();
            let a: Vec<(u32, Option<u8>)> = st.a.ins.iter().map(|&i| ain(i)).collect();
            let kc = c.iter().find_map(|x| x.1);
            let ka = a.iter().find_map(|x| x.1);
            if kc.is_some() || ka.is_some() {
                let want: Rgba = {
                    let k = kc.unwrap_or([0; 3]);
                    [k[0], k[1], k[2], ka.unwrap_or(0)]
                };
                let slot = match kregs.iter().position(|r| (kc.is_none() || r[..3] == want[..3]) && (ka.is_none() || r[3] == want[3])) {
                    Some(s) => s,
                    None if kregs.len() < 4 => {
                        kregs.push(want);
                        g.set_tev_kcolor(kregs.len() as u32 - 1, want);
                        kregs.len() - 1
                    }
                    None => {
                        // Out of KONST registers: reuse the last (approximation).
                        kregs[3] = want;
                        g.set_tev_kcolor(3, want);
                        3
                    }
                };
                g.set_tev_kcolor_sel(stage, GX_TEV_KCSEL_K0 + slot as u32);
                g.set_tev_kalpha_sel(stage, GX_TEV_KASEL_K0_A + slot as u32);
            } else {
                g.set_tev_kcolor_sel(stage, GX_TEV_KCSEL_1);
                g.set_tev_kalpha_sel(stage, GX_TEV_KASEL_1);
            }
            g.set_tev_color_in(stage, c[0].0, c[1].0, c[2].0, c[3].0);
            g.set_tev_alpha_in(stage, a[0].0, a[1].0, a[2].0, a[3].0);
            g.set_tev_color_op(stage, st.c.op, st.c.bias, st.c.scale, st.c.clamp, st.out);
            g.set_tev_alpha_op(stage, st.a.op, st.a.bias, st.a.scale, st.a.clamp, st.out);
            g.set_tev_swap_mode(stage, GX_TEV_SWAP0, GX_TEV_SWAP0);
        }
    }

    /// `HSD_MObjSetup` for the MObj descriptor `mobj`. Returns the TObjs (for `HSD_MObjUnset`).
    pub fn mobj_setup<G: GxBackend>(&mut self, g: &mut G, d: &Dat, mobj: u32) {
        self.state_init_tev();
        let (rendermode, mat, pe) = mobj_load(d, mobj);
        self.set_material_color(mat.ambient, mat.diffuse, mat.specular, mat.alpha);
        if rendermode & RENDER_SPECULAR != 0 {
            self.set_material_shininess(mat.shininess);
        }
        let mut tobjs = tt::load_desc_list(d, d.ptr(mobj + 8));
        self.tobj_kinds = tobjs.iter().fold(0, |k, o| k | 1 << o.coord_kind());
        self.tobj_setup(g, d, &mut tobjs);
        self.tobj_setup_texture_coord_gen(g, &tobjs);
        // setup_tev (MObjSetupTev): the compiled TEV, then HSD_TObjSetupVolatileTev (shadow /
        // bump only: not ported).
        let tev = make_texp(d, rendermode, &mat, &tobjs);
        self.setup_tev(g, &tev);
        self.setup_render_mode_with_custom_pe(g, rendermode, pe);
    }

    /// `HSD_MObjUnset`: `HSD_TObjSetup(NULL)` (only resets tobj_head in the C).
    pub fn mobj_unset<G: GxBackend>(&mut self, _g: &mut G) {}
}
