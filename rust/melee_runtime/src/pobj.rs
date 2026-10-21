//! Port of the logic of `src/sysdolphin/baselib/pobj.c` (the polygon object): flags, the shape-anim
//! entry points (RemoveAnim / AddAnim / ReqAnim / Anim and their `*All` list walkers,
//! `ShapeSetSetAnimResult`, `PObjUpdateFunc`), the matrix-mark statics and the four vertex
//! `decode_*_xyz` helpers.
//!
//! Native shape: `HsdPObj` keeps the fields these functions touch. The C union `u` is reduced to
//! the shape-set member (`shape_set`, `None` = NULL; the C dereferences it unconditionally for a
//! SHAPEANIM PObj, the port panics). The `pobj->next` chain is a slice (`*All` walkers). The shape
//! set's `blend` union keeps both members (`bp` the additive weights, `bl` the single blend); the C
//! overlays them, which only matters if SHAPESET_ADDITIVE flips on a live set (nothing does).
//! The AObj target is `()` (a shape anim's AObjDesc has obj_id 0). GcLayout is delegated to
//! `melee_types::HSD_PObj` / `HSD_ShapeSet` / `HSD_ShapeAnim`.
//!
//! Float rule: no fused site (helper_pobj.ll's fmuladds are all in math_ppc.h / the AObj/FObj
//! code, ported there). `decode_*` divide `(f32) src` by `(f32) scale` (int -> float conversion).
//!
//! Not ported: load / envelope / shape-set alloc, ResolveRefs, vertex-desc setup, shape-anim
//! display-list interpretation and display (GX), the GenoPal skinning path, class functions.
use crate::aobj::{AObjGlobals, HsdAObj, HsdAObjDesc};
use crate::fobj::ObjUpdate;

macro_rules! delegate_layout {
    ($t:ty, $g:ty) => {
        impl melee_core::gc::GcLayout for $t {
            const GC_NAME: &'static str = <$g as melee_core::gc::GcLayout>::GC_NAME;
            const GC_SIZE: u32 = <$g as melee_core::gc::GcLayout>::GC_SIZE;
            const GC_ALIGN: u32 = <$g as melee_core::gc::GcLayout>::GC_ALIGN;
            const GC_FIELDS: &'static [melee_core::gc::GcField] = <$g as melee_core::gc::GcLayout>::GC_FIELDS;
        }
    };
}

/// `POBJ_ANIM`.
pub const POBJ_ANIM: u32 = 1 << 3;
/// `POBJ_SKIN`.
pub const POBJ_SKIN: u16 = 0;
/// `POBJ_SHAPEANIM`.
pub const POBJ_SHAPEANIM: u16 = 1 << 12;
/// `POBJ_ENVELOPE`.
pub const POBJ_ENVELOPE: u16 = 2 << 12;
/// `SHAPESET_ADDITIVE`.
pub const SHAPESET_ADDITIVE: u16 = 1 << 1;
/// `HSD_A_S_W0`.
pub const HSD_A_S_W0: u32 = 2;

/// `HSD_ShapeSet` (the fields the ported logic uses).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdShapeSet {
    pub flags: u16,
    pub nb_shape: u16,
    /// `blend.bp` (SHAPESET_ADDITIVE).
    pub bp: Vec<f32>,
    /// `blend.bl`.
    pub bl: f32,
    pub aobj: Option<HsdAObj<()>>,
}
delegate_layout!(HsdShapeSet, melee_types::HSD_ShapeSet);

/// `HSD_ShapeAnim` (`aobjdesc` `None` = NULL); the `next` chain is a slice.
#[derive(Clone, Debug)]
pub struct HsdShapeAnim {
    pub aobjdesc: Option<HsdAObjDesc>,
}
delegate_layout!(HsdShapeAnim, melee_types::HSD_ShapeAnim);

/// `HSD_PObj` (the fields the ported logic uses).
#[derive(Clone, Debug, PartialEq)]
pub struct HsdPObj {
    pub flags: u16,
    pub n_display: u16,
    /// `u.shape_set` (meaningful only for a SHAPEANIM PObj).
    pub shape_set: Option<HsdShapeSet>,
}
delegate_layout!(HsdPObj, melee_types::HSD_PObj);

impl HsdPObj {
    /// `pobj_type(pobj)`.
    pub fn pobj_type(&self) -> u16 {
        self.flags & 0x3000
    }
    fn shape_set_mut(&mut self) -> &mut HsdShapeSet {
        self.shape_set.as_mut().expect("SHAPEANIM PObj with NULL shape_set")
    }
}

/// `HSD_PObjGetFlags` (NULL -> 0).
pub fn get_flags(pobj: Option<&HsdPObj>) -> u32 {
    pobj.map_or(0, |p| p.flags as u32)
}

/// `HSD_PObjRemoveAnimByFlags`.
pub fn remove_anim_by_flags(pobj: Option<&mut HsdPObj>, flags: u32) {
    let Some(p) = pobj else { return };
    if flags & POBJ_ANIM != 0 && p.pobj_type() == POBJ_SHAPEANIM {
        p.shape_set_mut().aobj = None;
    }
}

/// `HSD_PObjRemoveAnimAllByFlags` over the `next` chain.
pub fn remove_anim_all_by_flags(list: &mut [HsdPObj], flags: u32) {
    for p in list {
        remove_anim_by_flags(Some(p), flags);
    }
}

/// `HSD_PObjAddAnim`. Panics where the C's HSD_ASSERT(247) fires (not SHAPEANIM or NULL set).
pub fn add_anim(pobj: Option<&mut HsdPObj>, shapeanim: Option<&HsdShapeAnim>) {
    let Some(p) = pobj else { return };
    assert!(p.pobj_type() == POBJ_SHAPEANIM && p.shape_set.is_some(), "pobj.c:247");
    let Some(sa) = shapeanim else { return };
    let st = p.shape_set_mut();
    st.aobj = None;
    st.aobj = sa.aobjdesc.as_ref().map(|d| HsdAObj::load_desc(d, |_| None));
}

/// `HSD_PObjAddAnimAll`: walks both chains in step until the PObj chain ends. `shapeanim` empty
/// = NULL. A shorter anim chain passes NULL to AddAnim once, then the C's `next_p(NULL)` step
/// dereferences NULL; the port panics there.
pub fn add_anim_all(list: &mut [HsdPObj], shapeanim: &[HsdShapeAnim]) {
    if list.is_empty() || shapeanim.is_empty() {
        return;
    }
    for (i, p) in list.iter_mut().enumerate() {
        add_anim(Some(p), shapeanim.get(i));
        assert!(i < shapeanim.len(), "HSD_PObjAddAnimAll: next_p(NULL)");
    }
}

/// `HSD_PObjReqAnimByFlags`.
pub fn req_anim_by_flags(pobj: Option<&mut HsdPObj>, startframe: f32, flags: u32) {
    let Some(p) = pobj else { return };
    if flags & POBJ_ANIM != 0 && p.pobj_type() == POBJ_SHAPEANIM {
        if let Some(a) = p.shape_set_mut().aobj.as_mut() {
            a.req_anim(startframe);
        }
    }
}

/// `HSD_PObjReqAnimAllByFlags`.
pub fn req_anim_all_by_flags(list: &mut [HsdPObj], startframe: f32, flags: u32) {
    for p in list {
        req_anim_by_flags(Some(p), startframe, flags);
    }
}

/// `ShapeSetSetAnimResult`. Panics on an out-of-range additive weight index (the C writes out
/// of bounds).
pub fn shape_set_set_anim_result(ss: &mut HsdShapeSet, ty: u32, fv: f32) {
    if ss.flags & SHAPESET_ADDITIVE != 0 {
        ss.bp[ty.wrapping_sub(HSD_A_S_W0) as i32 as isize as usize] = fv;
    } else {
        ss.bl = fv;
    }
}

/// `PObjUpdateFunc`.
pub fn pobj_update_func(pobj: Option<&mut HsdPObj>, ty: u32, fv: f32) {
    let Some(p) = pobj else { return };
    if p.pobj_type() == POBJ_SHAPEANIM {
        shape_set_set_anim_result(p.shape_set_mut(), ty, fv);
    }
}

/// `HSD_PObjAnim`.
pub fn anim(pobj: Option<&mut HsdPObj>, g: &mut AObjGlobals) {
    let Some(p) = pobj else { return };
    if p.pobj_type() != POBJ_SHAPEANIM {
        return;
    }
    let ss = p.shape_set_mut();
    let Some(mut a) = ss.aobj.take() else { return };
    {
        let mut f = |ty: u8, fv: f32| shape_set_set_anim_result(ss, ty as u32, fv);
        let mut up: ObjUpdate<'_> = Some(&mut f);
        a.interpret_anim(g, &mut up);
    }
    ss.aobj = Some(a);
}

/// `HSD_PObjAnimAll`.
pub fn anim_all(list: &mut [HsdPObj], g: &mut AObjGlobals) {
    for p in list {
        anim(Some(p), g);
    }
}

/// The file static `mtx_mark[2]`; `O` is the object handle (`None` = NULL).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MtxMarks<O: Copy> {
    pub obj: [Option<O>; 2],
    pub mark: [u32; 2],
}

impl<O: Copy> MtxMarks<O> {
    /// `HSD_PObjClearMtxMark`.
    pub fn clear(&mut self, obj: Option<O>, mark: u32) {
        for i in 0..2 {
            self.obj[i] = obj;
            self.mark[i] = mark;
        }
    }
    /// `HSD_PObjSetMtxMark`: the C stores only for a negative `idx` (inverted range check), an
    /// out-of-bounds write; the port panics there and is a no-op otherwise, as the C is.
    pub fn set(&mut self, idx: i32, _obj: Option<O>, _mark: u32) {
        if idx >= 2 {
            return;
        }
        assert!(idx >= 0, "HSD_PObjSetMtxMark: out-of-bounds store mtx_mark[{idx}]");
    }
    /// `HSD_PObjGetMtxMark`.
    pub fn get(&self, idx: i32) -> (Option<O>, u32) {
        if !(0..2).contains(&idx) {
            (None, 0)
        } else {
            (self.obj[idx as usize], self.mark[idx as usize])
        }
    }
}

/// `decode_u8_xyz`.
pub fn decode_u8_xyz(src: &[u8], scale: i32) -> [f32; 3] {
    [0, 1, 2].map(|i| src[i] as f32 / scale as f32)
}
/// `decode_s8_xyz`.
pub fn decode_s8_xyz(src: &[u8], scale: i32) -> [f32; 3] {
    [0, 1, 2].map(|i| src[i] as i8 as f32 / scale as f32)
}
/// `decode_u16_xyz` (native little-endian halfwords).
pub fn decode_u16_xyz(src: &[u16], scale: i32) -> [f32; 3] {
    [0, 1, 2].map(|i| src[i] as f32 / scale as f32)
}
/// `decode_s16_xyz` (native little-endian halfwords).
pub fn decode_s16_xyz(src: &[u16], scale: i32) -> [f32; 3] {
    [0, 1, 2].map(|i| src[i] as i16 as f32 / scale as f32)
}
