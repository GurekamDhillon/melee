//! grzebes.c against the C, by the TU state oracle (oracle/tuo, build.sh grzebes).
mod tuo_common;
#[path = "tuo_gen/grzebes_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_grzebes.cfg"), include_str!("data/tuo_grzebes.txt"), gen::glue());
}
