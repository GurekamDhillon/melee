//! melee_world::tu::efasync: float/int helpers bit-exact vs the C (cref/efasync_ref.c via
//! cref/gen_efasync_ref.sh, contraction on), plus smoke tests of the dispatcher and the queue
//! through a recording host.
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::{EF_Effect, EF_QueuedEffect, HSD_GObj, HSD_GObjProc, HSD_Generator, HSD_JObj, HSD_ObjAllocData, HSD_psAppSRT, Vec3};
use melee_world::tu::efasync::*;
use melee_world::tu::efdata::EfData;

const REF: &str = include_str!("data/tu_efasync_ref.txt");

fn hx(s: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(s, 16).unwrap())
}
fn bits(f: f32) -> String {
    format!("{:08x}", f.to_bits())
}

#[test]
fn float_helpers_bit_exact() {
    let mut n = 0;
    for line in REF.lines().filter(|l| l.starts_with("F ")) {
        let t: Vec<&str> = line[2..].split(' ').collect();
        let x = hx(t[0]);
        assert_eq!(bits(dispatch_3e8_scale(x)), t[1], "3e8 {line}");
        assert_eq!(bits(dispatch_427_fall(x)), t[2], "427 {line}");
        assert_eq!(bits(tau_times(x)), t[3], "tau {line}");
        assert_eq!(bits(facing_rot(x)), t[4], "facing {line}");
        n += 1;
    }
    assert_eq!(n, 3000);
}

#[test]
fn int_helpers_match_c() {
    let mut n = 0;
    for line in REF.lines().filter(|l| l.starts_with("I ")) {
        let t: Vec<&str> = line[2..].split(' ').collect();
        let c: u32 = t[0].parse().unwrap();
        assert_eq!(tint_konst(c).to_string(), t[1], "{line}");
        let (k, id): (i32, i32) = (t[2].parse().unwrap(), t[3].parse().unwrap());
        assert_eq!(mex_miss_key(k, id).to_string(), t[4], "{line}");
        n += 1;
    }
    assert_eq!(n, 1000);
}

/// A host whose creators hand out preallocated objects and record calls.
struct Mock {
    ef: EfData,
    st: EfAsyncState,
    log: Vec<String>,
    effects: Vec<Box<EF_Effect>>,
    gobjs: Vec<Box<HSD_GObj>>,
    jobjs: Vec<Box<HSD_JObj>>,
    gens: Vec<Box<HSD_Generator>>,
    srts: Vec<Box<HSD_psAppSRT>>,
    rand: f32,
    proc_: *mut HSD_GObjProc,
    queued: Vec<Box<EF_QueuedEffect>>,
}
impl Mock {
    fn new() -> Self {
        Self {
            ef: EfData::default(),
            st: EfAsyncState::default(),
            log: vec![],
            effects: vec![],
            gobjs: vec![],
            jobjs: vec![],
            gens: vec![],
            srts: vec![],
            rand: 0.25,
            proc_: null_mut(),
            queued: vec![],
        }
    }
    fn new_jobj(&mut self) -> *mut HSD_JObj {
        let mut j: Box<HSD_JObj> = Box::new(unsafe { core::mem::zeroed() });
        j.scale = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
        let p = &mut *j as *mut HSD_JObj;
        self.jobjs.push(j);
        p
    }
    fn new_effect(&mut self) -> *mut EF_Effect {
        let j = self.new_jobj();
        let mut g: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
        g.hsd_obj = j.cast();
        let gp = &mut *g as *mut HSD_GObj;
        self.gobjs.push(g);
        let mut e = Box::new(EF_Effect::zeroed());
        e.gobj = gp;
        let p = &mut *e as *mut EF_Effect;
        self.effects.push(e);
        p
    }
    fn new_gen(&mut self) -> *mut HSD_Generator {
        let mut s: Box<HSD_psAppSRT> = Box::new(unsafe { core::mem::zeroed() });
        let sp = &mut *s as *mut HSD_psAppSRT;
        self.srts.push(s);
        let mut g: Box<HSD_Generator> = Box::new(unsafe { core::mem::zeroed() });
        g.appsrt = sp;
        let p = &mut *g as *mut HSD_Generator;
        self.gens.push(g);
        p
    }
}
unsafe impl EfHost for Mock {
    fn ef_data(&mut self) -> &mut EfData { &mut self.ef }
    fn ef_async(&mut self) -> &mut EfAsyncState { &mut self.st }
    unsafe fn efLib_Create_Attach(&mut self, id: u32, _g: *mut HSD_GObj, _j: *mut HSD_JObj) -> *mut EF_Effect {
        self.log.push(format!("Attach {id:#x}"));
        self.new_effect()
    }
    unsafe fn efLib_Create_Attach_Pos(&mut self, id: u32, _g: *mut HSD_GObj, p: *mut Vec3) -> *mut EF_Effect {
        self.log.push(format!("AttachPos {id:#x} {}", (*p).x));
        self.new_effect()
    }
    unsafe fn efLib_CreateGenerator(&mut self, id: i32, _p: *mut Vec3) -> *mut HSD_Generator {
        self.log.push(format!("Gen {id:#x}"));
        self.new_gen()
    }
    unsafe fn efLib_CreateGenerator_AddAppSRT(&mut self, id: i32) -> *mut HSD_Generator {
        self.log.push(format!("AddAppSRT {id:#x}"));
        self.new_gen()
    }
    unsafe fn efLib_SetTevKonstColor(&mut self, _j: *mut HSD_JObj, n: i32, k: u32, c: u32) {
        self.log.push(format!("Tev {n} {k:#x} {c:#x}"));
    }
    unsafe fn efLib_SetParamGfxId(&mut self, _g: *mut HSD_GObj, id: i32) { self.log.push(format!("ParamGfx {id:#x}")); }
    unsafe fn efLib_AnimQueue(&mut self, i: i32) -> *mut HSD_JObj { self.log.push(format!("AnimQ {i}")); null_mut() }
    unsafe fn HSD_JObjAnimAll(&mut self, _j: *mut HSD_JObj) {}
    unsafe fn HSD_JObjSetMtxDirtySub(&mut self, j: *mut HSD_JObj) { (*j).flags |= 1 << 6; }
    unsafe fn HSD_Randf(&mut self) -> f32 { self.rand }
    unsafe fn HSD_Randi(&mut self, _m: i32) -> i32 { 0 }
    unsafe fn lb_8000B1CC(&mut self, _j: *mut HSD_JObj, o: *const Vec3, out: *mut Vec3) {
        *out = if o.is_null() { Vec3 { x: 0.0, y: 0.0, z: 0.0 } } else { *o };
    }
    unsafe fn efSync_Spawn(&mut self, id: i32, _g: *mut HSD_GObj, a: &[VaArg]) -> *mut c_void {
        self.log.push(format!("Sync {id:#x} {}", a.len()));
        null_mut()
    }
    unsafe fn Camera_RequestQuake(&mut self, k: u32, p: *mut Vec3) { self.log.push(format!("Quake {k} {}", (*p).x)); }
    unsafe fn HSD_ObjAlloc(&mut self, _d: *mut HSD_ObjAllocData) -> *mut c_void {
        let mut q = Box::new(EF_QueuedEffect::zeroed());
        let p = &mut *q as *mut EF_QueuedEffect;
        self.queued.push(q);
        p.cast()
    }
    unsafe fn HSD_ObjFree(&mut self, _d: *mut HSD_ObjAllocData, _p: *mut c_void) { self.log.push("Free".into()); }
    unsafe fn HSD_GObj_CurrentInvokedProc(&mut self) -> *mut HSD_GObjProc { self.proc_ }
}

#[test]
fn dispatch_3e8_scales_and_flags() {
    let mut m = Mock::new();
    let mut pos = Vec3 { x: 5.0, y: 0.0, z: 0.0 };
    let mut dmg = 30.0f32;
    let args = [VaArg::Vec3(&mut pos), VaArg::F32(&mut dmg)];
    let mut vl = VaList::new(&args);
    unsafe {
        let e = efAsync_Dispatch(&mut m, 0x3E8, null_mut(), &mut vl) as *mut EF_Effect;
        assert!(!e.is_null());
        assert_eq!((*e).state_flags & EF_STATE_ASYNC, EF_STATE_ASYNC);
        let j = (*(*e).gobj).hsd_obj as *mut HSD_JObj;
        assert_eq!((*j).scale.x, dispatch_3e8_scale(30.0));
        assert_ne!((*j).flags & (1 << 6), 0, "mtx dirtied");
    }
    assert_eq!(m.log, vec!["AttachPos 0x9 5"]);
    assert_eq!(vl.pos, 2);
}

#[test]
fn dispatch_427_chains_six_and_tint_and_unknown() {
    let mut m = Mock::new();
    let mut pos = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    let args = [VaArg::Vec3(&mut pos)];
    unsafe {
        let head = efAsync_Dispatch(&mut m, 0x427, null_mut(), &mut VaList::new(&args)) as *mut EF_Effect;
        let mut n = 0;
        let mut cur = head;
        while !cur.is_null() {
            assert_eq!((*cur).lifetime, 0x28);
            assert_eq!((*cur).params.y.to_bits(), dispatch_427_fall(0.25).to_bits());
            n += 1;
            cur = (*cur).next;
        }
        assert_eq!(n, 6);

        m.log.clear();
        let mut j: HSD_JObj = core::mem::zeroed();
        let args = [VaArg::JObj(&mut j), VaArg::U32(0x00606060)];
        efAsync_Dispatch(&mut m, 0x417, null_mut(), &mut VaList::new(&args));
        assert_eq!(m.ef.load_kind, EF_LOADKIND_SYNC);
        assert_eq!(m.log, vec!["Attach 0xb", "Tev 1 0x808080 0x606060", "ParamGfx 0x417"]);

        m.log.clear();
        m.ef.anim_count = 2;
        assert!(efAsync_Dispatch(&mut m, 0x1, null_mut(), &mut VaList::new(&[])).is_null());
        assert_eq!(m.log, vec!["AnimQ 1", "AnimQ 0"]);
        assert_eq!(m.ef.anim_count, 0);
    }
}

#[test]
fn queue_spawn_defer_and_flush() {
    let mut m = Mock::new();
    let mut head = EF_QueuedEffect::zeroed();
    let mut proc_: HSD_GObjProc = unsafe { core::mem::zeroed() };
    proc_.s_link = 3;
    m.proc_ = &mut proc_;
    let mut ofs = Vec3 { x: 7.0, y: 0.0, z: 0.0 };
    let mut f1 = 1.5f32;
    let mut f2 = 2.5f32;
    unsafe {
        let args = [VaArg::Vec3(&mut ofs), VaArg::F32(&mut f1), VaArg::F32(&mut f2)];
        efAsync_Spawn(&mut m, null_mut(), &mut head, EF_SPAWN_POS_OFFSET_PARAM2, 0x55, null_mut(), VaList::new(&args));
        let args = [VaArg::Vec3(&mut ofs)];
        efAsync_Spawn(&mut m, null_mut(), &mut head, EF_SPAWN_CAMERA_SHAKE, 2, null_mut(), VaList::new(&args));
        assert!(m.log.is_empty(), "queued while in a fighter proc");
        let q = head.next;
        assert_eq!(((*q).spawn_kind, (*q).gfx_id), (EF_SPAWN_CAMERA_SHAKE as u8, 2));
        assert_eq!(((*(*q).next).extra1, (*(*q).next).extra2), (1.5, 2.5));
        efAsync_QueueFlush(&mut m, null_mut(), &mut head);
        assert!(head.next.is_null());
        assert_eq!(m.log, vec!["Quake 2 7", "Free", "Sync 0x55 3", "Free"]);

        // outside a fighter proc it runs at once
        m.log.clear();
        m.proc_ = null_mut();
        efAsync_Spawn(&mut m, null_mut(), &mut head, EF_SPAWN_ATTACH, 0x10, null_mut(), VaList::new(&[]));
        assert_eq!(m.log, vec!["Sync 0x10 1", "Free"]);
    }
}

#[test]
fn dat_table_and_load_guards() {
    let st = EfAsyncState::default();
    let name = |i: usize| unsafe { core::ffi::CStr::from_ptr(st.dat_entries[i].ef_DAT_file) }.to_str().unwrap();
    assert_eq!(name(0), "EfCoData.dat");
    assert_eq!(name(21), "EfKbZd.dat");
    assert!(st.dat_entries[22].ef_DAT_file.is_null());
    assert_eq!(name(31), "EfMnData.dat");
    assert_eq!(name(49), "EfFeData.dat");
    assert!(st.dat_entries[50].ef_DAT_file.is_null());
    assert_eq!(DAT_NAMES.len(), 35);

    let mut m = Mock::new();
    m.st.mex_ready = true; // MxDt absent: no Mex_* calls
    unsafe {
        efAsync_LoadAsync(&mut m, 22); // empty row
        efAsync_LoadAsync(&mut m, 65); // out of range
        efAsync_LoadSync(&mut m, -1);
    }
    assert!(m.log.is_empty());
}
