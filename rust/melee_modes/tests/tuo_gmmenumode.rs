//! gmmenumode.c against the C, by the TU state oracle (oracle/tuo, build.sh gmmenumode).
mod tuo_common;
#[path = "tuo_gen/gmmenumode_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmmenumode.cfg"), include_str!("data/tuo_gmmenumode.txt"), gen::glue());
}
