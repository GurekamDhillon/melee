//! Slippi's netplay code list applied to the retail image (notes/referee.md section 9). Local only: skips without a
//! main.dol (`REFEREE_DOL`) or without the ssbm-asm checkout (`SLIPPI_ASM`, default the workspace's
//! `_build/slippi-src/ssbm-asm`). Prints counts and addresses only, never code words.
use mex_import::dol::{symbols, Dol};
use mex_import::gecko;
use mex_import::ppc::ir::{decode, Op};

fn ini() -> Option<String> {
    let root = std::env::var("SLIPPI_ASM").unwrap_or_else(|_| "E:/Projects/Melee Workspace/_build/slippi-src/ssbm-asm".into());
    std::fs::read_to_string(format!("{root}/Output/Netplay/GALE01r2.ini")).ok()
}

#[test]
fn netplay_list_applies() {
    let (Some(p), Some(text)) = (Dol::find(), ini()) else {
        eprintln!("gecko_apply: no main.dol or no ssbm-asm checkout: skipped");
        return;
    };
    let mut dol = Dol::load(&p).unwrap();
    let orig = Dol::load(&p).unwrap();
    let codes = gecko::parse_ini(&text, false);
    let names: Vec<&str> = codes.iter().map(|c| c.name.split('[').next().unwrap_or("").trim()).collect();
    println!("gecko_apply: {} enabled code groups: {names:?}", codes.len());
    let rep = gecko::apply(&mut dol, &codes);
    println!("gecko_apply: {} writes, {} inserts, {} outside the DOL, {} skipped", rep.writes, rep.inserts.len(), rep.outside.len(), rep.skipped.len());
    let syms = symbols().unwrap();
    let fname = |a: u32| syms.iter().filter(|s| s.function && a >= s.addr && a < s.addr + s.size).map(|s| s.name.clone()).next().unwrap_or_else(|| "?".into());
    for &(k, a) in &rep.skipped {
        println!("gecko_apply:   skipped type {k:02X} at {a:08X} ({})", fname(a));
    }
    for &a in &rep.outside {
        println!("gecko_apply:   outside {a:08X}");
    }
    // every insert: the hook branches to its block, the block's last word branches back to hook + 4, and every word
    // of the block that is reached as code decodes (data tables after a `blrl` may not: counted, not failed)
    let mut odd = 0;
    let mut twice = 0;
    for (k, &(hook, start, n)) in rep.inserts.iter().enumerate() {
        // a hook inserted twice: the later code's branch wins (the handler writes them in list order)
        if rep.inserts[k + 1..].iter().any(|i| i.0 == hook) {
            twice += 1;
            println!("gecko_apply:   {hook:08X} ({}) inserted again later: the later block wins", fname(hook));
            continue;
        }
        assert_eq!(decode(dol.word(hook).unwrap(), hook), Op::B { target: start, lk: false }, "hook {hook:08X}");
        let last = start + 4 * (n - 1);
        assert_eq!(decode(dol.word(last).unwrap(), last), Op::B { target: hook + 4, lk: false }, "return of {hook:08X}");
        odd += (start..last).step_by(4).filter(|&a| !decode(dol.word(a).unwrap(), a).gekko_executable()).count();
        // the replaced retail word must still exist in the original (it is not lost: the code re-issues it or replaces it)
        assert!(orig.word(hook).is_some());
    }
    println!("gecko_apply: {twice} hooks overridden");
    println!("gecko_apply: {odd} words inside inserted blocks do not decode (bl-over-data tables, strings)");
    // the [affects-gameplay] hooks of the list are all among the inserts
    let tagged = [
        0x8016_E510u32, 0x800D_65EC, 0x800C_9A44, 0x8006_B460, 0x8008_E54C, 0x8009_A0B8, 0x8009_98A4, 0x8009_3294, 0x8009_08F4,
        0x8012_39A8, 0x801C_154C, 0x801C_D37C, 0x801D_69B0, 0x8006_8EEC, 0x8006_A880, 0x800D_4C1C, 0x800D_4D68, 0x8008_0E80,
        0x8021_AAE4, 0x801C_65C8, 0x801D_4760, 0x801D_457C, 0x8001_65AC, 0x8008_653C, 0x800A_C5B8, 0x801D_24FC, 0x800D_B880,
        0x800D_BBD4, 0x8008_F090,
    ];
    for a in tagged {
        let hit = rep.inserts.iter().any(|i| i.0 == a) || dol.word(a) != orig.word(a);
        println!("gecko_apply:   [affects-gameplay] {a:08X} {} {}", fname(a), if hit { "patched" } else { "NOT PATCHED" });
        assert!(hit, "{a:08X} not patched");
    }
}
