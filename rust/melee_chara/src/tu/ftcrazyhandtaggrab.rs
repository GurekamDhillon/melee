//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandtaggrab.c. Float ops are plain f32 adds and one
//! f32->s32 cast (`(int) da->x13C_pos.x`, cabi::f2i32); no multiply, so no fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandtaggrab.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs, unused_imports)]

use crate::stubs as st;
use melee_types::*;

pub use crate::chars::r8::ftCrazyHand::ftcrazyhandtaggrab::ftCh_TagGrab_IASA;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
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
    hook!(IT_802F046C, it_802F046C, (gobj: *mut Item_GObj));
    hook!(LBAUDIOAX_800236B8, lbAudioAx_800236B8, (id: i32));
    hook!(LBAUDIOAX_8002438C, lbAudioAx_8002438C, (id: i32));
    // ftcrazyhandcapturewaitcrazyhand.c
    hook!(FTCH_GRABUNK1_8015B800, ftCh_GrabUnk1_8015B800, (gobj: *mut HSD_GObj));
}

/// `static inline func_8015ADD0_inline(gobj)`.
unsafe fn func_8015ADD0_inline(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;
        ext::it_802F046C((*fp).mv.ch.grab.x28);
        ext::it_802F046C((*fp).mv.ch.grab.x2C);
        ext::it_802F046C((*fp).mv.ch.grab.x30);
        ext::it_802F046C((*fp).mv.ch.grab.x34);
        (*fp).mv.ch.grab.x28 = core::ptr::null_mut();
        (*fp).mv.ch.grab.x2C = core::ptr::null_mut();
        (*fp).mv.ch.grab.x30 = core::ptr::null_mut();
        (*fp).mv.ch.grab.x34 = core::ptr::null_mut();
        ext::lbAudioAx_800236B8((*fp).mv.ch.unk0.x38);
        ext::lbAudioAx_800236B8((*fp).mv.ch.unk0.x3C);
        ext::lbAudioAx_800236B8((*fp).mv.ch.unk0.x40);
        if (*fp).mv.ch.unk0.x20 != 0 {
            ext::ftCh_GrabUnk1_8015B800((*fp).victim_gobj);
        }
        (*fp).self_vel.y = (*da).x12C_pos.x;
        (*fp).self_vel.z = (*da).x134_pos.x;
        (*fp).mv.ch.unk0.x8 = melee_runtime::cabi::f2i32((*da).x13C_pos.x as f64);
    }
}

pub fn ftCh_GrabUnk1_8015ADD0(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).cmd_vars[0] = 0;
        func_8015ADD0_inline(gobj);
        st::Fighter_ChangeMotionState(gobj, 0x181, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::ft0881::ft_800881d8(&mut *fp, 0x4E217, 127, 64);
        melee_fighter::hooks::play_sfx(&mut *fp, 0x4E219, 127, 64);
        (*fp).mv.ch.ch_dmg.x60 = 0;
        (*fp).mv.ch.ch_dmg.x64 = 0;
    }
}

/// Shared body of ftCh_TagGrab_Anim / ftCh_TagSqueeze_Anim (identical in the C).
unsafe fn tag_anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
            (*fp).mv.ch.ch_dmg.x60 += 1;
            if (*fp).mv.ch.ch_dmg.x60 <= (*da).x124 {
                ext::lbAudioAx_8002438C(0x81652);
            } else {
                (*fp).mv.ch.ch_dmg.x64 += 1;
                if (*fp).mv.ch.ch_dmg.x64 <= (*da).x128 {
                    ext::lbAudioAx_8002438C(0x81653);
                }
            }
            (*fp).cmd_vars[0] = 0;
        }
        if (*fp).mv.ch.unk0.x8 > 0 && st::ftAnim_IsFramesRemaining(gobj) == 0 {
            st::Fighter_ChangeMotionState(gobj, 0x182, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
            st::ftAnim_8006EBA4(gobj);
        }
        (*fp).mv.ch.unk0.x8 -= 1;
        if (*fp).mv.ch.unk0.x8 == 0 {
            melee_fighter::tu::ft_0d4d::ftCo_800D4F24(gobj, 0);
        }
    }
}

pub fn ftCh_TagGrab_Anim(gobj: *mut HSD_GObj) {
    unsafe { tag_anim(gobj) }
}

pub fn ftCh_TagSqueeze_Anim(gobj: *mut HSD_GObj) {
    unsafe { tag_anim(gobj) }
}

pub fn ftCh_TagGrab_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        (*fp).self_vel.y += (*da).x12C_pos.y;
        (*fp).self_vel.z += (*da).x134_pos.y;
    }
}

pub fn ftCh_TagGrab_Coll(_gobj: *mut HSD_GObj) {}
