//! Port of src/melee/ef/efdata.c: the effect library's shared globals (it defines no functions).
//!
//! The C's `.bss`/`.sbss` objects are the fields of [`EfData`]; efalt/efasync/eflib take it by
//! reference instead of naming the globals.
#![allow(non_snake_case)]

use melee_types::HSD_ObjAllocData;

/// The globals of efdata.c.
#[derive(Clone, Copy)]
pub struct EfData {
    /// `efLib_AllocData` (0x80458EB0).
    pub alloc_data: HSD_ObjAllocData,
    /// `efLib_AnimCount` (0x804D64F0): entries pending in `efLib_AnimQueue`.
    pub anim_count: i32,
    /// `efLib_EffectCount` (0x804D64EC).
    pub effect_count: u32,
    /// `efLib_LoadKind` (0x804D64E8): `EF_LOADKIND_ASYNC` / `EF_LOADKIND_SYNC`.
    pub load_kind: u32,
}

impl Default for EfData {
    fn default() -> Self {
        // SAFETY: HSD_ObjAllocData is plain data (the C object is zero-initialised .bss).
        Self { alloc_data: unsafe { core::mem::zeroed() }, anim_count: 0, effect_count: 0, load_kind: 0 }
    }
}
