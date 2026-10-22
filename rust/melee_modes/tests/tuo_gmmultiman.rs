//! gmmultiman.c against the C, by the TU state oracle (oracle/tuo, build.sh gmmultiman).
mod tuo_common;
#[path = "tuo_gen/gmmultiman_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmmultiman.cfg"), include_str!("data/tuo_gmmultiman.txt"), gen::glue());
}
