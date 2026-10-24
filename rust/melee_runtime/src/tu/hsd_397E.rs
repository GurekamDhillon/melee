//! Port of src/sysdolphin/baselib/hsd_397E.c: `baselib_mfspr`, the crash screen's special-purpose
//! register read. Every `case` is an inline `mfspr` under `#ifdef MWERKS_GEKKO`; the PC build
//! (clang, not MWERKS) compiles none of them, so every call reports the register number and
//! returns 0 (inventory.md section 3, "return 0").
#![allow(non_snake_case)]

/// Callees of hsd_397E.c.
pub trait Hsd397EHost {
    /// `OSReport` with the C format already applied.
    fn OSReport(&mut self, msg: &str);
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl Hsd397EHost for Ext {
        fn OSReport(&mut self, _: &str) { todo!("OSReport (dolphin/os)") }
    }
}

/// `baselib_mfspr`: the non-MWERKS build has only the `default:` arm.
pub fn baselib_mfspr<H: Hsd397EHost + ?Sized>(h: &mut H, spr: i32) -> i32 {
    h.OSReport(&format!("unsupported no. of special purpose register ({}).", spr));
    0
}
