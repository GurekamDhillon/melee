//! src/melee/ft/ftCo_800C7590.c: CaptureLeadead (held by a ReDead): entry `ftCo_800C7590`, its
//! Anim/IASA/Phys/Coll callbacks, the release `ftCo_800C7800` and the thunks fn_800C77E0 /
//! fn_800C7890.
//!
//! Fused sites (`tools/rust-port/fma_sites.py`, ftCo_800C7590: 2): `x720 * (x724 - handicap) +
//! x71C` and `percent * x730 + tmp2`, both -> `fmadds`; `x728 * (...)` is a separate statement,
//! not fused. Checked against the C (whole TU included, shipping contraction) by
//! tests/tu_ftco_800c7590.rs.

use super::ftco_800c7070::{ext as e70, fp, G};
use super::ftco_800c7434::{fn_800c74cc, fn_800c7568};
use crate::types::*;
use melee_core::fp::fmadds;

/// `ftCo_ReleaseItemAndVictim(gobj)` (ft/kinds/ftCommon/inlines.h).
pub fn ftco_release_item_and_victim(g: G) {
    melee_core::ct!("ftco_release_item_and_victim");
    ext::ftCommon_8007DB58(g);
    ext::ftCo_8009750C(g);
    ext::ftCo_800DD168(g);
}

/// `ftCo_800C7590(gobj)`: enter CaptureLeadead, held by item `dmg.x1868_source`.
pub fn ftco_800c7590(g: G) {
    melee_core::ct!("ftco_800c7590");
    ftco_release_item_and_victim(g);
    ext::ftCo_8008DCE0(g, -1, 0.0);
    ext::ftCommon_8007D7FC(fp(g));
    e70::Fighter_ChangeMotionState(g, ftCo_MS_CaptureLeadead, 0x1080, 0.0, 1.0, 0.0, core::ptr::null_mut());
    {
        let f = fp(g);
        // Fighter_SetDamageCallbacks (ft/inlines.h)
        f.take_dmg_cb = Some(fn_800c74cc);
        f.death2_cb = Some(fn_800c7568);
        f.mv.co.captureleadead.x0 = f.dmg.x1868_source;
    }
    ext::ftCommon_8007E2FC(fp(g));
    let cd = crate::hooks::common_data();
    let f = fp(g);
    let pid = f.player_idx as i32;
    let tmp1 = cd.x728 * (cd.x72C - (ext::Player_80033BB8(pid) + 1) as f32);
    let mut tmp2 = fmadds(cd.x720, cd.x724 - ext::Player_GetHandicap(pid) as f32, cd.x71C);
    tmp2 += tmp1;
    ext::ftCommon_InitGrab(f, 1, fmadds(f.dmg.x1830_percent, cd.x730, tmp2));
    ext::ftCommon_8007E2F4(f, 0x1FF);
    f.set_x221D_b5(1);
    f.set_x2220_b3(1);
    let item = unsafe { f.mv.co.captureleadead.x0 };
    ext::it_802EAAEC(item, g, 0, 0.0);
    ext::ftCommon_8007EBAC(fp(g), 0xC, 0);
    let pos = fp(g).cur_pos;
    ext::Camera_RequestQuake(QuakeKind_Small, &pos);
}

/// `ftCo_CaptureLeadead_Anim(gobj)`: mash out; at zero release and fly off.
pub fn ftco_capture_leadead_anim(g: G) {
    melee_core::ct!("ftco_capture_leadead_anim");
    let f = fp(g);
    let cd = crate::hooks::common_data();
    f.grab_timer -= cd.leadead_grab_timer_step;
    ext::ftCommon_GrabMash(f, cd.leadead_grab_break_threshold);
    if fp(g).grab_timer <= 0.0 {
        ext::it_802EAE80(unsafe { fp(g).mv.co.captureleadead.x0 });
        ftco_800c7800(g);
    }
}

/// `ftCo_CaptureLeadead_IASA(gobj)`: empty.
pub fn ftco_capture_leadead_iasa(_g: G) { melee_core::ct!("ftco_capture_leadead_iasa");}

/// `ftCo_CaptureLeadead_Phys(gobj)`: empty.
pub fn ftco_capture_leadead_phys(_g: G) { melee_core::ct!("ftco_capture_leadead_phys");}

/// `ftCo_CaptureLeadead_Coll(gobj)`: `ft_8008403C(gobj, fn_800C77E0)`.
pub fn ftco_capture_leadead_coll(g: G) {
    melee_core::ct!("ftco_capture_leadead_coll");
    ext::ft_8008403C(g, Some(fn_800c77e0))
}

/// `fn_800C77E0(gobj)`: `ftCo_80090780(gobj)`.
pub unsafe extern "C" fn fn_800c77e0(g: G) {
    melee_core::ct!("fn_800c77e0");
    ext::ftCo_80090780(g)
}

/// `ftCo_800C7800(gobj)`: CaptureCut away from the holder.
pub fn ftco_800c7800(g: G) {
    melee_core::ct!("ftco_800c7800");
    let cd = crate::hooks::common_data();
    let f = fp(g);
    if f.ground_or_air == GA_Ground {
        f.gr_vel = -f.facing_dir * cd.x370;
    } else {
        f.self_vel.x = -f.facing_dir * cd.x370;
    }
    e70::Fighter_ChangeMotionState(g, ftCo_MS_CaptureCut, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    fp(g).capture_timer = cd.x73C;
}

/// `fn_800C7890(gobj)`: `ftCo_800C7A30(gobj)`.
pub unsafe extern "C" fn fn_800c7890(g: G) {
    melee_core::ct!("fn_800c7890");
    super::ftco_800c78b0::ftco_800c7a30(g)
}

/// Callees. Unported ones panic unless set; ported ones default to their port.
pub mod ext {
    #![allow(non_snake_case, static_mut_refs)]
    use crate::types::*;
    type G = *mut HSD_GObj;
    macro_rules! hooks {
        ($($stat:ident $func:ident ($($a:ident: $t:ty),*) $(-> $r:ty)? => $dflt:expr;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            pub fn $func($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } { Some(f) => f($($a),*), None => $dflt }
            }
        )*};
    }
    pub(crate) use hooks;
    hooks! {
        FTCOMMON_8007DB58 ftCommon_8007DB58(g: G) => crate::ftcommon3::ftcommon_8007db58(g);
        FTCO_8009750C ftCo_8009750C(g: G) => panic!("TODO(group3 ftCo) ftCo_8009750C is not ported");
        FTCO_800DD168 ftCo_800DD168(g: G) => panic!("TODO(group3 ftCo) ftCo_800DD168 is not ported");
        FTCO_8008DCE0 ftCo_8008DCE0(g: G, a: i32, f: f32) => panic!("TODO(group3 ftCo) ftCo_8008DCE0({:?}, {a}, {f}) is not ported", g);
        FTCOMMON_8007D7FC ftCommon_8007D7FC(fp: &mut Fighter) => crate::ftcommon3::ftcommon_8007d7fc(fp);
        FTCOMMON_8007E2FC ftCommon_8007E2FC(fp: &mut Fighter) => crate::tu::ftcommon::ftcommon_8007e2fc(fp);
        PLAYER_80033BB8 Player_80033BB8(slot: i32) -> i32 => crate::player::player_80033bb8(slot);
        PLAYER_GET_HANDICAP Player_GetHandicap(slot: i32) -> i32 => crate::player::get_handicap(slot);
        FTCOMMON_INIT_GRAB ftCommon_InitGrab(fp: &mut Fighter, a: i32, t: f32) => crate::ftcommon2::init_grab(fp, a, t);
        FTCOMMON_8007E2F4 ftCommon_8007E2F4(fp: &mut Fighter, v: i16) => crate::ftcommon2::ftcommon_8007e2f4(fp, v);
        IT_802EAAEC it_802EAAEC(i: *mut Item_GObj, g: G, a: i32, f: f32) => panic!("TODO(group5 itleadead.c) it_802EAAEC({:?}, {:?}, {a}, {f}) is not ported", i, g);
        FTCOMMON_8007EBAC ftCommon_8007EBAC(fp: &mut Fighter, a: u32, b: u32) => crate::ftcommon2::ftcommon_8007ebac(fp, a, b);
        CAMERA_REQUEST_QUAKE Camera_RequestQuake(k: CmQuakeKind, p: &Vec3) => panic!("TODO(group2 cm) Camera_RequestQuake({k}, {:?}) is not ported", p as *const Vec3);
        FTCOMMON_GRAB_MASH ftCommon_GrabMash(fp: &mut Fighter, t: f32) -> bool => crate::ftcommon2::grab_mash(fp, t);
        IT_802EAE80 it_802EAE80(i: *mut Item_GObj) => panic!("TODO(group5 itleadead.c) it_802EAE80({:?}) is not ported", i);
        FT_8008403C ft_8008403C(g: G, cb: HSD_GObjEvent) => panic!("TODO(group3 ft_0840) ft_8008403C({:?}, {}) is not ported", g, cb.is_some());
        FTCO_80090780 ftCo_80090780(g: G) => panic!("TODO(group3 ftCo) ftCo_80090780({:?}) is not ported", g);
    }
}
