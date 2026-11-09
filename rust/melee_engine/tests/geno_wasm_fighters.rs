//! geno 12: Goku's fighter code compiled (rust/mods/geno_fighters/goku.wasm, from goku.c) against the geno 10 Lua it
//! replaces (tools/rust-port/golden/coverage/goku-xv2-lua/goku.lua, run by the same GLua the engine runs). Every
//! callback over the same io blocks must leave the same 64 words and return the same code: then the game half sees
//! no difference, and the C goldens the Lua build matches (mex_goku_cpu) hold for the wasm build. No disc needed.
use melee_engine::geno_wasm;
use melee_script::geno::consts::*;
use melee_script::geno::lua::{FighterCode, GLua, GLUA_FLOAT, GLUA_INT};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const FNS: [&str; 13] = [
    "charge_enter", "charge_frame", "rush_enter", "rush_frame", "rushcombo_enter", "rushcombo_frame", "recovery_enter",
    "counter_frame", "riposte_enter", "riposte_frame", "kicharge_frame", "it_targets", "nope",
];

fn layout() -> (Vec<String>, Vec<i32>) {
    // goku-xv2's lua.state: {charge: int, counter_damage: float, stored: int}
    (vec!["charge".into(), "counter_damage".into(), "stored".into()], vec![GLUA_INT, GLUA_FLOAT, GLUA_INT])
}

fn both(targets: &[&str]) -> (Box<GLua>, geno_wasm::GenoWasm) {
    let (names, types) = layout();
    let targets: Vec<String> = targets.iter().map(|s| s.to_string()).collect();
    let src = std::fs::read(format!("{ROOT}/tools/rust-port/golden/coverage/goku-xv2-lua/goku.lua")).unwrap();
    let mut l = GLua::new().unwrap();
    l.set_layout(&names, &types);
    l.set_targets(&targets);
    assert!(l.load(&src, "=lua/goku.lua"), "{}", l.err);
    let bytes = std::fs::read(format!("{ROOT}/rust/mods/geno_fighters/goku.wasm")).unwrap();
    let w = geno_wasm::load(&bytes, &names, &types, &targets).unwrap();
    (l, w)
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

fn compare(targets: &[&str], blocks: usize, seed: u64) -> usize {
    let (mut l, mut w) = both(targets);
    let mut r = Rng(seed);
    let mut faults = 0;
    for _ in 0..blocks {
        let base = io(&mut r);
        for name in FNS {
            let (fl, fw) = (l.find(name), FighterCode::find(&w, name));
            assert_eq!(fl >= 0, fw >= 0, "{name}: exported by one side only");
            if fl < 0 {
                continue;
            }
            let (mut a, mut b) = (base, base);
            let ra = unsafe { l.call(fl, a.as_mut_ptr()) };
            let rb = unsafe { FighterCode::call(&mut w, fw, b.as_mut_ptr()) };
            faults += (ra != 0) as usize;
            assert_eq!(ra, rb, "{name}: return codes differ (lua: {}, wasm: {})\nio {base:08x?}", l.err, FighterCode::err(&w));
            assert_eq!(a, b, "{name}: io words differ\nio   {base:08x?}\nlua  {a:08x?}\nwasm {b:08x?}");
        }
    }
    faults
}

#[test]
fn goku_wasm_equals_goku_lua() {
    // the define's Geno states (any order: both sides encode `go` by this list's index)
    let t = ["KameTap", "KameMid", "KameFull", "KameSuper", "KiCharge", "ITEnd", "ITEndAir", "RushCombo"];
    let faults = compare(&t, 4000, 0x6F6B_7567_0001);
    assert_eq!(faults, 0, "goku.lua faults on none of these blocks");
}

#[test]
fn goku_missing_target_faults_alike() {
    // without KameFull / KiCharge the `go` calls fault (GENO_LUA_FAULT_COMMAND) on both sides at the same point
    let faults = compare(&["KameTap", "KameMid", "ITEnd"], 2000, 0x6F6B_7567_0002);
    assert!(faults > 0, "some blocks reach a missing state");
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


/// The two vanilla packages' fighter Lua (pc/geno/mods/vanilla-{charger,riposte}/lua) against their geno 12 ports.
fn compare_pkg(lua: &str, wasm: &str, names: &[&str], types: &[i32], targets: &[&str], fns: &[&str], fill: fn(&mut Rng, &mut [u32]), blocks: usize) {
    let names: Vec<String> = names.iter().map(|s| s.to_string()).collect();
    let targets: Vec<String> = targets.iter().map(|s| s.to_string()).collect();
    let mut l = GLua::new().unwrap();
    l.set_layout(&names, types);
    l.set_targets(&targets);
    assert!(l.load(&std::fs::read(format!("{ROOT}/{lua}")).unwrap(), "=lua"), "{}", l.err);
    let mut w = geno_wasm::load(&std::fs::read(format!("{ROOT}/{wasm}")).unwrap(), &names, types, &targets).unwrap();
    let mut r = Rng(0x7661_6E69_6C6C_61);
    for _ in 0..blocks {
        let mut base = io(&mut r);
        fill(&mut r, &mut base);
        for name in fns {
            let (fl, fw) = (l.find(name), FighterCode::find(&w, name));
            assert!(fl >= 0 && fw >= 0, "{name} missing (lua {fl}, wasm {fw})");
            let (mut a, mut b) = (base, base);
            let ra = unsafe { l.call(fl, a.as_mut_ptr()) };
            let rb = unsafe { FighterCode::call(&mut w, fw, b.as_mut_ptr()) };
            assert_eq!((ra, a), (rb, b), "{name}: lua {} / wasm {}\nio {base:08x?}", l.err, FighterCode::err(&w));
        }
    }
}

#[test]
fn charger_wasm_equals_charger_lua() {
    compare_pkg(
        "pc/geno/mods/vanilla-charger/lua/charger.lua",
        "rust/mods/geno_fighters/charger.wasm",
        &["charge", "charged"],
        &[GLUA_INT, melee_script::geno::lua::GLUA_BOOL],
        &["Charge", "Release"],
        &["charge_enter", "charge_frame", "release_enter", "release_frame"],
        |r, w| {
            w[GENO_LUA_IO_STATE as usize] = [0, 1, 30, 58, 59, 60][r.below(6) as usize];
            w[GENO_LUA_IO_STATE as usize + 1] = r.below(2) as u32;
        },
        4000,
    );
}

#[test]
fn riposte_wasm_equals_riposte_lua() {
    compare_pkg(
        "pc/geno/mods/vanilla-riposte/lua/riposte.lua",
        "rust/mods/geno_fighters/riposte.wasm",
        &["power", "streak", "follow"],
        &[GLUA_FLOAT, GLUA_INT, melee_script::geno::lua::GLUA_BOOL],
        &["Parry", "Riposte", "Follow"],
        &["parry_enter", "parry_frame", "riposte_enter", "riposte_frame", "follow_enter", "follow_frame"],
        |r, w| {
            // action frames around 6 / 8 / 16 / 24 / 30, hit damage across 6 (9 / 1.5) and 20 (30 / 1.5), both sides
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
        },
        4000,
    );
}
