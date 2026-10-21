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
use melee_types::{ftCollisionBox, Fighter, FighterKind, Ft_Kind_Captain, Ft_Kind_Fox};
use melee_world::mp::coll::{self as mc, CollGlobals};
use melee_world::mp::lines::GroundColl;
use melee_world::gr::bgm::{self, BgmUnlocks, StageParam};
use melee_world::gr::grlast::LastBg;
use std::collections::HashMap;
use std::sync::Arc;

/// `ftCo_MS_Entry` / `EntryStart` / `EntryEnd` (ftmotionstates.c: 322-324).
const FTCO_MS_ENTRY: i32 = 322;
const FTCO_MS_ENTRY_START: i32 = 323;
const FTCO_MS_ENTRY_END: i32 = 324;
/// `ftCo_SM_EntryStart` (ft/kinds/ftCommon/forward.h): EntryStart's animation row.
const FTCO_SM_ENTRY_START: u32 = 238;
/// `ftCo_MS_Fall` (29) and its row `ftCo_SM_Fall` (20).
const FTCO_MS_FALL: i32 = 29;
const FTCO_SM_FALL: u32 = 20;
/// The entry beam: `efAsync_Spawn(.., 0x43E, ..)` -> efAsync_Dispatch -> `efLib_Create_Attach(0x24)`.
const EF_ENTRY_BEAM: u32 = 0x24;
/// `Gr_Kind_Last`'s StKind in a VS match (`grGroundParam->stage_params[0].stkind`).
const STKIND_FD: i32 = 0x20;
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
    pub fp: Box<Fighter>,
    pub entry: EntryVars,
    /// The fighter's JObj tree (`GET_JOBJ(gobj)`) and parts.
    pub skel: Skeleton,
    /// `fp->anim_id != -1`.
    animating: bool,
    /// Animation rows (`ftData->xC[smid]`) of the states the match reaches, with their figatrees.
    row_entry_start: Option<(AnimRow, FigaTree)>,
    row_fall: Option<(AnimRow, FigaTree)>,
    /// `ftData->x44`.
    pub ecb_desc: EcbDesc,
    /// `ft_data->x0->trophy_scale` and `model_scaling` (`co_attrs`).
    trophy_scale: f32,
    model_scaling: f32,
    /// `fp->mv.co.fall.smid` / `.x4` (the Fall row and its blend weight).
    fall_smid: i32,
    fall_x4: f32,
    /// `fp->x60C`: queued `efAsync_Spawn`s (flushed in the fighter's s_link 9 proc).
    ef_queue: Vec<u32>,
}

/// `p_ftCommonData` words the entry states read (PlCo.dat `ftLoadCommonData`, ft/types.h).
#[derive(Clone, Copy, Debug)]
struct EntryCommon {
    x6bc: i32,
    x6c0: i32,
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
}

struct EffectTrack {
    joint: JObjRef,
    chain: Vec<u32>,
    aobj: HsdAObj<()>,
}

/// A stage particle track and its AObj (see `stage::dptcl_tracks`).
struct DptclAnim {
    joint: JObjRef,
    aobj: HsdAObj<()>,
}

pub struct VsMatch {
    pub players: Vec<Player>,
    /// mplib's map collision (`groundCollLine` & co.) and mpcoll's statics.
    pub ground: GroundColl,
    pub cg: CollGlobals,
    pub rand: HsdRand,
    pub ps: PsSystem,
    pub frame: i32,
    /// Set when a frame needs code that is not ported; the run ends before logging that frame.
    pub stopped: Option<String>,
    last_bg: LastBg,
    dptcl: Vec<DptclAnim>,
    aobj_globals: AObjGlobals,
    /// Bind-pose world matrices of the joints a generator can follow (by DAT offset; an effect
    /// joint is `(instance << 24) | offset`).
    joint_mtx: HashMap<JObjRef, Mtx>,
    common: EntryCommon,
    effect_banks: Vec<crate::effect::EffectBank>,
    effects: Vec<Effect>,
    next_effect_instance: u32,
    /// Bank 0's command lists that `efLib_Cb_PtclAppSRTHook` acts on (0x96, 0x97, 0x98, 0x21B).
    appsrt_hook_lists: Vec<CmdRef>,
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
        _ => Err(format!("fighter {name:?} not wired yet (falcon, fox)")),
    }
}

impl VsMatch {
    pub fn new(disc: &mut crate::disc::Disc, stage_file: &str, fighters: &[&str], seed: u32, stocks: i32) -> Result<Self, String> {
        if stage_file != "GrNLa.dat" {
            return Err(format!("{stage_file}: only Final Destination's stage code is wired"));
        }
        let stage_bytes = disc.read(stage_file)?;
        let points = crate::stage::spawn_points(&stage_bytes)?;
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
        let common = EntryCommon { x6bc: be(ft_common + 0x6BC) as i32, x6c0: be(ft_common + 0x6C0) as i32 };

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
        let effect_banks = vec![efco];

        // grLast_OnInit -> grLast_8021A9C4 (map gobj 3) -> grLast_8021B920(1): the level gobjs
        // 4..8 are created in order; gobj 7's on_init is grLast_8021AC30.
        let last_bg = {
            let r = std::cell::RefCell::new(&mut rand);
            LastBg::init(&mut || r.borrow_mut().randf(), &mut |m| r.borrow_mut().randi(m))
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
        for (g, anim) in [(4, 0), (5, 4), (6, 6), (7, 8), (8, 10), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0)] {
            if !seen.insert((g, anim)) {
                continue;
            }
            for t in crate::stage::dptcl_tracks(&stage, g, anim) {
                let mut a: HsdAObj<()> = HsdAObj::load_desc(&t.aobj, |_| None);
                a.req_anim(0.0);
                a.set_rate(1.0);
                joint_mtx.insert(t.joint, crate::stage::bind_pose_matrix(&stage.data, &t.chain));
                dptcl.push(DptclAnim { joint: t.joint, aobj: a });
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
        let mut players = Vec::new();
        for (i, name) in fighters.iter().enumerate() {
            let slot = i as i32;
            let sp = points.iter().find(|p| p.id == slot).ok_or(format!("{stage_file}: no spawn point {slot}"))?;
            if !sp.exact {
                eprintln!("melee: spawn point {slot} is under a rotated/scaled joint: position not bit-exact (TODO group1 jobj)");
            }
            let files = crate::fighter_data::FighterFiles::load(disc, name)?;
            // Fighter_Create: the costume's joint tree (HSD_JObjLoadJoint) and ftParts_SetupParts;
            // ftPartsTable[kind]->parts_num (PlCo.dat pData[4]) checks the part walk.
            let kind = fighter_kind(name)?;
            let skel = {
                let (nr, sym) = crate::fighter_data::costume_for(name).ok_or("no costume file")?;
                let a = DatArchive::parse(&disc.read(nr)?).map_err(|e| format!("{nr}: {e}"))?;
                let at = a.public_offset(sym).ok_or(format!("{nr}: no {sym}"))?;
                let tables = be(common_root + 0x10);
                let pt = be(tables + 4 * kind as u32);
                Skeleton::load(&a.data, at, files.x8_x10(), files.model_scaling(), be(pt + 8))?
            };
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            fp.kind = kind;
            // Fighter_UnkInitLoad: fp->co_attrs = *ft_data->x0.
            fp.co_attrs = files.attrs();
            fp.player_id = slot as u8;
            // fighter.c Fighter_UnkInitReset/Fighter_ResetInputData_80068854 defaults that the hash reads.
            fp.shield_health = start_shield;
            fp.x680 = 255;
            fp.x67F = 255;
            fp.cur_pos.x = sp.pos[0];
            fp.cur_pos.y = sp.pos[1];
            fp.cur_pos.z = sp.pos[2];
            fp.facing_dir = direction(sp.pos[0]);
            // Fighter_UnkInitLoad_80068914: x34_scale.x = Player_GetModelScale (1 in a VS match);
            // Fighter_UnkInitReset: y = x; Fighter_80068E64: z = 1 off Flat Zone.
            fp.x34_scale.x = 1.0;
            fp.x34_scale.y = fp.x34_scale.x;
            fp.x34_scale.z = 1.0;
            let mut skel = skel;
            // Fighter_UnkProcessDeath_80068354: HSD_JObjSetTranslate(jobj, cur_pos) first; the
            // model scale comes after the spawn probe below.
            let root = skel.root;
            skel.set_translate(root, Vec3 { x: fp.cur_pos.x, y: fp.cur_pos.y, z: fp.cur_pos.z });
            stale::set_stocks(slot, stocks);
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
            players.push(Player {
                slot,
                fp,
                entry,
                skel,
                animating: false,
                fall_smid: 0,
                fall_x4: 0.0,
                row_entry_start: files.anim_row(FTCO_SM_ENTRY_START)?.zip(files.figatree(FTCO_SM_ENTRY_START)?),
                row_fall: files.anim_row(FTCO_SM_FALL)?.zip(files.figatree(FTCO_SM_FALL)?),
                ecb_desc,
                trophy_scale: files.trophy_scale(),
                model_scaling: files.model_scaling(),
                ef_queue: Vec::new(),
            });
            // ... Fighter_ChangeMotionState(Entry): no animation, cur_anim_frame -1;
            // ftaction.c:1408 frame_count = cur_anim_frame + x898_unk. Then ft_80084CB0: the
            // entry box is the ECB the probe left.
            let p = players.last_mut().unwrap();
            change_motion_state(p, FTCO_MS_ENTRY, None)?;
            p.fp.x3E4_fighterCmdScript.frame_count = p.fp.cur_anim_frame + p.fp.x898_unk;
            let e = p.fp.coll_data.ecb;
            p.entry.x2c = ftCollisionBox { top: e.top.y, bottom: e.bottom.y, left: e.left, right: e.right };
        }

        // gmvs.c -> Stage_80225074(fn_8016E5C0(...)): no alternate-music button held -> 0 -> 4.
        let param = crate::stage::ground_param(&stage).ok_or("stage dat: no grGroundParam")?;
        let rows = StageParam::parse_all(&stage.data, param);
        let pick = bgm::ground_801c24f8(&rows, STKIND_FD, bgm::stage_80225074_flags(0), &SaveUnlocks, &mut |m| rand.randi(m));
        if std::env::var_os("MELEE_GOLDEN_DUMP").is_some() {
            eprintln!("bgm {} alt {:?}; seed before frame 0 {:08X}; particle tracks {}", pick.bgm, pick.alt_flag, rand.logic_seed(), dptcl.len());
        }

        Ok(Self { players, ground, cg, rand, ps, frame: -1, stopped: None, last_bg, dptcl, aobj_globals: AObjGlobals::default(), joint_mtx, common, effect_banks, effects: Vec::new(), next_effect_instance: 1, appsrt_hook_lists })
    }

    fn words(&self) -> FrameWords {
        let mut w = FrameWords { frame: self.frame as u32, seed: self.rand.logic_seed(), ..Default::default() };
        for p in &self.players {
            // Slot = player*2 + follower; is_cpu false (human scenario).
            let s = p.slot * 2;
            w.slots[s as usize] = Some(nh_fighter(&p.fp, s, stale::stocks(p.slot), false));
            if std::env::var_os("MELEE_GOLDEN_FRAMES").is_some() {
                let f = &p.fp;
                eprintln!("NHD f{} s{} m{} pos {:08X} {:08X} {:08X} vel {:08X} {:08X} ga {} fc {:08X} j {} 67F {} 680 {} shield {:08X}",
                    self.frame, s, f.motion_id, f.cur_pos.x.to_bits(), f.cur_pos.y.to_bits(), f.cur_pos.z.to_bits(),
                    f.self_vel.x.to_bits(), f.self_vel.y.to_bits(), f.ground_or_air, f.x3E4_fighterCmdScript.frame_count.to_bits(),
                    f.x1968_jumpsUsed, f.x67F, f.x680, f.shield_health.to_bits());
            }
        }
        w
    }

    /// `Ground_801C1CD0` for the stage's particle tracks: `HSD_AObjInterpretAnim`, and for each
    /// 0x28 key `JObjUpdateFunc` -> `efLib_Cb_DPtcl(0, iv & 0x3F, (iv >> 6) & 0xFFFFFF, jobj)`.
    fn stage_anim(&mut self) -> Result<(), String> {
        let mut spawns = Vec::new();
        for t in &mut self.dptcl {
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
        self.dptcl_spawns(spawns)
    }

    /// The fighters' s_link 1..9 procs for one frame, in proc order (every fighter's s_link 1
    /// proc, then every fighter's s_link 3 proc, ...). Err: the frame needs an unported state.
    fn fighters(&mut self) -> Result<(), String> {
        let c = self.common;
        // s_link 1, Fighter_8006A360: the animation step (ftAnim_8006EBA4: AObj frame, then
        // ftAction_80073240's frame_count = cur_anim_frame + x898), then the state's anim_cb.
        for p in &mut self.players {
            if p.animating {
                // ftAnim_8006E9B4, no blend (x8A4 == 0), x594_b0 / x594_b2 clear.
                p.skel.anim(&mut AObjGlobals::default())?;
                p.fp.cur_anim_frame = p.skel.curr_frame().ok_or("ftAnim_8006F3DC: no animated part")?;
            }
            let fp = &mut p.fp;
            fp.x3E4_fighterCmdScript.frame_count = fp.cur_anim_frame + fp.x898_unk;
            match fp.motion_id {
                FTCO_MS_ENTRY => {
                    // ftCo_Entry_Anim
                    if p.entry.timer == 0 {
                        entry_start(p, c)?;
                    }
                    p.entry.timer -= 1;
                }
                FTCO_MS_ENTRY_START => {
                    // ftCo_EntryStart_Anim
                    p.entry.timer -= 1;
                    if p.entry.timer == 0 {
                        entry_end(p, c)?;
                    }
                }
                FTCO_MS_ENTRY_END => {
                    // ftCo_EntryEnd_Anim (Player_GetFlagsBit4 is clear in a plain VS match)
                    p.entry.timer -= 1;
                    if p.entry.timer == 0 {
                        // ftCommon_8007D92C: airborne -> ftCo_Fall_Enter.
                        fall_enter(p)?;
                    }
                }
                FTCO_MS_FALL => {
                    // ftCo_Fall_Anim -> ftCo_Fall_Anim_Inner(&fall.x4, Fall, FallF, FallB) and
                    // ftCo_800CC988(fall.x4): with no air drift the blend stays 0 and the row
                    // stays Fall (|self_vel.x / air_drift_max| <= x444 picks the neutral row).
                    fall_anim(p)?;
                }
                m => return Err(format!("player {}: motion {m} not ported", p.slot + 1)),
            }
        }
        // s_link 2, Fighter_8006ABA0: input_cb. ftCo_Fall_IASA's checks all read pad input, and
        // the C applies none before GO (self_vel.x stays 0 in the golden with the stick held):
        // the fighters' input is all zero here. TODO(group3): the IASA checks themselves.
        // s_link 4, Fighter_procUpdate: phys_cb, then the integration below.
        for p in &mut self.players {
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
                FTCO_MS_FALL => {
                    // ftCo_Fall_Phys -> ft_80084DB0: fall-fast check, fall, air drift.
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
                _ => {}
            }
            proc_update_integrate(&mut p.fp);
        }
        // s_link 6, Fighter_procMap (x221F_b3 clear): the ECB lock countdown, the root JObj to
        // cur_pos, the state's coll_cb, the root JObj to cur_pos again.
        for i in 0..self.players.len() {
            let p = &mut self.players[i];
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
                FTCO_MS_FALL => {
                    // ftCo_Fall_Coll: ft_800831CC(gobj, ftCo_80096CC8, ft_80082B1C).
                    if ft_800831cc(&mut self.ground, &mut self.cg, p)? {
                        return Err(format!("player {}: landed: ft_80082B1C -> Landing not ported", p.slot + 1));
                    }
                }
                m => return Err(format!("player {}: coll_cb of motion {m} not ported", p.slot + 1)),
            }
            let c = p.fp.cur_pos;
            p.skel.set_translate(root, Vec3 { x: c.x, y: c.y, z: c.z });
        }
        // s_link 9, Fighter_8006C80C: efAsync_QueueFlush(&fp->x60C).
        for i in 0..self.players.len() {
            let q = std::mem::take(&mut self.players[i].ef_queue);
            for gfx in q {
                self.ef_dispatch(i, gfx)?;
            }
        }
        Ok(())
    }

    /// efAsync_QueueProcessDeferred -> efSync_Spawn -> efAsync_Dispatch, for player `i`.
    fn ef_dispatch(&mut self, i: usize, gfx: u32) -> Result<(), String> {
        match gfx {
            0x43E => {
                // EF_SPAWN_ATTACH_PARAM: efLib_Create_Attach(0x24, gobj, fighter jobj), then
                // efAsync_SetEffectScaleXYZ(effect, entry.x8.x), the fighter JObj's scale x
                // (taken as co_attrs.model_scaling: positions only).
                let scale = self.players[i].model_scaling;
                self.ef_create_attach(EF_ENTRY_BEAM, i, scale)
            }
            g => Err(format!("efAsync_Dispatch {g:#x} is not ported")),
        }
    }

    /// `efLib_Create(gfx_id)` + `efLib_Create_Attach`: the effect's tracks and lifetime, then
    /// efAsync_Dispatch's anim queue (`HSD_JObjAnimAll` right away).
    fn ef_create_attach(&mut self, gfx_id: u32, owner: usize, scale: f32) -> Result<(), String> {
        let bank = (gfx_id / 1000) as usize;
        let b = self.effect_banks.get(bank).ok_or(format!("effect bank {bank} not loaded"))?;
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
        self.effects.push(Effect { lifetime, owner, scale, bank, tracks });
        self.effect_anim(self.effects.len() - 1)
    }

    /// Bind-pose world matrices of effect `k`'s joints, rooted at its owner's position.
    fn effect_matrices(&mut self, k: usize) {
        let e = &self.effects[k];
        let fp = &self.players[e.owner].fp;
        let d = &self.effect_banks[e.bank].archive.data;
        let pos = [fp.cur_pos.x, fp.cur_pos.y, fp.cur_pos.z];
        for t in &e.tracks {
            let m = crate::stage::bind_pose_matrix_rooted(d, &t.chain, pos, [e.scale; 3]);
            self.joint_mtx.insert(t.joint, m);
        }
    }

    /// `HSD_JObjAnimAll` on effect `k`: its particle keys -> `efLib_Cb_DPtcl`.
    fn effect_anim(&mut self, k: usize) -> Result<(), String> {
        self.effect_matrices(k);
        let mut spawns = Vec::new();
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
        self.dptcl_spawns(spawns)
    }

    /// `efLib_Cb_DPtcl(0, bank, id, jobj)` for each key.
    fn dptcl_spawns(&mut self, spawns: Vec<(i32, i32, JObjRef)>) -> Result<(), String> {
        for (bank, id, joint) in spawns {
            let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
            if bank == 0x1E {
                // grLib_801C99C0(bank, id, jobj, 1) -> hsd_8039EFAC(0, bank, id, jobj).
                self.ps.gen_create_on_jobj(&mut host, 0, bank, id, Some(joint));
                continue;
            }
            // efLib_SpawnParticleEffect(bank, id, jobj, 1): the special-cased ids are not ported;
            // the rest take the default hsd_8039EFAC(chk, bank, id, jobj).
            match id {
                0x4A38..=0x4A3A | 0xB798..=0xB79A | 0x2D | 0x2E | 0x31 | 0x127 | 0x2 | 0x6 | 0xA3 | 0xA7 | 0xAA | 0xF2
                | 0x12A | 0x132 | 0x133 | 0x16D..=0x170 | 0x7E2 | 0xD4 | 0x243 | 0xE3 => {
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

/// `ftCo_800C6408` (EntryStart), not a sub-fighter. Not modelled: the trophy-stand accessory JObj
/// (`ftCommon_SetAccessory`, render only), `lbAudioAx_80024304(0x8B)` and the colour animation
/// `ftCo_800BFFD0(0x75)` (neither hashed nor drawing from the RNG).
fn entry_start(p: &mut Player, c: EntryCommon) -> Result<(), String> {
    p.entry.timer = c.x6bc;
    let row = p.row_entry_start.clone();
    change_motion_state(p, FTCO_MS_ENTRY_START, row.as_ref())?;
    // temp_f0 = x34_scale.y * co_attrs.trophy_scale (x34_scale.y = Player_GetModelScale = 1 in VS).
    let s = 1.0f32 * p.trophy_scale;
    p.entry.x24 = s;
    // temp_f0_2 = 1.497345 * sp48.y: a double product rounded to f32.
    let h = (1.497345f64 * s as f64) as f32;
    p.entry.x28 = h;
    p.entry.x20 = h;
    // efAsync_Spawn(gobj, &fp->x60C, EF_SPAWN_ATTACH_PARAM, 0x43E, gobj->hsd_obj, &entry.x8):
    // queued (the current proc's s_link is below 9).
    p.ef_queue.push(0x43E);
    Ok(())
}

/// `ftCo_800C6B6C` (EntryEnd), not a sub-fighter.
fn entry_end(p: &mut Player, c: EntryCommon) -> Result<(), String> {
    p.entry.timer = c.x6c0;
    // HSD_JObjSetScale(GET_JOBJ(gobj), &entry.x8): the scale from before the entry squash.
    let root = p.skel.root;
    p.skel.set_scale(root, p.entry.x8);
    p.fp.cur_pos.y = p.entry.x4 + p.entry.x20;
    // Fighter_ChangeMotionState(EntryEnd, 0x3000, 0, 1, 0): no animation row.
    change_motion_state(p, FTCO_MS_ENTRY_END, None)
}

/// What `Fighter_ChangeMotionState(gobj, msid, flags, 0.0, 1.0, 0.0, NULL)` does to the hashed
/// fields and the skeleton in the entry states: `motion_id`; `HSD_JObjSetTranslate(jobj,
/// cur_pos)`; `ftPartSetRotX/Y/Z(fp, 0, 0, M_PI_2 * facing_dir, 0)`; `frame_speed_mul = 1`,
/// `cur_anim_frame = 0 - 1`, `x898 = 0`; with an animation row the figatree is attached
/// (`ftAnim_8006EBE8(gobj, 0, 1, 0)`: every row these states use has blend byte 0, checked against
/// the C), `ftAnim_8006E9B4` steps it to frame 0 and `ftAction_80073240` sets `frame_count`;
/// with none (`anim_id == -1`) `ftAnim_80070758` drops the AObjs and frame_count stays.
/// TODO(group3 fighter.c): the whole Fighter_ChangeMotionState.
fn change_motion_state(p: &mut Player, msid: i32, row: Option<&(AnimRow, FigaTree)>) -> Result<(), String> {
    let fp = &mut p.fp;
    fp.motion_id = msid;
    let root = p.skel.root;
    p.skel.set_translate(root, melee_runtime::mtx::Vec3 { x: fp.cur_pos.x, y: fp.cur_pos.y, z: fp.cur_pos.z });
    p.skel.set_part_rot(0, 0, 0.0);
    p.skel.set_part_rot(0, 1, (core::f64::consts::FRAC_PI_2 * fp.facing_dir as f64) as f32);
    p.skel.set_part_rot(0, 2, 0.0);
    fp.frame_speed_mul = 1.0;
    fp.cur_anim_frame = 0.0 - fp.frame_speed_mul;
    fp.x898_unk = 0.0;
    match row {
        Some((r, tree)) => {
            if r.flags & 0xA000_0000 != 0 {
                return Err(format!("anim row flags {:#x}: x594_b0 / x594_b2 paths not ported", r.flags));
            }
            p.skel.attach(tree, 0.0, fp.frame_speed_mul, r.looping());
            p.animating = true;
            p.skel.anim(&mut AObjGlobals::default())?;
            fp.cur_anim_frame = p.skel.curr_frame().ok_or("ftAnim_8006F3DC: no animated part")?;
            fp.x3E4_fighterCmdScript.frame_count = fp.cur_anim_frame + fp.x898_unk;
        }
        None => {
            p.animating = false;
            p.skel.remove_anim_all();
        }
    }
    Ok(())
}

/// `ftCo_Fall_Enter` (not Master/Crazy Hand; no hammer, `x2224_b2` clear).
fn fall_enter(p: &mut Player) -> Result<(), String> {
    if p.fp.item_gobj != core::ptr::null_mut() {
        return Err("ftCo_800C5240 with an item: not ported".into());
    }
    // Fighter_ChangeMotionState(Fall, Ft_MF_KeepFastFall, 0, 1, 0).
    let row = p.row_fall.clone();
    change_motion_state(p, FTCO_MS_FALL, row.as_ref())?;
    fc::clamp_air_drift(&mut p.fp);
    p.fall_smid = FTCO_SM_FALL as i32;
    p.fall_x4 = 0.0;
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
    let (smid, v) = if abs > cd.x444 {
        (if frac * fp.facing_dir > 0.0 { 21 } else { 22 }, (abs - cd.x444) / (1.0 - cd.x444))
    } else {
        (FTCO_SM_FALL as i32, 0.0)
    };
    p.fall_x4 += cd.x448 * (v - p.fall_x4);
    if p.fall_x4 != 0.0 {
        return Err(format!("Fall blend {} toward row {smid}: ftAnim_8006EDD0 + the skeleton blend are not ported", p.fall_x4));
    }
    Ok(())
}

/// The airborne, no-knockback part of `Fighter_procUpdate` after phys_cb: `self_vel +=
/// self_accel`, then `cur_pos += selfVel + kb_vel + atk_shield_kb` (+ nudge, + wind: zero on FD).
/// TODO(group3 fighter.c): the whole procUpdate (knockback decay, ground path, mpGetSpeed).
fn proc_update_integrate(fp: &mut Fighter) {
    debug_assert!(fp.x8c_kb_vel.x == 0.0 && fp.x8c_kb_vel.y == 0.0 && fp.dmg.x1948 == 0);
    fp.gr_vel += fp.xE4_ground_accel_1 + fp.xE8_ground_accel_2;
    fp.xE4_ground_accel_1 = 0.0;
    fp.xE8_ground_accel_2 = 0.0;
    fp.self_vel.x += fp.x74_self_accel.x;
    fp.self_vel.y += fp.x74_self_accel.y;
    fp.self_vel.z += fp.x74_self_accel.z;
    fp.x74_self_accel.x = 0.0;
    fp.x74_self_accel.y = 0.0;
    fp.x74_self_accel.z = 0.0;
    let v = fp.self_vel;
    fp.cur_pos.x += fp.xF8_playerNudgeVel.x;
    fp.cur_pos.z += fp.xF8_playerNudgeVel.y;
    fp.cur_pos.x += v.x;
    fp.cur_pos.y += v.y;
    fp.cur_pos.z += v.z;
    fp.cur_pos.x += fp.x8c_kb_vel.x;
    fp.cur_pos.y += fp.x8c_kb_vel.y;
    fp.cur_pos.x += fp.x98_atk_shield_kb.x;
    fp.cur_pos.y += fp.x98_atk_shield_kb.y;
    fp.cur_pos.z += fp.x98_atk_shield_kb.z;
}

impl Sim for VsMatch {
    fn step(&mut self) -> bool {
        if self.stopped.is_some() {
            return false;
        }
        let r = self.fighters().and_then(|_| self.stage_anim()).and_then(|_| {
            let rand = &mut self.rand;
            self.last_bg.update(&mut || rand.randf());
            self.effects_update()
        });
        if let Err(e) = r {
            self.stopped = Some(format!("frame {}: {e}", self.frame + 1));
            return false;
        }
        for k in 0..self.effects.len() {
            self.effect_matrices(k);
        }
        let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
        for mask in [PTCL_MASK_MAIN, PTCL_MASK_AUX] {
            self.ps.particles_update_all(&mut host, mask);
            self.ps.gen_update_all(&mut host, mask);
        }
        self.frame += 1;
        true
    }
    fn frame_words(&self) -> FrameWords {
        self.words()
    }
}
