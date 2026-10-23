//! tu::gm_1BFA vs the C (state oracle): gen/turef/tu_gm_1BFA_ref.c is the game's gm_1BFA.c compiled
//! natively (shipping contraction) with its callees stubbed and logged; the expected lines are
//! tests/data/tu_gm_1BFA.txt (regenerate with `python rust/melee_modes/gen/turef/tu_shard457_gm.py
//! tu_gm_1BFA`). Every line is a rolling hash of the callee log (plus its first bytes), the data the
//! function writes and, for the prize list, the allocation pool and the flag table.
#![allow(static_mut_refs, non_upper_case_globals)]

use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null_mut};
use melee_modes::tu::gm_1BFA as t;
use melee_types::*;

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}

const NCASES: usize = 200;
const NPOOL: usize = 400;
const NTROPHY: usize = TY_TROPHY_COUNT as usize;
const FUNC_NAMES: [&str; 7] = ["ApproachOnEnter", "ApproachVsOnEnter", "onExitVs", "gm_801BFC60", "PrizeOnEnter", "onExitPrize", "table"];

static mut VSAPP: VsApproachData = unsafe { core::mem::zeroed() };
static mut START: StartMeleeData = unsafe { core::mem::zeroed() };
static mut EXIT: MatchExitInfo = unsafe { core::mem::zeroed() };
static mut PRIZE: un_804A1F48_t = unsafe { core::mem::zeroed() };
static mut POOL: [un_804A1F48_t; NPOOL] = unsafe { core::mem::zeroed() };
static mut NALLOC: usize = 0;
static mut ALLOC_FAIL_AFTER: usize = NPOOL;
static mut CH: ChallengerData = unsafe { core::mem::zeroed() };
static mut T804: [u32; 0x42] = [0; 0x42];
static mut T970: [u32; NTROPHY] = [0; NTROPHY];
static mut T1: [u8; 0x42] = [0; 0x42];
static mut T2: [i32; NTROPHY] = [0; NTROPHY];
static mut T3: [i32; NTROPHY] = [0; NTROPHY];
static mut T4: [i32; NTROPHY] = [0; NTROPHY];
static mut TICK: u32 = 0;
static mut PENDING_RET: i32 = 0;
static mut UNLOCK_IDX: i32 = 0;
static mut RAND_U8: i32 = 0;
static mut STKIND: i32 = 0;
static mut SLOT: *mut c_void = null_mut();

// ---- folded log (the C folds after every callee)
static mut H: u32 = 2166136261;
static mut NCALLS: i32 = 0;
static mut HEAD: String = String::new();

fn lg(s: &str) {
    unsafe {
        for b in s.bytes() {
            H = (H ^ b as u32).wrapping_mul(16777619);
            if HEAD.len() < 120 {
                HEAD.push(b as char);
            }
        }
        NCALLS += 1;
    }
}

fn install_hooks() {
    use t::ext::*;
    unsafe {
        GM_GETCHALLENGERDATA = Some(|| {
            lg("chal");
            addr_of_mut!(CH)
        });
        GM_SETUPRULESDEFAULTS = Some(|_r| lg("rules"));
        GM_SETUPALLPLAYERDEFAULTS = Some(|_p| lg("players"));
        GM_GETCHALLENGERSTKIND = Some(|k| {
            lg(&format!("stkind({})", k));
            STKIND as StKind
        });
        GM_CKINDTOSELKIND = Some(|ck| {
            lg(&format!("sel({})", ck));
            ck ^ 0x5A
        });
        GM_SETUPHUMANPLAYER = Some(|p, ck, color, stocks, slot| {
            let idx = (p as usize - addr_of!(START.players) as usize) / core::mem::size_of::<PlayerInitData>();
            lg(&format!("human({},{},{},{},{})", idx, ck, color, stocks, slot));
        });
        GM_SETUPCPUPLAYER = Some(|p, ck, color, stocks, slot| {
            let idx = (p as usize - addr_of!(START.players) as usize) / core::mem::size_of::<PlayerInitData>();
            lg(&format!("cpu({},{},{},{},{})", idx, ck, color, stocks, slot));
        });
        GM_DECIDECHALLENGERCPULEVEL = Some(|ck, tag| {
            lg(&format!("lvl({},{})", ck, tag));
            RAND_U8 as u8
        });
        GM_LOADRUMBLEENABLED = Some(|s| lg(&format!("rumble({})", (s == addr_of_mut!(START)) as i32)));
        GM_80162968 = Some(|s| {
            lg(&format!("sec({})", s));
            0
        });
        GM_8016247C = Some(|a| {
            lg(&format!("xE({})", a));
            0
        });
        GM_UNLOCKCKIND = Some(|ck| lg(&format!("unlock({})", ck)));
        GMMAINLIB_8015DB2C = Some(|a| lg(&format!("relock({})", a)));
        GM_CKINDTOUNLOCKINDEX = Some(|ck| {
            lg(&format!("uidx({})", ck));
            UNLOCK_IDX as i8
        });
        GM_80173EEC = Some(|| lg("3EEC"));
        GM_80172898 = Some(|a| {
            lg(&format!("2898({})", a));
            0
        });
        GM_801721EC = Some(|| {
            lg("21EC");
            PENDING_RET != 0
        });
        GM_SETPENDINGGAMEMODE = Some(|m| lg(&format!("pend({})", m)));
        GM_SETNEWGAMEMODEPENDING = Some(|| lg("newpend"));
        TOY_803124BC = Some(|| lg("toy124BC"));
        GM_8017219C = Some(|i| {
            lg(&format!("have({})", i));
            T1[i as usize] != 0
        });
        GMMAINLIB_8015D804 = Some(|i| {
            lg(&format!("d804({})", i));
            addr_of_mut!(T804[i as usize])
        });
        GMMAINLIB_8015DA1C = Some(|i| {
            lg(&format!("da1c({})", i));
            T2[i as usize]
        });
        GMMAINLIB_8015DA90 = Some(|i| {
            lg(&format!("da90({})", i));
            T3[i as usize]
        });
        GMMAINLIB_8015D970 = Some(|i| {
            lg(&format!("d970({})", i));
            addr_of_mut!(T970[i as usize])
        });
        GMMAINLIB_8015DA40 = Some(|i| lg(&format!("da40({})", i)));
        LBTIME_GETTIMEINSECONDS = Some(|| {
            lg("time");
            TICK = TICK.wrapping_add(7);
            TICK
        });
        TOY_SETUNLOCKSTATE = Some(|id, v| lg(&format!("setunl({},{})", id, v as i32)));
        TOY_803048C0 = Some(|i| {
            lg(&format!("toy48C0({})", i));
            T4[i as usize]
        });
        GM_80172174 = Some(|| lg("2174"));
        GM_80174180 = Some(|| lg("4180"));
        HSD_MEMALLOC = Some(|size| {
            lg(&format!("alloc({})", size));
            if NALLOC < NPOOL && NALLOC < ALLOC_FAIL_AFTER {
                let p = addr_of_mut!(POOL[NALLOC]);
                NALLOC += 1;
                p as *mut c_void
            } else {
                null_mut()
            }
        });
        LBCARDNEW_ALLOCWORKAREA = Some(|| lg("allocwork"));
        LBCARDGAME_LOADARCHIVE = Some(|a| lg(&format!("cardload({})", a)));
        LBCARDGAME_SAVECHANGES = Some(|| lg("cardsave"));
        GMVSMELEE_APPROACHDATA = Some(|| addr_of_mut!(VSAPP_GLOBAL));
        GMVSMELEE_STARTDATA = Some(|| addr_of_mut!(START));
        GMVSMELEE_VSEXITINFO = Some(|| addr_of_mut!(EXIT));
        IF_SCENE_PRIZE_ENTERDATA = Some(|| addr_of_mut!(PRIZE));
    }
}

/// `gmVsMelee_ApproachData` (the C driver's global; distinct from the per-case `VSAPP` scene data).
static mut VSAPP_GLOBAL: VsApproachData = unsafe { core::mem::zeroed() };

fn fold_reset() {
    unsafe {
        H = 2166136261;
        NCALLS = 0;
        HEAD.clear();
    }
}

fn fill_bytes(r: &mut Rng, p: *mut u8, n: usize) {
    for i in 0..n {
        unsafe { *p.add(i) = r.rnd() as u8 };
    }
}

fn idp(p: *const c_void) -> String {
    unsafe {
        if p.is_null() {
            return "null".into();
        }
        if p == addr_of!(VSAPP_GLOBAL) as *const c_void {
            return "APP".into();
        }
        if p == addr_of!(START) as *const c_void {
            return "START".into();
        }
        if p == addr_of!(EXIT) as *const c_void {
            return "EXIT".into();
        }
        if p == addr_of!(PRIZE) as *const c_void {
            return "PRIZE".into();
        }
        if p == addr_of!(PRIZE.x8) as *const c_void {
            return "PRIZE.x8".into();
        }
        if p == addr_of!(SLOT) as *const c_void {
            return "SLOT".into();
        }
        for i in 0..NPOOL {
            if p == addr_of!(POOL[i]) as *const c_void {
                return format!("P{}", i);
            }
            if p == addr_of!(POOL[i].x8) as *const c_void {
                return format!("P{}.x8", i);
            }
        }
    }
    "?".into()
}

fn fname(f: Option<unsafe extern "C" fn(*mut GameModeState)>) -> &'static str {
    match f {
        None => "-",
        Some(f) => {
            let a = f as usize;
            if a == t::gm_ModeState_Approach_OnEnter as usize {
                "A"
            } else if a == t::gm_ModeState_ApproachVs_OnEnter as usize {
                "V"
            } else if a == t::onExitVs as usize {
                "XV"
            } else if a == t::gm_ModeState_Prize_OnEnter as usize {
                "P"
            } else if a == t::onExitPrize as usize {
                "XP"
            } else {
                "?"
            }
        }
    }
}

fn fnv_bytes(p: &[u8]) -> u32 {
    let mut h: u32 = 2166136261;
    for &b in p {
        h = (h ^ b as u32).wrapping_mul(16777619);
    }
    h
}

fn bytes_of<T>(x: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(x as *const T as *const u8, core::mem::size_of::<T>()) }
}

fn setup_case(r: &mut Rng, f: usize, scene: &mut GameModeState) {
    unsafe {
        CH.human_ckind = r.rnd() as u8;
        CH.human_color = r.rnd() as u8;
        CH.human_slot = r.rnd() as u8;
        CH.human_nametag = r.rnd() as u8;
        CH.cpu_ckind = r.rnd() as u8;
        CH.curr_mode = r.rnd() as u8;
        CH.prev_mode = r.rnd() as u8;
        CH.x8 = r.rnd() as u16;
        STKIND = (r.rnd() % 40) as i32;
        RAND_U8 = (r.rnd() & 0xFF) as i32;
        UNLOCK_IDX = (r.rnd() % 30) as i32 - 5;
        PENDING_RET = (r.rnd() & 1) as i32;
        NALLOC = 0;
        ALLOC_FAIL_AFTER = if r.rnd() % 4 == 0 { (r.rnd() % 30) as usize } else { NPOOL };
        TICK = r.rnd() & 0xFFFF;
        for i in 0..0x42 {
            T1[i] = (r.rnd() % 3 != 0) as u8;
            T804[i] = r.rnd();
        }
        for i in 0..NTROPHY {
            T2[i] = (r.rnd() % 3) as i32;
            T3[i] = (r.rnd() % 2) as i32;
            T4[i] = (r.rnd() % 4) as i32;
            T970[i] = r.rnd();
        }
        fill_bytes(r, addr_of_mut!(POOL) as *mut u8, core::mem::size_of_val(&POOL));
        fill_bytes(r, addr_of_mut!(PRIZE) as *mut u8, core::mem::size_of::<un_804A1F48_t>());
        PRIZE.x8 = null_mut();
        for i in 0..NPOOL {
            POOL[i].x8 = null_mut();
        }
        fill_bytes(r, addr_of_mut!(t::gm_8049E558) as *mut u8, 0x170);
        fill_bytes(r, addr_of_mut!(VSAPP) as *mut u8, core::mem::size_of::<VsApproachData>());
        fill_bytes(r, addr_of_mut!(START) as *mut u8, core::mem::size_of::<StartMeleeData>());
        fill_bytes(r, addr_of_mut!(EXIT) as *mut u8, core::mem::size_of::<MatchExitInfo>());
        EXIT.match_end.outcome = (r.rnd() % 10) as u8;
        EXIT.match_end.player_standings[0].stocks = (r.rnd() % 5) as i32 as i8 - 1;
        SLOT = addr_of_mut!(CH) as *mut c_void;
        *scene = GameModeState::zeroed();
        match f {
            0 => scene.info.enter_data = addr_of_mut!(VSAPP) as *mut c_void,
            1 => scene.info.enter_data = addr_of_mut!(START) as *mut c_void,
            2 => scene.info.exit_data = addr_of_mut!(EXIT) as *mut c_void,
            4 => scene.info.enter_data = addr_of_mut!(PRIZE) as *mut c_void,
            _ => {}
        }
    }
    fold_reset();
}

fn log_part() -> String {
    unsafe { format!("|log={:08x},{},{}", H, NCALLS, HEAD) }
}

fn pool_part() -> String {
    unsafe {
        let mut s = format!(
            "|prize={},{},{},{}|nalloc={}",
            PRIZE.x0,
            PRIZE.x2,
            PRIZE.x4,
            idp(PRIZE.x8),
            NALLOC
        );
        let mut h: u32 = 2166136261;
        for i in 0..NALLOC {
            let w: [u32; 4] = [
                POOL[i].x0 as u32,
                POOL[i].x2 as u32,
                POOL[i].x4 as u32,
                if POOL[i].x8.is_null() {
                    u32::MAX
                } else {
                    ((POOL[i].x8 as usize - addr_of!(POOL) as usize) / core::mem::size_of::<un_804A1F48_t>()) as u32
                },
            ];
            let mut bytes = [0u8; 16];
            for (k, v) in w.iter().enumerate() {
                bytes[k * 4..k * 4 + 4].copy_from_slice(&v.to_le_bytes());
            }
            h = (h ^ fnv_bytes(&bytes)).wrapping_mul(16777619);
        }
        s.push_str(&format!("|pool={:08x}", h));
        s
    }
}

#[test]
fn matches_c() {
    let want = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tu_gm_1BFA.txt")).unwrap();
    install_hooks();
    let mut r = Rng(0x2468ACE);
    let mut lines = want.lines();
    let mut scene = GameModeState::zeroed();
    for f in 0..FUNC_NAMES.len() {
        let ncases = if f == 6 { 1 } else { NCASES };
        for i in 0..ncases {
            let mut got = format!("{}.{} {}", f, i, FUNC_NAMES[f]);
            if f == 6 {
                let tbl = t::gm_Mode_ChallengerApproach_States();
                for k in 0..4 {
                    got.push_str(&format!(
                        "|{}:{},{},{},{},{},{},{},{}",
                        k,
                        tbl[k].id,
                        tbl[k].preload,
                        tbl[k].flags,
                        fname(tbl[k].on_enter),
                        fname(tbl[k].on_exit),
                        tbl[k].info.scene_kind,
                        idp(tbl[k].info.enter_data),
                        idp(tbl[k].info.exit_data)
                    ));
                }
            } else {
                setup_case(&mut r, f, &mut scene);
                let s: *mut GameModeState = &mut scene;
                unsafe {
                    match f {
                        0 => {
                            t::gm_ModeState_Approach_OnEnter(s);
                            got.push_str(&log_part());
                            got.push_str(&format!(
                                "|vs={},{}|ch={},{},{},{},{},{},{},{}",
                                VSAPP.cpu_ckind,
                                VSAPP.human_slot,
                                CH.human_ckind,
                                CH.human_color,
                                CH.human_slot,
                                CH.human_nametag,
                                CH.cpu_ckind,
                                CH.curr_mode,
                                CH.prev_mode,
                                CH.x8
                            ));
                        }
                        1 => {
                            t::gm_ModeState_ApproachVs_OnEnter(s);
                            got.push_str(&log_part());
                            let rl = &START.rules;
                            got.push_str(&format!(
                                "|rules={},{},{},{},{},{},{},{},{},{},{},{}",
                                rl.timer_enabled(),
                                rl.match_kind(),
                                rl.x0_3(),
                                rl.x6,
                                rl.x7,
                                rl.is_teams,
                                rl.x9,
                                rl.xA as i32,
                                rl.item_freq as i32,
                                rl.xD,
                                rl.stkind,
                                rl.time_limit
                            ));
                            for k in 0..3 {
                                let p = &START.players[k];
                                got.push_str(&format!(
                                    "|p{}={},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                                    k,
                                    p.ckind as i32,
                                    p.slot_type,
                                    p.stocks as i32,
                                    p.color,
                                    p.slot,
                                    p.spawn_pos as i32,
                                    p.spawn_dir as i32,
                                    p.sub_color,
                                    p.handicap as i32,
                                    p.team,
                                    p.nametag,
                                    p.xB,
                                    p.cpu_kind,
                                    p.cpu_level,
                                    p.damage,
                                    p.damage1,
                                    p.hp
                                ));
                            }
                        }
                        2 => {
                            t::onExitVs(s);
                            got.push_str(&log_part());
                        }
                        3 => {
                            let a0 = r.rnd();
                            let a1: i32 = if r.rnd() % 3 == 0 { 0 } else { (r.rnd() % 5) as i32 + 1 };
                            let a2 = r.rnd();
                            let a3 = r.rnd();
                            let ret = t::gm_801BFC60(a0, a1, a2, a3, addr_of_mut!(SLOT));
                            got.push_str(&format!(
                                "|arg={},{},{},{}|ret={}",
                                a0,
                                a1,
                                a2,
                                a3,
                                if ret == addr_of_mut!(SLOT) { "A".to_string() } else { idp(ret as *const c_void) }
                            ));
                            got.push_str(&log_part());
                            got.push_str(&format!(
                                "|slot={}",
                                if SLOT == addr_of_mut!(CH) as *mut c_void { "CH".to_string() } else { idp(SLOT) }
                            ));
                            got.push_str(&pool_part());
                        }
                        4 => {
                            t::gm_ModeState_Prize_OnEnter(s);
                            got.push_str(&log_part());
                            got.push_str(&pool_part());
                            got.push_str(&format!(
                                "|flags={:08x}|t970={:08x}",
                                fnv_bytes(&t::gm_8049E558),
                                fnv_bytes(bytes_of(&T970))
                            ));
                        }
                        _ => {
                            t::onExitPrize(s);
                            got.push_str(&log_part());
                        }
                    }
                }
            }
            let line = lines.next().expect("fewer lines than cases");
            assert_eq!(got, line, "function {} case {}", FUNC_NAMES[f], i);
        }
    }
    assert!(lines.next().is_none(), "more lines than cases");
}
