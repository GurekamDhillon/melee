//! Taking a hit with knockback: `Fighter_ProcessHit_8006D1EC`'s knockback branch (s_link 14) through the
//! ported `ftCo_8008EC90` -> `ftCo_8008DCE0` (`ftCo_Damage.c`), run on the player's stand-in GObj (`Player::gobj`,
//! `user_data` -> the Fighter), and the damage states themselves (`ftCo_Damage_*`, `ftCo_DamageFly_*`).
//!
//! The ported TU reaches the rest of the game through `ext::hook::*` function pointers with no context, so the
//! calls that need the match (`Fighter_ChangeMotionState`, `ftAnim_8006EBA4`) are routed here through a
//! thread-local pointer to the running [`VsMatch`] ([`with_sim`]); a hook that cannot proceed records its
//! reason in `VsMatch::dmg_err`, which the caller turns into the run's stop reason.

use super::*;
use melee_fighter::hooks as h;
use melee_fighter::tu::ftCo_Damage as dm;
use melee_types::HSD_GObj;
use std::cell::Cell;

thread_local! {
    static SIM: Cell<*mut VsMatch> = const { Cell::new(std::ptr::null_mut()) };
}

pub(super) fn sim<'a>() -> &'a mut VsMatch {
    let p = SIM.with(|c| c.get());
    assert!(!p.is_null(), "a damage hook ran outside VsMatch::with_sim");
    unsafe { &mut *p }
}

/// Makes `s` the match the ported code's hooks reach (set at the start of every step).
pub(super) fn set_sim(s: *mut VsMatch) {
    SIM.with(|c| c.set(s));
}

impl VsMatch {
    /// Runs `f` with this match reachable from the damage hooks.
    pub(super) fn with_sim<R>(&mut self, f: impl FnOnce(&mut VsMatch) -> R) -> R {
        let prev = SIM.with(|c| c.replace(self as *mut VsMatch));
        let r = f(self);
        SIM.with(|c| c.set(prev));
        r
    }

    /// The player whose stand-in GObj is `g`.
    fn player_of(&self, g: *mut HSD_GObj) -> usize {
        self.players.iter().position(|p| &*p.gobj as *const HSD_GObj == g as *const HSD_GObj).expect("a GObj that is not a player's")
    }

    pub(super) fn dmg_fail(&mut self, why: impl Into<String>) {
        if self.dmg_err.is_none() {
            self.dmg_err = Some(why.into());
        }
    }

    pub(super) fn take_dmg_err(&mut self) -> Result<(), String> {
        match self.dmg_err.take() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

/// The submotion (`ftCo_SM_*`, the animation row) of a common motion state.
pub(super) fn common_smid(msid: i32) -> Result<u32, String> {
    match msid {
        // ftCo_MS_DamageHi1 .. DamageFlyRoll -> ftCo_SM_DamageHi1 (165) ..
        75..=91 => Ok(165 + (msid - 75) as u32),
        _ => Err(format!("the animation row of motion {msid} is not mapped")),
    }
}

/// Installs the hooks the ported damage code calls through (once per process).
pub(super) fn install_hooks() {
    install_state_hooks();
    unsafe {
        // Without `--audio` the sound calls (start / stop by handle) are no-ops with an invalid handle.
        use melee_fighter::hooks as h;
        if h::LBAUDIOAX_800237A8.is_none() {
            h::LBAUDIOAX_800237A8 = Some(|_, _, _| -1);
        }
        if h::LBAUDIOAX_80023870.is_none() {
            h::LBAUDIOAX_80023870 = Some(|_, _, _, _| -1);
        }
        if h::LBAUDIOAX_80024B94.is_none() {
            h::LBAUDIOAX_80024B94 = Some(|_, _| 0);
        }
        // Armor, script-game hit rules and Geno: not part of a vanilla match.
        dm::ext::hook::ScriptGame_ArmorResetReaction = Some(|_| ());
        dm::ext::hook::ScriptGame_ArmorSubtract = Some(|_, kb| kb);
        dm::ext::hook::ScriptGame_ArmorReact = Some(|_| 0);
        dm::ext::hook::ScriptGame_CapsNoFlinch = Some(|_| 0);
        dm::ext::hook::ScriptGame_ShockTake = Some(|_, _| 0);
        dm::ext::hook::Geno_TakeStunBonus = Some(|_| 0);
        // Colour animations and the background flash are render only.
        dm::ext::hook::ftCo_800BFFD0 = Some(|_, _, _| false);
        dm::ext::hook::lbBgFlash_80021C48 = Some(|_, _| ());
        // gm_8016B014: the "auto-hurt in training" rule is off in VS.
        dm::ext::hook::gm_8016B014 = Some(|| false);
        dm::ext::hook::Camera_RequestQuake = Some(|_, _| sim().dmg_fail("Camera_RequestQuake (an airborne level 3 hit) is not ported"));
        dm::ext::hook::efAsync_Spawn = Some(|_, _, _, _, _, _| sim().dmg_fail("efAsync_Spawn from the damage code is not ported"));
        dm::ext::hook::Fighter_ChangeMotionState = Some(|g, msid, flags, start, speed, blend, other| {
            let s = sim();
            if !other.is_null() {
                s.dmg_fail("Fighter_ChangeMotionState with a thrower is not ported");
                return;
            }
            let pi = s.player_of(g);
            let r = (|| -> Result<(), String> {
                let smid = common_smid(msid)?;
                let p = &mut s.players[pi];
                let row = load_row(p, smid)?;
                if blend != 0.0 {
                    return Err("Fighter_ChangeMotionState with an explicit blend is not ported".into());
                }
                change_motion_state_flags(p, msid, row.as_ref(), start, speed, flags, &mut s.rand)
            })();
            if let Err(e) = r {
                s.dmg_fail(e);
            }
        });
        // Fighter_8006CDA4: only a held item (or a head item) can be dropped; neither exists here.
        dm::ext::hook::un_8032233C = Some(|_, _| ());  // the crowd's reaction is sound only
        h::MEX_ON_KNOCKBACK_ENTER = Some(|kind, g| knockback_face(kind, g, 3));
        h::MEX_ON_KNOCKBACK_EXIT = Some(|kind, g| knockback_face(kind, g, 2));
        dm::ext::hook::Fighter_8006CDA4 = Some(|fp, _| {
            if !fp.item_gobj.is_null() || !fp.x197C.is_null() {
                sim().dmg_fail("Fighter_8006CDA4: a fighter holding an item is not ported");
            }
        });
        dm::ext::hook::ftAnim_8006EBA4 = Some(|g| {
            let s = sim();
            let pi = s.player_of(g);
            if let Err(e) = anim_step(&mut s.players[pi], &mut s.rand) {
                s.dmg_fail(e);
            }
        });
    }
}

/// `ftData_OnKnockbackEnter/Exit[kind]` (`ftFx_Init_OnKnockbackEnter`: `Fighter_OnKnockbackEnter(gobj, 1)`, the face
/// texture animation, then `ftAnim_ApplyPartAnim(gobj, 3, anim, 0)` and `(gobj, 4, anim, 0)` on the hands).
fn knockback_face(kind: i32, g: *mut HSD_GObj, anim: i32) {
    let s = sim();
    let pi = s.player_of(g);
    if !is_fx_kind(kind as u32) {
        s.dmg_fail(format!("ftData_OnKnockback{{Enter,Exit}} of kind {kind} is not ported"));
        return;
    }
    let p = &mut s.players[pi];
    // ftAnim_800704F0: the face texture animation (render only) and its flag.
    p.fp.set_x221E_b7(1);
    for slot in [3usize, 4] {
        let Some(pd) = p.part_anim[slot].clone() else {
            s.dmg_fail("ftData->x1C hand slot missing");
            return;
        };
        let pdat = p.pdat.clone();
        if let Err(e) = p.skel.apply_part_anim(&pdat, &pd, slot, anim, 0.0) {
            s.dmg_fail(e);
            return;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Proc {
    Anim,
    Iasa,
    Phys,
    Coll,
}

impl VsMatch {
    /// `Fighter_8006A1BC` (s_link 0): the hitlag countdown.
    pub(super) fn hitlag_countdown(&mut self) -> Result<(), String> {
        for pi in 0..self.players.len() {
            let g: *mut HSD_GObj = &mut *self.players[pi].gobj;
            let fp = &mut self.players[pi].fp;
            if fp.x221F_b3() != 0 {
                continue;
            }
            if fp.dmg.x1954 > 0.0 {
                return Err("Fighter_8006A1BC: x1954 (a delayed hitlag exit) is not ported".into());
            }
            if fp.x221A_b1() != 0 {
                return Err("Fighter_8006A1BC: x221A_b1 is not ported".into());
            }
            if fp.dmg.x195c_hitlag_frames > 0.0 {
                fp.dmg.x195c_hitlag_frames -= 1.0;
                if fp.dmg.x195c_hitlag_frames <= 0.0 {
                    fp.dmg.x195c_hitlag_frames = 0.0;
                    if fp.x221A_b3() != 0 {
                        melee_fighter::ftdmgfall::ftco_80090718(fp);
                        fp.set_x221A_b3(0);
                    }
                    if fp.dmg.x1954 == 0.0 && fp.x2219_b7() == 0 {
                        // Fighter_8006D10C: post_hitlag_cb, then the hitlag flag off.
                        if let Some(cb) = fp.post_hitlag_cb {
                            self.with_sim(|_| unsafe { cb(g) });
                            self.take_dmg_err()?;
                        }
                        let fp = &mut self.players[pi].fp;
                        fp.set_x2219_b5(0);
                        if !fp.x1A5C.is_null() {
                            return Err("Fighter_8006D10C: a partner (x1A5C) is not ported".into());
                        }
                    }
                    self.players[pi].fp.set_allow_sdi(0);
                }
            }
            let fp = &mut self.players[pi].fp;
            if fp.dmg.x18F4 != 0 || fp.x200C != 0 || fp.x2010 != 0 {
                return Err("Fighter_8006A1BC: x18F4 / a mushroom queue is not ported".into());
            }
            let b5 = fp.x2219_b5();
            fp.set_x2219_b6(b5);
        }
        Ok(())
    }

    /// One state callback (anim, input, phys or coll) of the damage states, run by the ported
    /// `ftCo_Damage*` code on the player's stand-in GObj.
    pub(super) fn damage_proc(&mut self, pi: usize, which: Proc) -> Result<(), String> {
        let g: *mut HSD_GObj = &mut *self.players[pi].gobj;
        let m = self.players[pi].fp.motion_id as i32;
        self.with_sim(|_| match (which, m) {
            (Proc::Anim, 75..=86) => dm::ftCo_Damage_Anim(g),
            (Proc::Anim, 87..=90) => dm::ftCo_DamageFly_Anim(g),
            (Proc::Anim, _) => dm::ftCo_DamageFlyRoll_Anim(g),
            (Proc::Iasa, 75..=86) => dm::ftCo_Damage_IASA(g),
            (Proc::Iasa, 87..=90) => dm::ftCo_DamageFly_IASA(g),
            (Proc::Iasa, _) => dm::ftCo_DamageFlyRoll_IASA(g),
            (Proc::Phys, 75..=86) => dm::ftCo_Damage_Phys(g),
            (Proc::Phys, 87..=90) => dm::ftCo_DamageFly_Phys(g),
            (Proc::Phys, _) => dm::ftCo_DamageFlyRoll_Phys(g),
            (Proc::Coll, 75..=86) => dm::ftCo_Damage_Coll(g),
            (Proc::Coll, 87..=90) => dm::ftCo_DamageFly_Coll(g),
            (Proc::Coll, _) => dm::ftCo_DamageFlyRoll_Coll(g),
        });
        self.take_dmg_err()
    }
}

/// The state transitions and the input / physics / collision leaves of the damage states.
pub(super) fn install_state_hooks() {
    unsafe {
        dm::ext::hook::ftAnim_IsFramesRemaining = Some(|g| {
            let s = sim();
            let pi = s.player_of(g);
            s.players[pi].skel.frames_remaining()
        });
        dm::ext::hook::ft_8008A2BC = Some(|g| {
            let s = sim();
            let pi = s.player_of(g);
            if let Err(e) = ft_8008a2bc(&mut s.players[pi], &mut s.rand) {
                s.dmg_fail(e);
            }
        });
        dm::ext::hook::ftCo_Fall_Enter = Some(|g| {
            let s = sim();
            let pi = s.player_of(g);
            if let Err(e) = fall_enter(&mut s.players[pi], &mut s.rand) {
                s.dmg_fail(e);
            }
        });
        dm::ext::hook::ftCo_Wait_IASA = Some(|g| {
            let s = sim();
            let pi = s.player_of(g);
            if let Err(e) = wait_iasa(&mut s.players[pi], &mut s.rand) {
                s.dmg_fail(e);
            }
        });
        dm::ext::hook::ftCo_Jump_GetInput = Some(|g| {
            let s = sim();
            let pi = s.player_of(g);
            jump_input_kind(&s.players[pi].fp) as u32
        });
        dm::ext::hook::ft_80084F3C = Some(|g| {
            // ft_80084F3C: the ground friction, then the self velocity along the floor.
            let s = sim();
            let pi = s.player_of(g);
            let f = &mut *s.players[pi].fp;
            let mut friction = f.co_attrs.ground_friction;
            if fc::abs_c(f.gr_vel) > f.co_attrs.walk_max_vel {
                friction *= melee_fighter::hooks::common_data().friction_when_above_walk_speed;
            }
            fc::calc_ground_accel_deaccel(f, friction);
            fc::set_self_movement_from_grounded_movement(f);
        });
        dm::ext::hook::ft_800848DC = Some(|g, _cb| {
            // ft_800848DC(gobj, ftCo_8008FC94): mpColl_8004B108; off the floor either the ledge slip or the
            // callback (ftCommon_8007D5D4).
            let s = sim();
            let pi = s.player_of(g);
            let r = (|| -> Result<(), String> {
                let p = &mut s.players[pi];
                let joints = {
                    let d = p.ecb_desc;
                    ecb_joints(&mut p.skel, &d)
                };
                let fp = &mut *p.fp;
                fp.coll_data.last_pos = fp.coll_data.cur_pos;
                fp.coll_data.cur_pos = fp.cur_pos;
                let on = mc::mp_coll_8004b108(&mut s.ground, &mut s.cg, &mut fp.coll_data, &joints).map_err(|e| format!("{} not ported", e.0))?;
                fp.cur_pos = fp.coll_data.cur_pos;
                if !on {
                    let env = fp.coll_data.env_flags;
                    if (fp.facing_dir < 0.0 && env & mc::COLLIDE_RIGHT_LEDGE_SLIP != 0) || (fp.facing_dir > 0.0 && env & mc::COLLIDE_LEFT_LEDGE_SLIP != 0) {
                        return Err("ftCo_8009F39C (a ledge slip while hit) is not ported".into());
                    }
                    melee_fighter::ftcommon2::ftcommon_8007d5d4(fp);
                }
                Ok(())
            })();
            if let Err(e) = r {
                s.dmg_fail(e);
            }
        });
        // The airborne leaves.
        dm::ext::hook::ft_80084DB0 = Some(|_| sim().dmg_fail("ft_80084DB0 (a hit fighter in the air) is not ported"));
        dm::ext::hook::ft_80084EEC = Some(|_| sim().dmg_fail("ft_80084EEC (a hit fighter in the air) is not ported"));
        dm::ext::hook::ft_80081DD4 = Some(|_| {
            sim().dmg_fail("ft_80081DD4 (a hit fighter in the air) is not ported");
            false
        });
        dm::ext::hook::ftCo_80090780 = Some(|_| sim().dmg_fail("ftCo_80090780 (DamageFall) is not ported"));
        dm::ext::hook::ftCo_Fall_IASA_Inner = Some(|_| {
            sim().dmg_fail("ftCo_Fall_IASA_Inner from the damage states is not ported");
            false
        });
        dm::ext::hook::ftCo_800CB870 = Some(|_| {
            sim().dmg_fail("ftCo_800CB870 from the damage states is not ported");
            false
        });
        dm::ext::hook::ftCo_800CB8E0 = Some(|_| {
            sim().dmg_fail("ftCo_800CB8E0 is not ported");
            false
        });
        dm::ext::hook::ftCo_800D69C4 = Some(|_| {
            sim().dmg_fail("ftCo_800D69C4 is not ported");
            false
        });
        melee_fighter::tu::ftcoll::ext::FTCO_800BFFD0 = Some(|_, _, _| false);
    }
}
