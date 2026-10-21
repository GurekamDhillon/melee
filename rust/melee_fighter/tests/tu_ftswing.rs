//! ftswing.c / ftlipstickswing.c / ftstarrodswing.c. The pure parts are checked against the C
//! (tests/data/tu_ftswing_ref.c: the table and fn_800CCEC4 cut verbatim, assert -> -999,
//! built with clang -O2 -ffp-contract=on); the state setters get smoke tests.
use melee_fighter::tu::{ftlipstickswing as lip, ftstarrodswing as star, ftswing as sw};
use melee_fighter::types::*;

#[test]
fn swing_tables_match_c() {
    let exp = include_str!("data/tu_ftswing.txt");
    let mut n = 0;
    for line in exp.lines() {
        let w: std::vec::Vec<i32> = line.split(' ').skip(1).map(|s| s.parse().unwrap()).collect();
        if line.starts_with("k ") {
            let got = std::panic::catch_unwind(|| sw::fn_800CCEC4(w[0])).unwrap_or(-999);
            assert_eq!(got, w[1], "{line}");
        } else {
            assert_eq!(sw::get_anim_id_kind(Ft_Kind_Mario as i32, w[0], w[1]), w[2], "{line}");
        }
        n += 1;
    }
    assert_eq!(n, 66);
}

fn with_fighter(f: impl FnOnce(*mut HSD_GObj, &mut Fighter)) {
    let mut x8 = ftData_x8::zeroed();
    let mut fd = ftData::zeroed();
    fd.x8 = &mut x8;
    let mut fp = Fighter::zeroed();
    fp.ft_data = &mut fd;
    let mut g = HSD_GObj::zeroed();
    g.user_data = &mut fp as *mut Fighter as *mut _;
    let gp: *mut HSD_GObj = &mut g;
    f(gp, unsafe { &mut *((*gp).user_data as *mut Fighter) });
}

#[test]
fn accessory_cb_without_item_is_noop() {
    with_fighter(|g, fp| {
        fp.cmd_vars = [7, 8, 0, 0];
        lip::ft_800CDB9C(g);
        star::ft_800CD914(g);
        assert_eq!(fp.cmd_vars, [7, 8, 0, 0]);
    });
}

#[test]
fn check_throw_b0_clears() {
    with_fighter(|_, fp| {
        unsafe { fp.anon_2210.anon_0.set_throw_flags_b0(1) };
        assert!(lip::ft_check_throw_b0(fp));
        assert!(!lip::ft_check_throw_b0(fp));
    });
}
