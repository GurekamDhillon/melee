//! src/melee/ft/ftdrawcommon.c: fighter render callbacks (model, debug collision overlays, FoD
//! reflection). TARGET_PC paths kept: the Workbench flag override (`FT_DRAW_FLAG`), the m-ex
//! onModelRender dispatch, the surface/toon/motion wrapper of ftDrawCommon_80080E18 and the
//! "disable FoD reflection" patch. Float math: the view-offset add (`v + 0`, plain adds) and
//! PSMTXConcat / PSMTXMultVec from `melee_runtime::mtx` (retail dataflow). GX/HSD/lbColl/lbGx
//! callees are TODO stubs in `ext`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]

use crate::types::*;
use melee_runtime::mtx as m;

pub type G = *mut HSD_GObj;

#[inline(always)]
fn fp<'a>(g: G) -> &'a mut Fighter {
    unsafe { &mut *((*g).user_data as *mut Fighter) }
}

/// Callees / globals in TUs not ported yet (settable; unset panics). TODO rewire.
pub mod ext {
    use super::G;
    use crate::types::*;
    use core::ffi::c_void;
    macro_rules! stubs {
        ($($stat:ident $name:ident ($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $d:expr)?;)*) => {$(
            pub static mut $stat: Option<fn($($t),*) $(-> $r)?> = None;
            #[allow(unreachable_code)]
            pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { $stat } {
                    Some(f) => f($($a),*),
                    None => { $( return $d; )? panic!(concat!("TODO stub: ", stringify!($name))) }
                }
            }
        )*};
    }
    stubs! {
        WB_RENDER_ACTIVE Wb_RenderActive() -> i32;
        WB_RENDER_OPTION Wb_RenderOption(opt: i32, fallback: i32) -> i32;
        FTLIB_80087074 ftLib_80087074(g: G, out: *mut Vec3) -> i32;
        FTLIB_80086A8C ftLib_80086A8C(g: G) -> i32;
        FTLIB_800860C4 ftLib_800860C4() -> i32;
        HSD_COBJ_GET_CURRENT HSD_CObjGetCurrent() -> *mut HSD_CObj;
        HSD_COBJ_GET_INV_VIEWING_MTX_PTR HSD_CObjGetInvViewingMtxPtr(c: *mut HSD_CObj) -> *mut [f32; 4];
        CAMERA_800310B8 Camera_800310B8() -> *mut HSD_CObj;
        CAMERA_80031060 Camera_80031060() -> i32;
        LBCOLL_80009F54 lbColl_80009F54(h: *mut HitCapsule, arg: i32, scale: f32) -> i32;
        LBCOLL_8000A044 lbColl_8000A044(h: *mut HitCapsule, arg: i32, scale: f32) -> i32;
        LBCOLL_8000A1A8 lbColl_8000A1A8(h: *mut Fighter_x1614_t, arg: i32, scale: f32) -> i32;
        LBCOLL_8000A244 lbColl_8000A244(h: *mut HurtCapsule, arg: i32, mtx: *mut [f32; 4], z: f32) -> i32;
        LBCOLL_8000A460 lbColl_8000A460(h: *mut Fighter_x1670_t, arg: i32) -> i32;
        LBCOLL_8000A584 lbColl_8000A584(h: *mut HurtCapsule, kind: u32, arg: i32, mtx: *mut [f32; 4], z: f32) -> i32;
        LBCOLL_8000A78C lbColl_8000A78C(h: *mut HitResult, arg: i32, mtx: *mut [f32; 4], z: f32) -> i32;
        LBCOLL_8000A95C lbColl_8000A95C(h: *mut HitResult, arg: i32, mtx: *mut [f32; 4], z: f32) -> i32;
        LBCOLL_8000AB2C lbColl_8000AB2C(h: *mut HitResult, arg: i32, mtx: *mut [f32; 4], z: f32) -> i32;
        FTCOMMON_8007F804 ftCommon_8007F804(fp: *mut Fighter) -> *mut [f32; 4];
        LB_800117F4 lb_800117F4(d: *mut DynamicsDesc, c0: *mut GXColor, c1: *mut GXColor, bone: i32, arg: i32) -> i32;
        FTCO_800B395C ftCo_800B395C(g: G, arg: i32) -> i32;
        LB_80014770 lb_80014770(v: *mut Vec3, arg: i32) -> i32;
        LBGX_8001E2F8 lbGx_8001E2F8(q: *mut Quaternion, pos: *mut Vec3, c: *const U8Vec4, arg: i32, facing: f32) -> i32;
        HSD_STATE_INVALIDATE HSD_StateInvalidate(s: i32);
        FTPARTS_800750C8 ftParts_800750C8(fp: *mut Fighter, i: i32, b: i32);
        FTCO_8009F5AC ftCo_8009F5AC(fp: *mut Fighter) = unsafe { crate::tu::ftCo_09f4::ftCo_8009F5AC(fp) };
        FTCO_8009F7F8 ftCo_8009F7F8(fp: *mut Fighter) = unsafe { crate::tu::ftCo_09f4::ftCo_8009F7F8(fp) };
        FTCO_800C8AF0 ftCo_800C8AF0(fp: *mut Fighter);
        FTCO_800C2600 ftCo_800C2600(g: G, arg: i32);
        HSD_JOBJ_DISP_ALL HSD_JObjDispAll(j: *mut HSD_JObj, mtx: *mut [f32; 4], flags: u32, rendermode: u32);
        HSD_GOBJ_80390EB8 HSD_GObj_80390EB8(arg: i32) -> u32;
        // `Mex_OnModelRenderDispatch(kind, gobj, arg1, mtx, ftData_UnkMtxFunc0[kind])`.
        MEX_ON_MODEL_RENDER Mex_OnModelRenderDispatch_UnkMtxFunc0(kind: i32, g: G, arg1: i32, mtx: *mut [f32; 4]);
        HSD_JOBJ_SET_MTX_DIRTY_SUB HSD_JObjSetMtxDirtySub(j: *mut HSD_JObj);
        SCRIPT_MOTION_DRAW_BEGIN script_motion_draw_begin(g: G) -> i32;
        MOTION_FIGHTER_END MotionFighterEnd();
        GENO_TOON_ENSURE GenoToonEnsure(slot: i32, kind: i32);
        SURFACE_BEGIN SurfaceBegin(slot: i32);
        SURFACE_END SurfaceEnd();
        MEX_ENABLED_DISABLE_FOD Mex_Enabled_disable_fod_reflection() -> i32;
        // `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]`.
        PLINK_FIGHTER_HEAD plink_fighter_head() -> G;
    }
    #[allow(dead_code)]
    fn _unused(_: *mut c_void) {}
}

const WB_RENDER_MODEL: i32 = 0;

/// `FT_DRAW_FLAG(fp, bit)` on TARGET_PC (`ftDrawCommon_RenderFlag`): MSB-first bit of x21FC_flag,
/// unless Workbench is drawing, where only the model (bit 7) is drawn, on its option.
pub fn draw_flag(f: &Fighter, bit: i32) -> bool {
    let fallback = (unsafe { f.x21FC_flag.byte } as i32 & (1 << (7 - bit))) != 0;
    if ext::Wb_RenderActive() == 0 {
        return fallback;
    }
    if bit == 7 { ext::Wb_RenderOption(WB_RENDER_MODEL, fallback as i32) != 0 } else { false }
}

static ftDrawCommon_804D3A88: U8Vec4 = U8Vec4 { x: 0xFF, y: 0xFF, z: 0xFF, w: 0x80 };
static ftDrawCommon_804D3A8C: U8Vec4 = U8Vec4 { x: 0x80, y: 0x80, z: 0xFF, w: 0x80 };
static ftDrawCommon_804D3A90: U8Vec4 = U8Vec4 { x: 0x80, y: 0x80, z: 0x80, w: 0x80 };

/// `ftDrawCommon_8008051C`: view matrix offset to the fighter's draw offset, or NULL.
pub fn ftDrawCommon_8008051C(g: G, out: &mut Mtx) -> Option<*mut [f32; 4]> {
    let v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut sp54 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    if ext::ftLib_80087074(g, &mut sp54) != 0 {
        let cur = ext::HSD_CObjGetCurrent();
        let mut t = m::identity();
        t[0][3] = sp54.x + v.x;
        t[1][3] = sp54.y + v.y;
        t[2][3] = sp54.z + v.z;
        *out = m::concat(unsafe { &(*cur).view_mtx }, &t);
        return Some(out.as_mut_ptr());
    }
    None
}

fn opt(p: Option<*mut [f32; 4]>) -> *mut [f32; 4] {
    p.unwrap_or(core::ptr::null_mut())
}

/// `ftDrawCommon_800805C8`: debug overlays, then (bit 7) the model itself.
pub fn ftDrawCommon_800805C8(g: G, arg1: i32, arg2: bool) {
    let f = fp(g);
    let fpp: *mut Fighter = f;
    let mut inv = false;
    if draw_flag(f, 6) {
        for i in 0..f.x914.len() {
            if ext::lbColl_80009F54(&mut f.x914[i], arg1, f.x34_scale.y) != 0 {
                inv = true;
            }
        }
        let n = f.hurt_capsules_len as usize;
        if f.x221D_b6() != 0 {
            for i in 0..n {
                if ext::lbColl_8000A584(&mut f.hurt_capsules[i].capsule, 1, arg1, ext::ftCommon_8007F804(fpp), f.cur_pos.z) != 0 {
                    inv = true;
                }
            }
        } else {
            let f2 = f.x1988;
            if f2 == 0 && f.x198C == 0 {
                for i in 0..n {
                    if ext::lbColl_8000A244(&mut f.hurt_capsules[i].capsule, arg1, ext::ftCommon_8007F804(fpp), f.cur_pos.z) != 0 {
                        inv = true;
                    }
                }
            } else {
                let k: u32 = if f.x198C == 2 || f2 == 2 { 2 } else { 1 };
                for i in 0..n {
                    if ext::lbColl_8000A584(&mut f.hurt_capsules[i].capsule, k, arg1, ext::ftCommon_8007F804(fpp), f.cur_pos.z) != 0 {
                        inv = true;
                    }
                }
            }
        }
        if f.reflecting() != 0 && ext::lbColl_8000A95C(&mut f.reflect_hit, arg1, ext::ftCommon_8007F804(fpp), f.cur_pos.z) != 0 {
            inv = true;
        }
        if f.x2218_b6() != 0 && ext::lbColl_8000AB2C(&mut f.absorb_hit, arg1, ext::ftCommon_8007F804(fpp), f.cur_pos.z) != 0 {
            inv = true;
        }
        if f.anon_221b.x221B_b0() != 0 && ext::lbColl_8000A78C(&mut f.shield_hit, arg1, ext::ftCommon_8007F804(fpp), f.cur_pos.z) != 0 {
            inv = true;
        }
    }
    if draw_flag(f, 5) {
        for i in 0..f.x166C as usize {
            if ext::lbColl_8000A460(unsafe { f.x1670.as_mut_ptr().add(i) }, arg1) != 0 {
                inv = true;
            }
        }
        for i in 0..f.dynamics_num.max(0) as u32 as usize {
            let mut c0 = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFE };
            let mut c1 = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };
            let bone = f.dynamic_bone_sets[i].bone_id;
            if ext::lb_800117F4(&mut f.dynamic_bone_sets[i].dyn_desc, &mut c0, &mut c1, bone, arg1) != 0 {
                inv = true;
            }
        }
    }
    if draw_flag(f, 3) && ext::ftCo_800B395C(g, arg1) != 0 {
        inv = true;
    }
    if draw_flag(f, 4) && f.x2223_b5() != 0 && ext::lb_80014770(&mut f.dmg.x1930.x0, arg1) != 0 {
        inv = true;
    }
    if draw_flag(f, 2) {
        let p = &mut f.x294_itPickup;
        if f.ground_or_air as i32 == 0 {
            if ext::lbGx_8001E2F8(&mut p.gr_light_offset, &mut f.cur_pos, &ftDrawCommon_804D3A88, arg1, f.facing_dir) != 0 {
                inv = true;
            }
            if ext::lbGx_8001E2F8(&mut p.gr_heavy_offset, &mut f.cur_pos, &ftDrawCommon_804D3A90, arg1, f.facing_dir) != 0 {
                inv = true;
            }
        } else if ext::lbGx_8001E2F8(&mut p.air_light_offset, &mut f.cur_pos, &ftDrawCommon_804D3A8C, arg1, f.facing_dir) != 0 {
            inv = true;
        }
    }
    if draw_flag(f, 1) && f.x2227_b2() == 0 && ext::lbColl_8000A044(&mut f.x1064_thrownHitbox, arg1, f.x34_scale.y) != 0 {
        inv = true;
    }
    if draw_flag(f, 0) && f.x2229_b4() == 0 {
        for i in 0..f.x1614.len() {
            if ext::lbColl_8000A1A8(&mut f.x1614[i], arg1, f.x34_scale.y) != 0 {
                inv = true;
            }
        }
    }
    if inv {
        ext::HSD_StateInvalidate(-1);
    }
    if !draw_flag(f, 7) {
        return;
    }
    if f.invisible() == 0 && f.x221E_b5() == 0 && arg2 {
        ext::ftParts_800750C8(fpp, 1, 0);
        ext::ftParts_800750C8(fpp, 4, 0);
        if f.is_metal() != 0 || f.x2226_b5() != 0 || f.x2227_b3() != 0 {
            ext::ftParts_800750C8(fpp, 0, 0);
            ext::ftParts_800750C8(fpp, 2, 1);
        } else {
            ext::ftParts_800750C8(fpp, 2, 0);
            ext::ftParts_800750C8(fpp, 0, 1);
        }
        ext::ftCo_8009F5AC(fpp);
        f.set_x2223_b2(0);
        f.set_x2227_b7(1);
        f.set_x2228_b0(0);
        let mut sp78: Mtx = [[0.0; 4]; 3];
        let mtx = opt(ftDrawCommon_8008051C(g, &mut sp78));
        ext::HSD_JObjDispAll(unsafe { (*g).hsd_obj } as *mut HSD_JObj, mtx, ext::HSD_GObj_80390EB8(arg1), 0);
        ext::Mex_OnModelRenderDispatch_UnkMtxFunc0(f.kind as i32, g, arg1, mtx);
        ext::ftCo_800C8AF0(fpp);
        ext::ftCo_8009F7F8(fpp);
    }
    if !f.x20A0_accessory.is_null() {
        ext::HSD_JObjDispAll(f.x20A0_accessory, core::ptr::null_mut(), ext::HSD_GObj_80390EB8(arg1), 0);
    }
    ext::ftCo_800C2600(g, arg1);
}

/// `ftDrawCommon_80080C28`: the FoD floor-reflection render callback.
pub fn ftDrawCommon_80080C28(g: G, flag_index: i32) {
    let f = fp(g);
    let fpp: *mut Fighter = f;
    if !draw_flag(f, 7) {
        return;
    }
    if f.invisible() == 0 && f.x221E_b5() == 0 && f.x2226_b5() == 0 {
        ext::ftCo_8009F5AC(fpp);
        if !f.x5AC.xC[1].is_null() {
            ext::ftParts_800750C8(fpp, 0, 0);
            ext::ftParts_800750C8(fpp, 2, 0);
            ext::ftParts_800750C8(fpp, 4, 0);
            ext::ftParts_800750C8(fpp, 1, 1);
        }
        f.set_x2223_b2(0);
        f.set_x2223_b3(0);
        f.set_x2227_b7(0);
        f.set_x2228_b0(1);
        let mut sp70: Mtx = [[0.0; 4]; 3];
        let vmtx = opt(ftDrawCommon_8008051C(g, &mut sp70));
        ext::HSD_JObjDispAll(unsafe { (*g).hsd_obj } as *mut HSD_JObj, vmtx, ext::HSD_GObj_80390EB8(flag_index), 0);
        ext::Mex_OnModelRenderDispatch_UnkMtxFunc0(f.kind as i32, g, flag_index, vmtx);
        ext::ftCo_800C8AF0(fpp);
        ext::ftCo_8009F7F8(fpp);
    }
    if !f.x20A0_accessory.is_null() {
        ext::HSD_JObjDispAll(f.x20A0_accessory, core::ptr::null_mut(), ext::HSD_GObj_80390EB8(flag_index), 0);
    }
}

/// `ftDrawCommon_80080E18_inline0/1`: the shadow/outline passes; `b3` is the x2223_b3 value.
fn pass(g: G, flag_index: i32, b3: u8) {
    let f = fp(g);
    let fpp: *mut Fighter = f;
    if draw_flag(f, 7) && f.invisible() == 0 && f.x221E_b5() == 0 && f.x2226_b5() == 0 {
        ext::ftParts_800750C8(fpp, 0, 0);
        ext::ftParts_800750C8(fpp, 1, 0);
        ext::ftParts_800750C8(fpp, 2, 0);
        ext::ftParts_800750C8(fpp, 4, 1);
        f.set_x2223_b2(1);
        f.set_x2223_b3(b3);
        let j = unsafe { (*g).hsd_obj } as *mut HSD_JObj;
        let ret = ext::HSD_GObj_80390EB8(flag_index);
        let mut sp: Mtx = [[0.0; 4]; 3];
        let mtx = opt(ftDrawCommon_8008051C(g, &mut sp));
        ext::HSD_JObjDispAll(j, mtx, ret, 0);
    }
}

/// `ftDrawCommon_80080E18_inline2`: put the fighter at a camera-space point (C reads
/// `mv.co.walk.fast_anim_frame` as a Vec3, kept).
fn place_in_view(g: G) {
    let f = fp(g);
    let j = unsafe { (*g).hsd_obj } as *mut HSD_JObj;
    let mp = ext::HSD_CObjGetInvViewingMtxPtr(ext::Camera_800310B8());
    let src = unsafe { *(core::ptr::addr_of!(f.mv.co.walk.fast_anim_frame) as *const Vec3) };
    let rows = unsafe { core::slice::from_raw_parts(mp as *const [f32; 4], 3) };
    let r = m::mult_vec(rows, &melee_core::vec::Vec3 { x: src.x, y: src.y, z: src.z });
    f.cur_pos = Vec3 { x: r.x, y: r.y, z: r.z };
    assert!(!j.is_null(), "jobj.h:481");
    unsafe {
        (*j).translate = f.cur_pos;
        if (*j).flags & (1 << 25) == 0 && !((*j).flags & (1 << 23) == 0 && (*j).flags & (1 << 6) != 0) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
}

fn surface_impl(g: G, arg1: i32) {
    let f = fp(g);
    if f.x221F_b3() == 0 && (ext::Wb_RenderActive() != 0 || ext::ftLib_80086A8C(g) != 0) {
        match ext::Camera_80031060() {
            1 => {
                if fp(g).x2220_b7() != 0 {
                    place_in_view(g);
                }
                if !fp(g).x5AC.xC[4].is_null() {
                    pass(g, arg1, 0);
                    ftDrawCommon_800805C8(g, arg1, true);
                    pass(g, arg1, 1);
                }
            }
            0 => {
                fp(g).set_x2223_b3(0);
                let a2 = fp(g).x5AC.xC[4].is_null();
                ftDrawCommon_800805C8(g, arg1, a2);
            }
            _ => {}
        }
    }
}

/// `ftDrawCommon_80080E18` (TARGET_PC wrapper: motion draw, Geno toon, per-port surface).
pub fn ftDrawCommon_80080E18(g: G, arg1: i32) {
    let motion = ext::script_motion_draw_begin(g);
    let (pid, kind) = (fp(g).player_id as i32, fp(g).kind as i32);
    ext::GenoToonEnsure(pid + 1, kind);
    ext::SurfaceBegin(fp(g).player_id as i32 + 1);
    surface_impl(g, arg1);
    ext::SurfaceEnd();
    if motion != 0 {
        ext::MotionFighterEnd();
    }
}

unsafe extern "C" fn cb_80080E18(g: *mut HSD_GObj, a: i32) { ftDrawCommon_80080E18(g, a) }
unsafe extern "C" fn cb_80080C28(g: *mut HSD_GObj, a: i32) { ftDrawCommon_80080C28(g, a) }

fn for_each_fighter(mut f: impl FnMut(G)) {
    let mut g = ext::plink_fighter_head();
    while !g.is_null() {
        f(g);
        g = unsafe { (*g).next };
    }
}

/// `ftDrawCommon_80081118`: normal render callback on every fighter.
pub fn ftDrawCommon_80081118() {
    for_each_fighter(|g| unsafe { (*g).render_cb = Some(cb_80080E18) });
}

/// `ftDrawCommon_80081140`: FoD reflection callback (m-ex opt-in: none with 4+ players).
pub fn ftDrawCommon_80081140() {
    if ext::Mex_Enabled_disable_fod_reflection() != 0 && ext::ftLib_800860C4() >= 4 {
        for_each_fighter(|g| unsafe { (*g).render_cb = None });
        return;
    }
    for_each_fighter(|g| unsafe { (*g).render_cb = Some(cb_80080C28) });
}

fn parts4(order: [i32; 4]) {
    for_each_fighter(|g| {
        let f: *mut Fighter = fp(g);
        if unsafe { !(*f).x5AC.xC[1].is_null() } {
            ext::ftParts_800750C8(f, order[0], 0);
            ext::ftParts_800750C8(f, order[1], 0);
            ext::ftParts_800750C8(f, order[2], 0);
            ext::ftParts_800750C8(f, order[3], 1);
        }
    });
}

/// `ftDrawCommon_80081168`.
pub fn ftDrawCommon_80081168() { parts4([1, 2, 4, 0]) }
/// `ftDrawCommon_80081200`.
pub fn ftDrawCommon_80081200() { parts4([0, 2, 4, 1]) }
