
#![allow(non_upper_case_globals)]

//! Port of src/melee/gr/stage.c (port round 6): the stage camera / blast-zone getters, the
//! fixed-camera projection, spawn-point helpers, the StKind -> GrKind map (with the PC m-ex
//! tail fill) and the selected-stage forwarders, plus `Stage_StageSlotSelect` from
//! stage_stage_slot_retail.inc.
//!
//! The C globals `stage_info`, `selected_stage`, `stage_id_map` and the m-ex `done` flag are
//! fields of [`StageWorld`]. `selected_stage.entry` is an index into the map.
//!
//! The TARGET_PC build routes ten getters through `ScriptGame_ArenaBound`
//! (pc/gameworld/script_arena.inc); its read side is ported here as [`ScriptArenaView::bound`]
//! over a copy of the fields it reads. TODO(pc gameworld: script_arena state owner).
//!
//! Calls into code not ported yet go through [`StageHooks`]:
//! Ground_* (TODO(group2: gr/ground.c)), HSD_Randi (TODO(group1: baselib/random.c)),
//! gm_8016B238 / gm_8016B3A0 (TODO(group6: gm)), lbAudioAx_80023F28 (TODO(group1: lb audio)),
//! Mex_GrExternalCount / Mex_GrKindForExt / OSReport (TODO(pc: mex)).
//!
//! FMA (`python tools/rust-port/fma_sites.py src/melee/gr/stage.c`): only Stage_80224CAC fuses,
//! the two `another_vec * t + arg0` lines. lbVector_ApplyEulerRotation is melee_runtime's.
use crate::mp::lines::GroundColl;

use melee_core::fp::fmadds;

use melee_runtime::lbvector::apply_euler_rotation;

use crate::vec::Vec3;

use melee_types::*;

/// `0.0174532923847F`, stage.c's degrees-to-radians constant.
pub(crate) const DEG2RAD: f32 = 0.017_453_292_384_7_f32;

/// `ST_MEX_EXT_MAX`.
pub const ST_MEX_EXT_MAX: usize = 384;

/// The fields of pc/gameworld/script_arena.inc's `script_arena` that `ScriptGame_ArenaBound`
/// reads. `camera` / `blast` are StageBlastZone order: left, right, top, bottom.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScriptArenaView {
    pub owner: i32,
    pub x: f32,
    pub y: f32,
    pub camera: [f32; 4],
    pub blast: [f32; 4],
}

impl ScriptArenaView {
    /// `ScriptGame_ArenaBound(field, fallback)`: fields 0..7 are camera L/R/T/B, blast L/R/T/B
    /// (origin-relative, plus the arena origin); 8/9 the origin x/y.
    pub fn bound(&self, field: i32, fallback: f32) -> f32 {
        if self.owner == 0 {
            return fallback;
        }
        if field == 8 {
            return self.x;
        }
        if field == 9 {
            return self.y;
        }
        let b = if field < 4 { &self.camera } else { &self.blast };
        let f = field & 3;
        b[f as usize] + if f < 2 { self.x } else { self.y }
    }
}

/// Calls from stage.c into code this crate has not ported.
pub trait StageHooks {
    /// `Ground_801C2D24(index, out)`. TODO(group2: gr/ground.c)
    fn ground_801c2d24(&mut self, index: i32, out: &mut Vec3) -> bool;
    /// `HSD_Randi(n)`. TODO(group1: baselib/random.c)
    fn hsd_randi(&mut self, n: i32) -> i32;
    /// `gm_8016B238()`. TODO(group6: gm)
    fn gm_8016b238(&mut self) -> bool;
    /// `gm_8016B3A0()`. TODO(group6: gm)
    fn gm_8016b3a0(&mut self) -> bool;
    /// `Ground_801C28AC(stkind, arg, &out)`. TODO(group2: gr/ground.c)
    fn ground_801c28ac(&mut self, stkind: StKind, arg: i32, out: &mut i32) -> bool;
    /// `lbAudioAx_80023F28(id)`. TODO(group1: lb/lbaudio_ax.c)
    fn lbaudioax_80023f28(&mut self, id: i32);
    /// `Ground_801C5A84(id)`. TODO(group2: gr/ground.c)
    fn ground_801c5a84(&mut self, id: i32);
    /// `Ground_801C5AA4(b)`. TODO(group2: gr/ground.c)
    fn ground_801c5aa4(&mut self, b: bool);
    /// `Ground_801C06B8(grkind)`. TODO(group2: gr/ground.c)
    fn ground_801c06b8(&mut self, grkind: GrKind);
    /// `Ground_801C0754(&pair)`. TODO(group2: gr/ground.c)
    fn ground_801c0754(&mut self, pair: &StageIdPair);
    /// `Ground_801C0800(&pair)`. TODO(group2: gr/ground.c)
    fn ground_801c0800(&mut self, pair: &StageIdPair);
    /// `Ground_OnLoad(&pair)`. TODO(group2: gr/ground.c)
    fn ground_on_load(&mut self, pair: &StageIdPair);
    /// `Ground_801C0FB8(&pair)`. TODO(group2: gr/ground.c)
    fn ground_801c0fb8(&mut self, pair: &StageIdPair);
    /// `Ground_DemoInit(&pair, arg)`. TODO(group2: gr/ground.c)
    fn ground_demo_init(&mut self, pair: &StageIdPair, arg: i32);
    /// `Mex_GrExternalCount()`. TODO(pc: mex)
    fn mex_gr_external_count(&mut self) -> i32;
    /// `Mex_GrKindForExt(ext)`. TODO(pc: mex)
    fn mex_gr_kind_for_ext(&mut self, ext: i32) -> i32;
    /// The `OSReport` of Stage_MexInitIds when the m-ex count overflows the map.
    fn os_report_ext_overflow(&mut self, _n: i32) {}
}

/// stage.c's globals.
pub struct StageWorld {
    pub stage_info: Box<StageInfo>,
    /// `selected_stage.stkind`
    pub selected_stkind: StKind,
    /// `selected_stage.entry`, as an index into `stage_id_map` (NULL = None).
    pub selected_entry: Option<usize>,
    /// `default_stage_pair`
    pub default_stage_pair: StageIdPair,
    /// `stage_id_map[ST_MEX_EXT_MAX]` grkinds (unk1/unk2 are 0 in every row).
    pub stage_id_map: [GrKind; ST_MEX_EXT_MAX],
    /// Stage_MexInitIds' `static bool done`.
    pub mex_done: bool,
    pub arena: ScriptArenaView,
}

impl Default for StageWorld {
    fn default() -> Self {
        let mut map = [Gr_Kind_Unk00; ST_MEX_EXT_MAX];
        map[..STAGE_ID_MAP_VANILLA.len()].copy_from_slice(&STAGE_ID_MAP_VANILLA);
        StageWorld {
            stage_info: Box::new(StageInfo::zeroed()),
            selected_stkind: St_Kind_Izumi,
            selected_entry: None,
            default_stage_pair: StageIdPair { grkind: Gr_Kind_Izumi, stkind: St_Kind_Izumi },
            stage_id_map: map,
            mex_done: false,
            arena: ScriptArenaView::default(),
        }
    }
}

impl StageWorld {
    #[inline]
    fn ci(&self) -> &StageCameraInfo {
        &self.stage_info.cam_info
    }
    #[inline]
    fn arena_bound(&self, field: i32, fallback: f32) -> f32 {
        self.arena.bound(field, fallback)
    }

    /// `Stage_GetCamBoundsLeftOffset`
    pub fn get_cam_bounds_left_offset(&self) -> f32 {
        self.arena_bound(0, self.ci().cam_bounds.left + self.ci().cam_x_offset)
    }
    /// `Stage_GetCamBoundsRightOffset`
    pub fn get_cam_bounds_right_offset(&self) -> f32 {
        self.arena_bound(1, self.ci().cam_bounds.right + self.ci().cam_x_offset)
    }
    /// `Stage_GetCamBoundsTopOffset`
    pub fn get_cam_bounds_top_offset(&self) -> f32 {
        self.arena_bound(2, self.ci().cam_bounds.top + self.ci().cam_y_offset)
    }
    /// `Stage_GetCamBoundsBottomOffset`
    pub fn get_cam_bounds_bottom_offset(&self) -> f32 {
        self.arena_bound(3, self.ci().cam_bounds.bottom + self.ci().cam_y_offset)
    }
    /// `Stage_GetCamPanAngleRadians`
    pub fn get_cam_pan_angle_radians(&self) -> f32 {
        DEG2RAD * self.ci().cam_pan_degrees
    }
    /// `Stage_GetCamMaxDepth`
    pub fn get_cam_max_depth(&self) -> f32 {
        self.ci().cam_max_depth
    }
    /// `Stage_GetCamZoomRate`
    pub fn get_cam_zoom_rate(&self) -> f32 {
        self.ci().cam_zoom_rate
    }
    /// `Stage_GetCamInfoX20`
    pub fn get_cam_info_x20(&self) -> f32 {
        self.ci().x20
    }
    /// `Stage_GetCamInfoX24`
    pub fn get_cam_info_x24(&self) -> f32 {
        self.ci().x24
    }
    /// `Stage_GetCamFixedZoom`
    pub fn get_cam_fixed_zoom(&self) -> f32 {
        self.ci().cam_fixed_zoom
    }
    /// `Stage_GetCamTrackRatio`
    pub fn get_cam_track_ratio(&self) -> f32 {
        self.ci().cam_track_ratio
    }
    /// `Stage_GetCamTrackSmooth`
    pub fn get_cam_track_smooth(&self) -> f32 {
        self.ci().cam_track_smooth
    }
    /// `Stage_GetBlastZoneRightOffset`
    pub fn get_blast_zone_right_offset(&self) -> f32 {
        self.arena_bound(5, self.stage_info.blast_zone.right + self.ci().cam_x_offset)
    }
    /// `Stage_GetBlastZoneLeftOffset`
    pub fn get_blast_zone_left_offset(&self) -> f32 {
        self.arena_bound(4, self.stage_info.blast_zone.left + self.ci().cam_x_offset)
    }
    /// `Stage_GetBlastZoneTopOffset`
    pub fn get_blast_zone_top_offset(&self) -> f32 {
        self.arena_bound(6, self.stage_info.blast_zone.top + self.ci().cam_y_offset)
    }
    /// `Stage_GetBlastZoneBottomOffset`
    pub fn get_blast_zone_bottom_offset(&self) -> f32 {
        self.arena_bound(7, self.stage_info.blast_zone.bottom + self.ci().cam_y_offset)
    }
    /// `Stage_CalcUnkCamY` (TARGET_PC body).
    pub fn calc_unk_cam_y(&self) -> f32 {
        0.5 * (self.get_cam_bounds_bottom_offset() + self.get_blast_zone_bottom_offset())
    }
    /// `Stage_CalcUnkCamYBounds` (TARGET_PC body).
    pub fn calc_unk_cam_y_bounds(&self) -> f32 {
        0.5 * (self.get_cam_bounds_bottom_offset() + self.calc_unk_cam_y())
    }
    /// `Stage_UnkSetVec3TCam_Offset`
    pub fn unk_set_vec3_tcam_offset(&self) -> Vec3 {
        Vec3 {
            x: self.arena_bound(8, self.ci().cam_x_offset),
            y: self.arena_bound(9, self.ci().cam_y_offset),
            z: 0.0,
        }
    }
    /// `Stage_GetPauseCamZPosMin`
    pub fn get_pause_cam_zpos_min(&self) -> f32 {
        self.ci().pausecam_zpos_min
    }
    /// `Stage_GetPauseCamZPosInit`
    pub fn get_pause_cam_zpos_init(&self) -> f32 {
        self.ci().pausecam_zpos_init
    }
    /// `Stage_GetPauseCamZPosMax`
    pub fn get_pause_cam_zpos_max(&self) -> f32 {
        self.ci().pausecam_zpos_max
    }
    /// `Stage_GetCamAngleRadiansUp`
    pub fn get_cam_angle_radians_up(&self) -> f32 {
        DEG2RAD * self.ci().cam_angle_up
    }
    /// `Stage_GetCamAngleRadiansDown`
    pub fn get_cam_angle_radians_down(&self) -> f32 {
        DEG2RAD * self.ci().cam_angle_down
    }
    /// `Stage_GetCamAngleRadiansLeft`
    pub fn get_cam_angle_radians_left(&self) -> f32 {
        DEG2RAD * self.ci().cam_angle_left
    }
    /// `Stage_GetCamAngleRadiansRight`
    pub fn get_cam_angle_radians_right(&self) -> f32 {
        DEG2RAD * self.ci().cam_angle_right
    }
    /// `Stage_SetVecToFixedCamPos`
    pub fn set_vec_to_fixed_cam_pos(&self) -> Vec3 {
        v3(self.ci().fixed_cam_pos)
    }
    /// `Stage_GetCamFixedFov`
    pub fn get_cam_fixed_fov(&self) -> f32 {
        self.ci().fixed_cam_fov
    }
    /// `Stage_MexInitIds`: fill the external ids past the vanilla initializer from m-ex, once.
    pub fn mex_init_ids(&mut self, h: &mut dyn StageHooks) {
        if self.mex_done {
            return;
        }
        self.mex_done = true;
        let mut n = h.mex_gr_external_count();
        if n > ST_MEX_EXT_MAX as i32 {
            h.os_report_ext_overflow(n);
            n = ST_MEX_EXT_MAX as i32;
        }
        for e in 0..n.max(0) {
            let k = h.mex_gr_kind_for_ext(e);
            if k >= 0 {
                self.stage_id_map[e as usize] = k as GrKind;
            }
        }
    }
    fn selected_grkind(&self) -> GrKind {
        // The C dereferences selected_stage.entry; NULL there is a crash.
        self.stage_id_map[self.selected_entry.expect("selected_stage.entry is NULL")]
    }
    /// `Stage_StageSlotSelect` (stage_stage_slot_retail.inc): select the first StKind whose
    /// map row is `kind`; -1 if none.
    pub fn stage_slot_select(&mut self, kind: i32) -> i32 {
        for i in 0..ST_MEX_EXT_MAX {
            if self.stage_id_map[i] as i32 == kind {
                self.selected_stkind = i as StKind;
                self.selected_entry = Some(i);
                return i as i32;
            }
        }
        -1
    }
}

/// `stage_id_map` initializer (vanilla rows, gr/stage.c), indexed by StKind.
pub const STAGE_ID_MAP_VANILLA: [GrKind; 286] = [
    Gr_Kind_Unk00, Gr_Kind_Test, Gr_Kind_Izumi, Gr_Kind_PStadium,
    Gr_Kind_Castle, Gr_Kind_Kongo, Gr_Kind_Zebes, Gr_Kind_Corneria,
    Gr_Kind_Story, Gr_Kind_Onett, Gr_Kind_MuteCity, Gr_Kind_RCruise,
    Gr_Kind_Garden, Gr_Kind_GreatBay, Gr_Kind_Shrine, Gr_Kind_Kraid,
    Gr_Kind_Yorster, Gr_Kind_Greens, Gr_Kind_Fourside, Gr_Kind_Inishie1,
    Gr_Kind_Inishie2, Gr_Kind_Unk26, Gr_Kind_Venom, Gr_Kind_Pura,
    Gr_Kind_BigBlue, Gr_Kind_Icemt, Gr_Kind_Icemt, Gr_Kind_Flatzone,
    Gr_Kind_OldPupupu, Gr_Kind_OldYoshi, Gr_Kind_OldKongo, Gr_Kind_Battle,
    Gr_Kind_Last, Gr_Kind_TMario, Gr_Kind_TCaptain, Gr_Kind_TClink,
    Gr_Kind_TDonkey, Gr_Kind_TDrmario, Gr_Kind_TFalco, Gr_Kind_TFox,
    Gr_Kind_TIceclimber, Gr_Kind_TKirby, Gr_Kind_TKoopa, Gr_Kind_TLink,
    Gr_Kind_TLuigi, Gr_Kind_TMars, Gr_Kind_TMewtwo, Gr_Kind_TNess,
    Gr_Kind_TPeach, Gr_Kind_TPichu, Gr_Kind_TPikachu, Gr_Kind_TPurin,
    Gr_Kind_TSamus, Gr_Kind_TSeak, Gr_Kind_TYoshi, Gr_Kind_TZelda,
    Gr_Kind_TGamewatch, Gr_Kind_TEmblem, Gr_Kind_TGanon, Gr_Kind_KinokoRoute,
    Gr_Kind_Castle, Gr_Kind_Kongo, Gr_Kind_Garden, Gr_Kind_ShrineRoute,
    Gr_Kind_Shrine, Gr_Kind_Zebes, Gr_Kind_ZebesRoute, Gr_Kind_Greens,
    Gr_Kind_Greens, Gr_Kind_Greens, Gr_Kind_Corneria, Gr_Kind_Corneria,
    Gr_Kind_PStadium, Gr_Kind_BigBlueRoute, Gr_Kind_MuteCity, Gr_Kind_Onett,
    Gr_Kind_Icemt, Gr_Kind_Icemt, Gr_Kind_Battle, Gr_Kind_Battle,
    Gr_Kind_Last, Gr_Kind_Last, Gr_Kind_Pushon, Gr_Kind_FigureGet,
    Gr_Kind_Homerun, Gr_Kind_Heal, Gr_Kind_Castle, Gr_Kind_RCruise,
    Gr_Kind_Kongo, Gr_Kind_Garden, Gr_Kind_GreatBay, Gr_Kind_Shrine,
    Gr_Kind_Zebes, Gr_Kind_Kraid, Gr_Kind_Story, Gr_Kind_Yorster,
    Gr_Kind_Izumi, Gr_Kind_Greens, Gr_Kind_Corneria, Gr_Kind_Venom,
    Gr_Kind_PStadium, Gr_Kind_Inishie1, Gr_Kind_Inishie2, Gr_Kind_MuteCity,
    Gr_Kind_BigBlue, Gr_Kind_Onett, Gr_Kind_Fourside, Gr_Kind_PStadium,
    Gr_Kind_Castle, Gr_Kind_Battle, Gr_Kind_Castle, Gr_Kind_Inishie2,
    Gr_Kind_Shrine, Gr_Kind_GreatBay, Gr_Kind_Last, Gr_Kind_PStadium,
    Gr_Kind_Icemt, Gr_Kind_Icemt, Gr_Kind_Inishie1, Gr_Kind_GreatBay,
    Gr_Kind_Shrine, Gr_Kind_Corneria, Gr_Kind_Venom, Gr_Kind_GreatBay,
    Gr_Kind_PStadium, Gr_Kind_Battle, Gr_Kind_Inishie2, Gr_Kind_Kongo,
    Gr_Kind_Shrine, Gr_Kind_GreatBay, Gr_Kind_Inishie1, Gr_Kind_GreatBay,
    Gr_Kind_BigBlue, Gr_Kind_Zebes, Gr_Kind_Story, Gr_Kind_Yorster,
    Gr_Kind_Greens, Gr_Kind_Izumi, Gr_Kind_Greens, Gr_Kind_Icemt,
    Gr_Kind_Corneria, Gr_Kind_MuteCity, Gr_Kind_PStadium, Gr_Kind_PStadium,
    Gr_Kind_Inishie1, Gr_Kind_Onett, Gr_Kind_Fourside, Gr_Kind_BigBlue,
    Gr_Kind_Battle, Gr_Kind_Battle, Gr_Kind_Battle, Gr_Kind_Shrine,
    Gr_Kind_GreatBay, Gr_Kind_Shrine, Gr_Kind_GreatBay, Gr_Kind_Castle,
    Gr_Kind_Kongo, Gr_Kind_GreatBay, Gr_Kind_Story, Gr_Kind_Inishie2,
    Gr_Kind_MuteCity, Gr_Kind_PStadium, Gr_Kind_Izumi, Gr_Kind_Inishie1,
    Gr_Kind_Shrine, Gr_Kind_RCruise, Gr_Kind_Garden, Gr_Kind_Izumi,
    Gr_Kind_Inishie2, Gr_Kind_Onett, Gr_Kind_PStadium, Gr_Kind_Icemt,
    Gr_Kind_PStadium, Gr_Kind_Flatzone, Gr_Kind_MuteCity, Gr_Kind_Battle,
    Gr_Kind_Last, Gr_Kind_RCruise, Gr_Kind_Kongo, Gr_Kind_GreatBay,
    Gr_Kind_Zebes, Gr_Kind_Story, Gr_Kind_Greens, Gr_Kind_Corneria,
    Gr_Kind_PStadium, Gr_Kind_Inishie1, Gr_Kind_MuteCity, Gr_Kind_Onett,
    Gr_Kind_Pura, Gr_Kind_Icemt, Gr_Kind_Castle, Gr_Kind_Shrine,
    Gr_Kind_Izumi, Gr_Kind_Battle, Gr_Kind_Yorster, Gr_Kind_Inishie2,
    Gr_Kind_Garden, Gr_Kind_Venom, Gr_Kind_Fourside, Gr_Kind_Last,
    Gr_Kind_Flatzone, Gr_Kind_Kraid, Gr_Kind_Battle, Gr_Kind_Shrine,
    Gr_Kind_Castle, Gr_Kind_Story, Gr_Kind_Onett, Gr_Kind_Izumi,
    Gr_Kind_PStadium, Gr_Kind_Zebes, Gr_Kind_GreatBay, Gr_Kind_Yorster,
    Gr_Kind_Icemt, Gr_Kind_MuteCity, Gr_Kind_RCruise, Gr_Kind_Figure1,
    Gr_Kind_Battle, Gr_Kind_Corneria, Gr_Kind_Garden, Gr_Kind_Kongo,
    Gr_Kind_Last, Gr_Kind_Zebes, Gr_Kind_Castle, Gr_Kind_Inishie2,
    Gr_Kind_Kraid, Gr_Kind_Yorster, Gr_Kind_Fourside, Gr_Kind_Figure2,
    Gr_Kind_Venom, Gr_Kind_Greens, Gr_Kind_Shrine, Gr_Kind_Izumi,
    Gr_Kind_Inishie1, Gr_Kind_Corneria, Gr_Kind_BigBlueRoute, Gr_Kind_GreatBay,
    Gr_Kind_OldPupupu, Gr_Kind_Fourside, Gr_Kind_Izumi, Gr_Kind_Inishie2,
    Gr_Kind_PStadium, Gr_Kind_Shrine, Gr_Kind_Shrine, Gr_Kind_Pura,
    Gr_Kind_BigBlue, Gr_Kind_Battle, Gr_Kind_Flatzone, Gr_Kind_Shrine,
    Gr_Kind_Figure3, Gr_Kind_Story, Gr_Kind_Inishie1, Gr_Kind_Last,
    Gr_Kind_Last, Gr_Kind_Garden, Gr_Kind_Story, Gr_Kind_Castle,
    Gr_Kind_RCruise, Gr_Kind_GreatBay, Gr_Kind_Shrine, Gr_Kind_MuteCity,
    Gr_Kind_Corneria, Gr_Kind_PStadium, Gr_Kind_Onett, Gr_Kind_Icemt,
    Gr_Kind_Inishie2, Gr_Kind_Pura, Gr_Kind_Last, Gr_Kind_Flatzone,
    Gr_Kind_Venom, Gr_Kind_PStadium, Gr_Kind_GreatBay, Gr_Kind_Shrine,
    Gr_Kind_Last, Gr_Kind_Battle, Gr_Kind_PStadium, Gr_Kind_Last,
    Gr_Kind_Inishie2, Gr_Kind_Izumi, Gr_Kind_Flatzone, Gr_Kind_Castle,
    Gr_Kind_Last, Gr_Kind_GreatBay, Gr_Kind_Battle, Gr_Kind_PStadium,
    Gr_Kind_Shrine, Gr_Kind_Battle,
];

#[inline]
pub(crate) fn v3(v: melee_types::Vec) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}

pub use crate::tu::stage::stage_80224dc8;
