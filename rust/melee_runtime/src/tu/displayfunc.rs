//! `src/sysdolphin/baselib/displayfunc.c`: `HSD_JObjDisp` / `HSD_JObjDispDObj` /
//! `HSD_JObjDispSub`, `HSD_JObjMakePositionMtx`, `HSD_JObjFindSkeleton`,
//! `_HSD_mkEnvelopeModelNodeMtx`, over a [`DispModel`]: a joint tree's descriptors (DAT) plus the
//! world matrices and runtime flags the JObj system computed for it this frame.
//!
//! Not ported: billboards (mk*BillBoardMtx: joints with JOBJ_BILLBOARD_FIELD are drawn with the
//! plain position matrix), the z-sorted list (`zsort_listing` is off in Melee's main scene; the
//! XLU / TEXEDGE passes run in tree order), particle joints' `sptcl_callback`, specular init.
use super::state::{Dat, GxBackend, HsdRender};
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
}

impl<'a> DispModel<'a> {
    pub fn new(dat: &'a [u8], joints: Vec<DispJoint>) -> Self {
        let by_desc = joints.iter().enumerate().map(|(i, j)| (j.desc, i)).collect();
        DispModel { dat: Dat(dat), joints, by_desc }
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
    /// `HSD_JObjMakePositionMtx` (billboards: TODO displayfunc.c:222, drawn unbillboarded).
    pub fn jobj_make_position_mtx(&self, m: &DispModel, j: usize, vmtx: &Mtx) -> Mtx {
        mtx::concat(vmtx, &m.joints[j].mtx)
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
