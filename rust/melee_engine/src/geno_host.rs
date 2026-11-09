//! Geno's native half for the engine: the registry (melee_script::geno, ported from pc/platform/geno_registry.c)
//! behind the scalar API the game half (melee_modes::tu::geno_game) calls, `gw_Geno_*` in the C. One registry per
//! process: the boot one (every mounted mod's geno.json, set_boot_mods) or, under the test suite, one parsed from a
//! test's text (`Geno_TestInstall` / `Geno_TestRestore`).
#![allow(non_snake_case, static_mut_refs)]
use melee_script::geno::consts::*;
use melee_script::geno::registry::{Env, Profile, Registry};

/// The registry's view of the mounted mods: files under `<mods dir>/<mod id>/` (geno.json, words files, a define's Lua
/// module). No m-ex slots are installed in this engine. Tests (no mods dir) read nothing.
pub struct FsEnv;
impl Env for FsEnv {
    fn read_mod_file(&self, mod_id: &str, rel: &str) -> Option<Vec<u8>> {
        let (dir, _) = unsafe { BOOT_MODS.as_ref()? };
        if rel.contains("..") || mod_id.contains("..") {
            return None;
        }
        std::fs::read(dir.join(mod_id).join(rel.replace('\\', "/"))).ok()
    }
}
#[allow(non_upper_case_globals)]
static NoEnv: FsEnv = FsEnv;

static mut BOOT: Option<Registry> = None;
static mut SAVED: Option<Option<Registry>> = None;
static mut POOL_GEN: i32 = 1;
static mut IASA_WARN_GEN: i32 = 0;
static mut IASA_WARNED: Vec<u64> = Vec::new();

fn flush_log(r: &mut Registry) {
    for l in r.log.drain(..) {
        eprintln!("{l}");
    }
}

static mut BOOT_MODS: Option<(std::path::PathBuf, Vec<String>)> = None;

/// The mounted mods folder (empty when none): the boot registry's words files and the m-ex slots come from the host
/// (`mex_host::GenoEnv`, gw_Geno's gn_reg file reads and `gw_Mex_SlotInternal`).
fn dir_of_boot() -> std::path::PathBuf {
    unsafe { BOOT_MODS.as_ref().map(|b| b.0.clone()).unwrap_or_default() }
}

/// The mounted mods whose geno.json the boot registry reads (the folder and the active ids). Before the first query.
pub fn set_boot_mods(dir: &std::path::Path, ids: &[&str]) {
    unsafe {
        BOOT_MODS = Some((dir.to_path_buf(), ids.iter().map(|s| s.to_string()).collect()));
        BOOT = None;
        POOL_GEN += 1;
    }
}

/// The installed registry; the boot one is read on first use from the mounted mods (`set_boot_mods`), every active
/// mod's `<dir>/<id>/geno.json` in id order, as gn_reg does.
fn reg() -> &'static mut Registry {
    unsafe {
        if BOOT.is_none() {
            let mut r = Registry::default();
            if let Some((dir, ids)) = BOOT_MODS.as_ref() {
                let mut ids = ids.clone();
                ids.sort();
                for id in &ids {
                    let Ok(t) = std::fs::read(dir.join(id).join("geno.json")) else { continue };
                    if t.len() > 1 << 20 {
                        continue;
                    }
                    r.parse_text(&t, id, &crate::mex_host::GenoEnv { dir: dir.clone() });
                }
                if r.files > 0 {
                    let n = r.p.len();
                    r.log.push(format!("geno: {} geno.json file(s), {} fighter entr{}", r.files, n, if n == 1 { "y" } else { "ies" }));
                }
            }
            r.build_kinds(&crate::mex_host::GenoEnv { dir: dir_of_boot() });
            flush_log(&mut r);
            BOOT = Some(r);
        }
        BOOT.as_mut().unwrap()
    }
}

fn at(p: i32) -> Option<&'static Profile> {
    let r = reg();
    if p >= 0 && (p as usize) < r.p.len() { Some(&r.p[p as usize]) } else { None }
}
fn st(p: i32, s: i32) -> Option<&'static Profile> {
    at(p).filter(|x| s >= 0 && (s as usize) < x.nstate)
}
fn art(p: i32, a: i32) -> Option<&'static Profile> {
    at(p).filter(|x| a >= 0 && (a as usize) < x.nart)
}
fn ix<T: Copy>(n: usize, i: i32, v: &[T]) -> Option<T> {
    if i >= 0 && (i as usize) < n { Some(v[i as usize]) } else { None }
}

pub fn ProfileForKind(kind: i32) -> i32 {
    let r = reg();
    if r.p.is_empty() {
        return -1;
    }
    let k = r.profile_for_kind(kind, &crate::mex_host::GenoEnv { dir: dir_of_boot() });
    flush_log(r);
    k
}
pub fn ProfileCount() -> i32 { reg().p.len() as i32 }
pub fn Scale(p: i32) -> f32 { at(p).map_or(1.0, |x| if x.scale > 0.0 { x.scale } else { 1.0 }) }
pub fn MaxJumps(p: i32) -> i32 { at(p).map_or(-1, |x| x.max_jumps) }
pub fn JumpVyCount(p: i32) -> i32 { at(p).map_or(0, |x| x.njvy as i32) }
pub fn JumpVyBits(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.njvy, i, &x.jvy)).map_or(0, |v| v as i32) }
pub fn AttrCount(p: i32) -> i32 { at(p).map_or(0, |x| x.nattr as i32) }
pub fn AttrIndex(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nattr, i, &x.attr_index)).unwrap_or(-1) }
pub fn AttrBits(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nattr, i, &x.attr_bits)).map_or(0, |v| v as i32) }
fn ev_ok(ev: i32) -> bool { ev >= 0 && ev < GENO_EV_COUNT as i32 }
pub fn HookCount(p: i32, ev: i32) -> i32 { at(p).filter(|_| ev_ok(ev)).map_or(0, |x| x.nhook[ev as usize] as i32) }
pub fn Hook(p: i32, ev: i32, i: i32) -> i32 { at(p).filter(|_| ev_ok(ev)).and_then(|x| ix(x.nhook[ev as usize], i, &x.hook[ev as usize])).unwrap_or(0) }
pub fn HookArg(p: i32, ev: i32, i: i32) -> i32 { at(p).filter(|_| ev_ok(ev)).and_then(|x| ix(x.nhook[ev as usize], i, &x.hook_arg[ev as usize])).unwrap_or(0) }
pub fn SpecialCount(p: i32) -> i32 { at(p).map_or(0, |x| x.nspec as i32) }
pub fn SpecialIndex(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nspec, i, &x.spec_index)).unwrap_or(-1) }
pub fn SpecialBits(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nspec, i, &x.spec_bits)).map_or(0, |v| v as i32) }
pub fn OnLandCount(p: i32) -> i32 { at(p).map_or(0, |x| x.nland as i32) }
pub fn OnLandFrom(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nland, i, &x.land_from)).map_or(-1, |v| v as i32) }
pub fn OnLandTo(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nland, i, &x.land_to)).map_or(-1, |v| v as i32) }
pub fn MotionAnim(p: i32, motion: i32) -> i32 {
    let Some(x) = at(p) else { return -1 };
    (0..x.nmanim).find(|&i| x.manim_motion[i] == motion).map_or(-1, |i| x.manim_row[i])
}
pub fn OverlayCount(p: i32) -> i32 { at(p).map_or(0, |x| x.nov as i32) }
pub fn OverlayAnim(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nov, i, &x.ov_anim)).unwrap_or(-1) }
pub fn OverlaySlot(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nov, i, &x.ov_slot)).unwrap_or(-1) }
pub fn OverlayMoveTag(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nov, i, &x.ov_tag)).unwrap_or(0) }
pub fn ProfilePoolWords(p: i32) -> i32 {
    let Some(x) = at(p) else { return 0 };
    let r = reg();
    (0..x.nov).map(|i| r.slot_len[x.ov_slot[i] as usize] as i32).sum()
}
pub fn SlotCount() -> i32 { reg().slot_off.len() as i32 }
pub fn SlotOffset(s: i32) -> i32 { let r = reg(); ix(r.slot_off.len(), s, &r.slot_off).map_or(-1, |v| v as i32) }
pub fn SlotRelative(s: i32) -> i32 { let r = reg(); ix(r.slot_rel.len(), s, &r.slot_rel).map_or(0, |v| v as i32) }
pub fn SlotLen(s: i32) -> i32 { let r = reg(); ix(r.slot_len.len(), s, &r.slot_len).map_or(0, |v| v as i32) }
pub fn PoolWords() -> i32 { reg().pool.len() as i32 }
pub fn PoolWord(i: i32) -> i32 { let r = reg(); ix(r.pool.len(), i, &r.pool).map_or(0, |v| v as i32) }
pub fn PoolGen() -> i32 {
    reg();
    unsafe { POOL_GEN }
}
pub fn StateCount(p: i32) -> i32 { at(p).map_or(0, |x| x.nstate as i32) }
/// gw_Geno_StateName: the state's geno.json name (the Lab's state browser); None when there is no such state.
pub fn StateName(p: i32, s: i32) -> Option<String> { st(p, s).map(|x| x.st_name[s as usize].clone()) }
/// gw_Geno_ComboStateName: the name of state s of profile p (the LAB's combo panel), "" when there is none.
pub fn ComboStateName(p: i32, s: i32) -> String { st(p, s).map_or(String::new(), |x| x.st_name[s as usize].clone()) }
pub fn StateBehavior(p: i32, s: i32) -> i32 { st(p, s).map_or(GENO_BHV_NONE as i32, |x| x.st_bhv[s as usize]) }
pub fn StateCb(p: i32, s: i32, slot: i32) -> i32 {
    st(p, s).filter(|_| slot >= 0 && slot < GENO_CB_SLOTS as i32).map_or(-1, |x| x.st_cb[s as usize][slot as usize])
}
pub fn StateAnim(p: i32, s: i32) -> i32 { st(p, s).map_or(-1, |x| x.st_anim[s as usize]) }
pub fn StateAnimFrom(p: i32, s: i32) -> i32 { st(p, s).map_or(-1, |x| x.st_anim_from[s as usize] as i32) }
pub fn StateLike(p: i32, s: i32) -> i32 { st(p, s).map_or(-1, |x| x.st_like[s as usize] as i32) }
pub fn StateFlagsSet(p: i32, s: i32) -> i32 { st(p, s).map_or(0, |x| x.st_flags_set[s as usize] as i32) }
pub fn StateFlags(p: i32, s: i32) -> i32 { st(p, s).map_or(0, |x| x.st_flags[s as usize] as i32) }
pub fn StateMoveTag(p: i32, s: i32) -> i32 { st(p, s).map_or(0, |x| x.st_tag[s as usize]) }
pub fn StateMoveId(p: i32, s: i32) -> i32 { st(p, s).map_or(-1, |x| x.st_move_id[s as usize]) }
pub fn StateNext(p: i32, s: i32) -> i32 { st(p, s).map_or(-1, |x| x.st_next[s as usize] as i32) }
pub fn StateLand(p: i32, s: i32) -> i32 { st(p, s).map_or(-1, |x| x.st_land[s as usize] as i32) }
pub fn StateLagBits(p: i32, s: i32) -> i32 { st(p, s).map_or(0, |x| x.st_lag[s as usize] as i32) }
pub fn StateMotion(p: i32, s: i32) -> i32 { st(p, s).map_or(0, |x| x.st_motion[s as usize] as i32) }
pub fn StateGravityBits(p: i32, s: i32) -> i32 { st(p, s).map_or(0, |x| x.st_grav[s as usize] as i32) }
pub fn ParamCount(p: i32) -> i32 { at(p).map_or(0, |x| x.nparam as i32) }
pub fn ParamId(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nparam, i, &x.param_id)).unwrap_or(-1) }
pub fn ParamBits(p: i32, i: i32) -> i32 { at(p).and_then(|x| ix(x.nparam, i, &x.param_bits)).map_or(0, |v| v as i32) }
pub fn Special(p: i32, which: i32) -> i32 {
    at(p).filter(|_| which >= 0 && which < GENO_SP_COUNT as i32).map_or(-1, |x| x.special[which as usize] as i32)
}
pub fn StateCounter(p: i32, s: i32, field: i32) -> i32 {
    match st(p, s) {
        Some(x) if x.st_ctr_on[s as usize] && (0..=3).contains(&field) => x.st_ctr[s as usize][field as usize] as i32,
        _ => -1,
    }
}
pub fn StateHasCounter(p: i32, s: i32) -> i32 { st(p, s).map_or(0, |x| x.st_ctr_on[s as usize] as i32) }
pub fn SpecialSelect(p: i32, which: i32, i: i32) -> i32 {
    let Some(x) = at(p) else { return -1 };
    if which < 0 || which >= GENO_SP_COUNT as i32 {
        return -1;
    }
    let w = which as usize;
    match i {
        -1 => x.sp_sel_var[w],
        -2 => x.sp_sel_n[w] as i32,
        _ => ix(x.sp_sel_n[w], i, &x.sp_sel_t[w]).map_or(-1, |v| v as i32),
    }
}
pub fn FxBindings(p: i32) -> i32 { at(p).map_or(-1, |x| x.fx_set) }
pub fn ArticleCount(p: i32) -> i32 { at(p).map_or(0, |x| x.nart as i32) }
pub fn ArticleParam(p: i32, a: i32, id: i32) -> i32 {
    art(p, a).filter(|_| id >= 0 && id < GENO_AP_COUNT as i32).map_or(0, |x| x.art_param[a as usize][id as usize] as i32)
}
pub fn ArticleHitCount(p: i32, a: i32) -> i32 { art(p, a).map_or(0, |x| x.art_nhit[a as usize] as i32) }
pub fn ArticleHitParam(p: i32, a: i32, h: i32, id: i32) -> i32 {
    art(p, a)
        .filter(|x| h >= 0 && (h as usize) < x.art_nhit[a as usize] && id >= 0 && id < GENO_AH_COUNT as i32)
        .map_or(0, |x| x.art_hit[a as usize][h as usize][id as usize] as i32)
}
/// Article models (an HSD archive loaded into Geno's persistent region) are not hosted yet: no model, the article
/// spawns without one (the C answers 0 the same way when no mod reserved the region).
pub fn ArticleJoint(_p: i32, _a: i32) -> i32 { 0 }
pub fn ProfileIdWord(p: i32) -> u32 { at(p).map_or(0, |x| x.id as u32) }

pub fn WarnIasa(p: i32, s: i32) {
    let Some(x) = st(p, s) else { return };
    unsafe {
        if IASA_WARN_GEN != POOL_GEN {
            IASA_WARNED = vec![0; reg().p.len()];
            IASA_WARN_GEN = POOL_GEN;
        }
        if IASA_WARNED.len() < reg().p.len() {
            IASA_WARNED.resize(reg().p.len(), 0);
        }
        let bit = 1u64 << (s as u32 & 63);
        if IASA_WARNED[p as usize] & bit != 0 {
            return;
        }
        IASA_WARNED[p as usize] |= bit;
    }
    eprintln!("geno: {}: state {}: script contains IASA but iasa is none; use iasa: interrupt to enable cancels", x.name, x.st_name[s as usize]);
}

const EVENT_FMT: [&str; 46] = [
    "geno: kind %d player %d reset (profile %d)%.0d",
    "geno: kind %d player %d air jump %d of %d (beyond Melee's multi-jump table)",
    "geno: kind %d player %d hook geno.log arg %d LA0 %d",
    "geno: kind %d player %d script sub-command %d is unknown - skipped%.0d",
    "geno: kind %d player %d script calls unknown hook %d%.0d",
    "geno: kind %d player %d attributes overridden: %d field(s), max_jumps %d",
    "geno: kind %d player %d air jump %d of %d",
    "geno: kind %d player %d change action: motion %d -> target 0x%08x",
    "geno: kind %d player %d change action to Geno state %d: no such state - ignored%.0d",
    "geno: kind %d player %d landed in motion %d -> target 0x%08x",
    "geno: kind %d player %d rehit: cleared the hit lists of hitbox mask 0x%x (every %d frames)",
    "geno: kind %d player %d autolink: victim launched at angle %d, kb %d",
    "geno: kind %d player %d special attributes: %d word(s) overridden (first word %d)",
    "geno: kind %d player %d subaction %d script replaced by overlay slot %d",
    "geno: kind %d player %d engine value 0x%x is read-only or unknown - write ignored%.0d",
    "geno: kind %d player %d too many change-action checks (max %d) - dropped%.0d",
    "geno: kind %d player %d entered Geno state %d from motion %d",
    "geno: kind %d player %d glide: jump held %d frames in air jump motion %d",
    "geno: kind %d player %d special %d -> Geno target 0x%08x",
    "geno: kind %d player %d drill rush bounce (%d: 1 hit, 2 wall, 3 shield) at frame %d",
    "geno: kind %d player %d Geno state %d row built: subaction %d",
    "geno: kind %d player %d glide ended (%d: 1 shield, 2 timeout, 3 attack) after %d frames",
    "geno: kind %d player %d tornado rise %d (vy x100 = %d)",
    "geno: kind %d player %d root motion: state %d took off at action frame %d",
    "geno: kind %d player %d root motion: state %d landed -> target 0x%08x",
    "geno: kind %d player %d root motion: state %d grabbed the ledge (mode %d)",
    "geno: kind %d player %d cape: vanished at frame %d (stick x100 %d)",
    "geno: kind %d player %d cape: reappear %d (0 end, 1 N, 2 F, 3 B; +10 air) held 0x%x",
    "geno: kind %d player %d article %d spawned as item kind %d",
    "geno: kind %d player %d article %d despawned (%d: 0 timeout, 1 hit, 2 shield, 3 stage, 4 clank)",
    "geno: kind %d player %d article %d not spawned (%d: 0 no such article, 1 max_live, 2 item limit)",
    "geno: kind %d player %d on_hit: damage x100 %d from port %d",
    "geno: kind %d player %d COUNTER: hit in Geno state %d at action frame %d",
    "geno: kind %d player %d article %d at frame %d",
    "geno: article %d spawned at x100 (%d, %d), vx x1000 %d",
    "geno: article %d touched the stage: from x100 (%d, %d) to y x100 %d",
    "geno: article %d model on the item: %d joint(s), %d display object(s), scale x100 %d",
    "geno: article %d: %d effect(s) attached at article frame %d%.0d",
    "geno: article %d frame %d: %d generator(s) on it, %d live particle(s) from them",
    "geno:   particles per group of 4 generators (hi/lo byte pairs): 0x%04x 0x%04x 0x%04x 0x%04x",
    "geno: kind %d player %d lockon: target %d, aim %d deg",
    "geno:   particles behind the ball x100 %d (along the facing), above x100 %d, forward speed x1000 %d (%d)",
    "geno: kind %d player %d aim_stick: stick used %d, heading %d deg",
    "geno: kind %d player %d dash.search: %d (0 none, 1 + port locked), stick heading %d deg (-1 none)",
    "geno: kind %d player %d dash.aim: flags %d (1 target, 2 stick, +4 x kind), heading %d deg",
    "geno: article %d model: %d (1 drawn, 2 hidden because its fx package is its look - \"show_model\": true keeps both, 3 drawn with its fx package)%.0d",
];

/// gw_Geno_Event: the game half's log lines, at most 40 per kind (rollback resimulates frames).
pub fn Event(what: i32, a: i32, b: i32, c: i32, d: i32) {
    static mut COUNT: [i32; 64] = [0; 64];
    if what < 0 || what as usize >= EVENT_FMT.len() {
        return;
    }
    let n = unsafe {
        COUNT[what as usize] += 1;
        COUNT[what as usize]
    };
    if n > 40 {
        if n == 41 {
            eprintln!("geno: (further event {what} lines suppressed)");
        }
        return;
    }
    let mut out = String::new();
    let mut args = [a, b, c, d].into_iter();
    let f = EVENT_FMT[what as usize];
    let mut it = f.char_indices().peekable();
    while let Some((_, ch)) = it.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        let mut spec = String::new();
        while let Some(&(_, c2)) = it.peek() {
            it.next();
            if c2.is_ascii_alphabetic() {
                spec.push(c2);
                break;
            }
            spec.push(c2);
        }
        let v = args.next().unwrap_or(0);
        out.push_str(&match spec.as_str() {
            "d" => v.to_string(),
            "x" => format!("{:x}", v as u32),
            "08x" => format!("{:08x}", v as u32),
            "04x" => format!("{:04x}", v as u32),
            ".0d" => if v == 0 { String::new() } else { v.to_string() },
            _ => v.to_string(),
        });
    }
    eprintln!("{out}");
}

/// MELEE_GENO_STUNLOG=1: each damage-state entry's hitstun.
pub fn diag_stun(port: i32, base: i32, bonus: i32) {
    if std::env::var("MELEE_GENO_STUNLOG").ok().as_deref().is_some_and(|v| v.starts_with('1')) {
        if port <= -1000 {
            eprintln!("geno: contact flags 0x{:X}: P{} hit P{} (no damage state / damage / hitlag as flagged)", base, bonus + 1, -1000 - port + 1);
        } else {
            eprintln!("geno: hitstun P{}: base {} + bonus {} = {} frames", port + 1, base, bonus, base + bonus);
        }
    }
}

// ---- test support (never called outside the suite) ----
pub fn TestInstall(text: *const i8) -> i32 {
    let t = unsafe { core::ffi::CStr::from_ptr(text) }.to_bytes();
    reg();
    unsafe {
        if SAVED.is_none() {
            SAVED = Some(BOOT.take());
        }
        let mut r = Registry::default();
        let n = r.parse_text(t, "test", &NoEnv);
        r.build_kinds(&NoEnv);
        flush_log(&mut r);
        BOOT = Some(r);
        POOL_GEN += 1;
        n
    }
}
pub fn TestRestore() {
    unsafe {
        if let Some(s) = SAVED.take() {
            BOOT = s;
            POOL_GEN += 1;
        }
    }
}
pub fn TestReloadLayout(text: *const i8) -> i32 {
    let t = unsafe { core::ffi::CStr::from_ptr(text) }.to_bytes();
    let mut next = Registry::default();
    if next.parse_text(t, "reload-test", &NoEnv) < 0 {
        return -1;
    }
    next.build_kinds(&NoEnv);
    Registry::reload_layout_changed(reg(), &next).is_some() as i32
}

/// Point the game half's (and the test suite's) registry hooks here. Called before seams_gen fills the rest.
pub fn install() {
    macro_rules! set {
        ($m:path; $($h:ident = $f:expr),* $(,)?) => {{ use $m as hk; unsafe { $( hk::$h = Some($f); )* } }};
    }
    set!(melee_modes::tu::geno_game::ext::hook;
        Geno_ProfileForKind = ProfileForKind, Geno_ProfileCount = ProfileCount, Geno_MaxJumps = MaxJumps, Geno_Scale = Scale,
        Geno_JumpVyCount = JumpVyCount, Geno_JumpVyBits = JumpVyBits, Geno_AttrCount = AttrCount, Geno_AttrIndex = AttrIndex,
        Geno_AttrBits = AttrBits, Geno_HookCount = HookCount, Geno_Hook = Hook, Geno_HookArg = HookArg, Geno_Event = Event,
        Geno_SpecialCount = SpecialCount, Geno_SpecialIndex = SpecialIndex, Geno_SpecialBits = SpecialBits,
        Geno_OnLandCount = OnLandCount, Geno_MotionAnim = MotionAnim, Geno_OnLandFrom = OnLandFrom, Geno_OnLandTo = OnLandTo,
        Geno_OverlayCount = OverlayCount, Geno_OverlayAnim = OverlayAnim, Geno_OverlaySlot = OverlaySlot,
        Geno_SlotCount = SlotCount, Geno_SlotOffset = SlotOffset, Geno_SlotLen = SlotLen, Geno_SlotRelative = SlotRelative,
        Geno_PoolWords = PoolWords, Geno_PoolWord = PoolWord, Geno_PoolGen = PoolGen, Geno_StateCount = StateCount,
        Geno_StateBehavior = StateBehavior, Geno_StateCb = StateCb, Geno_StateAnim = StateAnim, Geno_StateAnimFrom = StateAnimFrom,
        Geno_StateLike = StateLike, Geno_StateFlagsSet = StateFlagsSet, Geno_StateFlags = StateFlags, Geno_StateMoveId = StateMoveId,
        Geno_StateNext = StateNext, Geno_StateLand = StateLand, Geno_StateLagBits = StateLagBits, Geno_StateMotion = StateMotion,
        Geno_StateGravityBits = StateGravityBits, Geno_ParamCount = ParamCount, Geno_ParamId = ParamId, Geno_ParamBits = ParamBits,
        Geno_Special = Special, Geno_ArticleCount = ArticleCount, Geno_ArticleParam = ArticleParam,
        Geno_ArticleHitCount = ArticleHitCount, Geno_ArticleHitParam = ArticleHitParam, Geno_ArticleJoint = ArticleJoint,
        Geno_StateCounter = StateCounter, Geno_StateHasCounter = StateHasCounter, Geno_SpecialSelect = SpecialSelect,
        Geno_FxBindings = FxBindings, Geno_ProfileIdWord = ProfileIdWord, Geno_WarnIasa = WarnIasa, diag_geno_stun = diag_stun,
    );
    set!(melee_modes::tu::geno_tests::ext::hook;
        Geno_TestInstall = TestInstall, Geno_TestRestore = TestRestore, Geno_TestReloadLayout = TestReloadLayout,
        Geno_ArticleCount = ArticleCount, Geno_ArticleParam = ArticleParam, Geno_ArticleHitCount = ArticleHitCount,
        Geno_ArticleHitParam = ArticleHitParam, Geno_StateCounter = StateCounter, Geno_HookCount = HookCount,
        Geno_SpecialSelect = SpecialSelect,
    );
}

// ---- gw_snap's slots for the game half (geno_game_items.inc, the define snapshot tests): snap_open(k) opens a
// session, snap_save(frame) / snap_load(frame) keep and put back whole-game states (crate::snapshot) ----
struct SnapStore {
    base: crate::snapshot::Base,
    slots: Vec<(i32, crate::snapshot::SnapRef)>,
}
/// Set once by the first snap_open and never changed after, so a load (which puts the static sections back) keeps it.
static mut SNAP: *mut SnapStore = core::ptr::null_mut();

fn host_heap<R>(f: impl FnOnce() -> R) -> R {
    let was = crate::arena::active();
    crate::arena::set_active(false);
    let r = f();
    crate::arena::set_active(was);
    r
}
pub fn snap_open(_k: i32) -> i32 {
    unsafe {
        if SNAP.is_null() {
            SNAP = host_heap(|| Box::into_raw(Box::new(SnapStore { base: crate::snapshot::Base::capture(), slots: Vec::new() })));
        }
    }
    0
}
pub fn snap_save(frame: i32) {
    unsafe {
        if SNAP.is_null() {
            snap_open(1);
        }
        let st = &mut *SNAP;
        host_heap(|| {
            let s = st.base.save();
            st.slots.retain(|x| x.0 != frame);
            st.slots.push((frame, s));
        });
    }
}
pub fn snap_load(frame: i32) -> i32 {
    unsafe {
        if SNAP.is_null() {
            return -1;
        }
        let st = &mut *SNAP;
        let Some(s) = st.slots.iter().find(|x| x.0 == frame).map(|x| x.1.clone()) else { return -1 };
        let was = crate::arena::active();
        crate::arena::set_active(false);
        st.base.load(&s);
        crate::arena::set_active(was);
        0
    }
}

/// The out-of-TU callees the game half and its tests reach that no link-table row answers.
pub fn install_callees() {
    use melee_modes::tu::geno_game::ext::hook as g;
    use melee_modes::tu::geno_tests::ext::hook as t;
    unsafe {
        g::snap_open = Some(snap_open);
        g::snap_save = Some(snap_save);
        g::snap_load = Some(snap_load);
        t::snap_open = Some(snap_open);
        t::snap_save = Some(snap_save);
        t::snap_load = Some(snap_load);
        t::memset = Some(|d, c, n| { core::ptr::write_bytes(d as *mut u8, c as u8, n); d });
        t::memcpy = Some(|d, s, n| { core::ptr::copy(s as *const u8, d as *mut u8, n); d });
        g::ftCommon_SetSelfMovementFromGroundedMovement = Some(|gobj| melee_fighter::tu::ftcommon::ftCommon_SetSelfMovementFromGroundedMovement_fp(gobj as _));
        g::ftCommon_CalcSelfAccel_DeaccelAir = Some(|fp| melee_fighter::tu::ftcommon::ftCommon_CalcSelfAccel_DeaccelAir_fp(&mut *fp));
        g::ftCommon_CalcSelfAccel_Drift = Some(|fp| melee_fighter::tu::ftcommon::ftCommon_CalcSelfAccel_Drift_fp(&mut *fp));
        g::ftCommon_Fall = Some(|fp, a, b| melee_fighter::tu::ftcommon::ftCommon_Fall_fp(&mut *fp, a, b));
    }
}

/// The registry API by name, for the `ext_fns!` callees that reach the engine's by-name table (script_game, ...).
/// Define-only queries (native definitions are not hosted yet) answer the "no define" value.
pub fn by_name(name: &str, a: &[u64]) -> Option<u64> {
    let i = |k: usize| a.get(k).copied().unwrap_or(0) as i32;
    let v: i32 = match name {
        "Geno_ProfileForKind" => ProfileForKind(i(0)),
        "Geno_ProfileCount" => ProfileCount(),
        "Geno_StateMoveTag" => StateMoveTag(i(0), i(1)),
        "Geno_OverlayMoveTag" => OverlayMoveTag(i(0), i(1)),
        "Geno_OverlayCount" => OverlayCount(i(0)),
        "Geno_OverlayAnim" => OverlayAnim(i(0), i(1)),
        "Geno_CommonMoveTag" => CommonMoveTag(i(0), i(1)),
        "Geno_StateCount" => StateCount(i(0)),
        "Geno_StateCounter" => StateCounter(i(0), i(1), i(2)),
        "Geno_HookCount" => HookCount(i(0), i(1)),
        "Geno_ArticleCount" => ArticleCount(i(0)),
        "Geno_MaxJumps" => MaxJumps(i(0)),
        "Geno_SpecialSelect" => SpecialSelect(i(0), i(1), i(2)),
        "Geno_DefineBaseKind" => DefineBaseKind(i(0)),
        "Geno_DefineBaseCK" => DefineBaseCK(i(0)),
        "Geno_DefineArtBaseCK" => DefineArtBaseCK(i(0)),
        "Geno_DefineAiLike" => DefineAiLike(i(0)),
        "Geno_DefineIsNone" => DefineIsNone(i(0)),
        "Geno_DefinePlanInt" => DefinePlanInt(i(0), i(1), i(2)),
        "Geno_DefineCount" => DefineCount(),
        "Geno_DefineCKAt" => DefineCKAt(i(0)),
        "Geno_ComboParam" => ComboParam(i(0), i(1)),
        "Geno_LuaSlots" => LuaSlots(i(0)),
        _ => return None,
    };
    Some(v as i64 as u64)
}

// ---- native defines (geno_define_registry.inc and its slices): the gw_Geno_Define* / Lua / combo / meter / record API ----

fn built() -> &'static mut Registry {
    let r = reg();
    if !r.kinds_built {
        r.build_kinds(&crate::mex_host::GenoEnv { dir: dir_of_boot() });
        flush_log(r);
    }
    r
}

/// Re-attach the profiles once the m-ex slots exist (`mex_host::install`): a query made earlier (a scenario's
/// define token resolves at parse time) built the kinds without them, and an attach to an m-ex Pl file stayed inactive.
pub fn rebind_kinds() {
    unsafe {
        if let Some(r) = BOOT.as_mut() {
            r.build_kinds(&crate::mex_host::GenoEnv { dir: dir_of_boot() });
            r.log.clear();
        }
    }
}

pub fn DefineCount() -> i32 { built().define_count() }
pub fn DefineCKAt(i: i32) -> i32 { built().define_ck_at(i) }
pub fn CommonMoveTag(p: i32, motion: i32) -> i32 { reg().common_move_tag(p, motion) }
pub fn DefineBaseKind(kind: i32) -> i32 { built().define_base_kind(kind) }
pub fn DefinePartnerKind(kind: i32) -> i32 { built().define_partner_kind(kind) }
pub fn DefineBaseCK(ck: i32) -> i32 { built().define_base_ck(ck) }
pub fn DefineResolve(token: &str) -> i32 { built().define_resolve(token) }
pub fn DefineIdentityWord(kind: i32, high: i32) -> i32 { built().define_identity_word(kind, high != 0) }
pub fn DefineOverrideCount(kind: i32) -> i32 { built().define_override_count(kind) }
pub fn DefineOverrideField(kind: i32, row: i32, field: i32) -> i32 { built().define_override_field(kind, row, field) }
pub fn DefineIsNone(kind: i32) -> i32 { built().define_is_none(kind) }
pub fn DefineTemplateSkeleton(kind: i32) -> i32 { built().define_template_skeleton(kind) }
pub fn DefineCostumeCount(kind: i32) -> i32 { built().define_costume_count(kind) }
pub fn DefineCostumeCountCK(ck: i32) -> i32 { built().define_costume_count_ck(ck) }
pub fn DefineArtBaseCK(ck: i32) -> i32 { built().define_art_base_ck(ck) }
pub fn DefineTeamCostume(ck: i32, team: i32) -> i32 { built().define_team_costume(ck, team) }
/// gw_Geno_DefineHasArt (no skins are installed in this engine).
pub fn DefineHasArt(ck: i32, what: i32, costume: i32) -> i32 { built().define_pres_file(ck, what, costume).is_some() as i32 }
pub fn DefineAiLike(kind: i32) -> i32 {
    static mut SAID: [bool; 128] = [false; 128];
    let r = built();
    let v = r.define_ai_like(kind);
    if v >= 0 {
        unsafe {
            if !SAID[kind as usize & 127] {
                SAID[kind as usize & 127] = true;
                let p = r.definitions.profile_for_kind(kind);
                eprintln!(
                    "geno: CPU AI: {} (kind {kind}) is read as retail kind {v} (ai.like): its per-kind switches and AI tables answer for that fighter",
                    r.p[p as usize].name
                );
            }
        }
    }
    v
}
pub fn DefineKirbyCopy(kind: i32) -> i32 { built().define_kirby_copy(kind) }
pub fn DefineThrownAs(kind: i32) -> i32 { built().define_thrown_as(kind) }
pub fn DefineThrownAsRow(kind: i32, row: i32) -> i32 { built().define_thrown_as_row(kind, row) }
/// gw_Geno_ThrowBankLimit: MELEE_THROW_BANKS overrides; two arenas when a define names two or more stand-in fighters.
pub fn ThrowBankLimit(def: i32) -> i32 {
    let need = built().throw_banks_needed();
    match std::env::var("MELEE_THROW_BANKS") {
        Ok(e) if !e.is_empty() => e.trim().parse().unwrap_or(0),
        _ => if need >= 2 && def < 2 { 2 } else { def },
    }
}

/// A NUL-terminated copy of a registry string that stays put for the process (the C hands out pointers into the
/// registry; the game half keeps them in its tables). Interned, so a repeated query returns the same pointer.
fn intern(s: &str) -> *const core::ffi::c_char {
    static mut POOL: Vec<(String, Box<[u8]>)> = Vec::new();
    unsafe {
        if let Some(x) = POOL.iter().find(|x| x.0 == s) {
            return x.1.as_ptr() as *const _;
        }
        let mut b = s.as_bytes().to_vec();
        b.push(0);
        let b = b.into_boxed_slice();
        let ptr = b.as_ptr();
        POOL.push((s.to_string(), b));
        ptr as *const _
    }
}
pub fn DefineString(kind: i32, which: i32, costume: i32) -> *const core::ffi::c_char {
    built().define_string(kind, which, costume).map_or(core::ptr::null(), intern)
}
/// gw_Geno_DefineName: snprintf the define's name into out (cap bytes). 1 when it is a define.
pub fn DefineName(ck: i32, out: *mut i8, cap: i32) -> i32 {
    let Some(n) = built().define_name(ck) else { return 0 };
    if out.is_null() || cap <= 0 {
        return 0;
    }
    let b = n.as_bytes();
    let k = b.len().min(cap as usize - 1);
    unsafe {
        core::ptr::copy_nonoverlapping(b.as_ptr() as *const i8, out, k);
        *out.add(k) = 0;
    }
    1
}
/// gw_Geno_DefineCostumeName: a base "none" define's declared costume name (no skins here).
pub fn DefineCostumeName(ck: i32, costume: i32, out: *mut i8, cap: i32) -> i32 {
    let Some(n) = built().define_costume_name(ck, costume) else { return 0 };
    if out.is_null() || cap <= 0 {
        return 0;
    }
    let b = n.as_bytes();
    let k = b.len().min(cap as usize - 1);
    unsafe {
        core::ptr::copy_nonoverlapping(b.as_ptr() as *const i8, out, k);
        *out.add(k) = 0;
    }
    (k > 0) as i32
}
fn plan(kind: i32) -> Option<&'static melee_script::geno::plan::Plan> {
    let r = built();
    let env = crate::mex_host::GenoEnv { dir: dir_of_boot() };
    let had = r.log.len();
    let p = r.define_plan(kind, &env).map(|p| p as *const melee_script::geno::plan::Plan);
    if r.log.len() > had {
        flush_log(r);
    }
    p.map(|p| unsafe { &*p })
}
/// gw_Geno_DefinePlanInt: a base "none" define's plan (plan.json in its mod's files/), "no plan" (0 / -1) otherwise.
pub fn DefinePlanInt(kind: i32, what: i32, i: i32) -> i32 {
    match plan(kind) {
        Some(p) => p.int(what, i),
        None => if what == 0 { 0 } else { -1 },
    }
}
/// gw_Geno_DefinePlanText: 0 clip name, 1 clip symbol, 2 joint_to_part, 3 part_to_joint (NULL when none).
pub fn DefinePlanText(kind: i32, which: i32, i: i32) -> *const core::ffi::c_char {
    plan(kind).and_then(|p| p.text(which, i)).map_or(core::ptr::null(), |p| p as *const _)
}
// ---- slice 6: a define's own sound clips (geno_define_registry.inc gn_clip_load / gn_define_audio_play / VoiceState) ----
//
// A clip is a .gnsnd from the define's mod `files/` (tools/geno/audio.py: 32-byte big-endian header, s16 BE samples, 32 kHz),
// decoded once (first use) and kept for the process. It plays through `melee_audio`'s native clip slots (shim_ax.c
// gw_Audio_PlayOwnedClip): summed into the main mix after the voices and aux buses, one 5 ms sub-frame at a time. Nothing
// here is simulation state; the owner is the fighter address as an opaque scalar (arena-allocated, so the same in a
// resimulation of the same frame).
//
// Rollback (netplay): the clip cache, the PCM and the start ledger live in one host-heap value reached through a pointer
// set before the match ([`GnHost`], `host_heap`), never in a static a load rewrites (net.md's rule), and the mixer's clip
// slots are boxed outside the restored `Sound` (melee_audio `ClipBank`): clips play in real time. The netplay harness
// reports each logic frame ([`net_frame`]) and the confirmed frame ([`net_confirmed`]). A start is keyed by (owner, row,
// start frame): a resimulated frame that starts a clip the ledger already holds keeps the playing instance; after a
// rollback the starts at or after the loaded frame are tentative until resimulated again, and the ones the new timeline
// does not repeat are cut at the first new frame; a start at or before the confirmed frame leaves the ledger.

/// The retail "stop this voice slot" id (gn_define_audio_play).
const GN_STOP_SFX: i32 = 540001;

#[derive(Default)]
struct GnClip {
    tried: bool,
    pcm: Option<(std::sync::Arc<[i16]>, i32, i32)>, // samples (interleaved), frames, channels
}

/// One started define clip instance (the netplay ledger).
struct GnStart {
    owner: u32,
    p: usize,
    slot: usize,
    sfx: i32,
    frame: i32,
    id: u32,
    tentative: bool,
}

#[derive(Default)]
struct GnNet {
    on: bool,
    frame: i32,
    resim: bool,
    last: i32,
    live: Vec<GnStart>,
}

/// Define audio state that must not roll back: on the system heap (made outside the arena), reached through [`GN_HOST`].
#[derive(Default)]
struct GnHost {
    /// Clip cache per profile: [0] the announcer call, [1 + row] the voice rows.
    clips: Vec<Vec<GnClip>>,
    /// Retail sound ids a define played without an audio.voice row (named once each, at most 64).
    seen: Vec<i32>,
    net: GnNet,
}
/// Set once (before a match: [`net_begin`], or the first offline use) and never changed after, so a load keeps it.
static mut GN_HOST: *mut GnHost = core::ptr::null_mut();

/// The host value, made on first use; every change to it goes through `host_heap` so nothing it owns is in the arena.
fn gn_host() -> &'static mut GnHost {
    unsafe {
        if GN_HOST.is_null() {
            GN_HOST = host_heap(|| Box::into_raw(Box::default()));
        }
        &mut *GN_HOST
    }
}

/// Netplay: a rollback match is about to start (call outside the game scope, before the first snapshot).
pub fn net_begin() {
    let h = gn_host();
    host_heap(|| h.net = GnNet { on: true, frame: i32::MIN, last: i32::MIN, ..Default::default() });
}

/// Netplay: the match is over; the ledger is dropped (the clips play out).
pub fn net_end() {
    let h = gn_host();
    host_heap(|| h.net = GnNet::default());
}

fn gn_name(p: usize) -> String {
    built().p.get(p).map_or_else(|| "?".to_string(), |x| x.name.clone())
}

/// Netplay: logic frame `frame` is about to run (`resim`: a resimulation of an earlier tick's frame).
pub fn net_frame(frame: i32, resim: bool) {
    let h = gn_host();
    if !h.net.on {
        return;
    }
    if frame <= h.net.last {
        let mut n = 0;
        for e in h.net.live.iter_mut().filter(|e| e.frame >= frame) {
            e.tentative = true;
            n += 1;
        }
        if n != 0 {
            eprintln!("geno: define audio: rollback to frame {frame}: {n} start(s) await resimulation");
        }
    }
    if !resim && h.net.live.iter().any(|e| e.tentative) {
        let cut: Vec<GnStart> = host_heap(|| {
            let (cut, keep) = std::mem::take(&mut h.net.live).into_iter().partition(|e| e.tentative);
            h.net.live = keep;
            cut
        });
        for e in &cut {
            let n = melee_audio::sim::with(|s| s.ax.stop_clip_id(e.id)).unwrap_or(0);
            eprintln!(
                "geno: define audio: {}: cut mispredicted start sfx {} owner {:08x} frame {} ({n} playing instance)",
                gn_name(e.p), e.sfx, e.owner, e.frame
            );
        }
        host_heap(|| drop(cut));
    }
    h.net.frame = frame;
    h.net.resim = resim;
    h.net.last = frame;
}

/// Netplay: every frame up to `conf` has both peers' inputs; its starts are final.
pub fn net_confirmed(conf: i32) {
    let h = gn_host();
    if !h.net.on || !h.net.live.iter().any(|e| e.frame <= conf && !e.tentative) {
        return;
    }
    let done: Vec<GnStart> = host_heap(|| {
        let (done, keep) = std::mem::take(&mut h.net.live).into_iter().partition(|e| e.frame <= conf && !e.tentative);
        h.net.live = keep;
        done
    });
    for e in &done {
        eprintln!("geno: define audio: {}: confirmed start sfx {} owner {:08x} frame {}", gn_name(e.p), e.sfx, e.owner, e.frame);
    }
    host_heap(|| drop(done));
}

/// gn_gnsnd_decode: a v1 .gnsnd to native s16, or why it is not usable.
pub fn gnsnd_decode(b: &[u8]) -> Result<(Vec<i16>, i32, i32), &'static str> {
    if b.len() < 32 || &b[0..4] != b"GNSD" {
        return Err("not a .gnsnd (magic)");
    }
    let w = |i: usize| u32::from_be_bytes([b[i * 4], b[i * 4 + 1], b[i * 4 + 2], b[i * 4 + 3]]);
    if w(1) != 1 {
        return Err("unknown .gnsnd version");
    }
    if w(2) != 32000 {
        return Err("sample rate is not 32000");
    }
    if !(1..=2).contains(&w(3)) {
        return Err("channels must be 1 or 2");
    }
    if !(1..=320000).contains(&w(4)) {
        return Err("frames must be 1..320000");
    }
    if w(5) != 32 || w(7) != 0 || w(6) != w(4) * w(3) * 2 || b.len() as u64 != 32 + w(6) as u64 {
        return Err("header and length disagree");
    }
    let n = (w(4) * w(3)) as usize;
    let pcm = (0..n).map(|i| i16::from_be_bytes([b[32 + i * 2], b[33 + i * 2]])).collect();
    Ok((pcm, w(4) as i32, w(3) as i32))
}

fn gn_clip_slot(p: usize, slot: usize) -> &'static mut GnClip {
    let c = &mut gn_host().clips;
    host_heap(|| {
        if c.len() <= p {
            c.resize_with(p + 1, Vec::new);
        }
        if c[p].len() <= slot {
            c[p].resize_with(slot + 1, GnClip::default);
        }
    });
    &mut c[p][slot]
}

#[cfg(test)]
fn gn_clips_clear() {
    host_heap(|| gn_host().clips.clear());
}

/// gn_clip_load: decode the clip on first use; None (logged once) when the file is missing or unusable.
fn gn_clip_load(p: usize, slot: usize, file: &str) -> Option<(std::sync::Arc<[i16]>, i32, i32)> {
    let c = gn_clip_slot(p, slot);
    if let Some(x) = c.pcm.as_ref() {
        return Some(x.clone());
    }
    if c.tried {
        return None;
    }
    c.tried = true;
    let r = built();
    let (mod_id, name) = (r.p[p].mod_id.clone(), r.p[p].name.clone());
    let env = crate::mex_host::GenoEnv { dir: dir_of_boot() };
    use melee_script::geno::Env as _;
    // on the system heap: a read into the copy-on-write arena fails (ERROR_NOACCESS) during a rollback match
    let Some(raw) = host_heap(|| env.read_mod_file(&mod_id, &format!("files/{file}"))) else {
        eprintln!("geno: {name}: audio file {file} is not in the mod's files/");
        return None;
    };
    match host_heap(|| gnsnd_decode(&raw).map(|(pcm, f, ch)| (std::sync::Arc::<[i16]>::from(pcm), f, ch))) {
        Ok((pcm, frames, ch)) => {
            eprintln!("geno: {name}: audio {file}: {frames} frames, {ch} channel(s)");
            c.pcm = Some((pcm, frames, ch));
            c.pcm.clone()
        }
        Err(why) => {
            eprintln!("geno: {name}: audio file {file} is not usable: {why}");
            None
        }
    }
}

/// gw_rb_active (as the define audio sees it): the netplay harness is running a resimulated frame ([`net_frame`]).
fn gn_rb_active() -> bool {
    let n = &gn_host().net;
    n.on && n.resim
}

/// gn_define_audio_play: start the define's clip (`which` 0 = its announcer call, `sfx` ignored; 1 = the voice row for retail
/// sound id `sfx`). 1 when the define answers for it (played, or declared but unusable: stay silent rather than fall back to
/// the donor's voice), 0 when it has nothing (the caller plays the retail sound). The stop id cuts its stoppable rows.
pub fn define_audio_play(ck: i32, which: i32, sfx: i32, owner: u32) -> i32 {
    let r = built();
    let p = r.define_profile_ck(ck);
    if p < 0 {
        return 0;
    }
    let p = p as usize;
    let Some(a) = r.p[p].def.aud.as_ref() else { return 0 };
    let name = r.p[p].name.clone();
    if which == 1 && sfx == GN_STOP_SFX {
        let (mut any, mut stopped) = (false, 0);
        for i in 0..a.voice_sfx.len() {
            if !a.voice_stop.get(i).copied().unwrap_or(false) {
                continue;
            }
            any = true;
            // stopping an unloaded clip must not load it
            let pcm = gn_clip_slot(p, 1 + i).pcm.as_ref().map(|x| x.0.clone());
            // a resimulated stop is applied too (idempotent on the real-time clips): the new timeline's stops count
            if let Some(pcm) = pcm {
                stopped += melee_audio::sim::with(|s| s.ax.stop_owned_clips(&pcm, owner)).unwrap_or(0);
            }
        }
        if any && !gn_rb_active() && !crate::engine::speculative() {
            eprintln!("geno: {name}: stop id 540001 cut {stopped} stoppable clip instance(s)");
        }
        return any as i32;
    }
    let (slot, file, vol) = if which == 0 {
        if a.announcer.is_empty() {
            return 0;
        }
        (0, a.announcer.clone(), 127)
    } else {
        let Some(i) = a.voice_sfx.iter().position(|&x| x == sfx) else {
            // a define with an audio block is being authored: name each retail sound id it plays that has no row, once
            let seen = &mut gn_host().seen;
            if !seen.contains(&sfx) && seen.len() < 64 {
                host_heap(|| seen.push(sfx));
                eprintln!("geno: {name}: plays retail sound {sfx} (no audio.voice row for it)");
            }
            return 0;
        };
        (1 + i, a.voice_file[i].clone(), a.voice_vol[i])
    };
    if crate::engine::speculative() {
        // a lab lookahead tick (melee/src/lab.rs): the same answer, no clip, no ledger entry
        return 1;
    }
    let Some((pcm, frames, ch)) = gn_clip_load(p, slot, &file) else { return 1 };
    let net = &mut gn_host().net;
    if net.on {
        let f = net.frame;
        if let Some(e) = net.live.iter_mut().find(|e| e.owner == owner && e.p == p && e.slot == slot && e.frame == f) {
            // the same start, resimulated: the playing instance is it
            e.tentative = false;
            return 1;
        }
        let id = host_heap(|| melee_audio::sim::with(|s| s.ax.play_owned_clip_id(pcm, frames, ch, vol, owner)).unwrap_or(0));
        host_heap(|| net.live.push(GnStart { owner, p, slot, sfx, frame: f, id, tentative: false }));
        eprintln!(
            "geno: define audio: {name}: start sfx {sfx} owner {owner:08x} frame {f}{} ({file})",
            if net.resim { " (resimulated)" } else { "" }
        );
        return 1;
    }
    host_heap(|| melee_audio::sim::with(|s| s.ax.play_owned_clip(pcm, frames, ch, vol, owner)));
    eprintln!("geno: {name}: plays its own {} ({file})", if which == 0 { "announcer call" } else { "voice line" });
    1
}

/// gw_Geno_DefineAudioPlay (the announcer call from gm_1601: no owner).
pub fn DefineAudioPlay(ck: i32, which: i32, sfx: i32) -> i32 {
    define_audio_play(ck, which, sfx, 0)
}

/// gw_Geno_DefineVoicePlayOwner (ft_0877 / ft_0881's voice calls): by FighterKind, owned by the fighter.
pub fn DefineVoicePlayOwner(kind: i32, sfx: i32, owner: u32) -> i32 {
    if kind < 0x21 { 0 } else { define_audio_play(kind + 1, 1, sfx, owner) }
}

/// gw_Geno_DefineVoiceState: on an action change (motion set; -1 on a fighter reset), cut the owner's state-scoped clips whose
/// row's state mask does not include the new state. Returns how many instances were stopped.
pub fn DefineVoiceState(kind: i32, owner: u32, motion: i32) -> i32 {
    if kind < 0x21 {
        return 0;
    }
    let r = built();
    let p = r.define_profile_ck(kind + 1);
    if p < 0 {
        return 0;
    }
    let p = p as usize;
    let Some(a) = r.p[p].def.aud.as_ref() else { return 0 };
    let s = motion - melee_script::gd::players::GENO_MOTION_BASE;
    let nstate = r.p[p].nstate as i32;
    let mut stopped = 0;
    for i in 0..a.voice_sfx.len() {
        let mask = a.voice_states.get(i).copied().unwrap_or(0);
        if mask == 0 || (s >= 0 && s < nstate && s < 64 && mask & (1u64 << s) != 0) {
            continue;
        }
        let Some(pcm) = gn_clip_slot(p, 1 + i).pcm.as_ref().map(|x| x.0.clone()) else { continue };
        stopped += melee_audio::sim::with(|x| x.ax.stop_owned_clips(&pcm, owner)).unwrap_or(0);
    }
    if stopped != 0 && !gn_rb_active() {
        eprintln!("geno: {}: state exit to {motion} cut {stopped} stoppable clip instance(s), owner {owner:08x}", r.p[p].name);
    }
    stopped
}

// slice 5: fighter Lua
pub fn LuaFn(p: i32, s: i32, phase: i32) -> i32 { reg().lua_fn(p, s, phase) as i32 }
pub fn LuaSlots(p: i32) -> i32 { reg().lua_slots(p) }
pub fn LuaSlotType(p: i32, i: i32) -> i32 { reg().lua_slot_type(p, i) }

static mut FAULT_FRAME: i32 = i32::MAX;
static mut FAULT_HARD: i32 = -1;
fn fault_hard() -> bool {
    unsafe {
        if FAULT_HARD < 0 {
            FAULT_HARD = (std::env::var("MELEE_GENO_FAULT_ONLINE").ok().as_deref() == Some("hard")) as i32;
        }
        FAULT_HARD == 1
    }
}
pub fn FaultSetHard(on: i32) { unsafe { FAULT_HARD = (on != 0) as i32; FAULT_FRAME = i32::MAX; } }
pub fn FaultNote(frame: i32) { unsafe { if frame < FAULT_FRAME { FAULT_FRAME = frame; } } }
pub fn FaultRolledBack() { unsafe { FAULT_FRAME = i32::MAX; } }
pub fn FaultConfirmed(confirmed: i32) -> i32 { unsafe { (FAULT_FRAME != i32::MAX && confirmed >= FAULT_FRAME) as i32 } }
/// Set by the netplay session: (enabled, rollback active, current frame) for the hard fault policy.
pub static mut NET_STATE: Option<fn() -> (bool, bool, i32)> = None;

/// gw_Geno_LuaCall over the game half's io block (GENO_LUA_IO_WORDS native words).
pub fn LuaCall(p: i32, s: i32, phase: i32, io: *mut core::ffi::c_void) -> i32 {
    let r = reg();
    let rc = unsafe { r.lua_call(p, s, phase, io as *mut u32) };
    if rc > 0 && fault_hard() {
        if let Some(f) = unsafe { NET_STATE } {
            let (net, rb, frame) = f();
            if net && rb {
                FaultNote(frame);
            }
        }
    }
    flush_log(r);
    rc
}

// geno 11: combo graph and meter
pub fn ComboParam(p: i32, what: i32) -> i32 { reg().combo_param(p, what) }
pub fn ComboWord(p: i32, route: i32, word: i32) -> i32 { reg().combo_word(p, route, word) }
/// gw_Geno_ComboName: route `route`'s name, "" when the profile has no such route.
pub fn ComboName(p: i32, route: i32) -> String { reg().combo_name(p, route).to_string() }
pub fn MeterParam(p: i32, what: i32) -> i32 { reg().meter_param(p, what) }
/// gw_Geno_ComboNote: the log of a taken link (rate-limited; never state).
pub fn ComboNote(p: i32, route: i32, from: i32, to: i32, frame: i32, ki_links: i32) {
    static mut LOGGED: i32 = 0;
    unsafe {
        if LOGGED >= 4000 {
            return;
        }
        LOGGED += 1;
    }
    eprintln!(
        "geno: combo link {}: route {route} ({}) motion {from} -> {to} at action frame {frame}, ki {:.2}",
        (ki_links >> 24) & 0xFF,
        reg().combo_name(p, route),
        (ki_links & 0xFFFFFF) as f64 / 100.0
    );
}

// slice 6: records (MELEE_GENO_RECORDS names the file; with no card folder hosted, nothing is written without it)
static mut RECORDS: Option<melee_script::geno::define::Records> = None;
fn rec_path() -> Option<std::path::PathBuf> {
    std::env::var("MELEE_GENO_RECORDS").ok().filter(|e| !e.is_empty()).map(std::path::PathBuf::from)
}
fn rec_row(ck: i32) -> Option<(usize, std::path::PathBuf)> {
    let r = built();
    let p = r.define_profile_ck(ck);
    if p < 0 {
        return None;
    }
    let path = rec_path()?;
    let (key, name) = (r.p[p as usize].def.define_key.clone(), r.p[p as usize].name.clone());
    let recs = unsafe { RECORDS.get_or_insert_with(Default::default) };
    let mut log = Vec::new();
    let i = recs.row(&path, &key, &name, &mut log);
    for l in log {
        eprintln!("{l}");
    }
    i.map(|i| (i, path))
}
pub fn RecordAdd(ck: i32, field: i32, value: i32) -> i32 {
    let Some((i, _)) = rec_row(ck) else { return 0 };
    if !(0..melee_script::geno::define::GN_REC_FIELDS as i32).contains(&field) {
        return 0;
    }
    unsafe { RECORDS.as_mut().unwrap().add(i, field as usize, value as u32) };
    1
}
pub fn RecordCommit(ck: i32) -> i32 {
    let Some((i, path)) = rec_row(ck) else { return 0 };
    let recs = unsafe { RECORDS.as_ref().unwrap() };
    let mut log = Vec::new();
    let ok = recs.save_file(&path, &mut log);
    for l in log {
        eprintln!("{l}");
    }
    if ok {
        let r = &recs.recs[i];
        eprintln!("geno: {}: record written to {} ({} match(es), {} win(s), {} KO(s))", r.1, path.display(), r.2[0], r.2[1], r.2[3]);
    }
    ok as i32
}
pub fn RecordGet(ck: i32, field: i32) -> i32 {
    if !(0..melee_script::geno::define::GN_REC_FIELDS as i32).contains(&field) {
        return -1;
    }
    match rec_row(ck) {
        Some((i, _)) => unsafe { RECORDS.as_ref().unwrap().recs[i].2[field as usize] as i32 },
        None => -1,
    }
}

/// slice 7: the online identity inputs of define number `index` (geno_define_online.inc).
pub fn DefineOnlineInfo(index: i32) -> Option<melee_script::geno::define::OnlineInfo> { built().define_online_info(index) }

/// Point every TU's define / Lua / combo / record callee at the registry (after `install`, before seams_gen's defaults).
pub fn install_define() {
    use melee_fighter::platform::hook as pf;
    use melee_modes::tu::geno_game::ext::hook as g;
    use melee_modes::tu::geno_tests::ext::hook as t;
    use melee_modes::tu::gm_1601::ext::hook as gm;
    unsafe {
        pf::Geno_DefineBaseKind = Some(DefineBaseKind);
        pf::Geno_DefineIsNone = Some(DefineIsNone);
        pf::Geno_DefineThrownAs = Some(DefineThrownAs);
        pf::Geno_DefineThrownAsRow = Some(DefineThrownAsRow);
        pf::Geno_DefineAiLike = Some(DefineAiLike);
        pf::Geno_DefineTemplateSkeleton = Some(DefineTemplateSkeleton);
        pf::Geno_DefineKirbyCopy = Some(DefineKirbyCopy);
        melee_fighter::hooks::GENO_DEFINE_IS_NONE = Some(DefineIsNone);
        // the per-crate copies of the define queries (not routed by name)
        melee_fighter::hooks::GENO_DEFINE_BASE_KIND = Some(DefineBaseKind);
        melee_fighter::tu::ftdynamics::ext::GENO_DEFINE_BASE_KIND = Some(DefineBaseKind);
        melee_fighter::tu::ftdynamics::ext::GENO_DEFINE_TEMPLATE_SKELETON = Some(DefineTemplateSkeleton);
        melee_fighter::tu::ftcoll::ext::GENO_DEFINE_IS_NONE = Some(DefineIsNone);
        melee_fighter::tu::ftcoll::ext::GENO_DEFINE_TEMPLATE_SKELETON = Some(DefineTemplateSkeleton);
        melee_fighter::tu::ftcpuattack::ext::GENO_DEFINE_AI_LIKE = Some(DefineAiLike);
        melee_chara::tu::ftmariospecialn::ext::GENO_DEFINE_BASE_KIND = Some(DefineBaseKind);
        melee_fighter::hooks::GENO_DEFINE_TEMPLATE_SKELETON = Some(DefineTemplateSkeleton);
        {
            use melee_fighter::tu::ftdata::ext as fd;
            fd::GENO_DEFINE_BASE_KIND = Some(DefineBaseKind);
            fd::GENO_DEFINE_IS_NONE = Some(DefineIsNone);
            fd::GENO_DEFINE_TEMPLATE_SKELETON = Some(DefineTemplateSkeleton);
            fd::GENO_DEFINE_COUNT = Some(DefineCount);
            fd::GENO_DEFINE_CK_AT = Some(DefineCKAt);
            fd::GENO_DEFINE_COSTUME_COUNT = Some(DefineCostumeCount);
            fd::GENO_DEFINE_STRING = Some(DefineString);
            fd::GENO_DEFINE_PLAN_INT = Some(DefinePlanInt);
            fd::GENO_DEFINE_PLAN_TEXT = Some(DefinePlanText);
            fd::GENO_DEFINE_PARTNER_KIND = Some(DefinePartnerKind);
            fd::GENO_DEFINE_IDENTITY_WORD = Some(|k, w| DefineIdentityWord(k, w) as u32);
            fd::GENO_DEFINE_THROWN_AS = Some(DefineThrownAs);
            fd::GENO_DEFINE_THROWN_AS_ROW = Some(DefineThrownAsRow);
            fd::GENO_THROW_BANK_LIMIT = Some(ThrowBankLimit);
            // gw_PcCostumeAlloc: boot descriptors (a define's costume list and strings), zeroed, outside the match arena
            // (the C's persistent costume region: snapshots never move them)
            fd::PC_COSTUME_ALLOC = Some(|bytes| {
                assert!(bytes > 0, "skins255: descriptor pool exhausted");
                host_heap(|| Box::leak(vec![0u64; bytes.div_ceil(8)].into_boxed_slice()).as_mut_ptr() as *mut core::ffi::c_void)
            });
        }
        install_art();
        g::Geno_DefineBaseKind = Some(DefineBaseKind);
        g::Geno_DefineOverrideCount = Some(DefineOverrideCount);
        g::Geno_DefineOverrideField = Some(DefineOverrideField);
        g::Geno_DefineVoiceState = Some(|k, o, m| DefineVoiceState(k, o as u32, m));
        melee_fighter::hooks::GENO_DEFINE_VOICE_PLAY_OWNER = Some(|k, sfx, fp| DefineVoicePlayOwner(k, sfx, fp as usize as u32));
        g::Geno_LuaFn = Some(LuaFn);
        g::Geno_LuaSlots = Some(LuaSlots);
        g::Geno_LuaCall = Some(LuaCall);
        g::Geno_ComboParam = Some(ComboParam);
        g::Geno_ComboWord = Some(ComboWord);
        g::Geno_ComboNote = Some(ComboNote);
        t::Geno_DefineCount = Some(DefineCount);
        t::Geno_DefineCKAt = Some(DefineCKAt);
        t::Geno_ComboParam = Some(ComboParam);
        t::Geno_ComboWord = Some(ComboWord);
        t::Geno_MeterParam = Some(MeterParam);
        gm::Geno_DefineBaseCK = Some(DefineBaseCK);
        gm::Geno_DefineBaseKind = Some(DefineBaseKind);
        gm::Geno_DefineHasArt = Some(DefineHasArt);
        gm::Geno_DefineArtBaseCK = Some(DefineArtBaseCK);
        gm::Geno_DefineAudioPlay = Some(DefineAudioPlay);
        gm::Geno_DefineCostumeCountCK = Some(DefineCostumeCountCK);
        gm::Geno_DefineTeamCostume = Some(DefineTeamCostume);
        gm::Geno_DefineName = Some(DefineName);
        gm::Geno_RecordAdd = Some(RecordAdd);
        gm::Geno_RecordCommit = Some(RecordCommit);
        melee_runtime::tu::lbaudio_ax::ext::hook::Geno_DefineBaseCK = Some(DefineBaseCK);
        melee_world::tu::ifstock::ext::hook::Geno_DefineArtBaseCK = Some(DefineArtBaseCK);
        melee_world::tu::ifstock::ext::hook::Geno_DefineBaseCK = Some(DefineBaseCK);
        melee_world::tu::ifstock::ext::hook::Geno_DefineName = Some(DefineName);
        melee_fighter::tu::ftCo_0A01::ext::hook::Geno_DefineAiLike = Some(DefineAiLike);
        // GenoDefine_Load (ftdata) and the game half's row tables: the real retail tables (melee_link)
        {
            use melee_fighter::tu::ftdata::ext as fd;
            use melee_link::retail_ms as ms;
            fd::GENO_COMMON_ROWS = Some(|| (&raw mut ms::ftmotionstates::ftData_MotionStateList) as *mut melee_types::MotionState);
            fd::GENO_DEFINE_APPLY_ROWS = Some(|k, c, sp, n| melee_modes::tu::geno_game::GenoGame_DefineApplyRows(k, c as _, sp as _, n));
            fd::GENO_DEFINE_BUDGETS = Some(|kind| {
                use melee_script::geno::consts::*;
                let p = ProfileForKind(kind);
                [
                    OverlayCount(p), GENO_MAX_OVERLAYS as i32, ProfilePoolWords(p), PoolWords(), GENO_POOL_WORDS as i32, SlotCount(),
                    GENO_MAX_SLOTS as i32, StateCount(p), GENO_MAX_STATES as i32, melee_modes::tu::geno_game::Geno_ProfileStorageBytes() as i32,
                ]
            });
            use melee_modes::tu::geno_game as gg;
            gg::ext::ftData_MotionStateList = (&raw mut ms::ftmotionstates::ftData_MotionStateList) as *mut _;
            gg::ext::ftMr_Init_MotionStateTable = (&raw mut ms::ftmario::ftMr_Init_MotionStateTable) as *mut _;
            gg::ext::ftCa_Init_MotionStateTable = (&raw mut ms::ftcaptain::ftCa_Init_MotionStateTable) as *mut _;
            gg::ext::ftFx_Init_MotionStateTable = (&raw mut ms::ftfox::ftFx_Init_MotionStateTable) as *mut _;
            gg::ext::ftFc_Init_MotionStateTable = (&raw mut ms::ftfalco::ftFc_Init_MotionStateTable) as *mut _;
            gg::ext::ftMs_Init_MotionStateTable = (&raw mut ms::ftmars::ftMs_Init_MotionStateTable) as *mut _;
            gg::ext::ftPr_Init_MotionStateTable = (&raw mut ms::ftpurin::ftPr_Init_MotionStateTable) as *mut _;
            gg::ext::ftSk_Init_MotionStateTable = (&raw mut ms::ftseak::ftSk_Init_MotionStateTable) as *mut _;
            gg::ext::ftZd_Init_MotionStateTable = (&raw mut ms::ftzelda::ftZd_Init_MotionStateTable) as *mut _;
        }
        t::memcmp = Some(|a, b, n| {
            let (x, y) = (core::slice::from_raw_parts(a as *const u8, n), core::slice::from_raw_parts(b as *const u8, n));
            x.iter().zip(y).find(|(p, q)| p != q).map_or(0, |(p, q)| *p as i32 - *q as i32)
        });
        t::strcmp = Some(|a, b| {
            let (x, y) = (core::ffi::CStr::from_ptr(a as _).to_bytes(), core::ffi::CStr::from_ptr(b as _).to_bytes());
            x.iter().chain([0u8].iter()).zip(y.iter().chain([0u8].iter())).find(|(p, q)| p != q).map_or(0, |(p, q)| *p as i32 - *q as i32)
        });
        t::GenoDefine_FtDataFieldName = Some(|i| intern(melee_fighter::tu::ftdata::GenoDefine_FtDataFieldName(i)) as *const i8);
        t::GenoGame_DefinePPCViolation = Some(melee_fighter::tu::ftdata::GenoGame_DefinePPCViolation);
        t::GenoDefine_FtDataSharedMask = Some(|a, b| melee_fighter::tu::ftdata::GenoDefine_FtDataSharedMask(a as _, b as _));
    }
}

/// The Geno part of the netplay identity (gw_mexid.c's `gw_Geno_SaltForPlFile` per fighter and `mx_add_defines`):
/// every attached fighter's Pl file with its profile salt, and every define's online content id, read from the mods
/// `ids` under `dir` before boot. 0 when no geno.json changes anything, so a vanilla handshake is unchanged.
pub fn netplay_identity(dir: &std::path::Path, ids: &[&str]) -> u64 {
    let env = crate::mex_host::GenoEnv { dir: dir.to_path_buf() };
    let mut r = Registry::default();
    let mut ids = ids.to_vec();
    ids.sort();
    for id in &ids {
        let Ok(t) = std::fs::read(dir.join(id).join("geno.json")) else { continue };
        if t.len() <= 1 << 20 {
            r.parse_text(&t, id, &env);
        }
    }
    r.build_kinds(&env);
    let mut parts: Vec<String> = Vec::new();
    let mut files: Vec<String> = r.p.iter().filter(|p| !p.defined).map(|p| p.plfile.to_ascii_lowercase()).collect();
    files.sort();
    files.dedup();
    for f in files {
        let s = r.salt_for_pl_file(&f);
        if s != 0 {
            parts.push(format!("{f}={s:016x}"));
        }
    }
    for d in 0..r.define_count() {
        if let Some(o) = r.define_online_info(d) {
            // mx_add_defines: the resource files' bytes join the identity. A base "none" define's files are its mod's
            // (files/): one missing gives it no online identity. A base "mario" define's are the disc's (the disc
            // identity covers them).
            use melee_script::geno::Env as _;
            let mod_id = r.p[r.definitions.profile_for_kind(o.ck - 1).max(0) as usize].mod_id.clone();
            let mut fh = String::new();
            let mut ok = true;
            if o.none {
                for f in &o.files {
                    match env.read_mod_file(&mod_id, &format!("files/{f}")) {
                        Some(b) => fh.push_str(&format!(",{:016x}", b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &c| (h ^ c as u64).wrapping_mul(0x100_0000_01b3)))),
                        None => {
                            eprintln!("mexid: define {} has no online identity: resource file {f} is missing", o.key);
                            ok = false;
                            break;
                        }
                    }
                }
            }
            if ok {
                parts.push(format!("define {}={:016x}/{}{fh}", o.key, o.id, o.none as i32));
            }
        }
    }
    if parts.is_empty() {
        return 0;
    }
    parts.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, s| s.bytes().chain([0u8]).fold(h, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3)))
}

// ---- slice 6: a define's own menu / HUD art (gw_Geno_DefineArtOpen over gw_runtime.c's .gxtex handles) ----

/// gw_runtime.c's GwGxTex: a parsed .gxtex blob (big-endian header, GX-tiled payload).
struct GxTex {
    blob: Vec<u8>,
    format: u32,
    width: u32,
    height: u32,
    image_off: u32,
    image_size: u32,
}
/// GW_GXTEX_MAX open handles.
static mut GXTEX: [Option<GxTex>; 8] = [const { None }; 8];

fn gxtex_get(h: i32) -> Option<&'static GxTex> {
    unsafe { if (0..8).contains(&h) { GXTEX[h as usize].as_ref() } else { None } }
}
/// gw_gxtex_install: a v1 "GXTX" blob into a free handle, -1 when it is malformed or every handle is open.
fn gxtex_install(blob: Vec<u8>, label: &str) -> i32 {
    let be = |o: usize| u32::from_be_bytes(blob[o..o + 4].try_into().unwrap());
    let Some(i) = (0..8).find(|&i| unsafe { GXTEX[i].is_none() }) else {
        eprintln!("gxtex: no free slot for '{label}' (8 open)");
        return -1;
    };
    if blob.len() < 64 {
        eprintln!("gxtex: {label} is {} bytes, too short for a header", blob.len());
        return -1;
    }
    if be(0) != 0x4758_5458 || be(4) != 1 {
        eprintln!("gxtex: {label} is not a v1 .gxtex");
        return -1;
    }
    let (image_size, tlut_size, image_off, tlut_off) = (be(28), be(32), be(36), be(40));
    if image_off as u64 + image_size as u64 > blob.len() as u64 || (tlut_size != 0 && tlut_off as u64 + tlut_size as u64 > blob.len() as u64) {
        eprintln!("gxtex: {label}: image {image_size}@{image_off} tlut {tlut_size}@{tlut_off} do not fit in {} bytes", blob.len());
        return -1;
    }
    let (format, width, height, tl_n, tl_f) = (be(8), be(12), be(16), be(24), be(20));
    let t = GxTex { format, width, height, image_off, image_size, blob };
    eprintln!(
        "gxtex: {label} -> handle {i}: {}x{} fmt {}, image {} B, tlut {} entries fmt {}",
        t.width, t.height, t.format, t.image_size, tl_n, tl_f
    );
    unsafe { GXTEX[i] = Some(t) };
    i as i32
}
pub fn GxTex_Width(h: i32) -> i32 { gxtex_get(h).map_or(0, |t| t.width as i32) }
pub fn GxTex_Height(h: i32) -> i32 { gxtex_get(h).map_or(0, |t| t.height as i32) }
pub fn GxTex_Format(h: i32) -> i32 { gxtex_get(h).map_or(-1, |t| t.format as i32) }
pub fn GxTex_ImageSize(h: i32) -> i32 { gxtex_get(h).map_or(0, |t| t.image_size as i32) }
pub fn GxTex_CopyImage(h: i32, dst: *mut core::ffi::c_void) {
    if let Some(t) = gxtex_get(h) {
        if !dst.is_null() {
            unsafe { core::ptr::copy_nonoverlapping(t.blob.as_ptr().add(t.image_off as usize), dst as *mut u8, t.image_size as usize) };
        }
    }
}
pub fn GxTex_Close(h: i32) {
    if (0..8).contains(&h) {
        unsafe { GXTEX[h as usize] = None };
    }
}

/// gw_Geno_DefineArtOpen: a .gxtex handle for a define's art (`what` GN_PRES_*), -1 when none. No skins are installed
/// here, so it is the declared file of the mod's files/ (gn_pres_open_file); stock / emblem / results must be unpaletted.
pub fn DefineArtOpen(ck: i32, what: i32, costume: i32) -> i32 {
    use melee_script::geno::define::*;
    if !(0..GN_PRES_KINDS as i32).contains(&what) {
        return -1;
    }
    let plain = what as usize == GN_PRES_STOCK || what as usize == GN_PRES_EMBLEM || what as usize == GN_PRES_RESULTS;
    let r = built();
    let p = r.define_profile_ck(ck);
    if p < 0 {
        return -1;
    }
    let (mod_id, name) = (r.p[p as usize].mod_id.clone(), r.p[p as usize].name.clone());
    // results without its own file falls back to the portrait (gn_pres_file)
    let file = r.define_pres_file(ck, what, costume).map(str::to_string);
    let Some(file) = file else { return -1 };
    let env = crate::mex_host::GenoEnv { dir: dir_of_boot() };
    use melee_script::geno::Env as _;
    // on the system heap: a read into the copy-on-write arena fails (ERROR_NOACCESS) during a rollback match
    let Some(raw) = host_heap(|| env.read_mod_file(&mod_id, &format!("files/{file}"))) else {
        static mut SAID: Vec<String> = Vec::new();
        unsafe {
            if !SAID.contains(&file) && SAID.len() < 16 {
                SAID.push(file.clone());
                eprintln!("geno: {name}: presentation file {file} is not in the mod's files/");
            }
        }
        return -1;
    };
    let h = gxtex_install(raw, &file);
    if h >= 0 && plain && GxTex_Format(h) >= 8 {
        eprintln!("geno: {name}: presentation file {file} needs a format without a palette for stock/results");
        GxTex_Close(h);
        return -1;
    }
    h
}

/// The HUD's art callees (ifstock: the stock icon of a define).
pub fn install_art() {
    use melee_world::tu::ifstock::ext::hook as s;
    unsafe {
        s::Geno_DefineArtOpen = Some(DefineArtOpen);
        s::GxTex_Width = Some(GxTex_Width);
        s::GxTex_Height = Some(GxTex_Height);
        s::GxTex_Format = Some(GxTex_Format);
        s::GxTex_ImageSize = Some(GxTex_ImageSize);
        s::GxTex_CopyImage = Some(GxTex_CopyImage);
        s::GxTex_Close = Some(GxTex_Close);
        s::UI_TextI4 = Some(|t, w, h, st, d| unsafe { crate::uigen::UI_TextI4(t as _, w, h, st, d as _) });
    }
}

#[cfg(test)]
mod tests {
    /// The netplay identity covers a base "none" define: its resource files' bytes change it, and a missing one drops
    /// the define (mx_add_defines). Uses the text fixture pc/geno/mods/vanilla-courier with stand-in file bytes.
    #[test]
    fn identity_covers_base_none_files() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pc/geno/mods/vanilla-courier");
        let d = std::env::temp_dir().join(format!("genonone-id-{}", std::process::id()));
        let m = d.join("vanilla-courier");
        let _ = std::fs::remove_dir_all(&d);
        fn copy(a: &std::path::Path, b: &std::path::Path) {
            std::fs::create_dir_all(b).unwrap();
            for e in std::fs::read_dir(a).unwrap() {
                let e = e.unwrap();
                if e.file_type().unwrap().is_dir() { copy(&e.path(), &b.join(e.file_name())) } else { std::fs::copy(e.path(), b.join(e.file_name())).unwrap(); }
            }
        }
        copy(&src, &m);
        let files = ["GnCourierAJ.dat", "plan.json", "GnCourier_default.dat", "GnCourier_red.dat", "GnCourier_blue.dat", "GnCourier_green.dat"];
        for f in files {
            std::fs::write(m.join("files").join(f), f).unwrap();
        }
        let ids = ["vanilla-courier"];
        let a = super::netplay_identity(&d, &ids);
        std::fs::write(m.join("files/plan.json"), "other").unwrap();
        let b = super::netplay_identity(&d, &ids);
        std::fs::remove_file(m.join("files/plan.json")).unwrap();
        let c = super::netplay_identity(&d, &ids);
        let _ = std::fs::remove_dir_all(&d);
        assert_ne!(a, 0);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }
}

/// geno_define_tests.inc test_geno_gnsnd_audio and the audio-lifetime fixture, against the native clip slots.
#[cfg(test)]
mod clip_tests {
    use super::*;

    fn gnsnd() -> Vec<u8> {
        let mut b = vec![0u8; 40];
        b[0..4].copy_from_slice(b"GNSD");
        b[7] = 1;
        b[10] = 0x7D; // 32000
        b[15] = 1;
        b[19] = 4;
        b[23] = 32;
        b[27] = 8;
        b[32..40].copy_from_slice(&[0x12, 0x34, 0xFF, 0xFE, 0x80, 0x00, 0x7F, 0xFF]);
        b
    }

    #[test]
    fn gnsnd_decode_and_refusals() {
        let b = gnsnd();
        let (pcm, frames, ch) = gnsnd_decode(&b).unwrap();
        assert_eq!((frames, ch, pcm), (4, 1, vec![0x1234, -2, -32768, 32767]));
        for (i, v) in [(0usize, b'X'), (7, 2), (11, 1), (15, 3), (19, 5)] {
            let mut c = b.clone();
            c[i] = v;
            assert!(gnsnd_decode(&c).is_err(), "byte {i} = {v} decoded");
        }
        assert!(gnsnd_decode(&b[..39]).is_err() && gnsnd_decode(&b[..10]).is_err());
    }

    /// One test (the registry and the sound are process-wide): the C's two in-engine checks in order.
    #[test]
    fn define_clips_play_stop_and_state_exit_like_c() {
        melee_audio::sim::install(melee_audio::Sound::new());
        let stat = || melee_audio::sim::with(|s| s.ax.clip_stat(0)).unwrap();
        let stop = |pcm: &std::sync::Arc<[i16]>, o: u32| melee_audio::sim::with(|s| s.ax.stop_owned_clips(pcm, o)).unwrap();

        // test_geno_gnsnd_audio: declared but missing files answer "handled" and play nothing; undeclared is not handled
        let t = c"{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"t7\",\"name\":\"T7\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"audio\":{\"announcer\":\"gn_absent_call.gnsnd\",\"voice\":[{\"sfx\":77,\"file\":\"gn_absent_v.gnsnd\",\"volume\":80}]}}]}";
        assert_eq!(TestInstall(t.as_ptr()), 1);
        let ck = DefineResolve("t7");
        assert!(ck >= 34, "ck {ck}");
        let before = stat();
        assert_eq!((DefineAudioPlay(ck, 0, 0), DefineAudioPlay(ck, 1, 77), stat()), (1, 1, before));
        assert_eq!((DefineAudioPlay(ck, 1, 78), DefineAudioPlay(8, 0, 0), DefineVoicePlayOwner(0, 77, 0)), (0, 0, 0));
        // a loaded clip reaches the mixer (put in the announcer slot as the loader would)
        let p = built().define_profile_ck(ck) as usize;
        let (pcm, f, c) = gnsnd_decode(&gnsnd()).unwrap();
        gn_clip_slot(p, 0).pcm = Some((pcm.into(), f, c));
        assert_eq!((DefineAudioPlay(ck, 0, 0), stat()), (1, before + 1));
        // and is mixed: 4 frames at gain 127
        let out = melee_audio::sim::with(|s| *s.ax.run_frame(&[0u8; 64])).unwrap();
        assert_eq!(&out[0..8], &[0x1234, 0x1234, -2, -2, -32768, -32768, 32767, 32767]);
        TestRestore();
        gn_clips_clear();

        // the audio-lifetime fixture: a stoppable hum scoped to two states, a plain voice, a stoppable legacy row
        let t = c"{\"geno\":9,\"fighters\":[{\"define\":{\"key\":\"t7\",\"name\":\"T7\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"audio\":{\"voice\":[{\"sfx\":77,\"file\":\"hum.gnsnd\",\"stoppable\":true,\"while_states\":[\"Charge\",\"Fire\"]},{\"sfx\":78,\"file\":\"voice.gnsnd\"},{\"sfx\":79,\"file\":\"legacy.gnsnd\",\"stoppable\":true}]},\"states\":[{\"name\":\"Charge\",\"subaction\":57},{\"name\":\"Fire\",\"subaction\":58}]}]}";
        assert_eq!(TestInstall(t.as_ptr()), 1);
        let ck = DefineResolve("t7");
        assert!(ck >= 34);
        let (p, kind) = (built().define_profile_ck(ck) as usize, ck - 1);
        let tones: Vec<std::sync::Arc<[i16]>> = (1..=3).map(|k| vec![100 * k, -100 * k, 100 * k, -100 * k].into()).collect();
        for (i, t) in tones.iter().enumerate() {
            gn_clip_slot(p, 1 + i).pcm = Some((t.clone(), 4, 1));
        }
        let base = melee_script::gd::players::GENO_MOTION_BASE;
        let play = |sfx, o| DefineVoicePlayOwner(kind, sfx, o);
        play(77, 1);
        play(77, 1);
        play(77, 2);
        play(78, 1);
        play(79, 1);
        assert_eq!((DefineVoiceState(kind, 1, base), DefineVoiceState(kind, 1, base + 1), DefineVoiceState(0, 1, 14)), (0, 0, 0), "charge, fire or retail fighter cut hum");
        assert_eq!((DefineVoiceState(kind, 1, 14), DefineVoiceState(kind, 1, 14)), (2, 0));
        assert_eq!((stop(&tones[0], 2), stop(&tones[1], 1), stop(&tones[2], 1)), (1, 1, 1), "exit cuts the owner's hums only");
        play(77, 1);
        play(77, 2);
        play(78, 1);
        play(79, 1);
        assert_eq!(play(540001, 1), 1);
        assert_eq!((stop(&tones[0], 1), stop(&tones[2], 1), stop(&tones[0], 2), stop(&tones[1], 1)), (0, 0, 1, 1), "540001 stops only its fighter's stoppable clips");
        play(77, 1);
        assert_eq!(DefineVoiceState(kind, 1, -1), 1, "fighter reset left its hum playing");
        TestRestore();
        gn_clips_clear();
    }
}
