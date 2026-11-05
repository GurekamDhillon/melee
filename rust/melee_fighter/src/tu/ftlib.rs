//! src/melee/ft/ftlib.c: the functions left out of `crate::ftlib` (round 3), which holds the
//! other 93 (oracle-verified by tests/ftlib_ref.rs) and is re-exported here.
//! ftLib_8008777C has no fused site (fma_sites.py); it is a plain atan2f of the floor normal.
#![allow(non_snake_case)]

#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]
#[allow(unused_imports)]
use crate::ftcommon2 as c2;
#[allow(unused_imports)]
use melee_core::fp::fmadds;
#[allow(unused_imports)]
use crate::ftlib::{G, each, fp as fp_ftlib, get_item, get_kind, get_motion_id, msid_in, rumble_helper, skip_opponent, update_logic_off_camera};
pub use crate::ftlib::*;

use crate::hooks as h;
use crate::types::*;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => melee_core::unported!(concat!("TODO stub: ", stringify!($name))),
                }
            }
        )*};
    }
    stubs! {
        FTCO_800C8348 ftCo_800C8348(g: G, a: i32, b: i32);
        IT_8026B560 it_8026B560(it: G) -> f32;
        IT_8026B574 it_8026B574(it: G) -> f32;
        CAMERA_REQUEST_QUAKE Camera_RequestQuake(k: CmQuakeKind, pos: &Vec3);
        FTDATA_8008572C ftData_8008572C(kind: i32);
        EFASYNC_LOAD_SYNC efAsync_LoadSync(id: i32);
        FTDATA_80085820 ftData_80085820(kind: i32, costume: u8);
        FTDATA_80085A14 ftData_80085A14(kind: i32);
        FTDATA_800857E0 ftData_800857E0(kind: i32);
        // `ftData_UnkBytePerCharacter[kind]` (ftdata.c table).
        FTDATA_UNK_BYTE ftData_UnkBytePerCharacter(kind: i32) -> i32;
        // `CostumeListsForeachCharacter[kind].numCostumes`.
        COSTUME_COUNT CostumeListsForeachCharacter_numCostumes(kind: i32) -> u8;
    }
}

/// `ftLib_800871A8`: fighter picks up a metal box.
pub fn ftlib_800871a8(g: G, item: G) {
    melee_core::ct!("ftlib_800871a8");
    assert!(h::it_get_kind(item) == It_Kind_MetalB as i32, "ftlib.c:1117");
    ext::ftCo_800C8348(g, ext::it_8026B560(item) as i32, ext::it_8026B574(item) as i32);
    h::ftco_800d105c(g);
    h::ft_80081c88(g, fp(g).x34_scale.y);
    crate::ftcommon2::ftcommon_8007ebac(fp(g), 0xC, 0);
    let pos = fp(g).cur_pos;
    ext::Camera_RequestQuake(QuakeKind_Small, &pos);
    h::play_sfx(fp(g), 0x121, 0x7F, 0x40);
}

/// `ftLib_80087508`: load one costume's archives for `ft_kind` (C param is s8).
pub fn ftlib_80087508(ft_kind: i8, arg1: u8) {
    melee_core::ct!("ftlib_80087508");
    let k = ft_kind as i32;
    ext::ftData_8008572C(k);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(k));
    ext::ftData_80085820(k, arg1);
    ext::ftData_80085A14(k);
    ext::ftData_800857E0(k);
}

/// `ftLib_80087574`: load every costume's archives for `ft_kind`.
pub fn ftlib_80087574(ft_kind: i8) {
    melee_core::ct!("ftlib_80087574");
    let k = ft_kind as i32;
    ext::ftData_8008572C(k);
    ext::efAsync_LoadSync(ext::ftData_UnkBytePerCharacter(k));
    let mut i: u8 = 0;
    while i < ext::CostumeListsForeachCharacter_numCostumes(k) {
        ext::ftData_80085820(k, i);
        i = i.wrapping_add(1);
    }
    ext::ftData_80085A14(k);
    ext::ftData_800857E0(k);
}

/// `ftLib_80087610` (TARGET_PC path): Kirby hat archives for every valid selectable kind.
pub fn ftlib_80087610(arg0: u8) {
    melee_core::ct!("ftlib_80087610");
    for i in 0..SELKIND_COUNT as i32 {
        h::ftkb_specialn_800eed50(crate::player::player_800325c8(i as _, 0) as i32, arg0 as i32);
        if i == CKind_Zelda as i32 || i == CKind_Seak as i32 {
            h::ftkb_specialn_800eed50(crate::player::player_800325c8(i as _, 1) as i32, arg0 as i32);
        }
    }
}

/// `ftLib_8008777C`: floor slope angle; asserts the fighter is grounded.
pub fn ftlib_8008777c(g: G) -> f32 {
    melee_core::ct!("ftlib_8008777c");
    let f = fp(g);
    assert!(f.ground_or_air == GA_Ground, "ftlib.c:1517");
    let n = f.coll_data.floor.normal;
    melee_runtime::lbtrigf::atan2f(-n.x, n.y)
}

/// `ftLib_UpdateLogicOffCamera` (TARGET_PC, ftlib.c): the off-screen flag `ftLib_80086A8C` writes from
/// the draw callback, computed from logic (the projection of the camera box through the game camera).
pub fn ftLib_UpdateLogicOffCamera(gobj: G) {
    melee_core::ct!("ftLib_UpdateLogicOffCamera");
    let f = fp(gobj);
    if f.x221F_b3() != 0 {
        return; // ftDrawCommon_80080E18 skips the test then too
    }
    if f.x2229_b3() == 0 && f.x2220_b7() == 0 && !h::camera_80030a50().is_null() {
        let cam = f.x890_cameraBox;
        let on = h::camera_logic_to_screen(cam, &mut f.x2188);
        f.set_x221F_b0(!on as u8);
    } else {
        f.set_x221F_b0(0);
    }
}

// ---- folded from ftlib.rs (consolidation steps 21/22) ----
/// `ftLib_800860C4`: number of fighter gobjs.
pub fn ftlib_800860c4() -> i32 {
    melee_core::ct!("ftlib_800860c4");
    let mut n = 0;
    each(|_| {
        n += 1;
        false
    });
    n
}

/// `ftLib_80086198`: the opponent with the lowest percent.
pub fn ftlib_80086198(g: G) -> G {
    melee_core::ct!("ftlib_80086198");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut min_percent: i32 = 999;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if !ftlib_80086fd4(g, cur) {
            let cf = fp_ftlib(cur);
            if cf.x221F_b3() != 0 {
                return false;
            }
            if h::gm_8016b168() && own.is_some_and(|f| cf.team == f.team) {
                return false;
            }
            if cf.dmg.x1830_percent < min_percent as f32 {
                min_percent = cf.dmg.x1830_percent as i32;
                result = cur;
            }
        }
        false
    });
    result
}

/// `ftLib_8008627C`: the closest opponent to `pos` (camera target bone).
pub fn ftlib_8008627c(pos: &Vec3, g: G) -> G {
    melee_core::ct!("ftlib_8008627c");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut min_dist = f32::MAX;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if ftlib_80086fd4(g, cur) || skip_opponent(own, fp_ftlib(cur)) {
            return false;
        }
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_800866dc(cur, &mut v);
        let dx = pos.x - v.x;
        let dy = pos.y - v.y;
        let dist = fmadds(dx, dx, dy * dy);
        if dist < min_dist {
            min_dist = dist;
            result = cur;
        }
        false
    });
    result
}

/// `ftLib_80086368`: the closest opponent on the `facing_dir` side of `v`.
pub fn ftlib_80086368(v: &Vec3, g: G, facing_dir: f32) -> G {
    melee_core::ct!("ftlib_80086368");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut min_diff = f32::MAX;
    let mut result: G = core::ptr::null_mut();
    each(|cur| {
        if ftlib_80086fd4(g, cur) || skip_opponent(own, fp_ftlib(cur)) {
            return false;
        }
        let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_800866dc(cur, &mut p);
        if (facing_dir == -1.0 && p.x > v.x) || (facing_dir == 1.0 && p.x < v.x) {
            return false;
        }
        let dx = v.x - p.x;
        let dy = v.y - p.y;
        let diff = fmadds(dx, dx, dy * dy);
        if diff < min_diff {
            min_diff = diff;
            result = cur;
        }
        false
    });
    result
}

/// `ftLib_800864A8`: which side (-1 / +1) most opponents are on; ties draw `HSD_Randi(2)`.
pub fn ftlib_800864a8(v: &Vec3, g: G) -> f32 {
    melee_core::ct!("ftlib_800864a8");
    let own = if g.is_null() { None } else { Some(&*fp_ftlib(g)) };
    let mut result: i32 = 0;
    each(|cur| {
        if ftlib_80086fd4(g, cur) {
            return false;
        }
        let cf = fp_ftlib(cur);
        if skip_opponent(own, cf) {
            return false;
        }
        // TARGET_PC: Slippi WhispyBlowDirFix.
        if h::slippi_codes() == 2 && cf.motion_id <= 0xB {
            return false;
        }
        let mut p = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ftlib_800866dc(cur, &mut p);
        result += if p.x - v.x < 0.0 { -1 } else { 1 };
        false
    });
    if result == 0 {
        result = if h::hsd_randi(2) != 0 { 1 } else { -1 };
    }
    if result < 0 { -1.0 } else { 1.0 }
}

pub fn ftlib_800865c0(g: G) -> f32 { unsafe { melee_items::tu::ftlib::ftLib_800865C0(g as _) as _ } }

pub fn ftlib_800865cc(g: G) -> i32 { unsafe { melee_items::tu::ftlib::ftLib_800865CC(g as _) as _ } }

pub fn ftlib_800865d8(g: G, x: &mut f32, y: &mut f32) { unsafe { melee_items::tu::ftlib::ftLib_800865D8(g as _, x as _, y as _); } }

/// `ftLib_800865F0`: the joint of bone `ftParts_GetBoneIndex(fp_ftlib, 4)`.
pub fn ftlib_800865f0(g: G) -> *mut HSD_JObj {
    melee_core::ct!("ftlib_800865f0");
    let f = fp_ftlib(g);
    let i = h::ftparts_get_bone_index(f, 4);
    unsafe { (*f.parts.offset(i as isize)).joint }
}

pub fn ftlib_80086630(g: G, part: i32) -> *mut HSD_JObj { unsafe { melee_items::tu::ftlib::ftLib_80086630(g as _, part as _) as _ } }

pub fn ftlib_80086644(g: G, pos: &mut Vec3) { melee_core::ct!("ftlib_80086644"); unsafe { melee_world::ftlib::ftLib_80086644(g, pos) } }

pub fn ftlib_80086664(g: G, pos: &Vec3) { melee_core::ct!("ftlib_80086664"); fp_ftlib(g).cur_pos = *pos; }

pub fn ftlib_80086684(g: G, pos: &mut Vec3) { melee_core::ct!("ftlib_80086684"); unsafe { melee_world::ftlib::ftLib_80086684(g, pos) } }

/// `ftLib_800866DC`: the camera target bone's world position (`lb_8000B1CC`).
pub fn ftlib_800866dc(g: G, v: &mut Vec3) {
    melee_core::ct!("ftlib_800866dc");
    let f = fp_ftlib(g);
    let i = unsafe { (*(*f.ft_data).x0).camera_zoom_target_bone };
    let j = ftlib_80086630(g, i);
    let off = f.co_attrs.x170;
    h::lb_8000b1cc(j, &off, v);
}

pub fn ftlib_80086724(g: G, other: G) {
    melee_core::ct!("ftlib_80086724");
    let f = fp_ftlib(g);
    if !f.item_gobj.is_null() || !f.x1978.is_null() {
        h::ftcommon_8007e6dc(g, other, 1);
    }
}

pub fn ftlib_80086764(g: G) {
    melee_core::ct!("ftlib_80086764");
    if !fp_ftlib(g).x1984_heldItemSpec.is_null() {
        h::ftcommon_8007e82c(g);
    }
}

pub fn ftlib_800867a0(g: G, arg1: G) -> bool { unsafe { melee_items::tu::ftlib::ftLib_800867A0(g as _, arg1 as _) as _ } }

pub fn ftlib_800867cc(g: G) -> G { unsafe { melee_items::tu::ftlib::ftLib_800867CC(g as _) as _ } }

pub fn ftlib_800867d8(g: G) -> bool { melee_core::ct!("ftlib_800867d8"); fp_ftlib(g).x221D_b4() != 0 }

pub fn ftlib_800867e8(g: G) {
    melee_core::ct!("ftlib_800867e8");
    h::fighter_reset_input_data(g);
    fp_ftlib(g).set_x221D_b4(1);
}

pub fn ftlib_80086824() {
    melee_core::ct!("ftlib_80086824");
    each(|c| {
        h::ftlib_800867e8(c);
        false
    });
}

pub fn ftlib_8008688c(g: G) { melee_core::ct!("ftlib_8008688c"); fp_ftlib(g).set_x221D_b4(0); }

pub fn ftlib_800868a4() {
    melee_core::ct!("ftlib_800868a4");
    each(|c| {
        h::ftlib_8008688c(c);
        false
    });
}

pub fn ftlib_800868d4(g: G, arg1: G) -> bool {
    melee_core::ct!("ftlib_800868d4");
    let f = fp_ftlib(g);
    if f.item_gobj != arg1 && f.x197C != arg1 && f.x1980 != arg1 {
        return true;
    }
    !(f.invisible() != 0
        || f.x221E_b5() != 0
        || (f.x221E_b3() == 0 && f.item_gobj == arg1)
        || (f.x221E_b4() == 0 && (f.x197C == arg1 || f.x1980 == arg1)))
}

/// `ftLib_80086960`: is `arg` a fighter gobj.
pub fn ftlib_80086960(arg: G) -> bool { melee_core::ct!("ftlib_80086960"); unsafe { melee_world::ftlib::ftLib_80086960(arg) } }

pub fn ftlib_80086984(g: G) -> *mut CollData { melee_core::ct!("ftlib_80086984"); unsafe { melee_world::ftlib::ftLib_80086984(g) } }

pub fn ftlib_80086990(g: G, v: &mut Vec3) {
    melee_core::ct!("ftlib_80086990");
    let f = fp_ftlib(g);
    let tmp = 0.5f32 * (f.coll_data.ecb.top.y + f.coll_data.ecb.bottom.y);
    v.x = f.cur_pos.x + 0.0;
    v.y = f.cur_pos.y + tmp;
    v.z = f.cur_pos.z + 0.0;
}

pub fn ftlib_800869d4(g: G) -> f32 { unsafe { melee_items::tu::ftlib::ftLib_800869D4(g as _) as _ } }

pub fn ftlib_800869f8(g: G) -> f32 { melee_core::ct!("ftlib_800869f8"); fp_ftlib(g).x34_scale.x * fp_ftlib(g).co_attrs.model_scaling }

pub fn ftlib_80086a0c(g: G) -> f32 { unsafe { melee_items::tu::ftlib::ftLib_80086A0C(g as _) as _ } }

pub fn ftlib_80086a18(g: G) -> bool {
    melee_core::ct!("ftlib_80086a18");
    matches!(fp_ftlib(g).motion_id as i32, ftCo_MS_GuardOn | ftCo_MS_Guard | ftCo_MS_GuardSetOff)
}

pub fn ftlib_80086a4c(g: G, val: f32) { melee_core::ct!("ftlib_80086a4c"); unsafe { melee_world::ftlib::ftLib_80086A4C(g, val) } }

pub fn ftlib_80086a58(g: G, x: &mut S32Vec2) -> bool {
    melee_core::ct!("ftlib_80086a58");
    let f = fp_ftlib(g);
    if f.x2229_b3() == 0 && f.x2220_b7() == 0 {
        *x = f.x2188;
    }
    false
}

/// `ftLib_80086A8C` (TARGET_PC): always true; the flag write moved to `update_logic_off_camera`.
pub fn ftlib_80086a8c(_g: G) -> bool { melee_core::ct!("ftlib_80086a8c"); true }

pub fn ftlib_80086b64(g: G) -> bool { melee_core::ct!("ftlib_80086b64"); unsafe { melee_world::ftlib::ftLib_80086B64(g) } }

pub fn ftlib_80086b74(g: G) -> *mut CmSubject { melee_core::ct!("ftlib_80086b74"); unsafe { melee_world::ftlib::ftLib_80086B74(g) } }

pub fn ftlib_80086b80(g: G) -> f32 { melee_core::ct!("ftlib_80086b80"); unsafe { melee_world::ftlib::ftLib_80086B80(g) } }

pub fn ftlib_80086b90(g: G, v: &mut Vec3) { melee_core::ct!("ftlib_80086b90"); unsafe { melee_world::ftlib::ftLib_80086B90(g, v) } }

pub fn ftlib_80086bb4(g: G) -> bool {
    melee_core::ct!("ftlib_80086bb4");
    let cam = ftlib_80086b74(g);
    h::camera_80031154(unsafe { &mut (*cam).pos })
}

pub fn ftlib_80086be0(g: G) -> u8 { melee_core::ct!("ftlib_80086be0"); unsafe { melee_world::ftlib::ftLib_80086BE0(g) } }

pub fn ftlib_80086bec(g: G, v: &mut Vec3) { melee_core::ct!("ftlib_80086bec"); unsafe { melee_world::ftlib::ftLib_80086BEC(g, v) } }

pub fn ftlib_80086c18(g: G, a: i32, b: i32) { melee_core::ct!("ftlib_80086c18"); rumble_helper(g, a, b, 0); }

pub fn ftlib_80086c9c(a: i32, b: i32) {
    melee_core::ct!("ftlib_80086c9c");
    each(|c| {
        ftlib_80086c18(c, a, b);
        false
    });
}

pub fn ftlib_80086d40(g: G, a: i32, b: i32) { melee_core::ct!("ftlib_80086d40"); rumble_helper(g, a, b, 1); }

pub fn ftlib_80086dc4(a: i32, b: i32) {
    melee_core::ct!("ftlib_80086dc4");
    each(|c| {
        ftlib_80086d40(c, a, b);
        false
    });
}

pub fn ftlib_80086e68(g: G) {
    melee_core::ct!("ftlib_80086e68");
    let f = fp_ftlib(g);
    if h::player_8003544c(f.player_id as i32, f.is_sub_fighter() as i32) {
        h::hsd_pad_rumble_remove_id(f.x618_player_id, 1);
    }
}

pub fn ftlib_80086eb4(g: G) -> i32 { unsafe { melee_items::tu::ftlib::ftLib_80086EB4(g as _) as _ } }

pub fn ftlib_80086ec0(g: G) -> bool { melee_core::ct!("ftlib_80086ec0"); unsafe { melee_world::ftlib::ftLib_80086EC0(g) } }

pub fn ftlib_80086ed0(g: G) -> bool {
    melee_core::ct!("ftlib_80086ed0");
    let f = fp_ftlib(g);
    !(f.invisible() != 0
        || f.x221E_b2() != 0
        || h::player_get_more_flags_bit4(f.player_id as i32) != 0
        || f.is_sandbag() != 0
        || f.x2229_b3() != 0
        || f.x2220_b7() != 0)
}

pub fn ftlib_80086f4c(g: G) -> bool {
    melee_core::ct!("ftlib_80086f4c");
    let f = fp_ftlib(g);
    f.x221F_b2() == 0 && f.x221F_b1() == 0 && f.x209A == 1
}

pub fn ftlib_80086f80(g: G) -> f32 {
    melee_core::ct!("ftlib_80086f80");
    let f = fp_ftlib(g);
    if f.invisible() != 0 {
        return unsafe { (*(*f.ft_data).x0).name_tag_height };
    }
    f.co_attrs.name_tag_height
}

pub fn ftlib_80086fa8(g: G) -> bool {
    melee_core::ct!("ftlib_80086fa8");
    matches!(get_motion_id(g),
        ftCo_MS_SwordSwing1 | ftCo_MS_SwordSwing3 | ftCo_MS_SwordSwing4 | ftCo_MS_SwordSwingDash)
}

/// `ftLib_80086FD4`: same gobj or same player slot.
pub fn ftlib_80086fd4(g0: G, g1: G) -> bool {
    melee_core::ct!("ftlib_80086fd4");
    if !g0.is_null() && !g1.is_null() {
        if g0 == g1 {
            return true;
        }
        if fp_ftlib(g0).player_id == fp_ftlib(g1).player_id {
            return true;
        }
    }
    false
}

pub fn ftlib_8008701c(g: G) -> bool { melee_core::ct!("ftlib_8008701c"); unsafe { melee_world::ftlib::ftLib_8008701C(g) } }

pub fn ftlib_8008702c(a: i32) { melee_core::ct!("ftlib_8008702c"); h::ftdata_80085560(a, 1); }

pub fn ftlib_80087050(a: i32) { melee_core::ct!("ftlib_80087050"); h::ftdata_80085560(a, -1); }

pub fn ftlib_80087074(g: G, v: &mut Vec3) -> bool {
    melee_core::ct!("ftlib_80087074");
    let f = fp_ftlib(g);
    if f.dmg.x18B8 != 0.0 || f.dmg.x18BC != 0.0 {
        v.x = f.dmg.x18B8;
        v.y = f.dmg.x18BC;
        v.z = 0.0;
        return true;
    }
    false
}

/// `ftLib_800870BC`: `p_ftCommonData->x6D8[controller_index]` (a flexible array in the C).
pub fn ftlib_800870bc(g: G, val: &mut i32) -> bool {
    melee_core::ct!("ftlib_800870bc");
    let i = fp_ftlib(g).x61A_controller_index;
    if i != 0 {
        *val = unsafe { *h::common_data().x6D8.as_ptr().add(i as usize) };
        return true;
    }
    false
}

/// `ftLib_800870F0`: `dmg.x1830_percent = x` (int -> float).
pub fn ftlib_800870f0(g: G, x: i32) { melee_core::ct!("ftlib_800870f0"); fp_ftlib(g).dmg.x1830_percent = x as f32; }

/// `ftLib_80087120`: `(s32) dmg.x1830_percent` (PPC fctiwz saturates; NaN differs, see ftcommon).
pub fn ftlib_80087120(g: G) -> i32 { melee_core::ct!("ftlib_80087120"); fp_ftlib(g).dmg.x1830_percent as i32 }

pub fn ftlib_80087140(g: G) {
    melee_core::ct!("ftlib_80087140");
    if fp_ftlib(g).x221F_b3() == 0 {
        h::ftco_800d4f24(g, 1);
        c2::ftcommon_8007ed2c(fp_ftlib(g));
        h::fighter_unk_process_death(g);
        h::ftcommon_8007d92c(g);
    }
}

pub fn ftlib_80087284(g: G) -> bool { melee_core::ct!("ftlib_80087284"); get_motion_id(g) >= ftCo_MS_LightThrowF4 }

pub fn ftlib_800872b0(g: G) -> *mut LbShadow { melee_core::ct!("ftlib_800872b0"); unsafe { melee_world::ftlib::ftLib_800872B0(g) } }

pub fn ftlib_800872bc(g: G) -> bool {
    melee_core::ct!("ftlib_800872bc");
    let f = fp_ftlib(g);
    f.x221E_b1() == 0 && f.x2226_b4() == 0 && f.is_sub_fighter() == 0
}

pub fn ftlib_80087300(g: G) -> i32 { melee_core::ct!("ftlib_80087300"); fp_ftlib(g).dmg.x18c4_source_ply }

pub fn ftlib_8008730c(g: G) -> i32 { melee_core::ct!("ftlib_8008730c"); fp_ftlib(g).x221F_b5() as i32 }

pub fn ftlib_8008731c(g: G) -> i32 { melee_core::ct!("ftlib_8008731c"); unsafe { melee_world::ftlib::ftLib_8008731C(g) } }

pub fn ftlib_8008732c(g: G) -> bool { melee_core::ct!("ftlib_8008732c"); unsafe { melee_world::ftlib::ftLib_8008732C(g) } }

pub fn ftlib_80087354(g: G) -> bool { melee_core::ct!("ftlib_80087354"); msid_in(g, ftCo_MS_DeadUp, ftCo_MS_DeadUpFallHitCameraIce) }

pub fn ftlib_8008737c(g: G) -> bool { melee_core::ct!("ftlib_8008737c"); msid_in(g, ftCo_MS_DeadUpStar, ftCo_MS_DeadUpStarIce) }

pub fn ftlib_800873a4(g: G) -> bool { melee_core::ct!("ftlib_800873a4"); msid_in(g, ftCo_MS_DeadUpFall, ftCo_MS_DeadUpFallHitCameraIce) }

pub fn ftlib_800873cc(g: G) -> bool { melee_core::ct!("ftlib_800873cc"); msid_in(g, ftCo_MS_Rebirth, ftCo_MS_RebirthWait) }

pub fn ftlib_800873f4(g: G) -> bool { melee_core::ct!("ftlib_800873f4"); unsafe { melee_world::ftlib::ftLib_800873F4(g) } }

/// `ftLib_8008741C`: the fighter gobj with spawn number `i`.
pub fn ftlib_8008741c(i: u32) -> G { melee_core::ct!("ftlib_8008741c"); each(|c| fp_ftlib(c).x8_spawnNum as u32 == i) }

pub fn ftlib_80087454(g: G) -> f32 { melee_core::ct!("ftlib_80087454"); fp_ftlib(g).dmg.x18A4_knockbackMagnitude }

pub fn ftlib_80087460(g: G) -> u32 { melee_core::ct!("ftlib_80087460"); fp_ftlib(g).x8_spawnNum as u32 }

/// `ftLib_8008746C` (TARGET_PC): the crowd cheer sfx id; m-ex relative ids (5000..9999) are
/// rebased onto the port kind's sound bank.
pub fn ftlib_8008746c(g: G) -> i32 {
    melee_core::ct!("ftlib_8008746c");
    let f = fp_ftlib(g);
    if c2::ftcommon_80080144(f) {
        return 0x1FBD1;
    }
    let id = unsafe { (*(*f.ft_data).x4C_sfx).x34 };
    if (5000..10000).contains(&id) {
        let ssm = h::mex_ssm_for_port_kind(f.kind as i32);
        if ssm >= 0 {
            return id - 5000 + ssm * 10000;
        }
    }
    id
}

pub fn ftlib_800874bc(g: G) -> i32 { melee_core::ct!("ftlib_800874bc"); unsafe { melee_world::ftlib::ftLib_800874BC(g) } }

pub fn ftlib_800874cc(g: G, arg1: *mut core::ffi::c_void, arg2: i32) {
    melee_core::ct!("ftlib_800874cc");
    let f = fp_ftlib(g);
    h::pl_8003e4a4(f.player_id as i32, f.is_sub_fighter() as i32, arg1, arg2);
}

pub fn ftlib_800876b4(g: G) { melee_core::ct!("ftlib_800876b4"); h::ftanim_is_frames_remaining(g); }

pub fn ftlib_800876d4(g: G) -> bool { melee_core::ct!("ftlib_800876d4"); fp_ftlib(g).smash_attrs.state as i32 == 2 }

pub fn ftlib_800876f4(g: G) -> i32 { melee_core::ct!("ftlib_800876f4"); fp_ftlib(g).dmg.x18CC }

pub fn ftlib_80087700(g: G) -> i32 { melee_core::ct!("ftlib_80087700"); fp_ftlib(g).dmg.x18D0 }

pub fn ftlib_8008770c(g: G, dst: *mut core::ffi::c_void) {
    melee_core::ct!("ftlib_8008770c");
    if get_kind(g) == Ft_Kind_GameWatch { h::ftgw_init_8014a7f4(g, dst) } else { h::ftkb_init_800eeb00(g, dst) }
}

pub fn ftlib_80087744(g: G, dst: *mut core::ffi::c_void) {
    melee_core::ct!("ftlib_80087744");
    if get_kind(g) == Ft_Kind_GameWatch { h::ftgw_init_8014a814(g, dst) } else { h::ftkb_init_800eeb1c(g, dst) }
}

pub fn ftlib_800877d4(g: G) -> bool { melee_core::ct!("ftlib_800877d4"); fp_ftlib(g).xF8_playerNudgeVel.x != 0.0 }

/// `ftLib_IsFramesRemaining` (was ftLib_800876B4): `void` in ftlib.c, it only calls ftAnim_IsFramesRemaining; its one caller
/// (gm_1798.c, declared `s32`) reads the return register, which still holds that result in the C build.
pub unsafe fn ftLib_IsFramesRemaining(gobj: *mut melee_types::HSD_GObj) -> i32 {
    melee_core::ct!("ftLib_IsFramesRemaining");
    crate::tu::ftanim::ftAnim_IsFramesRemaining(gobj as _) as i32
}
