//! Which instructions the retail game code uses (the DOL referee's step 1 scope, notes/referee.md). Local only: skips
//! when no main.dol is found (`REFEREE_DOL`). Prints counts per decoded op form; no instruction words are printed.
use mex_import::dol::{symbols, Dol};
use mex_import::ppc::ir::*;

fn form(op: &Op, w: u32) -> String {
    let rc = w & 1 != 0;
    match op {
        Op::Arith { op, rc, .. } => format!("{op:?}{}", if *rc { "." } else { "" }),
        Op::Logic { op, rc, .. } => format!("{op:?}{}", if *rc { "." } else { "" }),
        Op::Un { op, rc, .. } => format!("{op:?}{}", if *rc { "." } else { "" }),
        Op::FArith { op, single, .. } => format!("f{op:?}{}{}", if *single { "s" } else { "" }, if rc { "." } else { "" }),
        Op::FUnary { op, .. } => format!("f{op:?}{}", if rc { "." } else { "" }),
        Op::Fsel { .. } => format!("fsel{}", if rc { "." } else { "" }),
        Op::CrLog { op, .. } => format!("cr{op:?}"),
        Op::Load { size, sign, update, rev, ea, .. } => format!("ld{size}{}{}{}{}", if *sign { "a" } else { "" }, if *update { "u" } else { "" }, if *rev { "br" } else { "" }, if matches!(ea, Ea::Idx(_)) { "x" } else { "" }),
        Op::Store { size, update, rev, ea, .. } => format!("st{size}{}{}{}", if *update { "u" } else { "" }, if *rev { "br" } else { "" }, if matches!(ea, Ea::Idx(_)) { "x" } else { "" }),
        Op::LoadF { single, update, ea, .. } => format!("lf{}{}{}", if *single { "s" } else { "d" }, if *update { "u" } else { "" }, if matches!(ea, Ea::Idx(_)) { "x" } else { "" }),
        Op::StoreF { single, update, ea, .. } => format!("stf{}{}{}", if *single { "s" } else { "d" }, if *update { "u" } else { "" }, if matches!(ea, Ea::Idx(_)) { "x" } else { "" }),
        Op::Bad => format!("BAD opcd {}{}", w >> 26, if w >> 26 == 31 || w >> 26 == 63 || w >> 26 == 4 || w >> 26 == 19 { format!(" xo {}", (w >> 1) & 0x3FF) } else { String::new() }),
        Op::BadSpr { spr } => format!("BADSPR {spr}"),
        other => {
            let d = format!("{other:?}");
            let n = d.split([' ', '{']).next().unwrap_or("").to_string();
            match other {
                Op::AddIc { rc: true, .. } | Op::Rlwinm { rc: true, .. } | Op::Rlwimi { rc: true, .. } | Op::Rlwnm { rc: true, .. } | Op::Srawi { rc: true, .. } => n + ".",
                Op::CmpI { unsigned: true, .. } | Op::Cmp { unsigned: true, .. } => n + "l",
                _ => n,
            }
        }
    }
}

#[test]
fn census() {
    let Some(p) = Dol::find() else {
        eprintln!("dol_census: no main.dol (set REFEREE_DOL): skipped");
        return;
    };
    let dol = Dol::load(&p).unwrap();
    let syms = symbols().unwrap();
    // game code = every symbols.txt function (the SDK, MSL and HSD included); a word is counted once
    let mut counts: std::collections::BTreeMap<String, (u64, Vec<String>)> = Default::default();
    let (mut words, mut funcs) = (0u64, 0u64);
    for s in syms.iter().filter(|s| s.function && s.size > 0 && dol.in_text(s.addr)) {
        funcs += 1;
        for a in (s.addr..s.addr + s.size).step_by(4) {
            let Some(w) = dol.word(a) else { continue };
            words += 1;
            let e = counts.entry(form(&decode(w, a), w)).or_default();
            e.0 += 1;
            if e.1.len() < 4 && !e.1.contains(&s.name) {
                e.1.push(s.name.clone());
            }
        }
    }
    println!("dol_census: {funcs} functions, {words} words");
    for (k, (n, f)) in &counts {
        println!("{n:>8}  {k:<24} {}", f.join(" "));
    }
}
