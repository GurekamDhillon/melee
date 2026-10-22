//! gmmovieend.c against the C, by the TU state oracle (oracle/tuo, build.sh gmmovieend).
mod tuo_common;
#[path = "tuo_gen/gmmovieend_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmmovieend.cfg"), include_str!("data/tuo_gmmovieend.txt"), gen::glue());
}
