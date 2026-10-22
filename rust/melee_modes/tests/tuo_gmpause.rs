//! gmpause.c against the C, by the TU state oracle (oracle/tuo, build.sh gmpause).
mod tuo_common;
#[path = "tuo_gen/gmpause_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmpause.cfg"), include_str!("data/tuo_gmpause.txt"), gen::glue());
}
