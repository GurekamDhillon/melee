// The state-oracle tests of the stage TUs (tests/gro_*.rs) run with the image below 4 GiB: the game keeps pointers
// in `u32` fields (`gp->u.castle11.xD8 = (u32) gobj`), which the ports reproduce with `ptr_u32` / `u32_ptr`
// (src/tu/grinl.rs). That is only lossless while every pointer is < 2^32, so the test executables are linked
// not-large-address-aware and based low.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        for a in ["/LARGEADDRESSAWARE:NO", "/DYNAMICBASE:NO", "/BASE:0x10000000", "/HIGHENTROPYVA:NO"] {
            println!("cargo:rustc-link-arg-tests={a}");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
