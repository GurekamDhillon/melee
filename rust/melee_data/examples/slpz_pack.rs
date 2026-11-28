//! `cargo run -p melee_data --example slpz_pack -- IN.slp OUT.slpz` compresses one replay (proof tooling).
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = std::fs::read(&a[1]).expect("read");
    let z = melee_data::slp::compress_slp(&d).expect("compress");
    assert_eq!(melee_data::slp::to_slp(z.clone()).expect("round trip"), d, "round trip differs");
    std::fs::write(&a[2], z).expect("write");
}
