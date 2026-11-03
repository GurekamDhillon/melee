//! C globals that more than one crate reads while the defining TU lives in a crate above a reader (the crate
//! graph is runtime < world < fighter < chara < items < modes). One static each: the owning TU's module and every
//! reader's `ext` block `pub use` the item here (tools/rust-port/engine/fix_ext_statics.py), so the engine sets
//! and the port reads the same word. Values are the C initialisers.
#![allow(non_upper_case_globals)]
use melee_types::*;

/// db.c `/* 4D4A08 */ DbLKind DbLevel`.
pub static mut DbLevel: DbLKind = 0;
/// gm_18A1.c `UnkAllstarData gm_80473A18`.
pub static mut gm_80473A18: UnkAllstarData = unsafe { core::mem::zeroed() };
/// gmmultiman.c `gm_CStickSmashTargetTest`.
pub static mut gm_CStickSmashTargetTest: i32 = 0;
/// lb_0146.c colours.
pub static mut lb_ColorWhite: GXColor = GXColor { r: 255, g: 255, b: 255, a: 255 };
pub static mut lb_ColorBlack: GXColor = GXColor { r: 0, g: 0, b: 0, a: 255 };
/// iteffect.c fills these from ItCo (`it_8027870C`).
pub static mut it_804D6D38: *mut *mut Article = core::ptr::null_mut();
pub static mut it_804D6D28: *mut ItemCommonData = core::ptr::null_mut();
/// fighter.c `p_ftCommonData` (PlCo symbol 0) and the PlCo tables read across crates.
pub static mut p_ftCommonData: *mut ftCommonData = core::ptr::null_mut();
pub static mut Fighter_804D64FC: *mut Fighter_804D64FC_t = core::ptr::null_mut();
pub static mut Fighter_804D6518: *mut Fighter_804D6518_t = core::ptr::null_mut();
pub static mut Fighter_804D651C: *mut Fighter_804D651C_t = core::ptr::null_mut();
pub static mut Fighter_804D6524: *mut Fighter_804D6524_t = core::ptr::null_mut();
pub static mut Fighter_804D6534: *mut core::ffi::c_void = core::ptr::null_mut();

/// ftdata.c `ftData_UnkBytePerCharacter` (`-1` entries are 0xFF): the effect bank per character kind, read by
/// the effect loader (efasync) and the fighter data.
/// Sized `Ft_Kind_Max` as the C: the m-ex kind slots are filled at fighter init (ftData_MexInitKinds); the rest are 0.
pub static mut ftData_UnkBytePerCharacter: [u8; melee_types::Ft_Kind_Max as usize] = {
    let head: [u8; 33] = [
        1, 3, 4, 8, 5, 12, 6, 17, 10, 15, 14, 14, 7, 2, 9, 11, 13, 18, 16, 17, 6, 1, 3, 7, 0xFF, 19,
        49, 0xFF, 0xFF, 0xFF, 0xFF, 12, 0xFF,
    ];
    let mut t = [0u8; melee_types::Ft_Kind_Max as usize];
    let mut i = 0;
    while i < 33 {
        t[i] = head[i];
        i += 1;
    }
    t
};
