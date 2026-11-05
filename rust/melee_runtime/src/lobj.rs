//! Port of the logic half of `src/sysdolphin/baselib/lobj.c`: light flags, the active-light table,
//! the current-light list (priority ordered, ref-counted), `LObjUpdateFunc`, the light-ID mapping,
//! colour / spot / attenuation setters and the position / interest WObjs.
//!
//! Native shape: lights live in an arena ([`LObjSystem::objs`], `LObjId` = slot; `None` = a light
//! the class system destroyed). `next` is an arena id. The C statics (`current_lights`,
//! `active_lights`, `nb_active_lights`, the four light masks) are fields of [`LObjSystem`];
//! `current_lights` (an `HSD_SList`) is a `Vec` in list order. `HSD_Obj::ref_count` is kept with
//! the C's `ref_INC` / `ref_DEC` semantics (post-decrement, `0xFFFF` = no-ref); when
//! `HSD_LObjUnrefThis` releases + destroys a light its slot becomes `None` (LObjRelease's AObj
//! remove and WObj unrefs are the drop of the owned fields). The C union `u {point; spot; attn}`
//! is six raw words ([`HsdLObj::u`]) with the C's member offsets as accessors. GcLayout is
//! delegated to `melee_types::HSD_LObj`.
//!
//! Float rule: lobj.c's ported functions have no fused site (`fma_sites.py`); `LObjUpdateFunc`'s
//! colour writes are `255.0 * fv` in f64 converted to `u8` the way gwtool lowers `fptoui ... to
//! i8` (unsigned saturation to 32 bits, then truncation: [`f2u8_unsigned`]); GetLightVector uses
//! the round 1 PSVEC ports.
//!
//! Not ported here: the GX half (HSD_LObjSetup, SetupSpecularInit, the setup_*_lightobj statics,
//! SetupInit), the anim functions (Anim / AnimAll / ReqAnim / ReqAnimAll / AddAnim / AddAnimAll),
//! LObjLoad / LoadDesc / Alloc and the class functions (SetDefaultClass, GetDefaultClass,
//! LObjRelease, Amnesia, InfoInit), Set{Position,Interest}WObj (need `HSD_WObjUnref`'s class
//! release).
use crate::aobj::HsdAObj;
use crate::jobj::{JObjHost, JObjId, JObjTree};
use crate::mtx::{self, Vec3};
use crate::wobj::HsdWObj;

/// `MAX_GXLIGHT`.
pub const MAX_GXLIGHT: usize = 9;
pub const LOBJ_AMBIENT: u32 = 0;
pub const LOBJ_INFINITE: u32 = 1;
pub const LOBJ_POINT: u32 = 2;
pub const LOBJ_SPOT: u32 = 3;
pub const LOBJ_TYPE_MASK: u32 = 3;
pub const LOBJ_HIDDEN: u16 = 1 << 5;
pub const LOBJ_RAW_PARAM: u16 = 1 << 6;
/// `GX_LIGHT_NULL`.
pub const GX_LIGHT_NULL: u32 = 0;
/// `GX_MAX_LIGHT`.
pub const GX_MAX_LIGHT: u32 = 0x100;
/// `HSD_OBJ_NOREF`.
pub const HSD_OBJ_NOREF: u16 = 0xFFFF;

/// `HSD_A_L_*` animation track types.
pub mod track {
    pub const LITC_R: u32 = 9;
    pub const LITC_G: u32 = 10;
    pub const LITC_B: u32 = 11;
    pub const VIS: u32 = 12;
    pub const A0: u32 = 13;
    pub const A1: u32 = 14;
    pub const A2: u32 = 15;
    pub const K0: u32 = 16;
    pub const K1: u32 = 17;
    pub const K2: u32 = 18;
    pub const CUTOFF: u32 = 19;
    pub const REFDIST: u32 = 20;
    pub const REFBRIGHT: u32 = 21;
    pub const LITC_A: u32 = 22;
}

/// Arena slot of a light.
pub type LObjId = usize;

/// `GXColor` (r, g, b, a).
pub type GxColor = [u8; 4];

/// Native `HSD_LObj` (GameCube record `melee_types::HSD_LObj`, 0xD4 bytes). The GX light objects
/// (`lightobj`, `spec_lightobj`) belong to the display half and are not kept.
#[derive(Clone, Debug, PartialEq)]
pub struct HsdLObj<R> {
    /// `parent.ref_count` (0x04).
    pub ref_count: u16,
    /// 0x08.
    pub flags: u16,
    /// 0x0A.
    pub priority: u16,
    /// 0x0C.
    pub next: Option<LObjId>,
    /// 0x10.
    pub color: GxColor,
    /// 0x14.
    pub hw_color: GxColor,
    /// 0x18.
    pub position: Option<HsdWObj<R>>,
    /// 0x1C.
    pub interest: Option<HsdWObj<R>>,
    /// 0x20-0x37: the union `u` as raw words (point / spot / attn).
    pub u: [u32; 6],
    /// 0x38.
    pub shininess: f32,
    /// 0x3C.
    pub lvec: Vec3,
    /// 0x48.
    pub aobj: Option<HsdAObj<JObjId>>,
    /// 0x4C (`GXLightID`).
    pub id: u32,
    /// 0x90 (`GXLightID`).
    pub spec_id: u32,
}

impl<R> melee_core::gc::GcLayout for HsdLObj<R> {
    const GC_NAME: &'static str = <melee_types::HSD_LObj as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_LObj as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_LObj as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_LObj as melee_core::gc::GcLayout>::GC_FIELDS;
}

impl<R> Default for HsdLObj<R> {
    fn default() -> Self {
        HsdLObj {
            ref_count: 0,
            flags: 0,
            priority: 0,
            next: None,
            color: [0; 4],
            hw_color: [0; 4],
            position: None,
            interest: None,
            u: [0; 6],
            shininess: 0.0,
            lvec: Vec3::default(),
            aobj: None,
            id: 0,
            spec_id: 0,
        }
    }
}

fn wf(w: &mut u32, f: f32) {
    *w = f.to_bits();
}

/// gwtool's lowering of `fptoui double to i8`: `llvm.fptoui.sat.i32` then truncate. (This is
/// not `cabi::f2u8`, which saturates signed: the two differ for negative values.)
#[inline(always)]
pub fn f2u8_unsigned(f: f64) -> u8 {
    melee_core::ppc::fctiwz(f) as u8
}

impl<R> HsdLObj<R> {
    /// `HSD_LObjGetType`.
    pub fn get_type(&self) -> u32 {
        self.flags as u32 & LOBJ_TYPE_MASK
    }
    /// `HSD_LObjGetPriority` (header inline; returns the low byte).
    pub fn get_priority(&self) -> u8 {
        self.priority as u8
    }
    /// `HSD_LObjSetColor`.
    pub fn set_lobj_color(&mut self, c: GxColor) {
        self.color = c;
    }
    /// `HSD_LObjGetColor`.
    pub fn get_color(&self) -> GxColor {
        self.color
    }

    /// `LObjUpdateFunc` (the C's NULL check is the caller's `Option`).
    pub fn update_func(&mut self, ty: u32, fv: f32) {
        let raw = self.flags & LOBJ_RAW_PARAM != 0;
        match ty {
            track::VIS => {
                if (fv as f64) < 0.5 {
                    self.flags &= !LOBJ_HIDDEN;
                } else {
                    self.flags |= LOBJ_HIDDEN;
                }
            }
            // attn.a0 / spot.cutoff share word 0; a1 is word 1, spot.ref_dist word 3; a2 is
            // word 2, spot.ref_br word 2.
            track::A0 | track::CUTOFF => wf(&mut self.u[0], fv),
            track::A1 | track::REFDIST => wf(&mut self.u[if raw { 1 } else { 3 }], fv),
            track::A2 | track::REFBRIGHT => wf(&mut self.u[2], fv),
            track::K0 if raw => wf(&mut self.u[3], fv),
            track::K1 if raw => wf(&mut self.u[4], fv),
            track::K2 if raw => wf(&mut self.u[5], fv),
            track::LITC_R => self.color[0] = f2u8_unsigned(255.0 * fv as f64),
            track::LITC_G => self.color[1] = f2u8_unsigned(255.0 * fv as f64),
            track::LITC_B => self.color[2] = f2u8_unsigned(255.0 * fv as f64),
            track::LITC_A => self.color[3] = f2u8_unsigned(255.0 * fv as f64),
            _ => {}
        }
    }

    /// `HSD_LObjSetSpot` (writes `u.point.cutoff` / `point_func`).
    pub fn set_spot(&mut self, cutoff: f32, point_func: i32) {
        wf(&mut self.u[0], cutoff);
        self.u[1] = point_func as u32;
    }
    /// `HSD_LObjSetDistAttn` (writes `u.point.ref_dist` / `ref_br` / `dist_func`).
    pub fn set_dist_attn(&mut self, ref_dist: f32, ref_br: f32, dist_func: i32) {
        wf(&mut self.u[3], ref_dist);
        wf(&mut self.u[2], ref_br);
        self.u[4] = dist_func as u32;
    }
    /// `HSD_LObjSetAttnA`.
    pub fn set_attn_a(&mut self, a0: f32, a1: f32, a2: f32) {
        wf(&mut self.u[0], a0);
        wf(&mut self.u[1], a1);
        wf(&mut self.u[2], a2);
    }
    /// `HSD_LObjSetAttnK`.
    pub fn set_attn_k(&mut self, k0: f32, k1: f32, k2: f32) {
        wf(&mut self.u[3], k0);
        wf(&mut self.u[4], k1);
        wf(&mut self.u[5], k2);
    }
    /// `HSD_LObjSetAttn`.
    pub fn set_attn(&mut self, a0: f32, a1: f32, a2: f32, k0: f32, k1: f32, k2: f32) {
        self.set_attn_a(a0, a1, a2);
        self.set_attn_k(k0, k1, k2);
    }

    /// `HSD_LObjSetPosition` (allocates the WObj on first use).
    pub fn set_position(&mut self, p: &Vec3) {
        self.position.get_or_insert_with(HsdWObj::alloc).set_position(p);
    }
    /// `HSD_LObjSetInterest`.
    pub fn set_interest(&mut self, p: &Vec3) {
        self.interest.get_or_insert_with(HsdWObj::alloc).set_position(p);
    }
    /// `HSD_LObjGetPosition`: `None` (false, `position` untouched) without a position WObj.
    pub fn get_position<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H) -> Option<Vec3>
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
    {
        self.position.as_mut().map(|w| w.get_position(tree, host))
    }
    /// `HSD_LObjGetInterest`.
    pub fn get_interest<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H) -> Option<Vec3>
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
    {
        self.interest.as_mut().map(|w| w.get_position(tree, host))
    }
    /// `HSD_LObjGetLightVector`: interest - position, normalised (a missing WObj reads (0,0,0)).
    pub fn get_light_vector<D, H>(&mut self, tree: &mut JObjTree<D, R>, host: &mut H) -> Vec3
    where
        D: Copy + PartialEq,
        R: Copy + PartialEq,
        H: JObjHost<D, R>,
    {
        let position = self.get_position(tree, host).unwrap_or_default();
        let interest = self.get_interest(tree, host).unwrap_or_default();
        mtx::vec_normalize(&mtx::vec_sub(&interest, &position))
    }
}

/// `HSD_LightID2Index`. Any other ID fails the C's assert (panics here).
pub fn light_id_to_index(id: u32) -> u32 {
    match id {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        0x10 => 4,
        0x20 => 5,
        0x40 => 6,
        0x80 => 7,
        GX_MAX_LIGHT => 8,
        _ => panic!("HSD_LightID2Index: bad light id {id:#x} (lobj.c:1170)"),
    }
}

/// `HSD_Index2LightID`.
pub fn index_to_light_id(index: u32) -> u32 {
    match index {
        0..=7 => <u32 as melee_core::ppc::Ppc>::pshl(1, (index) as i64),
        8 => GX_MAX_LIGHT,
        _ => GX_LIGHT_NULL,
    }
}

/// lobj.c's light arena and file statics.
#[derive(Clone, Debug)]
pub struct LObjSystem<R> {
    pub objs: Vec<Option<HsdLObj<R>>>,
    /// `current_lights` in list order.
    pub current: Vec<LObjId>,
    pub active: [Option<LObjId>; MAX_GXLIGHT],
    pub nb_active: i32,
    pub lightmask_diffuse: i32,
    pub lightmask_specular: i32,
    pub lightmask_attnfunc: i32,
    pub lightmask_alpha: i32,
}

impl<R> Default for LObjSystem<R> {
    fn default() -> Self {
        LObjSystem {
            objs: Vec::new(),
            current: Vec::new(),
            active: [None; MAX_GXLIGHT],
            nb_active: 0,
            lightmask_diffuse: 0,
            lightmask_specular: 0,
            lightmask_attnfunc: 0,
            lightmask_alpha: 0,
        }
    }
}

impl<R> LObjSystem<R> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds a light to the arena (the native stand-in for `HSD_LObjAlloc`'s object memory).
    pub fn insert(&mut self, l: HsdLObj<R>) -> LObjId {
        self.objs.push(Some(l));
        self.objs.len() - 1
    }
    pub fn get(&self, id: LObjId) -> &HsdLObj<R> {
        self.objs[id].as_ref().expect("destroyed LObj")
    }
    pub fn get_mut(&mut self, id: LObjId) -> &mut HsdLObj<R> {
        self.objs[id].as_mut().expect("destroyed LObj")
    }

    /// `HSD_LObjGetFlags`.
    pub fn get_flags(&self, l: Option<LObjId>) -> u32 {
        l.map_or(0, |l| self.get(l).flags as u32)
    }
    /// `HSD_LObjSetFlags` (the C's `u16 |= u32` keeps the low half).
    pub fn set_flags(&mut self, l: Option<LObjId>, flags: u32) {
        if let Some(l) = l {
            self.get_mut(l).flags |= flags as u16;
        }
    }
    /// `HSD_LObjClearFlags`.
    pub fn clear_flags(&mut self, l: Option<LObjId>, flags: u32) {
        if let Some(l) = l {
            self.get_mut(l).flags &= !flags as u16;
        }
    }
    /// `HSD_LObjGetLightMaskDiffuse`.
    pub fn get_light_mask_diffuse(&self) -> i32 {
        self.lightmask_diffuse
    }
    /// `HSD_LObjGetLightMaskAttnFunc`.
    pub fn get_light_mask_attn_func(&self) -> i32 {
        self.lightmask_attnfunc
    }
    /// `HSD_LObjGetLightMaskAlpha`.
    pub fn get_light_mask_alpha(&self) -> i32 {
        self.lightmask_alpha
    }
    /// `HSD_LObjGetLightMaskSpecular`.
    pub fn get_light_mask_specular(&self) -> i32 {
        self.lightmask_specular
    }

    /// `HSD_LObjSetActive`. A non-ambient light past the ninth writes out of bounds in the C;
    /// here it panics.
    pub fn set_active(&mut self, l: LObjId) {
        let idx = if self.get(l).get_type() == LOBJ_AMBIENT {
            let idx = MAX_GXLIGHT - 1;
            if self.active[idx].is_some() {
                return;
            }
            idx
        } else {
            let idx = self.nb_active;
            self.nb_active += 1;
            idx as usize
        };
        self.active[idx] = Some(l);
        self.get_mut(l).id = index_to_light_id(idx as u32);
    }
    /// `HSD_LObjGetNbActive`.
    pub fn get_nb_active(&self) -> i32 {
        self.nb_active
    }
    /// `HSD_LObjGetActiveByID`.
    pub fn get_active_by_id(&self, id: u32) -> Option<LObjId> {
        let idx = light_id_to_index(id) as i32;
        if (0..MAX_GXLIGHT as i32).contains(&idx) {
            self.active[idx as usize]
        } else {
            None
        }
    }
    /// `HSD_LObjGetActiveByIndex` (the ambient slot 8 is not reachable).
    pub fn get_active_by_index(&self, idx: i32) -> Option<LObjId> {
        if (0..MAX_GXLIGHT as i32 - 1).contains(&idx) {
            self.active[idx as usize]
        } else {
            None
        }
    }
    /// `HSD_LObjClearActive`.
    pub fn clear_active(&mut self) {
        self.active = [None; MAX_GXLIGHT];
        self.nb_active = 0;
    }

    /// `LObjUpdateFunc(lobj, type, val)`.
    pub fn update_func(&mut self, l: Option<LObjId>, ty: u32, fv: f32) {
        if let Some(l) = l {
            self.get_mut(l).update_func(ty, fv);
        }
    }

    /// `ref_INC`.
    fn ref_inc(&mut self, l: LObjId) {
        let o = self.get_mut(l);
        o.ref_count = o.ref_count.wrapping_add(1);
        assert!(o.ref_count != HSD_OBJ_NOREF, "ref_INC: object.h:93");
    }
    /// `ref_DEC`.
    fn ref_dec(&mut self, l: LObjId) -> bool {
        let o = self.get_mut(l);
        if o.ref_count == HSD_OBJ_NOREF {
            return true;
        }
        let r = o.ref_count == 0;
        o.ref_count = o.ref_count.wrapping_sub(1);
        r
    }

    /// `HSD_LObjUnrefThis`: release + destroy when the count runs out (the slot becomes `None`).
    pub fn unref_this(&mut self, l: Option<LObjId>) {
        if let Some(l) = l {
            if self.ref_dec(l) {
                self.objs[l] = None;
            }
        }
    }

    /// `HSD_LObjAddCurrent`: (re)inserts before the first light of higher priority.
    pub fn add_current(&mut self, l: Option<LObjId>) {
        let Some(l) = l else { return };
        if self.current.contains(&l) {
            self.delete_current(Some(l));
        }
        self.ref_inc(l);
        let pri = self.get(l).get_priority();
        let pos = self
            .current
            .iter()
            .position(|&c| self.get(c).get_priority() > pri)
            .unwrap_or(self.current.len());
        self.current.insert(pos, l);
    }

    /// `HSD_LObjDeleteCurrent`.
    pub fn delete_current(&mut self, l: Option<LObjId>) {
        let Some(l) = l else { return };
        if let Some(pos) = self.current.iter().position(|&c| c == l) {
            for a in self.active.iter_mut() {
                if *a == Some(l) {
                    *a = None;
                }
            }
            self.current.remove(pos);
            self.unref_this(Some(l));
        }
    }

    /// The static inline `LObjRemoveAll`.
    fn remove_all_current(&mut self) {
        self.clear_active();
        while let Some(&c) = self.current.first() {
            self.unref_this(Some(c));
            self.current.remove(0);
        }
    }

    /// `HSD_LObjDeleteCurrentAll`: the chain from `l`, or every current light for NULL.
    pub fn delete_current_all(&mut self, l: Option<LObjId>) {
        if l.is_some() {
            let mut cur = l;
            while let Some(c) = cur {
                self.delete_current(Some(c));
                cur = self.get(c).next;
            }
            return;
        }
        self.remove_all_current();
    }

    /// `HSD_LObjSetCurrentAll` (adds `l` only, not its chain).
    pub fn set_current_all(&mut self, l: Option<LObjId>) {
        self.remove_all_current();
        self.add_current(l);
    }

    /// `HSD_LObj_803668EC` (the static inline `LObjReplaceAll`: adds the whole chain).
    pub fn replace_all(&mut self, l: Option<LObjId>) {
        self.remove_all_current();
        let mut cur = l;
        while let Some(c) = cur {
            self.add_current(Some(c));
            cur = self.get(c).next;
        }
    }

    /// `HSD_LObjGetCurrentByType`.
    pub fn get_current_by_type(&self, flags: u16) -> Option<LObjId> {
        let ty = flags as u32 & LOBJ_TYPE_MASK;
        self.current.iter().copied().find(|&c| self.get(c).get_type() == ty)
    }

    /// `HSD_LObjRemoveAll`: deletes from the current list and unrefs every light of the chain.
    pub fn remove_all(&mut self, l: Option<LObjId>) {
        let mut cur = l;
        while let Some(c) = cur {
            let next = self.get(c).next;
            self.delete_current(Some(c));
            self.unref_this(Some(c));
            cur = next;
        }
    }

    /// `HSD_LObjGetNext` (header inline).
    pub fn get_next(&self, l: Option<LObjId>) -> Option<LObjId> {
        l.and_then(|l| self.get(l).next)
    }
    /// `HSD_LObjGetPositionWObj`.
    pub fn get_position_wobj(&self, l: Option<LObjId>) -> Option<&HsdWObj<R>> {
        l.and_then(|l| self.get(l).position.as_ref())
    }
    /// `HSD_LObjGetInterestWObj`.
    pub fn get_interest_wobj(&self, l: Option<LObjId>) -> Option<&HsdWObj<R>> {
        l.and_then(|l| self.get(l).interest.as_ref())
    }
}
