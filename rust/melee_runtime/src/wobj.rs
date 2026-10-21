//! Port of `src/sysdolphin/baselib/wobj.c`: the world object (a position that a camera eye or
//! interest, or a light, points at, optionally driven by an AObj whose target JObj carries a
//! path spline).
//!
//! Native shape: `HsdWObj<R>` owns its AObj (`Option<HsdAObj<JObjId>>`, the target JObj lives
//! in a [`JObjTree`]) and its RObj chain as `Vec<R>`. Everything wobj.c calls that is not ported
//! yet (RObj, the class system, the `HSD_JObj.u.spline` member that the native JObj does not keep)
//! goes through [`WObjHost`].
//!
//! Float rule: `fma_sites.py` finds no fused site in wobj.c. The position updates go through
//! `MTXMultVec` (`mtx::mult_vec`, ported with its own sites) and `splArcLengthPoint`
//! (`spline::arc_length_point`).
//!
//! Not ported: `HSD_WObjSetDefaultClass`, `WObjRelease`, `WObjAmnesia`, `WObjInfoInit` (class
//! system; dropping the native object is the release), `HSD_WObjUnref` (refcount).
use crate::aobj::{AObjGlobals, HsdAObj, HsdAObjDesc};
use crate::jobj::{JObjHost, JObjId, JObjTree};
use crate::mtx::{self, Vec3};
use crate::spline::{self, Spline};

/// WObj flag bit 0: `pos` is in the AObj target JObj's local space (set by the path update).
pub const WOBJ_POS_LOCAL: u32 = 1;
/// WObj flag bit 1: position changed.
pub const WOBJ_POS_DIRTY: u32 = 2;

/// Native `HSD_WObj` (GameCube record `melee_types::HSD_WObj`, size 0x24).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdWObj<R> {
    pub flags: u32,
    pub pos: Vec3,
    pub aobj: Option<HsdAObj<JObjId>>,
    pub robj: Vec<R>,
}

impl<R> melee_core::gc::GcLayout for HsdWObj<R> {
    const GC_NAME: &'static str = <melee_types::HSD_WObj as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_WObj as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_WObj as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_WObj as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// Native `HSD_WObjDesc` (`class_name` is dropped: the class system is not ported).
#[derive(Clone, Debug)]
pub struct HsdWObjDesc<RD> {
    pub pos: Vec3,
    pub robjdesc: Option<RD>,
}

/// Native `HSD_WObjAnim`.
#[derive(Clone, Debug)]
pub struct HsdWObjAnim<RA> {
    pub aobjdesc: Option<HsdAObjDesc>,
    pub robjanim: Option<RA>,
}

/// The not-yet-ported code wobj.c calls (on top of what jobj.c needs).
pub trait WObjHost<D, R>: JObjHost<D, R> {
    type RObjDesc;
    type RObjAnim;
    /// `jp->u.spline` of the AObj target. TODO(group1 jobj): the native JObj keeps only the
    /// DObj member of the union.
    fn jobj_spline(&self, j: JObjId) -> Option<Spline<'_>>;
    /// `HSD_RObjRemoveAnimAll`. TODO(group1 robj).
    fn robj_remove_anim_all(&mut self, robj: &mut [R]);
    /// `HSD_RObjReqAnimAll`. TODO(group1 robj).
    fn robj_req_anim_all(&mut self, robj: &mut [R], frame: f32);
    /// `HSD_RObjAddAnimAll`. TODO(group1 robj).
    fn robj_add_anim_all(&mut self, robj: &mut [R], anim: Option<&Self::RObjAnim>);
    /// `HSD_RObjAnimAll`. TODO(group1 robj).
    fn robj_anim_all(&mut self, robj: &mut [R]);
    /// `HSD_RObjLoadDesc` + `HSD_RObjResolveRefsAll`. TODO(group1 robj).
    fn robj_load_desc(&mut self, desc: Option<&Self::RObjDesc>) -> Vec<R>;
    /// `HSD_AObjLoadDesc`'s `obj_id != 0` lookup. TODO(group1 id/jobj).
    fn resolve_aobj_obj(&mut self, obj_id: u32) -> Option<JObjId>;
}

/// The local-to-world flush shared by SetPositionX/Y/Z and GetPosition: if `pos` is local to
/// the AObj target, bring it to world space through the target's matrix and clear bit 0.
fn flush_local<D, R, H>(
    flags: &mut u32,
    pos: &mut Vec3,
    target: Option<Option<JObjId>>,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
) where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    if *flags & WOBJ_POS_LOCAL != 0 {
        if let Some(Some(jp)) = target {
            tree.setup_matrix(host, Some(jp));
            *pos = mtx::mult_vec(&tree.get(jp).mtx, pos);
        }
        *flags &= 0xFFFF_FFFE;
    }
}

fn set_position_raw(flags: &mut u32, pos: &mut Vec3, p: &Vec3) {
    *pos = *p;
    *flags |= WOBJ_POS_DIRTY;
    *flags &= 0xFFFF_FFFE;
}

fn set_axis<D, R, H>(
    flags: &mut u32,
    pos: &mut Vec3,
    target: Option<Option<JObjId>>,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
    axis: usize,
    val: f32,
) where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: JObjHost<D, R>,
{
    flush_local(flags, pos, target, tree, host);
    match axis {
        0 => pos.x = val,
        1 => pos.y = val,
        _ => pos.z = val,
    }
    *flags |= WOBJ_POS_DIRTY;
}

/// `WObjUpdateFunc` on split-out fields (so it can run while the AObj is borrowed).
fn update_func<D, R, H>(
    flags: &mut u32,
    pos: &mut Vec3,
    target: Option<Option<JObjId>>,
    tree: &mut JObjTree<D, R>,
    host: &mut H,
    ty: u8,
    fv: f32,
) where
    D: Copy + PartialEq,
    R: Copy + PartialEq,
    H: WObjHost<D, R>,
{
    match ty {
        4 => {
            let mut fv = fv;
            if (fv as f64) < 0.0 {
                fv = 0.0;
            }
            if 1.0 < fv as f64 {
                fv = 1.0;
            }
            let target = target.expect("HSD_ASSERT(148, wobj->aobj)");
            let jp = target.expect("HSD_ASSERT(150, jp)");
            let spl = host.jobj_spline(jp).expect("HSD_ASSERT(151, jp->u.spline)");
            // `p` is a zero-initialised local in the shipping build; splArcLengthPoint leaves it
            // untouched where get_spline_point has no answer.
            let p = spline::arc_length_point(&spl, fv).unwrap_or_default();
            set_position_raw(flags, pos, &p);
            *flags |= WOBJ_POS_LOCAL;
        }
        5 => set_axis(flags, pos, target, tree, host, 0, fv),
        6 => set_axis(flags, pos, target, tree, host, 1, fv),
        7 => set_axis(flags, pos, target, tree, host, 2, fv),
        _ => {}
    }
}

impl<R> Default for HsdWObj<R> {
    fn default() -> Self {
        Self::alloc()
    }
}

impl<R> HsdWObj<R> {
    /// `HSD_WObjAlloc` (zeroed object; the class pointer is not kept).
    pub fn alloc() -> Self {
        HsdWObj { flags: 0, pos: Vec3::default(), aobj: None, robj: Vec::new() }
    }

    fn target(&self) -> Option<Option<JObjId>> {
        self.aobj.as_ref().map(|a| a.hsd_obj)
    }

    /// `HSD_WObjClearFlags` (header inline).
    pub fn clear_flags(&mut self, flags: u32) {
        self.flags &= !flags;
    }

    /// `HSD_WObjSetPosition`.
    pub fn set_position(&mut self, p: &Vec3) {
        set_position_raw(&mut self.flags, &mut self.pos, p);
    }

    /// `HSD_WObjSetPositionX`.
    pub fn set_position_x<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H, v: f32)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
    {
        let t = self.target();
        set_axis(&mut self.flags, &mut self.pos, t, tree, host, 0, v);
    }

    /// `HSD_WObjSetPositionY`.
    pub fn set_position_y<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H, v: f32)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
    {
        let t = self.target();
        set_axis(&mut self.flags, &mut self.pos, t, tree, host, 1, v);
    }

    /// `HSD_WObjSetPositionZ`.
    pub fn set_position_z<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H, v: f32)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
    {
        let t = self.target();
        set_axis(&mut self.flags, &mut self.pos, t, tree, host, 2, v);
    }

    /// `HSD_WObjGetPosition`.
    pub fn get_position<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H) -> Vec3
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
    {
        let t = self.target();
        flush_local(&mut self.flags, &mut self.pos, t, tree, host);
        self.pos
    }

    /// `WObjUpdateFunc(wobj, type, val)` (static in the C; exposed for the oracle).
    pub fn update_func<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H, ty: u8, fv: f32)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: WObjHost<D, R>,
    {
        let t = self.target();
        update_func(&mut self.flags, &mut self.pos, t, tree, host, ty, fv);
    }

    /// `HSD_WObjRemoveAnim`.
    pub fn remove_anim<D, H>(&mut self, host: &mut H)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: WObjHost<D, R>,
    {
        self.aobj = None; // HSD_AObjRemove
        host.robj_remove_anim_all(&mut self.robj);
    }

    /// `HSD_WObjReqAnim`.
    pub fn req_anim<D, H>(&mut self, host: &mut H, frame: f32)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: WObjHost<D, R>,
    {
        if let Some(a) = self.aobj.as_mut() {
            a.req_anim(frame);
        }
        host.robj_req_anim_all(&mut self.robj, frame);
    }

    /// `HSD_WObjAddAnim` (`anim == NULL` is the caller not calling).
    pub fn add_anim<D, H>(&mut self, host: &mut H, anim: &HsdWObjAnim<H::RObjAnim>)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: WObjHost<D, R>,
    {
        self.aobj = None; // HSD_AObjRemove of the old one
        self.aobj = anim
            .aobjdesc
            .as_ref()
            .map(|d| HsdAObj::load_desc(d, |id| host.resolve_aobj_obj(id)));
        host.robj_add_anim_all(&mut self.robj, anim.robjanim.as_ref());
    }

    /// `HSD_WObjInterpretAnim`.
    pub fn interpret_anim<D, H>(&mut self, g: &mut AObjGlobals, tree: &mut JObjTree<D, R>, host: &mut H)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: WObjHost<D, R>,
    {
        if let Some(mut a) = self.aobj.take() {
            let t = Some(a.hsd_obj);
            {
                let flags = &mut self.flags;
                let pos = &mut self.pos;
                let mut f = |ty: u8, fv: f32| update_func(flags, pos, t, tree, host, ty, fv);
                a.interpret_anim(g, &mut Some(&mut f));
            }
            self.aobj = Some(a);
        }
        host.robj_anim_all(&mut self.robj);
    }

    /// `WObjLoad` / `HSD_WObjInit` (identical bodies).
    pub fn init<D, H>(&mut self, host: &mut H, desc: &HsdWObjDesc<H::RObjDesc>)
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: WObjHost<D, R>,
    {
        self.set_position(&desc.pos);
        self.robj.clear(); // HSD_RObjRemoveAll
        self.robj = host.robj_load_desc(desc.robjdesc.as_ref());
    }

    /// `HSD_WObjLoadDesc` (default class only).
    pub fn load_desc<D, H>(host: &mut H, desc: &HsdWObjDesc<H::RObjDesc>) -> Self
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: WObjHost<D, R>,
    {
        let mut w = Self::alloc();
        w.init(host, desc);
        w
    }
}
