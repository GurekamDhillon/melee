//! Port of the logic half of `src/sysdolphin/baselib/cobj.c`: the camera object's eye / interest
//! / up / roll math, the viewing matrix and its inverse, the projection parameters and their
//! getters and setters, and `makeProjectionMtx`.
//!
//! Native shape: `HsdCObj<R>` owns its eye-position and interest WObjs (`Option<HsdWObj<R>>`;
//! the WObj positions may be local to an AObj target JObj, so the functions that read them take
//! the [`JObjTree`] and its [`JObjHost`], as wobj.rs does). The C union `u { roll; up }` is kept
//! as one `Vec3` (`roll` is `u.x`, as in the C's storage). `projection_param` is the C's four
//! floats (`perspective.fov/aspect` alias `frustum/ortho.top/bottom`). `proj_mtx` (the inverse
//! viewing matrix, HSD_MtxAlloc'd on first use) is `Option<Mtx>`; a new one starts zeroed (the
//! C's object memory is not cleared, and `PSMTXInverse` leaves it untouched for a singular
//! view). GcLayout is delegated to `melee_types::HSD_CObj` (size 0x94 with the TARGET_PC tail).
//!
//! `HSD_CObjUpdateView` (TARGET_PC) reads `View_ProjectionScaleBits()` (platform); the functions
//! that call it take that value as `proj_scale` (the float the bits encode).
//!
//! Float rule (`fma_sites.py`): roll2upvec fuses `y*y + z*z` and `x*x + z*z` (ported with
//! `fmadds`); math_ppc `sqrtf` is `gekko::sqrtf`; the PSMTX/PSVEC/C_MTXLookAt/MTX44 calls are the
//! round 1 `mtx` ports. `DegToRad(a)` is `a * 0.01745329252f` (f32).
//!
//! Not ported here: the display half (EraseScreen, SetCurrent / EndCurrent and the four setup*Camera
//! functions: GX), the anim functions (RemoveAnim*, AddAnim, Anim, ReqAnim, CObjUpdateFunc: need
//! the camera AObj wiring), load / init / class (CObjLoad, HSD_CObjInit, LoadDesc, Alloc,
//! CObjInit, CObjRelease, CObjAmnesia, CObjInfoInit, SetDefaultClass), `HSD_CObjGetCurrent`.
use crate::gekko::{sqrtf, tanf};
use crate::hsd_mtx::fabsf_bitwise;
use crate::jobj::{JObjHost, JObjTree};
use crate::lbtrigf::atan2f;
use crate::mtx::{self, Mtx, Mtx44, Vec3};
use crate::wobj::HsdWObj;
use melee_core::fp::fmadds;

/// `PROJ_PERSPECTIVE`.
pub const PROJ_PERSPECTIVE: u8 = 1;
/// `PROJ_FRUSTUM`.
pub const PROJ_FRUSTUM: u8 = 2;
/// `PROJ_ORTHO`.
pub const PROJ_ORTHO: u8 = 3;
/// `GX_PERSPECTIVE`.
pub const GX_PERSPECTIVE: u32 = 0;
/// `GX_ORTHOGRAPHIC`.
pub const GX_ORTHOGRAPHIC: u32 = 1;
/// CObj flag bit 0: `u` holds an up vector (else a roll).
pub const COBJ_UP_VECTOR: u32 = 1;
/// util.h `FLT_MIN`.
pub const FLT_MIN: f32 = 1.17549435e-38;

/// `HSD_RectF32`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RectF32 {
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
}

/// `HSD_RectS16`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RectS16 {
    pub xmin: i16,
    pub xmax: i16,
    pub ymin: i16,
    pub ymax: i16,
}

/// `Scissor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scissor {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
}

/// Native `HSD_CObj` (GameCube record `melee_types::HSD_CObj`, 0x94 bytes on TARGET_PC).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdCObj<R> {
    pub flags: u32,
    pub viewport: RectF32,
    pub scissor: Scissor,
    pub eyepos: Option<HsdWObj<R>>,
    pub interest: Option<HsdWObj<R>>,
    /// The union `u`: `roll` is `u.x`, `up` is all three.
    pub u: Vec3,
    pub near: f32,
    pub far: f32,
    /// perspective {fov, aspect} / frustum, ortho {top, bottom, left, right}.
    pub projection_param: [f32; 4],
    pub projection_type: u8,
    pub view_mtx: Mtx,
    /// `proj_mtx`: the inverse viewing matrix (None = NULL).
    pub proj_mtx: Option<Mtx>,
    pub view_authored_aspect: f32,
    pub view_auto: u8,
}

impl<R> melee_core::gc::GcLayout for HsdCObj<R> {
    const GC_NAME: &'static str = <melee_types::HSD_CObj as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_CObj as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_CObj as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_CObj as melee_core::gc::GcLayout>::GC_FIELDS;
}

impl<R> Default for HsdCObj<R> {
    /// Zeroed object (what CObjInit leaves before a load).
    fn default() -> Self {
        HsdCObj {
            flags: 0,
            viewport: RectF32::default(),
            scissor: Scissor::default(),
            eyepos: None,
            interest: None,
            u: Vec3::default(),
            near: 0.0,
            far: 0.0,
            projection_param: [0.0; 4],
            projection_type: 0,
            view_mtx: [[0.0; 4]; 3],
            proj_mtx: None,
            view_authored_aspect: 0.0,
            view_auto: 0,
        }
    }
}

/// util.h `vec_normalize_check`: -1 (dst untouched) if every component is within `FLT_MIN` of 0.
pub fn vec_normalize_check(src: &Vec3, dst: &mut Vec3) -> i32 {
    if fabsf_bitwise(src.x) <= FLT_MIN && fabsf_bitwise(src.y) <= FLT_MIN && fabsf_bitwise(src.z) <= FLT_MIN {
        return -1;
    }
    *dst = mtx::vec_normalize(src);
    0
}

#[inline]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.01745329252f32
}

/// `HSD_CObjSetMtxDirty`.
pub fn set_mtx_dirty<R>(cobj: &mut HsdCObj<R>) {
    cobj.flags |= (1 << 30) | (1 << 31);
}

/// `HSD_CObjMtxIsDirty`.
pub fn mtx_is_dirty<R>(cobj: &HsdCObj<R>) -> bool {
    cobj.flags & (1 << 30) != 0
        || cobj.eyepos.as_ref().is_some_and(|w| w.flags & 2 != 0)
        || cobj.interest.as_ref().is_some_and(|w| w.flags & 2 != 0)
}

/// `HSD_CObjGetFlags`.
pub fn get_flags<R>(cobj: &HsdCObj<R>) -> u32 {
    cobj.flags
}
/// `HSD_CObjSetFlags`.
pub fn set_flags<R>(cobj: Option<&mut HsdCObj<R>>, flags: u32) {
    if let Some(c) = cobj {
        c.flags |= flags;
    }
}
/// `HSD_CObjClearFlags`.
pub fn clear_flags<R>(cobj: Option<&mut HsdCObj<R>>, flags: u32) {
    if let Some(c) = cobj {
        c.flags &= !flags;
    }
}

/// `HSD_CObjGetInterestWObj`.
pub fn get_interest_wobj<R>(cobj: &HsdCObj<R>) -> Option<&HsdWObj<R>> {
    cobj.interest.as_ref()
}
/// `HSD_CObjSetInterestWObj` (the C just stores the pointer; natively the CObj takes it).
pub fn set_interest_wobj<R>(cobj: &mut HsdCObj<R>, interest: Option<HsdWObj<R>>) {
    cobj.interest = interest;
}
/// `HSD_CObjGetEyePositionWObj`.
pub fn get_eye_position_wobj<R>(cobj: &HsdCObj<R>) -> Option<&HsdWObj<R>> {
    cobj.eyepos.as_ref()
}
/// `HSD_CObjSetEyePositionWObj`.
pub fn set_eye_position_wobj<R>(cobj: &mut HsdCObj<R>, eyepos: Option<HsdWObj<R>>) {
    cobj.eyepos = eyepos;
}

/// `HSD_CObjGetInterest` (`HSD_WObjGetPosition` of the interest WObj).
pub fn get_interest<R, D, H>(cobj: &mut HsdCObj<R>, tree: &mut JObjTree<D, R>, host: &mut H) -> Vec3
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    cobj.interest.as_mut().expect("HSD_WObjGetPosition: wobj").get_position(tree, host)
}
/// `HSD_CObjSetInterest`.
pub fn set_interest<R>(cobj: &mut HsdCObj<R>, p: &Vec3) {
    cobj.interest.as_mut().expect("HSD_WObjSetPosition: wobj").set_position(p);
}
/// `HSD_CObjGetEyePosition`.
pub fn get_eye_position<R, D, H>(cobj: &mut HsdCObj<R>, tree: &mut JObjTree<D, R>, host: &mut H) -> Vec3
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    cobj.eyepos.as_mut().expect("HSD_WObjGetPosition: wobj").get_position(tree, host)
}
/// `HSD_CObjSetEyePosition`.
pub fn set_eye_position<R>(cobj: &mut HsdCObj<R>, p: &Vec3) {
    cobj.eyepos.as_mut().expect("HSD_WObjSetPosition: wobj").set_position(p);
}

/// `HSD_CObjGetEyeVector`: 0 and the unit eye->interest vector, or -1 and (0, 0, -1).
pub fn get_eye_vector<R, D, H>(cobj: Option<&mut HsdCObj<R>>, tree: &mut JObjTree<D, R>, host: &mut H) -> (i32, Vec3)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    if let Some(c) = cobj {
        if c.eyepos.is_some() && c.interest.is_some() {
            let eyepos = get_eye_position(c, tree, host);
            let interest = get_interest(c, tree, host);
            let mut eye = mtx::vec_sub(&interest, &eyepos);
            let v = eye;
            if vec_normalize_check(&v, &mut eye) == 0 {
                return (0, eye);
            }
        }
    }
    (-1, Vec3 { x: 0.0, y: 0.0, z: -1.0 })
}

/// `HSD_CObjGetEyeDistance`.
pub fn get_eye_distance<R, D, H>(cobj: Option<&mut HsdCObj<R>>, tree: &mut JObjTree<D, R>, host: &mut H) -> f32
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let Some(c) = cobj else { return 0.0 };
    assert!(c.eyepos.is_some(), "HSD_ASSERT(807, cobj->eyepos)");
    assert!(c.interest.is_some(), "HSD_ASSERT(808, cobj->interest)");
    let position = get_eye_position(c, tree, host);
    let interest = get_interest(c, tree, host);
    mtx::vec_mag(&mtx::vec_sub(&interest, &position))
}

/// `upvec2roll` (static).
pub fn upvec2roll<R, D, H>(cobj: &mut HsdCObj<R>, up: &Vec3, tree: &mut JObjTree<D, R>, host: &mut H) -> f32
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let (r, eye) = get_eye_vector(Some(cobj), tree, host);
    if r != 0 {
        return 0.0;
    }
    let mut dot = mtx::vec_dot(up, &eye).abs();
    dot = 1.0 - dot;
    if dot < FLT_MIN {
        0.0
    } else {
        let orig = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let uy = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
        let vmtx = mtx::look_at(&orig, &uy, &eye);
        let v = mtx::mult_vec_sr(&vmtx, up);
        if fabsf_bitwise(v.y) == 0.0 {
            if -v.x >= 0.0 {
                1.5707963267948966f64 as f32
            } else {
                -1.5707963267948966f64 as f32
            }
        } else {
            atan2f(-v.x, v.y)
        }
    }
}

/// `roll2upvec` (static): (result, up). `up` is only written on success.
pub fn roll2upvec<R, D, H>(
    cobj: &mut HsdCObj<R>,
    roll: f32,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
) -> (i32, Option<Vec3>)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let (res, eye) = get_eye_vector(Some(cobj), tree, host);
    if res != 0 {
        return (res, None);
    }
    let mut v0 = Vec3::default();
    if 1.0 - (eye.y.abs() as f64) < 0.0001 {
        v0.x = sqrtf(fmadds(eye.y, eye.y, eye.z * eye.z));
        v0.y = eye.y * (-eye.x / v0.x);
        v0.z = eye.z * (-eye.x / v0.x);
    } else {
        v0.y = sqrtf(fmadds(eye.x, eye.x, eye.z * eye.z));
        v0.x = eye.x * (-eye.y / v0.y);
        v0.z = eye.z * (-eye.y / v0.y);
    }
    let m = mtx::rot_axis_rad(&eye, -roll);
    let v1 = mtx::mult_vec_sr(&m, &v0);
    (0, Some(mtx::vec_normalize(&v1)))
}

/// `HSD_CObjGetUpVector`: 0 and the up vector, or -1 and (0, 1, 0).
pub fn get_up_vector<R, D, H>(cobj: Option<&mut HsdCObj<R>>, tree: &mut JObjTree<D, R>, host: &mut H) -> (i32, Vec3)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    if let Some(c) = cobj {
        if c.flags & COBJ_UP_VECTOR != 0 {
            return (0, c.u);
        }
        let roll = c.u.x;
        if let (0, Some(up)) = roll2upvec(c, roll, tree, host) {
            return (0, up);
        }
    }
    (-1, Vec3 { x: 0.0, y: 1.0, z: 0.0 })
}

/// `HSD_CObjSetUpVector` (`up` None = NULL: no-op).
pub fn set_up_vector<R, D, H>(cobj: Option<&mut HsdCObj<R>>, up: Option<&Vec3>, tree: &mut JObjTree<D, R>, host: &mut H)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let (Some(c), Some(up)) = (cobj, up) else { return };
    if c.flags & COBJ_UP_VECTOR != 0 {
        let mut v = Vec3::default();
        let up = if vec_normalize_check(up, &mut v) != 0 { Vec3 { x: 0.0, y: 1.0, z: 0.0 } } else { v };
        if c.u.x != up.x || c.u.y != up.y || c.u.z != up.z {
            set_mtx_dirty(c);
        }
        c.u = up;
    } else {
        let roll = upvec2roll(c, up, tree, host);
        set_roll(Some(c), roll, tree, host);
    }
}

/// `HSD_CObjGetLeftVector`: 0 and the unit left vector, or -1 and (1, 0, 0).
pub fn get_left_vector<R, D, H>(cobj: Option<&mut HsdCObj<R>>, tree: &mut JObjTree<D, R>, host: &mut H) -> (i32, Vec3)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    if let Some(c) = cobj {
        let (r, eye) = get_eye_vector(Some(&mut *c), tree, host);
        if r == 0 {
            let (r, up) = get_up_vector(Some(c), tree, host);
            if r == 0 {
                let l = mtx::vec_cross(&up, &eye);
                let mut left = l;
                if vec_normalize_check(&l, &mut left) == 0 {
                    return (0, left);
                }
            }
        }
    }
    (-1, Vec3 { x: 1.0, y: 0.0, z: 0.0 })
}

/// `HSD_CObjSetRoll`.
pub fn set_roll<R, D, H>(cobj: Option<&mut HsdCObj<R>>, roll: f32, tree: &mut JObjTree<D, R>, host: &mut H)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let Some(c) = cobj else { return };
    if c.flags & COBJ_UP_VECTOR != 0 {
        // The C passes an uninitialised local when roll2upvec fails (zero in the shipping build).
        let up = roll2upvec(c, roll, tree, host).1.unwrap_or_default();
        set_up_vector(Some(c), Some(&up), tree, host);
    } else {
        if c.u.x != roll {
            c.flags |= 0xC000_0000;
        }
        c.u.x = roll;
    }
}

/// The body shared by `HSD_CObjSetupViewingMtx`, `GetViewingMtxPtr` and `GetInvViewingMtxPtr`.
fn rebuild_view<R, D, H>(cobj: &mut HsdCObj<R>, tree: &mut JObjTree<D, R>, host: &mut H)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    if cobj.flags & 2 == 0 && mtx_is_dirty(cobj) {
        let eyepos = get_eye_position(cobj, tree, host);
        let up = get_up_vector(Some(&mut *cobj), tree, host).1;
        let interest = get_interest(cobj, tree, host);
        cobj.view_mtx = mtx::look_at(&eyepos, &up, &interest);
        cobj.eyepos.as_mut().expect("eyepos").clear_flags(2);
        cobj.interest.as_mut().expect("interest").clear_flags(2);
        cobj.flags &= !0x4000_0000;
        cobj.flags |= 0x8000_0000;
    }
}

/// `HSD_CObjSetupViewingMtx` (the TARGET_PC `diag_cobj_view` instrumentation is not ported: it
/// only logs).
pub fn setup_viewing_mtx<R, D, H>(cobj: &mut HsdCObj<R>, tree: &mut JObjTree<D, R>, host: &mut H)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    rebuild_view(cobj, tree, host);
}

/// `HSD_CObjGetViewingMtxPtr` (rebuild if dirty, then `HSD_CObjGetViewingMtxPtrDirect`).
pub fn get_viewing_mtx_ptr<'a, R, D, H>(cobj: &'a mut HsdCObj<R>, tree: &mut JObjTree<D, R>, host: &mut H) -> &'a Mtx
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    rebuild_view(cobj, tree, host);
    &cobj.view_mtx
}

/// `HSD_CObjGetViewingMtx`.
pub fn get_viewing_mtx<R, D, H>(cobj: &mut HsdCObj<R>, tree: &mut JObjTree<D, R>, host: &mut H) -> Mtx
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    *get_viewing_mtx_ptr(cobj, tree, host)
}

/// `HSD_CObjGetInvViewingMtxPtrDirect`: None where the C returns NULL (no inverse ever built).
pub fn get_inv_viewing_mtx_ptr_direct<R>(cobj: &mut HsdCObj<R>) -> Option<&Mtx> {
    if cobj.flags & (1 << 31) != 0 {
        let cur = cobj.proj_mtx.get_or_insert([[0.0; 4]; 3]);
        if let Some(inv) = mtx::inverse(&cobj.view_mtx) {
            *cur = inv;
        }
        cobj.flags &= !(1 << 31);
    }
    cobj.proj_mtx.as_ref()
}

/// `HSD_CObjGetInvViewingMtxPtr`.
pub fn get_inv_viewing_mtx_ptr<'a, R, D, H>(
    cobj: &'a mut HsdCObj<R>,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
) -> Option<&'a Mtx>
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    rebuild_view(cobj, tree, host);
    get_inv_viewing_mtx_ptr_direct(cobj)
}

/// `HSD_CObjUpdateView` (TARGET_PC): an auto-view perspective camera's aspect follows the
/// window (`proj_scale` = the float `View_ProjectionScaleBits()` encodes).
pub fn update_view<R>(cobj: Option<&mut HsdCObj<R>>, proj_scale: f32) {
    if let Some(c) = cobj {
        if c.view_auto != 0 && c.projection_type == PROJ_PERSPECTIVE {
            c.projection_param[1] = c.view_authored_aspect * proj_scale;
        }
    }
}

/// `HSD_CObjGetFov`.
pub fn get_fov<R>(cobj: Option<&HsdCObj<R>>) -> f32 {
    match cobj {
        Some(c) if c.projection_type == PROJ_PERSPECTIVE => c.projection_param[0],
        _ => 0.0,
    }
}
/// `HSD_CObjSetFov`.
pub fn set_fov<R>(cobj: Option<&mut HsdCObj<R>>, fov: f32) {
    if let Some(c) = cobj {
        if c.projection_type == PROJ_PERSPECTIVE {
            c.projection_param[0] = fov;
        }
    }
}
/// `HSD_CObjGetAspect`.
pub fn get_aspect<R>(mut cobj: Option<&mut HsdCObj<R>>, proj_scale: f32) -> f32 {
    update_view(cobj.as_deref_mut(), proj_scale);
    match cobj {
        Some(c) if c.projection_type == PROJ_PERSPECTIVE => c.projection_param[1],
        _ => 0.0,
    }
}
/// `HSD_CObjSetAspect`.
pub fn set_aspect<R>(cobj: Option<&mut HsdCObj<R>>, aspect: f32, proj_scale: f32) {
    let Some(c) = cobj else { return };
    if c.projection_type != PROJ_PERSPECTIVE {
        return;
    }
    c.projection_param[1] = aspect;
    if c.view_auto != 0 {
        c.view_authored_aspect = aspect;
        update_view(Some(c), proj_scale);
    }
}
/// `HSD_CObjGetAuthoredAspect` (TARGET_PC).
pub fn get_authored_aspect<R>(cobj: Option<&mut HsdCObj<R>>, proj_scale: f32) -> f32 {
    if let Some(c) = &cobj {
        if c.view_auto != 0 {
            return c.view_authored_aspect;
        }
    }
    get_aspect(cobj, proj_scale)
}

#[inline]
fn half_tan<R>(c: &HsdCObj<R>) -> f32 {
    tanf(0.5f32 * deg_to_rad(c.projection_param[0]))
}

/// `HSD_CObjGetTop`.
pub fn get_top<R>(cobj: Option<&HsdCObj<R>>) -> f32 {
    let Some(c) = cobj else { return 0.0 };
    match c.projection_type {
        PROJ_PERSPECTIVE => c.near * half_tan(c),
        PROJ_FRUSTUM | PROJ_ORTHO => c.projection_param[0],
        _ => 0.0,
    }
}
/// `HSD_CObjGetBottom`.
pub fn get_bottom<R>(cobj: Option<&HsdCObj<R>>) -> f32 {
    let Some(c) = cobj else { return 0.0 };
    match c.projection_type {
        PROJ_PERSPECTIVE => -c.near * half_tan(c),
        PROJ_FRUSTUM | PROJ_ORTHO => c.projection_param[1],
        _ => 0.0,
    }
}
/// `HSD_CObjGetLeft`.
pub fn get_left<R>(mut cobj: Option<&mut HsdCObj<R>>, proj_scale: f32) -> f32 {
    update_view(cobj.as_deref_mut(), proj_scale);
    let Some(c) = cobj else { return 0.0 };
    match c.projection_type {
        PROJ_PERSPECTIVE => c.projection_param[1] * (-c.near * half_tan(c)),
        PROJ_FRUSTUM | PROJ_ORTHO => c.projection_param[2],
        _ => 0.0,
    }
}
/// `HSD_CObjGetRight`.
pub fn get_right<R>(mut cobj: Option<&mut HsdCObj<R>>, proj_scale: f32) -> f32 {
    update_view(cobj.as_deref_mut(), proj_scale);
    let Some(c) = cobj else { return 0.0 };
    match c.projection_type {
        PROJ_PERSPECTIVE => c.projection_param[1] * (c.near * half_tan(c)),
        PROJ_FRUSTUM | PROJ_ORTHO => c.projection_param[3],
        _ => 0.0,
    }
}
fn set_frustum_slot<R>(cobj: Option<&mut HsdCObj<R>>, i: usize, v: f32) {
    if let Some(c) = cobj {
        if c.projection_type == PROJ_FRUSTUM || c.projection_type == PROJ_ORTHO {
            c.projection_param[i] = v;
        }
    }
}
/// `HSD_CObjSetTop` (perspective: no-op).
pub fn set_top<R>(cobj: Option<&mut HsdCObj<R>>, v: f32) {
    set_frustum_slot(cobj, 0, v)
}
/// `HSD_CObjSetBottom`.
pub fn set_bottom<R>(cobj: Option<&mut HsdCObj<R>>, v: f32) {
    set_frustum_slot(cobj, 1, v)
}
/// `HSD_CObjSetLeft`.
pub fn set_left<R>(cobj: Option<&mut HsdCObj<R>>, v: f32) {
    set_frustum_slot(cobj, 2, v)
}
/// `HSD_CObjSetRight`.
pub fn set_right<R>(cobj: Option<&mut HsdCObj<R>>, v: f32) {
    set_frustum_slot(cobj, 3, v)
}
/// `HSD_CObjGetNear`.
pub fn get_near<R>(cobj: Option<&HsdCObj<R>>) -> f32 {
    cobj.map_or(0.0, |c| c.near)
}
/// `HSD_CObjSetNear`.
pub fn set_near<R>(cobj: Option<&mut HsdCObj<R>>, v: f32) {
    if let Some(c) = cobj {
        c.near = v;
    }
}
/// `HSD_CObjGetFar`.
pub fn get_far<R>(cobj: Option<&HsdCObj<R>>) -> f32 {
    cobj.map_or(0.0, |c| c.far)
}
/// `HSD_CObjSetFar`.
pub fn set_far<R>(cobj: Option<&mut HsdCObj<R>>, v: f32) {
    if let Some(c) = cobj {
        c.far = v;
    }
}
/// `HSD_CObjGetScissor` (NULL cobj: `out` untouched).
pub fn get_scissor<R>(cobj: Option<&HsdCObj<R>>, out: &mut Scissor) {
    if let Some(c) = cobj {
        *out = c.scissor;
    }
}
/// `HSD_CObjSetScissor`.
pub fn set_scissor<R>(cobj: Option<&mut HsdCObj<R>>, s: &Scissor) {
    if let Some(c) = cobj {
        c.scissor = *s;
    }
}
/// `HSD_CObjSetScissorx4`.
pub fn set_scissor_x4<R>(cobj: Option<&mut HsdCObj<R>>, left: u16, right: u16, top: u16, bottom: u16) {
    if let Some(c) = cobj {
        c.scissor = Scissor { left, right, top, bottom };
    }
}
/// `HSD_CObjGetViewportf`.
pub fn get_viewportf<R>(cobj: Option<&HsdCObj<R>>, out: &mut RectF32) {
    if let Some(c) = cobj {
        *out = c.viewport;
    }
}
/// `HSD_CObjSetViewport` (s16 -> f32).
pub fn set_viewport<R>(cobj: Option<&mut HsdCObj<R>>, v: &RectS16) {
    if let Some(c) = cobj {
        c.viewport =
            RectF32 { xmin: v.xmin as f32, xmax: v.xmax as f32, ymin: v.ymin as f32, ymax: v.ymax as f32 };
    }
}
/// `HSD_CObjSetViewportf`.
pub fn set_viewportf<R>(cobj: Option<&mut HsdCObj<R>>, v: &RectF32) {
    if let Some(c) = cobj {
        c.viewport = *v;
    }
}
/// `HSD_CObjSetViewportfx4`.
pub fn set_viewportf_x4<R>(cobj: Option<&mut HsdCObj<R>>, left: f32, right: f32, top: f32, bottom: f32) {
    if let Some(c) = cobj {
        c.viewport = RectF32 { xmin: left, xmax: right, ymin: top, ymax: bottom };
    }
}
/// `HSD_CObjGetProjectionType` (NULL: 1).
pub fn get_projection_type<R>(mut cobj: Option<&mut HsdCObj<R>>, proj_scale: f32) -> i32 {
    update_view(cobj.as_deref_mut(), proj_scale);
    cobj.map_or(1, |c| c.projection_type as i32)
}
/// `HSD_CObjSetProjectionType` (stored as u8, truncating).
pub fn set_projection_type<R>(cobj: Option<&mut HsdCObj<R>>, t: u32) {
    if let Some(c) = cobj {
        c.projection_type = t as u8;
    }
}
/// `HSD_CObjSetPerspective`.
pub fn set_perspective<R>(cobj: Option<&mut HsdCObj<R>>, fov: f32, aspect: f32, proj_scale: f32) {
    let Some(c) = cobj else { return };
    c.projection_type = PROJ_PERSPECTIVE;
    c.projection_param[0] = fov;
    c.projection_param[1] = aspect;
    if c.view_auto != 0 {
        c.view_authored_aspect = aspect;
        update_view(Some(c), proj_scale);
    }
}
/// `HSD_CObjSetFrustum`.
pub fn set_frustum<R>(cobj: Option<&mut HsdCObj<R>>, top: f32, bottom: f32, left: f32, right: f32) {
    if let Some(c) = cobj {
        c.projection_type = PROJ_FRUSTUM;
        c.projection_param = [top, bottom, left, right];
    }
}
/// `HSD_CObjSetOrtho`.
pub fn set_ortho<R>(cobj: Option<&mut HsdCObj<R>>, top: f32, bottom: f32, left: f32, right: f32) {
    if let Some(c) = cobj {
        c.projection_type = PROJ_ORTHO;
        c.projection_param = [top, bottom, left, right];
    }
}
/// `HSD_CObjGetPerspective`: (fov, aspect) of a perspective camera, else None (outputs untouched).
pub fn get_perspective<R>(mut cobj: Option<&mut HsdCObj<R>>, proj_scale: f32) -> Option<(f32, f32)> {
    update_view(cobj.as_deref_mut(), proj_scale);
    match cobj {
        Some(c) if c.projection_type == PROJ_PERSPECTIVE => Some((c.projection_param[0], c.projection_param[1])),
        _ => None,
    }
}
/// `HSD_CObjGetOrtho`: (top, bottom, left, right) of an ortho camera, else None.
pub fn get_ortho<R>(cobj: Option<&HsdCObj<R>>) -> Option<[f32; 4]> {
    match cobj {
        Some(c) if c.projection_type == PROJ_ORTHO => Some(c.projection_param),
        _ => None,
    }
}

/// `makeProjectionMtx`: the GX projection type and `mtx` (left untouched, and type 0 =
/// GX_PERSPECTIVE from the zero-initialised local, for an unknown projection type).
pub fn make_projection_mtx<R>(cobj: &mut HsdCObj<R>, mtx: &mut Mtx44, proj_scale: f32) -> u32 {
    update_view(Some(&mut *cobj), proj_scale);
    let p = cobj.projection_param;
    match cobj.projection_type {
        PROJ_PERSPECTIVE => {
            *mtx = mtx::perspective(p[0], p[1], cobj.near, cobj.far);
            GX_PERSPECTIVE
        }
        PROJ_FRUSTUM => {
            *mtx = mtx::frustum(p[0], p[1], p[2], p[3], cobj.near, cobj.far);
            GX_PERSPECTIVE
        }
        PROJ_ORTHO => {
            *mtx = mtx::ortho(p[0], p[1], p[2], p[3], cobj.near, cobj.far);
            GX_ORTHOGRAPHIC
        }
        _ => 0,
    }
}
