//! grzakogenerator.c against the C, by the TU state oracle (oracle/tuo, build.sh grzakogenerator).
mod tuo_common;
#[path = "tuo_gen/grzakogenerator_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_grzakogenerator.cfg"), include_str!("data/tuo_grzakogenerator.txt"), gen::glue());
}
