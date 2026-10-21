#![allow(non_snake_case)]
use melee_runtime::tu::dbeffect::pad;
use melee_runtime::tu::dbscreenshot::{fn_802289F8, DbScreenshot, DbScreenshotHost};

#[derive(Default)]
struct H { down: u32, pressed: u32, btn: [u32; 4], trig: [u32; 4], log: Vec<String> }
impl DbScreenshotHost for H {
    fn db_ButtonsDown(&mut self, _: i32) -> u32 { self.down }
    fn db_ButtonsPressed(&mut self, _: i32) -> u32 { self.pressed }
    fn gm_SetGameSpeed(&mut self, s: f32) { self.log.push(format!("speed {s}")) }
    fn gm_ResetGameSpeed(&mut self) { self.log.push("reset".into()) }
    fn HSD_PadMasterStatus_button(&mut self, i: usize) -> u32 { self.btn[i] }
    fn HSD_PadMasterStatus_trigger(&mut self, i: usize) -> u32 { self.trig[i] }
    fn HSD_VIWaitXFBFlush(&mut self) {}
    fn HSD_VIGetXFBLastDrawDone(&mut self) -> i32 { 1 }
    fn HSD_VIData_xfb_buffer(&mut self, i: i32) -> i32 { 0x1000 * i }
    fn HSD_VIData_fb_width(&mut self) -> i32 { 640 }
    fn HSD_VIData_xfb_height(&mut self) -> i32 { 480 }
    fn HSD_ASSERTREPORT(&mut self, _: i32, _: &str) { panic!("no xfb") }
    fn hsd_80393A5C(&mut self, p: &str, b: i32, l: i32) -> i32 { self.log.push(format!("{p} {b:x} {l}")); 0 }
}

#[test]
fn toggle_5x_and_screenshot() {
    let mut s = DbScreenshot::default();
    let mut h = H::default();
    s.fn_Setup5xSpeed();
    h.down = pad::A;
    h.pressed = pad::DPADRIGHT;
    s.fn_Check5xSpeed(&mut h, 0);
    s.fn_Check5xSpeed(&mut h, 0);
    assert_eq!(h.log, ["speed 5", "reset"]);
    h.log.clear();
    s.db_InitScreenshot();
    s.db_TakeScreenshotIfPending(&mut h);
    assert!(h.log.is_empty());
    h.btn[2] = pad::Y;
    h.trig[2] = pad::DPADUP;
    s.db_CheckScreenshot(&mut h);
    assert_eq!(s.pending, 1);
    s.db_TakeScreenshotIfPending(&mut h);
    s.pending = 1;
    s.db_TakeScreenshotIfPending(&mut h);
    assert_eq!(h.log, ["shot/screenshot00.frb 1000 614400", "shot/screenshot01.frb 1000 614400"]);
    assert_eq!(fn_802289F8(&mut h, "DVD:x", 0, 0), -1);
}
