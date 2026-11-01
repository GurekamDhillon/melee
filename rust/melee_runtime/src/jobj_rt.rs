
//! Port round 8 of `src/sysdolphin/baselib/jobj.c`: the animation half (`JObjUpdateFunc`, the
//! ReqAnim / RemoveAnim / AddAnim / Anim walkers, `JObjSortAnim`), the IK solvers
//! (`resolveIKJoint1`, `resolveIKJoint2`, `jobj_get_effector_checked` and its static inlines),
//! `HSD_JObjSetCurrent` / `GetCurrent` and `HSD_JObjSetDPtclCallback`.
//!
//! Native shape: the round 5 [`JObjTree`] with `R = u32`, an index into the RObj arena
//! [`JObjRt::robjs`] (each JObj's `robj` Vec is its `robj->next` chain in list order). [`JObjRt`]
//! is the `JObjHost` that wires round 7's RObj port into the matrix build: the effector bone
//! length, `HSD_RObjUpdateAll(jobj->robj, jobj, JObjUpdateFunc)` and the IK solvers. The
//! jobj.c file statics are [`JObjGlobals`].
//!
//! `HSD_ObjData` is [`ObjData`] (three words; `fv` / `iv` / `p` read them as the C union does).
//!
//! Calls into code that is not ported go through [`JObjExt`]: the DObj anim walkers
//! (TODO(group1 dobj)), `splArcLengthPoint` on the AObj target's spline (the native JObj keeps
//! only the DObj member of its union; TODO(group1 jobj spline)), `HSD_JObjUnref` /
//! `HSD_JObjRef` (TODO(group1 class)), AObjLoadDesc's obj_id lookup (TODO(group1 jobj load)) and
//! robj.c's own host calls (`RObjHost`).
//!
//! One ordering difference, inside `HSD_RObjUpdateAll` only: while the RObj chain is being
//! solved, a JObjUpdateFunc write to the IK hint's `rotate_x` (a ROTX expression on a JOINT1)
//! and the SETBYTE / SETFLOAT / 0x28-0x2A callbacks are applied right after the update instead
//! of during it. Nothing in `HSD_RObjUpdateAll` reads the IK hint, so the first is not
//! observable; the second needs an expression RObj of type >= 20 (none in the game's data that
//! the port has seen) and a callback that reads JObj state.
//!
//! FMA: `fma_sites.py` finds one fused site, in resolveIKJoint1
//! (`2*(a+b) - d` -> `fmuladd(2, a+b, -d)`), ported with `fmadds`. The rest is the PSVEC /
//! PSMTX / HSD_Mtx ports and math_ppc `sqrtf` ([`crate::gekko::sqrtf`]); `acosf` is lbtrigf's.

use crate::aobj::{AObjGlobals, HsdAObj, HsdAObjDesc};

use crate::fobj::HsdFObj;

use crate::gekko::sqrtf;

use crate::hsd_mtx::{self, fabsf_bitwise};

use crate::jobj::*;

use crate::lbtrigf::acosf;

use crate::mtx::{self, Mtx, Vec3};

use crate::robj::{self, HsdRObj, HsdRObjAnimJoint, RObjGlobals, RObjHost, RObjVal, REFTYPE_IKHINT, REFTYPE_JOBJ};

use melee_core::fp::fmadds;

use std::cell::RefCell;

use std::rc::Rc;

pub const HSD_A_J_ROTX: u32 = 1;

pub const HSD_A_J_ROTY: u32 = 2;

pub const HSD_A_J_ROTZ: u32 = 3;

pub const HSD_A_J_PATH: u32 = 4;

pub const HSD_A_J_TRAX: u32 = 5;

pub const HSD_A_J_TRAY: u32 = 6;

pub const HSD_A_J_TRAZ: u32 = 7;

pub const HSD_A_J_SCAX: u32 = 8;

pub const HSD_A_J_SCAY: u32 = 9;

pub const HSD_A_J_SCAZ: u32 = 10;

pub const HSD_A_J_NODE: u32 = 11;

pub const HSD_A_J_BRANCH: u32 = 12;

pub const HSD_A_J_SETBYTE0: u32 = 20;

pub const HSD_A_J_SETFLOAT0: u32 = 30;

/// fobj.h `TYPE_JOBJ`.
pub const TYPE_JOBJ: u8 = 12;

/// `HSD_ObjData`: `fv` / `iv` are word 0, `p` is words 0-2.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjData {
    pub w: [u32; 3],
}

impl ObjData {
    pub fn from_f(f: f32) -> Self {
        ObjData { w: [f.to_bits(), 0, 0] }
    }
    pub fn from_vec(v: Vec3) -> Self {
        ObjData { w: [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()] }
    }
    #[inline]
    pub fn fv(&self) -> f32 {
        f32::from_bits(self.w[0])
    }
    #[inline]
    pub fn set_fv(&mut self, f: f32) {
        self.w[0] = f.to_bits();
    }
    #[inline]
    pub fn iv(&self) -> i32 {
        self.w[0] as i32
    }
    #[inline]
    pub fn p(&self) -> Vec3 {
        Vec3 { x: f32::from_bits(self.w[0]), y: f32::from_bits(self.w[1]), z: f32::from_bits(self.w[2]) }
    }
}

impl From<RObjVal> for ObjData {
    fn from(v: RObjVal) -> Self {
        match v {
            RObjVal::Vec(p) => ObjData::from_vec(p),
            RObjVal::F(f) => ObjData::from_f(f),
        }
    }
}

/// `ufc_callback`: (jobj, type, value).
pub type UfcCallback = Box<dyn FnMut(JObjId, u32, f32)>;

/// `DPCtlCallback`: (0, lo, hi, jobj).
pub type DPtclCallback = Box<dyn FnMut(i32, i32, i32, JObjId)>;

/// jobj.c's file statics.
#[derive(Default)]
pub struct JObjGlobals {
    /// `ufc_callbacks` (SList order).
    pub ufc_callbacks: Vec<UfcCallback>,
    pub dptcl_callback: Option<DPtclCallback>,
    pub jsound_callback: Option<Box<dyn FnMut(i32)>>,
    pub ptcltgt_callback: Option<Box<dyn FnMut(JObjId, i32)>>,
    pub current_jobj: Option<JObjId>,
}

/// A JObjUpdateFunc effect outside the tree (see the module doc for when it is deferred).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UfcEvent {
    /// The IK hint's `rotate_x` of `jobj`'s RObj chain.
    IkRotateX(JObjId, f32),
    /// SETBYTE / SETFLOAT: every `ufc_callbacks` entry gets (jobj, type, value).
    Ufc(JObjId, u32, f32),
    DPtcl(i32, i32, JObjId),
    JSound(i32),
    PtclTgt(JObjId, i32),
}

/// `JObjUpdateFunc` minus its outside effects, which it returns (in call order) for the caller
/// to apply. `spline_point(target, s)` is `splArcLengthPoint(&p, target->u.spline, s)`.
pub fn update_func_core<D: Copy + PartialEq, R: Copy + PartialEq>(
    tree: &mut JObjTree<D, R>,
    jobj: Option<JObjId>,
    ty: u32,
    val: Option<&mut ObjData>,
    spline_point: &mut dyn FnMut(JObjId, f32) -> Vec3,
    events: &mut Vec<UfcEvent>,
) {
    let Some(j) = jobj else { return };
    match ty {
        HSD_A_J_PATH => {
            let v = need(val);
            if (v.fv() as f64) < 0.0 {
                v.set_fv(0.0);
            }
            if 1.0 < v.fv() as f64 {
                v.set_fv(1.0);
            }
            let jp = tree.get(j).aobj.as_ref().expect("HSD_ASSERT(0x24B, jobj->aobj)").hsd_obj;
            let jp = jp.expect("HSD_ASSERT(0x24D, jp)");
            let p = spline_point(jp, v.fv());
            set_translate(tree, j, 0, p.x);
            set_translate(tree, j, 1, p.y);
            set_translate(tree, j, 2, p.z);
        }
        HSD_A_J_ROTX => {
            let f = need(val).fv();
            if tree.get(j).flags & JOBJ_JOINT1 != 0 {
                events.push(UfcEvent::IkRotateX(j, f));
            }
            set_rotation(tree, j, 0, f);
        }
        HSD_A_J_ROTY => set_rotation(tree, j, 1, need(val).fv()),
        HSD_A_J_ROTZ => set_rotation(tree, j, 2, need(val).fv()),
        HSD_A_J_TRAX => set_translate(tree, j, 0, need(val).fv()),
        HSD_A_J_TRAY => set_translate(tree, j, 1, need(val).fv()),
        HSD_A_J_TRAZ => set_translate(tree, j, 2, need(val).fv()),
        HSD_A_J_SCAX | HSD_A_J_SCAY | HSD_A_J_SCAZ => {
            let v = need(val);
            if fabsf_bitwise(v.fv()) < 1e-3 {
                v.set_fv(1e-3);
            }
            let f = v.fv();
            let o = tree.get_mut(j);
            match ty {
                HSD_A_J_SCAX => o.scale.x = f,
                HSD_A_J_SCAY => o.scale.y = f,
                _ => o.scale.z = f,
            }
            if o.flags & JOBJ_MTX_INDEP_SRT == 0 {
                tree.set_mtx_dirty(Some(j));
            }
        }
        HSD_A_J_BRANCH => {
            if need(val).fv() as f64 > 0.5 {
                tree.clear_flags_all(Some(j), JOBJ_HIDDEN);
            } else {
                tree.set_flags_all(Some(j), JOBJ_HIDDEN);
            }
        }
        HSD_A_J_NODE => {
            if need(val).fv() as f64 > 0.5 {
                tree.clear_flags(Some(j), JOBJ_HIDDEN);
            } else {
                tree.set_flags(Some(j), JOBJ_HIDDEN);
            }
        }
        // `cb(jobj, type, val->iv)`: the s32 converts to the callback's f32 parameter.
        20..=29 => events.push(UfcEvent::Ufc(j, ty, need(val).iv() as f32)),
        30..=39 => events.push(UfcEvent::Ufc(j, ty, need(val).fv())),
        0x28 => {
            let iv = need(val).iv();
            events.push(UfcEvent::DPtcl(iv & 0x3F, (iv >> 6) & 0xFF_FFFF, j));
        }
        0x29 => events.push(UfcEvent::JSound(need(val).iv())),
        0x2A => events.push(UfcEvent::PtclTgt(j, need(val).iv())),
        0x32..=0x35 => {
            let p = need(val).p();
            let col = (ty - 0x32) as usize;
            let m = &mut tree.get_mut(j).mtx;
            m[0][col] = p.x;
            m[1][col] = p.y;
            m[2][col] = p.z;
        }
        0x36..=0x39 => {
            let o = tree.get(j);
            let m: Mtx = match o.parent {
                Some(p) => hsd_mtx::inverse_concat(&tree.get(p).mtx, &o.mtx),
                None => o.mtx,
            };
            let o = tree.get_mut(j);
            if ty == 0x36 || ty == 0x38 {
                o.translate = hsd_mtx::get_translate(&m);
            }
            if ty == 0x36 || ty == 0x37 {
                let r = hsd_mtx::get_rotation(&m);
                o.rotate.x = r.x;
                o.rotate.y = r.y;
                o.rotate.z = r.z;
            }
            if ty == 0x36 || ty == 0x39 {
                o.scale = hsd_mtx::get_scale(&m);
            }
        }
        _ => {}
    }
}

pub(crate) fn need(v: Option<&mut ObjData>) -> &mut ObjData {
    v.expect("JObjUpdateFunc: val")
}

/// `HSD_JObjSetRotationX/Y/Z`.
pub(crate) fn set_rotation<D: Copy + PartialEq, R: Copy + PartialEq>(t: &mut JObjTree<D, R>, j: JObjId, axis: u8, v: f32) {
    let o = t.get_mut(j);
    assert!(o.flags & JOBJ_USE_QUATERNION == 0, "HSD_ASSERT(640, !(jobj->flags & JOBJ_USE_QUATERNION))");
    match axis {
        0 => o.rotate.x = v,
        1 => o.rotate.y = v,
        _ => o.rotate.z = v,
    }
    if o.flags & JOBJ_MTX_INDEP_SRT == 0 {
        t.set_mtx_dirty(Some(j));
    }
}

/// `HSD_JObjSetTranslateX/Y/Z`.
pub(crate) fn set_translate<D: Copy + PartialEq, R: Copy + PartialEq>(t: &mut JObjTree<D, R>, j: JObjId, axis: u8, v: f32) {
    let o = t.get_mut(j);
    match axis {
        0 => o.translate.x = v,
        1 => o.translate.y = v,
        _ => o.translate.z = v,
    }
    if o.flags & JOBJ_MTX_INDEP_SRT == 0 {
        t.set_mtx_dirty(Some(j));
    }
}

/// `JObjSortAnim`: moves the first JOBJ-type track to the front.
pub fn sort_anim<O>(aobj: Option<&mut HsdAObj<O>>) {
    let Some(a) = aobj else { return };
    if let Some(i) = a.fobj.iter().position(|f: &HsdFObj| f.obj_type == TYPE_JOBJ) {
        let f = a.fobj.remove(i);
        a.fobj.insert(0, f);
    }
}

/// Native `HSD_AnimJoint` (GameCube record `melee_types::HSD_AnimJoint`, 0x14).
#[derive(Clone, Debug, Default)]
pub struct HsdAnimJoint {
    pub child: Option<Box<HsdAnimJoint>>,
    pub next: Option<Box<HsdAnimJoint>>,
    pub aobjdesc: Option<HsdAObjDesc>,
    /// The `robj_anim->next` chain.
    pub robj_anim: Vec<HsdRObjAnimJoint>,
    pub flags: u32,
}

/// Native `HSD_MatAnimJoint` / `HSD_ShapeAnimJoint`: a tree whose payload (`matanim` /
/// `shapeanimdobj`) is the DObj port's handle `A` (TODO(group1 dobj)).
#[derive(Clone, Debug)]
pub struct HsdSubAnimJoint<A> {
    pub child: Option<Box<HsdSubAnimJoint<A>>>,
    pub next: Option<Box<HsdSubAnimJoint<A>>>,
    pub anim: Option<A>,
}

/// Not-yet-ported code jobj.c's animation / IK half calls.
pub trait JObjExt: RObjHost {
    /// The DObj handle of the tree.
    type D: Copy + PartialEq;
    type MatAnim;
    type ShapeAnim;
    /// `HSD_DObjRemoveAnimAllByFlags`. TODO(group1 dobj).
    fn dobj_remove_anim_all_by_flags(&mut self, dobj: &[Self::D], flags: u32);
    /// `HSD_DObjReqAnimAllByFlags`. TODO(group1 dobj).
    fn dobj_req_anim_all_by_flags(&mut self, dobj: &[Self::D], frame: f32, flags: u32);
    /// `HSD_DObjAddAnimAll`. TODO(group1 dobj).
    fn dobj_add_anim_all(&mut self, dobj: &[Self::D], mat: Option<&Self::MatAnim>, shape: Option<&Self::ShapeAnim>);
    /// `HSD_DObjAnimAll`. TODO(group1 dobj).
    fn dobj_anim_all(&mut self, dobj: &[Self::D]);
    /// `splArcLengthPoint(&p, jp->u.spline, s)`. TODO(group1 jobj spline).
    fn spline_point(&mut self, jp: JObjId, s: f32) -> Vec3;
    /// AObjLoadDesc's `obj_id` lookup. TODO(group1 jobj load).
    fn aobj_resolve_obj(&mut self, id: u32) -> Option<JObjId>;
    /// `HSD_JObjRef`. TODO(group1 class).
    fn jobj_ref(&mut self, j: Option<JObjId>);
    /// `HSD_JObjUnref`. TODO(group1 class).
    fn jobj_unref(&mut self, j: Option<JObjId>);
}

/// jobj.c's runtime: the RObj arena, the file statics and the not-ported hooks. It is the
/// `JObjHost` for its tree.
pub struct JObjRt<X> {
    pub robjs: Vec<HsdRObj>,
    pub g: JObjGlobals,
    pub rg: RObjGlobals,
    pub ag: AObjGlobals,
    /// Shared so `HSD_RObjUpdateAll` can get it as its `RObjHost` while this struct is its
    /// `JObjHost` (nested matrix builds); every use borrows it only for the one call.
    pub ext: Rc<RefCell<X>>,
}

/// The `RObjHost` view of [`JObjRt::ext`].
pub struct ExtProxy<X>(pub Rc<RefCell<X>>);

impl<X: RObjHost> RObjHost for ExtProxy<X> {
    fn jobj_ref_this(&mut self, j: JObjId) {
        self.0.borrow_mut().jobj_ref_this(j)
    }
    fn jobj_unref_this(&mut self, j: Option<JObjId>) {
        self.0.borrow_mut().jobj_unref_this(j)
    }
    fn byte_code_eval(&mut self, b: Option<&[u8]>, a: &[f32], n: u32) -> f32 {
        self.0.borrow_mut().byte_code_eval(b, a, n)
    }
}

pub(crate) type Tree<D> = JObjTree<D, u32>;

impl<X> JObjRt<X> {
    pub fn new(ext: X) -> Self {
        JObjRt { robjs: Vec::new(), g: JObjGlobals::default(), rg: RObjGlobals::default(), ag: AObjGlobals::default(), ext: Rc::new(RefCell::new(ext)) }
    }

    /// Adds `chain` to the arena as `j`'s RObj chain (replacing its handles).
    pub fn set_robj_chain<D: Copy + PartialEq>(&mut self, tree: &mut Tree<D>, j: JObjId, chain: Vec<HsdRObj>) {
        let base = self.robjs.len() as u32;
        let n = chain.len() as u32;
        self.robjs.extend(chain);
        tree.get_mut(j).robj = (base..base + n).collect();
    }

    /// A copy of `j`'s chain, in order.
    pub fn chain<D: Copy + PartialEq>(&self, tree: &Tree<D>, j: JObjId) -> Vec<HsdRObj> {
        tree.get(j).robj.iter().map(|&i| self.robjs[i as usize].clone()).collect()
    }

    fn put_chain<D: Copy + PartialEq>(&mut self, tree: &Tree<D>, j: JObjId, chain: Vec<HsdRObj>) {
        for (&i, r) in tree.get(j).robj.iter().zip(chain) {
            self.robjs[i as usize] = r;
        }
    }

    /// `HSD_RObjGetByType(jobj->robj, ty, sub)` as an arena index.
    pub fn robj_by_type<D: Copy + PartialEq>(&self, tree: &Tree<D>, j: JObjId, ty: u32, sub: u32) -> Option<u32> {
        let ids = &tree.get(j).robj;
        ids.iter().copied().find(|&i| {
            let c = &self.robjs[i as usize];
            c.flags & 0x8000_0000 != 0
                && c.flags & robj::ROBJ_TYPE_MASK == ty
                && (sub == 0 || sub == c.flags & 0x0FFF_FFFF)
        })
    }
}

impl<X: JObjExt> JObjRt<X> {
    /// Applies outside effects of JObjUpdateFunc in order.
    fn apply_events(&mut self, tree: &Tree<X::D>, events: Vec<UfcEvent>) {
        for e in events {
            match e {
                UfcEvent::IkRotateX(j, f) => {
                    if let Some(i) = self.robj_by_type(tree, j, REFTYPE_IKHINT, 0) {
                        self.robjs[i as usize].u.ik_hint.rotate_x = f;
                    }
                }
                UfcEvent::Ufc(j, ty, f) => {
                    for cb in self.g.ufc_callbacks.iter_mut() {
                        cb(j, ty, f);
                    }
                }
                UfcEvent::DPtcl(lo, hi, j) => {
                    if let Some(cb) = self.g.dptcl_callback.as_mut() {
                        cb(0, lo, hi, j);
                    }
                }
                UfcEvent::JSound(iv) => {
                    if let Some(cb) = self.g.jsound_callback.as_mut() {
                        cb(iv);
                    }
                }
                UfcEvent::PtclTgt(j, iv) => {
                    if let Some(cb) = self.g.ptcltgt_callback.as_mut() {
                        cb(j, iv);
                    }
                }
            }
        }
    }

    /// `JObjUpdateFunc`.
    pub fn update_func(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>, ty: u32, val: Option<&mut ObjData>) {
        let mut ev = Vec::new();
        let ext = self.ext.clone();
        update_func_core(tree, jobj, ty, val, &mut |jp, s| ext.borrow_mut().spline_point(jp, s), &mut ev);
        self.apply_events(tree, ev);
    }

    /// `HSD_JObjRemoveAnimByFlags`.
    pub fn remove_anim_by_flags(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>, flags: u32) {
        let Some(j) = jobj else { return };
        if flags & 1 != 0 {
            tree.get_mut(j).aobj = None; // HSD_AObjRemove
        }
        if tree.get(j).union_type_dobj() {
            let d = tree.get(j).dobj.clone();
            self.ext.borrow_mut().dobj_remove_anim_all_by_flags(&d, flags);
        }
        let mut c = self.chain(tree, j);
        robj::remove_anim_all_by_flags(&mut c, flags);
        self.put_chain(tree, j, c);
    }

    /// `HSD_JObjRemoveAnimAllByFlags`.
    pub fn remove_anim_all_by_flags(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>, flags: u32) {
        let Some(j) = jobj else { return };
        self.remove_anim_by_flags(tree, Some(j), flags);
        if tree.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = tree.get(j).child;
            while let Some(x) = c {
                self.remove_anim_all_by_flags(tree, Some(x), flags);
                c = tree.get(x).next;
            }
        }
    }

    /// `HSD_JObjRemoveAnim`.
    pub fn remove_anim(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>) {
        self.remove_anim_by_flags(tree, jobj, 0x7FF);
    }

    /// `HSD_JObjRemoveAnimAll`.
    pub fn remove_anim_all(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>) {
        self.remove_anim_all_by_flags(tree, jobj, 0x7FF);
    }

    /// `HSD_JObjReqAnimByFlags`.
    pub fn req_anim_by_flags(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>, flags: u32, frame: f32) {
        let Some(j) = jobj else { return };
        if flags & 1 != 0 {
            if let Some(a) = tree.get_mut(j).aobj.as_mut() {
                a.req_anim(frame);
            }
        }
        if tree.get(j).flags & (JOBJ_PTCL | JOBJ_SPLINE) == 0 {
            let d = tree.get(j).dobj.clone();
            self.ext.borrow_mut().dobj_req_anim_all_by_flags(&d, frame, flags);
        }
        let mut c = self.chain(tree, j);
        robj::req_anim_all_by_flags(&mut c, frame, flags);
        self.put_chain(tree, j, c);
    }

    /// `HSD_JObjReqAnimAllByFlags`.
    pub fn req_anim_all_by_flags(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>, flags: u32, frame: f32) {
        let Some(j) = jobj else { return };
        self.req_anim_by_flags(tree, Some(j), flags, frame);
        if tree.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = tree.get(j).child;
            while let Some(x) = c {
                self.req_anim_all_by_flags(tree, Some(x), flags, frame);
                c = tree.get(x).next;
            }
        }
    }

    /// `HSD_JObjReqAnimAll`.
    pub fn req_anim_all(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>, frame: f32) {
        self.req_anim_all_by_flags(tree, jobj, 0x7FF, frame);
    }

    /// `HSD_JObjReqAnim`.
    pub fn req_anim(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>, frame: f32) {
        self.req_anim_by_flags(tree, jobj, 0x7FF, frame);
    }

    /// `HSD_JObjAddAnim`.
    pub fn add_anim(
        &mut self,
        tree: &mut Tree<X::D>,
        jobj: Option<JObjId>,
        an: Option<&HsdAnimJoint>,
        mat: Option<&HsdSubAnimJoint<X::MatAnim>>,
        sh: Option<&HsdSubAnimJoint<X::ShapeAnim>>,
    ) {
        let Some(j) = jobj else { return };
        if let Some(an) = an {
            tree.get_mut(j).aobj = None; // HSD_AObjRemove of the old one
            let ext = self.ext.clone();
            let mut a = an.aobjdesc.as_ref().map(|d| HsdAObj::load_desc(d, |id| ext.borrow_mut().aobj_resolve_obj(id)));
            sort_anim(a.as_mut());
            tree.get_mut(j).aobj = a;
            let mut c = self.chain(tree, j);
            robj::add_anim_all(&mut c, &an.robj_anim, &mut |id| ext.borrow_mut().aobj_resolve_obj(id));
            self.put_chain(tree, j, c);
            if an.flags & 1 != 0 {
                tree.set_flags(Some(j), JOBJ_CLASSICAL_SCALE);
            } else {
                tree.clear_flags(Some(j), JOBJ_CLASSICAL_SCALE);
            }
        }
        if tree.get(j).union_type_dobj() {
            let d = tree.get(j).dobj.clone();
            self.ext.borrow_mut().dobj_add_anim_all(
                &d,
                mat.and_then(|m| m.anim.as_ref()),
                sh.and_then(|s| s.anim.as_ref()),
            );
        }
    }

    /// `HSD_JObjAddAnimAll`.
    pub fn add_anim_all(
        &mut self,
        tree: &mut Tree<X::D>,
        jobj: Option<JObjId>,
        an: Option<&HsdAnimJoint>,
        mat: Option<&HsdSubAnimJoint<X::MatAnim>>,
        sh: Option<&HsdSubAnimJoint<X::ShapeAnim>>,
    ) {
        let Some(j) = jobj else { return };
        self.add_anim(tree, Some(j), an, mat, sh);
        if tree.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut jp = tree.get(j).child;
            let mut aj = an.and_then(|a| a.child.as_deref());
            let mut mj = mat.and_then(|a| a.child.as_deref());
            let mut sj = sh.and_then(|a| a.child.as_deref());
            while let Some(x) = jp {
                self.add_anim_all(tree, Some(x), aj, mj, sj);
                jp = tree.get(x).next;
                aj = aj.and_then(|a| a.next.as_deref());
                mj = mj.and_then(|a| a.next.as_deref());
                sj = sj.and_then(|a| a.next.as_deref());
            }
        }
    }

    /// `HSD_JObjAnim`.
    pub fn anim(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>) {
        let Some(j) = jobj else { return };
        tree.check_depend(Some(j));
        if let Some(mut a) = tree.get_mut(j).aobj.take() {
            // The AObj is taken out while it runs so the update can write the JObj; JObjUpdateFunc
            // reads `jobj->aobj->hsd_obj` only (PATH), which is constant during the call.
            let target = a.hsd_obj;
            let mut ag = std::mem::take(&mut self.ag);
            {
                let ext = self.ext.clone();
                let mut ev: Vec<UfcEvent> = Vec::new();
                let mut pending: Vec<UfcEvent> = Vec::new();
                let tr = &mut *tree;
                let mut f = |ty: u8, fv: f32| {
                    let mut d = ObjData::from_f(fv);
                    let mut sp = |jp: JObjId, s: f32| ext.borrow_mut().spline_point(jp, s);
                    let saved = tr.get_mut(j).aobj.replace(HsdAObj { hsd_obj: target, ..HsdAObj::alloc() });
                    update_func_core(tr, Some(j), ty as u32, Some(&mut d), &mut sp, &mut ev);
                    tr.get_mut(j).aobj = saved;
                    pending.append(&mut ev);
                };
                a.interpret_anim(&mut ag, &mut Some(&mut f));
                tree.get_mut(j).aobj = Some(a);
                self.ag = ag;
                self.apply_events(tree, pending);
            }
        }
        let mut c = self.chain(tree, j);
        robj::anim_all(&mut c, &mut self.ag);
        self.put_chain(tree, j, c);
        if tree.get(j).union_type_dobj() {
            let d = tree.get(j).dobj.clone();
            self.ext.borrow_mut().dobj_anim_all(&d);
        }
    }

    /// `JObjAnimAll`.
    pub fn anim_all_sub(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>) {
        let Some(j) = jobj else { return };
        self.anim(tree, Some(j));
        if tree.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = tree.get(j).child;
            while let Some(x) = c {
                self.anim_all_sub(tree, Some(x));
                c = tree.get(x).next;
            }
        }
    }

    /// `HSD_JObjAnimAll` (the PC profiler brackets are not ported).
    pub fn anim_all(&mut self, tree: &mut Tree<X::D>, jobj: Option<JObjId>) {
        if jobj.is_some() {
            self.ag.init_end_callback();
            self.anim_all_sub(tree, jobj);
            self.ag.invoke_callbacks();
        }
    }

    /// `HSD_JObjSetCurrent`.
    pub fn set_current(&mut self, jobj: Option<JObjId>) {
        self.ext.borrow_mut().jobj_ref(jobj);
        let old = self.g.current_jobj;
        self.ext.borrow_mut().jobj_unref(old);
        self.g.current_jobj = jobj;
    }

    /// `HSD_JObjGetCurrent`.
    pub fn get_current(&self) -> Option<JObjId> {
        self.g.current_jobj
    }

    /// `HSD_JObjSetDPtclCallback`.
    pub fn set_dptcl_callback(&mut self, cb: Option<DPtclCallback>) {
        self.g.dptcl_callback = cb;
    }

    /// `HSD_RObjGetGlobalPosition(jobj->robj, ty, p)`.
    fn global_position(&mut self, tree: &mut Tree<X::D>, j: JObjId, ty: i32) -> Option<Vec3> {
        let c = self.chain(tree, j);
        robj::get_global_position(&c, ty, tree, self).1
    }

    /// `HSD_RObjUpdateAll(jobj->robj, jobj, JObjUpdateFunc)`.
    pub fn robj_update(&mut self, tree: &mut Tree<X::D>, j: JObjId) {
        let mut c = self.chain(tree, j);
        if c.is_empty() {
            return;
        }
        let mut rg = std::mem::take(&mut self.rg);
        let mut pending: Vec<UfcEvent> = Vec::new();
        {
            // The update needs the host for nested matrix builds; its own outside effects are
            // collected (see the module doc). The spline hook is not reachable: RObjUpdateAll
            // passes types 0x32-0x38 and expression types, and PATH would need the JObj's AObj.
            let mut ext_sp: Vec<(JObjId, f32)> = Vec::new();
            let mut upd = |t: &mut Tree<X::D>, o: JObjId, ty: u32, v: Option<RObjVal>| {
                let mut d = v.map(ObjData::from);
                let mut sp = |jp: JObjId, s: f32| {
                    ext_sp.push((jp, s));
                    panic!("JObjUpdateFunc PATH from HSD_RObjUpdateAll (TODO(group1 jobj spline))")
                };
                update_func_core(t, Some(o), ty, d.as_mut(), &mut sp, &mut pending);
            };
            let mut rhost = ExtProxy(self.ext.clone());
            robj::update_all(&mut c, &mut rg, tree, self, &mut rhost, j, &mut upd);
        }
        self.rg = rg;
        self.put_chain(tree, j, c);
        self.apply_events(tree, pending);
    }

    /// `resolveIKJoint1`.
    pub fn resolve_ik_joint1(&mut self, tree: &mut Tree<X::D>, j: JObjId) {
        let mut var_r30 = false;
        let mut var_f29 = 0.0f32;
        let var_r31 = jobj_get_joint2(tree, tree.get(j).child);
        let mut sp_b0 = Vec3::default();
        let mut sp_bc = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
        let var_f5 = 1e-8f32;
        if let Some(s) = tree.get(j).scl {
            sp_bc = s;
        }
        let r = self.robj_by_type(tree, j, REFTYPE_IKHINT, 0).expect("HSD_ASSERT(0x853, robj)");
        let hint = self.robjs[r as usize].u.ik_hint;
        let temp_f26 = hint.rotate_x;
        let temp_f30 = hint.bone_length * sp_bc.x;
        let var_r28 = match var_r31 {
            Some(j2) => {
                let r2 = self.robj_by_type(tree, j2, REFTYPE_IKHINT, 0).expect("HSD_ASSERT(0x85E, robj)");
                let rr = &self.robjs[r2 as usize];
                var_f29 = rr.u.ik_hint.bone_length * tree.get(j2).scale.x * sp_bc.x;
                var_r30 = rr.flags & 4 != 0;
                self.effector_checked(tree, tree.get(j2).child)
            }
            None => self.effector_checked(tree, tree.get(j).child),
        };
        let Some(eff) = var_r28 else { return };
        if self.robj_by_type(tree, j, REFTYPE_JOBJ, 3).is_none() && !tree.get(j).robj.is_empty() {
            self.robj_update(tree, j);
            if tree.mtx_is_dirty(j) {
                tree.make_matrix(self, j);
                tree.get_mut(j).flags &= 0xFFFF_FFBF;
            }
        }
        if let Some(p) = tree.get(j).parent {
            sp_b0 = hsd_mtx::get_translate(&tree.get(p).mtx);
        }
        if let Some(p) = self.global_position(tree, eff, 1) {
            tree.get_mut(eff).translate = p;
        }
        let sp8c = mtx::vec_sub(&tree.get(eff).translate, &sp_b0);
        let temp_f31 = mtx::vec_dot(&sp8c, &sp8c);
        // Locals the C leaves uninitialised on the short-vector path: the shipping build
        // zero-initialises them (-ftrivial-auto-var-init=zero).
        let (var_f1, mut var_f29_2, var_f28, var_f27, sp80, sp74);
        if temp_f31 > var_f5 {
            let sp68 = sp8c;
            let mut sp5c;
            let sp50;
            if let Some(g) = self.global_position(tree, j, 3) {
                sp5c = mtx::vec_sub(&g, &sp_b0);
                if temp_f26 != 0.0 {
                    let m = mtx::rot_axis_rad(&sp68, temp_f26);
                    sp5c = mtx::mult_vec(&m, &sp5c);
                }
                let s50 = mtx::vec_cross(&sp68, &sp5c);
                sp5c = mtx::vec_cross(&s50, &sp68);
                sp50 = s50;
            } else {
                let m = &tree.get(j).mtx;
                let s50 = Vec3 { x: m[0][2], y: m[1][2], z: m[2][2] };
                sp5c = mtx::vec_cross(&s50, &sp68);
                sp50 = mtx::vec_cross(&sp68, &sp5c);
            }
            let f4 = sqrtf(1.0 / (1e-10 + mtx::vec_dot(&sp50, &sp50)));
            sp80 = mtx::vec_scale(&sp50, f4);
            let f4_2 = sqrtf(1.0 / (1e-10 + mtx::vec_dot(&sp5c, &sp5c)));
            sp74 = mtx::vec_scale(&sp5c, f4_2);
            let temp_f5 = temp_f30 * temp_f30;
            var_f28 = var_f29 * var_f29;
            let temp_f1_7 = temp_f5 - var_f28;
            let temp_f1_8 = 0.25 * (fmadds(2.0, temp_f5 + var_f28, -temp_f31) - ((temp_f1_7 * temp_f1_7) / temp_f31));
            let mut f27 = temp_f1_8;
            if temp_f1_8 < 0.0 {
                f27 = 0.0;
            }
            var_f27 = f27;
            let temp_f5_2 = (temp_f5 - var_f27) / temp_f31;
            let f4_3 = sqrtf(1.0 / (1e-10 + temp_f5_2));
            var_f1 = temp_f5_2 * f4_3;
            let f5 = sqrtf(1.0 / (1e-10 + var_f27));
            var_f29_2 = var_f27 * f5;
        } else {
            var_f1 = 0.0;
            var_f29_2 = temp_f30;
            var_f28 = 0.0;
            var_f27 = 0.0;
            sp80 = Vec3::default();
            sp74 = Vec3::default();
        }
        if var_r30 {
            var_f29_2 = -var_f29_2;
        }
        let mut sp98 = if (var_f28 - var_f27) < temp_f31 {
            mtx::vec_scale(&sp8c, var_f1)
        } else {
            mtx::vec_scale(&sp8c, -var_f1)
        };
        let sp5c = mtx::vec_scale(&sp74, var_f29_2);
        sp98 = mtx::vec_add(&sp98, &sp5c);
        let f4_4 = sqrtf(1.0 / (1e-10 + mtx::vec_dot(&sp98, &sp98)));
        sp98 = mtx::vec_scale(&sp98, f4_4);
        let sp5c = mtx::vec_cross(&sp80, &sp98);
        let m = &mut tree.get_mut(j).mtx;
        m[0][0] = sp98.x * sp_bc.x;
        m[1][0] = sp98.y * sp_bc.x;
        m[2][0] = sp98.z * sp_bc.x;
        m[0][1] = sp5c.x * sp_bc.y;
        m[1][1] = sp5c.y * sp_bc.y;
        m[2][1] = sp5c.z * sp_bc.y;
        m[0][2] = sp80.x * sp_bc.z;
        m[1][2] = sp80.y * sp_bc.z;
        m[2][2] = sp80.z * sp_bc.z;
        m[0][3] = sp_b0.x;
        m[1][3] = sp_b0.y;
        m[2][3] = sp_b0.z;
    }

    /// `resolveIKJoint2`.
    pub fn resolve_ik_joint2(&mut self, tree: &mut Tree<X::D>, j: JObjId) {
        let mut var_f31 = 1.0f32;
        let mut sp_a0 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
        let eff = self.effector_checked(tree, tree.get(j).child);
        let (Some(eff), Some(par)) = (eff, tree.get(j).parent) else { return };
        if let Some(s) = tree.get(j).scl {
            sp_a0 = s;
        }
        let pm = tree.get(par).mtx;
        let sp88 = Vec3 { x: pm[0][3], y: pm[1][3], z: pm[2][3] };
        let mut sp7c = Vec3 { x: pm[0][0], y: pm[1][0], z: pm[2][0] };
        let f4 = sqrtf(1.0 / (1e-10 + mtx::vec_dot(&sp7c, &sp7c)));
        sp7c = mtx::vec_scale(&sp7c, f4);
        if let Some(s) = tree.get(par).scl {
            var_f31 = s.x;
        }
        let r = self.robj_by_type(tree, par, REFTYPE_IKHINT, 0).expect("HSD_ASSERT(0x8FC, robj)");
        sp7c = mtx::vec_scale(&sp7c, self.robjs[r as usize].u.ik_hint.bone_length * var_f31);
        let sp94 = mtx::vec_add(&sp88, &sp7c);
        sp7c = mtx::vec_sub(&tree.get(eff).translate, &sp94);
        sp7c = mtx::vec_scale(&sp7c, sqrtf(1.0 / (1e-10 + mtx::vec_dot(&sp7c, &sp7c))));
        let t28 = self.robj_by_type(tree, j, 0x2000_0000, 5);
        let t29 = self.robj_by_type(tree, j, 0x2000_0000, 6);
        if t28.is_some() || t29.is_some() {
            let mut clamped = false;
            let r = self.robj_by_type(tree, j, REFTYPE_IKHINT, 0).expect("HSD_ASSERT(0x91E, robj)");
            let var_r30 = self.robjs[r as usize].flags & 4 != 0;
            let sp28 = mtx::vec_normalize(&Vec3 { x: pm[0][0], y: pm[1][0], z: pm[2][0] });
            let d = mtx::vec_dot(&sp28, &sp7c);
            let mut a = if d >= 1.0 {
                0.0
            } else if d <= -1.0 {
                std::f64::consts::PI as f32
            } else {
                acosf(d)
            };
            if !var_r30 {
                a = -a;
            }
            if let Some(lo) = t28.map(|i| self.robjs[i as usize].u.limit).filter(|&l| a < l) {
                a = lo;
                clamped = true;
            } else if let Some(i) = t29 {
                let hi = self.robjs[i as usize].u.limit;
                if hi < a {
                    a = hi;
                    clamped = true;
                }
            }
            if clamped {
                let sp1c = Vec3 { x: pm[0][2], y: pm[1][2], z: pm[2][2] };
                let m = mtx::rot_axis_rad(&sp1c, a);
                sp7c = mtx::mult_vec(&m, &sp28);
            }
        }
        let mut sp64 = Vec3 { x: pm[0][2], y: pm[1][2], z: pm[2][2] };
        let mut sp70 = mtx::vec_cross(&sp64, &sp7c);
        let f4_2 = sqrtf(1.0 / (1e-10 + mtx::vec_dot(&sp70, &sp70)));
        sp70 = mtx::vec_scale(&sp70, f4_2);
        sp64 = mtx::vec_cross(&sp7c, &sp70);
        let m = &mut tree.get_mut(j).mtx;
        m[0][0] = sp7c.x * sp_a0.x;
        m[1][0] = sp7c.y * sp_a0.x;
        m[2][0] = sp7c.z * sp_a0.x;
        m[0][1] = sp70.x * sp_a0.y;
        m[1][1] = sp70.y * sp_a0.y;
        m[2][1] = sp70.z * sp_a0.y;
        m[0][2] = sp64.x * sp_a0.z;
        m[1][2] = sp64.y * sp_a0.z;
        m[2][2] = sp64.z * sp_a0.z;
        m[0][3] = sp94.x;
        m[1][3] = sp94.y;
        m[2][3] = sp94.z;
    }

    /// `jobj_get_effector_checked`.
    pub fn effector_checked(&self, tree: &Tree<X::D>, j: Option<JObjId>) -> Option<JObjId> {
        let eff = jobj_get_effector(tree, j).expect("HSD_ASSERT(2084, eff)");
        self.robj_by_type(tree, eff, REFTYPE_JOBJ, 1).map(|_| eff)
    }
}

impl<X: JObjExt> JObjHost<X::D, u32> for JObjRt<X> {
    fn ik_hint_bone_length(&mut self, robj: &[u32]) -> Option<f32> {
        robj.iter()
            .map(|&i| &self.robjs[i as usize])
            .find(|c| c.flags & 0x8000_0000 != 0 && c.flags & robj::ROBJ_TYPE_MASK == REFTYPE_IKHINT)
            .map(|c| c.u.ik_hint.bone_length)
    }
    fn robj_update_all(&mut self, tree: &mut Tree<X::D>, jobj: JObjId) {
        self.robj_update(tree, jobj);
    }
    fn resolve_ik_joint1(&mut self, tree: &mut Tree<X::D>, jobj: JObjId) {
        JObjRt::resolve_ik_joint1(self, tree, jobj);
    }
    fn resolve_ik_joint2(&mut self, tree: &mut Tree<X::D>, jobj: JObjId) {
        JObjRt::resolve_ik_joint2(self, tree, jobj);
    }
    fn unref(&mut self, _tree: &mut Tree<X::D>, jobj: JObjId) {
        self.ext.borrow_mut().jobj_unref(Some(jobj));
    }
}

pub use crate::tu::jobj::jobj_get_effector;
pub use crate::tu::jobj::jobj_get_joint2;
