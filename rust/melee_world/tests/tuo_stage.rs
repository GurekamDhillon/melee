//! stage.c against the C, by the TU state oracle (oracle/tuo, build.sh stage).
mod tuo_common;
#[path = "tuo_gen/stage_gen.rs"]
mod gen;

#[test]
fn state_oracle() {
    tuo_common::run(include_str!("data/tuo_stage.cfg"), include_str!("data/tuo_stage.txt"), gen::glue());
}
