//! `mex-import referee` / `referee-plan` (notes/referee.md). The DOL comes from `--dol`, `$REFEREE_DOL` or the
//! checkout's `orig/` (never printed). Output files hold engine values and addresses only, no disc bytes.

use crate::dol::{symbols, Dol};
use crate::referee::{parse_line, Referee};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

fn referee(kv: &BTreeMap<String, String>) -> Result<Referee, String> {
    let p = kv.get("dol").map(PathBuf::from).or_else(Dol::find).ok_or("no main.dol (--dol or $REFEREE_DOL)")?;
    Ok(Referee::new(Dol::load(&p)?, symbols()?))
}

/// The static shape of each named function: size, conditional branches, callees, instructions the interpreter
/// cannot run in it and in its callee tree (to choose the leaf class).
pub fn plan(kv: &BTreeMap<String, String>) -> Result<(), String> {
    let r = referee(kv)?;
    let fns: Vec<String> = kv.get("fns").ok_or("--fns a,b")?.split(',').map(str::to_string).collect();
    for n in fns {
        let Some(f) = r.func(&n).cloned() else {
            println!("{n}: no symbol");
            continue;
        };
        // the callee tree, breadth first, to depth 6
        let mut seen = BTreeSet::new();
        let mut q = vec![(f.addr, 0)];
        let mut tree = vec![];
        while let Some((a, d)) = q.pop() {
            if !seen.insert(a) || d > 6 {
                continue;
            }
            let Some(s) = r.func_at(a).cloned() else { continue };
            if a != f.addr {
                tree.push(format!("{}{}", s.name, match r.unsupported(&s) {
                    0 => String::new(),
                    k => format!("[{k} unsupported]"),
                }));
            }
            for c in r.callees(&s) {
                q.push((c, d + 1));
            }
        }
        println!("{n} {:08X} size {:#x} cond-branches {} unsupported {} callees: {}", f.addr, f.size, r.cond_branches(&f).len(), r.unsupported(&f), tree.join(" "));
    }
    Ok(())
}

pub fn run(kv: &BTreeMap<String, String>) -> Result<(), String> {
    let mut r = referee(kv)?;
    let caps = kv.get("capture").ok_or("--capture FILE[,FILE]")?;
    let only: Option<BTreeSet<String>> = kv.get("fns").map(|v| v.split(',').map(str::to_string).collect());
    let examples: usize = kv.get("examples").and_then(|v| v.parse().ok()).unwrap_or(5);
    #[derive(Default)]
    struct Per {
        calls: u64,
        compared: u64,
        mismatched: u64,
        faults: BTreeMap<String, u64>,
        global_reads: BTreeSet<u32>,
        entered: BTreeSet<u32>,
        examples: Vec<String>,
    }
    let mut per: BTreeMap<String, Per> = BTreeMap::new();
    // --cov-in: outcomes already covered (a previous run's --cov-out); "new" below counts what this run adds
    let mut base: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
    if let Some(ci) = kv.get("cov-in") {
        if let Ok(t) = std::fs::read_to_string(ci) {
            for l in t.lines() {
                let v: Vec<&str> = l.split('\t').collect();
                if let (Some(a), Some(t1), Some(n1)) = (v.first().and_then(|a| u32::from_str_radix(a, 16).ok()), v.get(1).and_then(|x| x.parse().ok()), v.get(2).and_then(|x| x.parse().ok())) {
                    base.insert(a, (t1, n1));
                }
            }
        }
    }
    let mut mism_tsv = String::from("function\tsource\tframe\twhat\n");
    for cap in caps.split(',') {
        let txt = std::fs::read_to_string(cap).map_err(|e| format!("{cap}: {e}"))?;
        let src = std::path::Path::new(cap).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        for l in txt.lines() {
            let Ok(c) = parse_line(l) else { continue };
            if only.as_ref().is_some_and(|o| !o.contains(&c.name)) {
                continue;
            }
            let o = r.check(&c);
            let p = per.entry(c.name.clone()).or_default();
            p.calls += 1;
            p.global_reads.extend(o.global_reads.iter().copied());
            p.entered.extend(o.entered.iter().copied());
            if let Some(f) = o.fault {
                *p.faults.entry(f).or_default() += 1;
                continue;
            }
            p.compared += 1;
            if !o.mismatches.is_empty() {
                p.mismatched += 1;
                let what = o.mismatches.join("; ");
                let _ = writeln!(mism_tsv, "{}\t{src}\t{}\t{what}", c.name, c.frame);
                if p.examples.len() < examples {
                    p.examples.push(format!("{src} frame {}: {what}", c.frame));
                }
            }
        }
    }
    // coverage: each checked function's own conditional branches, by outcome; and its retail callees'
    let mut sum = String::new();
    let mut unc = String::from("function\taddress\toutcome\n");
    let mut cov_tsv = String::from("function\tcalls\tcompared\tmismatched\toutcomes_hit\toutcomes\tcallee_outcomes_hit\tcallee_outcomes\n");
    let (mut tc, mut tcmp, mut tm) = (0, 0, 0);
    for (name, p) in &per {
        let f = r.func(name).cloned().unwrap();
        let own = r.cond_branches(&f);
        let hit = |a: &u32| r.cov.get(a).map_or(0, |e| (e.0 > 0) as u32 + (e.1 > 0) as u32);
        let own_hit: u32 = own.iter().map(hit).sum();
        let mut callee_b = vec![];
        for &e in p.entered.iter().filter(|&&e| e != f.addr) {
            if let Some(s) = r.func_at(e) {
                callee_b.extend(r.cond_branches(&s.clone()));
            }
        }
        let callee_hit: u32 = callee_b.iter().map(hit).sum();
        for a in &own {
            let e = r.cov.get(a).copied().unwrap_or((0, 0));
            if e.0 == 0 {
                let _ = writeln!(unc, "{name}\t{a:08X}\ttaken");
            }
            if e.1 == 0 {
                let _ = writeln!(unc, "{name}\t{a:08X}\tnot-taken");
            }
        }
        let callees: Vec<String> = p.entered.iter().filter(|&&e| e != f.addr).filter_map(|&e| r.func_at(e).map(|s| s.name.clone())).collect();
        let _ = writeln!(
            sum,
            "{name}: {} calls, {} compared, {} mismatched; retail branch outcomes {own_hit}/{} (callees {callee_hit}/{}); callees interpreted: {}",
            p.calls,
            p.compared,
            p.mismatched,
            2 * own.len(),
            2 * callee_b.len(),
            if callees.is_empty() { "none".into() } else { callees.join(" ") }
        );
        for (f, k) in &p.faults {
            let _ = writeln!(sum, "  not compared ({k}): {f}");
        }
        if !p.global_reads.is_empty() {
            let g: Vec<String> = p.global_reads.iter().take(8).map(|a| format!("{a:08X}")).collect();
            let _ = writeln!(sum, "  read mutable globals (image values, not the engine's): {}{}", g.join(" "), if p.global_reads.len() > 8 { " ..." } else { "" });
        }
        for e in &p.examples {
            let _ = writeln!(sum, "  mismatch: {e}");
        }
        let _ = writeln!(cov_tsv, "{name}\t{}\t{}\t{}\t{own_hit}\t{}\t{callee_hit}\t{}", p.calls, p.compared, p.mismatched, 2 * own.len(), 2 * callee_b.len());
        tc += p.calls;
        tcmp += p.compared;
        tm += p.mismatched;
    }
    // outcomes this run adds over --cov-in, among the checked functions' own branches
    let mut new_out = 0u32;
    for name in per.keys() {
        let f = r.func(name).cloned().unwrap();
        for a in r.cond_branches(&f) {
            let e = r.cov.get(&a).copied().unwrap_or((0, 0));
            let b = base.get(&a).copied().unwrap_or((0, 0));
            new_out += (e.0 > 0 && b.0 == 0) as u32 + (e.1 > 0 && b.1 == 0) as u32;
        }
    }
    let _ = writeln!(sum, "total: {} functions, {tc} calls, {tcmp} compared, {tm} mismatched; new branch outcomes vs --cov-in: {new_out}", per.len());
    if let Some(co) = kv.get("cov-out") {
        let mut t = String::new();
        let mut all = base.clone();
        for (a, e) in &r.cov {
            let x = all.entry(*a).or_default();
            x.0 += e.0;
            x.1 += e.1;
        }
        for (a, e) in &all {
            let _ = writeln!(t, "{a:08X}\t{}\t{}", e.0, e.1);
        }
        std::fs::write(co, t).map_err(|e| e.to_string())?;
    }
    print!("{sum}");
    if let Some(out) = kv.get("out") {
        std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
        for (n, t) in [("summary.txt", &sum), ("mismatches.tsv", &mism_tsv), ("uncovered.tsv", &unc), ("coverage.tsv", &cov_tsv)] {
            std::fs::write(format!("{out}/{n}"), t).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
