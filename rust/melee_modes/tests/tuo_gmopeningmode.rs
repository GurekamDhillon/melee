//! gmopeningmode.c against the C, by the TU state oracle (oracle/tuo, build.sh gmopeningmode).
mod tuo_common;
#[path = "tuo_gen/gmopeningmode_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmopeningmode.cfg"), include_str!("data/tuo_gmopeningmode.txt"), gen::glue());
}
