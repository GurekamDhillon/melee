//! Written by gen/mtu/mk_cm.py from the same table as gen/mtu/tu/cm.c.
//! tu::camera vs the C (camera.c, shipping contraction; regenerate the expected file with
//! `python rust/melee_world/gen/mtu/mk_cm.py run`).
#![allow(static_mut_refs, non_snake_case, non_upper_case_globals, unused_imports, unused_mut, unused_assignments, unused_variables, unused_unsafe)]
mod mtu_harness;
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;
use melee_world::tu as m;
use melee_world::tu::camera as cm;
use mtu_harness::*;
#[path = "mtu/cm_gen.rs"]
mod g;

static mut SUBJ: [CmSubject; 8] = unsafe { core::mem::zeroed() };
static mut CO: [HSD_CObj; 3] = unsafe { core::mem::zeroed() };
static mut WO: [HSD_WObj; 4] = unsafe { core::mem::zeroed() };
static mut GO: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };
static mut MTXA: [[f32; 4]; 4] = [[0.0; 4]; 4];
static mut BND: [CameraBounds; 2] = unsafe { core::mem::zeroed() };
static mut S2: [S32Vec2; 2] = unsafe { core::mem::zeroed() };
static mut V3: [Vec3; 3] = unsafe { core::mem::zeroed() };

fn rnn() -> f32 {
    const K: [f32; 4] = [1.0, 13.0, 100.0, 1000.0];
    let a = ((rnd() % 41) as i32 - 20) as f32 / 20.0;
    a * K[(rnd() % 4) as usize]
}
fn rng(lo: i32, hi: i32, div: i32) -> f32 {
    (lo + (rnd() % (hi - lo + 1) as u32) as i32) as f32 / div as f32
}
fn rpos() -> f32 {
    ((rnd() % 401) as i32 - 200) as f32 / 4.0
}
fn rposz() -> f32 {
    (rnd() % 600) as f32
}
unsafe fn fill_vec(v: *mut Vec3) {
    (*v).x = rpos(); (*v).y = rpos(); (*v).z = rpos();
}
unsafe fn fill_ts(t: *mut CameraTransformState) {
    fill_vec(&raw mut (*t).interest); fill_vec(&raw mut (*t).target_interest); fill_vec(&raw mut (*t).position); fill_vec(&raw mut (*t).target_position);
    (*t).position.z = rposz() + 50.0; (*t).target_position.z = rposz() + 50.0;
    (*t).fov = (15 + (rnd() % 60)) as f32; (*t).target_fov = (15 + (rnd() % 60)) as f32;
}

unsafe fn setup() {
    SUBJ = core::mem::zeroed(); CO = core::mem::zeroed(); WO = core::mem::zeroed(); GO = core::mem::zeroed(); MTXA = [[0.0; 4]; 4];
    BND = core::mem::zeroed(); S2 = core::mem::zeroed(); V3 = core::mem::zeroed();
    cm::ext::stage_info.grkind = (rnd() % 50) as _;
    cm::cm_804D6458 = null_mut(); cm::cm_804D645C = null_mut(); cm::cm_804D6460 = null_mut(); cm::cm_804D6464 = null_mut(); cm::cm_804D6468 = null_mut();
    cm::cm_largemap = core::mem::zeroed();
    g::fill_HSD_GObj(&mut GO[0]); g::fill_HSD_GObj(&mut GO[1]); g::fill_HSD_GObj(&mut GO[2]);
    for i in 0..3 {
        g::fill_HSD_CObj(&mut CO[i]);
        CO[i].eyepos = if rnd() & 1 != 0 { &raw mut WO[i] } else { null_mut() };
        CO[i].interest = if rnd() & 1 != 0 { &raw mut WO[i + 1] } else { null_mut() };
        CO[i].proj_mtx = null_mut();
        CO[i].flags = (fv() & 1) as _;
    }
    GO[0].hsd_obj = &raw mut CO[0] as *mut _; GO[1].hsd_obj = &raw mut CO[1] as *mut _; GO[2].hsd_obj = &raw mut CO[2] as *mut _;
    for i in 0..4 { WO[i].pos.x = rpos(); WO[i].pos.y = rpos(); WO[i].pos.z = rpos(); }
    for i in 0..16 { MTXA[i / 4][i % 4] = rnn(); }
    cm::Camera_Init(8);
    let n = rnd() % 6;
    for _ in 0..n {
        let s = cm::Camera_80029044((rnd() % 3) as i32);
        (*s).pos.x = rpos(); (*s).pos.y = rpos(); (*s).pos.z = 0.0; (*s).bone_pos.x = rpos(); (*s).bone_pos.y = rpos(); (*s).bone_pos.z = rpos();
        (*s).facing_dir = if rnd() & 1 != 0 { 1.0 } else { -1.0 };
        (*s).state_timer = if rnd() % 3 == 0 { (rnd() % 700) as i16 } else { 0 };
        (*s).set_was_framed((rnd() & 1) as _); (*s).set_force_inactive((rnd() % 8 == 0) as _); (*s).set_on_ledge((rnd() & 1) as _);
        (*s).ext.h.x = rpos() * 0.1; (*s).ext.h.y = rpos() * 0.1; (*s).ext.v.x = rpos() * 0.1; (*s).ext.v.y = rpos() * 0.1; (*s).ext.v.z = rpos() * 0.1;
        (*s).target_ext.h.x = rpos() * 0.1; (*s).target_ext.h.y = rpos() * 0.1; (*s).target_ext.v.x = rpos() * 0.1; (*s).target_ext.v.y = rpos() * 0.1; (*s).target_ext.v.z = rpos() * 0.1;
    }
    let gc = &raw mut cm::game_camera;
    (*gc).mode = (rnd() % 9) as _;
    fill_ts(&raw mut (*gc).transform); fill_ts(&raw mut (*gc).transform_copy);
    (*gc).gobj = if rnd() % 4 != 0 { &raw mut GO[0] } else { null_mut() };
    (*gc).quake_gobj = if rnd() % 3 == 0 { &raw mut GO[2] } else { null_mut() };
    (*gc).translation.x = rpos() * 0.1; (*gc).translation.y = rpos() * 0.1;
    (*gc).quake_offset.x = rnn(); (*gc).quake_offset.y = rnn();
    (*gc).quake_scale = (rnd() % 5) as f32 * 0.5;
    for i in 0..5 { (*gc).quake_frames_left[i] = if rnd() % 3 == 0 { (rnd() % 30) as i32 } else { 0 }; }
    for i in 0..16 {
        (*gc).quakes[0][i].kind = if rnd() % 3 == 0 { (rnd() % 5) as _ } else { 0 };
        (*gc).quakes[1][i].kind = (rnd() % 5) as _;
        fill_vec(&raw mut (*gc).quakes[0][i].epicenter); fill_vec(&raw mut (*gc).quakes[1][i].epicenter);
    }
    (*gc).x2B0 = rpos(); (*gc).x2B4 = rpos() * 100.0; (*gc).x2B8 = if rnd() % 5 == 0 { 1000 + (rnd() % 5) as i16 } else { (rnd() % 20) as i16 };
    (*gc).x2BA = (rnd() % 5) as _; (*gc).x2BC = if rnd() % 3 == 0 { 1.0 } else { (rnd() % 7) as f32 * 0.5 }; (*gc).x2C0 = (rnd() % 400) as f32 - 50.0;
    (*gc).set_x398_b0((rnd() & 1) as _); (*gc).set_x398_b1((rnd() & 1) as _); (*gc).set_x398_b2((rnd() & 1) as _); (*gc).set_x398_b3((rnd() & 1) as _); (*gc).set_x398_b4((rnd() & 1) as _); (*gc).set_x398_b5((rnd() & 1) as _);
    (*gc).set_x399_b0_b1((rnd() & 3) as _); (*gc).set_x399_b2((rnd() & 1) as _); (*gc).set_x399_b3((rnd() & 1) as _); (*gc).set_x399_b4((rnd() & 1) as _); (*gc).set_x399_b5((rnd() & 1) as _); (*gc).set_x399_b6((rnd() & 1) as _); (*gc).set_x399_b7((rnd() & 1) as _);
    (*gc).set_x39A_b0((rnd() & 1) as _); (*gc).set_x39A_b1((rnd() & 1) as _); (*gc).set_x39A_b2((rnd() & 1) as _);
    (*gc).nearz = (rnd() % 10) as f32 * 0.1; (*gc).farz = (rnd() % 20000) as f32;
    (*gc).background_color.r = fv() as u8; (*gc).background_color.g = fv() as u8; (*gc).background_color.b = fv() as u8;
    cm::cm_804D6464 = if rnd() & 1 != 0 { &raw mut CO[2] } else { null_mut() };
    cm::cm_80453004.last_mode = (rnd() % 9) as _; cm::cm_80453004.ply_slot = (rnd() % 4) as _;
    fill_vec(&raw mut cm::cm_80453004.follow_int_pos); fill_vec(&raw mut cm::cm_80453004.follow_eye_pos); fill_vec(&raw mut cm::cm_80453004.free_int_pos); fill_vec(&raw mut cm::cm_80453004.free_eye_pos);
    cm::cm_80453004.follow_fov = rpos(); cm::cm_80453004.free_fov = rpos();
    cm::cm_script = core::mem::zeroed();
    let cs = &raw mut cm::cm_script;
    if rnd() % 3 == 0 {
        (*cs).state = (rnd() % 3) as _; (*cs).lift_bounds = (rnd() & 1) as _; (*cs).elapsed = (rnd() % 20) as _; (*cs).duration = (rnd() % 30) as _;
        (*cs).ease = (rnd() % 4) as _; (*cs).path_count = (rnd() % 5) as _; (*cs).path_frame = (rnd() % 12) as _;
        (*cs).follow_kind = if rnd() % 3 == 0 { 1 + (rnd() % 3) as i32 } else { 0 }; (*cs).follow_id = (rnd() % 8) as _;
        (*cs).shake_left = (rnd() % 8) as _; (*cs).shake_total = 1 + (rnd() % 8) as i32; (*cs).shake_tick = (rnd() % 100) as _;
        (*cs).shake_intensity = rnn(); (*cs).saved_roll = rnn(); (*cs).saved_far = (rnd() % 20000) as f32; (*cs).saved_up_vector = (rnd() & 1) as _;
        fill_vec(&raw mut (*cs).saved_up); fill_vec(&raw mut (*cs).follow_point); fill_vec(&raw mut (*cs).follow_offset); fill_vec(&raw mut (*cs).follow_eye_delta);
        fill_ts(&raw mut (*cs).normal);
        fill_vec(&raw mut (*cs).pose.eye); fill_vec(&raw mut (*cs).pose.interest); (*cs).pose.fov = rpos(); (*cs).pose.roll = rnn();
        fill_vec(&raw mut (*cs).from.eye); fill_vec(&raw mut (*cs).from.interest); (*cs).from.fov = rpos(); (*cs).from.roll = rnn();
        fill_vec(&raw mut (*cs).to.eye); fill_vec(&raw mut (*cs).to.interest); (*cs).to.fov = rpos(); (*cs).to.roll = rnn();
        for i in 0..5 {
            fill_vec(&raw mut (*cs).path[i].pose.eye); fill_vec(&raw mut (*cs).path[i].pose.interest); (*cs).path[i].pose.fov = rpos(); (*cs).path[i].pose.roll = rnn();
            (*cs).path[i].frame = (i as u32 * 3 + rnd() % 3) as i32;
        }
        (*cs).completion = (rnd() % 4) as _;
    }
    for i in 0..2 {
        BND[i].x_min = rpos(); BND[i].y_min = rpos(); BND[i].x_max = BND[i].x_min + (rnd() % 200) as f32; BND[i].y_max = BND[i].y_min + (rnd() % 150) as f32;
        BND[i].total_subjects = (rnd() % 5) as _; BND[i].z_pos = rposz();
    }
    cm::ext::gm_CStickSmashTargetTest = (rnd() & 1) as _;
    for i in 0..4 { g::fill_HSD_PadStatus(&mut cm::ext::HSD_PadCopyStatus[i]); }
    cm::ext::HSD_GObj_CameraKind = fv() as u8;
    fill_vec(&raw mut V3[0]);
}

unsafe fn hash_all() {
    g::hash_Camera(&cm::game_camera);
    g::hash_CameraDebugMode(&cm::cm_80453004);
    for i in 0..8 { g::hash_CmSubject(&SUBJ[i]); }
    for i in 0..3 { g::hash_HSD_CObj(&CO[i]); g::hash_HSD_GObj(&GO[i]); }
    for i in 0..4 { hm(WO[i].pos.x.to_bits()); hm(WO[i].pos.y.to_bits()); hm(WO[i].pos.z.to_bits()); }
    for i in 0..2 { g::hash_CameraBounds(&BND[i]); }
    for i in 0..2 { hm(S2[i].x as u32); hm(S2[i].y as u32); }
    for i in 0..3 { hm(V3[i].x.to_bits()); hm(V3[i].y.to_bits()); hm(V3[i].z.to_bits()); }
    hm(rh(&cm::cm_script));
    hm(pi(cm::cm_largemap.primary)); hm(pi(cm::cm_largemap.secondary)); hm(cm::cm_largemap.primary_far.to_bits()); hm(cm::cm_largemap.secondary_far.to_bits());
    hm(pi(cm::cm_804D6458)); hm(pi(cm::cm_804D645C)); hm(pi(cm::cm_804D6460)); hm(pi(cm::cm_804D6464)); hm(pi(cm::cm_804D6468));
    hm(cm::cm_803BCCA0.x40.to_bits()); hm(cm::ext::stage_info.grkind as u32);
}

#[test]
fn cm_matches_c() {
    unsafe {
        std::panic::set_hook(Box::new(|i| { let m = i.to_string(); if !(m.contains(": assertion failed") || m.contains("lineID")) { eprintln!("{}", m); } }));
        g::install();
        cm::ext::hook::OSReport = Some(|_| {});
        preg(&raw const cm::game_camera, core::mem::size_of::<Camera>());
        preg(&raw const SUBJ, core::mem::size_of_val(&SUBJ));
        preg(&raw const CO, core::mem::size_of_val(&CO));
        preg(&raw const WO, core::mem::size_of_val(&WO));
        preg(&raw const GO, core::mem::size_of_val(&GO));
        preg(&raw const MTXA, core::mem::size_of_val(&MTXA));
        preg(&raw const BND, core::mem::size_of_val(&BND));
        preg(&raw const S2, core::mem::size_of_val(&S2));
        preg(&raw const V3, core::mem::size_of_val(&V3));
        preg(&raw const cm::cm_script, core::mem::size_of_val(&cm::cm_script));
        preg(&raw const cm::cm_80453004, core::mem::size_of_val(&cm::cm_80453004));
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_cm.txt"));
        for op in 0..34u32 {
            for k in 0..500u32 {
                case(op, k);
                setup();
                let mut ret = 0u32;
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match op {
                    0 => { cm::Camera_Init(1 + (rnd() % 8) as i32); }
                    1 => { cm::Camera_Create(); }
                    2 => { cm::Camera_ForgetGameCamera(); }
                    3 => { cm::Camera_RefreshViewingMtx(); }
                    4 => { ret = cm::Camera_ScriptCompletion() as u32; }
                    5 => { ret = cm::Camera_8002928C((&raw mut SUBJ[(rnd() % 8) as usize])) as u32; }
                    6 => { cm::Camera_8002A4AC(&raw mut GO[0]); }
                    7 => { { let ts = (if rnd() & 1 != 0 { &raw mut cm::game_camera.transform } else { &raw mut cm::game_camera.transform_copy }); cm::Camera_8002A768(ts, fv() as i32); } }
                    8 => { { let ts = (if rnd() & 1 != 0 { &raw mut cm::game_camera.transform } else { &raw mut cm::game_camera.transform_copy }); cm::Camera_8002AF68(&raw mut CO[(rnd() % 3) as usize], ts); } }
                    9 => { cm::Camera_8002B0E0(); }
                    10 => { { let ts = (if rnd() & 1 != 0 { &raw mut cm::game_camera.transform } else { &raw mut cm::game_camera.transform_copy }); cm::Camera_8002B1F8(ts); } }
                    11 => { cm::Camera_8002B3D4(null_mut()); }
                    12 => { cm::Camera_8002F3AC(); }
                    13 => { { let ts = (if rnd() & 1 != 0 { &raw mut cm::game_camera.transform } else { &raw mut cm::game_camera.transform_copy }); cm::Camera_ApplyQuake(&raw mut BND[0], ts); } }
                    14 => { if cm::game_camera.gobj.is_null() { cm::game_camera.gobj = &raw mut GO[0]; } ret = cm::Camera_LogicToScreen((&raw mut SUBJ[(rnd() % 8) as usize]), &raw mut S2[0]) as u32; }
                    15 => { ret = pi(cm::Camera_PCMainCObj()); }
                    16 => { ret = cm::Camera_ProjectToScreen(&raw mut CO[(rnd() % 3) as usize], &raw mut V3[0], &raw mut S2[0]) as u32; }
                    17 => { ret = cm::Camera_ProjectToScreen(&raw mut CO[(rnd() % 3) as usize], &raw mut V3[0], null_mut()) as u32; }
                    18 => { { let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; cm::Camera_SetBackgroundColor(r, g, b); } }
                    19 => { cm::Camera_SetQuakeScale(rw()); }
                    20 => { cm::Camera_UpdateQuakes(&raw mut BND[0]); }
                    21 => { cm::cm_script_update(); }
                    22 => { cm::fn_8002F360(&raw mut GO[0]); }
                    23 => { cm::fn_800301D0(&raw mut GO[0], fv() as isize); }
                    24 => { cm::update_bounds(&raw mut BND[0], &raw mut BND[1]); }
                    25 => { { let ts = (if rnd() & 1 != 0 { &raw mut cm::game_camera.transform } else { &raw mut cm::game_camera.transform_copy }); cm::update_transform(&raw mut BND[0], ts); } }
                    26 => { { let x = rw(); let y = rw(); cm::Camera_SetQuakeOffset(x, y); } }
                    27 => { if cm::game_camera.gobj.is_null() { cm::game_camera.gobj = &raw mut GO[0]; } ret = cm::Camera_80030BBC(&raw mut V3[0], &raw mut S2[0]) as u32; }
                    28 => { if cm::game_camera.gobj.is_null() { cm::game_camera.gobj = &raw mut GO[0]; } ret = cm::Camera_80030BBC(&raw mut V3[0], null_mut()) as u32; }
                    29 => { { let k = (rnd() % 4) as u32 + 1; cm::Camera_RequestQuake(k, if rnd() & 1 != 0 { &raw mut V3[0] } else { null_mut() }); } }
                    30 => { ret = cm::Camera_ScriptDetach() as u32; }
                    31 => { cm::Camera_ScriptAttach((rnd() % 6) as i32 - 1); }
                    32 => { cm::Camera_ScriptCommit(); }
                    33 => { cm::Camera_800293E0(); }
                    _ => unreachable!(),
                }
                }));
                if let Err(e) = r {
                    let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                    if !(msg.contains(": assertion failed") || msg.contains("lineID") || msg.contains("__assert")) { std::panic::resume_unwind(e); }
                    hm(0xA55E87);
                }
                hash_all();
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 34 * 500);
    }
}
