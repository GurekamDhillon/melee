//! Build-time identity for the window title: the git short hash and the worktree / branch name
//! (`MELEE_GIT_HASH`, `MELEE_GIT_WHERE`; "unknown" outside a git checkout).
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let o = Command::new("git").args(args).output().ok()?;
    o.status.success().then(|| String::from_utf8_lossy(&o.stdout).trim().to_string()).filter(|s| !s.is_empty())
}

fn main() {
    let hash = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    // the worktree's directory name, else the branch
    let top = git(&["rev-parse", "--show-toplevel"]);
    let wt = top.as_deref().and_then(|t| std::path::Path::new(t).file_name()).map(|n| n.to_string_lossy().to_string());
    let branch = git(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let place = match (wt, branch) {
        (Some(w), _) if w != "melee" => w,
        (_, Some(b)) => b,
        (Some(w), None) => w,
        _ => "unknown".into(),
    };
    println!("cargo:rustc-env=MELEE_GIT_HASH={hash}");
    println!("cargo:rustc-env=MELEE_GIT_WHERE={place}");
    // re-run when HEAD moves (a commit, a checkout)
    if let Some(dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        println!("cargo:rerun-if-changed={dir}/HEAD");
        println!("cargo:rerun-if-changed={dir}/logs/HEAD");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
