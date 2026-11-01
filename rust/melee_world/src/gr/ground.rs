
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

pub(crate) const M_PI: f64 = 3.14159265358979323846;

/// Callees outside ground.c. Defaults are stubs.
#[allow(unused_variables)]
pub trait GroundHooks {
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_800237a8(&mut self, sfx: i32, vol: i32, pan: i32) -> i32 {
        unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800237A8(sfx, vol, pan) }
    }
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_80023870(&mut self, a: i32, vol: i32, pan: i32, d: i32) -> i32 {
        unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023870(a, vol, pan, d) }
    }
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_800236b8(&mut self, id: i32) -> i32 {
        unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_800236B8(id) }
    }
    /// TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_80023710(&mut self, id: i32) -> bool {
        unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80023710(id) }
    }
    /// TODO(group1: lb/lbaudio_ax.c) volume
    fn lbaudioax_80024b58(&mut self, id: i32, v: i32) {
        unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024B58(id, v) }
    }
    /// TODO(group1: lb/lbaudio_ax.c) pan
    fn lbaudioax_80024b1c(&mut self, id: i32, v: i32) {
        unsafe { melee_runtime::tu::lbaudio_ax::lbAudioAx_80024B1C(id, v) }
    }
    /// TODO(group1: baselib/random.c)
    fn hsd_randi(&mut self, n: i32) -> i32 {
        melee_runtime::tu::random::HSD_Randi(melee_runtime::tu::globals::rand(), n)
    }
    /// TODO(group1: baselib/spline.c)
    fn spl_get_spline_point(&mut self, spline: *mut HSD_Spline, t: f32) -> Vec3 {
        let mut p = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        unsafe { melee_runtime::tu::spline_world::splGetSplinePoint(&mut p, spline, t) };
        Vec3 { x: p.x, y: p.y, z: p.z }
    }
    /// TODO(group1: lb/lbshadow.c)
    fn lbshadow_8000e9f0(&mut self, spline: *mut HSD_Spline, t: f32) -> Vec3 {
        let mut p = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        unsafe { melee_runtime::tu::lbshadow::lbShadow_8000E9F0(&mut p, &*spline, t) };
        Vec3 { x: p.x, y: p.y, z: p.z }
    }
    /// TODO(group2: gr/grzebes.c)
    fn grzebes_801da3f4(&mut self, arg: *mut core::ffi::c_void) {
        unsafe { crate::tu::grzebes::grZebes_801DA3F4(arg as _) }
    }
    /// grKongo_801D8270 (gr/grkongo.c): ported in `gr::leaves`.
    fn grkongo_801d8270(&mut self, g: *mut HSD_GObj) {
        // SAFETY: the C dereferences `GET_GROUND(gobj)` unchecked; callers pass a live Ground GObj.
        unsafe { super::leaves::grkongo_801d8270(g) }
    }
    /// TODO(group2: gr/grkongo.c)
    fn grkongo_801d828c(&mut self, g: *mut HSD_GObj) {
        unsafe { crate::tu::grkongo::grKongo_801D828C(g) }
    }
    /// grOldKongo_802105AC (gr/groldkongo.c): ported in `gr::leaves`.
    fn groldkongo_802105ac(&mut self, g: *mut HSD_GObj) {
        // SAFETY: the C dereferences `GET_GROUND(gobj)` unchecked; callers pass a live Ground GObj.
        unsafe { super::leaves::groldkongo_802105ac(g) }
    }
    /// TODO(group2: gr/groldkongo.c)
    fn groldkongo_802105c8(&mut self, g: *mut HSD_GObj) {
        unsafe { crate::tu::groldkongo::grOldKongo_802105C8(g) }
    }
    /// Camera_SetModeToFixed: `CameraWorld` implements it (cm::camera).
    fn camera_set_mode_to_fixed(&mut self) {
        unsafe { crate::tu::camera::Camera_SetModeToFixed() }
    }
    /// Camera_SetModeToStandard: `CameraWorld` implements it (cm::camera).
    fn camera_set_mode_to_standard(&mut self) {
        unsafe { crate::tu::camera::Camera_SetModeToStandard() }
    }
    /// TODO(group3: pl/player.c)
    fn player_get_entity(&mut self, slot: i32) -> *mut HSD_GObj {
        crate::pl::Player_GetEntity(slot)
    }
    /// TODO(group3: pl/player.c)
    fn player_get_entity_at_index(&mut self, slot: i32, i: i32) -> *mut HSD_GObj {
        crate::pl::Player_GetEntityAtIndex(slot, i)
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

pub(crate) fn color_mut(p: &mut GroundParam, c: ParamColor) -> &mut _GXColor {
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

    /// Ground_SetParamY (HSD_ASSERT(521, 0) when param is NULL)
    pub fn ground_set_param_y(&mut self, y: f32) {
        assert!(!self.stage_info.param.is_null(), "ground.c:521");
        self.param_mut().y = y;
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

    // ---- night 3 round 5: the map-gobj / x280 / x694 tables and the stage-param scale ----

    /// Ground_MexParamStkind (TARGET_PC, static): the stkind whose StageParam row applies.
    pub fn ground_mex_param_stkind(&self, stkind: StKind) -> StKind {
        let Some(param) = self.param() else { return stkind };
        let p = param.stage_params;
        let count = param.stage_param_count as isize;
        if p.is_null() || count <= 0 {
            return stkind;
        }
        // SAFETY: `stage_params` points at `stage_param_count` rows, as the C assumes.
        let rows = unsafe { core::slice::from_raw_parts(p, count as usize) };
        if rows.iter().any(|r| r.stkind == stkind) {
            return stkind;
        }
        if let Some(r) = rows.iter().find(|r| r.stkind == u32::MAX) {
            return r.stkind;
        }
        if count == 1 {
            return rows[0].stkind;
        }
        stkind
    }
    /// Ground_GetMapGObj: `stage_info.map_gobjs[map_id]` (index checked; the C is not).
    pub fn ground_get_map_gobj(&self, map_id: i32) -> *mut Ground_GObj { self.stage_info.map_gobjs[map_id as usize] }

    // ---- per-stage scroll-offset getters (Ground_GetMapGObj + the gobj's Ground) ----


    // ---- night 3 round 5: fog colour, tilt, the light-list scan, the xC callback ----

}

/// Ground_GetP1Fighter
pub fn ground_get_p1_fighter(h: &mut dyn GroundHooks) -> *mut HSD_GObj { h.player_get_entity(0) }

/// Ground_GetP1Fighter2
pub fn ground_get_p1_fighter2(h: &mut dyn GroundHooks) -> *mut HSD_GObj { h.player_get_entity_at_index(0, 1) }

/// C float -> int conversion (fctiwz / cvttss2si); inputs the game passes are in range.
#[inline]
pub(crate) fn f2i(x: f32) -> i32 {
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[derive(Default)]
    struct Rec {
        log: std::vec::Vec<String>,
        next_id: i32,
        playing: bool,
    }
    impl GroundHooks for Rec {
        fn camera_set_mode_to_fixed(&mut self) { self.log.push("fixed".into()); }
        fn camera_set_mode_to_standard(&mut self) { self.log.push("std".into()); }
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

pub use crate::tu::ground::ground_801c1d38;
pub use crate::tu::ground::ground_801c498c;
pub use crate::tu::ground::ground_801c4b50;
pub use crate::tu::ground::ground_801c53ec;
pub use crate::tu::ground::ground_801c5414;
pub use crate::tu::ground::ground_801c5440;
pub use crate::tu::ground::ground_801c54dc;
pub use crate::tu::ground::ground_801c5544;
pub use crate::tu::ground::ground_801c55ac;
pub use crate::tu::ground::ground_801c5630;
pub use crate::tu::ground::ground_801c5694;
pub use crate::tu::ground::ground_801c5a28;
pub use crate::tu::ground::ground_801c5a60;
pub use crate::tu::ground::ground_801c5aec;
