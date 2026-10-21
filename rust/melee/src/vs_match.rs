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
//! - Each frame: `ftCo_Entry_Anim` (timer decrement). When the timer reaches 0 the C calls
//!   `ftCo_800C6408` (EntryStart: Fighter_ChangeMotionState + the entry animation), which is not
//!   ported: the run stops there and says so.
//! - Seed: the HSD_Rand state (ported, melee_runtime::random) seeded with MELEE_TEST_SEED. No
//!   ported code draws from it yet; in the C build every draw in these frames is made by the HSD
//!   particle generator (generator.c hsd_8039DAD4/8039EE24/8039F05C, particle.c hsd_8039930C) for
//!   the entry effects, so the `seed` column diverges from frame 0.

use crate::{FrameWords, Sim};
use melee_fighter::stale;
use melee_runtime::dat::DatArchive;
use melee_runtime::random::HsdRand;
use melee_types::neutral_hash::nh_fighter;
use melee_types::{Fighter, FighterKind, Ft_Kind_Captain, Ft_Kind_Fox};

/// `ftCo_MS_Entry` (ftmotionstates.c: 322).
const FTCO_MS_ENTRY: i32 = 322;

pub struct Player {
    pub slot: i32,
    pub fp: Box<Fighter>,
    /// `fp->mv.co.entry.timer` (kept beside the Fighter until the motion-state union is wired).
    pub entry_timer: i32,
}

pub struct VsMatch {
    pub players: Vec<Player>,
    pub rand: HsdRand,
    pub frame: i32,
    /// Set when a frame needs code that is not ported; the run ends before logging that frame.
    pub stopped: Option<String>,
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
        let points = crate::stage::spawn_points(&disc.read(stage_file)?)?;
        let plco = DatArchive::parse(&disc.read("PlCo.dat")?).map_err(|e| format!("PlCo.dat: {e}"))?;
        let common = plco.public_offset("ftLoadCommonData").ok_or("PlCo.dat: no ftLoadCommonData")?;
        // Fighter_LoadCommonData: the root is a pointer table, `p_ftCommonData = pData[0]`.
        // `p_ftCommonData->x260_startShieldHealth`; FT_SCRIPT_VALUE is the identity in vanilla.
        let be = |at: u32| u32::from_be_bytes(plco.data[at as usize..][..4].try_into().unwrap());
        let ft_common = be(common);
        let start_shield = f32::from_bits(be(ft_common + 0x260));

        let mut players = Vec::new();
        for (i, name) in fighters.iter().enumerate() {
            let slot = i as i32;
            let sp = points.iter().find(|p| p.id == slot).ok_or(format!("{stage_file}: no spawn point {slot}"))?;
            if !sp.exact {
                eprintln!("melee: spawn point {slot} is under a rotated/scaled joint: position not bit-exact (TODO group1 jobj)");
            }
            let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
            fp.kind = fighter_kind(name)?;
            fp.player_id = slot as u8;
            // fighter.c Fighter_UnkInitReset/Fighter_ResetInputData_80068854 defaults that the hash reads.
            fp.shield_health = start_shield;
            fp.x680 = 255;
            fp.x67F = 255;
            fp.cur_pos.x = sp.pos[0];
            fp.cur_pos.y = sp.pos[1];
            fp.cur_pos.z = sp.pos[2];
            fp.facing_dir = direction(sp.pos[0]);
            stale::set_stocks(slot, stocks);
            // The Entry state starts airborne (ported ftCommon_8007D5D4: jumps used 1, ECB lock).
            melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut fp);
            // ftCo_Entry_Enter: Fighter_ChangeMotionState(Entry) with no animation leaves
            // cur_anim_frame -1; ftaction.c:1408 frame_count = cur_anim_frame + x898_unk.
            fp.motion_id = FTCO_MS_ENTRY;
            fp.cur_anim_frame = -1.0;
            fp.x3E4_fighterCmdScript.frame_count = fp.cur_anim_frame + fp.x898_unk;
            if std::env::var_os("MELEE_GOLDEN_DUMP").is_some() {
                eprintln!("p{} kind {} motion {} pos {:?} facing {} ga {} shield {} fc {} jumps {} 67F {} 680 {} stocks {}",
                    slot + 1, fp.kind, fp.motion_id, (fp.cur_pos.x, fp.cur_pos.y, fp.cur_pos.z), fp.facing_dir, fp.ground_or_air,
                    fp.shield_health, fp.x3E4_fighterCmdScript.frame_count, fp.x1968_jumpsUsed, fp.x67F, fp.x680, stale::stocks(slot));
            }
            players.push(Player { slot, fp, entry_timer: 5 * (slot + 1) });
        }
        let mut rand = HsdRand::default();
        rand.set_logic_seed(seed);
        Ok(Self { players, rand, frame: -1, stopped: None })
    }

    fn words(&self) -> FrameWords {
        let mut w = FrameWords { frame: self.frame as u32, seed: self.rand.logic_seed(), ..Default::default() };
        for p in &self.players {
            // Slot = player*2 + follower; is_cpu false (human scenario).
            let s = p.slot * 2;
            w.slots[s as usize] = Some(nh_fighter(&p.fp, s, stale::stocks(p.slot), false));
        }
        w
    }
}

impl Sim for VsMatch {
    fn step(&mut self) -> bool {
        if self.stopped.is_some() {
            return false;
        }
        for p in &mut self.players {
            if p.fp.motion_id == FTCO_MS_ENTRY {
                // ftCo_Entry_Anim
                if p.entry_timer == 0 {
                    self.stopped = Some(format!(
                        "frame {}: player {} Entry timer hit 0 -> ftCo_800C6408 (EntryStart) is not ported",
                        self.frame + 1, p.slot + 1));
                    return false;
                }
                p.entry_timer -= 1;
            }
        }
        self.frame += 1;
        true
    }
    fn frame_words(&self) -> FrameWords {
        self.words()
    }
}
