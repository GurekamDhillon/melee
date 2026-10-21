//! Port of src/melee/db/dbcamera.c (debug: stage-visual cycling, camera info overlay, the
//! debug free/follow camera driven by the C-stick).
//!
//! Upward callees (cm/camera.c lives in melee_world, gr, ft, CObj) go through [`DbCameraHost`];
//! [`ext::Ext`] is the unported-callee stub set. Float math is native and follows the shipping
//! build's contraction (checked by tests/tu_db_pure.rs against oracle/tu_db_ref.c).
#![allow(non_snake_case)]

use super::dbcpu::{show_overlay, DevTextHost, DevTextId, Rgba};
use super::dbeffect::pad;
use crate::lbtrigf::{acosf, atan2f};
use crate::lbvector;
use crate::mtx::{self, Vec3};

/// `GM_CAMERA_MODE` (gm/forward.h).
pub const GM_CAMERA_MODE: i32 = 0x0A;

/// The `cm_80453004` debug-camera vectors this TU edits.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DbgCamVecs {
    pub free_eye_pos: Vec3,
    pub free_int_pos: Vec3,
    pub follow_eye_offset: Vec3,
    pub follow_int_offset: Vec3,
    pub follow_eye_pos: Vec3,
    pub follow_int_pos: Vec3,
}

/// Pad state read from `HSD_PadMasterStatus[port]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PadMaster {
    pub button: u32,
    pub nml_sub_stick_x: f32,
    pub nml_sub_stick_y: f32,
}

/// HSD_CObj readings (camera gobj's hsd_obj).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CObjView {
    pub eye: Vec3,
    pub interest: Vec3,
    pub fov: f32,
    pub eye_vector: Vec3,
    pub eye_distance: f32,
    pub left_vector: Vec3,
    pub up_vector: Vec3,
}

#[allow(non_snake_case)]
pub trait DbCameraHost: DevTextHost {
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn db_MiscVisualEffectsStatus(&mut self) -> i32;
    fn Camera_80030A60(&mut self, v: i32);
    fn Camera_80030A8C(&mut self, v: i32);
    fn Camera_80030B0C(&mut self, v: i32);
    fn Camera_80030B38(&mut self, v: i32);
    fn Camera_80030B64(&mut self, v: i32);
    fn Camera_80030B90(&mut self, v: i32);
    fn Camera_SetStageVisible(&mut self, v: i32);
    fn Camera_SetBackgroundColor(&mut self, r: u8, g: u8, b: u8);
    fn Camera_80030154(&mut self) -> bool;
    fn Camera_80030178(&mut self) -> bool;
    fn Camera_8003006C(&mut self);
    fn Camera_8002FEEC(&mut self, player: i32);
    fn Camera_800300F0(&mut self);
    /// `Camera_8003019C`: the follow target, or NULL.
    fn Camera_8003019C(&mut self) -> Option<Vec3>;
    /// `Camera_80030A50` + `gobj->hsd_obj`: None when there is no camera gobj.
    fn Camera_80030A50(&mut self) -> Option<CObjView>;
    fn cm_80453004(&mut self) -> &mut DbgCamVecs;
    fn Ground_ApplyStageBackgroundColor(&mut self);
    fn gm_8018841C(&mut self) -> i32;
    fn gm_GetCurrentGameMode(&mut self) -> i32;
    fn Mex_Enabled(&mut self, name: &str) -> bool;
    fn HSD_PadMasterStatus(&mut self, port: i32) -> PadMaster;
    /// fn_802270C4 loop body: every fighter with a shadow gets `x20A4.x0_b5 = v`, then
    /// `lbShadow_8000EEE0(gobj)`.
    fn fighters_set_shadow_hidden(&mut self, v: bool);
    /// fn_8022713C loop body: every ground gobj's `x10_flags.b7 = v`.
    fn grounds_set_flag_b7(&mut self, v: bool);
}

#[allow(non_snake_case)]
pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire to cm/camera.c, gr/ground.c, gm, lbshadow, controller).
    pub struct Ext;
    impl DbCameraHost for Ext {
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed") }
        fn db_MiscVisualEffectsStatus(&mut self) -> i32 { todo!("db_MiscVisualEffectsStatus") }
        fn Camera_80030A60(&mut self, _: i32) { todo!("Camera_80030A60") }
        fn Camera_80030A8C(&mut self, _: i32) { todo!("Camera_80030A8C") }
        fn Camera_80030B0C(&mut self, _: i32) { todo!("Camera_80030B0C") }
        fn Camera_80030B38(&mut self, _: i32) { todo!("Camera_80030B38") }
        fn Camera_80030B64(&mut self, _: i32) { todo!("Camera_80030B64") }
        fn Camera_80030B90(&mut self, _: i32) { todo!("Camera_80030B90") }
        fn Camera_SetStageVisible(&mut self, _: i32) { todo!("Camera_SetStageVisible") }
        fn Camera_SetBackgroundColor(&mut self, _: u8, _: u8, _: u8) { todo!("Camera_SetBackgroundColor") }
        fn Camera_80030154(&mut self) -> bool { todo!("Camera_80030154") }
        fn Camera_80030178(&mut self) -> bool { todo!("Camera_80030178") }
        fn Camera_8003006C(&mut self) { todo!("Camera_8003006C") }
        fn Camera_8002FEEC(&mut self, _: i32) { todo!("Camera_8002FEEC") }
        fn Camera_800300F0(&mut self) { todo!("Camera_800300F0") }
        fn Camera_8003019C(&mut self) -> Option<Vec3> { todo!("Camera_8003019C") }
        fn Camera_80030A50(&mut self) -> Option<CObjView> { todo!("Camera_80030A50") }
        fn cm_80453004(&mut self) -> &mut DbgCamVecs { todo!("cm_80453004") }
        fn Ground_ApplyStageBackgroundColor(&mut self) { todo!("Ground_ApplyStageBackgroundColor") }
        fn gm_8018841C(&mut self) -> i32 { todo!("gm_8018841C") }
        fn gm_GetCurrentGameMode(&mut self) -> i32 { todo!("gm_GetCurrentGameMode") }
        fn Mex_Enabled(&mut self, _: &str) -> bool { todo!("Mex_Enabled") }
        fn HSD_PadMasterStatus(&mut self, _: i32) -> PadMaster { todo!("HSD_PadMasterStatus") }
        fn fighters_set_shadow_hidden(&mut self, _: bool) { todo!("lbShadow_8000EEE0 loop") }
        fn grounds_set_flag_b7(&mut self, _: bool) { todo!("Ground x10_flags loop") }
    }
    impl DevTextHost for Ext {
        fn DevText_GetGObj(&mut self) -> Option<u32> { todo!("DevText_GetGObj") }
        fn DevText_Create(&mut self, _: i32, _: i32, _: i32, _: i32, _: i32, _: usize) -> DevTextId { todo!("DevText_Create") }
        fn DevText_Show(&mut self, _: Option<u32>, _: DevTextId) { todo!("DevText_Show") }
        fn DevText_HideCursor(&mut self, _: DevTextId) { todo!("DevText_HideCursor") }
        fn DevText_SetBGColor(&mut self, _: DevTextId, _: Rgba) { todo!("DevText_SetBGColor") }
        fn DevText_SetTextColor(&mut self, _: DevTextId, _: Rgba) { todo!("DevText_SetTextColor") }
        fn DevText_SetScale(&mut self, _: DevTextId, _: f32, _: f32) { todo!("DevText_SetScale") }
        fn DevText_Erase(&mut self, _: DevTextId) { todo!("DevText_Erase") }
        fn DevText_SetCursorXY(&mut self, _: DevTextId, _: i32, _: i32) { todo!("DevText_SetCursorXY") }
        fn DevText_SetCursorX(&mut self, _: DevTextId, _: i32) { todo!("DevText_SetCursorX") }
        fn DevText_Printf(&mut self, _: DevTextId, _: &str) { todo!("DevText_Printf") }
        fn DevText_ShowBackground(&mut self, _: DevTextId) { todo!("DevText_ShowBackground") }
        fn DevText_HideBackground(&mut self, _: DevTextId) { todo!("DevText_HideBackground") }
        fn DevText_ShowText(&mut self, _: DevTextId) { todo!("DevText_ShowText") }
        fn DevText_HideText(&mut self, _: DevTextId) { todo!("DevText_HideText") }
    }
}

pub const G_BG: Rgba = Rgba(0, 0, 0, 0x80);
pub const G_FG: Rgba = Rgba(0xFF, 0xFF, 0xFF, 0xFF);

/// `db` (MiscStageVisualsStatus), `db_CameraInfoDisplay`, `db_CameraInfoDisplayTimer`,
/// `db_ShowCameraInfo`.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbCamera {
    pub misc_stage_visuals_status: u32,
    pub camera_info_display: DevTextId,
    pub camera_info_display_timer: u8,
    pub show_camera_info: u8,
}

// ---- pure helpers (verified bit for bit) --------------------------------------------------

/// `cstick_threshold` (static inline).
#[inline]
pub fn cstick_threshold(cstick: f32, val: f32) -> f32 {
    let a = if cstick < 0.0 { -cstick } else { cstick };
    if a < val {
        0.0
    } else {
        cstick
    }
}

/// fn_80227188's `ABS(v) > 99999.0F ? -1 : (int) v`.
#[inline]
pub fn info_int(v: f32) -> i32 {
    let a = if v < 0.0 { -v } else { v };
    if a > 99999.0 {
        -1
    } else {
        crate::cabi::f2i32(v as f64)
    }
}

/// fn_80227EB0 / fn_80227FE0 pan scale: `0.03F * (2.0F * (dist * tanf(MTXDegToRad(fov) / 2)))`.
#[inline]
pub fn pan_scale(eye_dist: f32, fov: f32) -> f32 {
    0.03f32 * (2.0f32 * (eye_dist * crate::gekko::tanf((fov * 0.01745329252f32) / 2.0)))
}

/// fn_80228124 pan scale: `0.03f * (2.0f * (dist * tanf(0.017453292f * fov / 2)))`.
#[inline]
pub fn pan_scale_follow(dist: f32, fov: f32) -> f32 {
    0.03f32 * (2.0f32 * (dist * crate::gekko::tanf((0.017453292f32 * fov) / 2.0)))
}

/// fn_80227D38 dolly: `dist * -(0.05F * arg2 - 1.0F)`; the inner expression is one fmadds.
#[inline]
pub fn dolly_follow(dist: f32, arg2: f32) -> f32 {
    dist * -melee_core::fp::fmsubs(0.05, arg2, 1.0)
}

/// fn_80227CAC dolly: `dist * (1.0F - 0.05F * y)`; the inner expression is one fused op.
#[inline]
pub fn dolly_free(dist: f32, y: f32) -> f32 {
    dist * ((1.0f64 - 0.05f32 as f64 * y as f64) as f32)
}

/// `fn_802279E8` (Debug_OrbitAroundInterest): `cobj` supplies the eye vector and distance.
pub fn fn_802279e8(cobj: &CObjView, camera_pos: &mut Vec3, camera_interest: &Vec3, cstick_x: f32, cstick_y: f32) {
    if cstick_x != 0.0 || cstick_y != 0.0 {
        let up_vec = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
        let mut forward_vec = cobj.eye_vector;
        let pitch = 57.29578f32 * acosf(mtx::vec_dot(&forward_vec, &up_vec));
        let mut new_pitch = 2.0f32 * cstick_y;
        if pitch + new_pitch > 179.0 {
            new_pitch = 179.0 - pitch;
        }
        if pitch + new_pitch < 1.0 {
            new_pitch = 1.0 - pitch;
        }
        let axis = mtx::vec_normalize(&mtx::vec_cross(&up_vec, &forward_vec));
        let m = mtx::rot_axis_rad(&axis, 0.017453292f32 * new_pitch);
        forward_vec = mtx::mult_vec(&m, &forward_vec);
        let eye_dist = cobj.eye_distance;
        let m = mtx::rot_axis_rad(&up_vec, 0.017453292f32 * (2.0f32 * cstick_x));
        forward_vec = mtx::mult_vec(&m, &forward_vec);
        forward_vec = mtx::vec_scale(&forward_vec, eye_dist);
        *camera_pos = mtx::vec_sub(camera_interest, &forward_vec);
    }
}

fn follow_commit(c: &mut DbgCamVecs, target: &Vec3) {
    // The C stores int_offset+target into follow_eye_pos and eye_offset+target into
    // follow_int_pos (sic).
    let mut a = c.follow_int_offset;
    lbvector::add(&mut a, target);
    c.follow_eye_pos = a;
    let mut b = c.follow_eye_offset;
    lbvector::add(&mut b, target);
    c.follow_int_pos = b;
}

impl DbCamera {
    /// `fn_SetupMiscStageVisuals` (0x80226E00).
    pub fn fn_setup_misc_stage_visuals(&mut self) {
        self.misc_stage_visuals_status = 0;
    }

    /// `fn_CheckMiscStageEffects` (0x80226E0C).
    pub fn fn_check_misc_stage_effects<H: DbCameraHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if (h.db_ButtonsDown(player) & pad::R) != 0 && (h.db_ButtonsPressed(player) & pad::DPADDOWN) != 0 {
            self.misc_stage_visuals_status = self.misc_stage_visuals_status.wrapping_add(1);
            // (fn_802270C4, B0C, A60, A8C, fn_8022713C, B38, B64, B90)
            let row: [i32; 8] = match self.misc_stage_visuals_status {
                1 => [0, 0, 0, 0, 0, 0, 0, 0],
                2 => [1, 1, 0, 1, 0, 0, 0, 1],
                3 => [0, 0, 0, 1, 1, 0, 0, 1],
                4 => [0, 0, 1, 1, 1, 0, 0, 1],
                5 => [0, 0, 1, 1, 1, 1, 0, 1],
                6 => [0, 0, 1, 1, 1, 0, 1, 1],
                _ => [1, 0, 0, 0, 0, 0, 0, 0],
            };
            self.fn_802270c4(h, row[0]);
            h.Camera_80030B0C(row[1]);
            h.Camera_80030A60(row[2]);
            h.Camera_80030A8C(row[3]);
            self.fn_8022713c(h, row[4]);
            h.Camera_80030B38(row[5]);
            h.Camera_80030B64(row[6]);
            h.Camera_80030B90(row[7]);
            if !(1..=6).contains(&self.misc_stage_visuals_status) {
                self.misc_stage_visuals_status = 0;
            }
        }
        if (h.db_ButtonsDown(player) & pad::X) != 0 && (h.db_ButtonsPressed(player) & pad::DPADDOWN) != 0 {
            match h.db_MiscVisualEffectsStatus() {
                0 | 1 => {
                    h.Ground_ApplyStageBackgroundColor();
                    h.Camera_SetStageVisible(1);
                }
                _ => h.Camera_SetStageVisible(0),
            }
        }
    }

    /// `fn_802270C4`.
    pub fn fn_802270c4<H: DbCameraHost + ?Sized>(&mut self, h: &mut H, arg0: i32) {
        h.fighters_set_shadow_hidden(arg0 == 0);
    }

    /// `fn_8022713C`.
    pub fn fn_8022713c<H: DbCameraHost + ?Sized>(&mut self, h: &mut H, arg0: i32) {
        h.grounds_set_flag_b7(arg0 & 1 != 0);
    }

    /// `fn_SetupCameraInfo` (0x80227174).
    pub fn fn_setup_camera_info(&mut self) {
        self.camera_info_display = None;
        self.camera_info_display_timer = 0;
        self.show_camera_info = 0;
    }

    /// `fn_CheckCameraInfo_helper` (static, inlined in the C).
    pub fn fn_check_camera_info_helper<H: DbCameraHost + ?Sized>(&mut self, h: &mut H) {
        let gobj = h.DevText_GetGObj();
        if self.camera_info_display.is_none() && gobj.is_some() {
            self.camera_info_display = h.DevText_Create(12, 420, 360, 32, 3, 0xC0);
            let t = self.camera_info_display;
            if t.is_some() {
                show_overlay(h, gobj, t, G_BG, G_FG, 12.0, 16.0);
                h.DevText_HideBackground(t);
                h.DevText_HideText(t);
            }
        }
    }

    /// `fn_80227188`: refresh the camera info overlay.
    pub fn fn_80227188<H: DbCameraHost + ?Sized>(&mut self, h: &mut H) {
        let cam = h.Camera_80030A50();
        let t = self.camera_info_display;
        if t.is_none() {
            return;
        }
        match cam {
            Some(cobj) if self.show_camera_info != 0 => {
                if self.camera_info_display_timer > 1 {
                    let camera = cobj.eye;
                    let interest = cobj.interest;
                    let fov = cobj.fov;
                    let ang = atan2f(interest.y - camera.y, -(interest.z - camera.z)) * 57.29577951f32;
                    h.DevText_Erase(t);
                    h.DevText_SetCursorXY(t, 0, 0);
                    let (ez, ey, ex) = (info_int(camera.z), info_int(camera.y), info_int(camera.x));
                    h.DevText_Printf(t, &format!("EYE {},{},{}", ex, ey, ez));
                    h.DevText_SetCursorXY(t, 0, 1);
                    let (iz, iy, ix) = (info_int(interest.z), info_int(interest.y), info_int(interest.x));
                    h.DevText_Printf(t, &format!("INT {},{},{}", ix, iy, iz));
                    h.DevText_SetCursorXY(t, 0, 2);
                    h.DevText_Printf(
                        t,
                        &format!("FOV {}  ANG {}", crate::cabi::f2i32(fov as f64), crate::cabi::f2i32(ang as f64)),
                    );
                    h.DevText_ShowBackground(t);
                    h.DevText_ShowText(t);
                } else if self.camera_info_display_timer == 1 {
                    h.DevText_HideBackground(t);
                    h.DevText_HideText(t);
                } else {
                    return;
                }
                self.camera_info_display_timer = self.camera_info_display_timer.wrapping_sub(1);
            }
            _ => {
                h.DevText_HideBackground(t);
                h.DevText_HideText(t);
            }
        }
    }

    /// `fn_CheckCameraInfo` (0x80227484), with the TARGET_PC m-ex dpad-down rotate opt-in.
    pub fn fn_check_camera_info<H: DbCameraHost + ?Sized>(
        &mut self,
        h: &mut H,
        player: i32,
        buttons_down: i32,
        buttons_pressed: i32,
        cstick_x: f32,
        cstick_y: f32,
    ) {
        if h.gm_8018841C() == 0 && h.gm_GetCurrentGameMode() != GM_CAMERA_MODE {
            let abs = |v: f32| if v < 0.0 { -v } else { v };
            let stick = !h.Camera_80030178()
                && !h.Camera_80030154()
                && (abs(cstick_x) > 0.6 || abs(cstick_y) > 0.6)
                && !(h.Mex_Enabled("rotate_camera_dpad_down")
                    && (h.HSD_PadMasterStatus(player).button & pad::DPADDOWN) != 0);
            if stick {
                self.fn_check_camera_info_helper(h);
                h.Camera_8003006C();
            } else if (buttons_pressed as u32 & pad::DPADUP) != 0 {
                self.fn_check_camera_info_helper(h);
                if (buttons_down as u32 & (pad::Y | pad::X | pad::L | pad::R)) == 0 {
                    if h.Camera_80030178() {
                        h.Camera_8002FEEC(player);
                    } else if h.Camera_80030154() {
                        h.Camera_800300F0();
                    } else {
                        h.Camera_8003006C();
                    }
                }
            }
        }
        if h.Camera_80030154() {
            let cam = h.Camera_80030A50();
            self.fn_802277e8(h, cam, player);
        } else if h.Camera_80030178() {
            let cam = h.Camera_80030A50();
            self.fn_80227904(h, cam, player);
        }
        if h.Camera_80030178() && self.misc_stage_visuals_status.wrapping_sub(3) <= 1 {
            if (buttons_pressed as u32 & pad::B) != 0 {
                self.show_camera_info = (self.show_camera_info == 0) as u8;
            }
        } else {
            self.show_camera_info = 0;
        }
        self.fn_80227188(h);
        if (h.db_ButtonsDown(player) & pad::X) != 0 && (h.db_ButtonsPressed(player) & pad::DPADDOWN) != 0 {
            match h.db_MiscVisualEffectsStatus() {
                2 => h.Camera_SetBackgroundColor(0xFF, 0xFF, 0xFF),
                3 => h.Camera_SetBackgroundColor(0, 0, 0),
                _ => {}
            }
        }
    }

    /// `fn_802277E8`: follow-camera input.
    pub fn fn_802277e8<H: DbCameraHost + ?Sized>(&mut self, h: &mut H, cam: Option<CObjView>, port: i32) {
        let p = h.HSD_PadMasterStatus(port & 0xFF);
        let cx = cstick_threshold(p.nml_sub_stick_x, 0.2);
        let cy = cstick_threshold(p.nml_sub_stick_y, 0.2);
        let r4 = h.Camera_8003019C();
        if p.button & 1 != 0 {
            fn_80227d38(h, cam, r4, cy);
        } else if p.button & 2 != 0 {
            fn_80228124(h, cam, r4, -cx, -cy);
        } else {
            fn_80227ba8(h, cam, r4, cx, cy);
        }
    }

    /// `fn_80227904` (Debug_UpdateCamera): free-camera input.
    pub fn fn_80227904<H: DbCameraHost + ?Sized>(&mut self, h: &mut H, cam: Option<CObjView>, port: i32) {
        let p = h.HSD_PadMasterStatus(port & 0xFF);
        let cx = cstick_threshold(p.nml_sub_stick_x, 0.2);
        let cy = cstick_threshold(p.nml_sub_stick_y, 0.2);
        if p.button & 1 != 0 {
            fn_80227cac(h, cam, cy);
        } else if p.button & 2 != 0 {
            fn_80227fe0(h, cam, -cx, -cy);
        } else if !(h.Mex_Enabled("rotate_camera_dpad_down") && (p.button & pad::DPADDOWN) != 0) {
            fn_80227b64(h, cam, cx, cy);
        }
        self.camera_info_display_timer = 0x3C;
    }
}

fn cobj_of(cam: Option<CObjView>) -> CObjView {
    // GET_COBJ on a NULL gobj would fault in the C; the debug camera is always live here.
    cam.expect("GET_COBJ(NULL camera gobj)")
}

/// `fn_80227B64` (Debug_UpdateFreecamRotation).
pub fn fn_80227b64<H: DbCameraHost + ?Sized>(h: &mut H, cam: Option<CObjView>, cx: f32, cy: f32) {
    if cx != 0.0 || cy != 0.0 {
        let cobj = cobj_of(cam);
        let c = h.cm_80453004();
        let int = c.free_int_pos;
        fn_802279e8(&cobj, &mut c.free_eye_pos, &int, cx, cy);
    }
}

/// `fn_80227BA8`.
pub fn fn_80227ba8<H: DbCameraHost + ?Sized>(h: &mut H, cam: Option<CObjView>, arg1: Option<Vec3>, a2: f32, a3: f32) {
    if let Some(target) = arg1 {
        let cobj = cobj_of(cam);
        let c = h.cm_80453004();
        let int = c.follow_int_offset;
        fn_802279e8(&cobj, &mut c.follow_eye_offset, &int, a2, a3);
        follow_commit(c, &target);
        return;
    }
    if a2 != 0.0 || a3 != 0.0 {
        let cobj = cobj_of(cam);
        let c = h.cm_80453004();
        let int = c.free_int_pos;
        fn_802279e8(&cobj, &mut c.free_eye_pos, &int, a2, a3);
    }
}

/// `fn_80227CAC` (Debug_UpdateFreecamDolly).
pub fn fn_80227cac<H: DbCameraHost + ?Sized>(h: &mut H, cam: Option<CObjView>, cy: f32) {
    if cy != 0.0 {
        let cobj = cobj_of(cam);
        let fwd = mtx::vec_scale(&cobj.eye_vector, dolly_free(cobj.eye_distance, cy));
        let c = h.cm_80453004();
        c.free_eye_pos = mtx::vec_sub(&c.free_int_pos, &fwd);
    }
}

/// `fn_80227D38`.
pub fn fn_80227d38<H: DbCameraHost + ?Sized>(h: &mut H, cam: Option<CObjView>, arg1: Option<Vec3>, a2: f32) {
    if let Some(target) = arg1 {
        if a2 != 0.0 {
            let cobj = cobj_of(cam);
            let v = mtx::vec_scale(&cobj.eye_vector, dolly_follow(cobj.eye_distance, a2));
            let c = h.cm_80453004();
            c.follow_eye_offset = mtx::vec_sub(&c.follow_int_offset, &v);
        }
        follow_commit(h.cm_80453004(), &target);
    } else if a2 != 0.0 {
        let cobj = cobj_of(cam);
        let v = mtx::vec_scale(&cobj.eye_vector, dolly_follow(cobj.eye_distance, a2));
        let c = h.cm_80453004();
        c.free_eye_pos = mtx::vec_sub(&c.free_int_pos, &v);
    }
}

/// `fn_80227EB0`: pan both `a1` and `a2` along the camera's left/up vectors.
pub fn fn_80227eb0(cobj: &CObjView, a1: &mut Vec3, a2: &mut Vec3, x: f32, y: f32) {
    let dist = pan_scale(cobj.eye_distance, cobj.fov);
    if x != 0.0 {
        let v = mtx::vec_scale(&cobj.left_vector, dist * x);
        *a2 = mtx::vec_add(a2, &v);
        *a1 = mtx::vec_add(a1, &v);
    }
    if y != 0.0 {
        let v = mtx::vec_scale(&cobj.up_vector, -dist * y);
        *a2 = mtx::vec_add(a2, &v);
        *a1 = mtx::vec_add(a1, &v);
    }
}

/// `fn_80227FE0` (Debug_UpdateFreecamPosition).
pub fn fn_80227fe0<H: DbCameraHost + ?Sized>(h: &mut H, cam: Option<CObjView>, cx: f32, cy: f32) {
    if cx != 0.0 || cy != 0.0 {
        let cobj = cobj_of(cam);
        let s = pan_scale(cobj.eye_distance, cobj.fov);
        let c = h.cm_80453004();
        if cx != 0.0 {
            let v = mtx::vec_scale(&cobj.left_vector, s * cx);
            c.free_int_pos = mtx::vec_add(&c.free_int_pos, &v);
            c.free_eye_pos = mtx::vec_add(&c.free_eye_pos, &v);
        }
        if cy != 0.0 {
            let v = mtx::vec_scale(&cobj.up_vector, -s * cy);
            c.free_int_pos = mtx::vec_add(&c.free_int_pos, &v);
            c.free_eye_pos = mtx::vec_add(&c.free_eye_pos, &v);
        }
    }
}

/// `fn_80228124`.
pub fn fn_80228124<H: DbCameraHost + ?Sized>(h: &mut H, cam: Option<CObjView>, arg1: Option<Vec3>, a2: f32, a3: f32) {
    if let Some(target) = arg1 {
        let cobj = cobj_of(cam);
        let dist = pan_scale_follow(cobj.eye_distance, cobj.fov);
        let c = h.cm_80453004();
        if a2 != 0.0 {
            let v = mtx::vec_scale(&cobj.left_vector, dist * a2);
            c.follow_int_offset = mtx::vec_add(&c.follow_int_offset, &v);
            c.follow_eye_offset = mtx::vec_add(&c.follow_eye_offset, &v);
        }
        if a3 != 0.0 {
            let v = mtx::vec_scale(&cobj.up_vector, -dist * a3);
            c.follow_int_offset = mtx::vec_add(&c.follow_int_offset, &v);
            c.follow_eye_offset = mtx::vec_add(&c.follow_eye_offset, &v);
        }
        follow_commit(c, &target);
    } else if a2 != 0.0 || a3 != 0.0 {
        // fn_80227EB0_dummy_inline(camera, &free_eye_pos, &free_int_pos, ...)
        let cobj = cobj_of(cam);
        let c = h.cm_80453004();
        let (mut e, mut i) = (c.free_eye_pos, c.free_int_pos);
        fn_80227eb0(&cobj, &mut e, &mut i, a2, a3);
        c.free_eye_pos = e;
        c.free_int_pos = i;
    }
}
