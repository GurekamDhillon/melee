//! Geno's native half for the engine: the registry (melee_script::geno, ported from pc/platform/geno_registry.c)
//! behind the scalar API the game half (melee_modes::tu::geno_game) calls, `gw_Geno_*` in the C. One registry per
//! process: the boot one (every mounted mod's geno.json, MELEE_GENO_MODS) or, under the test suite, one parsed from a
//! test's text (`Geno_TestInstall` / `Geno_TestRestore`).
#![allow(non_snake_case, static_mut_refs)]
use melee_script::geno::consts::*;
use melee_script::geno::registry::{Profile, Registry};
use melee_script::geno::NoEnv;

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

/// The installed registry; the boot one is read on first use from the mods named by `MELEE_GENO_MODS`
/// (`id=dir;id=dir...`, each dir holding a geno.json; mount order).
fn reg() -> &'static mut Registry {
    unsafe {
        if BOOT.is_none() {
            let mut r = Registry::default();
            if let Ok(list) = std::env::var("MELEE_GENO_MODS") {
                for item in list.split(';').filter(|s| !s.is_empty()) {
                    let (id, dir) = item.split_once('=').unwrap_or((item, item));
                    let id = if id == dir { std::path::Path::new(dir).file_name().and_then(|s| s.to_str()).unwrap_or(dir) } else { id };
                    match std::fs::read(std::path::Path::new(dir).join("geno.json")) {
                        Ok(t) => {
                            r.parse_text(&t, id, &NoEnv);
                        }
                        Err(e) => eprintln!("geno: {dir}/geno.json: {e}"),
                    }
                }
                if r.files > 0 {
                    let n = r.p.len();
                    r.log.push(format!("geno: {} geno.json file(s), {} fighter entr{}", r.files, n, if n == 1 { "y" } else { "ies" }));
                }
            }
            r.build_kinds(&NoEnv);
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
    let k = r.profile_for_kind(kind, &NoEnv);
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
