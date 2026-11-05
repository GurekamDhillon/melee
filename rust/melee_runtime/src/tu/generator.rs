//! Native-pointer port of `src/sysdolphin/baselib/generator.c`: the particle generators (emitters). Every function
//! of the C file, on the real `HSD_Generator` records (host memory from the `HSD_ObjAlloc` pool `hsd_804D0F90`),
//! the real generator list (`hsd_804D78FC` head, `next` links, the `hsd_804D78F8` cursor) and the real `HSD_JObj`s.
//! The owned-model port (`crate::generator`, arena + ids, oracle `tests/oracle_generator.rs`) stays as the test
//! oracle that fixed the float contraction: every fused site below is the one that port marks with `fmadds`
//! (hsd_8039D71C 12, hsd_8039DAD4 22, hsd_8039EE24 1, hsd_8039F05C 3); `M_PI` arithmetic is f64 as in the C.
//!
//! GameCube vs native: the C stores pointers in `u32` globals (`hsd_804D78F8`, `hsd_804D78F4`) and passes
//! `(s32) gen` / `(s32) gen->cmdList` to `hsd_80398F0C`; here they are pointers.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_assignments, unused_mut, unused_variables, dead_code, clippy::all)]
use crate::gekko::{cosf, sinf, sqrtf};
use crate::hsd_mtx::fabsf_bitwise;
use crate::lbtrigf::atan2f;
use crate::tu::mtx_pc::raw as mx;
use crate::tu::particle::{
    hsd_804D78DA, hsd_804D78E0, hsd_804D78E8, hsd_804D78EC, hsd_804D78F4, hsd_80398F0C, hsd_8039D0A0, psCamera,
    psCmdListArray, psTexGroupArray, ptclref_804D0E5C,
};
use crate::tu::psappsrt::{psAddGeneratorAppSRT_begin, psRemoveGeneratorSRT};
use core::ptr::null_mut;
use melee_core::fp::fmadds;
use melee_types::*;

const M_PI: f64 = core::f64::consts::PI;
const M_PI_2: f64 = core::f64::consts::FRAC_PI_2;
const FLT_MIN: f32 = 1.1754944e-38;
const HALF_PI_F: f32 = 1.5707964;

/// `hsd_804D78FC`: the generator list head.
pub static mut hsd_804D78FC: *mut HSD_Generator = null_mut();
/// `hsd_804D78F8`: the cursor (the last generator the walkers passed; `hsd_8039D9C8` inserts after its next).
pub static mut hsd_804D78F8: *mut HSD_Generator = null_mut();
/// `hsd_804D7900`: called on every generator `hsd_8039F05C` creates (eflib installs `efLib_Cb_PtclAppSRTHook`).
pub static mut hsd_804D7900: Option<unsafe extern "C" fn(*mut HSD_Generator)> = None;
/// `hsd_804D0F90`: the generator pool.
pub static mut hsd_804D0F90: hsd_804D0F60_t = unsafe { core::mem::zeroed() };
static mut lbl_804D6368: u16 = 0x100;
/// Debug aid (`MELEE_PS_TRACE`): log every generator created / emitting to stderr.
pub static mut GEN_TRACE: bool = false;

#[inline]
unsafe fn ref_INC(o: *mut HSD_JObj) {
    if !o.is_null() {
        let r = &mut (*(o as *mut HSD_Obj)).ref_count;
        *r = r.wrapping_add(1);
        assert!(*r != 0xFFFF, "object.h:87 HSD_OBJ(o)->ref_count != HSD_OBJ_NOREF");
    }
}

#[inline]
fn randf() -> f32 {
    crate::tu::random::global::HSD_Randf()
}

pub unsafe extern "C" fn hsd_8039D1E4(gen: *mut HSD_Generator, userfunc: *mut core::ffi::c_void) {
    melee_core::ct!("hsd_8039D1E4");
    (*gen).userfunc = userfunc as *mut _PSUserFunc;
}

pub unsafe extern "C" fn hsd_8039D1EC() -> u16 {
    melee_core::ct!("hsd_8039D1EC");
    lbl_804D6368 = lbl_804D6368.wrapping_add(1);
    if lbl_804D6368 < 256 {
        lbl_804D6368 = 256;
    }
    lbl_804D6368
}

pub unsafe extern "C" fn hsd_8039D214(gen: *mut HSD_Generator) {
    melee_core::ct!("hsd_8039D214");
    if (*gen).r#type & 0x100 == 0 {
        return;
    }
    let jobj = (*gen).jobj;
    if jobj.is_null() {
        return;
    }
    crate::tu::jobj::HSD_JObjSetupMatrix(jobj);
    if (*gen).r#type & 0x200 != 0 {
        (*gen).pos.x = (*(*gen).jobj).mtx[0][3];
        (*gen).pos.y = (*(*gen).jobj).mtx[1][3];
        (*gen).pos.z = (*(*gen).jobj).mtx[2][3];
    }
    if (*gen).r#type & 0x800 != 0 {
        if !(*gen).appsrt.is_null() && (*(*gen).appsrt).gp == gen {
            (*(*gen).appsrt).translate.x = (*(*gen).jobj).mtx[0][3];
            (*(*gen).appsrt).translate.y = (*(*gen).jobj).mtx[1][3];
            (*(*gen).appsrt).translate.z = (*(*gen).jobj).mtx[2][3];
        }
    }
    if (*gen).r#type & 0x1000 != 0 {
        if !(*gen).appsrt.is_null() && (*(*gen).appsrt).gp == gen {
            crate::tu::mtx_world::HSD_MtxGetScale((*(*gen).jobj).mtx.as_mut_ptr(), &raw mut (*(*gen).appsrt).scale);
        }
    }
}

pub unsafe extern "C" fn hsd_8039D354(_unused: u32) {
    melee_core::ct!("hsd_8039D354");
    crate::tu::objalloc::HSD_ObjAllocInit(&raw mut hsd_804D0F90.alloc_data, core::mem::size_of::<HSD_Generator>(), 4);
    hsd_804D78FC = null_mut();
    hsd_804D78E0 = 0;
    hsd_804D78DA = 0;
    hsd_804D78F4 = null_mut();
    psCamera = null_mut();
    hsd_804D78E8 = None;
    hsd_804D78EC = None;
    hsd_804D78F8 = null_mut();
    hsd_804D7900 = None;
}

pub unsafe extern "C" fn hsd_8039D3AC(gen: *mut HSD_Generator, prev: *mut HSD_Generator) -> *mut HSD_Generator {
    melee_core::ct!("hsd_8039D3AC");
    if (*gen).r#type & 0x80 != 0 {
        hsd_8039D0A0(gen);
    }
    if (*gen).numChild != 0 {
        (*gen).random = 0.0;
        (*gen).genLife = 1;
        return gen;
    }
    if (*gen).r#type & 0x100 != 0 {
        if !(*gen).jobj.is_null() {
            if (*gen).r#type & 0x1800 != 0 {
                let srt = (*gen).appsrt;
                if !srt.is_null() {
                    if (*srt).gp == gen {
                        if (*srt).usedCount != 1 {
                            (*gen).random = 0.0;
                            (*gen).genLife = 1;
                            return gen;
                        }
                    }
                }
            }
        }
    }
    if prev.is_null() {
        hsd_804D78FC = (*gen).next;
    } else {
        (*prev).next = (*gen).next;
    }
    if !(*gen).appsrt.is_null() {
        psRemoveGeneratorSRT(gen);
    }
    if !(*gen).jobj.is_null() {
        crate::tu::jobj::HSD_JObjUnref((*gen).jobj);
        (*gen).jobj = null_mut();
    }
    crate::tu::objalloc::HSD_ObjFree(&raw mut hsd_804D0F90.alloc_data, gen as *mut core::ffi::c_void);
    hsd_804D78E0 = hsd_804D78E0.wrapping_sub(1);
    prev
}

pub unsafe extern "C" fn hsd_8039D4DC(gen: *mut HSD_Generator) {
    melee_core::ct!("hsd_8039D4DC");
    let mut cur = hsd_804D78FC;
    hsd_804D78F8 = null_mut();
    while !cur.is_null() {
        if cur == gen {
            hsd_804D78F8 = hsd_8039D3AC(gen, hsd_804D78F8);
            if !hsd_804D78F8.is_null() {
                while !(*hsd_804D78F8).next.is_null() {
                    hsd_804D78F8 = (*hsd_804D78F8).next;
                }
            } else if !hsd_804D78FC.is_null() {
                hsd_804D78F8 = hsd_804D78FC;
                while !(*hsd_804D78F8).next.is_null() {
                    hsd_804D78F8 = (*hsd_804D78F8).next;
                }
            }
            return;
        }
        hsd_804D78F8 = cur;
        cur = (*cur).next;
    }
}

pub unsafe extern "C" fn hsd_8039D580(jobj: *mut HSD_JObj) {
    melee_core::ct!("hsd_8039D580");
    if !jobj.is_null() {
        let mut cur = hsd_804D78FC;
        while !cur.is_null() {
            let next = (*cur).next;
            if (*cur).jobj == jobj {
                hsd_8039D4DC(cur);
            }
            cur = next;
        }
    }
}

pub unsafe extern "C" fn hsd_8039D5DC(mut jobj: *mut HSD_JObj) {
    melee_core::ct!("hsd_8039D5DC");
    if jobj.is_null() {
        return;
    }
    let mut gen = hsd_804D78FC;
    while !gen.is_null() {
        let next = (*gen).next;
        if (*gen).jobj == jobj {
            hsd_8039D4DC(gen);
        }
        gen = next;
    }
    if (*jobj).flags & 0x1000 != 0 {
        return;
    }
    jobj = if jobj.is_null() { null_mut() } else { (*jobj).child };
    while !jobj.is_null() {
        hsd_8039D5DC(jobj);
        jobj = if jobj.is_null() { null_mut() } else { (*jobj).next };
    }
}

pub unsafe extern "C" fn hsd_8039D688(jobj: *mut HSD_JObj, _unused1: *mut *mut f32, _unused2: i32) {
    melee_core::ct!("hsd_8039D688");
    let mut gen = hsd_804D78FC;
    hsd_804D78F8 = null_mut();
    while !gen.is_null() {
        let next = (*gen).next;
        if (*gen).jobj == jobj {
            if !(*gen).appsrt.is_null() {
                let ty = (*gen).r#type;
                if ty & 0x100 != 0 {
                    (*gen).r#type = ty | 0x80;
                }
            }
            hsd_804D78F8 = hsd_8039D3AC(gen, hsd_804D78F8);
        } else {
            hsd_804D78F8 = gen;
        }
        gen = next;
    }
}

pub unsafe extern "C" fn hsd_8039D71C(gen: *mut HSD_Generator) {
    melee_core::ct!("hsd_8039D71C");
    if (*gen).r#type & 0x100 == 0 {
        return;
    }
    let jobj = (*gen).jobj;
    if jobj.is_null() {
        return;
    }
    crate::tu::jobj::HSD_JObjSetupMatrix(jobj);
    if (*gen).r#type & 0x200 != 0 {
        (*gen).pos.x = (*(*gen).jobj).mtx[0][3];
        (*gen).pos.y = (*(*gen).jobj).mtx[1][3];
        (*gen).pos.z = (*(*gen).jobj).mtx[2][3];
    }
    if (*gen).r#type & 0x800 != 0 {
        if !(*gen).appsrt.is_null() && (*(*gen).appsrt).gp == gen {
            (*(*gen).appsrt).translate.x = (*(*gen).jobj).mtx[0][3];
            (*(*gen).appsrt).translate.y = (*(*gen).jobj).mtx[1][3];
            (*(*gen).appsrt).translate.z = (*(*gen).jobj).mtx[2][3];
        }
    }
    if (*gen).r#type & 0x1000 != 0 {
        if !(*gen).appsrt.is_null() && (*(*gen).appsrt).gp == gen {
            crate::tu::mtx_world::HSD_MtxGetScale((*(*gen).jobj).mtx.as_mut_ptr(), &raw mut (*(*gen).appsrt).scale);
        }
    }
    let m = &(*jobj).mtx;
    let mut col = Vec { x: m[0][0], y: m[1][0], z: m[2][0] };
    mx::PSVECNormalize(&mut col, &mut col);
    let (col0_x, col0_y, col0_z) = (col.x, col.y, col.z);
    col = Vec { x: m[0][1], y: m[1][1], z: m[2][1] };
    mx::PSVECNormalize(&mut col, &mut col);
    let (col1_x, col1_y, col1_z) = (col.x, col.y, col.z);
    col = Vec { x: m[0][2], y: m[1][2], z: m[2][2] };
    mx::PSVECNormalize(&mut col, &mut col);
    let (cx, cy, cz) = (col.x, col.y, col.z);
    let (vx, vy, vz) = ((*gen).vel.x, (*gen).vel.y, (*gen).vel.z);
    (*gen).vel.x = fmadds(cx, vz, fmadds(col0_x, vx, col1_x * vy));
    (*gen).vel.y = fmadds(cy, vz, fmadds(col0_y, vx, col1_y * vy));
    (*gen).vel.z = fmadds(cz, vz, fmadds(col0_z, vx, col1_z * vy));
    if (*gen).r#type & 0xF == 1 {
        let (ax, ay, az) = ((*gen).aux.line.x2, (*gen).aux.line.y2, (*gen).aux.line.z2);
        (*gen).aux.line.x2 = fmadds(m[0][2], az, fmadds(m[0][0], ax, m[0][1] * ay));
        (*gen).aux.line.y2 = fmadds(m[1][2], az, fmadds(m[1][0], ax, m[1][1] * ay));
        (*gen).aux.line.z2 = fmadds(m[2][2], az, fmadds(m[2][0], ax, m[2][1] * ay));
    }
}

pub unsafe extern "C" fn hsd_8039D9C8() -> *mut HSD_Generator {
    melee_core::ct!("hsd_8039D9C8");
    let gen = crate::tu::objalloc::HSD_ObjAlloc(&raw mut hsd_804D0F90.alloc_data) as *mut HSD_Generator;
    if !gen.is_null() {
        core::ptr::write_bytes(gen as *mut u8, 0, core::mem::size_of::<HSD_Generator>());
    }
    if gen.is_null() {
        return null_mut();
    }
    hsd_804D78E0 = hsd_804D78E0.wrapping_add(1);
    if hsd_804D78E0 > hsd_804D78DA {
        hsd_804D78DA = hsd_804D78E0;
    }
    if hsd_804D78F8.is_null() || (*hsd_804D78F8).next.is_null() {
        if hsd_804D78FC.is_null() {
            (*gen).next = null_mut();
            hsd_804D78FC = gen;
        } else {
            (*gen).next = (*hsd_804D78FC).next;
            (*hsd_804D78FC).next = gen;
        }
    } else {
        (*gen).next = (*(*hsd_804D78F8).next).next;
        (*(*hsd_804D78F8).next).next = gen;
    }
    lbl_804D6368 = lbl_804D6368.wrapping_add(1);
    if lbl_804D6368 < 0x100 {
        lbl_804D6368 = 0x100;
    }
    (*gen).idnum = lbl_804D6368;
    (*gen).appsrt = null_mut();
    (*gen).userfunc = null_mut();
    gen
}

#[inline]
fn pole(y: f32, x: f32, sign_of: f32) -> f32 {
    if fabsf_bitwise(x) < FLT_MIN {
        if sign_of >= 0.0 { HALF_PI_F } else { -HALF_PI_F }
    } else {
        atan2f(y, x)
    }
}

#[inline]
fn half_snap(v: f32) -> f32 {
    if v > 0.5 { 1.0 } else { 0.0 }
}

/// The `hsd_80398F0C(gen->linkNo, gen->bank, kind, gen->texGroup, gen->cmdList, gen->life, 0, gen, pos, vel, size, grav, fric)` call.
#[inline]
unsafe fn spawn(gen: *mut HSD_Generator, kind: i32, p: Vec, v: Vec, a: f32, b: f32, c: f32) {
    hsd_80398F0C(
        (*gen).linkNo as i32,
        (*gen).bank as i32,
        kind,
        (*gen).texGroup as u16,
        (*gen).cmdList,
        (*gen).life as i32,
        0,
        gen,
        p.x,
        p.y,
        p.z,
        v.x,
        v.y,
        v.z,
        a,
        b,
        c,
    );
}

pub unsafe extern "C" fn hsd_8039DAD4(gen: *mut HSD_Generator) -> f32 {
    melee_core::ct!("hsd_8039DAD4");
    let mut angle3 = 0.0f32;
    let mut angle1 = 0.0f32;
    let (mut sin_az, mut cos_az, mut sin_el) = (0.0f32, 0.0f32, 0.0f32);
    let (mut angle_step, mut cur_angle, mut cone_angle, mut elevation, mut radius) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    if (*gen).count < 1.0 {
        return (*gen).count;
    }
    if GEN_TRACE {
        eprintln!("GEN emit id {:#x} count {} type {:#x}", (*gen).idnum, (*gen).count, (*gen).r#type);
    }
    let mut vel_copy = (*gen).vel;
    let vel_mag_sq = sqrtf(vel_copy.z * vel_copy.z + (vel_copy.x * vel_copy.x + vel_copy.y * vel_copy.y)); // squares of memory: not fused on the console (notes/fp-audit.md)
    let mut rot_mtx: Mtx = [[0.0; 4]; 3];
    mx::PSMTXIdentity(rot_mtx.as_mut_ptr());
    let shape = (*gen).r#type & 0xF;

    if (*gen).r#type & 0x100 != 0 && !(*gen).jobj.is_null() && (*gen).r#type & 0x400 != 0 && (*gen).kind & 0x30000 == 0 {
        let mut jobj_mtx: Mtx = [[0.0; 4]; 3];
        mx::PSMTXCopy((*(*gen).jobj).mtx.as_mut_ptr(), jobj_mtx.as_mut_ptr());
        for c in 0..3 {
            let mut v = Vec { x: jobj_mtx[0][c], y: jobj_mtx[1][c], z: jobj_mtx[2][c] };
            mx::PSVECNormalize(&mut v, &mut v);
            rot_mtx[0][c] = v.x;
            rot_mtx[1][c] = v.y;
            rot_mtx[2][c] = v.z;
        }
        rot_mtx[2][3] = 0.0;
        rot_mtx[1][3] = 0.0;
        rot_mtx[0][3] = 0.0;
    }

    if (*gen).kind & 0x10000 != 0 {
        assert!(!psCamera.is_null(), "generator.c:677 psCamera");
        let eye = &(*(*psCamera).eyepos).pos;
        let mut look_dir = Vec { x: eye.x - (*gen).pos.x, y: eye.y - (*gen).pos.y, z: eye.z - (*gen).pos.z };
        mx::PSVECNormalize(&mut look_dir, &mut look_dir);
        let mut cam_up = Vec { x: 0.0, y: 0.0, z: 0.0 };
        crate::tu::cobj_world::HSD_CObjGetUpVector(psCamera, &mut cam_up);
        mx::PSVECNormalize(&mut cam_up, &mut cam_up);
        let mut cross1 = Vec { x: 0.0, y: 0.0, z: 0.0 };
        mx::PSVECCrossProduct(&mut cam_up, &mut look_dir, &mut cross1);
        mx::PSVECCrossProduct(&mut look_dir, &mut cross1, &mut cam_up);
        rot_mtx[0][0] = cross1.x;
        rot_mtx[1][0] = cross1.y;
        rot_mtx[2][0] = cross1.z;
        rot_mtx[0][1] = cam_up.x;
        rot_mtx[1][1] = cam_up.y;
        rot_mtx[2][1] = cam_up.z;
        rot_mtx[0][2] = look_dir.x;
        rot_mtx[1][2] = look_dir.y;
        rot_mtx[2][2] = look_dir.z;
    }

    if shape != 1 && vel_mag_sq > 1.1920929e-7 {
        let mut vel_norm = Vec { x: (*gen).vel.x, y: (*gen).vel.y, z: (*gen).vel.z };
        mx::PSVECNormalize(&mut vel_norm, &mut vel_norm);
        sin_az = pole(vel_norm.y, vel_norm.z, vel_norm.y);
        cos_az = sinf(sin_az);
        sin_az = cosf(sin_az);
        let projected_z = fmadds(vel_norm.y, cos_az, vel_norm.z * sin_az);
        elevation = pole(vel_norm.x, projected_z, vel_norm.x);
        sin_el = sinf(elevation);
        let cos_el = cosf(elevation);
        let neg_cos_az = -cos_az;
        let mut trig_mtx: Mtx = [
            [cos_el, 0.0, sin_el, 0.0],
            [neg_cos_az * sin_el, sin_az, cos_az * cos_el, 0.0],
            [-sin_az * sin_el, neg_cos_az, sin_az * cos_el, 0.0],
        ];
        mx::PSMTXConcat(rot_mtx.as_mut_ptr(), trig_mtx.as_mut_ptr(), rot_mtx.as_mut_ptr());
    }
    elevation = vel_mag_sq;

    if shape == 2 {
        angle1 = pole(rot_mtx[1][2], rot_mtx[2][2], rot_mtx[1][2]);
        let first = rot_mtx[2][2] * cosf(angle1);
        let comb = fmadds(rot_mtx[1][2], sinf(angle1), first);
        angle3 = pole(rot_mtx[0][2], comb, rot_mtx[0][2]);
    }

    if (*gen).angle < 0.0 {
        let n = melee_core::ppc::fctiwz_f32((*gen).count) as f32;
        match shape {
            0 | 3 | 4 | 6 | 7 => {
                let min_a = (*gen).aux.disc.minAngle;
                let rnd = randf();
                angle_step = ((*gen).aux.disc.maxAngle - min_a) / n;
                cur_angle = fmadds(angle_step, rnd, min_a);
            }
            _ => {
                let rnd = randf();
                cur_angle = ((M_PI * rnd as f64) * 2.0) as f32;
                angle_step = ((2.0 * M_PI) / (melee_core::ppc::fctiwz_f32((*gen).count) as f64)) as f32;
            }
        }
    }

    let eps = 0.001f32 as f64;
    while (*gen).count >= 1.0 {
        match shape {
            0 | 3 | 4 | 6 | 7 => {
                if (*gen).radius < 0.0 {
                    sin_az = -(*gen).radius;
                    radius = 1.0;
                } else {
                    radius = randf();
                    if shape == 3 || shape == 4 {
                        radius = sqrtf(radius);
                    }
                    sin_az = radius * (*gen).radius;
                }
                match shape {
                    6 => {
                        let h = (*gen).aux.cone.height;
                        if (*gen).angle < 0.0 {
                            cur_angle = cur_angle + angle_step;
                            cone_angle = if fabsf_bitwise(sin_az) < FLT_MIN {
                                if h >= 0.0 { -(*gen).angle } else { (M_PI - (*gen).angle as f64) as f32 }
                            } else {
                                ((M_PI_2 - atan2f(h, sin_az) as f64) - (*gen).angle as f64) as f32
                            };
                        } else {
                            cur_angle = (*gen).aux.cone.minAngle;
                            let rnd = randf();
                            cone_angle = (*gen).aux.cone.maxAngle - cur_angle;
                            cur_angle = fmadds(cone_angle, rnd, cur_angle);
                            cone_angle = if fabsf_bitwise(sin_az) < FLT_MIN {
                                if h >= 0.0 { (*gen).angle } else { (M_PI + (*gen).angle as f64) as f32 }
                            } else {
                                ((*gen).angle as f64 + (M_PI_2 - atan2f(h, sin_az) as f64)) as f32
                            };
                        }
                    }
                    7 => {
                        if (*gen).angle < 0.0 {
                            cone_angle = (M_PI_2 - (*gen).angle as f64) as f32;
                            cur_angle = cur_angle + angle_step;
                        } else {
                            cur_angle = (*gen).aux.disc.minAngle;
                            let rnd = randf();
                            cone_angle = (*gen).aux.disc.maxAngle - cur_angle;
                            cur_angle = fmadds(cone_angle, rnd, cur_angle);
                            cone_angle = (M_PI_2 + (*gen).angle as f64) as f32;
                        }
                    }
                    _ => {
                        if (*gen).angle < 0.0 {
                            cone_angle = -(*gen).angle;
                            cur_angle += angle_step;
                            cone_angle = radius * cone_angle;
                        } else {
                            cur_angle = (*gen).aux.disc.minAngle;
                            let rnd = randf();
                            cone_angle = radius * (*gen).angle;
                            sin_el = (*gen).aux.disc.maxAngle - cur_angle;
                            cur_angle = fmadds(sin_el, rnd, cur_angle);
                        }
                    }
                }
                cos_az = cur_angle;
                let mut emit_pos = Vec { x: sin_az * cosf(cos_az), y: sin_az * sinf(cos_az), z: 0.0 };
                if shape == 6 || shape == 7 {
                    emit_pos.z = randf();
                    if shape == 6 {
                        emit_pos.x *= 1.0 - emit_pos.z;
                        emit_pos.y *= 1.0 - emit_pos.z;
                    }
                    emit_pos.z *= (*gen).aux.cone.height;
                }
                let sin_ca = elevation * sinf(cone_angle);
                let mut vec = Vec { x: sin_ca * cosf(cos_az), y: sin_ca * sinf(cos_az), z: elevation * cosf(cone_angle) };
                if shape == 3 {
                    vec.x *= radius;
                    vec.y *= radius;
                    vec.z *= radius;
                }
                mx::PSMTXMultVec(rot_mtx.as_mut_ptr(), &mut emit_pos, &mut emit_pos);
                emit_pos.x += (*gen).pos.x;
                emit_pos.y += (*gen).pos.y;
                emit_pos.z += (*gen).pos.z;
                mx::PSMTXMultVec(rot_mtx.as_mut_ptr(), &mut vec, &mut vec);
                cur_angle = cos_az;
                spawn(gen, (*gen).kind as i32, emit_pos, vec, (*gen).size, (*gen).grav, (*gen).fric);
            }
            1 => {
                let rnd = randf();
                let mut emit_pos = Vec { x: rnd * (*gen).aux.line.x2, y: rnd * (*gen).aux.line.y2, z: rnd * (*gen).aux.line.z2 };
                mx::PSMTXMultVec(rot_mtx.as_mut_ptr(), &mut emit_pos, &mut emit_pos);
                emit_pos.x += (*gen).pos.x;
                emit_pos.y += (*gen).pos.y;
                emit_pos.z += (*gen).pos.z;
                let mut vec = Vec { x: 0.0, y: 0.0, z: 0.0 };
                mx::PSMTXMultVec(rot_mtx.as_mut_ptr(), &mut vel_copy, &mut vec);
                spawn(gen, (*gen).kind as i32, emit_pos, vec, (*gen).size, (*gen).grav, (*gen).fric);
            }
            2 => {
                sin_az = if (*gen).radius < 0.0 { 1.0 } else { randf() };
                if (*gen).angle < 0.0 {
                    cur_angle += angle_step;
                } else {
                    let rnd = randf();
                    cur_angle = (2.0 * (M_PI * rnd as f64)) as f32;
                }
                (*gen).aux.line.x2 = elevation;
                spawn(
                    gen,
                    ((*gen).kind | 4) as i32,
                    Vec { x: 0.0, y: 0.0, z: 0.0 },
                    Vec { x: cur_angle, y: sin_az, z: 0.0 },
                    (*gen).size,
                    angle1,
                    angle3,
                );
            }
            5 => {
                let mut emit_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
                emit_pos.x = randf();
                emit_pos.y = randf();
                emit_pos.z = randf();
                let r = &(*gen).aux.rect;
                match r.flag {
                    1 => emit_pos.x = half_snap(emit_pos.x),
                    2 => emit_pos.y = half_snap(emit_pos.y),
                    3 => {
                        let rnd = randf();
                        let a2 = r.x;
                        if rnd > a2 / (a2 + r.y) {
                            emit_pos.y = half_snap(emit_pos.y);
                        } else {
                            emit_pos.x = half_snap(emit_pos.x);
                        }
                    }
                    4 => emit_pos.z = half_snap(emit_pos.z),
                    5 => {
                        let rnd = randf();
                        let a2 = r.x;
                        if rnd > a2 / (a2 + r.z) {
                            emit_pos.z = half_snap(emit_pos.z);
                        } else {
                            emit_pos.x = half_snap(emit_pos.x);
                        }
                    }
                    6 => {
                        let rnd = randf();
                        let a2 = r.y;
                        if rnd > a2 / (a2 + r.z) {
                            emit_pos.z = half_snap(emit_pos.z);
                        } else {
                            emit_pos.y = half_snap(emit_pos.y);
                        }
                    }
                    7 => {
                        let rnd = randf();
                        let a2 = r.z;
                        let b2 = r.y;
                        let c2 = r.x;
                        let r0 = 1.0 / fmadds(c2, b2 + a2, b2 * a2);
                        if rnd < r0 * (c2 * b2) {
                            emit_pos.z = half_snap(emit_pos.z);
                        } else if rnd > fmadds(-r0, c2 * a2, 1.0) {
                            emit_pos.y = half_snap(emit_pos.y);
                        } else {
                            emit_pos.x = half_snap(emit_pos.x);
                        }
                    }
                    _ => {}
                }
                emit_pos.x -= 0.5;
                emit_pos.y -= 0.5;
                emit_pos.z -= 0.5;
                let (xx, xy, xz, yx, yy, yz, zx, zy, zz) = (r.xx, r.xy, r.xz, r.yx, r.yy, r.yz, r.zx, r.zy, r.zz);
                let mut tmpvec = Vec {
                    x: fmadds(zx, emit_pos.z, fmadds(xx, emit_pos.x, yx * emit_pos.y)),
                    y: fmadds(zy, emit_pos.z, fmadds(xy, emit_pos.x, yy * emit_pos.y)),
                    z: fmadds(zz, emit_pos.z, fmadds(xz, emit_pos.x, yz * emit_pos.y)),
                };
                mx::PSMTXMultVec(rot_mtx.as_mut_ptr(), &mut tmpvec, &mut emit_pos);
                emit_pos.x += (*gen).pos.x;
                emit_pos.y += (*gen).pos.y;
                emit_pos.z += (*gen).pos.z;
                let mag = sqrtf(zx * zx + zy * zy + zz * zz); // squares of memory: not fused on the console (notes/fp-audit.md)
                let scale = elevation / mag;
                let mut vec = Vec { x: zx * scale, y: zy * scale, z: zz * scale };
                mx::PSMTXMultVec(rot_mtx.as_mut_ptr(), &mut vec, &mut vec);
                spawn(gen, (*gen).kind as i32, emit_pos, vec, (*gen).size, (*gen).grav, (*gen).fric);
            }
            8 => {
                let r0 = (*gen).aux.cone.height;
                if r0 == 0.0 || (r0 as f64 - M_PI).abs() < eps {
                    radius = randf();
                    radius = sqrtf(radius);
                    radius = (M_PI_2 * radius as f64) as f32;
                    let rnd2 = randf();
                    if rnd2 < 0.5 {
                        radius = (M_PI - radius as f64) as f32;
                    }
                } else {
                    radius = randf();
                    radius = sqrtf(radius);
                    radius = (*gen).aux.cone.height * radius;
                }
                cone_angle = randf();
                cone_angle = (2.0 * (M_PI * cone_angle as f64)) as f32;
                let r2 = (*gen).radius;
                if r2 < 0.0 {
                    sin_az = -r2;
                } else {
                    sin_az = randf();
                    sin_az = sqrtf(sin_az);
                    sin_az = r2 * sin_az;
                }
                let cos_r = cosf(cone_angle);
                cos_az = sinf(radius);
                let mut vec = Vec { x: 0.0, y: 0.0, z: 0.0 };
                vec.x = cos_az * cos_r;
                sin_el = sinf(cone_angle);
                let sin_r = sinf(radius);
                vec.y = sin_r * sin_el;
                vec.z = cosf(radius);
                let mut emit_pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
                mx::PSMTXMultVec(rot_mtx.as_mut_ptr(), &mut vec, &mut emit_pos);
                let s = (*gen).aux.rect.x;
                vec.x = emit_pos.x * s;
                vec.y = emit_pos.y * s;
                vec.z = emit_pos.z * s;
                if (*gen).radius >= 0.0 && s < 0.0 {
                    let scale = sin_az / (*gen).radius;
                    vec.x *= scale;
                    vec.y *= scale;
                    vec.z *= scale;
                }
                emit_pos.x = fmadds(sin_az, emit_pos.x, (*gen).pos.x);
                emit_pos.y = fmadds(sin_az, emit_pos.y, (*gen).pos.y);
                emit_pos.z = fmadds(sin_az, emit_pos.z, (*gen).pos.z);
                spawn(gen, (*gen).kind as i32, emit_pos, vec, (*gen).size, (*gen).grav, (*gen).fric);
            }
            _ => {
                if let Some(f) = hsd_804D78E8 {
                    f(gen, rot_mtx.as_mut_ptr());
                }
            }
        }
        (*gen).count -= 1.0;
    }
    (*gen).count
}

pub unsafe extern "C" fn hsd_8039EE24(mask: u32) {
    melee_core::ct!("hsd_8039EE24");
    while !hsd_804D78F4.is_null() {
        let gp = (*hsd_804D78F4).data as *mut HSD_Generator;
        let owned = crate::tu::particle::ext::ScriptGame_StageSlotParticleBegin((*gp).linkNo as i32);
        hsd_8039D71C(gp);
        hsd_804D78F4 = crate::tu::list::HSD_SListRemove(hsd_804D78F4);
        if !(*gp).jobj.is_null() {
            crate::tu::jobj::HSD_JObjUnref((*gp).jobj);
            (*gp).jobj = null_mut();
        }
        if owned != 0 {
            crate::tu::particle::ext::ScriptGame_StageSlotProcEnd();
        }
    }

    let mut gen = hsd_804D78FC;
    hsd_804D78F8 = null_mut();
    while !gen.is_null() {
        if mask & crate::cabi::shl(1, (*gen).linkNo as u32 + 16) != 0 {
            hsd_804D78F8 = gen;
            gen = (*gen).next;
            continue;
        }
        if (*gen).kind & 0x800 != 0 {
            hsd_804D78F8 = gen;
            gen = (*gen).next;
            continue;
        }
        let owned = crate::tu::particle::ext::ScriptGame_StageSlotParticleBegin((*gen).linkNo as i32);
        hsd_8039D214(gen);
        if (*gen).random < 0.0 {
            (*gen).count -= (*gen).random;
        } else {
            let r = randf();
            (*gen).count = fmadds((*gen).random, r, (*gen).count);
        }
        if (*gen).count >= 1.0 {
            (*gen).count = hsd_8039DAD4(gen);
        }
        if (*gen).genLife != 0 {
            let life = (*gen).genLife - 1;
            (*gen).genLife = life;
            if life == 0 {
                hsd_804D78F8 = hsd_8039D3AC(gen, hsd_804D78F8);
                if !hsd_804D78F8.is_null() {
                    gen = (*hsd_804D78F8).next;
                } else {
                    gen = hsd_804D78FC;
                }
                if owned != 0 {
                    crate::tu::particle::ext::ScriptGame_StageSlotProcEnd();
                }
                continue;
            }
        }
        hsd_804D78F8 = gen;
        gen = (*gen).next;
        if owned != 0 {
            crate::tu::particle::ext::ScriptGame_StageSlotProcEnd();
        }
    }
}

pub unsafe extern "C" fn hsd_8039EFAC(link_no: i32, bank: i32, gfx_id: i32, jobj: *mut HSD_JObj) -> *mut HSD_Generator {
    melee_core::ct!("hsd_8039EFAC");
    let gen = hsd_8039F05C(link_no, bank, gfx_id);
    if gen.is_null() {
        return null_mut();
    }
    (*gen).jobj = jobj;
    if !jobj.is_null() {
        ref_INC(jobj);
    }
    (*gen).r#type |= if (*gen).kind & 0x20000 != 0 { 0x500 } else { 0x700 };
    gen
}

pub unsafe extern "C" fn hsd_8039F05C(mut link_no: i32, bank: i32, idx: i32) -> *mut HSD_Generator {
    melee_core::ct!("hsd_8039F05C");
    if ext::ScriptGame_StageSlotNativeContext() != 0 {
        link_no = 3;
    }
    if bank >= 0x41 {
        return null_mut();
    }
    if link_no >= 8 {
        return null_mut();
    }
    if idx >= psCmdListArray[bank as usize] {
        return null_mut();
    }
    let cmd: *mut HSD_PSCmdList = *ptclref_804D0E5C[bank as usize].add(idx as usize);
    if cmd.is_null() {
        return null_mut();
    }
    let gen = hsd_8039D9C8();
    if !gen.is_null() {
        (*gen).r#type = (*cmd).r#type;
        (*gen).bank = bank as u8;
        (*gen).linkNo = link_no as u8;
        (*gen).kind = (*cmd).kind;
        (*gen).texGroup = (*cmd).texGroup as u8;
        (*gen).life = (*cmd).life;
        (*gen).genLife = (*cmd).genLife;
        (*gen).pos.z = 0.0;
        (*gen).pos.y = 0.0;
        (*gen).pos.x = 0.0;
        (*gen).vel.x = (*cmd).vx;
        (*gen).vel.y = (*cmd).vy;
        (*gen).vel.z = (*cmd).vz;
        (*gen).grav = (*cmd).grav;
        (*gen).fric = (*cmd).fric;
        (*gen).size = (*cmd).size;
        (*gen).cmdList = (&raw mut (*cmd).cmdList) as *mut u8;
        (*gen).radius = (*cmd).radius;
        (*gen).angle = (*cmd).angle;
        (*gen).random = (*cmd).random;

        if (*gen).kind & 0x100 != 0 {
            if (*gen).random < 0.0 {
                (*gen).count = if 1.0 + (*gen).random > 1.1920929e-7 { 1.0 } else { 0.0 };
            } else {
                (*gen).count = 0.9999999;
            }
        } else if (*gen).random < 0.0 {
            (*gen).count = 0.0;
        } else {
            (*gen).count = randf();
        }

        let tg: *mut HSD_PSTexGroup = *psTexGroupArray[bank as usize].add((*gen).texGroup as usize);
        if !tg.is_null() && (*tg).palflag != 0 {
            (*gen).kind |= 0x10;
        }

        (*gen).jobj = null_mut();
        (*gen).numChild = 0;

        match (*gen).r#type & 0xF {
            0 | 3 | 4 => {
                let p1 = (*cmd).param1;
                if p1 == 0.0 && (*cmd).param2 == 0.0 {
                    (*gen).aux.disc.minAngle = 0.0;
                    (*gen).aux.disc.maxAngle = 6.2831855;
                } else {
                    (*gen).aux.disc.minAngle = p1;
                    (*gen).aux.disc.maxAngle = (*cmd).param2;
                }
            }
            1 => {
                (*gen).aux.line.x2 = (*cmd).param1;
                (*gen).aux.line.y2 = (*cmd).param2;
                (*gen).aux.line.z2 = (*cmd).param3;
            }
            2 => {}
            6 | 7 => {
                let p1 = (*cmd).param1;
                if p1 == 0.0 && (*cmd).param2 == 0.0 {
                    (*gen).aux.cone.minAngle = 0.0;
                    (*gen).aux.cone.maxAngle = 6.2831855;
                } else {
                    (*gen).aux.cone.minAngle = p1;
                    (*gen).aux.cone.maxAngle = (*cmd).param2;
                }
                (*gen).aux.cone.height = (*cmd).param3;
            }
            5 => {
                (*gen).aux.rect.x = (*cmd).param1;
                (*gen).aux.rect.xx = (*cmd).param1;
                (*gen).aux.rect.y = (*cmd).param2;
                (*gen).aux.rect.yy = (*cmd).param2;
                (*gen).aux.rect.z = (*cmd).param3;
                (*gen).aux.rect.zz = (*cmd).param3;
                (*gen).aux.rect.zy = 0.0;
                (*gen).aux.rect.zx = 0.0;
                (*gen).aux.rect.yz = 0.0;
                (*gen).aux.rect.yx = 0.0;
                (*gen).aux.rect.xz = 0.0;
                (*gen).aux.rect.xy = 0.0;
                (*gen).aux.rect.flag = 0;
                if (*cmd).param1 < 0.0 {
                    (*gen).aux.rect.flag |= 1;
                }
                if (*cmd).param2 < 0.0 {
                    (*gen).aux.rect.flag |= 2;
                }
                if (*cmd).param3 < 0.0 {
                    (*gen).aux.rect.flag |= 4;
                }
            }
            8 => {
                let v = (*gen).vel;
                (*gen).aux.sphere.speed = sqrtf(v.z * v.z + (v.x * v.x + v.y * v.y)); // squares of memory: not fused on the console (notes/fp-audit.md)
                let mag = sqrtf(v.x * v.x + v.z * v.z);
                if mag < FLT_MIN {
                    (*gen).aux.sphere.latMid = if v.y >= 0.0 { HALF_PI_F } else { -HALF_PI_F };
                } else {
                    (*gen).aux.sphere.latMid = atan2f(v.y, mag);
                }
                if fabsf_bitwise(v.x) < FLT_MIN {
                    (*gen).aux.sphere.lonMid = if v.z >= 0.0 { HALF_PI_F } else { -HALF_PI_F };
                } else {
                    (*gen).aux.sphere.lonMid = atan2f(v.z, v.x);
                }
                (*gen).aux.sphere.latRange = (*cmd).param1;
                if (*gen).aux.sphere.latRange < 0.0 {
                    (*gen).aux.sphere.latRange = -(*gen).aux.sphere.latRange;
                    (*gen).aux.sphere.speed = -(*gen).aux.sphere.speed;
                }
                (*gen).aux.sphere.lonRange = (*cmd).param2;
            }
            _ => {
                if let Some(f) = hsd_804D78EC {
                    f(gen);
                }
            }
        }

        if (*gen).kind & 0x20000 != 0 {
            (*gen).r#type |= 0x800;
            psAddGeneratorAppSRT_begin(gen, 0);
            if !(*gen).appsrt.is_null() {
                (*(*gen).appsrt).xA2 = 1;
                (*(*gen).appsrt).gp = gen;
            }
        }

        if let Some(f) = hsd_804D7900 {
            f(gen);
        }
        if GEN_TRACE {
            eprintln!(
                "GEN new id {:#x} link {} bank {} idx {} type {:#x} kind {:#x} random {} count {} life {} genLife {} vel {} {} {} size {} radius {} angle {}",
                (*gen).idnum, link_no, bank, idx, (*gen).r#type, (*gen).kind, (*gen).random, (*gen).count, (*gen).life, (*gen).genLife,
                (*gen).vel.x, (*gen).vel.y, (*gen).vel.z, (*gen).size, (*gen).radius, (*gen).angle
            );
        }
    }
    gen
}

pub unsafe extern "C" fn hsd_8039F6CC(link_no: i32, bank: i32, gfx_id: i32, jobj: *mut HSD_JObj) -> *mut HSD_Generator {
    melee_core::ct!("hsd_8039F6CC");
    let gen = hsd_8039F05C(link_no, bank, gfx_id);
    if gen.is_null() {
        return null_mut();
    }
    (*gen).jobj = jobj;
    if !jobj.is_null() {
        ref_INC(jobj);
    }
    (*gen).r#type |= if (*gen).kind & 0x20000 != 0 { 0x500 } else { 0x700 };
    hsd_804D78F4 = crate::tu::list::HSD_SListAllocAndAppend(hsd_804D78F4, gen as *mut core::ffi::c_void);
    gen
}

/// TARGET_PC: drop every generator (and queued joint-bound generator) of the stage slot (`linkNo` 3).
pub unsafe extern "C" fn HSD_StageSlotParticlesClear() {
    melee_core::ct!("HSD_StageSlotParticlesClear");
    let mut cur = hsd_804D78F4;
    let mut prev: *mut HSD_SList = null_mut();
    while !cur.is_null() {
        let g = (*cur).data as *mut HSD_Generator;
        if (*g).linkNo == 3 {
            let next = crate::tu::list::HSD_SListRemove(cur);
            if !prev.is_null() {
                (*prev).next = next;
            } else {
                hsd_804D78F4 = next;
            }
            cur = next;
        } else {
            prev = cur;
            cur = (*cur).next;
        }
    }
    crate::tu::particle::HSD_StageSlotParticleLinkClear();
    let mut gen = hsd_804D78FC;
    let mut previous: *mut HSD_Generator = null_mut();
    while !gen.is_null() {
        let next = (*gen).next;
        if (*gen).linkNo == 3 {
            if hsd_8039D3AC(gen, previous) == gen {
                previous = gen;
            }
        } else {
            previous = gen;
        }
        gen = next;
    }
}

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case)]
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn ScriptGame_StageSlotNativeContext() -> i32 = 0;
    }
}
