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

fn sim<'a>() -> &'a mut VsMatch {
    let p = SIM.with(|c| c.get());
    assert!(!p.is_null(), "a damage hook ran outside VsMatch::with_sim");
    unsafe { &mut *p }
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

    fn dmg_fail(&mut self, why: impl Into<String>) {
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
