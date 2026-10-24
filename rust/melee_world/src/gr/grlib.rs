//! src/melee/gr/grlib.c, round 8: all 24 functions (incl. the static quake procs and the
//! PointInsideColl inline). TARGET_PC bodies.
//!
//! State is `StageWorld::stage_info` plus `GrLib::mtx` (the C static `grLib_8049EF58`).
//! Raw HSD pointers stay raw, as in the C; every callee in another TU goes through `GrLibHooks`,
//! whose default bodies panic with the owning group.
//!
//! FMA: fma_sites.py reports 4 fused sites, all in PointInsideColl: `0.5f*height + offset` and
//! `0.5f*(top+bottom) + topn` (and the x twins). `- point->y` is a separate subtract.
#![allow(non_upper_case_globals)]
use super::stage::StageWorld;
use crate::vec::Vec3;
use melee_types::*;

/// Which static proc grLib_801C9CEC installs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuakeProc {
    /// grLib_801C9BC8 (loop quake)
    Loop,
    /// grLib_801C9C40 (one-shot quake, frees itself when the anim ends)
    OneShot,
}

/// Callees outside grlib.c. Defaults are stubs.
#[allow(unused_variables)]
pub trait GrLibHooks {
    /// TODO(group1: baselib/particle.c) hsd_8039F05C
    fn hsd_8039f05c(&mut self, a: i32, b: i32, c: i32) -> *mut HSD_Generator {
        unimplemented!("TODO(group1): hsd_8039F05C")
    }
    /// TODO(group1: baselib/psappsrt.c)
    fn ps_add_generator_app_srt_begin(&mut self, g: *mut HSD_Generator, n: i32) -> *mut HSD_psAppSRT {
        unimplemented!("TODO(group1): psAddGeneratorAppSRT_begin")
    }
    /// TODO(group1: baselib/particle.c) hsd_8039D4DC (kill generator)
    fn hsd_8039d4dc(&mut self, g: *mut HSD_Generator) {
        unimplemented!("TODO(group1): hsd_8039D4DC")
    }
    /// TODO(group1: baselib/particle.c) hsd_8039D5DC
    fn hsd_8039d5dc(&mut self, j: *mut HSD_JObj) {
        unimplemented!("TODO(group1): hsd_8039D5DC")
    }
    /// TODO(group1: baselib/particle.c) hsd_8039EFAC
    fn hsd_8039efac(&mut self, a: i32, b: i32, c: i32, j: *mut HSD_JObj) -> *mut HSD_Generator {
        unimplemented!("TODO(group1): hsd_8039EFAC")
    }
    /// TODO(group1: baselib/particle.c) hsd_8039F6CC
    fn hsd_8039f6cc(&mut self, a: i32, b: i32, c: i32, j: *mut HSD_JObj) {
        unimplemented!("TODO(group1): hsd_8039F6CC")
    }
    /// TODO(group1: baselib/particle.c) the generator list head `hsd_804D78FC`
    fn generator_list_head(&mut self) -> *mut HSD_Generator {
        unimplemented!("TODO(group1): hsd_804D78FC")
    }
    /// TODO(group2: gr/ground.c) Ground_801C2D24
    fn ground_801c2d24(&mut self, i: i32, out: &mut Vec) -> bool {
        unsafe { crate::tu::ground::Ground_801C2D24(i, out as *mut Vec as _) != 0 }
    }
    /// TODO(group1: lb/lb_00F9.c) lb_8000FD48
    fn lb_8000fd48(&mut self, j: *mut HSD_JObj, d: *mut DynamicsDesc, count: i32) {
        unsafe { melee_runtime::tu::lb_00F9::lb_8000FD48(j, d, count as usize) }
    }
    /// TODO(group1: lb/lb_00F9.c) lb_80011710
    fn lb_80011710(&mut self, a: *mut DynamicsDesc, b: *mut DynamicsDesc) {
        unsafe { melee_runtime::tu::lb_00F9::lb_80011710(a, b) }
    }
    /// TODO(group1: lb/lb_00F9.c) lb_8000FD18
    fn lb_8000fd18(&mut self, a: *mut core::ffi::c_void) {
        unsafe { melee_runtime::tu::lb_00F9::lb_8000FD18(a as _) }
    }
    /// TODO(group1: lb/lb_00F9.c) lb_8001044C(arg0, 0, 0, 0.0f, 0, 0, 0, 0)
    fn lb_8001044c_zero(&mut self, a: *mut core::ffi::c_void) {
        unsafe { melee_runtime::tu::lb_00F9::lb_8001044C(a as _, core::ptr::null_mut(), 0, 0.0, false, 0 as _, 0, false) }
    }
    /// TODO(group1: baselib/jobj.c) HSD_JObjAnimAll
    fn jobj_anim_all(&mut self, j: *mut HSD_JObj) {
        unimplemented!("TODO(group1): HSD_JObjAnimAll")
    }
    /// HSD_JObjGetTranslationX/Y (baselib/jobj.h inlines, melee_types::inline): `j->translate`.
    /// The inlines' `HSD_ASSERT(jobj)` is the null check.
    fn jobj_translation_xy(&mut self, j: *mut HSD_JObj) -> (f32, f32) {
        assert!(!j.is_null(), "HSD_JObjGetTranslation: jobj");
        // SAFETY: non-null, and the caller passes a live JObj as the C does.
        let j = unsafe { &*j };
        (melee_types::inline::HSD_JObjGetTranslationX(j), melee_types::inline::HSD_JObjGetTranslationY(j))
    }
    /// Camera_SetQuakeOffset: `CameraWorld` implements it (cm::camera).
    fn camera_set_quake_offset(&mut self, x: f32, y: f32) {
        crate::cm::camera::world().camera_set_quake_offset(x, y)
    }
    /// TODO(group1: baselib/gobj.c) HSD_GObjFree
    fn gobj_free(&mut self, g: *mut HSD_GObj) {
        unsafe { melee_runtime::tu::gobj_native::HSD_GObjFree(g) }
    }
    /// TODO(group1: baselib/gobj.c) GObj_Create
    fn gobj_create(&mut self, class: i32, plink: i32, prio: u8) -> *mut HSD_GObj {
        unsafe { melee_runtime::tu::gobj_native::GObj_Create(class as u16, plink as u8, prio) }
    }
    /// TODO(group1: baselib/jobj.c) HSD_JObjLoadJoint
    fn jobj_load_joint(&mut self, j: *mut HSD_Joint) -> *mut HSD_JObj {
        unsafe { melee_runtime::tu::jobj::HSD_JObjLoadJoint(j) }
    }
    /// TODO(group1: baselib/gobjobject.c) HSD_GObjObject_80390A70(gobj, HSD_GObj_JObjKind, jobj)
    fn gobj_add_jobj(&mut self, g: *mut HSD_GObj, j: *mut HSD_JObj) {
        unsafe { melee_runtime::tu::gobj_native::HSD_GObjObject_80390A70(g, melee_runtime::tu::gobj_native::HSD_GObj_JObjKind, j as _) }
    }
    /// TODO(group1: baselib/gobjproc.c) HSD_GObj_SetupProc(gobj, proc, 1)
    fn gobj_setup_proc(&mut self, g: *mut HSD_GObj, p: QuakeProc) {
        let f: unsafe extern "C" fn(*mut HSD_GObj) = match p {
            QuakeProc::Loop => crate::tu::grlib::grLib_801C9BC8,
            QuakeProc::OneShot => crate::tu::grlib::grLib_801C9C40,
        };
        unsafe { melee_runtime::tu::gobj_native::HSD_GObj_SetupProc(g, Some(f), 1) };
    }
    /// TODO(group1: baselib/jobj.c) HSD_JObjAddAnimAll(jobj, anim, NULL, NULL)
    fn jobj_add_anim_all(&mut self, j: *mut HSD_JObj, a: *mut HSD_AnimJoint) {
        unimplemented!("TODO(group1): HSD_JObjAddAnimAll")
    }
    /// TODO(group1: baselib/jobj.c) HSD_JObjReqAnimAll(jobj, 0)
    fn jobj_req_anim_all(&mut self, j: *mut HSD_JObj, f: f32) {
        unimplemented!("TODO(group1): HSD_JObjReqAnimAll")
    }
    /// TODO(group1: baselib/jobj.c) HSD_ForeachAnim(jobj, 6, 0x20, HSD_AObjSetFlags, AOBJ_ARG_AU, AOBJ_LOOP)
    fn foreach_anim_set_loop(&mut self, j: *mut HSD_JObj) {
        unimplemented!("TODO(group1): HSD_ForeachAnim")
    }
    /// TODO(group3: the HSD_GOBJ_PLINK_FIGHTER list) each fighter's `coll_data`, in list order
    fn fighter_colls(&mut self) -> std::vec::Vec<*const CollData> {
        let mut v = std::vec::Vec::new();
        unsafe {
            let mut cur = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
            while !cur.is_null() {
                v.push(&(*((*cur).user_data as *mut Fighter)).coll_data as *const CollData);
                cur = (*cur).next;
            }
        }
        v
    }
    /// TODO(group5: the HSD_GOBJ_PLINK_ITEM list) each item's (itGetKind, `x378_itemColl`)
    fn item_colls(&mut self) -> std::vec::Vec<(ItemKind, *const CollData)> {
        let mut v = std::vec::Vec::new();
        unsafe {
            let mut cur = *melee_runtime::tu::gobj_native::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_ITEM as usize);
            while !cur.is_null() {
                let ip = (*cur).user_data as *mut Item;
                v.push(((*ip).kind, &(*ip).x378_itemColl as *const CollData));
                cur = (*cur).next;
            }
        }
        v
    }
}

/// grLib's own static state.
#[derive(Clone, Copy, Debug, Default)]
pub struct GrLib {
    /// `static VecMtx grLib_8049EF58`
    pub mtx: [Vec3; 4],
}

/// grLib_801C96E8
pub fn grlib_801c96e8(gp: &Ground) -> bool {
    gp.x10_flags.b4() != 0
}

/// grLib_801C96F8
pub fn grlib_801c96f8(w: &StageWorld, h: &mut dyn GrLibHooks, arg0: i32, arg1: i32, arg2: &Vec3) -> *mut HSD_Generator {
    let g = h.hsd_8039f05c(0, arg1, arg0);
    if !g.is_null() {
        unsafe {
            let mut srt = (*g).appsrt;
            if srt.is_null() {
                srt = h.ps_add_generator_app_srt_begin(g, 1);
            } else {
                (*srt).xA2 = 0;
            }
            if srt.is_null() {
                h.hsd_8039d4dc(g);
                return core::ptr::null_mut();
            }
            (*srt).translate = Vec { x: arg2.x, y: arg2.y, z: arg2.z };
            let scale = w.ground_801c0498();
            (*srt).scale.x *= scale;
            (*srt).scale.y *= scale;
            (*srt).scale.z *= scale;
            (*g).pos.z = 0.0;
            (*g).pos.y = 0.0;
            (*g).pos.x = 0.0;
        }
    }
    g
}

/// grLib_801C97DC
pub fn grlib_801c97dc(h: &mut dyn GrLibHooks, arg0: i32, arg1: i32, j: *mut HSD_JObj) {
    h.hsd_8039f6cc(0, arg1, arg0, j);
}
/// grLib_801C9808
pub fn grlib_801c9808(h: &mut dyn GrLibHooks, arg0: i32, arg1: i32, j: *mut HSD_JObj) -> *mut HSD_Generator {
    h.hsd_8039efac(0, arg1, arg0, j)
}
/// grLib_801C9834
pub fn grlib_801c9834(h: &mut dyn GrLibHooks, g: *mut HSD_Generator) {
    h.hsd_8039d4dc(g);
}
/// grLib_801C9854
pub fn grlib_801c9854(h: &mut dyn GrLibHooks, j: *mut HSD_JObj) {
    h.hsd_8039d5dc(j);
}
/// grLib_801C9874
pub fn grlib_801c9874(h: &mut dyn GrLibHooks, g: *mut HSD_Generator) {
    unsafe { (*g).r#type |= 0x80 };
    h.hsd_8039d4dc(g);
}

fn kill_generators_on(h: &mut dyn GrLibHooks, j: *mut HSD_JObj) {
    let mut cur = h.generator_list_head();
    while !cur.is_null() {
        let next = unsafe { (*cur).next };
        if unsafe { (*cur).jobj } == j {
            unsafe { (*cur).r#type |= 0x80 };
            h.hsd_8039d4dc(cur);
        }
        cur = next;
    }
}

/// grLib_801C98A0
pub fn grlib_801c98a0(h: &mut dyn GrLibHooks, j: *mut HSD_JObj) {
    if j.is_null() {
        return;
    }
    kill_generators_on(h, j);
}

/// grLib_801C9908 (recursive over the joint tree, stops at instances)
pub fn grlib_801c9908(h: &mut dyn GrLibHooks, j: *mut HSD_JObj) {
    if j.is_null() {
        return;
    }
    kill_generators_on(h, j);
    if unsafe { (*j).flags } & JOBJ_INSTANCE as u32 != 0 {
        return;
    }
    let mut c = unsafe { (*j).child };
    while !c.is_null() {
        grlib_801c9908(h, c);
        c = unsafe { (*c).next };
    }
}

/// grLib_801C99C0
pub fn grlib_801c99c0(h: &mut dyn GrLibHooks, bank: i32, gfx_id: i32, j: *mut HSD_JObj, flag: i32) {
    if flag != 0 {
        h.hsd_8039efac(0, bank, gfx_id, j);
    } else {
        h.hsd_8039f6cc(0, bank, gfx_id, j);
    }
}

impl GrLib {
    /// grLib_801C9A10: fills the four points; returns the matrix (the C returns its address).
    pub fn grlib_801c9a10(&mut self, h: &mut dyn GrLibHooks) -> &[Vec3; 4] {
        for i in 0..4 {
            let mut v = Vec { x: self.mtx[i].x, y: self.mtx[i].y, z: self.mtx[i].z };
            h.ground_801c2d24(i as i32, &mut v);
            self.mtx[i] = Vec3 { x: v.x, y: v.y, z: v.z };
        }
        &self.mtx
    }
    /// grLib_801C9A70 (HSD_ASSERT(290, 0) outside 0..3)
    pub fn grlib_801c9a70(&self, arg0: i32) -> Vec3 {
        assert!((0..4).contains(&arg0), "grlib.c:290: 0");
        self.mtx[arg0 as usize]
    }
}

/// grLib_801C9B20
pub fn grlib_801c9b20(h: &mut dyn GrLibHooks, j: *mut HSD_JObj, a1: *mut DynamicsDesc, a2: *mut DynamicsDesc) {
    let count = unsafe { (*a1).count };
    h.lb_8000fd48(j, a2, count as i32);
    h.lb_80011710(a1, a2);
}
/// grLib_801C9B6C
pub fn grlib_801c9b6c(h: &mut dyn GrLibHooks, a: *mut core::ffi::c_void) {
    h.lb_8000fd18(a);
}
/// grLib_801C9B8C
pub fn grlib_801c9b8c(h: &mut dyn GrLibHooks, a: *mut core::ffi::c_void) {
    h.lb_8001044c_zero(a);
}

/// grLib_801C9BC8 (static proc, loop quake)
pub fn grlib_801c9bc8(h: &mut dyn GrLibHooks, gobj: *mut HSD_GObj) {
    let j = unsafe { (*gobj).hsd_obj } as *mut HSD_JObj;
    h.jobj_anim_all(j);
    let (x, y) = h.jobj_translation_xy(j);
    h.camera_set_quake_offset(x, y);
}

/// grLib_801C9C40 (static proc, one-shot quake)
pub fn grlib_801c9c40(h: &mut dyn GrLibHooks, gobj: *mut HSD_GObj) {
    let j = unsafe { (*gobj).hsd_obj } as *mut HSD_JObj;
    let aobj = unsafe { (*j).aobj };
    h.jobj_anim_all(j);
    let (x, y) = h.jobj_translation_xy(j);
    h.camera_set_quake_offset(x, y);
    if aobj.is_null() || unsafe { (*aobj).flags } & 0x4000_0000 != 0 {
        h.gobj_free(gobj);
    }
}

impl StageWorld {
    /// grLib_801C9CEC
    pub fn grlib_801c9cec(&self, h: &mut dyn GrLibHooks, kind: CmQuakeKind) -> *mut HSD_GObj {
        let idx = match kind {
            QuakeKind_Loop => 0,
            QuakeKind_Small => 1,
            QuakeKind_Medium => 2,
            QuakeKind_Large => 3,
            _ => return core::ptr::null_mut(),
        };
        let set = self.stage_info.quake_model_set;
        if set.is_null() {
            return core::ptr::null_mut();
        }
        let gobj = h.gobj_create(HSD_GOBJ_CLASS_STAGE, 18, kind as u8);
        let j = h.jobj_load_joint(unsafe { (*set).joint });
        h.gobj_add_jobj(gobj, j);
        h.gobj_setup_proc(gobj, if kind == QuakeKind_Loop { QuakeProc::Loop } else { QuakeProc::OneShot });
        h.jobj_add_anim_all(j, unsafe { *(*set).anims.add(idx) });
        h.jobj_req_anim_all(j, 0.0);
        if kind == QuakeKind_Loop {
            h.foreach_anim_set_loop(j);
        }
        gobj
    }
    /// grLib_801C9E40
    pub fn grlib_801c9e40(&self) -> i32 {
        self.stage_info.x708 as i32
    }
    /// grLib_801C9E50
    pub fn grlib_801c9e50(&mut self, v: i16) {
        self.stage_info.x708 = v;
    }
    /// grLib_801C9E60
    pub fn grlib_801c9e60(&self, h: &mut dyn GrLibHooks, v: &mut Vec3) -> bool {
        match self.stage_info.grkind {
            // The three per-stage getters are ported on StageWorld (gr::ground, night 3 round 5).
            Gr_Kind_RCruise | Gr_Kind_BigBlue | Gr_Kind_Icemt => {
                let _ = h;
                let mut t = Vec { x: v.x, y: v.y, z: v.z };
                match self.stage_info.grkind {
                    Gr_Kind_RCruise => self.grrcruise_80201918(&mut t),
                    Gr_Kind_BigBlue => self.grbigblue_801ef7d8(&mut t),
                    _ => self.gricemt_801fa728(&mut t),
                }
                *v = Vec3 { x: t.x, y: t.y, z: t.z };
            }
            _ => {
                v.z = 0.0;
                v.y = 0.0;
                v.x = 0.0;
                return false;
            }
        }
        true
    }
}

#[inline]
fn abs_c(x: f32) -> f32 {
    if x < 0.0 { -x } else { x }
}

/// PointInsideColl (static inline in grlib.c), on the six CollData fields it reads:
/// ecb.top.y, ecb.bottom.y, ecb.right.x, ecb.left.x, cur_pos.x, cur_pos.y.
pub fn point_inside_box(top: f32, bottom: f32, right: f32, left: f32, pos_x: f32, pos_y: f32, p: &Vec3, offset: f32) -> bool {
    let height = top - bottom;
    let comp = 0.5f32.mul_add(height, offset);
    let y = 0.5f32.mul_add(top + bottom, pos_y) - p.y;
    if abs_c(y) > comp {
        return false;
    }
    let width = right - left;
    let comp = 0.5f32.mul_add(width, offset);
    let x = 0.5f32.mul_add(left + right, pos_x) - p.x;
    if abs_c(x) > comp {
        return false;
    }
    true
}

/// PointInsideColl
pub fn point_inside_coll(cd: &CollData, p: &Vec3, offset: f32) -> bool {
    point_inside_box(cd.ecb.top.y, cd.ecb.bottom.y, cd.ecb.right.x, cd.ecb.left.x, cd.cur_pos.x, cd.cur_pos.y, p, offset)
}

/// grLib_801C9EE8
pub fn grlib_801c9ee8(h: &mut dyn GrLibHooks, p: &Vec3, offset: f32) -> bool {
    for cd in h.fighter_colls() {
        if point_inside_coll(unsafe { &*cd }, p, offset) {
            return true;
        }
    }
    for (kind, cd) in h.item_colls() {
        if kind != It_PKind_Random && point_inside_coll(unsafe { &*cd }, p, offset) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Rec {
        log: std::vec::Vec<String>,
        gens: std::vec::Vec<Box<HSD_Generator>>,
        fighters: std::vec::Vec<Box<CollData>>,
        items: std::vec::Vec<(ItemKind, Box<CollData>)>,
    }
    impl GrLibHooks for Rec {
        fn hsd_8039d4dc(&mut self, g: *mut HSD_Generator) {
            self.log.push(format!("kill {}", unsafe { (*g).idnum }));
        }
        fn generator_list_head(&mut self) -> *mut HSD_Generator {
            self.gens.first_mut().map_or(core::ptr::null_mut(), |g| &mut **g)
        }
        fn ground_801c2d24(&mut self, i: i32, out: &mut Vec) -> bool {
            out.x = i as f32;
            out.y = -(i as f32);
            true
        }
        fn fighter_colls(&mut self) -> std::vec::Vec<*const CollData> {
            self.fighters.iter().map(|c| &**c as *const _).collect()
        }
        fn item_colls(&mut self) -> std::vec::Vec<(ItemKind, *const CollData)> {
            self.items.iter().map(|(k, c)| (*k, &**c as *const _)).collect()
        }
    }

    fn coll(x: f32, y: f32) -> Box<CollData> {
        let mut c = Box::new(CollData::zeroed());
        c.cur_pos.x = x;
        c.cur_pos.y = y;
        c.ecb.top.y = 10.0;
        c.ecb.bottom.y = 0.0;
        c.ecb.left.x = -3.0;
        c.ecb.right.x = 3.0;
        c
    }

    #[test]
    fn generators_and_tree() {
        let mut h = Rec::default();
        let mut jobjs: std::vec::Vec<Box<HSD_JObj>> = (0..3).map(|_| Box::new(HSD_JObj::zeroed())).collect();
        let mut jp: std::vec::Vec<*mut HSD_JObj> = jobjs.iter_mut().map(|j| &mut **j as *mut _).collect();
        // tree: 0 -> child 1 -> next 2
        unsafe {
            (*jp[0]).child = jp[1];
            (*jp[1]).next = jp[2];
        }
        for (i, owner) in [0usize, 1, 2, 0].iter().enumerate() {
            let mut g = Box::new(HSD_Generator::zeroed());
            g.idnum = i as u16;
            g.jobj = jp[*owner];
            h.gens.push(g);
        }
        for i in 0..3 {
            let n: *mut HSD_Generator = &mut *h.gens[i + 1];
            h.gens[i].next = n;
        }
        grlib_801c98a0(&mut h, jp[0]);
        assert_eq!(h.log, ["kill 0", "kill 3"]);
        assert_eq!(h.gens[0].r#type & 0x80, 0x80);
        assert_eq!(h.gens[1].r#type & 0x80, 0);
        h.log.clear();
        grlib_801c9908(&mut h, jp[0]);
        assert_eq!(h.log, ["kill 0", "kill 3", "kill 1", "kill 2"]);
        h.log.clear();
        unsafe { (*jp[0]).flags = JOBJ_INSTANCE as u32 };
        grlib_801c9908(&mut h, jp[0]);
        assert_eq!(h.log, ["kill 0", "kill 3"]);
        grlib_801c98a0(&mut h, core::ptr::null_mut());
    }

    #[test]
    fn mtx_and_accessors() {
        let mut h = Rec::default();
        let mut l = GrLib::default();
        l.grlib_801c9a10(&mut h);
        assert_eq!(l.grlib_801c9a70(3), Vec3 { x: 3.0, y: -3.0, z: 0.0 });
        let mut w = StageWorld::default();
        w.grlib_801c9e50(-5);
        assert_eq!(w.grlib_801c9e40(), -5);
        let mut v = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
        assert!(!w.grlib_801c9e60(&mut h, &mut v));
        assert_eq!(v, Vec3 { x: 0.0, y: 0.0, z: 0.0 });
        w.stage_info.grkind = Gr_Kind_BigBlue;
        v.x = 7.0;
        assert!(w.grlib_801c9e60(&mut h, &mut v)); // no map gobj 34: zeroed
        assert_eq!(v, Vec3 { x: 0.0, y: 0.0, z: 0.0 });
        assert!(w.grlib_801c9cec(&mut h, QuakeKind_Loop).is_null());
        assert!(w.grlib_801c9cec(&mut h, QuakeKind_None).is_null());
    }

    #[test]
    fn point_lists() {
        let mut h = Rec::default();
        let p = Vec3 { x: 0.0, y: 5.0, z: 0.0 };
        assert!(!grlib_801c9ee8(&mut h, &p, 0.0));
        h.items.push((It_PKind_Random, coll(0.0, 0.0)));
        assert!(!grlib_801c9ee8(&mut h, &p, 0.0));
        h.items.push((0, coll(0.0, 0.0)));
        assert!(grlib_801c9ee8(&mut h, &p, 0.0));
        h.items.clear();
        h.fighters.push(coll(10.0, 0.0));
        assert!(!grlib_801c9ee8(&mut h, &p, 0.0));
        assert!(grlib_801c9ee8(&mut h, &p, 7.0));
    }
}
