//! tu::gm_19EF against the C TU compiled natively with shipping contraction (state oracle:
//! gen/turef/tu_gm_19EF_ref.c, data tests/data/tu_gm_19EF.txt; regenerate with
//! `python rust/melee_modes/gen/turef/tu_shard529.py gm_19EF`).
#![allow(static_mut_refs, non_snake_case, unused)]
mod common;
use common::*;
use melee_modes::tu::gm_19EF as t;
use melee_modes::tu_ext as ext;
use melee_types::*;

fn btn() -> u64 {
    let r = rnd();
    (if r & 1 != 0 { 0x1000u64 } else { 0 }) | (if r & 2 != 0 { 0x100u64 } else { 0 })
}
unsafe fn mkjobj() -> *mut HSD_JObj {
    az_t::<HSD_JObj>()
}
unsafe fn mkjobjd() -> *mut HSD_JObj {
    let j = mkjobj();
    (*j).u.dobj = az_t::<HSD_DObj>();
    (*(*j).u.dobj).mobj = az(64) as *mut HSD_MObj;
    (*j).aobj = az_t::<HSD_AObj>();
    (*(*j).aobj).curr_frame = (rnd() % 4) as f32;
    (*(*j).aobj).end_frame = (rnd() % 4) as f32;
    j
}
unsafe fn mktree() -> *mut HSD_JObj {
    let root = mkjobj();
    let c1 = mkjobj();
    let c2 = mkjobj();
    let c3 = mkjobj();
    let n1 = mkjobj();
    let k1 = mkjobj();
    let k2 = mkjobj();
    let k3 = mkjobj();
    let mut prev = c2;
    (*root).child = c1;
    (*c1).child = c2;
    (*c1).next = n1;
    (*c2).child = c3;
    for _ in 0..12 {
        let s = mkjobj();
        (*prev).next = s;
        prev = s;
    }
    (*n1).child = k1;
    (*k1).next = k2;
    (*k2).next = k3;
    root
}
unsafe fn mkgobj(j: *mut HSD_JObj) -> *mut HSD_GObj {
    let g = az_t::<HSD_GObj>();
    (*g).hsd_obj = j as *mut _;
    g
}

include!("stubs/gm_19EF.rs");

unsafe fn mkscene() -> *mut SceneDesc {
    let s = az_t::<SceneDesc>();
    (*s).models = az(16) as *mut *mut DynamicModelDesc;
    *(*s).models = az_t::<DynamicModelDesc>();
    (**(*s).models).joint = az(64) as *mut HSD_Joint;
    (*s).cameras = az(core::mem::size_of::<SceneCameraDesc>() * 2) as *mut SceneCameraDesc;
    (*(*s).cameras).desc = az(64) as *mut HSD_CObjDesc;
    (*s).lights = az(16) as *mut *mut LightList;
    s
}
fn t_lbArchive_80016DBC(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    unsafe {
        out("lbArchive_80016DBC");
        trs(file);
        for pair in rest.chunks(2) {
            trp(pair[0] as *const u8);
            trs(pair[1] as *const core::ffi::c_char);
            *(pair[0] as *mut *mut core::ffi::c_void) = mkscene() as *mut _;
        }
        out("\n");
        az_t::<HSD_Archive>()
    }
}
fn t_lbArchive_LoadSymbols(file: *const core::ffi::c_char, rest: &[*mut core::ffi::c_void]) -> *mut HSD_Archive {
    unsafe {
        out("lbArchive_LoadSymbols");
        trs(file);
        for pair in rest.chunks(2) {
            trp(pair[0] as *const u8);
            trs(pair[1] as *const core::ffi::c_char);
            *(pair[0] as *mut *mut core::ffi::c_void) =
                if pair[0] as usize == &raw mut t::lbl_804D66AC as usize { az(64) as *mut _ } else { mkscene() as *mut _ };
        }
        out("\n");
        az_t::<HSD_Archive>()
    }
}
fn t_HSD_SisLib_803A6B98(text: *mut HSD_Text, x: f32, y: f32, fmt: *const core::ffi::c_char) -> i32 {
    out("HSD_SisLib_803A6B98");
    trp(text as *const u8);
    trf(x);
    trf(y);
    trs(fmt);
    out("\n");
    0
}
fn t_lb_80011E24(j: *mut HSD_JObj, o: *mut *mut HSD_JObj, id: i32, end: i32) -> i32 {
    unsafe {
        out("lb_80011E24");
        trp(j as *const u8);
        trp(o as *const u8);
        tri(id as i64);
        tri(end as i64);
        out("\n");
        *o = mkjobjd();
        1
    }
}

unsafe fn state() {
    let s = &t::lbl_80479A98;
    dump_t("x0", &s.x0);
    dump_t("x4", &s.x4);
    dump_t("x8", &s.x8);
    dump_t("xC", &s.xC);
    dump_t("x10", &s.x10);
    dump_t("x14", &s.x14);
    dump_t("x15", &s.x15);
    dump_t("x16", &s.x16);
    dump_t("x18", &s.x18);
    dump_t("x1C", &s.x1C);
    dump_t("x1E", &s.x1E);
    dump_t("x20", &s.x20);
    dump_t("x22", &s.x22);
    dump_t("x24", &s.x24);
    dump_t("gobj", &s.x28_gobj);
    dump_t("jobjs", &s.x28_jobjs);
    dump_t("x54", &s.x54);
    dump_t("x58", &s.x58);
    dump_t("x5C", &s.x5C);
    dump_t("x60", &s.x60);
    dump_t("x64", &s.x64);
    dump_t("x68", &s.x68);
    dump_t("x6C", &s.x6C);
    dump_t("x70", &s.x70);
    dump_t("b0", &t::lbl_804D66B0);
    dump_t("b8", &t::lbl_804D66B8);
    dump_t("c0", &t::lbl_804D66C0);
    dump_t("c8", &t::lbl_804D66C8);
    dump_t("d0", &t::lbl_804D66D0);
    dump_t("d8", &t::lbl_804D66D8);
    dump_t("e0", &t::lbl_804D66E0);
    dump_t("e8", &t::lbl_804D66E8);
    dump_t("a98", &t::lbl_804D6698);
    dump_t("a9c", &t::lbl_804D669C);
    dump_t("aa0", &t::lbl_804D66A0);
    dump_t("aa4", &t::lbl_804D66A4);
    dump_t("aa8", &t::lbl_804D66A8);
    dump_t("aac", &t::lbl_804D66AC);
    dump_t("af0", &t::lbl_804D66F0);
    dump_t("af4", &t::lbl_804D66F4);
    dump_t("af8", &t::lbl_804D66F8);
    dump_arena();
    state_end();
}
unsafe fn pick() -> i32 {
    const V: [i32; 8] = [0xA, 0x19, 0x4F, 0x50, 0x5A, 0x64, 0x6E, 0x78];
    let r = rnd() % 10;
    if r < 8 { V[r as usize] } else { (rnd() % 0x80) as i32 }
}
unsafe fn seed_state() {
    let s = &mut t::lbl_80479A98;
    core::ptr::write_bytes(s as *mut t::gm_19EF_state_t, 0, 1);
    const X0S: [u8; 6] = [0, 1, 8, 9, 0xA, 3];
    s.x0 = X0S[(rnd() % 6) as usize];
    s.x4 = (rnd() % 4) as i32;
    s.x8 = (rnd() % 2000) as i32;
    s.xC = if rnd() % 2 != 0 { s.x8 } else { s.x8 / 2 };
    s.x10 = 1 + rnd() % 30;
    s.x14 = (rnd() % 2) as u8;
    s.x15 = (rnd() % 4) as u8;
    s.x16 = (rnd() % 4) as u8;
    s.x1C = (rnd() % 20) as u16;
    s.x1E = (rnd() % 30) as u16;
    s.x20 = (rnd() % 2) as u8;
    s.x22 = (rnd() % 500) as u16;
    s.x24 = az_t::<HSD_Text>();
    s.x28_gobj = mkgobj(mktree());
    for i in 0..10 {
        s.x28_jobjs[i] = mkjobjd();
    }
    s.x54 = mkjobjd();
    s.x58 = mkjobjd();
    s.x5C = mkjobjd();
    s.x60 = (rnd() % 2) as u8;
    s.x64 = (rnd() % 60) as i32;
    s.x68 = (rnd() % 30) as i32;
    s.x6C = (rnd() % 30) as i32;
    s.x70 = (rnd() % 12) as i32;
    t::lbl_804D66B0.x0 = mkjobjd();
    t::lbl_804D66B0.x4 = (rnd() % 0x340) as i32;
    t::lbl_804D66B8.x0 = mkjobjd();
    t::lbl_804D66C0.x0 = mkjobjd();
    t::lbl_804D66C0.x4 = pick();
    t::lbl_804D66C8.x0 = mkjobjd();
    t::lbl_804D66C8.x4 = pick();
    t::lbl_804D66D0.x0 = mkjobjd();
    t::lbl_804D66D0.x4 = if rnd() % 4 == 0 { 0x2B1 + (rnd() % 2) as i32 } else { (rnd() % 0x2B4) as i32 };
    t::lbl_804D66D8.x0 = mkjobjd();
    t::lbl_804D66E0.x0 = mktree();
    t::lbl_804D66E0.x4 = if rnd() % 4 == 0 { 0x31 + (rnd() % 2) as i32 } else { (rnd() % 0x54) as i32 };
    t::lbl_804D66E8.x0 = mkjobjd();
    t::lbl_804D6698 = az_t::<HSD_Archive>();
    t::lbl_804D66F0 = az_t::<HSD_Archive>();
    t::lbl_804D66F4 = mkjobjd();
    t::lbl_804D66F8 = if rnd() % 4 == 0 { 0xBC + rnd() % 3 } else { rnd() % 0xC0 };
}

#[test]
fn matches_c() {
    unsafe {
        reset_all();
        gen_stubs::install();
        ext::HOOK_lbArchive_80016DBC = Some(t_lbArchive_80016DBC);
        ext::HOOK_lbArchive_LoadSymbols = Some(t_lbArchive_LoadSymbols);
        ext::HOOK_HSD_SisLib_803A6B98 = Some(t_HSD_SisLib_803A6B98);
        ext::HOOK_lb_80011E24 = Some(t_lb_80011E24);
        ext::HSD_GObj_JObjKind = 0x2A;
        ext::HSD_GObj_CameraKind = 0x2B;
        ext::HSD_GObj_LightKind = 0x2C;
        ext::HSD_GObj_FogKind = 0x2D;
        size_t::<DebugGameOverData>("DebugGameOverData");
        size_t::<HSD_JObj>("HSD_JObj");
        size_t::<HSD_Text>("HSD_Text");
        reg("b0", &t::lbl_804D66B0, core::mem::size_of::<t::unk_jobj_node>());
        reg("b8", &t::lbl_804D66B8, core::mem::size_of::<t::unk_jobj_node>());
        reg("c0", &t::lbl_804D66C0, core::mem::size_of::<t::unk_jobj_node>());
        reg("c8", &t::lbl_804D66C8, core::mem::size_of::<t::unk_jobj_node>());
        reg("d0", &t::lbl_804D66D0, core::mem::size_of::<t::unk_jobj_node>());
        reg("d8", &t::lbl_804D66D8, core::mem::size_of::<t::unk_jobj_node>());
        reg("e0", &t::lbl_804D66E0, core::mem::size_of::<t::unk_jobj_node>());
        reg("e8", &t::lbl_804D66E8, core::mem::size_of::<t::unk_jobj_node>());
        reg("a9c", &t::lbl_804D669C, 8);
        reg("aa0", &t::lbl_804D66A0, 8);
        reg("aa4", &t::lbl_804D66A4, 8);
        reg("aa8", &t::lbl_804D66A8, 8);
        reg("aac", &t::lbl_804D66AC, 8);
        reg("af4", &t::lbl_804D66F4, 8);
        reg("st", &t::lbl_80479A98, core::mem::size_of::<t::gm_19EF_state_t>());
        fnreg("fn_8019F810", t::fn_8019F810_cb as usize);
        fnreg("fn_8019F6EC", t::fn_8019F6EC as usize);
        fnreg("fn_801A0B60", t::fn_801A0B60 as usize);
        fnreg("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize);
        fnreg("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize);
        fnreg("HSD_GObj_803910D8", ext::HSD_GObj_803910D8 as usize);

        for c in 0..600usize {
            case(c);
            rseed(100 + c as u32);
            arena_reset();
            seed_state();
            t::fn_8019EFC4(&raw mut ext::HSD_PadCopyStatus[0]);
            state();
        }
        for c in 0..120usize {
            case(1000 + c);
            rseed(2000 + c as u32);
            arena_reset();
            seed_state();
            t::lbl_80479A98.x8 = match c % 5 {
                0 => 0,
                1 => (rnd() % 10) as i32,
                2 => 0x7FFFFFFF,
                _ => (rnd() % 100000) as i32,
            };
            t::fn_8019F1D0();
            state();
        }
        for c in 0..1500usize {
            const ARGS: [u32; 8] = [0, 0x1100, 0x1000, 0x100, 0x40000, 0x80000, 0xC0000, 0x41100];
            case(2000 + c);
            rseed(3000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_8019F2D4(ARGS[(rnd() % 8) as usize]);
            state();
        }
        for c in 0..600usize {
            case(4000 + c);
            rseed(5000 + c as u32);
            arena_reset();
            seed_state();
            if c % 3 == 0 {
                t::lbl_80479A98.x0 = 9;
            }
            t::fn_8019F6EC(mkgobj(mkjobjd()));
            state();
        }
        for c in 0..1500usize {
            case(6000 + c);
            rseed(7000 + c as u32);
            arena_reset();
            seed_state();
            ext::HSD_PadCopyStatus[t::lbl_80479A98.x15 as usize].trigger = rnd();
            t::fn_8019F810();
            state();
        }
        for c in 0..48usize {
            case(9000 + c);
            rseed(10000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_8019F9C4(rnd() % 40);
            state();
        }
        for c in 0..80usize {
            case(10000 + c);
            rseed(11000 + c as u32);
            arena_reset();
            seed_state();
            let d = az_t::<DebugGameOverData>();
            (*d).x0 = if c % 7 == 0 { 0 } else { rnd() % 100000 };
            (*d).x4 = rnd() as i32;
            (*d).x8 = rnd();
            (*d).xC = rnd() as i32;
            (*d).ckind = rnd() % 30;
            (*d).slot = (rnd() % 4) as u8;
            (*d).x15 = rnd() as u8;
            (*d).x16 = (rnd() % 100) as u16;
            (*d).x18 = (rnd() % 6000) as u16;
            t::gm_Scene_GOver_OnEnter(d as *mut _);
            state();
            t::gm_Scene_GOver_OnExit(d as *mut _);
            dump_t("d", d);
            state();
        }
        for c in 0..400usize {
            case(11000 + c);
            rseed(12000 + c as u32);
            arena_reset();
            seed_state();
            t::fn_801A0B60(mkgobj(mkjobjd()));
            state();
        }
        for c in 0..4usize {
            case(12000 + c);
            rseed(13000 + c as u32);
            arena_reset();
            seed_state();
            t::gm_Scene_ComingSoon_OnEnter(core::ptr::null_mut());
            state();
            t::gm_Scene_ComingSoon_OnExit(core::ptr::null_mut());
            state();
        }
    }
    check("tu_gm_19EF");
}
