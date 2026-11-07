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
    // melee_corev1 (these sources as a core v1 game library): the host maps the image at 0x82800000, beside MEM1,
    // and refuses it anywhere else, so no ASLR
    if std::env::var_os("CARGO_FEATURE_COREV1").is_some() && std::env::var("TARGET").unwrap_or_default().contains("windows-msvc") {
        println!("cargo:rustc-cdylib-link-arg=/BASE:0x82800000");
        println!("cargo:rustc-cdylib-link-arg=/DYNAMICBASE:NO");
    }
    version_resource(&hash);
}

/// A Windows version resource on melee.exe (`FileVersion` etc., the block Explorer and the Melee Unlocked launcher
/// read), written as a compiled .res and handed to link.exe: no rc.exe or resource crate needed.
fn version_resource(hash: &str) {
    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("windows-msvc") {
        return;
    }
    let ver = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut nums: Vec<u16> = ver.split(|c: char| !c.is_ascii_digit()).filter_map(|x| x.parse().ok()).take(3).collect();
    nums.resize(4, 0);
    let ms = (nums[0] as u32) << 16 | nums[1] as u32;
    let ls = (nums[2] as u32) << 16 | nums[3] as u32;
    let strings = [
        ("CompanyName", "GD"),
        ("FileDescription", "Melee (native Rust port)"),
        ("FileVersion", ver.as_str()),
        ("InternalName", "melee"),
        ("OriginalFilename", "melee.exe"),
        ("ProductName", "Melee Rust Port"),
        ("ProductVersion", ver.as_str()),
        ("Comments", hash),
    ];
    let mut fixed = Vec::new();
    for v in [0xFEEF04BDu32, 0x0001_0000, ms, ls, ms, ls, 0x3F, 0, 0x0004_0004, 1, 0, 0, 0] {
        fixed.extend(v.to_le_bytes());
    }
    let table: Vec<Vec<u8>> = strings.iter().map(|(k, v)| node(k, &utf16z(v), 1, (v.encode_utf16().count() + 1) as u16, &[])).collect();
    let sfi = node("StringFileInfo", &[], 1, 0, &[node("040904B0", &[], 1, 0, &table)]);
    let var = node("VarFileInfo", &[], 1, 0, &[node("Translation", &[0x09, 0x04, 0xB0, 0x04], 0, 4, &[])]);
    let info = node("VS_VERSION_INFO", &fixed, 0, fixed.len() as u16, &[sfi, var]);
    // .res: an empty first entry, then RT_VERSION (16) #1, language 0x0409
    let mut res = Vec::new();
    res.extend(entry(&[], 0, 0, 0));
    res.extend(entry(&info, 16, 1, 0x0409));
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("melee_version.res");
    std::fs::write(&out, res).unwrap();
    println!("cargo:rustc-link-arg-{}={}", if std::env::var_os("CARGO_FEATURE_COREV1").is_some() { "cdylib" } else { "bins" }, out.display());
}

fn utf16z(s: &str) -> Vec<u8> {
    s.encode_utf16().chain([0]).flat_map(u16::to_le_bytes).collect()
}

fn pad4(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}

/// One VS_VERSIONINFO-style node: wLength, wValueLength, wType, key, value, children (each 4-aligned).
fn node(key: &str, value: &[u8], kind: u16, value_len: u16, children: &[Vec<u8>]) -> Vec<u8> {
    let mut v = vec![0u8; 4];
    v.extend(kind.to_le_bytes());
    v.extend(utf16z(key));
    pad4(&mut v);
    v.extend(value);
    for c in children {
        pad4(&mut v);
        v.extend(c);
    }
    let len = v.len() as u16;
    v[0..2].copy_from_slice(&len.to_le_bytes());
    v[2..4].copy_from_slice(&value_len.to_le_bytes());
    v
}

/// A .res entry header (numeric type and name) followed by its data, padded to 4.
fn entry(data: &[u8], ty: u16, name: u16, lang: u16) -> Vec<u8> {
    let mut h = Vec::new();
    h.extend((data.len() as u32).to_le_bytes());
    h.extend(32u32.to_le_bytes());
    h.extend([0xFFFFu16, ty, 0xFFFF, name].iter().flat_map(|x| x.to_le_bytes()));
    h.extend(0u32.to_le_bytes()); // DataVersion
    h.extend(if ty == 0 { 0u16 } else { 0x1030u16 }.to_le_bytes()); // MOVEABLE | PURE | DISCARDABLE
    h.extend(lang.to_le_bytes());
    h.extend(0u32.to_le_bytes());
    h.extend(0u32.to_le_bytes());
    h.extend(data);
    pad4(&mut h);
    h
}
