#![allow(non_snake_case)]
use melee_world::tu::cmsnap::*;

#[derive(Default)]
struct H { log: Vec<String> }
impl CmSnapHost for H {
    fn HSD_ASSERT(&mut self, line: i32, c: bool) { assert!(c, "assert line {line:#x}") }
    fn lb_800122C8(&mut self, i: ImagePtr) { self.log.push(format!("capture {i:?}")) }
    fn lb_800121FC(&mut self, w: i32, h: i32, f: i32, id: i32) -> ImagePtr { self.log.push(format!("alloc {w} {h} {f} {id:#x}")); Some(42) }
    fn create_snap_gobj(&mut self) { self.log.push("gobj".into()) }
}

#[test]
fn snapshot_lifecycle() {
    let mut s = CmSnap::default();
    let mut h = H::default();
    s.cmSnap_800316B4(&mut h);
    assert_eq!(h.log, ["gobj", "alloc 640 480 4 0x7d6"]);
    assert_eq!(s.cmSnap_80031618(), None);
    s.cmSnap_80031640(&mut h, 0);
    assert_eq!(s.status, status::SLEEP);
    s.cmSnap_800315C8(&mut h);
    let mut got = vec![];
    for _ in 0..4 {
        got.push(s.cmSnap_80031618());
        s.cmSnap_80031640(&mut h, 0);
    }
    assert_eq!(got, [None, None, None, None]);
    assert_eq!(h.log[2], "capture Some(42)");
    assert_eq!(s.status, status::UNK5);
    assert_eq!(s.cmSnap_80031618(), Some(42));
    assert_eq!(s.status, status::SLEEP);
}

#[test]
#[should_panic(expected = "assert line 0x55")]
fn request_while_busy_asserts() {
    let mut s = CmSnap { status: status::UNK2, ..Default::default() };
    s.cmSnap_800315C8(&mut H::default());
}
