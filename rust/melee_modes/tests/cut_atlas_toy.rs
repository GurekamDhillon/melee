//! tu::gmfrontend_atlas_toy vs the C text itself (gen/cref/cut/atlas_toy.c; regenerate with
//! `python rust/melee_modes/gen/cref/cutref.py atlas_toy`).
use melee_modes::tu::gmfrontend_atlas_toy::Fad_ToyWindowKnown;

#[test]
fn fad_toy_window_known_matches_c() {
    let exp = include_str!("data/cut_atlas_toy.txt");
    let mut n = 0;
    for (k, line) in exp.lines().enumerate() {
        assert_eq!(line, format!("{} {}", k, Fad_ToyWindowKnown(k as u8)), "kind {k}");
        n += 1;
    }
    assert_eq!(n, 256);
}
