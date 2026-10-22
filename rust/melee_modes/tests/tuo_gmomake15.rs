//! gmomake15.c against the C, by the TU state oracle (oracle/tuo, build.sh gmomake15).
mod tuo_common;
#[path = "tuo_gen/gmomake15_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmomake15.cfg"), include_str!("data/tuo_gmomake15.txt"), gen::glue());
}
