//! tu::ftCo_CaptureKirby vs the C (gen/cref/tu_shard121_ref.c, shipping contraction):
//! ftCo_800BD19C and ftCo_800BD39C (incl. lbVector_Len and the jobj scale/dirty) bit-exact.
mod shard121;

#[test]
fn capture_kirby_matches_c() {
    let n = shard121::run(|op| op >= 5);
    assert!(n > 1000, "{n}");
}
