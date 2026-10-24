//! Port of src/sysdolphin/baselib/hsd_393C.c: the debug text console. `OSReport` text is captured
//! into a ring of length-suffixed lines (`fn_80393C14`), and a cursor (`x14` column, `x18` row
//! from the newest line, `x1C` position, `x20` line length) reads it back a byte at a time.
//!
//! The one static, `hsd_804CF7E8`, is [`Hsd393C::state`] (a `ParticleConsoleState` from
//! melee_types, GC size 0x28). The ring buffer is caller-owned memory handed to
//! [`Hsd393C::hsd_80393DA0`]; this port keeps the C's raw pointer. Calls out of the TU go through
//! [`Hsd393CHost`] (default [`ext::Ext`]: TODO panics named like the C).
#![allow(non_snake_case)]

use melee_types::ParticleConsoleState;

/// Callees of hsd_393C.c.
pub trait Hsd393CHost {
    /// `HSD_SetReportCallback(fn_80393C14)` when `enable`, `HSD_SetReportCallback(NULL)` otherwise.
    /// The host must route reports to [`Hsd393C::fn_80393C14`] while enabled.
    fn HSD_SetReportCallback(&mut self, enable: bool);
    fn HSD_VIWaitXFBFlushNoYield(&mut self);
    fn HSD_VIGetXFBLastDrawDone(&mut self) -> i32;
    /// `HSD_VIData.xfb[i].buffer` (VIData base + 0x58 + i * 0x60).
    fn HSD_VIData_xfb_buffer(&mut self, i: i32) -> u32;
    /// `HSD_VIData.nb_xfb`.
    fn HSD_VIData_nb_xfb(&mut self) -> i32;
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl Hsd393CHost for Ext {
        fn HSD_SetReportCallback(&mut self, _: bool) { todo!("HSD_SetReportCallback (baselib/debug.c)") }
        fn HSD_VIWaitXFBFlushNoYield(&mut self) { todo!("HSD_VIWaitXFBFlushNoYield (baselib/video.c)") }
        fn HSD_VIGetXFBLastDrawDone(&mut self) -> i32 { todo!("HSD_VIGetXFBLastDrawDone (baselib/video.c)") }
        fn HSD_VIData_xfb_buffer(&mut self, _: i32) -> u32 { todo!("HSD_VIData (baselib/video.c)") }
        fn HSD_VIData_nb_xfb(&mut self) -> i32 { todo!("HSD_VIData (baselib/video.c)") }
    }
}

/// The globals of hsd_393C.c.
pub struct Hsd393C {
    /// `hsd_804CF7E8` (0x804CF7E8).
    pub state: ParticleConsoleState,
}

impl Default for Hsd393C {
    fn default() -> Self {
        Hsd393C { state: ParticleConsoleState::zeroed() }
    }
}

impl Hsd393C {
    /// `fn_80393C14` (0x80393C14): append report text to the line ring.
    ///
    /// # Safety
    /// `state.out_buf` must point to `state.buf_size` writable bytes (set by `hsd_80393DA0`).
    pub unsafe fn fn_80393C14(&mut self, buf: &[u8]) {
        let st = &mut self.state;
        let out_size = st.buf_size;
        let mut write_pos = st.xC;
        let out_buf = st.out_buf;
        let mut line_len = st.x11;
        let tmp = out_size.wrapping_sub(1);

        for &c in buf {
            match c {
                b'\r' => {}
                b'\n' => {
                    let prev = *out_buf.add((write_pos as u32).wrapping_add(tmp) as usize % out_size as usize);
                    if line_len != 0 || prev != 0 {
                        *out_buf.add(write_pos as usize) = line_len;
                        write_pos = ((write_pos as u32).wrapping_add(1) % out_size) as i32;
                        st.x1C += 1;
                        st.x18 += 1;
                        line_len = 0;
                    }
                }
                _ => {
                    *out_buf.add(write_pos as usize) = c;
                    line_len = line_len.wrapping_add(1);
                    if line_len == 0x36 {
                        write_pos = ((write_pos as u32).wrapping_add(1) % out_size) as i32;
                        *out_buf.add(write_pos as usize) = line_len;
                        st.x1C += 1;
                        st.x18 += 1;
                        line_len = 0;
                    }
                    write_pos = ((write_pos as u32).wrapping_add(1) % out_size) as i32;
                    st.x1C += 1;
                }
            }
        }
        st.xC = write_pos;
        st.x11 = line_len;
    }

    /// `hsd_80393D2C` (0x80393D2C): enable/disable capture; returns the old enabled bit.
    pub fn hsd_80393D2C<H: Hsd393CHost + ?Sized>(&mut self, h: &mut H, enable: i32) -> i32 {
        let old = self.state.x0_b1() as i32;
        if enable != 0 {
            self.state.set_x0_b1(1);
            h.HSD_SetReportCallback(true);
        } else {
            self.state.set_x0_b1(0);
            h.HSD_SetReportCallback(false);
        }
        old
    }

    /// `hsd_80393DA0` (0x80393DA0): reset the console onto `buf` (`size` bytes) and enable capture.
    ///
    /// # Safety
    /// `buf` must point to `size` writable bytes that outlive the console.
    pub unsafe fn hsd_80393DA0<H: Hsd393CHost + ?Sized>(&mut self, h: &mut H, buf: *mut u8, size: usize) {
        // memset(&hsd_804CF7E8, 0, sizeof - sizeof(x24)): everything but the trailing x24.
        let x24 = self.state.x24;
        self.state = ParticleConsoleState::zeroed();
        self.state.x24 = x24;
        self.state.out_buf = buf;
        self.state.buf_size = size as u32;
        core::ptr::write_bytes(buf, 0, size);
        self.state.set_x0_b0(1);
        self.state.set_x0_b1(1);
        h.HSD_SetReportCallback(true);
    }

    /// `hsd_80393E34` (0x80393E34): current column / row.
    pub fn hsd_80393E34(&self, col_out: Option<&mut i32>, row_out: Option<&mut i32>) {
        if let Some(c) = col_out {
            *c = self.state.x14;
        }
        if let Some(r) = row_out {
            *r = self.state.x18;
        }
    }

    /// `hsd_80393E68` (0x80393E68): seek the cursor to line `row` from the newest, column `col`.
    ///
    /// # Safety
    /// The ring buffer pointer must be valid (see `hsd_80393DA0`).
    pub unsafe fn hsd_80393E68(&mut self, col: u32, row: u32) {
        let sp = &mut self.state;
        if sp.x0_b0() == 0 {
            return;
        }
        let mut byte_val: u32 = sp.x11 as u32;
        let mut counter: u32 = 0;
        let buf_size = sp.buf_size;
        let out_buf = sp.out_buf;
        let sum = (sp.xC as u32).wrapping_add(buf_size);
        let mut pos = byte_val;

        while counter < row && pos < buf_size {
            pos = pos.wrapping_add(1);
            byte_val = *out_buf.add((sum.wrapping_sub(pos) % buf_size) as usize) as u32;
            counter += 1;
            pos = pos.wrapping_add(byte_val);
        }

        sp.x18 = counter as i32;
        sp.x1C = pos as i32;
        sp.x20 = byte_val as i32;

        sp.x14 = if col < byte_val { col as i32 } else { byte_val as i32 };
    }

    /// `hsd_80393EF4` (0x80393EF4): move the cursor by deltas.
    ///
    /// # Safety
    /// The ring buffer pointer must be valid (see `hsd_80393DA0`).
    pub unsafe fn hsd_80393EF4(&mut self, col_delta: i32, row_delta: i32) {
        if self.state.x0_b0() == 0 {
            return;
        }
        let buf_size = self.state.buf_size;

        if (self.state.x1C as u32) >= buf_size {
            let col = self.state.x14.wrapping_add(col_delta) as u32;
            let row = self.state.x18.wrapping_add(row_delta) as u32;
            self.hsd_80393E68(col, row);
            return;
        }

        let out_buf = self.state.out_buf;
        let mut pos = self.state.x1C as u32;
        let mut byte_val = self.state.x20 as u32;
        let sum = (self.state.xC as u32).wrapping_add(buf_size);
        let mut col_delta = col_delta;
        let mut row_delta = row_delta;

        if row_delta > 0 {
            let mut counter: u32 = 0;
            while counter < row_delta as u32 && pos < buf_size {
                pos = pos.wrapping_add(1);
                byte_val = *out_buf.add((sum.wrapping_sub(pos) % buf_size) as usize) as u32;
                counter += 1;
                pos = pos.wrapping_add(byte_val);
            }
            self.state.x18 = self.state.x18.wrapping_add(counter as i32);
            self.state.x1C = pos as i32;

            if col_delta >= 0 {
                self.state.x14 = self.state.x14.wrapping_add(col_delta);
                if (self.state.x14 as u32) > byte_val {
                    self.state.x14 = byte_val as i32;
                }
            } else {
                let col = self.state.x14 as u32;
                col_delta = col_delta.wrapping_neg();
                if col > col_delta as u32 {
                    self.state.x14 = col.wrapping_sub(col_delta as u32) as i32;
                } else {
                    self.state.x14 = 0;
                }
            }
        } else if row_delta < 0 {
            if col_delta < 0 && (self.state.x14 as u32) < (col_delta.wrapping_neg() as u32) {
                col_delta = 0;
            } else {
                col_delta = self.state.x14.wrapping_add(col_delta);
            }

            if row_delta < 0 && (self.state.x18 as u32) < (row_delta.wrapping_neg() as u32) {
                row_delta = 0;
            } else {
                row_delta = self.state.x18.wrapping_add(row_delta);
            }

            self.hsd_80393E68(col_delta as u32, row_delta as u32);
        }
    }

    /// The shared read-one-byte tail of `hsd_80394068` / `hsd_80394128` (identical code in both
    /// after the early-outs).
    unsafe fn read_byte(&mut self) -> u8 {
        let st = &self.state;
        let idx = (st.x14 as u32).wrapping_add((st.xC as u32).wrapping_add(st.buf_size).wrapping_sub(st.x1C as u32));
        if (st.x14 as u32) < (st.x20 as u32) {
            let result = *st.out_buf.add((idx % st.buf_size) as usize);
            self.state.x14 += 1;
            result
        } else {
            self.hsd_80393EF4(0, -1);
            self.state.x14 = 0;
            0
        }
    }

    /// `hsd_80394068` (0x80394068): next byte at the cursor (0 at end of line, advancing a line).
    ///
    /// # Safety
    /// The ring buffer pointer must be valid (see `hsd_80393DA0`).
    pub unsafe fn hsd_80394068(&mut self) -> u8 {
        if self.state.x0_b0() == 0 {
            return 0;
        }
        if (self.state.x1C as u32) > self.state.buf_size {
            return 0;
        }
        self.read_byte()
    }

    /// `hsd_80394128` (0x80394128): seek to (`col`, `row`) then read a byte like `hsd_80394068`.
    ///
    /// # Safety
    /// The ring buffer pointer must be valid (see `hsd_80393DA0`).
    pub unsafe fn hsd_80394128(&mut self, col: i32, row: i32) -> u8 {
        self.hsd_80393E68(col as u32, row as u32);
        if self.state.x0_b0() == 0 {
            return 0;
        }
        if (self.state.x1C as u32) > self.state.buf_size {
            return 0;
        }
        self.read_byte()
    }
}

/// `hsd_803941E8` (0x803941E8): find the (up to) two XFBs that are not the one last drawn.
/// `xfb_out[0..2]` receive their addresses, `xfb_cur` the last-drawn one (and is zeroed if its
/// value is moved into `xfb_out[0]`). Returns how many `xfb_out` slots were filled.
pub fn hsd_803941E8<H: Hsd393CHost + ?Sized>(h: &mut H, xfb_out: &mut [i32; 2], mut xfb_cur: Option<&mut u32>) -> i32 {
    h.HSD_VIWaitXFBFlushNoYield();
    let last_draw = h.HSD_VIGetXFBLastDrawDone();

    if last_draw != -1 {
        if let Some(cur) = xfb_cur.as_deref_mut() {
            *cur = h.HSD_VIData_xfb_buffer(last_draw);
        }
    }

    let nb_xfb = h.HSD_VIData_nb_xfb();
    let mut i = 0;
    while i < nb_xfb {
        if i != last_draw {
            let buf = h.HSD_VIData_xfb_buffer(i);
            xfb_out[0] = buf as i32;
            if buf != 0 {
                break;
            }
        }
        i += 1;
    }

    i += 1;
    while i < nb_xfb {
        if i != last_draw {
            let buf = h.HSD_VIData_xfb_buffer(i);
            xfb_out[1] = buf as i32;
            if buf != 0 {
                break;
            }
        }
        i += 1;
    }

    if xfb_out[0] as u32 == 0 {
        if let Some(cur) = xfb_cur {
            xfb_out[0] = *cur as i32;
            *cur = 0;
        } else {
            return 0;
        }
    }

    if xfb_out[1] as u32 != 0 {
        return 2;
    }
    1
}
