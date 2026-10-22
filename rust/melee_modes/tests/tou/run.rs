//! Case runner: builds a world, runs the Rust port (and, with the oracle library, the C) on it,
//! hashes (call log, return value, final state) and compares.
use std::collections::HashMap;
use std::vec::Vec;

use melee_modes::tu::gmtoulib::hook;

use super::ext::{rust_hook, with_run, Persist, Rng, Run};
use super::gen_tables::*;
use super::recipes::{build_args, Arg};
use super::space::{Fnv, Space};
use super::world::{build, World};

fn names_null(name: &str) -> bool {
    matches!(name, "fn_80195AF0" | "fn_80195CCC" | "fn_8018ECA8" | "fn_8018F00C")
}

pub struct Ctx {
    pub specs: Vec<GSpec>,
    pub init_a: Vec<Vec<u8>>,
    pub fn_names_a: HashMap<usize, u64>,
    #[cfg(feature = "c_oracle")]
    pub c: CSide,
}

#[cfg(feature = "c_oracle")]
pub struct CSide {
    pub regions: Vec<super::space::Region>,
    pub fn_names: HashMap<usize, u64>,
    pub fn_idx: HashMap<(String, String), i32>,
    pub fn_addr: HashMap<String, usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub hash: u64,
    pub ret: u64,
    pub log: u64,
    pub ncalls: u32,
    pub trace: Vec<String>,
}

impl Ctx {
    pub fn new() -> Ctx {
        unsafe { hook::HOOK = Some(rust_hook) };
        let specs = globals();
        let init_a: Vec<Vec<u8>> = specs
            .iter()
            .map(|s| {
                let v = (s.rust)();
                unsafe { std::slice::from_raw_parts(v[0].0, v[0].1) }.to_vec()
            })
            .collect();
        let mut fn_names_a = HashMap::new();
        for (n, a) in rust_fn_addrs() {
            fn_names_a.insert(a, super::space::name_hash(n));
        }
        #[cfg(feature = "c_oracle")]
        let c = unsafe { c_side(&specs) };
        Ctx {
            specs,
            init_a,
            fn_names_a,
            #[cfg(feature = "c_oracle")]
            c,
        }
    }

    pub fn world(&self, seed: u64) -> World {
        let mut w = build(seed, &self.specs, &self.init_a);
        w.sp.fn_names = self.fn_names_a.clone();
        w
    }
}

#[cfg(feature = "c_oracle")]
mod creg {
    use std::ffi::{c_char, c_void, CStr};
    use std::sync::Mutex;
    pub static GLOBALS: Mutex<Vec<(String, usize, u64)>> = Mutex::new(Vec::new());
    pub static FUNCS: Mutex<Vec<(String, usize)>> = Mutex::new(Vec::new());
    pub extern "C" fn reg_global(n: *const c_char, a: *mut c_void, s: u64) {
        GLOBALS.lock().unwrap().push((unsafe { CStr::from_ptr(n) }.to_str().unwrap().to_string(), a as usize, s));
    }
    pub extern "C" fn reg_func(n: *const c_char, a: *mut c_void) {
        FUNCS.lock().unwrap().push((unsafe { CStr::from_ptr(n) }.to_str().unwrap().to_string(), a as usize));
    }
}

#[cfg(feature = "c_oracle")]
unsafe fn c_side(specs: &[GSpec]) -> CSide {
    use std::ffi::CStr;
    for tu in TUS.iter().chain(["stub"].iter()) {
        cffi::globals(tu, creg::reg_global);
        cffi::funcs(tu, creg::reg_func);
    }
    let globals = creg::GLOBALS.lock().unwrap().clone();
    let gm: HashMap<&str, (usize, u64)> = globals.iter().map(|(n, a, s)| (n.as_str(), (*a, *s))).collect();
    let regions = specs
        .iter()
        .map(|s| {
            let (a, sz) = gm.get(s.name).unwrap_or_else(|| panic!("C global {} not registered", s.name));
            super::space::Region { addr: *a as *mut u8, size: *sz as usize, dups: vec![] }
        })
        .collect();
    let mut fn_names = HashMap::new();
    let mut fn_addr = HashMap::new();
    for (n, a) in creg::FUNCS.lock().unwrap().iter() {
        fn_names.insert(*a, super::space::name_hash(n));
        fn_addr.insert(n.clone(), *a);
    }
    let mut fn_idx = HashMap::new();
    for tu in TUS {
        for i in 0..cffi::nfuncs(tu) {
            let n = CStr::from_ptr(cffi::name(tu, i)).to_str().unwrap().to_string();
            fn_idx.insert((tu.to_string(), n), i);
        }
    }
    CSide { regions, fn_names, fn_idx, fn_addr }
}

fn words(args: &[Arg]) -> Vec<u64> {
    args.iter().map(|a| a.word()).collect()
}

fn finish(sp: &Space, specs: &[GSpec], run: Run, ret: u64, ret_class: &str, name: &str) -> Outcome {
    let mut h = Fnv::new();
    h.u64(run.log.0);
    let mut r = if ret_class == "p" { sp.norm(ret) } else { ret };
    // functions returning a string literal: compare the string, not its address
    if ret_class == "p" && r == ret && ret != 0 && name == "fn_8018F5F0" {
        let mut f = Fnv::new();
        f.str(&unsafe { std::ffi::CStr::from_ptr(ret as usize as *const std::ffi::c_char) }.to_string_lossy());
        r = 0xD000_0000_0000_0000 | (f.0 & 0x0FFF_FFFF_FFFF_FFFF);
    }
    h.u64(r);
    sp.state_hash(specs, &mut h);
    Outcome { hash: h.0, ret: r, log: run.log.0, ncalls: run.ncalls, trace: run.trace.unwrap_or_default() }
}

pub fn run_rust(ctx: &Ctx, w: &mut World, seed: u64, tu: &str, name: &str, ret_class: &str, args: &[Arg], trace: bool) -> Outcome {
    let wd = words(args);
    let mut run = Run {
        sp: &mut w.sp as *mut Space,
        persist: Persist { names_null: names_null(name), ..w.persist },
        rng: Rng::new(seed ^ 0x5EED),
        log: Fnv::new(),
        trace: if trace || std::env::var("TOU_TRACE").is_ok() { Some(vec![]) } else { None },
        ncalls: 0,
    };
    let ret = with_run(&mut run, || unsafe { call_rust(tu, name, &wd) });
    finish(&w.sp, &ctx.specs, run, ret, ret_class, name)
}

#[cfg(feature = "c_oracle")]
pub fn run_c(ctx: &Ctx, w: &World, seed: u64, tu: &str, name: &str, ret_class: &str, args: &[Arg], trace: bool) -> (Outcome, Space) {
    let mut b = Space::new();
    b.regions = ctx.c.regions.clone();
    b.fn_names = ctx.c.fn_names.clone();
    w.sp.copy_world_to(&mut b, &ctx.specs);
    b.bf_to_native_all(&ctx.specs);
    let wd: Vec<u64> = args
        .iter()
        .map(|a| match a {
            Arg::P(p) => b.denorm(w.sp.norm(*p as u64)),
            Arg::Cb(k) => if *k == 0 { 0 } else { *ctx.c.fn_addr.get(super::recipes::CB_NAMES[*k as usize]).expect("C callback") as u64 },
            o => o.word(),
        })
        .collect();
    let plink = b.denorm(w.sp.norm(w.persist.plink as u64)) as usize;
    let persist = Persist { plink, names_null: names_null(name), ..w.persist };
    let mut run = Run {
        sp: &mut b as *mut Space,
        persist,
        rng: Rng::new(seed ^ 0x5EED),
        log: Fnv::new(),
        trace: if trace || std::env::var("TOU_TRACE").is_ok() { Some(vec![]) } else { None },
        ncalls: 0,
    };
    let idx = *ctx.c.fn_idx.get(&(tu.to_string(), name.to_string())).unwrap_or_else(|| panic!("C fn {tu}::{name} missing"));
    let ret = with_run(&mut run, || unsafe { cffi::call(tu, idx, wd.as_ptr()) });
    b.bf_to_rust_all(&ctx.specs);
    let out = finish(&b, &ctx.specs, run, ret, ret_class, name);
    (out, b)
}

/// Case seed for (function, case number).
pub fn case_seed(name: &str, case: u32) -> u64 {
    let mut f = Fnv::new();
    f.str(name);
    f.u64(case as u64);
    f.0
}

pub struct CaseResult {
    pub rust: Outcome,
    #[cfg(feature = "c_oracle")]
    pub c: Option<(Outcome, String)>,
    pub skipped: bool,
}

/// Run one case on the Rust side (and the C side with the oracle library).
pub fn run_case(ctx: &Ctx, tu: &str, name: &str, spell: &[&str], ret_class: &str, case: u32) -> CaseResult {
    let seed = case_seed(name, case);
    let mut w = ctx.world(seed);
    let args = match build_args(name, spell, &mut w) {
        Some(a) => a,
        None => {
            return CaseResult {
                rust: Outcome::default(),
                #[cfg(feature = "c_oracle")]
                c: None,
                skipped: true,
            }
        }
    };
    w.sp.bf_clean_rust_all(&ctx.specs);
    if std::env::var("TOU_DUMP").is_ok() {
        let me = unsafe { &*std::ptr::addr_of!(melee_modes::tu::gmtoulib::gm_80477738) };
        for (i, p) in me.player_standings.iter().enumerate() {
            eprintln!("DUMP standing {i}: pkind {} small_loser {}", p.pkind, p.is_small_loser);
        }
        let e = unsafe { &(*std::ptr::addr_of!(melee_modes::tu::gmtoulib::lbl_80473AB8)) };
        eprintln!("DUMP e0.x0 {} slots x4E {:?}", e[0].x0, e[0].slots.iter().map(|s| s.x4E).collect::<Vec<_>>());
    }
    // the C side starts from the same world, so copy before the Rust run mutates it
    #[cfg(feature = "c_oracle")]
    let only_rust = std::env::var("TOU_SIDE").map_or(false, |s| s == "rust");
    #[cfg(feature = "c_oracle")]
    let (c_out, c_space) = if std::env::var("TOU_SIDE").map_or(false, |s| s == "c") || !only_rust {
        let r = run_c(ctx, &w, seed, tu, name, ret_class, &args, false);
        if std::env::var("TOU_SIDE").map_or(false, |s| s == "c") {
            return CaseResult { rust: Outcome::default(), c: None, skipped: false };
        }
        r
    } else {
        (Outcome::default(), Space::new())
    };
    let rust = run_rust(ctx, &mut w, seed, tu, name, ret_class, &args, false);
    #[cfg(feature = "c_oracle")]
    if only_rust {
        return CaseResult { rust, c: None, skipped: false };
    }
    #[cfg(feature = "c_oracle")]
    let c = {
        let mut detail = String::new();
        if c_out.hash != rust.hash {
            // rerun both with call traces to find the first divergence
            let mut w2 = ctx.world(seed);
            let args2 = build_args(name, spell, &mut w2).unwrap();
            w2.sp.bf_clean_rust_all(&ctx.specs);
            let (cc, cs) = run_c(ctx, &w2, seed, tu, name, ret_class, &args2, true);
            let rr = run_rust(ctx, &mut w2, seed, tu, name, ret_class, &args2, true);
            detail += &format!("rerun hash rust {:#x} c {:#x} (first run rust {:#x} c {:#x})
", rr.hash, cc.hash, rust.hash, c_out.hash);
            detail += &format!("ret rust {:#x} c {:#x}; log rust {:#x} c {:#x}; calls {} vs {}\n", rr.ret, cc.ret, rr.log, cc.log, rr.ncalls, cc.ncalls);
            let n = rr.trace.len().max(cc.trace.len());
            for i in 0..n {
                let (a, b) = (rr.trace.get(i), cc.trace.get(i));
                if a != b {
                    detail += &format!("first call divergence at #{i}:\n  rust: {a:?}\n  c:    {b:?}\n");
                    break;
                }
            }
            detail += &w2.sp.describe_diff(&cs, &ctx.specs);
            detail += &format!("args {:?}\n", args2.iter().map(|a| a.word()).collect::<Vec<_>>());
            let _ = &c_space;
        }
        Some((c_out, detail))
    };
    CaseResult {
        rust,
        #[cfg(feature = "c_oracle")]
        c,
        skipped: false,
    }
}
