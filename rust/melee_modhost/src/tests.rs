//! Loader refusals (spec V6) on hand-written modules; the accepting path is covered by mods/hello_mod.
use super::*;

fn module(body: &str) -> Vec<u8> {
    wat::parse_str(format!("(module {body})")).unwrap()
}

/// A valid manifest at 1024 with `hooks` at 2048 (one decl: addr, mode, fn 0), memory pages min/max.
fn mod_with(addr: u32, mode: u32, mem: &str, extra: &str, abi_min: u32) -> Vec<u8> {
    mod_full("t.ok", &[(addr, mode)], 0, mem, &format!("(func $f0 (param i32 f32)) {extra}"), "", abi_min)
}

/// A manifest with `hooks` (addr, mode) -> fn i, required features `req`; `funcs` defines `$f0..`, `imports` comes first.
fn mod_full(id: &str, hooks: &[(u32, u32)], req: u32, mem: &str, funcs: &str, imports: &str, abi_min: u32) -> Vec<u8> {
    let mut m = vec![0u8; 124];
    let w = |m: &mut Vec<u8>, i: usize, v: u32| m[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    for (i, v) in [MANIFEST_MAGIC, 124, abi_min, ABI_VERSION, 1, 0x10000, req, 0, 4, hooks.len() as u32, 2048].iter().enumerate() { w(&mut m, i, *v); }
    m[44..44 + id.len()].copy_from_slice(id.as_bytes());
    let mut h = vec![0u8; 20 * hooks.len()];
    for (i, (addr, mode)) in hooks.iter().enumerate() {
        h[i * 20..i * 20 + 4].copy_from_slice(&addr.to_le_bytes());
        h[i * 20 + 4..i * 20 + 8].copy_from_slice(&mode.to_le_bytes());
        h[i * 20 + 16..i * 20 + 20].copy_from_slice(&(i as u32).to_le_bytes());
    }
    let n = hooks.len();
    let elems = (0..n).map(|i| format!("$f{i}")).collect::<Vec<_>>().join(" ");
    let esc = |b: &[u8]| b.iter().map(|x| format!("\\{x:02x}")).collect::<String>();
    module(&format!(
        r#"{imports} (memory (export "memory") {mem}) (table (export "__indirect_function_table") {n} {n} funcref)
        {funcs} (elem (i32.const 0) {elems})
        (global (export "gd_mod_manifest") i32 (i32.const 1024)) (global (export "gdmod_abi_version") i32 (i32.const 512))
        (data (i32.const 512) "\00\00\01\00") (data (i32.const 1024) "{}") (data (i32.const 2048) "{}")"#,
        esc(&m), esc(&h)))
}

#[test]
fn accepts_valid() {
    Host::new().load(&mod_with(0x8006CC7C, POST, "1 1", "", ABI_VERSION)).unwrap();
}

#[test]
fn refusals() {
    let cases = [
        (mod_with(0x8006CC7C, POST, "1", "", ABI_VERSION), "memory"),
        (mod_with(0x8006CC7C, POST, "1 1", "", ABI_VERSION + 1), "abi_min"),
        (mod_with(0x12345678, POST, "1 1", "", ABI_VERSION), "not in hooks-v1"),
        (mod_with(0x8006CC7C, REPLACE, "1 1", "", ABI_VERSION), "replace"),
        (mod_with(0x8006CC7C, PRE, "1 1", "", ABI_VERSION), "signature"),
        (mod_with(0x80068E98, POST, "1 1", "", ABI_VERSION), "not wired"),
        (mod_with(0x8006CC7C, POST, "1 1", "(func $s) (start $s)", ABI_VERSION), "start"),
        (mod_full("t.ok", &[(0x800CB250, REPLACE)], 0, "1 1", "(func $f0 (param i32))", "", ABI_VERSION), "GD_FEATURE_REPLACE"),
        (mod_full("t.ok", &[(0x8026862C, REPLACE)], FEATURE_REPLACE, "1 1", "(func $f0 (param i32) (result i32) i32.const 0)", "", ABI_VERSION), "not allowed"),
    ];
    for (b, want) in cases {
        let e = Host::new().load(&b).unwrap_err();
        assert!(e.contains(want), "{want}: {e}");
    }
    let mut h = Host::new();
    h.load(&mod_with(0x8006CC7C, POST, "1 1", "", ABI_VERSION)).unwrap();
    assert!(h.load(&mod_with(0x8006CC7C, POST, "1 1", "", ABI_VERSION)).unwrap_err().contains("already loaded"));
}

#[test]
fn foreign_import_refused() {
    let b = module(r#"(import "env" "x" (func))"#);
    assert!(Host::new().load(&b).unwrap_err().contains("module must be gd"));
}

#[test]
fn table_rows() {
    assert_eq!(hook_row(0x8006CC7C).unwrap().0, "damage.fighter_take_damage");
    assert_eq!(hook_row(0x80390CFC).unwrap().0, "frame.run_procs");
}

/// The in-repo Rust example (tools/rust-port/modhost/build_mods.sh); skipped when it has not been built.
#[test]
fn hello_mod_loads() {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/mods/hello_mod.wasm");
    let Ok(b) = std::fs::read(p) else { eprintln!("skip: {p} not built"); return };
    let mut h = Host::new();
    h.load(&b).unwrap();
    assert_eq!(h.mods[0].id, "gd.hello");
    assert!(!h.mods[0].disabled);
}

/// Generic rows: pre skip, replace with gd_orig_call (once only), post order, handle-less args (no fighter in a unit
/// test, so the item / stage rows are used).
#[test]
fn replace_orig_call_and_chains() {
    // item.update0 void(gd_item): replace calls the original twice (the second is refused), post counts
    let a = mod_full("a.replace", &[(0x802693E4, REPLACE)], FEATURE_REPLACE, "1 1",
        "(func $f0 (param i32) call $oc call $oc)", r#"(import "gd" "gd_orig_call_void" (func $oc))"#, ABI_VERSION);
    let b = mod_full("b.post", &[(0x802693E4, POST), (0x801C0FB8, PRE)], 0, "1 1",
        "(func $f0 (param i32) (i32.store (i32.const 4096) (i32.add (i32.load (i32.const 4096)) (i32.const 1)))) (func $f1 (param i32) (result i32) i32.const 1)", "", ABI_VERSION);
    let mut h = Host::new();
    h.load(&a).unwrap();
    h.load(&b).unwrap();
    // a second replace on the same row is refused
    let c = mod_full("c.replace", &[(0x802693E4, REPLACE)], FEATURE_REPLACE, "1 1", "(func $f0 (param i32))", "", ABI_VERSION);
    assert!(h.load(&c).unwrap_err().contains("one replace per address"));
    install(Box::new(h));
    let mut calls = 0;
    melee_core::modhook::active(0x802693E4).expect("row hooked");
    dispatch_for_test(0x802693E4, &[Arg::Item(core::ptr::null_mut())], &mut || { calls += 1; Ret::Void });
    assert_eq!(calls, 1, "the replace's original runs once");
    // stage.on_start: the pre hook skips, the original never runs
    let mut ran = false;
    dispatch_for_test(0x801C0FB8, &[Arg::Ptr(core::ptr::null_mut())], &mut || { ran = true; Ret::Void });
    assert!(!ran);
    let host = host().unwrap();
    assert!(host.mods.iter().all(|m| !m.disabled));
    assert!(melee_core::modhook::active(0x800CB250).is_none(), "an unhooked row stays off");
}

#[test]
fn sig_parse() {
    let s = parse_sig("void(gd_fighter,i32,i32,f32,f32,f32)").unwrap();
    assert_eq!(s.args.len(), 6);
    assert!(s.ret.is_none());
    assert_eq!(parse_sig("gd_gobj(gd_spawn)").unwrap().ret, Some(Ty::Handle));
}

/// Every built example (build/mods/*.wasm) loads together; skipped when none has been built.
#[test]
fn examples_load_together() {
    let d = concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/mods");
    let Ok(rd) = std::fs::read_dir(d) else { eprintln!("skip: {d} not built"); return };
    let mut h = Host::new();
    let mut files: Vec<_> = rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "wasm")).collect();
    files.sort();
    let mut ui = crate::ui::PresHost::new();
    for f in &files {
        let b = std::fs::read(f).unwrap();
        // gd_ui presentation mods go to the presentation host (install_dir skips them the same way)
        if crate::ui::is_presentation(&b) {
            if crate::lab::imports_lab(&b) {
                ui.install_lab(); // as PresHost::from_env does for a Lab mod
            }
            ui.load(&b, std::env::temp_dir()).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
            continue;
        }
        h.load(&b).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
    }
    let ids: Vec<&str> = h.mods.iter().map(|m| m.id.as_str()).collect();
    for want in ["gd.hello", "gd.moves", "gd.world"] {
        assert!(ids.contains(&want), "{want} missing from {ids:?}");
    }
}
