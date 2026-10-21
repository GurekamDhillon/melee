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
fn len3(v: &GcVec) -> f32 {
    sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)))
}

/// `it_802A3C98` (itlinkhookshot.c): `*arg2 = normalize(*arg0 - *arg1)`, returns the length.
/// The zero test and the reciprocal are in f64 (`(f64) 1.0F / len`), rounded to f32.
pub fn it_802a3c98(arg0: &GcVec, arg1: &GcVec, arg2: &mut GcVec) -> f32 {
    arg2.x = arg0.x - arg1.x;
    arg2.y = arg0.y - arg1.y;
    arg2.z = arg0.z - arg1.z;
    let len = len3(arg2);
    let inv: f32 = if len as f64 == 0.0 { 0.0 } else { (1.0f64 / len as f64) as f32 };
    arg2.x *= inv;
    arg2.y *= inv;
    arg2.z *= inv;
    len
}

/// `it_802A6A78_normalize_diff` (itlinkhookshot.c, static inline): `*vec = normalize(*a - *b)`,
/// returns the f32 length widened to f64.
pub fn it_802a6a78_normalize_diff(a: &GcVec, b: &GcVec, vec: &mut GcVec) -> f64 {
    vec.x = a.x - b.x;
    vec.y = a.y - b.y;
    vec.z = a.z - b.z;
    let len = len3(vec) as f64;
    let inv: f32 = if len == 0.0 { 0.0 } else { (1.0f64 / len) as f32 };
    vec.x *= inv;
    vec.y *= inv;
    vec.z *= inv;
    len
}

/// `it_802A6A78_normalize_diff_rev` (itlinkhookshot.c, static inline): as
/// [`it_802a6a78_normalize_diff`] with the parameters named the other way round (`b` first),
/// still `*a - *b`.
pub fn it_802a6a78_normalize_diff_rev(b: &GcVec, a: &GcVec, vec: &mut GcVec) -> f64 {
    it_802a6a78_normalize_diff(a, b, vec)
}

/// `vec3_eq_mtx` (itlinkhookshot.c, static inline): the translation column of `m`.
pub fn vec3_eq_mtx(vec: &mut GcVec, m: &Mtx) {
    vec.x = m[0][3];
    vec.y = m[1][3];
    vec.z = m[2][3];
}

/// `link_fighter_compare` (itlinkhookshot.c, static inline): motion 0x168, 0x169, 0xD4 or 0xD6.
pub fn link_fighter_compare(fp: &Fighter) -> bool {
    matches!(fp.motion_id, 0x168 | 0x169 | 0xD4 | 0xD6)
}

/// `it_2F28_UnkMotion0_Phys` (it_2F28.c): decelerate the xy velocity by `unk2.x0` along its
/// direction; below that the x velocity is zeroed (y is left as is, as the C does).
pub fn it_2f28_unk_motion0_phys(item: &mut Item) {
    let v = item.x40_vel;
    let var_f4 = sqrtf(fmadds(v.x, v.x, v.y * v.y));
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

/// `it_802E614C` (it_2E5A.c, static inline): fill a `SpawnItem` for kind `It_Kind_Unk4` at `pos`.
pub fn it_802e614c(parent_gobj1: *mut HSD_GObj, parent_gobj2: *mut HSD_GObj, spawn: &mut SpawnItem,
    pos: &GcVec, vel: &GcVec) {
    spawn.kind = It_Kind_Unk4;
    spawn.prev_pos = *pos;
    spawn.pos = spawn.prev_pos;
    spawn.x3C_damage = 0;
    spawn.vel = *vel;
    spawn.x0_parent_gobj = parent_gobj1;
    spawn.x4_parent_gobj2 = parent_gobj2;
    // SAFETY: plain-data union member.
    unsafe { spawn.x44_flag.anon_0.set_b0(1) };
    spawn.x40 = 0;
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
