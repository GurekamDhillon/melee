#![allow(non_snake_case, non_upper_case_globals)]
//! Native port of src/melee/ft/kinds/ftCommon/ftCo_CaptureKirby.c (being inhaled by Kirby).
//! Fused sites (tools/rust-port/fma_sites.py): ftCo_800BD19C's squared distance (two f32
//! fmuladd) and lbVector_Len's (melee_runtime::lbvector::len, verified there); lowered to fmadds.
//! ftCo_800BD19C and ftCo_800BD39C are checked bit-exact against the C by
//! tests/tu_ftCo_CaptureKirby.rs (gen/cref/tu_shard121.py).

use crate::types::*;
use melee_core::fp::fmadds;
use melee_types::inline as ji;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}
#[inline(always)]
fn jobj<'a>(g: G) -> &'a mut HSD_JObj {
    unsafe { &mut *((*g).hsd_obj as *mut HSD_JObj) }
}

/// Calls out of the TU. `hook::<name>` overrides; unset = the port, or a TODO panic.
pub mod ext {
    use super::*;
    macro_rules! ext_fns {
        ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
            pub mod hook {
                #[allow(unused_imports)] use super::*;
                $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } { Some(f) => f($($a),*), None => ($def)($($a),*) }
            } )*
        };
    }
    ext_fns! {
        ftKb_SpecialN_800F5AF0(g: G) -> f32 = |_| todo!("ftKb_SpecialN_800F5AF0: installed by melee_link");
        ftKb_SpecialN_800F5B20(g: G, out: &mut Vec2) = |_, _| todo!("ftKb_SpecialN_800F5B20: installed by melee_link");
        ftKb_SpecialN_800F5B3C(g: G) -> f32 = |_| todo!("ftKb_SpecialN_800F5B3C: installed by melee_link");
        ftKb_SpecialN_800F5B4C(g: G) -> f32 = |_| todo!("ftKb_SpecialN_800F5B4C: installed by melee_link");
        ftKb_SpecialN_800F5B5C(g: G, out: &mut Vec3) = |_, _| todo!("ftKb_SpecialN_800F5B5C: installed by melee_link");
        ftCo_ReleaseItemAndVictim(g: G) = crate::tu::ftco_800c7590::ftco_release_item_and_victim;
        Fighter_ChangeMotionState(g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G) = |g: G, msid: i32, flags: u32, start: f32, speed: f32, blend: f32, other: G| { unsafe { crate::tu::fighter::Fighter_ChangeMotionState(g as _, msid as _, flags as _, start as _, speed as _, blend as _, other as _); } };
        ftCommon_8007D5D4(f: &mut Fighter) = crate::ftcommon2::ftcommon_8007d5d4;
        ftCommon_InitGrab(f: &mut Fighter, a: i32, timer: f32) = crate::ftcommon2::init_grab;
        ftCommon_8007E2F4(f: &mut Fighter, v: i16) = crate::ftcommon2::ftcommon_8007e2f4;
        ftAnim_8006EBA4(g: G) = |g: G| { unsafe { crate::tu::ftanim::ftAnim_8006EBA4(g as _); } };
        ftCommon_8007EBAC(f: &mut Fighter, a: u32, b: u32) = crate::ftcommon2::ftcommon_8007ebac;
        ftCommon_8007E2FC(g: G) = |a0| crate::tu::ftcommon::ftcommon_8007e2fc(unsafe { &mut *((*(a0 as *mut HSD_GObj)).user_data as *mut Fighter) });
        HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj) = |a0| unsafe { melee_runtime::tu::jobj_ptr::HSD_JObjSetMtxDirtySub(a0 as _) };
    }
}

struct Dirty;
impl ji::JObjDirty for Dirty {
    fn set_mtx_dirty(&mut self, j: &mut HSD_JObj) {
        if !ji::HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    fn set_mtx_dirty_sub(&mut self, j: &mut HSD_JObj) {
        ext::HSD_JObjSetMtxDirtySub(j);
    }
}

#[inline(always)]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// `ftCo_800BD19C`: squared distance from the fighter to `pos`.
pub fn ftCo_800BD19C(g: G, pos: &Vec3) -> f32 {
    melee_core::ct!("ftCo_800BD19C");
    let f = fp(g);
    let dx = f.cur_pos.x - pos.x;
    let dy = f.cur_pos.y - pos.y;
    let dz = f.cur_pos.z - pos.z;
    // squares of computed expressions: not fused on the console (fmuls x3, fadds x2; notes/fp-audit.md)
    dx * dx + dy * dy + dz * dz
}

/// `ftCo_800BD1DC`: enter CaptureKirby.
pub fn ftCo_800BD1DC(g: G, vg: G) {
    melee_core::ct!("ftCo_800BD1DC");
    let f = fp(g);
    let vf = fp(vg);
    ext::ftCo_ReleaseItemAndVictim(g);
    f.x1A5C = vg;
    f.victim_gobj = vg;
    f.x221B.set_x221B_b5(0);
    f.x221B.set_x221B_b7(0);
    f.facing_dir = -vf.facing_dir;
    ext::Fighter_ChangeMotionState(g, ftCo_MS_CaptureKirby as i32, Ft_MF_None as u32, 0.0, 1.0, 0.0, core::ptr::null_mut());
    let f = fp(g);
    ext::ftCommon_8007D5D4(f);
    // inlineA0
    {
        let v = f.victim_gobj;
        f.mv.co.capturekirby.x8.x = ext::ftKb_SpecialN_800F5B4C(v);
        f.mv.co.capturekirby.x8.y = ext::ftKb_SpecialN_800F5B3C(fp(g).victim_gobj);
        let mut v0 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ext::ftKb_SpecialN_800F5B5C(fp(g).victim_gobj, &mut v0);
        let f = fp(g);
        f.mv.co.capturekirby.pos_offset.x = f.cur_pos.x - v0.x;
        f.mv.co.capturekirby.pos_offset.y = f.cur_pos.y - v0.y;
        let mut v1 = Vec2 { x: 0.0, y: 0.0 };
        ext::ftKb_SpecialN_800F5B20(f.victim_gobj, &mut v1);
        let f = fp(g);
        f.mv.co.capturekirby.x10.x = v1.x;
        f.mv.co.capturekirby.x10.y = v1.y;
        let mut s = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        ji::HSD_JObjGetScale(jobj(g), &mut s);
        f.mv.co.capturekirby.scale.x = s.x;
        f.mv.co.capturekirby.scale.y = s.y;
        f.mv.co.capturekirby.scale.z = s.z;
        f.set_x2226_b2(1);
    }
    let t = ext::ftKb_SpecialN_800F5AF0(fp(g).victim_gobj);
    let f = fp(g);
    ext::ftCommon_InitGrab(f, 0, t);
    ext::ftCommon_8007E2F4(f, 0x1FF);
    ext::ftAnim_8006EBA4(g);
    ext::ftCommon_8007EBAC(fp(g), 1, 0);
    ext::ftCommon_8007E2FC(g);
    fp(g).accessory1_cb = Some(ftCo_800BD39C_cb);
}

/// `ftCo_CaptureKirby_Anim`.
pub fn ftCo_CaptureKirby_Anim(_g: G) { melee_core::ct!("ftCo_CaptureKirby_Anim");}
/// `ftCo_CaptureKirby_IASA`.
pub fn ftCo_CaptureKirby_IASA(_g: G) { melee_core::ct!("ftCo_CaptureKirby_IASA");}
/// `ftCo_CaptureKirby_Phys`.
pub fn ftCo_CaptureKirby_Phys(_g: G) { melee_core::ct!("ftCo_CaptureKirby_Phys");}
/// `ftCo_CaptureKirby_Coll`.
pub fn ftCo_CaptureKirby_Coll(_g: G) { melee_core::ct!("ftCo_CaptureKirby_Coll");}

/// `ftCo_800BD39C`: shrink toward Kirby's mouth and follow it (accessory1 callback).
pub fn ftCo_800BD39C(g: G) {
    melee_core::ct!("ftCo_800BD39C");
    let f = fp(g);
    let mut pos = unsafe {
        Vec3 { x: f.mv.co.capturekirby.pos_offset.x, y: f.mv.co.capturekirby.pos_offset.y, z: 0.0 }
    };
    unsafe {
        // inlineB0
        let dist = melee_runtime::lbvector::len(&*(&pos as *const Vec3 as *const _));
        let x8x = f.mv.co.capturekirby.x8.x;
        if dist < x8x {
            let mut q = dist / x8x;
            let d = f.mv.co.capturekirby.x8.y;
            q *= d;
            let s = (1.0f32 - d) + q;
            let sc = Vec3 {
                x: s * f.mv.co.capturekirby.scale.x, // C: mv.co.guard.x2C (same GC offset fp+236C; native union arms differ)
                y: s * f.mv.co.capturekirby.scale.y,
                z: s * f.mv.co.capturekirby.scale.z,
            };
            ji::HSD_JObjSetScale(jobj(g), &sc, &mut Dirty);
        }
        // inlineB1
        let k = &mut f.mv.co.capturekirby;
        if abs_c(pos.x) > k.x10.x {
            pos.x = k.x10.x * (if pos.x < 0.0 { -1 } else { 1 }) as f32;
        }
        if abs_c(pos.y) > k.x10.y {
            pos.y = k.x10.y * (if pos.y < 0.0 { -1 } else { 1 }) as f32;
        }
        k.pos_offset.x -= pos.x;
        k.pos_offset.y -= pos.y;
        ext::ftKb_SpecialN_800F5B5C(f.victim_gobj, &mut pos);
        let f = fp(g);
        f.cur_pos.x = pos.x + f.mv.co.capturekirby.pos_offset.x;
        f.cur_pos.y = pos.y + f.mv.co.capturekirby.pos_offset.y;
        f.cur_pos.z = pos.z;
    }
}
/// `ftCo_800BD39C` as an `HSD_GObjEvent`.
pub unsafe extern "C" fn ftCo_800BD39C_cb(g: G) {
    melee_core::ct!("ftCo_800BD39C_cb");
    ftCo_800BD39C(g)
}
