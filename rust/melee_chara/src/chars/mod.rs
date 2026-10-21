//! Hand ports of character functions (`src/melee/ft/kinds/<Char>/<tu>.c`), one module per C TU
//! under one module per character directory, C names kept so each maps back to its source.
//!
//! Conventions (shared with group 3's `melee_fighter::ftcommon`):
//! - A function that took `HSD_GObj*` and only used it for `GET_FIGHTER(gobj)` takes the
//!   fighter (`&Fighter` / `&mut Fighter`). Those that test the gobj for NULL take `Option`.
//! - `fp->dat_attrs` is read through [`da`], typed by the struct the C casts it to.
//! - Every f32 expression keeps the C operation order; fused sites (clang contraction in the
//!   shipping build, `tools/rust-port/fma_sites.py`) are written with `melee_core::fp::fmadds`.
//! - Calls into other groups go through stubs/hooks; none of the functions here need one except
//!   [`caps_max_jumps`] (the PC port's script caps add-on, group 6).
//!
//! Every function here is checked against the C (shipping-build contraction) by
//! tests/chars_ref.rs; the list is gen/ported.txt.
#![allow(non_snake_case, unused_unsafe)]

use melee_types::Fighter;

/// `(T*) fp->dat_attrs`: the character's special attributes block (loaded from its Pl*.dat).
/// The block is owned by the fighter data, not by `fp`, so the borrow is not tied to `fp`.
#[inline(always)]
pub(crate) fn da<T>(fp: &Fighter) -> &'static T {
    // SAFETY: dat_attrs points at the character's attribute block for the fighter's lifetime;
    // the C reads it through the same cast.
    unsafe { &*(fp.dat_attrs as *const T) }
}

/// `ABS(x)` from Runtime/platform.h (`ABS(-0.0) == -0.0`).
pub(crate) use melee_fighter::ftcommon::abs_c;

/// `ScriptGame_CapsMaxJumps(fp)` behind `FT_CAPS_MAX_JUMPS` (pc/gameworld/script_fighter_caps.h,
/// TARGET_PC). TODO(group6): the script caps add-on lives in melee_modes; until it is ported the
/// hook is unset and the vanilla value (`co_attrs.max_jumps`) is used.
pub static mut CAPS_MAX_JUMPS: Option<fn(fp: &Fighter) -> i32> = None;

#[inline]
pub fn caps_max_jumps(fp: &Fighter) -> i32 {
    match unsafe { CAPS_MAX_JUMPS } {
        Some(f) => f(fp),
        None => fp.co_attrs.max_jumps,
    }
}

/// Round 2: thin callbacks translated by gen/port_r2.py (Coll/Phys/Anim wrappers, motion-state
/// transitions) that call group 3 through [`crate::stubs`]. Path: `chars::r2::<Char>::<tu>::<fn>`.
pub mod r2;
/// Round 3: gen/port_r3.py output (field assignments, calls into round-1/2 ports). Path: `chars::r3::<Char>::<tu>::<fn>`.
pub mod r3;
/// Callback entries stored into Fighter by the round-3 ports (gen/port_r3.py).
pub mod cb;

pub mod ftCaptain;
pub mod ftCrazyHand;
pub mod ftDonkey;
pub mod ftFox;
pub mod ftGameWatch;
pub mod ftGigaKoopa;
pub mod ftKirby;
pub mod ftKoopa;
pub mod ftLink;
pub mod ftLuigi;
pub mod ftMario;
pub mod ftMars;
pub mod ftMasterHand;
pub mod ftMewtwo;
pub mod ftNana;
pub mod ftNess;
pub mod ftPeach;
pub mod ftPikachu;
pub mod ftPopo;
pub mod ftPurin;
pub mod ftSamus;
pub mod ftSeak;
pub mod ftYoshi;
pub mod ftZelda;
