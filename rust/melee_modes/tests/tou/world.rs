//! The fixture: a fully populated tournament/title world built from a seed. Every global the
//! TUs read is set to a valid random value (indices in range, every pointer the code follows
//! non-null and pointing at a readable object), so any function can be called on it. The same
//! world is then copied into the C side, byte for byte with pointers re-based.
use std::ptr::addr_of_mut;

use melee_modes::tu::{gmtitle, gmtitlemode, gmtou_0, gmtou_1, gmtou_2, gmtoulib, gmtoumode, gmtoycollection};
use melee_types::*;
use std::vec::Vec;

use super::ext::{Persist, Rng};
use super::gen_tables::GSpec;
use super::space::{Region, Space};

pub const N_GOBJ: usize = 32;
pub const N_TEXT: usize = 48;
pub const CHAIN: usize = 18;

pub struct World {
    pub sp: Space,
    pub rng: Rng,
    pub persist: Persist,
    pub gobjs: Vec<*mut HSD_GObj>,
    pub jroots: Vec<*mut HSD_JObj>,
    pub texts: Vec<*mut HSD_Text>,
    pub cur_entry: usize,
}

pub fn zero_bytes(p: *mut u8, n: usize) {
    unsafe { std::ptr::write_bytes(p, 0, n) }
}

unsafe fn r<'a, T>(p: *mut T) -> &'a mut T {
    &mut *p
}

impl World {
    fn text(&mut self) -> *mut HSD_Text {
        let t = self.sp.alloc_ty("HSD_Text", 1) as *mut HSD_Text;
        unsafe {
            (*t).hidden = self.rng.below(2) as u8;
            (*t).default_kerning = self.rng.below(2) as u8;
            (*t).pos_x = self.rng.f32r(-100.0, 400.0);
            (*t).pos_y = self.rng.f32r(-100.0, 400.0);
        }
        t
    }

    fn jobj_chain(&mut self) -> *mut HSD_JObj {
        // one root with a child chain; every node has a DObj with a MObj
        let mut nodes: Vec<*mut HSD_JObj> = Vec::new();
        for k in 0..=CHAIN {
            let j = self.sp.alloc_ty("HSD_JObj", 1) as *mut HSD_JObj;
            let d = self.sp.alloc_ty("HSD_DObj", 1) as *mut HSD_DObj;
            let m = self.sp.alloc_ty("HSD_MObj", 1) as *mut HSD_MObj;
            unsafe {
                (*d).mobj = m;
                (*j).u.dobj = d;
                (*j).flags = (self.rng.next() as u32) & !(JOBJ_USE_QUATERNION as u32);
                (*j).translate = Vec3 { x: self.rng.f32r(-20.0, 20.0), y: self.rng.f32r(-20.0, 20.0), z: self.rng.f32r(-20.0, 20.0) };
                (*j).scale = Vec3 { x: self.rng.f32r(0.5, 3.0), y: self.rng.f32r(0.5, 3.0), z: self.rng.f32r(0.5, 3.0) };
                (*j).rotate.x = self.rng.f32r(-1.0, 1.0);
            }
            let _ = k;
            nodes.push(j);
        }
        unsafe {
            (*nodes[0]).child = nodes[1];
            for i in 1..nodes.len() {
                (*nodes[i]).parent = nodes[0];
                if i + 1 < nodes.len() {
                    (*nodes[i]).next = nodes[i + 1];
                }
            }
        }
        nodes[0]
    }

    fn scene_desc(&mut self) -> *mut SceneDesc {
        let sd = self.sp.alloc_ty("SceneDesc", 1) as *mut SceneDesc;
        let models = self.sp.alloc(8, super::space::PTRS, 16) as *mut *mut DynamicModelDesc;
        // the array of model pointers must be a pointer array for normalisation: allocate each
        // slot as part of the SceneDesc's helper below
        let mut arr: Vec<*mut DynamicModelDesc> = Vec::new();
        for _ in 0..16 {
            arr.push(self.sp.alloc_ty("DynamicModelDesc", 1) as *mut DynamicModelDesc);
        }
        let cam = self.sp.alloc_ty("SceneCameraDesc", 1) as *mut SceneCameraDesc;
        let camdesc = self.sp.alloc_ty("HSD_CObjDesc", 1) as *mut HSD_CObjDesc;
        let lights = self.sp.alloc(8, super::space::PTRS, 4) as *mut *mut LightList;
        let ll = self.sp.alloc_ty("LightList", 1) as *mut LightList;
        let fog = self.sp.alloc_ty("SceneFogDesc", 1) as *mut SceneFogDesc;
        let fogdesc = self.sp.alloc_ty("HSD_FogDesc", 1) as *mut HSD_FogDesc;
        unsafe {
            for (i, m) in arr.iter().enumerate() {
                *models.add(i) = *m;
                (**m).joint = self.sp.alloc_ty("HSD_Joint", 1) as *mut HSD_Joint;
            }
            (*sd).models = models;
            (*cam).desc = camdesc;
            (*sd).cameras = cam;
            *lights = ll;
            (*sd).lights = lights;
            (*fog).desc = fogdesc;
            (*sd).fogs = fog;
        }
        sd
    }

    pub fn gobj(&self, i: usize) -> *mut HSD_GObj {
        self.gobjs[i % N_GOBJ]
    }
}

pub fn restore_initial(specs: &[GSpec], init: &[Vec<u8>], regs: &[Region]) {
    for (i, _s) in specs.iter().enumerate() {
        let r = &regs[i];
        let n = init[i].len().min(r.size);
        unsafe { std::ptr::copy_nonoverlapping(init[i].as_ptr(), r.addr, n) };
        for &(d, ds) in &r.dups {
            zero_bytes(d, ds);
        }
    }
}

/// Build the world for `seed` in the Rust statics (space A).
pub fn build(seed: u64, specs: &[GSpec], init: &[Vec<u8>]) -> World {
    let mut sp = Space::new();
    sp.regions = specs
        .iter()
        .map(|s| {
            let v = (s.rust)();
            Region { addr: v[0].0, size: v[0].1, dups: v[1..].to_vec() }
        })
        .collect();
    restore_initial(specs, init, &sp.regions);
    let mut w = World { sp, rng: Rng::new(seed), persist: Persist::default(), gobjs: vec![], jroots: vec![], texts: vec![], cur_entry: 0 };
    unsafe { fill(&mut w) };
    w.sp.bf_clean_rust_all(specs);
    w
}

unsafe fn fill(w: &mut World) {
    // ---- scene graph objects
    for _ in 0..N_TEXT {
        let t = w.text();
        w.texts.push(t);
    }
    for _ in 0..N_GOBJ {
        let root = w.jobj_chain();
        w.jroots.push(root);
        let g = w.sp.alloc_ty("HSD_GObj", 1) as *mut HSD_GObj;
        (*g).hsd_obj = root as *mut core::ffi::c_void;
        (*g).classifier = w.rng.below(30) as u16;
        (*g).p_link = w.rng.below(32) as u8;
        (*g).user_data = (w.rng.below(6)) as usize as *mut core::ffi::c_void;
        w.gobjs.push(g);
    }
    // persistent objects handed out by the stubs
    let rules = w.sp.alloc_ty("GameRules", 1) as *mut GameRules;
    {
        let rl = r(rules);
        rl.mode = w.rng.below(3) as u8;
        rl.time_limit = w.rng.below(10) as u8;
        rl.stock_count = 1 + w.rng.below(4) as u8;
        rl.handicap = w.rng.below(3) as u8;
        rl.damage_ratio = w.rng.below(10) as u8;
        rl.stock_time_limit = w.rng.below(10) as u8;
        rl.pause = w.rng.below(2) as u8;
        rl.score_display = w.rng.below(2) as u8;
    }
    let prefs = w.sp.alloc_ty("GamePrefs", 1) as *mut GamePrefs;
    (*prefs).item_freq = w.rng.below(6) as u8;
    let pre = w.sp.alloc_ty("PreloadedGameModeState", 1);
    let exitb = w.sp.alloc_raw(0x100);
    let mut names = [0usize; 8];
    for (i, n) in names.iter_mut().enumerate() {
        let p = w.sp.alloc_raw(32);
        let s = format!("Name{}{}", i, "x".repeat((w.rng.below(8)) as usize));
        std::ptr::copy_nonoverlapping(s.as_ptr(), p, s.len());
        *n = p as usize - w.sp.base as usize;
    }
    let plink = w.sp.alloc(8, super::space::PTRS, 64) as *mut *mut HSD_GObj;
    {
        // two gobjs on p_link 27, one on 20 (fn_8018E618 frees them)
        let g27a = w.sp.alloc_ty("HSD_GObj", 1) as *mut HSD_GObj;
        let g27b = w.sp.alloc_ty("HSD_GObj", 1) as *mut HSD_GObj;
        let g20 = w.sp.alloc_ty("HSD_GObj", 1) as *mut HSD_GObj;
        (*g27a).p_link = 27;
        (*g27a).next = g27b;
        (*g27b).p_link = 27;
        (*g20).p_link = 20;
        if w.rng.chance(3, 4) {
            *plink.add(27) = g27a;
            *plink.add(20) = g20;
        }
    }
    let b = w.sp.base as usize;
    w.persist = Persist {
        rules: rules as usize - b,
        prefs: prefs as usize - b,
        preload: pre as usize - b,
        exit_buf: exitb as usize - b,
        names,
        plink: plink as usize,
        names_null: false,
    };

    // ---- stub-owned data
    gmtoulib::ext::HSD_GObjPLinkHead = plink;
    gmtoulib::ext::HSD_GObj_JObjKind = 3;
    gmtoulib::ext::HSD_GObj_FogKind = 5;
    gmtoulib::ext::HSD_GObj_LightKind = 4;
    gmtoulib::ext::HSD_GObj_CameraKind = 2;
    gmtitle::ext::HSD_GObj_JObjKind = 3;
    gmtitle::ext::HSD_GObj_FogKind = 5;
    gmtitle::ext::HSD_GObj_LightKind = 4;
    gmtitle::ext::HSD_GObj_CameraKind = 2;
    for p in gmtoulib::ext::HSD_PadMasterStatus.iter_mut() {
        p.err = [0i8, 0, 0, -1, -2][w.rng.below(5) as usize];
    }
    gmtitle::ext::gm_804D67EC = w.rng.below(6200) as u32;
    gmtitle::ext::gm_804D67F0 = w.sp.alloc_ty("HSD_SObjDesc", 1) as *mut HSD_SObjDesc;
    let dbl = [0u32, 1, 2, 3, 4][w.rng.below(5) as usize];
    gmtitle::ext::DbLevel = dbl;
    gmtitlemode::ext::DbLevel = dbl;
    for (i, c) in gmtitle::ext::db_build_timestamp.iter_mut().enumerate() {
        *c = if i < 14 { b"Jan  1 2000 12:0"[i] as i8 } else { 0 };
    }

    // ---- tournament state
    let tm = r(addr_of_mut!(gmtoulib::gm_804771C4));
    tm.cur_option = w.rng.range(0, 0x28) as i32;
    tm.match_type = w.rng.below(2) as i32;
    tm.combatants = w.rng.range(2, 8) as i32;
    tm.entrants = w.rng.range(0, 31) as i32;
    tm.hmn_cpu_count = w.rng.range(1, 4) as i32;
    tm.stage_selection_type = w.rng.below(4) as i32;
    tm.cpu_level = w.rng.below(10) as i32;
    tm.x1C = w.rng.below(2) as i32;
    tm.x20 = w.rng.below(2) as i32;
    tm.x24 = w.rng.below(200) as i32;
    tm.x28 = w.rng.below(30) as u32;
    tm.x2C = w.rng.below(2) as u8;
    tm.x2D = w.rng.below(2) as u8;
    tm.x2E = w.rng.range(1, 40) as u8;
    tm.x2F = w.rng.below(8) as u8;
    tm.x30 = w.rng.range(2, 4) as u8;
    tm.x31 = w.rng.below(2) as u8;
    tm.x32 = w.rng.below(2) as u8;
    tm.x33 = if w.rng.chance(1, 5) { 0xFF } else { w.rng.below(7) as u8 };
    tm.pad_x34[0] = if w.rng.chance(1, 5) { 0xFF } else { w.rng.below(7) as u8 };
    let mut perm: Vec<u8> = (0..64).collect();
    for i in (1..64).rev() {
        let j = w.rng.below(i as u64 + 1) as usize;
        perm.swap(i, j);
    }
    for i in 0..64 {
        let e = &mut tm.x37[i];
        e.x0 = [0u8, 1, 3][w.rng.below(3) as usize];
        e.x1 = w.rng.below(10) as u8;
        e.x2 = w.rng.range(1, 9) as u8;
        e.x3 = w.rng.below(25) as u8;
        e.x4 = w.rng.below(25) as u8;
        e.x5 = w.rng.below(2) as u8;
        e.x6 = w.rng.below(2) as u8;
        e.x7 = w.rng.below(6) as u8;
        e.x8 = w.rng.below(6) as u8;
        e.x9 = if w.rng.chance(1, 3) { 0x320 + w.rng.below(0x60) as u16 } else { w.rng.below(0x3E8) as u16 };
        e.xB = w.rng.below(256) as u8;
        e.xC = w.rng.below(256) as u8;
        e.xD = perm[i];
        e.xE = w.rng.below(16) as u8;
        e.xF = w.rng.below(16) as u8;
    }
    for i in 0..4 {
        let s = &mut tm.x4B8[i];
        s.x0 = w.rng.below(4) as u8;
        s.x1 = w.rng.below(25) as u8;
        s.x2 = w.rng.below(2) as u8;
        s.x3 = w.rng.below(6) as u8;
        s.x4 = w.rng.below(10) as u8;
        s.x5 = w.rng.range(1, 9) as u8;
        s.x6 = w.rng.below(0x90) as u16;
    }
    tm.x4E0 = w.texts[0];
    tm.x4E4 = w.texts[1];
    for i in 0..6 {
        tm.x4E8[i] = w.texts[2 + i];
        tm.x500[i] = w.texts[8 + i];
    }
    for i in 0..3 {
        tm.x518[i] = w.texts[14 + i];
        tm.x534[i] = w.texts[30 + i];
    }
    for i in 0..4 {
        tm.x524[i] = w.texts[17 + i];
    }

    // bracket
    let be = &mut *addr_of_mut!(gmtoulib::lbl_80473AB8);
    let cur = w.rng.below(64) as usize;
    w.cur_entry = cur;
    for (i, e) in be.iter_mut().enumerate() {
        e.x0 = if w.rng.chance(3, 4) { 1 } else { 0 };
        e.x1 = (i == cur) as u8;
        e.x2 = w.rng.below(2) as u8;
        e.x3 = w.rng.below(4) as u8;
        e.x4 = w.rng.below(4) as u8;
        e.x5 = w.rng.below(64) as u8;
        e.x6 = w.rng.below(4) as u8;
        e.xC = w.rng.range(-50, 600) as i32;
        e.x10 = w.rng.range(-50, 400) as i32;
        e.x14 = w.rng.range(1, 400) as i32;
        e.x18 = if w.rng.chance(1, 6) { 0 } else { w.rng.range(1, 300) as i32 };
        e.x1C = w.rng.f32r(0.5, 6.0);
        e.x20 = GXColor { r: w.rng.below(256) as u8, g: if w.rng.chance(1, 2) { 0 } else { w.rng.below(256) as u8 }, b: w.rng.below(256) as u8, a: 255 };
        e.x24 = w.rng.below(256) as u8;
        e.x25 = w.rng.below(256) as u8;
        e.x26 = w.rng.below(256) as u8;
        e.x27 = w.rng.below(256) as u8;
        e.x28 = w.rng.below(256) as u8;
        for s in e.slots.iter_mut() {
            s.x2C = w.gobjs[w.rng.below(N_GOBJ as u64) as usize];
            s.x30 = w.rng.below(2) as u8;
            s.x32 = w.rng.below(2) as u8;
            s.x34 = w.rng.range(-50, 700) as i32;
            s.x38 = w.rng.range(-50, 700) as i32;
            s.x3C = w.rng.range(-50, 700) as i32;
            s.x40 = w.rng.range(-50, 700) as i32;
            s.x44 = w.rng.range(-50, 700) as i32;
            s.x48 = w.rng.range(-50, 700) as i32;
            s.x4C = w.rng.below(5) as u8;
            s.x4D = w.rng.below(25) as u8;
            s.x4E = w.rng.below(4) as u8;
            s.x4F = w.rng.below(6) as u8;
            s.x50 = w.rng.below(64) as u8;
            s.x51 = w.rng.range(1, 9) as u8;
            s.x52 = w.rng.below(10) as u8;
            s.x54 = w.rng.below(0x100) as u16;
        }
    }
    // region source tables (BracketSrcEntry arrays, 0x28 bytes each)
    let boxes = &mut *addr_of_mut!(gmtoulib::lbl_804771B8);
    for k in 0..3 {
        let a = w.sp.alloc_raw(0x28 * 1200);
        for i in 0..0x28 * 1200 {
            *a.add(i) = (w.rng.below(256)) as u8;
        }
        match k {
            0 => boxes.box2 = a as *mut core::ffi::c_void,
            1 => boxes.box3 = a as *mut core::ffi::c_void,
            _ => boxes.box4 = a as *mut core::ffi::c_void,
        }
    }
    gmtoulib::lbl_804D663C = w.rng.below(4) as i32;
    gmtoulib::lbl_804D6638 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtoulib::lbl_804D6630 = w.rng.range(0, 140) as i32;
    gmtoulib::lbl_804D6634 = w.rng.below(4) as i32;
    {
        let t = &mut *addr_of_mut!(gmtoulib::lbl_803D9D20);
        for i in 0..t.x72.len() {
            t.x72[i] = if w.rng.chance(4, 5) { 1 } else { 0 };
        }
        let c = &mut *addr_of_mut!(gmtoulib::lbl_803D9DAC);
        for v in [&mut c.current, &mut c.target, &mut c.step] {
            v.x = w.rng.f32r(-300.0, 600.0);
            v.y = w.rng.f32r(-300.0, 600.0);
            v.z = w.rng.f32r(-200.0, 600.0);
        }
        let d = &mut *addr_of_mut!(gmtoulib::lbl_803D9DD0);
        d.cobj = w.sp.alloc_ty("HSD_CObj", 1) as *mut HSD_CObj;
        d.xD0 = w.rng.f32r(10.0, 90.0);
        let l = &mut *addr_of_mut!(gmtoulib::lbl_803D9F0C);
        l.x0 = w.rng.range(-1, 24) as i32;
        l.x4 = w.rng.range(-1, 24) as i32;
        l.x8 = w.rng.range(-1, 24) as i32;
    }
    let me = &mut *addr_of_mut!(gmtoulib::gm_80477738);
    fill_bytes_small(w, me as *mut MatchEnd as *mut u8, std::mem::size_of::<MatchEnd>());
    me.n_winners = w.rng.range(0, 4) as u8;
    me.is_teams = w.rng.below(2) as u8;
    for p in me.player_standings.iter_mut() {
        p.pkind = w.rng.below(4) as u8;
        p.is_small_loser = w.rng.below(4) as u8;
        p.is_big_loser = w.rng.below(4) as u8;
    }

    // ---- menu state
    let s = &mut *addr_of_mut!(gmtou_0::lbl_804799B8);
    s.x0 = w.rng.below(2) as u8;
    s.x1 = w.rng.below(2) as u8;
    s.x2 = w.rng.below(12) as u8;
    s.x3 = w.rng.below(52) as u8;
    s.x4 = w.rng.below(64) as u8;
    s.x5 = w.rng.below(36) as u8;
    s.x6 = w.rng.below(9) as u8;
    s.x7 = w.rng.below(6) as u8;
    s.x8 = w.rng.below(6) as u8;
    s.x9 = w.rng.below(6) as u8;
    s.xA = w.rng.below(11) as u8;
    s.xB = w.rng.below(6) as u8;
    s.xC_counter = w.rng.below(900) as u16;
    s.xE = w.rng.below(100) as u8;
    s.xF = w.rng.below(30) as u8;
    for b in s.pad2.iter_mut() {
        *b = w.rng.below(120) as u8;
    }
    gmtou_0::lbl_804D6640 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_0::lbl_804D6644 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_0::lbl_804D6648 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_0::lbl_804D664C = w.scene_desc();
    gmtou_0::lbl_804D6650 = w.scene_desc();
    gmtou_0::lbl_804D6654 = w.rng.range(0, 0x13) as u32;
    gmtou_0::lbl_804D6658 = w.rng.range(0, 12) as i32;
    gmtou_0::lbl_804D665C = w.rng.range(0, 4) as i32;

    gmtou_1::lbl_804D6660 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_1::lbl_804D6664 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_1::lbl_804D6668 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_1::lbl_804D666C = w.scene_desc();
    gmtou_1::lbl_804D6670 = w.scene_desc();
    gmtou_1::lbl_804D6674 = w.scene_desc();
    gmtou_1::lbl_804D6678 = w.rng.below(2) as i32;
    gmtou_1::lbl_804D667C = w.rng.below(2) as i32;
    for b in gmtou_1::lbl_804D6680.iter_mut() {
        *b = w.rng.below(4) as u8;
    }
    gmtou_1::lbl_804D4190 = w.rng.range(-1, 29) as i32;
    gmtou_1::lbl_804D4194 = w.rng.range(-1, 29) as i32;
    let d = &mut *addr_of_mut!(gmtou_1::lbl_804799D8);
    d.x0 = w.rng.below(0x2100) as u32;
    d.x4 = w.rng.below(0xA8) as u32;
    d.x8 = w.rng.below(0x60) as i32;
    d.xC = w.rng.below(0xA0) as i32;
    d.x10 = w.rng.below(900) as u16;
    for i in 0..4 {
        d.x12[i] = w.rng.below(0x300) as u16;
        d.x1D[i] = w.rng.below(0x29) as u8;
        d.x21[i] = w.rng.below(12) as u8;
        d.x25[i] = w.rng.below(3) as u8;
        d.x44[i] = 6 + w.rng.below(3) as u8;
        let a = &mut d.x2A[i];
        a.start = w.rng.below(0xA0) as u8;
        a.cur = w.rng.below(0xA0) as u8;
        a.end = w.rng.below(0xA0) as u8;
        a.state = w.rng.below(6) as u8;
        a.done = w.rng.below(2) as u8;
        a.r#loop = w.rng.below(2) as u8;
    }
    d.x1A = w.rng.below(0xA8) as u8;
    d.x1B = w.rng.below(0x62) as u8;
    d.x1C = w.rng.below(0x50) as u8;
    d.x29 = w.rng.below(14) as u8;
    d.x48 = addr_of_mut!(gmtoulib::gm_80477738);
    d.x4C = w.rng.below(4) as u8;
    d.x4D = w.rng.below(4) as u8;
    for b in d.x4E.iter_mut() {
        *b = 0x40 + w.rng.below(0x30) as u8;
    }
    d.x64 = Vec3 { x: w.rng.f32r(-5.0, 5.0), y: w.rng.f32r(-5.0, 5.0), z: w.rng.f32r(-5.0, 5.0) };
    d.x70 = Vec3 { x: w.rng.f32r(0.5, 3.0), y: w.rng.f32r(0.5, 3.0), z: w.rng.f32r(0.5, 3.0) };
    {
        let l = &mut *addr_of_mut!(gmtou_1::lbl_803DA0D0);
        for v in l.bounce_y.iter_mut() {
            *v = w.rng.f32r(-5.0, 5.0);
        }
    }

    gmtou_2::lbl_804D6688 = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_2::lbl_804D668C = w.sp.alloc_ty("HSD_Archive", 1) as *mut HSD_Archive;
    gmtou_2::lbl_804D6690 = w.scene_desc();
    gmtou_2::lbl_804D6694 = w.scene_desc();
    let t = &mut *addr_of_mut!(gmtou_2::lbl_80479A58);
    t.x0 = w.rng.below(0x40) as u32;
    t.x4 = w.rng.below(900) as u16;
    t.xE = w.rng.below(0xA8) as u8;
    t.xF = w.rng.below(70) as u8;
    t.x1C = w.rng.below(13) as u8;
    for i in 0..4 {
        t.x6[i] = w.rng.below(700) as u16;
        t.x10[i] = w.rng.below(0x29) as u8;
        t.x18[i] = w.rng.below(0x70) as u8;
        t.x38[i] = 6 + w.rng.below(2) as u8;
        let a = &mut t.x1D[i];
        a.a = w.rng.below(0xA0) as u8;
        a.b = w.rng.below(0xA0) as u8;
        a.c = w.rng.below(0xA0) as u8;
        a.x0 = w.rng.below(5) as u8;
        a.x1 = w.rng.below(2) as u8;
        a.x2 = w.rng.below(2) as u8;
    }
    {
        let l = &mut *addr_of_mut!(gmtou_2::lbl_803DA2E0);
        for v in l.x20.iter_mut() {
            *v = w.rng.f32r(-5.0, 5.0);
        }
    }

    // ---- title scene
    gmtitle::model_desc_0 = std::mem::zeroed();
    gmtitle::model_desc_1 = std::mem::zeroed();
    for m in [addr_of_mut!(gmtitle::model_desc_0), addr_of_mut!(gmtitle::model_desc_1)] {
        (*m).joint = w.sp.alloc_ty("HSD_Joint", 1) as *mut HSD_Joint;
    }
    gmtitle::cobj_desc = w.sp.alloc_ty("HSD_CObjDesc", 1) as *mut HSD_CObjDesc;
    gmtitle::list_list = w.sp.alloc(8, super::space::PTRS, 2) as *mut *mut LightList;
    let fd = w.sp.alloc_ty("HSD_FogDesc", 1) as *mut HSD_FogDesc;
    (*fd).color = GXColor { r: w.rng.below(256) as u8, g: w.rng.below(256) as u8, b: w.rng.below(256) as u8, a: 255 };
    gmtitle::fog_desc = fd;
    gmtitle::countdown_timer = w.rng.range(0, 24) as i32;
    gmtitle::frame_count = w.rng.range(0, 606) as u32;
    gmtitle::bg_initialized = w.rng.below(2) as u8;
    for b in gmtitle::debug_text_buffer.iter_mut() {
        *b = 0;
    }

    // ---- gmtoumode / small TUs
    let sm = &mut *addr_of_mut!(gmtoumode::gm_804876D8);
    fill_bytes_small(w, sm as *mut StartMeleeData as *mut u8, std::mem::size_of::<StartMeleeData>());
    for i in 0..sm.rules.bf_0.len() {
        sm.rules.bf_0[i] = 0;
    }
    for p in sm.players.iter_mut() {
        p.bf_c = [0, 0];
    }
    let mei = &mut *addr_of_mut!(gmtoumode::gm_80487810);
    fill_bytes_small(w, mei as *mut MatchExitInfo as *mut u8, std::mem::size_of::<MatchExitInfo>());
    mei.match_end.n_winners = w.rng.range(0, 4) as u8;
    for p in mei.match_end.player_standings.iter_mut() {
        p.pkind = w.rng.below(4) as u8;
    }
    for g in [&mut gmtoumode::gm_80489A98[..], &mut gmtoumode::gm_8048BD20[..], &mut gmtoumode::gm_8048BE68[..]] {
        for b in g.iter_mut() {
            *b = w.rng.below(256) as u8;
        }
    }
    gmtitlemode::exit_data.buttons = w.rng.next() as i32 & 0x000F_FFFF;
    gmtoycollection::gm_804D6918 = [std::ptr::null_mut(); 2];
}

unsafe fn fill_bytes_small(w: &mut World, p: *mut u8, n: usize) {
    for i in 0..n {
        *p.add(i) = if w.rng.chance(1, 3) { 0 } else { w.rng.below(8) as u8 };
    }
}
