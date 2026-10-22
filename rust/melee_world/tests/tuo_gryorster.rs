//! gryorster.c against the C, by the TU state oracle (oracle/tuo, build.sh gryorster).
mod tuo_common;
#[path = "tuo_gen/gryorster_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_gryorster.cfg"), include_str!("data/tuo_gryorster.txt"), gen::glue());
}
