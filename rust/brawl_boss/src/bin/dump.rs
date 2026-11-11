//! `brawl_boss_dump <stage archive or boss file> [out]`: a human-readable listing of a decoded boss file.
//!
//! The listing is derived from the player's disc: it goes to the terminal, or to `out` (keep that under `_build`,
//! never inside a repository). File paths are never printed.

use brawl_boss::archive;
use brawl_boss::file::{BossFile, CATEGORIES, THREADS};
use brawl_boss::script::{Arg, Target};
use std::fmt::Write as _;

fn target(bf: &BossFile, t: &Target) -> String {
    match t {
        Target::None => "-".into(),
        Target::Local(o) => format!("@{o:#06x}"),
        Target::Import(i) => format!("<{}>", bf.import_name(*i).unwrap_or("?")),
        Target::Unresolved(v) => format!("?{v:#x}"),
    }
}

fn arg(bf: &BossFile, a: &Arg) -> String {
    match a {
        Arg::Int(i) => format!("{i}"),
        Arg::Scalar(f) => format!("{f}f"),
        Arg::Ptr(t) => target(bf, t),
        Arg::Bool(b) => format!("{b}"),
        Arg::Offset(o) => format!("off:{o:#x}"),
        Arg::Var(v) => format!("v{}.{}.{}", v.mem, v.kind, v.id),
        Arg::Req { not, id } => format!("{}R{id}", if *not { "!" } else { "" }),
        Arg::Other { ty, raw } => format!("t{ty}:{raw:#x}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: brawl_boss_dump <stage archive or boss file> [out]");
        std::process::exit(2);
    }
    let raw = match std::fs::read(&args[1]) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("cannot read the input ({})", e.kind());
            std::process::exit(1);
        }
    };
    let bytes = if archive::looks_like_boss_file(&raw, None) {
        raw
    } else {
        match archive::find_petey_file(&raw) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("boss file not found: {e}");
                std::process::exit(1);
            }
        }
    };
    let bf = match BossFile::parse(&bytes) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("decode failed: {e}");
            std::process::exit(1);
        }
    };
    let mut o = String::new();
    let h = bf.header;
    let _ = writeln!(o, "# boss file: {} bytes, data {:#x}, {} relocations, {} roots, {} imports", h.file_size, h.data_size, h.reloc_count, h.root_count, h.import_count);
    for r in &bf.roots {
        let _ = writeln!(o, "root {:<24} @{:#06x}", r.name, r.at);
    }
    for (i, im) in bf.imports.iter().enumerate() {
        let _ = writeln!(o, "import {i:2} {} ({} uses)", im.name, im.fields.len());
    }
    let _ = writeln!(o, "\n# motions");
    for (i, m) in bf.motions.iter().enumerate() {
        let _ = writeln!(o, "{i:2} {:<20} flags {:#010x}", m.name, m.flags);
    }
    let cat = ["game", "effect", "sound"];
    let _ = writeln!(o, "\n# timelines (category, thread, motion)");
    for c in 0..CATEGORIES {
        for t in 0..THREADS {
            let s: Vec<String> = bf.timelines[c][t].iter().map(|x| target(&bf, x)).collect();
            let _ = writeln!(o, "{} {t}: {}", cat[c], s.join(" "));
        }
    }
    let _ = writeln!(o, "\n# status scripts (enter / loop / exit)");
    for s in 0..bf.status_enter.len() {
        let _ = writeln!(o, "{s:2} {} {} {}", target(&bf, &bf.status_enter[s]), target(&bf, &bf.status_loop[s]), target(&bf, &bf.status_exit[s]));
    }
    let _ = writeln!(o, "\n# rules (prio mask cond action status default lock)");
    for (i, r) in bf.rules.iter().enumerate() {
        let _ = writeln!(o, "{i:2} {:5} {:#04x} {} {} {} {} {}", r.priority, r.target_mask, target(&bf, &r.condition), target(&bf, &r.action), r.target_status, r.default_enabled as u8, r.lock as u8);
    }
    let _ = writeln!(o, "\ninit {}", target(&bf, &bf.init));
    let _ = writeln!(o, "\n# params");
    for (id, _, _) in brawl_boss::params::LAYOUT {
        if let Some(f) = bf.params.raw(id) {
            let w: Vec<String> = f.iter().map(|x| format!("{x:08x}")).collect();
            let _ = writeln!(o, "{id} {}", w.join(" "));
        }
    }
    let _ = writeln!(o, "\n# scripts");
    for (at, s) in &bf.scripts {
        let _ = writeln!(o, "@{at:#06x}");
        for c in &s.cmds {
            let a: Vec<String> = c.args.iter().map(|x| arg(&bf, x)).collect();
            let _ = writeln!(o, "  {:04X} {}", c.id(), a.join(" "));
        }
    }
    match args.get(2) {
        Some(p) => {
            if let Err(e) = std::fs::write(p, o) {
                eprintln!("cannot write the listing ({})", e.kind());
                std::process::exit(1);
            }
        }
        None => print!("{o}"),
    }
}
