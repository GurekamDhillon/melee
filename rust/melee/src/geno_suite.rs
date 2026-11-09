//! `melee geno-tests [--iso DISC] [--filter SUB]`: Geno's game-side test suite (pc/geno/geno_tests.c, ported in
//! melee_modes::tu::geno_tests) run inside a booted engine, TAP output like the C build's `--test`
//! (`MELEE_TEST_FILTER=geno`). Each test runs under catch_unwind, so a TODO stop counts as a failure, not a crash.
use std::process::ExitCode;

static mut TESTS: Vec<(String, unsafe extern "C" fn() -> i32)> = Vec::new();
static mut FAIL_MSG: Option<String> = None;

pub fn run(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let filter = get("--filter").or_else(|| std::env::var("MELEE_TEST_FILTER").ok()).unwrap_or_default();
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("usage: melee geno-tests --iso DISC.iso [--filter SUB]");
        return ExitCode::from(2);
    };
    let disc = match crate::engine_disc(&iso) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::from(2);
        }
    };
    let scene = get("--scene").unwrap_or_else(|| "p1=none;p2=none;p3=none;p4=none;stage=ext:32;match=time;minutes=0;items=off".into());
    let scn = melee_engine::scenario::Scenario::parse(&format!("mode=vs;at=match;{scene}")).expect("scene");
    let _e = match melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("melee: engine boot: {e}");
            return ExitCode::from(2);
        }
    };
    use melee_modes::tu::geno_tests as gt;
    unsafe {
        gt::ext::hook::TestRegister = Some(|name, f| {
            let n = core::ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
            if let Some(f) = f {
                (*core::ptr::addr_of_mut!(TESTS)).push((n, f));
            }
        });
        gt::ext::hook::TestFail = Some(|msg| {
            let slot = &mut *core::ptr::addr_of_mut!(FAIL_MSG);
            if slot.is_none() {
                *slot = Some(core::ffi::CStr::from_ptr(msg).to_string_lossy().into_owned());
            }
        });
        gt::GenoTestRegisterAll();
    }
    let tests = unsafe { (*core::ptr::addr_of!(TESTS)).clone() };
    // the ported tests are extern "C": a panic inside aborts the process, so the default run is one child per test
    if let Some(one) = get("--one") {
        let Some((_, f)) = tests.iter().find(|t| t.0 == one) else { println!("not ok - {one} (no such test)"); return ExitCode::FAILURE };
        let rc = unsafe { f() };
        let msg = unsafe { (*core::ptr::addr_of_mut!(FAIL_MSG)).take() };
        if rc == 0 && msg.is_none() { println!("RESULT ok"); return ExitCode::SUCCESS; }
        println!("RESULT fail (rc {rc}) {}", msg.unwrap_or_default());
        return ExitCode::FAILURE;
    }
    if !args.iter().any(|a| a == "--inproc") {
        let exe = std::env::current_exe().unwrap();
        let (mut pass, mut fail, mut n) = (0, 0, 0);
        for (name, _) in &tests {
            if !filter.is_empty() && !name.contains(&filter) { continue; }
            n += 1;
            let mut cmd = std::process::Command::new(&exe);
            cmd.args(["geno-tests", "--iso", &iso, "--scene", &scene, "--one", name]);
            let out = cmd.output().expect("spawn");
            let so = String::from_utf8_lossy(&out.stdout);
            let se = String::from_utf8_lossy(&out.stderr);
            if let Some(l) = so.lines().find(|l| l.starts_with("RESULT ")) {
                if l == "RESULT ok" { pass += 1; println!("ok {n} - {name}"); }
                else { fail += 1; println!("not ok {n} - {name} {}", &l[7..]); }
            } else {
                fail += 1;
                let why = se.lines().skip_while(|l| !l.contains("panicked at")).take(2).collect::<Vec<_>>().join(" | ");
                println!("not ok {n} - {name} CRASH {why}");
            }
        }
        println!("TESTS: pass={pass} fail={fail} total={n}");
        return if fail == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    let (mut pass, mut fail, mut n) = (0, 0, 0);
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for (name, f) in tests {
        if !filter.is_empty() && !name.contains(&filter) {
            continue;
        }
        n += 1;
        unsafe { FAIL_MSG = None };
        let r = std::panic::catch_unwind(|| unsafe { f() });
        let msg = unsafe { (*core::ptr::addr_of_mut!(FAIL_MSG)).take() };
        match r {
            Ok(0) if msg.is_none() => {
                pass += 1;
                println!("ok {n} - {name}");
            }
            Ok(rc) => {
                fail += 1;
                println!("not ok {n} - {name} (rc {rc}) {}", msg.unwrap_or_default());
            }
            Err(p) => {
                fail += 1;
                let s = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                println!("not ok {n} - {name} PANIC {s} {}", msg.unwrap_or_default());
            }
        }
    }
    std::panic::set_hook(prev);
    println!("TESTS: pass={pass} fail={fail} total={n}");
    if fail == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// `melee mexid --iso DISC [--mods ...]`: the fighter identity table (melee_engine::mexid) on this disc and mods, one
/// line per entry as gw_mexid.c's `mexid_disc_table` test logs it ("mexid:   fighter <ck> <hash> <name>"), so the two
/// builds' tables diff line for line (the C's stage lines aside). Fails when a retail fighter has no identity.
pub fn mexid(args: &[String]) -> ExitCode {
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let Some(iso) = get("--iso").or_else(|| std::env::var("GW_ISO_VANILLA").ok()) else {
        eprintln!("usage: melee mexid --iso DISC.iso [--mods ...]");
        return ExitCode::from(2);
    };
    if let Err(e) = crate::set_engine_mods(args) {
        eprintln!("melee: {e}");
        return ExitCode::from(2);
    }
    let disc = match crate::engine_disc(&iso) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("melee: {e}");
            return ExitCode::from(2);
        }
    };
    let scn = melee_engine::scenario::Scenario::parse("mode=vs;at=match;p1=none;p2=none;p3=none;p4=none;stage=ext:32;match=time;minutes=0;items=off").expect("scene");
    let _e = match melee_engine::Engine::boot_vs(disc, 64 << 20, melee_platform::PadSystem::headless(), scn) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("melee: engine boot: {e}");
            return ExitCode::from(2);
        }
    };
    let t = melee_engine::mexid::Fighters();
    for e in &t {
        println!("mexid:   {:<7} {:>3} {:016x} {}", "fighter", e.local, e.hash, e.name);
    }
    let retail = t.iter().filter(|e| (0..26).contains(&e.local)).count();
    if retail < 26 {
        eprintln!("mexid: only {retail} of the 26 retail fighters have identities");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
