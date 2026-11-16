//! First-run pipeline warm pass (notes/zero-hitch.md "First-run warm pass"): when the pipeline warm list
//! (melee_gx `warmlist`, `gpucache/`) is still small, the game starts a hidden helper process at below-normal
//! priority that plays short CPU matches from the player's own disc across the stages and fighters, so their GPU
//! pipelines are recorded into the warm list (and the driver's shader cache) without the player meeting them first on
//! a frame. The game thread only spawns a thread here; the thread waits on the children. Nothing the pass produces is
//! committed or shipped: the list is local GPU state, derived from the disc on the player's machine.
//!
//! MELEE_WARM_PASS=1 forces it on. Off with MELEE_WARM_PASS=0, and for every agent/test run (MELEE_WINDOW_HIDDEN, MELEE_TURBO, MELEE_PROFILE, MELEE_PAD_SCRIPT,
//! --headless, --max-frames, --replay, --record set) and inside a warm child itself.

/// Below this many records the list counts as a first run (a full pass leaves several hundred).
const SMALL: usize = 150;

/// (fighter, fighter, stage) per helper match: every legal stage once, the cast spread across them.
const MATCHES: &[(&str, &str, &str)] = &[
    ("fox", "falco", "FINAL DESTINATION"),
    ("marth", "sheik", "BATTLEFIELD"),
    ("falcon", "jigglypuff", "YOSHI'S STORY"),
    ("peach", "samus", "FOUNTAIN OF DREAMS"),
    ("pikachu", "ganon", "POKEMON STADIUM"),
    ("drmario", "luigi", "DREAM LAND N64"),
    ("mario", "yoshi", "BATTLEFIELD"),
    ("dk", "link", "FINAL DESTINATION"),
    ("younglink", "roy", "YOSHI'S STORY"),
    ("ics", "kirby", "FOUNTAIN OF DREAMS"),
    ("ness", "bowser", "POKEMON STADIUM"),
    ("mewtwo", "gw", "DREAM LAND N64"),
    ("pichu", "zelda", "BATTLEFIELD"),
];

fn skipped(args: &[String]) -> bool {
    let env_on = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty() && v != "0");
    std::env::var("MELEE_WARM_PASS").is_ok_and(|v| v == "0")
        || env_on("MELEE_WARM_CHILD")
        || env_on("MELEE_WINDOW_HIDDEN")
        || env_on("MELEE_TURBO")
        || env_on("MELEE_PROFILE")
        || env_on("MELEE_PAD_SCRIPT")
        || args.iter().any(|a| matches!(a.as_str(), "--headless" | "--max-frames" | "--replay" | "--record" | "--spectate"))
}

/// Called once from `play` after the disc opened. Never blocks: at most it starts one background thread.
pub fn maybe_start(iso: &str, args: &[String]) {
    // MELEE_WARM_PASS=1 forces the pass (a test of it from a hidden/scripted run)
    if std::env::var_os("MELEE_WARM_CHILD").is_some() || (!std::env::var("MELEE_WARM_PASS").is_ok_and(|v| v == "1") && skipped(args)) {
        return;
    }
    let n = melee_gx::render::warmlist::record_count();
    if n >= SMALL {
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    let iso = iso.to_string();
    eprintln!("warm pass: pipeline list has {n} records; warming in a hidden below-normal helper");
    let _ = std::thread::Builder::new().name("warm pass".into()).spawn(move || {
        let t = std::time::Instant::now();
        for (i, (a, b, st)) in MATCHES.iter().enumerate() {
            let mut c = std::process::Command::new(&exe);
            c.args(["play", "--iso", &iso, "--widescreen", "--windowed", "--window", "1280x720", "--no-audio"])
                .args(["--match", &format!("{a}/cpu9,{b}/cpu9,{st}"), "--max-frames", "900"])
                .env("MELEE_WARM_CHILD", "1")
                .env_remove("MELEE_WARM_PASS")
                .env("MELEE_WINDOW_HIDDEN", "1")
                .env("MELEE_TURBO", "1")
                .env("MELEE_VOLUME", "0")
                .env("MELEE_FULLSCREEN", "off")
                .env("MELEE_WINDOW_TITLE", format!("melee warm pass {}/{}", i + 1, MATCHES.len()))
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x4000;
                const CREATE_NO_WINDOW: u32 = 0x0800_0000;
                c.creation_flags(BELOW_NORMAL_PRIORITY_CLASS | CREATE_NO_WINDOW);
            }
            let Ok(mut child) = c.spawn() else { return };
            // a hard limit per match: a stuck helper is killed, the pass goes on
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) if start.elapsed() < std::time::Duration::from_secs(120) => {
                        std::thread::sleep(std::time::Duration::from_millis(250))
                    }
                    _ => {
                        let _ = child.kill();
                        let _ = child.wait();
                        break;
                    }
                }
            }
        }
        eprintln!(
            "warm pass: done, {} records, {:.0} s",
            melee_gx::render::warmlist::record_count(),
            t.elapsed().as_secs_f64()
        );
    });
}
