//! grzebesroute.c against the C, by the TU state oracle (oracle/tuo, build.sh grzebesroute).
mod tuo_common;
#[path = "tuo_gen/grzebesroute_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_grzebesroute.cfg"), include_str!("data/tuo_grzebesroute.txt"), gen::glue());
}
