//! Hot reload of per-clip animation overrides in the offline LAB (docs/rust-port/notes/anim-overrides.md).
//!
//! Trigger: while the LAB runs offline, `poll` (called from the LAB step only) stats the override files every
//! `POLL_FRAMES` command-clock frames; when a file's size or modified time changes it re-reads them all
//! (`anim_override::reload`, all or nothing) and restarts each fighter's current motion so the new clip plays at once.
//! `MELEE_ANIM_HOTRELOAD=0` turns the poll off. `reload_now` is the engine call a LAB button can use.
//! Refused (never run) online, during replays / spectating and outside the LAB: the caller passes `Ctx` and nothing else
//! calls it. Nothing here runs on normal play paths, and with no override installed `poll` returns at its first test.

use melee_fighter::anim_override as ao;
use std::path::PathBuf;
use std::time::SystemTime;

pub const POLL_FRAMES: i32 = 30;

/// Where a reload was asked from.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    /// The LAB runs offline (no lockstep partner).
    pub offline_lab: bool,
    /// A replay plays, a spectate feed drives the match, or a synctest / hosted rollback session runs.
    pub replay_or_net: bool,
}

static mut SEEN: Vec<(PathBuf, Option<(u64, SystemTime)>)> = Vec::new();
static mut SEEN_GEN: u32 = u32::MAX;

fn stamp(p: &PathBuf) -> Option<(u64, SystemTime)> {
    let m = std::fs::metadata(p).ok()?;
    Some((m.len(), m.modified().ok()?))
}

fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("MELEE_ANIM_HOTRELOAD").map_or(true, |v| v != "0"))
}

/// The LAB step's poll (offline LAB only). Answers the reload's message when one happened.
pub fn poll(ctx: Ctx, clock: i32) -> Option<String> {
    if !ao::active() || !enabled() || !ctx.offline_lab || ctx.replay_or_net || clock % POLL_FRAMES != 0 {
        return None;
    }
    let seen = unsafe { &mut *core::ptr::addr_of_mut!(SEEN) };
    // (re)baseline when the table was installed anew (a new boot)
    if unsafe { SEEN_GEN } != ao::generation() || seen.len() != ao::entries().len() {
        *seen = ao::entries().iter().map(|e| (e.path.clone(), stamp(&e.path))).collect();
        unsafe { SEEN_GEN = ao::generation() };
        return None;
    }
    let mut changed = false;
    for (p, s) in seen.iter_mut() {
        let now = stamp(p);
        if now.is_some() && now != *s {
            *s = now;
            changed = true;
        }
    }
    if !changed {
        return None;
    }
    let msg = match reload_now(ctx) {
        Ok(m) => m,
        Err(m) => m,
    };
    unsafe { SEEN_GEN = ao::generation() };
    Some(msg)
}

/// Re-read the override files and restart every fighter's current motion. Refused outside the offline LAB.
pub fn reload_now(ctx: Ctx) -> Result<String, String> {
    if !ctx.offline_lab || ctx.replay_or_net {
        return Err("anim reload refused: only in the offline LAB (not online, in replays, spectating or synctest)".into());
    }
    if melee_data::netid::frozen() {
        return Err(format!("anim reload refused: {}", melee_data::netid::FROZEN_MSG));
    }
    if !ao::active() {
        return Err("anim reload: no animation overrides are mounted".into());
    }
    let stress: usize = std::env::var("MELEE_ANIM_RELOAD_STRESS").ok().and_then(|v| v.parse().ok()).unwrap_or(1).max(1);
    let held0 = ao::held_bytes();
    let mut changed = 0;
    for _ in 0..stress {
        changed = changed.max(ao::reload().map_err(|e| format!("{e}; reload refused, previous data kept"))?);
    }
    let restarted = unsafe { restart_all() };
    let mut msg = format!("anim reload: {} override(s), {changed} changed, {restarted} fighter(s) restarted", ao::entries().len());
    if stress > 1 {
        msg.push_str(&format!(" ({stress} reloads, held bytes {held0} -> {})", ao::held_bytes()));
    }
    Ok(msg)
}

/// Restart the current motion of every fighter on ports 1-6 (frame 0, rate 1, no blend): the motion reloads its clip,
/// so an overridden one binds the new bytes.
unsafe fn restart_all() -> usize {
    let mut n = 0;
    for slot in 0..6 {
        let fp = melee_modes::tu::script_game::script_fighter(slot);
        if fp.is_null() || (*fp).gobj.is_null() || (*fp).motion_id < 0 {
            continue;
        }
        melee_fighter::tu::fighter::Fighter_ChangeMotionState((*fp).gobj as _, (*fp).motion_id as i32, 0, 0.0, 1.0, -1.0, core::ptr::null_mut());
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refused_outside_the_offline_lab() {
        for ctx in [Ctx { offline_lab: false, replay_or_net: false }, Ctx { offline_lab: true, replay_or_net: true }, Ctx { offline_lab: false, replay_or_net: true }] {
            assert!(reload_now(ctx).unwrap_err().contains("refused"));
            assert!(poll(ctx, 0).is_none());
        }
    }
}
