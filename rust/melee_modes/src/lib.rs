//! Game modes and menus (gm, mn) and the port's game-side code (script_game, geno, mex_tests).
//!
//! Port group 6 (docs/rust-port/inventory.md, docs/rust-port/group-modes.md). Raw c2rust output
//! for this group's TUs is written to `src/raw/` by `tools/rust-port/transpile.sh`; it is
//! reference input for the hand port and is deliberately not declared as a module here.
//!
//! Round 1 (night 2): the leaf functions of gm, mn, if, ty and the lb TUs group 1 left, i.e.
//! those with no calls and no globals, each verified bit for bit against the C (`tests/oracle.rs`).
#![allow(dead_code)]

pub use melee_core;

pub mod gm;
pub mod hud;
pub mod lb;
pub mod mn;
/// Round 3: leaf functions over game structs (pointer arguments), `tests/oracle_ptr.rs`.
pub mod gm_ptr;
pub mod hud_ptr;
pub mod lb_ptr;
pub mod mn_ptr;
/// Round 4: callers of rounds 1-3 (and of other groups' ported code), `tests/oracle_r4.rs`.
pub mod gm_r4;
pub mod lb_r4;
pub mod mn_r4;
/// Round 5: leaf functions over nested pointers that round 3 skipped, `tests/oracle_r5.rs`.
pub mod gm_r5;
pub mod hud_r5;
pub mod lb_r5;
pub mod mn_r5;
/// Round 6 (night 3 round 2): callers of round 5 and leftover leaves, `tests/oracle_r6.rs`.
pub mod gm_r6;
pub mod hud_r6;
pub mod lb_r6;

pub mod gm_r7;
pub mod hud_r7;
/// Round 8 (night 3 round 4): lb leaves over nested pointers, `tests/oracle_r8.rs`.
pub mod lb_r8;

/// `GXColor` (libs/dolphin/include/dolphin/gx/GXStruct.h). TODO(group0): melee_types does not
/// generate it (no group-0 header reaches GXStruct.h); move it there when it does.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GXColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

melee_core::gc_layout!(GXColor, "GXColor", size 4, align 1, [
    r @ 0x0: 1 "u8",
    g @ 0x1: 1 "u8",
    b @ 0x2: 1 "u8",
    a @ 0x3: 1 "u8",
]);

impl GXColor {
    /// `r<<24 | g<<16 | b<<8 | a`: the colour's GC (big-endian) word.
    pub fn to_u32(self) -> u32 {
        u32::from_be_bytes([self.r, self.g, self.b, self.a])
    }
    pub fn from_u32(v: u32) -> Self {
        let [r, g, b, a] = v.to_be_bytes();
        GXColor { r, g, b, a }
    }
}
