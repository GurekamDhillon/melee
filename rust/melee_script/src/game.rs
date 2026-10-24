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
}

/// No game: nothing is present (the C build before a scene).
pub struct NoGame;
impl ScriptGame for NoGame {}

/// `gs_players_present`
pub fn players_present(g: &dyn ScriptGame, slot: usize) -> bool {
    g.fighter_i(slot as i32, SI_PRESENT) == 1
}
