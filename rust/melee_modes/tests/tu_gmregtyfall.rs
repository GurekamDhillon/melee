//! State oracle for tu/gmregtyfall.rs (src/oracle.rs, gen/tu/run.py).
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_modes::fd;
use melee_modes::oracle::*;
use melee_modes::tu::gmregtyfall as tu;
use melee_modes::tu::gmregtyfall::ext;
use melee_types::*;

/// A GObj whose `hsd_obj` is `obj`.
fn gobj(c: &mut Case, obj: *mut c_void) -> *mut HSD_GObj {
    let g = c.obj::<HSD_GObj>();
    unsafe {
        (*g).next = null_mut();
        (*g).prev = null_mut();
        (*g).next_gx = null_mut();
        (*g).prev_gx = null_mut();
        (*g).proc = null_mut();
        (*g).user_data = null_mut();
        (*g).x34_unk = null_mut();
        (*g).hsd_obj = obj;
    }
    g
}

/// A JObj with null links.
fn jobj(c: &mut Case) -> *mut HSD_JObj {
    let j = c.obj::<HSD_JObj>();
    unsafe {
        (*j).next = null_mut();
        (*j).parent = null_mut();
        (*j).child = null_mut();
        (*j).aobj = null_mut();
        (*j).robj = null_mut();
        (*j).scl = null_mut();
        (*j).flags &= !(JOBJ_USE_QUATERNION as u32);
    }
    j
}

/// `depth` nested children below a root (depth 0 = just the root); each has a `next` sibling.
fn jobj_chain(c: &mut Case, depth: usize) -> *mut HSD_JObj {
    let root = jobj(c);
    let mut cur = root;
    for _ in 0..depth {
        let ch = jobj(c);
        let sib = jobj(c);
        unsafe {
            (*cur).child = ch;
            (*ch).parent = cur;
            (*ch).next = sib;
            (*sib).parent = cur;
        }
        cur = ch;
    }
    root
}

fn scene_desc(c: &mut Case) -> *mut SceneDesc {
    let sd = c.zobj::<SceneDesc>();
    let cams = c.zobj::<SceneCameraDesc>();
    let anims = c.zbuf(16) as *mut *mut HSD_CameraAnim;
    unsafe {
        (*cams).desc = c.buf(0x40).cast();
        *anims = c.buf(0x20).cast();
        (*cams).anims = anims;
        (*sd).cameras = cams;
        (*sd).lights = c.zbuf(16).cast();
        (*sd).models = c.zbuf(16).cast();
        let m = c.zobj::<DynamicModelDesc>();
        (*m).joint = c.buf(0x40).cast();
        *(*sd).models = m;
    }
    sd
}

fn sobj_pool(c: &mut Case) {
    c.ret_pool("HSD_SObjLib_803A477C", core::mem::size_of::<HSD_SObj>());
}

unsafe fn set_globals_flags(c: &mut Case) {
    tu::gm_804D6758 = c.rng.range(0, 1) as i32;
    tu::gm_804D675C = c.rng.range(0, 1) as i32;
    tu::gm_804D6784 = c.rng.range(0, 1) as i32;
    tu::gm_804D6780 = c.rng.range(0, 1) as i32;
    tu::gm_804D6790 = [0, 0, 1, 7][c.rng.range(0, 3) as usize];
    tu::gm_804D6794 = [0, 5, 0xB4, 0xBE, 0xC0, 0x10][c.rng.range(0, 5) as usize];
    tu::gm_804D6750 = [0, 1, 2, 3, 0x80, 0xFF, 0x100, 0x101, 0x320, -5][c.rng.range(0, 9) as usize];
    tu::gm_804D6740 = [0, 1, 0x1E, 0xFF][c.rng.range(0, 3) as usize];
}

fn b_flash(c: &mut Case) {
    unsafe {
        set_globals_flags(c);
        let o1 = c.buf(0x20).cast();
        tu::gm_804D6754 = gobj(c, o1);
        let o2 = c.buf(0x20).cast();
        tu::gm_804D677C = gobj(c, o2);
        tu::gm_804D6760 = HSD_SObjDesc { image: null_mut(), tlut: null_mut() };
        tu::gm_804D6788 = HSD_SObjDesc { image: null_mut(), tlut: null_mut() };
    }
    sobj_pool(c);
    let o = c.buf(0x20).cast();
    let g = gobj(c, o);
    c.arg(g);
}

fn b_gobj_only(c: &mut Case) {
    let o = c.buf(0x40).cast();
    let g = gobj(c, o);
    c.arg(g);
}

fn b_gobj_int(c: &mut Case) {
    unsafe { set_globals_flags(c) };
    c.ret_int("HSD_CObjSetCurrent", 0, 1);
    let o = c.buf(0x40).cast();
    let g = gobj(c, o);
    c.arg(g);
    let v = c.rng.range(-3, 9) as i32;
    c.arg(v);
}

fn b_walk(c: &mut Case) {
    c.out("Player_LoadPlayerCoords", 1, 12);
    c.mem_arg("Player_80032A04", 1, 12);
    let g = gobj(c, null_mut());
    c.arg(g);
}

fn b_68d8(c: &mut Case) {
    c.ret_int("gm_801BEFB0", 0, 32);
    c.ret_int("gm_801BEFD0", 0, 5);
    c.ret_flt("Player_80032BB0", -4.0, 4.0);
    if c.rng.chance(1, 2) {
        c.ret_ptr("Player_GetEntityAtIndex", null_mut::<c_void>());
    }
    unsafe {
        tu::gm_804D6744 = c.buf_or_null(16, 4).cast();
    }
}

fn b_camera(c: &mut Case) {
    unsafe {
        tu::gm_804D6748 = scene_desc(c);
    }
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    c.ret_pool("lb_80013B14", 0x80);
}

fn b_final(c: &mut Case) {
    unsafe {
        let mut gs = [null_mut::<HSD_GObj>(); 5];
        for g in gs.iter_mut() {
            let o = c.buf(0x20).cast();
            *g = gobj(c, o);
        }
        tu::gm_804D6768 = gs[0];
        tu::gm_804D6778 = gs[1];
        tu::gm_804D6754 = gs[2];
        tu::gm_804D676C = gs[3];
        tu::gm_804D677C = gs[4];
        let img = core::ptr::addr_of_mut!(tu::gm_80480964);
        c.rng.fill(img.cast(), core::mem::size_of_val(&*img));
    }
    let aobj = c.obj::<HSD_AObj>();
    let cobj = c.obj::<HSD_CObj>();
    unsafe {
        (*cobj).aobj = aobj;
        (*aobj).curr_frame = c.rng.range(0, 40) as f32;
        (*aobj).end_frame = if c.rng.chance(1, 2) { (*aobj).curr_frame } else { c.rng.range(0, 40) as f32 };
    }
    let g = gobj(c, cobj.cast());
    c.arg(g);
}

fn b_load(c: &mut Case) {
    c.ret_int("gm_GetCurrentGameMode", 19, 28);
    c.ret_int("gm_801BF050", 0, 12);
    c.ret_int("gm_801BEFB0", 0, 32);
}

fn b_setup_main(c: &mut Case) {
    c.ret_int("gm_801BEFB0", 0, 32);
    let depth = c.rng.range(2, 6) as usize;
    let tree = jobj_chain(c, depth);
    c.ret_ptr("HSD_JObjLoadJoint", tree);
    let pj = jobj(c);
    c.ret_ptr("Player_80036EA0", pj);
    unsafe { tu::gm_804D6798 = c.buf(0x40).cast() };
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    let pr = c.rng.range(0, 255) as u8;
    c.arg(pr);
}

fn b_setup_trophy(c: &mut Case) {
    c.ret_int("gm_801BEFB0", 0, 32);
    c.ret_int("gm_GetCurrentGameMode", 19, 28);
    c.ret_int("gm_801BF050", 0, 12);
    let depth = c.rng.range(1, 3) as usize;
    let tree = jobj_chain(c, depth);
    c.ret_ptr("HSD_JObjLoadJoint", tree);
    c.ret_flt("Toy_803060BC", -10.0, 10.0);
    // the main model gobj (set by SetupMain): hsd_obj is a deep tree
    let d2 = c.rng.range(3, 7) as usize;
    let main_tree = jobj_chain(c, d2);
    unsafe {
        tu::gm_804D6768 = gobj(c, main_tree.cast());
        tu::gm_804D67AC = scene_desc(c);
    }
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    let pr = c.rng.range(0, 255) as u8;
    c.arg(pr);
}

fn b_enter(c: &mut Case) {
    c.ret_int("gm_GetCurrentGameMode", 19, 28);
    c.ret_int("gm_801BF050", 0, 12);
    c.ret_int("gm_801BEFB0", 0, 32);
    c.ret_int("gm_801BEFD0", 0, 5);
    c.ret_flt("Player_80032BB0", -4.0, 4.0);
    c.ret_flt("Toy_803060BC", -10.0, 10.0);
    let depth = c.rng.range(2, 6) as usize;
    let tree = jobj_chain(c, depth);
    c.ret_ptr("HSD_JObjLoadJoint", tree);
    let pj = jobj(c);
    c.ret_ptr("Player_80036EA0", pj);
    unsafe {
        tu::gm_804D6748 = scene_desc(c);
        tu::gm_804D67AC = scene_desc(c);
        tu::gm_804D6798 = c.buf(0x40).cast();
        tu::gm_804D6744 = c.buf_or_null(16, 4).cast();
    }
    // the model gobj built by SetupMain is reached through gm_804D6768 after it is set
    c.ret_pool("GObj_Create", core::mem::size_of::<HSD_GObj>());
    c.ret_pool("lb_80013B14", 0x80);
    c.arg(null_mut::<c_void>());
}

fn b_frame(c: &mut Case) {
    unsafe { tu::gm_804D6740 = [0, 0, 0, 1, 0x1E][c.rng.range(0, 4) as usize] };
    c.ret_int("gm_801BF010", 0, 5);
    // START pressed about half the time
    c.ret_int("gm_GetButtonsTriggered", 0, 0x1FFF);
}

fn b_659c(c: &mut Case) {
    c.ret_int("gm_GetCurrentGameMode", 19, 28);
    c.ret_int("gm_801BF050", 0, 12);
    let v = c.rng.range(0, 40) as i32;
    c.arg(v);
}

fn b_6630(c: &mut Case) {
    let p = c.zbuf(8);
    c.ret_ptr("gm_GetCurrentSceneExitData", p);
    let v = c.rng.range(-3, 300) as i32;
    c.arg(v);
}

fn b_initimages(c: &mut Case) {
    unsafe {
        let img = core::ptr::addr_of_mut!(tu::gm_804809D0);
        c.rng.fill(img.cast(), core::mem::size_of_val(&*img));
    }
}

fn b_trophyz(c: &mut Case) {
    c.ret_flt("Toy_803060BC", -10.0, 10.0);
    let v = c.rng.range(0, 3) as i32;
    c.arg(v);
}

#[test]
fn gmregtyfall_vs_c() {
    let funcs = [
        fd!("gm_801A659C", "i", "i32", 40, b_659c, |a| { r(tu::gm_801A659C(ai32(a, 0))) }),
        fd!("gm_801A6630", "i", "v", 12, b_6630, |a| { tu::gm_801A6630(ai32(a, 0)); 0 }),
        fd!("fn_801A6664", "p", "v", 120, b_flash, |a| { tu::fn_801A6664(p(a, 0)); 0 }),
        fd!("fn_801A6844", "p", "v", 6, b_gobj_only, |a| { tu::fn_801A6844(p(a, 0)); 0 }),
        fd!("fn_801A6868", "p", "v", 40, b_walk, |a| { tu::fn_801A6868(p(a, 0)); 0 }),
        fd!("gm_801A68D8", "", "v", 24, b_68d8, |a| { tu::gm_801A68D8(); 0 }),
        fd!("fn_801A6A48", "pi", "v", 16, b_gobj_int, |a| { tu::fn_801A6A48(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fn_801A6ACC", "pi", "v", 16, b_gobj_int, |a| { tu::fn_801A6ACC(p(a, 0), ai32(a, 1)); 0 }),
        fd!("fn_801A6B6C", "p", "v", 24, b_final, |a| { tu::fn_801A6B6C(p(a, 0)); 0 }),
        fd!("fn_801A6C30", "p", "v", 6, b_gobj_only, |a| { tu::fn_801A6C30(p(a, 0)); 0 }),
        fd!("gm_801A6C54", "", "v", 12, b_camera, |a| { tu::gm_801A6C54(); 0 }),
        fd!("fn_801A6D78", "pi", "v", 12, b_gobj_int, |a| { tu::fn_801A6D78(p(a, 0), ai32(a, 1)); 0 }),
        fd!("gm_801A6DC0", "", "v", 12, b_camera, |a| { tu::gm_801A6DC0(); 0 }),
        fd!("gm_801A6EE4", "", "v", 40, b_load, |a| { tu::gm_801A6EE4(); 0 }),
        fd!("initImages", "", "v", 4, b_initimages, |a| { tu::initImages(); 0 }),
        fd!("gm_801A7070_SetupMain", "i", "v", 24, b_setup_main, |a| { tu::gm_801A7070_SetupMain(au8(a, 0)); 0 }),
        fd!("getTrophyTranslateZ", "i", "f", 8, b_trophyz, |a| { r(tu::getTrophyTranslateZ(ai32(a, 0))) }),
        fd!("gm_801A7070_SetupTrophy", "i", "v", 40, b_setup_trophy, |a| { tu::gm_801A7070_SetupTrophy(au8(a, 0)); 0 }),
        fd!("gm_Scene_ToyFall_OnEnter", "p", "v", 40, b_enter, |a| { tu::gm_Scene_ToyFall_OnEnter(p(a, 0)); 0 }),
        fd!("gm_Scene_ToyFall_OnFrame", "", "v", 40, b_frame, |a| { tu::gm_Scene_ToyFall_OnFrame(); 0 }),
    ];
    let fn_table: std::vec::Vec<(&str, usize)> = std::vec![
        ("fn_801A6664", tu::fn_801A6664 as usize),
        ("fn_801A6844", tu::fn_801A6844 as usize),
        ("fn_801A6868", tu::fn_801A6868 as usize),
        ("fn_801A6A48", tu::fn_801A6A48 as usize),
        ("fn_801A6ACC", tu::fn_801A6ACC as usize),
        ("fn_801A6B6C", tu::fn_801A6B6C as usize),
        ("fn_801A6C30", tu::fn_801A6C30 as usize),
        ("fn_801A6D78", tu::fn_801A6D78 as usize),
        ("HSD_GObj_JObjCallback", ext::HSD_GObj_JObjCallback as usize),
        ("HSD_GObj_LObjCallback", ext::HSD_GObj_LObjCallback as usize),
        ("HSD_SObjLib_803A49E0", ext::HSD_SObjLib_803A49E0 as usize),
        ("HSD_SObjLib_803A54EC", ext::HSD_SObjLib_803A54EC as usize),
    ];
    run_tu(&TuOracle { tu: "gmregtyfall", funcs: &funcs, globals: tu::oracle_globals(), fn_table });
}
