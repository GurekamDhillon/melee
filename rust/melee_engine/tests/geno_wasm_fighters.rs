//! geno 12: the fighter modules (rust/mods/geno_fighters: goku, goku_ki, charger, riposte, and the game-half suite's
//! fixtures tests/charge, counter, empty) against the fighter Lua they replace. Each was compared word for word with
//! its Lua through the engine's GLua over the io blocks below (every callback, the 64 io words and the return code)
//! before Lua left the port (commits e1c938332 .. 8a98e4134); the digests of what the Lua produced over those blocks
//! were recorded then and are what each module must reproduce now. The Lua texts stay as the reference:
//! tools/rust-port/golden/coverage/goku-xv2-lua, pc/geno/mods/vanilla-{charger,riposte}/lua, pc/geno/geno_tests.c.
//! A changed module that still matches the Lua would match these digests; any other change fails here.
use melee_engine::geno_wasm;
use melee_script::geno::code::{FighterCode, GLUA_BOOL, GLUA_FLOAT, GLUA_INT};
use melee_script::geno::consts::*;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const FNS: [&str; 12] = [
    "charge_enter", "charge_frame", "rush_enter", "rush_frame", "rushcombo_enter", "rushcombo_frame", "recovery_enter",
    "counter_frame", "riposte_enter", "riposte_frame", "kicharge_frame", "it_targets",
];
const GOKU_SLOTS: [&str; 3] = ["charge", "counter_damage", "stored"];
const GOKU_TYPES: [i32; 3] = [GLUA_INT, GLUA_FLOAT, GLUA_INT];
const GOKU_TARGETS: [&str; 8] = ["KameTap", "KameMid", "KameFull", "KameSuper", "KiCharge", "ITEnd", "ITEndAir", "RushCombo"];

/// FNV-1a over a call's return code and its 64 io words (as recorded from the Lua).
fn fold(mut h: u64, rc: i32, io: &[u32]) -> u64 {
    for w in std::iter::once(rc as u32).chain(io.iter().copied()) {
        for b in w.to_le_bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100_0000_01b3);
        }
    }
    h
}

/// Runs `fns` of a module over `blocks` io blocks from `seed` (`fill` adjusts each block, as when the digest was
/// recorded), returns the digest and the number of faulting calls.
#[allow(clippy::too_many_arguments)]
fn run(wasm: &str, names: &[&str], types: &[i32], targets: &[&str], fns: &[&str], seed: u64, fill: fn(&mut Rng, &mut [u32]), blocks: usize) -> (u64, usize) {
    let names: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    let targets: Vec<String> = targets.iter().map(|s| s.to_string()).collect();
    let mut w = geno_wasm::load(&std::fs::read(format!("{ROOT}/{wasm}")).unwrap(), &names, types, &targets).unwrap();
    let mut r = Rng(seed);
    let (mut digest, mut faults) = (0xcbf2_9ce4_8422_2325u64, 0);
    for _ in 0..blocks {
        let mut base = io(&mut r);
        fill(&mut r, &mut base);
        for name in fns {
            let f = FighterCode::find(&w, name);
            assert!(f >= 0, "{wasm}: {name} is not exported");
            let mut b = base;
            let rc = unsafe { FighterCode::call(&mut w, f, b.as_mut_ptr()) };
            faults += (rc != 0) as usize;
            digest = fold(digest, rc, &b);
        }
    }
    (digest, faults)
}

fn none(_: &mut Rng, _: &mut [u32]) {}

fn layout() -> (Vec<String>, Vec<i32>) {
    (GOKU_SLOTS.iter().map(|s| s.to_string()).collect(), GOKU_TYPES.to_vec())
}

#[test]
fn goku_matches_goku_lua() {
    let (d, faults) = run("rust/mods/geno_fighters/goku.wasm", &GOKU_SLOTS, &GOKU_TYPES, &GOKU_TARGETS, &FNS, 0x6F6B_7567_0001, none, 4000);
    assert_eq!((d, faults), (0xd63d3f2354c2a60f, 0), "goku.lua's outputs");
}

#[test]
fn goku_missing_targets_fault_as_lua() {
    let (d, faults) = run("rust/mods/geno_fighters/goku.wasm", &GOKU_SLOTS, &GOKU_TYPES, &["KameTap", "KameMid", "ITEnd"], &FNS, 0x6F6B_7567_0002, none, 2000);
    assert!(faults > 0);
    assert_eq!(d, 0x113adab9fe113560, "goku.lua's outputs and faults with missing states");
}

const VSEED: u64 = 0x7661_6E69_6C6C_61;

#[test]
fn goku_ki_matches_goku_ki_lua() {
    let (d, _) = run("rust/mods/geno_fighters/goku_ki.wasm", &GOKU_SLOTS, &GOKU_TYPES, &GOKU_TARGETS, &FNS, VSEED, |r, w| {
        w[GENO_LUA_IO_KI as usize] = [0, 999, 1000, 1001, 9999, 10000, 10001, 15000][r.below(8) as usize];
        w[GENO_LUA_IO_ACTION_FRAME as usize] = [1, 2, 3, 12, 20][r.below(5) as usize];
    }, 4000);
    assert_eq!(d, 0x6e44daeac34f2adb, "goku_ki.lua's outputs");
}

#[test]
fn charger_matches_charger_lua() {
    let (d, _) = run("rust/mods/geno_fighters/charger.wasm", &["charge", "charged"], &[GLUA_INT, GLUA_BOOL], &["Charge", "Release"],
        &["charge_enter", "charge_frame", "release_enter", "release_frame"], VSEED, |r, w| {
            w[GENO_LUA_IO_STATE as usize] = [0, 1, 30, 58, 59, 60][r.below(6) as usize];
            w[GENO_LUA_IO_STATE as usize + 1] = r.below(2) as u32;
        }, 4000);
    assert_eq!(d, 0xe58091fe960c36a0, "charger.lua's outputs");
}

#[test]
fn riposte_matches_riposte_lua() {
    let (d, _) = run("rust/mods/geno_fighters/riposte.wasm", &["power", "streak", "follow"], &[GLUA_FLOAT, GLUA_INT, GLUA_BOOL],
        &["Parry", "Riposte", "Follow"], &["parry_enter", "parry_frame", "riposte_enter", "riposte_frame", "follow_enter", "follow_frame"], VSEED, |r, w| {
            w[GENO_LUA_IO_ACTION_FRAME as usize] = [0, 5, 6, 7, 8, 9, 16, 17, 23, 24, 25, 29, 30, 31][r.below(14) as usize];
            w[GENO_LUA_IO_HIT_DAMAGE as usize] = match r.below(4) {
                0 => 6.0f32.to_bits(),
                1 => 20.0f32.to_bits(),
                _ => r.f(0.0, 40.0),
            };
            w[GENO_LUA_IO_HIT_FROM as usize] = r.below(3) as u32;
            w[GENO_LUA_IO_STATE as usize] = r.f(0.0, 35.0);
            w[GENO_LUA_IO_STATE as usize + 1] = r.below(6) as u32;
            w[GENO_LUA_IO_STATE as usize + 2] = r.below(2) as u32;
        }, 4000);
    assert_eq!(d, 0xa4e3ce887521d5a6, "riposte.lua's outputs");
}

/// The game-half suite's fixtures against the inline Lua it carried (pc/geno/geno_tests.c).
#[test]
fn suite_fixtures_match_their_lua() {
    let (d, _) = run("rust/mods/geno_fighters/tests/charge.wasm", &["charge", "armed"], &[GLUA_INT, GLUA_BOOL], &["Charge", "Release", "Bad"],
        &["enter", "frame", "rel", "relf", "boom"], VSEED, |r, w| {
            w[GENO_LUA_IO_STATE as usize] = [0, 2, 3, 59, 60, 61][r.below(6) as usize];
            w[GENO_LUA_IO_STATE as usize + 1] = r.below(2) as u32;
        }, 3000);
    assert_eq!(d, 0xa666dfc78fe14bfa, "charge fixture");
    let (d, _) = run("rust/mods/geno_fighters/tests/counter.wasm", &["power", "seen", "from"], &[GLUA_FLOAT, GLUA_INT, GLUA_INT], &["Parry", "Answer", "Spin"],
        &["answer", "strike", "spin"], VSEED, |r, w| {
            w[GENO_LUA_IO_COUNTERED as usize] = r.below(3) as u32;
            w[GENO_LUA_IO_HIT_FROM as usize] = r.below(3) as u32;
            w[GENO_LUA_IO_HIT_DAMAGE as usize] = match r.below(3) { 0 => 6.0f32.to_bits(), 1 => 20.0f32.to_bits(), _ => r.f(0.0, 40.0) };
        }, 3000);
    assert_eq!(d, 0x778d9e9395438501, "counter fixture");
    let (d, _) = run("rust/mods/geno_fighters/tests/empty.wasm", &["armed"], &[GLUA_INT], &["Light2", "Heavy"], &["f"], VSEED, none, 3000);
    assert_eq!(d, 0x18d33f5d9bd80121, "empty fixture");
}

/// xorshift64*: the io blocks are a fixed sequence, so a failure reproduces
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn f(&mut self, lo: f32, hi: f32) -> u32 {
        (lo + (hi - lo) * (self.below(1 << 24) as f32 / (1u32 << 24) as f32)).to_bits()
    }
}

/// An io block as the game half fills it, with values around the script's thresholds.
fn io(r: &mut Rng) -> [u32; GENO_LUA_IO_WORDS as usize] {
    let mut w = [0u32; GENO_LUA_IO_WORDS as usize];
    w[GENO_LUA_IO_FLAGS as usize] = r.below(8) as u32;
    w[GENO_LUA_IO_ACTION_FRAME as usize] = [0, 7, 8, 11, 12, 13, 15, 19, 20, 21, 22, 40][r.below(12) as usize];
    w[GENO_LUA_IO_MOTION as usize] = r.below(400) as u32;
    w[GENO_LUA_IO_HELD as usize] = held(r);
    w[GENO_LUA_IO_PRESSED as usize] = held(r);
    w[GENO_LUA_IO_STICK_X as usize] = r.f(-1.0, 1.0);
    w[GENO_LUA_IO_STICK_Y as usize] = r.f(-1.0, 1.0);
    w[GENO_LUA_IO_VEL_X as usize] = r.f(-3.0, 3.0);
    w[GENO_LUA_IO_VEL_Y as usize] = r.f(-4.0, 4.0);
    w[GENO_LUA_IO_PERCENT as usize] = r.f(0.0, 200.0);
    // hit damage across both clamps (10 / 1.25 = 8, 18 / 1.25 = 14.4) and exactly on them
    w[GENO_LUA_IO_HIT_DAMAGE as usize] = match r.below(4) {
        0 => 8.0f32.to_bits(),
        1 => 14.4f32.to_bits(),
        _ => r.f(0.0, 30.0),
    };
    w[GENO_LUA_IO_KI as usize] = r.below(20000) as u32;
    // state: charge and stored around 10 / 20 / 45 / 60, counter damage a float
    let charge = [0, 9, 10, 19, 20, 44, 45, 59, 60, 61][r.below(10) as usize];
    w[GENO_LUA_IO_STATE as usize] = charge;
    w[GENO_LUA_IO_STATE as usize + 1] = r.f(0.0, 20.0);
    w[GENO_LUA_IO_STATE as usize + 2] = [0, 5, 10, 33, 60][r.below(5) as usize];
    // stale commands and fault word from a previous call: both must reset them
    w[GENO_LUA_IO_NCMDS as usize] = 3;
    w[GENO_LUA_IO_FAULT as usize] = 9;
    for i in GENO_LUA_IO_CMDS as usize..w.len() {
        w[i] = r.next() as u32;
    }
    w
}

fn held(r: &mut Rng) -> u32 {
    let all = [GENO_BTN_ATTACK, GENO_BTN_SPECIAL, GENO_BTN_JUMP, GENO_BTN_SHIELD, GENO_BTN_GRAB, GENO_BTN_BLAST];
    all.iter().filter(|_| r.below(2) == 1).fold(0, |a, b| a | b)
}

#[test]
fn module_rules() {
    let (names, types) = layout();
    let t: Vec<String> = vec![];
    assert!(geno_wasm::load(b"\0asm\x01\0\0\0", &names, &types, &t).is_err(), "no exported memory");
    assert!(geno_wasm::load(b"nonsense", &names, &types, &t).is_err());
    // a module importing outside gd_geno is refused (wat: one env import)
    let env_import = [
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // magic, version
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // type section: () -> ()
        0x02, 0x09, 0x01, 0x03, b'e', b'n', b'v', 0x01, b'f', 0x00, 0x00, // import env.f (func, type 0)
    ];
    let e = geno_wasm::load(&env_import, &names, &types, &t).err().unwrap();
    assert!(e.contains("only gd_geno"), "{e}");
}

/// A mod folder's files for the registry (geno.json's `wasm.module` is read through it) and the engine's wasm loader.
struct DirEnv(std::collections::HashMap<String, Vec<u8>>);
impl melee_script::geno::Env for DirEnv {
    fn read_mod_file(&self, _mod_id: &str, rel: &str) -> Option<Vec<u8>> {
        self.0.get(rel).cloned()
    }
    fn fighter_wasm(&self, m: &[u8], n: &[String], t: &[i32], g: &[String]) -> Result<Box<dyn FighterCode>, String> {
        geno_wasm::load(m, n, t, g).map(|w| Box::new(w) as Box<dyn FighterCode>)
    }
}

fn define(version: i32, code: &str, state_code: &str) -> String {
    format!(
        "{{\"geno\":{version},\"fighters\":[{{\"define\":{{\"key\":\"wasm-goku\",\"name\":\"Wasm Goku\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}{code},\
\"states\":[{{\"name\":\"KameCharge\",\"behavior\":\"geno.ground\"{state_code}}},{{\"name\":\"KameTap\",\"behavior\":\"geno.ground\"}},\
{{\"name\":\"KameMid\",\"behavior\":\"geno.ground\"}},{{\"name\":\"KameFull\",\"behavior\":\"geno.ground\"}}]}}]}}"
    )
}

#[test]
fn registry_wasm_block() {
    use melee_script::geno::Registry;
    let wasm = std::fs::read(format!("{ROOT}/rust/mods/geno_fighters/goku.wasm")).unwrap();
    let env = DirEnv([("wasm/goku.wasm".to_string(), wasm)].into_iter().collect());
    let block = ",\"wasm\":{\"module\":\"wasm/goku.wasm\",\"state\":{\"charge\":\"int\",\"counter_damage\":\"float\",\"stored\":\"int\"}}";
    let st = ",\"wasm\":{\"enter\":\"charge_enter\",\"frame\":\"charge_frame\"}";
    let mut r = Registry::default();
    assert_eq!(r.parse_text(define(12, block, st).as_bytes(), "wasm-goku", &env), 1, "{:?}", r.log);
    assert!(r.lua_fn(0, 0, GENO_LUA_PHASE_ENTER as i32) && r.lua_fn(0, 0, GENO_LUA_PHASE_FRAME as i32) && !r.lua_fn(0, 1, 0));
    assert_eq!((r.lua_slots(0), r.lua_slot_name(0, 1), r.lua_slot_type(0, 1)), (3, "counter_damage", GLUA_FLOAT));
    // a frame with the charge at 45 and special released: go KameFull (state 3 of the define)
    let mut io = [0u32; GENO_LUA_IO_WORDS as usize];
    io[GENO_LUA_IO_STATE as usize] = 45;
    assert_eq!(unsafe { r.lua_call(0, 0, GENO_LUA_PHASE_FRAME as i32, io.as_mut_ptr()) }, 0);
    assert_eq!(io[GENO_LUA_IO_NCMDS as usize..GENO_LUA_IO_NCMDS as usize + 2], [1, 0]);
    assert_eq!(io[GENO_LUA_IO_CMDS as usize..GENO_LUA_IO_CMDS as usize + 3], [GENO_LUA_CMD_GO, (GENO_TGT_GENO << 28) | 3, 0]);
    // the module and its slot layout are part of the id
    let id = r.p[0].id;
    let mut r2 = Registry::default();
    let block2 = block.replace("\"stored\":\"int\"", "\"stored\":\"float\"");
    assert_eq!(r2.parse_text(define(12, &block2, st).as_bytes(), "wasm-goku", &env), 1);
    assert_ne!(r2.p[0].id, id);
    // refusals, each logged
    for (text, why) in [
        (define(11, block, st), "\"wasm\" needs \"geno\": 12"),
        (define(12, &format!("{block},\"lua\":{{\"source\":\"return {{}}\"}}"), st), "either \"lua\" or \"wasm\""),
        (define(12, block, ",\"wasm\":{\"enter\":\"nope\"}"), "is not an export of the module"),
        (define(12, ",\"wasm\":{\"module\":\"wasm/missing.wasm\"}", ""), "cannot read wasm.module"),
        (define(12, ",\"wasm\":{\"module\":\"../x.wasm\"}", ""), "must be a relative path inside the mod"),
        (define(12, "", st), "the entry has no \"wasm\" block"),
    ] {
        let mut r = Registry::default();
        r.parse_text(text.as_bytes(), "wasm-goku", &env);
        assert!(r.p.is_empty() && r.log.iter().any(|l| l.contains(why)), "{why}: {:?}", r.log);
    }
    let mut r = Registry::default();
    r.parse_text(b"{\"geno\":12,\"fighters\":[{\"attach\":\"kirby\",\"wasm\":{\"module\":\"wasm/goku.wasm\"}}]}", "wasm-goku", &env);
    assert!(r.log.iter().any(|l| l.contains("an attach entry cannot carry a \"wasm\" block")), "{:?}", r.log);
}


