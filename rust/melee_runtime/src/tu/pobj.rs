//! `src/sysdolphin/baselib/pobj.c`, the display half: `HSD_PObjDisp`, `PObjSetupMtx`
//! (`SetupRigidModelMtx` / `SetupSharedVtxModelMtx` / `SetupEnvelopeModelMtx`), `setupArrayDesc`,
//! `setupVtxDesc`, `HSD_ClearVtxDesc`, `PObjDispSimplePrimitive`. (The logic half is the verified
//! port in `crate::pobj`.)
//!
//! Descriptor-driven like the rest of the draw path: `pobj` is the `HSD_PObjDesc` offset (+0x4
//! next, +0x8 verts, +0xC flags u16, +0xE n_display u16, +0x10 display, +0x14 jobj / shape set /
//! envelope list). Envelope joints are resolved through [`DispModel::by_desc`].
//!
//! Not ported: shape-animated PObjs (`PObjDispShapeAnim` / `drawShapeAnim`: counted in
//! `stats.shape_anim_skipped` and not drawn), the PC envelope cache / geno palette paths (both
//! produce the same matrices), perf counters.
use super::displayfunc::{DispModel, JOBJ_LIGHTING};
use super::state::gx::*;
use super::state::{GxBackend, HsdRender};
use super::tobj::{TEX_COORD_HILIGHT, TEX_COORD_REFLECTION};
use crate::hsd_mtx;
use crate::mtx::{self, Mtx};

pub const POBJ_SKIN: u32 = 0;
pub const POBJ_SHAPEANIM: u32 = 1 << 12;
pub const POBJ_ENVELOPE: u32 = 1 << 13;
pub const POBJ_TYPE_MASK: u32 = 0x3000;
pub const POBJ_CULLFRONT: u32 = 1 << 14;
pub const POBJ_CULLBACK: u32 = 1 << 15;

pub const HSD_MTX_RIGID: u32 = 0;
pub const HSD_MTX_ENVELOPE: u32 = 1;

const SETUP_NORMAL: u32 = 1;
const SETUP_REFLECTION: u32 = 2;
const SETUP_HIGHLIGHT: u32 = 4;
const SETUP_NORMAL_PROJECTION: u32 = SETUP_REFLECTION | SETUP_HIGHLIGHT;
const SETUP_JOINT0: u32 = 8;
const SETUP_JOINT1: u32 = 16;

const GX_VA_NRM: u32 = 10;
const GX_VA_NBT: u32 = 25;
const GX_VA_CLR0: u32 = 11;
const GX_VA_CLR1: u32 = 12;

/// One `HSD_VtxDescList` entry (0x18 bytes).
#[derive(Clone, Copy, Debug)]
pub struct VtxDesc {
    pub attr: u32,
    pub attr_type: u32,
    pub comp_cnt: u32,
    pub comp_type: u32,
    pub frac: u8,
    pub stride: u16,
    pub vertex: Option<u32>,
}

pub fn vtx_descs(m: &DispModel, verts: u32) -> Vec<VtxDesc> {
    let d = &m.dat;
    let mut out = Vec::new();
    let mut v = verts;
    while out.len() < 27 {
        let attr = d.u32_(v);
        if attr == GX_VA_NULL {
            break;
        }
        out.push(VtxDesc {
            attr,
            attr_type: d.u32_(v + 4),
            comp_cnt: d.u32_(v + 8),
            comp_type: d.u32_(v + 0xC),
            frac: d.u8_(v + 0x10),
            stride: d.u16_(v + 0x12),
            // An indexed attribute always has an array; offset 0 is valid there (the vertex
            // buffers usually open the DAT data section), so it is not read as NULL.
            vertex: Some(d.u32_(v + 0x14)).filter(|&p| (p as usize) < d.0.len()),
        });
        v += 0x18;
    }
    out
}

/// Bytes one attribute takes per vertex in a display list.
fn attr_size(a: &VtxDesc) -> usize {
    match a.attr_type {
        GX_INDEX8 => 1,
        GX_INDEX16 => 2,
        GX_DIRECT => {
            if a.attr <= GX_VA_TEX7MTXIDX {
                return 1;
            }
            let comp = match a.comp_type {
                0 | 1 => 1,
                2 | 3 => 2,
                _ => 4,
            };
            match a.attr {
                GX_VA_POS => comp * if a.comp_cnt == 0 { 2 } else { 3 },
                GX_VA_NRM | GX_VA_NBT => comp * if a.comp_cnt == 0 { 3 } else { 9 },
                GX_VA_CLR0 | GX_VA_CLR1 => match a.comp_type {
                    0 | 3 => 2,
                    1 | 4 => 3,
                    _ => 4,
                },
                _ => comp * if a.comp_cnt == 0 { 1 } else { 2 },
            }
        }
        _ => 0,
    }
}

/// The largest index each indexed attribute of the display list uses (to size the arrays the
/// backend snapshots). A malformed list stops the scan where it breaks.
pub fn scan_display_list(dl: &[u8], attrs: &[VtxDesc]) -> Vec<(u32, u32)> {
    let mut max: Vec<(u32, u32)> = Vec::new();
    let mut i = 0;
    'outer: while i < dl.len() {
        let op = dl[i];
        if op == 0 {
            i += 1;
            continue;
        }
        if op & 0x80 == 0 || i + 3 > dl.len() {
            break;
        }
        let n = u16::from_be_bytes([dl[i + 1], dl[i + 2]]) as usize;
        i += 3;
        for _ in 0..n {
            for a in attrs {
                let s = attr_size(a);
                if i + s > dl.len() {
                    break 'outer;
                }
                let idx = match a.attr_type {
                    GX_INDEX8 => Some(dl[i] as u32),
                    GX_INDEX16 => Some(u16::from_be_bytes([dl[i], dl[i + 1]]) as u32),
                    _ => None,
                };
                if let Some(x) = idx {
                    match max.iter_mut().find(|e| e.0 == a.attr) {
                        Some(e) => e.1 = e.1.max(x),
                        None => max.push((a.attr, x)),
                    }
                }
                i += s;
            }
        }
    }
    max
}

/// `HSD_Index2PosNrmMtx`.
fn index_to_pos_nrm_mtx(i: u32) -> u32 {
    GX_PNMTX0 + 3 * i
}
/// `HSD_Index2TexMtx`.
fn index_to_tex_mtx(i: u32) -> u32 {
    GX_TEXMTX0 + 3 * i
}

impl HsdRender {
    fn model_key(m: &DispModel) -> usize {
        m.dat.0.as_ptr() as usize
    }

    /// `HSD_ClearVtxDesc`.
    pub fn clear_vtx_desc<G: GxBackend>(&mut self, g: &mut G) {
        g.clear_vtx_desc();
        self.prev_array = None;
        self.prev_vtxdesc = None;
    }

    /// `GetSetupFlags` (the JObj is the one being drawn).
    fn get_setup_flags(&self, m: &DispModel, j: usize, rendermode: u32) -> u32 {
        let mut flags = 0;
        if rendermode & 0x400_0000 == 0 {
            if m.joints[j].flags & JOBJ_LIGHTING != 0 {
                flags |= SETUP_NORMAL;
            }
            if self.tobj_kinds & (1 << TEX_COORD_REFLECTION) != 0 {
                flags |= SETUP_NORMAL | SETUP_REFLECTION;
            }
            if self.tobj_kinds & (1 << TEX_COORD_HILIGHT) != 0 {
                flags |= SETUP_NORMAL | SETUP_HIGHLIGHT;
            }
        }
        flags
    }

    fn load_normal<G: GxBackend>(g: &mut G, lighting: bool, flags: u32, pm: &Mtx, pn: u32, tex: u32) {
        if flags & SETUP_NORMAL != 0 {
            let n = hsd_mtx::inverse_transpose(pm);
            if lighting {
                g.load_nrm_mtx_imm(&n, pn);
            }
            if flags & SETUP_NORMAL_PROJECTION != 0 {
                g.load_tex_mtx_imm(&n, tex, GX_MTX3x4);
            }
        }
    }

    /// `SetupRigidModelMtx`.
    fn setup_rigid_model_mtx<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, pmtx: &Mtx, rendermode: u32) {
        if self.mtx_mark[0] == (Some(j), HSD_MTX_RIGID) {
            return;
        }
        self.mtx_mark[0] = (Some(j), HSD_MTX_RIGID);
        g.set_current_mtx(GX_PNMTX0);
        g.load_pos_mtx_imm(pmtx, GX_PNMTX0);
        let flags = self.get_setup_flags(m, j, rendermode);
        Self::load_normal(g, m.joints[j].flags & JOBJ_LIGHTING != 0, flags, pmtx, GX_PNMTX0, GX_TEXMTX0);
    }

    /// `SetupSharedVtxModelMtx` (`pobj->u.jobj` = joint `other`).
    fn setup_shared_vtx_model_mtx<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, other: usize, vmtx: &Mtx, pmtx: &Mtx, rendermode: u32) {
        let mut flags = 0;
        // pobj.c: `obj != jobj && mark != RIGID` (sic: both conditions, as the C has them).
        if self.mtx_mark[0].0 != Some(j) && self.mtx_mark[0].1 != HSD_MTX_RIGID {
            flags |= SETUP_JOINT0;
        }
        if self.mtx_mark[1].0 != Some(other) && self.mtx_mark[1].1 != HSD_MTX_RIGID {
            flags |= SETUP_JOINT1;
        }
        if flags == 0 {
            return;
        }
        flags |= self.get_setup_flags(m, j, rendermode);
        let lighting = m.joints[j].flags & 0x80 != 0;
        g.set_current_mtx(GX_PNMTX0);
        g.load_pos_mtx_imm(pmtx, GX_PNMTX0);
        Self::load_normal(g, lighting, flags, pmtx, GX_PNMTX0, GX_TEXMTX0);
        let mm = mtx::concat(vmtx, &m.joints[other].mtx);
        g.load_pos_mtx_imm(&mm, GX_PNMTX1);
        Self::load_normal(g, lighting, flags, &mm, GX_PNMTX1, GX_TEXMTX1);
    }

    /// `SetupEnvelopeModelMtx`.
    fn setup_envelope_model_mtx<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, list: Option<u32>, vmtx: &Mtx, rendermode: u32) {
        // HSD_PObjClearMtxMark(NULL, HSD_MTX_ENVELOPE)
        self.mtx_mark = [(None, HSD_MTX_ENVELOPE); 2];
        let flags = self.get_setup_flags(m, j, rendermode);
        let right = m.mk_envelope_model_node_mtx(j);
        let d = &m.dat;
        let Some(mut lp) = list else { return };
        let ident = mtx::identity();
        let mut idx = 0u32;
        while idx < 10 {
            let Some(edesc0) = d.ptr(lp) else { break };
            // The envelope's (joint, weight) terms.
            let mut terms = Vec::new();
            let mut e = edesc0;
            while let Some(jd) = d.ptr(e) {
                terms.push((jd, d.f32_(e + 4)));
                e += 8;
                if terms.len() > 64 {
                    break;
                }
            }
            let joint = |jd: u32| m.by_desc(jd);
            let mut mtxv: Mtx;
            if terms.first().is_some_and(|t| t.1 >= 1.0 - f32::EPSILON) {
                let Some(ji) = joint(terms[0].0) else {
                    lp += 4;
                    idx += 1;
                    continue;
                };
                mtxv = if right.is_some() {
                    mtx::concat(&m.joints[ji].mtx, &m.envelope_mtx(ji).unwrap_or(ident))
                } else {
                    m.joints[ji].mtx
                };
            } else {
                mtxv = [[0.0; 4]; 3];
                for &(jd, w) in &terms {
                    let Some(ji) = joint(jd) else { continue };
                    let tmp = mtx::concat(&m.joints[ji].mtx, &m.envelope_mtx(ji).unwrap_or(ident));
                    mtxv = hsd_mtx::scaled_add(&tmp, &mtxv, w);
                }
            }
            if let Some(r) = &right {
                mtxv = mtx::concat(&mtxv, r);
            }
            let tmp = mtx::concat(vmtx, &mtxv);
            let mtx_no = index_to_pos_nrm_mtx(idx);
            g.load_pos_mtx_imm(&tmp, mtx_no);
            Self::load_normal(g, m.joints[j].flags & JOBJ_LIGHTING != 0, flags, &tmp, mtx_no, index_to_tex_mtx(idx));
            lp += 4;
            idx += 1;
        }
    }

    /// `setupArrayDesc` + `setupVtxDesc` + `GXCallDisplayList` (`PObjDispSimplePrimitive`).
    fn pobj_disp_simple_primitive<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, pobj: u32) {
        let d = &m.dat;
        let Some(verts) = d.ptr(pobj + 8) else { return };
        let Some(disp) = d.ptr(pobj + 0x10) else { return };
        let n_display = d.u16_(pobj + 0xE) as usize;
        let dl_full = d.tail(disp);
        let dl = &dl_full[..dl_full.len().min(n_display << 5)];
        let descs = vtx_descs(m, verts);
        let key = (Self::model_key(m), verts);
        let extents = self.extents.entry((key.0, pobj)).or_insert_with(|| scan_display_list(dl, &descs)).clone();
        // setupArrayDesc: the C caches by desc list; the array lengths here depend on the
        // display list too, so a list shared by two PObjs is re-bound per PObj.
        for a in &descs {
            if a.attr_type != GX_DIRECT {
                if let Some(base) = a.vertex {
                    let n = extents.iter().find(|e| e.0 == a.attr).map_or(0, |e| e.1) + 1;
                    let len = (n as usize * a.stride.max(1) as usize + 16).min(d.0.len() - base as usize);
                    g.set_array(a.attr, &d.0[base as usize..base as usize + len], a.stride as u8);
                }
            }
        }
        self.prev_array = Some(key);
        if self.prev_vtxdesc != Some(key) {
            g.clear_vtx_desc();
            for a in &descs {
                g.set_vtx_desc(a.attr, a.attr_type);
                if a.attr > GX_VA_TEX7MTXIDX {
                    g.set_vtx_attr_fmt(GX_VTXFMT0, a.attr, a.comp_cnt, a.comp_type, a.frac);
                }
            }
            self.prev_vtxdesc = Some(key);
        }
        g.call_display_list(dl);
    }

    /// `HSD_PObjDisp` for the PObj descriptor `pobj` of joint `j`.
    pub fn pobj_disp<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, pobj: u32, vmtx: &Mtx, pmtx: &Mtx, rendermode: u32) {
        let flags = m.dat.u16_(pobj + 0xC) as u32;
        match flags & (POBJ_CULLFRONT | POBJ_CULLBACK) {
            0 => self.state_set_cull_mode(g, GX_CULL_NONE),
            POBJ_CULLFRONT => self.state_set_cull_mode(g, GX_CULL_FRONT),
            POBJ_CULLBACK => self.state_set_cull_mode(g, GX_CULL_BACK),
            _ => return,
        }
        let u = m.dat.ptr(pobj + 0x14);
        match flags & POBJ_TYPE_MASK {
            POBJ_SHAPEANIM => {
                self.stats.shape_anim_skipped += 1;
                return;
            }
            POBJ_ENVELOPE => {
                self.stats.envelope_pobjs += 1;
                self.setup_envelope_model_mtx(g, m, j, u, vmtx, rendermode);
            }
            _ => match u.and_then(|jd| m.by_desc(jd)) {
                None => self.setup_rigid_model_mtx(g, m, j, pmtx, rendermode),
                Some(other) => self.setup_shared_vtx_model_mtx(g, m, j, other, vmtx, pmtx, rendermode),
            },
        }
        self.stats.pobjs += 1;
        self.pobj_disp_simple_primitive(g, m, pobj);
    }
}
