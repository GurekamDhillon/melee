//! src/melee/ft/kinds/ftCrazyHand/ftcrazyhandwait11.c. Float math: scale interpolation
//! (`scl - da->xFC`, `/= x100`, `+= step`): no multiply-add anywhere, so no fused sites.
//! Smoke-tested by tests/tu_ftcrazyhandwait11.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::inline::{HSD_JObjGetScale, HSD_JObjMtxIsDirty, HSD_JObjSetScale, JObjDirty};
use melee_types::*;

/// Already ported (round 8): re-exported under this TU.
pub use crate::chars::r8::ftCrazyHand::ftcrazyhandwait11::ftCh_Wait1_1_IASA;

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
    hook!(FTCH_GRABUNK1_8015BC88, ftCh_GrabUnk1_8015BC88, (gobj: *mut HSD_GObj));
    // `HSD_JObjSetMtxDirtySub(jobj)` (baselib/jobj.c).
    hook!(HSD_JOBJSETMTXDIRTYSUB, HSD_JObjSetMtxDirtySub, (jobj: *mut HSD_JObj));
}

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

/// `get_jobj(gobj)` (static inline): `gobj->hsd_obj`.
#[inline(always)]
unsafe fn get_jobj(gobj: *mut HSD_GObj) -> *mut HSD_JObj {
    (*gobj).hsd_obj as *mut HSD_JObj
}

pub fn fn_80159AA4(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let jobj = get_jobj(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
        let mut pos = Vec3::zeroed();
        let mut scl = Vec3::zeroed();

        st::Fighter_ChangeMotionState(gobj, 0x175, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
        (*fp).cur_pos.x = pos.x;
        (*fp).cur_pos.y = (*da).xF4 as f32;

        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;

        (*fp).mv.ch.unk0.x0 = (*da).xF8 as f32;
        (*fp).mv.ch.ch_backcrush.x5C = (*da).x100;

        HSD_JObjGetScale(&*jobj, &mut scl);
        (*fp).mv.ch.ch_backcrush.x50.x = scl.x;
        (*fp).mv.ch.ch_backcrush.x50.y = scl.y;
        (*fp).mv.ch.ch_backcrush.x50.z = scl.z;

        (*fp).mv.ch.ch_backcrush.x44.x = scl.x - (*da).xFC;
        (*fp).mv.ch.ch_backcrush.x44.y = scl.y - (*da).xFC;
        (*fp).mv.ch.ch_backcrush.x44.z = scl.z - (*da).xFC;

        (*fp).mv.ch.ch_backcrush.x44.x /= (*da).x100 as f32;
        (*fp).mv.ch.ch_backcrush.x44.y /= (*da).x100 as f32;
        (*fp).mv.ch.ch_backcrush.x44.z /= (*da).x100 as f32;

        scl.x = (*da).xFC;
        scl.y = (*da).xFC;
        scl.z = (*da).xFC;
        HSD_JObjSetScale(&mut *jobj, &scl, &mut Dirty);

        (*fp).cmd_vars[0] = 1;
    }
}

/// `fn_80159AA4` as a Fighter callback entry (`ftCh_Init_801597F0(gobj, fn_80159AA4)`).
pub unsafe extern "C" fn cb_fn_80159AA4(g: *mut HSD_GObj) {
    fn_80159AA4(g);
}

pub fn ftCh_Wait1_1_Anim(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;

        if (*fp).cmd_vars[0] != 0 {
            let mut scl = Vec3::zeroed();
            let jobj = get_jobj(gobj);
            (*fp).mv.ch.ch_backcrush.x5C -= 1;
            if (*fp).mv.ch.ch_backcrush.x5C < 0 {
                (*fp).cmd_vars[0] = 0;
                scl.x = (*fp).mv.ch.ch_backcrush.x50.x;
                scl.y = (*fp).mv.ch.ch_backcrush.x50.y;
                scl.z = (*fp).mv.ch.ch_backcrush.x50.z;
            } else {
                HSD_JObjGetScale(&*jobj, &mut scl);
                scl.x += (*fp).mv.ch.ch_backcrush.x44.x;
                scl.y += (*fp).mv.ch.ch_backcrush.x44.y;
                scl.z += (*fp).mv.ch.ch_backcrush.x44.z;
            }
            HSD_JObjSetScale(&mut *jobj, &scl, &mut Dirty);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.x = 0.0;
            ext::ftCh_GrabUnk1_8015BC88(gobj);
        }
    }
}

pub fn ftCh_Wait1_1_Phys(gobj: *mut HSD_GObj) {
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.ch.unk0.x0 -= 1.0;
        if (*fp).mv.ch.unk0.x0 > 0.0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftCrazyHand_DatAttrs;
            melee_fighter::tu::ftbosslib::ftBossLib_8015BF74(gobj, (*da).xDC);
        } else {
            (*fp).self_vel.x = 0.0;
        }
        melee_fighter::tu::ftbosslib::ftBossLib_8015C190(gobj);
    }
}

pub fn ftCh_Wait1_1_Coll(_gobj: *mut HSD_GObj) {}
