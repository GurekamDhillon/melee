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
const DEG2RAD: f32 = 0.017_453_292_384_7_f32;
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
    /// `Stage_80224CAC`: where the fixed camera's view ray from `fixed_cam_pos` meets z = 0.
    pub fn stage_80224cac(&self) -> Vec3 {
        let mut another = Vec3 { x: 0.0, y: 0.0, z: -100.0 };
        let p = v3(self.ci().fixed_cam_pos);
        let rot = Vec3 {
            x: DEG2RAD * self.ci().fixed_cam_vert_angle,
            y: DEG2RAD * self.ci().fixed_cam_horz_angle,
            z: 0.0,
        };
        apply_euler_rotation(&mut another, &rot);
        let t = p.z / -another.z;
        Vec3 { x: fmadds(another.x, t, p.x), y: fmadds(another.y, t, p.y), z: 0.0 }
    }
    /// `Stage_SetVecToFixedCamPos`
    pub fn set_vec_to_fixed_cam_pos(&self) -> Vec3 {
        v3(self.ci().fixed_cam_pos)
    }
    /// `Stage_GetCamFixedFov`
    pub fn get_cam_fixed_fov(&self) -> f32 {
        self.ci().fixed_cam_fov
    }
    /// `Stage_80224E38`
    pub fn stage_80224e38(&self, h: &mut dyn StageHooks, out: &mut Vec3, arg1: i32) {
        h.ground_801c2d24(arg1.wrapping_add(4), out);
    }
    /// `Stage_80224E64`: spawn point `arg0`; kind 4 probes the floor near x = 0 with
    /// `mpCheckMultiple` (floor only) in 10-unit steps, up then down.
    /// Panics where the C asserts (`arg0 == -1`, stage.c:360).
    pub fn stage_80224e64(
        &self, h: &mut dyn StageHooks, coll: &mut GroundColl, arg0: i32, out: &mut Vec3,
    ) {
        if arg0 == -1 {
            panic!("stage.c:360: no!=St_Player_InitPos_None");
        }
        if arg0 == 4 {
            let mut found: Option<Vec3> = None;
            let mut c = -10.0f32;
            while c < 100.0 {
                if let Some(hit) = coll.mp_check_multiple(0.0, 10.0 + c, 0.0, c, 1, -1, -1) {
                    found = Some(hit.pos);
                    break;
                }
                c += 10.0;
            }
            if found.is_none() {
                c = -10.0;
                while c > -100.0 {
                    if let Some(hit) = coll.mp_check_multiple(0.0, c, 0.0, c - 10.0, 1, -1, -1)
                    {
                        found = Some(hit.pos);
                        break;
                    }
                    c -= 10.0;
                }
            }
            *out = found.unwrap_or(Vec3 { x: 0.0, y: 0.0, z: 0.0 });
            return;
        }
        h.ground_801c2d24(arg0, out);
    }
    /// `Stage_80224FDC`: a random spawn point, 21 tries over indices 0x7F.., then the C's
    /// `counter2 = rand_output` loop over 0..4, kept as is.
    pub fn stage_80224fdc(&self, h: &mut dyn StageHooks, out: &mut Vec3) -> bool {
        let mut counter = 0x15;
        while counter != 0 {
            let r = h.hsd_randi(counter);
            if h.ground_801c2d24(r.wrapping_add(0x7F), out) {
                return true;
            }
            counter -= 1;
        }
        let mut counter2 = 4;
        while counter2 != 0 {
            let r = h.hsd_randi(counter2);
            if h.ground_801c2d24(r, out) {
                return true;
            }
            counter2 = r;
        }
        false
    }
    /// `Stage_80225074`. Panics where the C asserts (stage.c:526).
    pub fn stage_80225074(&self, h: &mut dyn StageHooks, arg0: i32) -> i32 {
        let r31 = if h.gm_8016b238() {
            if self.stage_info.unk8C.b0() != 0 || arg0 == 2 {
                0x12
            } else {
                0x11
            }
        } else if h.gm_8016b3a0() {
            match arg0 {
                2 => 0x22,
                1 => 0x64,
                _ => 0x24,
            }
        } else {
            match arg0 {
                0 => 4,
                3 => 1,
                2 => 2,
                1 => 0x44,
                _ => panic!("stage.c:526: assert"),
            }
        };
        let mut sp_c = 0i32;
        let tmp = h.ground_801c28ac(self.selected_stkind, r31, &mut sp_c);
        h.lbaudioax_80023f28(sp_c);
        h.ground_801c5a84(sp_c);
        h.ground_801c5aa4(tmp);
        arg0
    }
    /// `Stage_80225194`
    pub fn stage_80225194(&self) -> StKind {
        self.selected_stkind
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
    /// `Stage_8022519C`: StKind -> GrKind.
    pub fn stage_8022519c(&mut self, h: &mut dyn StageHooks, stkind: StKind) -> GrKind {
        self.mex_init_ids(h);
        self.stage_id_map[stkind as usize]
    }
    /// `Stage_802251B4`
    pub fn stage_802251b4(&mut self, h: &mut dyn StageHooks, stkind: StKind) {
        self.mex_init_ids(h);
        let g = self.stage_id_map[stkind as usize];
        h.ground_801c06b8(g);
    }
    fn selected_grkind(&self) -> GrKind {
        // The C dereferences selected_stage.entry; NULL there is a crash.
        self.stage_id_map[self.selected_entry.expect("selected_stage.entry is NULL")]
    }
    /// `Stage_802251E8`: select `stkind`.
    pub fn stage_802251e8(&mut self, h: &mut dyn StageHooks, stkind: StKind) {
        self.mex_init_ids(h);
        self.selected_stkind = stkind;
        self.selected_entry = Some(stkind as usize);
        let pair = StageIdPair { grkind: self.selected_grkind(), stkind: self.selected_stkind };
        h.ground_801c0754(&pair);
    }
    /// `Stage_8022524C`
    pub fn stage_8022524c(&self, h: &mut dyn StageHooks) {
        let pair = StageIdPair { grkind: self.selected_grkind(), stkind: self.selected_stkind };
        h.ground_801c0800(&pair);
    }
    /// `Stage_80225298`
    pub fn stage_80225298(&self, h: &mut dyn StageHooks) {
        let pair = StageIdPair { grkind: self.selected_grkind(), stkind: self.selected_stkind };
        h.ground_on_load(&pair);
    }
    /// `Stage_802252E4`
    pub fn stage_802252e4(&self, h: &mut dyn StageHooks, stkind: StKind) {
        let pair = StageIdPair { grkind: self.selected_grkind(), stkind };
        h.ground_801c0fb8(&pair);
    }
    /// `Stage_8022532C`
    pub fn stage_8022532c(&self, h: &mut dyn StageHooks, stkind: StKind, arg1: i32) {
        let pair = StageIdPair { grkind: self.selected_grkind(), stkind };
        h.ground_demo_init(&pair, arg1);
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

/// `Stage_80224DC8`
pub fn stage_80224dc8(arg: i32) -> bool {
    arg == 0x3b || arg == 0x3f || arg == 0x42 || arg == 0x49 || arg == 0x4c
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
fn v3(v: melee_types::Vec) -> Vec3 {
    Vec3 { x: v.x, y: v.y, z: v.z }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Log {
        calls: std::vec::Vec<String>,
        rand: std::vec::Vec<i32>,
        hit: Option<i32>,
        b238: bool,
        b3a0: bool,
    }
    impl StageHooks for Log {
        fn ground_801c2d24(&mut self, i: i32, out: &mut Vec3) -> bool {
            self.calls.push(format!("2D24 {i}"));
            out.x = i as f32;
            self.hit == Some(i)
        }
        fn hsd_randi(&mut self, n: i32) -> i32 {
            self.calls.push(format!("rand {n}"));
            if self.rand.is_empty() { 0 } else { self.rand.remove(0) }
        }
        fn gm_8016b238(&mut self) -> bool { self.b238 }
        fn gm_8016b3a0(&mut self) -> bool { self.b3a0 }
        fn ground_801c28ac(&mut self, s: StKind, a: i32, out: &mut i32) -> bool {
            self.calls.push(format!("28AC {s} {a:#x}"));
            *out = 77;
            true
        }
        fn lbaudioax_80023f28(&mut self, id: i32) { self.calls.push(format!("audio {id}")); }
        fn ground_801c5a84(&mut self, id: i32) { self.calls.push(format!("5A84 {id}")); }
        fn ground_801c5aa4(&mut self, b: bool) { self.calls.push(format!("5AA4 {b}")); }
        fn ground_801c06b8(&mut self, g: GrKind) { self.calls.push(format!("06B8 {g}")); }
        fn ground_801c0754(&mut self, p: &StageIdPair) { self.calls.push(format!("0754 {} {}", p.grkind, p.stkind)); }
        fn ground_801c0800(&mut self, p: &StageIdPair) { self.calls.push(format!("0800 {} {}", p.grkind, p.stkind)); }
        fn ground_on_load(&mut self, p: &StageIdPair) { self.calls.push(format!("load {} {}", p.grkind, p.stkind)); }
        fn ground_801c0fb8(&mut self, p: &StageIdPair) { self.calls.push(format!("0FB8 {} {}", p.grkind, p.stkind)); }
        fn ground_demo_init(&mut self, p: &StageIdPair, a: i32) { self.calls.push(format!("demo {} {} {a}", p.grkind, p.stkind)); }
        fn mex_gr_external_count(&mut self) -> i32 { 0 }
        fn mex_gr_kind_for_ext(&mut self, _: i32) -> i32 { -1 }
    }

    #[test]
    fn hook_driven() {
        let mut w = StageWorld::default();
        let mut l = Log::default();
        // 80225074: plain mode table, then the 1P / special-mode branches.
        for (a, r) in [(0, 4), (3, 1), (2, 2), (1, 0x44)] {
            l.calls.clear();
            assert_eq!(w.stage_80225074(&mut l, a), a);
            assert_eq!(l.calls, [format!("28AC 2 {r:#x}"), "audio 77".into(), "5A84 77".into(), "5AA4 true".into()]);
        }
        l.b3a0 = true;
        for (a, r) in [(2, 0x22), (1, 0x64), (0, 0x24)] {
            l.calls.clear();
            w.stage_80225074(&mut l, a);
            assert_eq!(l.calls[0], format!("28AC 2 {r:#x}"));
        }
        l.b238 = true;
        l.calls.clear();
        w.stage_80225074(&mut l, 0);
        assert_eq!(l.calls[0], "28AC 2 0x11");
        w.stage_info.unk8C.set_b0(1);
        l.calls.clear();
        w.stage_80225074(&mut l, 0);
        assert_eq!(l.calls[0], "28AC 2 0x12");

        // 80224FDC: second loop feeds the random result back as the next bound.
        let mut v = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        l.calls.clear();
        l.rand = [0; 21].iter().copied().chain([3, 1, 0]).collect();
        l.hit = None;
        assert!(!w.stage_80224fdc(&mut l, &mut v));
        let rands: std::vec::Vec<_> = l.calls.iter().filter(|c| c.starts_with("rand")).cloned().collect();
        assert_eq!(rands.len(), 24);
        assert_eq!(&rands[21..], ["rand 4", "rand 3", "rand 1"]);
        l.calls.clear();
        l.rand = vec![5];
        l.hit = Some(0x84);
        assert!(w.stage_80224fdc(&mut l, &mut v));
        assert_eq!(l.calls, ["rand 21", "2D24 132"]);

        // forwarders and selection
        l.calls.clear();
        w.stage_80224e38(&mut l, &mut v, 3);
        w.stage_802251e8(&mut l, 31);
        w.stage_8022524c(&mut l);
        w.stage_80225298(&mut l);
        w.stage_802252e4(&mut l, 5);
        w.stage_8022532c(&mut l, 6, 9);
        w.stage_802251b4(&mut l, 2);
        let g = Gr_Kind_Battle;
        assert_eq!(l.calls, [
            "2D24 7".to_string(), format!("0754 {g} 31"), format!("0800 {g} 31"),
            format!("load {g} 31"), format!("0FB8 {g} 5"), format!("demo {g} 6 9"),
            format!("06B8 {}", Gr_Kind_Izumi),
        ]);
        assert_eq!(w.stage_80225194(), 31);
    }
}
