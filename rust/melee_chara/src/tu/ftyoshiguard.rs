//! src/melee/ft/kinds/ftYoshi/ftyoshiguard.c: Yoshi's egg shield (guard on, hold, off, damage,
//! reflect). Most states are thin: they call the shared guard states in
//! `melee_fighter::tu::ftCo_Guard`, which are ported. Fused sites (tools/rust-port/fma_sites.py,
//! C IR `llvm.fmuladd`): `ftYs_Shield_8012C600`'s `x28C * (x19A4 * (1 - lightshield)) + x290`
//! (1), as `fmadds`. `ftYs_Shield_8012C1D4`'s guard hold and the `ftYs_Shield_8012C914` helpers
//! have no float products.
//!
//! Callees in TUs not yet ported are `ext` hooks named as the C function; unset, they panic.
//! `ftYs_Shield_8012C1D4` is the function `melee_fighter::tu::ftCo_Guard::ftCo_800928CC` calls
//! for Yoshi through its own `ext` hook; installing it there is wiring, not done here.
//! Smoke-tested by tests/tu_ftyoshi.rs (ftYs_Init_8012BAC0, ftYs_Init_8012B918, ftYs_Shield_8012CC1C);
//! the rest is unverified.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, clippy::all)]

use crate::stubs as st;
use melee_core::fp::fmadds;
use melee_fighter::tu::ftCo_Guard as g;
use melee_types::*;


use crate::tu::ftyoshi::{ftYs_Init_8012B8A4, ftYs_Unk1_803B75C0};

/// Calls into TUs that are not ported yet (or that have no native port reachable from here).
/// Each is a settable hook; an unset hook panics, except where `hook_or!` names a native port.
pub mod ext {
    use melee_types::*;
    macro_rules! hook {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            }
        };
    }
    macro_rules! hook_or {
        ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
            pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(non_snake_case)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $hook } {
                    Some(f) => f($($a),*),
                    None => $port($($a),*),
                }
            }
        };
    }
    hook!(FTCOLL_8007B0C0, ftColl_8007B0C0, (g: *mut HSD_GObj, state: i32));
    hook!(FTCOLL_HURTBOXINIT, ftColl_HurtboxInit, (fp: *mut Fighter, capsule: *mut FighterHurtCapsule, hurt: *const ftHurtboxInit));
    hook!(FTPARTS_GETBONEINDEX, ftParts_GetBoneIndex, (fp: *mut Fighter, part: i32) -> Fighter_Part);
    hook!(EFASYNC_SPAWN, efAsync_Spawn, (gobj: *mut HSD_GObj, slot: *mut core::ffi::c_void, part: i32, id: i32, jobj: *mut HSD_JObj, xbc: *mut core::ffi::c_void));
    hook!(HSD_JOBJSETTRANSLATE, HSD_JObjSetTranslate, (jobj: *mut HSD_JObj, v: *const Vec3));

    pub static mut FTANIM_8006FA58: Option<fn(*mut Fighter, i32, *mut HSD_Joint)> = None;
    #[allow(non_snake_case)]
    pub fn ftAnim_8006FA58(fp: *mut Fighter, part: i32, joint: *mut HSD_Joint) {
        match unsafe { FTANIM_8006FA58 } {
            Some(f) => f(fp, part, joint),
            None => { unsafe { let _ = melee_fighter::tu::ftanim::ftAnim_8006FA58(&mut *(fp as *mut _), part as _, joint as _); } },
        }
    }
    hook_or!(FT_PLAYSFX, ft_PlaySFX, (fp: *mut Fighter, sfx: i32, vol: i32, pan: i32),
        |fp: *mut Fighter, sfx: i32, vol: i32, pan: i32| unsafe { melee_fighter::hooks::play_sfx(&mut *fp, sfx, vol, pan) });
}

/// `ftYs_MF`-free: the shield states' motion flags (`Ft_MF_None` is 0).
const MF_NONE: u32 = 0;

#[inline(always)]
unsafe fn fpp(gobj: *mut HSD_GObj) -> *mut Fighter {
    (*gobj).user_data as *mut Fighter
}

#[inline(always)]
fn common() -> &'static ftCommonData {
    melee_fighter::hooks::common_data()
}

/// `ftYs_Init_8012BDA0`: the intangible hurtbox set at on-load of the shield.
pub fn ftYs_Init_8012BDA0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftColl_8007B0C0(gobj, HurtCapsule_Intangible as i32);
        let mut hurt: ftHurtboxInit = core::mem::zeroed();
        hurt.bone_idx = (*(*(*fp).ft_data).x8).x11 as Fighter_Part;
        hurt.height = HurtHeight_Mid;
        hurt.is_grabbable = 1;
        hurt.a_offset = ftYs_Unk1_803B75C0;
        hurt.b_offset = ftYs_Unk1_803B75C0;
        hurt.scale = 1.0;
        ext::ftColl_HurtboxInit(fp, (*fp).hurt_capsules.as_mut_ptr(), &hurt);
    }
}

/// `ftYs_Init_8012BE3C`: the shield's hip effect, then the Disabled hurtbox set.
pub fn ftYs_Init_8012BE3C(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftParts_80074B0C(gobj, 0, 0);
        ext::ftColl_8007B0C0(gobj, HurtCapsule_Enabled as i32);
        let xbc = &mut (*fp).co_attrs.xBC as *mut _ as *mut core::ffi::c_void;
        let part = ext::ftParts_GetBoneIndex(fp, 4);
        let jobj = (*(*fp).parts.add(part as usize)).joint;
        let fp1 = fpp(gobj);
        ext::efAsync_Spawn(gobj, &mut (*fp1).x60C as *mut _ as *mut core::ffi::c_void, 4, 1231, jobj, xbc);
    }
}

/// `ftYs_Init_8012B8A4_no_inline_2` (static): the shield matanim, not inlined in the C.
fn ftYs_Init_8012B8A4_no_inline_2(gobj: *mut HSD_GObj) {
    crate::tu::ftyoshi::ftYs_Init_8012B8A4(gobj);
}

/// `ftYs_Init_8012B8A4_no_inline` (static).
fn ftYs_Init_8012B8A4_no_inline(gobj: *mut HSD_GObj) {
    ftYs_Init_8012B8A4_no_inline_2(gobj);
}

/// `ftYs_Init_8012BECC_sub` (static): the shield reset shared by the enter and the wall-hit paths.
fn ftYs_Init_8012BECC_sub(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftColl_8007B0C0(gobj, HurtCapsule_Disabled as i32);
        (*fp).mv.ys.guard.xC = 0;
        (*fp).mv.ys.guard.x0 = 0.0;
        (*fp).mv.ys.guard.x10 = common().x268;
        (*fp).mv.ys.guard.x24 = 0;
        (*fp).mv.ys.guard.x20 = 0;
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let joint = (*(*fp).parts.add((*(*(*fp).ft_data).x8).x11 as usize)).joint;
        ext::HSD_JObjSetTranslate(joint, &mut v);
        ftYs_Init_8012B8A4_no_inline(gobj);
        g::ftCo_80091D58(&mut *fp);
        ext::ft_PlaySFX(fp, 0x6E, 0x7F, 0x40);
    }
}

/// `ftYs_Init_8012BECC`: enter the guard-on state at frame 0 and reset the shield.
pub fn ftYs_Init_8012BECC(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, ftYs_MS_GuardOn_0 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    st::ftAnim_8006EBA4(gobj);
    g::ftCo_80092450(gobj);
    ftYs_Init_8012BECC_sub(gobj);
}

/// `spawnEffect` (static inline): the shield's hip effect when the guard is struck.
fn spawnEffect(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::ftParts_80074B0C(gobj, 0, 0);
        ext::ftColl_8007B0C0(gobj, HurtCapsule_Enabled as i32);
        let co_xbc = &mut (*fp).co_attrs.xBC as *mut _ as *mut core::ffi::c_void;
        let part = ext::ftParts_GetBoneIndex(fp, FtPart_HipN as i32);
        let fp2 = fpp(gobj);
        let jobj = (*(*fp).parts.add(part as usize)).joint;
        ext::efAsync_Spawn(gobj, &mut (*fp2).x60C as *mut _ as *mut core::ffi::c_void, FtPart_HipN as i32, 1231, jobj, co_xbc);
    }
}

/// `ftYs_GuardOn_0_Anim`.
pub fn ftYs_GuardOn_0_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ys.guard.x0 += 1.0;
        g::ftCo_80092BCC(gobj);
        if g::ftCo_800925A4(gobj) {
            spawnEffect(gobj);
        } else if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            g::ftCo_800928CC(gobj);
        } else {
            ftYs_Init_8012B8A4(gobj);
        }
    }
}

/// `ftYs_GuardOn_0_IASA`: the first interrupts that leave the guard.
pub fn ftYs_GuardOn_0_IASA(gobj: *mut HSD_GObj) {
    if melee_fighter::tu::ftCo_ItemThrow::ftCo_8009515C(gobj) {
        return;
    }
    if melee_fighter::tu::ftCo_Escape::ftCo_80099794(gobj) {
        return;
    }
    if melee_fighter::tu::ftCo_Escape::ftCo_8009917C(gobj) {
        return;
    }
    if melee_fighter::tu::ftCo_Catch::ftCo_Catch_CheckInput(gobj) {
        return;
    }
    if melee_fighter::tu::ftCo_Pass::ftCo_8009A080(gobj) {
        return;
    }
}

/// `ftYs_GuardOn_0_Phys`.
pub fn ftYs_GuardOn_0_Phys(gobj: *mut HSD_GObj) {
    g::ftCo_GuardOn_Phys(gobj);
}

/// `ftYs_GuardOn_0_Coll`.
pub fn ftYs_GuardOn_0_Coll(gobj: *mut HSD_GObj) {
    g::ftCo_GuardOn_Coll(gobj);
}

/// `inlineA0` (static inline): the intangible hurtbox set used by the shield states.
fn inlineA0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftColl_8007B0C0(gobj, HurtCapsule_Intangible as i32);
        let mut hurt: ftHurtboxInit = core::mem::zeroed();
        hurt.bone_idx = (*(*(*fp).ft_data).x8).x11 as Fighter_Part;
        hurt.height = HurtHeight_Mid;
        hurt.is_grabbable = 1;
        hurt.a_offset = ftYs_Unk1_803B75C0;
        hurt.b_offset = ftYs_Unk1_803B75C0;
        hurt.scale = 1.0;
        ext::ftColl_HurtboxInit(fp, (*fp).hurt_capsules.as_mut_ptr(), &hurt);
    }
}

/// `ftYs_Shield_8012C1D4`: enter the guard-hold state (Yoshi's `ftCo_800928CC`).
pub fn ftYs_Shield_8012C1D4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftYs_MS_GuardHold as i32, MF_NONE, 0.0, 1.0, 0.0, core::ptr::null_mut());
        ext::ftAnim_8006FA58(fp, FtPart_TransN as i32, (*(*fp).x108_costume_joint).child);
        st::ftParts_80074B0C(gobj, 0, 1);
        inlineA0(gobj);
        g::ftCo_80092450(gobj);
        (*fp).set_x221E_b4(0);
        ftYs_Init_8012B8A4(gobj);
        g::ftCo_80091D58(&mut *fp);
    }
}

/// `ftYs_GuardHold_Anim`.
pub fn ftYs_GuardHold_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ys.guard.x0 += 1.0;
        g::ftCo_80092BCC(gobj);
        if g::ftCo_800925A4(gobj) {
            spawnEffect(gobj);
        } else if (*fp).mv.ys.guard.xC != 0 || ((*fp).anon_221b.x221B_b0() == 0 && (*fp).reflecting() == 0) {
            g::ftCo_80092BE8(gobj);
        } else {
            ftYs_Init_8012B8A4(gobj);
        }
    }
}

/// `ftYs_GuardHold_IASA`: the held-shield interrupts (no state change is taken here).
pub fn ftYs_GuardHold_IASA(gobj: *mut HSD_GObj) {
    if !melee_fighter::tu::ftCo_ItemThrow::ftCo_8009515C(gobj)
        && !melee_fighter::tu::ftCo_Escape::ftCo_80099794(gobj)
        && !melee_fighter::tu::ftCo_Escape::ftCo_8009917C(gobj)
        && !melee_fighter::tu::ftCo_Catch::ftCo_Catch_CheckInput(gobj)
        && !melee_fighter::tu::ftCo_Pass::ftCo_8009A080(gobj)
    {
        return;
    }
}

/// `ftYs_GuardHold_Phys`.
pub fn ftYs_GuardHold_Phys(gobj: *mut HSD_GObj) {
    g::ftCo_Guard_Phys(gobj);
}

/// `ftYs_GuardHold_Coll`.
pub fn ftYs_GuardHold_Coll(gobj: *mut HSD_GObj) {
    g::ftCo_Guard_Coll(gobj);
}

/// `ftYs_Shield_8012C49C`: the guard-off state, then the hurtbox set.
pub fn ftYs_Shield_8012C49C(gobj: *mut HSD_GObj) {
    st::Fighter_ChangeMotionState(gobj, 343, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
    ftYs_Init_8012BE3C(gobj);
}

/// `ftYs_GuardOff_Anim`.
pub fn ftYs_GuardOff_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        (*fp).mv.ys.guard.x0 += 1.0;
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::ft_8008A2BC(gobj);
        }
    }
}

/// `ftYs_GuardOff_IASA`.
pub fn ftYs_GuardOff_IASA(gobj: *mut HSD_GObj) {
    if melee_fighter::tu::ftCo_Escape::ftCo_80099794(gobj) {
        return;
    }
}

/// `ftYs_GuardOff_Phys`.
pub fn ftYs_GuardOff_Phys(gobj: *mut HSD_GObj) {
    g::ftCo_GuardOff_Phys(gobj);
}

/// `ftYs_GuardOff_Coll`.
pub fn ftYs_GuardOff_Coll(gobj: *mut HSD_GObj) {
    g::ftCo_GuardOff_Coll(gobj);
}

/// `ftYs_Shield_8012C600`: the guard-damage knockback and the hitlag callbacks.
pub fn ftYs_Shield_8012C600(gobj: *mut HSD_GObj, arg1: bool) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, 0x158, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        (*fp).hitlag_cb = Some(g::ftCo_80093240);
        (*fp).active_timer.lstick.x = 0xFE;
        (*fp).post_hitlag_cb = Some(g::ftCo_800932DC);
        if (*fp).x221C_b2() == 0 {
            st::ftParts_80074B0C(gobj, 0, 1);
        }
        inlineA0(gobj);
        let p = common();
        // C: `p->x28C * (fp->x19A4 * (1.0F - fp->lightshield_amount)) + p->x290`, fused.
        let temp = fmadds(p.x28C, (*fp).x19A4 as f32 * (1.0_f32 - (*fp).lightshield_amount), p.x290);
        if !arg1 {
            (*fp).gr_vel = temp * p.x294;
            let var_f0 = if (*fp).specialn_facing_dir < 0.0 { (*fp).gr_vel } else { -(*fp).gr_vel };
            (*fp).gr_vel = var_f0;
        }
        g::ftCo_80092450(gobj);
        g::ftCo_80091D58(&mut *fp);
    }
}

/// `ftYs_GuardDamage_Anim`.
pub fn ftYs_GuardDamage_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        g::ftCo_80093BC0(gobj);
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            if (*fp).mv.ys.guard.xC != 0 {
                g::ftCo_80092BE8(gobj);
            } else {
                g::ftCo_800928CC(gobj);
            }
        }
    }
}

/// `ftYs_GuardDamage_IASA` (empty in the C).
pub fn ftYs_GuardDamage_IASA(_gobj: *mut HSD_GObj) {}

/// `ftYs_GuardDamage_Phys`.
pub fn ftYs_GuardDamage_Phys(gobj: *mut HSD_GObj) {
    g::ftCo_GuardSetOff_Phys(gobj);
}

/// `ftYs_GuardDamage_Coll`.
pub fn ftYs_GuardDamage_Coll(gobj: *mut HSD_GObj) {
    g::ftCo_GuardSetOff_Coll(gobj);
}

/// `ftYs_Shield_8012C850`: the reflect-start state, with its timers and flags.
pub fn ftYs_Shield_8012C850(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftYs_MS_GuardOn_1 as i32, 16, (*fp).cur_anim_frame, 1.0, 0.0, core::ptr::null_mut());
        (*fp).active_timer.trigger = 254;
        (*fp).set_x221A_b7(0);
        (*fp).anon_221b.set_x221B_b0(0);
        (*fp).set_x221C_b3(1);
        (*fp).set_x221C_b1(1);
        (*fp).set_x221C_b2(1);
        (*fp).mv.ys.guard.x14 = common().x2A4;
        (*fp).mv.ys.guard.x18 = common().x2B4;
        g::ftCo_8009370C(gobj, Some(cb_ftYs_Shield_8012CACC));
    }
}

/// `ftYs_Shield_8012C914_inline` (static): the reset of the shield after a reflect.
fn ftYs_Shield_8012C914_inline(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        ext::ftColl_8007B0C0(gobj, HurtCapsule_Disabled as i32);
        (*fp).mv.ys.guard.xC = 0;
        (*fp).mv.ys.guard.x0 = 0.0;
        (*fp).mv.ys.guard.x10 = common().x268;
        (*fp).mv.ys.guard.x24 = 0;
        (*fp).mv.ys.guard.x20 = 0;
        let mut translate = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let joint = (*(*fp).parts.add((*(*(*fp).ft_data).x8).x11 as usize)).joint;
        ext::HSD_JObjSetTranslate(joint, &mut translate);
        ftYs_Init_8012B8A4(gobj);
        g::ftCo_80091D58(&mut *fp);
        ext::ft_PlaySFX(fp, 0x6E, 0x7F, 0x40);
    }
}

/// `ftYs_Shield_8012C914`: the guard-on state entered from the reflect, then the reset.
pub fn ftYs_Shield_8012C914(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        st::Fighter_ChangeMotionState(gobj, ftYs_MS_GuardOn_1 as i32, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        (*fp).active_timer.trigger = 254;
        (*fp).set_x221C_b3(1);
        (*fp).set_x221C_b1(1);
        (*fp).set_x221C_b2(1);
        (*fp).mv.ys.guard.x14 = common().x2A4;
        (*fp).mv.ys.guard.x18 = common().x2B4;
        g::ftCo_8009370C(gobj, Some(cb_ftYs_Shield_8012CACC));
        ftYs_Shield_8012C914_inline(gobj);
    }
}

/// `ftYs_Shield_8012CACC` (empty in the C): the reflect callback.
pub fn ftYs_Shield_8012CACC(_arg0: *mut HSD_GObj) {}

/// The C `ftYs_Shield_8012CACC` as an `HSD_GObjEvent` callback.
pub unsafe extern "C" fn cb_ftYs_Shield_8012CACC(g: *mut HSD_GObj) {
    ftYs_Shield_8012CACC(g);
}

/// `ftYs_GuardOn_1_Anim`.
pub fn ftYs_GuardOn_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = fpp(gobj);
        g::ftCo_80093BC0(gobj);
        (*fp).mv.ys.guard.x0 += 1.0;
        g::ftCo_80092BCC(gobj);
        if g::ftCo_800925A4(gobj) {
            spawnEffect(gobj);
        } else if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            g::ftCo_800928CC(gobj);
        } else {
            ftYs_Init_8012B8A4(gobj);
        }
    }
}

/// `ftYs_GuardOn_1_IASA`.
pub fn ftYs_GuardOn_1_IASA(gobj: *mut HSD_GObj) {
    g::ftCo_GuardReflect_IASA(gobj);
}

/// `ftYs_GuardOn_1_Phys`.
pub fn ftYs_GuardOn_1_Phys(gobj: *mut HSD_GObj) {
    g::ftCo_GuardReflect_Phys(gobj);
}

/// `ftYs_GuardOn_1_Coll`.
pub fn ftYs_GuardOn_1_Coll(gobj: *mut HSD_GObj) {
    g::ftCo_GuardReflect_Coll(gobj);
}

/// `ftYs_Shield_8012CC1C`: shield-button press with shield health left drops into the guard-off.
pub fn ftYs_Shield_8012CC1C(gobj: *mut HSD_GObj) -> bool {
    unsafe {
        let fp = fpp(gobj);
        if ((*fp).input.held_buttons[0] & HSD_PAD_LR) != 0 && (*fp).shield_health >= 0.0 {
            g::ftCo_800928CC(gobj);
            return true;
        }
        false
    }
}
