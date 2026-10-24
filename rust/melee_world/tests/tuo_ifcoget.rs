//! ifcoget.c against the C, by the TU state oracle (oracle/tuo, build.sh ifcoget).
mod tuo_common;
#[path = "tuo_gen/ifcoget_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_ifcoget.cfg"), include_str!("data/tuo_ifcoget.txt"), gen::glue());
}
