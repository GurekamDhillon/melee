//! Port of src/melee/it/itspawn.c: the random item spawner (its timer, the stage's allowed-item weights),
//! the weighted pick tables it_804A0E50 / it_804A0E60 / it_804A0E30.x4 and the item-spawn helpers that
//! place an item at a position. The MUST_MATCH-only layout hack (`sdata2_order`) is not compiled on PC and
//! is not ported. Callees of other TUs go through `ext` (unset hooks panic "TODO").
//! Checked against the C by the state oracle `tests/tu_itspawn.rs` (gen/mtu).
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_mut, unused_assignments, unused_variables, clippy::missing_safety_doc)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::helpers::COLLIDE_CEILING_MASK;
#[allow(unused_imports)]
use crate::itlib::it_8026b724;
#[allow(unused_imports)]
use crate::itlib::it_8026b73c;
#[allow(unused_imports)]
use crate::itlib::it_8026bb88;
#[allow(unused_imports)]
use crate::itlib::it_8026bbcc;
#[allow(unused_imports)]
use crate::itlib::it_802714c0;
#[allow(unused_imports)]
use crate::itlib::it_8027346c;
#[allow(unused_imports)]
use crate::itlib::it_80273484;
#[allow(unused_imports)]
use crate::itlib::it_8027349c;
#[allow(unused_imports)]
use crate::itlib::it_802734b4;
#[allow(unused_imports)]
use crate::itlib::it_80275228;
#[allow(unused_imports)]
use crate::itlib::it_80275240;
#[allow(unused_imports)]
use crate::itlib::COLLIDE_FLOOR_MASK;
#[allow(unused_imports)]
use crate::layout::it_2728_DatAttrs;
#[allow(unused_imports)]
use crate::layout::it_804D6D40_t;
#[allow(unused_imports)]
use crate::layout::itPokemonSpawn_DatAttrs;
#[allow(unused_imports)]
use crate::layout::Collide_LeftWallMask;
#[allow(unused_imports)]
use crate::layout::Collide_RightWallMask;
#[allow(unused_imports)]
use crate::layout::Fighter;
#[allow(unused_imports)]
use crate::layout::HitCapsule;
#[allow(unused_imports)]
use crate::layout::ItemLink;
#[allow(unused_imports)]
use crate::layout::ItemStateDesc;
#[allow(unused_imports)]
use crate::layout::PokemonSelectionState;
#[allow(unused_imports)]
use crate::layout::lb_80014638_arg1_t;
#[allow(unused_imports)]
use crate::layout::HSD_GOBJ_CLASS_ITEM;
#[allow(unused_imports)]
use crate::layout::It_Kind_L_Gun_Ray;
#[allow(unused_imports)]
use crate::layout::It_PKind_Marumine;
#[allow(unused_imports)]
use crate::layout::It_PKind_Start;
#[allow(unused_imports)]
use crate::layout::It_PKind_Terminate;
#[allow(unused_imports)]
use std::ptr::null_mut as null_mut_itlib3;
#[allow(unused_imports)]
use crate::itlib3::{c_f32_to_s32};
#[allow(unused_imports)]
use crate::itkinds7::bisect_value;
#[allow(unused_imports)]
use crate::layout::itEgg_ItemVars;
#[allow(unused_imports)]
use crate::layout::itLizardonAttributes;
#[allow(unused_imports)]
use crate::layout::itPeachTurnipAttributes;
#[allow(unused_imports)]
use crate::layout::itTaruAttributes;
#[allow(unused_imports)]
use crate::layout::it_2E5A_Attrs;
#[allow(unused_imports)]
use crate::layout::ItemCommonData;
#[allow(unused_imports)]
use crate::layout::ItemPickTable;
#[allow(unused_imports)]
use melee_core::fp::frsqrte;
#[allow(unused_imports)]
use melee_runtime::random::HsdRand;
#[allow(unused_imports)]
use crate::helpers::pika_scale;
#[allow(unused_imports)]
use crate::itkinds11::it_80272cc0;
#[allow(unused_imports)]
use crate::itkinds13::swap_velocity;
#[allow(unused_imports)]
use crate::itkinds5::it_802ac5d8;
#[allow(unused_imports)]
use crate::itkinds5::it_802b3368;
#[allow(unused_imports)]
use crate::itkinds8::get_min_life;
#[allow(unused_imports)]
use crate::itkinds9::my_sqrtf_accurate;
#[allow(unused_imports)]
use crate::itlib::it_8026b3a8;
#[allow(unused_imports)]
use crate::itlib::it_80275444;
#[allow(unused_imports)]
use crate::itlib::it_80275594;
#[allow(unused_imports)]
use crate::itlib3::it_80272828;
#[allow(unused_imports)]
use crate::itlib3::it_80272860;
#[allow(unused_imports)]
use crate::layout::Article;
#[allow(unused_imports)]
use crate::layout::CommandInfo;
#[allow(unused_imports)]
use crate::layout::HSD_GObj;
#[allow(unused_imports)]
use crate::layout::HSD_JObj;
#[allow(unused_imports)]
use crate::layout::HSD_Joint;
#[allow(unused_imports)]
use crate::layout::Item;
#[allow(unused_imports)]
use crate::layout::ItemStateTable;
#[allow(unused_imports)]
use crate::layout::Vec as GcVec;
#[allow(unused_imports)]
use crate::layout::it_8026C47C_arg0_t;
#[allow(unused_imports)]
use crate::layout::itBombHeiAttributes;
#[allow(unused_imports)]
use crate::layout::itClimbersStringAttributes;
#[allow(unused_imports)]
use crate::layout::itLikelikeAttributes;
#[allow(unused_imports)]
use crate::layout::itNessPKFirepillarAttributes;
#[allow(unused_imports)]
use crate::layout::itSamusBombAttributes;
#[allow(unused_imports)]
use crate::layout::itSScopeAttributes;
#[allow(unused_imports)]
use crate::layout::ItemKind;
#[allow(unused_imports)]
use crate::layout::It_Kind_Capsule;
#[allow(unused_imports)]
use crate::layout::It_Kind_RabbitC;
#[allow(unused_imports)]
use melee_core::fp::fmsubs;
#[allow(unused_imports)]
use melee_core::vec::Vec3 as RVec;
#[allow(unused_imports)]
use melee_runtime::lbvector;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

/// One argument of a variadic effect spawn (`efSync_Spawn` / `efAsync_Spawn` are C varargs; the port keeps the
/// C type of each extra argument so the eventual implementation can read it back).
#[derive(Clone, Copy)]
pub enum EfArg {
    Int(i32),
    Ptr(*mut c_void),
    F32P(*const f32),
    Vec3P(*const Vec3),
}

// BEGIN GENERATED EXT (rust/melee_items/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(unreachable_code, non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(=> $fb:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $( return $fb; )? melee_core::unported!(concat!("TODO: ", stringify!($name), " is not ported")) }
                }
            } )*
        };
    }
    ext_fns! {
    fn GObj_Create(a0: u16, a1: u8, a2: u8) -> *mut HSD_GObj;
    fn Ground_801C2AD8() -> *mut i32 => unsafe { melee_world::tu::ground::Ground_801C2AD8() };
    fn Ground_801C2AE8(a0: u32) -> f32 => unsafe { melee_world::tu::ground::Ground_801C2AE8(a0) };
    fn HSD_GObj_SetupProc(a0: *mut HSD_GObj, a1: Option<unsafe extern "C" fn(*mut HSD_GObj)>, a2: u8) -> *mut HSD_GObjProc => unsafe { melee_world::tu_ext::HSD_GObj_SetupProc(a0, a1, a2) };
    fn HSD_MemAlloc(a0: i32) -> *mut c_void => unsafe { melee_runtime::tu::memory::HSD_MemAlloc(a0) };
    fn HSD_Randf() -> f32 => unsafe { melee_runtime::tu::random::global::HSD_Randf() };
    fn HSD_Randi(a0: i32) -> i32 => unsafe { melee_runtime::tu::random::global::HSD_Randi(a0) };
    fn Item_80268B18(a0: *mut SpawnItem) -> *mut HSD_GObj => unsafe { crate::tu::item::Item_80268B18(a0) };
    fn Item_80268B9C(a0: *mut SpawnItem) => unsafe { crate::tu::item::Item_80268B9C(a0) };
    fn Stage_80224FDC(a0: *mut Vec) -> i32 => unsafe { melee_world::tu::stage::Stage_80224FDC(a0 as _) };
    fn Stage_80225194() -> u32 => unsafe { melee_world::tu::stage::Stage_80225194() };
    fn db_AreItemSpawnsEnabled() -> i32 => unsafe { crate::tu::dbitem::db_AreItemSpawnsEnabled() };
    fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: &[EfArg]) -> *mut c_void;
    fn gm_8016AE80() -> i32;
    fn gm_8016AE94() -> f32;
    fn gm_8016AEA4() -> u64;
    fn gm_8016B238() -> i32;
    fn it_8026B684(a0: *mut Vec) -> f32 => unsafe { crate::tu::it_26B1::it_8026B684(a0 as _) };
    fn it_80272828(a0: i32) -> *mut HSD_GObj => unsafe { crate::tu::it_2725::it_80272828(a0) };
    fn it_80274ED8() => unsafe { crate::tu::it_2725::it_80274ED8() };
    fn mpColl_8004D024(a0: *mut Vec) -> i32 => unsafe { melee_world::tu::mpcoll::mpColl_8004D024(a0) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
    }
    // Data defined in other TUs (the tests set them).
    pub use crate::tu::item::Item_804A0C64;
    pub use crate::tu::it_3F14::it_804D6D28;
}
// END GENERATED EXT

/// `it_804A0E60`: the monster-item weighted pick table (ItemPickTable), built by `it_8026CF04`.
pub static mut it_804A0E60: ItemPickTable = unsafe { core::mem::zeroed() };
/// `it_804A0E50`: the second-tier pick table (kinds from `It_Kind_BombHei` on), built by `it_8026CD50`.
pub static mut it_804A0E50: ItemPickTable = unsafe { core::mem::zeroed() };
/// `it_804A0E30`: the random item spawner.
pub static mut it_804A0E30: RandomItemSpawner = unsafe { core::mem::zeroed() };

/// `GW_PICK_TABLE_EMPTY` (the PC guard of `it_8026C75C`): an empty table has no backing arrays.
#[inline(always)]
unsafe fn pick_table_empty(t: *const ItemPickTable) -> bool {
    (*t).size == 0 || (*t).x4.is_null() || (*t).xC.is_null()
}

/// `it_8026C47C`: fills the 8-word bitset `arg_struct` with one bit per item kind that
/// `it_80272828` reports as present (kinds 0..238, 32 kinds per word).
pub unsafe extern "C" fn it_8026C47C(arg_struct: *mut it_8026C47C_arg0_t) {
    melee_core::ct!("it_8026C47C");
    let mut it_kind: u32;
    let mut unused: u32;
    let mut bit_idx: i32;
    let mut word: *mut i32;

    it_kind = It_Kind_Capsule as u32;
    unused = it_kind;
    let _ = unused;
    bit_idx = 0;
    word = &raw mut (*arg_struct).unk0;
    (*arg_struct).unk0 = 0;
    (*arg_struct).unk4 = 0;
    (*arg_struct).unk8 = 0;
    (*arg_struct).unkC = 0;
    (*arg_struct).unk10 = 0;
    (*arg_struct).unk14 = 0;
    (*arg_struct).unk18 = 0;
    (*arg_struct).unk1C = 0;
    while it_kind < 238 {
        if !ext::it_80272828(it_kind as ItemKind).is_null() {
            *word |= 1i32.wrapping_shl(bit_idx as u32);
        }
        it_kind += 1;
        bit_idx += 1;
        if it_kind & It_Kind_RabbitC as u32 == 0 {
            bit_idx = 0;
            word = word.add(1);
        }
    }
}

/// Bisection search in the item table for a given value (`bisectValue`).
unsafe fn bisectValue(val: i32, table: *mut ItemPickTable, lo: i32, hi: i32) -> i32 {
    // base case, done bisecting
    if lo == hi - 1 {
        return lo;
    }
    let mid = (lo + hi) / 2;
    if (*(*table).xC.add(mid as usize)) as i32 > val {
        // recurse into lower half
        bisectValue(val, table, lo, mid)
    } else if (*(*table).xC.add(mid as usize + 1)) as i32 > val {
        mid
    } else {
        // recurse into upper half
        bisectValue(val, table, mid, hi)
    }
}

/// `it_8026C65C`: a random kind from a weighted pick table.
pub unsafe extern "C" fn it_8026C65C(table: *mut ItemPickTable) -> ItemKind {
    melee_core::ct!("it_8026C65C");
    let temp_r6 = (*table).x8 as i32;
    let idx = bisectValue(ext::HSD_Randi(temp_r6), table, 0, (*table).size as i32);
    *(*table).x4.add(idx as usize) as ItemKind
}

/// `it_8026C704`: true when the M-Ball may not be picked (item cap reached, or the stage has no M-Ball).
pub unsafe extern "C" fn it_8026C704() -> bool {
    melee_core::ct!("it_8026C704");
    let mut result = false;
    let obj = &raw const ext::Item_804A0C64;
    if (*obj).x1C >= (*obj).x20 || !it_8026D324(It_Kind_M_Ball) {
        result = true;
    }
    result
}

/// `it_8026C75C`: decides the item kind for a spawn from the given pick table. Returns -1 for "none".
pub unsafe extern "C" fn it_8026C75C(table: *mut ItemPickTable) -> ItemKind {
    melee_core::ct!("it_8026C75C");
    let mut chk1: bool;
    let mut saved: i32 = 0;
    let mut chk2: bool;
    let ret: ItemKind;
    let kind: ItemKind;
    let tbl = table;

    chk1 = false;
    let obj = &raw const ext::Item_804A0C64;
    if (*obj).x1C >= (*obj).x20 || !it_8026D324(It_Kind_M_Ball) {
        chk1 = true;
    }
    chk2 = false;
    if (*tbl).x8 == 0 {
        return -1;
    }
    if chk1 {
        if *(*tbl).x4.add((*tbl).size as usize - 1) as ItemKind == It_Kind_M_Ball {
            let i_last = (*tbl).size as i32 - 1;
            if i_last < 1 {
                return -1;
            }
            saved = (*tbl).x8 as i32;
            chk2 = true;
            (*tbl).x8 = *(*tbl).xC.add(i_last as usize);
            (*tbl).size = (*tbl).size.wrapping_sub(1);
        }
    }
    // GW_PICK_TABLE_EMPTY guard (PC): see the comment at the C definition.
    if pick_table_empty(tbl) {
        if chk1 && chk2 {
            (*tbl).x8 = saved as u16;
            (*tbl).size = (*tbl).size.wrapping_add(1);
        }
        return -1;
    }
    kind = it_8026C65C(tbl);

    ret = kind;
    if chk1 && chk2 {
        (*tbl).x8 = saved as u16;
        (*tbl).size = (*tbl).size.wrapping_add(1);
        if kind == It_Kind_M_Ball {
            return -1;
        }
    }
    ret
}

/// `it_8026C88C_inline`: counts the spawner down, spawns a random item when it reaches zero and re-arms it.
unsafe fn it_8026C88C_inline(alloc: *mut RandomItemSpawner) {
    let mut pos: *mut Vec3;
    let mut chk: bool;
    let mut spawn_gobj: *mut HSD_GObj;
    let mut spawn: SpawnItem = core::mem::zeroed();
    if ext::db_AreItemSpawnsEnabled() != 0 {
        (*alloc).x0 -= 1;
        if (*alloc).x0 == 0 {
            spawn.kind = it_8026C75C(&raw mut (*alloc).x4);
            if spawn.kind != -1 {
                pos = &raw mut spawn.prev_pos;
                if it_8026CB3C(pos) {
                    spawn.pos = *pos;
                    spawn.facing_dir = ext::it_8026B684(pos);
                    chk = true;
                    spawn.x3C_damage = 0;
                    spawn.vel.x = 0.0;
                    spawn.vel.y = 0.0;
                    spawn.vel.z = 0.0;
                    spawn.x0_parent_gobj = null_mut();
                    spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
                    spawn.x44_flag.anon_0.set_b0(1);
                    spawn.x40 = 0;
                } else {
                    chk = false;
                }
                if chk {
                    spawn_gobj = ext::Item_80268B18(&raw mut spawn);
                    if !spawn_gobj.is_null() {
                        ext::efSync_Spawn(0x420, spawn_gobj, &[EfArg::Vec3P(pos)]);
                        ext::it_80274ED8();
                    }
                }
            }
            // The spawn interval: a random value in the stage's range for the current item frequency.
            let idx = (ext::gm_8016AE80() as i32) * 2;
            let range: *const i32 = (*ext::it_804D6D28).xFC.as_ptr().add(idx as usize);
            let randf = ext::HSD_Randf();
            let diff = *range.add(1) - *range;
            (*alloc).x0 = melee_core::ppc::fctiwz_f32(fmadds(diff as f32, randf, *range as f32));
            (*alloc).x0 = melee_core::ppc::fctiwz_f32(((*alloc).x0 as f32) * ext::Ground_801C2AE8(ext::Stage_80225194()));
        }
    }
}

/// `fn_8026C88C`: the spawner's per-frame proc.
pub unsafe extern "C" fn fn_8026C88C(gobj: *mut HSD_GObj) {
    melee_core::ct!("fn_8026C88C");
    let _ = gobj;
    let alloc = &raw mut it_804A0E30;
    it_8026C88C_inline(alloc);
}

/// `it_8026CA4C`: total weight of the kinds set in `arg2` (from kind `arg3`) into `alloc.x8`.
pub unsafe extern "C" fn it_8026CA4C(alloc: *mut ItemPickTable, arg1: *mut i32, arg2: u64, arg3: i32, arg4: f32) {
    melee_core::ct!("it_8026CA4C");
    let mut mask = arg2;
    let mut p = arg1.add(arg3 as usize);
    let mut i = arg3;
    let mut sum: i32 = 0;

    while i < It_Kind_L_Gun_Ray {
        if mask & 1 != 0 {
            // sum += arg4 * *p + 0.99f; (the product-add fuses; the outer += does not)
            let val = fmadds(arg4, *p as f32, 0.99f32);
            sum = melee_core::ppc::fctiwz_f32((sum as f32) + val);
        }
        p = p.add(1);
        i += 1;
        mask >>= 1;
    }
    (*alloc).x8 = sum as u16;
}

/// `it_8026CB3C`: true when `vec` is over a stage floor that is not blocked by a map collision.
pub unsafe extern "C" fn it_8026CB3C(vec: *mut Vec3) -> bool {
    melee_core::ct!("it_8026CB3C");
    if ext::Stage_80224FDC(vec) == 0 {
        return false;
    }
    (*vec).z = 0.0;
    if ext::mpColl_8004D024(vec) != 0 {
        return false;
    }
    true
}

/// `it_8026CB9C`: builds the first-tier pick table `it_804A0E30.x4` (kinds below `It_Kind_L_Gun_Ray`).
pub unsafe extern "C" fn it_8026CB9C(counts: *mut i32, mask: u64, weight: f32) {
    melee_core::ct!("it_8026CB9C");
    let spawner = &raw mut it_804A0E30;
    let mut p: *mut i32;
    let mut cnt: i32;
    let mut it_kind: ItemKind;
    let mut cnt2: i32;
    let mut it_kind2: ItemKind;
    let mut p2: *mut i32;
    let mut cumulative: i32;
    let mut idx: i32;
    let mut mask = mask;
    let backup = mask;

    p = counts;
    it_kind = 0;
    cnt = 0;
    while it_kind < It_Kind_L_Gun_Ray {
        if mask & 1 != 0 && *p != 0 {
            cnt += 1;
        }
        p = p.add(1);
        it_kind += 1;
        mask >>= 1;
    }
    (*spawner).x4.size = cnt as u8;
    let item_kinds: *mut *mut u8 = &raw mut (*spawner).x4.x4;
    *item_kinds = ext::HSD_MemAlloc((cnt * 4) as i32) as *mut u8;
    let weights: *mut *mut u16 = &raw mut (*spawner).x4.xC;
    *weights = ext::HSD_MemAlloc((cnt * 4) as i32) as *mut u16;

    idx = 0;
    cnt2 = 0;
    mask = backup;
    p2 = counts;
    it_kind2 = 0;
    cumulative = 0;
    while it_kind2 < It_Kind_L_Gun_Ray {
        if mask & 1 != 0 && *p2 != 0 {
            *(*item_kinds).add(cnt2 as usize) = it_kind2 as u8;
            *(*weights).add(idx as usize) = cumulative as u16;
            cnt2 += 1;
            idx += 1;
            cumulative = melee_core::ppc::fctiwz_f32((cumulative as f32) + fmadds(weight, *p2 as f32, 0.99f32));
        }
        p2 = p2.add(1);
        it_kind2 += 1;
        mask >>= 1;
    }
}

/// `it_8026CD50`: builds the second-tier pick table `it_804A0E50` (kinds from `It_Kind_BombHei` on).
pub unsafe extern "C" fn it_8026CD50(counts: *mut i32, mask: u64, weight: f32) {
    melee_core::ct!("it_8026CD50");
    let mut p: *mut i32;
    let mut cnt: i32;
    let mut it_kind: ItemKind;
    let mut it_kind2: ItemKind;
    let mut cnt2: i32;
    let mut p2: *mut i32;
    let mut idx: i32;
    let mut cumulative: i32;
    let mut mask = mask;
    let backup = mask;

    p = counts.add(It_Kind_BombHei as usize);
    cnt = 0;
    it_kind = It_Kind_BombHei;
    while it_kind < It_Kind_L_Gun_Ray {
        if mask & 1 != 0 && *p != 0 {
            cnt += 1;
        }
        p = p.add(1);
        it_kind += 1;
        mask >>= 1;
    }
    // Non-MUST_MATCH layout (the one the PC build uses): the table is its own global.
    let tbl = &raw mut it_804A0E50;
    (*tbl).size = cnt as u8;
    let item_kinds: *mut *mut u8 = &raw mut (*tbl).x4;
    *item_kinds = ext::HSD_MemAlloc((cnt * 4) as i32) as *mut u8;
    let weights: *mut *mut u16 = &raw mut (*tbl).xC;
    *weights = ext::HSD_MemAlloc((cnt * 4) as i32) as *mut u16;

    idx = 0;
    cnt2 = 0;
    mask = backup;
    p2 = counts.add(It_Kind_BombHei as usize);
    it_kind2 = It_Kind_BombHei;
    cumulative = 0;
    while it_kind2 < It_Kind_L_Gun_Ray {
        if mask & 1 != 0 && *p2 != 0 {
            *(*item_kinds).add(cnt2 as usize) = it_kind2 as u8;
            *(*weights).add(idx as usize) = cumulative as u16;
            cnt2 += 1;
            idx += 1;
            cumulative = melee_core::ppc::fctiwz_f32((cumulative as f32) + fmadds(weight, *p2 as f32, 0.99f32));
        }
        p2 = p2.add(1);
        it_kind2 += 1;
        mask >>= 1;
    }
}

/// `it_8026CF04`: builds the monster-item weighted pick table `it_804A0E60` from the common data's
/// first four weights (`x128`).
pub unsafe extern "C" fn it_8026CF04() {
    melee_core::ct!("it_8026CF04");
    let counts: *mut u32 = (*ext::it_804D6D28).x128.as_ptr() as *mut u32;
    let mut sum: i32;
    let mut cumulative: u32;
    let mut idx: u32;
    let mut i: i32;
    sum = *counts as i32;
    sum = sum.wrapping_add(*counts.add(1) as i32);
    sum = sum.wrapping_add(*counts.add(2) as i32);
    sum = sum.wrapping_add(*counts.add(3) as i32);
    if sum != 0 {
        let tbl = &raw mut it_804A0E60;
        (*tbl).x8 = sum as u16;
        (*tbl).size = 4;
        (*tbl).x4 = ext::HSD_MemAlloc((((*tbl).size as usize) * 4) as i32) as *mut u8;
        (*tbl).xC = ext::HSD_MemAlloc((((*tbl).size as usize) * 4) as i32) as *mut u16;
        idx = 0;
        i = 0;
        cumulative = 0;
        while i < 4 {
            *(*tbl).x4.add(i as usize) = (It_Kind_Kuriboh + i) as u8;
            *(*tbl).xC.add(idx as usize) = cumulative as u16;
            // The C also has `(void) xC[(u32) &item_common->x128[idx]]`: a dead read of a pointer-as-index
            // with no effect on state; it is dropped.
            cumulative = cumulative.wrapping_add(*counts.add(idx as usize));
            i += 1;
            idx += 1;
        }
    }
}

/// `it_8026D018_inline`: the first-tier table for this stage; false when there is nothing to pick.
unsafe fn it_8026D018_inline() -> bool {
    let stage_mask = it_804A0E30.x18;
    let stage_info = ext::Ground_801C2AD8();
    let chk = ext::gm_8016AE80() as i32;
    let weight = ext::gm_8016AE94();

    if stage_mask == 0 || stage_info.is_null() || chk == -1 {
        return false;
    }
    it_8026CA4C(&raw mut it_804A0E30.x4, stage_info, stage_mask, 0, weight);
    if it_804A0E30.x4.x8 == 0 {
        return false;
    }
    it_8026CB9C(stage_info, stage_mask, weight);
    true
}

/// `it_8026D018_inline2`: the second-tier table (kinds from `It_Kind_BombHei`).
unsafe fn it_8026D018_inline2() {
    let mut stage_mask = it_804A0E30.x18;
    let stage_info = ext::Ground_801C2AD8();
    let weight = ext::gm_8016AE94();

    if stage_mask != 0 && !stage_info.is_null() {
        stage_mask >>= 6;
        it_8026CA4C(&raw mut it_804A0E50, stage_info, stage_mask, 6, weight);
        if it_804A0E50.x8 != 0 {
            it_8026CD50(stage_info, stage_mask, weight);
        }
    }
}

/// `it_8026D018_inline3`: re-arms the spawner's countdown from a random point in `range`.
unsafe fn it_8026D018_inline3(randf: f32, range: *const i32) {
    let diff = *range.add(1) - *range;
    it_804A0E30.x0 = melee_core::ppc::fctiwz_f32(fmadds(diff as f32, randf, *range as f32));
    it_804A0E30.x0 = melee_core::ppc::fctiwz_f32((it_804A0E30.x0 as f32) * ext::Ground_801C2AE8(ext::Stage_80225194()));
}

/// `it_8026D018`: sets up the item spawner for the match (called at match start).
pub unsafe extern "C" fn it_8026D018() {
    melee_core::ct!("it_8026D018");
    if ext::gm_8016B238() == 0 && (ext::gm_8016AE80() as i32) != -1 {
        it_804A0E30.x18 = ext::gm_8016AEA4();
        if it_8026D018_inline() {
            it_8026D018_inline2();
            it_8026CF04();
            ext::HSD_GObj_SetupProc(ext::GObj_Create(5, 7, 0), Some(fn_8026C88C), 0);
            // argument order: HSD_Randf() is evaluated before gm_8016AE80() (left to right)
            let randf = ext::HSD_Randf();
            let idx = (ext::gm_8016AE80() as i32) * 2;
            it_8026D018_inline3(randf, (*ext::it_804D6D28).xFC.as_ptr().offset(idx as isize));
        }
    }
}

/// `it_8026D258`: spawns an item of `kind` at `pos` (z forced to 0). False when the item cap is reached.
pub unsafe extern "C" fn it_8026D258(pos: *mut Vec3, kind: ItemKind) -> bool {
    melee_core::ct!("it_8026D258");
    let mut spawn: SpawnItem = core::mem::zeroed();
    let mut item_spawn_chk = false;
    let obj = &raw const ext::Item_804A0C64;
    if (*obj).x60 < (*obj).x64 {
        spawn.kind = kind;
        spawn.prev_pos = *pos;
        spawn.prev_pos.z = 0.0;
        spawn.pos = spawn.prev_pos;
        spawn.facing_dir = ext::it_8026B684(&raw mut spawn.prev_pos);
        spawn.x3C_damage = 0;
        spawn.vel.z = 0.0;
        spawn.vel.y = 0.0;
        spawn.vel.x = 0.0;
        spawn.x0_parent_gobj = null_mut();
        spawn.x4_parent_gobj2 = spawn.x0_parent_gobj;
        spawn.x44_flag.anon_0.set_b0(1);
        spawn.x40 = 0;
        ext::Item_80268B9C(&raw mut spawn);
        item_spawn_chk = true;
    }
    item_spawn_chk
}

/// `it_8026D324`: true when `kind` is allowed on the current stage (the stage has items, a stage table
/// exists and the match's item frequency is set).
pub unsafe extern "C" fn it_8026D324(kind: ItemKind) -> bool {
    melee_core::ct!("it_8026D324");
    let temp_r29 = it_804A0E30.x18;
    let temp_r30 = ext::Ground_801C2AD8();
    let temp_r3 = ext::gm_8016AE80() as i32;
    if temp_r29 == 0 || temp_r30.is_null() || temp_r3 == -1 {
        return false;
    }
    // PPC `srd` by 64 or more yields 0.
    let bit = temp_r29.checked_shr(kind as u32).unwrap_or(0);
    if bit & 1 == 0 {
        return false;
    }
    true
}

/// `it_8026D3CC`: true when a heart, a tomato or a food item is allowed on this stage.
pub unsafe extern "C" fn it_8026D3CC() -> bool {
    melee_core::ct!("it_8026D3CC");
    let mut result = it_8026D324(It_Kind_Heart);
    result |= it_8026D324(It_Kind_Tomato);
    result |= it_8026D324(It_Kind_Foods);
    result
}

// ---- folded from itkinds14.rs (consolidation steps 21/22) ----
// ---- itspawn.c --------------------------------------------------------------------------

/// `it_8026C47C`: one bit per item kind `It_Kind_Capsule..238` that has a live item on the
/// p-link list (`it_80272828`), 32 kinds per word (`!(kind & It_Kind_RabbitC)` starts a word).
/// # Safety
/// As `it_80272828` (`head` the item p-link list head).
pub unsafe fn it_8026c47c(head: *mut HSD_GObj, arg: &mut it_8026C47C_arg0_t) {
    melee_core::ct!("it_8026c47c");
    let mut it_kind = It_Kind_Capsule as u32;
    let mut bit_idx: i32 = 0;
    *arg = it_8026C47C_arg0_t::zeroed();
    // SAFETY: eight consecutive i32 fields (unk0..unk1C), repr(C).
    let words = core::slice::from_raw_parts_mut(arg as *mut it_8026C47C_arg0_t as *mut i32, 8);
    let mut w = 0usize;
    while it_kind < 238 {
        if !it_80272828(head, it_kind as ItemKind).is_null() {
            words[w] |= 1i32.wrapping_shl(bit_idx as u32);
        }
        it_kind += 1;
        bit_idx += 1;
        if it_kind & It_Kind_RabbitC as u32 == 0 {
            bit_idx = 0;
            w += 1;
        }
    }
}

// ---- folded from itkinds9.rs (consolidation steps 21/22) ----
/// `it_8026C65C` (itspawn.c): weighted kind pick from an `ItemPickTable`.
/// # Safety
/// `table.x4` holds `table.size` kinds and `table.xC` the `size + 1` cumulative weights.
pub unsafe fn it_8026c65c(rand: &mut HsdRand, table: &ItemPickTable) -> i32 {
    melee_core::ct!("it_8026c65c");
    let total = table.x8 as i32;
    let v = rand.randi(total);
    *table.x4.offset(bisect_value(v, table, 0, table.size as i32) as isize) as i32
}

// ---- folded from itlib3.rs (consolidation steps 21/22) ----
// ---- itspawn.c -------------------------------------------------------------------------

/// `it_8026CA4C`: sum of the scaled, rounded-up weights of the kinds selected by `mask`
/// (bit 0 = kind `start`), for kinds below `It_Kind_L_Gun_Ray`. `sum += f` is
/// `sum = (s32) ((f32) sum + f)`; 1 fused site: `fmadds(arg4, (f32) w, 0.99)`. The result is
/// stored to the u16 `x8`.
pub fn it_8026ca4c(alloc: &mut ItemPickTable, weights: &[i32], mask: u64, start: i32, arg4: f32) {
    melee_core::ct!("it_8026ca4c");
    let mut mask = mask;
    let mut sum: i32 = 0;
    let mut i = start;
    while i < It_Kind_L_Gun_Ray {
        if mask & 1 != 0 {
            let f = fmadds(arg4, weights[i as usize] as f32, 0.99f32);
            sum = c_f32_to_s32(sum as f32 + f);
        }
        i += 1;
        mask >>= 1;
    }
    alloc.x8 = sum as u16;
}
