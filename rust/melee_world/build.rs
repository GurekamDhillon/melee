//! The whole-TU oracle tests (tests/tu_*.rs) run game code that stores pointers in 32-bit ints
//! (`(int) ptr` in the gr stage TUs, `(s32) fnptr` in gm): like the C oracle (gen/cref/run_tu.py,
//! linked `/BASE:0x10000000 /LARGEADDRESSAWARE:NO`) they must see every pointer below 2 GiB.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        for a in ["/BASE:0x10000000", "/LARGEADDRESSAWARE:NO", "/DYNAMICBASE:NO", "/HIGHENTROPYVA:NO"] {
            println!("cargo:rustc-link-arg-tests={a}");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
