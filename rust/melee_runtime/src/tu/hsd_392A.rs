//! Port of src/sysdolphin/baselib/hsd_392A.c: performance overlay data (`fn_80392934` samples
//! `HSD_PerfLastStat` and tracks peaks, `fn_80392A08` configures, `fn_80392A3C` builds the
//! `PerfDispItem` chain hsd_3924 draws).
//!
//! All globals (`hsd_804CE3F8`, `hsd_804D7858..88`, `lbl_804D6088..A4`) live in [`Hsd392A`]; the
//! returned item chain points into that struct, so keep it in place while the chain is in use.
#![allow(non_snake_case)]

use core::ffi::c_void;
use melee_types::{HSD_PerfStat, PerfDispItem};

const LBL_804D6090: i32 = -1;
const LBL_804D6094: i32 = 0xFF0000FFu32 as i32;
const LBL_804D6098: i32 = 0x00FF00FF;
const LBL_804D609C: i32 = 0x00FFFFFF;
const LBL_804D60A0: i32 = 0x8080FF;
const LBL_804D60A4: i32 = 0xC0C000FFu32 as i32;

/// Globals of hsd_392A.c.
pub struct Hsd392A {
    /// `hsd_804CE3F8[6]`
    pub hsd_804CE3F8: [PerfDispItem; 6],
    pub hsd_804D7858: f32,
    pub hsd_804D785C: f32,
    pub hsd_804D7860: f32,
    pub hsd_804D7864: f32,
    pub hsd_804D7868: f32,
    pub hsd_804D786C: f32,
    pub hsd_804D7870: i32,
    pub hsd_804D7874: i32,
    pub hsd_804D7878: i32,
    pub hsd_804D787C: f32,
    pub hsd_804D7880: f32,
    pub hsd_804D7884: f32,
    pub hsd_804D7888: i32,
    /// `lbl_804D6088` (initially 4): frame-budget bar scale.
    pub lbl_804D6088: i32,
    /// `lbl_804D608C` (initially 1): text rows on/off.
    pub lbl_804D608C: i32,
}

impl Default for Hsd392A {
    fn default() -> Self {
        Hsd392A {
            hsd_804CE3F8: [PerfDispItem::zeroed(); 6],
            hsd_804D7858: 0.0,
            hsd_804D785C: 0.0,
            hsd_804D7860: 0.0,
            hsd_804D7864: 0.0,
            hsd_804D7868: 0.0,
            hsd_804D786C: 0.0,
            hsd_804D7870: 0,
            hsd_804D7874: 0,
            hsd_804D7878: 0,
            hsd_804D787C: 0.0,
            hsd_804D7880: 0.0,
            hsd_804D7884: 0.0,
            hsd_804D7888: 0,
            lbl_804D6088: 4,
            lbl_804D608C: 1,
        }
    }
}

/// C `printf("%2.3f")` of a promoted float.
fn fmt_f(v: f32) -> String {
    let d = v as f64;
    if d.is_nan() {
        return if d.is_sign_negative() { "-nan".into() } else { "nan".into() };
    }
    if d.is_infinite() {
        return if d < 0.0 { "-inf".into() } else { "inf".into() };
    }
    let s = format!("{:.3}", d.abs());
    let s = if d.is_sign_negative() { format!("-{s}") } else { s };
    format!("{s:>2}")
}

unsafe fn write_text(item: *mut PerfDispItem, s: &str) {
    let dst = (*item).content.text.as_mut_ptr() as *mut u8;
    core::ptr::copy_nonoverlapping(s.as_ptr(), dst, s.len());
    *dst.add(s.len()) = 0;
}

impl Hsd392A {
    /// `fn_80392934`: sample `HSD_PerfLastStat`, update peaks (with 60-frame decay) and maxima.
    pub fn fn_80392934(&mut self, last: &HSD_PerfStat) {
        let cpu = last.cpu_time;
        self.hsd_804D7858 = cpu;
        let draw = last.draw_time;
        self.hsd_804D785C = draw;
        let total = last.total_time;
        self.hsd_804D7860 = total;

        // `a < x || counter-- < 0`: the decrement only runs when the first test fails.
        if self.hsd_804D7864 < cpu || post_dec(&mut self.hsd_804D7870) < 0 {
            self.hsd_804D7864 = cpu;
            self.hsd_804D7870 = 60;
        }
        if self.hsd_804D7868 < draw || post_dec(&mut self.hsd_804D7874) < 0 {
            self.hsd_804D7868 = draw;
            self.hsd_804D7874 = 60;
        }
        if self.hsd_804D786C < total || post_dec(&mut self.hsd_804D7878) < 0 {
            self.hsd_804D786C = total;
            self.hsd_804D7878 = 60;
        }
        if self.hsd_804D787C < cpu {
            self.hsd_804D787C = cpu;
        }
        if self.hsd_804D7880 < draw {
            self.hsd_804D7880 = draw;
        }
        if self.hsd_804D7884 < total {
            self.hsd_804D7884 = total;
        }
    }

    /// `fn_80392A08`: set mode/scale; turning `enable` on (from off) resets the maxima.
    pub fn fn_80392A08(&mut self, mode: i32, scale: i32, enable: i32) {
        self.lbl_804D6088 = mode;
        self.lbl_804D608C = scale;
        if self.hsd_804D7888 == 0 && enable != 0 {
            self.hsd_804D787C = 0.0;
            self.hsd_804D7880 = 0.0;
            self.hsd_804D7884 = 0.0;
        }
        self.hsd_804D7888 = enable;
    }

    /// `fn_80392A3C`: build the item chain; returns its head (inside `self`) or NULL.
    ///
    /// # Safety
    /// The returned pointer is only valid while `self` is neither moved nor dropped.
    pub unsafe fn fn_80392A3C(&mut self) -> *mut c_void {
        let items = self.hsd_804CE3F8.as_mut_ptr();
        let mut count: usize = 0;
        let num_frames = self.lbl_804D6088;
        if num_frames != 0 {
            let counts = (*items).content.bytes.as_mut_ptr();
            let colors = counts.add(4);
            (*items).r#type = 1;
            (counts as *mut i32).write(1);
            let mut bar_count: usize = 1;
            let green = LBL_804D6098;
            (colors as *mut i32).write(green);
            if num_frames > 1 {
                let mut val = num_frames - 1;
                if val > 3 {
                    val = 3;
                }
                (counts.add(bar_count * 8) as *mut i32).write(val);
                (colors.add(bar_count * 8) as *mut i32).write(LBL_804D60A4);
                bar_count = 2;
            }
            if num_frames > 4 {
                (counts.add(bar_count * 8) as *mut i32).write(num_frames - 4);
                (colors.add(bar_count * 8) as *mut i32).write(LBL_804D6094);
                bar_count += 1;
            }
            (counts.add(bar_count * 8) as *mut i32).write(-1);
            (*items).next = items.add(1) as *mut c_void;

            let nf = num_frames as f32;
            (*items.add(1)).r#type = 2;
            count = 4;
            (*items.add(1)).content.gradient[0].pos = self.hsd_804D7860 / nf;
            (*items.add(1)).content.gradient[0].color = LBL_804D609C as u32;
            (*items.add(1)).content.gradient[1].pos = (melee_core::ppc::fctiwz_f32(0.9999f32 + self.hsd_804D7860)) as f32 / nf;
            (*items.add(1)).content.gradient[1].color = LBL_804D60A0 as u32;
            (*items.add(1)).content.gradient[2].pos = -1.0;
            (*items.add(1)).next = items.add(2) as *mut c_void;
            (*items.add(2)).r#type = 2;
            (*items.add(2)).content.gradient[0].pos = self.hsd_804D785C / nf;
            (*items.add(2)).content.gradient[0].color = LBL_804D6090 as u32;
            (*items.add(2)).content.gradient[1].pos = -1.0;
            (*items.add(2)).next = items.add(3) as *mut c_void;
            (*items.add(3)).r#type = 2;
            (*items.add(3)).content.gradient[0].pos = self.hsd_804D7858 / nf;
            (*items.add(3)).content.gradient[0].color = green as u32;
            (*items.add(3)).content.gradient[1].pos = -1.0;
            (*items.add(3)).next = items.add(4) as *mut c_void;
        }
        if self.lbl_804D608C != 0 {
            (*items.add(count)).r#type = 0;
            let entry = items.add(count);
            let s = format!(
                "\\c00ff00{} \\cffffff{} \\c00ffff{}  \\c00ff00{} \\cffffff{} \\c00ffff{}",
                fmt_f(self.hsd_804D7858),
                fmt_f(self.hsd_804D785C),
                fmt_f(self.hsd_804D7860),
                fmt_f(self.hsd_804D7864),
                fmt_f(self.hsd_804D7868),
                fmt_f(self.hsd_804D786C)
            );
            write_text(entry, &s);
            (*entry).next = items.add(count + 1) as *mut c_void;
            count += 1;
            if self.hsd_804D7888 != 0 {
                (*items.add(count)).r#type = 0;
                let entry2 = items.add(count);
                let s = format!(
                    "\\c00ff00{} \\cffffff{} \\c00ffff{}",
                    fmt_f(self.hsd_804D787C),
                    fmt_f(self.hsd_804D7880),
                    fmt_f(self.hsd_804D7884)
                );
                write_text(entry2, &s);
                (*entry2).next = items.add(count + 1) as *mut c_void;
                count += 1;
            }
        }
        if count == 0 {
            return core::ptr::null_mut();
        }
        (*items.add(count - 1)).next = core::ptr::null_mut();
        items as *mut c_void
    }
}

/// `x--` as an expression: the old value; the stored value wraps like machine arithmetic.
#[inline]
fn post_dec(x: &mut i32) -> i32 {
    let old = *x;
    *x = old.wrapping_sub(1);
    old
}
