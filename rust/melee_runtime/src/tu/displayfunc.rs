//! `src/sysdolphin/baselib/displayfunc.c`: `HSD_JObjDisp` / `HSD_JObjDispDObj` /
//! `HSD_JObjDispSub`, `HSD_JObjMakePositionMtx`, `HSD_JObjFindSkeleton`,
//! `_HSD_mkEnvelopeModelNodeMtx`, over a [`DispModel`]: a joint tree's descriptors (DAT) plus the
//! world matrices and runtime flags the JObj system computed for it this frame.
//!
//! Not wired into the descriptor-driven draw path: the z-sorted list (`zsort_listing` is off in
//! Melee's main scene; the XLU / TEXEDGE passes run in tree order; the list itself is
//! [`ZListState`]), particle joints' `sptcl_callback` ([`DispStatics::disp_ptcl`]), specular init.
use super::state::gx::*;
use super::state::{Dat, GxBackend, HsdRender, Rgba};
use crate::gekko::sqrtf;
use crate::robj::{mtx_col_mag, mtx_col_vec};
use melee_core::fp::fmadds;
use melee_core::vec::Vec3;
use crate::mtx::{self, Mtx};
use std::collections::HashMap;

pub const JOBJ_SKELETON: u32 = 1 << 0;
pub const JOBJ_SKELETON_ROOT: u32 = 1 << 1;
pub const JOBJ_HIDDEN: u32 = 1 << 4;
pub const JOBJ_PTCL: u32 = 1 << 5;
pub const JOBJ_LIGHTING: u32 = 1 << 7;
pub const JOBJ_INSTANCE: u32 = 1 << 12;
pub const JOBJ_SPLINE: u32 = 1 << 14;
pub const JOBJ_TRSP_SHIFT: u32 = 18;
pub const JOBJ_OPA: u32 = 0x40000;
pub const JOBJ_XLU: u32 = 0x80000;
pub const JOBJ_TEXEDGE: u32 = 0x100000;
pub const HSD_TRSP_OPA: u32 = 1;
pub const HSD_TRSP_XLU: u32 = 2;
pub const HSD_TRSP_TEXEDGE: u32 = 4;
pub const DOBJ_TRSP_SHIFT: u32 = 1;

/// One joint of a drawn tree.
#[derive(Clone, Copy, Debug)]
pub struct DispJoint {
    /// The `HSD_Joint` descriptor (DAT offset).
    pub desc: u32,
    /// The JObj's runtime flags.
    pub flags: u32,
    /// `jobj->mtx` (world, after HSD_JObjSetupMatrix).
    pub mtx: Mtx,
    pub parent: Option<usize>,
}

/// A JObj tree to draw: its DAT and joints in tree (preorder) order.
pub struct DispModel<'a> {
    pub dat: Dat<'a>,
    pub joints: Vec<DispJoint>,
    by_desc: HashMap<u32, usize>,
    /// Animated materials (`HSD_MObjAnim` / `HSD_TObjAnim` results) by DObj descriptor offset: the
    /// descriptor's material, PE and TObjs are replaced by these when the DObj is drawn.
    pub mat_anim: Option<&'a HashMap<u32, super::mobj::MatOverride>>,
}

impl<'a> DispModel<'a> {
    pub fn new(dat: &'a [u8], joints: Vec<DispJoint>) -> Self {
        let by_desc = joints.iter().enumerate().map(|(i, j)| (j.desc, i)).collect();
        DispModel { dat: Dat(dat), joints, by_desc, mat_anim: None }
    }
    /// This model drawn with the animated materials `a`.
    pub fn with_mat_anim(mut self, a: &'a HashMap<u32, super::mobj::MatOverride>) -> Self {
        self.mat_anim = Some(a);
        self
    }
    /// The JObj loaded from joint descriptor `desc` (`HSD_IDGetDataFromTable`).
    pub fn by_desc(&self, desc: u32) -> Option<usize> {
        self.by_desc.get(&desc).copied()
    }
    /// `jobj->envelopemtx` (the descriptor's `mtx`, `HSD_Joint` +0x38).
    pub fn envelope_mtx(&self, j: usize) -> Option<Mtx> {
        self.dat.ptr(self.joints[j].desc + 0x38).map(|m| self.dat.mtx(m))
    }

    /// `HSD_JObjFindSkeleton`.
    pub fn find_skeleton(&self, mut j: Option<usize>) -> Option<usize> {
        while let Some(i) = j {
            if self.joints[i].flags & (JOBJ_SKELETON | JOBJ_SKELETON_ROOT) != 0 {
                return Some(i);
            }
            j = self.joints[i].parent;
        }
        None
    }

    /// `_HSD_mkEnvelopeModelNodeMtx`: `None` for a skeleton root.
    pub fn mk_envelope_model_node_mtx(&self, m: usize) -> Option<Mtx> {
        if self.joints[m].flags & JOBJ_SKELETON_ROOT != 0 {
            return None;
        }
        let x = self.find_skeleton(Some(m)).expect("displayfunc.c:422 HSD_JObjFindSkeleton");
        let ident = mtx::identity();
        if x == m {
            Some(mtx::inverse(&self.envelope_mtx(x).unwrap_or(ident)).unwrap_or(ident))
        } else if self.joints[x].flags & JOBJ_SKELETON_ROOT != 0 {
            Some(crate::hsd_mtx::inverse_concat(&self.joints[x].mtx, &self.joints[m].mtx))
        } else {
            let n = mtx::concat(&self.joints[x].mtx, &self.envelope_mtx(x).unwrap_or(ident));
            Some(crate::hsd_mtx::inverse_concat(&n, &self.joints[m].mtx))
        }
    }

    /// The joint's DObj descriptor chain head (`jobj->u.dobj`), if the union holds DObjs.
    pub fn dobj(&self, j: usize) -> Option<u32> {
        if self.joints[j].flags & (JOBJ_PTCL | JOBJ_SPLINE) != 0 {
            return None;
        }
        self.dat.ptr(self.joints[j].desc + 0x10)
    }
}

impl HsdRender {
    /// `HSD_JObjMakePositionMtx`, billboards included. `DispJoint` carries no `rotate`, so the R
    /// billboard (`JOBJ_RBILLBOARD`) uses `rotate.z = 0`; call [`HSD_JObjMakePositionMtx`] with
    /// the joint's own rotation to draw that kind.
    pub fn jobj_make_position_mtx(&self, m: &DispModel, j: usize, vmtx: &Mtx) -> Mtx {
        HSD_JObjMakePositionMtx(m.joints[j].flags, 0.0, vmtx, &m.joints[j].mtx)
    }

    /// `HSD_JObjDispSub`.
    pub fn jobj_disp_sub<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, vmtx: &Mtx, pmtx: &Mtx, trsp_mask: u32, rendermode: u32) {
        let dobj_trsp = trsp_mask << DOBJ_TRSP_SHIFT;
        // HSD_PObjClearMtxMark(NULL, 0)
        self.mtx_mark = [(None, 0); 2];
        let mut d = m.dobj(j);
        let mut guard = 0;
        while let Some(dobj) = d {
            guard += 1;
            if guard > 256 {
                break;
            }
            if self.dobj_flags(&m.dat, dobj) & dobj_trsp != 0 {
                self.dobj_disp(g, m, j, dobj, vmtx, pmtx, rendermode);
            }
            d = m.dat.ptr(dobj + 4);
        }
    }

    /// `HSD_JObjDispDObj`.
    pub fn jobj_disp_dobj<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, vmtx: &Mtx, trsp_mask: u32, rendermode: u32) {
        let flags = m.joints[j].flags;
        if flags & JOBJ_HIDDEN != 0 {
            return;
        }
        let xlu_bits = flags & (trsp_mask << JOBJ_TRSP_SHIFT);
        if xlu_bits == 0 {
            return;
        }
        let pmtx = self.jobj_make_position_mtx(m, j, vmtx);
        if xlu_bits & JOBJ_OPA != 0 {
            self.jobj_disp_sub(g, m, j, vmtx, &pmtx, HSD_TRSP_OPA, rendermode);
        }
        if xlu_bits & JOBJ_TEXEDGE != 0 {
            self.jobj_disp_sub(g, m, j, vmtx, &pmtx, HSD_TRSP_TEXEDGE, rendermode);
        }
        if xlu_bits & JOBJ_XLU != 0 {
            self.jobj_disp_sub(g, m, j, vmtx, &pmtx, HSD_TRSP_XLU, rendermode);
        }
    }

    /// `HSD_JObjDisp` (the DObj union; particle joints are skipped).
    pub fn jobj_disp<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, vmtx: &Mtx, trsp_mask: u32, rendermode: u32) {
        if m.dobj(j).is_some() {
            self.jobj_disp_dobj(g, m, j, vmtx, trsp_mask, rendermode);
        }
    }

    /// `HSD_JObjDispAll`: every joint of the tree (preorder), stopping below INSTANCE joints as
    /// the C walk does.
    pub fn jobj_disp_all<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, vmtx: &Mtx, trsp_mask: u32, rendermode: u32) {
        for j in 0..m.joints.len() {
            let mut p = m.joints[j].parent;
            let mut under_instance = false;
            while let Some(i) = p {
                if m.joints[i].flags & JOBJ_INSTANCE != 0 {
                    under_instance = true;
                    break;
                }
                p = m.joints[i].parent;
            }
            if !under_instance {
                self.jobj_disp(g, m, j, vmtx, trsp_mask, rendermode);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The rest of displayfunc.c: billboards, the Z list, StateInitDirect, the erase rectangle.
// ---------------------------------------------------------------------------------------------

pub const JOBJ_BILLBOARD_FIELD: u32 = 0xE00;
pub const JOBJ_BILLBOARD: u32 = 0x200;
pub const JOBJ_VBILLBOARD: u32 = 0x400;
pub const JOBJ_HBILLBOARD: u32 = 0x600;
pub const JOBJ_RBILLBOARD: u32 = 0x800;
pub const JOBJ_PBILLBOARD: u32 = 0x2000;

/// `zOne` / `yOne` / `zOne2`.
pub const zOne: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
pub const yOne: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
pub const zOne2: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

/// `mkVBillBoardMtx` (static): the billboard position matrix from `src = vmtx * jobj->mtx`.
pub fn mkVBillBoardMtx(jobj_flags: u32, src: &Mtx) -> Mtx {
    let mut dst: Mtx = [[0.0; 4]; 3];
    let pos = mtx_col_vec(src, 3);
    let ay = mtx_col_vec(src, 1);

    let mut sx = mtx_col_mag(src, 0);
    let mut sz = mtx_col_mag(src, 2);

    let ax = if jobj_flags & JOBJ_PBILLBOARD != 0 { mtx::vec_cross(&pos, &ay) } else { mtx::vec_cross(&ay, &zOne) };
    let az = mtx::vec_cross(&ax, &ay);
    sx /= mtx::vec_mag(&ax);
    sz /= mtx::vec_mag(&az);

    dst[0][0] = sx * ax.x;
    dst[1][0] = sx * ax.y;
    dst[2][0] = sx * ax.z;

    set_col_vec(&mut dst, 1, &ay);

    dst[0][2] = sz * az.x;
    dst[1][2] = sz * az.y;
    dst[2][2] = sz * az.z;

    set_col_vec(&mut dst, 3, &pos);
    dst
}

/// `mkHBillBoardMtx` (static).
pub fn mkHBillBoardMtx(jobj_flags: u32, src: &Mtx) -> Mtx {
    let mut dst: Mtx = [[0.0; 4]; 3];
    let pos = mtx_col_vec(src, 3);
    let ax = mtx_col_vec(src, 0);

    let mut sy = mtx_col_mag(src, 1);
    let mut sz = mtx_col_mag(src, 2);

    let az = if jobj_flags & JOBJ_PBILLBOARD != 0 {
        let uy_y = sqrtf(fmadds(pos.x, pos.x, pos.z * pos.z));
        let uy = Vec3 { x: -pos.y / uy_y * pos.x, y: uy_y, z: -pos.y / uy_y * pos.z };
        mtx::vec_cross(&ax, &uy)
    } else {
        mtx::vec_cross(&ax, &yOne)
    };
    let ay = mtx::vec_cross(&az, &ax);
    sy /= mtx::vec_mag(&ay);
    sz /= mtx::vec_mag(&az);

    set_col_vec(&mut dst, 0, &ax);

    dst[0][1] = sy * ay.x;
    dst[1][1] = sy * ay.y;
    dst[2][1] = sy * ay.z;

    dst[0][2] = sz * az.x;
    dst[1][2] = sz * az.y;
    dst[2][2] = sz * az.z;

    set_col_vec(&mut dst, 3, &pos);
    dst
}

/// `mkBillBoardMtx` (static).
pub fn mkBillBoardMtx(jobj_flags: u32, src: &Mtx) -> Mtx {
    let mut dst: Mtx = [[0.0; 4]; 3];
    let mut sx = mtx_col_mag(src, 0);
    let mut sz = mtx_col_mag(src, 2);

    let mut ay = mtx_col_vec(src, 1);
    let mut sy = mtx::vec_mag(&ay);
    let pos = mtx_col_vec(src, 3);

    let ax;
    let az;
    if jobj_flags & JOBJ_PBILLBOARD != 0 {
        ax = mtx::vec_cross(&pos, &ay);
        ay = mtx::vec_cross(&ax, &pos);
        az = pos;
        sz /= -mtx::vec_mag(&az);
    } else {
        ax = mtx::vec_cross(&ay, &zOne2);
        ay = mtx::vec_cross(&zOne2, &ax);
        az = zOne2;
        sz /= mtx::vec_mag(&zOne2);
    }
    sx /= mtx::vec_mag(&ax);
    sy /= mtx::vec_mag(&ay);

    dst[0][0] = sx * ax.x;
    dst[1][0] = sx * ax.y;
    dst[2][0] = sx * ax.z;

    dst[0][1] = sy * ay.x;
    dst[1][1] = sy * ay.y;
    dst[2][1] = sy * ay.z;

    dst[0][2] = sz * az.x;
    dst[1][2] = sz * az.y;
    dst[2][2] = sz * az.z;

    set_col_vec(&mut dst, 3, &pos);
    dst
}

/// `mkRBillBoardMtx` (static): `rotate_z` is `jobj->rotate.z`.
pub fn mkRBillBoardMtx(rotate_z: f32, src: &Mtx) -> Mtx {
    let sx = mtx_col_mag(src, 0);
    let sy = mtx_col_mag(src, 1);
    let sz = mtx_col_mag(src, 2);
    let scl = mtx::scale(sx, sy, sz);
    let mut rot: Mtx = [[0.0; 4]; 3];
    mtx::rot_rad(&mut rot, b'z', rotate_z);
    rot[0][3] = src[0][3];
    rot[1][3] = src[1][3];
    rot[2][3] = src[2][3];
    mtx::concat(&rot, &scl)
}

/// `HSD_MtxSetColVec` (mtx.h inline).
fn set_col_vec(m: &mut Mtx, col: usize, v: &Vec3) {
    m[0][col] = v.x;
    m[1][col] = v.y;
    m[2][col] = v.z;
}

/// `HSD_JObjMakePositionMtx` with billboards: `jobj_flags`, `rotate_z` (`jobj->rotate.z`, read
/// only by the R billboard), the view matrix and the joint's world matrix.
pub fn HSD_JObjMakePositionMtx(jobj_flags: u32, rotate_z: f32, vmtx: &Mtx, jobj_mtx: &Mtx) -> Mtx {
    if jobj_flags & JOBJ_BILLBOARD_FIELD != 0 {
        let m = mtx::concat(vmtx, jobj_mtx);
        match jobj_flags & JOBJ_BILLBOARD_FIELD {
            JOBJ_BILLBOARD => mkBillBoardMtx(jobj_flags, &m),
            JOBJ_VBILLBOARD => mkVBillBoardMtx(jobj_flags, &m),
            JOBJ_HBILLBOARD => mkHBillBoardMtx(jobj_flags, &m),
            JOBJ_RBILLBOARD => mkRBillBoardMtx(rotate_z, &m),
            _ => panic!("displayfunc.c:322 unkown type of billboard."),
        }
    } else {
        mtx::concat(vmtx, jobj_mtx)
    }
}

/// One `HSD_ZList` node: a deferred TEXEDGE / XLU draw of a joint. `jobj` is the caller's joint
/// handle; `vmtx` is the private copy of the view matrix (NULL in the C when the caller passed
/// none, then the current camera's is used at draw time).
#[derive(Clone, Debug)]
pub struct ZListEntry<J> {
    pub pmtx: Mtx,
    pub vmtx: Option<Mtx>,
    pub jobj: J,
    pub rendermode: u32,
}

/// The Z list's statics: `zlist_top` (draw order of insertion, all entries), the TEXEDGE and XLU
/// sort chains with their counts. The C's linked `HSD_ZList` pool (`zlist_alloc_data`) is a `Vec`
/// arena here; chains hold indices. `zsort_listing` / `zsort_sorting` are the C's file-static ints
/// (never written in the shipped build: both 0).
#[derive(Clone, Debug)]
pub struct ZListState<J> {
    pub entries: Vec<ZListEntry<J>>,
    texedge: Vec<usize>,
    xlu: Vec<usize>,
    pub zsort_listing: bool,
    pub zsort_sorting: bool,
}

impl<J> Default for ZListState<J> {
    fn default() -> Self {
        ZListState { entries: Vec::new(), texedge: Vec::new(), xlu: Vec::new(), zsort_listing: false, zsort_sorting: false }
    }
}

impl<J: Clone> ZListState<J> {
    /// `HSD_ZListInitAllocData`: the pool (`HSD_ObjAllocInit(&zlist_alloc_data, sizeof(HSD_ZList), 4)`)
    /// is the arena `Vec`; nothing to set up.
    pub fn HSD_ZListInitAllocData(&mut self) {}

    /// The `zsort_listing != 0` branch of `HSD_JObjDispDObj`: queue the joint for the sorted passes.
    pub fn add(&mut self, jobj: J, pmtx: Mtx, vmtx: Option<Mtx>, rendermode: u32, xlu_bits: u32) {
        if xlu_bits & (JOBJ_TEXEDGE | JOBJ_XLU) == 0 {
            return;
        }
        let idx = self.entries.len();
        self.entries.push(ZListEntry { pmtx, vmtx, jobj, rendermode });
        if xlu_bits & JOBJ_TEXEDGE != 0 {
            self.texedge.push(idx);
        }
        if xlu_bits & JOBJ_XLU != 0 {
            self.xlu.push(idx);
        }
    }

    /// `zlist_sort`: stable merge sort of `list` by `pmtx[2][3]`, equal keys keep the earlier
    /// entry first (`fore <= hind`), and a NaN key takes the `hind` side exactly as the C compare.
    fn zlist_sort(&self, list: &[usize]) -> Vec<usize> {
        if list.len() <= 1 {
            return list.to_vec();
        }
        let nb_fore = list.len() / 2;
        let fore = self.zlist_sort(&list[..nb_fore]);
        let hind = self.zlist_sort(&list[nb_fore..]);
        let mut out = Vec::with_capacity(list.len());
        let (mut i, mut j) = (0, 0);
        while i < fore.len() && j < hind.len() {
            if self.entries[fore[i]].pmtx[2][3] <= self.entries[hind[j]].pmtx[2][3] {
                out.push(fore[i]);
                i += 1;
            } else {
                out.push(hind[j]);
                j += 1;
            }
        }
        out.extend_from_slice(&fore[i..]);
        out.extend_from_slice(&hind[j..]);
        out
    }

    /// `_HSD_ZListSort`.
    pub fn HSD_ZListSort(&mut self) {
        if self.zsort_sorting {
            self.texedge = self.zlist_sort(&self.texedge);
            self.xlu = self.zlist_sort(&self.xlu);
        }
    }

    /// `_HSD_ZListDisp`: draws the TEXEDGE chain then the XLU chain through `disp(jobj, vmtx,
    /// pmtx, trsp, rendermode)` (the entry's own view matrix, else `camera_vmtx`), then clears.
    pub fn HSD_ZListDisp(&mut self, camera_vmtx: &Mtx, mut disp: impl FnMut(&J, &Mtx, &Mtx, u32, u32)) {
        for &i in &self.texedge {
            let e = &self.entries[i];
            disp(&e.jobj, e.vmtx.as_ref().unwrap_or(camera_vmtx), &e.pmtx, HSD_TRSP_TEXEDGE, e.rendermode);
        }
        for &i in &self.xlu {
            let e = &self.entries[i];
            disp(&e.jobj, e.vmtx.as_ref().unwrap_or(camera_vmtx), &e.pmtx, HSD_TRSP_XLU, e.rendermode);
        }
        self.HSD_ZListClear();
    }

    /// `_HSD_ZListClear`.
    pub fn HSD_ZListClear(&mut self) {
        self.entries.clear();
        self.texedge.clear();
        self.xlu.clear();
    }

    /// `_HSD_DispForgetMemory(lo, hi)`: the pool's memory is gone; drop the lists.
    pub fn HSD_DispForgetMemory(&mut self) {
        self.HSD_ZListClear();
    }
}

/// `sptcl_callback` and `erase_color`: displayfunc.c's other statics.
pub struct DispStatics {
    /// `HSD_JObjSetSPtclCallback`: `(0, bank, offset, jobj)`.
    pub sptcl_callback: Option<fn(i32, i32, i32, usize)>,
    pub erase_color: Rgba,
}

impl Default for DispStatics {
    fn default() -> Self {
        DispStatics { sptcl_callback: None, erase_color: [0, 0, 0, 0] }
    }
}

impl DispStatics {
    /// `HSD_JObjSetSPtclCallback`.
    pub fn HSD_JObjSetSPtclCallback(&mut self, func: Option<fn(i32, i32, i32, usize)>) {
        self.sptcl_callback = func;
    }

    /// `HSD_SetEraseColor`.
    pub fn HSD_SetEraseColor(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.erase_color = [r, g, b, a];
    }

    /// The particle branch of `HSD_JObjDisp`: for each entry of the joint's particle list
    /// (`jobj->u.ptcl`, data words), call the callback when the high bit is set, then keep only
    /// the active bit (`JOBJ_PTCL_ACTIVE`).
    pub fn disp_ptcl(&self, jobj: usize, data: &mut [u32]) {
        const JOBJ_PTCL_BANK_MASK: u32 = 0x3F;
        const JOBJ_PTCL_ACTIVE: u32 = 0x7FFF_FFFF;
        if let Some(cb) = self.sptcl_callback {
            for d in data.iter_mut() {
                if *d & 0x8000_0000 != 0 {
                    let bank = (JOBJ_PTCL_BANK_MASK & *d) as i32;
                    let offset = ((*d >> 6) & 0xFF_FFFF) as i32;
                    cb(0, bank, offset, jobj);
                }
                *d &= JOBJ_PTCL_ACTIVE;
            }
        }
    }
}

/// `HSD_StateInitDirect`.
pub fn HSD_StateInitDirect<G: GxBackend>(r: &mut HsdRender, g: &mut G, vtxfmt: u32, rendermode: u32) {
    const GX_VA_CLR0: u32 = 11;
    const GX_POS_XYZ: u32 = 1;
    const GX_CLR_RGBA: u32 = 1;
    const GX_F32: u32 = 4;
    const GX_RGBA8: u32 = 5;
    r.clear_vtx_desc(g);
    g.set_vtx_attr_fmt(vtxfmt, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
    g.set_vtx_attr_fmt(vtxfmt, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    r.state_init_tev();
    r.setup_render_mode(g, rendermode | 0x2800_0000);
    g.set_vtx_desc(GX_VA_POS, GX_DIRECT);
    g.set_vtx_desc(GX_VA_CLR0, GX_DIRECT);
    g.set_current_mtx(GX_PNMTX0);
}

/// The immediate-mode GX calls `HSD_EraseRect` makes beyond [`GxBackend`].
pub trait EraseGx: GxBackend {
    /// `GXInitTexObj(&texobj, depth_image, 4, 4, GX_TF_Z8, GX_REPEAT, GX_REPEAT, GX_DISABLE)` +
    /// `GXLoadTexObj(&texobj, GX_TEXMAP0)` over the 4x4 all-255 Z8 image.
    fn load_z8_texture_map0(&mut self, image: &[u8; 128]);
    /// `GXSetZTexture(op, GX_TF_Z8, 0)`.
    fn set_z_texture(&mut self, op: u32);
    /// `GXBegin(GX_QUADS, GX_VTXFMT0, 4)` .. `GXEnd()` with `GXPosition3f32` / `GXColor4u8` /
    /// `GXTexCoord2u8` per vertex.
    fn draw_quad(&mut self, verts: &[([f32; 3], Rgba, [u8; 2]); 4]);
}

/// `depth_image` (128 bytes of 255; the C array is 32-byte aligned).
pub const DEPTH_IMAGE: [u8; 128] = [255; 128];

/// `HSD_EraseRect`.
pub fn HSD_EraseRect<G: EraseGx>(
    r: &mut HsdRender,
    g: &mut G,
    erase_color: Rgba,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
    z: f32,
    enable_color: bool,
    enable_alpha: bool,
    enable_depth: bool,
) {
    const GX_ZT_DISABLE: u32 = 0;
    const GX_ZT_REPLACE: u32 = 1;
    const GX_ALWAYS: u32 = 7;
    const GX_AOP_OR: u32 = 1;
    const GX_BM_LOGIC: u32 = 2;
    const GX_BL_ONE: u32 = 1;
    const GX_BL_ZERO: u32 = 0;
    const GX_LO_COPY: u32 = 3;
    const GX_LIGHT_NULL: u32 = 0;
    const GX_VA_CLR0: u32 = 11;
    const GX_VA_TEX0: u32 = 13;
    const GX_POS_XYZ: u32 = 1;
    const GX_CLR_RGBA: u32 = 1;
    const GX_F32: u32 = 4;
    const GX_RGBA8: u32 = 5;
    const GX_TEX_ST: u32 = 1;
    const GX_U8: u32 = 0;
    const GX_TEXCOORD0: u32 = 0;
    const GX_TEXMAP0: u32 = 0;

    if !enable_color && !enable_alpha && !enable_depth {
        return;
    }

    if enable_depth {
        g.load_z8_texture_map0(&DEPTH_IMAGE);
        g.set_num_tex_gens(1);
        g.set_tex_coord_gen2(GX_TEXCOORD0, GX_TG_MTX2x4, GX_TG_TEX0, GX_IDENTITY, false, GX_PTIDENTITY);
        g.set_num_tev_stages(1);
        g.set_tev_order(0, GX_TEXCOORD0, GX_TEXMAP0, GX_COLOR0A0);
        g.set_tev_op(0, GX_PASSCLR);
        g.set_z_texture(GX_ZT_REPLACE);
    } else {
        g.set_num_tex_gens(0);
        g.set_num_tev_stages(1);
        g.set_tev_order(0, GX_TEXCOORD_NULL, GX_TEXMAP_NULL, GX_COLOR0A0);
        g.set_tev_op(0, GX_PASSCLR);
    }

    g.set_cull_mode(GX_CULL_NONE);

    g.set_alpha_compare(GX_ALWAYS, 0, GX_AOP_OR, GX_ALWAYS, 0);
    g.set_z_comp_loc(true);
    g.set_z_mode(true, GX_ALWAYS, enable_depth);
    g.set_blend_mode(GX_BM_LOGIC, GX_BL_ONE, GX_BL_ZERO, GX_LO_COPY);
    g.set_color_update(enable_color);
    g.set_alpha_update(enable_alpha);

    g.set_num_chans(1);
    g.set_chan_ctrl(GX_COLOR0A0, false, GX_SRC_REG, GX_SRC_VTX, GX_LIGHT_NULL, GX_DF_NONE, GX_AF_NONE);

    g.clear_vtx_desc();
    g.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_POS, GX_POS_XYZ, GX_F32, 0);
    g.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_CLR0, GX_CLR_RGBA, GX_RGBA8, 0);
    g.set_vtx_attr_fmt(GX_VTXFMT0, GX_VA_TEX0, GX_TEX_ST, GX_U8, 0);
    g.load_pos_mtx_imm(&mtx::identity(), GX_PNMTX0);
    g.set_current_mtx(GX_PNMTX0);
    g.set_vtx_desc(GX_VA_POS, GX_DIRECT);
    g.set_vtx_desc(GX_VA_CLR0, GX_DIRECT);
    g.set_vtx_desc(GX_VA_TEX0, GX_DIRECT);

    let c = erase_color;
    g.draw_quad(&[
        ([left, top, z], c, [0, 0]),
        ([right, top, z], c, [1, 0]),
        ([right, bottom, z], c, [1, 1]),
        ([left, bottom, z], c, [0, 1]),
    ]);

    g.set_z_texture(GX_ZT_DISABLE);

    r.invalidate_all();
}
