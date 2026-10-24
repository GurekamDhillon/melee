//! grvenom.c against the C, by the TU state oracle (oracle/tuo, build.sh grvenom).
mod tuo_common;
#[path = "tuo_gen/grvenom_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_grvenom.cfg"), include_str!("data/tuo_grvenom.txt"), gen::glue());
}
