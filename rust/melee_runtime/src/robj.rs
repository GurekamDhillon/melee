//! Port of `src/sysdolphin/baselib/robj.c` (round 7): reference objects (constraints, limits,
//! expressions, IK hints) and the constraint solver `HSD_RObjUpdateAll`.
//!
//! Native shape: an RObj chain (`robj->next`) is a slice / `Vec<HsdRObj>` in list order. The C
//! union `u` is [`RObjU`] with one field per member; which one is read is decided by the type
//! bits of `flags` exactly as in the C. Not modelled: reading one union member after writing
//! another (only possible when a caller ORs new type bits into a loaded RObj with
//! `HSD_RObjSetFlags`; the game never does).
//! The `HSD_Rvalue` chain is a `Vec<HsdRvalue>`; `HSD_RvalueList` arrays are slices without the
//! NULL-joint terminator. JObj targets are [`JObjId`]s of the caller's [`JObjTree`].
//! The update callback (`HSD_ObjUpdateFunc`, JObjUpdateFunc in the game) is [`RObjUpdate`]: it
//! gets the tree, the object, the type and the value (`None` for the C's NULL).
//!
//! Calls into code that is not ported go through [`RObjHost`]: `HSD_JObjRefThis` /
//! `HSD_JObjUnrefThis` (TODO(group1 class)) and `HSD_ByteCodeEval` (TODO(group1 bytecode)).
//! `expEvaluate`'s argument buffer (`arg_buf`, `arg_buf_size`) is [`RObjGlobals`]; the native
//! buffer starts zeroed (the C's HSD_MemAlloc memory is uninitialised).
//!
//! FMA: `fma_sites.py` finds no fused site in robj.c's own code; the fused sites on its paths are
//! math_ppc's `sqrtf` ([`crate::gekko::sqrtf`]), `HSD_MtxColMag` ([`mtx_col_mag`]) and the
//! PSVEC / HSD_Mtx helpers (ported in rounds 1-2).
//! GameCube records: `melee_types::HSD_RObj` (0x1C), `HSD_RObjDesc` (0x0C), `HSD_Rvalue` (0x0C).

use crate::aobj::{AObjGlobals, HsdAObj, HsdAObjDesc};
use crate::gekko::sqrtf;
use crate::hsd_mtx::{self, fabsf_bitwise};
use crate::hsd_util::get_nb_bits;
use crate::jobj::{JObjHost, JObjId, JObjTree, JOBJ_CLASSICAL_SCALE};
use crate::mtx::{self, Mtx, Vec3};
use melee_core::fp::fmadds;

pub const ROBJ_TYPE_MASK: u32 = 0x7000_0000;
pub const REFTYPE_EXP: u32 = 0x0000_0000;
pub const REFTYPE_JOBJ: u32 = 0x1000_0000;
pub const REFTYPE_LIMIT: u32 = 0x2000_0000;
pub const REFTYPE_BYTECODE: u32 = 0x3000_0000;
pub const REFTYPE_IKHINT: u32 = 0x4000_0000;
/// fobj.h `TYPE_ROBJ`.
pub const TYPE_ROBJ: u8 = 1;

const EPS: f32 = 1.00000001335e-10f32;
/// `HSD_RObj_80406E74`.
const ORIENT_TYPES: [u32; 3] = [0x32, 0x33, 0x34];

/// `HSD_ExpDesc::func` / `HSD_Exp::expr.func`: gets the whole argument buffer.
pub type ExpFunc = fn(&[f32]) -> f32;

/// `HSD_Exp::expr`.
#[derive(Clone, Debug)]
pub enum ExpExpr {
    /// A zeroed `HSD_Exp` (memset with a NULL desc): calling it would crash in the C.
    Null,
    Func(ExpFunc),
    /// `expr.bytecode` (`None` = NULL).
    ByteCode(Option<Vec<u8>>),
}

/// Native `HSD_Rvalue` (GameCube record `melee_types::HSD_Rvalue`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdRvalue {
    pub flags: u32,
    pub jobj: Option<JObjId>,
}

/// One `HSD_RvalueList` entry (the array ends at the first NULL joint; that entry is not kept).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HsdRvalueList {
    pub flags: u32,
    /// `joint`: the ID-table key of the target joint (never 0).
    pub joint: u32,
}

/// Native `HSD_Exp`.
#[derive(Clone, Debug)]
pub struct HsdExp {
    pub expr: ExpExpr,
    pub rvalue: Vec<HsdRvalue>,
    /// `u32::MAX` = the C's -1 ("not counted yet").
    pub nb_args: u32,
    pub is_bytecode: u8,
}

impl Default for HsdExp {
    fn default() -> Self {
        HsdExp { expr: ExpExpr::Null, rvalue: Vec::new(), nb_args: 0, is_bytecode: 0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HsdIKHint {
    pub bone_length: f32,
    pub rotate_x: f32,
}

/// The C union `HSD_RObj::u`.
#[derive(Clone, Debug, Default)]
pub struct RObjU {
    pub jobj: Option<JObjId>,
    pub exp: HsdExp,
    pub limit: f32,
    pub ik_hint: HsdIKHint,
}

/// Native `HSD_RObj` (GameCube record `melee_types::HSD_RObj`, 0x1C).
#[derive(Clone, Debug, Default)]
pub struct HsdRObj {
    pub flags: u32,
    pub u: RObjU,
    pub aobj: Option<HsdAObj<JObjId>>,
}

impl melee_core::gc::GcLayout for HsdRObj {
    const GC_NAME: &'static str = <melee_types::HSD_RObj as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_RObj as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_RObj as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_RObj as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// `HSD_RObjDesc::u`, by the type bits of `flags`.
#[derive(Clone, Debug)]
pub enum RObjDescU {
    /// REFTYPE_EXP: `HSD_ExpDesc*` (None = NULL).
    Exp(Option<HsdExpDesc>),
    /// REFTYPE_BYTECODE: `HSD_ByteCodeExpDesc*`.
    ByteCode(Option<HsdByteCodeExpDesc>),
    /// REFTYPE_JOBJ: the joint's ID-table key.
    Joint(u32),
    Limit(f32),
    IkHint(HsdIKHint),
}

#[derive(Clone, Debug)]
pub struct HsdExpDesc {
    pub func: Option<ExpFunc>,
    /// `rvalue` (None = NULL list).
    pub rvalue: Option<Vec<HsdRvalueList>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HsdByteCodeExpDesc {
    pub bytecode: Option<Vec<u8>>,
    pub rvalue: Option<Vec<HsdRvalueList>>,
}

/// Native `HSD_RObjDesc` (GameCube record `melee_types::HSD_RObjDesc`, 0x0C).
#[derive(Clone, Debug)]
pub struct HsdRObjDesc {
    pub flags: u32,
    pub u: RObjDescU,
}

impl melee_core::gc::GcLayout for HsdRObjDesc {
    const GC_NAME: &'static str = <melee_types::HSD_RObjDesc as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_RObjDesc as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_RObjDesc as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_RObjDesc as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// One `HSD_RObjAnimJoint` (chain = slice).
#[derive(Clone, Debug)]
pub struct HsdRObjAnimJoint {
    pub aobjdesc: Option<HsdAObjDesc>,
}

/// robj.c's file statics `arg_buf` / `arg_buf_size`.
#[derive(Clone, Debug, Default)]
pub struct RObjGlobals {
    pub arg_buf: Option<Vec<f32>>,
    pub arg_buf_size: u32,
}

impl RObjGlobals {
    /// `_HSD_RObjForgetMemory` for a range that contains `arg_buf`.
    pub fn forget_memory(&mut self) {
        self.arg_buf = None;
        self.arg_buf_size = 0;
    }
}

/// The value an update callback gets (`HSD_ObjData*`, `None` for NULL).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RObjVal {
    Vec(Vec3),
    F(f32),
}

/// `HSD_ObjUpdateFunc` as robj.c calls it: (tree, obj, type, val).
pub type RObjUpdate<'a, D, R> = dyn FnMut(&mut JObjTree<D, R>, JObjId, u32, Option<RObjVal>) + 'a;

/// The not-yet-ported code robj.c calls.
pub trait RObjHost {
    /// `HSD_JObjRefThis`. TODO(group1 class).
    fn jobj_ref_this(&mut self, j: JObjId);
    /// `HSD_JObjUnrefThis` (NULL-safe in the C). TODO(group1 class).
    fn jobj_unref_this(&mut self, j: Option<JObjId>);
    /// `HSD_ByteCodeEval(bytecode, args, nb_args)`. TODO(group1 bytecode).
    fn byte_code_eval(&mut self, bytecode: Option<&[u8]>, args: &[f32], nb_args: u32) -> f32;
}

/// `HSD_MtxColMag` (mtx.h inline): `sqrtf(m0*m0 + m1*m1 + m2*m2)`, contracted as clang does.
pub fn mtx_col_mag(m: &Mtx, col: usize) -> f32 {
    let t = fmadds(m[0][col], m[0][col], m[1][col] * m[1][col]);
    sqrtf(fmadds(m[2][col], m[2][col], t))
}

/// `HSD_MtxColVec`.
#[inline]
pub fn mtx_col_vec(m: &Mtx, col: usize) -> Vec3 {
    Vec3 { x: m[0][col], y: m[1][col], z: m[2][col] }
}

/// `HSD_RObjSetFlags`.
pub fn set_flags(robj: Option<&mut HsdRObj>, flags: u32) {
    if let Some(r) = robj {
        r.flags |= flags;
    }
}

/// `HSD_RObjGetByType` (and robj.c's `inlined_HSD_RObjGetByType`): index of the first RObj in
/// `chain` with bit 31 set, the given type and (when `subtype != 0`) that subtype.
pub fn get_by_type(chain: &[HsdRObj], ty: u32, subtype: u32) -> Option<usize> {
    chain.iter().position(|c| {
        c.flags & 0x8000_0000 != 0
            && c.flags & ROBJ_TYPE_MASK == ty
            && (subtype == 0 || subtype == c.flags & 0x0FFF_FFFF)
    })
}

/// `RObjUpdateFunc` (static): the AObj track sets or clears bit 31.
pub fn robj_update_func(robj: Option<&mut u32>, ty: u8, fv: f32) {
    let Some(flags) = robj else { return };
    if ty != TYPE_ROBJ {
        return;
    }
    if fv as f64 >= 0.5 {
        *flags |= 0x8000_0000;
    } else {
        *flags &= 0x7FFF_FFFF;
    }
}

/// `HSD_RObjAnim`.
pub fn anim(robj: Option<&mut HsdRObj>, g: &mut AObjGlobals) {
    let Some(r) = robj else { return };
    if let Some(a) = r.aobj.as_mut() {
        let flags = &mut r.flags;
        let mut f = |t: u8, fv: f32| robj_update_func(Some(&mut *flags), t, fv);
        a.interpret_anim(g, &mut Some(&mut f));
    }
}

/// `HSD_RObjAnimAll`.
pub fn anim_all(chain: &mut [HsdRObj], g: &mut AObjGlobals) {
    for r in chain.iter_mut() {
        anim(Some(r), g);
    }
}

/// `HSD_RObjRemoveAnimByFlags`.
pub fn remove_anim_by_flags(robj: Option<&mut HsdRObj>, flags: u32) {
    let Some(r) = robj else { return };
    if r.aobj.is_some() && flags & 0x80 != 0 {
        r.aobj = None; // HSD_AObjRemove
    }
}

/// `HSD_RObjRemoveAnimAllByFlags`.
pub fn remove_anim_all_by_flags(chain: &mut [HsdRObj], flags: u32) {
    for r in chain.iter_mut() {
        remove_anim_by_flags(Some(r), flags);
    }
}

/// `HSD_RObjRemoveAnimAll`.
pub fn remove_anim_all(chain: &mut [HsdRObj]) {
    remove_anim_all_by_flags(chain, 0x7FF);
}

/// `HSD_RObjReqAnimByFlags`.
pub fn req_anim_by_flags(robj: Option<&mut HsdRObj>, startframe: f32, flags: u32) {
    let Some(r) = robj else { return };
    if let Some(a) = r.aobj.as_mut() {
        if flags & 0x80 != 0 {
            a.req_anim(startframe);
        }
    }
}

/// `HSD_RObjReqAnimAllByFlags`.
pub fn req_anim_all_by_flags(chain: &mut [HsdRObj], startframe: f32, flags: u32) {
    for r in chain.iter_mut() {
        req_anim_by_flags(Some(r), startframe, flags);
    }
}

/// `HSD_RObjReqAnimAll`.
pub fn req_anim_all(chain: &mut [HsdRObj], startframe: f32) {
    req_anim_all_by_flags(chain, startframe, 0x7FF);
}

/// `HSD_RObjAddAnim`. `resolve_obj` is AObjLoadDesc's obj_id lookup (TODO(group1 jobj load)).
pub fn add_anim(
    robj: Option<&mut HsdRObj>,
    anim: Option<&HsdRObjAnimJoint>,
    resolve_obj: &mut dyn FnMut(u32) -> Option<JObjId>,
) {
    let (Some(r), Some(an)) = (robj, anim) else { return };
    r.aobj = None; // HSD_AObjRemove of the old one
    // HSD_AObjLoadDesc(NULL) returns NULL.
    r.aobj = an.aobjdesc.as_ref().map(|d| HsdAObj::load_desc(d, |id| resolve_obj(id)));
}

/// `HSD_RObjAddAnimAll`: pairs the chains up to the shorter one.
pub fn add_anim_all(
    chain: &mut [HsdRObj],
    anim: &[HsdRObjAnimJoint],
    resolve_obj: &mut dyn FnMut(u32) -> Option<JObjId>,
) {
    for (r, a) in chain.iter_mut().zip(anim.iter()) {
        add_anim(Some(r), Some(a), resolve_obj);
    }
}

/// `HSD_RObjGetConstraintType` (static).
pub fn get_constraint_type(robj: Option<&HsdRObj>) -> u32 {
    robj.map_or(0, |r| r.flags & 0x0FFF_FFFF)
}

/// `HSD_RObjGetGlobalPosition`: the mean translation of the active JObj constraints of `ty`.
/// Returns the count and, when it is not 0, the position (the C leaves `*p` untouched at 0).
pub fn get_global_position<D, R, H>(
    chain: &[HsdRObj],
    ty: i32,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
) -> (i32, Option<Vec3>)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut n: i32 = 0;
    for rp in chain {
        if rp.flags & ROBJ_TYPE_MASK == REFTYPE_JOBJ
            && rp.flags & 0x8000_0000 != 0
            && ty as u32 == get_constraint_type(Some(rp))
        {
            let j = rp.u.jobj.expect("HSD_ASSERT(498, rp->u.jobj)");
            tree.setup_matrix(host, Some(j));
            n = n.wrapping_add(1);
            let m = &tree.get(j).mtx;
            v.x += m[0][3];
            v.y += m[1][3];
            v.z += m[2][3];
        }
    }
    if n != 0 {
        let f = 1.0f32 / n as f32;
        (n, Some(Vec3 { x: f * v.x, y: f * v.y, z: f * v.z }))
    } else {
        (n, None)
    }
}

/// `set_dirup_matrix` (static). `dir` / `uv` are written back as in the C.
pub fn set_dirup_matrix<D, R>(
    dir: &mut Vec3,
    uv: &mut Vec3,
    scale: &Vec3,
    tree: &mut JObjTree<D, R>,
    obj: JObjId,
    update: &mut RObjUpdate<'_, D, R>,
) {
    let mut z = mtx::vec_cross(dir, uv);
    let kdir = sqrtf(1.0f32 / (EPS + mtx::vec_dot(dir, dir)));
    *dir = mtx::vec_scale(dir, kdir);
    let kz = sqrtf(1.0f32 / (EPS + mtx::vec_dot(&z, &z)));
    z = mtx::vec_scale(&z, kz);
    *uv = mtx::vec_cross(&z, dir);
    let v = Vec3 { x: dir.x * scale.x, y: dir.y * scale.x, z: dir.z * scale.x };
    update(tree, obj, 50, Some(RObjVal::Vec(v)));
    let v = Vec3 { x: uv.x * scale.y, y: uv.y * scale.y, z: uv.z * scale.y };
    update(tree, obj, 51, Some(RObjVal::Vec(v)));
    let v = Vec3 { x: z.x * scale.z, y: z.y * scale.z, z: z.z * scale.z };
    update(tree, obj, 52, Some(RObjVal::Vec(v)));
    update(tree, obj, 55, None);
}

/// `resolveCnsDirUp` (static).
pub fn resolve_cns_dir_up<D, R, H>(
    chain: &[HsdRObj],
    tree: &mut JObjTree<D, R>,
    host: &mut H,
    obj: JObjId,
    update: &mut RObjUpdate<'_, D, R>,
) where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let mut this_scale = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    let mut up = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    let (n, p) = get_global_position(chain, 2, tree, host);
    if n == 0 {
        return;
    }
    let mut this_pos = p.unwrap();
    let m = &tree.get(obj).mtx;
    let dir = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
    this_pos = mtx::vec_sub(&this_pos, &dir);
    let (n3, u) = get_global_position(chain, 3, tree, host);
    if n3 != 0 {
        up = mtx::vec_sub(&u.unwrap(), &dir);
    } else {
        let k = 1.0f32 - mtx::vec_dot(&this_pos, &up);
        if fabsf_bitwise(k) < EPS {
            up = Vec3 { x: 0.0, y: 0.0, z: 1.0 };
        }
    }
    if let Some(s) = tree.get(obj).scl {
        this_scale = s;
    }
    set_dirup_matrix(&mut this_pos, &mut up, &this_scale, tree, obj, update);
}

/// `HSD_JObjGetMtxPtr` (jobj.h inline): set up the matrix, then read it.
fn mtx_ptr<D, R, H>(tree: &mut JObjTree<D, R>, host: &mut H, j: JObjId) -> Mtx
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    tree.setup_matrix(host, Some(j));
    tree.get(j).mtx
}

/// `resolveCnsOrientation` (static).
pub fn resolve_cns_orientation<D, R, H>(
    chain: &[HsdRObj],
    tree: &mut JObjTree<D, R>,
    host: &mut H,
    obj: JObjId,
    update: &mut RObjUpdate<'_, D, R>,
) where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let Some(ri) = get_by_type(chain, 0x1000_0000, 4) else { return };
    let target = chain[ri].u.jobj;
    let tgt = target.expect("HSD_ASSERT(1144, jobj)");
    let parent = tree.parent_of(target);
    if tree.get_flags(target) & JOBJ_CLASSICAL_SCALE == 0 || parent.is_none() {
        let mtx0 = mtx_ptr(tree, host, tgt);
        for i in 0..3 {
            let mut v = mtx_col_vec(&mtx0, i);
            let mut sval = mtx::vec_mag(&v);
            if sval > 1e-10f32 {
                sval = 1.0f32 / sval;
            }
            sval *= mtx_col_mag(&tree.get(obj).mtx, i);
            v.x *= sval;
            v.y *= sval;
            v.z *= sval;
            update(tree, obj, ORIENT_TYPES[i], Some(RObjVal::Vec(v)));
        }
    } else {
        // Arguments are evaluated left to right; the pointers are read after both setups.
        let p = parent.unwrap();
        tree.setup_matrix(host, Some(p));
        tree.setup_matrix(host, Some(tgt));
        let mut mtx1 = hsd_mtx::inverse_concat(&tree.get(p).mtx, &tree.get(tgt).mtx);
        for i in 0..3 {
            let mut v = mtx_col_vec(&mtx1, i);
            let mut sval = mtx::vec_mag(&v);
            if sval > 1e-10f32 {
                sval = 1.0f32 / sval;
            }
            sval *= mtx_col_mag(&tree.get(obj).mtx, i);
            v.x *= sval;
            v.y *= sval;
            v.z *= sval;
            mtx1[0][i] = v.x;
            mtx1[1][i] = v.y;
            mtx1[2][i] = v.z;
        }
        let mut j = Some(p);
        while let Some(jj) = j {
            let mut mtx0 = if let Some(pp) = tree.get(jj).parent {
                tree.setup_matrix(host, Some(pp));
                tree.setup_matrix(host, Some(jj));
                hsd_mtx::inverse_concat(&tree.get(pp).mtx, &tree.get(jj).mtx)
            } else {
                mtx_ptr(tree, host, jj)
            };
            for i in 0..3 {
                let mut v = mtx_col_vec(&mtx0, i);
                let mut sval = mtx::vec_mag(&v);
                if sval > 1e-10f32 {
                    sval = 1.0f32 / sval;
                }
                v.x *= sval;
                v.y *= sval;
                v.z *= sval;
                mtx0[0][i] = v.x;
                mtx0[1][i] = v.y;
                mtx0[2][i] = v.z;
            }
            mtx1 = mtx::concat(&mtx0, &mtx1);
            j = tree.get(jj).parent;
        }
        for i in 0..3 {
            let v = mtx_col_vec(&mtx1, i);
            update(tree, obj, ORIENT_TYPES[i], Some(RObjVal::Vec(v)));
        }
    }
    update(tree, obj, 0x37, None);
}

/// `resolveLimits` (static): clamp the JObj's rotation / translation to the LIMIT RObjs and
/// rebuild its matrix when one applied.
pub fn resolve_limits<D, R, H>(chain: &[HsdRObj], tree: &mut JObjTree<D, R>, host: &mut H, obj: JObjId)
where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    let is_limit = |r: &HsdRObj| r.flags & ROBJ_TYPE_MASK == REFTYPE_LIMIT;
    if !chain.iter().any(is_limit) {
        return;
    }
    let mut update_mtx = false;
    for rp in chain.iter().filter(|r| is_limit(r)) {
        let l = rp.u.limit;
        let jo = tree.get_mut(obj);
        // (field, is_lower_bound); cases 11/12 clamp translate.y again, as in the C.
        let (f, lower): (&mut f32, bool) = match rp.flags & 0x0FFF_FFFF {
            1 => (&mut jo.rotate.x, true),
            2 => (&mut jo.rotate.x, false),
            3 => (&mut jo.rotate.y, true),
            4 => (&mut jo.rotate.y, false),
            5 => (&mut jo.rotate.z, true),
            6 => (&mut jo.rotate.z, false),
            7 => (&mut jo.translate.x, true),
            8 => (&mut jo.translate.x, false),
            9 | 11 => (&mut jo.translate.y, true),
            10 | 12 => (&mut jo.translate.y, false),
            _ => continue,
        };
        if lower {
            if *f < l {
                *f = l;
            }
        } else if *f > l {
            *f = l;
        }
        update_mtx = true;
    }
    if update_mtx {
        tree.make_matrix(host, obj);
    }
}

/// `expEvaluate` (static): gather the rvalue arguments, evaluate, send the result as `ty`.
#[allow(clippy::too_many_arguments)]
pub fn exp_evaluate<D, R, H, X>(
    exp: &mut HsdExp,
    ty: u32,
    g: &mut RObjGlobals,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
    rhost: &mut X,
    obj: JObjId,
    update: &mut RObjUpdate<'_, D, R>,
) where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
    X: RObjHost,
{
    if exp.nb_args == u32::MAX {
        let mut nb: u32 = 0;
        for rv in &exp.rvalue {
            nb = nb.wrapping_add(get_nb_bits(rv.flags));
        }
        exp.nb_args = nb;
    }
    if g.arg_buf.is_none() {
        if g.arg_buf_size == 0 {
            g.arg_buf_size = 100;
        }
        g.arg_buf = Some(vec![0.0; g.arg_buf_size as usize]);
    }
    if g.arg_buf_size < exp.nb_args {
        panic!(
            "Number of argment of expression exceeds the argument buffer\nsize. (requested num of arg {}, allocated {})",
            exp.nb_args, g.arg_buf_size
        );
    }
    let k = 57.29578f32;
    let buf = g.arg_buf.as_mut().unwrap();
    let mut cur = 0usize;
    for rv in &exp.rvalue {
        let j = rv.jobj.expect("HSD_ASSERT(0x467, jobj)");
        tree.setup_matrix(host, Some(j));
        let jo = tree.get(j);
        let mut bit: u32 = 1;
        while bit != 0 && bit <= rv.flags {
            let val = match rv.flags & bit {
                0x1 => Some(k * jo.rotate.x),
                0x2 => Some(k * jo.rotate.y),
                0x4 => Some(k * jo.rotate.z),
                0x10 => Some(jo.translate.x),
                0x20 => Some(jo.translate.y),
                0x40 => Some(jo.translate.z),
                0x80 => Some(jo.scale.x),
                0x100 => Some(jo.scale.y),
                0x200 => Some(jo.scale.z),
                0x10000 => Some(k * hsd_mtx::get_rotation(&jo.mtx).x),
                0x20000 => Some(k * hsd_mtx::get_rotation(&jo.mtx).y),
                0x40000 => Some(k * hsd_mtx::get_rotation(&jo.mtx).z),
                0x100000 => Some(jo.mtx[0][3]),
                0x200000 => Some(jo.mtx[1][3]),
                0x400000 => Some(jo.mtx[2][3]),
                0x800000 => Some(hsd_mtx::get_scale(&jo.mtx).x),
                0x1000000 => Some(hsd_mtx::get_scale(&jo.mtx).y),
                0x2000000 => Some(hsd_mtx::get_scale(&jo.mtx).z),
                _ => None,
            };
            if let Some(v) = val {
                buf[cur] = v;
                cur += 1;
            }
            bit <<= 1;
        }
    }
    let mut fv = if exp.is_bytecode != 0 {
        let code = match &exp.expr {
            ExpExpr::ByteCode(b) => b.as_deref(),
            _ => None,
        };
        rhost.byte_code_eval(code, buf, exp.nb_args)
    } else {
        match &exp.expr {
            ExpExpr::Func(f) => f(buf),
            _ => panic!("expEvaluate: NULL expression function"),
        }
    };
    if ty.wrapping_sub(1) <= 2 {
        fv *= 0.017453292f32;
    }
    update(tree, obj, ty, Some(RObjVal::F(fv)));
}

/// `HSD_RObjUpdateAll`: position, direction/up, orientation and limit constraints, then every
/// active expression.
#[allow(clippy::too_many_arguments)]
pub fn update_all<D, R, H, X>(
    chain: &mut [HsdRObj],
    g: &mut RObjGlobals,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
    rhost: &mut X,
    obj: JObjId,
    update: &mut RObjUpdate<'_, D, R>,
) where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
    X: RObjHost,
{
    if chain.is_empty() {
        return;
    }
    let (n, p) = get_global_position(chain, 1, tree, host);
    if n != 0 {
        update(tree, obj, 0x35, Some(RObjVal::Vec(p.unwrap())));
        update(tree, obj, 0x38, None);
    }
    resolve_cns_dir_up(chain, tree, host, obj, update);
    resolve_cns_orientation(chain, tree, host, obj, update);
    resolve_limits(chain, tree, host, obj);
    for rp in chain.iter_mut() {
        if rp.flags & ROBJ_TYPE_MASK == 0 && rp.flags & 0x8000_0000 != 0 {
            let ty = rp.flags & 0x0FFF_FFFF;
            exp_evaluate(&mut rp.u.exp, ty, g, tree, host, rhost, obj, update);
        }
    }
}

/// `HSD_RvalueAlloc`.
pub fn rvalue_alloc() -> HsdRvalue {
    HsdRvalue::default()
}

/// `HSD_RvalueRemove`.
pub fn rvalue_remove<X: RObjHost>(rv: HsdRvalue, rhost: &mut X) {
    rhost.jobj_unref_this(rv.jobj);
}

/// `HSD_RvalueRemoveAll`.
pub fn rvalue_remove_all<X: RObjHost>(rvs: Vec<HsdRvalue>, rhost: &mut X) {
    for rv in rvs {
        rvalue_remove(rv, rhost);
    }
}

/// `loadRvalue` (static): one zero-jobj rvalue per list entry (None list -> empty).
pub fn load_rvalue(list: Option<&[HsdRvalueList]>) -> Vec<HsdRvalue> {
    list.map_or_else(Vec::new, |l| {
        l.iter().map(|e| HsdRvalue { flags: e.flags, jobj: None }).collect()
    })
}

/// `dummy_func` (static).
pub fn dummy_func(_args: &[f32]) -> f32 {
    0.0
}

/// `expLoadDesc` (static).
pub fn exp_load_desc(desc: Option<&HsdExpDesc>) -> HsdExp {
    let mut e = HsdExp::default();
    if let Some(d) = desc {
        e.expr = ExpExpr::Func(d.func.unwrap_or(dummy_func));
        e.rvalue = load_rvalue(d.rvalue.as_deref());
        e.nb_args = u32::MAX;
    }
    e
}

/// `bcexpLoadDesc` (static).
pub fn bcexp_load_desc(desc: Option<&HsdByteCodeExpDesc>) -> HsdExp {
    let mut e = HsdExp::default();
    if let Some(d) = desc {
        e.expr = ExpExpr::ByteCode(d.bytecode.clone());
        e.rvalue = load_rvalue(d.rvalue.as_deref());
        e.nb_args = u32::MAX;
        e.is_bytecode = 1;
    }
    e
}

/// `HSD_RObjLoadDesc` for one desc (the C recurses down `next`; see [`load_desc_list`]).
pub fn load_desc(desc: &HsdRObjDesc) -> HsdRObj {
    let mut r = HsdRObj { flags: desc.flags, ..Default::default() };
    match (r.flags & ROBJ_TYPE_MASK, &desc.u) {
        (REFTYPE_JOBJ, _) => {}
        (REFTYPE_LIMIT, RObjDescU::Limit(l)) => {
            r.u.limit = match r.flags & 0x0FFF_FFFF {
                1..=6 => 0.017453292f32 * *l,
                _ => *l,
            };
        }
        (REFTYPE_EXP, RObjDescU::Exp(e)) => r.u.exp = exp_load_desc(e.as_ref()),
        (REFTYPE_BYTECODE, RObjDescU::ByteCode(b)) => {
            r.u.exp = bcexp_load_desc(b.as_ref());
            r.flags &= !ROBJ_TYPE_MASK;
        }
        (REFTYPE_IKHINT, RObjDescU::IkHint(h)) => r.u.ik_hint = *h,
        (REFTYPE_LIMIT | REFTYPE_EXP | REFTYPE_BYTECODE | REFTYPE_IKHINT, _) => {
            panic!("robj desc union does not match its type bits")
        }
        _ => panic!("unexpected type of robj.\n"),
    }
    r
}

/// `HSD_RObjLoadDesc` over a desc chain.
pub fn load_desc_list(descs: &[HsdRObjDesc]) -> Vec<HsdRObj> {
    descs.iter().map(load_desc).collect()
}

/// `HSD_RObjRemove`.
pub fn remove<X: RObjHost>(r: HsdRObj, rhost: &mut X) {
    match r.flags & ROBJ_TYPE_MASK {
        REFTYPE_JOBJ => rhost.jobj_unref_this(r.u.jobj),
        0 => rvalue_remove_all(r.u.exp.rvalue, rhost),
        _ => {}
    }
    // HSD_AObjRemove + HSD_RObjFree: dropped here.
}

/// `HSD_RObjRemoveAll`.
pub fn remove_all<X: RObjHost>(chain: Vec<HsdRObj>, rhost: &mut X) {
    for r in chain {
        remove(r, rhost);
    }
}

/// `HSD_RObjAlloc`.
pub fn alloc() -> HsdRObj {
    HsdRObj::default()
}

/// `HSD_RvalueResolveRefs`. `lookup` is `HSD_IDGetData(key, NULL)`.
pub fn rvalue_resolve_refs<X: RObjHost>(
    rv: Option<&mut HsdRvalue>,
    list: Option<&HsdRvalueList>,
    lookup: &mut dyn FnMut(u32) -> Option<JObjId>,
    rhost: &mut X,
) {
    let (Some(rv), Some(l)) = (rv, list) else { return };
    rhost.jobj_unref_this(rv.jobj);
    rv.jobj = lookup(l.joint);
    let j = rv.jobj.expect("HSD_ASSERT(1333, rvalue->jobj)");
    rhost.jobj_ref_this(j);
}

/// `HSD_RvalueResolveRefsAll`.
pub fn rvalue_resolve_refs_all<X: RObjHost>(
    rvs: &mut [HsdRvalue],
    list: Option<&[HsdRvalueList]>,
    lookup: &mut dyn FnMut(u32) -> Option<JObjId>,
    rhost: &mut X,
) {
    let Some(list) = list else { return };
    for (rv, l) in rvs.iter_mut().zip(list.iter()) {
        rvalue_resolve_refs(Some(rv), Some(l), lookup, rhost);
    }
}

/// `HSD_RObjResolveRefs`.
pub fn resolve_refs<X: RObjHost>(
    r: Option<&mut HsdRObj>,
    desc: Option<&HsdRObjDesc>,
    lookup: &mut dyn FnMut(u32) -> Option<JObjId>,
    rhost: &mut X,
) {
    let (Some(r), Some(d)) = (r, desc) else { return };
    match r.flags & ROBJ_TYPE_MASK {
        REFTYPE_JOBJ => {
            let RObjDescU::Joint(key) = d.u else { panic!("robj desc is not a joint") };
            rhost.jobj_unref_this(r.u.jobj);
            r.u.jobj = lookup(key);
            let j = r.u.jobj.expect("HSD_ASSERT(883, robj->u.jobj)");
            rhost.jobj_ref_this(j);
        }
        0 => {
            // desc->u.exp->rvalue (an HSD_ByteCodeExpDesc has `rvalue` at the same offset).
            let list = match &d.u {
                RObjDescU::Exp(Some(e)) => e.rvalue.as_deref(),
                RObjDescU::ByteCode(Some(b)) => b.rvalue.as_deref(),
                _ => panic!("NULL expression desc"),
            };
            rvalue_resolve_refs_all(&mut r.u.exp.rvalue, list, lookup, rhost);
        }
        _ => {}
    }
}

/// `HSD_RObjResolveRefsAll`: pairs the chains up to the shorter one.
pub fn resolve_refs_all<X: RObjHost>(
    chain: &mut [HsdRObj],
    descs: &[HsdRObjDesc],
    lookup: &mut dyn FnMut(u32) -> Option<JObjId>,
    rhost: &mut X,
) {
    for (r, d) in chain.iter_mut().zip(descs.iter()) {
        resolve_refs(Some(r), Some(d), lookup, rhost);
    }
}

/// `HSD_RObjSetConstraintObj`. The target is always a JObj natively (the C's
/// `hsdObjIsDescendantOf(.., hsdJObj)` check is the type).
pub fn set_constraint_obj<X: RObjHost>(r: Option<&mut HsdRObj>, o: JObjId, rhost: &mut X) {
    let Some(r) = r else { return };
    if r.u.jobj.is_some() {
        rhost.jobj_unref_this(r.u.jobj);
        r.u.jobj = None;
    }
    r.u.jobj = Some(o);
    rhost.jobj_ref_this(o);
}
