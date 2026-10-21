//! Port of src/melee/db/dbscreenshot.c (debug 5x game speed toggle, and Y + d-up XFB
//! screenshots to the USB debug channel).
//!
//! The C globals live in [`DbScreenshot`]; calls out of the TU go through [`DbScreenshotHost`],
//! whose default implementation [`ext::Ext`] is the unported-callee stub set (panics, named like
//! the C).
#![allow(non_snake_case)]

use super::dbeffect::pad;

/// Callees of dbscreenshot.c.
pub trait DbScreenshotHost {
    fn db_ButtonsDown(&mut self, player: i32) -> u32;
    fn db_ButtonsPressed(&mut self, player: i32) -> u32;
    fn gm_SetGameSpeed(&mut self, speed: f32);
    fn gm_ResetGameSpeed(&mut self);
    /// `HSD_PadMasterStatus[i].button` / `.trigger`.
    fn HSD_PadMasterStatus_button(&mut self, i: usize) -> u32;
    fn HSD_PadMasterStatus_trigger(&mut self, i: usize) -> u32;
    fn HSD_VIWaitXFBFlush(&mut self);
    fn HSD_VIGetXFBLastDrawDone(&mut self) -> i32;
    /// `HSD_VIData.xfb[i].buffer` (as the C's `int`).
    fn HSD_VIData_xfb_buffer(&mut self, i: i32) -> i32;
    /// `HSD_VIData.current.vi.rmode.fbWidth * xfbHeight`.
    fn HSD_VIData_fb_width(&mut self) -> i32;
    fn HSD_VIData_xfb_height(&mut self) -> i32;
    /// `HSD_ASSERTREPORT(line, 0, msg)`.
    fn HSD_ASSERTREPORT(&mut self, line: i32, msg: &str);
    fn hsd_80393A5C(&mut self, path: &str, buf: i32, len: i32) -> i32;
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl DbScreenshotHost for Ext {
        fn db_ButtonsDown(&mut self, _: i32) -> u32 { todo!("db_ButtonsDown (db/dbinit.c)") }
        fn db_ButtonsPressed(&mut self, _: i32) -> u32 { todo!("db_ButtonsPressed (db/dbinit.c)") }
        fn gm_SetGameSpeed(&mut self, _: f32) { todo!("gm_SetGameSpeed (gm)") }
        fn gm_ResetGameSpeed(&mut self) { todo!("gm_ResetGameSpeed (gm)") }
        fn HSD_PadMasterStatus_button(&mut self, _: usize) -> u32 { todo!("HSD_PadMasterStatus (baselib/controller.c)") }
        fn HSD_PadMasterStatus_trigger(&mut self, _: usize) -> u32 { todo!("HSD_PadMasterStatus (baselib/controller.c)") }
        fn HSD_VIWaitXFBFlush(&mut self) { todo!("HSD_VIWaitXFBFlush (baselib/video.c)") }
        fn HSD_VIGetXFBLastDrawDone(&mut self) -> i32 { todo!("HSD_VIGetXFBLastDrawDone (baselib/video.c)") }
        fn HSD_VIData_xfb_buffer(&mut self, _: i32) -> i32 { todo!("HSD_VIData (baselib/video.c)") }
        fn HSD_VIData_fb_width(&mut self) -> i32 { todo!("HSD_VIData (baselib/video.c)") }
        fn HSD_VIData_xfb_height(&mut self) -> i32 { todo!("HSD_VIData (baselib/video.c)") }
        fn HSD_ASSERTREPORT(&mut self, _: i32, _: &str) { todo!("HSD_ASSERTREPORT (baselib/debug.c)") }
        fn hsd_80393A5C(&mut self, _: &str, _: i32, _: i32) -> i32 { todo!("hsd_80393A5C (baselib/hsd_3933.c)") }
    }
}

/// The globals of dbscreenshot.c.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DbScreenshot {
    /// `db_ScreenshotNumber` (0x804D6B94).
    pub number: i32,
    /// `db_ScreenshotPending` (0x804D6B90).
    pub pending: u32,
    /// `db_5xSpeedStatus.b0` (0x804D6B8C).
    pub speed5x: bool,
    /// `db_804D6B88`.
    pub db_804D6B88: i32,
}

/// `fn_802289F8`: only the "USB:" device is written.
pub fn fn_802289F8<H: DbScreenshotHost + ?Sized>(h: &mut H, path: &str, buf: i32, len: i32) -> i32 {
    match path.strip_prefix("USB:") {
        Some(rest) => h.hsd_80393A5C(rest, buf, len),
        None => -1,
    }
}

impl DbScreenshot {
    /// `fn_Setup5xSpeed` (0x802287C4).
    pub fn fn_Setup5xSpeed(&mut self) {
        self.speed5x = false;
    }

    /// `fn_Check5xSpeed` (0x802287D8).
    pub fn fn_Check5xSpeed<H: DbScreenshotHost + ?Sized>(&mut self, h: &mut H, player: i32) {
        if h.db_ButtonsDown(player) & pad::A != 0 && h.db_ButtonsPressed(player) & pad::DPADRIGHT != 0 {
            self.fn_Toggle5xSpeed(h);
        }
    }

    /// `fn_Toggle5xSpeed` (0x80228820).
    pub fn fn_Toggle5xSpeed<H: DbScreenshotHost + ?Sized>(&mut self, h: &mut H) {
        self.speed5x = !self.speed5x;
        if self.speed5x {
            h.gm_SetGameSpeed(5.0);
        } else {
            h.gm_ResetGameSpeed();
        }
    }

    /// `db_InitScreenshot` (0x8022886C).
    pub fn db_InitScreenshot(&mut self) {
        self.number = 0;
        self.pending = 0;
    }

    /// `db_CheckScreenshot` (0x8022887C).
    pub fn db_CheckScreenshot<H: DbScreenshotHost + ?Sized>(&mut self, h: &mut H) {
        for i in 0..4 {
            if h.HSD_PadMasterStatus_button(i) & pad::Y != 0 && h.HSD_PadMasterStatus_trigger(i) & pad::DPADUP != 0 {
                self.pending = 1;
            }
        }
    }

    /// `db_TakeScreenshotIfPending` (0x8022892C).
    pub fn db_TakeScreenshotIfPending<H: DbScreenshotHost + ?Sized>(&mut self, h: &mut H) {
        if self.pending != 0 {
            h.HSD_VIWaitXFBFlush();
            let idx = h.HSD_VIGetXFBLastDrawDone();
            // gwtool zero-initialises locals, so a missing XFB writes from address 0.
            let mut buf = 0;
            if idx != -1 {
                buf = h.HSD_VIData_xfb_buffer(idx);
            } else {
                h.HSD_ASSERTREPORT(61, "cant find xfb!\n");
            }
            let n = self.number;
            self.number = n.wrapping_add(1);
            let path = format!("USB:shot/screenshot{:02}.frb", n);
            let len = h.HSD_VIData_fb_width().wrapping_mul(h.HSD_VIData_xfb_height()).wrapping_mul(2);
            fn_802289F8(h, &path, buf, len);
            self.pending = 0;
        }
    }
}
