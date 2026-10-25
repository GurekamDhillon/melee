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

    // ---- the Geno combo graph (gw_GenoGame_Combo* / gw_Geno_Combo*): `docs/geno.md` section 25
    /// `gw_GenoGame_ComboInfo(port, what)`: what 3 = route count (negative: the fighter has no combo graph), 11 = profile,
    /// 0 links, 1 ki, 2 last route, 4 hit, 5 blocked, 6 taken, 7 max links, 8 ki max, 9 action frame, 16+r = route state.
    fn combo_info(&self, _slot: i32, _what: i32) -> i32 {
        -1
    }
    /// `gw_GenoGame_ComboSetKi(port, hundredths)`: the new value, negative without a combo graph.
    fn combo_set_ki(&mut self, _slot: i32, _ki: i32) -> i32 {
        -1
    }
    /// `gw_Geno_ComboWord(profile, route, GENO_CR_*)`
    fn combo_word(&self, _profile: i32, _route: i32, _word: i32) -> i32 {
        0
    }
    fn combo_name(&self, _profile: i32, _route: i32) -> String {
        String::new()
    }
    fn combo_state_name(&self, _profile: i32, _n: i32) -> String {
        String::new()
    }
    // ---- arena-hooks / largemap bounds (`gw_ScriptGame_Arena*`, `MapBound(s)`, `StageBounds`): pc/platform/gw_script_arena.inc.
    // All mutable arena state (including ownership) lives in the game's snapshotted state; floats cross as bit patterns.
    fn arena_owner(&self) -> i32 {
        0
    }
    fn arena_origin(&mut self, _owner: i32, _x_bits: i32, _y_bits: i32, _frames: i32) {}
    fn arena_spawn(&mut self, _owner: i32, _slot: i32, _x_bits: i32, _y_bits: i32) -> i32 {
        0
    }
    /// axis 0 x, 1 y, 2 z; `fallback` is returned when the slot has no position
    fn arena_spawn_pos(&self, _slot: i32, _axis: i32, fallback: f32) -> f32 {
        fallback
    }
    fn arena_bounds(&mut self, _owner: i32, _blast: bool, _l: i32, _r: i32, _t: i32, _b: i32) {}
    fn arena_restore(&mut self) {}
    /// 0-3 camera l/r/t/b, 4-7 blast l/r/t/b (float bits), 8/9 origin x/y (bits), 10 transition frames left
    fn arena_read(&self, _field: i32) -> i32 {
        0
    }
    /// id -1 field 0: group count; field 0: enabled; field 1: owner
    fn arena_group(&self, _id: i32, _field: i32) -> i32 {
        0
    }
    fn arena_group_set(&mut self, _owner: i32, _id: i32, _enabled: bool) {}
    fn arena_release(&mut self, _owner: i32) {}
    /// `gw_ScriptGame_StageBounds`: blast(0-3), camera(4-7), main_floor(8-11) as float bits; 12 has main floor; 13 surface_top
    fn stage_bounds_i(&self, _field: i32) -> i32 {
        0
    }
    /// `gw_ScriptGame_MapBounds(which, l, r, b, t)`; which -1 restores the retail bounds
    fn map_bounds(&mut self, _which: i32, _l: i32, _r: i32, _b: i32, _t: i32) {}
    /// which 0 camera, 1 blast; field 0 left, 1 right, 2 bottom, 3 top (bits)
    fn map_bound(&self, _which: i32, _field: i32) -> i32 {
        0
    }
    // ---- scalar fighter capabilities (`gw_ScriptGame_Caps*`, `IntrWin*`, `Armor*`): pc/platform/gw_script_fighter_caps*.inc.
    // Records live in the game's snapshotted state; floats cross as bit patterns.
    fn caps_movement(&mut self, _entity: i32, _owner: i32, _jumps: i32, _flags: i32) -> i32 {
        0
    }
    /// field 0 present, 1 air jumps, 2 flag bits, 3 retail jumps (negative: none)
    fn caps_movement_read(&self, _entity: i32, field: i32) -> i32 {
        if field == 3 { -1 } else { 0 }
    }
    fn caps_effect(&mut self, _entity: i32, _owner: i32, _kind: i32, _value_bits: i32, _frames: i32) -> i32 {
        0
    }
    /// field 0 frames left, 1 value bits
    fn caps_effect_read(&self, _entity: i32, _kind: i32, _field: i32) -> i32 {
        0
    }
    fn caps_armour(&mut self, _entity: i32, _owner: i32, _damage_bits: i32, _kb_bits: i32) -> i32 {
        0
    }
    /// field 0 present, 1 damage bits, 2 knockback bits
    fn caps_armour_read(&self, _entity: i32, _field: i32) -> i32 {
        0
    }
    /// > 0 given; otherwise the refusal reason code
    fn caps_give_item(&mut self, _entity: i32, _owner: i32, _kind: i32) -> i32 {
        0
    }
    fn caps_give_item_reason(&self, _reason: i32) -> String {
        "no game".into()
    }
    /// nth = 0: the nearest; n >= 1: the nth within `radius_bits`; 0 = none
    fn caps_opponent(&self, _entity: i32, _radius_bits: i32, _nth: i32, _exclude: i32) -> i32 {
        0
    }
    fn caps_status(&mut self, _entity: i32, _channel: i32, _owner: i32, _value_bits: i32, _frames: i32) -> i32 {
        0
    }
    fn caps_status_read(&self, _entity: i32, _channel: i32, _field: i32) -> i32 {
        0
    }
    fn intr_win_set(&mut self, _entity: i32, _owner: i32, _frames: i32, _exits: i32, _flags: i32) -> i32 {
        0
    }
    /// 0 frames, 1 owner, 2 exit mask, 3 key, 4 opened, 5 cancels
    fn intr_win_read(&self, _entity: i32, _field: i32) -> i32 {
        0
    }
    #[allow(clippy::too_many_arguments)]
    fn armor_set(&mut self, _e: i32, _owner: i32, _ty: i32, _value_bits: i32, _frames: i32, _state: i32, _from: i32, _to: i32, _dir: i32) -> i32 {
        0
    }
    /// field 0 present, 1 value, 2 remaining, 3 frames, 4 state, 5 from, 6 to, 7 direction, 8 enabled
    fn armor_read(&self, _e: i32, _ty: i32, _field: i32) -> i32 {
        0
    }
    fn armor_clear(&mut self, _e: i32, _owner: i32, _ty: i32) -> i32 {
        0
    }
    /// `gs_caps_release` + `ArmorRelease`: drop everything the owner (script index + 1) set
    fn caps_release(&mut self, _owner: i32) {}
    // ---- crits (`gw_ScriptGame_Crit*`) and CPU overlays (`CpuTechnical*`, `CpuAssist*`)
    /// field 0 owner, 1 tag mask, 2 min percent bits, 3 force, 4 hits, 5 crits, 6 max multiplier bits, 10+4*slot..13+4*slot chance/mult/max/launch bits
    fn crit_read(&self, _entity: i32, _field: i32) -> i32 {
        0
    }
    #[allow(clippy::too_many_arguments)]
    fn crit_set(&mut self, _e: i32, _owner: i32, _slot: i32, _chance: i32, _mult: i32, _mult_max: i32, _launch: i32, _min_pct: i32, _force: i32, _flags: i32) -> i32 {
        0
    }
    fn crit_seed(&mut self, _seed: i32) {}
    fn cpu_technical_state(&self, _slot: i32, _owner: i32, _field: i32) -> i32 {
        0
    }
    fn cpu_technical(&mut self, _slot: i32, _owner: i32, _skill: i32, _seed: i32) -> i32 {
        0
    }
    /// field 0 enabled, 1-5 permille probabilities, 6 tech dir, 7 seed, 8-12 opportunities, 13-17 performed
    fn cpu_assist_state(&self, _slot: i32, _owner: i32, _field: i32) -> i32 {
        0
    }
    fn cpu_assist_clear_slot(&mut self, _slot: i32, _owner: i32) -> i32 {
        0
    }
    #[allow(clippy::too_many_arguments)]
    fn cpu_assist(&mut self, _slot: i32, _owner: i32, _lc: i32, _ps: i32, _tech: i32, _tdir: i32, _wd: i32, _ff: i32, _seed: i32) -> i32 {
        0
    }
    // ---- fighter reserve/bench (`gw_ScriptGame_FighterBench*`, `FighterCall`, `FighterBenched`, `FighterReserveRead`)
    /// 0 ok; otherwise the refusal code (1-11) as gs_reserve_result's reasons; `calling` 1 for fighter_call
    fn fighter_bench_check(&self, _slot: i32, _owner: i32, _calling: i32) -> i32 {
        1
    }
    fn fighter_bench(&mut self, _slot: i32, _owner: i32) -> i32 {
        1
    }
    fn fighter_call(&mut self, _slot: i32, _owner: i32, _x: i32, _y: i32, _facing: i32, _intangible: i32) -> i32 {
        1
    }
    fn fighter_benched(&self, _slot: i32) -> i32 {
        0
    }
    /// field 0 present, 1 kind, 2 action, 3 airborne, 4/5 x/y bits, 6 cpu kind, 7.. flags, 13 refusal code
    fn fighter_reserve_read(&self, _slot: i32, _entity: i32, _field: i32) -> i32 {
        0
    }
    /// `gw_ScriptMods_GameEvent`: the online Envoy native triggered rules see every event, even a resimulated frame's
    fn mods_game_event(&mut self, _what: i32, _a: i32, _b: i32, _c: i32, _d: i32) {}
    /// `gw_ScriptGame_ActionCategory`: 1 jump .. 8 perfect shield for the motion, 0 none
    fn action_category(&self, _motion: i32) -> i32 {
        0
    }
    // ---- zones (`gw_ScriptGame_Zone*`): pc/platform/gw_script_zones.inc over `zones_core::SzState` in the game's snapshotted state
    fn zone_field(&self, _slot: i32, _field: i32) -> i32 {
        0
    }
    fn zone_text(&self, _slot: i32, _text: i32, _byte: i32) -> i32 {
        0
    }
    /// field -1 clears the draft
    fn zone_draft(&mut self, _field: i32, _value: i32) {}
    fn zone_draft_text(&mut self, _text: i32, _byte: i32, _value: i32) {}
    /// handle 0 adds; the new/updated handle, or <0 refused
    fn zone_commit(&mut self, _handle: i32, _owner: i32, _serial_floor: i32) -> i32 {
        -1
    }
    fn zone_remove(&mut self, _handle: i32, _owner: i32) -> i32 {
        0
    }
    /// owner 0: every zone
    fn zones_release(&mut self, _owner: i32) {}
    /// bit k: the hook of kind k+1 (enter, exit, none, some) has a listener
    fn zones_arm(&mut self, _mask: i32) {}
    /// the frame step; `terminal` 1 retires every zone, 2 the owner's removed ones, 0 a normal frame
    fn zones_frame(&mut self, _terminal: i32) {}
    /// field 0 entity id, 1/2 x/y bits (zone ignored); 3 inside zone slot `zone`; 4 frames inside slot `zone`
    fn zone_entity(&self, _entity: i32, _field: i32, _zone: i32) -> i32 {
        0
    }
    fn zone_event(&self, _event: i32, _field: i32) -> i32 {
        0
    }
    fn zone_event_text(&self, _event: i32, _text: i32, _byte: i32) -> i32 {
        0
    }
    fn zone_event_count(&self) -> i32 {
        0
    }
    // ---- skill ring, shock, objectives (`gw_ScriptGame_Skill*`, `Shock*`): gw_script_skill.inc, gw_script_shock.inc
    fn skill_row_count(&self) -> i32 {
        0
    }
    /// ring row idx (0 newest): field 0 frame, 1 kind, 2 entity, 3.. i1 i2 i3 f1 f2 (bits)
    fn skill_row(&self, _idx: i32, _field: i32) -> i32 {
        0
    }
    /// 0 combo_count, 1 combo_damage (bits), 2 combo_attacker, 3 air_frames, 4 frames_in_state, 5 aerial_hit
    fn skill_state(&self, _entity: i32, _field: i32) -> i32 {
        0
    }
    /// 0 wave_min_speed (bits), 1 wavedash max airborne, 2 ledge dash, 3 dash dance, 4 lcancel window, 5 powershield window
    fn skill_const(&self, _idx: i32) -> i32 {
        0
    }
    #[allow(clippy::too_many_arguments)]
    fn shock_set(&mut self, _entity: i32, _owner: i32, _frames: i32, _charges: i32, _mult_bits: i32, _bonus: i32, _flags: i32) -> i32 {
        0
    }
    /// 0 owner, 1 frames, 2 charges, 3 bonus, 4 multiplier (bits), 5 hits, 6 applied
    fn shock_read(&self, _entity: i32, _field: i32) -> i32 {
        0
    }
    // ---- contacts (`gw_ScriptGame_Contact*`): pc/platform/gw_script_contacts.inc
    /// surface 0 floor, 1 left wall, 2 right wall, 3 ceiling, 4 ledge; field SC_LINE (-1 none), OWNER, PART, KIND, PASS,
    /// X0, Y0, X1, Y1, X, Y, NX, NY (float bits); `line` -1 the current contact, otherwise a floor line index
    fn contact_field(&self, _slot: i32, _surface: i32, _field: i32, _line: i32) -> i32 {
        -1
    }
    /// bit 0 jump, 1 pass-through, 2 knocked, 3 dead, 4 rebirth
    fn contact_flags(&self, _slot: i32) -> i32 {
        0
    }
    /// the next floor line index the fighter crossed upward after `after`, or -1
    fn contact_cross(&self, _slot: i32, _after: i32) -> i32 {
        -1
    }
    // ---- fly cursor attack (`gw_GenoFly_*`) and stage camera parameters (`gw_ScriptGame_CameraParam*`)
    fn geno_fly_target(&mut self, _slot: i32, _x_bits: i32, _y_bits: i32) -> i32 {
        -1
    }
    #[allow(clippy::too_many_arguments)]
    fn geno_fly_attack_config(&mut self, _slot: i32, _flags: i32, _damage: i32, _radius_bits: i32, _active_gap: i32, _kb_kbg: i32, _angle: i32) -> i32 {
        -1
    }
    /// 0 attacking, 1 targeting, 2 damage, 3 radius (bits), 4/5 x/y (bits), 6 pulses, 7 every_frame, 8 active, 9 gap, 10 kb, 11 kbg, 12 angle, 13 phase
    fn geno_fly_state(&self, _slot: i32, _field: i32) -> i32 {
        0
    }
    fn geno_fly_clear(&mut self, _slot: i32) {}
    fn camera_param_read(&self, _field: i32) -> i32 {
        0
    }
    fn camera_param_owner(&self) -> i32 {
        0
    }
    /// `bits` are the ten parameters in the order min_dist .. pitch_gain; mask says which the script set
    fn camera_params(&mut self, _owner: i32, _mask: i32, _bits: &[i32]) -> i32 {
        0
    }
    // ---- echoes and the fighter history (`gw_ScriptGame_Echo*`, `FighterHistory*`): pc/platform/gw_script_echo.inc
    fn fighter_history_depth(&self) -> i32 {
        0
    }
    /// hitbox -1: the fighter row (0 present, 1 generation, 2 action state, 3 move, 4 airborne, 5.. action_frame x y z facing,
    /// 10 hitbox count, 11 attack id); hitbox 0..4: 0 present, 1 identity, 2 damage ... see gd::echo
    fn fighter_history_read(&self, _entity: i32, _age: i32, _hitbox: i32, _field: i32) -> i32 {
        0
    }
    /// `rest` are delay, move, element, airborne, damage bits, knockback bits, new element, once_per_move; handle 0 allocates
    fn echo_add_with_handle(&mut self, _entity: i32, _owner: i32, _rest: &[i32], _handle: i32) -> i32 {
        0
    }
    fn echo_remove(&mut self, _handle: i32, _owner: i32) -> i32 {
        0
    }
    fn echo_read(&self, _entity: i32, _index: i32, _field: i32) -> i32 {
        0
    }
    fn echo_owner(&self, _entity: i32) -> i32 {
        0
    }
    fn echo_next_handle(&self) -> i32 {
        1
    }
    /// replace the entity's whole rule table (count rows of 9 ints, flattened to 8 rows); nonzero when accepted
    fn echo_replace(&mut self, _entity: i32, _owner: i32, _count: i32, _rules: &[i32]) -> i32 {
        0
    }
    /// field 0 armed, 1 element, 2 flash frames
    fn echo_state(&self, _entity: i32, _handle: i32, _field: i32) -> i32 {
        0
    }
    // ---- scripted CPU controller (`gw_ScriptGame_CpuCtl`, `CpuAttr`, `CpuScriptMode`): pc/platform/gw_script_cpu_ctl.inc
    /// setters, op GCC_* 1..23 (PAD_COMMIT, SCRIPT_BEGIN/BYTE/RUN, DEST_X/Y, TARGET, CANCEL, STAGE+k, GOTO_*); `sub` selects Nana
    fn cpu_ctl(&mut self, _slot: i32, _sub: i32, _op: i32, _a: i32) -> i32 {
        0
    }
    /// getters, op GCC_G_* 100..118; -1 (ACTIVE, GOTO_STATE) when there is no such CPU fighter
    fn cpu_ctl_read(&self, _slot: i32, _sub: i32, _op: i32, _a: i32) -> i32 {
        -1
    }
    fn cpu_attr(&self, _slot: i32, _id: i32) -> i32 {
        0
    }
    // ---- part overrides and measurements (`gw_ScriptGame_Parts*`, script_parts.h) and raw memory peeks
    /// field PART_I_*: REFRESH(0) rebuilds and returns the count, COUNT(1) the cached count (<0 none)
    fn parts_i(&self, _slot: i32, _index: i32, _field: i32) -> i32 {
        -1
    }
    fn parts_f(&self, _slot: i32, _index: i32, _field: i32) -> f32 {
        0.0
    }
    /// ctl PART_CTL_*; <0 refused
    fn parts_control(&mut self, _slot: i32, _part: i32, _ctl: i32, _value: i32, _owner: i32) -> i32 {
        -1
    }
    /// the address of the slot's joint JObj, for effect anchors
    fn parts_joint(&self, _slot: i32, _joint: i32) -> i32 {
        0
    }
    fn parts_reset(&mut self, _owner: i32) {}
    /// `gd.lab_peek`: n raw bytes of MEM1 at a 0x80000000-based address, None out of range
    fn mem1_read(&self, _addr: u32, _n: usize) -> Option<Vec<u8>> {
        None
    }
    // ---- entity identity (`gw_ScriptGame_EntityField`, `EntityEpoch`, `EntityAddressField`): pc/platform/gw_script_spine_refs.inc
    /// kind 1 fighter (id = port*2+sub), 2 item, 3 echo, 4 stage object, 5 stage actor. field 0 live, 1 epoch, 2 generation,
    /// 3 definition, 4 owner fighter id + 1, 5 role bits
    fn entity_field(&self, _kind: i32, _id: i32, _field: i32) -> i32 {
        0
    }
    fn entity_epoch(&self) -> i32 {
        0
    }
    /// field 0 live, 1 epoch, 2 generation, 3 definition, 6 id of the entity at a native address
    fn entity_address_field(&self, _kind: i32, _address: i32, _field: i32) -> i32 {
        0
    }
    /// `gw_Snap_Resimulating`: the rollback is replaying a frame (no hooks, no queued events)
    fn resimulating(&self) -> bool {
        false
    }
    // ---- hit rules / fighter status (`gw_ScriptGame_HitRules*`, `FighterStatus`, `HitRuleRead`): pc/platform/gw_script_hit_rules.inc.
    // `entity` is port*2 (+1 for the independent partner). A rule is 16 ints (floats as bits); field 12 is the stable handle.
    fn hit_rules_owner(&self, _entity: i32) -> i32 {
        0
    }
    fn hit_rules_count(&self, _entity: i32) -> i32 {
        0
    }
    fn fighter_status(&self, _entity: i32) -> i32 {
        0
    }
    fn hit_rule_read(&self, _entity: i32, _index: i32, _field: i32) -> i32 {
        0
    }
    /// `rules` is count rows of 16 ints, flattened (32 rows); returns nonzero when accepted
    fn hit_rules_replace(&mut self, _entity: i32, _owner: i32, _count: i32, _status: i32, _rules: &[i32]) -> i32 {
        0
    }
    /// a script unloaded (owner = script index + 1, 0 = all): drop every hit rule, echo, enemy, item ... it holds
    fn owner_release(&mut self, _owner: i32) {}
    // ---- the recorder (`pc/platform/gw_script_rec.inc`): `.gdlp` live-playback recordings. The Lua half
    // (gating, names, argument shapes) is ported in `gd::rec`; these are the recorder's state machine and files.
    fn rec_start(&mut self, _name: &str) -> Result<String, String> {
        Err("recording is not available".into())
    }
    /// frames written
    fn rec_stop(&mut self) -> Result<i32, String> {
        Err("not recording".into())
    }
    fn rec_mark(&mut self, _label: &str) -> Result<i32, String> {
        Err("not recording".into())
    }
    fn rec_play(&mut self, _name: &str, _map: &str, _looping: bool) -> Result<(), String> {
        Err("recording is not available".into())
    }
    fn rec_play_stop(&mut self) -> Result<(), String> {
        Err("not playing".into())
    }
    fn rec_trim(&mut self, _name: &str, _from: i32, _to: i32, _dst: &str) -> Result<String, String> {
        Err("recording is not available".into())
    }
    /// `name` None: the last recording made this session. Result: the new recording's name.
    fn rec_keep_last(&mut self, _seconds: f64, _name: Option<&str>) -> Result<String, String> {
        Err("no recording named (and none made this session)".into())
    }
    fn rec_save_history(&mut self, _name: &str, _seconds: f64) -> Result<String, String> {
        Err("recording is not available".into())
    }
    /// (path, frames)
    fn rec_export_slp(&mut self, _name: &str, _out: &str) -> Result<(String, i32), String> {
        Err("recording is not available".into())
    }
    fn rec_status(&self) -> RecStatus {
        RecStatus::default()
    }
    fn rec_list(&self) -> Vec<RecRow> {
        Vec::new()
    }
}

/// `gd.rec_status()`: `mode` is "idle" / "arming" / "recording" / "loading" / "playing" / "working".
#[derive(Clone, Debug)]
pub struct RecStatus {
    pub mode: &'static str,
    pub name: String,
    pub last: String,
    pub frames: i32,
    pub frame: i32,
    pub looping: bool,
    pub base: i32,
}
impl Default for RecStatus {
    fn default() -> Self {
        RecStatus { mode: "idle", name: String::new(), last: String::new(), frames: 0, frame: 0, looping: false, base: 0 }
    }
}

/// One row of `gd.rec_list()`. `read` None: the file could not be read (only `name`, `bytes`, `ok = false` and `why` show).
#[derive(Clone, Debug, Default)]
pub struct RecRow {
    pub name: String,
    pub bytes: i64,
    pub read: Option<RecRead>,
    pub ok: bool,
    pub why: String,
}
#[derive(Clone, Debug, Default)]
pub struct RecRead {
    pub frames: i32,
    pub marks: i32,
    pub complete: bool,
    pub retail: bool,
    pub format: i32,
    pub whole: bool,
    pub embedded: bool,
    pub exportable: bool,
    pub fighters: String,
}

/// No game: nothing is present (the C build before a scene).
pub struct NoGame;
impl ScriptGame for NoGame {}

/// `gs_players_present`
pub fn players_present(g: &dyn ScriptGame, slot: usize) -> bool {
    g.fighter_i(slot as i32, SI_PRESENT) == 1
}
