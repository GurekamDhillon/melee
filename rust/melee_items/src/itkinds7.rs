
//! Port round 7 (night 5): hand ports of 12 item functions whose callees are all ported, from
//! the decomp C (raw c2rust output as reference). Verified against verbatim C copies
//! (tests/c_oracle/round7_bodies.h, extracted by gen_round7.py) in tests/oracle_round7.rs.
//!
//! Fused sites (`tools/rust-port/fma_sites.py` on the source TUs, shipping-build contraction):
//! f32 `fmadds` in `it_802A3C98` 2, `it_802A6A78_normalize_diff` 2,
//! `it_802A6A78_normalize_diff_rev` 2 (`x*x + y*y + z*z` = `fma(z, z, fma(x, x, y*y))`), and
//! `it_2F28_UnkMotion0_Phys` 1 (`fma(x, x, y*y)`). None elsewhere. `sqrtf` is MSL's
//! math_ppc.h `extern inline` (frsqrte + 3 Newton steps): `melee_runtime::gekko::sqrtf`.

use crate::layout::{
    Article, CollData, Fighter, HSD_GObj, Item, ItemPickTable, Mtx, SpawnItem, Vec as GcVec,
    itFoodsAttributes, It_Kind_Unk4,
};

use melee_core::fp::fmadds;

use melee_runtime::gekko::sqrtf;

/// `|v|` of the stored difference as the C writes it: `sqrtf(x*x + y*y + z*z)`, two fused sites.
#[inline(always)]
pub(crate) fn len3(v: &GcVec) -> f32 {
    sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)))
}

/// `it_2F28_UnkMotion0_Phys` (it_2F28.c): decelerate the xy velocity by `unk2.x0` along its
/// direction; below that the x velocity is zeroed (y is left as is, as the C does).
pub fn it_2f28_unk_motion0_phys(item: &mut Item) {
    let v = item.x40_vel;
    let var_f4 = sqrtf(v.x * v.x + v.y * v.y); // memory/computed squares: not fused on the console (notes/fp-audit.md)
    // SAFETY: plain-data union member.
    let d = unsafe { item.xDD4_itemVar.unk2.x0 };
    if var_f4 > d {
        item.x40_vel.x = (item.x40_vel.x * (var_f4 - d)) / var_f4;
        item.x40_vel.y = (item.x40_vel.y * (var_f4 - d)) / var_f4;
    } else {
        item.x40_vel.x = 0.0;
    }
}

/// `bisectValue` (itspawn.c, static): the index `i` in `[lo, hi)` with
/// `table.xC[i] <= val < table.xC[i + 1]` (recursive bisection, as the C).
///
/// # Safety
/// `table.xC` must point to at least `hi + 1` entries.
pub unsafe fn bisect_value(val: i32, table: &ItemPickTable, lo: i32, hi: i32) -> i32 {
    if lo == hi - 1 {
        return lo;
    }
    let mid = (lo + hi) / 2;
    if *table.xC.offset(mid as isize) as i32 > val {
        bisect_value(val, table, lo, mid)
    } else {
        if *table.xC.offset(mid as isize + 1) as i32 > val {
            return mid;
        }
        bisect_value(val, table, mid, hi)
    }
}

/// `itTarucann_UnkMotion9_Anim_inline` (ittarucann.c, static inline):
/// `xDCC_flag.b3 = (tarucann.x20 == 0)`.
pub fn it_tarucann_unk_motion9_anim_inline(ip: &mut Item) {
    // SAFETY: plain-data union member.
    let none = unsafe { ip.xDD4_itemVar.tarucann.x20.is_null() };
    ip.xDCC_flag.set_b3(none as u8);
}

/// `itKyasarin_SetFallCollPos` (itkyasarin.c, static inline): `coll.cur_pos = *pos = ip.pos`.
pub fn it_kyasarin_set_fall_coll_pos(ip: &Item, coll: &mut CollData, pos: &mut GcVec) {
    coll.cur_pos = ip.pos;
    *pos = ip.pos;
}

/// `itArwinglaser_GetCollArg` (itarwinglaser.c, static inline): `arwinglaser.xE48`.
pub fn it_arwinglaser_get_coll_arg(ip: &Item) -> i32 {
    // SAFETY: plain-data union member.
    unsafe { ip.xDD4_itemVar.arwinglaser.xE48 }
}

/// `getRandMax` (itfoods.c, static inline): `x0` of the foods special attributes.
///
/// # Safety
/// `article.x4_specialAttributes` must point to an `itFoodsAttributes`.
pub unsafe fn get_rand_max(article: &Article) -> u32 {
    (*(article.x4_specialAttributes as *const itFoodsAttributes)).x0 as u32
}

pub use crate::tu::itlinkhookshot::it_802a3c98;
pub use crate::tu::itlinkhookshot::it_802a6a78_normalize_diff;
pub use crate::tu::itlinkhookshot::it_802a6a78_normalize_diff_rev;
pub use crate::tu::it_2E5A::it_802e614c;
pub use crate::tu::itlinkhookshot::link_fighter_compare_fp as link_fighter_compare;
pub use crate::tu::itlinkhookshot::vec3_eq_mtx_fp as vec3_eq_mtx;
