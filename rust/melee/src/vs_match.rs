//! A native VS match, as far as the ported code reaches: the first golden scenario
//! (`pad_falcon_fox_fd`, docs/rust-port/golden/README.md).
//!
//! What runs natively, frame by frame (golden frame 0 = the first logged match frame):
//! - Disc: `GrNLa.dat` (spawn points, `stage::spawn_points`) and `PlCo.dat` (`ftLoadCommonData`
//!   +0x260 start shield) from the user's ISO.
//! - Match start per player slot `i` (C order and sources cited inline): Fighter defaults
//!   (fighter.c ~560 shield, ~839 x67F/x680), `getSpawnPoint` -> `Stage_80224E64(i)` ->
//!   `Ground_801C2D24(i)`, `direction(x)` (gmvs.c), stocks into the ported player table
//!   (`melee_fighter::stale::set_stocks`), airborne via the ported `ftCommon_8007D5D4`, then
//!   `ftCo_Entry_Enter` (ft_0C31.c: motion Entry, timer = `Player_GetUnk4C`, which gmvs.c ~1741
//!   sets to 5, 10, ... per entering player).
//! - Before golden frame 0 the C draws 10 times (MELEE_RAND_TRACE, in order): the boot's fresh
//!   save (`gmMainLib_8015F600` -> `Toy_80305058`, one `HSD_Randi`), FD's background gobj init
//!   (`grLast_8021AC30`, 4), each fighter's CPU init (`Fighter_UnkProcessDeath_80068354` ->
//!   `ftCo_800A101C`, 2 each) and the music pick (`Stage_80225074` -> `Ground_801C24F8`, 1).
//! - Each frame, in GObj proc order: `ftCo_Entry_Anim` (timer decrement); the ground anim proc
//!   (`Ground_801C1CD0`: the stage JObj animations; only their particle tracks, FObj 0x28, are
//!   run: map gobj 4's anim 0 has one at frame 0, the FD background emitter `efLib_Cb_DPtcl` ->
//!   `grLib_801C99C0` -> `hsd_8039EFAC(0, 0x1E, 30000, jobj)`); FD's sway gobj
//!   (`grLast_8021ADD0`); then efLib's two particle procs (`hsd_8039CEAC` + `hsd_8039EE24`, masks
//!   0x60000 and 0x90000), the ported particle system (melee_runtime::particle).
//! - Then the entry sequence (`VsMatch::fighters`): Entry -> `ftCo_800C6408` EntryStart (+ the
//!   entry beam effect, efLib, whose particle tracks spawn bank-0 generators) -> `ftCo_800C6B6C`
//!   EntryEnd -> `ftCo_Fall_Enter` Fall. The run stops when a falling fighter reaches the floor:
//!   environment collision (mpColl) and the bone-driven ECB are not ported. Frame 64 is 1 ulp off
//!   in y for that reason (RUST_PORT.md, sim round 1).
//! - Approximations (none read by the frame hash): the emitter's JObj matrix is the joint's bind
//!   pose (the stage JObjs are not instantiated); the CPU init's floor probe is skipped (x54/x64);
//!   `Toy_80305058`'s trophy choice is not computed (its draw is); the background map state machine
//!   (`grLast_8021B2E8`/`8021B5C4`) is not run (its first change needs 1800+ frames).

#[path = "input.rs"]
pub mod input;

mod attackair;
mod cpu;
mod dashrun;
mod damage;
mod fox;
mod greens;
mod hit;
mod story;
mod rebirth;
mod samus;
mod sheik;
use fox::{ItemCmd, FoxVars, SimItem};
use rebirth::{PlayerRespawn, RespawnCtx};

use crate::fighter_data::skeleton::{FigaTree, Skeleton};
use crate::fighter_data::{AnimRow, EcbDesc};
use crate::{FrameWords, Sim};
use melee_fighter::ftcommon as fc;
use melee_fighter::stale;
use melee_runtime::aobj::{AObjGlobals, HsdAObj};
use melee_runtime::dat::DatArchive;
use melee_runtime::mtx::Mtx;
use melee_runtime::particle::{CmdRef, GenId, JObjRef, PsHost, PsSystem};
use melee_runtime::random::HsdRand;
use melee_types::neutral_hash::nh_fighter;
use melee_runtime::mtx::Vec3;
use melee_types::{Gr_Kind_Greens, Gr_Kind_Story, Gr_Kind_Battle, Gr_Kind_Last, ftCollisionBox, Fighter, FighterKind, Ft_Kind_Captain, Ft_Kind_Fox};
use melee_world::mp::coll::{self as mc, CollGlobals};
use melee_world::mp::lines::GroundColl;
use melee_world::gr::bgm::{self, BgmUnlocks, StageParam};
use melee_world::gr::grlast::LastBg;
use std::collections::HashMap;
use std::sync::Arc;

/// `ftCo_MS_DeadDown`.
const FTCO_MS_DEAD_DOWN: i32 = 0;
/// `ftCo_MS_KneeBend` / `JumpF` / `JumpB` (ftmotionstates.c 24-26) and their rows (15-17).
const FTCO_MS_KNEE_BEND: i32 = 24;
const FTCO_MS_JUMP_F: i32 = 25;
const FTCO_MS_JUMP_B: i32 = 26;
const FTCO_SM_KNEE_BEND: u32 = 15;
/// `ftCo_MS_JumpAerialF` / `B`, `FallAerial` and the rows (18, 19; 23-25).
const FTCO_MS_JUMP_AERIAL_F: i32 = 27;
const FTCO_MS_JUMP_AERIAL_B: i32 = 28;
const FTCO_MS_FALL_AERIAL: i32 = 32;
const FTCO_SM_JUMP_AERIAL_F: u32 = 18;
const FOX_AIR_N_START: i32 = fox::FX_MS_SPECIAL_AIR_N_START;
const FOX_AIR_N_LOOP: i32 = fox::FX_MS_SPECIAL_AIR_N_LOOP;
const FOX_AIR_N_END: i32 = fox::FX_MS_SPECIAL_AIR_N_END;
const FTCO_SM_FALL_AERIAL: u32 = 23;
/// `ftCo_MS_GuardOn` .. `GuardReflect` (ftmotionstates.c 178-182) and their rows
/// (`ftCo_SM_GuardOn` 37, `Guard` 38, `GuardOff` 39).
const FTCO_MS_GUARD_ON: i32 = 178;
const FTCO_MS_GUARD: i32 = 179;
const FTCO_MS_GUARD_OFF: i32 = 180;
const FTCO_MS_GUARD_SET_OFF: i32 = 181;
const FTCO_MS_GUARD_REFLECT: i32 = 182;
const FTCO_SM_GUARD_ON: u32 = 37;
const FTCO_SM_GUARD_OFF: u32 = 39;
/// `ftCo_MS_Entry` / `EntryStart` / `EntryEnd` (ftmotionstates.c: 322-324).
const FTCO_MS_ENTRY: i32 = 322;
const FTCO_MS_ENTRY_START: i32 = 323;
const FTCO_MS_ENTRY_END: i32 = 324;
/// `ftCo_SM_EntryStart` (ft/kinds/ftCommon/forward.h): EntryStart's animation row.
const FTCO_SM_ENTRY_START: u32 = 238;
/// `ftCo_MS_Fall` (29) and its row `ftCo_SM_Fall` (20).
const FTCO_MS_FALL: i32 = 29;
const FTCO_SM_FALL: u32 = 20;
/// `ftCo_MS_Wait` (14), `WalkSlow`.. `WalkFast` (15..17), their rows (`ftCo_SM_Wait1_0` 2,
/// `WalkSlow`.. 7..9).
const FTCO_MS_WAIT: i32 = 14;
const FTCO_SM_WAIT: u32 = 2;
const FTCO_MS_WALK_SLOW: i32 = 15;
const FTCO_MS_WALK_FAST: i32 = 17;
const FTCO_SM_WALK_SLOW: u32 = 7;
/// `ftCo_MS_Turn` (18) and its row `ftCo_SM_Turn` (10).
const FTCO_MS_TURN: i32 = 18;
const FTCO_SM_TURN: u32 = 10;
/// `ftCo_MS_Landing` (42) and its row `ftCo_SM_Landing` (35).
const FTCO_MS_LANDING: i32 = 42;
const FTCO_MS_OTTOTTO: i32 = 245;
const FTCO_MS_OTTOTTO_WAIT: i32 = 246;
const FTCO_SM_OTTOTTO: u32 = 210;
const FTCO_SM_LANDING: u32 = 35;
/// `ftCo_MS_LandingAirN..Lw` (the submotions follow at `ftCo_SM_LandingAirN` = 73).
const FTCO_MS_LANDING_AIR_N: i32 = 70;
const FTCO_SM_LANDING_AIR_N: u32 = 73;
/// `ftCo_MS_DamageHi1` .. `ftCo_MS_DamageFlyRoll`.
fn is_damage_motion(m: i32) -> bool {
    (75..=91).contains(&m)
}

fn is_landing_air(m: i32) -> bool {
    (FTCO_MS_LANDING_AIR_N..FTCO_MS_LANDING_AIR_N + 5).contains(&m)
}
/// Final Destination's floor materials (mplib.c `mpLib_803BF248[Gr_Kind_Last]` = `mpLib_803BE7A8`,
/// each entry's `x0`): the friction multiplier `mpLib_800569EC(floor.flags)` returns.
const FD_MATERIAL_FRICTION: [f32; 20] =
    [1.0, 1.0, 1.5, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.1, 0.9, 1.0, 0.2, 1.0, 0.1, 1.0, 1.0];

/// `ft_GetGroundFrictionMultiplier` (not Popo/Nana) -> `mpColl_8004CA6C` on Final Destination.
fn fd_ground_friction(fp: &Fighter) -> f32 {
    if fp.coll_data.floor.index != -1 {
        FD_MATERIAL_FRICTION[(fp.coll_data.floor.flags as u8) as usize]
    } else {
        1.0
    }
}
/// The entry beam: `efAsync_Spawn(.., 0x43E, ..)` -> efAsync_Dispatch -> `efLib_Create_Attach(0x24)`.
const EF_ENTRY_BEAM: u32 = 0x24;
/// `Gr_Kind_Last`'s StKind in a VS match (`grGroundParam->stage_params[0].stkind`).
pub const STKIND_FD: i32 = 0x20;
pub const STKIND_BF: i32 = 0x1F;
/// Fox and Falco share the ftFx code (and the `ftFox_DatAttrs` record).
fn is_fx_kind(k: FighterKind) -> bool {
    k == Ft_Kind_Fox || k == melee_types::Ft_Kind_Falco
}

/// Dream Land (Green Greens, `GrGr.dat`, Gr_Kind_Greens).
pub const STKIND_DL: i32 = 0x11;
/// Yoshi's Story (`GrSt.dat`, Gr_Kind_Story).
pub const STKIND_YS: i32 = 8;
/// efLib's particle procs (eflib.c `efLib_particles_proc_main` / `_aux`, TARGET_PC masks).
const PTCL_MASK_MAIN: u32 = 0x20000 | 0x40000;
const PTCL_MASK_AUX: u32 = 0x10000 | 0x80000;

/// `fp->mv.co.entry` (ft/types.h `ftCo_Entry` motion vars; kept beside the Fighter until the
/// motion-state union is wired). GC offsets inside `fp->mv`: timer +0, x4 +4, x20 +0x20,
/// x24 +0x24, x28 +0x28.
#[derive(Clone, Copy)]
pub struct EntryVars {
    pub timer: i32,
    pub x4: f32,
    /// `x8`: the root JObj's scale at Entry_Enter; `x14`: the squashed scale.
    pub x8: Vec3,
    pub x14: Vec3,
    pub x20: f32,
    pub x24: f32,
    pub x28: f32,
    /// `x2C`: the entry ECB box (`ft_80084CB0` copy of the ECB at Entry_Enter).
    pub x2c: ftCollisionBox,
}

impl Default for EntryVars {
    fn default() -> Self {
        let z = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        EntryVars { timer: 0, x4: 0.0, x8: z, x14: z, x20: 0.0, x24: 0.0, x28: 0.0, x2c: ftCollisionBox::zeroed() }
    }
}

pub struct Player {
    pub slot: i32,
    /// The partner fighter (`player_entity[1]`): Zelda behind Sheik, hash slot `slot * 2 + 1`.
    pub follower: bool,
    /// `ftData->x30`: the hurtbox capsules (part, a_offset, b_offset, scale).
    pub hurtboxes: Vec<(u32, [f32; 3], [f32; 3], f32)>,
    pub fp: Box<Fighter>,
    /// The stand-in `HSD_GObj` the ported fighter code takes (`user_data` -> `fp`).
    pub gobj: Box<melee_types::HSD_GObj>,
    /// An m-ex fighter's ftFunction code (crate::mex), run on `fp` at onLoad / onFrame.
    pub mex: Option<crate::mex::MexFighter>,
    pub entry: EntryVars,
    /// The fighter's JObj tree (`GET_JOBJ(gobj)`) and parts.
    pub skel: Skeleton,
    /// `fp->anim_id != -1`.
    animating: bool,
    /// Animation rows (`ftData->xC[smid]`) of the states the match reaches, with their figatrees.
    row_entry_start: Option<(AnimRow, FigaTree)>,
    row_fall: Option<(AnimRow, FigaTree)>,
    row_landing: Option<(AnimRow, FigaTree)>,
    row_turn: Option<(AnimRow, FigaTree)>,
    row_wait: Option<(AnimRow, FigaTree)>,
    /// WalkSlow, WalkMiddle, WalkFast.
    row_walk: [Option<(AnimRow, FigaTree)>; 3],
    /// `fp->mv.co.walk`.
    walk: WalkVars,
    /// `fp->mv.co.turn` (has_turned, just_turned, facing_after, frames_to_turn, x8, x1C).
    turn: TurnVars,
    /// `fp->mv.co.landing.allow_interrupt`.
    landing_allow_interrupt: bool,
    /// `ftData->x44`.
    pub ecb_desc: EcbDesc,
    /// `ft_data->x0->trophy_scale` and `model_scaling` (`co_attrs`).
    trophy_scale: f32,
    model_scaling: f32,
    /// `fp->mv.co.fall.smid` / `.x4` (the Fall row and its blend weight).
    fall_smid: i32,
    fall_x4: f32,
    /// `fp->x60C`: queued `efAsync_Spawn`s (flushed in the fighter's s_link 9 proc), in
    /// queue order (the C prepends, so the flush runs the newest first).
    ef_queue: Vec<QueuedEffect>,
    /// Effect work a motion change did before s_link 9 (`efAsync_QueueFlush` of the queue, then
    /// `ftCommon_8007DB24`'s `efLib_DestroyAll`), replayed in order at the s_link 9 proc: nothing
    /// between a fighter's motion change and its s_link 9 proc reads the effects.
    ef_ops: Vec<EfOp>,
    /// PlXx.dat's data (command scripts are DAT offsets into it).
    pdat: Arc<Vec<u8>>,
    /// The command script (`x3E4_fighterCmdScript.u` as a DAT offset, 0 = NULL) and its stack
    /// (`loop_count`, `event_return`). timer / frame_count live in the Fighter.
    script: u32,
    script_sp: usize,
    script_ret: [u32; 8],
    /// `ftData->x1C`: the hand-pose slots.
    part_anim: [Option<crate::fighter_data::skeleton::PartAnimData>; 5],
    /// `x221D_b3` (input history primed) and `x221D_b4` (input locked until GO).
    x221d_b3: bool,
    input_locked: bool,
    /// The fighter's files (any other animation row, loaded on first use into `rows`).
    files: Arc<crate::fighter_data::FighterFiles>,
    rows: HashMap<u32, Option<(AnimRow, FigaTree)>>,
    /// `ftPartsTable[kind]->part_to_joint` (`ftParts_GetBoneIndex`).
    part_to_joint: Vec<u8>,
    /// `fp->dat_attrs` for Captain Falcon (`ftData->x4`, `ftCaptain_DatAttrs`).
    ca_attrs: Option<melee_types::ftCaptain_DatAttrs>,
    /// `fp->mv.ca.speciallw`: `x0` (hit count) and `friction`.
    ca_lw_x0: f32,
    ca_lw_friction: f32,
    /// `fp->x914[4]`: the live hitboxes (only where they are: no hit detection is ported, so
    /// [`VsMatch::hit_guard`] stops the run when one comes near the other fighter).
    hitboxes: [Option<Hitbox>; 4],
    /// `fp->mv.co.attack1.x0` (A pressed again inside the jab window).
    attack1_x0: bool,
    /// `fp->mv.co.unk_800D3680.x40`: the frames left in a Dead* state (`p_ftCommonData->x500`).
    dead_timer: i32,
    /// The player's HUD `IfDamageFlags.randomize_velocity` (set by `ifStatus_802F6C04`, a self
    /// destruct): the HUD proc's next run draws 8 `HSD_Randf`s (`ifStatus_PercentOnDeathAnimationThink`).
    hud_randomize: bool,
    /// The percent display's state (`IfDamageState`, ifstatus.c).
    hud: hit::HudDamage,
    /// `fp->mv.co.guard` and `x221B_b0` (the shield is up).
    guard: GuardVars,
    /// `fp->mv.co.kneebend` / `fp->mv.co.jump`.
    jump: JumpVars,
    dash: dashrun::DashVars,
    run: dashrun::RunVars,
    /// Samus' side special (samus.rs).
    ss: samus::SsVars,
    ss_attrs: Option<samus::SsAttrs>,
    /// Fox's blaster state (fox.rs) and the item work he hands to the item procs.
    fx: FoxVars,
    item_cmds: Vec<ItemCmd>,
    /// `fp->dat_attrs` for Fox.
    fx_attrs: Option<melee_types::ftFox_DatAttrs>,
    /// Sheik's Needle Storm state (sheik.rs) and `fp->dat_attrs`.
    sk: sheik::SeakVars,
    sk_attrs: Option<melee_types::ftSeakAttributes>,
    /// The thrown needle's article (`ft_data->x48_items[0]`).
    sk_needle: Option<sheik::NeedleAttrs>,
    /// The next `Fighter_ChangeMotionState` has `Ft_MF_KeepGfx`.
    keep_gfx: bool,
    /// The next one has `anim_blend == -1` (no blend frames).
    blend_zero: bool,
    /// The player's respawn data (rebirth.rs) and `ft_data->x24`, the idle animations
    /// (`{anim id, weight}` up to the -1 terminator).
    rb: PlayerRespawn,
    wait_anims: Vec<(i32, i32)>,
    /// Dynamic bones (tail, hair): `Fighter_8006D9AC` at s_link 0x10.
    dynamics: Option<crate::dynamics::Dynamics>,
    /// The match's map collision and common data for the state enters that cannot take them as
    /// arguments (set at the start of every frame; `special_checks`).
    gc: *mut GroundColl,
    common_c: EntryCommon,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JumpVars {
    pub jump_input: i32,
    pub is_short_hop: bool,
    pub x4: bool,
}

/// `fp->mv.co.guard` (ft/kinds/ftCommon/types.h) plus `fp->x221B_b0`, whose ft_0877 setter
/// (`ftColl_8007B1B8`) is the only writer here.
#[derive(Clone, Copy, Debug, Default)]
pub struct GuardVars {
    pub x0: f32,
    pub x4: f32,
    pub x8: f32,
    pub xc: bool,
    pub x10: f32,
    pub x14: f32,
    pub x18: f32,
    pub x1c: i32,
    pub x20: i32,
    pub x24: i32,
    pub x2c: f32,
    pub shield_on: bool,
}

/// A live `HitCapsule` (`ftAction_8007121C`): its part, size and offset.
#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pub part: usize,
    pub scale: f32,
    pub offset: Vec3,
    pub hit_group: u32,
}

/// `ftCo_Walk`'s motion vars (ft/types.h `mv.co.walk`).
#[derive(Clone, Copy, Debug, Default)]
pub struct WalkVars {
    pub x0: f32,
    pub accel_mul: f32,
    pub msid: i32,
    pub slow_anim_frame: f32,
    pub middle_anim_frame: f32,
    pub fast_anim_frame: f32,
    pub slow_anim_rate: f32,
    pub middle_anim_rate: f32,
    pub fast_anim_rate: f32,
}

/// `ftCo_Turn`'s motion vars (ft/types.h `mv.co.turn`).
#[derive(Clone, Copy, Debug, Default)]
pub struct TurnVars {
    pub has_turned: bool,
    pub just_turned: bool,
    pub facing_after: f32,
    pub frames_to_turn: f32,
    pub x8: f32,
    pub x1c: u32,
}

/// `EF_QueuedEffect`: an `efAsync_Spawn` waiting for the s_link 9 flush.
#[derive(Clone, Copy, Debug)]
pub struct QueuedEffect {
    pub kind: u8,
    pub gfx: u32,
    /// The JObj as a fighter part index (`fp->parts[part].joint`).
    pub part: usize,
    pub params: Vec3,
    pub extra1: f32,
    /// A second float argument (`EF_SPAWN_POS_OFFSET_PARAM2`'s rotation).
    pub extra2: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum EfOp {
    Spawn(QueuedEffect),
    /// `efLib_DestroyAll(gobj)`.
    DestroyAll,
}

/// `EF_SPAWN_ATTACH_PARAM` / `EF_SPAWN_POS_OFFSET_PARAM` (ef/types.h spawn kinds 3 and 5).
const EF_SPAWN_ATTACH_PARAM: u8 = 3;
const EF_SPAWN_POS_OFFSET_PARAM: u8 = 5;
/// `EF_SPAWN_POS_OFFSET`: `(jobj, &ofs)`.
const EF_SPAWN_POS_OFFSET: u8 = 2;
/// `EF_SPAWN_POS_OFFSET_PARAM2`.
const EF_SPAWN_POS_OFFSET_PARAM2: u8 = 6;
/// A direct `efSync_Spawn` (not queued): kind 0 here.
const EF_SPAWN_SYNC: u8 = 99;
/// `EF_SPAWN_ATTACH` (ef/types.h): `(jobj)`.
const EF_SPAWN_ATTACH: u8 = 0;

/// `p_ftCommonData` words the entry states read (PlCo.dat `ftLoadCommonData`, ft/types.h).
#[derive(Clone, Copy, Debug, Default)]
struct EntryCommon {
    x6bc: i32,
    x6c0: i32,
    x804d6524_x30: f32,
    crowd_horiz_margin: f32,
}

/// One live effect from `efLib_Create` (EF_Effect + its JObj tree): only its particle tracks.
struct Effect {
    lifetime: u16,
    /// Owner fighter slot (the RObj position constraint follows its root JObj).
    owner: usize,
    scale: f32,
    /// Index into `VsMatch::effect_banks`.
    bank: usize,
    tracks: Vec<EffectTrack>,
    /// `efLib_Create_Attach_Pos`: the root's fixed translation (None: follows the owner).
    fixed_pos: Option<[f32; 3]>,
    /// `HSD_JObjSetRotationY/Z(root, ..)` by the spawner (None: the model's own rotation).
    rot_y: Option<f32>,
    rot_z: Option<f32>,
}

struct EffectTrack {
    joint: JObjRef,
    chain: Vec<u32>,
    aobj: HsdAObj<()>,
}

/// The root JObj of a live effect, as the particle callbacks read it (`efLib_SpawnParticleEffect`).
#[derive(Clone, Copy, Debug)]
struct EffectRoot {
    pos: [f32; 3],
    rot_z: Option<f32>,
    scale: f32,
}

/// A stage particle track and its AObj (see `stage::dptcl_tracks`).
struct DptclAnim {
    joint: JObjRef,
    aobj: HsdAObj<()>,
    /// The init-time first step has run.
    started: bool,
    /// The map gobj whose animation owns the track.
    gobj: u32,
    /// Created mid-frame by a stage proc: its first end-of-frame step belongs to the next frame.
    skip: bool,
}

pub struct VsMatch {
    /// `it_804A0E30.x0`: frames until the random item spawner (`fn_8026C88C`) next fires.
    pub item_timer: Option<i32>,
    pub players: Vec<Player>,
    /// mplib's map collision (`groundCollLine` & co.) and mpcoll's statics.
    pub ground: GroundColl,
    pub cg: CollGlobals,
    /// `HSD_PadGameStatus[4]`.
    pub pads: [input::PadStatus; 4],
    /// The golden frame whose s_link 0 runs the GO callback (input unlock).
    go_frame: i32,
    pub rand: HsdRand,
    pub ps: PsSystem,
    pub frame: i32,
    /// Set when a frame needs code that is not ported; the run ends before logging that frame.
    pub stopped: Option<String>,
    /// The match BGM `Ground_801C24F8` picked (an `hps_files` index; lbAudioAx_80023F28 plays it).
    pub bgm: i32,
    last_bg: Option<LastBg>,
    dptcl: Vec<DptclAnim>,
    /// Green Greens' per-frame stage procs (greens.rs).
    greens: Option<greens::Greens>,
    story: Option<story::Story>,
    aobj_globals: AObjGlobals,
    /// Bind-pose world matrices of the joints a generator can follow (by DAT offset; an effect
    /// joint is `(instance << 24) | offset`).
    joint_mtx: HashMap<JObjRef, Mtx>,
    common: EntryCommon,
    /// `efAsync_DatEntries[bank]` that are loaded (0 common, 3 Fox, 4 Captain).
    effect_banks: Vec<Option<crate::effect::EffectBank>>,
    effects: Vec<Effect>,
    next_effect_instance: u32,
    /// Bank 0's command lists that `efLib_Cb_PtclAppSRTHook` acts on (0x96, 0x97, 0x98, 0x21B).
    appsrt_hook_lists: Vec<CmdRef>,
    /// `Stage_GetBlastZone{Left,Right,Top,Bottom}Offset()` (the stage's dead range).
    blast: BlastZone,
    /// Fighter parts that generators follow: (`JObjRef`, player index, part). Their matrices in
    /// `joint_mtx` are the part's world translation (no rotation: particle positions are not hashed).
    fighter_jobjs: Vec<(JObjRef, usize, usize)>,
    /// The live items (fox.rs).
    items: Vec<SimItem>,
    /// `Fighter_804D64FC->x20`.
    cpu_dist: Vec<f32>,
    plco_bytes: std::sync::Arc<[u8]>,
    cpu_tbl: u32,
    /// `stage_info.grkind`.
    grkind: i32,
    /// `stage_info.unk8C.b4` (grBattle_OnInit): respawn at the stage point of the player's slot.
    per_slot_spawn: bool,
    /// `Stage_80224E38(&v, n)`: the stage's respawn points (points 4..8), `scene_state->fighters[].x8`
    /// (the respawn spots' countdowns) and `PlCo`'s start shield.
    respawn_points: Vec<Vec3>,
    respawn_busy: [i32; 6],
    start_shield: f32,
    /// Why a damage hook could not proceed (damage.rs).
    dmg_err: Option<String>,
}

/// Green Greens' `grGreens_Init` (the map gobjs' OnInit in order 6, 5 for the draws that matter): the
/// bomb-block pattern (`grGreens_80214B58`), the block timer, the wind timer, and the 18 block items
/// (`grGreens_80214FA8` -> `grGreens_80215358`). Returns the blocks as items in creation order.
/// `it_8026D018`: the random item spawner's setup. The tables it builds (`it_8026CA4C`,
/// `it_8026CB9C`, `it_8026CF04`) only matter once an item is picked; the setup's one RNG draw is the
/// first timer: `x0 = (int)(diff * randf + range[0])`, then `x0 *= Ground_801C2AE8(stkind)`.
fn item_spawner_init(disc: &mut crate::disc::Disc, stage: &DatArchive, stkind: i32, item_freq: i32, rand: &mut HsdRand) -> Result<i32, String> {
    let be = |d: &[u8], at: u32| u32::from_be_bytes(d[at as usize..at as usize + 4].try_into().unwrap());
    // ItCo.usd (the English disc's language file) else ItCo.dat: itPublicData -> x0 = ItemCommonData; xFC = s32 ranges.
    let bytes = match disc.read("ItCo.usd") {
        Ok(b) => b,
        Err(_) => disc.read("ItCo.dat")?,
    };
    let a = DatArchive::parse(&bytes).map_err(|e| format!("ItCo: {e}"))?;
    let public = a.public_offset("itPublicData").ok_or("ItCo: no itPublicData")?;
    let common = be(&a.data, public);
    let at = common + 0xFC + 8 * item_freq as u32;
    let (r0, r1) = (be(&a.data, at) as i32, be(&a.data, at + 4) as i32);
    let randf = rand.randf();
    let diff = (r1 - r0) as f32;
    let x0 = melee_core::fp::fmadds(diff, randf, r0 as f32) as i32;
    // Ground_801C2AE8: (0.01f * param->x68) * (0.01f * row.x18).
    let param = crate::stage::ground_param(stage).ok_or("no grGroundParam")?;
    let (rows, n) = (be(&stage.data, param + 0xB0), be(&stage.data, param + 0xB4));
    let s16 = |at: u32| i16::from_be_bytes(stage.data[at as usize..at as usize + 2].try_into().unwrap());
    let mut mult = None;
    for i in 0..n {
        let e = rows + i * 0x64;
        if be(&stage.data, e) as i32 == stkind {
            mult = Some((0.01f32 * s16(param + 0x68) as f32) * (0.01f32 * s16(e + 0x18) as f32));
            break;
        }
    }
    let mult = mult.ok_or("Ground_801C2AE8: no stage param row")?;
    Ok(((x0 as f32) * mult) as i32)
}

fn greens_init(stage: &DatArchive, rand: &mut HsdRand, scale: f32) -> Result<(Vec<fox::SimItem>, greens::InitDraws), String> {
    let y = crate::stage::yakumono_words(stage, 0x1E).ok_or("GrGr.dat: no yakumono_param")?;
    let y = |i: usize| y[i] as i32;
    let range = |rand: &mut HsdRand, min: i32, max: i32| -> i32 {
        if min > max {
            let d = min - max;
            max + if d != 0 { rand.randi(d) } else { 0 }
        } else if min < max {
            let d = max - min;
            min + if d != 0 { rand.randi(d) } else { 0 }
        } else {
            min
        }
    };
    let mut table = [0u8; 30];
    for idx in [0x12usize, 0x0C, 0x0D, 0x06, 0x07, 0x08, 0x00, 0x01, 0x02, 0x17, 0x10, 0x11, 0x09, 0x0A, 0x0B, 0x03, 0x04, 0x05] {
        let r = if y(8) != 0 { rand.randi(y(8)) } else { 0 };
        table[idx] = if r != 0 { 1 } else { 2 };
    }
    // grGreens_80214674: the block timer.
    let block_timer = range(rand, y(0), y(1));
    // grGreens_802139C4: the wind timer and its first side.
    let wind_xc = range(rand, y(14), y(13));
    let wind_x14 = rand.randi(2);
    // grGreens_8021483C: the 5x6 grid of block cells from the 30 block joints' positions.
    let (mut lmax, mut lmin, mut ltop, mut lbot) = (-3.4028235e38f32, 3.4028235e38f32, -3.4028235e38f32, 3.4028235e38f32);
    let (mut rmax, mut rtop, mut rmin, mut rbot) = (-3.4028235e38f32, -3.4028235e38f32, 3.4028235e38f32, 3.4028235e38f32);
    for i in 0..30usize {
        let (v, _) = crate::stage::map_joint_world(stage, 6, 5 + i).ok_or("GrGr.dat: block joint missing")?;
        // The map JObj tree hangs under the root scaled by grGroundParam->y.
        let v = [v[0] * scale, v[1] * scale, v[2] * scale];
        if v[0] < 0.0 {
            lmax = lmax.max(v[0]);
            lmin = lmin.min(v[0]);
            ltop = ltop.max(v[1]);
            lbot = lbot.min(v[1]);
        } else {
            rmax = rmax.max(v[0]);
            rmin = rmin.min(v[0]);
            rtop = rtop.max(v[1]);
            rbot = rbot.min(v[1]);
        }
    }
    let mut grid = [[(0.0f32, 0.0f32); 6]; 5];
    let (xs, ys) = ((lmax - lmin) / 2.0, (ltop - lbot) / 4.0);
    for i in 0..5 {
        for j in 0..3 {
            grid[i][j] = (xs * j as f32 + lmin, ys * i as f32 + lbot);
        }
    }
    let (xs, ys) = ((rmax - rmin) / 2.0, (rtop - rbot) / 4.0);
    for i in 0..5 {
        for j in 3..6 {
            grid[i][j] = (xs * (j - 3) as f32 + rmin, ys * i as f32 + rbot);
        }
    }
    let mut items = Vec::new();
    for j in 0..5 {
        for col in 0..6 {
            if matches!(table[j * 6 + col], 1 | 2) {
                let (x, yy) = grid[j][col];
                items.push(fox::SimItem { kind: 160, msid: 0, pos: Vec3 { x, y: yy, z: 0.0 }, vel: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, class: fox::ItemClass::GreensBlock });
            }
        }
    }
    if std::env::var_os("MELEE_GOLDEN_DUMP").is_some() { for it in &items { eprintln!("block {:?} {:?}", it.pos.x, it.pos.y); } }
    Ok((items, greens::InitDraws { block_timer, wind_xc, wind_x14, table }))
}

/// Green Greens' block map joints (map gobj 6, joints 0..29 <-> JObjs 5..34): each used block
/// (`grGreens_80215358`: JObj translate = cell position / `Ground_801C0498()`, then
/// `mpLib_80055E9C`) moves its joint to the cell; the unused ones keep the hidden JObj
/// (`grGreens_80214FA8`) that `Ground_UpdateMapColl` turns into `mpJointHide`.
fn greens_sync_joints(stage: &DatArchive, ground: &mut melee_world::mp::lines::GroundColl, scale: f32, blocks: &[fox::SimItem]) -> Result<(), String> {
    let d = &stage.data;
    let head = stage.public_offset("map_head").ok_or("GrGr.dat: no map_head")?;
    let gobjs = u32::from_be_bytes(d[head as usize + 8..][..4].try_into().unwrap());
    let root = u32::from_be_bytes(d[(gobjs + 6 * 0x34) as usize..][..4].try_into().unwrap());
    let mut mt = crate::fighter_data::skeleton::MapTree::load(d, root, scale)?;
    let f = (1.0f64 / scale as f64) as f32;
    for num in 0..30usize {
        let j = *mt.dfs.get(5 + num).ok_or("GrGr.dat: block JObj missing")?;
        match blocks.get(num) {
            Some(b) => {
                mt.set_translate(j, Vec3 { x: b.pos.x * f, y: b.pos.y * f, z: b.pos.z * f });
                let m = mt.mtx_of(j);
                if !(m[0][0] == m[1][1] && m[0][0] == m[2][2]) {
                    return Err("mpLib_80055E9C: non-uniform block matrix not ported".into());
                }
                ground.mp_lib_80055e9c(num as i32, Some((m[0][0], m[0][3], m[1][3])));
                ground.mp_lib_80057424(num as i32);
            }
            None => ground.mp_lib_80055e9c(num as i32, None),
        }
    }
    Ok(())
}

/// `Stage_GetBlastZone*Offset()`: `stage_info.blast_zone.* + cam_info.cam_{x,y}_offset`
/// (`Ground_801C39C0` reads the camera range points 0x94-0x96, `Ground_801C3BB4` the dead range
/// points 0x97/0x98; the stage code overrides neither on Final Destination).
#[derive(Clone, Copy, Debug)]
pub struct BlastZone {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    /// `Stage_GetCamBoundsTopOffset()`.
    pub cam_top: f32,
    pub cam_left: f32,
    pub cam_right: f32,
    pub cam_bottom: f32,
}

impl BlastZone {
    fn from_points(points: &[crate::stage::SpawnPoint]) -> Result<Self, String> {
        let pt = |id: i32| points.iter().find(|p| p.id == id).map(|p| p.pos);
        let (a, b, c) = (pt(0x95), pt(0x96), pt(0x94));
        let (cam_x, cam_y) = match (a, b, c) {
            (Some(_), Some(_), Some(c)) => (c[0], c[1]),
            _ => return Err("stage: no camera range points (Ground_801C39C0 dummy range not wired)".into()),
        };
        let (Some(p1), Some(p2)) = (pt(0x97), pt(0x98)) else {
            return Err("stage: no dead range points (Ground_801C3BB4 dummy range not wired)".into());
        };
        let (lft, rgt) = if p1[0] < p2[0] { (p1[0] - cam_x, p2[0] - cam_x) } else { (p2[0] - cam_x, p1[0] - cam_x) };
        let (bot, top) = if p1[1] < p2[1] { (p1[1] - cam_y, p2[1] - cam_y) } else { (p2[1] - cam_y, p1[1] - cam_y) };
        // Ground_801C39C0: cam_bounds.top = the larger y of points 0x95 / 0x96 minus the offset.
        let (a, b) = (a.unwrap(), b.unwrap());
        let cam_top_raw = if a[1] < b[1] { b[1] - cam_y } else { a[1] - cam_y };
        let (a, b2) = (a, b);
        let _ = (a, b2);
        // Ground_801C39C0: left/right/bottom of the camera range, the same way.
        let (pa, pb) = (pt(0x95).unwrap(), pt(0x96).unwrap());
        let (cl, cr) = if pa[0] < pb[0] { (pa[0] - cam_x, pb[0] - cam_x) } else { (pb[0] - cam_x, pa[0] - cam_x) };
        let cb = if pa[1] < pb[1] { pa[1] - cam_y } else { pb[1] - cam_y };
        Ok(BlastZone { left: lft + cam_x, right: rgt + cam_x, top: top + cam_y, bottom: bot + cam_y, cam_top: cam_top_raw + cam_y, cam_left: cl + cam_x, cam_right: cr + cam_x, cam_bottom: cb + cam_y })
    }
}

/// The particle system's view of the match: the RNG and the stage joint matrices.
struct Host<'a> {
    rand: &'a mut HsdRand,
    joint_mtx: &'a HashMap<JObjRef, Mtx>,
    appsrt_hook_lists: &'a [CmdRef],
}

impl PsHost for Host<'_> {
    fn randf(&mut self) -> f32 {
        self.rand.randf()
    }
    fn jobj_setup_matrix(&mut self, j: JObjRef) -> Mtx {
        *self.joint_mtx.get(&j).expect("generator on a joint with no matrix")
    }
    /// `efLib_Cb_PtclAppSRTHook`: `hsd_8039D1E4(gen, lbl_803BF810)` for four bank-0 lists.
    fn generator_created(&mut self, ps: &mut PsSystem, g: GenId) {
        if let Some(c) = &ps.gen(g).cmd_list {
            if self.appsrt_hook_lists.contains(c) {
                panic!("efLib_Cb_PtclAppSRTHook: hsd_8039D1E4 user func (TODO(group5 ef user funcs))");
            }
        }
    }
}

/// The C build's save state in these runs: every character unlocked (the alternate-music roll
/// for FD's row, `x14 == 6`, happens there: MELEE_RAND_TRACE shows its draw).
struct SaveUnlocks;
impl BgmUnlocks for SaveUnlocks {
    fn mars(&self) -> bool {
        true
    }
    fn clink(&self) -> bool {
        true
    }
    fn toy_11a(&self) -> bool {
        panic!("Toy_803048C0(0x11A) (TODO(group6 ty))")
    }
    fn all_characters(&self) -> bool {
        true
    }
    fn gm_80164600(&self) -> bool {
        panic!("gm_80164600 (TODO(group6 gm))")
    }
}

/// gmvs.c `direction`: face the stage centre.
fn direction(x: f32) -> f32 {
    if x >= 0.0 {
        -1.0
    } else {
        1.0
    }
}

fn fighter_kind(name: &str) -> Result<FighterKind, String> {
    match name {
        "falcon" => Ok(Ft_Kind_Captain),
        "fox" => Ok(Ft_Kind_Fox),
        "falco" => Ok(melee_types::Ft_Kind_Falco),
        "marth" => Ok(melee_types::Ft_Kind_Mars),
        "sheik" => Ok(melee_types::Ft_Kind_Seak),
        "zelda" => Ok(melee_types::Ft_Kind_Zelda),
        "peach" => Ok(melee_types::Ft_Kind_Peach),
        "samus" => Ok(melee_types::Ft_Kind_Samus),
        // Akaneia's Wolf: an m-ex fighter built on Fox (its ftFunction overrides some slots; the
        // rest is Fox's vanilla code, as m-ex's per-kind tables copy them). The native match runs
        // it as Fox's kind with Wolf's files: crate::fighter_data::files_for.
        "wolf" => Ok(Ft_Kind_Fox),
        _ => Err(format!("fighter {name:?} not wired yet (falcon, fox, wolf)")),
    }
}

impl VsMatch {
    pub fn new(disc: &mut crate::disc::Disc, stage_file: &str, fighter_specs: &[&str], seed: u32, stocks: i32) -> Result<Self, String> {
        Self::new_ex(disc, stage_file, fighter_specs, seed, stocks, None)
    }

    /// `item_freq`: the VS rules' item frequency (`items=N`; None: items off).
    pub fn new_ex(disc: &mut crate::disc::Disc, stage_file: &str, fighter_specs: &[&str], seed: u32, stocks: i32, item_freq: Option<i32>) -> Result<Self, String> {
        // "fox" (human) or "fox/cpu9" (CPU, level 9).
        let cpu_levels: Vec<Option<i32>> = fighter_specs.iter().map(|s| s.split_once("/cpu").and_then(|(_, l)| l.parse().ok())).collect();
        let fighters: Vec<&str> = fighter_specs.iter().map(|s| s.split('/').next().unwrap()).collect();
        let fighters = &fighters[..];
        let stkind = match stage_file {
            "GrNLa.dat" => STKIND_FD,
            "GrNBa.dat" => STKIND_BF,
            "GrGr.dat" => STKIND_DL,
            "GrSt.dat" => STKIND_YS,
            _ => return Err(format!("{stage_file}: only Final Destination's and Battlefield's stage code is wired")),
        };
        let stage_bytes = disc.read(stage_file)?;
        let mut points = crate::stage::spawn_points(&stage_bytes)?;
        // Ground_801C0498 (grGroundParam->y): the stage JObj tree hangs under a root scaled by it
        // (Ground_801C34AC / get_jobj_inline), so every stage point is scaled (Battlefield 0.8).
        {
            let a = DatArchive::parse(&stage_bytes).map_err(|e| format!("{stage_file}: {e}"))?;
            let gp = crate::stage::ground_param(&a).ok_or("stage dat: no grGroundParam")?;
            let sc = f32::from_bits(u32::from_be_bytes(a.data[gp as usize..][..4].try_into().unwrap()));
            if sc != 1.0 && sc > 0.001 {
                for p in &mut points {
                    for c in &mut p.pos {
                        *c *= sc;
                    }
                }
            }
        }
        let blast = BlastZone::from_points(&points)?;
        if std::env::var_os("MELEE_GOLDEN_DUMP").is_some() {
            for p in &points {
                eprintln!("stage point {:#x}: {:?} exact {}", p.id, p.pos, p.exact);
            }
        }
        let stage = DatArchive::parse(&stage_bytes).map_err(|e| format!("{stage_file}: {e}"))?;
        let plco = DatArchive::parse(&disc.read("PlCo.dat")?).map_err(|e| format!("PlCo.dat: {e}"))?;
        let common = plco.public_offset("ftLoadCommonData").ok_or("PlCo.dat: no ftLoadCommonData")?;
        let common_root = common;
        // Fighter_LoadCommonData: the root is a pointer table, `p_ftCommonData = pData[0]`.
        // `p_ftCommonData->x260_startShieldHealth`; FT_SCRIPT_VALUE is the identity in vanilla.
        let be = |at: u32| u32::from_be_bytes(plco.data[at as usize..][..4].try_into().unwrap());
        let ft_common = be(common);
        let start_shield = f32::from_bits(be(ft_common + 0x260));
        // ftCommonData +0x6BC / +0x6C0: the EntryStart / EntryEnd durations.
        // Fighter_LoadCommonData: p_ftCommonData for the ported ftcommon code (melee_fighter::hooks).
        // Leaked: it lives as long as the process, as the C's archive does.
        let cd: melee_types::ftCommonData = *plco.load_at::<melee_types::ftCommonData>(ft_common).get();
        unsafe { melee_fighter::hooks::P_FT_COMMON_DATA = Box::leak(Box::new(cd)) };
        // Player_80036DD8: pl_804D6470 = *plLoadCommonData (PdPm.dat), plbonus.c's constants.
        {
            let pd = DatArchive::parse(&disc.read("PdPm.dat")?).map_err(|e| format!("PdPm.dat: {e}"))?;
            let at = pd.public_offset("plLoadCommonData").ok_or("PdPm.dat: no plLoadCommonData")?;
            let root = u32::from_be_bytes(pd.data[at as usize..][..4].try_into().unwrap());
            let t: melee_types::pl_804D6470_t = *pd.load_at::<melee_types::pl_804D6470_t>(root).get();
            unsafe { melee_fighter::hooks::PL_804D6470 = Box::leak(Box::new(t)) };
        }
        // gCrowdConfig = pData[21] (un_803222EC reads its knockback angle window).
        {
            let cc: melee_types::CrowdConfig = *plco.load_at::<melee_types::CrowdConfig>(be(common_root + 4 * 21)).get();
            unsafe { melee_fighter::tu::fighter::gCrowdConfig = Box::leak(Box::new(cc)) as *mut _ as *mut core::ffi::c_void };
        }
        // Fighter_804D6530 = pData[9]: three {Vec2 *shifts, count} rows (the hit model shift, ftCo_80090594).
        {
            let t = be(common_root + 4 * 9);
            let mut v: Vec<*mut melee_types::Vec2> = Vec::new();
            for k in 0..3u32 {
                let (arr, n) = (be(t + 8 * k), be(t + 8 * k + 4));
                let shifts: Vec<melee_types::Vec2> = (0..n).map(|i| melee_types::Vec2 { x: f32::from_bits(be(arr + 8 * i)), y: f32::from_bits(be(arr + 8 * i + 4)) }).collect();
                v.push(Box::leak(shifts.into_boxed_slice()).as_mut_ptr());
                v.push(n as usize as *mut melee_types::Vec2);
            }
            unsafe { melee_fighter::hooks::FIGHTER_804D6530 = Box::leak(v.into_boxed_slice()).as_mut_ptr() };
        }
        // plStale_InitAttackInstance and Fighter_804D6548 (PlCo pData[3]: the nine staling decrements).
        melee_fighter::stale::init_attack_instance();
        {
            let t = be(common_root + 4 * 3);
            let v: Vec<f32> = (0..9).map(|k| f32::from_bits(be(t + 4 * k))).collect();
            unsafe { melee_fighter::stale::STALE_DECREMENTS = Box::leak(v.into_boxed_slice()).as_ptr() };
        }
        // ft_80087D0C's lookups (the sound bank tables; no audio here, no Kirby remap).
        unsafe {
            melee_fighter::hooks::LBAUDIOAX_800233EC = Some(|id| id);
            melee_fighter::hooks::LBAUDIOAX_80023130 = Some(melee_audio::lbaudio::lb_80023130);
            melee_fighter::hooks::LBAUDIOAX_800230C8 = Some(melee_audio::sim::lbaudioax_800230c8);
            melee_fighter::hooks::LBAUDIOAX_80023220 = Some(melee_audio::lbaudio::lb_80023220);
        }
        // ft_PlaySFX from the ported fighter code (fast fall, ...): no audio here. Only the voice
        // ids 332..=370 draw (HSD_Randi(200) pitch), which the sim does not model.
        unsafe {
            melee_fighter::hooks::PLAY_SFX = Some(|_fp, id, _vol, _pan| {
                assert!(!(332..=370).contains(&id), "ft_PlaySFX {id}: the pitch draw is not modelled");
            })
        };
        let common = EntryCommon {
            x6bc: be(ft_common + 0x6BC) as i32,
            x6c0: be(ft_common + 0x6C0) as i32,
            // Fighter_804D6524 = pData[12]; ->x30 (ftCo_800D0EC8's scaling exponent).
            x804d6524_x30: f32::from_bits(be(be(common_root + 4 * 12) + 0x30)),
            // gCrowdConfig = pData[21]; ->horiz_margin (+0x2C) for un_803224DC.
            crowd_horiz_margin: f32::from_bits(be(be(common_root + 4 * 21) + 0x2C)),
        };
        damage::install_hooks();
        // ft_GetGroundFrictionMultiplier for the ported ftcommon code: FD's material table.
        unsafe { melee_fighter::hooks::GROUND_FRICTION_MUL = Some(fd_ground_friction) };

        let mut rand = HsdRand::default();
        rand.set_logic_seed(seed);

        // Boot (logic frame 0, the memory-card scene with a fresh save): gmMainLib_8015F600(1, 0)
        // -> Toy_SetUnlockState(Toy_80305058(2, 0x63, 0, 100.0f)): farg0 >= 100 picks from the
        // new trophies with one HSD_Randi(new_count). The draw is made here; the trophy id it
        // selects is not computed (TODO(group6 ty): trophy tables).
        let _ = rand.rand();

        // Stage load: the particle bank (grDatFiles_801C5FC0 locate; banks 0x40 and 0x1E).
        let mut ps = PsSystem::new();
        if let Some(bank) = crate::stage::particle_bank(&stage)? {
            let bank = Arc::new(bank);
            ps.load_bank(0x40, bank.clone(), None);
            ps.load_bank(0x1E, bank, None);
        }

        // efLib_Init / the scene's efAsync_LoadSync(0): EfCoData.dat, the common effect bank (bank 0:
        // particle data via psInitDataBankLoad(0, table[0], table[1]); effect descs at table[2]).
        let efco = crate::effect::EffectBank::load(&disc.read("EfCoData.dat")?, "EfCoData.dat", "effCommonDataTable")?;
        let mut appsrt_hook_lists = Vec::new();
        if let Some(b) = efco.particles()? {
            for i in [0x96, 0x97, 0x98, 0x21B] {
                if let Some(c) = b.cmd(i) {
                    appsrt_hook_lists.push(c.cmd_list.clone());
                }
            }
            ps.load_bank(0, Arc::new(b), None);
        }
        let mut effect_banks: Vec<Option<crate::effect::EffectBank>> = (0..51).map(|_| None).collect();
        effect_banks[0] = Some(efco);
        // The fighters' own banks (efAsync_LoadAsync at fighter load: ftCa / ftFx OnLoad), each
        // registering its particle bank under the same index.
        for name in fighters {
            let (file, table, idx) = match *name {
                "falcon" => ("EfCaData.dat", "effCaptainDataTable", 4),
                "fox" => ("EfFxData.dat", "effFoxDataTable", 3),
                // Falco has no bank of his own: he loads Fox's (once, see below).
                "falco" => ("EfFxData.dat", "effFoxDataTable", 3),
                "marth" => ("EfMsData.dat", "effMarsDataTable", 16),
                "sheik" => ("EfZdData.dat", "effZeldaDataTable", 17),
                "peach" => ("EfPeData.dat", "effPeachDataTable", 15),
                "samus" => ("EfSsData.dat", "effSamusDataTable", 2),
                // Wolf's own bank takes Fox's index: a match here has one Fox-kind fighter.
                "wolf" => ("EfWfData.dat", "effWolfDataTable", 3),
                n => return Err(format!("effect bank of {n:?} not wired")),
            };
            if effect_banks[idx].is_some() {
                continue;
            }
            let b = crate::effect::EffectBank::load(&disc.read(file)?, file, table)?;
            if let Some(pb) = b.particles()? {
                ps.load_bank(idx, Arc::new(pb), None);
            }
            effect_banks[idx] = Some(b);
        }
        // The VS scene's OnEnter: ifStatus_802F6EA4(1, .., fn_8016B7B4, fn_8016B7F8), the "Ready"
        // HUD (IfAll.usd: the English disc's language file). Its JObj animations are played to
        // frame 0 at creation and stepped once per frame by its s_link 0 proc (if_802F73C4) from
        // golden frame 0, so they end, and fn_8016B7F8 unlocks the fighters' input
        // (ftLib_800868A4), in the proc run of golden frame ceil(end) - 1.
        if std::env::var_os("MELEE_GOLDEN_PADTEST").is_some() {
            let mut pads = [input::PadStatus::default(); 2];
            for g in 0..1801 {
                for port in 0..2 {
                    pads[port].renew(&input::pad_fixed(port, g));
                    let q = &pads[port];
                    eprintln!("{} PAD p{} btn {:08X} st {:08X} {:08X} sub {:08X} {:08X} aL {:08X} aR {:08X}", g, port, q.button,
                        q.nml_stick_x.to_bits(), q.nml_stick_y.to_bits(), q.nml_sub_x.to_bits(), q.nml_sub_y.to_bits(),
                        q.nml_analog_l.to_bits(), q.nml_analog_r.to_bits());
                }
            }
        }
        let go_frame = {
            let ifall = disc.read("IfAll.usd")?;
            match input::ready_hud_end_frame(&ifall)? {
                Some(end) => end.ceil() as i32 - 1,
                None => i32::MAX,
            }
        };

        // grLast_OnInit -> grLast_8021A9C4 (map gobj 3) -> grLast_8021B920(1): the level gobjs
        // 4..8 are created in order; gobj 7's on_init is grLast_8021AC30.
        let mut items0: Vec<fox::SimItem> = Vec::new();
        let mut greens_draws: Option<greens::InitDraws> = None;
        // gmvs.c fn_8016D... -> it_8026D018 (before the stage's OnInit): the random item spawner's
        // first timer.
        let item_timer = match item_freq {
            Some(f) => Some(item_spawner_init(disc, &stage, stkind, f, &mut rand)?),
            None => None,
        };
        let last_bg = if stkind == STKIND_DL {
            let (items, draws) = greens_init(&stage, &mut rand, f32::from_bits(u32::from_be_bytes(stage.data[crate::stage::ground_param(&stage).ok_or("no grGroundParam")? as usize..][..4].try_into().unwrap())))?;
            items0 = items;
            greens_draws = Some(draws);
            None
        } else if stkind == STKIND_YS {
            // grStory_801E3030: the stage gobjs 1, 3, 2 are set up in that order. Only the Shy Guys'
            // (gobj 3, grStory_801E3234) reset_shyguy_timer draws: randi(timer_min..timer_rand) before
            // the constant 120 overwrites it.
            let y = crate::stage::yakumono_words(&stage, 3).ok_or("GrSt.dat: no yakumono_param")?;
            let timer_rand = f32::from_bits(y[1]) as i32;
            let _ = rand.randi(timer_rand);
            None
        } else if stkind == STKIND_BF {
            // grBattle_OnInit: grBattle_80219D84(3) -> grBattle_BG_Callback0: reset_bg_timer,
            // `HSD_Randi(1200) + 2400` (the first background change is 2400+ frames away).
            let _ = rand.randi(1200);
            None
        } else {
            let r = std::cell::RefCell::new(&mut rand);
            Some(LastBg::init(&mut || r.borrow_mut().randf(), &mut |m| r.borrow_mut().randi(m)))
        };
        // grLast_8021B920(1) starts these animations (grNLa_803E8010 rows {gobj, joint, joint,
        // anim}): grAnime_801C7FF8(lv, 2, 7, anim, 0, 1) and grAnime_801C8098(lv, 1, 7, 0, 0, 1)
        // for map gobjs 4..8. Their particle tracks run from the first match frame. Which joints
        // of each tree get the animation (grAnime_801C7C1C's `&aj[joint]`) is not modelled: every
        // particle track of those animations is run (FD has one, gobj 4 anim 0).
        // TODO(group2 granime).
        let mut dptcl = Vec::new();
        let mut joint_mtx = HashMap::new();
        let mut seen = std::collections::HashSet::new();
        // FD's grLast animation table; Battlefield's grBattle_OnInit starts gobj 0 and 1's anim 0
        // (grBattle_GObj0/1_Callback0 -> grAnime_801C8138), whose particle tracks are the 4 stage gens.
        let dptcl_rows: &[(u32, u32)] = if stkind == STKIND_YS { &[(1, 0), (3, 0), (3, 1), (2, 0)] } else if stkind == STKIND_DL { &[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)] } else if stkind == STKIND_BF { &[(0, 0), (1, 0)] } else { &[(4, 0), (5, 4), (6, 6), (7, 8), (8, 10), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0)] };
        for &(g, anim) in dptcl_rows {
            if !seen.insert((g, anim)) {
                continue;
            }
            for t in crate::stage::dptcl_tracks(&stage, g, anim) {
                let mut a: HsdAObj<()> = HsdAObj::load_desc(&t.aobj, |_| None);
                a.req_anim(0.0);
                a.set_rate(1.0);
                joint_mtx.insert(t.joint, crate::stage::bind_pose_matrix(&stage.data, &t.chain));
                dptcl.push(DptclAnim { joint: t.joint, aobj: a, started: false, gobj: g, skip: false });
            }
        }

        // mpLibLoad(stage_info.coll_data) with Ground_801C0498 (grGroundParam->y), then
        // mpLib_80058820 (the map-collision gobj and its bounds).
        let mut cg = CollGlobals::default();
        let mut ground = {
            let cd = stage.public_offset("coll_data").ok_or("stage dat: no coll_data")?;
            let gp = crate::stage::ground_param(&stage).ok_or("stage dat: no grGroundParam")?;
            let scale = f32::from_bits(u32::from_be_bytes(stage.data[gp as usize..][..4].try_into().unwrap()));
            mc::mp_lib_load(&stage.data, cd, scale)
        };
        ground.mp_lib_80058820();
        if stkind == STKIND_DL {
            let gp = crate::stage::ground_param(&stage).ok_or("stage dat: no grGroundParam")?;
            let scale = f32::from_bits(u32::from_be_bytes(stage.data[gp as usize..][..4].try_into().unwrap()));
            greens_sync_joints(&stage, &mut ground, scale, &items0)?;
        }
        // Fighter_804D64FC = pData[22]; ->x20: float[kind] (distance thresholds).
        let cpu_dist: Vec<f32> = {
            let t = be(be(common_root + 4 * 22) + 0x20);
            (0..40).map(|k| f32::from_bits(be(t + 4 * k))).collect()
        };
        let mut players = Vec::new();
        // Player_80031AD0: the main fighter, then (Sheik / Zelda) the partner fighter of the
        // transformation, created hidden (`has_transformation`: ftCo_800BFD04, Sleep).
        // fn_8016DEEC (the multi-player spawn pre-pass): each fighter faces the nearest other one
        // (largest |dx| wins ties by order); within 5 units it faces away from the other's facing
        // (+1 when that is unset).
        let mut facings = vec![0.0f32; fighters.len()];
        {
            let xs: Vec<f32> = (0..fighters.len())
                .map(|i| points.iter().find(|p| p.id == i as i32).map_or(0.0, |p| p.pos[0]))
                .collect();
            for i in 0..fighters.len() {
                let mut best: Option<(usize, f32)> = None;
                for j in 0..fighters.len() {
                    if j != i && best.map_or(true, |(_, d)| d < (xs[i] - xs[j]).abs()) {
                        best = Some((j, (xs[i] - xs[j]).abs()));
                    }
                }
                facings[i] = match best {
                    None => 1.0,
                    Some((j, _)) => {
                        let dx = xs[j] - xs[i];
                        if dx < -5.0 {
                            -1.0
                        } else if dx > 5.0 {
                            1.0
                        } else if facings[j] == 1.0 {
                            -1.0
                        } else {
                            1.0
                        }
                    }
                };
            }
        }
        let mut roster: Vec<(i32, &str, bool)> = Vec::new();
        for (i, name) in fighters.iter().enumerate() {
            roster.push((i as i32, name, false));
            if *name == "sheik" {
                roster.push((i as i32, "zelda", true));
            }
        }
        for &(slot, name, follower) in &roster {
            let sp = points.iter().find(|p| p.id == slot).ok_or(format!("{stage_file}: no spawn point {slot}"))?;
            if !sp.exact {
                eprintln!("melee: spawn point {slot} is under a rotated/scaled joint: position not bit-exact (TODO group1 jobj)");
            }
            let files = crate::fighter_data::FighterFiles::load(disc, name)?;
            if std::env::var_os("MELEE_GOLDEN_SCRIPTS").is_some() {
                // Debug: the command words of the rows the match reaches (opcode = word >> 26).
                const LEN: [u8; 49] = [5, 5, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 7, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 2, 1, 4];
                for row in [238u32, 20, 35, 10, 2, 3, 7, 8, 9] {
                    let mut at = files.script_at(row);
                    let mut out = Vec::new();
                    while at != 0 && out.len() < 40 {
                        let w = files.word(at);
                        let op = w >> 26;
                        let n = if op < 10 { if op == 5 || op == 7 { 2 } else { 1 } } else { LEN.get((op - 10) as usize).copied().unwrap_or(1) as u32 };
                        out.push(format!("{op:#x}:{w:08X}{}", if n > 1 { format!("+{}", n - 1) } else { String::new() }));
                        if op == 0 { break; }
                        at += 4 * n;
                    }
                    eprintln!("{name} row {row}: {}", out.join(" "));
                }
            }
            // Fighter_Create: the costume's joint tree (HSD_JObjLoadJoint) and ftParts_SetupParts;
            // ftPartsTable[kind]->parts_num (PlCo.dat pData[4]) checks the part walk.
            let kind = fighter_kind(name)?;
            let part_to_joint;
            let skel = {
                let (nr, sym) = crate::fighter_data::costume_for(name).ok_or("no costume file")?;
                let a = DatArchive::parse(&disc.read(nr)?).map_err(|e| format!("{nr}: {e}"))?;
                let at = a.public_offset(sym).ok_or(format!("{nr}: no {sym}"))?;
                let tables = be(common_root + 0x10);
                // ftPartsTable is indexed by the INTERNAL fighter id: an m-ex fighter's is its MxDt.dat
                // row (m-ex inserts its fighters before Master Hand), not the vanilla kind it runs as.
                let pt = be(tables + 4 * crate::mex::internal_id(disc, name)?.unwrap_or(kind as u32));
                // ftParts_80074E58's flags_b4 parts: part_to_joint[FtPart_TransN] and [0x35].
                let p2j = be(pt + 4) as usize;
                let b4 = [plco.data[p2j + 1] as usize, plco.data[p2j + 0x35] as usize];
                part_to_joint = plco.data[p2j..p2j + 0x40].to_vec();
                Skeleton::load(&a.data, at, files.x8_x10(), files.model_scaling(), be(pt + 8), b4)?
            };
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            fp.kind = kind;
            // Fighter_UnkInitLoad: fp->co_attrs = *ft_data->x0.
            fp.co_attrs = files.attrs();
            let (n0, n1) = files.x2c4();
            fp.x2C4 = melee_types::Vec2 { x: n0, y: n1 };
            fp.player_id = slot as u8;
            // fighter.c (Fighter_UnkInitReset): no attack yet (ft_800890BC), the damage multipliers.
            melee_fighter::stale::clear_attack_id(&mut fp);
            fp.dmg.x182c_behavior = 1.0;
            fp.x1960_vibrateMult = 1.0;
            fp.dmg.x1908 = -1;
            fp.dmg.x18C8 = -1;
            fp.dmg.x18ac_time_since_hit = -1;
            fp.dmg.x18c4_source_ply = 6;
            fp.dmg.x18D0 = -10;
            fp.dmg.x18F8 = 1;
            // fighter.c Fighter_UnkInitReset/Fighter_ResetInputData_80068854 defaults that the hash reads.
            fp.shield_health = start_shield;
            fp.x680 = 255;
            fp.x67F = 255;
            // Fighter_UnkInitReset: no last attacker (fighter.c 455, 654).
            fp.dmg.x18c4_source_ply = 6;
            fp.x2180 = 6;
            fp.cur_pos.x = sp.pos[0];
            fp.cur_pos.y = sp.pos[1];
            fp.cur_pos.z = sp.pos[2];
            fp.prev_pos = fp.cur_pos;
            fp.facing_dir = facings[slot as usize];
            // Fighter_UnkInitLoad_80068914: x34_scale.x = Player_GetModelScale (1 in a VS match);
            // Fighter_UnkInitReset: y = x; Fighter_80068E64: z = 1 off Flat Zone.
            fp.x34_scale.x = 1.0;
            fp.x34_scale.y = fp.x34_scale.x;
            fp.x34_scale.z = 1.0;
            let mut skel = skel;
            // Fighter_Create: ftCo_8009CF84 runs on the bind-pose skeleton at the origin (before the
            // spawn translate), so the chains' world positions (unk_2C) start from that pose.
            let dynamics = crate::dynamics::Dynamics::load(&files.dat.data, files.ft_data, &mut skel);
            // Fighter_UnkProcessDeath_80068354: HSD_JObjSetTranslate(jobj, cur_pos) first; the
            // model scale comes after the spawn probe below.
            let root = skel.root;
            skel.set_translate(root, Vec3 { x: fp.cur_pos.x, y: fp.cur_pos.y, z: fp.cur_pos.z });
            if !follower {
                stale::set_stocks(slot, stocks);
                // gmvs.c: Player_SetSlottype, SetPlayerAndEntityCpuType(gmvsmode.c: cpu_kind 4) / Level.
                if let Some(level) = cpu_levels[slot as usize] {
                    stale::set_slot_type(slot, melee_types::Gm_PKind_Cpu);
                    melee_fighter::player::set_player_and_entity_cpu_type(slot, 4);
                    melee_fighter::player::set_player_and_entity_cpu_level(slot, level);
                }
            }
            // Fighter_UnkProcessDeath_80068354 -> ftCo_800A101C(fp, Player_GetCpuType,
            // Player_GetCpuLevel, 0): two HSD_Randf. The co_attrs it reads for x558 are not loaded
            // (x558 is not hashed).
            {
                let cpu_type = melee_fighter::player::get_cpu_type(slot);
                let cpu_level = melee_fighter::player::get_cpu_level(slot);
                melee_fighter::ftcpu::ftco_800a101c(&mut fp, cpu_type, cpu_level, 0, None, None, &mut || rand.randf());
            }
            // The Entry state starts airborne (ported ftCommon_8007D5D4: jumps used 1, ECB lock).
            let ecb_desc = files.ecb_desc();
            // ft_80081B38: CollData init with the bone ECB source (ftData->x44).
            {
                fp.ecb_lock = 0;
                fp.coll_data.cur_pos = fp.cur_pos;
                fp.coll_data.last_pos = fp.coll_data.cur_pos;
                mc::mp_coll_80041ee4(&mut fp.coll_data, &cg);
                fp.coll_data.x34_flags.set_b1234(1);
                mc::mp_coll_set_ecb_source_jobj(&mut fp.coll_data, ecb_desc.unk_c * fp.x34_scale.y);
                let t = fp.x34_scale.y;
                mc::mp_coll_set_ledge_snap(&mut fp.coll_data, ecb_desc.ledge_snap_x * t, ecb_desc.ledge_snap_y * t, ecb_desc.ledge_snap_height * t);
                fp.coll_data.x50 = fp.co_attrs.weight;
                let v = 10.0f32 * fp.x34_scale.y;
                fp.coll_data.ecb_source.x128 = v;
                fp.coll_data.ecb_source.x12C = v;
            }
            // ft_80081938 writes dmg.x1930 (item-pickup box; not hashed, no items here).
            // smash_attrs.x2135 == -1: ft_80082A68, the floor probe 10 units either side.
            {
                fp.coll_data.cur_pos = fp.cur_pos;
                fp.coll_data.last_pos = fp.coll_data.cur_pos;
                fp.coll_data.last_pos.y += 10.0;
                fp.coll_data.cur_pos.y -= 10.0;
                let joints = ecb_joints(&mut skel, &ecb_desc);
                let hit = mc::mp_coll_800471f8(&mut ground, &mut cg, &mut fp.coll_data, &joints).map_err(|e| format!("spawn probe: {} not ported", e.0))?;
                if hit {
                    return Err("spawn probe found a floor: ftCommon_8007D6A4 (grounded start) not wired".into());
                }
            }
            // The Entry state starts airborne (ported ftCommon_8007D5D4: jumps used 1, ECB lock).
            melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut fp);
            {
                // HSD_JObjSetTranslate again, then Fighter_UpdateModelScale (GetModelScale; x from
                // x34_scale.z when != 1).
                skel.set_translate(root, Vec3 { x: fp.cur_pos.x, y: fp.cur_pos.y, z: fp.cur_pos.z });
                let ms = fp.x34_scale.y * fp.co_attrs.model_scaling;
                let sx = if fp.x34_scale.z != 1.0 { fp.x34_scale.z } else { ms };
                skel.set_scale(root, Vec3 { x: sx, y: ms, z: ms });
            }
            if std::env::var_os("MELEE_GOLDEN_DUMP").is_some() {
                eprintln!("p{} kind {} motion {} pos {:?} facing {} ga {} shield {} fc {} jumps {} 67F {} 680 {} stocks {}",
                    slot + 1, fp.kind, fp.motion_id, (fp.cur_pos.x, fp.cur_pos.y, fp.cur_pos.z), fp.facing_dir, fp.ground_or_air,
                    fp.shield_health, fp.x3E4_fighterCmdScript.frame_count, fp.x1968_jumpsUsed, fp.x67F, fp.x680, stale::stocks(slot));
            }
            // ftCo_800C61B0 (Entry_Enter): timer, the squashed root scale, entry.x4 = cur_pos.y.
            let x8 = skel.tree.get(root).scale;
            let x14 = Vec3 { x: x8.x, y: melee_fighter::hooks::common_data().x6C4, z: x8.z };
            skel.set_scale(root, x14);
            let entry = EntryVars { timer: 5 * (slot + 1), x4: fp.cur_pos.y, x8, x14, ..Default::default() };
            // Fighter_Create -> Mex_OnLoadDispatch (fighter.c TARGET_PC, after ftParts_SetupParts and
            // the anim/colour setup): an m-ex fighter's onLoad override (slot 0).
            let mut mex = match crate::fighter_data::files_for(name) {
                Some((file, data_name, _)) => crate::mex::MexFighter::load(file, data_name, &disc.read(file)?)?,
                None => None,
            };
            if let Some(m) = &mut mex {
                m.run(crate::mex::SLOT_ON_LOAD, slot, &mut fp);
            }
            let sk_needle = if kind == melee_types::Ft_Kind_Seak { Some(sheik::NeedleAttrs::from_dat(&files.dat.data, files.ft_data)) } else { None };
            let ss_attrs = if kind == melee_types::Ft_Kind_Samus { Some(samus::SsAttrs::from_dat(&files.dat.data, files.ft_data)) } else { None };
            let sk_attrs = if kind == melee_types::Ft_Kind_Seak {
                let x4 = be32(&files.dat.data, files.ft_data + 4);
                Some(*files.dat.load_at::<melee_types::ftSeakAttributes>(x4).get())
            } else {
                None
            };
            players.push(Player {
                dynamics,
                gc: std::ptr::null_mut(),
                common_c: Default::default(),
                slot,
                follower,
                hurtboxes: files.hurtboxes(),
                fp,
                gobj: Box::new(melee_types::HSD_GObj::zeroed()),
                mex,
                entry,
                skel,
                animating: false,
                fall_smid: 0,
                fall_x4: 0.0,
                row_entry_start: files.anim_row(FTCO_SM_ENTRY_START)?.zip(files.figatree(FTCO_SM_ENTRY_START)?),
                row_fall: files.anim_row(FTCO_SM_FALL)?.zip(files.figatree(FTCO_SM_FALL)?),
                row_landing: files.anim_row(FTCO_SM_LANDING)?.zip(files.figatree(FTCO_SM_LANDING)?),
                row_turn: files.anim_row(FTCO_SM_TURN)?.zip(files.figatree(FTCO_SM_TURN)?),
                row_wait: files.anim_row(FTCO_SM_WAIT)?.zip(files.figatree(FTCO_SM_WAIT)?),
                row_walk: [
                    files.anim_row(FTCO_SM_WALK_SLOW)?.zip(files.figatree(FTCO_SM_WALK_SLOW)?),
                    files.anim_row(FTCO_SM_WALK_SLOW + 1)?.zip(files.figatree(FTCO_SM_WALK_SLOW + 1)?),
                    files.anim_row(FTCO_SM_WALK_SLOW + 2)?.zip(files.figatree(FTCO_SM_WALK_SLOW + 2)?),
                ],
                walk: WalkVars::default(),
                turn: TurnVars::default(),
                landing_allow_interrupt: false,
                ecb_desc,
                trophy_scale: files.trophy_scale(),
                model_scaling: files.model_scaling(),
                ef_queue: Vec::new(),
                ef_ops: Vec::new(),
                pdat: Arc::new(files.dat.data.clone()),
                script: 0,
                script_sp: 0,
                script_ret: [0; 8],
                part_anim: files.part_anim_data(),
                x221d_b3: false,
                input_locked: false,
                ca_attrs: if kind == Ft_Kind_Captain {
                    let x4 = be32(&files.dat.data, files.ft_data + 4);
                    Some(*files.dat.load_at::<melee_types::ftCaptain_DatAttrs>(x4).get())
                } else {
                    None
                },
                fx_attrs: if is_fx_kind(kind) {
                    let x4 = be32(&files.dat.data, files.ft_data + 4);
                    Some(*files.dat.load_at::<melee_types::ftFox_DatAttrs>(x4).get())
                } else {
                    None
                },
                wait_anims: {
                    let d = &files.dat.data;
                    let at = be32(d, files.ft_data + 0x24);
                    let mut v = Vec::new();
                    if at != 0 {
                        let mut q = at;
                        loop {
                            let id = be32(d, q) as i32;
                            if id == -1 {
                                break;
                            }
                            v.push((id, be32(d, q + 4) as i32));
                            q += 8;
                        }
                    }
                    v
                },
                files: Arc::new(files),
                rows: HashMap::new(),
                part_to_joint,
                ca_lw_x0: 0.0,
                ca_lw_friction: 1.0,
                hitboxes: [None; 4],
                attack1_x0: false,
                dead_timer: 0,
                hud_randomize: false,
                hud: hit::HudDamage::default(),
                guard: GuardVars::default(),
                jump: JumpVars::default(),
                dash: dashrun::DashVars::default(),
                run: dashrun::RunVars::default(),
                fx: FoxVars::default(),
                ss: Default::default(),
                ss_attrs,
                sk: sheik::SeakVars::default(),
                sk_attrs,
                sk_needle,
                item_cmds: Vec::new(),
                keep_gfx: false,
                blend_zero: false,
                rb: PlayerRespawn::default(),
            });
            // ... Fighter_ChangeMotionState(Entry): no animation, cur_anim_frame -1;
            // ftaction.c:1408 frame_count = cur_anim_frame + x898_unk. Then ft_80084CB0: the
            // entry box is the ECB the probe left.
            let p = players.last_mut().unwrap();
            p.gobj.user_data = &mut *p.fp as *mut Fighter as *mut core::ffi::c_void;
            // ftColl_8007B320: the hurt capsules from ftData->x30.
            p.fp.hurt_capsules_len = p.hurtboxes.len().min(p.fp.hurt_capsules.len()) as _;
            for (i, (h, gr)) in p.files.hurtbox_meta().into_iter().enumerate().take(p.fp.hurt_capsules.len()) {
                let (part, a, b, sc) = p.hurtboxes[i];
                let c = &mut p.fp.hurt_capsules[i];
                c.capsule.bone_idx = part as _;
                c.height = h as _;
                c.is_grabbable = gr as _;
                c.capsule.state = melee_types::HurtCapsule_Enabled as _;
                c.capsule.a_offset = melee_types::Vec { x: a[0], y: a[1], z: a[2] };
                c.capsule.b_offset = melee_types::Vec { x: b[0], y: b[1], z: b[2] };
                c.capsule.scale = sc;
            }
            p.fp.gobj = &mut *p.gobj;
            // Fighter_Create_Inline2: x2DC/x2E0/x2E4 = the Walk rows' lengths (lbAnim_8001E8F8).
            p.fp.x2DC = p.row_walk[0].as_ref().map_or(0.0, |r| r.0.frames);
            p.fp.x2E0 = p.row_walk[1].as_ref().map_or(0.0, |r| r.0.frames);
            p.fp.x2E4 = p.row_walk[2].as_ref().map_or(0.0, |r| r.0.frames);
            // fighter.c 1063: x2E8 = lbAnim_8001E8F8(ftData_80085E50(fp, 0x25)), the GuardOn row.
            p.fp.x2E8 = load_row(p, FTCO_SM_GUARD_ON)?.map_or(0.0, |r| r.0.frames);
            // Fighter_Create -> ftLib_800867E8: Fighter_ResetInputData_80068854, x221D_b4 = true.
            input::reset_input(&mut p.fp);
            p.input_locked = true;
            if follower {
                // ftCo_800BFD04: Sleep, hidden, inactive camera.
                change_motion_state(p, melee_types::ftCo_MS_Sleep as i32, None, &mut rand)?;
                p.fp.set_invisible(1);
                p.fp.set_x221E_b1(1);
                p.fp.set_x221E_b2(1);
                p.fp.set_x2219_b1(1);
                p.fp.set_x221F_b3(1);
                p.fp.set_x221F_b1(1);
                p.animating = false;
                continue;
            }
            change_motion_state(p, FTCO_MS_ENTRY, None, &mut rand)?;
            // ftCo_Entry_Enter (ft_0C31.c:42).
            p.fp.set_invisible(1);
            p.fp.set_x2219_b1(1);
            p.fp.set_x221E_b1(1);
            p.fp.set_x221E_b2(1);
            p.fp.set_x221F_b1(1);
            p.fp.x3E4_fighterCmdScript.frame_count = p.fp.cur_anim_frame + p.fp.x898_unk;
            let e = p.fp.coll_data.ecb;
            p.entry.x2c = ftCollisionBox { top: e.top.y, bottom: e.bottom.y, left: e.left, right: e.right };
        }

        // gmvs.c -> Stage_80225074(fn_8016E5C0(...)): no alternate-music button held -> 0 -> 4.
        let param = crate::stage::ground_param(&stage).ok_or("stage dat: no grGroundParam")?;
        let rows = StageParam::parse_all(&stage.data, param);
        let pick = bgm::ground_801c24f8(&rows, stkind, bgm::stage_80225074_flags(0), &SaveUnlocks, &mut |m| rand.randi(m));
        if std::env::var_os("MELEE_GOLDEN_DUMP").is_some() {
            eprintln!("bgm {} alt {:?}; seed before frame 0 {:08X}; particle tracks {}", pick.bgm, pick.alt_flag, rand.logic_seed(), dptcl.len());
        }

        if std::env::var_os("MELEE_GOLDEN_RAND").is_some() {
            // Diagnostic: every draw as `R <golden frame> <seed after>` (compare with the C build's
            // MELEE_RAND_TRACE, whose script frame is the golden frame + 29).
            rand.trace = Some(|seed, stream| {
                if stream != 2 {
                    eprintln!("R {} {:08X}", unsafe { TRACE_FRAME }, seed);
                }
            });
        }
        let story_state = if stkind == STKIND_YS { Some(story::Story::new(std::sync::Arc::new(DatArchive::parse(&stage_bytes).map_err(|e| format!("{stage_file}: {e}"))?), &mut joint_mtx)?) } else { None };
        Ok(Self { item_timer, players, ground, cg, pads: [input::PadStatus::default(); 4], go_frame, rand, ps, frame: -1, stopped: None, bgm: pick.bgm as i32, last_bg, dptcl, story: story_state, greens: greens_draws.map(move |d| greens::Greens::new(std::sync::Arc::new(stage), d)), aobj_globals: AObjGlobals::default(), joint_mtx, common, effect_banks, effects: Vec::new(), next_effect_instance: 1, appsrt_hook_lists, blast, fighter_jobjs: Vec::new(), items: items0, cpu_dist, plco_bytes: std::sync::Arc::from(&plco.data[..]), cpu_tbl: be(common_root + 4 * 22), grkind: if stkind == STKIND_DL { Gr_Kind_Greens as i32 } else if stkind == STKIND_YS { Gr_Kind_Story as i32 } else if stkind == STKIND_BF { Gr_Kind_Battle as i32 } else { Gr_Kind_Last as i32 }, respawn_points: (4..8).filter_map(|id| points.iter().find(|p| p.id == id).map(|p| Vec3 { x: p.pos[0], y: p.pos[1], z: p.pos[2] })).collect(), respawn_busy: [0; 6], start_shield, per_slot_spawn: stkind == STKIND_BF, dmg_err: None })
    }

    /// The m-ex fighters' code-run report (crate::mex), one block per such fighter.
    pub fn mex_report(&self) -> String {
        self.players.iter().filter_map(|p| p.mex.as_ref().map(|m| format!("p{}: {}", p.slot + 1, m.report()))).collect()
    }

    fn words(&self) -> FrameWords {
        let (n_items, item_sum) = self.item_words();
        let mut w = FrameWords { frame: self.frame as u32, seed: self.rand.logic_seed(), items: n_items, itemsum: item_sum, ..Default::default() };
        for p in &self.players {
            // Slot = player*2 + follower; is_cpu false (human scenario).
            let s = p.slot * 2 + p.follower as i32;
            w.slots[s as usize] = Some(nh_fighter(&p.fp, s, stale::stocks(p.slot), stale::slot_type(p.slot) == melee_types::Gm_PKind_Cpu && p.fp.cpu.kind != 5));
            if std::env::var_os("MELEE_GOLDEN_CPU").is_some() && stale::slot_type(p.slot) == melee_types::Gm_PKind_Cpu {
                let f = &p.fp;
                let c = &f.cpu;
                let off = if c.csP.is_null() { 0 } else { (c.csP as usize).wrapping_sub(c.buffer.as_ptr() as usize) };
                let fl = u32::from_be_bytes(c.bf_f8);
                eprintln!("NHC s{} m{} p {:08X} {:08X} v {:08X} {:08X} fc {:08X} b {:X} st {:02X}{:02X} cs {:02X}{:02X} k{} x18 {} x30 {} x34 {} x38 {:08X} x54 {:08X} {:08X} x5C {:08X} h {:08X} {:08X} {:08X} {:08X} x60 {} x7C {} x80 {} x84 {} xC8 {} xEC {} cd {} off {} fl {:08X} xcc {} {} {} {} H {:08X}",
                    s, f.motion_id, f.cur_pos.x.to_bits(), f.cur_pos.y.to_bits(), f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.x3E4_fighterCmdScript.frame_count.to_bits(), c.buttons,
                    c.lstick.x as u8, c.lstick.y as u8, c.cstick.x as u8, c.cstick.y as u8, c.kind, c.x18, c.x30, c.x34, c.x38.to_bits(), c.x54.x.to_bits(), c.x54.y.to_bits(), c.x5C.to_bits(), c.x55C.to_bits(), c.x560.to_bits(), c.x564.to_bits(), c.x568.to_bits(), c.x60, c.x7C, c.x80, c.x84, c.xC8, c.xEC, c.command_duration, off, fl, c.xCC_array[0], c.xCC_array[1], c.xCC_array[2], c.xCC_array[3], w.slots[s as usize].unwrap_or(0));
            }
            if std::env::var_os("MELEE_GOLDEN_PJ").is_some() && p.slot == 0 {
                static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                if n <= 8 {
                    for q in 0..70usize.min(p.skel.parts.len()) {
                        let o = p.skel.tree.get(p.skel.part(q));
                        eprintln!("PJ n{} q{} r {:08X} {:08X} {:08X} t {:08X} {:08X} {:08X}", n, q, o.rotate.x.to_bits(), o.rotate.y.to_bits(), o.rotate.z.to_bits(), o.translate.x.to_bits(), o.translate.y.to_bits(), o.translate.z.to_bits());
                    }
                }
            }
            if std::env::var_os("MELEE_GOLDEN_FRAMES").is_some() {
                let f = &p.fp;
                eprintln!("NHD f{} s{} m{} pos {:08X} {:08X} {:08X} vel {:08X} {:08X} ga {} fc {:08X} j {} 67F {} 680 {} shield {:08X}",
                    self.frame, s, f.motion_id, f.cur_pos.x.to_bits(), f.cur_pos.y.to_bits(), f.cur_pos.z.to_bits(),
                    f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.ground_or_air, f.x3E4_fighterCmdScript.frame_count.to_bits(),
                    f.x1968_jumpsUsed, f.x67F, f.x680, f.shield_health.to_bits());
                eprintln!("NHE f{} s{} fac {} pct {:08X} hl {:08X} b5 {} kb {:08X} {:08X} gr {:08X}", self.frame, s, f.facing_dir, f.dmg.x1830_percent.to_bits(), f.dmg.x195c_hitlag_frames.to_bits(), f.x2219_b5(), f.x8c_kb_vel.x.to_bits(), f.x8c_kb_vel.y.to_bits(), f.gr_vel.to_bits());
            }
        }
        w
    }

    /// `Ground_801C1CD0` for the stage's particle tracks: `HSD_AObjInterpretAnim`, and for each
    /// 0x28 key `JObjUpdateFunc` -> `efLib_Cb_DPtcl(0, iv & 0x3F, (iv >> 6) & 0xFFFFFF, jobj)`.
    fn stage_anim(&mut self) -> Result<(), String> {
        let mut spawns = Vec::new();
        for t in &mut self.dptcl {
            if t.skip {
                t.skip = false;
                continue;
            }
            let joint = t.joint;
            let mut cb = |ty: u8, fv: f32| {
                if ty == 0x28 {
                    let iv = fv.to_bits() as i32;
                    spawns.push((iv & 0x3F, (iv >> 6) & 0xFF_FFFF, joint));
                }
            };
            let mut upd: melee_runtime::fobj::ObjUpdate<'_> = Some(&mut cb);
            // grAnime_801C8138 ends with HSD_JObjAnimAll: the first interpretation (the
            // FIRST_PLAY hold at frame 0, which spawns the frame-0 particle tracks) happens at
            // init, so the animation is one frame ahead of the match frame from frame 0 on.
            if !t.started {
                t.started = true;
                t.aobj.interpret_anim(&mut self.aobj_globals, &mut upd);
            }
            t.aobj.interpret_anim(&mut self.aobj_globals, &mut upd);
        }
        self.dptcl_spawns(spawns, None)
    }

    /// The fighters' s_link 1..9 procs for one frame, in proc order (every fighter's s_link 1
    /// proc, then every fighter's s_link 3 proc, ...). Err: the frame needs an unported state.
    fn fighters(&mut self) -> Result<(), String> {
        let c = self.common;
        for p in &mut self.players {
            p.gc = &mut self.ground;
            p.common_c = c;
        }
        // Ground_801C1CD0 (every map gobj's s_link 1 proc): `mpColl_804D64AC += 1`, so each
        // CollData's first substep of the frame (x38 != the counter) takes the Remap variants.
        self.cg.x804d64ac = self.cg.x804d64ac.wrapping_add(1);
        self.greens_step_anim();
        // The frame's HSD_PadRenewStatus (the pad script's sample for this frame).
        let g = self.frame + 1;
        for port in 0..4 {
            let raw = input::pad_fixed(port, g);
            self.pads[port].renew(&raw);
        }
        // fn_8016758C (the VS scene's per-frame proc): the respawn spots' countdowns.
        for b in &mut self.respawn_busy {
            if *b != 0 {
                *b -= 1;
            }
        }
        // fn_8026C88C (the random item spawner's proc, s_link 0): the countdown; the spawn itself
        // (it_8026C75C & co.) is not ported.
        if let Some(t) = &mut self.item_timer {
            *t -= 1;
            if *t == 0 {
                return Err("fn_8026C88C: the random item spawn (it_8026C75C, it_8026CB3C) is not ported".into());
            }
        }
        // s_link 0, the Ready HUD's proc (p_link 14, after the fighters' s_link 0 proc
        // Fighter_8006A1BC, which has nothing to do here): at its end, GO.
        if g == self.go_frame {
            for p in &mut self.players {
                p.input_locked = false;
            }
        }
        if g == self.go_frame {
            self.greens_start();
        }
        let sp: *mut VsMatch = self;
        // s_link 0, Fighter_8006A1BC: the hitlag countdown.
        self.hitlag_countdown()?;
        // s_link 1, Fighter_8006A360: the animation step (ftAnim_8006EBA4: AObj frame, then
        // ftAction_80073240's frame_count = cur_anim_frame + x898), then the state's anim_cb.
        for pi in 0..self.players.len() {
            let p = &mut self.players[pi];
            if p.follower {
                continue;
            }
            if p.fp.x221F_b3() == 0 {
                // Fighter_8006A360: pos_delta = cur_pos - prev_pos; prev_pos = cur_pos.
                let (c, pv) = (p.fp.cur_pos, p.fp.prev_pos);
                p.fp.pos_delta = melee_types::Vec { x: c.x - pv.x, y: c.y - pv.y, z: c.z - pv.z };
                p.fp.prev_pos = c;
            }
            if p.fp.x221F_b3() == 0 {
                // Fighter_8006A360's countdowns: the attacker memory, the invincibility timers.
                let fp = &mut p.fp;
                if fp.dmg.x18C8 != -1 && fp.dmg.x18C8 > 0 {
                    fp.dmg.x18C8 -= 1;
                    if fp.dmg.x18C8 == 0 {
                        fp.dmg.x18c4_source_ply = 6;
                        fp.dmg.x18C8 = -1;
                        fp.dmg.x18D0 = -0xA;
                    }
                }
                if fp.x1990 != 0 {
                    fp.x1990 -= 1;
                    if fp.x1990 == 0 && fp.x2221_b0() == 0 {
                        fp.x198C = if fp.x1994 != 0 { 1 } else { 0 };
                    }
                }
                if fp.x1994 != 0 {
                    fp.x1994 -= 1;
                    if fp.x1994 == 0 {
                        fp.x198C = if fp.x2221_b0() != 0 || fp.x1990 != 0 { 2 } else { 0 };
                    }
                }
                if fp.x1990 != 0 || fp.x1994 != 0 || fp.x198C != 0 {
                    // (ftCo_800C0694 / ftCo_800C0200: the colour animation, render only.)
                }
                if fp.dmg.x18fa_model_shift_frames != 0 {
                    fp.dmg.x18fa_model_shift_frames -= 1;
                    fp.dmg.x18FC += 1;
                    if fp.dmg.x18FC >= fp.dmg.x18FD {
                        fp.dmg.x18FC = 0;
                    }
                }
            }
            'anim: {
            if p.fp.x2219_b5() != 0 {
                break 'anim;
            }
            if p.fp.x209A > 1 && !p.input_locked {
                p.fp.x209A -= 1;
            }
            if p.fp.dmg.x18ac_time_since_hit != -1 {
                p.fp.dmg.x18ac_time_since_hit += 1;
            }
            if p.animating {
                // ftAnim_8006E9B4, no blend (x8A4 == 0), x594_b0 / x594_b2 clear.
                body_anim(p)?;
                p.fp.cur_anim_frame = p.skel.curr_frame().ok_or("ftAnim_8006F3DC: no animated part")?;
            }
            // ftAnim_8006EBA4: ftAction_80073240 (the command script), ftAnim_800707B0 (the x8B0
            // part-anim slots: none started here), ftCo_800DB500 (no item).
            ftaction_80073240(p, &mut self.rand)?;
            p.skel.part_anim_update(&p.part_anim)?;
            // ftCo_800D71D8: the item-grab attempt window (no grabbable item exists here).
            if p.fp.x209C != 0 {
                p.fp.x209C -= 1;
                if p.fp.x209C == 0 {
                    p.fp.set_x2224_b1(1);
                }
            }
            let kind = p.fp.kind;
            let fp = &mut p.fp;
            match fp.motion_id {
                FTCO_MS_ENTRY => {
                    // ftCo_Entry_Anim
                    if p.entry.timer == 0 {
                        entry_start(p, c, &mut self.rand)?;
                    }
                    p.entry.timer -= 1;
                }
                FTCO_MS_ENTRY_START => {
                    // ftCo_EntryStart_Anim
                    p.entry.timer -= 1;
                    if p.entry.timer == 0 {
                        entry_end(p, c, &mut self.rand)?;
                    }
                }
                FTCO_MS_ENTRY_END => {
                    // ftCo_EntryEnd_Anim (Player_GetFlagsBit4 is clear in a plain VS match)
                    p.entry.timer -= 1;
                    if p.entry.timer == 0 {
                        // ftCommon_8007D92C: airborne -> ftCo_Fall_Enter.
                        fall_enter(p, &mut self.rand)?;
                    }
                }
                FTCO_MS_FALL => {
                    // ftCo_Fall_Anim -> ftCo_Fall_Anim_Inner(&fall.x4, Fall, FallF, FallB) and
                    // ftCo_800CC988(fall.x4): with no air drift the blend stays 0 and the row
                    // stays Fall (|self_vel.x / air_drift_max| <= x444 picks the neutral row).
                    fall_anim(p)?;
                }
                FTCO_MS_TURN => {
                    // ftCo_Turn_Anim: ftCo_Turn_Anim_Inner, then Wait when the animation ends.
                    let t = &mut p.turn;
                    if t.frames_to_turn > 0.0 {
                        t.frames_to_turn -= 1.0;
                    } else if !t.has_turned {
                        t.has_turned = true;
                        t.just_turned = true;
                        p.fp.facing_dir = -p.fp.facing_dir;
                    }
                    if !p.skel.frames_remaining() {
                        ft_8008a2bc(p, &mut self.rand)?;
                    }
                }
                m if m == FTCO_MS_LANDING || is_landing_air(m) => {
                    // ftCo_Landing_Anim / ftCo_LandingAir_Anim: when the animation ends, ft_8008A2BC (Wait).
                    if !p.skel.frames_remaining() {
                        ft_8008a2bc(p, &mut self.rand)?;
                    }
                }
                FTCO_MS_WAIT => {
                    // ftCo_Wait_Anim -> ftCo_8008A7A8(gobj, ft_data->x24): a new random idle
                    // animation when this one ends.
                    rebirth::ft_8008a7a8(p, &mut self.rand)?;
                }
                m if (FTCO_MS_WALK_SLOW..=FTCO_MS_WALK_FAST).contains(&m) => {
                    // ftCo_Walk_Anim -> ftWalkCommon_800DFDDC: the animation rate from the speed.
                    let fp = &*p.fp;
                    let mut v = if fd_ground_friction(fp) < 1.0 { p.walk.x0 } else { fp.gr_vel };
                    let rate = if v * fp.facing_dir <= 0.0 {
                        0.0
                    } else {
                        v = fc::abs_c(v);
                        match fp.motion_id - p.walk.msid {
                            0 => v / p.walk.slow_anim_rate,
                            1 => v / p.walk.middle_anim_rate,
                            _ => v / p.walk.fast_anim_rate,
                        }
                    };
                    set_anim_rate(p, rate);
                }
                FTCO_MS_CATCH => {
                    // ftCo_Catch_Anim: fn_800D8EC8 (Link) / fn_800D9558 (Samus) do nothing for
                    // the kinds wired; Wait when the animation ends.
                    if matches!(p.fp.kind, melee_types::Ft_Kind_Link | melee_types::Ft_Kind_CLink | melee_types::Ft_Kind_Samus) {
                        return Err("tether grab (fn_800D8EC8 / fn_800D9558) not ported".into());
                    }
                    if !p.skel.frames_remaining() {
                        ft_8008a2bc(p, &mut self.rand)?;
                    }
                }
                m if is_tilt_motion(m) => {
                    // ftCo_AttackS3_Anim / ftCo_AttackHi3_Anim: ft_8008A2BC at the end.
                    if !p.skel.frames_remaining() {
                        ft_8008a2bc(p, &mut self.rand)?;
                    }
                }
                m if (FTCO_MS_ATTACK11..=FTCO_MS_ATTACK11 + 2).contains(&m) => {
                    // ftCo_Attack11/12/13_Anim: ft_8008A2BC when the animation ends.
                    if !p.skel.frames_remaining() {
                        ft_8008a2bc(p, &mut self.rand)?;
                    }
                }
                FTMS_MS_SPECIAL_LW => ftms_speciallw_anim(p, &mut self.rand)?,
                FTCA_MS_SPECIAL_LW => {
                    // ftCa_SpecialLw_Anim: at the end, SpecialLwEnd (ground) or EndAir.
                    if !p.skel.frames_remaining() {
                        let fp = &mut *p.fp;
                        fp.cmd_vars[2] = 0;
                        fp.cmd_vars[1] = 0;
                        fp.cmd_vars[0] = 0;
                        fp.anon_2210.throw_flags = 0;
                        if fp.ground_or_air == melee_types::GA_Ground {
                            ftcommon_8007d7fc(&mut self.ground, p, c)?;
                            let lag = p.ca_attrs.as_ref().ok_or("no Captain attrs")?.speciallw_ground_lag_mul;
                            let row = load_row(p, FTCA_SM_SPECIAL_LW_END)?;
                            change_motion_state_full(p, FTCA_MS_SPECIAL_LW_END, row.as_ref(), 0.0, lag, &mut self.rand)?;
                        } else {
                            melee_fighter::ftcommon2::ftcommon_8007d5d4(fp);
                            let row = load_row(p, FTCA_SM_SPECIAL_LW_END_AIR)?;
                            change_motion_state(p, FTCA_MS_SPECIAL_LW_END_AIR, row.as_ref(), &mut self.rand)?;
                        }
                    }
                }
                FTCA_MS_SPECIAL_LW_END | FTCA_MS_SPECIAL_LW_END_AIR => {
                    // ftCa_SpecialLwEnd_Anim -> ftCommon_8007D92C.
                    if !p.skel.frames_remaining() {
                        if p.fp.ground_or_air == melee_types::GA_Air {
                            fall_enter(p, &mut self.rand)?;
                        } else {
                            ft_8008a2bc(p, &mut self.rand)?;
                        }
                    }
                }
                m if is_guard_motion(m) => guard_anim(p, &mut self.rand)?,
                FTCO_MS_KNEE_BEND => kneebend_anim(p, &mut self.rand)?,
                FTCO_MS_OTTOTTO => {
                    // ftCo_Ottotto_Anim -> ftCo_8009A6B8: OttottoWait (the sounds are not modelled).
                    if !p.skel.frames_remaining() {
                        let row = load_row(p, FTCO_SM_OTTOTTO + 1)?;
                        change_motion_state(p, FTCO_MS_OTTOTTO_WAIT, row.as_ref(), &mut self.rand)?;
                    }
                }
                FTCO_MS_OTTOTTO_WAIT => {} // ftCo_OttottoWait_Anim is empty
                m if is_fx_kind(kind) && fox::is_fox_special_n(m) => fox::anim(p, &mut self.rand)?,
                m if kind == melee_types::Ft_Kind_Seak && sheik::is_sheik_special_n(m) => sheik::anim(p, &mut self.rand)?,
                m if is_jump_motion(m) => {
                    // ftCo_Jump_Anim.
                    if !p.skel.frames_remaining() {
                        fall_enter(p, &mut self.rand)?;
                    }
                }
                m if is_jump_aerial_motion(m) => {
                    // ftCo_JumpAerial_Anim.
                    if !p.skel.frames_remaining() {
                        fall_aerial_enter(p, &mut self.rand)?;
                    }
                }
                FTCO_MS_FALL_AERIAL => fall_anim(p)?,
                m if attackair::is_attack_air(m) => attackair::anim(p, &mut self.rand)?,
                dashrun::FTCO_MS_DASH => dashrun::dash_anim(p, &mut self.rand)?,
                dashrun::FTCO_MS_RUN => dashrun::run_anim(p),
                FTCO_MS_DEAD_DOWN => {
                    // ftCo_DeadDown_Anim: the countdown, then ftCo_800BFD9C (Sleep + the respawn).
                    p.dead_timer -= 1;
                    if p.dead_timer == 0 {
                        let mut ctx = RespawnCtx { ground: &mut self.ground, cg: &mut self.cg, rand: &mut self.rand, respawn_points: &self.respawn_points, cam_top: self.blast.cam_top, busy: &mut self.respawn_busy, start_shield: self.start_shield, per_slot_spawn: self.per_slot_spawn };
                        rebirth::dead_over(p, &mut ctx)?;
                    }
                }
                rebirth::FTCO_MS_REBIRTH | rebirth::FTCO_MS_REBIRTH_WAIT => {
                    let mut ctx = RespawnCtx { ground: &mut self.ground, cg: &mut self.cg, rand: &mut self.rand, respawn_points: &self.respawn_points, cam_top: self.blast.cam_top, busy: &mut self.respawn_busy, start_shield: self.start_shield, per_slot_spawn: self.per_slot_spawn };
                    rebirth::anim(p, &mut ctx)?;
                }
                rebirth::FTCO_MS_SLEEP => {} // ftCo_Sleep_Anim is empty
                m if samus::is_special_s(m) => samus::anim(p, &mut self.rand)?,
                m if is_damage_motion(m) => {
                    // ftCo_Damage*_Anim (damage.rs).
                    unsafe { (*sp).damage_proc(pi, damage::Proc::Anim)? };
                }
                m => return Err(format!("player {}: motion {m} not ported", p.slot + 1)),
            }
            }
            // The end of Fighter_8006A360 (fighter.c TARGET_PC): Mex_OnFrameDispatch(kind) runs an
            // m-ex fighter's onFrame override (slot 23) through the PPC interpreter.
            if let Some(m) = &mut p.mex {
                m.run(crate::mex::SLOT_ON_FRAME, p.slot, &mut p.fp);
            }
            // ftCommon_8007E0E4 (the end of Fighter_8006A360, outside hitlag): the player nudge.
            player_nudge(&mut self.players, pi, &self.ground);
        }
        // s_link 2, Fighter_8006ABA0: input_cb. ftCo_Fall_IASA's checks all read pad input, and
        // the C applies none before GO (self_vel.x stays 0 in the golden with the stick held):
        // the fighters' input is all zero here. TODO(group3): the IASA checks themselves.
        // s_link 2 (Fighter_8006ABA0): the CPU controller of every CPU fighter.
        {
            let grkind = self.grkind;
            let cpu: Vec<usize> = (0..self.players.len())
                .filter(|&i| !self.players[i].follower && stale::slot_type(self.players[i].slot) == melee_types::Gm_PKind_Cpu && self.players[i].fp.cpu.kind != 5 && self.players[i].fp.x221F_b3() == 0)
                .collect();
            for i in cpu {
                let me: *mut Fighter = &mut *self.players[i].fp;
                let fps: Vec<*mut Fighter> = self.players.iter_mut().map(|p| &mut *p.fp as *mut Fighter).collect();
                let mut items = self.items.clone();
                items.extend(fox::pending_lasers(&self.players));
                let mut cx = cpu::Cx { fps, ground: &mut self.ground, rand: &mut self.rand, items: &items, blast: self.blast, grkind, dist_thresh: &self.cpu_dist, h_deadzone: melee_fighter::hooks::common_data().horizontal_stick_deadzone, plco: &self.plco_bytes, tbl: self.cpu_tbl };
                cpu::ftco_800b3900(&mut cx, me)?;
            }
        }
        // s_link 3, Fighter_Spaghetti_8006AD10: the input step, then (outside hitlag) the
        // smash-charge state, the stick-activity stats, the press counters and input_cb.
        for i in 0..self.players.len() {
            if self.players[i].follower {
                continue;
            }
            let port = self.players[i].slot as usize;
            let pad = self.pads[port];
            let p = &mut self.players[i];
            let locked = p.input_locked;
            let is_cpu = stale::slot_type(p.slot) == melee_types::Gm_PKind_Cpu && p.fp.cpu.kind != 5;
            input::fighter_input(&mut p.fp, &pad, &mut p.x221d_b3, locked, is_cpu);
            if p.fp.x2219_b5() == 0 {
                if !p.fp.x1980.is_null() {
                    return Err("ftCommon_8007FFD8 (x1980 item) not ported".into());
                }
                // ftCo_800DF0D0: smash-charge state machine (state None here).
                if p.fp.smash_attrs.state != 0 {
                    return Err("ftCo_800DF0D0 smash charge state not wired".into());
                }
                ftcommon_8008031c(&mut p.fp);
                let jump = jump_get_input(&p.fp);
                input::increment_counters(&mut p.fp, jump);
                if is_damage_motion(p.fp.motion_id) {
                    self.damage_proc(i, damage::Proc::Iasa)?;
                } else {
                    input_cb(&mut self.players[i], &mut self.rand)?;
                }
            }
        }
        // s_link 4, the stage gobjs' procs (p_link 5, ahead of the fighters' p_link 8).
        self.greens_procs()?;
        self.story_procs()?;
        // s_link 4, Fighter_procUpdate: phys_cb, then the integration below.
        for pi in 0..self.players.len() {
            if self.players[pi].fp.x221F_b3() != 0 {
                continue;
            }
            if self.players[pi].fp.x2219_b5() == 0 && is_damage_motion(self.players[pi].fp.motion_id) {
                // ftCo_Damage*_Phys (damage.rs); the shared tail below is the same.
                self.damage_proc(pi, damage::Proc::Phys)?;
            }
            let p = &mut self.players[pi];
            if p.fp.x2219_b5() != 0 {
                // Hitlag: no phys_cb, no integration; the tail below runs.
            } else {
            // x2064_ledgeCooldown countdown (capture_timer is 0 here).
            if p.fp.x2064_ledgeCooldown != 0 {
                p.fp.x2064_ledgeCooldown -= 1;
            }
            let e = &mut p.entry;
            match p.fp.motion_id {
                FTCO_MS_ENTRY_START => {
                    // ftCo_EntryStart_Phys (not a sub-fighter). The JObj scale lines are not hashed.
                    let t = (c.x6bc - e.timer) as f32 / c.x6bc as f32;
                    // x14.y = t * (x8.y - x6C4) + x6C4 (one fused site); HSD_JObjSetScale(root).
                    let x6c4 = melee_fighter::hooks::common_data().x6C4;
                    e.x14.y = melee_core::fp::fmadds(t, e.x8.y - x6c4, x6c4);
                    let root = p.skel.root;
                    p.skel.set_scale(root, e.x14);
                    e.x28 = e.x20 * t;
                    p.fp.cur_pos.y = e.x4 + e.x28;
                }
                FTCO_MS_ENTRY_END => {
                    // ftCo_EntryEnd_Phys
                    let t = e.timer as f32 / c.x6bc as f32;
                    e.x28 = e.x20 * t;
                    p.fp.cur_pos.y = e.x4 + e.x28;
                }
                m if attackair::is_attack_air(m) => attackair::phys(p),
                dashrun::FTCO_MS_DASH => dashrun::dash_phys(p),
                dashrun::FTCO_MS_RUN => dashrun::run_phys(p),
                FTCO_MS_FALL | FTCO_MS_FALL_AERIAL | FTCO_MS_JUMP_AERIAL_F | FTCO_MS_JUMP_AERIAL_B => {
                    // ftCo_Fall_Phys / FallAerial_Phys / JumpAerial_Phys -> ft_80084DB0: fall-fast check, fall, air drift.
                    let f = &mut *p.fp;
                    fc::check_fall_fast(f);
                    if f.fall_fast() != 0 {
                        fc::fall_fast(f);
                    } else {
                        let (g, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
                        fc::fall(f, g, t);
                    }
                    fc::calc_self_accel_drift(f);
                }
                m if (FTCO_MS_WALK_SLOW..=FTCO_MS_WALK_FAST).contains(&m) => {
                    // ftCo_Walk_Phys -> ftWalkCommon_800E0060.
                    walk_phys(p);
                }
                FTCO_MS_CATCH => {
                    // ftCo_Catch_Phys.
                    let f = &mut *p.fp;
                    let fr = melee_fighter::hooks::common_data().x64 * f.co_attrs.ground_friction;
                    fc::calc_ground_accel_deaccel(f, fr);
                    fc::set_self_movement_from_grounded_movement(f);
                }
                m if is_tilt_motion(m) => {
                    // ftCo_AttackS3_Phys / ftCo_AttackHi3_Phys: ft_80084F3C.
                    let f = &mut *p.fp;
                    let mut friction = f.co_attrs.ground_friction;
                    if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
                        friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
                    }
                    fc::calc_ground_accel_deaccel(f, friction);
                    fc::set_self_movement_from_grounded_movement(f);
                }
                m if (FTCO_MS_ATTACK11..=FTCO_MS_ATTACK11 + 2).contains(&m) => {
                    // ftCo_Attack1x_Phys -> ft_80084FA8 -> ft_80085030(friction, facing_dir).
                    let f = &mut *p.fp;
                    let mut fr = f.co_attrs.ground_friction;
                    if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
                        fr *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
                    }
                    if x594_b0(f) {
                        f.xE4_ground_accel_1 = melee_core::fp::fmadds(f.x6A4_transNOffset.z, f.facing_dir, -f.gr_vel);
                    } else {
                        fc::calc_ground_accel_deaccel(f, fr);
                    }
                    fc::set_self_movement_from_grounded_movement(f);
                }
                FTMS_MS_SPECIAL_LW => {
                    // ftMs_SpecialLw_Phys: ft_80084F3C, ftColl_8007AEE0 (shield capsule updates).
                    let f = &mut *p.fp;
                    let mut friction = f.co_attrs.ground_friction;
                    if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
                        friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
                    }
                    fc::calc_ground_accel_deaccel(f, friction);
                    fc::set_self_movement_from_grounded_movement(f);
                }
                FTCA_MS_SPECIAL_LW | FTCA_MS_SPECIAL_LW_END | FTCA_MS_SPECIAL_LW_END_AIR if p.fp.ground_or_air != melee_types::GA_Ground => {
                    // The airborne halves: ftPartSetRotZ(fp, 0, 0), then SpecialLw: ft_80085134 +
                    // the friction + 800E3EAC; SpecialLwEnd: ft_80084EEC + the friction;
                    // SpecialLwEndAir: cmd_vars[0] ? ft_80084EEC : ft_80085134.
                    p.skel.set_part_rot(0, 2, 0.0);
                    let f = &mut *p.fp;
                    let m = f.motion_id;
                    let anim_vel = m == FTCA_MS_SPECIAL_LW || (m == FTCA_MS_SPECIAL_LW_END_AIR && f.cmd_vars[0] == 0);
                    if anim_vel {
                        // ft_80085134.
                        f.self_vel.x = f.x6A4_transNOffset.z * f.facing_dir;
                        f.self_vel.y = f.x6A4_transNOffset.y;
                    } else {
                        // ft_80084EEC.
                        let (gr, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
                        fc::fall(f, gr, t);
                        let fr = f.co_attrs.aerial_friction;
                        fc::calc_self_accel_deaccel(f, fr);
                    }
                    if m != FTCA_MS_SPECIAL_LW_END_AIR {
                        let fr = p.ca_lw_friction;
                        f.self_vel.x *= fr;
                        f.self_vel.y *= fr;
                    }
                    if m == FTCA_MS_SPECIAL_LW {
                        ftca_800e3eac(p)?;
                    }
                }
                FTCA_MS_SPECIAL_LW | FTCA_MS_SPECIAL_LW_END | FTCA_MS_SPECIAL_LW_END_AIR => {
                    // ftCa_SpecialLw_Phys / ftCa_SpecialLwEnd_Phys / EndAir_Phys (grounded halves).
                    ftcommon_8007e5ac(p);
                    let traction = p.ca_attrs.as_ref().ok_or("no Captain attrs")?.speciallw_ground_traction;
                    let f = &mut *p.fp;
                    if f.motion_id == FTCA_MS_SPECIAL_LW_END_AIR {
                        // ftCa_SpecialLwEndAir_Phys on the ground: ft_80085088 only (no friction).
                        ft_80085088(f);
                    } else if f.motion_id == FTCA_MS_SPECIAL_LW {
                        ft_80085088(f);
                    } else if f.cmd_vars[2] != 0 {
                        let fr = traction * f.co_attrs.ground_friction;
                        fc::calc_ground_accel_deaccel(f, fr);
                        fc::set_self_movement_from_grounded_movement(f);
                    } else {
                        // ft_80084F3C.
                        let mut friction = f.co_attrs.ground_friction;
                        if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
                            friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
                        }
                        fc::calc_ground_accel_deaccel(f, friction);
                        fc::set_self_movement_from_grounded_movement(f);
                    }
                    // ftCa_Special_Inline_Friction.
                    if f.motion_id != FTCA_MS_SPECIAL_LW_END_AIR {
                        let fr = p.ca_lw_friction;
                        f.self_vel.x *= fr;
                        f.self_vel.y *= fr;
                    }
                    if f.motion_id == FTCA_MS_SPECIAL_LW {
                        ftca_800e3eac(p)?;
                    }
                }
                m if samus::is_special_s(m) => samus::phys(p)?,
                m if is_jump_motion(m) => jump_phys(p),
                rebirth::FTCO_MS_REBIRTH | rebirth::FTCO_MS_REBIRTH_WAIT => {
                    let mut ctx = RespawnCtx { ground: &mut self.ground, cg: &mut self.cg, rand: &mut self.rand, respawn_points: &self.respawn_points, cam_top: self.blast.cam_top, busy: &mut self.respawn_busy, start_shield: self.start_shield, per_slot_spawn: self.per_slot_spawn };
                    rebirth::phys(p, &mut ctx)?;
                }
                rebirth::FTCO_MS_SLEEP => {}
                m if p.fp.kind == melee_types::Ft_Kind_Seak && sheik::is_sheik_special_n(m) => sheik::phys(p),
                FOX_AIR_N_START | FOX_AIR_N_LOOP | FOX_AIR_N_END if is_fx_kind(p.fp.kind) => {
                    // ftFx_SpecialAirN*_Phys -> ft_80084DB0.
                    let f = &mut *p.fp;
                    fc::check_fall_fast(f);
                    if f.fall_fast() != 0 {
                        fc::fall_fast(f);
                    } else {
                        let (g, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
                        fc::fall(f, g, t);
                    }
                    fc::calc_self_accel_drift(f);
                }
                m if m == FTCO_MS_LANDING || is_landing_air(m) || m == FTCO_MS_TURN || m == FTCO_MS_WAIT || is_guard_motion(m) || m == FTCO_MS_KNEE_BEND || (is_fx_kind(p.fp.kind) && (fox::FX_MS_SPECIAL_N_START..=fox::FX_MS_SPECIAL_N_END).contains(&m)) => {
                    // (Wait: then ftColl_8007AEE0, a hurtbox reset; not hashed.)
                    // ftCo_Landing_Phys / ftCo_Turn_Phys -> ft_80084F3C: ground friction, then the
                    // self velocity along the floor.
                    let f = &mut *p.fp;
                    let mut friction = f.co_attrs.ground_friction;
                    if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
                        friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
                    }
                    fc::calc_ground_accel_deaccel(f, friction);
                    fc::set_self_movement_from_grounded_movement(f);
                }
                _ => {}
            }
            proc_update_integrate(&mut p.fp)?;
            }
            // ftColl_80076528: the combo-push timer.
            if p.fp.x2092 != 0 {
                p.fp.x2092 -= 1;
                if p.fp.victim_gobj.is_null() && p.fp.ground_or_air == melee_types::GA_Ground {
                    return Err("ftColl_80076528: comboCount_Push is not ported".into());
                }
            }
            if let Some(cb) = p.fp.hitlag_cb {
                let g: *mut melee_types::HSD_GObj = &mut *p.gobj;
                unsafe { cb(g) };
            }
            if p.fp.ground_or_air == melee_types::GA_Ground {
                // mpGetSpeed(floor.index, &cur_pos, &d): the floor's own motion.
                let c = p.fp.cur_pos;
                if let Some(d) = self.ground.mp_get_speed(p.fp.coll_data.floor.index, melee_world::vec::Vec3 { x: c.x, y: c.y, z: c.z }) {
                    p.fp.cur_pos.x += d.x;
                    p.fp.cur_pos.y += d.y;
                    p.fp.cur_pos.z += d.z;
                }
            }
        }
        // The rest of Fighter_procUpdate: ftCo_800D3158, the blast zone check.
        for i in 0..self.players.len() {
            if self.players[i].follower {
                continue;
            }
            self.blast_check(i)?;
        }
        // The end of Fighter_procUpdate: ftColl_8007AF28 lets the next lbColl_800083C4 recompute the capsules.
        for p in &mut self.players {
            if p.fp.x221F_b3() != 0 {
                continue;
            }
            for k in 0..p.fp.hurt_capsules_len as usize {
                p.fp.hurt_capsules[k].capsule.set_skip_update_pos(0);
            }
        }
        // s_link 6, Fighter_procMap (x221F_b3 clear): the ECB lock countdown, the root JObj to
        // cur_pos, the state's coll_cb, the root JObj to cur_pos again.
        let c_common = self.common;
        for i in 0..self.players.len() {
            let p = &mut self.players[i];
            if p.follower {
                continue;
            }
            if p.fp.ecb_lock != 0 {
                p.fp.ecb_lock -= 1;
                if p.fp.ecb_lock == 0 {
                    // ftCommon_UnlockECB.
                    p.fp.coll_data.x130_flags &= !mc::COLLDATA_X130_LOCKED;
                }
            }
            let root = p.skel.root;
            let c = p.fp.cur_pos;
            p.skel.set_translate(root, Vec3 { x: c.x, y: c.y, z: c.z });
            match p.fp.motion_id {
                FTCO_MS_ENTRY => {} // ftCo_Entry_Coll is empty
                FTCO_MS_ENTRY_START | FTCO_MS_ENTRY_END => {
                    // ftCo_EntryStart_Coll / ftCo_EntryEnd_Coll (not a sub-fighter).
                    p.entry.x2c.bottom = -p.entry.x28;
                    if p.fp.ground_or_air == melee_types::GA_Air {
                        let b = p.entry.x2c;
                        if ft_80083e64(&mut self.ground, &mut self.cg, p, &b)? {
                            return Err("entry coll touched a floor: fn_800C63BC (ftCommon_8007D7FC) not wired".into());
                        }
                    }
                }
                m if dashrun::is_dash_or_run(m) => dashrun::coll(&mut self.ground, &mut self.cg, p, &mut self.rand)?,
                m if attackair::is_attack_air(m) => attackair::coll(&mut self.ground, &mut self.cg, p, c_common, &mut self.rand)?,
                FTCO_MS_FALL | FTCO_MS_FALL_AERIAL => {
                    // ftCo_Fall_Coll: ft_800831CC(gobj, ftCo_80096CC8, ft_80082B1C).
                    if ft_800831cc(&mut self.ground, &mut self.cg, p)? {
                        ft_80082b1c(&mut self.ground, p, c_common, &mut self.rand)?;
                    }
                }
                m if m == FTCO_MS_CATCH || is_tilt_motion(m) || (FTCO_MS_ATTACK11..=FTCO_MS_ATTACK11 + 2).contains(&m) => {
                    // ftCo_Attack1x_Coll -> ft_80084104: ft_800827A0 (mpColl_8004B2DC), else Fall;
                    // ftCo_Catch_Coll -> ft_800841B8(fn_800D8E30): the same, the grab cleanup first.
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    let r = mc::mp_coll_8004b2dc(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if !r {
                        if p.fp.motion_id == FTCO_MS_CATCH {
                            return Err("Catch off the floor: fn_800D949C / fn_800D9C64 not ported".into());
                        }
                        fall_enter(p, &mut self.rand)?;
                    }
                }
                FTMS_MS_SPECIAL_LW => {
                    // ftMs_SpecialLw_Coll: ft_800827A0 (mpColl_8004B2DC); off the floor the C goes
                    // to SpecialAirLw (ftCommon_GroundToAirStateChange), not ported.
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    let r = mc::mp_coll_8004b2dc(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if !r {
                        return Err("ftMs_SpecialLw_80138D38 (Counter off the floor -> SpecialAirLw) not ported".into());
                    }
                }
                FTCA_MS_SPECIAL_LW | FTCA_MS_SPECIAL_LW_END | FTCA_MS_SPECIAL_LW_END_AIR if p.fp.ground_or_air != melee_types::GA_Ground => {
                    // The airborne halves: ft_80081D0C (mpColl_800471F8), a floor -> ftCommon_8007D7FC.
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    let r = mc::mp_coll_800471f8(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if r {
                        ftcommon_8007d7fc(&mut self.ground, p, c_common)?;
                    }
                    if p.fp.motion_id == FTCA_MS_SPECIAL_LW && p.fp.cmd_vars[0] != 0 {
                        let env = p.fp.coll_data.env_flags;
                        let fd = p.fp.facing_dir;
                        if (fd == -1.0 && env & mc::COLLIDE_RIGHT_WALL_HUG != 0) || (fd == 1.0 && env & mc::COLLIDE_LEFT_WALL_HUG != 0) {
                            return Err("Falcon Kick into a wall (SpecialHiThrow1) not ported".into());
                        }
                    }
                }
                FTCA_MS_SPECIAL_LW | FTCA_MS_SPECIAL_LW_END | FTCA_MS_SPECIAL_LW_END_AIR => {
                    // ftCa_SpecialLw_Coll / ftCa_SpecialLwEnd_Coll (grounded): ft_80082708, no floor
                    // -> ftCommon_8007D5D4.
                    if p.fp.motion_id != FTCA_MS_SPECIAL_LW && p.fp.cmd_vars[1] != 0 {
                        return Err("ftCa_SpecialLwEnd_Coll: ft_800827A0 not ported".into());
                    }
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    let r = mc::mp_coll_8004b108(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if !r {
                        melee_fighter::ftcommon2::ftcommon_8007d5d4(fp);
                    }
                    // ftCa_SpecialLw_Coll's wall check (cmd_vars[0] and a wall hug on the facing side).
                    let env = fp.coll_data.env_flags;
                    if fp.motion_id == FTCA_MS_SPECIAL_LW && fp.cmd_vars[0] != 0
                        && ((fp.facing_dir == -1.0 && env & mc::COLLIDE_RIGHT_WALL_HUG != 0)
                            || (fp.facing_dir == 1.0 && env & mc::COLLIDE_LEFT_WALL_HUG != 0))
                    {
                        return Err("Falcon Kick into a wall (SpecialHiThrow1) not ported".into());
                    }
                }
                m if is_jump_motion(m) || is_jump_aerial_motion(m) => {
                    // ftCo_Jump_Coll: ft_800835B0(gobj, ftCo_80096CC8, ft_80082B1C): the Fall
                    // collision, then a ceiling hug (ftCo_8009EF68).
                    if ft_800831cc(&mut self.ground, &mut self.cg, p)? {
                        ft_80082b1c(&mut self.ground, p, c_common, &mut self.rand)?;
                    } else if p.fp.coll_data.env_flags & mc::COLLIDE_CEILING_HUG != 0 {
                        return Err("ftCo_8009EF68 (StopCeil) not ported".into());
                    }
                }
                m if p.fp.kind == melee_types::Ft_Kind_Seak && sheik::is_sheik_special_n(m) => {
                    // ftSk_Special*N*_Coll: ft_80082708 (ground, no floor -> GroundToAir) /
                    // ft_80081D0C (air, a floor -> AirToGround).
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    if m >= sheik::SK_MS_SPECIAL_AIR_N_START {
                        let r = mc::mp_coll_800471f8(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                            .map_err(|e| format!("{} not ported", e.0))?;
                        fp.cur_pos = fp.coll_data.cur_pos;
                        if r {
                            sheik::on_landed(&mut self.ground, p, c_common, &mut self.rand)?;
                        }
                    } else {
                        let r = mc::mp_coll_8004b108(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                            .map_err(|e| format!("{} not ported", e.0))?;
                        fp.cur_pos = fp.coll_data.cur_pos;
                        if !r {
                            sheik::on_walked_off(p, &mut self.rand)?;
                        }
                    }
                }
                FOX_AIR_N_START | FOX_AIR_N_LOOP | FOX_AIR_N_END if is_fx_kind(p.fp.kind) => {
                    // ftCo_AirCatchHit_Coll -> ft_80083090(gobj, ftCo_80096CC8, ftCo_80096D28).
                    if ft_800831cc(&mut self.ground, &mut self.cg, p)? {
                        return Err("ftCo_80096D28 (SpecialAirN landing) not ported".into());
                    }
                }
                m if m == FTCO_MS_TURN || m == FTCO_MS_KNEE_BEND || is_guard_motion(m) || (is_fx_kind(p.fp.kind) && (fox::FX_MS_SPECIAL_N_START..=fox::FX_MS_SPECIAL_N_END).contains(&m)) => {
                    // ftCo_Turn_Coll -> ft_80083F88 / the guard states' ft_800845B4: ft_80082708
                    // (mpColl_8004B108); no floor -> Fall (a ledge slip -> ftCo_8009F39C).
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    let r = mc::mp_coll_8004b108(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if !r {
                        if is_guard_motion(p.fp.motion_id) {
                            let (fd, env) = (p.fp.facing_dir, p.fp.coll_data.env_flags);
                            if (fd < 0.0 && env & mc::COLLIDE_RIGHT_LEDGE_SLIP != 0) || (fd > 0.0 && env & mc::COLLIDE_LEFT_LEDGE_SLIP != 0) {
                                return Err("ftCo_8009F39C (shield ledge slip) not ported".into());
                            }
                        }
                        fall_enter(p, &mut self.rand)?;
                    }
                }
                m if m == FTCO_MS_LANDING || is_landing_air(m) || m == FTCO_MS_WAIT || (FTCO_MS_WALK_SLOW..=FTCO_MS_WALK_FAST).contains(&m) => {
                    // ftCo_Landing_Coll / ftCo_Wait_Coll / ftCo_Walk_Coll -> ft_80084280.
                    if p.fp.xF8_playerNudgeVel.x != 0.0 && p.fp.xF8_playerNudgeVel.x * p.fp.facing_dir < 0.0 {
                        return Err("ft_800827A0 (pushed by another fighter) not ported".into());
                    }
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    fp.coll_data.lstick_x = fp.input.lstick[0].x;
                    // ftGetFacingDirInt.
                    mc::mp_coll_set_facing_dir(&mut fp.coll_data, if fp.facing_dir < 0.0 { -1 } else { 1 });
                    let r = mc::mp_coll_8004b4b0(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if !r {
                        if fp.coll_data.env_flags & mc::COLLIDE_EDGE != 0 {
                            // ftCo_8009A3C8 -> ftCo_8009A410.
                            return ottotto_enter(p, &mut self.rand);
                        }
                        fall_enter(p, &mut self.rand)?;
                    }
                }
                FTCO_MS_OTTOTTO | FTCO_MS_OTTOTTO_WAIT => {
                    // ftCo_Ottotto_Coll / ftCo_OttottoWait_Coll: ft_800827A0 (mpColl_8004B2DC, walk
                    // off) - no floor: Fall; else Wait once the fighter is far enough from the end.
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    let r = mc::mp_coll_8004b2dc(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if !r {
                        fall_enter(p, &mut self.rand)?;
                    } else {
                        let line = p.fp.coll_data.floor.index;
                        let pos = if p.fp.facing_dir > 0.0 { self.ground.mp_floor_get_right(line) } else { self.ground.mp_floor_get_left(line) };
                        let cd = melee_fighter::hooks::common_data();
                        if fc::abs_c(p.fp.cur_pos.x - pos.x) > cd.x478 + cd.x47C {
                            ft_8008a2bc(p, &mut self.rand)?;
                        }
                    }
                }
                FTCO_MS_DEAD_DOWN => {} // coll_cb NULL
                rebirth::FTCO_MS_REBIRTH => {
                    let mut ctx = RespawnCtx { ground: &mut self.ground, cg: &mut self.cg, rand: &mut self.rand, respawn_points: &self.respawn_points, cam_top: self.blast.cam_top, busy: &mut self.respawn_busy, start_shield: self.start_shield, per_slot_spawn: self.per_slot_spawn };
                    rebirth::coll(p, &mut ctx)?;
                }
                rebirth::FTCO_MS_REBIRTH_WAIT => {
                    // ftCo_RebirthWait_Coll: ft_80083844 (mpColl_80048654), a floor -> fn_800D5A30.
                    let joints = {
                        let d = p.ecb_desc;
                        ecb_joints(&mut p.skel, &d)
                    };
                    let fp = &mut *p.fp;
                    fp.coll_data.last_pos = fp.coll_data.cur_pos;
                    fp.coll_data.cur_pos = fp.cur_pos;
                    let r = mc::mp_coll_80048654(&mut self.ground, &mut self.cg, &mut fp.coll_data, &joints)
                        .map_err(|e| format!("{} not ported", e.0))?;
                    fp.cur_pos = fp.coll_data.cur_pos;
                    if !ft_80081a00(fp) && r {
                        // fn_800D5A30: ftColl_8007B7A4(gobj, x5D8) (the landing invincibility; the
                        // colour overlay is render only), then ft_8008A2BC from the air.
                        let x5d8 = melee_fighter::hooks::common_data().x5D8;
                        if x5d8 > fp.x1994 {
                            fp.x1994 = x5d8;
                        }
                        fp.x198C = if fp.x1990 != 0 { 2 } else { 1 };
                        if fp.ground_or_air == melee_types::GA_Air {
                            ftcommon_8007d7fc(&mut self.ground, p, c_common)?;
                        }
                        ft_8008a2bc(p, &mut self.rand)?;
                    }
                }
                m if samus::is_special_s(m) => samus::coll(&mut self.ground, &mut self.cg, p, &mut self.rand)?,
                m if is_damage_motion(m) => {
                    unsafe { (*sp).damage_proc(i, damage::Proc::Coll)? };
                }
                m => return Err(format!("player {}: coll_cb of motion {m} not ported", p.slot + 1)),
            }
            let c = p.fp.cur_pos;
            p.skel.set_translate(root, Vec3 { x: c.x, y: c.y, z: c.z });
        }
        // s_link 9, Fighter_8006C80C: efAsync_QueueFlush(&fp->x60C).
        for i in 0..self.players.len() {
            for op in std::mem::take(&mut self.players[i].ef_ops) {
                match op {
                    EfOp::Spawn(e) => self.ef_dispatch(i, e)?,
                    EfOp::DestroyAll => self.ef_destroy_all(i),
                }
            }
            let q = std::mem::take(&mut self.players[i].ef_queue);
            for e in q.into_iter().rev() {
                self.ef_dispatch(i, e)?;
            }
            // Fighter_8006C80C: accessory4_cb (ftFx_SpecialN_CreateBlasterShot).
            if self.players[i].fp.x221F_b3() == 0 && self.players[i].fp.x2219_b5() == 0 {
                fox::accessory4(&mut self.players[i])?;
                self.samus_accessory4(i)?;
            }
            self.seak_accessory(i)?;
        }
        // s_link 13, Fighter_8006CB94: the lasers' item procs of the frame, then the items' hits on fighters.
        self.lasers_step()?;
        self.item_hits()?;
        // s_link 14, Fighter_ProcessHit_8006D1EC: the shield's regeneration / upkeep, and the damage of a
        // hit without knockback (a hit with knockback is not ported: it returns Err).
        for p in &mut self.players {
            if p.fp.x221F_b3() != 0 {
                continue;
            }
            let cd = melee_fighter::hooks::common_data();
            let fp = &mut *p.fp;
            let start = cd.x260_startShieldHealth;
            if fp.x221A_b7() == 0 {
                if fp.shield_health < start {
                    fp.shield_health += cd.x27C;
                    if fp.shield_health > start {
                        fp.shield_health = start;
                    }
                }
            } else {
                let lsa = fp.lightshield_amount;
                let scale = 1.0 - melee_core::fp::fmadds(lsa, cd.x2E0 - cd.x2DC, cd.x2DC);
                fp.shield_health -= melee_core::fp::fmadds(cd.x284, fp.x19A0_shieldDamageTaken as f32 * scale, cd.x288);
                if fp.shield_health < 0.0 {
                    return Err(format!("player {}: shield break (pl_8003E058) not ported", p.slot + 1));
                }
            }
        }
        for pi in 0..self.players.len() {
            if self.players[pi].fp.x221F_b3() != 0 {
                continue;
            }
            self.process_hit_damage(pi)?;
        }
        // The items' s_link 14: a shot that dealt damage is destroyed.
        self.lasers_destroy_dealt();
        // The end of Fighter_ProcessHit_8006D1EC: ftCo_800A0DA4, the hurtbox extents the CPUs read
        // (the capsules' bone positions are lbColl_800083C4's, from the skeleton as it stands).
        for p in &mut self.players {
            if p.fp.x221F_b3() != 0 || p.follower {
                continue;
            }
            hurt_extents(p);
        }
        // s_link 0x10, Fighter_8006D9AC -> ftCo_8009E0A8: the dynamic bones.
        for p in &mut self.players {
            if p.fp.x221F_b3() != 0 || p.fp.x2219_b5() != 0 {
                continue;
            }
            let Some(dy) = &mut p.dynamics else { continue };
            if std::env::var_os("MELEE_NO_DYN").is_some() { continue; }
            if self.grkind == melee_types::Gr_Kind_Flatzone as i32 {
                return Err("ftCo_8009DD94 on Flatzone not ported".into());
            }
            let fi = crate::dynamics::FrameIn {
                kind: p.fp.kind as i32,
                motion_id: p.fp.motion_id as i32,
                grounded: p.fp.ground_or_air as i32 == melee_types::GA_Ground as i32,
                cur_pos_y: p.fp.cur_pos.y,
                scale_y: p.fp.x34_scale.y,
                x2228_b1: p.fp.x2228_b1() != 0,
                is_peach: p.fp.kind as i32 == melee_types::Ft_Kind_Peach as i32,
                is_mewtwo: p.fp.kind as i32 == melee_types::Ft_Kind_Mewtwo as i32,
                is_koopa: p.fp.kind as i32 == melee_types::Ft_Kind_Koopa as i32 || p.fp.kind as i32 == melee_types::Ft_Kind_GKoops as i32,
                mewtwo_motion_range: (0xDF..=0xE8).contains(&(p.fp.motion_id as i32)),
            };
            let ground = &mut self.ground;
            dy.step(&mut p.skel, &fi, &mut |ax, ay, bx, by, off| ground.mp_check_floor(ax, ay, bx, by, off, -1, -1, -1, None).map(|h| h.pos));
            if std::env::var_os("MELEE_GOLDEN_DYN").is_some() && p.slot == 0 {
                static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                for i in 18..=20usize {
                    let r = p.skel.tree.get(p.skel.part(i)).rotate;
                    eprintln!("DYN n{} p{} r {:08X} {:08X} {:08X}", n, i, r.x.to_bits(), r.y.to_bits(), r.z.to_bits());
                }
            }
        }
        // The item procs.
        self.items_update()?;
        Ok(())
    }

    /// Hit detection (ftColl / the hit procs) is not ported. A live hitbox only matters when it
    /// can reach the other fighter's hurtboxes, so the run stops when one comes within
    /// `HIT_GUARD_MARGIN` (plus its own size and offset) of the other fighter's position.
    fn hit_guard(&mut self) -> Result<(), String> {
        for a in 0..self.players.len() {
            for k in 0..4 {
                let Some(h) = self.players[a].hitboxes[k] else { continue };
                let pa = &mut self.players[a];
                let j = pa.skel.part(h.part);
                let w = pa.skel.world_pos(j);
                let reach = h.scale + (h.offset.x * h.offset.x + h.offset.y * h.offset.y + h.offset.z * h.offset.z).sqrt() + HIT_GUARD_MARGIN;
                for b in 0..self.players.len() {
                    // The hidden partner fighter (Zelda, Sleep) has no hurtboxes.
                    if b == a || self.players[b].follower {
                        continue;
                    }
                    let c = self.players[b].fp.cur_pos;
                    let (dx, dy) = (w.x - c.x, w.y - (c.y + 10.0));
                    if dx * dx + dy * dy < reach * reach {
                        return Err(format!("player {}'s hitbox {k} reaches player {}: hit detection not ported", a + 1, b + 1));
                    }
                }
            }
        }
        Ok(())
    }

    /// efAsync_QueueProcessDeferred -> efSync_Spawn -> efAsync_Dispatch, for player `i`.
    fn ef_dispatch(&mut self, i: usize, q: QueuedEffect) -> Result<(), String> {
        match (q.kind, q.gfx) {
            (EF_SPAWN_POS_OFFSET_PARAM, g @ (0x3FE | 0x400 | 0x401)) => {
                // lb_8000B1CC(jobj, &params, &pos), then efAsync_Dispatch:
                // efLib_CreateGenerator_Translate_FacingDir(0x5A, &pos, 0x401 ? dir : -dir):
                // hsd_8039F05C(0, 0, 0x5A) + an AppSRT (status 1) at pos, rot.y = +-pi/2 and
                // kind |= 0x40000 for a negative direction.
                let p = &mut self.players[i];
                let j = p.skel.part(q.part);
                let pos = p.skel.world_point(j, q.params);
                // 0x3FE: the same with id 0x107 and the direction as is.
                let dir = if g == 0x400 { -q.extra1 } else { q.extra1 };
                let gen_id = if g == 0x3FE { 0x107 } else { 0x5A };
                let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
                if let Some(gen) = self.ps.gen_create(&mut host, 0, 0, gen_id) {
                    let srt = match self.ps.gen(gen).appsrt {
                        Some(a) => Some(a),
                        None => Some(self.ps.add_generator_appsrt_begin(gen, 1)),
                    };
                    let Some(srt) = srt else { return Ok(()) };
                    let a = self.ps.srt_mut(srt);
                    a.translate = pos;
                    let half_pi = core::f32::consts::FRAC_PI_2;
                    a.rot.y = if dir < 0.0 { -half_pi } else { half_pi };
                    if dir < 0.0 {
                        self.ps.gen_mut(gen).kind |= 0x40000;
                    }
                }
                Ok(())
            }
            (EF_SPAWN_POS_OFFSET, 0x407) => {
                // lb_8000B1CC(jobj, &params, &pos), then efAsync_Dispatch 0x407:
                // efLib_CreateGenerator(0x3C, &pos).
                let p = &mut self.players[i];
                let j = p.skel.part(q.part);
                let pos = p.skel.world_point(j, q.params);
                let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
                if let Some(gen) = self.ps.gen_create(&mut host, 0, 0, 0x3C) {
                    self.ps.gen_mut(gen).pos = pos;
                }
                Ok(())
            }
            (EF_SPAWN_POS_OFFSET_PARAM, 0x404) => {
                // efAsync_QueueProcessDeferred: lb_8000B1CC(jobj, &params, &pos) (a zero offset
                // is the joint's world translation), then efSync_Spawn -> efAsync_Dispatch:
                // efLib_Create_Attach_Pos(0x18, gobj, &pos) + EF_STATE_ASYNC +
                // efAsync_SetEffectRotationZFromPtr(extra1) (the rotation is render-only here).
                let p = &mut self.players[i];
                if q.params.x != 0.0 || q.params.y != 0.0 || q.params.z != 0.0 {
                    return Err("lb_8000B1CC with a nonzero offset (MTXMultVec) not wired".into());
                }
                let j = p.skel.part(q.part);
                let pos = p.skel.world_pos(j);
                self.ef_create(0x18, i, 1.0, Some([pos.x, pos.y, pos.z]), None)
            }
            (EF_SPAWN_POS_OFFSET_PARAM2, g @ (0x3F7 | 0x3F8 | 0x3F9 | 0x3FD | 0x3FF)) => {
                // lb_8000B1CC(jobj, &offset, &pos), then efLib_Create_Attach_Pos(id, gobj, &pos) with
                // the facing (rot Y) and the floor angle (rot Z).
                let id = match g {
                    0x3F7 => 0x12,
                    0x3F8 => 0x13,
                    0x3F9 => 0x14,
                    0x3FD => 3,
                    _ => 5,
                };
                let p = &mut self.players[i];
                let j = p.skel.part(q.part);
                let pos = p.skel.world_point(j, q.params);
                let ry = if q.extra1 < 0.0 { -core::f32::consts::FRAC_PI_2 } else { core::f32::consts::FRAC_PI_2 };
                self.ef_create_rot(id, i, 1.0, Some([pos.x, pos.y, pos.z]), Some(ry), Some(q.extra2))
            }
            (EF_SPAWN_ATTACH_PARAM, 0x43E) => {
                // EF_SPAWN_ATTACH_PARAM: efLib_Create_Attach(0x24, gobj, fighter jobj), then
                // efAsync_SetEffectScaleXYZ(effect, entry.x8.x), the fighter JObj's scale x
                // (taken as co_attrs.model_scaling: positions only).
                let scale = self.players[i].model_scaling;
                self.ef_create(EF_ENTRY_BEAM, i, scale, None, None)
            }
            (EF_SPAWN_ATTACH_PARAM, 0x490) => {
                // efAlt_Spawn: efLib_Create_Attach(0xFA2, gobj, foot joint), params.z = the
                // angle, update = efLib_Cb_SetRotYZ_FromParamZ_FighterDir (rotation: render only).
                self.ef_create(0xFA2, i, 1.0, None, None)
            }
            (EF_SPAWN_ATTACH, g @ (0x402 | 0x403)) => {
                // efAsync_Dispatch: hsd_8039EFAC(0, 0, 0x59 / 0x5E, joint): a generator following
                // the fighter's part (its matrix is refreshed from the skeleton every frame).
                let r: JObjRef = ((0x80 + i as u32) << 24) | q.part as u32;
                if !self.fighter_jobjs.iter().any(|e| e.0 == r) {
                    self.fighter_jobjs.push((r, i, q.part));
                }
                self.refresh_fighter_jobj(r, i, q.part);
                let id = if g == 0x402 { 0x59 } else { 0x5E };
                let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
                self.ps.gen_create_on_jobj(&mut host, 0, 0, id, Some(r));
                Ok(())
            }
            (EF_SPAWN_SYNC, 1047 | 1048) => {
                // efSync_Spawn(0x417 / 0x418, gobj, shield joint, colour) -> efLib_Create_Attach(0xB / 0xC).
                let id = if q.gfx == 1047 { 0xB } else { 0xC };
                let scale = self.players[i].model_scaling;
                self.ef_create(id, i, scale, None, None)
            }
            (k, g) => Err(format!("efAsync_Dispatch kind {k} gfx {g:#x} is not ported")),
        }
    }

    /// `ftCo_800D3158(gobj)`: the blast zone check at the end of `Fighter_procUpdate`.
    fn blast_check(&mut self, i: usize) -> Result<(), String> {
        let b = self.blast;
        let p = &mut self.players[i];
        let fp = &*p.fp;
        if fp.x222A_b1() != 0 || fp.x2228_b5() != 0 || fp.is_sandbag() != 0 || fp.x2219_b2() != 0 || fp.x2219_b1() != 0 {
            return Ok(());
        }
        if fp.cur_pos.x > b.right {
            return Err(format!("player {}: right blast zone (ftCo_800D3950) not ported", p.slot + 1));
        }
        if fp.cur_pos.x < b.left {
            return Err(format!("player {}: left blast zone (ftCo_800D3680) not ported", p.slot + 1));
        }
        if fp.cur_pos.y > b.top && (fp.ground_or_air == melee_types::GA_Ground || fp.x2222_b3() != 0 || fp.x8c_kb_vel.y > melee_fighter::hooks::common_data().x4F0) {
            return Err(format!("player {}: top blast zone (star KO) not ported", p.slot + 1));
        }
        if fp.cur_pos.y < b.bottom {
            return self.dead_down(i);
        }
        Ok(())
    }

    /// `ftCo_800D331C`: what every death does first (the grab / item / metal / sfx cleanups the
    /// match has not reached are refused).
    fn dead_common(&mut self, i: usize) -> Result<(), String> {
        let p = &mut self.players[i];
        let fp = &mut *p.fp;
        if !fp.victim_gobj.is_null() {
            return Err("ftCo_800D331C: a fighter in a grab (ftCo_800DC920) not ported".into());
        }
        if fp.death1_cb.is_some() || fp.death2_cb.is_some() || fp.death3_cb.is_some() {
            return Err("ftCo_800D331C: death callbacks not ported".into());
        }
        if !fp.item_gobj.is_null() || !fp.x197C.is_null() || !fp.x1980.is_null() || fp.is_metal() != 0 || fp.x2226_b4() != 0 {
            return Err("ftCo_800D331C: item / metal / x2226_b4 cleanup not ported".into());
        }
        // ftCommon_8007E2FC.
        fp.xE4_ground_accel_1 = 0.0;
        fp.xE8_ground_accel_2 = 0.0;
        fp.x74_self_accel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.gr_vel = 0.0;
        fp.self_vel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.xF0_ground_kb_vel = 0.0;
        fp.x8c_kb_vel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.xF4_ground_attacker_shield_kb_vel = 0.0;
        fp.x98_atk_shield_kb = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.xD4_unk_vel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        // ftCommon_8007DB24.
        ftcommon_8007db24(p);
        Ok(())
    }

    /// `ftCo_800D34E0`: the death's bookkeeping. Only what feeds the hash is kept: the stock
    /// (`Player_LoseStock` when the rules count stocks), the rest is statistics
    /// (falls, KOs, the stale-move table) and the HUD.
    fn dead_bookkeeping(&mut self, i: usize) -> Result<(), String> {
        let p = &mut self.players[i];
        // gm_8016B094() / gm_8016B0E8(): this match is a time match (match_kind 0, x4_2 clear).
        stale::reset_stale_move_table(p.slot);
        // x18c4_source_ply == 6 (no last attacker): Player_IncSelfDestructs -> ifStatus_802F6C04
        // -> the HUD's explode animation + randomize_velocity.
        if p.fp.dmg.x18c4_source_ply != 6 {
            return Err("ftCo_800D34E0: a KO credited to another player (Player_UpdateKOsBySlot) not ported".into());
        }
        p.hud_randomize = true;
        Ok(())
    }

    /// `ftCo_800D3BC8`: the bottom blast zone KO (DeadDown).
    fn dead_down(&mut self, i: usize) -> Result<(), String> {
        let (cd_x500, cd_x4f4) = {
            let cd = melee_fighter::hooks::common_data();
            (cd.x500, cd.x4F4)
        };
        self.dead_common(i)?;
        let b = self.blast;
        let p = &mut self.players[i];
        p.dead_timer = cd_x500;
        change_motion_state(p, FTCO_MS_DEAD_DOWN, None, &mut self.rand)?;
        {
            let fp = &mut *p.fp;
            if fp.x221D_b6() != 0 {
                return Err("ftCo_800D3BC8: ft_800880D8 (looping sfx) not ported".into());
            }
            fp.set_x2219_b1(1);
            fp.set_x221E_b1(1);
            fp.set_x221E_b2(1);
            // pl_8003DF44 (the stale-move table's last-KO stamp), ft_PlaySFX and the quake are not hashed.
            fp.set_invisible(1);
            fp.set_x221F_b1(1);
        }
        self.dead_bookkeeping(i)?;
        // efSync_Spawn(0x42B or 0x42C, gobj, &pos, &rot_z, &p_ftCommonData->x4F4, color1, color2):
        // efLib_Create_Attach_Pos(0x19) (0x1A in a stamina match) at the clamped position, root
        // rot Z = 0, scale x4F4 (the tev colours are render only).
        let c = self.players[i].fp.cur_pos;
        let mut x = c.x;
        if x > b.right {
            x = b.right;
        }
        if x < b.left {
            x = b.left;
        }
        let pos = [x, b.bottom, c.z];
        self.ef_create(0x19, i, cd_x4f4, Some(pos), Some(0.0))?;
        // ftCo_800D4E50: coins (not a coin match).
        Ok(())
    }

    /// Refresh the matrix of a followed fighter part.
    fn refresh_fighter_jobj(&mut self, r: JObjRef, i: usize, part: usize) {
        let p = &mut self.players[i];
        let j = p.skel.part(part);
        let w = p.skel.world_pos(j);
        let mut m = melee_runtime::mtx::identity();
        m[0][3] = w.x;
        m[1][3] = w.y;
        m[2][3] = w.z;
        self.joint_mtx.insert(r, m);
    }

    /// `efLib_DestroyAll(gobj)`: every effect of player `i` freed after
    /// `HSD_JObjWalkTree(jobj, hsd_8039D688)` released the generators on its joints.
    fn ef_destroy_all(&mut self, i: usize) {
        let mut k = 0;
        while k < self.effects.len() {
            if self.effects[k].owner != i {
                k += 1;
                continue;
            }
            let e = self.effects.remove(k);
            let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
            for t in &e.tracks {
                self.ps.gen_release_jobj(&mut host, Some(t.joint));
            }
        }
    }

    /// `efLib_Create(gfx_id)` + `efLib_Create_Attach`: the effect's tracks and lifetime, then
    /// efAsync_Dispatch's anim queue (`HSD_JObjAnimAll` right away).
    fn ef_create(&mut self, gfx_id: u32, owner: usize, scale: f32, fixed_pos: Option<[f32; 3]>, rot_z: Option<f32>) -> Result<(), String> {
        self.ef_create_rot(gfx_id, owner, scale, fixed_pos, None, rot_z)
    }

    /// As [`Self::ef_create`], with the root's Y rotation too (`efAsync_SetEffectFacingDir`).
    fn ef_create_rot(&mut self, gfx_id: u32, owner: usize, scale: f32, fixed_pos: Option<[f32; 3]>, rot_y: Option<f32>, rot_z: Option<f32>) -> Result<(), String> {
        let bank = (gfx_id / 1000) as usize;
        let b = self.effect_banks.get(bank).and_then(|b| b.as_ref()).ok_or(format!("effect bank {bank} not loaded"))?;
        let desc = b.desc(gfx_id % 1000);
        let inst = self.next_effect_instance;
        self.next_effect_instance += 1;
        let tracks = crate::stage::dptcl_tracks_of(&b.archive, desc.joint, desc.animjoint)
            .into_iter()
            .map(|t| {
                let mut a: HsdAObj<()> = HsdAObj::load_desc(&t.aobj, |_| None);
                a.req_anim(0.0);
                a.set_rate(1.0);
                EffectTrack { joint: (inst << 24) | t.joint, chain: t.chain, aobj: a }
            })
            .collect();
        // effect->lifetime = (u32) desc->lifetime, +1 when nonzero (the 0x08000000 JObj flag for a
        // fractional lifetime is render-only).
        let mut lifetime = desc.lifetime as u32 as u16;
        if lifetime != 0 {
            lifetime += 1;
        }
        self.effects.push(Effect { lifetime, owner, scale, bank, tracks, fixed_pos, rot_y, rot_z });
        self.effect_anim(self.effects.len() - 1)
    }

    /// Bind-pose world matrices of effect `k`'s joints, rooted at its owner's position.
    fn effect_matrices(&mut self, k: usize) {
        let e = &self.effects[k];
        let fp = &self.players[e.owner].fp;
        let d = &self.effect_banks[e.bank].as_ref().unwrap().archive.data;
        let pos = e.fixed_pos.unwrap_or([fp.cur_pos.x, fp.cur_pos.y, fp.cur_pos.z]);
        for t in &e.tracks {
            let m = crate::stage::bind_pose_matrix_rooted(d, &t.chain, pos, [e.scale; 3], e.rot_y, e.rot_z);
            self.joint_mtx.insert(t.joint, m);
        }
    }

    /// `HSD_JObjAnimAll` on effect `k`: its particle keys -> `efLib_Cb_DPtcl`.
    fn effect_anim(&mut self, k: usize) -> Result<(), String> {
        self.effect_matrices(k);
        let mut spawns = Vec::new();
        let e = &self.effects[k];
        let root = e.fixed_pos.map(|p| EffectRoot { pos: p, rot_z: e.rot_z, scale: e.scale });
        for t in &mut self.effects[k].tracks {
            let joint = t.joint;
            let mut cb = |ty: u8, fv: f32| {
                if ty == 0x28 {
                    let iv = fv.to_bits() as i32;
                    spawns.push((iv & 0x3F, (iv >> 6) & 0xFF_FFFF, joint));
                }
            };
            let mut upd: melee_runtime::fobj::ObjUpdate<'_> = Some(&mut cb);
            t.aobj.interpret_anim(&mut self.aobj_globals, &mut upd);
        }
        self.dptcl_spawns(spawns, root)
    }

    /// `efLib_Cb_DPtcl(0, bank, id, jobj)` for each key. `root` is the root JObj of the effect
    /// the track belongs to (None for a stage track).
    fn dptcl_spawns(&mut self, spawns: Vec<(i32, i32, JObjRef)>, root: Option<EffectRoot>) -> Result<(), String> {
        for (bank, id, joint) in spawns {
            if std::env::var_os("MELEE_GOLDEN_RAND").is_some() {
                eprintln!("DPTCL frame {} bank {bank} id {id:#x} joint {joint:#x}", unsafe { TRACE_FRAME });
            }
            let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
            if bank == 0x1E {
                // grLib_801C99C0(bank, id, jobj, 1) -> hsd_8039EFAC(0, bank, id, jobj).
                self.ps.gen_create_on_jobj(&mut host, 0, bank, id, Some(joint));
                continue;
            }
            // efLib_SpawnParticleEffect(bank, id, jobj, 1): the special-cased ids are not ported;
            // the rest take the default hsd_8039EFAC(chk, bank, id, jobj).
            match id {
                0xD4 | 0x243 => {
                    // standalone generator on link 2: the effect root's rot.z + translation
                    // (+ scale for 0xD4).
                    let r = root.ok_or("efLib_SpawnParticleEffect 0xD4/0x243 outside a fixed-position effect")?;
                    let rot_z = r.rot_z.ok_or("efLib_SpawnParticleEffect: the root's own rotation is needed")?;
                    if let Some(gen) = self.ps.gen_create(&mut host, 2, bank, id) {
                        let srt = match self.ps.gen(gen).appsrt {
                            Some(a) => a,
                            None => self.ps.add_generator_appsrt_begin(gen, 1),
                        };
                        let a = self.ps.srt_mut(srt);
                        a.rot.z = rot_z;
                        a.translate = Vec3 { x: r.pos[0], y: r.pos[1], z: r.pos[2] };
                        if id == 0xD4 {
                            a.scale = Vec3 { x: r.scale, y: r.scale, z: r.scale };
                        }
                    }
                    continue;
                }
                0x2D | 0x2E | 0x31 => {
                    // eflib_create_generator_add_appsrt (hsd_8039EFAC(0, id / 1000, id, jobj), an
                    // AppSRT of status 0 with gp = the generator, type = (type & ~0x600) | 0x800),
                    // then type |= 0x1000.
                    if let Some(gen) = self.ps.gen_create_on_jobj(&mut host, 0, id / 1000, id, Some(joint)) {
                        let srt = match self.ps.gen(gen).appsrt {
                            Some(a) => a,
                            None => self.ps.add_generator_appsrt_begin(gen, 0),
                        };
                        self.ps.srt_mut(srt).gp = Some(gen);
                        let g = self.ps.gen_mut(gen);
                        g.type_ = (g.type_ & 0xF9FF) | 0x800 | 0x1000;
                    }
                    continue;
                }
                0x2 | 0x6 | 0xA3 | 0xA7 | 0xAA | 0xF2 | 0x12A | 0x132 | 0x133 => {
                    // eflib_create_generator_add_appsrt (as 0x2D below), then the generator's
                    // scale is the effect root's.
                    let r = root.ok_or("efLib_SpawnParticleEffect: the root's scale is needed outside an effect")?;
                    if let Some(gen) = self.ps.gen_create_on_jobj(&mut host, 0, id / 1000, id, Some(joint)) {
                        let srt = match self.ps.gen(gen).appsrt {
                            Some(a) => a,
                            None => self.ps.add_generator_appsrt_begin(gen, 0),
                        };
                        self.ps.srt_mut(srt).gp = Some(gen);
                        self.ps.srt_mut(srt).scale = Vec3 { x: r.scale, y: r.scale, z: r.scale };
                        let g = self.ps.gen_mut(gen);
                        g.type_ = (g.type_ & 0xF9FF) | 0x800;
                    }
                    continue;
                }
                0x4A38..=0x4A3A | 0xB798..=0xB79A | 0x127 | 0x16D..=0x170 | 0x7E2 | 0xE3 => {
                    return Err(format!("efLib_SpawnParticleEffect bank {bank} id {id:#x}: special case not ported"));
                }
                _ => {}
            }
            let chk = matches!(id, 0xFC | 0xFF | 0xF7 | 0x7918) as i32;
            self.ps.gen_create_on_jobj(&mut host, chk, bank, id, Some(joint));
        }
        Ok(())
    }

    /// The effects' s_link 15 proc, `efLib_Update`, ahead of efLib's particle procs (the effect
    /// GObjs sort before the particle GObj in p_link 11: GObj_Create priority 0 vs 1).
    fn effects_update(&mut self) -> Result<(), String> {
        let mut k = 0;
        while k < self.effects.len() {
            let e = &mut self.effects[k];
            if e.lifetime != 0 {
                e.lifetime -= 1;
                if e.lifetime == 0 {
                    // HSD_GObjFree: the JObjs stay referenced by their generators (ref_INC in
                    // hsd_8039EFAC), which keep the last matrix.
                    self.effects.remove(k);
                    continue;
                }
            }
            // scale_flags is EF_SCALE_NO_INHERIT after efLib_Create_Attach.
            self.effect_anim(k)?;
            k += 1;
        }
        Ok(())
    }
}

/// `ft_80083E64(gobj, box, cb)` minus the callback: true when it should run (a floor touched).
fn ft_80083e64(g: &mut GroundColl, cg: &mut CollGlobals, p: &mut Player, ecb: &ftCollisionBox) -> Result<bool, String> {
    let fp = &mut *p.fp;
    fp.coll_data.last_pos = fp.coll_data.cur_pos;
    fp.coll_data.cur_pos = fp.cur_pos;
    // ft_80082838: mirrored for facing < 0.
    let box_ = if fp.facing_dir < 0.0 {
        ftCollisionBox {
            top: ecb.top,
            bottom: ecb.bottom,
            left: melee_types::Vec2 { x: -ecb.right.x, y: ecb.left.y },
            right: melee_types::Vec2 { x: -ecb.left.x, y: ecb.right.y },
        }
    } else {
        *ecb
    };
    let r = mc::mp_coll_8004730c(g, cg, &mut fp.coll_data, &box_).map_err(|e| format!("{} not ported", e.0))?;
    fp.cur_pos = fp.coll_data.cur_pos;
    Ok(!ft_80081a00(fp) && r)
}

/// `ft_80081A00`: the item-pickup box (x1930) and its item scan. No items exist in the wired
/// scenarios, so it never takes the `ftCo_800C703C` branch; its x1930 / x2223_b5 writes are not
/// hashed. TODO(group3 ft_0819 + group5 items).
fn ft_80081a00(_fp: &mut Fighter) -> bool {
    false
}

/// `ft_800831CC(gobj, ftCo_80096CC8, cb)` up to the callback: true when `cb` (the landing) runs.
/// `ftWallJump_8008169C` / `ftCliffCommon_80081298` stop the run when their trigger flags are set.
fn ft_800831cc(g: &mut GroundColl, cg: &mut CollGlobals, p: &mut Player) -> Result<bool, String> {
    let joints = {
        let d = p.ecb_desc;
        ecb_joints(&mut p.skel, &d)
    };
    if std::env::var_os("MELEE_GOLDEN_ECB").is_some() {
        let c = p.fp.cur_pos;
        for (q, w) in joints.iter().enumerate() {
            eprintln!("J{} {:08X} {:08X} {:08X} pos {:08X} {:08X}", q, w.x.to_bits(), w.y.to_bits(), w.z.to_bits(), c.x.to_bits(), c.y.to_bits());
        }
    }
    let fp = &mut *p.fp;
    let dir = if fp.facing_dir < 0.0 { -1 } else { 1 };
    fp.coll_data.last_pos = fp.coll_data.cur_pos;
    fp.coll_data.cur_pos = fp.cur_pos;
    if fp.x2064_ledgeCooldown != 0 || fp.x2224_b2() != 0 {
        return Err("mpColl_80047AC8 (ledge cooldown) not ported".into());
    }
    mc::mp_coll_set_facing_dir(&mut fp.coll_data, dir);
    // ftCo_80096CC8: a platform passes only with the stick above p_ftCommonData->x25C.
    let x25c = melee_fighter::hooks::common_data().x25C;
    let lstick_y = fp.input.lstick[0].y;
    let gp: *const GroundColl = g;
    let mut cb = |line_id: i32| -> bool {
        // SAFETY: mpLineGetFlags only reads the line table, which the query does not change.
        line_id != -1 && (unsafe { (*gp).mp_line_get_flags(line_id) } & melee_world::mp::types::LINE_FLAG_PLATFORM == 0 || lstick_y > x25c)
    };
    let mut filt: melee_world::mp::check::LineFilter = Some(&mut cb);
    let r = mc::mp_coll_80047e14(g, cg, &mut fp.coll_data, &joints, &mut filt).map_err(|e| format!("{} not ported", e.0))?;
    fp.cur_pos = fp.coll_data.cur_pos;
    if ft_80081a00(fp) {
        return Ok(false);
    }
    if r {
        return Ok(true);
    }
    let env = fp.coll_data.env_flags;
    if fp.can_walljump() != 0 && env & (mc::COLLIDE_RIGHT_WALL_HUG | mc::COLLIDE_LEFT_WALL_HUG) != 0 {
        return Err("ftWallJump_8008169C with a wall hug: not ported".into());
    }
    if !(fp.input.lstick[0].y <= -melee_fighter::hooks::common_data().x480)
        && env & (mc::COLLIDE_LEFT_LEDGE_GRAB | mc::COLLIDE_RIGHT_LEDGE_GRAB) != 0
    {
        return Err("ftCliffCommon_80081298: ledge grab not ported".into());
    }
    Ok(false)
}

/// `lb_8000B1CC(ecb_source.x10C_joint[i], NULL)` for the six ECB parts (`ftData->x44`).
fn ecb_joints(skel: &mut Skeleton, d: &EcbDesc) -> [Vec3; 6] {
    core::array::from_fn(|i| {
        let j = skel.part(d.parts[i] as usize);
        skel.world_pos(j)
    })
}

fn be32(d: &[u8], at: u32) -> u32 {
    u32::from_be_bytes(d[at as usize..at as usize + 4].try_into().unwrap())
}

/// `ftCommon_8008031C`: the stick/trigger activity stats (Player_UpdateJoystickCountByIndex is
/// a stats counter, not hashed and not run here) and the activity timers they reset.
fn ftcommon_8008031c(fp: &mut Fighter) {
    let cd = melee_fighter::hooks::common_data();
    let t = fp.activity_timer;
    if (fc::abs_c(fp.input.lstick[0].x) >= cd.x7B8 && (t.lstick.x as f32) < cd.x7C0)
        || (fc::abs_c(fp.input.lstick[0].y) >= cd.x7B8 && (t.lstick.y as f32) < cd.x7C0)
    {
        fp.activity_timer.lstick.y = 0xFE;
        fp.activity_timer.lstick.x = 0xFE;
    }
    if fc::abs_c(fp.input.triggers[0]) >= cd.x7BC && (fp.activity_timer.trigger as f32) < cd.x7C0 {
        fp.activity_timer.trigger = 0xFE;
    }
}

/// `ftCo_Jump_GetInput` (tap jump on: the default control setting).
fn jump_get_input(fp: &Fighter) -> bool {
    let cd = melee_fighter::hooks::common_data();
    (fp.input.lstick[0].y >= cd.tap_jump_threshold && (fp.active_timer.lstick.y as i32) < cd.tap_jump_window as i32)
        || fp.input.pressed_buttons & input::HSD_PAD_XY != 0
}

/// Any input an IASA check could act on (sticks, C-stick, trigger, presses)?
fn any_input(fp: &Fighter) -> bool {
    let i = &fp.input;
    i.lstick[0].x != 0.0 || i.lstick[0].y != 0.0 || i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 || i.triggers[0] != 0.0
        || i.pressed_buttons != 0 || i.held_buttons[0] != 0
}

/// The motion's `input_cb` (IASA).
fn input_cb(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    match p.fp.motion_id {
        FTCO_MS_ENTRY | FTCO_MS_ENTRY_START | FTCO_MS_ENTRY_END => Ok(()), // empty IASAs
        FTCO_MS_FALL => fall_iasa(p, rand),
        m if attackair::is_attack_air(m) => attackair::iasa(p, rand),
        dashrun::FTCO_MS_DASH => dashrun::dash_iasa(p, rand),
        dashrun::FTCO_MS_RUN => dashrun::run_iasa(p, rand),
        FTCO_MS_LANDING => landing_iasa(p, rand),
        m if is_landing_air(m) => Ok(()), // ftCo_LandingAir_IASA is empty
        FTCO_MS_TURN => turn_iasa(p, rand),
        FTCO_MS_WAIT => wait_iasa(p, rand),
        m if (FTCO_MS_WALK_SLOW..=FTCO_MS_WALK_FAST).contains(&m) => walk_iasa(p, rand),
        FTCA_MS_SPECIAL_LW | FTCA_MS_SPECIAL_LW_END | FTCA_MS_SPECIAL_LW_END_AIR => Ok(()), // input_cb NULL
        FTMS_MS_SPECIAL_LW => Ok(()), // ftMs_SpecialLw_IASA is empty
        m if is_tilt_motion(m) => {
            // ftCo_AttackS3_IASA / ftCo_AttackHi3_IASA: ftCo_Wait_IASA once allowed.
            if p.fp.allow_interrupt() != 0 {
                wait_iasa(p, rand)
            } else {
                Ok(())
            }
        }
        m if (FTCO_MS_ATTACK11..=FTCO_MS_ATTACK11 + 2).contains(&m) => attack1_iasa(p, rand),
        FTCO_MS_CATCH => Ok(()), // ftCo_Catch_IASA is empty
        FTCO_MS_DEAD_DOWN => Ok(()), // input_cb NULL
        rebirth::FTCO_MS_SLEEP | rebirth::FTCO_MS_REBIRTH => Ok(()), // empty IASAs
        rebirth::FTCO_MS_REBIRTH_WAIT => rebirth::wait_iasa(p, rand),
        m if is_guard_motion(m) => guard_iasa(p, rand),
        FTCO_MS_KNEE_BEND => kneebend_iasa(p),
        FTCO_MS_OTTOTTO | FTCO_MS_OTTOTTO_WAIT => ottotto_iasa(p, rand),
        m if is_fx_kind(p.fp.kind) && fox::is_fox_special_n(m) => {
            fox::iasa(p);
            Ok(())
        }
        m if p.fp.kind == melee_types::Ft_Kind_Seak && sheik::is_sheik_special_n(m) => sheik::iasa(p, rand),
        m if is_jump_motion(m) || is_jump_aerial_motion(m) => jump_iasa(p, rand),
        FTCO_MS_FALL_AERIAL => fall_iasa(p, rand),
        m if samus::is_special_s(m) => Ok(()), // ftSs_SpecialS_IASA is empty
        m => Err(format!("player {}: input_cb of motion {m} not ported", p.slot + 1)),
    }
}

/// `ftCo_Fall_IASA_Inner` with no item, no parasol and not Peach: every check but the aerial
/// jump needs a press (B: SpecialAir; A: AttackAir / item checks; L/R: EscapeAir; X/Y: the
/// jump) or the C-stick (`ftCo_800DF478`); the jump also fires on a tap of the stick up
/// (`ftCo_JumpAerial_CheckInput`). Any of those stops the run.
fn fall_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if !p.fp.item_gobj.is_null() {
        return Err("ftCo_Fall_IASA with an item not ported".into());
    }
    if special_air_check(p, rand)? {
        return Ok(());
    }
    let fp = &*p.fp;
    let i = &fp.input;
    if !fp.item_gobj.is_null() {
        return Err("ftCo_Fall_IASA with an item not ported".into());
    }
    // The analog trigger alone reads nothing here (ftCo_80099A58 needs the digital R / L press);
    // L / R held with A pressed is ftCo_800D705C's item grab / the Z-air.
    if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY | input::HSD_PAD_A) != 0 || (i.held_buttons[0] & input::HSD_PAD_LR != 0 && i.pressed_buttons & input::HSD_PAD_A != 0) {
        return Err(format!("player {}: ftCo_Fall_IASA with a press {:#x}: not ported", p.slot + 1, i.pressed_buttons));
    }
    if attackair::check_input(p, rand)? {
        return Ok(());
    }
    let fp = &*p.fp;
    let i = &fp.input;
    if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY) != 0 || i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 {
        return Err(format!("player {}: ftCo_Fall_IASA with a press {:#x} / C-stick ({}, {}): not ported", p.slot + 1, i.pressed_buttons, i.cstick[0].x, i.cstick[0].y));
    }
    // ftCo_800CB870: the double jump.
    ftco_800cb870(p, rand).map(|_| ())
}

/// `ftCo_Landing_IASA`.
fn landing_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let fp = &*p.fp;
    let lag = fp.co_attrs.normal_landing_lag;
    if fp.cur_anim_frame < lag || !p.landing_allow_interrupt {
        return Ok(());
    }
    if special_checks(p, rand)? {
        return Ok(());
    }
    if catch_check(p)? {
        return catch_enter(p, rand);
    }
    if attack_checks(p, rand)? {
        return Ok(());
    }
    // 80091A4C (shield), jump, dash, and the squat check (first frame past the lag only).
    if ground_stick_gate(p, rand, "ftCo_Landing_IASA")? {
        return Ok(());
    }
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    let i = &fp.input;
    // ftCo_Turn_CheckInput -> ftCo_800C97A8.
    if i.lstick[0].x * fp.facing_dir <= cd.x34 {
        return turn_enter_basic(p, rand);
    }
    // ftCo_Walk_CheckInput -> ftWalkCommon_800DFC70.
    if i.lstick[0].x * fp.facing_dir >= cd.walk_stick_threshold {
        return walk_enter(p, 0.0, rand);
    }
    Ok(())
}

/// The stick-only grounded checks every grounded IASA ends with, after the press / C-stick /
/// trigger checks (which stop the run here): jump, dash, squat.
fn ground_stick_gate(p: &mut Player, rand: &mut HsdRand, what: &str) -> Result<bool, String> {
    // ftCo_80091A4C: the shield.
    if guard_check(p, rand)? {
        return Ok(true);
    }
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    let i = &fp.input;
    // X / Y are read by the jump check below (nothing before it in the IASA order does).
    if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY) != 0 || i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 || i.triggers[0] != 0.0 || i.held_buttons[0] & (input::HSD_PAD_Z | input::HSD_PAD_LR) != 0 {
        return Err(format!("player {}: {what} with a button / C-stick / trigger: not ported (pressed {:#x} held {:#x} c ({}, {}) trig {})", p.slot + 1, i.pressed_buttons, i.held_buttons[0], i.cstick[0].x, i.cstick[0].y, i.triggers[0]));
    }
    if jump_get_input(fp) {
        jump_check_input(p, rand)?;
        return Ok(true);
    }
    if fc::abs_c(i.lstick[0].x) >= cd.dash_smash_stick_threshold && (fp.active_timer.lstick.x as i32) < cd.dash_smash_window as i32 {
        dashrun::dash_check_input(p, rand)?;
        return Ok(true);
    }
    let landing_late = what == "ftCo_Landing_IASA" && fp.cur_anim_frame >= fp.frame_speed_mul + fp.co_attrs.normal_landing_lag;
    if i.lstick[0].y < -cd.x90 && !landing_late {
        return Err(format!("player {}: ftCo_800D5FB0 (Squat) not ported", p.slot + 1));
    }
    Ok(false)
}

/// `ftCo_Catch_CheckInput` (ftCo_800951D0 needs an item; fn_800D8E94 / fn_800D952C pass for
/// the kinds wired): L/R held and A pressed.
fn catch_check(p: &Player) -> Result<bool, String> {
    let fp = &*p.fp;
    if !fp.item_gobj.is_null() {
        return Err("ftCo_800951D0 (item throw) not ported".into());
    }
    if fp.input.held_buttons[0] & input::HSD_PAD_LR != 0 && fp.input.pressed_buttons & input::HSD_PAD_A != 0 {
        return Ok(true);
    }
    Ok(false)
}

/// `ftCo_800D8C54(gobj, ftCo_MS_Catch)`: self accel cleared, `mv.co.catch_.x0 = 0`, Catch.
/// `ftCommon_8007E2D0`'s grab callbacks are not modelled (a grab needs hit detection, which
/// [`VsMatch::hit_guard`] stops before).
fn catch_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let fp = &mut *p.fp;
    fp.x74_self_accel.z = 0.0;
    fp.x74_self_accel.y = 0.0;
    fp.x74_self_accel.x = 0.0;
    let row = load_row(p, melee_types::ftCo_SM_Catch as u32)?;
    change_motion_state(p, FTCO_MS_CATCH, row.as_ref(), rand)
}

/// The grounded special checks, in the IASAs' order: `ftCo_SpecialS_CheckInput`,
/// `ftCo_Attack100_CheckInput` (up-B), `ftCo_800D6824` (neutral B), `ftCo_800D68C0` (down-B).
/// Each reads its press counter (`x688`, `x686`, `x689`, `x687`: 0 on the frame of the press,
/// Fighter_Spaghetti_8006AD10); `ftData_Special*[kind]` is set for every kind wired.
fn special_checks(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let fp = &*p.fp;
    let who = p.slot + 1;
    if fp.x688 == 0 {
        if fp.kind == melee_types::Ft_Kind_Samus {
            samus::special_s_enter(p, rand)?;
            return Ok(true);
        }
        return Err(format!("player {who}: ftCo_SpecialS_CheckInput (side-B, kind {}) not ported", fp.kind));
    }
    if fp.x686 == 0 {
        return Err(format!("player {who}: ftCo_Attack100_CheckInput (up-B, kind {}) not ported", fp.kind));
    }
    if fp.x689 == 0 {
        if fp.kind == melee_types::Ft_Kind_Seak {
            sheik::special_n_enter(p, false, rand)?;
            return Ok(true);
        }
        if is_fx_kind(fp.kind) {
            fox::special_n_enter(p, false, rand)?;
            return Ok(true);
        }
        return Err(format!("player {who}: ftCo_800D6824 (neutral B, kind {}) not ported", fp.kind));
    }
    if fp.x687 == 0 {
        return match fp.kind {
            Ft_Kind_Captain => ftca_speciallw_enter(p, rand).map(|_| true),
            melee_types::Ft_Kind_Mars => ftms_speciallw_enter(p, rand).map(|_| true),
            k => Err(format!("player {who}: ftCo_800D68C0 (down-B, kind {k}) not ported")),
        };
    }
    Ok(false)
}

/// The grounded attack checks of the IASAs (`ftCo_AttackS4/Hi4/Lw4/S3/Hi3/Lw3_CheckInput`,
/// then `ftCo_Attack1_CheckInput`). Only the jab is wired: with A pressed, a neutral stick and
/// no C-stick no smash or tilt check passes (each needs a stick past a positive threshold);
/// anything else stops the run. `ftCo_Attack1_CheckInput` without A counts the jab window
/// (`hitlag_mul`) down.
fn attack_checks(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let fp = &mut *p.fp;
    let i = &fp.input;
    let a = i.pressed_buttons & input::HSD_PAD_A != 0;
    if i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 {
        return Err(format!("player {}: C-stick smash / tilt checks not ported", p.slot + 1));
    }
    if !a {
        if fp.hitlag_mul > 0.0 {
            fp.hitlag_mul -= 1.0;
        }
        return Ok(false);
    }
    if i.lstick[0].x != 0.0 || i.lstick[0].y != 0.0 {
        // ftCo_AttackS4 / Hi4 / Lw4_CheckInput (the smash flicks): a stick that passed the
        // threshold inside the window stops the run; ftCo_AttackS3 / Hi3 / Lw3_CheckInput (tilts).
        let cd = melee_fighter::hooks::common_data();
        let who = p.slot + 1;
        if fc::abs_c(i.lstick[0].x) >= cd.dash_smash_stick_threshold && (fp.active_timer.lstick.x as i32) < cd.dash_smash_window as i32 {
            return Err(format!("player {who}: ftCo_AttackS4 (forward smash) not ported"));
        }
        if i.lstick[0].y >= cd.xCC && (fp.active_timer.lstick.y as i32) < cd.xD0 as i32 {
            return Err(format!("player {who}: ftCo_AttackHi4 (up smash) not ported"));
        }
        if i.lstick[0].y <= cd.xD4 && (fp.active_timer.lstick.y as i32) < cd.xD8 as i32 {
            return Err(format!("player {who}: ftCo_AttackLw4 (down smash) not ported"));
        }
        // ftCo_GetLStickAngle: atan2f(y, |x|).
        let angle = melee_runtime::lbtrigf::atan2f(i.lstick[0].y, fc::abs_c(i.lstick[0].x));
        if i.lstick[0].x * fp.facing_dir >= cd.x98 && fc::abs_c(angle) < cd.x20_radians {
            if !fp.item_gobj.is_null() {
                return Err("ftCo_AttackS3 with an item not ported".into());
            }
            return attack_s3_enter(p, angle, rand).map(|_| true);
        }
        if i.lstick[0].y >= cd.attackhi3_stick_threshold_y && angle > cd.x20_radians {
            if !fp.item_gobj.is_null() {
                return Err("ftCo_AttackHi3 with an item not ported".into());
            }
            fp.set_allow_interrupt(0);
            let row = load_row(p, melee_types::ftCo_SM_AttackHi3 as u32)?;
            change_motion_state(p, melee_types::ftCo_MS_AttackHi3 as i32, row.as_ref(), rand)?;
            anim_step(p, rand)?;
            return Ok(true);
        }
        if i.lstick[0].y <= cd.xB0 && angle < -cd.x20_radians {
            return Err(format!("player {who}: ftCo_AttackLw3 (down tilt) not ported"));
        }
    }
    // ftCo_Attack1_CheckInput.
    if !fp.item_gobj.is_null() {
        return Err("ftCo_Attack1_CheckInput with an item not ported".into());
    }
    if fp.hitlag_mul > 0.0 && fp.x2218_b1() != 0 && (fp.unk_msid == 44 || fp.unk_msid == 45) {
        return Err(format!("player {}: jab 2/3 from the jab window (doAttack12/13) not ported", p.slot + 1));
    }
    if fp.kind == melee_types::Ft_Kind_GameWatch {
        return Err("ftGw_Attack11_Enter not ported".into());
    }
    attack11_enter(p, rand)?;
    Ok(true)
}

/// `ftCo_AttackS3.c decideAngle`: the angled forward tilt. `ftData_80085FD4(fp, msid)->x8` is the
/// size of `ftData->xC[msid]`'s animation (the table is indexed by the motion id as is).
fn attack_s3_enter(p: &mut Player, angle: f32, rand: &mut HsdRand) -> Result<(), String> {
    use melee_types::{ftCo_MS_AttackHi3, ftCo_MS_AttackLw3, ftCo_MS_AttackS3Hi, ftCo_MS_AttackS3HiS, ftCo_MS_AttackS3Lw, ftCo_MS_AttackS3LwS, ftCo_MS_AttackS3S};
    let cd = melee_fighter::hooks::common_data();
    let has = |p: &Player, m: u32| -> Result<bool, String> { Ok(p.files.anim_row(m)?.is_some()) };
    let msid = if angle > cd.x9C_radians && has(p, ftCo_MS_AttackS3S as u32)? {
        ftCo_MS_AttackS3Hi
    } else if angle > cd.xA0_radians && has(p, ftCo_MS_AttackS3LwS as u32)? {
        ftCo_MS_AttackS3HiS
    } else if angle < cd.xA8_radians && has(p, ftCo_MS_AttackLw3 as u32)? {
        ftCo_MS_AttackS3Lw
    } else if angle < cd.xA4_radians && has(p, ftCo_MS_AttackHi3 as u32)? {
        ftCo_MS_AttackS3LwS
    } else {
        ftCo_MS_AttackS3S
    } as i32;
    p.fp.set_allow_interrupt(0);
    // The submotion rows follow the motion states by two (ftCo_SM_AttackS3Hi = 53 = MS 51 + 2).
    let row = load_row(p, (msid + 2) as u32)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    anim_step(p, rand)
}

/// The grounded forward / up tilts (`ftCo_AttackS3` x5, `ftCo_AttackHi3`).
fn is_tilt_motion(m: i32) -> bool {
    (melee_types::ftCo_MS_AttackS3Hi as i32..=melee_types::ftCo_MS_AttackHi3 as i32).contains(&m)
}

/// `checkAttack11` (ftCo_Attack1.c): no item to pick up (`ftpickupitem_80094790`: no items
/// here), Attack11 (`Ft_MF_None` for the kinds wired), one `ftAnim_8006EBA4`, the jab window.
fn attack11_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    p.fp.set_allow_interrupt(0);
    p.fp.set_x2218_b1(0);
    let row = load_row(p, melee_types::ftCo_SM_Attack11 as u32)?;
    change_motion_state(p, FTCO_MS_ATTACK11, row.as_ref(), rand)?;
    anim_step(p, rand)?;
    let fp = &mut *p.fp;
    fp.hitlag_mul = fp.co_attrs.jab_2_input_window;
    fp.unk_msid = FTCO_MS_ATTACK11;
    fp.set_x2218_b2(0);
    p.attack1_x0 = false;
    fp.x1A54 = 0;
    Ok(())
}

/// `doAttack12Normal` / `doAttack13` (not Pikachu / Pichu / Marth): Attack12 or Attack13, with
/// no `ftAnim_8006EBA4` step.
fn attack1n_enter(p: &mut Player, rand: &mut HsdRand, msid: i32) -> Result<(), String> {
    if matches!(p.fp.kind, melee_types::Ft_Kind_Pikachu | melee_types::Ft_Kind_Pichu | melee_types::Ft_Kind_Mars) {
        return Err("rapid-jab kinds' Attack12/13 not ported".into());
    }
    p.fp.set_allow_interrupt(0);
    p.fp.set_x2218_b1(0);
    let smid = if msid == FTCO_MS_ATTACK11 + 1 { melee_types::ftCo_SM_Attack12 } else { melee_types::ftCo_SM_Attack13 } as u32;
    let row = load_row(p, smid)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    if msid == FTCO_MS_ATTACK11 + 1 {
        let fp = &mut *p.fp;
        fp.hitlag_mul = fp.co_attrs.jab_3_input_window;
        fp.unk_msid = msid;
        p.attack1_x0 = false;
    }
    Ok(())
}

/// `ftCo_Attack11_IASA` / `ftCo_Attack12_IASA` (Attack13's is `ftCo_Attack_800D6A50` + Wait's).
fn attack1_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let allow = p.fp.allow_interrupt() != 0;
    if allow {
        let i = &p.fp.input;
        if i.pressed_buttons & input::HSD_PAD_A != 0 && (i.lstick[0].x != 0.0 || i.lstick[0].y != 0.0) || i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 {
            return Err(format!("player {}: jab IASA smash / tilt checks not ported", p.slot + 1));
        }
    }
    // ftCo_Attack_800D6A50: A presses and releases count toward the rapid jab.
    {
        let fp = &mut *p.fp;
        if (fp.input.pressed_buttons | fp.input.released_buttons) & input::HSD_PAD_A != 0 {
            fp.x1A54 += 1;
        }
        if fp.x1A54 >= fp.co_attrs.rapid_jab_window && fp.x2218_b2() != 0 {
            return Err(format!("player {}: rapid jab (fn_800D6AC4) not ported", p.slot + 1));
        }
    }
    if p.fp.motion_id == FTCO_MS_ATTACK11 + 2 {
        if allow {
            return wait_iasa(p, rand);
        }
        return Ok(());
    }
    // checkAttack12 / checkAttack13.
    {
        let fp = &mut *p.fp;
        if fp.hitlag_mul > 0.0 {
            fp.hitlag_mul -= 1.0;
            if fp.input.pressed_buttons & input::HSD_PAD_A != 0 {
                p.attack1_x0 = true;
            }
        }
        if p.attack1_x0 && fp.x2218_b1() != 0 {
            let next = fp.motion_id + 1;
            return attack1n_enter(p, rand, next);
        }
    }
    if allow {
        if ground_stick_gate(p, rand, "ftCo_Attack11_IASA")? {
            return Ok(());
        }
        let cd = melee_fighter::hooks::common_data();
        let fp = &*p.fp;
        if fp.input.lstick[0].x * fp.facing_dir <= cd.x34 {
            return turn_enter_basic(p, rand);
        }
        if fp.input.lstick[0].x * fp.facing_dir >= cd.walk_stick_threshold {
            return walk_enter(p, 0.0, rand);
        }
    }
    Ok(())
}

/// `ftCo_Wait_IASA`.
fn wait_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if special_checks(p, rand)? {
        return Ok(());
    }
    if catch_check(p)? {
        return catch_enter(p, rand);
    }
    if attack_checks(p, rand)? {
        return Ok(());
    }
    if ground_stick_gate(p, rand, "ftCo_Wait_IASA")? {
        return Ok(());
    }
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    if fp.input.lstick[0].x * fp.facing_dir <= cd.x34 {
        return turn_enter_basic(p, rand);
    }
    if fp.input.lstick[0].x * fp.facing_dir >= cd.walk_stick_threshold {
        return walk_enter(p, 0.0, rand);
    }
    Ok(())
}



// ---------------------------------------------------------------------------------------------
// ftCo_Jump.c / ftCo_KneeBend.c: the ground jump (KneeBend -> JumpF / JumpB).
// ---------------------------------------------------------------------------------------------

/// `ftCo_JumpInput` (ftCommon/forward.h).
const JUMP_INPUT_NONE: i32 = 0;
const JUMP_INPUT_LSTICK: i32 = 1;
const JUMP_INPUT_CSTICK: i32 = 2;
const JUMP_INPUT_XY: i32 = 3;

fn is_jump_motion(m: i32) -> bool {
    m == FTCO_MS_JUMP_F || m == FTCO_MS_JUMP_B
}

fn is_jump_aerial_motion(m: i32) -> bool {
    m == FTCO_MS_JUMP_AERIAL_F || m == FTCO_MS_JUMP_AERIAL_B
}

/// `ftCo_Jump_GetInput` (tap jump on).
fn jump_input_kind(fp: &Fighter) -> i32 {
    let cd = melee_fighter::hooks::common_data();
    if fp.input.lstick[0].y >= cd.tap_jump_threshold && (fp.active_timer.lstick.y as i32) < cd.tap_jump_window as i32 {
        return JUMP_INPUT_LSTICK;
    }
    if fp.input.pressed_buttons & input::HSD_PAD_XY != 0 {
        return JUMP_INPUT_XY;
    }
    JUMP_INPUT_NONE
}

/// `ftCo_Jump_CheckInput` (no item): the KneeBend when a jump was asked for.
fn jump_check_input(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    if !p.fp.item_gobj.is_null() {
        return Err("ftCo_Jump_CheckInput with an item (ftCo_800C5240) not ported".into());
    }
    let k = jump_input_kind(&p.fp);
    if k != JUMP_INPUT_NONE {
        kneebend_enter(p, k, rand)?;
        return Ok(true);
    }
    Ok(false)
}

/// `ftCo_800CB024`: `ftCo_Jump_CheckInput`, then a C-stick flick up.
fn ftco_800cb024(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    if jump_check_input(p, rand)? {
        return Ok(true);
    }
    if p.fp.input.cstick[0].y >= melee_fighter::hooks::common_data().tap_jump_threshold {
        kneebend_enter(p, JUMP_INPUT_CSTICK, rand)?;
        return Ok(true);
    }
    Ok(false)
}

/// `ftCo_KneeBend_Enter`.
fn kneebend_enter(p: &mut Player, input_kind: i32, rand: &mut HsdRand) -> Result<(), String> {
    p.jump.jump_input = input_kind;
    p.jump.is_short_hop = false;
    let row = load_row(p, FTCO_SM_KNEE_BEND)?;
    change_motion_state(p, FTCO_MS_KNEE_BEND, row.as_ref(), rand)
}

/// `ftCo_KneeBend_Anim`.
fn kneebend_anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.fp.cur_anim_frame >= p.fp.co_attrs.jump_startup_time || !p.skel.frames_remaining() {
        // ft_800D2D0C (a screw-attack item) is false: ftCo_Jump_Enter.
        jump_enter(p, rand)?;
    }
    Ok(())
}

/// `ftCo_KneeBend_IASA`.
fn kneebend_iasa(p: &mut Player) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    let who = p.slot + 1;
    let fp = &*p.fp;
    let i = &fp.input;
    if fp.x686 == 0 {
        return Err(format!("player {who}: ftCo_Attack100_CheckInput (up-B) not ported"));
    }
    if i.held_buttons[0] & input::HSD_PAD_LR != 0 && i.pressed_buttons & input::HSD_PAD_A != 0 {
        return Err(format!("player {who}: ftCo_Catch_CheckInput out of KneeBend not ported"));
    }
    if (i.pressed_buttons & input::HSD_PAD_A != 0 && i.lstick[0].y >= cd.xCC) || (i.cstick[1].y < cd.xCC && i.cstick[0].y >= cd.xCC) {
        return Err(format!("player {who}: ftCo_AttackHi4 out of KneeBend not ported"));
    }
    // ftCo_KneeBend_Check_ShortHop.
    let released = (i.held_buttons[0] & input::HSD_PAD_XY == 0 && p.jump.jump_input == JUMP_INPUT_XY)
        || (i.lstick[0].y < cd.tap_jump_release_threshold && p.jump.jump_input == JUMP_INPUT_LSTICK)
        || (i.cstick[0].y < cd.tap_jump_release_threshold && p.jump.jump_input == JUMP_INPUT_CSTICK);
    if released {
        p.jump.is_short_hop = true;
    }
    Ok(())
}

/// `ftCo_Jump_Enter` (+ `ftCo_800CB110(gobj, true, 1.0)`).
fn jump_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp);
    let fp = &*p.fp;
    let (msid, smid) = if fp.input.lstick[0].x * fp.facing_dir > -cd.x78 { (FTCO_MS_JUMP_F, FTCO_SM_KNEE_BEND + 1) } else { (FTCO_MS_JUMP_B, FTCO_SM_KNEE_BEND + 2) };
    let row = load_row(p, smid)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    // ftCo_800CB110(gobj, true, 1.0): jump_mul = 1.
    p.jump.x4 = false;
    let jump_mul = 1.0f32;
    let hop = p.jump.is_short_hop;
    let fp = &mut *p.fp;
    let a = &fp.co_attrs;
    let (hi, vi, hv, mom, hmax) = (a.jump_h_initial_velocity, a.jump_v_initial_velocity, a.hop_v_initial_velocity, a.ground_to_air_jump_momentum_multiplier, a.jump_h_max_velocity);
    fp.self_vel.x *= mom * jump_mul;
    fp.self_vel.y *= cd.x438;
    fp.self_vel.z = 0.0;
    let h_init_v = fp.input.lstick[0].x * hi;
    let mut h_vel = jump_mul * h_init_v;
    let v_init_v = if hop { hv * jump_mul } else { vi * jump_mul };
    fp.self_vel.y = v_init_v;
    h_vel = fp.self_vel.x + h_vel;
    let h_max_vel = hmax * jump_mul;
    if fc::abs_c(h_vel) > h_max_vel {
        h_vel = if h_vel < 0.0 { -h_max_vel } else { h_max_vel };
    }
    fp.self_vel.x = h_vel;
    fp.active_timer.lstick.y = 0xFE;
    // (the jump SFX: ft_800881D8 / ft_PlaySFX, no draws.)
    fp.set_x2227_b0(1);
    Ok(())
}

/// `ftCo_Jump_IASA` / `ftCo_JumpAerial_IASA`: the air checks (specials, air dodge, air attack)
/// are refused when an input would fire; then the double jump (`ftCo_800CB870`).
fn jump_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if !p.fp.item_gobj.is_null() {
        return Err("ftCo_Jump_IASA with an item not ported".into());
    }
    if special_air_check(p, rand)? {
        return Ok(());
    }
    let who = p.slot + 1;
    let fp = &*p.fp;
    let i = &fp.input;
    if !fp.item_gobj.is_null() {
        return Err("ftCo_Jump_IASA with an item not ported".into());
    }
    if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY | input::HSD_PAD_A) != 0 || i.triggers[0] != 0.0 {
        return Err(format!("player {who}: ftCo_Jump_IASA with a press / trigger: not ported"));
    }
    if attackair::check_input(p, rand)? {
        return Ok(());
    }
    let fp = &*p.fp;
    let i = &fp.input;
    if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY) != 0 || i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 || i.triggers[0] != 0.0 {
        return Err(format!("player {who}: ftCo_Jump_IASA with a press / C-stick / trigger: not ported"));
    }
    ftco_800cb870(p, rand).map(|_| ())
}

/// `ftCo_800CB870` -> `ftCo_JumpAerial_CheckInput(gobj, false)`.
fn ftco_800cb870(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    if fp.can_multijump() != 0 {
        return Err(format!("player {}: ftCo_800D730C (multi-jump) not ported", p.slot + 1));
    }
    // ft_did_jump(fp, false).
    let tap = fp.input.lstick[0].y >= cd.tap_jump_threshold && (fp.active_timer.lstick.y as i32) < cd.tap_jump_window as i32;
    if (fp.x1968_jumpsUsed as i32) < fp.co_attrs.max_jumps as i32 && (tap || fp.input.pressed_buttons & input::HSD_PAD_XY != 0) {
        match fp.kind {
            melee_types::Ft_Kind_Ness | melee_types::Ft_Kind_Yoshi | melee_types::Ft_Kind_Peach | melee_types::Ft_Kind_Mewtwo => {
                return Err(format!("player {}: character-specific double jump not ported", p.slot + 1));
            }
            _ => {}
        }
        jump_aerial_enter(p, rand)?;
        return Ok(true);
    }
    Ok(false)
}

/// `ftCo_JumpAerial_Enter_Basic` + `ftCo_800CBAC4`.
fn jump_aerial_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    // ft_JumpAerial_MarkAir.
    let used = p.fp.x1968_jumpsUsed;
    melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp);
    if used > 1 {
        p.fp.x1968_jumpsUsed = used;
    }
    p.fp.cmd_vars[0] = 1;
    let fp = &*p.fp;
    let (msid, smid) = if fp.input.lstick[0].x * fp.facing_dir > -cd.x78 { (FTCO_MS_JUMP_AERIAL_F, FTCO_SM_JUMP_AERIAL_F) } else { (FTCO_MS_JUMP_AERIAL_B, FTCO_SM_JUMP_AERIAL_F + 1) };
    let vx = fp.input.lstick[0].x * fp.co_attrs.air_jump_h_multiplier;
    let vy = fp.co_attrs.jump_v_initial_velocity * fp.co_attrs.air_jump_v_multiplier;
    let row = load_row(p, smid)?;
    change_motion_state(p, msid, row.as_ref(), rand)?;
    let fp = &mut *p.fp;
    fp.set_x2221_b7(1);
    fp.self_vel = melee_types::Vec { x: vx, y: vy, z: 0.0 };
    fp.active_timer.lstick.y = 0xFE;
    fp.x1968_jumpsUsed += 1;
    Ok(())
}

/// `ftCo_FallAerial_Enter`.
fn fall_aerial_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let row = load_row(p, FTCO_SM_FALL_AERIAL)?;
    change_motion_state(p, FTCO_MS_FALL_AERIAL, row.as_ref(), rand)?;
    p.fall_smid = FTCO_SM_FALL_AERIAL as i32;
    p.fall_x4 = 0.0;
    Ok(())
}

/// `ftCo_SpecialAir_CheckInput`: B in the air.
fn special_air_check(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    if fp.input.pressed_buttons & input::HSD_PAD_B == 0 {
        return Ok(false);
    }
    let who = p.slot + 1;
    if fp.input.lstick[0].y >= cd.x21C {
        return Err(format!("player {who}: up-B in the air (ftData_SpecialAirHi) not ported"));
    }
    if fp.input.lstick[0].y <= -cd.x21C {
        return Err(format!("player {who}: down-B in the air (ftData_SpecialAirLw) not ported"));
    }
    if fc::abs_c(fp.input.lstick[0].x) >= cd.x218 {
        return Err(format!("player {who}: side-B in the air (ftData_SpecialAirS) not ported"));
    }
    if !is_fx_kind(fp.kind) && fp.kind != melee_types::Ft_Kind_Seak {
        return Err(format!("player {who}: neutral B in the air (kind {}) not ported", fp.kind));
    }
    // The turnaround on a fresh stick flick.
    let fp = &mut *p.fp;
    if (fp.active_duration.lstick.x as i32) < cd.x224 as i32 && ((fp.facing_dir == -1.0 && fp.x2228_b7() == 1) || (fp.facing_dir == 1.0 && fp.x2228_b7() == 0)) {
        fp.facing_dir = -fp.facing_dir;
    }
    if p.fp.kind == melee_types::Ft_Kind_Seak {
        sheik::special_n_enter(p, true, rand)?;
    } else {
        fox::special_n_enter(p, true, rand)?;
    }
    p.fp.set_x2227_b5(1);
    Ok(true)
}

/// `ftCo_Jump_Phys`: the first frame only arms `x4`; then `ft_80084DB0`.
fn jump_phys(p: &mut Player) {
    if !p.jump.x4 {
        p.jump.x4 = true;
        return;
    }
    let f = &mut *p.fp;
    fc::check_fall_fast(f);
    if f.fall_fast() != 0 {
        fc::fall_fast(f);
    } else {
        let (g, t) = (f.co_attrs.gravity, f.co_attrs.terminal_velocity);
        fc::fall(f, g, t);
    }
    fc::calc_self_accel_drift(f);
}

// ---------------------------------------------------------------------------------------------
// ftCo_Guard.c. Not modelled (nothing in a hash reads them): the shield bubble's pose blending
// (`ftCo_80091E78`: bone positions and the bubble's scale), the shield / reflect hit records
// (`ftColl_8007B1B8`, `ftColl_CreateReflectHit`: only their flags are kept), SFX and the stats.
// ---------------------------------------------------------------------------------------------

fn is_guard_motion(m: i32) -> bool {
    (FTCO_MS_GUARD_ON..=FTCO_MS_GUARD_REFLECT).contains(&m)
}

/// `ftCo_80091A4C`: a shoulder press starts the shield (a power shield inside the input window).
fn guard_check(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    let window = melee_fighter::hooks::common_data().powershield_input_window;
    let fp = &*p.fp;
    if fp.input.pressed_buttons & (input::HSD_PAD_R | input::HSD_PAD_L) != 0 && (fp.active_timer.trigger as i32) < window {
        ftco_800939b4(p, rand)?;
        return Ok(true);
    }
    if fp.input.held_buttons[0] & input::HSD_PAD_LR != 0 && fp.shield_health != 0.0 {
        ftco_800923b4(p, rand)?;
        return Ok(true);
    }
    Ok(false)
}

fn guard_kind_check(p: &Player) -> Result<(), String> {
    match p.fp.kind {
        melee_types::Ft_Kind_Yoshi => Err(format!("player {}: ftCo_Guard for Yoshi not ported", p.slot + 1)),
        _ => Ok(()),
    }
}

/// `ftCo_80092450_inline`: `ftColl_8007B1B8` (the shield hit record).
fn guard_shield_up(p: &mut Player) {
    p.guard.shield_on = true;
    p.fp.set_x221A_b7(1);
}

/// `ftCo_800939B4` -> `ftCo_80093A50`: the power shield.
fn ftco_800939b4(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    guard_kind_check(p)?;
    let cd = melee_fighter::hooks::common_data();
    let row = load_row(p, FTCO_SM_GUARD_ON)?;
    change_motion_state_ext(p, FTCO_MS_GUARD_REFLECT, row.as_ref(), 0.0, 1.0, true, rand)?;
    anim_step(p, rand)?; // ftAnim_8006EBA4
    let fp = &mut *p.fp;
    fp.active_timer.trigger = 0xFE;
    fp.set_x221C_b3(1);
    fp.set_x221C_b1(1);
    fp.set_x221C_b2(1);
    p.guard.x1c = 0;
    p.guard.x14 = cd.x2A4;
    p.guard.x18 = cd.x2B4;
    guard_shield_up(p);
    // ftCo_8009370C: ftColl_CreateReflectHit.
    p.fp.set_reflecting(1);
    ftco_800921dc(p)
}

/// `ftCo_800923B4` -> `ftCo_800924C0`: GuardOn.
fn ftco_800923b4(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    guard_kind_check(p)?;
    let row = load_row(p, FTCO_SM_GUARD_ON)?;
    change_motion_state_ext(p, FTCO_MS_GUARD_ON, row.as_ref(), 0.0, 1.0, true, rand)?;
    anim_step(p, rand)?; // ftAnim_8006EBA4
    let fp = &mut *p.fp;
    fp.set_x221C_b3(0);
    fp.set_x221C_b1(0);
    fp.set_x221C_b2(0);
    p.guard.x1c = 0;
    p.guard.x14 = 0.0;
    p.guard.x18 = 0.0;
    guard_shield_up(p);
    if p.fp.kind == melee_types::Ft_Kind_Mars {
        // ftCo_800923B4's Marth case: ftParts_80074B0C(gobj, 1, 1) (the sword's model part,
        // x221D_b2) and the shield sound (ft_PlaySFX 190115): render and audio only.
        p.fp.set_x221D_b2(1);
    }
    ftco_800921dc(p)
}

/// `ftCo_800921DC`: the shield's variables and bubble (effect 1047 = 0x417, `efLib_Create_Attach(0xB)`).
fn ftco_800921dc(p: &mut Player) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    p.ef_ops.push(EfOp::Spawn(QueuedEffect { kind: EF_SPAWN_SYNC, gfx: 1047, part: 0, params: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, extra1: 0.0, extra2: 0.0 }));
    let fp = &mut *p.fp;
    fp.set_x2219_b0(1);
    let g = &mut p.guard;
    g.xc = false;
    g.x0 = 0.0;
    g.x10 = cd.x268;
    g.x8 = 10.0;
    g.x4 = 0.0;
    g.x2c = 0.0;
    let dz = cd.analog_shoulder_deadzone;
    let mut lsa = (fp.input.triggers[0] - dz) / (1.0 - dz);
    if lsa < 0.0 {
        lsa = g.x2c;
    }
    fp.lightshield_amount = lsa;
    g.x20 = 0;
    g.x24 = 0;
    Ok(())
}

/// `ftCo_800925A4`: the shield's per-frame cost. True when it broke.
fn ftco_800925a4(p: &mut Player) -> Result<bool, String> {
    let cd = melee_fighter::hooks::common_data();
    if !p.guard.shield_on {
        return Ok(false);
    }
    let fp = &mut *p.fp;
    p.guard.x2c = fp.lightshield_amount;
    let dz = cd.analog_shoulder_deadzone;
    let v = (fp.input.triggers[0] - dz) / (1.0 - dz);
    fp.lightshield_amount = if v < 0.0 { p.guard.x2c } else { v };
    // shield_health -= x278 * (lsa * (x2F0 - x2EC) + x2EC), contracted.
    let inner = melee_core::fp::fmadds(fp.lightshield_amount, cd.x2F0 - cd.x2EC, cd.x2EC);
    fp.shield_health = melee_core::fp::fmadds(-cd.x278, inner, fp.shield_health);
    if fp.shield_health < 0.0 {
        return Err(format!("player {}: shield break (ftCo_80098B20) not ported", p.slot + 1));
    }
    if p.guard.x10 > 0.0 {
        p.guard.x10 -= 1.0;
        if p.guard.x10 < 0.0 {
            p.guard.x10 = 0.0;
        }
    }
    Ok(false)
}

/// `ftCo_80093BC0`: the reflect window's timers (GuardReflect_Anim).
fn ftco_80093bc0(p: &mut Player) {
    if p.fp.x221C_b3() != 0 {
        p.fp.set_x221C_b3(0);
    }
    if p.fp.x221C_b1() != 0 {
        p.guard.x14 -= 1.0;
        if p.guard.x14 < 0.0 {
            p.fp.set_x221C_b1(0);
            p.fp.set_reflecting(0);
            guard_shield_up(p);
        }
    }
    if p.fp.x221C_b2() != 0 {
        p.guard.x18 -= 1.0;
        if p.guard.x18 < 0.0 {
            p.fp.set_x221C_b2(0);
        }
    }
}

/// GuardOn_Anim / the inline in GuardReflect_Anim.
fn guard_on_anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    p.guard.x0 += 1.0;
    if !ftco_800925a4(p)? && p.guard.x0 >= p.fp.x2E8 {
        // ftCo_800928CC -> ftCo_80092908: Guard (SkipAnim), the bubble 1048.
        guard_kind_check(p)?;
        let row = load_row(p, FTCO_SM_GUARD_ON + 1)?;
        change_motion_state_ext(p, FTCO_MS_GUARD, row.as_ref(), 0.0, 1.0, true, rand)?;
        p.ef_ops.push(EfOp::Spawn(QueuedEffect { kind: EF_SPAWN_SYNC, gfx: 1048, part: 0, params: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, extra1: 0.0, extra2: 0.0 }));
        p.fp.set_x2219_b0(1);
        guard_shield_up(p);
    }
    Ok(())
}

/// `ftCo_80092BCC` + the exit test shared by the shield IASAs (`inlineC0`): true when the
/// shield was let go (GuardOff entered).
fn guard_release(p: &mut Player, rand: &mut HsdRand) -> Result<bool, String> {
    if p.fp.input.held_buttons[0] & input::HSD_PAD_LR == 0 {
        p.guard.xc = true;
    }
    if (p.guard.xc && p.guard.x10 == 0.0) || (!p.guard.shield_on && p.fp.reflecting() == 0) {
        // ftCo_80092BE8 -> ftCo_80092C54: GuardOff.
        guard_kind_check(p)?;
        let row = load_row(p, FTCO_SM_GUARD_OFF)?;
        change_motion_state(p, FTCO_MS_GUARD_OFF, row.as_ref(), rand)?;
        return Ok(true);
    }
    if p.guard.x1c != 0 {
        p.guard.x1c -= 1;
    }
    Ok(false)
}

/// The input checks the shield IASAs end with (item throw, spot dodge, roll, `800D8B9C`, grab,
/// jump, drop through): refused when one would fire.
fn guard_inputs(p: &mut Player, rand: &mut HsdRand, x800d8b9c: bool) -> Result<bool, String> {
    let cd = melee_fighter::hooks::common_data();
    let who = p.slot + 1;
    let fp = &*p.fp;
    let i = &fp.input;
    // ftCo_8009515C: no item; the throw timer.
    if p.guard.x20 != 0 {
        p.guard.x20 -= 1;
    }
    // ftCo_8009980C (spot dodge).
    if (i.lstick[0].y <= cd.x314 && (fp.active_timer.lstick.y as i32) < cd.x318) || i.cstick[0].y <= cd.x314 {
        return Err(format!("player {who}: shield spot dodge (ftCo_80099894) not ported"));
    }
    // ftCo_8009917C (roll).
    if (fc::abs_c(i.lstick[0].x) >= cd.x31C && (fp.active_timer.lstick.x as i32) < cd.x320) || fc::abs_c(i.cstick[0].x) >= cd.x31C {
        return Err(format!("player {who}: shield roll (ftCo_800992A8) not ported"));
    }
    if x800d8b9c {
        if i.pressed_buttons & input::HSD_PAD_A != 0 && p.guard.x24 != 0 {
            return Err(format!("player {who}: shield grab (ftCo_800D8C54) not ported"));
        }
        if p.guard.x24 != 0 {
            p.guard.x24 -= 1;
        }
    }
    // ftCo_Catch_CheckInput (shield + A).
    if i.held_buttons[0] & input::HSD_PAD_LR != 0 && i.pressed_buttons & input::HSD_PAD_A != 0 {
        return Err(format!("player {who}: shield grab (ftCo_Catch_CheckInput) not ported"));
    }
    // ftCo_800CB024: jump out of the shield.
    if jump_get_input(fp) || i.cstick[0].y >= cd.tap_jump_threshold {
        ftco_800cb024(p, rand)?;
        return Ok(true);
    }
    // ftCo_8009A080: drop through a platform (the floor here is solid).
    if i.held_buttons[0] & input::HSD_PAD_LR != 0 && i.lstick[0].y <= -cd.x464 && (fp.active_timer.lstick.y as i32) < cd.x468 as i32 {
        return Err(format!("player {who}: shield drop (ftCo_8009A228) not ported"));
    }
    Ok(false)
}

/// GuardOn_IASA / Guard_IASA / GuardReflect_IASA / GuardOff_IASA.
fn guard_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let m = p.fp.motion_id;
    if m == FTCO_MS_GUARD_OFF {
        // ftCo_GuardOff_IASA: with the reflect lag (x1C) cleared, spot dodge and jump.
        if p.guard.x1c != 0 {
            return Err(format!("player {}: GuardOff IASA with the reflect lag not ported", p.slot + 1));
        }
        let cd = melee_fighter::hooks::common_data();
        let fp = &*p.fp;
        if (fp.input.lstick[0].y <= cd.x314 && (fp.active_timer.lstick.y as i32) < cd.x318) || fp.input.cstick[0].y <= cd.x314 {
            return Err(format!("player {}: spot dodge out of GuardOff not ported", p.slot + 1));
        }
        ftco_800cb024(p, rand)?;
        return Ok(());
    }
    if guard_release(p, rand)? {
        return Ok(());
    }
    if m == FTCO_MS_GUARD_ON {
        // ftCo_80093694: pressing the shoulder again inside the power shield window.
        let window = melee_fighter::hooks::common_data().powershield_input_window;
        let fp = &*p.fp;
        if p.guard.x0 < window as f32 && fp.input.pressed_buttons & (input::HSD_PAD_R | input::HSD_PAD_L) != 0 && (fp.active_timer.trigger as i32) < window {
            return Err(format!("player {}: ftCo_80093850 (power shield from GuardOn) not ported", p.slot + 1));
        }
    }
    guard_inputs(p, rand, m != FTCO_MS_GUARD).map(|_| ())
}

/// The guard states' Anim callbacks (GuardOn / Guard / GuardOff / GuardReflect).
fn guard_anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    match p.fp.motion_id {
        FTCO_MS_GUARD_ON => guard_on_anim(p, rand),
        FTCO_MS_GUARD_REFLECT => {
            ftco_80093bc0(p);
            guard_on_anim(p, rand)
        }
        FTCO_MS_GUARD => {
            p.guard.x0 += 1.0;
            ftco_800925a4(p).map(|_| ())
        }
        FTCO_MS_GUARD_OFF => {
            p.guard.x0 += 1.0;
            if !p.skel.frames_remaining() {
                ft_8008a2bc(p, rand)?;
            }
            Ok(())
        }
        m => Err(format!("player {}: guard motion {m} not ported", p.slot + 1)),
    }
}

/// `ftCo_8009A410`: the teeter at a floor's end.
fn ottotto_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let row = load_row(p, FTCO_SM_OTTOTTO)?;
    change_motion_state(p, FTCO_MS_OTTOTTO, row.as_ref(), rand)?;
    let fp = &mut *p.fp;
    fp.self_vel.x = 0.0;
    fp.self_vel.y = 0.0;
    fp.self_vel.z = 0.0;
    fp.gr_vel = 0.0;
    Ok(())
}

/// `ftCo_Ottotto_IASA` / `ftCo_OttottoWait_IASA`: the Wait checks, then the teeter-walk check.
fn ottotto_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if special_checks(p, rand)? {
        return Ok(());
    }
    if catch_check(p)? {
        return catch_enter(p, rand);
    }
    if attack_checks(p, rand)? {
        return Ok(());
    }
    if ground_stick_gate(p, rand, "ftCo_Ottotto_IASA")? {
        return Ok(());
    }
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    if fp.input.lstick[0].x * fp.facing_dir <= cd.x34 {
        return turn_enter_basic(p, rand);
    }
    // ftCo_Walk_CheckInput_Ottotto.
    if fp.input.lstick[0].x * fp.facing_dir >= cd.teeter_walk_threshold && fp.input.lstick[0].x * fp.facing_dir >= cd.walk_stick_threshold {
        return walk_enter(p, 0.0, rand);
    }
    Ok(())
}

/// `ftCo_Walk_IASA`.
fn walk_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if catch_check(p)? {
        return catch_enter(p, rand);
    }
    if special_checks(p, rand)? || attack_checks(p, rand)? {
        return Ok(());
    }
    if ground_stick_gate(p, rand, "ftCo_Walk_IASA")? {
        return Ok(());
    }
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    // ft_8008A244: stick back or under the walk threshold -> Wait.
    let x = fp.input.lstick[0].x;
    if x * fp.facing_dir < 0.0 || fc::abs_c(x) < cd.walk_stick_threshold {
        return ft_8008a2bc(p, rand);
    }
    // ftWalkCommon_800DFEC8(gobj, ftCo_Walk_Enter): a walk-speed band change restarts the walk
    // row at the matching phase.
    let ty = walk_type(p);
    if p.walk.msid + ty != p.fp.motion_id {
        let frame = match ty {
            0 => p.walk.slow_anim_frame,
            1 => p.walk.middle_anim_frame,
            _ => p.walk.fast_anim_frame,
        };
        // ftAnim_8006F484: the end frame (lbGetJObjEndFrame of the animating skeleton).
        let end = p.skel.end_frame();
        let init = p.fp.cur_anim_frame;
        let quotient = melee_runtime::cabi::f2i32((init / end) as f64);
        let adjusted = melee_core::fp::fmadds(-end, quotient as f32, p.fp.cur_anim_frame);
        let final_frame = melee_runtime::cabi::f2i32((frame * (adjusted / end)) as f64);
        return walk_enter(p, final_frame as f32, rand);
    }
    Ok(())
}

/// `ftWalkCommon_GetWalkType_800DFBF8_fake`.
fn walk_type(p: &Player) -> i32 {
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    let v = fc::abs_c(fp.gr_vel);
    let m = p.walk.accel_mul;
    if v >= m * (cd.walk_fast_stick_threshold * fp.co_attrs.walk_max_vel) {
        2
    } else if v >= m * (cd.walk_middle_animation_stick_threshold * fp.co_attrs.walk_max_vel) {
        1
    } else {
        0
    }
}

/// `ftCo_Walk_Enter(gobj, frame)` -> `ftWalkCommon_800DFCA4(gobj, WalkSlow, 0, frame,
/// x2DC, x2E0, x2E4, slow_walk_max, mid_walk_point, fast_walk_min, accel_mul)`.
fn walk_enter(p: &mut Player, frame: f32, rand: &mut HsdRand) -> Result<(), String> {
    let fp = &*p.fp;
    if fp.is_metal() != 0 || fp.x34_scale.y != 1.0 || !fp.x197C.is_null() {
        return Err("ftCo_Walk_Enter accel_mul (metal / scale / x197C) not wired".into());
    }
    let accel_mul = 1.0f32;
    p.walk.accel_mul = accel_mul;
    let ty = walk_type(p);
    let msid = FTCO_MS_WALK_SLOW + ty;
    let row = p.row_walk[ty as usize].clone();
    change_motion_state_at(p, msid, row.as_ref(), frame, rand)?;
    anim_step(p, rand)?;
    let fp = &*p.fp;
    p.walk.x0 = fp.gr_vel;
    p.walk.msid = FTCO_MS_WALK_SLOW;
    p.walk.slow_anim_frame = fp.x2DC;
    p.walk.middle_anim_frame = fp.x2E0;
    p.walk.fast_anim_frame = fp.x2E4;
    p.walk.slow_anim_rate = fp.co_attrs.slow_walk_max;
    p.walk.middle_anim_rate = fp.co_attrs.mid_walk_point;
    p.walk.fast_anim_rate = fp.co_attrs.fast_walk_min;
    Ok(())
}

/// `ftWalkCommon_800E0060` (no fused site).
fn walk_phys(p: &mut Player) {
    let cd = melee_fighter::hooks::common_data();
    let accel_mul = p.walk.accel_mul;
    let fp = &mut *p.fp;
    let x = fp.input.lstick[0].x;
    let mut accel = x * fp.co_attrs.walk_accel_mul * accel_mul;
    // getWalkAccel.
    accel += if x > 0.0 { accel_mul * fp.co_attrs.walk_accel_base } else { accel_mul * -fp.co_attrs.walk_accel_base };
    let target_vel = x * fp.co_attrs.walk_max_vel * accel_mul;
    if target_vel != 0.0 {
        let mult = fp.gr_vel / target_vel;
        if mult > 0.0 && mult < 1.0 {
            accel *= (1.0 - mult) * cd.walk_accel_taper_gain;
        }
    }
    p.walk.x0 = target_vel * cd.x440;
    let friction = fp.co_attrs.ground_friction;
    fc::calc_ground_accel_dash_run(fp, accel, target_vel, friction);
    fc::set_self_movement_from_grounded_movement(fp);
}

/// `ftAnim_SetAnimRate` (x2223_b0 clear) -> `ftAnim_8006F0FC`: every AObj of both skeletons
/// and `frame_speed_mul`.
fn set_anim_rate(p: &mut Player, rate: f32) {
    p.skel.set_rate(rate);
    p.fp.frame_speed_mul = rate;
}

/// `ft_8008A2BC` -> `ft_8008A348(gobj, 0)`: Wait (no item, grounded, not Peach / Link).
fn ft_8008a2bc(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.fp.ground_or_air == melee_types::GA_Air {
        return Err("ft_8008A348 from the air (ftCommon_8007D7FC) not wired".into());
    }
    let row = p.row_wait.clone();
    change_motion_state(p, FTCO_MS_WAIT, row.as_ref(), rand)?;
    // ftCo_8008A698 (an item) is false; ftCommon_8007EFC0(fp, x5F0).
    let v = melee_fighter::hooks::common_data().x5F0;
    melee_fighter::ftcommon2::ftcommon_8007efc0(&mut p.fp, v);
    Ok(())
}

/// `ftAction_80072CD8`: the footstep (`ft_80084BFC`, the floor material's step sound and
/// effect), then the script's own sound command.
fn ftaction_80072cd8(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let w0 = be32(&p.pdat, p.script);
    let mut sp60 = 1;
    let fp = &*p.fp;
    if fp.ground_or_air == melee_types::GA_Ground && fp.coll_data.floor.index != -1 {
        let m = (fp.coll_data.floor.flags as u8) as usize;
        let (sfx, play, gfx) = FD_STEP[m];
        let (sp64, gfx_id) = if sfx != -1 { (sfx, gfx) } else { (-1, gfx) };
        if sfx == -1 {
            sp60 = 1;
        } else {
            sp60 = play;
        }
        if sp64 != -1 {
            // ftAction_80071B50 on a copy of the command with the material's sfx id.
            sound_command(p, w0, sp64, rand)?;
        }
        if gfx_id != -1 {
            return Err(format!("footstep gfx {gfx_id:#x} not wired"));
        }
    }
    if sp60 != 0 {
        // ftAction_80071B50(gobj, cmd): the script's own sound command (3 words).
        let sfx = be32(&p.pdat, p.script + 4) as i32;
        sound_command(p, w0, sfx, rand)?;
    }
    p.script += 12;
    Ok(())
}

/// `ftAction_80071B50`: a sound command (`behavior` = bits 18..25 of the first word). Only `ft_PlaySFX`
/// (behavior 0) draws RNG: `HSD_Randi(200)` for a (remapped, `ft_80087D0C`) id in 332..=370. The
/// audio itself is not simulated.
fn sound_command(p: &Player, w0: u32, sfx: i32, rand: &mut HsdRand) -> Result<(), String> {
    let behavior = (w0 >> 18) & 0xFF;
    if behavior == 0 {
        let id = melee_fighter::ft0877::ft_80087d0c(&p.fp, sfx);
        if (332..=370).contains(&id) {
            let _ = rand.randi(200);
        }
    } else if behavior >= 10 {
        return Err("ftAction_80071B50 stop-sound behaviours not wired".into());
    }
    Ok(())
}

/// `ftCo_Turn_Enter_Basic` -> `ftCo_Turn_Enter(gobj, Turn, 0, 0, standing_turn_frames, 0)`:
/// the turn vars, Fighter_ChangeMotionState, then one `ftAnim_8006EBA4` step right away.
fn turn_enter_basic(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let frames = p.fp.co_attrs.standing_turn_frames;
    p.turn = TurnVars { has_turned: false, just_turned: false, facing_after: -p.fp.facing_dir, frames_to_turn: frames, x8: 0.0, x1c: 0 };
    let row = p.row_turn.clone();
    change_motion_state(p, FTCO_MS_TURN, row.as_ref(), rand)?;
    anim_step(p, rand)
}

/// The main stick's direction bits in `HSD_PadStatus.button` (0x10000..0x80000): no grounded
/// check reads them.
const PAD_STICK_DIRS: u32 = 0xF_0000;

/// [`VsMatch::hit_guard`]'s margin around a fighter (a fighter's hurtboxes reach about 20
/// units from its position).
const HIT_GUARD_MARGIN: f32 = 30.0;

/// `ftCo_MS_Catch` (212).
const FTCO_MS_CATCH: i32 = 212;

/// `ftCo_MS_Attack11` (44); Attack12 / Attack13 follow.
const FTCO_MS_ATTACK11: i32 = 44;

/// `ftCa_MS_SpecialLw` (357), `SpecialLwEnd` (358) and their rows (`ftCa_SM_SpecialLw` 311, 312).
const FTCA_MS_SPECIAL_LW: i32 = 357;
const FTCA_MS_SPECIAL_LW_END: i32 = 358;
/// `ftCa_MS_SpecialLwEndAir` (362) and its row (`ftCa_SM_SpecialLwEndAir` 315).
const FTCA_MS_SPECIAL_LW_END_AIR: i32 = 362;
const FTCA_SM_SPECIAL_LW_END_AIR: u32 = 315;
const FTCA_SM_SPECIAL_LW: u32 = 311;
const FTCA_SM_SPECIAL_LW_END: u32 = 312;

/// `ftData->xC[smid]` with its figatree (cached).
fn load_row(p: &mut Player, smid: u32) -> Result<Option<(AnimRow, FigaTree)>, String> {
    if let Some(r) = p.rows.get(&smid) {
        return Ok(r.clone());
    }
    let r = p.files.anim_row(smid)?.zip(p.files.figatree(smid)?);
    p.rows.insert(smid, r.clone());
    Ok(r)
}

/// Marth's Counter (`ftMs_MS_SpecialLw` 369 .. `SpecialAirLwHit` 372) and its rows (`ftMs_SM_*` 323..326).
const FTMS_MS_SPECIAL_LW: i32 = 369;
const FTMS_MS_SPECIAL_LW_HIT: i32 = 370;
const FTMS_MS_SPECIAL_AIR_LW: i32 = 371;
const FTMS_MS_SPECIAL_AIR_LW_HIT: i32 = 372;
const FTMS_SM_SPECIAL_LW: u32 = 323;

/// `ftMs_SpecialLw_Enter`: `self_vel.y = 0`, SpecialLw, `ftAnim_8006EBA4`, `cmd_vars[1] = 0`
/// (`mv.ms.speciallw.x0 = 0` only feeds the counter-hit damage).
fn ftms_speciallw_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    p.fp.self_vel.y = 0.0;
    let row = load_row(p, FTMS_SM_SPECIAL_LW)?;
    change_motion_state(p, FTMS_MS_SPECIAL_LW, row.as_ref(), rand)?;
    anim_step(p, rand)?;
    p.fp.cmd_vars[1] = 0;
    Ok(())
}

/// `ftMs_SpecialLw_Anim`: the script's `cmd_vars[1]` opens the counter window (`ftColl_8007B1B8`:
/// the shield-hit capsule, which only matters once a hit can land: `hit_guard` stops first), and
/// the end of the animation returns to Wait.
fn ftms_speciallw_anim(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let fp = &mut *p.fp;
    if fp.cmd_vars[1] == 1 {
        fp.cmd_vars[1] = 2;
        // (x221B_b0 / b1, shield_unk0 / 1: the counter capsule's flags, read only by hit detection.)
    }
    if !p.skel.frames_remaining() {
        ft_8008a2bc(p, rand)?;
    }
    Ok(())
}

/// `ftParts_GetBoneIndex(fp, part)`.
fn bone_index(p: &Player, part: u32) -> usize {
    p.part_to_joint[part as usize] as usize
}

/// `ftCa_SpecialLw_Enter` (Falcon Kick on the ground). `deal_dmg_cb = ftCa_SpecialHi_800E400C`
/// and Fighter_SetEffectHitlagCallbacks are not modelled (no hit reaches them here).
fn ftca_speciallw_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let fp = &mut *p.fp;
    fp.cmd_vars[2] = 0;
    fp.cmd_vars[1] = 0;
    fp.cmd_vars[0] = 0;
    fp.anon_2210.throw_flags = 0;
    p.ca_lw_x0 = 0.0;
    p.ca_lw_friction = 1.0;
    let row = load_row(p, FTCA_SM_SPECIAL_LW)?;
    change_motion_state(p, FTCA_MS_SPECIAL_LW, row.as_ref(), rand)?;
    anim_step(p, rand)
}

/// `ftCommon_8007DB24`: `x2219_b0 = 0`, `efLib_DestroyAll(gobj)` (deferred, `ef_ops`).
fn ftcommon_8007db24(p: &mut Player) {
    p.fp.set_x2219_b0(0);
    p.ef_ops.push(EfOp::DestroyAll);
}

/// `ftCommon_8007E5AC`: the root's Z rotation to the floor angle.
fn ftcommon_8007e5ac(p: &mut Player) {
    let n = p.fp.coll_data.floor.normal;
    let t = -melee_runtime::lbtrigf::atan2f(n.x, n.y);
    p.skel.set_part_rot(0, 2, t);
}

/// `ft_80085088`: the TransN velocity on the ground (x594_b0), else ground friction; then
/// `ftCommon_SetSelfMovementFromGroundedMovement`.
fn ft_80085088(fp: &mut Fighter) {
    if x594_b0(fp) {
        fp.gr_vel = fp.x6A4_transNOffset.z * fp.facing_dir;
    } else {
        let f = fp.co_attrs.ground_friction;
        fc::calc_ground_accel_deaccel(fp, f);
    }
    fc::set_self_movement_from_grounded_movement(fp);
}

/// `ftCa_SpecialHi_800E3EAC`: the script's throw flag b1 spawns the flame on the foot
/// (`efAsync_Spawn(gobj, &x60C, 3, 0x490, RFootJA (SpecialLw) / LFootJA (SpecialAirLw), &angle)`).
fn ftca_800e3eac(p: &mut Player) -> Result<(), String> {
    let fp = &mut *p.fp;
    if unsafe { fp.anon_2210.anon_0.throw_flags_b1() } == 0 {
        return Ok(());
    }
    unsafe { fp.anon_2210.anon_0.set_throw_flags_b1(0) };
    if fp.x2219_b0() != 0 {
        ftcommon_8007db24(p);
        return Ok(());
    }
    let (angle, part) = match fp.motion_id {
        0x165 => (0.0f32, melee_types::FtPart_RFootJA),
        0x167 => return Err("Falcon Kick flame in the air (LFootJA, flame angle) not wired".into()),
        _ => return Err("ftCa_SpecialHi_800E3EAC: sp1C / part uninitialised (C UB) for this motion".into()),
    };
    fp.set_x2219_b0(1);
    let part = bone_index(p, part);
    p.ef_queue.push(QueuedEffect { kind: EF_SPAWN_ATTACH_PARAM, gfx: 0x490, part, params: Vec3 { x: angle, y: 0.0, z: 0.0 }, extra1: 0.0, extra2: 0.0 });
    Ok(())
}

/// `fp->x594_b0` (the row's TransN root-motion flag).
fn x594_b0(fp: &Fighter) -> bool {
    unsafe { fp.anon_594.anon_0.x594_b0() != 0 }
}

/// `ftAnim_8006E9B4`'s skeleton step; with `x594_b0` the TransN part of `ftAnim_8006E054`:
/// `x698 = x68C`, `x68C = translation * scale` (`ftCommon_GetModelScale`, or `model_scaling`
/// with `x594_b6`), `x6B0 = x6A4`, `x6A4 = x68C - x698` (lbVector_Diff).
fn body_anim(p: &mut Player) -> Result<(), String> {
    let b0 = x594_b0(&p.fp);
    if b0 && unsafe { p.fp.anon_594.anon_0.x594_b5() } != 0 {
        return Err("x594_b5 (part 0x35 root motion) not ported".into());
    }
    if let Some(t) = p.skel.anim(&mut AObjGlobals::default(), b0)? {
        let fp = &mut *p.fp;
        let s = if unsafe { fp.anon_594.anon_0.x594_b6() } == 0 { fp.x34_scale.y * fp.co_attrs.model_scaling } else { fp.co_attrs.model_scaling };
        fp.x698 = fp.x68C_transNPos;
        fp.x68C_transNPos = melee_types::Vec { x: t.x * s, y: t.y * s, z: t.z * s };
        fp.x6B0 = fp.x6A4_transNOffset;
        let (a, b) = (fp.x68C_transNPos, fp.x698);
        fp.x6A4_transNOffset = melee_types::Vec { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z };
    }
    Ok(())
}

/// `ftAnim_8006EBA4`: the body animation (ftAnim_8006E9B4, no blend), the command script, the
/// hand poses (ftAnim_800707B0) and ftCo_800DB500 (no item).
fn anim_step(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.animating {
        body_anim(p)?;
        p.fp.cur_anim_frame = p.skel.curr_frame().ok_or("ftAnim_8006F3DC: no animated part")?;
    }
    ftaction_80073240(p, rand)?;
    p.skel.part_anim_update(&p.part_anim)
}

/// `ftCo_Turn_IASA` with no button, C-stick or trigger input (each attack / special / catch /
/// guard check needs one): the facing flip around the checks, the tap jump, fn_800C9C2C (the
/// smash-turn dash), the press latch.
fn turn_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    if p.turn.just_turned {
        p.fp.input.pressed_buttons |= p.turn.x1c;
    }
    if !p.turn.has_turned {
        p.fp.facing_dir = -p.fp.facing_dir;
    }
    let i = &p.fp.input;
    if i.pressed_buttons & !PAD_STICK_DIRS != 0 || i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 || i.triggers[0] != 0.0 || i.held_buttons[0] & (input::HSD_PAD_A | input::HSD_PAD_B | input::HSD_PAD_Z) != 0 {
        return Err(format!("player {}: ftCo_Turn_IASA with a button / C-stick / trigger: not ported", p.slot + 1));
    }
    if !p.turn.has_turned {
        p.fp.facing_dir = -p.fp.facing_dir;
    }
    if jump_get_input(&p.fp) {
        jump_check_input(p, rand)?;
        return Ok(());
    }
    // fn_800C9C2C.
    let fp = &*p.fp;
    if fp.input.lstick[0].x * p.turn.facing_after >= cd.dash_smash_stick_threshold && (fp.active_timer.lstick.x as i32) < cd.dash_smash_window as i32 {
        p.turn.x8 = p.turn.facing_after;
    }
    if p.turn.just_turned && p.turn.x8 != 0.0 && p.fp.input.lstick[0].x * p.turn.facing_after >= cd.dash_smash_stick_threshold {
        return Err(format!("player {}: ftCo_Dash_Enter (turn into a dash) not ported", p.slot + 1));
    }
    if p.fp.input.pressed_buttons & input::HSD_PAD_A != 0 {
        p.turn.x1c |= input::HSD_PAD_A;
    }
    if p.fp.input.pressed_buttons & input::HSD_PAD_B != 0 {
        p.turn.x1c |= input::HSD_PAD_B;
    }
    if p.turn.just_turned {
        p.turn.just_turned = false;
    }
    Ok(())
}

/// `ftAction_80073240`: run the command script up to the current frame (lbcommand.c's
/// Command_00..08 and the ftAction_803C06E8 commands the wired scripts use).
fn ftaction_80073240(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    ftaction_run(p, rand, false)
}

/// `ftAction_80073240` (`skip` false: the ftAction_803C06E8 table) or `ftAction_80073354`
/// (`skip` true: the ftAction_803C07AC table, whose entries mostly just step over a command;
/// its throw_flags resets are not modelled: no throws here).
fn ftaction_run(p: &mut Player, rand: &mut HsdRand, skip: bool) -> Result<(), String> {
    const F32_MAX: f32 = 3.4028235e38;
    let fp = &mut *p.fp;
    fp.x3E4_fighterCmdScript.frame_count = fp.cur_anim_frame + fp.x898_unk;
    if p.script == 0 {
        return Ok(());
    }
    if fp.x3E4_fighterCmdScript.timer != F32_MAX {
        fp.x3E4_fighterCmdScript.timer -= fp.frame_speed_mul;
    }
    loop {
        if p.script == 0 {
            break;
        }
        let c = &mut p.fp.x3E4_fighterCmdScript;
        if c.timer == F32_MAX {
            if c.frame_count >= p.fp.frame_speed_mul {
                break;
            }
            c.timer = -c.frame_count;
        } else if c.timer > 0.0 {
            break;
        }
        let w = be32(&p.pdat, p.script);
        let op = w >> 26;
        let value = w & 0x3FF_FFFF;
        match op {
            // Command_00: end.
            0x00 => p.script = 0,
            // Command_01: SynchronousTimer.
            0x01 => {
                c.timer += value as f32;
                p.script += 4;
            }
            // Command_02: AsynchronousTimer.
            0x02 => {
                c.timer = value as f32 - c.frame_count;
                p.script += 4;
            }
            // Command_03: SetLoop (return address, count).
            0x03 => {
                p.script_ret[p.script_sp] = p.script + 4;
                p.script_ret[p.script_sp + 1] = value;
                p.script_sp += 2;
                p.script += 4;
            }
            // Command_04: ExecuteLoop.
            0x04 => {
                let n = &mut p.script_ret[p.script_sp - 1];
                *n = n.wrapping_sub(1);
                if *n as i32 != 0 {
                    p.script = p.script_ret[p.script_sp - 2];
                } else {
                    p.script += 4;
                    p.script_sp -= 2;
                }
            }
            // Command_05: Subroutine (the pointer word follows).
            0x05 => {
                p.script += 4;
                p.script_ret[p.script_sp] = p.script + 4;
                p.script_sp += 1;
                p.script = be32(&p.pdat, p.script);
            }
            // Command_06: Return.
            0x06 => {
                p.script_sp -= 1;
                p.script = p.script_ret[p.script_sp];
            }
            // Command_07: Goto.
            0x07 => {
                p.script += 4;
                p.script = be32(&p.pdat, p.script);
            }
            // Command_08: SetTimerAnimation.
            0x08 => {
                p.script += 4;
                c.timer = F32_MAX;
            }
            // ftAction_800727C8: ftAnim_ApplyPartAnim(gobj, unk1, unk2, (f32) unk3);
            // ftAction_8007283C (skip table): the same with blend 0.
            0x29 => {
                let slot = ((w >> 19) & 0x7F) as usize;
                let anim = (((w >> 12) & 0x7F) as i32) << 25 >> 25; // s32 unk2 : 7
                let blend = if skip { 0.0 } else { (w & 0xFFF) as f32 };
                let pd = p.part_anim[slot].clone().ok_or("ftData->x1C slot missing")?;
                let pdat = p.pdat.clone();
                p.skel.apply_part_anim(&pdat, &pd, slot, anim, blend)?;
                p.script += 4;
            }
            // ftAction_80072A5C: ftCo_800BFFD0 (start a colour-overlay animation, fp->x408 / x488 /
            // x508: render only); ftAction_80072AAC (skip table) just steps. One word.
            0x2E => p.script += 4,
            // ftAction_80072B3C: the afterimage (sword trail) setup, x2100 / x2101 (ftafterimage.c:
            // render only); ftAction_80072B84 (skip table) just steps. One word.
            0x31 => p.script += 4,
            // ftAction_800726F4: TObj (eye texture) animation. Render only.
            0x28 => p.script += 4,
            // ftAction_800728F8: rumble (ftCommon_8007EC30 / 8007EBAC). No sim effect.
            0x2B => p.script += 4,
            // ftAction_80072C6C: ft_8008A1B8(gobj, value): x221C_u16_y = flags; without bit 2
            // the root's rotation x is reset.
            0x34 => {
                p.fp.set_x221C_u16_y(value as _);
                if value & 4 == 0 {
                    p.skel.set_part_rot(0, 0, 0.0);
                }
                p.script += 4;
            }
            // ftAction_80072E4C: the landing dust (+ the floor material's landing sound);
            // ftAction_80072FE0 (skip table): three words.
            0x37 if skip => p.script += 12,
            0x37 => ftaction_80072e4c(p, rand)?,
            // ftAction_80072CD8: the footstep (floor material sound / effect); ftAction_80072E24
            // (skip table): three words.
            0x36 if skip => p.script += 12,
            0x36 => ftaction_80072cd8(p, rand)?,
            // ftAction_80071028: spawn an effect (5 words; skipped when invisible);
            // ftAction_800711DC (skip table).
            0x0A if skip => p.script += 20,
            0x0A => ftaction_80071028(p, rand)?,
            // ftAction_8007121C: create a hitbox (5 words); ftAction_800715EC (skip table).
            0x0B if skip => p.script += 20,
            0x0B => ftaction_8007121c(p)?,
            // ftAction_8007162C (damage) / 80071708 (flags): one word each, not modelled.
            0x0C | 0x0E => p.script += 4,
            // ftAction_8007169C: hitbox size.
            0x0D if skip => p.script += 4,
            0x0D => {
                let id = ((w >> 23) & 7) as usize;
                if let Some(h) = p.hitboxes.get_mut(id).and_then(|h| h.as_mut()) {
                    h.scale = 0.003906f32 * (w & 0x7F_FFFF) as f32;
                }
                p.script += 4;
            }
            // ftAction_80071784: ftColl_8007AFC8 (remove hitbox `id`); ftAction_800717D8:
            // ftColl_8007AFF8 (remove all). The skip table steps over them.
            0x0F | 0x10 if skip => p.script += 4,
            0x0F => {
                let id = (w & 0x3FF_FFFF) as usize;
                if let Some(h) = p.hitboxes.get_mut(id) {
                    *h = None;
                }
                p.script += 4;
            }
            0x10 => {
                p.hitboxes = [None; 4];
                p.fp.set_x2219_b3(0);
                p.script += 4;
            }
            // ftAction_80071820: cmd_vars[idx] = value (both tables).
            0x13 => {
                let idx = ((w >> 24) & 3) as usize;
                p.fp.cmd_vars[idx] = w & 0xFF_FFFF;
                p.script += 4;
            }
            // ftAction_800718A4: throw flag b3 (+ cmd_timer = the script timer) or b4.
            0x14 => {
                match value {
                    0 => {
                        unsafe { p.fp.anon_2210.anon_0.set_throw_flags_b3(1) };
                        p.fp.cmd_timer = p.fp.x3E4_fighterCmdScript.timer;
                    }
                    1 => unsafe { p.fp.anon_2210.anon_0.set_throw_flags_b4(1) },
                    _ => {}
                }
                p.script += 4;
            }
            // ftAction_80071908 / 8007192C / 80071974: throw flags b1 / b2 / b0;
            // ftAction_80071950: allow_interrupt.
            0x15 => {
                unsafe { p.fp.anon_2210.anon_0.set_throw_flags_b1(1) };
                p.script += 4;
            }
            0x16 => {
                unsafe { p.fp.anon_2210.anon_0.set_throw_flags_b2(1) };
                p.script += 4;
            }
            0x17 => {
                p.fp.set_allow_interrupt(1);
                p.script += 4;
            }
            0x18 => {
                unsafe { p.fp.anon_2210.anon_0.set_throw_flags_b0(1) };
                p.script += 4;
            }
            // ftAction_80071998: the airborne state.
            0x19 => {
                match value {
                    1 => melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp),
                    2 => melee_fighter::ftcommon2::ftcommon_8007d60c(&mut p.fp),
                    0 => return Err("script ftCommon_8007D7FC (land) not wired".into()),
                    _ => {}
                }
                p.script += 4;
            }
            // ftAction_80071A14 / 80071A58 / 80071A9C: body / hurt-capsule states (the
            // hurtboxes; read only by hit detection, which is not ported). One word each.
            0x1A | 0x1B | 0x1C => p.script += 4,
            // ftAction_80071AE8: jab combo allowed (x2218_b1); ftAction_80071B28: x2218_b2.
            0x1D => {
                if value == 0 || !p.fp.x197C.is_null() {
                    p.fp.set_x2218_b1(1);
                }
                p.script += 4;
            }
            0x1E => {
                p.fp.set_x2218_b2((value & 1) as u8);
                p.script += 4;
            }
            // ftAction_80071E04: the throw's hit data (xDF4[idx], 3 words; read only by a throw,
            // which needs a grab, which needs hit detection); ftAction_80071F0C (skip table).
            0x22 => p.script += 12,
            // ftAction_80071D40 / 80071D94 / 80071DCC: model part / DObj visibility (render).
            0x1F | 0x20 | 0x21 => p.script += 4,
            // ftAction_80071F34: ftCommon_8007F5CC(gobj, unk27.value): the held item's visibility
            // (fp->item_gobj is NULL here), x221E_b3.
            0x23 => {
                p.fp.set_x221E_b3((value != 0) as u8);
                p.script += 4;
            }
            // ftAction_80071F78 / 80071FA0: x221E_b4 / x221E_b5.
            0x24 => {
                p.fp.set_x221E_b4((value & 1) as u8);
                p.script += 4;
            }
            0x25 => {
                p.fp.set_x221E_b5((value & 1) as u8);
                p.script += 4;
            }
            // ftAction_80071FC8: a random sound out of up to six (HSD_Randi(range)), 7 words;
            // ftAction_800722C8 (skip table).
            0x26 if skip => p.script += 28,
            0x26 => {
                let range = (w & 0x3F) as i32;
                let behavior = (w >> 6) & 0xF;
                let pick = rand.randi(range);
                let sfx = be32(&p.pdat, p.script + 4 + 4 * pick as u32) as i32;
                if behavior == 0 && (332..=515).contains(&sfx) {
                    return Err(format!("ft_PlaySFX {sfx}: ft_80087D0C's remap and the pitch draw not wired"));
                }
                p.script += 28;
            }
            // ftAction_80071B50: a sound command; ftAction_80071CA4 (skip table): three words.
            0x11 if skip => p.script += 12,
            0x11 => ftaction_80071b50(p)?,
            op => return Err(format!("command script opcode {op:#x} (ftAction_803C06E8[{}]) not ported", op as i32 - 10)),
        }
        if p.fp.x3E4_fighterCmdScript.timer == F32_MAX {
            break;
        }
    }
    Ok(())
}

/// `ftAction_80071028`: the effect command -> `ftCo_8009F834(gobj, gfx, bone, common, destroy,
/// offset, range, unk)` (offsets and ranges in 1/256 units, read in the C's swapped order).
fn ftaction_80071028(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.fp.invisible() != 0 {
        p.script += 20;
        return Ok(());
    }
    let w = [0u32, 4, 8, 12, 16].map(|o| be32(&p.pdat, p.script + o));
    let use_unk_bone = (w[0] >> 15) & 1 != 0;
    if use_unk_bone {
        return Err("ftAction_80071028 useUnkBone (ft_data->x8->x12) not wired".into());
    }
    let bone = (w[0] >> 18) & 0xFF;
    let common = (w[0] >> 17) & 1 != 0;
    let destroy = (w[0] >> 16) & 1 != 0;
    let gfx = w[1] >> 16;
    let k = 0.003906f32;
    let hi = |x: u32| ((x >> 16) as i16) as f32;
    let lo_s = |x: u32| ((x & 0xFFFF) as i16) as f32;
    let lo_u = |x: u32| (x & 0xFFFF) as f32;
    let hi_u = |x: u32| (x >> 16) as f32;
    let offset = Vec3 { x: k * hi(w[2]), y: k * lo_s(w[2]), z: k * hi(w[3]) };
    let range = Vec3 { x: k * lo_u(w[3]), y: k * hi_u(w[4]), z: k * lo_u(w[4]) };
    p.script += 20;
    if destroy {
        p.fp.set_x2219_b0(1);
    }
    let part = match bone {
        0x8D => return Err("ftCo_8009F834 part 0x8D (ft_data->x54 cycle) not wired".into()),
        0x8E => return Err("ftCo_8009F834 part 0x8E (ft_data->x8->x10) not wired".into()),
        b if common => bone_index(p, b),
        b => b as usize,
    };
    ftco_8009f834(p, rand, gfx, part, offset, range)
}

/// `ftAction_8007121C`: create (or refresh) hitbox `id` on a part (`ftParts_GetBoneIndex` for
/// common bone ids); `x2219_b3` set on a new one; `ftCommon_80080484`. Damage staling and the
/// rest of the HitCapsule are not modelled (read only by hit detection).
fn ftaction_8007121c(p: &mut Player) -> Result<(), String> {
    let w = [0u32, 4, 8, 12, 16].map(|o| be32(&p.pdat, p.script + o));
    // spawn_hitbox_skip.xF_b4: byte 0xF (word 3's low byte) bit 4 (MSB first) = 0x08.
    if w[3] & 0x08 != 0 && unsafe { p.fp.x1064_thrownHitbox.anon_134.owner.is_null() } {
        p.script += 20;
        return Ok(());
    }
    let id = ((w[0] >> 23) & 7) as usize;
    let hit_group = (w[0] >> 20) & 7;
    let bone = ((w[0] >> 11) & 0xFF) as usize;
    let common = (w[0] >> 10) & 1 != 0;
    if id >= 4 {
        return Err(format!("hitbox id {id} (PC slot remap) not wired"));
    }
    let part = if common { bone_index(p, bone as u32) } else { bone };
    let size = (w[1] >> 16) as f32;
    let z = ((w[1] & 0xFFFF) as i16) as f32;
    let y = ((w[2] >> 16) as i16) as f32;
    let x = ((w[2] & 0xFFFF) as i16) as f32;
    let k = 0.003906f32;
    let fresh = match &p.hitboxes[id] {
        None => true,
        Some(h) => h.hit_group != hit_group,
    };
    if fresh {
        p.fp.set_x2219_b3(1);
    }
    p.hitboxes[id] = Some(Hitbox { part, scale: k * size, offset: Vec3 { x: k * z, y: k * y, z: k * x }, hit_group });
    if p.fp.smash_attrs.x2138_smashSinceHitbox == -1.0 {
        p.fp.smash_attrs.x2138_smashSinceHitbox = 0.0;
    }
    p.script += 20;
    Ok(())
}

/// `ftAction_80071B50`: behaviours 0-6 play `sfx` (word 2) on a channel, 10-15 stop one; no
/// audio here. Only `ft_PlaySFX` (behaviour 0) draws from the RNG, `HSD_Randi(200)` for a
/// remapped id in 332..370 (`ft_80087D0C`, not ported: such ids stop the run).
fn ftaction_80071b50(p: &mut Player) -> Result<(), String> {
    let w0 = be32(&p.pdat, p.script);
    let behavior = (w0 >> 18) & 0xFF;
    match behavior {
        0..=6 => {
            let sfx = be32(&p.pdat, p.script + 4) as i32;
            if behavior == 0 && (332..=515).contains(&sfx) {
                return Err(format!("ft_PlaySFX {sfx}: ft_80087D0C's remap and the HSD_Randi(200) pitch draw not wired"));
            }
            p.script += 12;
        }
        10..=15 => p.script += 12,
        _ => p.script += 8,
    }
    Ok(())
}

/// Final Destination's floor-material step rows (mplib.c `mpLib_803BE7A8`: x4[0] (step sfx),
/// x14[0] (play-default flag), x14[1] (step gfx)) per material.
const FD_STEP: [(i32, i32, i32); 20] = [
    (-1, 0, -1), (0x161, 1, -1), (0x152, 1, -1), (0x155, 1, -1), (0x16D, 1, -1), (0x15B, 1, -1), (0x158, 1, -1), (0x16A, 1, -1), (0x15E, 0, -1), (-1, 0, -1),
    (0x14F, 1, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (0x14C, 1, -1), (0x164, 1, -1), (0x170, 0, -1), (0x15E, 1, -1), (0x167, 1, -1), (-1, 0, -1),
];

/// Final Destination's floor-material sound/effect rows (mplib.c `mpLib_803BE7A8`, fields
/// x20[0] (landing sfx), x30[0] (play-default flag) and x30[1] (landing gfx)) per material.
const FD_LANDING: [(i32, i32, i32); 20] = [
    (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (0x04B, 1, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1),
    (0x20E, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1), (-1, 0, -1),
];

/// `ftAction_80072E4C`: `ft_80084C38` (the floor material's landing sfx / gfx), the effect
/// (`ftCo_8009F834(gobj, gfx, TopN, 0, 0, 0, 0, 0)`), the default landing sound 0x46 (no RNG:
/// not in ft_PlaySFX's 332..370 pitch range), the material sound command, and the rumble.
fn ftaction_80072e4c(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    let w0 = be32(&p.pdat, p.script);
    let cmd_flag = (w0 >> 16) & 0xFF & 1;
    let mut sp60 = 1;
    let mut gfx_id = -1i32;
    let mut sp64 = -1i32;
    // ft_80084A80(1, ...): grounded with a floor -> the material row.
    let fp = &*p.fp;
    let found = if fp.ground_or_air == melee_types::GA_Ground && fp.coll_data.floor.index != -1 {
        let flags = fp.coll_data.floor.flags; // mpLineGetFlags == the line's lo_flags == floor.flags here
        let (sfx, play, gfx) = FD_LANDING[(flags as u8) as usize];
        sp60 = play;
        sp64 = sfx;
        if sfx == -1 {
            sp60 = 1;
        }
        gfx_id = gfx;
        true
    } else {
        false
    };
    if found && cmd_flag == 0 && sp64 != -1 {
        return Err(format!("landing material sfx {sp64:#x} (ftAction_80071B50) not wired"));
    }
    if gfx_id == -1 {
        gfx_id = (w0 & 0xFFFF) as i32;
    }
    ftco_8009f834(p, rand, gfx_id as u32, 0, Vec3 { x: 0.0, y: 0.0, z: 0.0 }, Vec3 { x: 0.0, y: 0.0, z: 0.0 })?;
    if sp60 != 0 {
        // ft_PlaySFX(fp, 0x46, 0x7F, 0x40): no audio here; 0x46 draws no pitch randomness.
        if cmd_flag == 0 {
            // ftAction_80071B50(gobj, cmd): the script's own sound command (3 words).
            let behavior = (w0 >> 18) & 0xFF;
            if behavior <= 6 {
                let sfx = be32(&p.pdat, p.script + 4) as i32;
                if (332..=370).contains(&sfx) {
                    return Err(format!("ft_PlaySFX {sfx}: the HSD_Randi(200) pitch draw needs ft_80087D0C's remap (not wired)"));
                }
            } else if behavior >= 10 {
                return Err("ftAction_80071B50 stop-sound behaviours not wired".into());
            }
            p.script += 12;
        }
    }
    if sp60 == 0 || cmd_flag != 0 {
        p.script += 12;
    }
    // ftCommon_8007EBAC(fp, 0x16, 0): rumble.
    Ok(())
}

/// `ftCo_8009F834(gobj, gfx_id, part, 0, 0, offset, range, 0)` for the effect ids the wired
/// scripts reach: three HSD_Randf jitter draws (`x += 2 * range * (r - 0.5)`, one fused site
/// each), then the per-id efAsync_Spawn.
fn ftco_8009f834(p: &mut Player, rand: &mut HsdRand, gfx_id: u32, part: usize, offset: Vec3, range: Vec3) -> Result<(), String> {
    if gfx_id < 0x250 || gfx_id / 1000 == 0x1E {
        return Err(format!("ftCo_8009F834 gfx {gfx_id:#x} (< 0x250 path) not wired"));
    }
    if matches!(gfx_id, 0x402 | 0x403 | 0x409 | 0x412 | 0x413 | 0x414 | 0x422 | 0x487 | 0x4D1 | 0x4E5 | 0x4E6 | 0x4FE | 0x500 | 0x501 | 0x502) {
        // efAsync_Spawn(gobj, &x60C, EF_SPAWN_ATTACH, gfx_id, joint).
        p.ef_queue.push(QueuedEffect { kind: EF_SPAWN_ATTACH, gfx: gfx_id, part, params: Vec3 { x: 0.0, y: 0.0, z: 0.0 }, extra1: 0.0, extra2: 0.0 });
        return Ok(());
    }
    // block_70: the jitter.
    let mut sp84 = offset;
    let r = rand.randf();
    sp84.x = melee_core::fp::fmadds(2.0 * range.x, r - 0.5, sp84.x);
    let r = rand.randf();
    sp84.y = melee_core::fp::fmadds(2.0 * range.y, r - 0.5, sp84.y);
    let r = rand.randf();
    let z_range = 2.0f32 * range.z;
    sp84.z = melee_core::fp::fmadds(z_range, r - 0.5, sp84.z);
    match gfx_id {
        0x404 | 0x406 => {
            // sp80 = the floor angle when grounded; EF_SPAWN_POS_OFFSET_PARAM(gfx, joint, &sp84, &sp80).
            let fp = &*p.fp;
            let sp80 = if fp.ground_or_air == melee_types::GA_Ground {
                let n = fp.coll_data.floor.normal;
                melee_runtime::lbtrigf::atan2f(-n.x, n.y)
            } else {
                0.0
            };
            p.ef_queue.push(QueuedEffect { kind: EF_SPAWN_POS_OFFSET_PARAM, gfx: gfx_id, part, params: sp84, extra1: sp80, extra2: 0.0 });
            Ok(())
        }
        0x3F7 | 0x3F8 | 0x3F9 | 0x3FD | 0x3FF => {
            // sp7C = the floor angle when grounded; EF_SPAWN_POS_OFFSET_PARAM2(gfx, joint, &sp84,
            // &fp->facing_dir, &sp7C).
            let fp = &*p.fp;
            let sp7c = if fp.ground_or_air == melee_types::GA_Ground {
                let n = fp.coll_data.floor.normal;
                melee_runtime::lbtrigf::atan2f(-n.x, n.y)
            } else {
                0.0
            };
            let extra1 = fp.facing_dir;
            p.ef_queue.push(QueuedEffect { kind: EF_SPAWN_POS_OFFSET_PARAM2, gfx: gfx_id, part, params: sp84, extra1, extra2: sp7c });
            Ok(())
        }
        0x3ED | 0x3EF | 0x3F0 | 0x3F1 | 0x3F2 | 0x3F5 | 0x3FE | 0x400 | 0x401 | 0x4D9 => {
            // EF_SPAWN_POS_OFFSET_PARAM(gfx, joint, &sp84, &fp->facing_dir).
            let extra1 = p.fp.facing_dir;
            p.ef_queue.push(QueuedEffect { kind: EF_SPAWN_POS_OFFSET_PARAM, gfx: gfx_id, part, params: sp84, extra1, extra2: 0.0 });
            Ok(())
        }
        0x407 => {
            // efAsync_Spawn(gobj, &x60C, EF_SPAWN_POS_OFFSET, gfx_id, joint, &sp84).
            p.ef_queue.push(QueuedEffect { kind: EF_SPAWN_POS_OFFSET, gfx: gfx_id, part, params: sp84, extra1: 0.0, extra2: 0.0 });
            Ok(())
        }
        // efAsync_Spawn(.., EF_SPAWN_CAMERA_SHAKE, 2 / 3 / 4, joint, &sp84): a camera quake at
        // s_link 9 (Camera_RequestQuake). The camera is not simulated; the C trace shows it
        // draws nothing from the RNG.
        0x513..=0x515 => Ok(()),
        g => Err(format!("ftCo_8009F834 gfx {g:#x} not wired")),
    }
}

/// `ftCo_800C6408` (EntryStart), not a sub-fighter. Not modelled: the trophy-stand accessory JObj
/// (`ftCommon_SetAccessory`, render only), `lbAudioAx_80024304(0x8B)` and the colour animation
/// `ftCo_800BFFD0(0x75)` (neither hashed nor drawing from the RNG).
fn entry_start(p: &mut Player, c: EntryCommon, rand: &mut HsdRand) -> Result<(), String> {
    p.entry.timer = c.x6bc;
    let row = p.row_entry_start.clone();
    change_motion_state(p, FTCO_MS_ENTRY_START, row.as_ref(), rand)?;
    p.fp.set_x2219_b1(1);
    // temp_f0 = x34_scale.y * co_attrs.trophy_scale (x34_scale.y = Player_GetModelScale = 1 in VS).
    let s = 1.0f32 * p.trophy_scale;
    p.entry.x24 = s;
    // temp_f0_2 = 1.497345 * sp48.y: a double product rounded to f32.
    let h = (1.497345f64 * s as f64) as f32;
    p.entry.x28 = h;
    p.entry.x20 = h;
    // efAsync_Spawn(gobj, &fp->x60C, EF_SPAWN_ATTACH_PARAM, 0x43E, gobj->hsd_obj, &entry.x8):
    // queued (the current proc's s_link is below 9).
    let x8 = p.entry.x8;
    p.ef_queue.push(QueuedEffect { kind: EF_SPAWN_ATTACH_PARAM, gfx: 0x43E, part: 0, params: Vec3 { x: x8.x, y: 0.0, z: 0.0 }, extra1: 0.0, extra2: 0.0 });
    Ok(())
}

/// `ftCo_800C6B6C` (EntryEnd), not a sub-fighter.
fn entry_end(p: &mut Player, c: EntryCommon, rand: &mut HsdRand) -> Result<(), String> {
    p.entry.timer = c.x6c0;
    // HSD_JObjSetScale(GET_JOBJ(gobj), &entry.x8): the scale from before the entry squash.
    let root = p.skel.root;
    p.skel.set_scale(root, p.entry.x8);
    p.fp.cur_pos.y = p.entry.x4 + p.entry.x20;
    // Fighter_ChangeMotionState(EntryEnd, 0x3000, 0, 1, 0): no animation row.
    change_motion_state(p, FTCO_MS_ENTRY_END, None, rand)?;
    p.fp.set_x2219_b1(1);
    Ok(())
}

/// What `Fighter_ChangeMotionState(gobj, msid, flags, 0.0, 1.0, 0.0, NULL)` does to the hashed
/// fields and the skeleton in the entry states: `motion_id`; `HSD_JObjSetTranslate(jobj,
/// cur_pos)`; `ftPartSetRotX/Y/Z(fp, 0, 0, M_PI_2 * facing_dir, 0)`; `frame_speed_mul = 1`,
/// `cur_anim_frame = 0 - 1`, `x898 = 0`; with an animation row the figatree is attached
/// (`ftAnim_8006EBE8(gobj, 0, 1, 0)`: every row these states use has blend byte 0, checked against
/// the C), `ftAnim_8006E9B4` steps it to frame 0 and `ftAction_80073240` sets `frame_count`;
/// with none (`anim_id == -1`) `ftAnim_80070758` drops the AObjs and frame_count stays.
/// TODO(group3 fighter.c): the whole Fighter_ChangeMotionState.
fn change_motion_state(p: &mut Player, msid: i32, row: Option<&(AnimRow, FigaTree)>, rand: &mut HsdRand) -> Result<(), String> {
    change_motion_state_at(p, msid, row, 0.0, rand)
}

/// `Fighter_ChangeMotionState(gobj, msid, 0, anim_start, anim_speed, 0.0, NULL)`.
fn change_motion_state_at(p: &mut Player, msid: i32, row: Option<&(AnimRow, FigaTree)>, anim_start: f32, rand: &mut HsdRand) -> Result<(), String> {
    change_motion_state_full(p, msid, row, anim_start, 1.0, rand)
}

/// `Fighter_ChangeMotionState(gobj, msid, 0, anim_start, 1.0, 0.0, NULL)` (see
/// [`change_motion_state`]); a nonzero `anim_start` takes the C's start-mid-animation path
/// (`ftAnim_8006EBE8(start - speed)`, two `ftAnim_8006E9B4` steps, `timer = -start`,
/// `ftAction_80073354`, the script run in its skip table).
fn change_motion_state_full(p: &mut Player, msid: i32, row: Option<&(AnimRow, FigaTree)>, anim_start: f32, anim_speed: f32, rand: &mut HsdRand) -> Result<(), String> {
    change_motion_state_ext(p, msid, row, anim_start, anim_speed, false, rand)
}

/// `Fighter_ChangeMotionState(gobj, msid, skip_anim ? Ft_MF_SkipAnim : 0, anim_start, anim_speed, 0.0, NULL)`.
/// With `Ft_MF_SkipAnim` the C leaves the animation block out and ends in the `anim_id = -1` path
/// (x594 = 0, the AObjs dropped, no script): the same as a state without a row. The guard states
/// pose the body themselves (`ftCo_80091E78`).
fn change_motion_state_ext(p: &mut Player, msid: i32, row: Option<&(AnimRow, FigaTree)>, anim_start: f32, anim_speed: f32, skip_anim: bool, rand: &mut HsdRand) -> Result<(), String> {
    change_motion_state_flags(p, msid, row, anim_start, anim_speed, if skip_anim { melee_types::Ft_MF_SkipAnim as u32 } else { 0 }, rand)
}

/// `Fighter_ChangeMotionState(gobj, msid, flags, anim_start, anim_speed, 0.0, NULL)`: `flags` is the caller's
/// `Ft_MF_*` OR the motion state's own `x4_flags` (the C tests both through one word).
fn change_motion_state_flags(p: &mut Player, msid: i32, row: Option<&(AnimRow, FigaTree)>, anim_start: f32, anim_speed: f32, flags: u32, rand: &mut HsdRand) -> Result<(), String> {
    let row = if flags & melee_types::Ft_MF_SkipAnim as u32 != 0 { None } else { row };
    // efAsync_QueueFlush(gobj, &fp->x60C) (newest first) and, with x2219_b0 and no
    // Ft_MF_KeepGfx, ftCommon_8007DB24 (efLib_DestroyAll): replayed at s_link 9 (`ef_ops`).
    for q in std::mem::take(&mut p.ef_queue).into_iter().rev() {
        p.ef_ops.push(EfOp::Spawn(q));
    }
    let keep_gfx = std::mem::take(&mut p.keep_gfx);
    if p.fp.x2219_b0() != 0 && !keep_gfx {
        ftcommon_8007db24(p);
    }
    if p.fp.x2219_b3() != 0 {
        // ftColl_8007AFF8: every hitbox off (Ft_MF_SkipHit is never passed here).
        p.hitboxes = [None; 4];
        p.fp.set_x2219_b3(0);
    }
    let fp = &mut p.fp;
    if !(14..=17).contains(&msid) {
        fp.hitlag_mul = 0.0;
    }
    fp.motion_id = msid;
    // ft_800890D0(fp, new_motion_state->move_id): a new attack instance unless the same attack goes on. Only
    // Fox's / Falco's neutral B (attack 18) is an attack here; every other state has move_id 1.
    melee_fighter::stale::new_attack(fp, if fox::is_fox_special_n(msid) { 18 } else { 1 });
    // The flags Fighter_ChangeMotionState clears (fighter.c ~1381): the entry / death states set
    // them again right after the change.
    fp.set_x221F_b3(0);
    fp.set_x2219_b1(0);
    fp.set_x2219_b2(0);
    fp.set_invisible(0);
    fp.set_x221E_b1(0);
    fp.set_x221E_b2(0);
    fp.set_x221F_b1(0);
    fp.set_x221A_b7(0);
    p.guard.shield_on = false;
    fp.set_x221C_b3(0);
    fp.set_reflecting(0);
    fp.set_x221C_b4(0);
    fp.set_x2218_b6(0);
    fp.set_x221D_b5(0);
    fp.x209C = 0;
    if fp.ground_or_air == melee_types::GA_Ground {
        fp.set_x2224_b1(0);
    }
    // The rest of the clears of Fighter_ChangeMotionState (fighter.c ~1380..1500).
    if flags & melee_types::Ft_MF_SkipThrowException as u32 == 0 {
        unsafe { fp.x1064_thrownHitbox.anon_134.owner = std::ptr::null_mut() };
    }
    if flags & melee_types::Ft_MF_SkipHitStun as u32 == 0 && fp.x221C_b6() != 0 {
        fp.set_x221C_b6(0);
        fp.x2098 = melee_fighter::hooks::common_data().x4CC as _;
    }
    fp.dmg.x182c_behavior = 1.0;
    fp.dmg.kb_applied = 0.0;
    fp.dmg.x18A8 = 0.0;
    fp.dmg.armor1 = 0.0;
    fp.dmg.x18a0 = 0.0;
    fp.anon_221b.set_x221B_b0(0);
    fp.shield_unk0 = 0.0;
    fp.shield_unk1 = 0.0;
    fp.x1A6A = 0;
    fp.set_x221D_b7(0);
    fp.set_x221E_b5(0);
    fp.set_x221E_b6(0);
    fp.set_x2220_b3(0);
    fp.set_x2220_b7(0);
    fp.set_x2223_b0(0);
    fp.set_x2222_b3(0);
    fp.set_x2224_b5(0);
    fp.set_x2225_b1(0);
    fp.set_x2225_b4(0);
    fp.set_x2224_b4(0);
    fp.set_x2222_b7(0);
    fp.smash_attrs.x2138_smashSinceHitbox = -1.0;
    fp.x100 = if flags & melee_types::Ft_MF_UnkUpdatePhys as u32 != 0 { 0.0 } else { -1.0 };
    if flags & melee_types::Ft_MF_KeepSwordTrail as u32 == 0 {
        fp.x2100 = -1;
    }
    if flags & melee_types::Ft_MF_SkipNametagVis as u32 == 0 {
        fp.x209A = 0;
    }
    fp.lstick_angle = 0.0;
    fp.hitlag_cb = None;
    fp.post_hitlag_cb = None;
    fp.pre_hitlag_cb = None;
    fp.take_dmg_cb = None;
    fp.take_dmg_2_cb = None;
    fp.deal_dmg_cb = None;
    fp.shield_hit_cb = None;
    fp.reflect_hit_cb = None;
    fp.hurtbox_detect_cb = None;
    let root = p.skel.root;
    p.skel.set_translate(root, melee_runtime::mtx::Vec3 { x: fp.cur_pos.x, y: fp.cur_pos.y, z: fp.cur_pos.z });
    // ftAnim_80070F28 (end the hand poses), ftAnim_80070E74 (restart the slots with a default
    // pose x10: none is set for the wired kinds).
    p.skel.clear_part_anims(&p.part_anim);
    if p.skel.x8b0.iter().any(|s| s.x10 != -1) {
        return Err("ftAnim_80070E74 with a default hand pose: not wired".into());
    }
    p.skel.set_part_rot(0, 0, 0.0);
    p.skel.set_part_rot(0, 1, (core::f64::consts::FRAC_PI_2 * fp.facing_dir as f64) as f32);
    p.skel.set_part_rot(0, 2, 0.0);
    fp.frame_speed_mul = anim_speed;
    fp.x8A0_unk = anim_speed;
    fp.cur_anim_frame = anim_start - fp.frame_speed_mul;
    fp.x898_unk = 0.0;
    // bone_index = x596_bits.x7, read before x594 changes (the previous row's flags).
    let bone_index = unsafe { fp.anon_594.anon_0.x596_bits.x7() };
    match row {
        Some((r, tree)) => {
            if r.flags & 0x2000_0000 != 0 {
                return Err(format!("anim row flags {:#x}: x594_b2 path not ported", r.flags));
            }
            if r.flags & 0x0400_0000 != 0 {
                return Err(format!("anim row flags {:#x}: x594_b5 path not ported", r.flags));
            }
            // fp->x594_s32 = row->x10_animCurrFlags (stored in GameCube byte order, which the
            // generated bitfield accessors read).
            fp.anon_594.x594_s32 = i32::from_ne_bytes(r.flags.to_be_bytes());
            // anim_blend 0.0 -> the row's default blend byte.
            let blend = if std::mem::take(&mut p.blend_zero) { 0.0 } else { r.blend as f32 };
            p.fp.anim_id = r.id as i32;

            // x3E4_fighterCmdScript.u = row->xC; loop_count = 0.
            p.script = r.script;
            p.script_sp = 0;
            if anim_start != 0.0 {
                p.skel.attach(tree, anim_start - anim_speed, anim_speed, r.looping(), blend);
                p.animating = true;
                body_anim(p)?;
                p.fp.cur_anim_frame = p.skel.curr_frame().ok_or("ftAnim_8006F3DC: no animated part")?;
                if x594_b0(&p.fp) {
                    let fp = &mut *p.fp;
                    fp.x6B0 = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
                    fp.x6A4_transNOffset = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
                    fp.x698 = fp.x68C_transNPos;
                }
                p.fp.x3E4_fighterCmdScript.timer = -anim_start;
            } else {
                p.skel.attach(tree, anim_start, anim_speed, r.looping(), blend);
                p.animating = true;
                p.fp.x3E4_fighterCmdScript.timer = 0.0;
            }
            body_anim(p)?;
            p.fp.cur_anim_frame = p.skel.curr_frame().ok_or("ftAnim_8006F3DC: no animated part")?;
            if bone_index != 0 && r.blend != 0 {
                // The previous row's x596_bits.x7 part: its anim-skeleton translation and
                // rotation onto the body (HSD_JObjSetTranslate, ftParts_JObjSetRotation).
                p.skel.copy_part_tr(bone_index as usize);
            }
            if x594_b0(&p.fp) {
                let fp = &mut *p.fp;
                if anim_start == 0.0 {
                    fp.x6B0 = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
                    fp.x6A4_transNOffset = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
                    fp.x698 = fp.x68C_transNPos;
                } else if fp.ground_or_air == melee_types::GA_Ground {
                    // (Ft_MF_SkipAnimVel clear in every change wired here.)
                    let v = fp.x6A4_transNOffset.z * fp.facing_dir;
                    fp.self_vel.x = v;
                    fp.gr_vel = v;
                }
            }
            if anim_start != 0.0 {
                // ftAction_80073354: catch the script up through the skip table.
                ftaction_run(p, rand, true)?;
            } else {
                // ftCo_800C0408 (colour-animation scripts: render only), then ftAction_80073240.
                ftaction_80073240(p, rand)?;
            }
        }
        None => {
            fp.anim_id = -1;
            p.blend_zero = false;
            // anim_id == -1: x594 = 0, x3E4.u = NULL, the AObjs dropped, x8A4 = x8A8 = 0.
            fp.anon_594.x594_s32 = 0;
            p.animating = false;
            p.script = 0;
            p.skel.remove_anim_all();
        }
    }
    Ok(())
}

/// `ftCommon_8007E0E4` (with 8007DD7C): grounded fighters on the same / adjacent floor line push
/// each other apart; the result is `xF8_playerNudgeVel`, applied by the integration.
fn player_nudge(players: &mut [Player], i: usize, g: &GroundColl) {
    let cd = melee_fighter::hooks::common_data();
    {
        let f = &mut players[i].fp;
        f.xF8_playerNudgeVel.y = 0.0;
        f.xF8_playerNudgeVel.x = 0.0;
    }
    let f = &players[i].fp;
    if !(f.x2219_b1() == 0 && f.x2219_b5() == 0 && f.ground_or_air == melee_types::GA_Ground) {
        return;
    }
    if f.is_sub_fighter() != 0 {
        return; // (Nana's 8007DFD0 branch: not wired)
    }
    let (phi_f30, phi_f31) = (cd.x454, cd.x458);
    // ftCommon_8007F8B4.
    let mut sp10 = Vec3 { x: f.cur_pos.x + f.xD4_unk_vel.x, y: f.cur_pos.y + f.xD4_unk_vel.y, z: f.cur_pos.z + f.xD4_unk_vel.z };
    let (a2c4, a_dir, arg_gnd) = (f.x2C4, f.facing_dir, f.coll_data.floor.index);
    let mut nx = 0.0f32;
    let mut ny = 0.0f32;
    if f.x221D_b5() == 0 {
        let mut phi_r28 = false;
        for (j, o) in players.iter().enumerate() {
            if j == i {
                phi_r28 = true;
                continue;
            }
            let c = &o.fp;
            if c.x221F_b3() != 0 || c.ground_or_air != melee_types::GA_Ground || !c.victim_gobj.is_null() || c.is_sub_fighter() != 0 {
                continue;
            }
            let cur_gnd = c.coll_data.floor.index;
            if !(arg_gnd == cur_gnd || cur_gnd == g.mp_line_get_next(arg_gnd) || cur_gnd == g.mp_line_get_prev(arg_gnd)) {
                continue;
            }
            let sp24 = Vec3 { x: c.cur_pos.x + c.xD4_unk_vel.x, y: c.cur_pos.y + c.xD4_unk_vel.y, z: c.cur_pos.z + c.xD4_unk_vel.z };
            let vt = c.x2C4;
            let f0 = melee_core::fp::fmadds(a2c4.x, a_dir, sp10.x) - melee_core::fp::fmadds(c.facing_dir, vt.x, sp24.x);
            if fc::abs_c(f0) < a2c4.y + vt.y {
                let sgn = |neg: bool, x: f32| if neg { -x } else { x };
                if f0 != 0.0 {
                    nx += sgn(f0 < 0.0, cd.x450);
                } else {
                    nx += sgn(phi_r28, cd.x450);
                }
                if sp10.z - sp24.z != 0.0 {
                    ny += sgn(sp10.z - sp24.z < 0.0, cd.x454);
                } else if f0 != 0.0 {
                    ny += sgn(f0 < 0.0, cd.x454);
                } else {
                    ny += sgn(phi_r28, cd.x454);
                }
            }
        }
    }
    if ny == 0.0 && sp10.z != 0.0 {
        ny = if sp10.z < 0.0 { phi_f30 } else { -phi_f30 };
    }
    if (ny > 0.0 && sp10.z < 0.0 && sp10.z + ny >= 0.0) || (ny < 0.0 && sp10.z > 0.0 && sp10.z + ny <= 0.0) {
        ny = -sp10.z;
        sp10.z = 0.0;
    }
    if sp10.z + ny > phi_f31 {
        ny = phi_f31 - sp10.z;
    } else if sp10.z + ny < -phi_f31 {
        ny = -phi_f31 - sp10.z;
    }
    let f = &mut players[i].fp;
    f.xF8_playerNudgeVel.x = nx;
    f.xF8_playerNudgeVel.y = ny;
}

/// `ft_80082B1C`: the Fall landing callback. Below `ftCo_800D0EC8` (the heavy-landing speed)
/// it is `ftCo_Landing_Enter_Basic`, above it `ft_8008A2BC` (Wait, not ported).
fn ft_80082b1c(g: &mut GroundColl, p: &mut Player, c: EntryCommon, rand: &mut HsdRand) -> Result<(), String> {
    // ftCo_800D0EC8 = -ftCo_CalcYScaledKnockback(p_ftCommonData->x310, x34_scale.y, 804D6524->x30).
    let cd = melee_fighter::hooks::common_data();
    let thr = -calc_y_scaled_knockback(cd.x310, p.fp.x34_scale.y, c.x804d6524_x30);
    if p.fp.self_vel.y > thr {
        // ft_8008A2BC -> ft_8008A348(gobj, 0): off the air first (ftCommon_8007D7FC), then Wait. Peach's
        // parasol (ft_8008A348's item check) cannot be held here.
        if p.fp.ground_or_air == melee_types::GA_Air {
            ftcommon_8007d7fc(g, p, c)?;
        }
        return ft_8008a2bc(p, rand);
    }
    landing_enter_basic(g, p, c, rand)
}

/// `ftCo_Landing_Enter_Basic`.
fn landing_enter_basic(g: &mut GroundColl, p: &mut Player, c: EntryCommon, rand: &mut HsdRand) -> Result<(), String> {
    // No hammer (ftCo_800C5240) -> ftCo_Landing_Enter(Landing, true, 0, 0, 1).
    if p.fp.item_gobj != core::ptr::null_mut() {
        return Err("ftCo_800C5240 with an item: not ported".into());
    }
    ftcommon_8007d7fc(g, p, c)?;
    let row = p.row_landing.clone();
    change_motion_state(p, FTCO_MS_LANDING, row.as_ref(), rand)?;
    p.landing_allow_interrupt = true;
    // The per-kind switch (Mario, Peach, Marth, G&W, ICs, Kirby, Mewtwo) touches neither kind wired.
    Ok(())
}

/// `ftCo_CalcYScaledKnockback` (ftchangeparam.c; no fused site).
fn calc_y_scaled_knockback(arg0: f32, scale: f32, arg2: f32) -> f32 {
    assert!(scale != 0.0, "ftchangeparam.c:0x1E: scale != 0.0F");
    if arg2 == 0.0 {
        return arg0;
    }
    if arg2 < 0.0 {
        return arg0 / calc_y_scaled_knockback(1.0, scale, -arg2);
    }
    if scale >= 1.0 || arg2 <= 1.0 {
        // One fused site: fmuladd((scale - 1) * arg0, arg2, arg0).
        return melee_core::fp::fmadds((scale - 1.0) * arg0, arg2, arg0);
    }
    arg0 * scale / arg2
}

/// `ftCommon_8007D7FC`: the air-to-ground bookkeeping, then `ftCommon_8007D6A4` (land).
fn ftcommon_8007d7fc(g: &mut GroundColl, p: &mut Player, c: EntryCommon) -> Result<(), String> {
    let fp = &mut *p.fp;
    if fp.ground_or_air == melee_types::GA_Air {
        // un_803224DC(spawn, x, kb_mag): out of the floor box +- horiz_margin plays a crowd gasp.
        let b = g.coll_box[1];
        if fp.cur_pos.x < c.crowd_horiz_margin + b.left || fp.cur_pos.x > b.right - c.crowd_horiz_margin {
            return Err("un_803224DC out of bounds -> un_8032201C (crowd) not ported".into());
        }
        // pl_8003FC44(player_id, is_sub_fighter): sets a stats bit (xDD0.bit0) nothing hashed reads.
    }
    // ftCommon_8007D6A4 (TARGET_PC floor guard: the floor line must be in range and enabled).
    let line = fp.coll_data.floor.index;
    if !(line >= 0 && line < g.line_count && g.mp_lib_80054ed8(line)) {
        return Err("ftCommon_PcFloorGuard: no floor under the fighter (falls)".into());
    }
    if unsafe { fp.anon_594.anon_0.x594_b0() } != 0 {
        fp.self_vel.x = fp.x6A4_transNOffset.z * fp.facing_dir;
    }
    let max = fp.co_attrs.ground_max_horizontal_velocity;
    fc::clamp_ground_vel(fp, max);
    fp.ground_or_air = melee_types::GA_Ground;
    fp.gr_vel = fp.self_vel.x;
    fp.x1968_jumpsUsed = 0;
    fp.x1969_walljumpUsed = 0;
    fp.set_x2227_b0(0);
    // ftCommon_UnlockECB.
    fp.ecb_lock = 0;
    fp.coll_data.x130_flags &= !mc::COLLDATA_X130_LOCKED;
    Ok(())
}

/// `ftCo_Fall_Enter` (not Master/Crazy Hand; no hammer, `x2224_b2` clear).
fn fall_enter(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.fp.item_gobj != core::ptr::null_mut() {
        return Err("ftCo_800C5240 with an item: not ported".into());
    }
    // Fighter_ChangeMotionState(Fall, Ft_MF_KeepFastFall, 0, 1, 0).
    let row = p.row_fall.clone();
    change_motion_state(p, FTCO_MS_FALL, row.as_ref(), rand)?;
    fc::clamp_air_drift(&mut p.fp);
    p.fall_smid = FTCO_SM_FALL as i32;
    p.fall_x4 = 0.0;
    if p.fp.ground_or_air == melee_types::GA_Ground {
        melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp);
    }
    Ok(())
}

/// `ftCo_Fall_Anim_Inner(gobj, &fall.x4, Fall, FallF, FallB)` + `ftCo_800CC988`.
fn fall_anim(p: &mut Player) -> Result<(), String> {
    let cd = melee_fighter::hooks::common_data();
    let fp = &p.fp;
    let mut frac = fp.self_vel.x / fp.co_attrs.air_drift_max;
    if frac > 1.0 {
        frac = 1.0;
    } else if frac < -1.0 {
        frac = -1.0;
    }
    let abs = fc::abs_c(frac);
    let (neutral, fwd, back) = if fp.motion_id == FTCO_MS_FALL_AERIAL { (FTCO_SM_FALL_AERIAL as i32, 24, 25) } else { (FTCO_SM_FALL as i32, 21, 22) };
    let (smid, v) = if abs > cd.x444 {
        (if frac * fp.facing_dir > 0.0 { fwd } else { back }, (abs - cd.x444) / (1.0 - cd.x444))
    } else {
        (neutral, 0.0)
    };
    p.fall_x4 += cd.x448 * (v - p.fall_x4);
    if p.fall_x4 != 0.0 && smid != p.fall_smid {
        // ftAnim_8006EDD0(fp, smid, cur_anim_frame, 1), HSD_JObjAnimAll(x8AC), then
        // ftAnim_8006FE9C(TransN, x4, 1 - x4) or ftAnim_8006FF74 at x4 == 1.
        let (row, tree) = load_row(p, smid as u32)?.ok_or(format!("Fall row {smid} has no figatree"))?;
        let frame = p.fp.cur_anim_frame;
        p.skel.anim_skeleton_set(&tree, frame, 1.0, row.looping());
        p.skel.anim_all2(&mut AObjGlobals::default())?;
        let x4 = p.fall_x4;
        p.skel.blend_from2(x4, 1.0 - x4, x4 != 1.0);
        p.fall_smid = smid;
    }
    // ftCo_800CC988(gobj, x4): the same blend again, every frame the mix is nonzero.
    if p.fall_x4 != 0.0 {
        p.skel.anim_all2(&mut AObjGlobals::default())?;
        let x4 = p.fall_x4;
        p.skel.blend_from2(x4, 1.0 - x4, x4 != 1.0);
    }
    Ok(())
}

/// `Fighter_procUpdate` after phys_cb (outside hitlag): the knockback decay, `self_vel += self_accel`, and
/// `cur_pos += selfVel + kb_vel + atk_shield_kb` (+ nudge). Wind (`ftColl_GetWindOffsetVec`) is not modelled.
fn proc_update_integrate(fp: &mut Fighter) -> Result<(), String> {
    use melee_core::fp::fmadds;
    use melee_runtime::gekko::{cosf, sinf, sqrtf};
    use melee_runtime::lbtrigf::atan2f;
    let c = melee_fighter::hooks::common_data();
    let air = fp.ground_or_air == melee_types::GA_Air;
    let kbx0 = fp.x8c_kb_vel.x;
    if kbx0 != 0.0 || fp.x8c_kb_vel.y != 0.0 {
        if air {
            if fp.is_sandbag() != 0 {
                return Err("a sandbag's knockback decay is not ported".into());
            }
            let (kx, ky) = (fp.x8c_kb_vel.x, fp.x8c_kb_vel.y);
            let a = atan2f(ky, kx);
            let d = c.x204_knockbackFrameDecay;
            if sqrtf(fmadds(kx, kx, ky * ky)) < d {
                fp.x8c_kb_vel.x = 0.0;
                fp.x8c_kb_vel.y = 0.0;
            } else {
                fp.x8c_kb_vel.x = fmadds(-d, cosf(a), fp.x8c_kb_vel.x);
                fp.x8c_kb_vel.y = fmadds(-d, sinf(a), fp.x8c_kb_vel.y);
            }
            fp.xF0_ground_kb_vel = 0.0;
        } else {
            if fp.xF0_ground_kb_vel == 0.0 {
                fp.xF0_ground_kb_vel = kbx0;
            }
            let fr = melee_fighter::hooks::ground_friction_mul(fp) * fp.co_attrs.ground_friction * c.x200;
            fc::apply_grounded_knockback_friction(fp, fr);
            let n = fp.coll_data.floor.normal;
            fp.x8c_kb_vel.x = n.y * fp.xF0_ground_kb_vel;
            fp.x8c_kb_vel.y = -n.x * fp.xF0_ground_kb_vel;
        }
    }
    if fp.x98_atk_shield_kb.x != 0.0 || fp.x98_atk_shield_kb.y != 0.0 {
        return Err("an attacker-shield knockback (x98) is not ported".into());
    }
    fp.gr_vel += fp.xE4_ground_accel_1 + fp.xE8_ground_accel_2;
    fp.xE4_ground_accel_1 = 0.0;
    fp.xE8_ground_accel_2 = 0.0;
    fp.self_vel.x += fp.x74_self_accel.x;
    fp.self_vel.y += fp.x74_self_accel.y;
    fp.self_vel.z += fp.x74_self_accel.z;
    fp.x74_self_accel.x = 0.0;
    fp.x74_self_accel.y = 0.0;
    fp.x74_self_accel.z = 0.0;
    let mut v = fp.self_vel;
    if fp.dmg.x1948 != 0 {
        let c2 = 1.0f32 - fp.dmg.x194C as f32 / fp.dmg.x1948 as f32;
        v.x = fmadds(c2, fp.self_vel.x - fp.xA4_unk_vel.x, fp.xA4_unk_vel.x);
        v.y = fmadds(c2, fp.self_vel.y - fp.xA4_unk_vel.y, fp.xA4_unk_vel.y);
        fp.dmg.x194C -= 1;
        if fp.dmg.x194C == 0 {
            fp.dmg.x1948 = 0;
        }
    }
    fp.cur_pos.x += fp.xF8_playerNudgeVel.x;
    fp.cur_pos.z += fp.xF8_playerNudgeVel.y;
    if fp.x2222_b6() != 0 && fp.x2222_b7() == 0 {
        return Err("Fighter_procUpdate: x2222_b6 (the unk_vel path) is not ported".into());
    }
    fp.cur_pos.x += v.x;
    fp.cur_pos.y += v.y;
    fp.cur_pos.z += v.z;
    fp.cur_pos.x += fp.x8c_kb_vel.x;
    fp.cur_pos.y += fp.x8c_kb_vel.y;
    fp.cur_pos.x += fp.x98_atk_shield_kb.x;
    fp.cur_pos.y += fp.x98_atk_shield_kb.y;
    fp.cur_pos.z += fp.x98_atk_shield_kb.z;
    Ok(())
}

/// The golden frame being simulated (`MELEE_GOLDEN_RAND` trace).
static mut TRACE_FRAME: i32 = 0;

impl Sim for VsMatch {
    fn step(&mut self) -> bool {
        unsafe { TRACE_FRAME = self.frame + 1 };
        if self.stopped.is_some() {
            return false;
        }
        let r = self.fighters().and_then(|_| self.hit_guard()).and_then(|_| self.stage_anim()).and_then(|_| {
            let rand = &mut self.rand;
            if let Some(b) = &mut self.last_bg {
                b.update(&mut || rand.randf());
            }
            self.effects_update()
        });
        if let Err(e) = r {
            self.stopped = Some(format!("frame {}: {e}", self.frame + 1));
            return false;
        }
        if std::env::var_os("MELEE_GOLDEN_RAND").is_some() {
            for p in &self.players {
                eprintln!("R {} MOTION slot {} {} id {} frame {} pos {} {} vel {} {} face {}", unsafe { TRACE_FRAME }, p.slot, p.follower, p.fp.motion_id, p.fp.cur_anim_frame, p.fp.cur_pos.x, p.fp.cur_pos.y, p.fp.self_vel.x, p.fp.self_vel.y, p.fp.facing_dir);
            }
        }
        for k in 0..self.effects.len() {
            self.effect_matrices(k);
        }
        for e in self.fighter_jobjs.clone() {
            self.refresh_fighter_jobj(e.0, e.1, e.2);
        }
        let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
        for mask in [PTCL_MASK_MAIN, PTCL_MASK_AUX] {
            if std::env::var_os("MELEE_GOLDEN_RAND").is_some() {
                eprintln!("R {} PHASE {mask:#x}", unsafe { TRACE_FRAME });
            }
            self.ps.particles_update_all(&mut host, mask);
            self.ps.gen_update_all(&mut host, mask);
        }
        // The HUD's procs (p_link 14+, after the particles), per player: ifStatus_802F5B48 (the damage read and
        // the shake flag), ifStatus_802F4EDC (the death animation's 8 draws, else the shake: ifStatus_802F4B84).
        for p in &mut self.players {
            if p.follower {
                continue;
            }
            let rand = &mut self.rand;
            p.hud.step(p.fp.dmg.x1830_percent, p.hud_randomize, &mut || rand.randf());
            p.hud_randomize = false;
        }
        self.frame += 1;
        true
    }
    fn frame_words(&self) -> FrameWords {
        self.words()
    }
}

/// `ftColl_8007B320` + `lbColl_800083C4` for each hurt capsule, then `ftCo_800A0DA4`.
fn hurt_extents(p: &mut Player) {
    unsafe { melee_fighter::tu::ftCo_0A01::ext::hook::lbColl_800083C4 = Some(|_| ()) };
    let n = p.hurtboxes.len().min(p.fp.hurt_capsules.len());
    p.fp.hurt_capsules_len = n as _;
    for i in 0..n {
        let (part, a, b, scale) = p.hurtboxes[i];
        // lbColl_800083C4 only computes a capsule's positions once per frame (skip_update_pos, cleared by
        // ftColl_8007AF28 at the end of Fighter_procUpdate): the s_link 13 hit test usually did it already.
        p.fp.hurt_capsules[i].capsule.scale = scale;
        if p.fp.hurt_capsules[i].capsule.skip_update_pos() != 0 {
            continue;
        }
        let j = p.skel.part(part as usize);
        let pa = p.skel.world_point(j, Vec3 { x: a[0], y: a[1], z: a[2] });
        let pb = p.skel.world_point(j, Vec3 { x: b[0], y: b[1], z: b[2] });
        let c = &mut p.fp.hurt_capsules[i].capsule;
        c.a_pos = melee_types::Vec { x: pa.x, y: pa.y, z: pa.z };
        c.b_pos = melee_types::Vec { x: pb.x, y: pb.y, z: pb.z };
        c.set_skip_update_pos(1);
        if std::env::var_os("MELEE_GOLDEN_HRT").is_some() && p.slot == 0 {
            eprintln!("HRT t{} i{} bone {} a {:08X} {:08X} {:08X} b {:08X} {:08X} {:08X} sc {:08X}", p.fp.cpu.x7C, i, part, pa.x.to_bits(), pa.y.to_bits(), pa.z.to_bits(), pb.x.to_bits(), pb.y.to_bits(), pb.z.to_bits(), scale.to_bits());
        }
    }
    if std::env::var_os("MELEE_GOLDEN_HRT").is_some() && p.slot == 0 && (1..=5).contains(&p.fp.cpu.x7C) {
        for q in 0..70usize.min(p.skel.parts.len()) {
            let j = p.skel.part(q);
            let w = p.skel.world_pos(j);
            eprintln!("PRT t{} q{} {:08X} {:08X} {:08X}", p.fp.cpu.x7C, q, w.x.to_bits(), w.y.to_bits(), w.z.to_bits());
        }
    }
    unsafe { melee_fighter::tu::ftCo_0A01::ftCo_800A0DA4(&mut p.fp) };
    if std::env::var_os("MELEE_GOLDEN_HC").is_some() && p.slot == 0 {
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if (118..=121).contains(&n) {
            for q in 0..n_caps_dbg(p) {
                let c = &p.fp.hurt_capsules[q].capsule;
                eprintln!("HC g{} i{} a {:08X} {:08X} {:08X} b {:08X} {:08X} {:08X}", n, q, c.a_pos.x.to_bits(), c.a_pos.y.to_bits(), c.a_pos.z.to_bits(), c.b_pos.x.to_bits(), c.b_pos.y.to_bits(), c.b_pos.z.to_bits());
            }
        }
    }
}

fn n_caps_dbg(p: &Player) -> usize {
    p.fp.hurt_capsules_len as usize
}
