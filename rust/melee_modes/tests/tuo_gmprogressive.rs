//! gmprogressive.c against the C, by the TU state oracle (oracle/tuo, build.sh gmprogressive).
mod tuo_common;
#[path = "tuo_gen/gmprogressive_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmprogressive.cfg"), include_str!("data/tuo_gmprogressive.txt"), gen::glue());
}
