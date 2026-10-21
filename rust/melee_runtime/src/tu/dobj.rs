//! `src/sysdolphin/baselib/dobj.c`, the display half: `DObjLoad`'s transparency bits and
//! `HSD_DObjDisp`. (The flag functions are the verified ports in `crate::dobj`.)
use super::displayfunc::DispModel;
use super::state::{Dat, GxBackend, HsdRender};
use crate::mtx::Mtx;

pub const DOBJ_HIDDEN: u32 = 1;

impl HsdRender {
    /// The DObj's flags after `DObjLoad`: `HSD_DObjModifyFlags(dobj, 2/8/4, 0xE)` from the MObj's
    /// blending bits (0: OPA 2, XLU: TEXEDGE 8, XLU|NO_ZUPDATE: XLU 4). Descriptors carry no DObj
    /// flags of their own, so a loaded DObj starts at 0.
    pub fn dobj_flags(&self, d: &Dat, dobj: u32) -> u32 {
        let Some(mobj) = d.ptr(dobj + 8) else { return 0 };
        match d.u32_(mobj + 4) & 0x6000_0000 {
            0 => 2,
            0x4000_0000 => 8,
            0x6000_0000 => 4,
            // dobj.c:312 "mobj has unexpected blending flags": the C panics.
            _ => 0,
        }
    }

    /// `HSD_DObjDisp` (no LAB script-parts overrides).
    pub fn dobj_disp<G: GxBackend>(&mut self, g: &mut G, m: &DispModel, j: usize, dobj: u32, vmtx: &Mtx, pmtx: &Mtx, rendermode: u32) {
        if self.dobj_flags(&m.dat, dobj) & DOBJ_HIDDEN != 0 {
            return;
        }
        let mobj = m.dat.ptr(dobj + 8);
        if rendermode & 0x400_0000 == 0 {
            if let Some(mo) = mobj {
                self.mobj_setup(g, &m.dat, mo);
            }
        }
        let mut p = m.dat.ptr(dobj + 0xC);
        let mut guard = 0;
        while let Some(pobj) = p {
            guard += 1;
            if guard > 1024 {
                break;
            }
            self.pobj_disp(g, m, j, pobj, vmtx, pmtx, rendermode);
            p = m.dat.ptr(pobj + 4);
        }
        if rendermode & 0x400_0000 == 0 {
            self.mobj_unset(g);
        }
    }
}
