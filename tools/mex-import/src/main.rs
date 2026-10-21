//! mex-import: an m-ex disc -> gdmods over a vanilla disc, plus a struct-offset inventory of the
//! mods' PowerPC code against melee_types' GameCube layout tables. See README.md.
//!
//!   mex-import import --vanilla V.iso --mod M.iso --name akaneia --out DIR [--version 1.0.1]
//!                     [--no-split] [--dry-run] [--enable id,id] [--no-code]
//!   mex-import code   --mod M.iso [--vanilla V.iso] [--out DIR --name N]
//!   mex-import tables --mod M.iso
//!
//! Disc paths may also come from the environment: --vanilla defaults to $GW_ISO_VANILLA.
//! Everything written is disc-derived: keep --out outside any repository.

mod analyze;
mod disc;
mod hsd;
mod layout;
mod mexfn;
mod pack;
mod report;
mod sha256;

use disc::Disc;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

struct Args {
    cmd: String,
    kv: BTreeMap<String, String>,
    flags: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let cmd = it.next().ok_or("usage: mex-import <import|code|tables> [options] (see README.md)")?;
    let mut kv = BTreeMap::new();
    let mut flags = vec![];
    let rest: Vec<String> = it.collect();
    let mut i = 0;
    while i < rest.len() {
        let a = &rest[i];
        let Some(k) = a.strip_prefix("--") else { return Err(format!("unexpected argument {a}")) };
        if ["dry-run", "no-split", "no-code"].contains(&k) {
            flags.push(k.to_string());
            i += 1;
        } else {
            let v = rest.get(i + 1).ok_or(format!("--{k} needs a value"))?;
            kv.insert(k.to_string(), v.clone());
            i += 2;
        }
    }
    Ok(Args { cmd, kv, flags })
}

fn open(args: &Args, key: &str, env: &str) -> Result<Disc, String> {
    let p = args.kv.get(key).cloned().or_else(|| std::env::var(env).ok()).ok_or(format!("--{key} (or ${env}) is required"))?;
    Disc::open(Path::new(&p)).map_err(|e| format!("{p}: {e}"))
}

fn mb(n: u64) -> String {
    format!("{:.1} MB", n as f64 / (1u64 << 20) as f64)
}

/// Refuse to write disc-derived output where git could pick it up: inside a work tree, unless
/// git ignores that path (the workspace's `_build/` is ignored).
fn check_out_dir(out: &Path) -> Result<(), String> {
    let abs = std::path::absolute(out).map_err(|e| e.to_string())?;
    let mut p: Option<&Path> = Some(&abs);
    while let Some(d) = p {
        if d.join(".git").exists() {
            let probe = abs.join("probe.dat");
            let ignored = std::process::Command::new("git")
                .arg("-C")
                .arg(d)
                .args(["check-ignore", "-q", "--no-index"])
                .arg(&probe)
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ignored {
                return Ok(());
            }
            return Err(format!(
                "--out {} is inside the git work tree {} and not ignored there: disc-derived output must never be committable",
                abs.display(),
                d.display()
            ));
        }
        p = d.parent();
    }
    Ok(())
}

fn code_inventory(m: &Disc, only: Option<&[String]>) -> Result<report::CodeReport, String> {
    let reg = layout::Registry::new();
    let mut rep = report::CodeReport::default();
    let names: Vec<String> = match only {
        Some(l) => l.to_vec(),
        None => m.files.keys().cloned().collect(),
    };
    for n in names.iter().filter(|n| n.to_ascii_lowercase().ends_with(".dat")) {
        let bytes = m.read_path(n).map_err(|e| e.to_string())?;
        rep.scan_file(&reg, n, &bytes);
    }
    Ok(rep)
}

fn write_code_report(out: &Path, name: &str, rep: &report::CodeReport) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let j = out.join(format!("{name}.mex-code.json"));
    std::fs::write(&j, rep.json()).map_err(|e| e.to_string())?;
    let t = out.join(format!("{name}.mex-code.tsv"));
    let head = "blob\tfunction\tcode_off\tinsn\tdir\tsize\tdisp\tclass\trecord\tgc_off\tc_path\tnative_path\taddr\n";
    std::fs::write(&t, format!("{head}{}", rep.tsv)).map_err(|e| e.to_string())?;
    println!("wrote {} and {}", j.display(), t.display());
    Ok(())
}

fn cmd_tables(args: &Args) -> Result<(), String> {
    let m = open(args, "mod", "GW_ISO_MOD")?;
    let raw = m.read_path("MxDt.dat").map_err(|e| e.to_string())?;
    let t = hsd::Tables::parse(&raw)?;
    println!("MxDt.dat v{}.{}  ({} bytes)", t.version.0, t.version.1, raw.len());
    for (k, v) in &t.meta {
        println!("  {k:<22} {v}");
    }
    let ext = t.ext_of_internal();
    for k in hsd::FIRST_NEW_FIGHTER..t.meta["internal_id_count"] - 6 {
        let (n, f) = t.fighter_claims(k, ext.get(&(k as u8)).copied());
        if let Some(n) = n {
            println!("  fighter {k:>3} {n:<24} {}", f.join(" "));
        }
    }
    Ok(())
}

fn cmd_code(args: &Args) -> Result<(), String> {
    let m = open(args, "mod", "GW_ISO_MOD")?;
    let only = match args.kv.get("vanilla") {
        Some(_) => {
            let v = open(args, "vanilla", "GW_ISO_VANILLA")?;
            let d = pack::delta(&v, &m, |_, _| {}).map_err(|e| e.to_string())?;
            Some(d.new.into_iter().chain(d.modified).collect::<Vec<_>>())
        }
        None => None,
    };
    let rep = code_inventory(&m, only.as_deref())?;
    print!("{}", rep.summary_text());
    if let Some(out) = args.kv.get("out") {
        check_out_dir(Path::new(out))?;
        write_code_report(Path::new(out), args.kv.get("name").map(|s| s.as_str()).unwrap_or("mex"), &rep)?;
    }
    Ok(())
}

fn cmd_import(args: &Args) -> Result<(), String> {
    let t0 = std::time::Instant::now();
    let v = open(args, "vanilla", "GW_ISO_VANILLA")?;
    let m = open(args, "mod", "GW_ISO_MOD")?;
    let name = args.kv.get("name").ok_or("--name is required (the gdmod id prefix, e.g. akaneia)")?.clone();
    let version = args.kv.get("version").cloned().unwrap_or_default();
    let dry = args.flags.iter().any(|f| f == "dry-run");
    let out = PathBuf::from(args.kv.get("out").ok_or("--out is required (a mods folder outside the repository)")?);
    check_out_dir(&out)?;
    println!("vanilla  {}  ({} files{})", v.path.display(), v.files.len(), if v.is_image { format!(", {}", v.game_id) } else { String::new() });
    println!("mod      {}  ({} files{})", m.path.display(), m.files.len(), if m.is_image { format!(", {}", m.game_id) } else { String::new() });
    println!("diffing (same-size files are byte-compared) ...");
    let d = pack::delta(&v, &m, |_, _| {}).map_err(|e| e.to_string())?;
    let size = |l: &[String]| l.iter().map(|p| m.files[p].size()).sum::<u64>();
    println!("  NEW       {:>5} files {:>12}", d.new.len(), mb(size(&d.new)));
    println!("  MODIFIED  {:>5} files {:>12}", d.modified.len(), mb(size(&d.modified)));
    println!("  same      {:>5} files (not packed)", d.same);
    if !d.gone.is_empty() {
        println!("  note: {} vanilla paths are absent from the mod disc; the overlay cannot delete, so they stay (e.g. {})", d.gone.len(), d.gone[0]);
    }
    let mut pack_files: Vec<String> = d.new.iter().chain(&d.modified).cloned().collect();
    pack_files.sort();
    let has_mxdt = pack_files.iter().any(|p| p.eq_ignore_ascii_case("MxDt.dat"));
    let no_split = args.flags.iter().any(|f| f == "no-split");
    let mods = if has_mxdt && !no_split {
        let raw = m.read_path("MxDt.dat").map_err(|e| e.to_string())?;
        let t = hsd::Tables::parse(&raw).map_err(|e| format!("MxDt.dat: {e}"))?;
        let s = pack::split(&t, &pack_files, &name);
        for n in &s.notes {
            println!("  {n}");
        }
        println!("  m-ex tables v{}.{}: {} entities, {} shared files kept in the base", t.version.0, t.version.1, s.entities, s.shared);
        s.mods
    } else {
        vec![pack::ModPlan { id: name.clone(), kind: if has_mxdt { "base" } else { "misc" }, name: name.clone(), files: pack_files.clone() }]
    };
    let base_id = mods.iter().find(|m| m.kind == "base").map(|m| m.id.clone());
    let mut total = 0;
    for p in &mods {
        let sz = size(&p.files);
        total += sz;
        println!("{:34} {:8} {:4} files {:>9.1} MB  {}", p.id, p.kind, p.files.len(), sz as f64 / 1e6, p.name);
    }
    println!("{} gdmods, {} files, {:.1} MB", mods.len(), pack_files.len(), total as f64 / 1e6);
    if dry {
        println!("dry run: nothing written (would be {})", out.display());
        return Ok(());
    }
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    // per-file digests (the mod.json hash and the manifest need them)
    let mut digest = BTreeMap::new();
    for (i, p) in pack_files.iter().enumerate() {
        digest.insert(p.clone(), m.sha256(&m.files[p]).map_err(|e| e.to_string())?);
        if (i + 1) % 200 == 0 {
            println!("    ... hashed {}/{}", i + 1, pack_files.len());
        }
    }
    let mut manifest = String::from("{\n  \"version\": 1,\n");
    manifest += &format!("  \"name\": {},\n  \"vanilla\": {},\n  \"mod\": {},\n  \"mods\": {{\n", report::q(&name), report::q(&v.path.file_name().unwrap().to_string_lossy()), report::q(&m.path.file_name().unwrap().to_string_lossy()));
    let mut rows = vec![];
    for p in &mods {
        let hash = pack::mod_hash(&p.files, &digest);
        let json = pack::mod_json(p, &name, &version, if p.kind == "base" { None } else { base_id.as_deref() }, &hash);
        pack::write_mod(&out, p, &m, &json).map_err(|e| format!("{}: {e}", p.id))?;
        let files: Vec<String> = p.files.iter().map(|f| format!("      {}: {{\"size\": {}, \"sha256\": \"{}\"}}", report::q(f), m.files[f].size(), digest[f])).collect();
        rows.push(format!("    {}: {{\"kind\": \"{}\", \"hash\": \"{hash}\", \"files\": {{\n{}\n    }}}}", report::q(&p.id), p.kind, files.join(",\n")));
    }
    manifest += &rows.join(",\n");
    manifest += "\n  }\n}\n";
    // beside the mods, never inside one: a file inside a mod folder would mount as a disc path
    std::fs::write(out.join(format!("{name}.mex-import.json")), manifest).map_err(|e| e.to_string())?;
    if let Some(en) = args.kv.get("enable") {
        let mut s = String::from("# written by mex-import\n");
        for id in en.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            s += id;
            s.push('\n');
        }
        std::fs::write(out.join("enabled.txt"), s).map_err(|e| e.to_string())?;
    }
    println!("wrote {} gdmods to {} ({:.0}s)", mods.len(), out.display(), t0.elapsed().as_secs_f64());
    if !args.flags.iter().any(|f| f == "no-code") {
        let rep = code_inventory(&m, Some(&pack_files))?;
        print!("{}", rep.summary_text());
        write_code_report(&out, &name, &rep)?;
    }
    println!("This output is disc content. Do not commit it, ship it, or move it into a repository.");
    Ok(())
}

fn main() -> ExitCode {
    let r = parse_args().and_then(|a| match a.cmd.as_str() {
        "import" => cmd_import(&a),
        "code" => cmd_code(&a),
        "tables" => cmd_tables(&a),
        c => Err(format!("unknown command {c} (import | code | tables)")),
    });
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mex-import: {e}");
            ExitCode::FAILURE
        }
    }
}
