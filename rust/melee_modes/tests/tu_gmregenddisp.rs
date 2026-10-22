//! State oracle for tu/gmregenddisp.rs (src/oracle.rs, gen/tu/run.py).
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::gmregenddisp as tu;
use melee_modes::tu::gmregenddisp::ext;
use melee_modes::tu::gmregtyfall as ty;
use melee_types::*;

/// Toy_803048C0 results for two passes over the 26 trophies with `n1` / `n2` owned.
fn toy_seq(c: &mut Case, n1: usize, n2: usize) {
    let mut v = std::vec::Vec::new();
    for n in [n1, n2] {
        let mut blk = [0u64; 26];
        let mut k = 0;
        while k < n.min(26) {
            let i = c.rng.range(0, 25) as usize;
            if blk[i] == 0 {
                blk[i] = c.rng.range(1, 3) as u64;
                k += 1;
            }
        }
        v.extend_from_slice(&blk);
    }
    c.ret_seq_u64("Toy_803048C0", &v);
}

fn pick(c: &mut Case, xs: &[usize]) -> usize {
    xs[c.rng.range(0, xs.len() as i64 - 1) as usize]
}

fn b_fog(c: &mut Case) {
    let n1 = pick(c, &[0, 3, 5, 6, 9, 20]);
    let n2 = pick(c, &[0, 7, 13, 14, 26]);
    toy_seq(c, n1, n2);
    let fog = c.obj::<HSD_Fog>();
    let a = c.aobj();
    let frames = [0.0f32, 150.0, 184.0, 185.0, 186.0, 199.0, 200.0, 201.0, 250.0];
    unsafe {
        (*fog).aobj = a;
        (*a).curr_frame = frames[c.rng.range(0, 8) as usize];
    }
    let g = c.gobj(fog.cast());
    c.arg(g);
}

fn b_gobj_any(c: &mut Case) {
    let g = c.gobj_any();
    c.arg(g);
}

fn b_jobj_gobj(c: &mut Case) {
    let j = c.jobj();
    let g = c.gobj(j.cast());
    c.arg(g);
}

fn b_8114(c: &mut Case) {
    let j = c.jobj_chain(1);
    c.ret_flt("Toy_803060BC", -5.0, 5.0);
    c.arg(j);
    let t = c.rng.range(0, 30) as i32;
    c.arg(t);
}

fn b_851c(c: &mut Case) {
    c.ret_seq_u64("gm_801A4BA8", &[0x174, 0, 0x174, 5]);
    b_jobj_gobj(c);
}

fn b_85e4(c: &mut Case) {
    let j = c.jobj();
    c.ret_flt("HSD_Randf", 0.0, 1.0);
    c.arg(j);
    let a1 = if c.idx < 30 { c.idx as i32 } else { c.rng.range(0, 30) as i32 };
    c.arg(a1);
    let a2 = c.rng.range(-5, 40) as i32;
    c.arg(a2);
}

fn b_8d54(c: &mut Case) {
    for v in unsafe { (*core::ptr::addr_of_mut!(tu::gm_80480AD0)).iter_mut() } {
        *v = c.rng.range(0, 100) as i32;
    }
    let n = pick(c, &[0, 5, 12, 20, 26]);
    toy_seq(c, n, 0);
    c.ret_int("gm_801BEFB0", 0, 33);
    let out = c.zbuf(0x1A * 4);
    c.arg(out);
}

fn scenes(c: &mut Case) {
    unsafe {
        ty::gm_804D67A0 = c.scene_desc();
        ty::gm_804D67A4 = c.scene_desc();
        ty::gm_804D67AC = c.scene_desc();
        ty::gm_804D6798 = c.buf(0x40).cast();
        ty::gm_804D679C = c.buf(0x40).cast();
    }
}

/// `n` prebuilt GObjs (each with a joint tree behind it) for GObj_Create to hand out.
fn gobj_seq(c: &mut Case, n: usize) {
    let mut v = std::vec::Vec::new();
    for _ in 0..n {
        let d = c.rng.range(0, 4) as usize;
        let t = c.jobj_chain(d);
        let g = c.gobj(t.cast());
        v.push(g as usize as u64);
    }
    c.ret_seq_u64("GObj_Create", &v);
}

/// `n` joint trees for HSD_JObjLoadJoint (each has a child, as the C expects).
fn jobj_seq(c: &mut Case, n: usize) {
    let mut v = std::vec::Vec::new();
    for _ in 0..n {
        let d = c.rng.range(1, 4) as usize;
        let t = c.jobj_chain(d);
        v.push(t as usize as u64);
    }
    c.ret_seq_u64("HSD_JObjLoadJoint", &v);
}

fn b_9094(c: &mut Case) {
    scenes(c);
    for v in unsafe { (*core::ptr::addr_of_mut!(tu::gm_80480AD0)).iter_mut() } {
        *v = c.rng.range(0, 100) as i32;
    }
    let n = pick(c, &[0, 3, 12, 26]);
    toy_seq(c, n, 0);
    c.ret_int("gm_801BEFB0", 0, 33);
    let dsp = c.obj::<TyDspEntry>();
    c.ret_ptr("tyDisplay_8031B9DC", dsp);
    c.ret_pool("HSD_JObjAlloc", core::mem::size_of::<HSD_JObj>());
    jobj_seq(c, 60);
    c.ret_pool("HSD_ArchiveGetPublicAddress", 0x40);
    c.ret_flt("HSD_Randf", 0.0, 1.0);
    gobj_seq(c, 30);
}

fn b_camproc(c: &mut Case) {
    let n1 = pick(c, &[0, 3, 5, 6, 9, 20]);
    let n2 = pick(c, &[0, 7, 13, 14, 26]);
    toy_seq(c, n1, n2);
    unsafe { tu::gm_804D67C4 = [1, 2, 0xB4][c.rng.range(0, 2) as usize] };
    let cobj = c.obj::<HSD_CObj>();
    let a = c.aobj();
    let frames = [0.0f32, 159.0, 160.0, 161.0, 189.0, 190.0, 191.0, 250.0];
    unsafe {
        (*cobj).aobj = a;
        (*a).curr_frame = frames[c.rng.range(0, 7) as usize];
        (*a).end_frame = frames[c.rng.range(0, 7) as usize];
    }
    let g = c.gobj(cobj.cast());
    c.arg(g);
}

fn b_init(c: &mut Case) {
    unsafe {
        for g in (*core::ptr::addr_of_mut!(tu::gm_80480A00)).iter_mut() {
            *g = c.buf(8).cast();
        }
    }
    let n = pick(c, &[0, 5, 26]);
    toy_seq(c, n, 0);
    c.ret_int("HSD_Randi", 0, 9999);
    c.arg(0u64);
}

fn b_fogsetup(c: &mut Case) {
    scenes(c);
    c.ret_pool("HSD_FogLoadDesc", core::mem::size_of::<HSD_Fog>());
    gobj_seq(c, 3);
}

fn b_lightsetup(c: &mut Case) {
    scenes(c);
    c.ret_pool("lb_80011AC4", core::mem::size_of::<HSD_LObj>());
    gobj_seq(c, 3);
}

fn b_camsetup(c: &mut Case) {
    scenes(c);
    c.ret_pool("lb_80013B14", 0x80);
    gobj_seq(c, 3);
}

fn b_9630(c: &mut Case) {
    scenes(c);
    let n = pick(c, &[0, 4, 14, 26]);
    toy_seq(c, n, 0);
    for v in unsafe { (*core::ptr::addr_of_mut!(tu::gm_80480AD0)).iter_mut() } {
        *v = c.rng.range(0, 100) as i32;
    }
    c.ret_int("gm_801BEFB0", 0, 33);
    c.ret_int("HSD_Randi", 0, 9999);
    c.ret_flt("HSD_Randf", 0.0, 1.0);
    c.ret_flt("Toy_803060BC", -5.0, 5.0);
    let dsp = c.obj::<TyDspEntry>();
    c.ret_ptr("tyDisplay_8031B9DC", dsp);
    c.ret_pool("HSD_FogLoadDesc", core::mem::size_of::<HSD_Fog>());
    c.ret_pool("lb_80011AC4", core::mem::size_of::<HSD_LObj>());
    c.ret_pool("lb_80013B14", 0x80);
    c.ret_pool("HSD_JObjAlloc", core::mem::size_of::<HSD_JObj>());
    c.ret_pool("HSD_ArchiveGetPublicAddress", 0x40);
    jobj_seq(c, 60);
    gobj_seq(c, 60);
}

fn b_entry(c: &mut Case) {
    let p = c.buf(0x1A * 4);
    c.arg(p);
    let i = c.rng.range(0, 25) as i32;
    c.arg(i);
}

fn b_enddisp_ptr(c: &mut Case) {
    let _ = c;
}

#[test]
fn gmregenddisp_vs_c() {
    let funcs = [
        fd!("fn_801A7FB4", "p", "v", 60, b_fog, |a| { tu::fn_801A7FB4(p(a, 0)); 0 }),
        fd!("fn_801A80CC", "p", "v", 4, b_gobj_any, |a| { tu::fn_801A80CC(p(a, 0)); 0 }),
        fd!("fn_801A80F0", "p", "v", 4, b_jobj_gobj, |a| { tu::fn_801A80F0(p(a, 0)); 0 }),
        fd!("gm_801A8114", "pi", "v", 30, b_8114, |a| { tu::gm_801A8114(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fn_801A851C", "p", "v", 12, b_851c, |a| { tu::fn_801A851C(p(a, 0)); 0 }),
        fd!("gm_801A85E4", "pii", "v", 80, b_85e4, |a| { tu::gm_801A85E4(p(a, 0), ai32(a, 1), ai32(a, 2)); 0 }),
        fd!("gm_801A8D54", "p", "v", 40, b_8d54, |a| { tu::gm_801A8D54(p(a, 0)); 0 }),
        fd!("gm_801A9094_get_entry", "pi", "i32", 16, b_entry, |a| {
            r(tu::gm_801A9094_get_entry(core::slice::from_raw_parts(p::<i32>(a, 0), 0x1A), ai32(a, 1)))
        }),
        fd!("gm_801A9094_get_bg", "", "p", 4, b_9094, |a| { r(tu::gm_801A9094_get_bg()) }),
        fd!("gm_801A9094_create_gobj", "", "p", 4, b_9094, |a| { r(tu::gm_801A9094_create_gobj()) }),
        fd!("gm_801A9094", "", "v", 40, b_9094, |a| { tu::gm_801A9094(); 0 }),
        fd!("fn_801A9498", "p", "v", 4, b_jobj_gobj, |a| { tu::fn_801A9498(p(a, 0)); 0 }),
        fd!("fn_801A94BC", "p", "v", 80, b_camproc, |a| { tu::fn_801A94BC(p(a, 0)); 0 }),
        fd!("gm_801A8114_inline", "pi", "v", 8, b_8114, |a| { tu::gm_801A8114_inline(p(a, 0), ai32(a, 1)); 0 }),
        fd!("gm_801A9630_init", "o", "v", 12, b_init, |a| { let mut cursor: *mut i32 = core::ptr::null_mut(); tu::gm_801A9630_init(&mut cursor); 0 }),
        fd!("gm_801A9630_fog", "", "v", 8, b_fogsetup, |a| { tu::gm_801A9630_fog(); 0 }),
        fd!("gm_801A9630_light", "", "v", 8, b_lightsetup, |a| { tu::gm_801A9630_light(); 0 }),
        fd!("gm_801A9630_camera", "", "v", 8, b_camsetup, |a| { tu::gm_801A9630_camera(); 0 }),
        fd!("gm_801A9630", "", "v", 40, b_9630, |a| { tu::gm_801A9630(); 0 }),
    ];
    let mut globals = tu::oracle_globals();
    globals.extend(tu::oracle_globals_ext());
    let fn_table: std::vec::Vec<(&str, usize)> = std::vec![
        ("fn_801A7FB4", tu::fn_801A7FB4 as usize),
        ("fn_801A80CC", tu::fn_801A80CC as usize),
        ("fn_801A80F0", tu::fn_801A80F0 as usize),
        ("fn_801A851C", tu::fn_801A851C as usize),
        ("fn_801A9498", tu::fn_801A9498 as usize),
        ("fn_801A94BC", tu::fn_801A94BC as usize),
        ("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize),
        ("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize),
        ("HSD_GObj_FogCallback", ext::HSD_GObj_FogCallback as usize),
        ("HSD_GObj_803910D8", ext::HSD_GObj_803910D8 as usize),
    ];
    let _ = (null_mut::<c_void>(), b_enddisp_ptr as fn(&mut Case));
    run_tu(&TuOracle { tu: "gmregenddisp", funcs: &funcs, globals, fn_table });
}
