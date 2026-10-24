//! `gdscript <script.lua | folder> [--frames N] [--exec "lua"]...`: load a script (or a mod folder with mod.json)
//! into the sandboxed engine with no game attached, run N script frames (tick, frame pre/post) and print the console.
//! Exit status is 1 when the script failed to load or raised errors. A smoke harness for the load-time gd surface.
use melee_script::{Config, Engine, MemHost, NoGame};
use std::cell::RefCell;
use std::path::PathBuf;
use std::process::ExitCode;
use std::rc::Rc;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut target = None;
    let mut frames = 0;
    let mut execs = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--frames" => frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(0),
            "--exec" => execs.extend(args.next()),
            _ => target = Some(a),
        }
    }
    let Some(target) = target else {
        eprintln!("usage: gdscript <script.lua | folder> [--frames N] [--exec LUA]...");
        return ExitCode::from(2);
    };
    let host = Rc::new(RefCell::new(MemHost { echo: false, ..MemHost::default() }));
    let exe = std::env::temp_dir().join("gdscript");
    let mut cfg = Config::from_env(exe);
    cfg.data_dir = std::env::temp_dir().join("gdscript-data");
    let abs = std::fs::canonicalize(&target).unwrap_or(PathBuf::from(&target));
    let abs_s = abs.to_string_lossy().trim_start_matches("\\?\\").to_string();
    // a mod folder (mod.json + scripts/*.lua) loads the way gs_scan_mods mounts it
    let as_mod = abs.is_dir() && abs.join("scripts").is_dir();
    if as_mod {
        cfg.mods_dir = abs.parent().unwrap().to_path_buf();
        cfg.active_mods = vec![abs.file_name().unwrap().to_string_lossy().into_owned()];
    }
    let eng = Engine::new(host.clone(), Box::new(NoGame), cfg).expect("engine");
    let loaded = if as_mod {
        eng.scan_mods();
        eng.st.borrow().scripts.iter().position(|x| x.used && x.origin == "mods")
    } else {
        eng.load_named(&abs_s)
    };
    for _ in 0..frames {
        eng.tick();
        eng.frame_pre();
        eng.frame_post();
    }
    for e in &execs {
        eng.exec(e);
    }
    for l in eng.console_lines() {
        println!("{l}");
    }
    let s = eng.st.borrow();
    let errors: i32 = s.scripts.iter().map(|x| x.errors).sum();
    if loaded.is_none() || errors > 0 {
        eprintln!("gdscript: {} ({} errors)", if loaded.is_none() { "did not load" } else { "loaded" }, errors);
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
