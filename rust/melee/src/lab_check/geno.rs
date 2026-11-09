//! lab-check 7: the Geno lab commands on the engine (gd-lab-v1.1): each op reaches its game function through
//! `EngineGame` and the `gd_geno_*` view publishes what the game then answers. Frozen (paused) ticks publish without
//! stepping, so a teleport's position is read back exactly.

use super::*;
use std::cell::RefCell;
use std::rc::Rc;
use melee_modhost::pres::Session;

/// gd_lab_fighter_v1's pos.x / pos.y (words 14, 15).
fn pos(sh: &Rc<RefCell<LabShared>>, port: usize) -> Option<(f32, f32)> {
    let f = sh.borrow().fighters[port]?;
    let w = |i: usize| f32::from_le_bytes(f[i * 4..i * 4 + 4].try_into().unwrap());
    Some((w(14), w(15)))
}

pub(super) fn geno_checks(lab: &mut LabRun, sh: &Rc<RefCell<LabShared>>, check: &mut dyn FnMut(bool, String)) {
    let c = LabCmd::new;
    let frozen = |lab: &mut LabRun, cmds: &[LabCmd]| {
        let p = script_pads(lab);
        lab.tick(p, cmds);
    };
    let result = || sh.borrow().geno.result.clone();
    lab.apply(&c(OP_PAUSE, 0, 1, 0, 0, 0));
    {
        let s = sh.borrow();
        eprintln!(
            "lab-check: info geno: profiles {:?}, {} profile table(s), {} define name(s), combo P1 {}",
            s.geno.profile, s.geno.states.len(), s.geno.defines.iter().flatten().count(), s.geno.combo[0][0]
        );
    }
    // fly on, then a teleport while frozen: the published position is the target
    frozen(lab, &[c(OP_GENO_FLY, 0, 1, 0, 0, 0)]);
    let r = result();
    let mode = sh.borrow().geno.fly_mode[0];
    check(r.seq == 1 && r.op == OP_GENO_FLY && r.rc >= 0 && mode == 1, format!("geno fly on (GenoFly_Set): rc {} {:?}, GenoFly_Get {mode}", r.rc, r.text));
    frozen(lab, &[c(OP_GENO_TELEPORT, 0, 0, 12.5f32.to_bits(), 40.0f32.to_bits(), 0)]);
    let r = result();
    let p = pos(sh, 0);
    check(r.rc == 0 && p == Some((12.5, 40.0)), format!("geno teleport (GenoFly_Teleport) to 12.5,40: rc {}, published position {p:?}", r.rc));
    // the fly cursor's attack: on (with every parameter), then the clear
    let atk = geno_attack_cmd(0, 1, 3, 6.0, 4, 20, 0, 0, 0).unwrap();
    frozen(lab, &[atk]);
    let (r, f) = (result(), sh.borrow().geno.fly[0]);
    check(r.rc == 0 && f[0] == 1 && f[2] == 3 && f32::from_bits(f[3] as u32) == 6.0 && f[8] == 4 && f[9] == 20, format!("geno fly attack (GenoFly_AttackConfig): rc {}, GenoFly_State attack {} damage {} radius {} active {} gap {}", r.rc, f[0], f[2], f32::from_bits(f[3] as u32), f[8], f[9]));
    frozen(lab, &[c(OP_GENO_FLY_CLEAR, 0, 0, 0, 0, 0)]);
    let f = sh.borrow().geno.fly[0];
    check(f[0] == 0 && f[1] == 0, format!("geno fly clear (GenoFly_Clear): attack {} target {}", f[0], f[1]));
    // ki on a fighter without a combo graph: the game refuses
    frozen(lab, &[c(OP_GENO_KI, 0, 0, 500, 0, 0)]);
    let r = result();
    let on = sh.borrow().geno.combo[0][0] >= 0;
    check(if on { r.rc >= 0 } else { r.rc == -1 && !r.text.is_empty() }, format!("geno combo ki (GenoGame_ComboSetKi), combo graph {on}: rc {} {:?}", r.rc, r.text));
    // online the op is ignored on both sides (never applied, no result)
    let seq = result().seq;
    sh.borrow_mut().set_session(Session::LabNetplay);
    lab.apply(&c(OP_GENO_FLY, 0, 0, 0, 0, 0));
    sh.borrow_mut().set_session(Session::Offline);
    check(result().seq == seq && sh.borrow().geno.fly_mode[0] == 1, "geno ops are ignored in LabNetplay".into());
    // fly off, then the hot reload (no mods mounted: gw_Geno_Reload re-reads nothing, the fighters are re-applied)
    frozen(lab, &[c(OP_GENO_FLY, 0, 0, 0, 0, 0), c(OP_GENO_RELOAD, 0, 0, 0, 0, 0)]);
    let r = result();
    let mode = sh.borrow().geno.fly_mode[0];
    check(r.op == OP_GENO_RELOAD && !r.text.is_empty() && mode == 0, format!("geno reload (gw_Geno_Reload): rc {} {:?}; fly off: GenoFly_Get {mode}", r.rc, r.text));
    lab.apply(&c(OP_PAUSE, 0, 0, 0, 0, 0));
    let h = run(lab, 60, &[]);
    check(h.iter().all(|&x| x != 0), "the match runs on after the Geno commands".into());
}
