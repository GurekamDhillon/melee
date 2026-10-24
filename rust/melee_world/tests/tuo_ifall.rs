//! ifall.c against the C, by the TU state oracle (oracle/tuo, build.sh ifall).
mod tuo_common;
#[path = "tuo_gen/ifall_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_ifall.cfg"), include_str!("data/tuo_ifall.txt"), gen::glue());
}
