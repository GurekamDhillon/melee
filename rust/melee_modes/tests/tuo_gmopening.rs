//! gmopening.c against the C, by the TU state oracle (oracle/tuo, build.sh gmopening).
mod tuo_common;
#[path = "tuo_gen/gmopening_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gmopening.cfg"), include_str!("data/tuo_gmopening.txt"), gen::glue());
}
