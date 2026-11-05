//! Complete native generated-record interface for `src/sysdolphin/baselib/lobj.c`.
//! The exact C-name exports below cover lights, animation dispatch, class loading/lifecycle,
//! active/current lists and GX setup. Cross-TU native dependencies live in `ext`.
//!
//! The pre-existing `HsdRender` DAT adapter is retained for renderer compatibility; that
//! adapter still approximates spot lights and attenuation. Use the generated-record
//! interface for complete TU semantics and native object ownership.
use super::state::{gx::*, ActiveLight, Dat, GxBackend, HsdRender, LightLoad, Rgba};
use crate::mtx::{self, Mtx, Vec3};

pub const LOBJ_AMBIENT: u32 = 0;
pub const LOBJ_INFINITE: u32 = 1;
pub const LOBJ_POINT: u32 = 2;
pub const LOBJ_SPOT: u32 = 3;
pub const LOBJ_DIFFUSE: u32 = 1 << 2;
pub const LOBJ_SPECULAR: u32 = 1 << 3;
pub const LOBJ_ALPHA: u32 = 1 << 4;
pub const LOBJ_HIDDEN: u32 = 1 << 5;

/// `HSD_LObjGetPosition` / `GetInterest` of a descriptor (`HSD_WObjDesc.pos` at +4).
fn wobj_pos(d: &Dat, w: Option<u32>, default: Vec3) -> Vec3 {
    match w {
        Some(w) => Vec3 {
            x: d.f32_(w + 4),
            y: d.f32_(w + 8),
            z: d.f32_(w + 0xC),
        },
        None => default,
    }
}

/// One light of a set: its descriptor plus the (possibly animated) flags and colour.
#[derive(Clone, Copy, Debug)]
pub struct LightIn {
    pub desc: u32,
    pub flags: u32,
    pub color: Rgba,
}

impl HsdRender {
    /// `HSD_LObjSetupInit` over `descs` (each an `HSD_LightDesc` offset in `d`) under the viewing
    /// matrix `vmtx`: assigns GX_LIGHT0.. in order, loads every diffuse light, rebuilds the masks.
    pub fn lobj_setup_init<G: GxBackend>(
        &mut self,
        g: &mut G,
        d: &Dat,
        descs: &[u32],
        vmtx: &Mtx,
    ) {
        let lights: Vec<LightIn> = descs
            .iter()
            .map(|&desc| LightIn { desc, flags: d.u16_(desc + 8) as u32, color: d.color(desc + 0xC) })
            .collect();
        self.lobj_setup_init_with(g, d, &lights, vmtx)
    }

    /// [`Self::lobj_setup_init`] over lights whose flags / colour were animated (`HSD_LObjAnim`:
    /// the `VIS` and `LITC_*` tracks).
    pub fn lobj_setup_init_with<G: GxBackend>(
        &mut self,
        g: &mut G,
        d: &Dat,
        lights: &[LightIn],
        vmtx: &Mtx,
    ) {
        self.lights.clear();
        self.lightmask_diffuse = 0;
        self.lightmask_specular = 0;
        self.lightmask_alpha = 0;
        self.lightmask_attnfunc = 0;
        let mut n = 0u32;
        for li in lights {
            let ld = li.desc;
            let (flags, color) = (li.flags, li.color);
            if flags & LOBJ_HIDDEN != 0 {
                continue;
            }
            let ty = flags & 3;
            if ty == LOBJ_AMBIENT {
                self.lights.push(ActiveLight {
                    flags,
                    color,
                    id: 0,
                });
                continue;
            }
            if n >= 8 {
                continue;
            }
            let id = <u32 as melee_core::ppc::Ppc>::pshl(1u32, (n) as i64);
            n += 1;
            let pos = wobj_pos(
                d,
                d.ptr(ld + 0x10),
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                },
            );
            let mut l = LightLoad {
                color,
                pos: [0.0; 3],
                dir: [0.0, 0.0, -1.0],
                attn: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            };
            if flags & LOBJ_DIFFUSE != 0 {
                // setup_diffuse_lightobj
                self.lightmask_diffuse |= id;
                if ty != LOBJ_INFINITE {
                    self.lightmask_attnfunc |= id;
                }
                if flags & LOBJ_ALPHA != 0 {
                    self.lightmask_alpha |= id;
                }
            }
            match ty {
                LOBJ_INFINITE => {
                    // setup_infinite_lightobj: the position pushed 2^20 out, into view space.
                    let p = Vec3 {
                        x: pos.x * 1048576.0,
                        y: pos.y * 1048576.0,
                        z: pos.z * 1048576.0,
                    };
                    let p = mtx::mult_vec(vmtx, &p);
                    l.pos = [p.x, p.y, p.z];
                }
                _ => {
                    // setup_point_lightobj (attenuation from the desc's union is not modelled:
                    // k0 = 1, no distance falloff).
                    let p = mtx::mult_vec(vmtx, &pos);
                    l.pos = [p.x, p.y, p.z];
                }
            }
            if flags & LOBJ_DIFFUSE != 0 {
                g.load_light(id, &l);
            }
            self.lights.push(ActiveLight { flags, color, id });
        }
    }

    /// `HSD_LObjSetup`: the diffuse light is loaded once by [`Self::lobj_setup_init`]; with no
    /// colour change (the C passes `lobj->color`) and no specular light object, nothing is left.
    pub fn lobj_setup<G: GxBackend>(&mut self, _g: &mut G, _l: &ActiveLight) {}
}

/// The stage light set (gr/ground.c `Ground_801C466C`): the first model group
/// (`map_head->unk8[i]`, 0x34 bytes) with a `x18` light set, a NULL-terminated `LightList*[]` of
/// `{ HSD_LightDesc* desc; HSD_LightAnim** anims; }`. APPROXIMATION: the C picks the group whose
/// stage callback has `flags_b0` and applies `Ground_801C20E0`'s per-light overrides (DIFFUSE /
/// SPECULAR / 0x400 bits from the stage's override table); neither is modelled here.
pub fn stage_light_descs(d: &Dat, map_head: u32) -> Vec<u32> {
    melee_core::ct!("stage_light_descs");
    let mut v = Vec::new();
    let Some(groups) = d.ptr(map_head + 8) else {
        return v;
    };
    let count = d.u32_(map_head + 0xC).min(64);
    let Some(list) = (0..count).find_map(|i| d.ptr(groups + i * 0x34 + 0x18))
    else {
        return v;
    };
    for i in 0..16 {
        let Some(ll) = d.ptr(list + 4 * i) else { break };
        if let Some(desc) = d.ptr(ll) {
            v.push(desc);
        }
    }
    v
}

#[allow(dead_code)]
fn unused_mtx_color(_: Mtx) -> u32 {
    GX_COLOR0
}

/// Existing arena interface, retained for hosts already using arena ownership.
pub use crate::lobj::{HsdLObj, LObjSystem};
/// Native generated-record interface. The renderer adapter above remains source compatible.
/// Unlike its DAT-only approximation, this interface implements the complete C TU.
pub use native::*;
#[allow(non_snake_case, non_upper_case_globals)]
pub mod native {
    use crate::mtx::{self, Mtx, Vec3};
    use core::{ffi::c_void, ptr::null_mut};
    use melee_types::{
        HSD_CObj, HSD_LObj, HSD_LObjInfo, HSD_LightAnim, HSD_LightDesc,
        HSD_ObjData, HSD_SList, HSD_WObj, _GXColor, _HSD_Class,
        _HSD_ClassInfo,
    };
    use std::cell::Cell;
    /// Process-wide in the engine (these are C globals: a rollback restores the executable's statics, a
    /// thread-local would escape it); per-thread under `cargo test`, where the oracle tests run in parallel.
    pub struct Gl<T: 'static>(Cell<T>);
    unsafe impl<T> Sync for Gl<T> {}
    impl<T> Gl<T> {
        pub const fn new(v: T) -> Self { Gl(Cell::new(v)) }
        pub fn with<R>(&'static self, f: impl FnOnce(&Cell<T>) -> R) -> R { f(&self.0) }
    }
    #[cfg(not(test))]
    static CLASS: Gl<*mut HSD_LObjInfo> = Gl::new(null_mut());
    #[cfg(not(test))]
    static DEFAULT: Gl<*mut HSD_LObjInfo> = Gl::new(null_mut());
    #[cfg(not(test))]
    static CURRENT: Gl<*mut HSD_SList> = Gl::new(null_mut());
    #[cfg(not(test))]
    static ACTIVE: Gl<[*mut HSD_LObj; 9]> = Gl::new([null_mut(); 9]);
    #[cfg(not(test))]
    static COUNT: Gl<i32> = Gl::new(0);
    #[cfg(not(test))]
    static MASK: Gl<[i32; 4]> = Gl::new([0; 4]);
    #[cfg(test)]
    thread_local! {
        static CLASS: Cell<*mut HSD_LObjInfo> = Cell::new({let mut c=HSD_LObjInfo::zeroed();c.parent.parent.head.info_init=Some(LObjInfoInit);Box::into_raw(Box::new(c))});
        static DEFAULT: Cell<*mut HSD_LObjInfo> = const {Cell::new(null_mut())};
        static CURRENT: Cell<*mut HSD_SList> = const {Cell::new(null_mut())};
        static ACTIVE: Cell<[*mut HSD_LObj;9]> = const {Cell::new([null_mut();9])};
        static COUNT: Cell<i32> = const {Cell::new(0)};
        static MASK: Cell<[i32;4]> = const {Cell::new([0;4])};
    }
    /// Class/static accessors used by hosts and oracle; pointers remain stable on this thread.
    pub fn hsdLObj() -> *mut HSD_LObjInfo {
        #[cfg(not(test))]
        if CLASS.with(Cell::get).is_null() {
            let mut c = HSD_LObjInfo::zeroed();
            c.parent.parent.head.info_init = Some(LObjInfoInit);
            CLASS.with(|x| x.set(Box::into_raw(Box::new(c))));
        }
        CLASS.with(Cell::get)
    }
    pub fn current_lights() -> std::vec::Vec<*mut HSD_LObj> {
        let mut v = std::vec::Vec::new();
        let mut p = CURRENT.with(Cell::get);
        unsafe {
            while !p.is_null() {
                v.push((*p).data.cast());
                p = (*p).next;
            }
        }
        v
    }
    fn mask(i: usize, id: i32) {
        MASK.with(|x| {
            let mut m = x.get();
            m[i] |= id;
            x.set(m)
        })
    }
    pub fn HSD_LObjGetLightMaskDiffuse() -> i32 {
        MASK.with(|x| x.get()[0])
    }
    pub fn HSD_LObjGetLightMaskSpecular() -> i32 {
        MASK.with(|x| x.get()[1])
    }
    pub fn HSD_LObjGetLightMaskAttnFunc() -> i32 {
        MASK.with(|x| x.get()[2])
    }
    pub fn HSD_LObjGetLightMaskAlpha() -> i32 {
        MASK.with(|x| x.get()[3])
    }
    pub use crate::lobj::light_id_to_index as HSD_LightID2Index;
    pub fn HSD_Index2LightID(i: u32) -> i32 {
        crate::lobj::index_to_light_id(i) as i32
    }
    pub unsafe fn HSD_LObjGetFlags(l: *mut HSD_LObj) -> u32 {
        if l.is_null() {
            0
        } else {
            (*l).flags as u32
        }
    }
    pub unsafe fn HSD_LObjSetFlags(l: *mut HSD_LObj, f: u32) {
        if !l.is_null() {
            (*l).flags |= f as u16;
        }
    }
    pub unsafe fn HSD_LObjClearFlags(l: *mut HSD_LObj, f: u32) {
        if !l.is_null() {
            (*l).flags &= !(f as u16);
        }
    }
    pub unsafe fn HSD_LObjGetType(l: *mut HSD_LObj) -> u32 {
        (*l).flags as u32 & 3
    }
    pub unsafe fn HSD_LObjSetActive(l: *mut HSD_LObj) {
        let mut a = ACTIVE.with(Cell::get);
        let i = if HSD_LObjGetType(l) == 0 {
            if !a[8].is_null() {
                return;
            }
            8
        } else {
            COUNT.with(|c| {
                let i = c.get();
                c.set(i + 1);
                i as usize
            })
        };
        a[i] = l;
        ACTIVE.with(|c| c.set(a));
        (*l).id = HSD_Index2LightID(i as u32) as u32;
    }
    pub fn HSD_LObjGetNbActive() -> i32 {
        COUNT.with(Cell::get)
    }
    pub fn HSD_LObjGetActiveByID(id: u32) -> *mut HSD_LObj {
        ACTIVE.with(|a| a.get()[HSD_LightID2Index(id) as usize])
    }
    pub fn HSD_LObjGetActiveByIndex(i: i32) -> *mut HSD_LObj {
        if (0..8).contains(&i) {
            ACTIVE.with(|a| a.get()[i as usize])
        } else {
            null_mut()
        }
    }
    pub fn HSD_LObjClearActive() {
        ACTIVE.with(|a| a.set([null_mut(); 9]));
        COUNT.with(|c| c.set(0));
    }
    // Reuse the already-oracle-verified arena port's scalar union/color/flag operations.
    unsafe fn scalar(
        l: *mut HSD_LObj,
        f: impl FnOnce(&mut crate::lobj::HsdLObj<()>),
    ) {
        if l.is_null() {
            return;
        }
        let mut a = crate::lobj::HsdLObj::default();
        a.flags = (*l).flags;
        a.color = [(*l).color.r, (*l).color.g, (*l).color.b, (*l).color.a];
        a.u = core::mem::transmute((*l).u);
        f(&mut a);
        (*l).flags = a.flags;
        (*l).color = _GXColor {
            r: a.color[0],
            g: a.color[1],
            b: a.color[2],
            a: a.color[3],
        };
        (*l).u = core::mem::transmute(a.u);
    }
    pub unsafe extern "C" fn LObjUpdateFunc(
        o: *mut c_void,
        t: i32,
        v: *mut HSD_ObjData,
    ) {
        if !o.is_null() {
            scalar(o.cast(), |a| a.update_func(t as u32, (*v).fv));
        }
    }
    pub unsafe fn HSD_LObjSetColor(l: *mut HSD_LObj, c: _GXColor) {
        (*l).color = c;
    }
    pub unsafe fn HSD_LObjGetColor(l: *mut HSD_LObj, c: *mut _GXColor) {
        *c = (*l).color;
    }
    pub unsafe fn HSD_LObjSetSpot(l: *mut HSD_LObj, c: f32, f: i32) {
        scalar(l, |a| a.set_spot(c, f));
    }
    pub unsafe fn HSD_LObjSetDistAttn(
        l: *mut HSD_LObj,
        d: f32,
        b: f32,
        f: i32,
    ) {
        scalar(l, |a| a.set_dist_attn(d, b, f));
    }
    pub unsafe fn HSD_LObjSetAttnA(l: *mut HSD_LObj, a: f32, b: f32, c: f32) {
        scalar(l, |x| x.set_attn_a(a, b, c));
    }
    pub unsafe fn HSD_LObjSetAttnK(l: *mut HSD_LObj, a: f32, b: f32, c: f32) {
        scalar(l, |x| x.set_attn_k(a, b, c));
    }
    pub unsafe fn HSD_LObjSetAttn(
        l: *mut HSD_LObj,
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        e: f32,
        f: f32,
    ) {
        scalar(l, |x| x.set_attn(a, b, c, d, e, f));
    }
    pub unsafe fn HSD_LObjGetPositionWObj(l: *mut HSD_LObj) -> *mut HSD_WObj {
        if l.is_null() {
            null_mut()
        } else {
            (*l).position
        }
    }
    pub unsafe fn HSD_LObjGetInterestWObj(l: *mut HSD_LObj) -> *mut HSD_WObj {
        if l.is_null() {
            null_mut()
        } else {
            (*l).interest
        }
    }
    pub unsafe fn HSD_LObjSetPositionWObj(l: *mut HSD_LObj, w: *mut HSD_WObj) {
        if !l.is_null() {
            ext::HSD_WObjUnref((*l).position);
            (*l).position = w;
        }
    }
    pub unsafe fn HSD_LObjSetInterestWObj(l: *mut HSD_LObj, w: *mut HSD_WObj) {
        if !l.is_null() {
            ext::HSD_WObjUnref((*l).interest);
            (*l).interest = w;
        }
    }
    pub unsafe fn HSD_LObjSetPosition(
        l: *mut HSD_LObj,
        v: *const melee_types::Vec,
    ) {
        assert!(!l.is_null());
        if (*l).position.is_null() {
            (*l).position = ext::HSD_WObjAlloc();
            assert!(!(*l).position.is_null());
        }
        ext::HSD_WObjSetPosition((*l).position, v);
    }
    pub unsafe fn HSD_LObjSetInterest(
        l: *mut HSD_LObj,
        v: *const melee_types::Vec,
    ) {
        assert!(!l.is_null());
        if (*l).interest.is_null() {
            (*l).interest = ext::HSD_WObjAlloc();
            assert!(!(*l).interest.is_null());
        }
        ext::HSD_WObjSetPosition((*l).interest, v);
    }
    pub unsafe fn HSD_LObjGetPosition(
        l: *mut HSD_LObj,
        v: *mut melee_types::Vec,
    ) -> i32 {
        if !l.is_null() && !(*l).position.is_null() {
            ext::HSD_WObjGetPosition((*l).position, v);
            1
        } else {
            0
        }
    }
    pub unsafe fn HSD_LObjGetInterest(
        l: *mut HSD_LObj,
        v: *mut melee_types::Vec,
    ) -> i32 {
        if !l.is_null() && !(*l).interest.is_null() {
            ext::HSD_WObjGetPosition((*l).interest, v);
            1
        } else {
            0
        }
    }
    fn vec(v: melee_types::Vec) -> Vec3 {
        Vec3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
    fn native(v: Vec3) -> melee_types::Vec {
        melee_types::Vec {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
    pub const HSD_LObj_803B94A0: melee_types::Vec = melee_types::Vec {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    pub const HSD_LObj_803B94AC: melee_types::Vec = HSD_LObj_803B94A0;
    pub unsafe fn HSD_LObjGetLightVector(
        l: *mut HSD_LObj,
        v: *mut melee_types::Vec,
    ) {
        if l.is_null() {
            return;
        }
        let mut p = HSD_LObj_803B94A0;
        let mut i = HSD_LObj_803B94AC;
        HSD_LObjGetPosition(l, &mut p);
        HSD_LObjGetInterest(l, &mut i);
        *v = native(mtx::vec_normalize(&mtx::vec_sub(&vec(i), &vec(p))));
    }
    pub unsafe fn HSD_LObjAnim(l: *mut HSD_LObj) {
        if !l.is_null() {
            ext::HSD_AObjInterpretAnim((*l).aobj, l.cast(), LObjUpdateFunc);
            ext::HSD_WObjInterpretAnim((*l).position);
            ext::HSD_WObjInterpretAnim((*l).interest);
        }
    }
    pub unsafe fn HSD_LObjAnimAll(mut l: *mut HSD_LObj) {
        while !l.is_null() {
            HSD_LObjAnim(l);
            l = (*l).next;
        }
    }
    pub unsafe fn HSD_LObjReqAnim(l: *mut HSD_LObj, f: f32) {
        if !l.is_null() {
            ext::HSD_AObjReqAnim((*l).aobj, f);
            ext::HSD_WObjReqAnim((*l).position, f);
            ext::HSD_WObjReqAnim((*l).interest, f);
        }
    }
    pub unsafe fn HSD_LObjReqAnimAll(mut l: *mut HSD_LObj, f: f32) {
        while !l.is_null() {
            HSD_LObjReqAnim(l, f);
            l = (*l).next;
        }
    }
    pub unsafe fn HSD_LObjAddAnim(l: *mut HSD_LObj, a: *mut HSD_LightAnim) {
        if l.is_null() || a.is_null() {
            return;
        }
        if !(*l).aobj.is_null() {
            ext::HSD_AObjRemove((*l).aobj);
        }
        (*l).aobj = ext::HSD_AObjLoadDesc((*a).aobjdesc);
        ext::HSD_WObjAddAnim((*l).position, (*a).position_anim);
        ext::HSD_WObjAddAnim((*l).interest, (*a).interest_anim);
    }
    pub unsafe fn HSD_LObjAddAnimAll(
        mut l: *mut HSD_LObj,
        mut a: *mut HSD_LightAnim,
    ) {
        while !l.is_null() {
            HSD_LObjAddAnim(l, a);
            l = (*l).next;
            if !a.is_null() {
                a = (*a).next;
            }
        }
    }
    pub unsafe fn HSD_LObjSetup(l: *mut HSD_LObj, c: _GXColor, s: f32) {
        if (*l).flags & 32 != 0 || HSD_LObjGetType(l) == 0 {
            return;
        }
        if (*l).flags & 4 != 0 {
            let h = (*l).hw_color;
            if [h.r, h.g, h.b, h.a] != [c.r, c.g, c.b, c.a] {
                ext::GXInitLightColor(&mut (*l).lightobj, c);
                (*l).hw_color = c;
                (*l).flags |= 128;
            }
            if (*l).flags & 128 != 0 {
                ext::GXLoadLightObjImm(&mut (*l).lightobj, (*l).id);
                (*l).flags &= !128;
            }
        }
        if (*l).spec_id != 0 {
            if (*l).shininess != s {
                (*l).shininess = s;
                let k = s * 0.5;
                ext::GXInitLightAttn(
                    &mut (*l).spec_lightobj,
                    0.,
                    0.,
                    1.,
                    k,
                    0.,
                    1. - k,
                );
                (*l).flags |= 256;
            }
            if (*l).flags & 256 != 0 {
                ext::GXLoadLightObjImm(&mut (*l).spec_lightobj, (*l).spec_id);
                (*l).flags &= !256;
            }
        }
    }
    pub unsafe fn HSD_LObjSetupSpecularInit(m: &Mtx) {
        let j = Vec3 {
            x: m[0][3],
            y: m[1][3],
            z: m[2][3],
        };
        let c = mtx::vec_normalize(&j);
        for i in 0..HSD_LObjGetNbActive() {
            let l = HSD_LObjGetActiveByIndex(i);
            if (*l).spec_id == 0 {
                continue;
            }
            let h = match HSD_LObjGetType(l) {
                2 | 3 => mtx::vec_add(
                    &mtx::vec_normalize(&mtx::vec_sub(&j, &vec((*l).lvec))),
                    &c,
                ),
                1 => mtx::vec_add(&vec((*l).lvec), &c),
                _ => panic!("lobj.c:617"),
            };
            let h = mtx::vec_normalize(&h);
            ext::GXInitLightDir(&mut (*l).spec_lightobj, h.x, h.y, h.z);
            (*l).flags |= 256;
        }
    }
    pub unsafe fn setup_diffuse_lightobj(l: *mut HSD_LObj) {
        ext::GXInitLightColor(&mut (*l).lightobj, (*l).color);
        (*l).hw_color = (*l).color;
        (*l).flags |= 128;
        mask(0, (*l).id as i32);
        match HSD_LObjGetType(l) {
            2 | 3 => mask(2, (*l).id as i32),
            1 => (),
            _ => panic!("lobj.c:642"),
        };
        if (*l).flags & 16 != 0 {
            mask(3, (*l).id as i32);
        }
    }
    pub unsafe fn setup_spec_lightobj(l: *mut HSD_LObj, m: &Mtx, id: i32) {
        (*l).spec_id = id as u32;
        if id == 0 {
            return;
        }
        ext::GXInitLightColor(&mut (*l).spec_lightobj, (*l).color);
        (*l).shininess = 50.;
        let x = (*l).shininess * 0.5;
        ext::GXInitLightAttn(
            &mut (*l).spec_lightobj,
            0.,
            0.,
            1.,
            x,
            0.,
            1. - x,
        );
        match HSD_LObjGetType(l) {
            2 | 3 => {
                HSD_LObjGetPosition(l, &mut (*l).lvec);
                (*l).lvec = native(mtx::mult_vec(m, &vec((*l).lvec)));
            }
            1 => {
                HSD_LObjGetLightVector(l, &mut (*l).lvec);
                (*l).lvec = native(mtx::vec_normalize(&mtx::mult_vec_sr(
                    m,
                    &vec((*l).lvec),
                )));
            }
            _ => panic!("lobj.c:680"),
        };
        (*l).flags |= 256;
        mask(1, id);
    }
    pub unsafe fn setup_infinite_lightobj(l: *mut HSD_LObj, m: &Mtx) {
        let mut p = melee_types::Vec::zeroed();
        HSD_LObjGetPosition(l, &mut p);
        p.x *= 1048576.;
        p.y *= 1048576.;
        p.z *= 1048576.;
        let p = mtx::mult_vec(m, &vec(p));
        if (*l).flags & 4 != 0 {
            ext::GXInitLightPos(&mut (*l).lightobj, p.x, p.y, p.z);
            ext::GXInitLightAttn(&mut (*l).lightobj, 1., 0., 0., 1., 0., 0.);
        }
        if (*l).flags & 8 != 0 {
            ext::GXInitLightPos(&mut (*l).spec_lightobj, p.x, p.y, p.z);
        }
    }
    pub unsafe fn setup_point_lightobj(l: *mut HSD_LObj, m: &Mtx) {
        ext::GXInitLightColor(&mut (*l).lightobj, (*l).color);
        (*l).hw_color = (*l).color;
        let mut p = melee_types::Vec::zeroed();
        HSD_LObjGetPosition(l, &mut p);
        let p = mtx::mult_vec(m, &vec(p));
        ext::GXInitLightPos(&mut (*l).lightobj, p.x, p.y, p.z);
        ext::GXInitLightPos(&mut (*l).spec_lightobj, p.x, p.y, p.z);
        if (*l).flags & 64 != 0 {
            let a = (*l).u.attn;
            ext::GXInitLightAttn(
                &mut (*l).lightobj,
                1.,
                0.,
                0.,
                a.k0,
                a.k1,
                a.k2,
            );
        } else {
            let s = (*l).u.spot;
            ext::GXInitLightDistAttn(
                &mut (*l).lightobj,
                s.ref_dist,
                s.ref_br,
                s.dist_func as i32,
            );
            ext::GXInitLightSpot(&mut (*l).lightobj, 0., 0);
            ext::GXInitLightDistAttn(
                &mut (*l).spec_lightobj,
                s.ref_dist,
                s.ref_br,
                s.dist_func as i32,
            );
        }
    }
    pub unsafe fn setup_spot_lightobj(l: *mut HSD_LObj, m: &Mtx) {
        let mut p = melee_types::Vec::zeroed();
        HSD_LObjGetPosition(l, &mut p);
        let p = mtx::mult_vec(m, &vec(p));
        let mut d = melee_types::Vec::zeroed();
        HSD_LObjGetLightVector(l, &mut d);
        let d = mtx::vec_normalize(&mtx::mult_vec_sr(m, &vec(d)));
        ext::GXInitLightPos(&mut (*l).lightobj, p.x, p.y, p.z);
        ext::GXInitLightPos(&mut (*l).spec_lightobj, p.x, p.y, p.z);
        ext::GXInitLightDir(&mut (*l).lightobj, d.x, d.y, d.z);
        if (*l).flags & 64 != 0 {
            let a = (*l).u.attn;
            ext::GXInitLightAttn(
                &mut (*l).lightobj,
                a.a0,
                a.a1,
                a.a2,
                a.k0,
                a.k1,
                a.k2,
            );
        } else {
            let s = (*l).u.spot;
            ext::GXInitLightDistAttn(
                &mut (*l).lightobj,
                s.ref_dist,
                s.ref_br,
                s.dist_func as i32,
            );
            ext::GXInitLightDistAttn(
                &mut (*l).spec_lightobj,
                s.ref_dist,
                s.ref_br,
                s.dist_func as i32,
            );
            ext::GXInitLightSpot(
                &mut (*l).lightobj,
                s.cutoff,
                s.spot_func as i32,
            );
        }
    }
    pub unsafe fn HSD_LObjSetupInit(c: *mut HSD_CObj) {
        let m = &(*c).view_mtx;
        let mut idx = 0;
        let mut num = 0;
        MASK.with(|x| x.set([0; 4]));
        HSD_LObjClearActive();
        let mut list = CURRENT.with(Cell::get);
        while idx < 8 && !list.is_null() {
            let l = (*list).data.cast::<HSD_LObj>();
            assert!(!l.is_null());
            let f = (*l).flags;
            if f & 32 != 0 {
                list = (*list).next;
                continue;
            }
            let t = HSD_LObjGetType(l);
            let r = f & 12;
            if t != 0 && (r == 8 && num >= 8 || !matches!(r, 4 | 8 | 12)) {
                list = (*list).next;
                continue;
            }
            HSD_LObjSetActive(l);
            idx = HSD_LObjGetNbActive();
            (*l).spec_id = 0;
            match t {
                1 => setup_infinite_lightobj(l, m),
                2 => setup_point_lightobj(l, m),
                3 => setup_spot_lightobj(l, m),
                0 => {
                    list = (*list).next;
                    continue;
                }
                _ => unreachable!(),
            }
            match r {
                4 | 12 => setup_diffuse_lightobj(l),
                8 => {
                    setup_spec_lightobj(l, m, (*l).id as i32);
                    num += 1;
                }
                _ => (),
            }
            list = (*list).next;
        }
        if HSD_LObjGetActiveByID(256).is_null() {
            while !list.is_null() {
                let l = (*list).data.cast::<HSD_LObj>();
                assert!(!l.is_null());
                if HSD_LObjGetType(l) == 0 && (*l).flags & 32 == 0 {
                    HSD_LObjSetActive(l);
                    break;
                }
                list = (*list).next;
            }
        }
        list = CURRENT.with(Cell::get);
        while !list.is_null() && num < 8 && idx < 8 {
            let l = (*list).data.cast::<HSD_LObj>();
            if (*l).flags & 0x2c == 12 {
                setup_spec_lightobj(l, m, HSD_Index2LightID(idx as u32));
                idx += 1;
                num += 1;
            }
            list = (*list).next;
        }
        for i in 0..8 {
            let l = HSD_LObjGetActiveByIndex(i);
            if l.is_null() {
                return;
            }
            HSD_LObjSetup(l, (*l).color, (*l).shininess);
        }
    }
    pub unsafe fn HSD_LObjAddCurrent(l: *mut HSD_LObj) {
        if l.is_null() {
            return;
        }
        if current_lights().contains(&l) {
            HSD_LObjDeleteCurrent(l);
        }
        (*l).parent.ref_count = (*l).parent.ref_count.wrapping_add(1);
        assert_ne!((*l).parent.ref_count, 65535);
        let mut p = CURRENT.with(Cell::as_ptr);
        while !(*p).is_null() {
            let c = (**p).data.cast::<HSD_LObj>();
            if (*c).priority as u8 > (*l).priority as u8 {
                break;
            }
            p = &mut (**p).next;
        }
        *p = super::super::list::HSD_SListAllocAndPrepend(*p, l.cast());
    }
    pub unsafe fn HSD_LObjUnrefThis(l: *mut HSD_LObj) {
        if l.is_null() {
            return;
        }
        let r = (*l).parent.ref_count;
        let release = if r == 65535 {
            true
        } else {
            (*l).parent.ref_count = r.wrapping_sub(1);
            r == 0
        };
        if release {
            let c = (*l).parent.parent.class_info;
            (*c).release.expect("LObj release")(l.cast());
            let c = (*l).parent.parent.class_info;
            (*c).destroy.expect("LObj destroy")(l.cast());
        }
    }
    pub unsafe fn HSD_LObjDeleteCurrent(l: *mut HSD_LObj) {
        if l.is_null() {
            return;
        }
        let mut p = CURRENT.with(Cell::as_ptr);
        while !(*p).is_null() {
            if (**p).data == l.cast() {
                ACTIVE.with(|a| {
                    let mut a0 = a.get();
                    for p in &mut a0 {
                        if *p == l {
                            *p = null_mut();
                        }
                    }
                    a.set(a0);
                });
                *p = super::super::list::HSD_SListRemove(*p);
                HSD_LObjUnrefThis(l);
                return;
            }
            p = &mut (**p).next;
        }
    }
    pub unsafe fn LObjRemoveAll() {
        HSD_LObjClearActive();
        loop {
            let p = CURRENT.with(Cell::get);
            if p.is_null() {
                break;
            }
            HSD_LObjUnrefThis((*p).data.cast());
            let p =
                super::super::list::HSD_SListRemove(CURRENT.with(Cell::get));
            CURRENT.with(|x| x.set(p));
        }
    }
    pub unsafe fn HSD_LObjDeleteCurrentAll(mut l: *mut HSD_LObj) {
        if l.is_null() {
            LObjRemoveAll();
            return;
        }
        while !l.is_null() {
            HSD_LObjDeleteCurrent(l);
            l = (*l).next;
        }
    }
    pub unsafe fn HSD_LObjSetCurrentAll(l: *mut HSD_LObj) {
        LObjRemoveAll();
        HSD_LObjAddCurrent(l);
    }
    pub unsafe fn LObjReplaceAll(mut l: *mut HSD_LObj) {
        LObjRemoveAll();
        while !l.is_null() {
            HSD_LObjAddCurrent(l);
            l = (*l).next;
        }
    }
    pub unsafe fn HSD_LObj_803668EC(l: *mut HSD_LObj) {
        LObjReplaceAll(l);
    }
    pub unsafe fn HSD_LObjGetCurrentByType(f: u16) -> *mut HSD_LObj {
        current_lights()
            .into_iter()
            .find(|&l| (*l).flags & 3 == f & 3)
            .unwrap_or(null_mut())
    }
    pub unsafe fn HSD_LObjRemoveAll(mut l: *mut HSD_LObj) {
        while !l.is_null() {
            let next = (*l).next;
            HSD_LObjDeleteCurrent(l);
            HSD_LObjUnrefThis(l);
            l = next;
        }
    }
    pub unsafe fn HSD_LObjSetDefaultClass(c: *mut HSD_LObjInfo) {
        if !c.is_null() {
            assert!(ext::hsdIsDescendantOf(c.cast(), hsdLObj().cast()) != 0);
        }
        DEFAULT.with(|x| x.set(c));
    }
    pub fn HSD_LObjGetDefaultClass() -> *mut HSD_LObjInfo {
        let c = DEFAULT.with(Cell::get);
        if c.is_null() {
            hsdLObj()
        } else {
            c
        }
    }
    pub unsafe fn HSD_LObjAlloc() -> *mut HSD_LObj {
        let l =
            ext::hsdNew(HSD_LObjGetDefaultClass().cast()).cast::<HSD_LObj>();
        assert!(!l.is_null());
        l
    }
    pub unsafe extern "C" fn LObjLoad(
        l: *mut HSD_LObj,
        d: *mut HSD_LightDesc,
    ) -> i32 {
        HSD_LObjSetColor(l, (*d).color);
        HSD_LObjSetFlags(l, (*d).flags as u32);
        match (*d).flags & 3 {
            0 => (),
            1 => HSD_LObjSetPositionWObj(
                l,
                ext::HSD_WObjLoadDesc((*d).position),
            ),
            2 => {
                HSD_LObjSetPositionWObj(
                    l,
                    ext::HSD_WObjLoadDesc((*d).position),
                );
                if (*d).attnflags & 1 != 0 {
                    HSD_LObjSetFlags(l, 64);
                    let a = *(*d).u.attn;
                    HSD_LObjSetAttnK(l, a.k0, a.k1, a.k2);
                } else {
                    let p = *(*d).u.point;
                    HSD_LObjSetDistAttn(
                        l,
                        p.ref_dist,
                        p.ref_br,
                        p.dist_func as i32,
                    );
                }
            }
            3 => {
                HSD_LObjSetPositionWObj(
                    l,
                    ext::HSD_WObjLoadDesc((*d).position),
                );
                HSD_LObjSetInterestWObj(
                    l,
                    ext::HSD_WObjLoadDesc((*d).interest),
                );
                if (*d).attnflags != 0 {
                    HSD_LObjSetFlags(l, 64);
                    let a = *(*d).u.attn;
                    HSD_LObjSetAttn(l, a.a0, a.a1, a.a2, a.k0, a.k1, a.k2);
                } else {
                    let p = *(*d).u.spot;
                    HSD_LObjSetDistAttn(
                        l,
                        p.ref_dist,
                        p.ref_br,
                        p.dist_func as i32,
                    );
                    HSD_LObjSetSpot(l, p.cutoff, p.spot_func as i32);
                }
            }
            _ => unreachable!(),
        }
        0
    }
    pub unsafe fn HSD_LObjLoadDesc(
        mut d: *mut HSD_LightDesc,
    ) -> *mut HSD_LObj {
        let mut top = null_mut();
        let mut p = &mut top as *mut *mut HSD_LObj;
        while !d.is_null() {
            let info = if (*d).class_name.is_null() {
                null_mut()
            } else {
                ext::hsdSearchClassInfo((*d).class_name)
            };
            *p = if info.is_null() {
                HSD_LObjAlloc()
            } else {
                let l = ext::hsdNew(info).cast::<HSD_LObj>();
                assert!(!l.is_null());
                l
            };
            let info = (*(*p)).parent.parent.class_info.cast::<HSD_LObjInfo>();
            (*info).load.expect("LObj load")(*p, d);
            p = &mut (**p).next;
            d = (*d).next;
        }
        *p = null_mut();
        top
    }
    pub unsafe extern "C" fn LObjRelease(o: *mut _HSD_Class) {
        let l = o.cast::<HSD_LObj>();
        ext::HSD_AObjRemove((*l).aobj);
        ext::HSD_WObjUnref((*l).position);
        ext::HSD_WObjUnref((*l).interest);
        let p = (*hsdLObj()).parent.parent.head.parent;
        (*p).release.expect("parent release")(o);
    }
    pub unsafe extern "C" fn LObjAmnesia(c: *mut _HSD_ClassInfo) {
        if c == DEFAULT.with(Cell::get).cast() {
            DEFAULT.with(|x| x.set(null_mut()));
        }
        if c == hsdLObj().cast() {
            CURRENT.with(|x| x.set(null_mut()));
        }
        let p = (*hsdLObj()).parent.parent.head.parent;
        (*p).amnesia.expect("parent amnesia")(c);
    }
    pub unsafe extern "C" fn LObjInfoInit() {
        let c = hsdLObj();
        ext::hsdInitClassInfo(
            c.cast(),
            ext::hsdObj(),
            c"sysdolphin_base_library".as_ptr(),
            c"hsd_lobj".as_ptr(),
            core::mem::size_of::<HSD_LObjInfo>() as i32,
            core::mem::size_of::<HSD_LObj>() as i32,
        );
        (*c).parent.parent.release = Some(LObjRelease);
        (*c).parent.parent.amnesia = Some(LObjAmnesia);
        (*c).load = Some(LObjLoad);
    }

    /// Cross-TU boundaries, in the `ext_fns!` form every TU uses (linked to the canonical TU function by
    /// `melee_link::install`; the GX light calls are host seams).
    pub mod ext {
        use super::*;
        use melee_types::{HSD_AObj, HSD_AObjDesc, HSD_WObjAnim, HSD_WObjDesc, _GXLightObj};
        macro_rules! ext_fns {
            ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
                pub mod hook {
                    use super::*;
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
        fn GXInitLightAttn(a0: *mut _GXLightObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32);
        fn GXInitLightColor(a0: *mut _GXLightObj, a1: _GXColor);
        fn GXInitLightDir(a0: *mut _GXLightObj, a1: f32, a2: f32, a3: f32);
        fn GXInitLightDistAttn(a0: *mut _GXLightObj, a1: f32, a2: f32, a3: i32);
        fn GXInitLightPos(a0: *mut _GXLightObj, a1: f32, a2: f32, a3: f32);
        fn GXInitLightSpot(a0: *mut _GXLightObj, a1: f32, a2: i32);
        fn GXLoadLightObjImm(a0: *mut _GXLightObj, a1: u32);
        fn HSD_AObjInterpretAnim(a0: *mut HSD_AObj, a1: *mut c_void, a2: unsafe extern "C" fn(*mut c_void, i32, *mut HSD_ObjData));
        fn HSD_AObjLoadDesc(a0: *mut HSD_AObjDesc) -> *mut HSD_AObj;
        fn HSD_AObjRemove(a0: *mut HSD_AObj);
        fn HSD_AObjReqAnim(a0: *mut HSD_AObj, a1: f32);
        fn HSD_WObjAddAnim(a0: *mut HSD_WObj, a1: *mut HSD_WObjAnim);
        fn HSD_WObjAlloc() -> *mut HSD_WObj;
        fn HSD_WObjGetPosition(a0: *mut HSD_WObj, a1: *mut melee_types::Vec);
        fn HSD_WObjInterpretAnim(a0: *mut HSD_WObj);
        fn HSD_WObjLoadDesc(a0: *mut HSD_WObjDesc) -> *mut HSD_WObj;
        fn HSD_WObjReqAnim(a0: *mut HSD_WObj, a1: f32);
        fn HSD_WObjSetPosition(a0: *mut HSD_WObj, a1: *const melee_types::Vec);
        fn HSD_WObjUnref(a0: *mut HSD_WObj);
        fn hsdInitClassInfo(a0: *mut _HSD_ClassInfo, a1: *mut _HSD_ClassInfo, a2: *const i8, a3: *const i8, a4: i32, a5: i32);
        fn hsdIsDescendantOf(a0: *mut c_void, a1: *mut c_void) -> i32;
        fn hsdNew(a0: *mut _HSD_ClassInfo) -> *mut c_void;
        fn hsdSearchClassInfo(a0: *const i8) -> *mut _HSD_ClassInfo;
        }
        // ---- C-oracle test shim: one string-dispatched hook (tests/oracle_lobj_complete.rs) -------------------
        pub type Hook = unsafe fn(&str, &[usize]) -> usize;
        thread_local! {static HOOK: core::cell::Cell<Option<Hook>> = const { core::cell::Cell::new(None) };}
        unsafe fn call(n: &str, a: &[usize]) -> usize {
            (HOOK.with(core::cell::Cell::get).expect("lobj ext hook"))(n, a)
        }
        /// Route every cross-TU callee through `h` (None: unset them again).
        pub fn set_hook(h: Option<Hook>) {
            HOOK.with(|x| x.set(h));
            unsafe {
                hook::GXInitLightAttn = if h.is_some() { Some(|a0, a1, a2, a3, a4, a5, a6| unsafe { let _ = call("GXInitLightAttn", &[a0 as usize, a1.to_bits() as usize, a2.to_bits() as usize, a3.to_bits() as usize, a4.to_bits() as usize, a5.to_bits() as usize, a6.to_bits() as usize]); }) } else { None };
                hook::GXInitLightColor = if h.is_some() { Some(|a0, a1| unsafe { let _ = call("GXInitLightColor", &[a0 as usize, u32::from_be_bytes([a1.r, a1.g, a1.b, a1.a]) as usize]); }) } else { None };
                hook::GXInitLightDir = if h.is_some() { Some(|a0, a1, a2, a3| unsafe { let _ = call("GXInitLightDir", &[a0 as usize, a1.to_bits() as usize, a2.to_bits() as usize, a3.to_bits() as usize]); }) } else { None };
                hook::GXInitLightDistAttn = if h.is_some() { Some(|a0, a1, a2, a3| unsafe { let _ = call("GXInitLightDistAttn", &[a0 as usize, a1.to_bits() as usize, a2.to_bits() as usize, a3 as usize]); }) } else { None };
                hook::GXInitLightPos = if h.is_some() { Some(|a0, a1, a2, a3| unsafe { let _ = call("GXInitLightPos", &[a0 as usize, a1.to_bits() as usize, a2.to_bits() as usize, a3.to_bits() as usize]); }) } else { None };
                hook::GXInitLightSpot = if h.is_some() { Some(|a0, a1, a2| unsafe { let _ = call("GXInitLightSpot", &[a0 as usize, a1.to_bits() as usize, a2 as usize]); }) } else { None };
                hook::GXLoadLightObjImm = if h.is_some() { Some(|a0, a1| unsafe { let _ = call("GXLoadLightObjImm", &[a0 as usize, a1 as usize]); }) } else { None };
                hook::HSD_AObjLoadDesc = if h.is_some() { Some(|a0| unsafe { call("HSD_AObjLoadDesc", &[a0 as usize]) as *mut HSD_AObj }) } else { None };
                hook::HSD_AObjRemove = if h.is_some() { Some(|a0| unsafe { let _ = call("HSD_AObjRemove", &[a0 as usize]); }) } else { None };
                hook::HSD_AObjReqAnim = if h.is_some() { Some(|a0, a1| unsafe { let _ = call("HSD_AObjReqAnim", &[a0 as usize, a1.to_bits() as usize]); }) } else { None };
                hook::HSD_WObjAddAnim = if h.is_some() { Some(|a0, a1| unsafe { let _ = call("HSD_WObjAddAnim", &[a0 as usize, a1 as usize]); }) } else { None };
                hook::HSD_WObjAlloc = if h.is_some() { Some(|| unsafe { call("HSD_WObjAlloc", &[]) as *mut HSD_WObj }) } else { None };
                hook::HSD_WObjGetPosition = if h.is_some() { Some(|a0, a1| unsafe { let _ = call("HSD_WObjGetPosition", &[a0 as usize, a1 as usize]); }) } else { None };
                hook::HSD_WObjInterpretAnim = if h.is_some() { Some(|a0| unsafe { let _ = call("HSD_WObjInterpretAnim", &[a0 as usize]); }) } else { None };
                hook::HSD_WObjLoadDesc = if h.is_some() { Some(|a0| unsafe { call("HSD_WObjLoadDesc", &[a0 as usize]) as *mut HSD_WObj }) } else { None };
                hook::HSD_WObjReqAnim = if h.is_some() { Some(|a0, a1| unsafe { let _ = call("HSD_WObjReqAnim", &[a0 as usize, a1.to_bits() as usize]); }) } else { None };
                hook::HSD_WObjSetPosition = if h.is_some() { Some(|a0, a1| unsafe { let _ = call("HSD_WObjSetPosition", &[a0 as usize, a1 as usize]); }) } else { None };
                hook::HSD_WObjUnref = if h.is_some() { Some(|a0| unsafe { let _ = call("HSD_WObjUnref", &[a0 as usize]); }) } else { None };
                hook::hsdInitClassInfo = if h.is_some() { Some(|a0, a1, a2, a3, a4, a5| unsafe { let _ = call("hsdInitClassInfo", &[a0 as usize, a1 as usize, a2 as usize, a3 as usize, a4 as usize, a5 as usize]); }) } else { None };
                hook::hsdIsDescendantOf = if h.is_some() { Some(|a0, a1| unsafe { call("hsdIsDescendantOf", &[a0 as usize, a1 as usize]) as i32 }) } else { None };
                hook::hsdNew = if h.is_some() { Some(|a0| unsafe { call("hsdNew", &[a0 as usize]) as *mut c_void }) } else { None };
                hook::hsdSearchClassInfo = if h.is_some() { Some(|a0| unsafe { call("hsdSearchClassInfo", &[a0 as usize]) as *mut _HSD_ClassInfo }) } else { None };
            }
        }
        /// `&hsdObj` (object.c): the parent class info.
        pub unsafe fn hsdObj() -> *mut _HSD_ClassInfo {
            core::ptr::addr_of_mut!(crate::tu::object::hsdObj) as *mut _HSD_ClassInfo
        }
    }
}
