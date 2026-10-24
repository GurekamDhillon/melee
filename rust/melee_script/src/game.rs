//! The game half of the scripting API as the native half sees it: the `gw_ScriptGame_*` scalar functions of
//! `pc/gameworld/script_game.c`. The field numbers are the shared enums of `pc/gameworld/script_lab.h` and the
//! `SI_*` / `SF_*` enums at the top of `gw_script.c`; they are kept numerically identical so the ported
//! `script_game.c` (melee_modes) can implement [`ScriptGame`] by forwarding.
//!
//! Every method has a default that answers "no such thing" (0, nothing present), which is what the C build
//! returns with no match running. Scalars cross, never pointers; floats are f32.

// ---- gw_script.c: ScriptGame_FighterF / FighterI
pub const SF_X: i32 = 0;
pub const SF_Y: i32 = 1;
pub const SF_VX: i32 = 2;
pub const SF_VY: i32 = 3;
pub const SF_PERCENT: i32 = 4;
pub const SF_FACING: i32 = 5;
pub const SF_ANIM_FRAME: i32 = 6;
pub const SF_HITLAG: i32 = 7;

pub const SI_PRESENT: i32 = 0;
pub const SI_KIND: i32 = 1;
pub const SI_CHAR: i32 = 2;
pub const SI_ACTION: i32 = 3;
pub const SI_AIRBORNE: i32 = 4;
pub const SI_STOCKS: i32 = 5;
pub const SI_COSTUME: i32 = 6;
pub const SI_SLOT_TYPE: i32 = 7;
pub const SI_FALLS: i32 = 8;
pub const SI_TEAM: i32 = 9;

/// `pc/gameworld/script_lab.h`
pub mod lab {
    // ScriptGame_LabF
    pub const F_ANIM_RATE: i32 = 0;
    pub const F_HITSTUN: i32 = 1;
    pub const F_KB_VX: i32 = 2;
    pub const F_KB_VY: i32 = 3;
    pub const F_SHIELD: i32 = 4;
    pub const F_ECB_TOP_X: i32 = 5;
    pub const F_ECB_TOP_Y: i32 = 6;
    pub const F_ECB_BOTTOM_X: i32 = 7;
    pub const F_ECB_BOTTOM_Y: i32 = 8;
    pub const F_ECB_LEFT_X: i32 = 9;
    pub const F_ECB_LEFT_Y: i32 = 10;
    pub const F_ECB_RIGHT_X: i32 = 11;
    pub const F_ECB_RIGHT_Y: i32 = 12;
    pub const F_GR_VEL: i32 = 13;
    pub const F_KB_APPLIED: i32 = 14;
    pub const F_Z: i32 = 15;
    pub const F_SCALE: i32 = 16;
    pub const F_CMD_TIMER: i32 = 17;
    pub const F_KB_LAST: i32 = 18;
    pub const F_SHIELD_X: i32 = 19;
    pub const F_SHIELD_Y: i32 = 20;
    pub const F_SHIELD_R: i32 = 21;
    pub const F_FLOOR_Y: i32 = 22;
    pub const F_COUNT: i32 = 23;
    // ScriptGame_LabI
    pub const I_ANIM_ID: i32 = 0;
    pub const I_NAME_KIND: i32 = 1;
    pub const I_INTANG_TIMER: i32 = 2;
    pub const I_INVINC_TIMER: i32 = 3;
    pub const I_BODY_STATE: i32 = 4;
    pub const I_TIMED_STATE: i32 = 5;
    pub const I_JUMPS_USED: i32 = 6;
    pub const I_MAX_JUMPS: i32 = 7;
    pub const I_WALLJUMPS_USED: i32 = 8;
    pub const I_IN_HITLAG: i32 = 9;
    pub const I_IN_HITSTUN: i32 = 10;
    pub const I_IASA: i32 = 11;
    pub const I_LEDGE_COOLDOWN: i32 = 12;
    pub const I_ECB_LOCK: i32 = 13;
    pub const I_DRAW_FLAGS: i32 = 14;
    pub const I_SUB: i32 = 15;
    pub const I_JOINTS: i32 = 16;
    pub const I_HURTBOXES: i32 = 17;
    pub const I_HITSTUN_TOTAL: i32 = 18;
    pub const I_HIDDEN: i32 = 19;
    pub const I_KIND: i32 = 20;
    pub const I_LR_AGE: i32 = 21;
    pub const I_JUMP_AGE: i32 = 22;
    pub const I_SHIELD_ON: i32 = 23;
    pub const I_ON_FLOOR: i32 = 24;
    pub const I_FLOOR_VALID: i32 = 25;
    pub const I_FLOOR_PASSTHROUGH: i32 = 26;
    pub const I_WALL: i32 = 27;
    pub const I_CEILING: i32 = 28;
    pub const I_LEDGE: i32 = 29;
    pub const I_COUNT: i32 = 30;
    // ScriptGame_HitI / HitF
    pub const HI_STATE: i32 = 0;
    pub const HI_GROUP: i32 = 1;
    pub const HI_BONE: i32 = 2;
    pub const HI_ANGLE: i32 = 3;
    pub const HI_KBG: i32 = 4;
    pub const HI_WBK: i32 = 5;
    pub const HI_BKB: i32 = 6;
    pub const HI_ELEMENT: i32 = 7;
    pub const HI_SHIELD_DMG: i32 = 8;
    pub const HI_SFX_SEVERITY: i32 = 9;
    pub const HI_SFX_KIND: i32 = 10;
    pub const HI_HIT_AIR: i32 = 11;
    pub const HI_HIT_GROUND: i32 = 12;
    pub const HI_CLANK: i32 = 13;
    pub const HI_REBOUND: i32 = 14;
    pub const HI_COUNT: i32 = 15;
    pub const HF_DAMAGE: i32 = 0;
    pub const HF_SIZE: i32 = 1;
    pub const HF_X: i32 = 2;
    pub const HF_Y: i32 = 3;
    pub const HF_Z: i32 = 4;
    pub const HF_PX: i32 = 5;
    pub const HF_PY: i32 = 6;
    pub const HF_PZ: i32 = 7;
    pub const HF_OX: i32 = 8;
    pub const HF_OY: i32 = 9;
    pub const HF_OZ: i32 = 10;
    pub const HF_COUNT: i32 = 11;
    // ScriptGame_HurtI / HurtF
    pub const UI_STATE: i32 = 0;
    pub const UI_BONE: i32 = 1;
    pub const UI_HEIGHT: i32 = 2;
    pub const UI_GRABBABLE: i32 = 3;
    pub const UF_AX: i32 = 0;
    pub const UF_AY: i32 = 1;
    pub const UF_AZ: i32 = 2;
    pub const UF_BX: i32 = 3;
    pub const UF_BY: i32 = 4;
    pub const UF_BZ: i32 = 5;
    pub const UF_SIZE: i32 = 6;
    // ScriptGame_CameraF
    pub const CAM_OK: i32 = 0;
    pub const CAM_VIEW: i32 = 1; // 12 entries
    pub const CAM_PROJ: i32 = 13;
    pub const CAM_P0: i32 = 14;
    pub const CAM_P1: i32 = 15;
    pub const CAM_P2: i32 = 16;
    pub const CAM_P3: i32 = 17;
    pub const CAM_NEAR: i32 = 18;
    pub const CAM_FAR: i32 = 19;
    pub const CAM_VP_XMIN: i32 = 20;
    pub const CAM_VP_XMAX: i32 = 21;
    pub const CAM_VP_YMIN: i32 = 22;
    pub const CAM_VP_YMAX: i32 = 23;
    pub const CAM_COUNT: i32 = 24;
    // LabStageDraw bits
    pub const STAGE_COLL: i32 = 1;
    pub const STAGE_TERRAIN: i32 = 2;
    pub const STAGE_LEDGES: i32 = 4;
    pub const STAGE_POINTS: i32 = 8;
    pub const STAGE_ZONES: i32 = 16;
    // ScriptGame_LabDObjI
    pub const DI_COUNT: i32 = 0;
    pub const DI_MODELS: i32 = 1;
    pub const DI_MODEL_STATE: i32 = 2;
    pub const DI_COSTUME_TOBJS: i32 = 3;
    pub const DI_FLAGS: i32 = 4;
    pub const DI_RENDER: i32 = 5;
    pub const DI_TOBJS: i32 = 6;
    pub const DI_SET_SOLID: u32 = 0x4000_0000;
    pub const DI_CLEAR_SOLID: u32 = 0x2000_0000;
    // ScriptGame_LabTObjF
    pub const TF_ID: i32 = 0;
    pub const TF_SRC: i32 = 1;
    pub const TF_FLAGS: i32 = 2;
    pub const TF_TU: i32 = 3;
    pub const TF_TV: i32 = 4;
    pub const TF_SU: i32 = 5;
    pub const TF_SV: i32 = 6;
    pub const TF_FRAME: i32 = 7;
    pub const TF_FMT: i32 = 8;
    pub const TF_W: i32 = 9;
    pub const TF_H: i32 = 10;
    pub const TF_COUNT: i32 = 11;
}

pub trait ScriptGame {
    // ---- fighters
    fn fighter_i(&self, _slot: i32, _field: i32) -> i32 {
        0
    }
    fn fighter_f(&self, _slot: i32, _field: i32) -> f32 {
        0.0
    }
    fn lab_i(&self, _slot: i32, _field: i32) -> i32 {
        0
    }
    fn lab_f(&self, _slot: i32, _field: i32) -> f32 {
        0.0
    }
    /// The figatree symbol of the playing animation (`ScriptGame_LabAnimSymbol`), None when there is none.
    fn lab_anim_symbol(&self, _slot: i32) -> Option<String> {
        None
    }
    fn hit_i(&self, _slot: i32, _i: i32, _field: i32) -> i32 {
        0
    }
    fn hit_f(&self, _slot: i32, _i: i32, _field: i32) -> f32 {
        0.0
    }
    fn hurt_i(&self, _slot: i32, _i: i32, _field: i32) -> i32 {
        0
    }
    fn hurt_f(&self, _slot: i32, _i: i32, _field: i32) -> f32 {
        0.0
    }
    fn joint_f(&self, _slot: i32, _i: i32, _comp: i32) -> f32 {
        0.0
    }
    fn joints_setup(&mut self, _slot: i32) {}
    /// -2 when the joint is not valid, -1 for a root.
    fn joint_parent(&self, _slot: i32, _i: i32) -> i32 {
        -2
    }
    fn lab_dobj_i(&mut self, _slot: i32, _d: i32, _field: u32) -> i32 {
        0
    }
    fn lab_tobj_f(&self, _slot: i32, _d: i32, _t: i32, _field: i32) -> f32 {
        0.0
    }
    fn lab_debug_draw(&mut self, _slot: i32, _write: i32, _flags: i32) -> i32 {
        0
    }
    /// `ScriptGame_LabStageDraw(mask, flags)`: negative when there is no match camera.
    fn lab_stage_draw(&mut self, _mask: i32, _flags: i32) -> i32 {
        -1
    }
    fn camera_f(&self, _field: i32) -> f32 {
        0.0
    }
    fn lab_anim_end(&self, _slot: i32) -> f32 {
        0.0
    }
    fn lab_attr_count(&self) -> i32 {
        0
    }
    fn lab_attr_name(&self, _i: i32) -> Option<String> {
        None
    }
    fn lab_attr_f(&self, _slot: i32, _i: i32) -> f32 {
        0.0
    }
    // ---- names the native half asks the registries for
    /// `gw_Geno_DefineName(ck)`: a Geno-defined fighter's name.
    fn geno_define_name(&self, _ck: i32) -> Option<String> {
        None
    }
    /// `gw_Mex_FighterName(gw_Mex_PortCKindToExt(ck))`
    fn mex_fighter_name(&self, _ck: i32) -> Option<String> {
        None
    }
    fn geno_profile_for_kind(&self, _kind: i32) -> i32 {
        -1
    }
    fn geno_state_name(&self, _profile: i32, _n: i32) -> Option<String> {
        None
    }
    // ---- virtual pads of CPU fighter slots 5-6 (ScriptGame_VirtualPadField / Seen / Release)
    fn virtual_pad_field(&mut self, _slot: i32, _k: i32, _v: i32, _set: bool) -> i32 {
        0
    }
    fn virtual_pad_seen(&self, _slot: i32, _k: i32) -> i32 {
        0
    }
    fn virtual_pad_release(&mut self, _slot: i32, _owner: i32) {}
    // ---- the script camera (gw_Camera_Script*)
    fn camera_script_get_bits(&self, _field: i32) -> i32 {
        0
    }
    fn camera_script_set(&mut self, _field: i32, _bits: i32) {}
    /// 0 attached, 1 detached, 2 attaching
    fn camera_script_state(&self) -> i32 {
        0
    }
    fn camera_script_mode(&self) -> i32 {
        0
    }
    fn camera_script_completion(&self) -> i32 {
        0
    }
    /// 1 move, 2 path
    fn camera_script_completion_kind(&self) -> i32 {
        0
    }
    fn camera_script_detach(&mut self) -> bool {
        false
    }
    fn camera_script_attach(&mut self, _frames: i32) {}
    fn camera_script_reset(&mut self) {}
    fn camera_script_commit(&mut self) {}
    fn camera_script_move_begin(&mut self) {}
    fn camera_script_move(&mut self, _frames: i32, _ease: i32) {}
    fn camera_script_path_clear(&mut self) {}
    fn camera_script_path_key(&mut self, _frame: i32) -> bool {
        false
    }
    fn camera_script_path_start(&mut self) {}
    fn camera_script_follow(&mut self, _kind: i32, _id: i32, _point: [i32; 3], _offset: [i32; 3]) -> bool {
        false
    }
    fn camera_script_shake(&mut self, _intensity_bits: i32, _frames: i32) {}
    fn camera_script_lift_bounds(&mut self, _lift: bool) {}
    // ---- fighter writes, CPU, debug movement
    fn set_stocks(&mut self, _slot: i32, _n: i32) {}
    fn impulse(&mut self, _slot: i32, _x_bits: i32, _y_bits: i32) -> bool {
        false
    }
    fn hit(&mut self, _slot: i32, _from: i32, _damage: i32, _angle: i32, _kbg: i32, _bkb: i32) -> bool {
        false
    }
    /// fight: 0 stand, 1 fight, 2 default
    fn cpu_mode(&mut self, _slot: i32, _fight: i32) -> bool {
        false
    }
    fn cpu_script_mode(&mut self, _slot: i32) -> bool {
        false
    }
    fn cpu_mode_info(&self, _slot: i32) -> i32 {
        0
    }
    fn geno_fly_get(&self, _slot: i32) -> i32 {
        -1
    }
    fn geno_fly_set(&mut self, _slot: i32, _mode: i32) -> i32 {
        -1
    }
    fn geno_fly_teleport(&mut self, _slot: i32, _x_bits: i32, _y_bits: i32) -> i32 {
        -1
    }
    fn geno_fly_set_speed(&mut self, _bits: i32) {}
    fn geno_fly_speed(&self) -> f32 {
        0.0
    }
    fn geno_fly_set_solid(&mut self, _solid: bool) {}
    fn geno_fly_solid(&self) -> bool {
        false
    }
    fn geno_fly_hold_hitbox(&mut self, _slot: i32, _on: bool, _action: i32, _frame: i32) -> i32 {
        -1
    }
    fn geno_fly_holding(&self, _slot: i32) -> bool {
        false
    }
    fn geno_fly_hold_hits(&self, _slot: i32) -> i32 {
        0
    }
    fn play_sound_options(&mut self, _id: i32, _volume: i32, _pitch: i32) -> i32 {
        0
    }
    // ---- action states, animation rows, script words (the LAB's browsers)
    fn lab_common_count(&self, _slot: i32) -> i32 {
        0
    }
    /// the animation index of a motion state's row, -1 none, < -1 invalid
    fn lab_motion_anim(&self, _slot: i32, _motion: i32) -> i32 {
        -2
    }
    fn lab_anim_symbol_for(&self, _slot: i32, _anim: i32) -> Option<String> {
        None
    }
    fn lab_row_count(&self, _slot: i32) -> i32 {
        0
    }
    fn lab_set_preview_row(&mut self, _slot: i32, _row: i32, _facing: i32) {}
    /// guest address of an animation's subaction script
    fn lab_script(&self, _slot: i32, _anim: i32) -> Option<u32> {
        None
    }
    fn lab_set_motion(&mut self, _slot: i32, _motion: i32, _rate_bits: i32, _lift_bits: i32) {}
    fn guest_ok(&self, _addr: u32, _len: u32) -> bool {
        false
    }
    fn guest_word(&self, _addr: u32) -> u32 {
        0
    }
    fn geno_script_original(&self, addr: u32) -> u32 {
        addr
    }
    fn mex_move_logic_entries(&self, _kind: i32) -> i32 {
        0
    }
    fn geno_state_count(&self, _profile: i32) -> i32 {
        0
    }
    // ---- snapshots (gw_snap.c)
    fn stage_slots_active(&self) -> bool {
        false
    }
    fn onep_state_active(&self) -> bool {
        false
    }
    /// `gw_snap_reserve(slots)`: the number of slots now available
    fn snap_reserve(&mut self, _slots: i32) -> i32 {
        0
    }
    fn snap_save_index(&mut self, _slot: i32, _tag: i32) {}
    /// 0 on success
    fn snap_load_index(&mut self, _slot: i32) -> i32 {
        -1
    }
    // ---- LAB mode, constants and edits
    fn geno_lab_mode_active(&self) -> bool {
        false
    }
    /// `gw_GenoLab_Leave(where)`: 0 css, 1 sss, 2 menu, 3 restart. False when refused.
    fn geno_lab_leave(&mut self, _where: i32) -> bool {
        false
    }
    fn lab_common_f(&self, _which: i32) -> f32 {
        0.0
    }
    fn lab_floor_y(&self, _x: f32, _y: f32, _depth: f32) -> f32 {
        -1.0e9
    }
    fn lab_set_shield(&mut self, _slot: i32, _health: f32) {}
    fn lab_set_scale(&mut self, _slot: i32, _scale: f32) {}
    // ---- rollback visualiser (gw_RbViz_*)
    fn rbviz_total(&self) -> i32 {
        0
    }
    fn rbviz_get(&self, _back: i32) -> Option<crate::gd::labmisc::RbRow> {
        None
    }
    fn rbviz_first_mismatch(&self) -> Option<crate::gd::labmisc::RbMismatch> {
        None
    }
    fn rbviz_reset(&mut self) {}
    // ---- scene
    fn stage_kind(&self) -> i32 {
        0
    }
    fn stage_external(&self) -> i32 {
        0
    }
    fn game_mode(&self) -> i32 {
        0
    }
    fn rng_seed(&self) -> u32 {
        0
    }
    fn set_percent(&mut self, _slot: i32, _percent: i32) {}

    // ---- the long rewind's keyframe store (gw_snap.c `gw_rw_*`); `user` is the script bookkeeping saved beside a keyframe
    fn rw_begin(&mut self, _tag: i32, _user: &[u8]) -> i32 {
        -1
    }
    fn rw_save(&mut self, _tag: i32, _user: &[u8]) -> i32 {
        -1
    }
    /// `gw_rw_load(k)`: the keyframe `k` (a tag from `rw_find_le`), its `user` bytes back
    fn rw_load(&mut self, _k: i32) -> Option<Vec<u8>> {
        None
    }
    fn rw_drop_from(&mut self, _tag: i32) -> i32 {
        0
    }
    fn rw_trim(&mut self, _limit: i32) {}
    fn rw_end(&mut self) {}
    /// the newest keyframe at or before `tag`
    fn rw_find_le(&self, _tag: i32) -> Option<i32> {
        None
    }
    fn rw_base_tag(&self) -> i32 {
        0
    }
    fn rw_latest(&self) -> i32 {
        0
    }
    fn rw_count(&self) -> i32 {
        0
    }
    fn rw_bytes(&self) -> f64 {
        0.0
    }
    fn rw_delta_bytes(&self) -> f64 {
        0.0
    }
    fn rw_ms_last_load(&self) -> f64 {
        0.0
    }
    fn rw_ms_save_avg(&self) -> f64 {
        0.0
    }
    fn rw_last_load_pages(&self) -> u32 {
        0
    }
    /// `gw_rw_capture`: copy the whole state for the exactness self-test (0 on success)
    fn rw_capture(&mut self) -> i32 {
        -1
    }
    /// `gw_rw_compare`: ([diff incl. render-owned, diff of the simulation], [hash then, hash now]); frees the copy
    fn rw_compare(&mut self) -> Option<([f64; 2], [u64; 2])> {
        None
    }
    fn rw_measure(&mut self, _on: bool) {}
    fn snap_slot_bytes(&self) -> u64 {
        0
    }
    /// `gw_Snap_LabResim`: a silent re-simulation frame is running
    fn snap_lab_resim(&mut self, _on: bool) {}
    // ---- the persistent state library (`gw_snap_file_*`); `hdr` is the script's fixed-size header
    fn snap_file_save(&mut self, _path: &str, _hdr: &[u8]) -> i32 {
        -1
    }
    fn snap_file_header(&self, _path: &str, _len: usize) -> Option<Vec<u8>> {
        None
    }
    fn snap_file_rewrite_header(&mut self, _path: &str, _hdr: &[u8]) -> i32 {
        -1
    }
    /// 0 ok, -2 another memory layout, -3 damaged, other: unreadable
    fn snap_file_load(&mut self, _path: &str) -> i32 {
        -1
    }
    /// `gs_identity`: exe hash, disc hash, mods hash, Geno data hash
    fn identity(&self) -> [u64; 4] {
        [0; 4]
    }
    /// `gw_Geno_Reload`: <0 refused (msg says why), 0 the layout changed (restart), >0 re-apply
    fn geno_reload(&mut self, _msg: &mut String) -> i32 {
        -1
    }
    /// `gw_GenoGame_LabReload`: fighters re-applied
    fn geno_lab_reload(&mut self) -> i32 {
        0
    }
    /// `gw_ScriptGame_ModeActive`: a Lua mode director runs the match
    fn mode_active(&self) -> bool {
        false
    }
    /// `gw_ScriptGame_LabKnockback`
    #[allow(clippy::too_many_arguments)]
    fn lab_knockback(&self, _slot: i32, _attacker: i32, _dmg_bits: i32, _kbg: i32, _wbk: i32, _bkb: i32, _pct_bits: i32, _post: i32) -> f32 {
        0.0
    }
    fn lab_kb_level(&self, _kb_bits: i32) -> i32 {
        0
    }
    /// `gw_ScriptGame_LabFlightF`: 0 gravity, 1 terminal, 2 friction, 3 facing, 4 air, 5 weight, 6 percent
    fn lab_flight_f(&self, _slot: i32, _which: i32) -> f32 {
        0.0
    }
}

/// No game: nothing is present (the C build before a scene).
pub struct NoGame;
impl ScriptGame for NoGame {}

/// `gs_players_present`
pub fn players_present(g: &dyn ScriptGame, slot: usize) -> bool {
    g.fighter_i(slot as i32, SI_PRESENT) == 1
}
