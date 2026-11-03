//! State oracle for tu/gmregclear.rs (src/oracle.rs, gen/tu/run.py; the C side is the patched
//! copy, gen/tu/patches/gmregclear.py).
use core::ffi::c_void;
use core::mem::offset_of;
use core::ptr::{addr_of_mut, null_mut};
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::gmregclear as tu;
use melee_modes::tu::gmregclear::ext;
use melee_types::*;

fn st() -> &'static mut tu::RegClear {
    unsafe { &mut *addr_of_mut!(tu::lbl_80472D28) }
}

fn pick<T: Copy>(c: &mut Case, xs: &[T]) -> T {
    xs[c.rng.range(0, xs.len() as i64 - 1) as usize]
}

/// A scene controller (random state, bitfields registered) for gmVs_GetSceneController.
fn vsc(c: &mut Case) -> *mut VsSceneController {
    let v = c.obj::<VsSceneController>();
    unsafe {
        c.bitfields::<StartMeleeRules>(&mut (*v).start, offset_of!(StartMeleeRules, bf_0), 0, 6);
        (*v).state.timer_seconds = c.rng.range(0, 600) as u32;
        (*v).state.match_result = pick(c, &[0u8, 1, 6, 6, 3]);
    }
    c.ret_ptr("gmVs_GetSceneController", v);
    v
}

fn srules(c: &mut Case) -> *mut StartMeleeRules {
    let r = c.obj::<StartMeleeRules>();
    unsafe { c.bitfields::<StartMeleeRules>(r, offset_of!(StartMeleeRules, bf_0), 0, 6) };
    c.ret_ptr("gm_GetStartMeleeRules", r);
    r
}

fn text_pools(c: &mut Case) {
    let n = core::mem::size_of::<HSD_Text>();
    c.ret_pool("HSD_SisLib_803A6754", n);
    c.ret_pool("HSD_SisLib_803A5ACC", n);
}

/// Text slot: null or a pool-like text.
fn maybe_text(c: &mut Case) -> *mut HSD_Text {
    if c.rng.chance(1, 3) {
        null_mut()
    } else {
        c.zobj::<HSD_Text>()
    }
}

/// A JObj with a parent (for `p->x8->parent` scale reads).
fn jobj_with_parent(c: &mut Case) -> *mut HSD_JObj {
    let j = c.jobj();
    let par = c.jobj();
    unsafe {
        (*j).parent = par;
        (*par).scale = Vec3 { x: c.rng.f32(), y: c.rng.f32(), z: c.rng.f32() };
    }
    j
}

/// Fill the scalar part of the screen state with small plausible values.
fn state_scalars(c: &mut Case) {
    let s = st();
    s.xC0 = pick(c, &[0u16, 1, 2, 5, 9]);
    s.xC4 = c.rng.range(0, 400) as u32;
    s.xC8 = pick(c, &[0u8, 1]);
    s.xCC = c.rng.range(-50, 5000) as i32;
    s.xD0 = c.rng.range(-50, 5000) as i32;
    s.xD4 = if c.rng.chance(1, 2) { s.xD0 } else { c.rng.range(-1, 5000) as i32 };
    s.xD8 = pick(c, &[0i32, 5, 0x3B, 0x3C, 0x50]);
    s.xDC = c.rng.range(-50, 5000) as i32;
    s.xE0 = if c.rng.chance(1, 2) { s.xDC } else { c.rng.range(-1, 5000) as i32 };
    s.xE4 = pick(c, &[0i32, 5, 0x3B, 0x3C, 0x50]);
    s.xE8 = c.rng.range(-50, 5000) as i32;
    s.xF0 = c.rng.range(-5000, 50000) as i32;
    s.xF4 = if c.rng.chance(1, 2) { s.xF0 } else { c.rng.range(-1, 5000) as i32 };
    s.xF8 = pick(c, &[0i32, 5, 0xB, 0xC, 0x1F, 0x20, 0x40]);
    s.xFC = c.rng.range(-100, 100000) as i32;
    s.x100 = if c.rng.chance(1, 3) { s.xFC } else { c.rng.range(-1, 5000) as i32 };
    s.x104 = c.rng.range(-100, 100000) as i32;
    s.x108 = pick(c, &[0x64i16, 0xC8, 0x1F4]);
    s.x10A = pick(c, &[0x64i16, 0xC8, 0x1F4]);
    s.x10C = pick(c, &[0.0f32, 0.5, 0.96, 0.99, 1.0, 1.5]);
    s.x110 = pick(c, &[0u32, 1, 0x14, 0x15, 0x29, 0x2A, 0x3E, 0x3F, 60, 90, 0xFFFF_FFFF, 0xFFFF_FFFE]);
    s.x114 = pick(c, &[0u8, 1]);
    s.x115 = pick(c, &[0u8, 1]);
    s.x116 = pick(c, &[0u8, 1]);
    s.x117 = pick(c, &[0u8, 1]);
    s.x118 = pick(c, &[0u8, 0, 1]);
    s.x119 = pick(c, &[0u8, 1]);
    s.x11A = pick(c, &[0u8, 1]);
    s.x11B = pick(c, &[0u8, 1]);
    s.x11C = pick(c, &[0u8, 3, 10]);
    s.x11D = pick(c, &[0u8, 1, 2, 9]);
    s.x11E = pick(c, &[0u8, 1]);
    s.x11F = pick(c, &[0u8, 1]);
    s.x108 = if s.x108 == 0 { 0x64 } else { s.x108 };
}

/// Valid JObj / text pointers in the state.
fn state_objects(c: &mut Case) {
    let s = st();
    s.x0 = c.gobj_any();
    s.x4 = c.jobj();
    s.x8 = jobj_with_parent(c);
    s.xC = c.jobj();
    s.x10 = c.jobj();
    s.x14 = c.jobj();
    s.x18 = c.jobj();
    s.x1C = c.jobj();
    s.x20 = c.jobj();
    s.x24 = c.jobj();
    s.x2C = c.gobj_any();
    s.x6C = maybe_text(c);
    s.x70 = maybe_text(c);
    s.x74 = maybe_text(c);
    s.x78 = maybe_text(c);
    s.x7C = maybe_text(c);
    s.x80 = maybe_text(c);
    s.x84 = c.zobj::<HSD_Text>();
    s.x5C = c.buf(0x20).cast();
    s.x60 = c.buf(0x20).cast();
    s.x4C.joint = c.buf(0x40).cast();
    unsafe {
        // p->x4->u.dobj->mobj->tobj->aobj (fn_801803FC)
        let t = c.zobj::<HSD_TObj>();
        (*t).aobj = c.aobj();
        let m = c.zobj::<HSD_MObj>();
        (*m).tobj = t;
        let d = c.zobj::<HSD_DObj>();
        (*d).mobj = m;
        (*s.x4).u.dobj = d;
    }
}

fn pad_status(c: &mut Case) {
    let pads = unsafe { &mut *addr_of_mut!(tu::HSD_PadMasterStatus) };
    for p in pads.iter_mut() {
        p.trigger = pick(c, &[0u32, 0x100, 0x1000, 0x1100, 0x4]);
    }
    c.ret_int("Player_GetPlayerId", 0, 3);
    c.ret_int("Player_GetPlayerSlotType", 0, 2);
}

fn b_f008(c: &mut Case) {
    c.ret_int("fn_80171A88", 0, 9);
}

fn b_f09c(c: &mut Case) {
    vsc(c);
    state_scalars(c);
    c.ret_int("fn_80171A88", 0, 9);
}

fn b_f14c(c: &mut Case) {
    vsc(c);
    state_scalars(c);
    let m = c.obj::<MatchEnd>();
    c.arg(m);
}

fn b_f1b8(c: &mut Case) {
    c.ret_int("fn_80171A88", 0, 9);
    c.ret_int("fn_8016F1F0", 0, 0x100);
    c.ret_int("fn_8016F160", 0, 1);
    c.ret_int("fn_801701C0", 0, 1);
    c.ret_int("fn_8016F180", 0, 0xFFFF);
    c.ret_int("fn_8016FFD4", -1000, 1000);
}

fn b_f294(c: &mut Case) {
    state_scalars(c);
}

fn b_f2a4(c: &mut Case) {
    text_pools(c);
    let arr = c.zbuf(8 * 8 + 32);
    c.ret_int("lbLang_IsSavedLanguageUS", 0, 1);
    c.ret_int("HSD_SisLib_803A611C", 0, 5);
    c.str_arg("HSD_SisLib_803A62A0", 1);
    c.str_arg("HSD_SisLib_803A62A0", 2);
    c.arg(arr);
    let (x, y) = (c.rng.f32(), c.rng.f32());
    c.arg(x);
    c.arg(y);
}

fn b_f47c(c: &mut Case) {
    c.ret_int("fn_80171A88", 0, 9);
    let arr = c.zbuf(8 * 8 + 32) as *mut i32;
    unsafe {
        // the text slots: slot 0 is the base text; the cache holds the last shown values
        *(arr as *mut *mut HSD_Text) = c.zobj::<HSD_Text>();
        let cache = (arr as *mut u8).add(64) as *mut i32;
        for i in 0..8 {
            *cache.add(i) = if c.rng.chance(1, 3) { 0 } else { c.rng.range(-20, 20) as i32 };
        }
    }
    let mut idxs = std::vec::Vec::new();
    let mut vals = std::vec::Vec::new();
    for _ in 0..40 {
        idxs.push(c.rng.range(-2, 12) as i32 as u32 as u64);
        vals.push(c.rng.range(-20, 20) as i32 as u32 as u64);
    }
    // fn_8016F548 is called twice per row (once for idx, once at the end) in the C
    c.ret_seq_u64("fn_8016F548", &idxs);
    c.ret_seq_u64("fn_8016FAD4", &vals);
    c.ret_int("fn_8016FFD4", -30, 30);
    c.arg(arr);
    let a1 = c.rng.range(0, 10) as i32;
    c.arg(a1);
}

fn b_f608(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    vsc(c);
    text_pools(c);
    c.mem_arg("lb_8000B1CC", 1, 12);
    c.out("lb_8000B1CC", 2, 12);
    c.arg(addr_of_mut!(tu::lbl_80472D28));
}

fn b_fa1c(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    unsafe { tu::lbl_804D65C0 = pick(c, &[0i32, 1, 3, 10, -4, -10000, 700]) };
    c.ret_seq_u64("HSD_JObjGetFlags", &[0, 0x10, 0, 0x11, 0x20, 0]);
    text_pools(c);
    c.out("lb_8000B1CC", 2, 12);
    c.arg(addr_of_mut!(tu::lbl_80472D28));
}

fn b_fba4(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    c.ret_seq_u64("HSD_JObjGetFlags", &[0, 0x10, 0, 0x11, 0x20, 0]);
    text_pools(c);
    c.out("lb_8000B1CC", 2, 12);
    c.arg(addr_of_mut!(tu::lbl_80472D28));
}

fn b_fe54(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    let ev = c.obj::<tu::RegClearEv>();
    unsafe { (*ev).x1C = c.buf(0x20).cast() };
    let g = c.gobj_any();
    unsafe { (*g).user_data = ev.cast() };
    c.arg(g);
}

fn b_ff1c(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    vsc(c);
    pad_status(c);
    text_pools(c);
    c.ret_int("fn_80171A88", 0, 9);
    c.ret_seq_u64("HSD_JObjGetFlags", &[0, 0x10, 0, 0x11, 0x20, 0]);
    c.out("lb_8000B1CC", 2, 12);
    c.mem_arg("lb_8000B1CC", 1, 12);
    c.ret_int("fn_8016F9A8", 0, 12);
    c.ret_int("fn_8016F870", -1, 8);
    c.ret_int("fn_8016F740", -1, 8);
    c.ret_int("fn_8016F548", -2, 10);
    c.ret_int("fn_8016FAD4", -30, 30);
    c.ret_int("fn_8016FFD4", -30, 30);
    c.ret_int("gm_GetButtonsTriggered", 0, 0x3FFFF);
    c.ret_int("gm_801A36C0", 0, 0x3FFFF);
    let j = c.jobj();
    let g = c.gobj(j.cast());
    c.arg(g);
}

fn b_f3fc(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    text_pools(c);
    pad_status(c);
    vsc(c);
    c.ret_seq_u64("HSD_JObjGetFlags", &[0, 0x10, 0, 0x11, 0x20, 0]);
    c.out("lb_8000B1CC", 2, 12);
    let n = core::mem::size_of::<HSD_GObj>();
    c.ret_pool("GObj_Create", n);
    let j = c.jobj_chain(2);
    c.ret_ptr("HSD_JObjLoadJoint", j);
    c.out("lb_80011E24", 1, 8);
    c.arg(addr_of_mut!(tu::lbl_80472D28));
}

fn b_setup_light(c: &mut Case) {
    state_objects(c);
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    c.ret_pool("lb_80011AC4", core::mem::size_of::<HSD_LObj>());
    let out = c.zbuf(8);
    c.arg(addr_of_mut!(tu::lbl_80472D28));
    c.arg(out);
}

fn b_sisetup(c: &mut Case) {
    c.ret_int("lbLang_IsSavedLanguageUS", 0, 1);
    c.str_arg("HSD_SisLib_803A62A0", 1);
    c.str_arg("HSD_SisLib_803A62A0", 2);
    let g = c.gobj_any();
    c.arg(g);
}

fn b_state_only(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    c.ret_pool("lb_80011AC4", core::mem::size_of::<HSD_LObj>());
    c.ret_pool("HSD_CObjLoadDesc", core::mem::size_of::<HSD_CObj>());
    c.arg(addr_of_mut!(tu::lbl_80472D28));
}

fn b_noargs(c: &mut Case) {
    state_scalars(c);
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
}

fn b_80630(c: &mut Case) {
    state_scalars(c);
    state_objects(c);
    vsc(c);
    srules(c);
    text_pools(c);
    pad_status(c);
    unsafe { tu::lbl_804D65C0 = c.rng.range(-5, 100) as i32 };
    c.ret_int("fn_80171A88", 0, 9);
    c.ret_seq_u64("HSD_JObjGetFlags", &[0, 0x10, 0, 0x11, 0x20, 0]);
    c.out("lb_8000B1CC", 2, 12);
    c.mem_arg("lb_8000B1CC", 1, 12);
    c.ret_int("fn_8016F9A8", 0, 12);
    c.ret_int("fn_8016F870", -1, 8);
    c.ret_int("fn_8016F740", -1, 8);
    c.ret_int("fn_8016F548", -2, 10);
    c.ret_int("fn_8016FAD4", -30, 30);
    c.ret_int("fn_8016FFD4", -30, 30);
    c.ret_int("fn_8016F1F0", 0, 0x100);
    c.ret_int("fn_8016F160", 0, 1);
    c.ret_int("fn_801701C0", 0, 1);
    c.ret_int("fn_8016F180", 0, 0xFFFF);
    c.ret_int("lbLang_IsSavedLanguageUS", 0, 1);
    c.ret_int("HSD_Randi", 0, 1);
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    c.ret_pool("lb_80011AC4", core::mem::size_of::<HSD_LObj>());
    c.ret_pool("HSD_CObjLoadDesc", core::mem::size_of::<HSD_CObj>());
    c.ret_pool("lb_800138EC", core::mem::size_of::<HSD_GObj>());
    let j = c.jobj_chain(2);
    c.ret_ptr("HSD_JObjLoadJoint", j);
    c.out("lb_80011E24", 1, 8);
    c.out("Ground_801C1DE4", 0, 4);
    c.out("Ground_801C1DE4", 1, 4);
    c.out("grPushOn_80219204", 1, 4);
    c.out("grPushOn_80219204", 2, 4);
    if c.rng.chance(2, 3) {
        c.out("lbArchive_80016DBC", 1, 8);
    }
    c.ret_seq_u64("Ground_801C1DD4", &[3]);
    let a0 = c.rng.range(-100, 100000) as i32;
    let a1 = c.rng.range(-100, 100000) as i32;
    let a2 = pick(c, &[0i32, 1, 2, 3, 3]);
    let a3 = pick(c, &[0i32, 1]);
    let me = c.obj::<MatchEnd>();
    c.arg(a0);
    c.arg(a1);
    c.arg(a2);
    c.arg(a3);
    c.arg(me);
}

#[test]
fn gmregclear_vs_c() {
    let funcs = [
        fd!("fn_8017F008", "", "i32", 20, b_f008, |a| { r(tu::fn_8017F008()) }),
        fd!("fn_8017F09C", "", "i32", 60, b_f09c, |a| { r(tu::fn_8017F09C()) }),
        fd!("fn_8017F14C", "p", "i32", 40, b_f14c, |a| { r(tu::fn_8017F14C(p(a, 0))) }),
        fd!("fn_8017F1B8", "", "i32", 12, b_f1b8, |a| { r(tu::fn_8017F1B8()) }),
        fd!("fn_8017F294", "", "i32", 6, b_f294, |a| { r(tu::fn_8017F294()) }),
        fd!("fn_8017F2A4", "pff", "i32", 24, b_f2a4, |a| { r(tu::fn_8017F2A4(p(a, 0), af32(a, 1), af32(a, 2))) }),
        fd!("fn_8017F47C", "pi", "i32", 60, b_f47c, |a| { r(tu::fn_8017F47C(p(a, 0), ai32(a, 1))) }),
        fd!("fn_8017F608", "p", "v", 80, b_f608, |a| { tu::fn_8017F608(p(a, 0)); 0 }),
        fd!("fn_8017FA1C", "p", "i32", 80, b_fa1c, |a| { r(tu::fn_8017FA1C(p(a, 0))) }),
        fd!("fn_8017FBA4", "p", "v", 80, b_fba4, |a| { tu::fn_8017FBA4(p(a, 0)); 0 }),
        fd!("fn_8017FE54", "p", "v", 40, b_fe54, |a| { tu::fn_8017FE54(p(a, 0)); 0 }),
        fd!("fn_8017FF1C", "p", "v", 120, b_ff1c, |a| { tu::fn_8017FF1C(p(a, 0)); 0 }),
        fd!("fn_801803FC", "p", "i32", 60, b_f3fc, |a| { r(tu::fn_801803FC(p(a, 0))) }),
        fd!("fn_80180630_CreateCameraGObj", "", "p", 4, b_noargs, |a| { r(tu::fn_80180630_CreateCameraGObj()) }),
        fd!("fn_80180630_LoadLightList", "p", "p", 6, b_state_only, |a| { r(tu::fn_80180630_LoadLightList(p(a, 0))) }),
        fd!("fn_80180630_LoadCameraDesc", "p", "p", 6, b_state_only, |a| { r(tu::fn_80180630_LoadCameraDesc(p(a, 0))) }),
        fd!("fn_80180630_GetModelDesc", "p", "p", 6, b_state_only, |a| { r(tu::fn_80180630_GetModelDesc(p(a, 0))) }),
        fd!("fn_80180630_SetupSisLib", "p", "v", 8, b_sisetup, |a| { tu::fn_80180630_SetupSisLib(p(a, 0)); 0 }),
        fd!("fn_80180630_CreateLightAndCamera", "pp", "v", 8, b_setup_light, |a| { tu::fn_80180630_CreateLightAndCamera(p(a, 0), p(a, 1)); 0 }),
        fd!("fn_80180630_GetX118", "p", "i8", 6, b_state_only, |a| { r(tu::fn_80180630_GetX118(p(a, 0))) }),
        fd!("fn_80180630", "iiiip", "v", 120, b_80630, |a| { tu::fn_80180630(ai32(a, 0), ai32(a, 1), ai32(a, 2), ai32(a, 3), p(a, 4)); 0 }),
        fd!("fn_80180AC0", "", "i32", 6, b_f294, |a| { r(tu::fn_80180AC0()) }),
    ];
    let mut globals = tu::oracle_globals();
    // HSD_PadMasterStatus (controller.c) was the last global of this TU's table when the C data
    // was generated; it is now melee_runtime's single global: keep its place in the layout.
    globals.push(GlobalDesc {
        name: "HSD_PadMasterStatus",
        addr: core::ptr::addr_of_mut!(tu::HSD_PadMasterStatus) as *mut u8,
        size: core::mem::size_of::<[HSD_PadStatus; 4]>(),
        mode: GMode::Rw,
    });
    globals.extend(tu::oracle_globals_ext());
    let fn_table: std::vec::Vec<(&str, usize)> = std::vec![
        ("fn_8017FE54", tu::fn_8017FE54 as usize),
        ("fn_8017FF1C", tu::fn_8017FF1C as usize),
        ("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize),
        ("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize),
        ("HSD_GObj_803910D8", ext::HSD_GObj_803910D8 as usize),
    ];
    let _ = null_mut::<c_void>();
    run_tu(&TuOracle { tu: "gmregclear", funcs: &funcs, globals, fn_table });
}
