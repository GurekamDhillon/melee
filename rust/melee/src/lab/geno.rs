//! The Lab's Geno side (gd-lab-v1.1, docs/rust-port/notes/geno-wasm-plan.md step 4): what the Geno lab commands do to
//! the match and what the `gd_geno_*` queries read. Every call goes to the game half through `ScriptGame` (the C's
//! gw_ScriptGame_* / gw_Geno_* / gw_GenoFly_*: `melee_engine::script_host::EngineGame` in the play flow), so no Geno
//! logic lives here; a test hands in a recording game instead.
//!
//! The C LAB (pc/platform/gw_script.c, gw_script_fly_attack.inc, gw_script_combo.inc) is the reference for the
//! sequencing: an attack switched on puts the fighter in flight first (gs_fly_attack_cfg), refusals read as gs_fly_err
//! says them, and the hot reload's three outcomes are gs_hot_do's (refused: data kept; layout changed: the LAB match
//! restarts; else the fighters are re-applied).

use melee_engine::script_host::ScriptGame;
use melee_modhost::lab::*;

/// What a Geno command did, for the play flow.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Applied {
    pub rc: i32,
    pub text: String,
    /// The command wrote gameplay state (the C's gs_rw_branch): a running lookahead is dropped.
    pub fork: bool,
    /// A reload installed new registry data: the history before it is another timeline's, and the tables republish.
    pub reloaded: bool,
    /// The layout changed but the match could not be restarted: the play flow freezes it (gs_hot_do pauses).
    pub pause: bool,
}

/// gs_fly_err
fn fly_err(rc: i32) -> &'static str {
    match rc {
        -1 => "no fighter on that port",
        -2 => "that fighter's state cannot fly (dead, held, respawning)",
        _ => "",
    }
}

fn fly(rc: i32) -> Applied {
    Applied { rc, text: fly_err(rc).into(), fork: true, ..Default::default() }
}

/// Apply one Geno command (None: not a Geno opcode).
pub fn apply(g: &mut dyn ScriptGame, c: &LabCmd) -> Option<Applied> {
    let slot = (c.port & 3) as i32;
    Some(match c.op {
        OP_GENO_FLY => fly(g.geno_fly_set(slot, c.a as i32)),
        OP_GENO_TELEPORT => fly(g.geno_fly_teleport(slot, c.b as i32, c.c as i32)),
        OP_GENO_FLY_ATTACK => {
            let (flags, damage, radius, active_gap, kb_kbg, angle) = geno_attack_args(c);
            if flags & 1 != 0 {
                let rc = g.geno_fly_set(slot, 1);
                if rc < 0 {
                    return Some(fly(rc));
                }
            }
            let rc = g.geno_fly_attack_config(slot, flags, damage, radius, active_gap, kb_kbg, angle);
            let mut a = fly(rc);
            if rc == -2 {
                a.text = "invalid cursor parameters".into();
            }
            a
        }
        OP_GENO_FLY_CLEAR => {
            g.geno_fly_clear(slot);
            fly(0)
        }
        OP_GENO_KI => {
            let rc = g.combo_set_ki(slot, c.b as i32);
            Applied { rc, text: if rc < 0 { "no combo meter on that port".into() } else { String::new() }, fork: true, ..Default::default() }
        }
        OP_GENO_RELOAD => reload(g),
        _ => return None,
    })
}

/// gs_hot_do without the C's rewind-and-replay to the request frame (the wasm Lab applies it on its command frame).
fn reload(g: &mut dyn ScriptGame) -> Applied {
    let mut msg = String::new();
    let safe = g.geno_reload(&mut msg);
    if safe < 0 {
        let why = if msg.is_empty() { "geno.json could not be loaded" } else { msg.as_str() };
        return Applied { rc: safe, text: format!("{why}; reload refused, previous data retained"), ..Default::default() };
    }
    if safe == 0 {
        let mut text = format!("{msg} - the state no longer fits the fighters' layout: restarting the match");
        let left = g.geno_lab_leave(3);
        if !left {
            text.push_str("; not a LAB match - restart it yourself");
        }
        return Applied { rc: 0, text, fork: true, reloaded: true, pause: !left };
    }
    let n = g.geno_lab_reload();
    let what = if msg.is_empty() { "geno.json unchanged" } else { msg.as_str() };
    Applied { rc: safe, text: format!("{what}; {n} fighter(s) re-applied"), fork: true, reloaded: true, pause: false }
}

/// The registry tables the name queries read: every profile's state and route names, every CSS kind's define name.
/// `routes(p)` is the profile's route count (gw_Geno_ComboParam(p, GENO_CP_ROUTES)).
pub fn tables(g: &dyn ScriptGame, v: &mut GenoView, profiles: i32, routes: &dyn Fn(i32) -> i32, kinds: i32) {
    v.states = (0..profiles.max(0)).map(|p| (0..g.geno_state_count(p).max(0)).map(|s| g.geno_state_name(p, s).unwrap_or_default()).collect()).collect();
    v.routes = (0..profiles.max(0)).map(|p| (0..routes(p).clamp(0, GENO_ROUTES_MAX as i32)).map(|r| g.combo_name(p, r)).collect()).collect();
    v.defines = (0..=kinds.max(0)).map(|ck| g.geno_define_name(ck)).collect();
    v.tables_gen = v.tables_gen.wrapping_add(1);
}

/// One port's per-frame part, `kind` the fighter's kind (None: no fighter on the port). Reads only.
pub fn publish_port(g: &dyn ScriptGame, v: &mut GenoView, port: usize, kind: Option<i32>) {
    let Some(kind) = kind else {
        v.clear_port(port);
        return;
    };
    let slot = port as i32;
    v.profile[port] = g.geno_profile_for_kind(kind);
    let cb = &mut v.combo[port];
    cb[0] = g.combo_info(slot, 0);
    if cb[0] < 0 {
        // no combo graph on this fighter: every code answers -1 (GenoGame_ComboInfo's geno_combo_on test)
        cb.fill(-1);
    } else {
        for w in 1..16 {
            cb[w] = g.combo_info(slot, w as i32);
        }
        let n = cb[3].clamp(0, GENO_ROUTES_MAX as i32) as usize;
        for r in 0..GENO_ROUTES_MAX {
            cb[16 + r] = if r < n { g.combo_info(slot, 16 + r as i32) } else { -1 };
        }
    }
    v.fly_mode[port] = g.geno_fly_get(slot);
    for f in 0..GENO_FLY_FIELDS {
        v.fly[port][f] = g.geno_fly_state(slot, f as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// A game that records the Geno calls it gets (the C's script_policy_test.c fixture idea).
    #[derive(Default)]
    struct G {
        calls: RefCell<Vec<String>>,
        reload_rc: i32,
        leave_ok: bool,
        fly_rc: i32,
    }
    impl G {
        fn log(&self, s: String) {
            self.calls.borrow_mut().push(s);
        }
        fn take(&self) -> Vec<String> {
            std::mem::take(&mut *self.calls.borrow_mut())
        }
    }
    impl ScriptGame for G {
        fn geno_fly_set(&mut self, slot: i32, mode: i32) -> i32 {
            self.log(format!("fly_set {slot} {mode}"));
            self.fly_rc
        }
        fn geno_fly_teleport(&mut self, slot: i32, x: i32, y: i32) -> i32 {
            self.log(format!("teleport {slot} {} {}", f32::from_bits(x as u32), f32::from_bits(y as u32)));
            0
        }
        fn geno_fly_attack_config(&mut self, slot: i32, flags: i32, damage: i32, radius_bits: i32, active_gap: i32, kb_kbg: i32, angle: i32) -> i32 {
            self.log(format!("attack {slot} {flags} {damage} {} {active_gap:#x} {kb_kbg:#x} {angle}", f32::from_bits(radius_bits as u32)));
            0
        }
        fn geno_fly_clear(&mut self, slot: i32) {
            self.log(format!("clear {slot}"));
        }
        fn combo_set_ki(&mut self, slot: i32, ki: i32) -> i32 {
            self.log(format!("ki {slot} {ki}"));
            if slot == 3 { -1 } else { ki }
        }
        fn geno_reload(&mut self, msg: &mut String) -> i32 {
            self.log("reload".into());
            *msg = "fixture reload".into();
            self.reload_rc
        }
        fn geno_lab_reload(&mut self) -> i32 {
            self.log("lab_reload".into());
            2
        }
        fn geno_lab_leave(&mut self, w: i32) -> bool {
            self.log(format!("leave {w}"));
            self.leave_ok
        }
        fn geno_profile_for_kind(&self, kind: i32) -> i32 {
            if kind == 9 { 1 } else { -1 }
        }
        fn combo_info(&self, slot: i32, what: i32) -> i32 {
            self.log(format!("info {slot} {what}"));
            match (slot, what) {
                (1, _) => -1,
                (_, 3) => 2,
                (_, w) => w * 10,
            }
        }
        fn geno_state_count(&self, p: i32) -> i32 {
            [1, 2][p as usize]
        }
        fn geno_state_name(&self, p: i32, s: i32) -> Option<String> {
            Some(format!("S{p}.{s}"))
        }
        fn combo_name(&self, p: i32, r: i32) -> String {
            format!("R{p}.{r}")
        }
        fn geno_define_name(&self, ck: i32) -> Option<String> {
            (ck == 35).then(|| "Def".into())
        }
        fn geno_fly_get(&self, slot: i32) -> i32 {
            slot
        }
        fn geno_fly_state(&self, _slot: i32, field: i32) -> i32 {
            field + 100
        }
    }

    #[test]
    fn commands_reach_the_game_functions() {
        let mut g = G::default();
        let a = apply(&mut g, &LabCmd::new(OP_GENO_FLY, 2, 1, 0, 0, 0)).unwrap();
        assert_eq!((g.take(), a.rc, a.fork), (vec!["fly_set 2 1".to_string()], 0, true));
        apply(&mut g, &LabCmd::new(OP_GENO_TELEPORT, 1, 0, 12.5f32.to_bits(), (-3.0f32).to_bits(), 0)).unwrap();
        assert_eq!(g.take(), ["teleport 1 12.5 -3"]);
        let c = geno_attack_cmd(0, 3, 7, 6.5, 4, 20, 50, 70, 361).unwrap();
        apply(&mut g, &c).unwrap();
        assert_eq!(g.take(), ["fly_set 0 1", "attack 0 3 7 6.5 0x140004 0x460032 361"], "on: into flight first, then the config");
        apply(&mut g, &geno_attack_cmd(0, 0, 0, 0.0, 0, 0, 0, 0, 0).unwrap()).unwrap();
        assert_eq!(g.take(), ["attack 0 0 0 0 0x0 0x0 0"], "off: no flight change");
        apply(&mut g, &LabCmd::new(OP_GENO_FLY_CLEAR, 3, 0, 0, 0, 0)).unwrap();
        assert_eq!(g.take(), ["clear 3"]);
        let a = apply(&mut g, &LabCmd::new(OP_GENO_KI, 0, 0, 2500, 0, 0)).unwrap();
        assert_eq!((g.take(), a.rc), (vec!["ki 0 2500".to_string()], 2500));
        let a = apply(&mut g, &LabCmd::new(OP_GENO_KI, 3, 0, 1, 0, 0)).unwrap();
        assert_eq!((a.rc, a.text.as_str()), (-1, "no combo meter on that port"));
        assert!(apply(&mut g, &LabCmd::new(OP_PAUSE, 0, 1, 0, 0, 0)).is_none(), "not a Geno op");
    }

    #[test]
    fn refusals_read_as_the_c_lab() {
        let mut g = G { fly_rc: -2, ..Default::default() };
        let a = apply(&mut g, &LabCmd::new(OP_GENO_FLY, 0, 1, 0, 0, 0)).unwrap();
        assert_eq!((a.rc, a.text.as_str()), (-2, "that fighter's state cannot fly (dead, held, respawning)"));
        let a = apply(&mut g, &geno_attack_cmd(0, 1, 3, 6.0, 0, 0, 0, 0, 0).unwrap()).unwrap();
        assert_eq!((g.take().len(), a.rc), (2, -2), "the flight refusal stops the attack");
    }

    #[test]
    fn reload_outcomes_like_gs_hot_do() {
        let mut g = G { reload_rc: -1, ..Default::default() };
        let a = apply(&mut g, &LabCmd::new(OP_GENO_RELOAD, 0, 0, 0, 0, 0)).unwrap();
        assert_eq!(g.take(), ["reload"]);
        assert_eq!((a.rc, a.text.as_str(), a.reloaded, a.fork), (-1, "fixture reload; reload refused, previous data retained", false, false));
        g.reload_rc = 1;
        let a = apply(&mut g, &LabCmd::new(OP_GENO_RELOAD, 0, 0, 0, 0, 0)).unwrap();
        assert_eq!(g.take(), ["reload", "lab_reload"]);
        assert_eq!((a.rc, a.text.as_str(), a.reloaded, a.pause), (1, "fixture reload; 2 fighter(s) re-applied", true, false));
        g.reload_rc = 0;
        g.leave_ok = true;
        let a = apply(&mut g, &LabCmd::new(OP_GENO_RELOAD, 0, 0, 0, 0, 0)).unwrap();
        assert_eq!(g.take(), ["reload", "leave 3"]);
        assert!(a.text.ends_with("restarting the match") && !a.pause, "{}", a.text);
        g.leave_ok = false;
        let a = apply(&mut g, &LabCmd::new(OP_GENO_RELOAD, 0, 0, 0, 0, 0)).unwrap();
        assert!(a.text.ends_with("restart it yourself") && a.pause, "{}", a.text);
    }

    /// A Lab mod (UI_DRAW + LAB + LAB_CPU, gd-lab-v1.1) whose frame reads two queries and queues two commands built
    /// from their answers: P(profile of port 0) flies, port 0's ki is set to 100 x the length of state 1's name.
    const E2E_WAT: &str = r#"(module
      (import "gd_lab" "gd_geno_profile" (func $profile (param i32) (result i32)))
      (import "gd_lab" "gd_geno_state_name" (func $state (param i32 i32 i32 i32) (result i32)))
      (import "gd_lab" "gd_geno_fly" (func $fly (param i32 i32) (result i32)))
      (import "gd_lab" "gd_geno_combo_ki" (func $ki (param i32 i32) (result i32)))
      (memory (export "memory") 1 1)
      (global (export "gd_mod_manifest") i32 (i32.const 256))
      (global (export "gdmod_ui_abi") i32 (i32.const 0x10000))
      (global (export "gdmod_lab_abi") i32 (i32.const 0x10001))
      (data (i32.const 256) "GDMD\7c\00\00\00\00\00\01\00\00\00\01\00\01\00\00\00\00\00\01\00\10\24\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00lab.geno")
      (func (export "gd_ui_frame") (param i32) (local $p i32)
        (local.set $p (call $profile (i32.const 0)))
        (drop (call $fly (local.get $p) (i32.const 1)))
        (drop (call $ki (i32.const 0) (i32.mul (call $state (local.get $p) (i32.const 1) (i32.const 1024) (i32.const 64)) (i32.const 100)))))
    )"#;

    #[test]
    fn wasm_query_then_command_reaches_the_game() {
        let mut h = melee_modhost::ui::PresHost::new();
        let sh = h.install_lab();
        h.load(&wat::parse_str(E2E_WAT).unwrap(), std::env::temp_dir()).unwrap();
        // the play flow's side: a lab run is active and has published this frame's view
        let mut g = G::default();
        {
            let mut s = sh.borrow_mut();
            s.active = true;
            s.apply_frame = 7;
            tables(&g, &mut s.geno, 2, &|_| 0, 0);
            publish_port(&g, &mut s.geno, 0, Some(9));
        }
        g.take();
        h.present(&melee_modhost::pres::PresFrame { session: melee_modhost::pres::Session::Offline, scene: (2, 2), match_frame: 0 });
        let cmds = std::mem::take(&mut sh.borrow_mut().outbox);
        assert_eq!(cmds, [LabCmd::new(OP_GENO_FLY, 1, 1, 0, 0, 0), LabCmd::new(OP_GENO_KI, 0, 0, 400, 0, 0)], "profile 1, \"S1.1\" is 4 bytes");
        // the commands as the lockstep / offline drain hands them over, then applied
        for c in &cmds {
            let c = LabCmd::from_bytes(&c.to_bytes()).unwrap();
            apply(&mut g, &c).unwrap();
        }
        assert_eq!(g.take(), ["fly_set 1 1", "ki 0 400"]);
    }

    #[test]
    fn publish_reads_the_game() {
        let g = G::default();
        let mut v = GenoView::default();
        tables(&g, &mut v, 2, &|p| p + 1, 40);
        assert_eq!(v.states, vec![vec!["S0.0".to_string()], vec!["S1.0".to_string(), "S1.1".to_string()]]);
        assert_eq!(v.route_name(1, 1), Some("R1.1"));
        assert_eq!(v.route_name(0, 1), None);
        assert_eq!((v.define_name(35), v.define_name(34), v.defines.len()), (Some("Def"), None, 41));
        assert_eq!(v.tables_gen, 1);
        publish_port(&g, &mut v, 0, Some(9));
        assert_eq!((v.profile[0], v.combo[0][1], v.combo[0][3], v.combo[0][16], v.combo[0][17], v.combo[0][18]), (1, 10, 2, 160, 170, -1));
        assert_eq!((v.fly_mode[0], v.fly[0][13]), (0, 113));
        g.take();
        publish_port(&g, &mut v, 1, Some(2));
        assert_eq!(g.take(), ["info 1 0"], "combo off: one call");
        assert!(v.combo[1].iter().all(|&x| x == -1) && v.profile[1] == -1);
        publish_port(&g, &mut v, 0, None);
        assert_eq!((v.profile[0], v.fly_mode[0], v.combo[0][1]), (-1, -1, -1));
    }
}
