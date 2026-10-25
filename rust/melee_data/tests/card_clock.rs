//! `gc_time_now` is local wall-clock seconds since 2000-01-01 (Aurora's getGCTime): compare with the
//! standard library's idea of the same on this machine.
#[test]
fn gc_time_is_local_wall_clock() {
    let t = melee_data::card::gc_time_now() as i64;
    let unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let off = t + 946_684_800 - unix; // local offset incl. DST
    eprintln!("gc_time_now={t} offset={off}");
    assert!(off % 900 == 0 && off.abs() <= 14 * 3600, "offset {off}");
}
