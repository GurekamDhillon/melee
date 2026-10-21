//! src/melee/ft/ft_0C31.c: the match-start entry sequence (Entry -> EntryStart -> EntryEnd):
//! the squashed root scale, the trophy-stand accessory that grows and shrinks under the
//! fighter, and the sub-fighter (Nana) following the leader's height; plus the out-of-line
//! `HSD_JObjSetMtxDirty` this TU defines.
//!
//! Fused sites (`tools/rust-port/fma_sites.py src/melee/ft/ft_0C31.c`): one f32 each in
//! ftCo_800C6408, fn_800C69F4, fn_800C6F34 (`facing * x - pos.x` -> `fmsubs`) and
//! ftCo_EntryStart_Phys (`t * (a - b) + b` -> `fmadds`). ftCo_800C6408's `1.5707963267948966 *`
//! and `1.497345 *` are f64 products rounded once to f32.
//! Checked against the C (whole TU included verbatim, shipping contraction) by
//! tests/tu_ft_0c31.rs.
#![allow(non_snake_case)]

use crate::types::*;
use melee_core::fp::{fmadds, fmsubs};
use melee_types::inline as ji;

pub type G = *mut HSD_GObj;

/// Callees outside this TU with no port yet: settable, panic while unset.
pub mod ext {
    use crate::types::*;
    type G = *mut HSD_GObj;
    macro_rules! ext_fns {
        ($($hook:ident $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported (ft_0C31 ext)")),
                }
            }
        )*};
    }
    ext_fns! {
        FIGHTER_CHANGE_MOTION_STATE Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, frame: f32, rate: f32, blend: f32, other: G);
        HSD_JOBJ_SET_MTX_DIRTY_SUB HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj);
        FTCOMMON_SET_ACCESSORY ftCommon_SetAccessory(fp: &mut Fighter, joint: *mut HSD_Joint);
        FIGHTER_804D6514 Fighter_804D6514() -> *mut HSD_Joint;
        EFASYNC_SPAWN efAsync_Spawn(g: G, queue: *mut core::ffi::c_void, kind: u32, id: u32, j: *mut HSD_JObj, v: *mut Vec3);
        LBAUDIOAX_80024304 lbAudioAx_80024304(id: i32) -> i32;
        FT_80083E64 ft_80083E64(g: G, ecb: &mut ftCollisionBox, cb: fn(G));
        FT_800846B0 ft_800846B0(g: G, ecb: &mut ftCollisionBox, cb: fn(G));
        FTCOLL_8007B760 ftColl_8007B760(g: G, n: i32);
    }
}

/// The macro / out-of-line dirty marks of the jobj.h inlines this TU uses.
struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        HSD_JObjSetMtxDirty(j);
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn jobj<'a>(g: G) -> &'a mut HSD_JObj {
    unsafe { &mut *((*g).hsd_obj as *mut HSD_JObj) }
}
#[inline(always)]
fn entry(f: &mut Fighter) -> &mut ftCommon_MotionVars_entry {
    unsafe { &mut f.mv.co.entry }
}
#[inline(always)]
fn cd() -> &'static ftCommonData {
    crate::hooks::common_data()
}
fn set_scale(j: *mut HSD_JObj, s: &Vec3) {
    ji::HSD_JObjSetScale(unsafe { &mut *j }, s, &mut Dirty);
}
fn leader(f: &Fighter, index: i32) -> G {
    crate::player::get_entity_at_index(f.player_id as i32, index)
}

/// `ftCo_800C61B0` (Entry_Enter): wait `Player_GetUnk4C` frames, invisible, root squashed to
/// `x6C4` in Y.
pub fn ftCo_800C61B0(g: G) {
    let f = fp(g);
    let timer = crate::player::get_unk4c(f.player_id as i32);
    let e = entry(f);
    e.timer = timer;
    ji::HSD_JObjGetScale(jobj(g), &mut e.x8);
    e.x14.x = e.x8.x;
    e.x14.y = cd().x6C4;
    e.x14.z = e.x8.z;
    let s = e.x14;
    set_scale(jobj(g), &s);
    let y = f.cur_pos.y;
    f.mv.co.common.x4.x = y;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_Entry as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_invisible(1);
    f.set_x2219_b1(1);
    f.set_x221E_b1(1);
    f.set_x221E_b2(1);
    f.set_x221F_b1(1);
    let mut bx = entry(f).x2C;
    crate::ft084e::ft_80084cb0(f, &mut bx);
    entry(fp(g)).x2C = bx;
}

/// `ftCo_Entry_Anim`.
pub fn ftCo_Entry_Anim(g: G) {
    if entry(fp(g)).timer == 0 {
        ftCo_800C6408(g);
    }
    let e = entry(fp(g));
    e.timer = e.timer.wrapping_sub(1);
}
/// `ftCo_Entry_IASA`: empty.
pub fn ftCo_Entry_IASA(_g: G) {}
/// `ftCo_Entry_Phys`: empty.
pub fn ftCo_Entry_Phys(_g: G) {}
/// `fn_800C63BC` (static): `ftCommon_8007D7FC`.
pub fn fn_800C63BC(g: G) {
    crate::ftcommon3::ftcommon_8007d7fc(fp(g));
}
/// `fn_800C63E0` (static): `ftCommon_8007D5D4`.
pub fn fn_800C63E0(g: G) {
    crate::ftcommon2::ftcommon_8007d5d4(fp(g));
}
/// `ftCo_Entry_Coll`: empty.
pub fn ftCo_Entry_Coll(_g: G) {}

/// The accessory's position: `cur_pos` shifted back by `ftCommon_800804EC` along the facing.
fn accessory_pos(f: &Fighter) -> Vec3 {
    Vec3 { x: -fmsubs(f.facing_dir, crate::ftcommon2::ftcommon_800804ec(f), f.cur_pos.x), y: f.cur_pos.y, z: f.cur_pos.z }
}

/// `ftCo_800C6408` (EntryStart_Enter): the trophy stand appears, the entry beam and sfx.
pub fn ftCo_800C6408(g: G) {
    let f = fp(g);
    entry(f).timer = cd().x6BC;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_EntryStart as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b1(1);
    if f.is_sub_fighter() == 0 {
        ext::ftCommon_SetAccessory(f, ext::Fighter_804D6514());
        let f = fp(g);
        let t = f.x34_scale.y * f.co_attrs.trophy_scale;
        let sp48 = Vec3 { x: t, y: t, z: t };
        entry(f).x24 = t;
        set_scale(f.x20A0_accessory, &sp48);
        let sp2c = Quaternion { x: 0.0, y: (1.5707963267948966f64 * f.facing_dir as f64) as f32, z: 0.0, w: 0.0 };
        ji::HSD_JObjSetRotation(unsafe { &mut *f.x20A0_accessory }, &sp2c, &mut Dirty);
        let t2 = (1.497345f64 * sp48.y as f64) as f32;
        entry(f).x28 = t2;
        entry(f).x20 = t2;
        let f2 = fp(g);
        let sp20 = accessory_pos(f2);
        ji::HSD_JObjSetTranslate(unsafe { &mut *f2.x20A0_accessory }, &sp20, &mut Dirty);
    }
    let f = fp(g);
    f.accessory1_cb = Some(fn_800C69F4_cb);
    let q = &mut f.x60C as *mut *mut core::ffi::c_void as *mut core::ffi::c_void;
    let x8 = &mut entry(f).x8 as *mut Vec3;
    ext::efAsync_Spawn(g, q, 3, 0x43E, unsafe { (*g).hsd_obj as *mut HSD_JObj }, x8);
    ext::lbAudioAx_80024304(0x8B);
    crate::hooks::ftco_800bffd0(fp(g), 0x75, 0);
}

/// `ftCo_EntryStart_Anim`.
pub fn ftCo_EntryStart_Anim(g: G) {
    let e = entry(fp(g));
    e.timer = e.timer.wrapping_sub(1);
    if e.timer == 0 {
        ftCo_800C6B6C(g);
    }
}
/// `ftCo_EntryStart_IASA`: empty.
pub fn ftCo_EntryStart_IASA(_g: G) {}

/// `ftCo_EntryStart_Phys`: the root unsquashes and the stand rises over `x6BC` frames.
pub fn ftCo_EntryStart_Phys(g: G) {
    let f = fp(g);
    let r6 = cd().x6BC;
    let f4 = cd().x6C4;
    let e = entry(f);
    let t = r6.wrapping_sub(e.timer) as f32 / r6 as f32;
    e.x14.y = fmadds(t, e.x8.y - f4, f4);
    let s = e.x14;
    set_scale(jobj(g), &s);
    let f = fp(g);
    if f.is_sub_fighter() == 0 {
        let a = entry(f).x24;
        set_scale(f.x20A0_accessory, &Vec3 { x: a, y: a * t, z: a });
        let e = entry(f);
        e.x28 = e.x20 * t;
        f.cur_pos.y = entry(f).x4 + entry(f).x28;
    } else {
        let l = leader(f, 0);
        f.cur_pos.y = fp(l).cur_pos.y;
    }
}

fn coll(g: G) {
    let f = fp(g);
    let x28 = if f.is_sub_fighter() == 0 { entry(f).x28 } else { entry(fp(leader(f, 0))).x28 };
    let f = fp(g);
    entry(f).x2C.bottom = -x28;
    let mut bx = entry(f).x2C;
    if f.ground_or_air == GA_Air {
        ext::ft_80083E64(g, &mut bx, fn_800C63BC);
    } else {
        ext::ft_800846B0(g, &mut bx, fn_800C63E0);
    }
    entry(fp(g)).x2C = bx;
}

/// `ftCo_EntryStart_Coll`: the ECB bottom follows the stand (the leader's for Nana).
pub fn ftCo_EntryStart_Coll(g: G) {
    coll(g);
}

/// The two accessory callbacks: stand at least as high as Nana, the stand follows.
fn accessory_follow(g: G, out_of_line: bool) {
    let f = fp(g);
    if f.is_sub_fighter() == 0 {
        let sub = leader(f, 1);
        if !sub.is_null() {
            let s = fp(sub);
            if s.x221F_b3() == 0 && s.cur_pos.y > f.cur_pos.y {
                f.cur_pos.y = s.cur_pos.y;
            }
        }
        let f = fp(g);
        let sp20 = accessory_pos(f);
        let _ = out_of_line;
        ji::HSD_JObjSetTranslateWithMtxDirtyOutOfLine(unsafe { &mut *f.x20A0_accessory }, &sp20, &mut Dirty);
    } else {
        let l = leader(f, 0);
        f.cur_pos.y = fp(l).cur_pos.y;
    }
}

/// `fn_800C69F4` (EntryStart's accessory callback).
pub fn fn_800C69F4(g: G) {
    accessory_follow(g, true);
}
/// `fn_800C69F4` as the C `HSD_GObjEvent` stored in `accessory1_cb`.
pub unsafe extern "C" fn fn_800C69F4_cb(g: *mut HSD_GObj) {
    fn_800C69F4(g)
}

/// `HSD_JObjSetMtxDirty` (out-of-line, defined in this TU).
pub fn HSD_JObjSetMtxDirty(j: *mut HSD_JObj) {
    if j.is_null() || ji::HSD_JObjMtxIsDirty(unsafe { &*j }) {
        return;
    }
    ext::HSD_JObjSetMtxDirtySub(j);
}

/// `ftCo_800C6B6C` (EntryEnd_Enter): full scale, top of the stand, then EntryEnd.
pub fn ftCo_800C6B6C(g: G) {
    let f = fp(g);
    entry(f).timer = cd().x6C0;
    let s = entry(f).x8;
    set_scale(jobj(g), &s);
    let f = fp(g);
    if f.is_sub_fighter() == 0 {
        f.cur_pos.y = entry(f).x4 + entry(f).x20;
    } else {
        let l = leader(f, 0);
        f.cur_pos.y = fp(l).cur_pos.y;
    }
    ext::Fighter_ChangeMotionState(g, ftCo_MS_EntryEnd as i32, 0x3000, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    f.set_x2219_b1(1);
    f.accessory1_cb = Some(fn_800C6F34_cb);
}

/// `ftCo_EntryEnd_Anim`: at 0, the optional respawn invincibility, then Fall / Wait.
pub fn ftCo_EntryEnd_Anim(g: G) {
    let f = fp(g);
    let e = entry(f);
    e.timer = e.timer.wrapping_sub(1);
    if e.timer == 0 {
        if crate::player::get_flags_bit4(f.player_id as i32) != 0 {
            ext::ftColl_8007B760(g, cd().x6C8);
        }
        crate::ftcommon3::ftcommon_8007d92c(g);
    }
}
/// `ftCo_EntryEnd_IASA`: empty.
pub fn ftCo_EntryEnd_IASA(_g: G) {}

/// `ftCo_EntryEnd_Phys`: the stand sinks back over `x6BC` frames.
pub fn ftCo_EntryEnd_Phys(g: G) {
    let f = fp(g);
    if f.is_sub_fighter() == 0 {
        let t = entry(f).timer as f32 / cd().x6BC as f32;
        let a = entry(f).x24;
        set_scale(f.x20A0_accessory, &Vec3 { x: a, y: a * t, z: a });
        let e = entry(f);
        e.x28 = e.x20 * t;
        f.cur_pos.y = entry(f).x4 + entry(f).x28;
    } else {
        let l = leader(f, 0);
        f.cur_pos.y = fp(l).cur_pos.y;
    }
}

/// `ftCo_EntryEnd_Coll`: as [`ftCo_EntryStart_Coll`].
pub fn ftCo_EntryEnd_Coll(g: G) {
    coll(g);
}

/// `fn_800C6F34` (EntryEnd's accessory callback).
pub fn fn_800C6F34(g: G) {
    accessory_follow(g, true);
}
/// `fn_800C6F34` as the C `HSD_GObjEvent` stored in `accessory1_cb`.
pub unsafe extern "C" fn fn_800C6F34_cb(g: *mut HSD_GObj) {
    fn_800C6F34(g)
}
