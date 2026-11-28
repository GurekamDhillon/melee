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
    // a helper is a child process: none under the YouGame app (its sandbox), and the helpers' matches took the CPU
    // from the game thread there while the app unpacked a disc (audio underruns)
    if melee_platform::yougame::refuse_child("warm pass") {
        return;
    }
    let n = melee_gx::render::warmlist::record_count();
    if n >= SMALL {
        return;
    }
    let Ok(exe) = std::env::current_exe() else { return };
    let iso = iso.to_string();
    eprintln!("warm pass: pipeline list has {n} records; warming in a hidden below-normal helper");
    let log = child_log_path(std::env::var_os("MELEE_LOG_FILE").map(std::path::PathBuf::from).as_deref());
    if let Some(l) = &log {
        let _ = std::fs::write(l, b""); // one pass per file: the previous pass's lines go
        eprintln!("warm pass: helpers log to {}", l.display());
    }
    let _ = std::thread::Builder::new().name("warm pass".into()).spawn(move || {
        let t = std::time::Instant::now();
        for (i, m) in MATCHES.iter().enumerate() {
            let mut c = child_command(&exe, &iso, i, *m, log.as_deref());
            let Ok(mut child) = c.spawn() else {
                eprintln!("warm pass: helper {}/{} did not start", i + 1, MATCHES.len());
                return;
            };
            eprintln!("warm pass: helper {}/{} started (pid {})", i + 1, MATCHES.len(), child.id());
            // a hard limit per match: a stuck helper is killed, the pass goes on
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(st)) => {
                        eprintln!("warm pass: helper {}/{} exited ({st})", i + 1, MATCHES.len());
                        break;
                    }
                    Ok(None) if start.elapsed() < std::time::Duration::from_secs(120) => {
                        std::thread::sleep(std::time::Duration::from_millis(250))
                    }
                    _ => {
                        let _ = child.kill();
                        let _ = child.wait();
                        eprintln!("warm pass: helper {}/{} killed after 120 s", i + 1, MATCHES.len());
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

/// The helpers' own log, next to the player's (`melee-slippi.log` -> `melee-slippi-warm.log`): a helper inherits
/// the parent's environment, and with MELEE_LOG_FILE it appended its whole session ("play: max frames reached",
/// "closed after 900 ticks", its heartbeats) into the player's log in the middle of the player's session.
/// None (no log at all; stderr is null) when the parent has no log file.
fn child_log_path(parent: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    let p = parent.filter(|p| !p.as_os_str().is_empty())?;
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "melee".into());
    Some(p.with_file_name(format!("{stem}-warm.log")))
}

/// One helper's command line and environment. The helper must never take a device from the player: before
/// MELEE_PAD_IGNORE_ADAPTER, a helper opened the GC adapter whenever the game window was in the background (the
/// adapter is released there for other programs), and the game's reclaim on focus then failed with "CreateFile failed
/// (error 5: another program holds it)" until the helper's match ended (Nov 27, melee-slippi.log).
fn child_command(exe: &std::path::Path, iso: &str, i: usize, (a, b, st): (&str, &str, &str), log: Option<&std::path::Path>) -> std::process::Command {
    let mut c = std::process::Command::new(exe);
    c.args(["play", "--iso", iso, "--widescreen", "--windowed", "--window", "1280x720", "--no-audio"])
        .args(["--match", &format!("{a}/cpu9,{b}/cpu9,{st}"), "--max-frames", "900"])
        .env("MELEE_WARM_CHILD", "1")
        .env_remove("MELEE_WARM_PASS")
        .env("MELEE_WINDOW_HIDDEN", "1")
        .env("MELEE_WINDOW_UNFOCUSED", "1")
        .env("MELEE_PAD_IGNORE_ADAPTER", "1")
        .env("MELEE_INPUT", "n")
        .env("MELEE_TURBO", "1")
        .env("MELEE_VOLUME", "0")
        .env("MELEE_FULLSCREEN", "off")
        .env("MELEE_WINDOW_TITLE", format!("melee warm pass {}/{}", i + 1, MATCHES.len()))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    match log {
        Some(l) => c.env("MELEE_LOG_FILE", l),
        None => c.env_remove("MELEE_LOG_FILE"),
    };
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x4000;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(BELOW_NORMAL_PRIORITY_CLASS | CREATE_NO_WINDOW);
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::path::Path;

    fn env_of<'a>(c: &'a std::process::Command, k: &str) -> Option<Option<&'a OsStr>> {
        c.get_envs().find(|(n, _)| *n == OsStr::new(k)).map(|(_, v)| v)
    }

    #[test]
    fn helper_never_takes_the_adapter_or_input() {
        let c = child_command(Path::new("melee.exe"), "game.iso", 0, MATCHES[0], None);
        assert_eq!(env_of(&c, "MELEE_PAD_IGNORE_ADAPTER"), Some(Some(OsStr::new("1"))));
        assert_eq!(env_of(&c, "MELEE_INPUT"), Some(Some(OsStr::new("n"))));
        assert_eq!(env_of(&c, "MELEE_WINDOW_HIDDEN"), Some(Some(OsStr::new("1"))));
        assert_eq!(env_of(&c, "MELEE_WINDOW_UNFOCUSED"), Some(Some(OsStr::new("1"))));
        assert_eq!(env_of(&c, "MELEE_WARM_CHILD"), Some(Some(OsStr::new("1"))));
    }

    #[test]
    fn helper_never_writes_the_players_log() {
        // no parent log: the inherited variable is removed, not passed on
        let c = child_command(Path::new("melee.exe"), "game.iso", 0, MATCHES[0], None);
        assert_eq!(env_of(&c, "MELEE_LOG_FILE"), Some(None));
        // a parent log: the helper gets its own file beside it
        let parent = Path::new(r"C:\play\melee-slippi.log");
        let own = child_log_path(Some(parent)).unwrap();
        assert_eq!(own, Path::new(r"C:\play\melee-slippi-warm.log"));
        assert_ne!(own, parent);
        let c = child_command(Path::new("melee.exe"), "game.iso", 3, MATCHES[3], Some(&own));
        assert_eq!(env_of(&c, "MELEE_LOG_FILE"), Some(Some(own.as_os_str())));
        assert_eq!(child_log_path(Some(Path::new(""))), None);
        assert_eq!(child_log_path(None), None);
    }
}
