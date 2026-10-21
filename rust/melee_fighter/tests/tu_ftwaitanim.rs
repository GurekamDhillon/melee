//! ftwaitanim.c: getAnimID checked against the C (tests/data/tu_ftwaitanim_ref.c, cut verbatim,
//! HSD_Randi -> the recorded roll, assert -> -999), 500 generated tables; smoke for the rest.
use melee_fighter::tu::ftwaitanim as w;
use melee_fighter::types::*;

#[test]
fn get_anim_id_matches_c() {
    let mut n = 0;
    for line in include_str!("data/tu_ftwaitanim.txt").lines() {
        let (l, r) = line.split_once(" | ").unwrap();
        let a: std::vec::Vec<i32> = l.split(' ').map(|s| s.parse().unwrap()).collect();
        let b: std::vec::Vec<i32> = r.split(' ').map(|s| s.parse().unwrap()).collect();
        let mut t: std::vec::Vec<WaitStruct> = (0..=a[0] as usize).map(|_| WaitStruct::zeroed()).collect();
        for k in 0..a[0] as usize {
            t[k].u.i = WaitStruct_u_i { x: a[1 + 2 * k], y: a[2 + 2 * k] };
        }
        t[a[0] as usize].u.i = WaitStruct_u_i { x: -1, y: 0 };
        let p = t.as_ptr() as usize;
        let got = std::panic::catch_unwind(|| w::get_anim_id_with(p as *const WaitStruct, b[0])).unwrap_or(-999);
        assert_eq!(got, b[1], "{line}");
        n += 1;
    }
    assert_eq!(n, 500);
}

#[test]
fn no_item_is_not_holding() {
    let fp = Fighter::zeroed();
    assert!(!w::ftCo_8008A698(&fp));
    let mut g = HSD_GObj::zeroed();
    let mut fp = Fighter::zeroed();
    g.user_data = &mut fp as *mut Fighter as *mut _;
    w::ftCo_8008A6D8(&mut g, -1); // -1: nothing happens
}
