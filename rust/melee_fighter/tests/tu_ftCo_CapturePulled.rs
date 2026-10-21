//! tu::ftCo_CapturePulled vs the C (gen/cref/tu_shard121_ref.c, shipping contraction):
//! ftCo_800DA824, fn_800DAA40, fn_800DAD18, ftCo_CapturePulledHi_Phys, ftCo_800DB464 bit-exact.
mod shard121;

#[test]
fn capture_pulled_matches_c() {
    let n = shard121::run(|op| op <= 4);
    assert!(n > 4000, "{n}");
}
