#![allow(non_snake_case)]
use melee_runtime::tu::dballoc::*;
use melee_runtime::tu::dbeffect::pad;

#[derive(Default)]
struct H { level: i32, down: u32, pressed: u32, log: Vec<String> }
impl DbAllocHost for H {
    fn DbLevel(&mut self) -> i32 { self.level }
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn HSD_ObjAllocGetPeak(&mut self, _: AllocPool) -> u32 { 7 }
    fn HSD_ObjAllocSetNumLimit(&mut self, p: AllocPool, n: u32) { self.log.push(format!("set {p:?} {n}")) }
    fn HSD_ObjAllocEnableNumLimit(&mut self, p: AllocPool) { self.log.push(format!("en {p:?}")) }
    fn HSD_ObjAllocDisableNumLimit(&mut self, p: AllocPool) { self.log.push(format!("dis {p:?}")) }
}

#[test]
fn limiter_toggles_only_in_develop() {
    let mut s = DbAlloc::default();
    let mut h = H { level: 0, down: pad::A | pad::B, pressed: pad::DPADUP, ..Default::default() };
    s.fn_update_obj_alloc_limiter(&mut h, 0);
    assert!(h.log.is_empty());
    h.level = DB_LKIND_DEVELOP;
    s.fn_update_obj_alloc_limiter(&mut h, 0);
    assert_eq!((s.b0, s.b1), (true, true));
    assert_eq!(h.log.len(), 8);
    assert_eq!(h.log[0], "set EfLib 7");
    h.log.clear();
    s.fn_update_obj_alloc_limiter(&mut h, 0);
    assert_eq!((s.b0, s.b1), (false, false));
    assert_eq!(h.log, ["dis EfLib", "dis Hsd804D0F90", "dis Hsd804D0F60", "dis PsAppSrt"]);
    s.b0 = true;
    s.fn_setup_obj_alloc_limiter();
    assert_eq!(s, DbAlloc::default());
}
