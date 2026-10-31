//! Hand port of src/melee/ft/ft_081B.c (fighter environment-collision wrappers): every function,
//! static inlines included (the `*_inline` helpers are private Rust helpers shared by their
//! callers). `ft_80084CB0` and `ft_80084DB0` were already ported in `crate::ft084e` and are
//! re-exported here.
//!
//! Fused sites (shipping contraction): 2, both in ft_80084CE4 (`(top + bottom) * 0.5F + pos.y`
//! -> `fmadds`). `ft_80081B38`, `ft_80081C88`, `ft_80081DD4/F2C/2084`, `ft_80082838` and
//! `ft_80084CE4` are checked against the C (cut verbatim, shipping contraction) by
//! tests/tu_ft_081b.rs; the rest are smoke-tested there with stubbed callees.
#![allow(static_mut_refs, non_snake_case)]

use crate::types::*;
use melee_core::fp::fmadds;

pub use crate::ft084e::{ft_80084cb0, ft_80084db0};

pub type G = *mut HSD_GObj;
/// `bool (*)(Fighter_GObj*, int)` passed through to mpColl.
pub type CollCb = fn(G, i32) -> bool;
/// `HSD_GObjEvent`.
pub type Ev = fn(G);

/// `Collide_FloorMask` (mp/forward.h:111, a `#define` over the generated enumerators).
pub const COLLIDE_FLOOR_MASK: i32 = Collide_FloorPush | Collide_FloorHug;

/// Calls out of this TU: settable fn pointers. Unported callees panic with a TODO when reached
/// unset; ported ones (`extd!`) fall back to the native port.
pub mod ext {
    use super::{CollCb, G};
    use crate::types::*;
    macro_rules! ext {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " not ported")),
                }
            }
        )*};
    }
    macro_rules! extd {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:path;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => $def($($a),*),
                }
            }
        )*};
    }
    ext! {
        MPCOLL_80041EE4 mpColl_80041EE4(c: *mut CollData);
        MPCOLL_SET_ECB_SOURCE_JOBJ mpColl_SetECBSource_JObj(c: *mut CollData, g: G, j0: *mut HSD_JObj, j1: *mut HSD_JObj, j2: *mut HSD_JObj, j3: *mut HSD_JObj, j4: *mut HSD_JObj, j5: *mut HSD_JObj, j6: *mut HSD_JObj, s: f32);
        MPCOLL_SET_LEDGE_SNAP mpColl_SetLedgeSnap(c: *mut CollData, x: f32, y: f32, h: f32);
        MPCOLL_800471F8 mpColl_800471F8(c: *mut CollData) -> bool;
        MPCOLL_800477E0 mpColl_800477E0(c: *mut CollData) -> bool;
        MPCOLL_800473CC mpColl_800473CC(c: *mut CollData) -> bool;
        MPCOLL_80048274 mpColl_80048274(c: *mut CollData) -> bool;
        MPCOLL_80048160 mpColl_80048160(c: *mut CollData) -> bool;
        MPCOLL_80048464 mpColl_80048464(c: *mut CollData) -> bool;
        MPCOLL_80048388 mpColl_80048388(c: *mut CollData) -> bool;
        MPCOLL_80048768 mpColl_80048768(c: *mut CollData) -> bool;
        MPCOLL_80048578 mpColl_80048578(c: *mut CollData) -> bool;
        MPCOLL_SET_FACING_DIR mpCollSetFacingDir(c: *mut CollData, dir: i32);
        MPCOLL_8004730C mpColl_8004730C(c: *mut CollData, b: *mut ftCollisionBox) -> bool;
        MPCOLL_800475F4 mpColl_800475F4(c: *mut CollData, b: *mut ftCollisionBox) -> bool;
        MPCOLL_80047A08 mpColl_80047A08(c: *mut CollData, b: *mut ftCollisionBox) -> bool;
        MPCOLL_8004B108 mpColl_8004B108(c: *mut CollData) -> bool;
        MPCOLL_8004B2DC mpColl_8004B2DC(c: *mut CollData) -> bool;
        MPCOLL_8004B21C mpColl_8004B21C(c: *mut CollData, b: *mut ftCollisionBox) -> bool;
        MPCOLL_8004B3F0 mpColl_8004B3F0(c: *mut CollData, b: *mut ftCollisionBox) -> bool;
        MPCOLL_80047AC8 mpColl_80047AC8(c: *mut CollData, cb: CollCb, g: G) -> bool;
        MPCOLL_80047E14 mpColl_80047E14(c: *mut CollData, cb: CollCb, g: G) -> bool;
        MPCOLL_80047BF4 mpColl_80047BF4(c: *mut CollData, cb: CollCb, g: G) -> bool;
        MPCOLL_80047F40 mpColl_80047F40(c: *mut CollData, cb: CollCb, g: G) -> bool;
        MPCOLL_80047D20 mpColl_80047D20(c: *mut CollData, cb: CollCb, g: G) -> bool;
        MPCOLL_8004806C mpColl_8004806C(c: *mut CollData, cb: CollCb, g: G) -> bool;
        MPCOLL_80048654 mpColl_80048654(c: *mut CollData) -> bool;
        MPCOLL_800474E0 mpColl_800474E0(c: *mut CollData) -> bool;
        MPCOLL_800476B4 mpColl_800476B4(c: *mut CollData, cb: CollCb, g: G) -> bool;
        MPCOLL_800478F4 mpColl_800478F4(c: *mut CollData) -> bool;
        MPCOLL_8004B4B0 mpColl_8004B4B0(c: *mut CollData) -> bool;
        MPCOLL_8004B5C4 mpColl_8004B5C4(c: *mut CollData) -> bool;
        MPCOLL_8004CA6C mpColl_8004CA6C(c: *mut CollData) -> f32;
        MP_COPY_COLL_DATA mpCopyCollData(a: *mut CollData, b: *mut CollData, n: i32);
        MPLIB_80054ED8 mpLib_80054ED8(i: i32) -> bool;
        MP_LINES_CONNECTED mpLinesConnected(a: i32, b: i32) -> bool;
        MP_LINE_GET_FLAGS mpLineGetFlags(i: i32) -> u32;
        MPLIB_80056A1C mpLib_80056A1C(f: u32, o: *mut i32) -> *mut i32;
        MPLIB_80056A8C mpLib_80056A8C(f: u32, o: *mut i32) -> *mut i32;
        MPLIB_80056AFC mpLib_80056AFC(f: u32, o: *mut i32) -> *mut i32;
        MPLIB_80056A54 mpLib_80056A54(f: u32, o: *mut i32) -> i32;
        MPLIB_80056AC4 mpLib_80056AC4(f: u32, o: *mut i32) -> i32;
        MPLIB_80056B34 mpLib_80056B34(f: u32, o: *mut i32) -> i32;
        MP_CHECK_RIGHT_WALL mpCheckRightWall(x0: f32, y0: f32, x1: f32, y1: f32, a: *mut Vec3, b: *mut i32, c: *mut u32, d: *mut Vec3, e: i32, f: i32) -> bool;
        MP_CHECK_LEFT_WALL mpCheckLeftWall(x0: f32, y0: f32, x1: f32, y1: f32, a: *mut Vec3, b: *mut i32, c: *mut u32, d: *mut Vec3, e: i32, f: i32) -> bool;
        FTCO_LANDING_ENTER_BASIC ftCo_Landing_Enter_Basic(g: G);
        FTCO_FALL_ENTER ftCo_Fall_Enter(g: G);
        FTCO_8009EF68 ftCo_8009EF68(g: G) -> bool;
        FTCO_8009A3C8 ftCo_8009A3C8(g: G) -> bool;
        FTCO_8009EDA4 ftCo_8009EDA4(g: G) -> bool;
        FTCO_8009F39C ftCo_8009F39C(g: G);
    }
    extd! {
        FT_80081A00 ft_80081A00(g: G) -> bool = crate::tu::ft_0819::ft_80081a00;
        FTWALLJUMP_8008169C ftWallJump_8008169C(g: G) -> bool = crate::tu::ftwalljump::ftwalljump_8008169c;
        FTCLIFFCOMMON_80081298 ftCliffCommon_80081298(g: G) -> bool = crate::tu::ftcliffcommon::ftcliffcommon_80081298;
        FT_8008A2BC ft_8008A2BC(g: G) = crate::tu::ft_08a1::ft_8008A2BC;
        FTCO_800D0EC8 ftCo_800D0EC8(f: &Fighter) -> f32 = crate::tu::ftchangeparam::ftco_800d0ec8;
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

#[inline(always)]
fn x44(f: &Fighter) -> &'static ftData_x44_t {
    unsafe { &*(*f.ft_data).x44 }
}

/// `ftGetFacingDirInt` (ft/inlines.h).
#[inline(always)]
fn dir_int(f: &Fighter) -> i32 {
    if f.facing_dir < 0.0 { -1 } else { 1 }
}

/// `last_pos = cur_pos; cur_pos = fp->cur_pos;` (the common prologue).
#[inline(always)]
fn begin(f: &mut Fighter) -> *mut CollData {
    f.coll_data.last_pos = f.coll_data.cur_pos;
    f.coll_data.cur_pos = f.cur_pos;
    &mut f.coll_data
}

#[inline(always)]
fn end(g: G) {
    let f = fp(g);
    f.cur_pos = f.coll_data.cur_pos;
}

/// `ft_80081B38`: initialise the ECB/collision data from `ft_data->x44`.
pub fn ft_80081b38(g: G) {
    melee_core::ct!("ft_80081b38");
    let f = fp(g);
    f.ecb_lock = 0;
    f.coll_data.cur_pos = f.cur_pos;
    f.coll_data.last_pos = f.coll_data.cur_pos;
    let coll: *mut CollData = &mut f.coll_data;
    ext::mpColl_80041EE4(coll);
    let f = fp(g);
    f.coll_data.x34_flags.set_b1234(1);
    let b = f.parts;
    let t = x44(f);
    let j = |i: i16| unsafe { (*b.offset(i as isize)).joint };
    let s = t.unkC * f.x34_scale.y;
    ext::mpColl_SetECBSource_JObj(coll, g, unsafe { (*b).joint }, j(t.unk0), j(t.unk2), j(t.unk4), j(t.unk6), j(t.unk8), j(t.unkA), s);
    let sc = fp(g).x34_scale.y;
    ext::mpColl_SetLedgeSnap(coll, t.ledge_snap_x * sc, t.ledge_snap_y * sc, t.ledge_snap_height * sc);
    let f = fp(g);
    f.coll_data.x50 = f.co_attrs.weight;
    let v = 10.0f32 * f.x34_scale.y;
    if f.coll_data.ecb_source.kind == ECBSource_JObj {
        f.coll_data.ecb_source.x128 = v;
        f.coll_data.ecb_source.x12C = v;
    }
}

/// `ft_80081C88`: rescale the ECB source / ledge snap for `scl_y`.
pub fn ft_80081c88(g: G, scl_y: f32) {
    melee_core::ct!("ft_80081c88");
    let f = fp(g);
    let t = x44(f);
    let tmp = t.unkC * scl_y;
    if f.coll_data.ecb_source.kind == ECBSource_JObj {
        f.coll_data.ecb_source.x124 = tmp;
    }
    let h = t.ledge_snap_height * scl_y;
    let y = t.ledge_snap_y * scl_y;
    let x = t.ledge_snap_x * scl_y;
    if f.coll_data.ecb_source.kind == ECBSource_JObj {
        f.coll_data.ledge_snap_x = x;
        f.coll_data.ledge_snap_y = y;
        f.coll_data.ledge_snap_height = h;
    }
    f.coll_data.x50 = f.co_attrs.weight;
    let v = 10.0f32 * f.x34_scale.y;
    if f.coll_data.ecb_source.kind == ECBSource_JObj {
        f.coll_data.ecb_source.x128 = v;
        f.coll_data.ecb_source.x12C = v;
    }
}

/// `ft_80081D0C`: ground collision; `GA_Air` when the fighter left the ground.
pub fn ft_80081d0c(g: G) -> GroundOrAir {
    melee_core::ct!("ft_80081d0c");
    let c = begin(fp(g));
    let r = ext::mpColl_800471F8(c);
    end(g);
    if ext::ft_80081A00(g) {
        return GA_Ground;
    }
    if r { GA_Air } else { GA_Ground }
}

/// Shared body of ft_80081DD4 / ft_80081F2C / ft_80082084.
fn air_coll(g: G, sdi: fn(*mut CollData) -> bool, cool: fn(*mut CollData) -> bool, snap: fn(*mut CollData) -> bool) -> bool {
    let f = fp(g);
    let c = begin(f);
    let r = if f.allow_sdi() != 0 {
        sdi(c)
    } else if f.x2064_ledgeCooldown != 0 || f.x2224_b2() != 0 {
        cool(c)
    } else {
        let t = x44(f);
        let tmp = t.ledge_snap_height * f.x34_scale.y * crate::hooks::common_data().x1CC;
        if f.coll_data.ecb_source.kind == ECBSource_JObj {
            f.coll_data.ledge_snap_height = tmp;
        }
        let r = snap(c);
        let f = fp(g);
        let tmp = t.ledge_snap_height * f.x34_scale.y;
        if f.coll_data.ecb_source.kind == ECBSource_JObj {
            f.coll_data.ledge_snap_height = tmp;
        }
        r
    };
    end(g);
    if ext::ft_80081A00(g) {
        return false;
    }
    r
}

/// `ft_80081DD4`.
pub fn ft_80081dd4(g: G) -> bool {
    melee_core::ct!("ft_80081dd4");
    air_coll(g, ext::mpColl_800477E0, ext::mpColl_800471F8, ext::mpColl_800473CC)
}
/// `ft_80081F2C`.
pub fn ft_80081f2c(g: G) -> bool {
    melee_core::ct!("ft_80081f2c");
    air_coll(g, ext::mpColl_80048274, ext::mpColl_80048160, ext::mpColl_80048464)
}
/// `ft_80082084`.
pub fn ft_80082084(g: G) -> bool {
    melee_core::ct!("ft_80082084");
    air_coll(g, ext::mpColl_80048388, ext::mpColl_80048768, ext::mpColl_80048578)
}

/// `begin; r = op(coll); end; ft_80081A00 ? false : r`.
fn simple(g: G, op: impl FnOnce(*mut CollData) -> bool) -> bool {
    let c = begin(fp(g));
    let r = op(c);
    end(g);
    if ext::ft_80081A00(g) {
        return false;
    }
    r
}

/// `ft_800821DC`.
pub fn ft_800821dc(g: G) -> bool {
    melee_core::ct!("ft_800821dc");
    simple(g, ext::mpColl_80048160)
}

/// `ft_CheckGroundAndLedge`.
pub fn ft_check_ground_and_ledge(g: G, dir: i32) -> bool {
    melee_core::ct!("ft_check_ground_and_ledge");
    let f = fp(g);
    let cool = f.x2064_ledgeCooldown != 0 || f.x2224_b2() != 0;
    simple(g, |c| {
        if cool {
            ext::mpColl_800471F8(c)
        } else {
            ext::mpCollSetFacingDir(c, dir);
            ext::mpColl_800473CC(c)
        }
    })
}

/// `ft_8008239C`.
pub fn ft_8008239c(g: G, dir: i32, height_attributes: *mut ftCollisionBox) -> bool {
    melee_core::ct!("ft_8008239c");
    let f = fp(g);
    let mut bx = ftCollisionBox::zeroed();
    let fd = f.facing_dir;
    let cool = f.x2064_ledgeCooldown != 0 || f.x2224_b2() != 0;
    simple(g, |c| {
        let cb = ft_80082838(&mut bx, height_attributes, fd);
        if cool {
            ext::mpColl_8004730C(c, cb)
        } else {
            ext::mpCollSetFacingDir(c, dir);
            ext::mpColl_800475F4(c, cb)
        }
    })
}

/// `ft_800824A0` (also `ft_80083E64_inline`).
pub fn ft_800824a0(g: G, ecb: *mut ftCollisionBox) -> bool {
    melee_core::ct!("ft_800824a0");
    let fd = fp(g).facing_dir;
    let mut bx = ftCollisionBox::zeroed();
    simple(g, |c| ext::mpColl_8004730C(c, ft_80082838(&mut bx, ecb, fd)))
}

/// `begin; op(coll); end; ft_80081A00 ? false : env_flags & FloorMask`.
fn floor_flag(g: G, op: impl FnOnce(*mut CollData)) -> bool {
    let c = begin(fp(g));
    op(c);
    end(g);
    if ext::ft_80081A00(g) {
        return false;
    }
    fp(g).coll_data.env_flags & COLLIDE_FLOOR_MASK != 0
}

/// `ft_80082578`.
pub fn ft_80082578(g: G) -> bool {
    melee_core::ct!("ft_80082578");
    floor_flag(g, |c| {
        ext::mpColl_800477E0(c);
    })
}

/// `ft_80082638`.
pub fn ft_80082638(g: G, arg1: *mut ftCollisionBox) -> bool {
    melee_core::ct!("ft_80082638");
    let fd = fp(g).facing_dir;
    let mut bx = ftCollisionBox::zeroed();
    floor_flag(g, |c| {
        ext::mpColl_80047A08(c, ft_80082838(&mut bx, arg1, fd));
    })
}

/// `ft_80082708` (EnvironmentCollision_AllowGroundToAir): `GA_Air` when it fell off a ledge.
pub fn ft_80082708(g: G) -> GroundOrAir {
    melee_core::ct!("ft_80082708");
    let c = begin(fp(g));
    let r = ext::mpColl_8004B108(c);
    end(g);
    if r { GA_Air } else { GA_Ground }
}

/// `ft_800827A0`.
pub fn ft_800827a0(g: G) -> bool {
    melee_core::ct!("ft_800827a0");
    let c = begin(fp(g));
    let r = ext::mpColl_8004B2DC(c);
    end(g);
    r
}

/// `ft_80082838`: mirror `arg1` into `arg0` when facing left, else return `arg1`.
pub fn ft_80082838(arg0: *mut ftCollisionBox, arg1: *mut ftCollisionBox, arg2: f32) -> *mut ftCollisionBox {
    melee_core::ct!("ft_80082838");
    if arg2 < 0.0 {
        unsafe {
            let s = *arg1;
            let d = &mut *arg0;
            d.top = s.top;
            d.bottom = s.bottom;
            d.left.y = s.left.y;
            d.right.y = s.right.y;
            d.right.x = -s.left.x;
            d.left.x = -s.right.x;
        }
        return arg0;
    }
    arg1
}

/// `begin; r = op(coll, mirrored box); end; r` (no ft_80081A00).
fn box_op(g: G, b: *mut ftCollisionBox, op: fn(*mut CollData, *mut ftCollisionBox) -> bool) -> bool {
    let f = fp(g);
    let c = begin(f);
    let mut sp = ftCollisionBox::zeroed();
    let r = op(c, ft_80082838(&mut sp, b, f.facing_dir));
    end(g);
    r
}

/// `ft_80082888`.
pub fn ft_80082888(g: G, arg1: *mut ftCollisionBox) -> bool {
    melee_core::ct!("ft_80082888");
    box_op(g, arg1, ext::mpColl_8004B21C)
}

/// `ft_80082978`.
pub fn ft_80082978(g: G, arg1: *mut ftCollisionBox) -> bool {
    melee_core::ct!("ft_80082978");
    box_op(g, arg1, ext::mpColl_8004B3F0)
}

/// `ft_80082A68`: snap to the ground within +-10 of `cur_pos`.
pub fn ft_80082a68(g: G) -> bool {
    melee_core::ct!("ft_80082a68");
    let f = fp(g);
    f.coll_data.cur_pos = f.cur_pos;
    f.coll_data.last_pos = f.coll_data.cur_pos;
    f.coll_data.last_pos.y += 10.0;
    f.coll_data.cur_pos.y -= 10.0;
    let hit = ext::mpColl_800471F8(&mut f.coll_data);
    if hit {
        end(g);
        true
    } else {
        false
    }
}

/// `ft_80082B1C`: the landing callback (heavy landing vs. basic landing).
pub fn ft_80082b1c(g: G) {
    melee_core::ct!("ft_80082b1c");
    let f = fp(g);
    if f.self_vel.y > ext::ftCo_800D0EC8(f) {
        ext::ft_8008A2BC(g);
    } else {
        ext::ftCo_Landing_Enter_Basic(g);
    }
}

/// `ftCo_AirCatchHit_Coll`.
pub fn ftco_air_catch_hit_coll(g: G) {
    melee_core::ct!("ftco_air_catch_hit_coll");
    if simple(g, ext::mpColl_800471F8) {
        ft_80082b1c(g);
    }
}

/// `ft_80082C74`.
pub fn ft_80082c74(g: G, cb: Ev) {
    melee_core::ct!("ft_80082c74");
    if ft_80081d0c(g) != GA_Ground {
        cb(g);
    }
}

/// `ft_80082D40` (`arg1` unused).
pub fn ft_80082d40(g: G, _arg1: f32) {
    melee_core::ct!("ft_80082d40");
    if simple(g, ext::mpColl_80048768) {
        ft_80082b1c(g);
    }
}

/// `ft_80082E3C`: another fighter standing on (x221D_b7) a floor joined to the ledge we touch.
pub fn ft_80082e3c(g: G) -> G {
    melee_core::ct!("ft_80082e3c");
    let mut cur = crate::hooks::plink_head(HSD_GOBJ_PLINK_FIGHTER);
    while !cur.is_null() {
        if cur != g {
            let c = fp(cur);
            if c.x221D_b7() != 0 {
                let f = fp(g);
                let floor_id = unsafe { c.mv.co.common.x0 };
                if c.facing_dir > 0.0
                    && f.coll_data.env_flags & Collide_LeftLedgeGrab != 0
                    && ext::mpLinesConnected(floor_id, f.coll_data.ledge_id_left)
                {
                    return cur;
                }
                let f = fp(g);
                if c.facing_dir < 0.0
                    && f.coll_data.env_flags & Collide_RightLedgeGrab != 0
                    && ext::mpLinesConnected(floor_id, f.coll_data.ledge_id_right)
                {
                    return cur;
                }
            }
        }
        cur = unsafe { (*cur).next };
    }
    core::ptr::null_mut()
}

/// `ftGetFacingDirInt2` (static inline).
#[inline]
pub fn ft_get_facing_dir_int2(g: G) -> i32 {
    melee_core::ct!("ft_get_facing_dir_int2");
    dir_int(fp(g))
}

/// `ft_80082F28`.
pub fn ft_80082f28(g: G) {
    melee_core::ct!("ft_80082f28");
    if ft_check_ground_and_ledge(g, ft_get_facing_dir_int2(g)) {
        ft_80082b1c(g);
        return;
    }
    if ext::ftWallJump_8008169C(g) {
        return;
    }
    ext::ftCliffCommon_80081298(g);
}

/// The `ft_80083090_inline` / `ft_80083318_inline` / `ft_80083464_inline` shape: on ledge
/// cooldown `cool`, else set the facing dir and `snap`.
fn cb_coll(g: G, arg1: CollCb, cool: fn(*mut CollData, CollCb, G) -> bool, snap: fn(*mut CollData, CollCb, G) -> bool) -> bool {
    let f = fp(g);
    let dir = dir_int(f);
    let cd = f.x2064_ledgeCooldown != 0 || f.x2224_b2() != 0;
    simple(g, |c| {
        if cd {
            cool(c, arg1, g)
        } else {
            ext::mpCollSetFacingDir(c, dir);
            snap(c, arg1, g)
        }
    })
}

/// `ft_80083090_inline`.
fn ft_80083090_inline(g: G, arg1: CollCb) -> bool {
    cb_coll(g, arg1, ext::mpColl_80047AC8, ext::mpColl_80047E14)
}

/// `ft_80083090`.
pub fn ft_80083090(g: G, arg1: CollCb, cb: Ev) {
    melee_core::ct!("ft_80083090");
    if ft_80083090_inline(g, arg1) {
        cb(g);
        return;
    }
    ext::ftCliffCommon_80081298(g);
}

/// Callback on a hit, else walljump, else cliff catch; true when any of them took over.
fn or_wall_cliff(g: G, hit: bool, cb: Ev) -> bool {
    if hit {
        cb(g);
        return true;
    }
    if ext::ftWallJump_8008169C(g) {
        return true;
    }
    ext::ftCliffCommon_80081298(g)
}

/// `ft_800831CC`.
pub fn ft_800831cc(g: G, arg1: CollCb, cb: Ev) {
    melee_core::ct!("ft_800831cc");
    let h = ft_80083090_inline(g, arg1);
    or_wall_cliff(g, h, cb);
}

/// `ft_80083318`.
pub fn ft_80083318(g: G, arg1: CollCb, cb: Ev) {
    melee_core::ct!("ft_80083318");
    let h = cb_coll(g, arg1, ext::mpColl_80047BF4, ext::mpColl_80047F40);
    or_wall_cliff(g, h, cb);
}

/// `ft_80083464`.
pub fn ft_80083464(g: G, arg1: CollCb, cb: Ev) {
    melee_core::ct!("ft_80083464");
    let h = cb_coll(g, arg1, ext::mpColl_80047D20, ext::mpColl_8004806C);
    or_wall_cliff(g, h, cb);
}

/// `ft_800835B0`.
pub fn ft_800835b0(g: G, arg1: CollCb, cb: Ev) {
    melee_core::ct!("ft_800835b0");
    let h = ft_80083090_inline(g, arg1);
    if or_wall_cliff(g, h, cb) {
        return;
    }
    ext::ftCo_8009EF68(g);
}

/// On ledge cooldown `cool`, else set the facing dir and `snap` (no callback argument):
/// `ft_8008370C_inline` / `ft_80083910_inline`.
fn dir_coll(g: G, cool: fn(*mut CollData) -> bool, snap: fn(*mut CollData) -> bool) -> bool {
    let f = fp(g);
    let dir = dir_int(f);
    let cd = f.x2064_ledgeCooldown != 0 || f.x2224_b2() != 0;
    simple(g, |c| {
        if cd {
            cool(c)
        } else {
            ext::mpCollSetFacingDir(c, dir);
            snap(c)
        }
    })
}

/// `ft_8008370C`.
pub fn ft_8008370c(g: G, cb: Ev) {
    melee_core::ct!("ft_8008370c");
    let h = dir_coll(g, ext::mpColl_800471F8, ext::mpColl_800473CC);
    or_wall_cliff(g, h, cb);
}

/// `ft_80083844` (with `ft_80083844_inline`).
pub fn ft_80083844(g: G, cb: Ev) {
    melee_core::ct!("ft_80083844");
    if simple(g, ext::mpColl_80048654) {
        cb(g);
    }
}

/// `ft_80083910`.
pub fn ft_80083910(g: G, cb: Ev) {
    melee_core::ct!("ft_80083910");
    let h = dir_coll(g, ext::mpColl_80048654, ext::mpColl_800474E0);
    or_wall_cliff(g, h, cb);
}

/// `ft_80083A48`.
pub fn ft_80083a48(g: G, cb: Ev) {
    melee_core::ct!("ft_80083a48");
    let h = ft_check_ground_and_ledge(g, 0);
    or_wall_cliff(g, h, cb);
}

/// `ft_80083B68`.
pub fn ft_80083b68(g: G) {
    melee_core::ct!("ft_80083b68");
    ft_80082578(g);
}

/// `ft_80083C00`.
pub fn ft_80083c00(g: G, cb: Ev) {
    melee_core::ct!("ft_80083c00");
    if ft_80082578(g) {
        cb(g);
    }
}

/// `ft_80083CE4` (with `ft_80083CE4_inline`).
pub fn ft_80083ce4(g: G, cb1: CollCb, cb2: Ev) {
    melee_core::ct!("ft_80083ce4");
    if floor_flag(g, |c| {
        ext::mpColl_800476B4(c, cb1, g);
    }) {
        cb2(g);
    }
}

/// `ft_80083DCC`.
pub fn ft_80083dcc(g: G) {
    melee_core::ct!("ft_80083dcc");
    let c = begin(fp(g));
    ext::mpColl_800478F4(c);
    end(g);
    ext::ft_80081A00(g);
}

/// `ft_80083E64`.
pub fn ft_80083e64(g: G, ecb: *mut ftCollisionBox, cb: Ev) {
    melee_core::ct!("ft_80083e64");
    if ft_800824a0(g, ecb) {
        cb(g);
    }
}

/// `ft_80083F88`.
pub fn ft_80083f88(g: G) {
    melee_core::ct!("ft_80083f88");
    if ft_80082708(g) == GA_Ground {
        ext::ftCo_Fall_Enter(g);
    }
}

/// `ft_8008403C`.
pub fn ft_8008403c(g: G, cb: Ev) {
    melee_core::ct!("ft_8008403c");
    if ft_80082708(g) == GA_Ground {
        cb(g);
    }
}

/// `ft_80084104`.
pub fn ft_80084104(g: G) {
    melee_core::ct!("ft_80084104");
    if !ft_800827a0(g) {
        ext::ftCo_Fall_Enter(g);
    }
}

/// `ft_800841B8`.
pub fn ft_800841b8(g: G, cb: Ev) {
    melee_core::ct!("ft_800841b8");
    if !ft_800827a0(g) {
        cb(g);
    }
}

/// Stick-aware edge collision (`ft_80084280_inline` / `ft_800843FC_inline`).
fn stick_coll(g: G, op: fn(*mut CollData) -> bool) -> bool {
    let f = fp(g);
    let c = begin(f);
    f.coll_data.lstick_x = f.input.lstick[0].x;
    ext::mpCollSetFacingDir(c, dir_int(fp(g)));
    let r = op(c);
    end(g);
    r
}

/// `ft_80084280`.
pub fn ft_80084280(g: G) {
    melee_core::ct!("ft_80084280");
    let f = fp(g);
    let n = f.xF8_playerNudgeVel.x;
    let r = if n != 0.0 && n * f.facing_dir < 0.0 {
        ft_800827a0(g)
    } else {
        stick_coll(g, ext::mpColl_8004B4B0)
    };
    if r || ext::ftCo_8009A3C8(g) {
        return;
    }
    ext::ftCo_Fall_Enter(g);
}

/// `ft_800843FC`.
pub fn ft_800843fc(g: G) {
    melee_core::ct!("ft_800843fc");
    if stick_coll(g, ext::mpColl_8004B5C4) || ext::ftCo_8009A3C8(g) {
        return;
    }
    ext::ftCo_Fall_Enter(g);
}

/// `ft_800844EC`.
pub fn ft_800844ec(g: G) {
    melee_core::ct!("ft_800844ec");
    if ft_80082708(g) != GA_Ground {
        ext::ftCo_8009EDA4(g);
    } else {
        ext::ftCo_Fall_Enter(g);
    }
}

#[inline]
fn ledge_slip(f: &Fighter) -> bool {
    (f.facing_dir < 0.0 && f.coll_data.env_flags & Collide_RightLedgeSlip != 0)
        || (f.facing_dir > 0.0 && f.coll_data.env_flags & Collide_LeftLedgeSlip != 0)
}

/// `ft_800845B4`.
pub fn ft_800845b4(g: G) {
    melee_core::ct!("ft_800845b4");
    if ft_80082708(g) == GA_Ground {
        if ledge_slip(fp(g)) {
            ext::ftCo_8009F39C(g);
            return;
        }
        ext::ftCo_Fall_Enter(g);
    }
}

/// `ft_800846B0`.
pub fn ft_800846b0(g: G, b: *mut ftCollisionBox, cb: Ev) {
    melee_core::ct!("ft_800846b0");
    if !box_op(g, b, ext::mpColl_8004B21C) {
        cb(g);
    }
}

/// `ft_800847D0`.
pub fn ft_800847d0(g: G, b: *mut ftCollisionBox) {
    melee_core::ct!("ft_800847d0");
    if !box_op(g, b, ext::mpColl_8004B3F0) {
        ext::ftCo_Fall_Enter(g);
    }
}

/// `ft_800848DC`.
pub fn ft_800848dc(g: G, cb: Ev) {
    melee_core::ct!("ft_800848dc");
    if ft_80082708(g) == GA_Ground {
        if ledge_slip(fp(g)) {
            ext::ftCo_8009F39C(g);
            return;
        }
        cb(g);
    }
}

/// `ft_800849EC`.
pub fn ft_800849ec(fp1: &mut Fighter, fp2: &mut Fighter) {
    melee_core::ct!("ft_800849ec");
    ext::mpCopyCollData(&mut fp1.coll_data, &mut fp2.coll_data, 2);
}

/// `ft_80084A18`.
pub fn ft_80084a18(g: G) -> bool {
    melee_core::ct!("ft_80084a18");
    ext::mpLib_80054ED8(fp(g).coll_data.floor.index)
}

/// `ft_GetGroundFrictionMultiplier`: 1 for the Ice Climbers, else the floor's.
pub fn ft_get_ground_friction_multiplier(f: &mut Fighter) -> f32 {
    melee_core::ct!("ft_get_ground_friction_multiplier");
    if f.kind == Ft_Kind_Popo || f.kind == Ft_Kind_Nana {
        return 1.0;
    }
    ext::mpColl_8004CA6C(&mut f.coll_data)
}

/// `ft_80084A80`: floor-line lookups for `arg0` in 0..=2 (the C reads an uninitialised pointer
/// otherwise; this panics).
pub fn ft_80084a80(arg0: i32, g: G, arg1: *mut i32, arg2: *mut i32, arg3: *mut i32) -> bool {
    melee_core::ct!("ft_80084a80");
    let f = fp(g);
    if f.ground_or_air != GA_Ground {
        return false;
    }
    let idx = f.coll_data.floor.index;
    if idx == -1 {
        return false;
    }
    let fl = ext::mpLineGetFlags(idx);
    let r25 = match arg0 {
        0 => ext::mpLib_80056A1C(fl, arg2),
        1 => ext::mpLib_80056A8C(fl, arg2),
        2 => ext::mpLib_80056AFC(fl, arg2),
        _ => panic!("ft_80084A80: bad selector {arg0}"),
    };
    unsafe {
        if *r25 != -1 {
            *arg1 = *r25;
            if arg0 == 2 {
                *arg1.add(1) = *r25.add(1);
                *arg1.add(2) = *r25.add(2);
                *arg1.add(3) = *r25.add(3);
            }
        } else {
            *arg1 = -1;
            *arg2 = 1;
        }
    }
    let mut sp24 = 0i32;
    let r31 = match arg0 {
        0 => ext::mpLib_80056A54(fl, &mut sp24),
        1 => ext::mpLib_80056AC4(fl, &mut sp24),
        _ => ext::mpLib_80056B34(fl, &mut sp24),
    };
    unsafe { *arg3 = if r31 != -1 { r31 } else { -1 } };
    true
}

/// `ft_80084BFC`.
pub fn ft_80084bfc(g: G, a: *mut i32, b: *mut i32, c: *mut i32) -> bool {
    melee_core::ct!("ft_80084bfc");
    ft_80084a80(0, g, a, b, c)
}
/// `ft_80084C38`.
pub fn ft_80084c38(g: G, a: *mut i32, b: *mut i32, c: *mut i32) -> bool {
    melee_core::ct!("ft_80084c38");
    ft_80084a80(1, g, a, b, c)
}
/// `ft_80084C74`.
pub fn ft_80084c74(g: G, a: *mut i32, b: *mut i32, c: *mut i32) -> bool {
    melee_core::ct!("ft_80084c74");
    ft_80084a80(2, g, a, b, c)
}

/// `ft_80084CE4`: a wall between the two fighters' ECB centres. 2 fused sites.
pub fn ft_80084ce4(attacker: &Fighter, victim: &Fighter) -> bool {
    melee_core::ct!("ft_80084ce4");
    let ay = fmadds(attacker.coll_data.ecb.top.y + attacker.coll_data.ecb.bottom.y, 0.5, attacker.cur_pos.y);
    let ax = 0.0f32 + attacker.cur_pos.x;
    let vy = fmadds(victim.coll_data.ecb.top.y + victim.coll_data.ecb.bottom.y, 0.5, victim.cur_pos.y);
    let vx = 0.0f32 + victim.cur_pos.x;
    let (n3, ni, nu) = (core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    if ax > vx {
        ext::mpCheckRightWall(ax, ay, vx, vy, n3, ni, nu, n3, -1, -1)
    } else {
        ext::mpCheckLeftWall(ax, ay, vx, vy, n3, ni, nu, n3, -1, -1)
    }
}
