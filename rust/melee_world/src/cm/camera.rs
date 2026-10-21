#![allow(non_upper_case_globals)]
//! Port of the camera framing / follow math and the `game_camera` accessors of
//! src/melee/cm/camera.c (port round 5, night 2).
//!
//! The C keeps everything in globals (`game_camera`, `cm_803BCCA0`, `cm_803BCB64`, the
//! `cm_804D6468` subject list, the stage's camera info). Here they are fields of
//! [`CameraWorld`]. The subject list is `subjects`, in the order the C walks it
//! (`cm_804D6468`, then `->prev`); a subject is addressed by its index in that list.
//!
//! Stage getters (`Stage_GetCam*`, `Stage_UnkSetVec3TCam_Offset`, `Ground_801C4368`) live in
//! gr/stage.c and gr/ground.c, not yet ported: their return values are plain inputs in
//! [`CamStage`]. TODO(group2: gr/stage.c, gr/ground.c).
//!
//! The TARGET_PC build wraps the four `Stage_GetCamBounds*Offset` calls in `cm_fly_*`, which
//! open the bounds to +-100000 while a fighter flies (`GenoFly_Any`) or a script camera lifts
//! them; that is [`CameraWorld::geno_fly_any`] / [`CameraWorld::script_lift_bounds`].
//!
//! FMA: every `fmadds` below is a site `tools/rust-port/fma_sites.py` reports for the
//! shipping build; nothing else fuses.
use melee_core::fp::fmadds;
use melee_runtime::gekko::tanf;
use melee_types::{
    Camera, CameraBounds, CameraQuake, CameraTransformState, CameraUnkGlobals, CmQuakeKind,
    CmSubject, Vec as Vec3, Vec2, _GXColor, CAMERA_BOSS_INTRO, CAMERA_DEBUG_FOLLOW,
    CAMERA_DEBUG_FREE, CAMERA_FREE, CAM_BOUNDS_INSIDE, CAM_BOUNDS_OUTSIDE_BOTTOM,
    CAM_BOUNDS_OUTSIDE_LEFT, CAM_BOUNDS_OUTSIDE_RIGHT, CAM_BOUNDS_OUTSIDE_TOP,
    CmSubjectState_Auto, CmSubjectState_Inactive, QuakeKind_Count, QuakeKind_Large,
    QuakeKind_Loop, QuakeKind_Medium, QuakeKind_None, QuakeKind_Small,
};

const F32_MAX: f32 = 3.402_823_466e38;
/// `MTXDegToRad` (dolphin/mtx.h): `a * 0.01745329252f`.
#[inline(always)]
fn deg_to_rad(a: f32) -> f32 {
    a * 0.017_453_292_52_f32
}

/// `cm_803BCB9C`: framing weight by subject count.
pub const CM_803BCB9C: [f32; 5] = [0.0, 1.5, 1.32, 1.16, 1.0];

/// `cm_803BCCA0`, the camera tuning table (camera.c initializer, in field order).
pub const CM_803BCCA0_INIT: [f32; 60] = [
    83.0, 1000.0, -30.0, 5.0, -7.0, 17.5, -17.5, 0.0, 0.0682, 60.0, 120.0, 0.05, 0.1, 120.0,
    900.0, 0.15, 38.0, 0.1, 0.1, 0.001, 0.1, 1.0, 1.0, 0.6, 0.6, 0.05, 0.1, 29.0, 0.1, 0.1, 0.1,
    0.1, 0.5, 0.5, 0.4, -11.0, 400.0, 0.2, 4.0, 0.05, 1.0, -7.0, 7.0, 0.5, 0.5, 0.004, 0.2,
    0.025, 0.2, 0.003, 0.2, 0.025, 0.2, 0.02, 1.0, 0.14, 1200.0, -0.2, 1.2, 0.0,
];

/// `cm_803BCCA0` as a `CameraUnkGlobals` (60 consecutive f32 at GC +0x0..+0xEC).
pub fn cm_803bcca0() -> CameraUnkGlobals {
    const _: () = assert!(core::mem::size_of::<CameraUnkGlobals>() == 60 * 4);
    // SAFETY: CameraUnkGlobals is repr(C), exactly 60 f32 fields (asserted above).
    unsafe { core::mem::transmute::<[f32; 60], CameraUnkGlobals>(CM_803BCCA0_INIT) }
}

/// The parts of `cm_803BCB64` (HSD_CameraDescPerspective) the ported math reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CamDesc {
    pub aspect: f32,
    /// viewport xmin, xmax, ymin, ymax
    pub viewport: [i16; 4],
}
impl Default for CamDesc {
    fn default() -> Self {
        CamDesc { aspect: 1.217_333_3, viewport: [0, 0x280, 0, 0x1E0] }
    }
}

/// Values of the stage camera getters (gr/stage.c / gr/ground.c, not yet ported).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CamStage {
    /// Raw `Stage_GetCamBounds{Left,Right,Top,Bottom}Offset()`.
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub track_ratio: f32,
    pub zoom_rate: f32,
    pub max_depth: f32,
    pub info_x20: f32,
    pub info_x24: f32,
    pub pan_angle_rad: f32,
    /// `Stage_UnkSetVec3TCam_Offset`.
    pub tcam_offset: [f32; 3],
    /// First output of `Ground_801C4368` (the second is never read here).
    pub ground_801c4368: f32,
}

/// Records of calls into code this crate does not own yet.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraStubLog {
    /// `HSD_GObjFree(game_camera.quake_gobj)` calls. TODO(group1: baselib/gobj.c).
    pub quake_gobj_frees: u32,
    /// `grLib_801C9CEC(kind)` calls. TODO(group2: gr/grlib.c).
    pub grlib_quake: std::vec::Vec<CmQuakeKind>,
}

/// The camera globals of camera.c that the ported functions touch.
pub struct CameraWorld {
    pub game_camera: Camera,
    pub globals: CameraUnkGlobals,
    pub desc: CamDesc,
    pub stage: CamStage,
    /// `cm_804D6468` list order (`->prev` walk).
    pub subjects: std::vec::Vec<CmSubject>,
    /// PC: `GenoFly_Any()`.
    pub geno_fly_any: bool,
    /// PC: `GenoFly_CameraSubject(subject)` per subject index.
    pub geno_fly_subject: std::vec::Vec<bool>,
    /// PC: `cm_script.state && cm_script.lift_bounds`.
    pub script_lift_bounds: bool,
    /// `cm_80453004.last_mode`.
    pub debug_last_mode: u32,
    /// Stand-in for the non-NULL `quake_gobj` handle `grLib_801C9CEC` returns.
    pub quake_gobj_token: usize,
    pub stubs: CameraStubLog,
}

impl Default for CameraWorld {
    fn default() -> Self {
        CameraWorld {
            game_camera: Camera::zeroed(),
            globals: cm_803bcca0(),
            desc: CamDesc::default(),
            stage: CamStage::default(),
            subjects: vec![],
            geno_fly_any: false,
            geno_fly_subject: vec![],
            script_lift_bounds: false,
            debug_last_mode: 0,
            quake_gobj_token: 0x1000,
            stubs: CameraStubLog::default(),
        }
    }
}

impl CameraWorld {
    #[inline]
    fn lifted(&self) -> bool {
        self.geno_fly_any || self.script_lift_bounds
    }
    /// PC `cm_fly_left` (= the build's `Stage_GetCamBoundsLeftOffset()`).
    pub fn bounds_left(&self) -> f32 {
        if self.lifted() { -100000.0 } else { self.stage.left }
    }
    pub fn bounds_right(&self) -> f32 {
        if self.lifted() { 100000.0 } else { self.stage.right }
    }
    pub fn bounds_top(&self) -> f32 {
        if self.lifted() { 100000.0 } else { self.stage.top }
    }
    pub fn bounds_bottom(&self) -> f32 {
        if self.lifted() { -100000.0 } else { self.stage.bottom }
    }
    /// `Ground_801C4368(&g, &_); g += 1.0;`
    #[inline]
    fn ground_plus_one(&self) -> f32 {
        (self.stage.ground_801c4368 as f64 + 1.0) as f32
    }

    /// Camera_80029124: which camera bounds `pos` lies outside of (CAM_BOUNDS_* flags).
    pub fn camera_80029124(&self, pos: &Vec3, distance: i32) -> u32 {
        let mut flags = CAM_BOUNDS_INSIDE as u32;
        let slope = self.ground_plus_one();
        let left = self.bounds_left();
        let right = self.bounds_right();
        let top = self.bounds_top();
        let bottom = if self.bounds_bottom() > slope { self.bounds_bottom() } else { slope };
        let d = distance as f32;
        if pos.x < left - d {
            flags |= CAM_BOUNDS_OUTSIDE_LEFT as u32;
        }
        if pos.x > right + d {
            flags |= CAM_BOUNDS_OUTSIDE_RIGHT as u32;
        }
        if pos.y > top + d {
            flags |= CAM_BOUNDS_OUTSIDE_TOP as u32;
        }
        if pos.y < bottom - d {
            flags |= CAM_BOUNDS_OUTSIDE_BOTTOM as u32;
        }
        flags
    }

    /// Camera_8002928C: is subject `i` framed this frame (ticks its auto timer).
    pub fn camera_8002928c(&mut self, i: usize) -> bool {
        if self.geno_fly_any && !self.geno_fly_subject.get(i).copied().unwrap_or(false) {
            return false;
        }
        let (left, right, top, bottom) =
            (self.bounds_left(), self.bounds_right(), self.bounds_top(), self.bounds_bottom());
        let s = &mut self.subjects[i];
        if s.state != CmSubjectState_Inactive && s.force_inactive() == 0 {
            if s.state == CmSubjectState_Auto {
                if s.state_timer != 0 {
                    s.state_timer = s.state_timer.wrapping_sub(1);
                    return false;
                }
                let cam_bound = |x: f32| x > 0.65 || x < 0.35;
                if cam_bound((s.pos.x - left) / (right - left))
                    || cam_bound((s.pos.y - bottom) / (top - bottom))
                {
                    if s.was_framed() != 0 {
                        s.set_was_framed(0);
                        s.state_timer = 600;
                    }
                    return false;
                }
                s.state_timer = 0;
            }
            s.set_was_framed(1);
            return true;
        }
        false
    }

    /// Camera_800293E0: ease every framed subject's extents toward their targets (0.5/frame).
    pub fn camera_800293e0(&mut self) {
        fn step(cur: &mut f32, target: f32) {
            let d = target - *cur;
            if d != 0.0 {
                if d > 0.5 {
                    *cur += 0.5;
                } else if d < -0.5 {
                    *cur -= 0.5;
                } else {
                    *cur = target;
                }
            }
        }
        for i in 0..self.subjects.len() {
            if self.camera_8002928c(i) {
                let s = &mut self.subjects[i];
                let t = s.target_ext;
                step(&mut s.ext.h.x, t.h.x);
                step(&mut s.ext.h.y, t.h.y);
                step(&mut s.ext.v.x, t.v.x);
                step(&mut s.ext.v.y, t.v.y);
                step(&mut s.ext.v.z, t.v.z);
            }
        }
    }

    /// The clamp block repeated five times in Camera_8002958C.
    fn clamp_to_bounds(&self, p: &mut Vec3) {
        let flags = self.camera_80029124(p, 0);
        if flags != CAM_BOUNDS_INSIDE as u32 {
            let g = self.ground_plus_one();
            if flags & CAM_BOUNDS_OUTSIDE_LEFT as u32 != 0 {
                p.x = self.bounds_left();
            }
            if flags & CAM_BOUNDS_OUTSIDE_RIGHT as u32 != 0 {
                p.x = self.bounds_right();
            }
            if flags & CAM_BOUNDS_OUTSIDE_TOP as u32 != 0 {
                p.y = self.bounds_top();
            }
            if flags & CAM_BOUNDS_OUTSIDE_BOTTOM as u32 != 0 {
                p.y = if self.bounds_bottom() > g { self.bounds_bottom() } else { g };
            }
        }
    }

    /// Camera_8002958C: the box framing every subject, clamped to the stage camera bounds.
    pub fn camera_8002958c(&mut self, bounds: &mut CameraBounds, transform: &CameraTransformState) {
        let n = self.subjects.len();
        let mut count = 0i32;
        for i in 0..n {
            if self.camera_8002928c(i) {
                count += 1;
            }
        }
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        if count != 0 {
            let weight = if (count as u32) < 5 { CM_803BCB9C[count as usize] } else { 1.0 };
            let mult = weight * self.stage.track_ratio;
            min_x = F32_MAX;
            min_y = F32_MAX;
            max_x = -F32_MAX;
            max_y = -F32_MAX;
            count = 0;
            for i in 0..n {
                if !self.camera_8002928c(i) {
                    continue;
                }
                count += 1;
                let s = &self.subjects[i];
                let (ext, spos) = (s.ext, s.pos);
                let mut base = spos;
                let mut test = spos;
                self.clamp_to_bounds(&mut base);
                test.x = fmadds(ext.h.x, mult, base.x);
                self.clamp_to_bounds(&mut test);
                if test.x < min_x {
                    min_x = test.x;
                }
                if test.x > max_x {
                    max_x = test.x;
                }
                test.x = fmadds(ext.h.y, mult, base.x);
                self.clamp_to_bounds(&mut test);
                if test.x < min_x {
                    min_x = test.x;
                }
                if test.x > max_x {
                    max_x = test.x;
                }
                test.y = fmadds(ext.v.y, mult, base.y);
                self.clamp_to_bounds(&mut test);
                if test.y < min_y {
                    min_y = test.y;
                }
                if test.y > max_y {
                    max_y = test.y;
                }
                test.y = fmadds(ext.v.x, mult, base.y);
                self.clamp_to_bounds(&mut test);
                if test.y < min_y {
                    min_y = test.y;
                }
                if test.y > max_y {
                    max_y = test.y;
                }
            }
        }
        if count == 0 {
            let o = self.stage.tcam_offset;
            min_x = o[0] - 40.0;
            min_y = o[1] - 40.0;
            max_x = 40.0 + o[0];
            max_y = 40.0 + o[1];
        }
        let pz = transform.position.z;
        let z_pos = if pz < 0.0 { -pz } else { pz };
        let z_factor = if z_pos < 80.0 {
            0.0
        } else if z_pos > 5000.0 {
            1.0
        } else {
            (z_pos - 80.0) / 4920.0
        };
        bounds.x_min = min_x;
        bounds.y_min = min_y - fmadds(390.0, z_factor, 10.0);
        bounds.x_max = max_x;
        bounds.y_max = max_y;
        bounds.total_subjects = count;
        bounds.z_pos = z_pos;
    }

    /// Camera_80029AAC: move the interest toward its target at the spread-dependent speed.
    pub fn camera_80029aac(&self, bounds: &CameraBounds, transform: &mut CameraTransformState, speed: f32) {
        let spread = if bounds.total_subjects != 0 {
            let dx = bounds.x_max - bounds.x_min;
            let dy = bounds.y_max - bounds.y_min;
            if dx > dy { dx } else { dy }
        } else {
            99999.0
        };
        let off_x = transform.target_interest.x - transform.interest.x;
        let off_y = transform.target_interest.y - transform.interest.y;
        let g = &self.globals;
        // get_follow_speed(g->x38, spread, g)
        let follow = if spread > g.x38 {
            g.x30
        } else if spread < g.x34 {
            g.x2C
        } else {
            fmadds((spread - g.x34) / (g.x38 - g.x34), g.x30 - g.x2C, g.x2C)
        };
        let x2bc = self.game_camera.x2BC;
        let delta = if x2bc > 0.0001 { 1.0 / x2bc } else { 1000.0 };
        let mut lerp = (follow * speed) * delta;
        if lerp > 1.0 {
            lerp = 1.0;
        } else if lerp < 0.0001 {
            lerp = 0.0001;
        }
        transform.interest.x = fmadds(off_x, lerp, transform.interest.x);
        transform.interest.y = fmadds(off_y, lerp, transform.interest.y);
    }

    /// Camera_80029BC4: the depth that fits the bounds box in the target fov.
    pub fn camera_80029bc4(&self, bounds: &mut CameraBounds, transform: &CameraTransformState) {
        let mut d = (bounds.y_max - bounds.y_min) / tanf(deg_to_rad(transform.target_fov));
        let x = (bounds.x_max - bounds.x_min)
            / (self.desc.aspect * tanf(deg_to_rad(transform.target_fov)));
        if x > d {
            d = x;
        }
        if d < self.stage.zoom_rate {
            d = self.stage.zoom_rate;
        }
        if d > self.stage.max_depth {
            d = self.stage.max_depth;
        }
        bounds.z_pos = d;
    }

    /// Camera_80029C88: move the position toward its target.
    pub fn camera_80029c88(&self, transform: &mut CameraTransformState, arg_scale: f32) {
        let dx = transform.target_position.x - transform.position.x;
        let dy = transform.target_position.y - transform.position.y;
        let dz = transform.target_position.z - transform.position.z;
        let mut scale = self.globals.x3C * arg_scale;
        if scale > 1.0 {
            scale = 1.0;
        }
        transform.position.x = fmadds(dx, scale, transform.position.x);
        transform.position.y = fmadds(dy, scale, transform.position.y);
        transform.position.z = fmadds(dz, scale, transform.position.z);
    }

    /// Camera_80029CF8: target interest and position framing the bounds box, with the stage's
    /// pan tilt. The C's HSD_ASSERTs (half-fov + angle < 90 degrees) panic.
    pub fn camera_80029cf8(&self, bounds: &CameraBounds, transform: &mut CameraTransformState) {
        let g = &self.globals;
        let st = &self.stage;
        let sp24 = st.tcam_offset;
        let dx = bounds.x_max - bounds.x_min;
        let dy = bounds.y_max - bounds.y_min;
        let spread = if dx > dy { dx } else { dy };
        // get_y_bias
        let t = if spread > g.x28 {
            g.x20
        } else if spread < g.x24 {
            g.x1C
        } else {
            fmadds(g.x20 - g.x1C, (spread - g.x24) / (g.x28 - g.x24), g.x1C)
        };
        let y_sum = (bounds.y_min - sp24[1]) + (bounds.y_max - sp24[1]);
        let base = fmadds(y_sum, 0.5 - t, sp24[1]);
        let mut angle = -deg_to_rad((base + g.x8) * st.info_x24);
        if angle > deg_to_rad(g.xC) {
            angle = deg_to_rad(g.xC);
        }
        if angle < deg_to_rad(g.x10) {
            angle = deg_to_rad(g.x10);
        }
        angle += st.pan_angle_rad;
        let y_angle = angle;
        let lim = deg_to_rad(90.0);
        let fov_u = fmadds(0.5, deg_to_rad(transform.fov), angle);
        assert!(fov_u < lim, "camera.c:1274 fov_u<MTXDegToRad(90.0F)");
        let fov_d = fmadds(0.5, deg_to_rad(transform.fov), -angle);
        assert!(fov_d < lim, "camera.c:1275 fov_d<MTXDegToRad(90.0F)");
        let tan_u = tanf(fov_u);
        let tan_d = tanf(fov_d);
        let dist_y = (bounds.y_max - bounds.y_min) / (tan_u + tan_d);
        let y_off = dist_y * tanf(y_angle);
        transform.target_interest.y = y_off + fmadds(-dist_y, tan_u, bounds.y_max);
        let sp14 = st.tcam_offset;
        let x_center = 0.5 * (bounds.x_min + bounds.x_max);
        let mut angle = -deg_to_rad((x_center - sp14[0]) * st.info_x20);
        if angle > deg_to_rad(g.x14) {
            angle = deg_to_rad(g.x14);
        }
        if angle < deg_to_rad(g.x18) {
            angle = deg_to_rad(g.x18);
        }
        let fov_r = fmadds(0.5, deg_to_rad(transform.fov), -angle);
        assert!(fov_r < lim, "camera.c:1288 fov_r<MTXDegToRad(90.0F)");
        let fov_l = fmadds(0.5, deg_to_rad(transform.fov), angle);
        assert!(fov_l < lim, "camera.c:1289 fov_l<MTXDegToRad(90.0F)");
        let aspect = self.desc.aspect;
        let tan_r = aspect * tanf(fov_r);
        let tan_l = aspect * tanf(fov_l);
        let dist_x = (bounds.x_max - bounds.x_min) / (tan_r + tan_l);
        let x_off = aspect * (dist_x * tanf(angle));
        transform.target_interest.x = fmadds(-dist_x, tan_r, bounds.x_max) - x_off;
        transform.target_interest.z = 0.0;
        let dist_y = if dist_y > dist_x { dist_y } else { dist_x };
        let mut cam_dist = dist_y;
        if dist_y < st.zoom_rate {
            cam_dist = st.zoom_rate;
        }
        if cam_dist > st.max_depth {
            cam_dist = st.max_depth;
        }
        transform.target_position.x = transform.target_interest.x + x_off;
        transform.target_position.y = transform.target_interest.y - y_off;
        transform.target_position.z = transform.target_interest.z + cam_dist;
    }

    /// Camera_ApplyQuake. `one_p_scale` is the PC gate
    /// `gm_IsCurrently1PMode_inline() && !Mex_Enabled("enable_c_stick_always_1p_camera2")`
    /// (TODO(group6: gm_IsCurrently1PMode)). The PC-only `diag_cam_translate` is a diagnostic
    /// print and is not ported.
    pub fn camera_apply_quake(&mut self, bounds: &CameraBounds, state: &CameraTransformState, one_p_scale: bool) {
        let gc = &self.game_camera;
        let g = &self.globals;
        let mut ix = gc.quake_offset.x * gc.quake_scale;
        let mut iy = gc.quake_offset.y * gc.quake_scale;
        ix *= 10.0;
        iy *= 10.0;
        if one_p_scale {
            ix *= g.xE8;
            iy *= g.xE8;
        }
        ix *= gc.x2BC;
        iy *= gc.x2BC;
        let half_h = bounds.z_pos * tanf(0.5 * (0.017_453_292 * state.fov));
        let vp = self.desc.viewport;
        let vxs = self.desc.aspect * (half_h / (0.5 * (vp[1] as i32 - vp[0] as i32) as f32));
        let vys = half_h / (0.5 * (vp[3] as i32 - vp[2] as i32) as f32);
        let dfy = self.stage.zoom_rate;
        let dfx = self.stage.max_depth - dfy;
        let ratio = if dfx < 0.001 { 0.5 } else { (bounds.z_pos - self.stage.zoom_rate) / dfx };
        let dfy = fmadds(ratio, g.x5C - g.x54, g.x54);
        let dfx = fmadds(ratio, g.x60 - g.x58, g.x58);
        self.camera_80030de4(dfx * (ix * vxs), dfy * (iy * vys));
        self.game_camera.quake_offset = Vec2 { x: 0.0, y: 0.0 };
    }

    pub fn camera_set_quake_offset(&mut self, x: f32, y: f32) {
        self.game_camera.quake_offset = Vec2 { x, y };
    }

    /// Camera_UpdateQuakes. `HSD_GObjFree` is a stub: TODO(group1: baselib/gobj.c).
    pub fn camera_update_quakes(&mut self) {
        let gc = &mut self.game_camera;
        let mut remaining = -1i32;
        for i in 0..16 {
            gc.quakes[1][i] = gc.quakes[0][i];
            gc.quakes[0][i].kind = QuakeKind_None;
        }
        for i in 0..QuakeKind_Count as usize {
            if gc.quake_frames_left[i] != 0 {
                gc.quake_frames_left[i] -= 1;
                remaining = i as i32;
            }
        }
        if remaining != -1 && !gc.quake_gobj.is_null() && gc.quake_frames_left[QuakeKind_Loop as usize] == 0 {
            self.stubs.quake_gobj_frees += 1;
            gc.quake_gobj = core::ptr::null_mut();
        }
    }

    /// Camera_RequestQuake. `grLib_801C9CEC` is a stub (TODO(group2: gr/grlib.c)); for
    /// QuakeKind_Loop its returned GObj is `quake_gobj_token`. An unknown kind leaves
    /// `quake_length` at the shipping build's zero-initialised 0.
    pub fn camera_request_quake(&mut self, kind: CmQuakeKind, pos: Option<Vec3>) {
        let mut len = 0i32;
        match kind {
            QuakeKind_Loop => {
                if self.game_camera.quake_gobj.is_null() {
                    self.stubs.grlib_quake.push(kind);
                    self.game_camera.quake_gobj = self.quake_gobj_token as *mut _;
                }
                len = 10;
            }
            QuakeKind_Small | QuakeKind_Medium | QuakeKind_Large => {
                self.stubs.grlib_quake.push(kind);
                len = 22;
            }
            _ => {}
        }
        // The C indexes quake_frames_left[kind] unchecked; kinds outside 0..5 are UB there.
        self.game_camera.quake_frames_left[kind as usize] = len;
        for i in 0..16 {
            let q: &mut CameraQuake = &mut self.game_camera.quakes[0][i];
            if q.kind == QuakeKind_None {
                q.kind = kind;
                q.epicenter = pos.unwrap_or(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
            }
        }
    }

    pub fn camera_stop_quake(&mut self, kind: CmQuakeKind) {
        self.game_camera.quake_frames_left[kind as usize] = 0;
    }

    /// Camera_8002E158: lerp `farg0 -> farg1` by the x341_b5_b6-selected progress; returns
    /// (done, value). Case 3 reads the zero-initialised `var_f4` (shipping build).
    pub fn camera_8002e158(&self, farg0: f32, farg1: f32) -> (bool, f32) {
        let gc = &self.game_camera;
        let mut ret = false;
        let mut f4 = 0.0f32;
        // SAFETY: plain-data unions of f32/i32.
        unsafe {
            match gc.x341_b5_b6() {
                0 => ret = true,
                1 => {
                    if gc.x378.s32_v == gc.x37C.s32_v {
                        ret = true;
                    } else {
                        f4 = gc.x378.s32_v as f32 / gc.x37C.s32_v as f32;
                    }
                }
                2 => {
                    if gc.x378.f32_v >= 1.0 {
                        ret = true;
                    } else {
                        f4 = gc.x378.f32_v;
                    }
                }
                _ => {}
            }
        }
        if ret { (true, farg1) } else { (false, fmadds(f4, farg1 - farg0, farg0)) }
    }

    pub fn camera_8002f260(&self) -> bool {
        self.game_camera.x341_b7() != 0
    }
    pub fn camera_set_mode_to_standard(&mut self) {
        self.game_camera.mode = melee_types::CAMERA_STANDARD;
    }
    pub fn camera_set_mode_to_fixed(&mut self) {
        self.game_camera.mode = melee_types::CAMERA_FIXED;
    }
    pub fn camera_800300f0(&mut self) {
        self.game_camera.mode = self.debug_last_mode;
    }
    pub fn camera_8003010c(&self) -> bool {
        self.game_camera.mode as i32 == CAMERA_FREE as i32
    }
    pub fn camera_80030130(&self) -> bool {
        self.game_camera.mode as i32 == CAMERA_BOSS_INTRO as i32
    }
    pub fn camera_80030154(&self) -> bool {
        self.game_camera.mode as i32 == CAMERA_DEBUG_FOLLOW as i32
    }
    pub fn camera_80030178(&self) -> bool {
        self.game_camera.mode as i32 == CAMERA_DEBUG_FREE as i32
    }
    pub fn camera_80030730(&mut self, v: f32) {
        self.globals.x40 = v;
    }
    pub fn camera_set_background_color(&mut self, r: u8, g: u8, b: u8) {
        let c = &mut self.game_camera.background_color;
        c.r = r;
        c.g = g;
        c.b = b;
    }
    /// Camera_GetBackgroundColor: alpha is the uninitialised local, zero in the shipping build.
    pub fn camera_get_background_color(&self) -> _GXColor {
        let c = self.game_camera.background_color;
        _GXColor { r: c.r, g: c.g, b: c.b, a: 0 }
    }
    pub fn camera_get_transform_position(&self) -> Vec3 {
        self.game_camera.transform.position
    }
    pub fn camera_get_transform_interest(&self) -> Vec3 {
        self.game_camera.transform.interest
    }
    pub fn camera_80030de4(&mut self, x: f32, y: f32) {
        self.game_camera.translation = Vec2 { x, y };
    }
    pub fn camera_80030df8(&mut self) {
        self.game_camera.translation = Vec2 { x: 0.0, y: 0.0 };
    }
    pub fn camera_80030e10(&self) -> f32 {
        if self.game_camera.x2B8 < 1 { 10000.0 } else { self.game_camera.x2B0 }
    }
    pub fn camera_set_quake_scale(&mut self, s: f32) {
        self.game_camera.quake_scale = s;
    }
    pub fn camera_80031060(&self) -> i32 {
        self.game_camera.x398_b6_b7() as i32
    }
    pub fn camera_80031074(&mut self, v: u8) {
        self.game_camera.set_x398_b6_b7(v);
    }
    pub fn camera_8003108c(&self) -> i32 {
        self.game_camera.x399_b0_b1() as i32
    }
    pub fn camera_800310a0(&mut self, v: u8) {
        self.game_camera.set_x399_b0_b1(v);
    }
    pub fn camera_800310e8(&mut self) {
        let gc = &mut self.game_camera;
        gc.set_x398_b0(0);
        gc.set_x398_b1(0);
        gc.set_x398_b2(0);
        gc.set_x398_b3(0);
        gc.set_x398_b4(0);
        gc.set_x398_b5(0);
    }
    pub fn camera_80031144(&self) -> f32 {
        self.game_camera.x2BC
    }
    pub fn camera_80031154(&self, p: &Vec3) -> bool {
        self.camera_80029124(p, 0) == CAM_BOUNDS_INSIDE as u32
    }
    /// Camera_8003118C: `arg1` converts to the s32 `distance` parameter (truncation; only
    /// in-range values are verified).
    pub fn camera_8003118c(&self, p: &Vec3, arg1: f32) -> bool {
        self.camera_80029124(p, arg1 as i32) == CAM_BOUNDS_INSIDE as u32
    }
    pub fn camera_800311cc(&mut self, v: f32) {
        self.game_camera.farz = v;
    }
    pub fn camera_800311dc(&mut self, v: f32) {
        self.game_camera.nearz = v;
    }
    /// PC Camera_StageSlotClipGet: (nearz, farz).
    pub fn camera_stage_slot_clip_get(&self) -> (f32, f32) {
        (self.game_camera.nearz, self.game_camera.farz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The accessors are one-line field moves; checked here, not against the C.
    #[test]
    fn accessors() {
        let mut w = CameraWorld::default();
        assert_eq!(w.globals.x0, 83.0);
        assert_eq!(w.globals.xE8, 1.2);
        assert_eq!(w.globals.x3C, 0.15);
        w.camera_80031074(3);
        w.camera_800310a0(2);
        assert_eq!((w.camera_80031060(), w.camera_8003108c()), (3, 2));
        w.game_camera.bf_398 = [0xFF; 3];
        w.camera_800310e8();
        assert_eq!(w.game_camera.x398_b0() | w.game_camera.x398_b5(), 0);
        assert_eq!(w.game_camera.x398_b6_b7(), 3);
        w.camera_set_background_color(1, 2, 3);
        let c = w.camera_get_background_color();
        assert_eq!((c.r, c.g, c.b, c.a), (1, 2, 3, 0));
        assert_eq!(w.camera_80030e10(), 10000.0);
        w.game_camera.x2B8 = 1;
        w.game_camera.x2B0 = 7.0;
        assert_eq!(w.camera_80030e10(), 7.0);
        w.camera_800311cc(5.0);
        w.camera_800311dc(0.5);
        assert_eq!(w.camera_stage_slot_clip_get(), (0.5, 5.0));
        w.game_camera.mode = CAMERA_FREE;
        assert!(w.camera_8003010c() && !w.camera_80030178());
        w.debug_last_mode = CAMERA_DEBUG_FREE;
        w.camera_800300f0();
        assert!(w.camera_80030178());
        w.camera_80030de4(1.0, 2.0);
        w.camera_80030df8();
        assert_eq!(w.game_camera.translation.x, 0.0);
    }
}
