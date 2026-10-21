//! src/melee/gr/ground.c, round 7: the stage_info / GroundParam accessors, the camera and
//! blast-zone setters, the per-Ground SFX slot helpers, and the two pure orientation helpers
//! (Ground_801C4B50's math, Ground_801C5AEC). TARGET_PC bodies.
//!
//! State is `StageWorld::stage_info` (the C global `stage_info`); `stage_info.param` stays the
//! raw pointer the C has (null-checked where the C checks, asserted non-null where it does
//! not). Everything the C calls in another group goes through `GroundHooks`, whose default
//! bodies panic with the owning group.
#![allow(non_upper_case_globals)]
use super::stage::StageWorld;
use crate::vec::Vec3;
use melee_runtime::gekko::{cosf, sinf};
use melee_runtime::lbtrigf::asinf;
use melee_runtime::lbvector::{crossprod_normalized, euler_angles_from_onb, normalize};
use melee_types::*;

const M_PI: f64 = 3.14159265358979323846;

/// Callees outside ground.c. Defaults are stubs.
#[allow(unused_variables)]
pub trait GroundHooks {
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_800237a8(&mut self, sfx: i32, vol: i32, pan: i32) -> i32 {
        unimplemented!("TODO(group1): lbAudioAx_800237A8")
    }
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_80023870(&mut self, a: i32, vol: i32, pan: i32, d: i32) -> i32 {
        unimplemented!("TODO(group1): lbAudioAx_80023870")
    }
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_800236b8(&mut self, id: i32) -> i32 {
        unimplemented!("TODO(group1): lbAudioAx_800236B8")
    }
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_80023710(&mut self, id: i32) -> bool {
        unimplemented!("TODO(group1): lbAudioAx_80023710")
    }
    /// TODO(group1: lb/lbaudio_ax.c) volume
    fn lbaudioax_80024b58(&mut self, id: i32, v: i32) {
        unimplemented!("TODO(group1): lbAudioAx_80024B58")
    }
    /// TODO(group1: lb/lbaudio_ax.c) pan
    fn lbaudioax_80024b1c(&mut self, id: i32, v: i32) {
        unimplemented!("TODO(group1): lbAudioAx_80024B1C")
    }
    /// TODO(group1: baselib/random.c)
    fn hsd_randi(&mut self, n: i32) -> i32 {
        unimplemented!("TODO(group1): HSD_Randi")
    }
    /// TODO(group1: baselib/spline.c)
    fn spl_get_spline_point(&mut self, spline: *mut HSD_Spline, t: f32) -> Vec3 {
        unimplemented!("TODO(group1): splGetSplinePoint")
    }
    /// TODO(group1: lb/lbshadow.c)
    fn lbshadow_8000e9f0(&mut self, spline: *mut HSD_Spline, t: f32) -> Vec3 {
        unimplemented!("TODO(group1): lbShadow_8000E9F0")
    }
    /// TODO(group2: gr/grzebes.c)
    fn grzebes_801da3f4(&mut self, arg: *mut core::ffi::c_void) {
        unimplemented!("TODO(group2): grZebes_801DA3F4")
    }
    /// TODO(group2: gr/grkongo.c)
    fn grkongo_801d8270(&mut self, g: *mut HSD_GObj) {
        unimplemented!("TODO(group2): grKongo_801D8270")
    }
    /// TODO(group2: gr/grkongo.c)
    fn grkongo_801d828c(&mut self, g: *mut HSD_GObj) {
        unimplemented!("TODO(group2): grKongo_801D828C")
    }
    /// TODO(group2: gr/groldkongo.c)
    fn groldkongo_802105ac(&mut self, g: *mut HSD_GObj) {
        unimplemented!("TODO(group2): grOldKongo_802105AC")
    }
    /// TODO(group2: gr/groldkongo.c)
    fn groldkongo_802105c8(&mut self, g: *mut HSD_GObj) {
        unimplemented!("TODO(group2): grOldKongo_802105C8")
    }
    /// TODO(group2: cm/camera.c, needs HSD_CObj)
    fn camera_set_mode_to_fixed(&mut self) {
        unimplemented!("TODO(group2): Camera_SetModeToFixed")
    }
    /// TODO(group2: cm/camera.c, needs HSD_CObj)
    fn camera_set_mode_to_standard(&mut self) {
        unimplemented!("TODO(group2): Camera_SetModeToStandard")
    }
    /// TODO(group3: pl/player.c)
    fn player_get_entity(&mut self, slot: i32) -> *mut HSD_GObj {
        unimplemented!("TODO(group3): Player_GetEntity")
    }
    /// TODO(group3: pl/player.c)
    fn player_get_entity_at_index(&mut self, slot: i32, i: i32) -> *mut HSD_GObj {
        unimplemented!("TODO(group3): Player_GetEntityAtIndex")
    }
    /// TODO(group5: ty/toy.c)
    fn toy_803124bc(&mut self) {
        unimplemented!("TODO(group5): Toy_803124BC")
    }
    /// TODO(group5: ty/toy.c)
    fn toy_8031234c(&mut self, a: i32) {
        unimplemented!("TODO(group5): Toy_8031234C")
    }
    /// TODO(group5: ty/toy.c)
    fn toy_80305918(&mut self, a: i32, b: i32, c: i32) {
        unimplemented!("TODO(group5): Toy_80305918")
    }
}

/// The nine GroundParam colours, named by their GC offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamColor {
    XB8, XBC, XC0, XC4, XC8, XCC, XD0, XD4, XD8,
}

fn color_mut(p: &mut GroundParam, c: ParamColor) -> &mut _GXColor {
    match c {
        ParamColor::XB8 => &mut p.xB8,
        ParamColor::XBC => &mut p.xBC,
        ParamColor::XC0 => &mut p.xC0,
        ParamColor::XC4 => &mut p.xC4,
        ParamColor::XC8 => &mut p.xC8,
        ParamColor::XCC => &mut p.xCC,
        ParamColor::XD0 => &mut p.xD0,
        ParamColor::XD4 => &mut p.xD4,
        ParamColor::XD8 => &mut p.xD8,
    }
}

impl StageWorld {
    fn param(&self) -> Option<&GroundParam> {
        // SAFETY: stage_info.param is either null or points at the loaded GroundParam (as in C).
        unsafe { self.stage_info.param.as_ref() }
    }
    fn param_mut(&mut self) -> &mut GroundParam {
        // SAFETY: as above; the C dereferences unconditionally here.
        unsafe { self.stage_info.param.as_mut() }.expect("stage_info.param is NULL")
    }

    /// Ground_801C0498
    pub fn ground_801c0498(&self) -> f32 {
        self.param().map_or(1.0, |p| p.y)
    }
    /// Ground_SetParamY (HSD_ASSERT(521, 0) when param is NULL)
    pub fn ground_set_param_y(&mut self, y: f32) {
        assert!(!self.stage_info.param.is_null(), "ground.c:521");
        self.param_mut().y = y;
    }
    /// Ground_801C0508
    pub fn ground_801c0508(&self) -> i32 {
        self.param().map_or(128, |p| p.x4 as i32)
    }
    /// The nine colour setters: Ground_801C052C (xB8), 0544 (xC0), 055C (xD0), 0574 (xD8),
    /// 058C (xC4), 05A4 (xCC), 05BC (xD4), 05D4 (xBC), 05EC (xC8).
    pub fn ground_set_param_color(&mut self, which: ParamColor, c: &_GXColor) {
        *color_mut(self.param_mut(), which) = *c;
    }
    /// The nine colour getters: Ground_801C0604 (xB8), 0618 (xC0), 062C (xD0), 0640 (xD8),
    /// 0654 (xC4), 0668 (xCC), 067C (xD4), 0690 (xBC), 06A4 (xC8).
    pub fn ground_param_color(&mut self, which: ParamColor) -> &mut _GXColor {
        color_mut(self.param_mut(), which)
    }
    pub fn ground_801c052c(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XB8, c) }
    pub fn ground_801c0544(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XC0, c) }
    pub fn ground_801c055c(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XD0, c) }
    pub fn ground_801c0574(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XD8, c) }
    pub fn ground_801c058c(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XC4, c) }
    pub fn ground_801c05a4(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XCC, c) }
    pub fn ground_801c05bc(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XD4, c) }
    pub fn ground_801c05d4(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XBC, c) }
    pub fn ground_801c05ec(&mut self, c: &_GXColor) { self.ground_set_param_color(ParamColor::XC8, c) }
    pub fn ground_801c0604(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XB8) }
    pub fn ground_801c0618(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XC0) }
    pub fn ground_801c062c(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XD0) }
    pub fn ground_801c0640(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XD8) }
    pub fn ground_801c0654(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XC4) }
    pub fn ground_801c0668(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XCC) }
    pub fn ground_801c067c(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XD4) }
    pub fn ground_801c0690(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XBC) }
    pub fn ground_801c06a4(&mut self) -> &mut _GXColor { self.ground_param_color(ParamColor::XC8) }

    /// Ground_801C1D6C
    pub fn ground_801c1d6c(&mut self, f: u32) { self.stage_info.flags |= f; }
    /// Ground_801C1D84
    pub fn ground_801c1d84(&self) -> u32 { self.stage_info.flags & 0x330 }
    /// Ground_801C1D98
    pub fn ground_801c1d98(&self) -> u32 { self.stage_info.flags & 0x20 }
    /// Ground_801C1DAC
    pub fn ground_801c1dac(&self) -> u32 { self.stage_info.flags & 0x100 }
    /// Ground_801C1DC0
    pub fn ground_801c1dc0(&self) -> u32 { self.stage_info.flags & 0x80 }
    /// Ground_801C1DD4
    pub fn ground_801c1dd4(&self) -> i32 { self.stage_info.x6D0 as i32 }
    /// Ground_801C1DE4: (*arg0, *arg1) = (x6D4, x6D2)
    pub fn ground_801c1de4(&self) -> (i32, i32) {
        (self.stage_info.x6D4 as i32, self.stage_info.x6D2 as i32)
    }
    /// Ground_801C1E00 (1-bit field: low bit kept)
    pub fn ground_801c1e00(&mut self, v: i32) { self.stage_info.unk8C.set_b2(v as u8); }
    /// Ground_801C1E18
    pub fn ground_801c1e18(&self) -> i32 { self.stage_info.unk8C.b2() as i32 }
    /// Ground_801C1E84
    pub fn ground_801c1e84(&self) -> *mut HSD_GObj { self.stage_info.x12C }

    /// Ground_801C3880
    pub fn ground_801c3880(&mut self, v: f32) { self.stage_info.cam_info.cam_bounds.top = v; }
    /// Ground_801C3890
    pub fn ground_801c3890(&mut self, v: f32) { self.stage_info.cam_info.cam_bounds.bottom = v; }
    /// Ground_801C38A0
    pub fn ground_801c38a0(&mut self, v: f32) { self.stage_info.cam_info.cam_bounds.left = v; }
    /// Ground_801C38AC
    pub fn ground_801c38ac(&mut self, v: f32) { self.stage_info.cam_info.cam_bounds.right = v; }
    /// Ground_801C38BC
    pub fn ground_801c38bc(&mut self, x: f32, y: f32) {
        let c = &mut self.stage_info.cam_info;
        c.cam_x_offset = x;
        c.cam_y_offset = y;
    }
    /// Ground_801C38D0
    pub fn ground_801c38d0(&mut self, tilt: f32, pan: f32, a: f32, b: f32) {
        let c = &mut self.stage_info.cam_info;
        c.cam_vertical_tilt = tilt;
        c.cam_pan_degrees = pan;
        c.x20 = a;
        c.x24 = b;
    }
    /// Ground_801C38EC
    pub fn ground_801c38ec(&mut self, depth: f32, zoom: f32) {
        let c = &mut self.stage_info.cam_info;
        c.cam_max_depth = depth;
        c.cam_zoom_rate = zoom;
    }
    /// Ground_801C3900
    #[allow(clippy::too_many_arguments)]
    pub fn ground_801c3900(&mut self, a8: f32, a9: f32, aa: f32, ab: f32, up: f32, down: f32, left: f32, right: f32) {
        let c = &mut self.stage_info.cam_info;
        c.x3C = a8;
        c.pausecam_zpos_min = a9;
        c.pausecam_zpos_init = aa;
        c.pausecam_zpos_max = ab;
        c.cam_angle_up = up;
        c.cam_angle_down = down;
        c.cam_angle_left = left;
        c.cam_angle_right = right;
    }
    /// Ground_801C392C
    pub fn ground_801c392c(&mut self, x: f32, y: f32, z: f32, fov: f32, vert: f32, horz: f32) {
        let c = &mut self.stage_info.cam_info;
        c.fixed_cam_pos.x = x;
        c.fixed_cam_pos.y = y;
        c.fixed_cam_pos.z = z;
        c.fixed_cam_fov = fov;
        c.fixed_cam_vert_angle = vert;
        c.fixed_cam_horz_angle = horz;
    }
    /// Ground_801C3950
    pub fn ground_801c3950(&mut self, v: f32) { self.stage_info.cam_info.cam_fixed_zoom = v; }
    /// Ground_801C3960
    pub fn ground_801c3960(&mut self, v: f32) { self.stage_info.cam_info.cam_track_ratio = v; }
    /// Ground_801C3970
    pub fn ground_801c3970(&mut self, v: f32) { self.stage_info.cam_info.cam_track_smooth = v; }
    /// Ground_801C3980
    pub fn ground_801c3980(&mut self, v: f32) { self.stage_info.blast_zone.top = v; }
    /// Ground_801C3990
    pub fn ground_801c3990(&mut self, v: f32) { self.stage_info.blast_zone.bottom = v; }
    /// Ground_801C39A0
    pub fn ground_801c39a0(&mut self, v: f32) { self.stage_info.blast_zone.left = v; }
    /// Ground_801C39B0
    pub fn ground_801c39b0(&mut self, v: f32) { self.stage_info.blast_zone.right = v; }

    /// Ground_801C3D44
    pub fn ground_801c3d44(&mut self, cb: Option<unsafe extern "C" fn(*mut Vec, i32) -> i32>, a8: f32, a9: f32) -> i32 {
        let s = &mut *self.stage_info;
        s.unk8C.set_b6(1);
        s.x70C = 0.5f32 * a8;
        s.x710 = 0.5f32 * a9;
        s.x90 = cb;
        if s.x714 != -1 {
            s.flags |= 0x10;
            s.x6D0 = s.x714 as i16;
        } else {
            s.flags &= 0xFFFF_FFEF;
        }
        s.x714
    }
    /// Ground_801C3DB4
    pub fn ground_801c3db4(&mut self, cb: Option<unsafe extern "C" fn(*mut Vec, i32) -> i32>, a8: f32, a9: f32) -> i32 {
        let s = &mut *self.stage_info;
        s.unk8C.set_b7(1);
        s.x718 = 0.5f32 * a8;
        s.x71C = 0.5f32 * a9;
        s.x94 = cb;
        if s.x720 != -1 {
            s.flags |= 0x40;
        } else {
            s.flags &= 0xFFFF_FFBF;
        }
        s.x720
    }
    /// Ground_801C4338
    pub fn ground_801c4338(&mut self) {
        let s = &mut *self.stage_info;
        s.x6D4 = s.x6D4.wrapping_sub(1);
        if s.x6D4 == 0 {
            s.flags |= 0x20;
        }
    }
    /// Ground_801C4368: (*slope, *intercept)
    pub fn ground_801c4368(&self) -> (f32, f32) {
        (self.stage_info.x724, self.stage_info.x724 - self.stage_info.x728)
    }
    /// Ground_801C438C
    pub fn ground_801c438c(&mut self, v: f32) {
        self.stage_info.x728 = self.stage_info.x724;
        self.stage_info.x724 = v;
    }
    /// Ground_801C43A4
    pub fn ground_801c43a4(&self, h: &mut dyn GroundHooks, arg: *mut core::ffi::c_void) {
        h.grzebes_801da3f4(arg);
    }
    /// Ground_801C4D70
    pub fn ground_801c4d70(&mut self, g: *mut HSD_GObj, v: &Vec3, f: f32) -> bool {
        self.stage_info.x72C = g;
        self.stage_info.x730 = Vec { x: v.x, y: v.y, z: v.z };
        self.stage_info.x73C = f;
        true
    }
    /// Ground_801C4DA0
    pub fn ground_801c4da0(&self) -> (Vec3, f32) {
        let v = self.stage_info.x730;
        (Vec3 { x: v.x, y: v.y, z: v.z }, self.stage_info.x73C)
    }
    /// Ground_801C4DD0
    pub fn ground_801c4dd0(&self, h: &mut dyn GroundHooks) -> bool {
        match self.stage_info.grkind {
            Gr_Kind_Kongo => h.grkongo_801d8270(self.stage_info.x72C),
            Gr_Kind_OldKongo => h.groldkongo_802105ac(self.stage_info.x72C),
            _ => {}
        }
        true
    }
    /// Ground_801C4E20
    pub fn ground_801c4e20(&self, h: &mut dyn GroundHooks) -> bool {
        match self.stage_info.grkind {
            Gr_Kind_Kongo => h.grkongo_801d828c(self.stage_info.x72C),
            Gr_Kind_OldKongo => h.groldkongo_802105c8(self.stage_info.x72C),
            _ => {}
        }
        true
    }
    /// Ground_801C5700
    pub fn ground_801c5700(&self, i: i32) -> *mut DynamicsDesc {
        match self.stage_info.on_touch_line {
            // SAFETY: the stage's own callback, as the C calls it.
            Some(f) => unsafe { f(i) },
            None => core::ptr::null_mut(),
        }
    }
    /// Ground_801C5740
    pub fn ground_801c5740(&mut self, v: i32) { self.stage_info.x6D8 = v; }
    /// Ground_801C5750
    pub fn ground_801c5750(&mut self) { self.stage_info.x6D8 = 0; }
    /// Ground_801C5764
    pub fn ground_801c5764(&self) -> i32 { self.stage_info.x6D8 }
    /// Ground_801C5774
    pub fn ground_801c5774(&self) -> i32 { self.stage_info.x6DC as i32 }
    /// Ground_801C5784
    pub fn ground_801c5784(&mut self, v: i32) { self.stage_info.x740 = v; }
    /// Ground_801C5794
    pub fn ground_801c5794(&self) -> i32 { self.stage_info.x740 }
    /// Ground_801C57F0 (arg unused)
    pub fn ground_801c57f0(&self, _arg: i32) -> f32 { self.stage_info.x6E0 }
    /// Ground_EnableMatchCamera
    pub fn ground_enable_match_camera(&self, h: &mut dyn GroundHooks) {
        // SAFETY: the C dereferences param unconditionally.
        let p = unsafe { self.stage_info.param.as_ref() }.expect("stage_info.param is NULL");
        if p.x4C_fixed_cam != 0 {
            h.camera_set_mode_to_fixed();
        } else {
            h.camera_set_mode_to_standard();
        }
    }
    /// Ground_801C5840
    pub fn ground_801c5840(&self, h: &mut dyn GroundHooks) -> i32 {
        let i = h.hsd_randi(1);
        self.stage_info.x6E4[i as usize]
    }
    /// Ground_801C5A84
    pub fn ground_801c5a84(&mut self, v: i32) { self.stage_info.x98 = v; }
    /// Ground_801C5A94
    pub fn ground_801c5a94(&self) -> i32 { self.stage_info.x98 }
    /// Ground_801C5AA4
    pub fn ground_801c5aa4(&mut self, b: bool) { self.stage_info.unk8C.set_b1(b as u8); }
    /// Ground_801C5ABC
    pub fn ground_801c5abc(&self) -> bool { self.stage_info.unk8C.b1() != 0 }
}

/// Ground_GetP1Fighter
pub fn ground_get_p1_fighter(h: &mut dyn GroundHooks) -> *mut HSD_GObj { h.player_get_entity(0) }
/// Ground_GetP1Fighter2
pub fn ground_get_p1_fighter2(h: &mut dyn GroundHooks) -> *mut HSD_GObj { h.player_get_entity_at_index(0, 1) }
/// Ground_801C53EC
pub fn ground_801c53ec(h: &mut dyn GroundHooks, sfx: u32) { h.lbaudioax_800237a8(sfx as i32, 0x7F, 0x40); }
/// Ground_801C5414
pub fn ground_801c5414(h: &mut dyn GroundHooks, a: i32, b: i32) { h.lbaudioax_80023870(a, 0x7F, 0x40, b); }
/// Ground_801C5A28
pub fn ground_801c5a28(h: &mut dyn GroundHooks) {
    h.toy_803124bc();
    h.toy_8031234c(0);
    h.toy_80305918(0, 0, 1);
}
/// Ground_801C5A60
pub fn ground_801c5a60(h: &mut dyn GroundHooks) { h.toy_8031234c(1); }

/// Ground_801C5440
pub fn ground_801c5440(h: &mut dyn GroundHooks, gp: Option<&mut Ground>, i: i32, sfx: u32) {
    if !(0..8).contains(&i) {
        return;
    }
    let Some(gp) = gp else { return };
    if sfx == 540000 {
        return;
    }
    if sfx != 540001 {
        if gp.x20[i as usize] != -1 {
            h.lbaudioax_800236b8(gp.x20[i as usize]);
        }
        gp.x20[i as usize] = h.lbaudioax_800237a8(sfx as i32, 0x7F, 0x40);
    } else {
        ground_801c5544(h, Some(gp), i);
    }
}
/// Ground_801C54DC
pub fn ground_801c54dc(h: &mut dyn GroundHooks, gp: Option<&Ground>, i: i32) -> bool {
    if !(0..8).contains(&i) {
        return false;
    }
    matches!(gp, Some(g) if g.x20[i as usize] != -1 && h.lbaudioax_80023710(g.x20[i as usize]))
}
/// Ground_801C5544
pub fn ground_801c5544(h: &mut dyn GroundHooks, gp: Option<&mut Ground>, i: i32) {
    if !(0..8).contains(&i) {
        return;
    }
    let Some(gp) = gp else { return };
    if gp.x20[i as usize] != -1 {
        h.lbaudioax_800236b8(gp.x20[i as usize]);
    }
    gp.x20[i as usize] = -1;
}
/// Ground_801C55AC (static)
pub fn ground_801c55ac(h: &mut dyn GroundHooks, gp: Option<&mut Ground>) {
    if let Some(gp) = gp {
        for i in 0..8 {
            ground_801c5544(h, Some(gp), i);
        }
    }
}
/// C float -> int conversion (fctiwz / cvttss2si); inputs the game passes are in range.
#[inline]
fn f2i(x: f32) -> i32 {
    x as i32
}
/// The volume Ground_801C5630 passes: `127 * val`.
pub fn ground_5630_volume(val: f32) -> i32 {
    f2i(127.0f32 * val)
}
/// The pan Ground_801C5694 passes: `63.5f * (val + 1)`.
pub fn ground_5694_pan(val: f32) -> i32 {
    f2i(63.5f32 * (val + 1.0f32))
}
/// Ground_801C5630
pub fn ground_801c5630(h: &mut dyn GroundHooks, gp: Option<&Ground>, i: i32, val: f32) {
    if !(0..8).contains(&i) {
        return;
    }
    if let Some(g) = gp {
        if g.x20[i as usize] != -1 {
            h.lbaudioax_80024b58(g.x20[i as usize], ground_5630_volume(val));
        }
    }
}
/// Ground_801C5694
pub fn ground_801c5694(h: &mut dyn GroundHooks, gp: Option<&Ground>, i: i32, val: f32) {
    if !(0..8).contains(&i) {
        return;
    }
    if let Some(g) = gp {
        if g.x20[i as usize] != -1 {
            h.lbaudioax_80024b1c(g.x20[i as usize], ground_5694_pan(val));
        }
    }
}

/// Ground_801C4B50's math after the two spline queries: tangent `vec0` (un-normalized, as
/// lbShadow_8000E9F0 returns it) -> euler angles.
pub fn ground_4b50_angles(mut vec0: Vec3) -> Vec3 {
    normalize(&mut vec0);
    let mut y0 = vec0.y;
    if vec0.y < 0.0 {
        y0 = -vec0.y;
    }
    let mut vec2 = if (y0 as f64) > 0.9f64 {
        Vec3 { x: 0.0, y: 0.0, z: 1.0 }
    } else {
        Vec3 { x: 0.0, y: 1.0, z: 0.0 }
    };
    let vec1 = crossprod_normalized(&vec0, &vec2);
    vec2 = crossprod_normalized(&vec1, &vec0);
    let result_y = asinf(vec1.z);
    let mut z1 = vec1.z;
    if vec1.z < 0.0 {
        z1 = -vec1.z;
    }
    let (result_x, result_z);
    if z1 >= 0.99999f32 {
        let mut phi = asinf(-vec0.y);
        if vec0.x * cosf(phi) * sinf(result_y) < 0.0 {
            phi = (M_PI - phi as f64) as f32;
        }
        result_x = phi;
        result_z = 0.0;
    } else {
        let mut rx = asinf(vec2.z / cosf(result_y));
        if vec0.z * cosf(rx) * cosf(result_y) < 0.0 {
            rx = (M_PI - rx as f64) as f32;
        }
        let mut rz = asinf(-vec1.y / cosf(result_y));
        if -vec1.x * cosf(result_y) * cosf(rz) < 0.0 {
            rz = (M_PI - rz as f64) as f32;
        }
        result_x = rx;
        result_z = rz;
    }
    Vec3 { x: result_x, y: result_y, z: result_z }
}

/// Ground_801C4B50: returns (*arg1 = spline point, *result = angles).
pub fn ground_801c4b50(h: &mut dyn GroundHooks, spline: *mut HSD_Spline, t: f32) -> (Vec3, Vec3) {
    let p = h.spl_get_spline_point(spline, t);
    let tangent = h.lbshadow_8000e9f0(spline, t);
    (p, ground_4b50_angles(tangent))
}

/// Ground_801C5AEC
pub fn ground_801c5aec(a: &Vec3, b: &Vec3, c: &Vec3) -> Vec3 {
    let mut v = euler_angles_from_onb(a, b, c);
    let ok = |x: f32| (if x < 0.0 { -x } else { x }) < 30000.0;
    if !ok(v.x) {
        v.x = 0.0;
    }
    if !ok(v.y) {
        v.y = 0.0;
    }
    if !ok(v.z) {
        v.z = 0.0;
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Rec {
        log: std::vec::Vec<String>,
        next_id: i32,
        playing: bool,
    }
    impl GroundHooks for Rec {
        fn lbaudioax_800237a8(&mut self, s: i32, v: i32, p: i32) -> i32 {
            self.log.push(format!("play {s} {v} {p}"));
            self.next_id += 1;
            self.next_id
        }
        fn lbaudioax_80023870(&mut self, a: i32, v: i32, p: i32, d: i32) -> i32 {
            self.log.push(format!("play4 {a} {v} {p} {d}"));
            0
        }
        fn lbaudioax_800236b8(&mut self, id: i32) -> i32 {
            self.log.push(format!("stop {id}"));
            0
        }
        fn lbaudioax_80023710(&mut self, _: i32) -> bool { self.playing }
        fn lbaudioax_80024b58(&mut self, id: i32, v: i32) { self.log.push(format!("vol {id} {v}")); }
        fn lbaudioax_80024b1c(&mut self, id: i32, v: i32) { self.log.push(format!("pan {id} {v}")); }
        fn hsd_randi(&mut self, _: i32) -> i32 { 1 }
        fn camera_set_mode_to_fixed(&mut self) { self.log.push("fixed".into()); }
        fn camera_set_mode_to_standard(&mut self) { self.log.push("std".into()); }
        fn grkongo_801d8270(&mut self, _: *mut HSD_GObj) { self.log.push("k70".into()); }
        fn groldkongo_802105c8(&mut self, _: *mut HSD_GObj) { self.log.push("ok c8".into()); }
        fn toy_803124bc(&mut self) { self.log.push("t bc".into()); }
        fn toy_8031234c(&mut self, a: i32) { self.log.push(format!("t 4c {a}")); }
        fn toy_80305918(&mut self, a: i32, b: i32, c: i32) { self.log.push(format!("t 18 {a}{b}{c}")); }
    }

    #[test]
    fn accessors() {
        let mut w = StageWorld::default();
        assert_eq!(w.ground_801c0498(), 1.0);
        assert_eq!(w.ground_801c0508(), 128);
        let mut p = Box::new(GroundParam::zeroed());
        w.stage_info.param = &mut *p;
        w.ground_set_param_y(3.5);
        assert_eq!(w.ground_801c0498(), 3.5);
        let c = _GXColor { r: 1, g: 2, b: 3, a: 4 };
        w.ground_801c05d4(&c);
        assert_eq!(w.ground_801c0690().b, 3);
        assert_eq!(p.xBC.a, 4);
        w.ground_801c1d6c(0x330 | 0x1);
        assert_eq!((w.ground_801c1d84(), w.ground_801c1d98(), w.ground_801c1dac(), w.ground_801c1dc0()), (0x330, 0x20, 0x100, 0));
        w.ground_801c1e00(3);
        assert_eq!(w.ground_801c1e18(), 1);
        w.ground_801c5aa4(true);
        assert!(w.ground_801c5abc());
        assert_eq!(w.stage_info.unk8C.bf_0[0], 0x60);
        w.stage_info.x714 = 7;
        assert_eq!(w.ground_801c3d44(None, 3.0, 5.0), 7);
        assert_eq!((w.stage_info.x70C, w.stage_info.x710, w.stage_info.x6D0, w.stage_info.flags & 0x10), (1.5, 2.5, 7, 0x10));
        w.stage_info.x720 = -1;
        w.stage_info.flags |= 0x40;
        assert_eq!(w.ground_801c3db4(None, 1.0, 1.0), -1);
        assert_eq!(w.stage_info.flags & 0x40, 0);
        w.stage_info.x6D4 = 1;
        w.stage_info.flags = 0;
        w.ground_801c4338();
        assert_eq!(w.ground_801c1d98(), 0x20);
        w.ground_801c438c(2.0);
        w.ground_801c438c(5.0);
        assert_eq!(w.ground_801c4368(), (5.0, 3.0));
        w.ground_801c4d70(core::ptr::null_mut(), &Vec3 { x: 1.0, y: 2.0, z: 3.0 }, 4.0);
        assert_eq!(w.ground_801c4da0().1, 4.0);
        w.ground_801c3900(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        w.ground_801c39a0(-9.0);
        assert_eq!(w.stage_info.blast_zone.left, -9.0);
        w.stage_info.x6E4 = [10, 20];
        let mut r = Rec::default();
        assert_eq!(w.ground_801c5840(&mut r), 20);
        w.ground_enable_match_camera(&mut r);
        p.x4C_fixed_cam = 1;
        w.ground_enable_match_camera(&mut r);
        w.stage_info.grkind = Gr_Kind_Kongo;
        w.ground_801c4dd0(&mut r);
        w.stage_info.grkind = Gr_Kind_OldKongo;
        w.ground_801c4e20(&mut r);
        ground_801c5a28(&mut r);
        assert_eq!(r.log, ["std", "fixed", "k70", "ok c8", "t bc", "t 4c 0", "t 18 001"]);
        w.stage_info.param = core::ptr::null_mut();
    }

    #[test]
    fn sfx_slots() {
        let mut r = Rec::default();
        let mut g = Ground::zeroed();
        g.x20 = [-1; 8];
        ground_801c5440(&mut r, Some(&mut g), 2, 100);
        ground_801c5440(&mut r, Some(&mut g), 2, 101);
        ground_801c5440(&mut r, Some(&mut g), 2, 540000);
        ground_801c5440(&mut r, Some(&mut g), 8, 1);
        ground_801c5630(&mut r, Some(&g), 2, 0.5);
        ground_801c5694(&mut r, Some(&g), 2, 0.0);
        r.playing = true;
        assert!(ground_801c54dc(&mut r, Some(&g), 2));
        assert!(!ground_801c54dc(&mut r, Some(&g), 3));
        ground_801c5440(&mut r, Some(&mut g), 2, 540001);
        assert_eq!(g.x20[2], -1);
        ground_801c55ac(&mut r, Some(&mut g));
        ground_801c5414(&mut r, 5, 6);
        assert_eq!(r.log, ["play 100 127 64", "stop 1", "play 101 127 64", "vol 2 63", "pan 2 63", "stop 2", "play4 5 127 64 6"]);
    }
}
