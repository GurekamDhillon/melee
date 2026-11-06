// SPDX-License-Identifier: MIT
//! RUSTC_WORKSPACE_WRAPPER for the opt-in coverage build (tools/rust-port/covtest/build_cov.sh,
//! docs/rust-port/notes/coverage-testing.md). Cargo runs `rustc_cov <rustc> <args...>` for every workspace crate; this adds
//! LLVM source-based coverage with branch counters (`-C instrument-coverage -Z coverage-options=branch`) only to the
//! ported game-code crates, plus the `melee` binary (it must be instrumented too, so the profiler runtime is linked;
//! report.py drops its files). RUSTC_BOOTSTRAP=1 is set for those rustc processes only (stable rustc, one -Z flag).
//! Never used by normal, fast, release or perf builds: only build_cov.sh sets it, with `--profile cov`.
use std::process::{exit, Command};

const GAME: &[&str] = &["melee_fighter", "melee_items", "melee_world", "melee_runtime", "melee_modes", "melee_chara", "melee"];

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("rustc_cov: no rustc given");
        exit(2);
    }
    let rustc = args.remove(0);
    let name = args.iter().position(|a| a == "--crate-name").and_then(|i| args.get(i + 1)).cloned().unwrap_or_default();
    let mut cmd = Command::new(&rustc);
    cmd.args(&args);
    if GAME.contains(&name.as_str()) && !args.iter().any(|a| a == "-" || a.starts_with("--print")) {
        cmd.args(["-C", "instrument-coverage", "-Z", "coverage-options=branch"]);
        cmd.env("RUSTC_BOOTSTRAP", "1");
    }
    match cmd.status() {
        Ok(s) => exit(s.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("rustc_cov: {rustc}: {e}");
            exit(2);
        }
    }
}
