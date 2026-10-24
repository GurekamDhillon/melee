//! Port of src/sysdolphin/baselib/hsd_3924.c: the on-screen performance/event overlay. A sorted
//! list of `{ Event callback; int priority }` records; each frame `hsd_8039254C` calls every
//! record's callback (first word, `DispItem* (*)(void*)`) and draws the returned item chain
//! (text, bars, gradients) with the hsd_3915 debug font.
//!
//! The TU's `hsd_804D7850` and `lbl_804D6080` live in [`Hsd3924`]. Allocation / slist calls go
//! through [`Hsd3924Host`] (default [`ext::Ext`]: TODO panics named like the C).
#![allow(non_snake_case)]

use super::hsd_3915::{DrawRectangle, Hsd3915, Hsd3915Host};
use core::ffi::c_void;
use melee_core::fp::{fmadds, fmsubs};
use melee_types::{DispBar, DispCallback, DispItem, Event, EventPriority, GXColor, HSD_SList};

/// Callees of hsd_3924.c outside hsd_3915.c.
pub trait Hsd3924Host {
    fn HSD_MemAlloc(&mut self, size: i32) -> *mut c_void;
    fn HSD_SListAllocAndAppend(&mut self, list: *mut HSD_SList, data: *mut c_void) -> *mut HSD_SList;
    fn HSD_SListAllocAndPrepend(&mut self, list: *mut HSD_SList, data: *mut c_void) -> *mut HSD_SList;
}

pub mod ext {
    use super::*;
    /// Unported callees (TODO: rewire once their TUs are ported).
    pub struct Ext;
    impl Hsd3924Host for Ext {
        fn HSD_MemAlloc(&mut self, _: i32) -> *mut c_void { todo!("HSD_MemAlloc (baselib/memory.c)") }
        fn HSD_SListAllocAndAppend(&mut self, _: *mut HSD_SList, _: *mut c_void) -> *mut HSD_SList { todo!("HSD_SListAllocAndAppend (baselib/list.c)") }
        fn HSD_SListAllocAndPrepend(&mut self, _: *mut HSD_SList, _: *mut c_void) -> *mut HSD_SList { todo!("HSD_SListAllocAndPrepend (baselib/list.c)") }
    }
}

/// The globals of hsd_3924.c.
#[derive(Clone, Copy)]
pub struct Hsd3924 {
    /// `hsd_804D7850`: head of the event list.
    pub hsd_804D7850: *mut HSD_SList,
    /// `lbl_804D6080`: row background colour `{ 0x40, 0x40, 0x40 }`; alpha is 0 (initializer omits
    /// it), so backgrounds are never drawn.
    pub lbl_804D6080: GXColor,
}

impl Default for Hsd3924 {
    fn default() -> Self {
        Hsd3924 { hsd_804D7850: core::ptr::null_mut(), lbl_804D6080: GXColor { r: 0x40, g: 0x40, b: 0x40, a: 0 } }
    }
}

const HSD_3915_DEFAULT_COLOR: GXColor = GXColor { r: 0xFF, g: 0xFF, b: 0xFF, a: 0xFF };

/// `count_bar_units` (static inline): sum of leading positive bar counts. The C steps the item
/// pointer by one `DispBar` and re-reads `content.bars[0]` (offset 8 from the new base).
unsafe fn count_bar_units(item: *const DispItem) -> i32 {
    let mut total: i32 = 0;
    let mut base = item as *const u8;
    loop {
        let count = (base.add(8) as *const i32).read();
        if count <= 0 {
            break;
        }
        total = total.wrapping_add(count);
        base = base.add(core::mem::size_of::<DispBar>());
    }
    total
}

/// `count_text_chars` (static inline): visible characters, skipping `\cRRGGBB` escapes.
unsafe fn count_text_chars(text: *const i8) -> i32 {
    if text.is_null() {
        return 0;
    }
    let mut count = 0i32;
    let mut len = 0isize;
    while *text.offset(len) != 0 {
        len += 1;
    }
    let mut i = 0isize;
    while i < len {
        if *text.offset(i) != b'\\' as i8 {
            count += 1;
        } else {
            i += 1;
            if matches!(*text.offset(i) as u8, b'c' | b'C') {
                i += 6;
            }
        }
        i += 1;
    }
    count
}

impl Hsd3924 {
    /// `hsd_80392474`: clear the event list.
    pub fn hsd_80392474(&mut self) {
        self.hsd_804D7850 = core::ptr::null_mut();
    }

    /// `fn_80392480`: find `event` in the priority-sorted list (returns its record), else insert
    /// a new `{ event, priority }` record after the last node with priority <= `priority`.
    ///
    /// # Safety
    /// The list was built by this function; `H` allocations are valid.
    pub unsafe fn fn_80392480<H: Hsd3924Host>(&mut self, host: &mut H, event: Event, priority: i32) -> *mut HSD_SList {
        let mut prev: *mut HSD_SList = core::ptr::null_mut();
        let mut cur = self.hsd_804D7850;
        while !cur.is_null() {
            let ret = (*cur).data as *mut EventPriority;
            if event != (*ret).event {
                if (*ret).priority <= priority {
                    prev = cur;
                }
                cur = (*cur).next;
            } else {
                return ret as *mut HSD_SList;
            }
        }
        let data = host.HSD_MemAlloc(core::mem::size_of::<EventPriority>() as i32) as *mut EventPriority;
        (*data).event = event;
        (*data).priority = priority;
        if !prev.is_null() {
            return host.HSD_SListAllocAndAppend(prev, data as *mut c_void);
        }
        let ret = host.HSD_SListAllocAndPrepend(self.hsd_804D7850, data as *mut c_void);
        self.hsd_804D7850 = ret;
        ret
    }

    /// `hsd_80392528`: register `event` at the default priority 0x80.
    ///
    /// # Safety
    /// As [`Self::fn_80392480`].
    pub unsafe fn hsd_80392528<H: Hsd3924Host>(&mut self, host: &mut H, event: Event) {
        self.fn_80392480(host, event, 0x80);
    }

    /// `hsd_8039254C`: run every callback and draw its items.
    ///
    /// # Safety
    /// Each list record's first word is a valid `DispItem* (*)(void*)`; returned chains are valid
    /// and terminated.
    pub unsafe fn hsd_8039254C<G: Hsd3915Host>(&mut self, g: &mut G, f: &mut Hsd3915) {
        let mut col_pos: i32 = 60;
        let mut first = true;
        let mut line: f32 = 1.0;
        let mut event_node = self.hsd_804D7850;
        let default_col = HSD_3915_DEFAULT_COLOR;
        let bg = self.lbl_804D6080;

        while !event_node.is_null() {
            let data = (*event_node).data;
            let cb: DispCallback = *(data as *const DispCallback);
            let mut item: *mut DispItem = (cb.expect("DispCallback"))(data);

            while !item.is_null() {
                if first {
                    if bg.a != 0 {
                        f.hsd_80391A04(g, 10.0, 10.0, 6);
                        DrawRectangle(g, -10.0, 5.0, 620.0, 10.0, &bg);
                    }
                    first = false;
                }
                match (*item).r#type {
                    0 => {
                        let text = (*item).content.text.as_ptr();
                        let char_count = count_text_chars(text);
                        if col_pos + char_count > 60 {
                            line -= 1.0;
                            col_pos = 0;
                            if bg.a != 0 {
                                DrawRectangle(g, -10.0, fmsubs(10.0, line, 5.0), 620.0, 10.0, &bg);
                            }
                        }
                        f.hsd_80391A04(g, 10.0, 10.0, 6);
                        let mut txt_col = default_col;
                        f.hsd_80391AC8(g, text, &mut txt_col, (col_pos * 10) as f32, 10.0 * line);
                        col_pos += char_count + 2;
                    }
                    2 => {
                        if col_pos != 0 {
                            line = (line as f64 - 0.5) as f32;
                            if bg.a != 0 {
                                DrawRectangle(g, -10.0, fmsubs(10.0, line, 5.0), 620.0, 5.0, &bg);
                            }
                        }
                        f.hsd_80391A04(g, 10.0, 10.0, 24);
                        let t2 = fmadds(10.0, line, 2.0);
                        super::hsd_3915::hsd_80391E18(g, (*item).content.gradient.as_ptr(), 0.0, t2, 600.0, t2);
                        col_pos = 60;
                    }
                    1 => {
                        let char_count = count_bar_units(item);
                        if char_count > 0 {
                            if col_pos != 0 {
                                line = (line as f64 - 0.5) as f32;
                                if bg.a != 0 {
                                    DrawRectangle(g, -10.0, fmsubs(10.0, line, 5.0), 620.0, 5.0, &bg);
                                }
                            }
                            f.hsd_80391A04(g, 10.0, 10.0, 12);
                            let bar_y = fmadds(10.0, line, 2.0);
                            let mut bar_x: f32 = 0.0;
                            let mut base = item as *const u8;
                            loop {
                                let count = (base.add(8) as *const i32).read();
                                if count <= 0 {
                                    break;
                                }
                                let prev_x = bar_x;
                                // `bar_x += (600.0F / (f32) char_count) * (f32) count`
                                bar_x = fmadds(600.0 / char_count as f32, count as f32, bar_x);
                                let bar_col = (base.add(12) as *const GXColor).read();
                                super::hsd_3915::hsd_80391F28(g, &bar_col, prev_x, bar_y, bar_x, bar_y, count as f32);
                                base = base.add(core::mem::size_of::<DispBar>());
                            }
                            col_pos = 60;
                        }
                    }
                    _ => {}
                }
                item = (*item).next;
            }
            event_node = (*event_node).next;
        }
    }
}
