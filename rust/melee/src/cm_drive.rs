//! The match camera (`cm/camera.c`), driven from the native golden match.
//!
//! The ported camera math lives in `melee_world::cm::camera::CameraWorld` (subject framing,
//! follow smoothing, zoom, quake). This module is the glue the C gets from the engine around it:
//!   - `Camera_Init` / `Camera_Create` defaults (`cm_803BCB64`'s eye, interest, fov);
//!   - the stage's camera info (`Ground_801C0800` from `grGroundParam`, `Ground_801C39C0` from the
//!     camera range points 0x94-0x96);
//!   - each fighter's `CmSubject` (`ftCamera_80076064` / `ftCamera_UpdateCameraBox`);
//!   - `Camera_8002B3D4` (standard mode, ported here: its inlines and `Camera_8002A768`);
//!   - `Camera_8002F3AC` (the match-start snap) and `Camera_8002AF68` (the CObj pose).
//!
//! APPROXIMATIONS: every motion state's camera callback is `ftCamera_UpdateCameraBox` (the table
//! differs only for the Dead*, Rebirth, Cliff, ThrowHi/ThrownHi and WarpStar states); no camera
//! quake is requested (the sim does not run `efAsync` camera shakes); VS mode only (no 1P
//! sub-stick zoom); no stage-specific eye floor (`Camera_8002AF68` only clamps on Castle,
//! Corneria, Zebes, Garden, Kinoko route and Home-run).
use crate::vs_match::CmSubjectIn;
use melee_runtime::lbtrigf::atan2f;
use melee_runtime::lbvector as lbv;
use melee_runtime::mtx::Vec3;
use melee_types::{
    CameraBounds, CameraTransformState, CmSubject, CmSubjectState_Active, Vec as TVec, CAMERA_STANDARD,
};
use melee_world::cm::camera::{world, CamStage, CameraWorld};

const F32_MAX: f32 = 3.402_823_466e38;
const DEG2RAD_MTX: f32 = 0.017_453_292_52;
/// `cm_803BCB64` (`HSD_CameraDescPerspective`): eye, interest, fov, near, far, aspect.
const DESC_EYE: [f32; 3] = [0.0, 40.241425, 300.241];
const DESC_INTEREST: [f32; 3] = [0.0, 10.0, 0.0];
const DESC_FOV: f32 = 30.0;
pub const CAMERA_ASPECT: f32 = 1.217_333_3;

fn be32(d: &[u8], at: u32) -> u32 {
    d.get(at as usize..at as usize + 4).map_or(0, |b| u32::from_be_bytes(b.try_into().unwrap()))
}
fn bef(d: &[u8], at: u32) -> f32 {
    f32::from_bits(be32(d, at))
}
fn v3(v: &TVec) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}
fn tv(v: &Vec3) -> TVec {
    TVec { x: v.x, y: v.y, z: v.z }
}

/// The CObj pose `Camera_8002AF68` leaves (what the draw uses).
#[derive(Clone, Copy, Debug)]
pub struct CamPose {
    pub eye: [f32; 3],
    pub interest: [f32; 3],
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

pub struct CmDrive {
    fixed_zoom: f32,
    smooth: f32,
}

/// `Ground_801C0800` + `Ground_801C39C0`: the stage's camera info from its file. `stage` is the
/// DAT data section, `points` the stage's points (already scaled by `grGroundParam->y`).
pub fn stage_cam(stage: &[u8], ground_param: u32, points: &[crate::stage::SpawnPoint]) -> Result<(CamStage, f32, f32), String> {
    let g = ground_param;
    let s16 = |at: u32| i16::from_be_bytes(stage[at as usize..at as usize + 2].try_into().unwrap()) as f32;
    let s32 = |at: u32| be32(stage, at) as i32 as f32;
    let (tilt, zoom, depth, pan) = (s16(g + 8), s32(g + 0xC), s32(g + 0x10), s32(g + 0x14));
    let (x18, x1c, ratio, fixed_zoom, smooth) = (bef(stage, g + 0x18), bef(stage, g + 0x1C), bef(stage, g + 0x20), bef(stage, g + 0x24), bef(stage, g + 0x28));
    let _ = tilt;
    let pt = |id: i32| points.iter().find(|p| p.id == id).map(|p| p.pos);
    let (Some(a), Some(b), Some(c)) = (pt(0x95), pt(0x96), pt(0x94)) else {
        return Err("stage: no camera range points (Ground_801C39C0 dummy range not wired)".into());
    };
    // Ground_801C39C0, in its operation order (offsets relative to point 0x94, then re-added).
    let (l, r) = if a[0] < b[0] { (a[0] - c[0], b[0] - c[0]) } else { (b[0] - c[0], a[0] - c[0]) };
    let (bot, top) = if a[1] < b[1] { (a[1] - c[1], b[1] - c[1]) } else { (b[1] - c[1], a[1] - c[1]) };
    let (xo, yo) = (c[0], c[1]);
    // Stage_GetCamBounds*Offset = cam_bounds.* + cam_{x,y}_offset; DEG2RAD of stage.c.
    const DEG2RAD: f32 = 0.017_453_292_384_7_f32;
    Ok((
        CamStage {
            left: l + xo,
            right: r + xo,
            top: top + yo,
            bottom: bot + yo,
            track_ratio: ratio,
            zoom_rate: zoom,
            max_depth: depth,
            info_x20: x1c,
            info_x24: x18,
            pan_angle_rad: DEG2RAD * pan,
            tcam_offset: [xo, yo, 0.0],
            // stage_info.x724 = -10000 (Ground_801C0378 reset), read by Ground_801C4368.
            ground_801c4368: -10000.0,
        },
        fixed_zoom,
        smooth,
    ))
}

/// `Camera_8002A768(transform, 0)`: pull the target pose back inside the camera bounds when the
/// view frustum's corners (on the z = 0 plane) leave them.
fn camera_8002a768(w: &CameraWorld, t: &mut CameraTransformState) {
    let mut out_mask = 0u32; // var_r30: corners that miss the plane
    let mut hit = 0u32; // var_r31
    let half_fov = 0.5 * (t.target_fov * DEG2RAD_MTX);
    let tp = v3(&t.target_position);
    let mut dist = lbv::diff(&v3(&t.target_interest), &tp);
    lbv::normalize(&mut dist);
    let pitch = atan2f(dist.y, -dist.z);
    let yaw = atan2f(-dist.x, -dist.z);
    let fwd = Vec3 { x: 0.0, y: 0.0, z: -1.0 };
    let aspect = w.desc.aspect;
    let mut corner = |sx: f32, sy: f32, bit: u32| -> Vec3 {
        let mut c = fwd;
        lbv::rotate(&mut c, 1, sx * half_fov);
        lbv::rotate(&mut c, 2, sy * half_fov);
        c.x *= aspect;
        lbv::normalize(&mut c);
        lbv::rotate(&mut c, 1, pitch);
        lbv::rotate(&mut c, 2, yaw);
        if c.z < -0.001 {
            let k = -tp.z / c.z;
            c.x *= k;
            c.y *= k;
            c.z *= k;
            lbv::add(&mut c, &tp);
        } else {
            out_mask |= bit;
        }
        c
    };
    let top_left = corner(1.0, 1.0, 1);
    let top_right = corner(1.0, -1.0, 2);
    let bottom_right = corner(-1.0, 1.0, 4);
    let bottom_left = corner(-1.0, -1.0, 8);
    let (bl, br, bt, bb) = (w.bounds_left(), w.bounds_right(), w.bounds_top(), w.bounds_bottom());
    let (mut left_o, mut bottom_o, mut right_o, mut top_o) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    if out_mask & 1 == 0 {
        if top_left.x < bl {
            let d = bl - top_left.x;
            if d > left_o {
                left_o = d;
                hit |= 4;
            }
        }
        if top_left.y > bt {
            let d = bt - top_left.y;
            if d < top_o {
                top_o = d;
                hit |= 1;
            }
        }
    }
    if out_mask & 2 == 0 {
        if top_right.x > br {
            let d = br - top_right.x;
            if d < right_o {
                right_o = d;
                hit |= 8;
            }
        }
        if top_right.y > bt {
            let d = bt - top_right.y;
            if d < top_o {
                top_o = d;
                hit |= 1;
            }
        }
    }
    if out_mask & 4 == 0 {
        if bottom_right.x < bl {
            let d = bl - bottom_right.x;
            if d > left_o {
                left_o = d;
                hit |= 4;
            }
        }
        if bottom_right.y < bb {
            let d = bb - bottom_right.y;
            if d > bottom_o {
                bottom_o = d;
                hit |= 2;
            }
        }
    }
    if out_mask & 8 == 0 {
        if bottom_left.x > br {
            let d = br - bottom_left.x;
            if d < right_o {
                right_o = d;
                hit |= 8;
            }
        }
        if bottom_left.y < bb {
            let d = bb - bottom_left.y;
            if d > bottom_o {
                bottom_o = d;
                hit |= 2;
            }
        }
    }
    if hit != 0 {
        let mut c = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        if hit & 4 != 0 && hit & 8 != 0 {
            c.x = 0.5 * (left_o + right_o);
        } else if hit & 4 != 0 {
            c.x = left_o;
        } else if hit & 8 != 0 {
            c.x = right_o;
        }
        if hit & 1 != 0 && hit & 2 != 0 {
            c.y = 0.5 * (bottom_o + top_o);
        } else if hit & 1 != 0 {
            c.y = top_o;
        } else if hit & 2 != 0 {
            c.y = bottom_o;
        }
        let mut p = v3(&t.target_position);
        lbv::add(&mut p, &c);
        t.target_position = tv(&p);
        let mut i = v3(&t.target_interest);
        lbv::add(&mut i, &c);
        t.target_interest = tv(&i);
    }
}

impl CmDrive {
    /// `Camera_Init` + `Camera_Create` + the stage's camera info, then one `Camera_8002F3AC`.
    pub fn new(stage: CamStage, fixed_zoom: f32, smooth: f32, subjects: &[CmSubjectIn]) -> Self {
        let w = world();
        *w = CameraWorld::default();
        w.stage = stage;
        let gc = &mut w.game_camera;
        let (eye, int) = (TVec { x: DESC_EYE[0], y: DESC_EYE[1], z: DESC_EYE[2] }, TVec { x: DESC_INTEREST[0], y: DESC_INTEREST[1], z: DESC_INTEREST[2] });
        for t in [&mut gc.transform, &mut gc.transform_copy] {
            t.interest = int;
            t.target_interest = int;
            t.position = eye;
            t.target_position = eye;
            t.target_fov = DESC_FOV;
            t.fov = DESC_FOV;
        }
        gc.nearz = 0.1;
        gc.farz = 16384.0;
        gc.mode = CAMERA_STANDARD;
        gc.quake_scale = 1.0;
        gc.x2BC = 1.0;
        gc.x2C0 = -1.0;
        let mut d = CmDrive { fixed_zoom, smooth };
        // Player creation order p1, p2: the list `cm_804D6468 -> prev` walks last-created first.
        w.subjects = subjects.iter().map(|_| CmSubject::zeroed()).collect();
        for (i, s) in subjects.iter().enumerate() {
            d.snap_subject(i, s);
        }
        w.subjects.reverse();
        // Camera_8002F3AC: one standard-mode update, then current := target.
        d.standard_update();
        let gc = &mut world().game_camera;
        gc.transform.position = gc.transform.target_position;
        gc.transform.interest = gc.transform.target_interest;
        gc.transform.fov = gc.transform.target_fov;
        gc.transform_copy.position = gc.transform_copy.target_position;
        gc.transform_copy.interest = gc.transform_copy.target_interest;
        gc.transform_copy.fov = gc.transform_copy.target_fov;
        d
    }

    fn set_h(&self, cb: &mut CmSubject, s: &[f32; 6], facing: f32) {
        if facing == 1.0 {
            cb.target_ext.h.x = s[2];
            cb.target_ext.h.y = s[1] * self.fixed_zoom;
            cb.facing_dir = 1.0;
        } else {
            cb.target_ext.h.x = -s[1] * self.fixed_zoom;
            cb.target_ext.h.y = -s[2];
            cb.facing_dir = -1.0;
        }
    }

    /// `ftCamera_80076018`.
    fn scaled(s: &CmSubjectIn) -> [f32; 6] {
        core::array::from_fn(|i| s.x3c[i] * s.scale_y)
    }

    /// `ftCamera_80076064`: initialise the camera box (index `i` in creation order).
    fn snap_subject(&mut self, i: usize, s: &CmSubjectIn) {
        let sp = Self::scaled(s);
        let mut cb = CmSubject::zeroed();
        cb.state = CmSubjectState_Active;
        // Camera_80028F5C's reset values.
        cb.ext.h.x = -1.0;
        cb.ext.h.y = 1.0;
        cb.ext.v.x = 1.0;
        cb.ext.v.y = -1.0;
        cb.ext.v.z = 1.0;
        self.set_h(&mut cb, &sp, s.facing);
        cb.target_ext.v = TVec { x: sp[3], y: sp[4], z: sp[5] };
        cb.ext.h = cb.target_ext.h;
        cb.ext.v = cb.target_ext.v;
        cb.pos = TVec { x: s.pos[0], y: s.pos[1] + sp[0], z: s.pos[2] };
        cb.bone_pos = cb.pos;
        world().subjects[i] = cb;
    }

    /// `ftCamera_UpdateCameraBox` for every fighter (`ins` in player order).
    fn update_subjects(&mut self, ins: &[CmSubjectIn]) {
        let n = ins.len();
        for (i, s) in ins.iter().enumerate() {
            let sp = Self::scaled(s);
            let mut cb = world().subjects[n - 1 - i];
            self.set_h(&mut cb, &sp, s.facing);
            cb.pos = TVec { x: s.pos[0], y: s.pos[1] + sp[0], z: s.pos[2] };
            cb.set_on_ledge(0);
            cb.bone_pos = TVec { x: s.bone[0], y: s.bone[1], z: s.bone[2] };
            world().subjects[n - 1 - i] = cb;
        }
    }

    /// `Camera_8002B3D4` (standard mode).
    fn standard_update(&mut self) {
        let w = world();
        // Camera_80030DF8
        w.game_camera.translation.x = 0.0;
        w.game_camera.translation.y = 0.0;
        w.camera_800293e0();
        // Camera_8002B0E0 is the 1P-mode sub-stick zoom: not in VS.
        let mut bounds = CameraBounds::zeroed();
        let mut bounds_copy = CameraBounds::zeroed();
        let mut t = w.game_camera.transform;
        Self::update_transform(w, &mut bounds, &mut t);
        w.game_camera.transform = t;
        let mut tc = w.game_camera.transform_copy;
        Self::update_transform(w, &mut bounds_copy, &mut tc);
        w.game_camera.transform_copy = tc;
        // update_zoom_distance
        if w.game_camera.x2BC == 1.0 {
            let t = &w.game_camera.transform;
            let dx = t.target_position.x - t.target_interest.x;
            let dy = t.target_position.y - t.target_interest.y;
            let dz = t.target_position.z - t.target_interest.z;
            w.game_camera.x2C0 = melee_runtime::gekko::sqrtf((dx * dx + dy * dy) + dz * dz);
        }
        // update_bounds (Camera_8002B1F8 returns at x2BC == 1.0)
        let smooth = self.smooth;
        let mut t = w.game_camera.transform;
        w.camera_80029aac(&bounds, &mut t, smooth);
        w.camera_80029c88(&mut t, smooth);
        w.game_camera.transform = t;
        let mut tc = w.game_camera.transform_copy;
        w.camera_80029aac(&bounds_copy, &mut tc, smooth);
        w.camera_80029c88(&mut tc, smooth);
        w.game_camera.transform_copy = tc;
        w.camera_update_quakes();
        let t = w.game_camera.transform;
        w.camera_apply_quake(&bounds, &t, false);
        // update_avg_bounds_width
        let gc = &mut w.game_camera;
        if gc.x2B8 > 0x3E8 {
            gc.x2B4 = gc.x2B0;
            gc.x2B8 = 1;
        }
        let left = w.stage.left;
        let gc = &mut w.game_camera;
        gc.x2B4 += w.stage.right - left;
        gc.x2B8 += 1;
        gc.x2B0 = gc.x2B4 / gc.x2B8 as f32;
    }

    fn update_transform(w: &mut CameraWorld, bounds: &mut CameraBounds, ts: &mut CameraTransformState) {
        w.camera_8002958c(bounds, ts);
        ts.target_fov = w.globals.x40;
        let delta = ts.target_fov - ts.fov;
        ts.fov += delta * w.globals.x44;
        w.camera_80029bc4(bounds, ts);
        // fighter_z_out_of_range: Camera_80030AF8 is false in VS.
        w.camera_80029cf8(bounds, ts);
        camera_8002a768(w, ts);
    }

    /// One frame: the fighters' camera boxes, then the camera proc.
    pub fn step(&mut self, ins: &[CmSubjectIn]) {
        self.update_subjects(ins);
        self.standard_update();
    }

    /// `Camera_8002AF68` on the main CObj.
    pub fn pose(&self) -> CamPose {
        let gc = &world().game_camera;
        let t = &gc.transform;
        CamPose {
            eye: [t.position.x + gc.translation.x, t.position.y + gc.translation.y, t.position.z],
            interest: [t.interest.x + gc.translation.x, t.interest.y + gc.translation.y, t.interest.z],
            fov: t.fov,
            near: gc.nearz,
            far: gc.farz,
            aspect: CAMERA_ASPECT,
        }
    }
}
