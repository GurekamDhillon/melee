#![allow(non_snake_case)]
mod common_devtext;
use common_devtext::Dt;
use melee_runtime::mtx::Vec3;
use melee_runtime::tu::dbcamera::*;
use melee_runtime::tu::dbeffect::pad;

#[derive(Default)]
struct H {
    dt: Dt,
    down: u32,
    pressed: u32,
    misc: i32,
    cam_mode: u8, // 0 normal, 1 = 80030154 (follow dbg), 2 = 80030178 (free dbg)
    pad: PadMaster,
    target: Option<Vec3>,
    v: DbgCamVecs,
    log: Vec<String>,
}
forward_devtext!(H);
fn cobj() -> CObjView {
    CObjView {
        eye: Vec3 { x: 0.0, y: 10.0, z: 100.0 },
        interest: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
        fov: 30.0,
        eye_vector: Vec3 { x: 0.0, y: -0.0995037, z: -0.995037 },
        eye_distance: 100.498756,
        left_vector: Vec3 { x: -1.0, y: 0.0, z: 0.0 },
        up_vector: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
    }
}
impl DbCameraHost for H {
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn db_MiscVisualEffectsStatus(&mut self) -> i32 { self.misc }
    fn Camera_80030A60(&mut self, v: i32) { self.log.push(format!("A60 {v}")) }
    fn Camera_80030A8C(&mut self, v: i32) { self.log.push(format!("A8C {v}")) }
    fn Camera_80030B0C(&mut self, v: i32) { self.log.push(format!("B0C {v}")) }
    fn Camera_80030B38(&mut self, v: i32) { self.log.push(format!("B38 {v}")) }
    fn Camera_80030B64(&mut self, v: i32) { self.log.push(format!("B64 {v}")) }
    fn Camera_80030B90(&mut self, v: i32) { self.log.push(format!("B90 {v}")) }
    fn Camera_SetStageVisible(&mut self, v: i32) { self.log.push(format!("vis {v}")) }
    fn Camera_SetBackgroundColor(&mut self, r: u8, g: u8, b: u8) { self.log.push(format!("bgc {r} {g} {b}")) }
    fn Camera_80030154(&mut self) -> bool { self.cam_mode == 1 }
    fn Camera_80030178(&mut self) -> bool { self.cam_mode == 2 }
    fn Camera_8003006C(&mut self) { self.log.push("006C".into()) }
    fn Camera_8002FEEC(&mut self, _: i32) { self.log.push("FEEC".into()) }
    fn Camera_800300F0(&mut self) { self.log.push("00F0".into()) }
    fn Camera_8003019C(&mut self) -> Option<Vec3> { self.target }
    fn Camera_80030A50(&mut self) -> Option<CObjView> { Some(cobj()) }
    fn cm_80453004(&mut self) -> &mut DbgCamVecs { &mut self.v }
    fn Ground_ApplyStageBackgroundColor(&mut self) { self.log.push("grbg".into()) }
    fn gm_8018841C(&mut self) -> i32 { 0 }
    fn gm_GetCurrentGameMode(&mut self) -> i32 { 0 }
    fn Mex_Enabled(&mut self, _: &str) -> bool { false }
    fn HSD_PadMasterStatus(&mut self, _: i32) -> PadMaster { self.pad }
    fn fighters_set_shadow_hidden(&mut self, v: bool) { self.log.push(format!("shadow {v}")) }
    fn grounds_set_flag_b7(&mut self, v: bool) { self.log.push(format!("gr {v}")) }
}

#[test]
fn stage_visuals_cycle_wraps_after_6() {
    let mut s = DbCamera::default();
    let mut h = H { down: pad::R, pressed: pad::DPADDOWN, ..Default::default() };
    for want in [1, 2, 3, 4, 5, 6, 0] {
        h.log.clear();
        s.fn_check_misc_stage_effects(&mut h, 0);
        assert_eq!(s.misc_stage_visuals_status, want);
        assert_eq!(h.log.len(), 8);
    }
    assert_eq!(h.log[0], "shadow false"); // default row: fn_802270C4(1)
    h.down = pad::X;
    h.log.clear();
    s.fn_check_misc_stage_effects(&mut h, 0);
    assert_eq!(h.log, ["grbg", "vis 1"]);
}

#[test]
fn free_camera_orbit_and_info_overlay() {
    let mut s = DbCamera::default();
    s.fn_setup_camera_info();
    let mut h = H { cam_mode: 2, ..Default::default() };
    h.v.free_eye_pos = Vec3 { x: 0.0, y: 10.0, z: 100.0 };
    h.pad = PadMaster { button: 0, nml_sub_stick_x: 0.5, nml_sub_stick_y: 0.1 };
    s.fn_check_camera_info(&mut h, 0, 0, pad::DPADUP as i32, 0.0, 0.0);
    assert_eq!(h.log, ["FEEC"]);
    assert!(s.camera_info_display.is_some());
    assert_eq!(s.camera_info_display_timer, 0x3C);
    // Orbit moved the eye around the interest and kept the distance.
    let e = h.v.free_eye_pos;
    let d = (e.x * e.x + e.y * e.y + e.z * e.z).sqrt();
    assert!((d - 100.498756).abs() < 0.01, "{d}");
    assert!(e.x != 0.0);
    // Toggle info with B, then the overlay prints and counts down.
    s.misc_stage_visuals_status = 3;
    h.pad = PadMaster::default();
    h.dt.log.clear();
    s.fn_check_camera_info(&mut h, 0, 0, pad::B as i32, 0.0, 0.0);
    assert_eq!(s.show_camera_info, 1);
    assert!(h.dt.log.iter().any(|l| l == "p EYE 0,10,100"), "{:?}", h.dt.log);
    assert!(h.dt.log.iter().any(|l| l == "p FOV 30  ANG -5"), "{:?}", h.dt.log);
    assert_eq!(s.camera_info_display_timer, 0x3B);
}

#[test]
fn follow_camera_pan_and_dolly() {
    let mut s = DbCamera::default();
    let t = Vec3 { x: 1.0, y: 2.0, z: 3.0 };
    let mut h = H { cam_mode: 1, target: Some(t), ..Default::default() };
    h.v.follow_eye_offset = Vec3 { x: 0.0, y: 0.0, z: 50.0 };
    h.pad = PadMaster { button: 1, nml_sub_stick_x: 0.0, nml_sub_stick_y: 1.0 };
    let c = cobj();
    s.fn_802277e8(&mut h, Some(c), 0);
    let f = dolly_follow(c.eye_distance, 1.0);
    assert_eq!(h.v.follow_eye_offset.z, 0.0 - c.eye_vector.z * f);
    assert_eq!(h.v.follow_int_pos.x, h.v.follow_eye_offset.x + 1.0);
    assert_eq!(h.v.follow_eye_pos, t);
    h.pad = PadMaster { button: 2, nml_sub_stick_x: 0.5, nml_sub_stick_y: 0.0 };
    s.fn_802277e8(&mut h, Some(c), 0);
    let k = pan_scale_follow(c.eye_distance, c.fov);
    assert_eq!(h.v.follow_int_offset.x, -1.0 * (k * -0.5));
    // Below the 0.2 dead zone nothing moves (free cam, no target).
    h.target = None;
    let before = h.v;
    h.pad = PadMaster { button: 0, nml_sub_stick_x: 0.19, nml_sub_stick_y: -0.19 };
    s.fn_802277e8(&mut h, Some(c), 0);
    assert_eq!(h.v, before);
}
