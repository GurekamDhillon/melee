//! Loader refusals (spec V6) on hand-written modules; the accepting path is covered by mods/hello_mod.
use super::*;

fn module(body: &str) -> Vec<u8> {
    wat::parse_str(format!("(module {body})")).unwrap()
}

/// A valid manifest at 1024 with `hooks` at 2048 (one decl: addr, mode, fn 0), memory pages min/max.
fn mod_with(addr: u32, mode: u32, mem: &str, extra: &str, abi_min: u32) -> Vec<u8> {
    let mut m = vec![0u8; 124];
    let w = |m: &mut Vec<u8>, i: usize, v: u32| m[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
    for (i, v) in [MANIFEST_MAGIC, 124, abi_min, ABI_VERSION, 1, 0x10000, 0, 0, 4, 1, 2048].iter().enumerate() { w(&mut m, i, *v); }
    m[44..48].copy_from_slice(b"t.ok");
    let mut h = vec![0u8; 20];
    h[0..4].copy_from_slice(&addr.to_le_bytes());
    h[4..8].copy_from_slice(&mode.to_le_bytes());
    let esc = |b: &[u8]| b.iter().map(|x| format!("\\{x:02x}")).collect::<String>();
    module(&format!(
        r#"(memory (export "memory") {mem}) (table (export "__indirect_function_table") 1 1 funcref)
        (func $post (param i32 f32)) (elem (i32.const 0) $post)
        (global (export "gd_mod_manifest") i32 (i32.const 1024)) (global (export "gdmod_abi_version") i32 (i32.const 512))
        (data (i32.const 512) "\00\00\01\00") (data (i32.const 1024) "{}") (data (i32.const 2048) "{}") {extra}"#,
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
    h.run_frame(false);
    h.run_frame(true);
    h.damage(0, 5.0, true); // no fighter: the accessors read 0 and count the stale handle
    assert!(h.host_data().bad_handles > 0);
    assert!(!h.mods[0].disabled);
}
