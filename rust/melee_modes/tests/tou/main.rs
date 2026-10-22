//! State oracle for the gm tournament / title TUs: runs every ported function on seeded synthetic
//! game worlds, and (with `--features c_oracle`, lib from oracle/tou/gen.py) the game's own C on
//! identical copies, comparing return value, callee call log and final state.
//!
//!   cargo test -p melee_modes --features c_oracle --test tou --release -- --nocapture
#![allow(dead_code, unused_imports)]
mod ext;
mod gen_tables;
mod recipes;
mod run;
mod space;
mod world;

use gen_tables::*;

/// the Rust statics are one shared world: tests must not overlap
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn cases() -> u32 {
    let dflt = if cfg!(feature = "c_oracle") { 200 } else { 10 };
    std::env::var("TOU_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(dflt)
}

#[test]
fn state_oracle() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    #[cfg(windows)]
    crash::install();
    let ctx = run::Ctx::new();
    let n = cases();
    let only = std::env::var("TOU_ONLY").ok();
    let (mut pass, mut fail, mut skipped_fns) = (vec![], vec![], vec![]);
    for &(tu, name, _i, spell, ret) in FUNCS {
        if let Some(o) = &only {
            if o != name {
                continue;
            }
        }
        eprintln!("-> {name}");
        let (mut ok, mut ran, mut bad) = (0u32, 0u32, 0u32);
        let mut first_bad = String::new();
        let start: u32 = std::env::var("TOU_START").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
        for c in start..n {
            if std::env::var("TOU_VERBOSE").is_ok() {
                eprintln!("CASE {c}");
            }
            let r = run::run_case(&ctx, tu, name, spell, ret, c);
            if r.skipped {
                continue;
            }
            ran += 1;
            #[cfg(feature = "c_oracle")]
            {
                let Some((co, detail)) = r.c.as_ref() else { ok += 1; continue };
                if co.hash == r.rust.hash {
                    ok += 1;
                } else {
                    bad += 1;
                    if first_bad.is_empty() {
                        first_bad = format!("case {c}: {detail}");
                    }
                }
            }
            #[cfg(not(feature = "c_oracle"))]
            {
                ok += 1;
            }
        }
        if ran == 0 {
            skipped_fns.push(name);
        } else if bad == 0 {
            pass.push((name, ran));
        } else {
            println!("FAIL {tu}::{name}: {bad}/{ran} cases differ\n{first_bad}");
            fail.push(name);
        }
        let _ = ok;
    }
    println!("pass {} fail {} no-recipe {}", pass.len(), fail.len(), skipped_fns.len());
    println!("no recipe: {:?}", skipped_fns);
    assert!(fail.is_empty(), "mismatching: {:?}", fail);
}

/// Per-region / per-object hashes of a space (position independent).
fn parts(sp: &space::Space, specs: &[GSpec]) -> Vec<(String, u64)> {
    let mut v = vec![];
    for (i, s) in specs.iter().enumerate() {
        if s.ro {
            continue;
        }
        let r = &sp.regions[i];
        let n = s.cmp.min(r.size);
        let mut h = space::Fnv::new();
        sp.hash_slots(&mut h, r.addr, n, n.max(1), s.ptrs);
        v.push((format!("global {}", s.name), h.0));
    }
    for (k, o) in sp.objs.iter().enumerate() {
        let (elem, ptrs) = space::layout_of(o.ty);
        let mut h = space::Fnv::new();
        sp.hash_slots(&mut h, unsafe { sp.base.add(o.off) }, o.size * o.count, elem, ptrs);
        let tn = if o.ty == space::RAW { "raw" } else if o.ty == space::PTRS { "ptrs" } else { TYPES[o.ty].0 };
        v.push((format!("obj {k} {tn} @{:#x}", o.off), h.0));
    }
    v
}

#[test]
fn world_is_position_independent() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let ctx = run::Ctx::new();
    let a = parts(&ctx.world(7).sp, &ctx.specs);
    let b = parts(&ctx.world(7).sp, &ctx.specs);
    let bad: Vec<_> = a.iter().zip(b.iter()).filter(|(x, y)| x != y).map(|(x, _)| x.0.clone()).collect();
    assert!(a.len() == b.len() && bad.is_empty(), "world differs between builds: {:?}", &bad[..bad.len().min(20)]);
}

#[test]
fn global_sizes() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let ctx = run::Ctx::new();
    let w = ctx.world(1);
    #[cfg(feature = "c_oracle")]
    for (i, s) in ctx.specs.iter().enumerate() {
        let (rs, cs) = (w.sp.regions[i].size, ctx.c.regions[i].size);
        if rs != cs || cs != s.c_size {
            println!("SIZE {} rust {} c {} spec c_size {} cmp {} ro {}", s.name, rs, cs, s.c_size, s.cmp, s.ro);
        }
    }
}

#[cfg(windows)]
mod crash {
    use std::ffi::c_void;
    #[repr(C)]
    pub struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut c_void,
        n: u32,
        info: [usize; 15],
    }
    #[repr(C)]
    pub struct ExceptionPointers {
        rec: *mut ExceptionRecord,
        ctx: *mut c_void,
    }
    extern "system" {
        fn SetUnhandledExceptionFilter(f: unsafe extern "system" fn(*mut ExceptionPointers) -> i32) -> *mut c_void;
        fn GetModuleHandleA(n: *const u8) -> *mut c_void;
    }
    unsafe extern "system" fn handler(p: *mut ExceptionPointers) -> i32 {
        let r = &*(*p).rec;
        let base = GetModuleHandleA(std::ptr::null()) as usize;
        eprintln!("CRASH code {:#x} at {:#x} (module+{:#x}) info {:x?}", r.code, r.address as usize, r.address as usize - base, &r.info[..2]);
        1
    }
    pub fn install() {
        unsafe { SetUnhandledExceptionFilter(handler) };
    }
}
