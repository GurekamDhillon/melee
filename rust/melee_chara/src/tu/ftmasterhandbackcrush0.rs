//! src/melee/ft/kinds/ftMasterHand/ftmasterhandbackcrush0.c: BackPunch coll and the BackCrush
//! scale ramp. Float math: `scl - da->x78`, `/= x7C`, `+= step`: no multiply-add shape, so no
//! fused sites. Checked bit for bit (MS_370 scale setup, Anim ramp) against the C by
//! tests/tu_ftmasterhandbackcrush0.rs.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use crate::stubs as st;
use melee_types::inline::{HSD_JObjGetScale, HSD_JObjMtxIsDirty, HSD_JObjSetScale, JObjDirty};
use melee_types::*;

/// Already ported (round 8): re-exported under this TU.
pub use crate::chars::r8::ftMasterHand::ftmasterhandbackcrush0::ftMh_BackCrush_IASA;

/// Calls into TUs that are not ported yet. Each is a settable hook; unset it panics.
pub mod ext {
    use melee_types::*;
    /// `HSD_JObjSetMtxDirtySub(jobj)` (baselib/jobj.c).
    pub static mut HSD_JOBJSETMTXDIRTYSUB: Option<fn(*mut HSD_JObj)> = None;
    #[allow(non_snake_case)]
    pub fn HSD_JObjSetMtxDirtySub(jobj: *mut HSD_JObj) {
        match unsafe { HSD_JOBJSETMTXDIRTYSUB } {
            Some(f) => f(jobj),
            None => unsafe { crate::adapt::HSD_JObjSetMtxDirtySub(jobj) },
        }
    }
}
pub use crate::tu::ftmasterhandbackairplane3::ext::ftMh_MS_389_80151018;

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

pub fn ftMh_BackPunch_Coll(_gobj: *mut HSD_GObj) { melee_core::ct!("ftMh_BackPunch_Coll");}

pub fn ftMh_MS_370_80153D2C(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_MS_370_80153D2C");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        st::Fighter_ChangeMotionState(gobj, 371, 0, 0.0, 1.0, 0.0, core::ptr::null_mut());
        st::ftAnim_8006EBA4(gobj);
        melee_fighter::tu::ftbosslib::ftBossLib_8015C208(gobj, &mut pos);
        (*fp).cur_pos.x = pos.x;
        ms_370_rest(gobj);
    }
}

/// `ftMh_MS_370_80153D2C` after `cur_pos.x = pos.x`: split out so the arithmetic can be checked
/// against the C without the fighter list `ftBossLib_8015C208` reads.
pub fn ms_370_rest(gobj: *mut HSD_GObj) {
    melee_core::ct!("ms_370_rest");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        let jobj = get_jobj(gobj);
        let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
        let mut scl = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

        (*fp).cur_pos.y = (*da).x70 as f32;

        (*fp).self_vel.z = 0.0;
        (*fp).self_vel.y = 0.0;
        (*fp).self_vel.x = 0.0;

        (*fp).mv.mh.unk0.x0 = (*da).x74 as f32;
        (*fp).mv.mh.unk0.x70 = (*da).x7C;

        HSD_JObjGetScale(&*jobj, &mut scl);
        (*fp).mv.mh.unk0.x64.x = scl.x;
        (*fp).mv.mh.unk0.x64.y = scl.y;
        (*fp).mv.mh.unk0.x64.z = scl.z;

        (*fp).mv.mh.unk0.x58.x = scl.x - (*da).x78;
        (*fp).mv.mh.unk0.x58.y = scl.y - (*da).x78;
        (*fp).mv.mh.unk0.x58.z = scl.z - (*da).x78;

        (*fp).mv.mh.unk0.x58.x /= (*da).x7C as f32;
        (*fp).mv.mh.unk0.x58.y /= (*da).x7C as f32;
        (*fp).mv.mh.unk0.x58.z /= (*da).x7C as f32;

        scl.x = (*da).x78;
        scl.y = (*da).x78;
        scl.z = (*da).x78;
        HSD_JObjSetScale(&mut *jobj, &scl, &mut Dirty);

        (*fp).cmd_vars[0] = 1;
    }
}

pub fn ftMh_BackCrush_Anim(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackCrush_Anim");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        if (*fp).cmd_vars[0] != 0 {
            let mut scl = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
            let jobj = get_jobj(gobj);
            (*fp).mv.mh.unk0.x70 -= 1;
            if (*fp).mv.mh.unk0.x70 < 0 {
                (*fp).cmd_vars[0] = 0;
                scl.x = (*fp).mv.mh.unk0.x64.x;
                scl.y = (*fp).mv.mh.unk0.x64.y;
                scl.z = (*fp).mv.mh.unk0.x64.z;
            } else {
                HSD_JObjGetScale(&*jobj, &mut scl);
                scl.x += (*fp).mv.mh.unk0.x58.x;
                scl.y += (*fp).mv.mh.unk0.x58.y;
                scl.z += (*fp).mv.mh.unk0.x58.z;
            }
            HSD_JObjSetScale(&mut *jobj, &scl, &mut Dirty);
        }
        if st::ftAnim_IsFramesRemaining(gobj) == 0 {
            let fp = (*gobj).user_data as *mut Fighter;
            (*fp).self_vel.x = 0.0;
            ftMh_MS_389_80151018(gobj);
        }
    }
}

pub fn ftMh_BackCrush_Phys(gobj: *mut HSD_GObj) {
    melee_core::ct!("ftMh_BackCrush_Phys");
    unsafe {
        let fp = (*gobj).user_data as *mut Fighter;
        (*fp).mv.mh.unk0.x0 -= 1.0;
        if (*fp).mv.mh.unk0.x0 > 0.0 {
            let da = (*(*fp).ft_data).ext_attr as *const ftMasterHand_SpecialAttrs;
            melee_fighter::tu::ftbosslib::ftBossLib_8015BF74(gobj, (*da).x58);
        } else {
            (*fp).self_vel.x = 0.0;
        }
        melee_fighter::tu::ftbosslib::ftBossLib_8015C190(gobj);
    }
}
