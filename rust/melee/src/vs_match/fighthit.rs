//! Fighter-vs-fighter hit detection through the ported `ftcoll.c` (`melee_fighter::tu::ftcoll`), run on the
//! players' stand-in GObjs. The ported code reaches the rest of the game through `ext::*` hook pointers; the
//! ones a vanilla VS match uses are set here (script-game, Geno and Mex hooks are the identity / zero).
//!
//! Joints: the hit and hurt capsules name their bone by `HSD_JObj*`. Each player owns an array of pseudo
//! joints (`Fighter::parts`, leaked, one per skeleton part) registered in [`JOINTS`]; the `lb_8000B1CC` hook maps a
//! pseudo joint back to the skeleton to compute positions.

use super::*;
use melee_fighter::hooks as h;
use melee_fighter::tu::ftcoll as fcl;
use melee_fighter::tu::ftcoll::ext as e;
use melee_types::{FighterBone, FighterPartsTable, HSD_GObj, HSD_JObj, HitCapsule, HurtCapsule, Item};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// Pseudo joint address -> (player index in `VsMatch::players`, skeleton part).
    static JOINTS: RefCell<HashMap<usize, (usize, usize)>> = RefCell::new(HashMap::new());
}

/// Gives player `idx` its `Fighter::parts` (pseudo joints) and, once per kind, the `ftPartsTable` entry.
pub(super) fn setup_player(idx: usize, p: &mut Player) {
    let n = p.skel.parts.len().max(1);
    let mut bones: Vec<FighterBone> = Vec::with_capacity(n);
    JOINTS.with(|j| {
        let mut j = j.borrow_mut();
        for k in 0..n {
            let jo = Box::into_raw(Box::new(HSD_JObj::zeroed()));
            j.insert(jo as usize, (idx, k));
            let mut b = FighterBone::zeroed();
            b.joint = jo;
            bones.push(b);
        }
    });
    let bones: &'static mut [FighterBone] = Box::leak(bones.into_boxed_slice());
    p.fp.parts = bones.as_mut_ptr();
    unsafe {
        if h::FT_PARTS_TABLE.is_null() {
            let tab: Vec<*mut FighterPartsTable> = vec![core::ptr::null_mut(); 40];
            h::FT_PARTS_TABLE = Box::leak(tab.into_boxed_slice()).as_mut_ptr();
        }
        let slot = h::FT_PARTS_TABLE.add(p.fp.kind as usize);
        if (*slot).is_null() {
            let mut t = FighterPartsTable::zeroed();
            let map: &'static mut [u8] = Box::leak(p.part_to_joint.clone().into_boxed_slice());
            t.part_to_joint = map.as_mut_ptr();
            t.parts_num = map.len() as u32;
            *slot = Box::into_raw(Box::new(t));
        }
    }
}

fn joint_of(j: *mut HSD_JObj) -> Option<(usize, usize)> {
    JOINTS.with(|m| m.borrow().get(&(j as usize)).copied())
}

/// `lb_8000B1CC(jobj, &offset, &out)`: the world position of `offset` in the pseudo joint's frame.
fn lb_8000b1cc(j: *mut HSD_JObj, off: &melee_types::Vec, out: &mut melee_types::Vec) {
    let s = super::damage::sim();
    let Some((pi, part)) = joint_of(j) else {
        panic!("lb_8000B1CC: a joint that is not a fighter part");
    };
    let p = &mut s.players[pi];
    let jr = p.skel.part(part);
    let w = p.skel.world_point(jr, Vec3 { x: off.x, y: off.y, z: off.z });
    *out = melee_types::Vec { x: w.x, y: w.y, z: w.z };
}

/// Which player's hurt capsule `hurt` is (by address), and its index.
fn hurt_owner(hurt: *mut HurtCapsule) -> Option<(usize, usize)> {
    let s = super::damage::sim();
    for (pi, p) in s.players.iter().enumerate() {
        let base = p.fp.hurt_capsules.as_ptr() as usize;
        let sz = core::mem::size_of_val(&p.fp.hurt_capsules[0]);
        let a = hurt as usize;
        if a >= base && a < base + sz * p.fp.hurt_capsules.len() {
            let i = (a - base) / sz;
            // hurt_capsules[i].capsule is the first member of the FighterHurtCapsule: the same address.
            return Some((pi, i));
        }
    }
    None
}

/// Refreshes the hurt capsule's positions from the skeleton (`lbColl_800083C4`, once a frame).
fn refresh_hurt(hurt: *mut HurtCapsule) {
    if let Some((pi, i)) = hurt_owner(hurt) {
        let s = super::damage::sim();
        hit::hurt_position(&mut s.players[pi], i);
    }
}

pub(super) fn install() {
    unsafe {
        // The player list: the stand-in GObjs, linked by `next` before every use (`link_players`).
        h::PLINK_HEAD = Some(|link| {
            if link != melee_types::HSD_GOBJ_PLINK_FIGHTER as i32 {
                return core::ptr::null_mut();
            }
            let s = super::damage::sim();
            s.players.first_mut().map_or(core::ptr::null_mut(), |p| &mut *p.gobj as *mut HSD_GObj)
        });
        h::LB_8000B1CC = Some(lb_8000b1cc);
        h::GM_8016B168 = Some(|| false);
        h::GM_8016B0B4 = Some(|| false);
        // Echo capsules (script mods): none, so the four x914 capsules and plain group equality.
        e::SCRIPTGAME_ECHO_CAPSULE_COUNT = Some(|_| 4);
        e::SCRIPTGAME_ECHO_CAPSULE = Some(|fp, i| &mut (*fp).x914[i as usize] as *mut HitCapsule);
        e::SCRIPTGAME_ECHO_IS_CAPSULE = Some(|_| 0);
        e::SCRIPTGAME_ECHO_BLOCKED = Some(|_, _| 0);
        e::SCRIPTGAME_ECHO_SAME_GROUP = Some(|a, b| ((*a).x4 == (*b).x4) as i32);
        e::SCRIPTGAME_ECHO_TARGET_ALLOWED = Some(|s, t| {
            if s.is_null() || t.is_null() || s == t || (*s).player_id == (*t).player_id {
                return 0;
            }
            (!h::gm_8016b168() || false || (*t).x2225_b4() != 0 || (*s).team != (*t).team) as i32
        });
        e::SCRIPTGAME_ECHO_CONNECTED = Some(|_, _| ());
        e::SCRIPTGAME_ECHO_PREPARE = Some(|| ());
        e::SCRIPTGAME_ECHO_HIT_INDEX = Some(|fp, hit| {
            for i in 0..4 {
                if hit == &mut (*fp).x914[i] as *mut HitCapsule {
                    return i as i32;
                }
            }
            -1
        });
        e::SCRIPTGAME_ECHO_REPORT_CONTEXT = Some(|_, _, _| 0);
        e::SCRIPTGAME_ECHO_CREDIT = Some(|_, _, _, _| 0);
        e::SCRIPTGAME_ECHO_SOURCE_X = Some(|_, fallback| fallback);
        // Script / Geno / Mex: identity or nothing in a vanilla match.
        e::SCRIPT_GAME_EVENT = Some(|_, _, _, _, _| ());
        e::SCRIPT_ENEMY_GAME_EVENT = Some(|_, _, _, _, _, _| ());
        e::SCRIPTGAME_ENEMY_CONTACT = Some(|_, _, _| 0);
        e::SCRIPTGAME_SKILL_PERFECT_SHIELD = Some(|_, _, _, _| ());
        e::SCRIPTGAME_INTR_WIN_HIT = Some(|_, _| ());
        e::GENO_COMBO_CONTACT = Some(|_, _| ());
        e::SCRIPTGAME_HIT_RULE_CONTACT = Some(|_, _, v, _| v);
        e::SCRIPTGAME_FIGHTER_DEALT_DAMAGE = Some(|_, d| d);
        e::SCRIPTGAME_HIT_RULE_WON = Some(|_, _| ());
        e::GENO_HIT_STUN_BONUS = Some(|_, _, _| ());
        e::GENO_HIT_FLAGS = Some(|_, _, _| 0);
        e::SCRIPTGAME_HIT_RULE_PERCENT_QUEUE = Some(|_, _, _| ());
        e::SCRIPTGAME_CRIT_HIT = Some(|_, _, _, _, _| ());
        e::SCRIPTGAME_REPORT_HIT_CONTEXT = Some(|_, _, _, _| ());
        e::SCRIPTGAME_SKILL_HIT = Some(|_, _, _| ());
        e::SCRIPTGAME_HIT_RULE_CONTEXT = Some(|_, _| ());
        e::SCRIPTGAME_HIT_RULE_DAMAGE = Some(|_| ());
        e::SCRIPTGAME_ITEM_MOD_DAMAGE = Some(|_, d| d);
        e::SCRIPT_CLANK_WANTED = Some(|| 0);
        e::SCRIPT_CLANK = Some(|_, _, _, _, _, _, _, _, _, _, _, _| ());
        e::SCRIPTMODS_CLANK = Some(|_, _, _, _| ());
        e::GENO_ITEM_FAMILY_EVENT = Some(|_, _, _| ());
        e::GENO_ARTICLE_STUN_BONUS = Some(|_, _, _| ());
        e::GENO_AUTOLINK = Some(|_, _, _, _, _| 0);
        e::GENO_DEFINE_IS_NONE = Some(|_| 0);
        e::GENO_DEFINE_TEMPLATE_SKELETON = Some(|_| 0);
        e::MEX_ENABLED = Some(|_| 0);
        // lbcollision.c.
        use melee_runtime::tu::lbcollision as lb;
        // lbColl_80005BB0's hit sound (lbAudioAx_80024184): audio only.
        lb::ext::hook::lbAudioAx_80024184 = Some(|_, _, _, _| 0);
        e::LBCOLL_80008688 = Some(|hit, k, v| lb::lbColl_80008688(hit, k, v));
        e::LBCOLL_80008820 = Some(|hit, k, v| lb::lbColl_80008820(hit, k, v));
        e::LBCOLL_COPY_HIT_CAPSULE = Some(|s, d| lb::lbColl_CopyHitCapsule(s, d));
        e::LBCOLL_80008440 = Some(|hit| lb::lbColl_80008440(hit));
        e::LBCOLL_80008428 = Some(|hit| lb::lbColl_80008428(hit));
        e::LBCOLL_80008434 = Some(|hit| lb::lbColl_80008434(hit));
        e::LBCOLL_80005BB0 = Some(|_, _| -1); // the hit sound: audio only
        e::LBCOLL_8000ACFC = Some(|v, hit| lb::lbColl_8000ACFC(v, hit));
        e::LBCOLL_80007ECC = Some(|hit, hurt, m, a, b, z| {
            refresh_hurt(hurt);
            lb::lbColl_80007ECC(hit, hurt, m as _, a, b, z)
        });
        e::LBCOLL_80007AFC = Some(|a, b, sa, sb| lb::lbColl_80007AFC(a, b, sa, sb));
        e::LBCOLL_80007BCC = Some(|hit, res, m, i, a, b, z| lb::lbColl_80007BCC(hit, res, m as _, i, a, b, z));
        e::LBCOLL_8000805C = Some(|hit, hurt, m, i, a, b, z| {
            refresh_hurt(hurt);
            lb::lbColl_8000805C(hit, hurt, m as _, i, a, b, z)
        });
        e::LBCOLL_80007DD8 = Some(|hit, res, m, pos, dist, s| lb::lbColl_80007DD8(hit, res, m as _, pos, dist, s));
        e::LBCOLL_80008248 = Some(|hit, hurt, m, a, b, z| {
            refresh_hurt(hurt);
            lb::lbColl_80008248(hit, hurt, m as _, a, b, z)
        });
        e::LBCOLL_80008D30 = Some(|hit, d| lb::lbColl_80008D30(hit, d as *const _));
        // Staling and ratios.
        e::FT_80089228 = Some(|fp, id, inst, d| melee_fighter::stale::apply_staling(&*fp, id, inst, d));
        e::FTCO_800DEEB8 = Some(|fp, d| melee_fighter::ft0df::ftco_800deeb8(&*fp, d));
        e::GM_8016B0D4 = Some(|| false);
        e::GM_8016B1C4 = Some(|| false);
        e::GM_8016B248 = Some(|| 1.0);
        e::PLAYER_GET_DEFENSE_RATIO = Some(|_| 1.0);
        e::PLAYER_GET_ATTACK_RATIO = Some(|_| 1.0);
        e::PL_8003E150 = Some(|_, _| ());
        e::PL_8003E17C = Some(|_, _, _| ());
        e::PL_80038144 = Some(|_, _, _, _, _, _, _| ());
        e::PL_8003EB30 = Some(|_, _, _, _, _, _| ());
        e::PL_80041B08 = Some(|_, _, _| ());
        e::PL_8003D60C = Some(|_| false);
        e::PL_80037ECC = Some(|_| ());
        use melee_fighter::tu::ft_07c1 as c1;
        use melee_fighter::tu::ft_07c6 as c6;
        c6::ext::GM_8016B0B4 = Some(|| false);
        c6::ext::GM_801A4BB8 = Some(|| 0);
        c6::ext::FTCOLL_8007B868 = Some(|g| fcl::ftcoll_8007b868(g));
        c6::ext::LB_8000B1CC = Some(|j, o, out| lb_8000b1cc(j, &*o, &mut *out));
        c1::ext::LB_8000B1CC = Some(|j, o, out| lb_8000b1cc(j, &*o, &mut *out));
        c1::ext::FTCOLL_8007B868 = Some(|g| fcl::ftcoll_8007b868(g));
        c1::ext::LBCOLL_80008440 = Some(|hit| lb::lbColl_80008440(hit));
        c1::ext::LBCOLL_8000ACFC = Some(|v, hit| lb::lbColl_8000ACFC(v as *mut _, hit));
        c1::ext::LBCOLL_80008688 = Some(|hit, k, v| lb::lbColl_80008688(hit, k, v as *mut _));
        c1::ext::LBCOLL_80007AFC = Some(|a, b, sa, sb| lb::lbColl_80007AFC(a, b, sa, sb));
        c1::ext::LBCOLL_800089B8 = Some(|hit, v| lb::lbColl_800089B8(hit, v as *mut _));
        e::EFSYNC_SPAWN = Some(|id, _g, pos, arg| {
            let s = super::damage::sim();
            if let Err(m) = ef_sync_spawn(s, id, unsafe { *pos }, arg) {
                s.dmg_fail(m);
            }
        });
        e::PLSTALE_UPDATE_STALE_MOVES_FROM_FIGHTER = Some(|a, b| unsafe { melee_fighter::tu::plstale::plStale_UpdateStaleMovesFromFighter(a, b) });
        e::PLSTALE_UPDATE_STALE_MOVES_FROM_ITEM = Some(|a, b| unsafe { melee_fighter::tu::plstale::plStale_UpdateStaleMovesFromItem(a, b) });
        e::FT_80088080 = Some(|fp| melee_fighter::ft0877::ft_80088080(&mut *fp));
    }
}

/// Runs a ported `ftAction_*` command handler on the player's stand-in GObj over the PlXx.dat words at `p.script`.
/// The native `CmdUnion` is 8 bytes (it has pointer members) while the script words are 4, so the words are copied
/// into a window of native unions (each word in the union's first four bytes, big-endian as in the DAT).
pub(super) fn run_cmd(p: &mut Player, f: fn(*mut HSD_GObj, &mut melee_types::CommandInfo)) {
    const N: usize = 8;
    let mut win: [melee_types::CmdUnion; N] = unsafe { core::mem::zeroed() };
    let at = p.script as usize;
    for (i, w) in win.iter_mut().enumerate() {
        if let Some(b) = p.pdat.get(at + 4 * i..at + 4 * i + 4) {
            unsafe { core::ptr::copy_nonoverlapping(b.as_ptr(), w as *mut _ as *mut u8, 4) };
        }
    }
    let mut cmd = melee_types::CommandInfo::zeroed();
    cmd.anon_8.u = win.as_mut_ptr();
    f(&mut *p.gobj as *mut HSD_GObj, &mut cmd);
    let used = (unsafe { cmd.anon_8.u } as usize - win.as_ptr() as usize) / core::mem::size_of::<melee_types::CmdUnion>();
    p.script += 4 * used as u32;
}

/// The `s_link` the host reports to `HSD_GObj_CurrentInvokedProc` (hitboxes made past 9 are positioned at once).
pub(super) fn set_s_link(n: u8) {
    unsafe { melee_fighter::tu::ftaction::CURRENT_S_LINK = n };
}

/// Links the players' stand-in GObjs into the fighter list.
pub(super) fn link_players(s: &mut VsMatch) {
    let n = s.players.len();
    for i in 0..n {
        let next = if i + 1 < n { &mut *s.players[i + 1].gobj as *mut HSD_GObj } else { core::ptr::null_mut() };
        let prev = if i > 0 { &mut *s.players[i - 1].gobj as *mut HSD_GObj } else { core::ptr::null_mut() };
        s.players[i].gobj.next = next;
        s.players[i].gobj.prev = prev;
    }
}

/// Runs `f`, turning a panic of the ported code ("TODO: X not ported") into the run's stop reason.
pub(super) fn guarded(what: &str, f: impl FnOnce()) -> Result<(), String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).map_err(|e| {
        let m = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
        format!("{what}: {m}")
    })
}

/// `Fighter_8006C80C` after the accessory callback: the hitboxes follow their bones.
pub(super) fn update_hits(s: &mut VsMatch, i: usize) -> Result<(), String> {
    let g = &mut *s.players[i].gobj as *mut HSD_GObj;
    guarded("ftColl_8007AE80", || {
        fcl::ftcoll_8007ae80(g);
        melee_fighter::tu::ft_07c1::ft_8007c224(g);
        melee_fighter::tu::ft_07c6::ft_8007c6dc(g);
    })
}

thread_local! {
    /// The fighter whose hit the damage log is being resolved for (the owner of the hit sparks).
    static HIT_VICTIM: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(super) fn set_hit_victim(i: usize) {
    HIT_VICTIM.with(|c| c.set(i));
}

/// `efSync_Spawn(id, NULL, &pos, arg)` from the hit code (`gobj` is NULL: positional effects only).
fn ef_sync_spawn(s: &mut VsMatch, id: i32, pos: melee_types::Vec, arg: *const f32) -> Result<(), String> {
    let owner = HIT_VICTIM.with(|c| c.get());
    let at = Vec3 { x: pos.x, y: pos.y, z: pos.z };
    match id {
        // efAsync_Dispatch 0x3E8: Randi(8) picks effect 9 or 10, scaled by 0.04 * damage + 0.3 (0.3 .. 1.5).
        0x3E8 => {
            let u_dmg = unsafe { *arg };
            let eid = if s.rand.randi(8) == 0 { 9 } else { 10 };
            let sc = (0.04f32 * u_dmg + 0.3f32).clamp(0.3, 1.5);
            s.ef_create_rot(eid, owner, sc, Some([at.x, at.y, at.z]), None, None)?;
            if let Some(e) = s.effects.last_mut() {
                e.detached = true;
            }
            Ok(())
        }
        // 0x3EF / 0x3F0: efLib_CreateGenerator_Translate_FacingDir(0x42, &pos, -+*arg).
        0x3EF => {
            s.gen_translate_facing(0x42, at, -unsafe { *arg });
            Ok(())
        }
        0x3F0 => {
            s.gen_translate_facing(0x42, at, unsafe { *arg });
            Ok(())
        }
        // 0x3F3: efLib_CreateGenerator(0xB, &pos).
        0x3F3 => {
            let mut host = Host { rand: &mut s.rand, joint_mtx: &s.joint_mtx, appsrt_hook_lists: &s.appsrt_hook_lists };
            if let Some(g) = s.ps.gen_create(&mut host, 0, 0, 0xB) {
                s.ps.gen_mut(g).pos = at;
            }
            Ok(())
        }
        _ => Err(format!("efSync_Spawn({id:#x}) from the hit code is not ported")),
    }
}

/// `Fighter_UnkProcessGrab_8006CA5C` (s_link 12) for a fighter in a grab state (`x221E_b6`): `ftColl_80078A2C` looks
/// for a victim of the Catch hitbox, `ftColl_8007BC90` for a grabbable item. A successful grab is not ported.
/// (`ftColl_8007BA0C`, the stage's bury devices, has none here.)
pub(super) fn grab_proc(s: &mut VsMatch, i: usize) -> Result<(), String> {
    {
        let fp = &s.players[i].fp;
        if fp.x221F_b3() != 0 || fp.x221E_b6() == 0 {
            return Ok(());
        }
    }
    link_players(s);
    let g = &mut *s.players[i].gobj as *mut HSD_GObj;
    guarded("ftColl_80078A2C", || fcl::ftcoll_80078a2c(g))?;
    if !s.players[i].fp.victim_gobj.is_null() {
        return Err(format!("player {}: a successful grab (ftCo_Catch -> CatchPull / CapturePulled) is not ported", s.players[i].slot + 1));
    }
    guarded("ftColl_8007BC90", || fcl::ftcoll_8007bc90(g))?;
    if !s.players[i].fp.target_item_gobj.is_null() {
        return Err("a grabbable item (it_8027B4A4) is not ported".into());
    }
    s.take_dmg_err()
}
