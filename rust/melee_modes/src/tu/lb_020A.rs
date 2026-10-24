//! Port of src/melee/lb/lb_020A.c (skeletal transform accumulation and the two-joint IK used to
//! plant a fighter's legs).
//! Checked against the C (whole TU, shipping contraction) by tests/tu_lb_020A.rs. The fused
//! multiply-add sites follow `tools/rust-port/fma_sites.py src/melee/lb/lb_020A.c`: 3 f64 in
//! `fn_80020AEC`, 2 f32 + 3 f64 in `lbBgFlash_80020E38`, 20 f32 in `lbBgFlash_80021410`, 3 f64 in
//! `sqrtf_store`.
#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]

use core::ffi::c_void;
use melee_core::fp::{fmadds, frsqrte};
use melee_types::*;

// BEGIN GENERATED EXT (rust/melee_modes/gen/cref/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:path)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => { $crate::ext_dflt!([$($def)?] ($($a),*) $name) }
                }
            } )*
        };
    }
    ext_fns! {
    fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj);
    fn HSD_JObjSetupMatrixSub(a0: *mut HSD_JObj);
    fn HSD_MkRotationMtx(a0: *mut c_void, a1: *mut Vec);
    fn HSD_MtxInverseConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void);
    fn HSD_MtxQuat(a0: *mut c_void, a1: *mut Quaternion);
    fn HSD_QuatLib_8037EB28(a0: *mut c_void, a1: *mut Vec) -> i32;
    fn MTXRotRad(a0: *mut c_void, a1: i8, a2: f32) = crate::tu::adapt::MTXRotRad__bc3d39;
    fn MatToQuat(a0: *mut c_void, a1: *mut Quaternion) -> i32;
    fn PSMTXConcat(a0: *mut c_void, a1: *mut c_void, a2: *mut c_void) = crate::tu::adapt::PSMTXConcat__e4df5b;
    fn PSMTXCopy(a0: *mut c_void, a1: *mut c_void);
    fn PSMTXMultVec(a0: *mut c_void, a1: *mut Vec, a2: *mut Vec) = crate::tu::adapt::PSMTXMultVec__aeae10;
    fn PSMTXQuat(a0: *mut c_void, a1: *mut Quaternion) = crate::tu::adapt::PSMTXQuat__fb47a1;
    fn PSMTXRotAxisRad(a0: *mut c_void, a1: *mut Vec, a2: f32) = crate::tu::adapt::PSMTXRotAxisRad__437995;
    fn PSMTXTranspose(a0: *mut c_void, a1: *mut c_void) = crate::tu::adapt::PSMTXTranspose__cc6675;
    fn PSVECMag(a0: *mut Vec) -> f32 = crate::tu::adapt::PSVECMag__307672;
    fn acosf(a0: f32) -> f32 = crate::tu::adapt::acosf__e9b06e;
    fn atan2f(a0: f32, a1: f32) -> f32 = crate::tu::adapt::atan2f__2f62eb;
    fn lbVector_Angle(a0: *mut Vec, a1: *mut Vec) -> f32 = crate::tu::adapt::lbVector_Angle__a15630;
    fn lbVector_Diff(a0: *mut Vec, a1: *mut Vec, a2: *mut Vec) -> *mut Vec = crate::tu::adapt::lbVector_Diff__3f20f8;
    fn lbVector_Normalize(a0: *mut Vec) -> f32 = crate::tu::adapt::lbVector_Normalize__748fa0;
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_JObjSetMtxDirtySub(a0: *mut HSD_JObj) { super::HSD_JObjSetMtxDirtySub(a0) }
    }
    // Data defined in other TUs (the tests set them).
}
// END GENERATED EXT

const MTX_DIRTY: u32 = JOBJ_MTX_DIRTY as u32;
const USER_DEF_MTX: u32 = JOBJ_USER_DEF_MTX as u32;
const MTX_INDEP_SRT: u32 = JOBJ_MTX_INDEP_SRT as u32;
const USE_QUATERNION: u32 = JOBJ_USE_QUATERNION as u32;

// ---- header inlines of sysdolphin/baselib/jobj.h (their asserts only fire on NULL, which the callers never pass)

unsafe fn HSD_JObjGetParent(jobj: *mut HSD_JObj) -> *mut HSD_JObj {
    if jobj.is_null() {
        core::ptr::null_mut()
    } else {
        (*jobj).parent
    }
}

unsafe fn HSD_JObjMtxIsDirty(jobj: *mut HSD_JObj) -> bool {
    ((*jobj).flags & USER_DEF_MTX) == 0 && ((*jobj).flags & MTX_DIRTY) != 0
}

unsafe fn HSD_JObjSetupMatrix(jobj: *mut HSD_JObj) {
    if jobj.is_null() || !HSD_JObjMtxIsDirty(jobj) {
        return;
    }
    ext::HSD_JObjSetupMatrixSub(jobj);
}

unsafe fn HSD_JObjGetMtxPtr(jobj: *mut HSD_JObj) -> *mut c_void {
    HSD_JObjSetupMatrix(jobj);
    (&raw mut (*jobj).mtx) as *mut c_void
}

/// the `HSD_JObjSetMtxDirty` macro
unsafe fn HSD_JObjSetMtxDirty(jobj: *mut HSD_JObj) {
    if !jobj.is_null() && !HSD_JObjMtxIsDirty(jobj) {
        ext::HSD_JObjSetMtxDirtySub(jobj);
    }
}

unsafe fn HSD_JObjGetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
    *rotate = (*jobj).rotate;
}

unsafe fn HSD_JObjSetRotation(jobj: *mut HSD_JObj, rotate: *mut Quaternion) {
    (*jobj).rotate = *rotate;
    if (*jobj).flags & MTX_INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

unsafe fn HSD_JObjSetRotationZ(jobj: *mut HSD_JObj, z: f32) {
    (*jobj).rotate.z = z;
    if (*jobj).flags & MTX_INDEP_SRT == 0 {
        HSD_JObjSetMtxDirty(jobj);
    }
}

unsafe fn HSD_JObjGetRotationZ(jobj: *mut HSD_JObj) -> f32 {
    (*jobj).rotate.z
}

/// Three Newton-Raphson steps on the Gekko `frsqrte` estimate, as the C spells them out inline:
/// `e = 0.5 * e * -((v * (e * e)) - 3.0)` with the multiply fused into the subtract (f64 fma).
#[inline]
fn newton_rsqrt(v: f32) -> f64 {
    let mut e = frsqrte(v as f64);
    for _ in 0..3 {
        let inner = (v as f64).mul_add(e * e, -3.0);
        e = 0.5 * e * -inner;
    }
    e
}

/// 020AEC: `out` = the accumulated transform from the root down to `jobj`, with every column
/// rescaled to the matching column length of `jobj`'s own world matrix.
pub unsafe fn fn_80020AEC(jobj: *mut HSD_JObj, out: *mut Mtx) {
    let mut tmp: Mtx = [[0.0; 4]; 3];
    let mut col: Vec3 = core::mem::zeroed();
    let outp = out as *mut c_void;

    ext::HSD_MtxInverseConcat(
        HSD_JObjGetMtxPtr(HSD_JObjGetParent(jobj)),
        HSD_JObjGetMtxPtr(jobj),
        outp,
    );

    for i in 0..3 {
        col.x = (*out)[0][i];
        col.y = (*out)[1][i];
        col.z = (*out)[2][i];

        let mut mag = ext::PSVECMag(&raw mut col);
        if mag > 1e-10f32 {
            mag = 1.0f32 / mag;
        }

        let mut sy = (*jobj).mtx[1][i];
        let mut sx = (*jobj).mtx[0][i];
        let mut sz = (*jobj).mtx[2][i];
        sy *= sy;
        sx *= sx;
        sz *= sz;
        let mut scale_sq = sy + sx;
        scale_sq = sz + scale_sq;

        if scale_sq > 0.0f32 {
            let e = newton_rsqrt(scale_sq);
            let scale_mag = (scale_sq as f64 * e) as f32;
            scale_sq = scale_mag;
        }

        let factor = mag * scale_sq;
        col.x *= factor;
        col.y *= factor;
        col.z *= factor;
        (*out)[0][i] = col.x;
        (*out)[1][i] = col.y;
        (*out)[2][i] = col.z;
    }

    let mut cur = HSD_JObjGetParent(jobj);
    while !cur.is_null() {
        if !HSD_JObjGetParent(cur).is_null() {
            ext::HSD_MtxInverseConcat(
                HSD_JObjGetMtxPtr(HSD_JObjGetParent(cur)),
                HSD_JObjGetMtxPtr(cur),
                (&raw mut tmp) as *mut c_void,
            );
        } else {
            ext::PSMTXCopy(HSD_JObjGetMtxPtr(cur), (&raw mut tmp) as *mut c_void);
        }

        for i in 0..3 {
            col.x = tmp[0][i];
            col.y = tmp[1][i];
            col.z = tmp[2][i];

            let mut mag = ext::PSVECMag(&raw mut col);
            if mag > 0.00001f32 {
                mag = 1.0f32 / mag;
            }

            col.x *= mag;
            col.y *= mag;
            col.z *= mag;
            tmp[0][i] = col.x;
            tmp[1][i] = col.y;
            tmp[2][i] = col.z;
        }

        ext::PSMTXConcat((&raw mut tmp) as *mut c_void, outp, outp);
        cur = HSD_JObjGetParent(cur);
    }
}

/// 020E38: turns `jobj` about its Z axis towards `dir`, clamped to `[-min_angle, max_angle]`.
pub unsafe fn lbBgFlash_80020E38(jobj: *mut HSD_JObj, dir: *mut Vec3, max_angle: f32, min_angle: f32, _unused: f32) {
    let dx2 = (*dir).x * (*dir).x;
    let dy2 = (*dir).y * (*dir).y;
    let dz2 = (*dir).z * (*dir).z;
    if dx2 + dy2 + dz2 == 0.0f32 {
        return;
    }

    HSD_JObjSetupMatrix(jobj);

    let z_col_y = (*jobj).mtx[1][2];
    let mut z_col_x = (*jobj).mtx[0][2];
    let z_col_z = (*jobj).mtx[2][2];
    z_col_x = fmadds(z_col_x, z_col_x, z_col_y * z_col_y);
    let mut mag_sq = z_col_x;
    mag_sq = fmadds(z_col_z, z_col_z, mag_sq);
    if mag_sq > 0.0f32 {
        let e = newton_rsqrt(mag_sq);
        let tmp = (mag_sq as f64 * e) as f32;
        mag_sq = tmp;
    }

    if mag_sq != 0.0f32 {
        let z_div_mag = z_col_z / mag_sq;
        let mut angle = ext::atan2f(-(*dir).x * z_div_mag, (*dir).y);

        if angle > max_angle {
            angle = max_angle;
        }
        if angle < -min_angle {
            angle = -min_angle;
        }

        if (*jobj).flags & USE_QUATERNION == 0 {
            HSD_JObjSetRotationZ(jobj, angle + HSD_JObjGetRotationZ(jobj));
        } else {
            let mut quat_mtx: Mtx = [[0.0; 4]; 3];
            let mut rot_mtx: Mtx = [[0.0; 4]; 3];
            let mut result_mtx: Mtx = [[0.0; 4]; 3];
            ext::PSMTXQuat((&raw mut quat_mtx) as *mut c_void, &raw mut (*jobj).rotate);
            ext::MTXRotRad((&raw mut rot_mtx) as *mut c_void, b'z' as i8, angle);
            ext::PSMTXConcat(
                (&raw mut quat_mtx) as *mut c_void,
                (&raw mut rot_mtx) as *mut c_void,
                (&raw mut result_mtx) as *mut c_void,
            );
            ext::MatToQuat((&raw mut result_mtx) as *mut c_void, &raw mut (*jobj).rotate);
            HSD_JObjSetMtxDirty(jobj);
        }

        HSD_JObjSetupMatrix(jobj);
    }
}

/// 02113C: rotates `jobj` by `angle` about the world-space `axis`.
pub unsafe fn fn_8002113C(jobj: *mut HSD_JObj, axis: *mut Vec3, angle: f32) {
    let mut tmp_mtx: Mtx = [[0.0; 4]; 3];
    let mut rot_mtx: Mtx = [[0.0; 4]; 3];
    let mut result: Mtx = [[0.0; 4]; 3];
    // The C reads and writes `rot` (a Vec3) through a `Quaternion*`: the .w travels beside it.
    let mut rot: Quaternion = core::mem::zeroed();
    let mut rot2: Quaternion = core::mem::zeroed();
    let mut local_axis: Vec3 = core::mem::zeroed();
    let mut mtx: Mtx = [[0.0; 4]; 3];

    HSD_JObjSetupMatrix(jobj);
    fn_80020AEC(jobj, &raw mut mtx);
    ext::PSMTXTranspose((&raw mut mtx) as *mut c_void, (&raw mut mtx) as *mut c_void);
    ext::PSMTXMultVec((&raw mut mtx) as *mut c_void, axis, &raw mut local_axis);
    ext::PSMTXRotAxisRad((&raw mut rot_mtx) as *mut c_void, &raw mut local_axis, -angle);

    if (*jobj).flags & USE_QUATERNION == 0 {
        HSD_JObjGetRotation(jobj, &raw mut rot);
        ext::HSD_MkRotationMtx((&raw mut tmp_mtx) as *mut c_void, (&raw mut rot) as *mut Vec3);
        ext::PSMTXConcat(
            (&raw mut tmp_mtx) as *mut c_void,
            (&raw mut rot_mtx) as *mut c_void,
            (&raw mut result) as *mut c_void,
        );
        ext::HSD_QuatLib_8037EB28((&raw mut result) as *mut c_void, (&raw mut rot) as *mut Vec3);
        HSD_JObjSetRotation(jobj, &raw mut rot);
    } else {
        HSD_JObjGetRotation(jobj, &raw mut rot2);
        ext::HSD_MtxQuat((&raw mut tmp_mtx) as *mut c_void, &raw mut rot2);
        ext::PSMTXConcat(
            (&raw mut tmp_mtx) as *mut c_void,
            (&raw mut rot_mtx) as *mut c_void,
            (&raw mut result) as *mut c_void,
        );
        ext::MatToQuat((&raw mut result) as *mut c_void, &raw mut rot2);
        HSD_JObjSetRotation(jobj, &raw mut rot2);
    }
}

/// `calc_acos` (static inline)
#[inline]
pub fn calc_acos(value: f32) -> f32 {
    ext::acosf(value)
}

/// `sqrtf_store` (static inline): MSL-style `sqrtf` with the Newton steps' `3 - g*g*x` fused.
#[inline]
pub fn sqrtf_store(x: f32, y: &mut f32) -> f32 {
    if x > 0.0f32 {
        let mut guess = frsqrte(x as f64);
        for _ in 0..3 {
            guess = 0.5 * guess * (-(guess * guess)).mul_add(x as f64, 3.0);
        }
        *y = (x as f64 * guess) as f32;
        return *y;
    }
    x
}

/// 021410: two-bone IK. Projects the chain onto the plane of `jobj1`'s Z axis, solves the
/// joint angles by the law of cosines (soft-limited near full extension) and rotates both joints.
pub unsafe fn lbBgFlash_80021410(data: *mut IKState) {
    let d = &mut *data;
    let mut axis: Vec3 = core::mem::zeroed();
    let mut diff_pos0_pos1: Vec3 = core::mem::zeroed();
    let mut pos1_from_pos0: Vec3 = core::mem::zeroed();
    let mut temp_delta: Vec3 = core::mem::zeroed();
    let mut sin_mag = 0.0f32;
    let mut len_ab_mag = 0.0f32;
    let mut len_bc_mag = 0.0f32;
    let mut len_ac_mag = 0.0f32;

    HSD_JObjSetupMatrix(d.jobj1);

    axis.x = (*d.jobj1).mtx[0][2];
    axis.y = (*d.jobj1).mtx[1][2];
    axis.z = (*d.jobj1).mtx[2][2];
    ext::lbVector_Normalize(&raw mut axis);

    let mut dot: f32;
    {
        let ny = axis.y;
        let nz = axis.z;
        let nx = axis.x;
        let mut dd: f32;

        dot = -fmadds(nz, d.pos1.z, fmadds(nx, d.pos1.x, ny * d.pos1.y));

        let x = d.pos4.x;

        dd = -(dot + fmadds(d.pos4.z, nz, fmadds(x, nx, d.pos4.y * ny)));
        d.pos4.x = fmadds(dd, nx, x);
        d.pos4.y = fmadds(dd, ny, d.pos4.y);
        d.pos4.z = fmadds(dd, nz, d.pos4.z);

        {
            let x = d.pos0.x;
            dd = -(dot + fmadds(d.pos0.z, nz, fmadds(x, nx, d.pos0.y * ny)));
            d.pos0.x = fmadds(dd, nx, x);
        }
        d.pos0.y = fmadds(dd, ny, d.pos0.y);
        d.pos0.z = fmadds(dd, nz, d.pos0.z);

        {
            let x = d.pos1.x;
            dd = -(dot + fmadds(d.pos1.z, nz, fmadds(x, nx, d.pos1.y * ny)));
            d.pos1.x = fmadds(dd, nx, x);
        }
        d.pos1.y = fmadds(dd, ny, d.pos1.y);
        d.pos1.z = fmadds(dd, nz, d.pos1.z);
    }

    let p_diff = ext::lbVector_Diff(&raw mut d.pos0, &raw mut d.pos1, &raw mut diff_pos0_pos1);
    dot = fmadds(axis.z, (*p_diff).z, fmadds(axis.x, (*p_diff).x, axis.y * (*p_diff).y));
    let sin_val = sqrtf_store(fmadds(-dot, dot, 1.0f32), &mut sin_mag);
    d.len0 = d.len0 * sin_val;

    ext::lbVector_Diff(&raw mut d.pos4, &raw mut d.pos0, &raw mut temp_delta);
    ext::lbVector_Diff(&raw mut d.pos1, &raw mut d.pos0, &raw mut pos1_from_pos0);
    let angle1 = ext::lbVector_Angle(&raw mut temp_delta, &raw mut pos1_from_pos0);

    ext::lbVector_Diff(&raw mut d.pos2, &raw mut d.pos1, &raw mut temp_delta);
    let pi: f64 = 3.141592653589793;
    let angle2 = (pi - ext::lbVector_Angle(&raw mut temp_delta, &raw mut pos1_from_pos0) as f64) as f32;

    let mut dx = d.pos0.x - d.pos4.x;
    let mut dz = d.pos0.z;
    dz -= d.pos4.z;
    let mut dy = d.pos0.y - d.pos4.y;
    dx *= dx;
    dy *= dy;
    dz *= dz;
    let mut len_ab = sqrtf_store(dz + (dx + dy), &mut len_ab_mag);

    dx = d.pos0.x - d.pos1.x;
    dz = d.pos0.z;
    dz -= d.pos1.z;
    dy = d.pos0.y - d.pos1.y;
    dx *= dx;
    dy *= dy;
    dz *= dz;
    let mut len_bc = sqrtf_store(dz + (dx + dy), &mut len_bc_mag);
    d.len0 = len_bc;

    dx = d.pos1.x - d.pos3.x;
    dz = d.pos1.z;
    dz -= d.pos3.z;
    dy = d.pos1.y - d.pos3.y;
    dx *= dx;
    dy *= dy;
    dz *= dz;
    let mut len_ac = sqrtf_store(dz + (dx + dy), &mut len_ac_mag);
    d.len1 = len_ac;

    let ten = 10.0f32;
    len_bc = d.len0;
    len_ac = d.len1;
    let mut sum_len = ten * (len_bc + len_ac);
    let eleven = 11.0f32;
    sum_len /= eleven;
    let mut sum_pow = sum_len * (sum_len * sum_len);
    for _ in 0..8 {
        sum_pow = sum_len * sum_pow;
    }
    let mut len_pow = len_ab * (len_ab * (len_ab * len_ab));
    for _ in 0..4 {
        len_pow = len_ab * len_pow;
    }
    len_pow = len_ab * len_pow;
    let last = len_ab * len_pow;
    if len_ab > sum_len {
        len_ab = ((eleven * sum_len) / ten) + (-sum_pow / (ten * last));
    }

    let a2 = len_bc * len_bc;
    let b2 = len_ab * len_ab;
    let two_a = 2.0f32 * len_bc;
    let c2 = len_ac * len_ac;

    let mut cos1 = ((a2 + b2) - c2) / (two_a * len_ab);
    let mut cos2 = ((a2 + c2) - b2) / (two_a * len_ac);

    if cos1 > 1.0f32 {
        cos1 = 1.0;
    } else if cos1 < -1.0f32 {
        cos1 = -1.0;
    }

    if cos2 > 1.0f32 {
        cos2 = 1.0;
    } else if cos2 < -1.0f32 {
        cos2 = -1.0;
    }

    let mut acos1 = calc_acos(cos1);
    let mut acos2 = calc_acos(cos2);
    let rem: f64 = 3.141592653589793 - acos2 as f64;
    if rem < 0.1745329201221466 {
        let ratio = (rem.abs() / 0.1745329201221466) as f32;
        acos2 = (2.9670597334676465 + (((ratio as f64) * ((acos2 as f64) - 2.9670597334676465)) as f32) as f64) as f32;
    }

    acos1 -= angle1;
    acos2 -= angle2;
    fn_8002113C(d.jobj0, &raw mut axis, acos1);
    fn_8002113C(d.jobj1, &raw mut axis, acos2);
}
