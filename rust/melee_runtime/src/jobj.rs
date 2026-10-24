//! Port of `src/sysdolphin/baselib/jobj.c` (round 5): the JObj tree, its flags, the dirty-matrix
//! propagation and the matrix build (`HSD_JObjMakeMatrix`, `HSD_JObjSetupMatrixSub`).
//!
//! Native shape: a [`JObjTree`] arena owns every JObj; links (`next`, `parent`, `child`) are
//! [`JObjId`]s. `D` is the DObj handle and `R` the RObj handle the caller uses; the C's
//! intrusive `dobj->next` / `robj->next` chains are `Vec`s in list order (front = head).
//! The AObj is the round 4 [`HsdAObj`] whose `hsd_obj` is a `JObjId`. `scl` is `Option<Vec3>`
//! (`HSD_VecAlloc` / `HSD_VecFree`). The class `make_mtx` method is the default
//! `HSD_JObjMakeMatrix` (no game class overrides it).
//!
//! Calls into code that is not ported yet go through [`JObjHost`] (round 8: `jobj_rt::JObjRt` implements
//! it over the RObj port and the IK solvers): the RObj queries, the IK solvers
//! `resolveIKJoint1/2` (all real in `JObjRt`) and `HSD_JObjUnref` (the HSD_Obj refcount / class
//! release: TODO(group1 class)).
//!
//! FMA: `fma_sites.py` finds no fused site in any function ported here; the matrix code is the
//! round 1/2 `hsd_mtx` / `mtx` ports (fused where mtx.c / mtx_pc.c fuse) and the effector branch
//! uses math_ppc's `sqrtf` (its double fma is in [`crate::gekko::sqrtf`]).
//! GameCube records: `melee_types::HSD_JObj` (0x88), `HSD_Joint` (0x40).

use crate::aobj::HsdAObj;
use crate::gekko::sqrtf;
use crate::hsd_mtx;
use crate::mtx::{self, Mtx, Quaternion, Vec3};

pub const JOBJ_SKELETON: u32 = 1 << 0;
pub const JOBJ_SKELETON_ROOT: u32 = 1 << 1;
pub const JOBJ_ENVELOPE_MODEL: u32 = 1 << 2;
pub const JOBJ_CLASSICAL_SCALE: u32 = 1 << 3;
pub const JOBJ_HIDDEN: u32 = 1 << 4;
pub const JOBJ_PTCL: u32 = 1 << 5;
pub const JOBJ_MTX_DIRTY: u32 = 1 << 6;
pub const JOBJ_INSTANCE: u32 = 1 << 12;
pub const JOBJ_SPLINE: u32 = 1 << 14;
pub const JOBJ_USE_QUATERNION: u32 = 1 << 17;
pub const JOBJ_JOINT1: u32 = 1 << 21;
pub const JOBJ_JOINT2: u32 = 2 << 21;
pub const JOBJ_JOINT: u32 = 3 << 21;
pub const JOBJ_EFFECTOR: u32 = 3 << 21;
pub const JOBJ_USER_DEF_MTX: u32 = 1 << 23;
pub const JOBJ_MTX_INDEP_PARENT: u32 = 1 << 24;
pub const JOBJ_MTX_INDEP_SRT: u32 = 1 << 25;
/// m-ex scale compensation (TARGET_PC block in HSD_JObjMakeMatrix).
pub const JOBJ_MEX_SCALE_COMPENSATE: u32 = 1 << 26;
pub const JOBJ_ROOT_OPA: u32 = 1 << 28;
pub const JOBJ_ROOT_XLU: u32 = 1 << 29;
pub const JOBJ_ROOT_TEXEDGE: u32 = 1 << 30;
pub const JOBJ_ROOT_MASK: u32 = JOBJ_ROOT_OPA | JOBJ_ROOT_TEXEDGE | JOBJ_ROOT_XLU;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JObjId(pub u32);

/// `HSD_JObj`. `u` is kept as the DObj chain only (the ptcl / spline members of the C union are
/// not ported; `union_type_dobj` still decides whether the chain is visible).
#[derive(Clone, Debug)]
pub struct HsdJObj<D, R> {
    pub next: Option<JObjId>,
    pub parent: Option<JObjId>,
    pub child: Option<JObjId>,
    pub flags: u32,
    pub dobj: Vec<D>,
    pub rotate: Quaternion,
    pub scale: Vec3,
    pub translate: Vec3,
    pub mtx: Mtx,
    pub scl: Option<Vec3>,
    pub aobj: Option<HsdAObj<JObjId>>,
    pub robj: Vec<R>,
}

impl<D, R> melee_core::gc::GcLayout for HsdJObj<D, R> {
    const GC_NAME: &'static str = <melee_types::HSD_JObj as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_JObj as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_JObj as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_JObj as melee_core::gc::GcLayout>::GC_FIELDS;
}

impl<D, R> HsdJObj<D, R> {
    /// `HSD_JObjAlloc` + `JObjInit` (the object memory is zeroed, then flags = MTX_DIRTY and
    /// scale = 1).
    pub fn new() -> Self {
        HsdJObj {
            next: None,
            parent: None,
            child: None,
            flags: JOBJ_MTX_DIRTY,
            dobj: Vec::new(),
            rotate: Quaternion::default(),
            scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
            translate: Vec3::default(),
            mtx: [[0.0; 4]; 3],
            scl: None,
            aobj: None,
            robj: Vec::new(),
        }
    }

    /// `HSD_JObjMtxIsDirty`.
    #[inline]
    pub fn mtx_is_dirty(&self) -> bool {
        self.flags & JOBJ_USER_DEF_MTX == 0 && self.flags & JOBJ_MTX_DIRTY != 0
    }

    /// `union_type_dobj`.
    #[inline]
    pub fn union_type_dobj(&self) -> bool {
        self.flags & (JOBJ_PTCL | JOBJ_SPLINE) == 0
    }
}

impl<D, R> Default for HsdJObj<D, R> {
    fn default() -> Self {
        Self::new()
    }
}

/// Read access to an `HSD_Joint` tree (for `HSD_JObjResetRST`).
pub trait JointSource {
    type Id: Copy;
    fn rotation(&self, j: Self::Id) -> Vec3;
    fn scale(&self, j: Self::Id) -> Vec3;
    fn position(&self, j: Self::Id) -> Vec3;
    fn child(&self, j: Self::Id) -> Option<Self::Id>;
    fn next(&self, j: Self::Id) -> Option<Self::Id>;
}

/// The not-yet-ported code jobj.c calls.
pub trait JObjHost<D, R> {
    /// `HSD_RObjGetByType(robj, REFTYPE_IKHINT, 0)` -> `u.ik_hint.bone_length`.
    /// Real in `jobj_rt::JObjRt` (robj.rs `get_by_type`).
    fn ik_hint_bone_length(&mut self, robj: &[R]) -> Option<f32>;
    /// `HSD_RObjUpdateAll(jobj->robj, jobj, JObjUpdateFunc)`. Real in `JObjRt`.
    fn robj_update_all(&mut self, tree: &mut JObjTree<D, R>, jobj: JObjId);
    /// `resolveIKJoint1`. Real in `JObjRt`.
    fn resolve_ik_joint1(&mut self, tree: &mut JObjTree<D, R>, jobj: JObjId);
    /// `resolveIKJoint2`. Real in `JObjRt`.
    fn resolve_ik_joint2(&mut self, tree: &mut JObjTree<D, R>, jobj: JObjId);
    /// `HSD_JObjUnref` (refcount + class release). TODO(group1 class).
    fn unref(&mut self, tree: &mut JObjTree<D, R>, jobj: JObjId);
}

/// Owner of every JObj.
#[derive(Clone, Debug, Default)]
pub struct JObjTree<D, R> {
    pub nodes: Vec<HsdJObj<D, R>>,
}

impl<D: Copy + PartialEq, R: Copy + PartialEq> JObjTree<D, R> {
    pub fn new() -> Self {
        JObjTree { nodes: Vec::new() }
    }

    /// `HSD_JObjAlloc`.
    pub fn alloc(&mut self) -> JObjId {
        self.nodes.push(HsdJObj::new());
        JObjId((self.nodes.len() - 1) as u32)
    }

    #[inline]
    pub fn get(&self, j: JObjId) -> &HsdJObj<D, R> {
        &self.nodes[j.0 as usize]
    }
    #[inline]
    pub fn get_mut(&mut self, j: JObjId) -> &mut HsdJObj<D, R> {
        &mut self.nodes[j.0 as usize]
    }

    /// `HSD_JObjGetChild` / `GetNext` / `GetParent`.
    pub fn child_of(&self, j: Option<JObjId>) -> Option<JObjId> {
        j.and_then(|j| self.get(j).child)
    }
    pub fn next_of(&self, j: Option<JObjId>) -> Option<JObjId> {
        j.and_then(|j| self.get(j).next)
    }
    pub fn parent_of(&self, j: Option<JObjId>) -> Option<JObjId> {
        j.and_then(|j| self.get(j).parent)
    }

    /// `HSD_JObjMtxIsDirty`.
    pub fn mtx_is_dirty(&self, j: JObjId) -> bool {
        self.get(j).mtx_is_dirty()
    }

    /// `HSD_JObjSetMtxDirtySub`.
    pub fn set_mtx_dirty_sub(&mut self, j: JObjId) {
        self.get_mut(j).flags |= 0x40;
        if self.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut child = self.get(j).child;
            while let Some(c) = child {
                if self.get(c).flags & JOBJ_MTX_INDEP_PARENT == 0 && !self.mtx_is_dirty(c) {
                    self.set_mtx_dirty_sub(c);
                }
                child = self.get(c).next;
            }
        }
    }

    /// `HSD_JObjSetMtxDirty` (header macro).
    pub fn set_mtx_dirty(&mut self, j: Option<JObjId>) {
        if let Some(j) = j {
            if !self.mtx_is_dirty(j) {
                self.set_mtx_dirty_sub(j);
            }
        }
    }

    /// `HSD_JObjCheckDepend`.
    pub fn check_depend(&mut self, j: Option<JObjId>) {
        let Some(j) = j else { return };
        if self.mtx_is_dirty(j) {
            return;
        }
        let o = self.get(j);
        let parent = o.parent;
        if o.flags & JOBJ_USER_DEF_MTX != 0 {
            if o.flags & JOBJ_MTX_INDEP_PARENT == 0 {
                if let Some(p) = parent {
                    if self.mtx_is_dirty(p) {
                        self.get_mut(j).flags |= JOBJ_MTX_DIRTY;
                    }
                }
            }
        } else {
            let eff = o.flags & JOBJ_EFFECTOR;
            if parent.is_some_and(|p| self.get(p).flags & JOBJ_MTX_DIRTY != 0)
                || eff == JOBJ_JOINT1
                || eff == JOBJ_JOINT2
                || eff == JOBJ_EFFECTOR
                || !o.robj.is_empty()
            {
                self.get_mut(j).flags |= JOBJ_MTX_DIRTY;
            }
        }
    }

    /// `JObjResetRST`.
    pub fn reset_rst_one<J: JointSource>(&mut self, j: Option<JObjId>, src: &J, joint: Option<J::Id>) {
        let (Some(j), Some(jt)) = (j, joint) else { return };
        let r = src.rotation(jt);
        let o = self.get_mut(j);
        o.rotate.x = r.x;
        o.rotate.y = r.y;
        o.rotate.z = r.z;
        o.scale = src.scale(jt);
        o.translate = src.position(jt);
        if o.flags & JOBJ_MTX_INDEP_SRT == 0 {
            self.set_mtx_dirty(Some(j));
        }
    }

    /// `HSD_JObjResetRST`.
    pub fn reset_rst<J: JointSource>(&mut self, j: Option<JObjId>, src: &J, joint: Option<J::Id>) {
        let (Some(jo), Some(_)) = (j, joint) else { return };
        self.reset_rst_one(j, src, joint);
        if self.get(jo).flags & JOBJ_INSTANCE == 0 {
            let mut cj = self.get(jo).child;
            let mut cjt = joint.and_then(|x| src.child(x));
            while let Some(c) = cj {
                self.reset_rst(Some(c), src, cjt);
                cj = self.get(c).next;
                cjt = cjt.and_then(|x| src.next(x));
            }
        }
    }

    /// `HSD_JObjWalkTree0`; `cb(jobj, type)` with type 1 = first child, 2 = later sibling.
    pub fn walk_tree0(&self, j: JObjId, cb: &mut dyn FnMut(JObjId, u32)) {
        let parent = self.get(j).parent.expect("HSD_JObjWalkTree0: jobj->parent");
        let ty = if self.get(parent).child == Some(j) { 1 } else { 2 };
        cb(j, ty);
        if self.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = self.get(j).child;
            while let Some(x) = c {
                self.walk_tree0(x, cb);
                c = self.get(x).next;
            }
        }
    }

    /// `HSD_JObjWalkTree` (the `cb_args` pointer is captured by the closure).
    pub fn walk_tree(&self, j: Option<JObjId>, cb: &mut dyn FnMut(JObjId, u32)) {
        let Some(j) = j else { return };
        cb(j, 0);
        if self.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = self.get(j).child;
            while let Some(x) = c {
                self.walk_tree0(x, cb);
                c = self.get(x).next;
            }
        }
    }

    /// `HSD_JObjSetupMatrix` (header inline).
    pub fn setup_matrix<H: JObjHost<D, R>>(&mut self, host: &mut H, j: Option<JObjId>) {
        let Some(j) = j else { return };
        if !self.mtx_is_dirty(j) {
            return;
        }
        self.setup_matrix_sub(host, j);
    }

    /// `HSD_JObjMakeMatrix` (with the TARGET_PC m-ex scale compensation, flag bit 26).
    pub fn make_matrix<H: JObjHost<D, R>>(&mut self, host: &mut H, j: JObjId) {
        let parent = self.get(j).parent;
        self.setup_matrix(host, parent);
        let pscl = parent.and_then(|p| self.get(p).scl);
        let o = self.get_mut(j);
        if o.flags & 8 != 0 {
            o.scl = pscl;
        } else {
            o.scl = Some(match pscl {
                Some(ps) => Vec3 { x: o.scale.x * ps.x, y: o.scale.y * ps.y, z: o.scale.z * ps.z },
                None => o.scale,
            });
        }
        let r3 = Vec3 { x: o.rotate.x, y: o.rotate.y, z: o.rotate.z };
        if o.flags & JOBJ_USE_QUATERNION != 0 {
            o.mtx = hsd_mtx::srt_quat(&o.scale, &o.rotate, &o.translate, pscl.as_ref());
        } else if o.flags & JOBJ_MEX_SCALE_COMPENSATE != 0 && parent.is_some() {
            let ps = self.get(parent.unwrap()).scale;
            let o = self.get_mut(j);
            let st = Vec3 { x: o.translate.x * ps.x, y: o.translate.y * ps.y, z: o.translate.z * ps.z };
            let m = hsd_mtx::srt(&o.scale, &r3, &st, pscl.as_ref());
            let mut comp = mtx::identity();
            comp[0][0] = (1.0 / ps.x as f64) as f32;
            comp[1][1] = (1.0 / ps.y as f64) as f32;
            comp[2][2] = (1.0 / ps.z as f64) as f32;
            o.mtx = mtx::concat(&comp, &m);
        } else {
            o.mtx = hsd_mtx::srt(&o.scale, &r3, &o.translate, pscl.as_ref());
        }
        if let Some(p) = parent {
            let pm = self.get(p).mtx;
            let o = self.get_mut(j);
            o.mtx = mtx::concat(&pm, &o.mtx);
        }
        if let Some(target) = self.get(j).aobj.as_ref().and_then(|a| a.hsd_obj) {
            self.setup_matrix(host, Some(target));
            let v = mtx::mult_vec(&self.get(target).mtx, &self.get(j).translate);
            let o = self.get_mut(j);
            o.mtx[0][3] = v.x;
            o.mtx[1][3] = v.y;
            o.mtx[2][3] = v.z;
        }
    }

    /// `HSD_JObjSetupMatrixSub` (the TARGET_PC `diag_jobj_mtx` instrumentation is not ported:
    /// it has no effect on the result).
    pub fn setup_matrix_sub<H: JObjHost<D, R>>(&mut self, host: &mut H, j: JObjId) {
        self.make_matrix(host, j);
        self.get_mut(j).flags &= !JOBJ_MTX_DIRTY;
        if self.get(j).flags & JOBJ_USER_DEF_MTX != 0 {
            return;
        }
        match self.get(j).flags & JOBJ_JOINT {
            JOBJ_JOINT1 => host.resolve_ik_joint1(self, j),
            JOBJ_JOINT2 => host.resolve_ik_joint2(self, j),
            JOBJ_EFFECTOR => {
                if let Some(p) = self.get(j).parent {
                    let mut x_scale = 1.0f32;
                    if let Some(bone_length) = host.ik_hint_bone_length(&self.get(p).robj) {
                        let pm = self.get(p).mtx;
                        let sp1c = Vec3 { x: pm[0][3], y: pm[1][3], z: pm[2][3] };
                        let mut sp10 = Vec3 { x: pm[0][0], y: pm[1][0], z: pm[2][0] };
                        sp10 = mtx::vec_scale(&sp10, sqrtf(1.0 / (1e-10 + mtx::vec_dot(&sp10, &sp10))));
                        if let Some(s) = self.get(p).scl {
                            x_scale = s.x;
                        }
                        sp10 = mtx::vec_scale(&sp10, bone_length * x_scale);
                        let sp28 = mtx::vec_add(&sp1c, &sp10);
                        let o = self.get_mut(j);
                        o.mtx[0][3] = sp28.x;
                        o.mtx[1][3] = sp28.y;
                        o.mtx[2][3] = sp28.z;
                    }
                }
            }
            _ => {
                if !self.get(j).robj.is_empty() {
                    host.robj_update_all(self, j);
                    if self.mtx_is_dirty(j) {
                        self.make_matrix(host, j);
                        self.get_mut(j).flags &= !JOBJ_MTX_DIRTY;
                    }
                }
            }
        }
        self.get_mut(j).flags &= !JOBJ_MTX_DIRTY;
    }

    /// `HSD_JObjRemove`: unlinks `j`, splicing its single child into its place; returns the
    /// child.
    pub fn remove<H: JObjHost<D, R>>(&mut self, host: &mut H, j: Option<JObjId>) -> Option<JObjId> {
        let j = j?;
        let child = self.get(j).child;
        if let Some(c) = child {
            assert!(self.get(c).next.is_none(), "HSD_JObjRemove: child->next == NULL");
        }
        let next = if child.is_some() { child } else { self.get(j).next };
        let prev = self.get_prev(Some(j));
        if let Some(p) = prev {
            self.get_mut(p).next = next;
        } else if let Some(par) = self.get(j).parent {
            self.get_mut(par).child = next;
        }
        if let Some(n) = next {
            if next == child {
                let (jn, jp) = (self.get(j).next, self.get(j).parent);
                let o = self.get_mut(n);
                o.next = jn;
                o.parent = jp;
            }
        }
        let o = self.get_mut(j);
        o.parent = None;
        o.child = None;
        o.next = None;
        host.unref(self, j);
        child
    }

    /// `HSD_JObjRemoveAll`: detaches `j` and every later sibling, unref'ing each.
    pub fn remove_all<H: JObjHost<D, R>>(&mut self, host: &mut H, j: Option<JObjId>) {
        let Some(mut cur) = j else { return };
        if let Some(par) = self.get(cur).parent {
            match self.get_prev(Some(cur)) {
                Some(p) => self.get_mut(p).next = None,
                None => self.get_mut(par).child = None,
            }
        }
        loop {
            let next = self.get(cur).next;
            let o = self.get_mut(cur);
            o.parent = None;
            o.next = None;
            host.unref(self, cur);
            match next {
                Some(n) => cur = n,
                None => break,
            }
        }
    }

    /// `RecalcParentTrspBits` (walks `next`, as the C does).
    pub fn recalc_parent_trsp_bits(&mut self, mut j: Option<JObjId>) {
        while let Some(x) = j {
            let mut child = self.get(x).child;
            let mut flags = !JOBJ_ROOT_MASK;
            while let Some(c) = child {
                let cf = self.get(c).flags;
                flags |= (cf | cf << 10) & JOBJ_ROOT_MASK;
                child = self.get(c).next;
            }
            if self.get(x).flags & !flags == 0 {
                break;
            }
            self.get_mut(x).flags &= flags;
            j = self.get(x).next;
        }
    }

    /// `UpdateParentTrspBits` (static).
    pub fn update_parent_trsp_bits(&mut self, mut j: Option<JObjId>, child: JObjId) {
        let cf = self.get(child).flags;
        let flags = (cf | cf << 10) & JOBJ_ROOT_MASK;
        while let Some(x) = j {
            if flags & !self.get(x).flags == 0 {
                break;
            }
            self.get_mut(x).flags |= flags;
            j = self.get(x).parent;
        }
    }

    /// `HSD_JObjAddChild`.
    pub fn add_child(&mut self, j: Option<JObjId>, child: Option<JObjId>) {
        let (Some(j), Some(child)) = (j, child) else { return };
        assert!(self.get(child).parent.is_none(), "child should be a orphan.");
        assert!(self.get(child).next.is_none(), "child should not have siblings");
        match self.get(j).child {
            None => self.get_mut(j).child = Some(child),
            Some(first) => {
                assert!(self.get(j).flags & JOBJ_INSTANCE == 0);
                let mut last = first;
                while let Some(n) = self.get(last).next {
                    assert!(last != child);
                    last = n;
                }
                self.get_mut(last).next = Some(child);
            }
        }
        self.get_mut(child).parent = Some(j);
        self.update_parent_trsp_bits(Some(j), child);
    }

    /// `HSD_JObjReparent`: returns `j`'s old `next`.
    pub fn reparent(&mut self, j: Option<JObjId>, parent: Option<JObjId>) -> Option<JObjId> {
        let j = j?;
        let next = self.get(j).next;
        if let Some(par) = self.get(j).parent {
            if self.get(par).child == Some(j) {
                self.get_mut(par).child = next;
            } else {
                let prev = self.get_prev(Some(j)).expect("HSD_JObjReparent: prev");
                self.get_mut(prev).next = next;
            }
            self.recalc_parent_trsp_bits(Some(par));
            self.get_mut(j).parent = None;
        }
        self.get_mut(j).next = None;
        self.add_child(parent, Some(j));
        next
    }

    /// `HSD_JObjAddNext`.
    pub fn add_next(&mut self, j: Option<JObjId>, next: Option<JObjId>) {
        let (Some(j), Some(next)) = (j, next) else { return };
        let parent = self.get(j).parent;
        let mut cur = match parent {
            Some(p) => {
                let c = self.get(p).child;
                self.get_mut(p).child = None;
                self.get_mut(j).flags &= !JOBJ_ROOT_MASK;
                c
            }
            None => Some(j),
        };
        self.reparent(Some(next), parent);
        match self.get(next).child {
            Some(mut c) => {
                while let Some(n) = self.get(c).next {
                    c = n;
                }
                self.get_mut(c).next = cur;
            }
            None => self.get_mut(next).child = cur,
        }
        while let Some(c) = cur {
            self.get_mut(c).parent = Some(next);
            self.update_parent_trsp_bits(Some(next), c);
            cur = self.get(c).next;
        }
    }

    /// `HSD_JObjGetPrev`.
    pub fn get_prev(&self, j: Option<JObjId>) -> Option<JObjId> {
        let j = j?;
        let par = self.get(j).parent?;
        if self.get(par).child == Some(j) {
            return None;
        }
        let mut cur = self.get(par).child;
        while let Some(c) = cur {
            if self.get(c).next == Some(j) {
                return Some(c);
            }
            cur = self.get(c).next;
        }
        panic!("can not find specified jobj. maybe jobj tree is broken.");
    }

    /// `HSD_JObjGetDObj`: the head of the DObj chain.
    pub fn get_dobj(&self, j: Option<JObjId>) -> Option<D> {
        let o = self.get(j?);
        if !o.union_type_dobj() {
            return None;
        }
        o.dobj.first().copied()
    }

    /// `HSD_JObjAddDObj`: prepends.
    pub fn add_dobj(&mut self, j: Option<JObjId>, dobj: Option<D>) {
        let (Some(j), Some(d)) = (j, dobj) else { return };
        if !self.get(j).union_type_dobj() {
            return;
        }
        self.get_mut(j).dobj.insert(0, d);
    }

    /// `HSD_JObjPrependRObj` (the caller passes an RObj that is in no chain).
    pub fn prepend_robj(&mut self, j: Option<JObjId>, robj: Option<R>) {
        let (Some(j), Some(r)) = (j, robj) else { return };
        self.get_mut(j).robj.insert(0, r);
    }

    /// `HSD_JObjDeleteRObj`.
    pub fn delete_robj(&mut self, j: Option<JObjId>, robj: Option<R>) {
        let (Some(j), Some(r)) = (j, robj) else { return };
        let list = &mut self.get_mut(j).robj;
        if let Some(i) = list.iter().position(|x| *x == r) {
            list.remove(i);
        }
    }

    /// `HSD_JObjGetFlags`.
    pub fn get_flags(&self, j: Option<JObjId>) -> u32 {
        j.map_or(0, |j| self.get(j).flags)
    }

    /// `HSD_JObjSetFlags`.
    pub fn set_flags(&mut self, j: Option<JObjId>, flags: u32) {
        let Some(j) = j else { return };
        if (self.get(j).flags ^ flags) & JOBJ_CLASSICAL_SCALE != 0 {
            self.set_mtx_dirty(Some(j));
        }
        self.get_mut(j).flags |= flags;
    }

    /// `HSD_JObjSetFlagsAll`.
    pub fn set_flags_all(&mut self, j: Option<JObjId>, flags: u32) {
        let Some(j) = j else { return };
        self.set_flags(Some(j), flags);
        if self.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = self.get(j).child;
            while let Some(x) = c {
                self.set_flags_all(Some(x), flags);
                c = self.get(x).next;
            }
        }
    }

    /// `HSD_JObjClearFlags`.
    pub fn clear_flags(&mut self, j: Option<JObjId>, flags: u32) {
        let Some(j) = j else { return };
        if (self.get(j).flags ^ flags) & JOBJ_CLASSICAL_SCALE != 0 {
            self.set_mtx_dirty(Some(j));
        }
        self.get_mut(j).flags &= !flags;
    }

    /// `HSD_JObjClearFlagsAll`.
    pub fn clear_flags_all(&mut self, j: Option<JObjId>, flags: u32) {
        let Some(j) = j else { return };
        self.clear_flags(Some(j), flags);
        if self.get(j).flags & JOBJ_INSTANCE == 0 {
            let mut c = self.get(j).child;
            while let Some(x) = c {
                self.clear_flags_all(Some(x), flags);
                c = self.get(x).next;
            }
        }
    }
}
