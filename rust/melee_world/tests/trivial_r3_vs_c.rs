//! Night-3 round-3 gr callbacks (gr::trivial) against the C (cref/gen_trivial_r3_ref.sh ->
//! tests/data/trivial_r3_ref.txt): every one of the 568, called with zero and junk arguments.
use melee_world::gr::trivial::{call_by_name, COUNT};

const DATA: &str = include_str!("data/trivial_r3_ref.txt");

#[test]
fn trivial_r3_match_c() {
    let mut seen = std::collections::BTreeSet::new();
    let mut rows = 0;
    for line in DATA.lines() {
        let v: Vec<&str> = line.split_whitespace().collect();
        let (tu, name, junk, ret) = (v[0], v[1], v[2] == "1", v[3]);
        let got = call_by_name(tu, name, junk).unwrap_or_else(|| panic!("not ported: {tu}::{name}"));
        let want = if ret == "-" { None } else { Some(u64::from_str_radix(ret, 16).unwrap()) };
        assert_eq!(got, want, "{line}");
        seen.insert((tu, name));
        rows += 1;
    }
    assert_eq!(seen.len(), COUNT);
    assert_eq!(rows, 2 * COUNT);
}
