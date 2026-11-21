//! `melee slpcheck --list FILE --out-dir DIR [--root DIR] [--jobs N] [--skip-existing] [--timeout SECS] [--min-free-gb G]
//! [slpcheck flags]`: check many replays with N at once, the single-file sweep's results (scratch/fullrun/sweep.sh).
//!
//! Each replay runs in its own `slpcheck --slp F --csv OUT` child of this exe, so its row is the single-file run's by
//! construction. Measured on the desktop (Nov 21): a process start-up (disc open, archive parse, engine boot) is
//! ~0.24 s (`--frames 1`); the rest of a replay is the simulation. Reusing one process for several replays saves only
//! that, and the engine does not survive a second `boot_vs` in one process (the HSD class table keeps the first
//! boot's pointers: `ForgetClassLibraryReal` unwraps a None, class.rs), so the gain is the parallelism.
//!
//! Failure rules as sweep.sh: no CSV from the child -> `<relpath>,crash-rc<code>`; past `--timeout` (default 900 s) the
//! child is killed -> `crash-rc124` (timeout(1)'s code). A new child starts only while at least `--min-free-gb`
//! (default 8) of physical memory is free. Output names: the path relative to `--root`, `/` -> `_`, `.slp` dropped.
//! stdout: the child's `SLPCHECK` summary line per replay, then a replays/s total.

use std::io::{BufRead, BufReader};
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn get(args: &[String], k: &str) -> Option<String> {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned()
}

/// sweep.sh's name rule: `${f#"$dir"/}`, `tr / _`, `sed 's/\.slp$//'`.
fn out_name(root: Option<&str>, f: &str) -> (String, String) {
    let fs = f.replace('\\', "/");
    let rel = match root {
        Some(r) => {
            let r = r.replace('\\', "/");
            fs.strip_prefix(&format!("{}/", r.trim_end_matches('/'))).map(str::to_string).unwrap_or(fs.clone())
        }
        None => fs.clone(),
    };
    let b = rel.replace('/', "_");
    let b = b.strip_suffix(".slp").map(str::to_string).unwrap_or(b);
    (rel, b)
}

#[cfg(windows)]
fn free_gb() -> Option<f64> {
    #[repr(C)]
    struct Ms {
        len: u32,
        load: u32,
        total: u64,
        avail: u64,
        rest: [u64; 5],
    }
    extern "system" {
        fn GlobalMemoryStatusEx(m: *mut Ms) -> i32;
    }
    let mut m = Ms { len: core::mem::size_of::<Ms>() as u32, load: 0, total: 0, avail: 0, rest: [0; 5] };
    (unsafe { GlobalMemoryStatusEx(&mut m) } != 0).then(|| m.avail as f64 / (1u64 << 30) as f64)
}
#[cfg(not(windows))]
fn free_gb() -> Option<f64> {
    let t = std::fs::read_to_string("/proc/meminfo").ok()?;
    let l = t.lines().find(|l| l.starts_with("MemAvailable:"))?;
    Some(l.split_whitespace().nth(1)?.parse::<f64>().ok()? / (1u64 << 20) as f64)
}

pub fn batch(args: &[String]) -> ExitCode {
    let (Some(list), Some(out_dir)) = (get(args, "--list"), get(args, "--out-dir")) else {
        eprintln!("usage: melee slpcheck --list FILE --out-dir DIR [--root DIR] [--jobs N] [--skip-existing] [--timeout SECS] [--min-free-gb G] [slpcheck flags]");
        return ExitCode::from(2);
    };
    let root = get(args, "--root");
    let jobs: usize = get(args, "--jobs").and_then(|v| v.parse().ok()).unwrap_or(1).max(1);
    let timeout = Duration::from_secs(get(args, "--timeout").and_then(|v| v.parse().ok()).unwrap_or(900));
    let min_free: f64 = get(args, "--min-free-gb").and_then(|v| v.parse().ok()).unwrap_or(8.0);
    let skip = args.iter().any(|a| a == "--skip-existing");
    // the children's own slpcheck flags: everything but the batch's
    let mut pass = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" | "--out-dir" | "--root" | "--jobs" | "--timeout" | "--min-free-gb" => i += 2,
            "--skip-existing" => i += 1,
            _ => {
                pass.push(args[i].clone());
                i += 1;
            }
        }
    }
    let text = match std::fs::read_to_string(&list) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("slpcheck --list: {e}");
            return ExitCode::from(2);
        }
    };
    let _ = std::fs::create_dir_all(&out_dir);
    let mut work = std::collections::VecDeque::new();
    for f in text.lines().map(|l| l.trim_end_matches('\r')).filter(|l| !l.is_empty()) {
        let (rel, b) = out_name(root.as_deref(), f);
        if skip && std::fs::metadata(format!("{out_dir}/{b}.csv")).is_ok_and(|m| m.len() > 0) {
            continue;
        }
        work.push_back((f.to_string(), rel, b));
    }
    let Ok(exe) = std::env::current_exe() else { return ExitCode::from(2) };
    println!("slpbatch: {} replays, {jobs} jobs", work.len());
    let queue = Arc::new(Mutex::new(work));
    let start_gate = Arc::new(Mutex::new(()));
    let t0 = Instant::now();
    let done = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let hs: Vec<_> = (0..jobs)
        .map(|_| {
            let (queue, pass, out_dir, done, exe, start_gate) = (queue.clone(), pass.clone(), out_dir.clone(), done.clone(), exe.clone(), start_gate.clone());
            std::thread::spawn(move || loop {
                let Some((f, rel, b)) = queue.lock().unwrap().pop_front() else { break };
                let tmp = format!("{out_dir}/{b}.tmp");
                let csv = format!("{out_dir}/{b}.csv");
                let _ = std::fs::remove_file(&tmp);
                {
                    // one start at a time, and only with the memory floor free (a child's working set grows over its boot)
                    let _g = start_gate.lock().unwrap();
                    while free_gb().is_some_and(|g| g < min_free) {
                        std::thread::sleep(Duration::from_secs(5));
                    }
                }
                let child = Command::new(&exe).arg("slpcheck").arg("--slp").arg(&f).arg("--csv").arg(&tmp).args(&pass).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn();
                let rc = match child {
                    Err(e) => {
                        eprintln!("slpbatch: spawn: {e}");
                        -1
                    }
                    Ok(mut c) => {
                        let out = c.stdout.take().unwrap();
                        let rd = std::thread::spawn(move || {
                            for l in BufReader::new(out).lines().map_while(Result::ok) {
                                if l.starts_with("SLPCHECK ") {
                                    println!("{l}");
                                }
                            }
                        });
                        let started = Instant::now();
                        let rc = loop {
                            match c.try_wait() {
                                Ok(Some(s)) => break s.code().unwrap_or(-1),
                                Ok(None) if started.elapsed() > timeout => {
                                    let _ = c.kill();
                                    let _ = c.wait();
                                    break 124;
                                }
                                Ok(None) => std::thread::sleep(Duration::from_millis(25)),
                                Err(_) => break -1,
                            }
                        };
                        let _ = rd.join();
                        rc
                    }
                };
                if rc != 124 && std::fs::metadata(&tmp).is_ok_and(|m| m.len() > 0) {
                    let _ = std::fs::rename(&tmp, &csv);
                } else {
                    let _ = std::fs::remove_file(&tmp);
                    let _ = std::fs::write(&csv, format!("{rel},crash-rc{rc}\n"));
                    println!("SLPCHECK {rel} crash-rc{rc}");
                }
                done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            })
        })
        .collect();
    for h in hs {
        let _ = h.join();
    }
    let s = t0.elapsed().as_secs_f64();
    let n = done.load(std::sync::atomic::Ordering::Relaxed);
    println!("slpbatch: {n} replays in {s:.1} s ({:.3} replays/s)", n as f64 / s.max(1e-9));
    ExitCode::SUCCESS
}
