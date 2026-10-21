//! Native port of src/melee/ef/efsync.c (`efSync_Spawn`, the gfx_id router, and the TARGET_PC
//! m-ex `efSync_MexSpawn`).
//!
//! The C's `...` / `va_list` is [`Va`]: every vararg the routes read is a pointer (`Vec3*`,
//! `f32*`, `HSD_JObj*`), taken in order. Float math: `M_TAU * HSD_Randf()` is an f64 product
//! stored to f32; `2.0f * cosf(x)` and its product are separate statements (no fused sites).
#![allow(non_snake_case, static_mut_refs, clippy::missing_safety_doc)]

use super::eflib::{self as l, efLib_AnimQueue, EF_DATA};
use melee_runtime::gekko::{cosf, sinf};
use melee_types::inline::*;
use melee_types::*;

type G = *mut HSD_GObj;
type J = *mut HSD_JObj;
type Gen = *mut HSD_Generator;
type Obj = *mut core::ffi::c_void;

const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;
const M_TAU: f64 = 6.283185307179586;
const EF_MEX_MDL_START: i32 = 5000;
const EF_MEX_END: i32 = 9000;

/// The variadic tail of `efSync_Spawn`: pointer arguments, read in order like `va_arg`.
pub struct Va<'a> {
    args: &'a [Obj],
    at: usize,
}
impl<'a> Va<'a> {
    pub fn new(args: &'a [Obj]) -> Self { Self { args, at: 0 } }
    fn ptr(&mut self) -> Obj {
        let p = self.args[self.at];
        self.at += 1;
        p
    }
    fn jobj(&mut self) -> J { self.ptr() as J }
    fn vec3(&mut self) -> &'static Vec3 { unsafe { &*(self.ptr() as *const Vec3) } }
    fn f32(&mut self) -> f32 { unsafe { *(self.ptr() as *const f32) } }
}

/// Calls into TUs that are not ported yet. Each panics until a later pass rewires it.
pub mod ext {
    use super::{Obj, Va, G};
    pub fn efAsync_Dispatch(_gfx_id: i32, _gobj: G, _va: &mut Va) -> Obj { todo!("efAsync_Dispatch (ef/efasync.c)") }
    pub fn efAlt_Spawn(_gfx_id: i32, _gobj: G, _va: &mut Va) -> Obj { todo!("efAlt_Spawn (ef/efalt.c)") }
    /// `efAsync_MexResolve(gobj, gfx_id, &id, &is_ptcl, &owner)` -> (type, id, is_ptcl, owner).
    pub fn efAsync_MexResolve(_gobj: G, _gfx_id: i32) -> (i32, i32, bool, G) { todo!("efAsync_MexResolve (ef/efasync.c)") }
    pub fn HSD_Randf() -> f32 { todo!("HSD_Randf (baselib/random.c)") }
}

struct Dirty;
impl JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) { l::ext::HSD_JObjSetMtxDirtySub(j) }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) { l::ext::HSD_JObjSetMtxDirtySub(j) }
}
#[inline(always)]
unsafe fn jobj_of(g: G) -> J { (*g).hsd_obj as J }
unsafe fn ejobj(e: *mut EF_Effect) -> J { jobj_of((*e).gobj) }
unsafe fn set_rot_y(j: J, v: f32) { HSD_JObjSetRotationY(&mut *j, v, &mut Dirty) }
unsafe fn set_rot_z(j: J, v: f32) { HSD_JObjSetRotationZ(&mut *j, v, &mut Dirty) }
unsafe fn set_scale(j: J, v: &Vec3) { HSD_JObjSetScale(&mut *j, v, &mut Dirty) }
unsafe fn get_scale(j: J) -> Vec3 {
    let mut s = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    HSD_JObjGetScale(&*j, &mut s);
    s
}
#[inline]
fn half_pi(d: f32) -> f32 { (if d < 0.0 { -M_PI_2 } else { M_PI_2 }) as f32 }

unsafe extern "C" fn pause_all(g: G) { l::efLib_PauseAll(g) }
unsafe extern "C" fn resume_all(g: G) { l::efLib_ResumeAll(g) }

/// `efSync_MexSpawn` (static, TARGET_PC).
pub unsafe fn efSync_MexSpawn(gfx_id: i32, gobj: G, va: &mut Va) -> Obj {
    let (ty, id, is_ptcl, owner) = ext::efAsync_MexResolve(gobj, gfx_id);
    if ty < 0 {
        return core::ptr::null_mut();
    }
    if !is_ptcl {
        let eff: *mut EF_Effect;
        match ty {
            0 | 2 => {
                eff = l::efLib_Create_Attach_Pos(id as u32, gobj, va.vec3());
                if !eff.is_null() {
                    set_rot_z(ejobj(eff), va.f32());
                }
            }
            1 => {
                eff = l::efLib_Create_Attach_Pos(id as u32, gobj, va.vec3());
                if !eff.is_null() {
                    set_scale(ejobj(eff), &(*jobj_of(gobj)).scale);
                }
            }
            3 => {
                let jobj = va.jobj();
                let mut w = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
                l::ext::lb_8000B1CC(jobj, core::ptr::null(), &mut w);
                eff = l::efLib_Create_Attach_Pos(id as u32, gobj, &w);
                if !eff.is_null() {
                    set_rot_y(ejobj(eff), 0.0);
                    set_rot_z(ejobj(eff), (*jobj).rotate.z);
                }
            }
            4 => {
                let pos = va.vec3();
                eff = l::efLib_Create_Attach_Pos(id as u32, gobj, pos);
                if !eff.is_null() {
                    set_rot_y(ejobj(eff), half_pi(va.f32()));
                    set_scale(ejobj(eff), &(*jobj_of(gobj)).scale);
                }
            }
            5 => eff = l::efLib_Create_Attach_Scale(id as u32, gobj, va.jobj()),
            6 => eff = l::efLib_Create_AttachChild_Scale(id as u32, gobj, va.jobj()),
            _ => return core::ptr::null_mut(),
        }
        let f = (*owner).user_data as *mut Fighter;
        (*f).pre_hitlag_cb = Some(pause_all);
        (*f).post_hitlag_cb = Some(resume_all);
        return eff as Obj;
    }
    match ty {
        0 => {
            let pos = va.vec3();
            let g = l::ext::hsd_8039F05C(0, id / 1000, id);
            if !g.is_null() {
                (*g).pos = *pos;
            }
            g as Obj
        }
        1..=3 => {
            let pos = va.vec3();
            let f = va.f32();
            l::efLib_CreateGenerator_Translate_FacingDir(id, pos, f) as Obj
        }
        5 => {
            let g = l::ext::hsd_8039EFAC(0, id / 1000, id, va.jobj());
            if !g.is_null() {
                (*g).size *= (*jobj_of(gobj)).scale.x;
            }
            g as Obj
        }
        6 => {
            let j = va.jobj();
            efLib_CreateGenerator_AppSRT_SetFacingDir_va(id, j, va) as Obj
        }
        7 => l::efLib_CreateGenerator_Attach_Scale(id, va.jobj(), gobj) as Obj,
        _ => core::ptr::null_mut(),
    }
}

unsafe fn attach_scale_dir(id: u32, gobj: G, va: &mut Va) -> Obj {
    let e = l::efLib_Create_Attach_Scale(id, gobj, va.jobj());
    if !e.is_null() {
        set_rot_y(ejobj(e), half_pi(va.f32()));
        l::ext::HSD_JObjAnimAll(ejobj(e));
    }
    e as Obj
}

unsafe fn facing_rotz(id: u32, gobj: G, va: &mut Va) -> Obj {
    let e = l::efLib_Create_Attach_Scale_FacingDir(id, gobj, va.jobj());
    if !e.is_null() {
        set_rot_z(ejobj(e), va.f32());
    }
    e as Obj
}

unsafe fn scale_dir_noanim(id: u32, gobj: G, va: &mut Va) -> Obj {
    let e = l::efLib_Create_Attach_Scale(id, gobj, va.jobj());
    if !e.is_null() {
        set_rot_y(ejobj(e), half_pi(va.f32()));
    }
    e as Obj
}

/// Generator whose appsrt is placed at the vararg and scaled by the spawner's scale.y.
unsafe fn gen_scaled(id: u32, gobj: G, va: &mut Va, rot_y: bool) -> Obj {
    let g = l::efLib_CreateGenerator_AddAppSRT(id as i32);
    if !g.is_null() {
        (*(*g).appsrt).translate = *va.vec3();
        let s = get_scale(jobj_of(gobj));
        let a = &mut *(*g).appsrt;
        a.scale.z = s.y;
        a.scale.y = s.y;
        a.scale.x = s.y;
        if rot_y {
            a.rot.y = M_PI_2 as f32;
        }
    }
    g as Obj
}

/// The 12-particle burst of 0x4CF / 0x4D0.
pub unsafe fn burst(gfx_id: i32, gobj: G, va: &mut Va) -> Obj {
    let translate = *va.vec3();
    let scale = if gfx_id == 0x4CF { va.f32() } else { 1.0f32 };
    let mut ret: *mut EF_Effect = core::ptr::null_mut();
    let mut last: *mut EF_Effect = core::ptr::null_mut();
    let mut cnt = 0;
    loop {
        let id = if ext::HSD_Randf() < 0.5 {
            if gfx_id == 0x4D0 { 0x1C } else { 0x1E }
        } else if gfx_id == 0x4D0 {
            0x1D
        } else {
            0x1F
        };
        let e = l::efLib_Create_Attach_Pos(id, gobj, &translate);
        if e.is_null() {
            break;
        }
        (*e).update = Some(l::efLib_Cb_SetOffset_FromParams);
        (*e).lifetime = 0x32;
        let ry = (M_TAU * ext::HSD_Randf() as f64) as f32;
        let rx = (M_TAU * ext::HSD_Randf() as f64) as f32;
        let j = ejobj(e);
        HSD_JObjSetScaleX(&mut *j, scale, &mut Dirty);
        HSD_JObjSetScaleY(&mut *j, scale, &mut Dirty);
        HSD_JObjSetScaleZ(&mut *j, scale, &mut Dirty);
        set_rot_y(j, ry);
        HSD_JObjSetRotationX(&mut *j, rx, &mut Dirty);
        let p = burst_params(rx, ry);
        (*e).params = p;
        if cnt != 0 {
            (*last).next = e;
            last = (*last).next;
        } else {
            ret = e;
            last = e;
        }
        cnt += 1;
        if cnt >= 0xC {
            break;
        }
    }
    ret as Obj
}

/// The burst particle velocity from its random angles (rot_x, rot_y).
pub fn burst_params(rx: f32, ry: f32) -> Vec3 {
    let px = sinf(ry);
    let t = 2.0f32 * cosf(rx);
    let x = t * px;
    let y = 2.0f32 * sinf(rx);
    let py = cosf(ry);
    let t = 2.0f32 * cosf(rx);
    Vec3 { x, y, z: t * py }
}

/// `efSync_Spawn`.
pub unsafe fn efSync_Spawn(mut gfx_id: i32, gobj: G, va: &mut Va) -> Obj {
    EF_DATA.load_kind = 0;
    EF_DATA.anim_count = 0;
    if gfx_id == 0x479 && l::ext::efAsync_DatEntries_data(1).is_null() {
        gfx_id = 0x506;
    }
    if gfx_id < 0x250 || gfx_id / 1000 == 0x1E {
        return l::efLib_CreateGenerator(gfx_id, va.vec3()) as Obj;
    }
    if gfx_id < 0x478 {
        return ext::efAsync_Dispatch(gfx_id, gobj, va);
    }
    if gfx_id < 0x4BA {
        return ext::efAlt_Spawn(gfx_id, gobj, va);
    }
    EF_DATA.load_kind = EF_LOADKIND_SYNC as u32;
    let ret: Obj = if (EF_MEX_MDL_START..EF_MEX_END).contains(&gfx_id) {
        efSync_MexSpawn(gfx_id, gobj, va)
    } else {
        spawn_switch(gfx_id, gobj, va)
    };
    while EF_DATA.anim_count != 0 {
        let c = EF_DATA.anim_count - 1;
        EF_DATA.anim_count = c;
        l::ext::HSD_JObjAnimAll(efLib_AnimQueue[c as usize]);
    }
    ret
}

unsafe fn spawn_switch(gfx_id: i32, gobj: G, va: &mut Va) -> Obj {
    use l::*;
    let n = core::ptr::null_mut::<core::ffi::c_void>();
    let fx = |g: Gen| g as Obj;
    let ef = |e: *mut EF_Effect| e as Obj;
    match gfx_id {
        0x4BB | 0x4BC | 0x50B => {
            let (a, b) = match gfx_id {
                0x4BB => (0x1770, 0x1772),
                0x4BC => (0x1771, 0x1773),
                _ => (0x4A38, 0x4A39),
            };
            let r = efLib_Create_Attach_Scale_FacingDir(a, gobj, va.jobj());
            if !r.is_null() {
                (*r).next = if gfx_id == 0x50B {
                    efLib_Create_Attach_Scale_FacingDir(b, gobj, va.jobj())
                } else {
                    efLib_Create_AttachChild_Scale(b, gobj, va.jobj())
                };
            }
            ef(r)
        }
        0x4BD => fx(l::ext::hsd_8039EFAC(0, 7, 0x1B58, va.jobj())),
        0x4BE => fx(l::ext::hsd_8039EFAC(0, 7, 0x1B5C, va.jobj())),
        0x4BF => {
            let j = va.jobj();
            let r = l::ext::hsd_8039EFAC(0, 7, 0x1B5D, j);
            l::ext::hsd_8039EFAC(0, 0, 0x5F, j);
            fx(r)
        }
        0x4C0 => ef(efLib_Create_Attach(0x1B58, gobj, va.jobj())),
        0x4C1 => facing_rotz(0x1B59, gobj, va),
        0x4C2 => facing_rotz(0x1B5A, gobj, va),
        0x4C3 => {
            let g = efLib_CreateGenerator_AddAppSRT(0x24C);
            if g.is_null() { n } else {
                (*(*g).appsrt).translate = *va.vec3();
                fx(g)
            }
        }
        0x4C4 => facing_rotz(0x1B5B, gobj, va),
        0x4C5 => facing_rotz(0x1B5C, gobj, va),
        0x4C6 => ef(efLib_Create_Attach_Scale_FacingDir(0x1F40, gobj, va.jobj())),
        0x4C7 => ef(efLib_Create_Attach_Scale_FacingDir(0x1F41, gobj, va.jobj())),
        0x4C8 => attach_scale_dir(0x1F42, gobj, va),
        0x4C9 => attach_scale_dir(0x1F43, gobj, va),
        0x4CA => ef(efLib_Create_Attach_Scale(0x1F44, gobj, va.jobj())),
        0x4CB => ef(efLib_Create_Attach_Scale(0x1F45, gobj, va.jobj())),
        0x4CC => {
            let r = efLib_Create_Attach_Pos(0x1F46, gobj, va.vec3());
            if !r.is_null() {
                let s = get_scale(jobj_of(gobj));
                set_scale(ejobj(r), &s);
                (*r).update = Some(efLib_Cb_SetRotY_FromFighterDir);
            }
            ef(r)
        }
        0x4CD => ef(efLib_Create_Attach(0x2328, gobj, va.jobj())),
        0x4CE => fx(efLib_CreateGenerator(0x2328, va.vec3())),
        0x4CF | 0x4D0 => burst(gfx_id, gobj, va),
        0x4EE => ef(efLib_Create_Attach(0x2710, gobj, va.jobj())),
        0x4EF => ef(efLib_Create_Attach(0x2711, gobj, va.jobj())),
        0x4F0 => ef(efLib_Create_Attach_Scale(0x2712, gobj, va.jobj())),
        0x4D5 => fx(l::ext::hsd_8039EFAC(0, 0xB, 0x2AF8, va.jobj())),
        0x4D6 => ef(efLib_Create_Attach_Scale(0x2AF8, gobj, va.jobj())),
        0x4D7 => fx(l::ext::hsd_8039EFAC(0, 0xB, 0x2AF9, va.jobj())),
        0x4D8 => gen_scaled(0x61, gobj, va, true),
        0x4D9 => {
            let r = efLib_Create_Attach_Pos(0x2EE0, gobj, va.vec3());
            if !r.is_null() {
                set_rot_y(ejobj(r), half_pi(va.f32()));
                let s = get_scale(jobj_of(gobj));
                set_scale(ejobj(r), &s);
            }
            ef(r)
        }
        0x4DA => {
            let j = va.jobj();
            let r = efLib_Create_Attach_Scale(0x2EE1, gobj, j);
            if !r.is_null() {
                let e1 = efLib_Create_Attach_Scale(0x2EE2, gobj, j);
                (*r).next = e1;
                if !e1.is_null() {
                    (*r).update = Some(efLib_Cb_ftKp_SpecialHi);
                }
            }
            ef(r)
        }
        0x4DB => fx(efLib_CreateGenerator_Attach_Scale(0x2EE5, va.jobj(), gobj)),
        0x4DC => fx(efLib_CreateGenerator_Attach_Scale(0x2EE6, va.jobj(), gobj)),
        0x4DD => fx(efLib_CreateGenerator_Attach_Scale(0x2EE7, va.jobj(), gobj)),
        0x4DE => fx(efLib_CreateGenerator_Attach_Scale(0x2EE8, va.jobj(), gobj)),
        0x4DF => fx(efLib_CreateGenerator_Attach_Scale(0x143, va.jobj(), gobj)),
        0x4E0 => ef(efLib_Create_Attach(0x32C8, gobj, va.jobj())),
        0x4E1 => {
            let r = efLib_Create(0x32C9, gobj);
            if !r.is_null() {
                let mut cursor = va.jobj();
                (*r).attach_jobj = cursor;
                (*r).update = Some(efLib_Cb_SetOffsetY_FromParamY);
                while !HSD_JObjGetParent(cursor.as_ref()).is_null() {
                    cursor = HSD_JObjGetParent(cursor.as_ref());
                }
                let s = get_scale(cursor);
                (*r).params = *va.vec3();
                (*r).params.y *= s.y;
            }
            ef(r)
        }
        0x4E2 => ef(efLib_Create_Attach_Pos(0x32CA, gobj, va.vec3())),
        0x4E3 => fx(efLib_CreateGenerator(0x18A, va.vec3())),
        0x4E4 => fx(efLib_CreateGenerator(0x194, va.vec3())),
        0x4E5 => fx(l::ext::hsd_8039EFAC(0, 0, 0x17D, va.jobj())),
        0x4E6 => fx(l::ext::hsd_8039EFAC(0, 0, 0x17E, va.jobj())),
        0x4E7 => fx(efLib_CreateGenerator(0x196, va.vec3())),
        0x4E8 => ef(efLib_Create_Attach_Pos(0x32CB, gobj, va.vec3())),
        0x4E9 => fx(l::ext::hsd_8039EFAC(0, 0xE, 0x36B0, va.jobj())),
        0x4EA => fx(l::ext::hsd_8039EFAC(0, 0xE, 0x36B1, va.jobj())),
        0x4EB | 0x4F1 | 0x510 => {
            let id = match gfx_id { 0x4EB => 0x36B6, 0x4F1 => 0x3E80, _ => 0xBF68 };
            let j = va.jobj();
            fx(efLib_CreateGenerator_AppSRT_SetFacingDir_va(id, j, va))
        }
        0x4EC => fx(l::ext::hsd_8039EFAC(0, 0xE, 0x36B7, va.jobj())),
        0x4ED => ef(efLib_Create_AttachChild(0x36B0, gobj, va.jobj())),
        0x4D1 => fx(l::ext::hsd_8039EFAC(0, 0, 0x64, va.jobj())),
        0x4D2 => ef(efLib_Create_Attach_Pos(0x3A98, gobj, va.vec3())),
        0x4D3 => {
            let j = va.jobj();
            let r = efLib_CreateGenerator_Attach_AddAppSRT(0x172, j);
            if !r.is_null() {
                efLib_CreateGenerator_Attach_AddAppSRT(0x173, j);
            }
            fx(r)
        }
        0x4D4 => fx(efLib_CreateGenerator_Attach(0x11E, va.jobj())),
        0x4F2 => ef(efLib_Create_Attach_Scale_FacingDir(0x3E80, gobj, va.jobj())),
        0x4F3 => ef(efLib_Create_Attach_Scale_FacingDir(0x3E81, gobj, va.jobj())),
        0x4F4 | 0x4F5 => {
            let r = efLib_Create_AttachChild(if gfx_id == 0x4F4 { 0x4268 } else { 0x4269 }, gobj, va.jobj());
            if !r.is_null() {
                let s = get_scale(jobj_of(gobj));
                if ejobj(r).is_null() {
                    panic!("jobj.h:823: jobj");
                }
                set_scale(ejobj(r), &s);
            }
            ef(r)
        }
        0x4F6 => ef(efLib_Create_Attach_Scale(0x426A, gobj, va.jobj())),
        0x4F7 => ef(efLib_Create_Attach_Scale(0x426B, gobj, va.jobj())),
        0x4F8 => fx(efLib_CreateGenerator_Attach_Scale(0x6E, va.jobj(), gobj)),
        0x4F9 => fx(efLib_CreateGenerator_Attach_Scale(0x1C8, va.jobj(), gobj)),
        0x4FA => fx(efLib_CreateGenerator_Attach_Scale(0x166, va.jobj(), gobj)),
        0x4FB => fx(l::ext::hsd_8039EFAC(0, 0, 0x71, va.jobj())),
        0x4FE => ef(efLib_Create_Attach(0x426C, gobj, va.jobj())),
        0x4FF | 0x500 => {
            let fp = (*gobj).user_data as *mut Fighter;
            ef(efLib_Create_Attach_Scale(0x426D, gobj, (*(*fp).parts.add(1)).joint))
        }
        0x501 => {
            let fp = (*gobj).user_data as *mut Fighter;
            let r = efLib_Create_Attach_Scale(0x426E, gobj, (*(*fp).parts.add(1)).joint);
            if !r.is_null() {
                set_rot_y(ejobj(r), half_pi((*fp).facing_dir));
                (*r).attach_jobj = (*(*fp).parts.add(85)).joint;
                (*r).update = Some(efLib_Cb_LifetimeEndSpawn);
                (*r).lifetime = 6;
            }
            ef(r)
        }
        0x502 => fx(l::ext::hsd_8039EFAC(0, 0, 0x1A6, va.jobj())),
        0x4FC => fx(l::ext::hsd_8039EFAC(0, 0x11, 0x426D, va.jobj())),
        0x4FD => fx(l::ext::hsd_8039EFAC(0, 0x11, 0x4271, va.jobj())),
        0x503 => fx(efLib_CreateGenerator(0x6A, va.vec3())),
        0x504 => gen_scaled(0x6D, gobj, va, false),
        0x505 => gen_scaled(0x79, gobj, va, false),
        0x506 => fx(efLib_CreateGenerator(0x4658, va.vec3())),
        0x507 => {
            let j = va.jobj();
            let r = efLib_Create_Attach(0x4650, gobj, j);
            if !r.is_null() {
                set_rot_y(ejobj(r), half_pi(va.f32()));
            }
            l::ext::hsd_8039EFAC(0, 0x12, 0x4650, j);
            ef(r)
        }
        0x508 => fx(efLib_CreateGenerator(0x4652, va.vec3())),
        0x509 => {
            let r = efLib_Create_Attach_Scale(0x4651, gobj, va.jobj());
            if !r.is_null() {
                (*r).update = Some(efLib_Cb_ftLg_SpecialLw);
            }
            ef(r)
        }
        0x50A => fx(l::ext::hsd_8039EFAC(0, 0, 0x5F, va.jobj())),
        0x50C => {
            let r = efLib_Create_Attach(0x4A3A, gobj, va.jobj());
            if !r.is_null() {
                (*r).params.z = va.f32();
                (*r).update = Some(efLib_Cb_SetRotYZ_FromParamZ_FighterDir);
            }
            ef(r)
        }
        0x50D => ef(efLib_Create_Attach(0x4A3B, gobj, va.jobj())),
        0x50E => scale_dir_noanim(0x4A3C, gobj, va),
        0x50F => scale_dir_noanim(0x4A3D, gobj, va),
        0x511 => ef(efLib_Create_Attach_Scale_FacingDir(0xBF68, gobj, va.jobj())),
        0x512 => ef(efLib_Create_Attach_Scale_FacingDir(0xBF69, gobj, va.jobj())),
        _ => n,
    }
}

/// `efLib_CreateGenerator_AppSRT_SetFacingDir(id, vlist)`: the f32* is read only when the
/// generator exists (the C's `va_arg` sits inside the `if`).
unsafe fn efLib_CreateGenerator_AppSRT_SetFacingDir_va(id: i32, j: J, va: &mut Va) -> Gen {
    let g = l::efLib_CreateGenerator_AppSRT_SetFacingDir(id, j, 0.0);
    if !g.is_null() {
        (*(*g).appsrt).rot.y = half_pi(va.f32());
    }
    g
}

