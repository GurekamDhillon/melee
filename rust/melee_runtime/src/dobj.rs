//! Port of the flag half of `src/sysdolphin/baselib/dobj.c` (the display object).
//!
//! The native DObj record is not built yet (its MObj / PObj members are other rounds), so these
//! work on the DObj's `flags` word (`HSD_DObj.flags`, GameCube offset 0x14 in
//! `melee_types::HSD_DObj`); `None` is the C's NULL DObj. `current_dobj` is [`DObjGlobals`].
//! `fma_sites.py`: no fused site in dobj.c.
//!
//! Not ported: the anim functions (need MObj/PObj anim: TODO(group1 mobj/pobj)), DObjLoad /
//! LoadDesc / ResolveRefs, Remove / RemoveAll / Alloc / class functions, HSD_DObjDisp (render).

/// dobj.c's file static `current_dobj`, as the caller's DObj handle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DObjGlobals<D> {
    pub current_dobj: Option<D>,
}

impl<D> DObjGlobals<D> {
    /// `HSD_DObjSetCurrent`.
    pub fn set_current(&mut self, dobj: Option<D>) {
        self.current_dobj = dobj;
    }
}

/// `HSD_DObjGetFlags`.
pub fn get_flags(flags: Option<&u32>) -> u32 {
    flags.copied().unwrap_or(0)
}

/// `HSD_DObjSetFlags`.
pub fn set_flags(flags: Option<&mut u32>, f: u32) {
    if let Some(x) = flags {
        *x |= f;
    }
}

/// `HSD_DObjClearFlags`.
pub fn clear_flags(flags: Option<&mut u32>, f: u32) {
    if let Some(x) = flags {
        *x &= !f;
    }
}

/// `HSD_DObjModifyFlags`.
pub fn modify_flags(flags: Option<&mut u32>, f: u32, mask: u32) {
    if let Some(x) = flags {
        *x = (*x & !mask) | (f & mask);
    }
}
