//! Port of src/sysdolphin/baselib/debugconsole_main.c: the on-screen exception console
//! (`ParticleScreenState` is a misnomer for its state): XFB text drawing, the crash report
//! (`Exception_ReportStackTrace`, `Exception_ReportCodeline`, the FPR / PSF dump) and the entry
//! points the OS error handlers call (`hsd_80397DA4`, `Exception_StoreDebugLevel`,
//! `hsd_80397DFC`).
//!
//! This file is the GameCube crash screen: it draws on the external framebuffer, walks raw
//! console memory and reads SPRs, so it is dead on PC. Ported here: the state record, the colour
//! tables, the SPR table (data), the console init / text / cell / redraw routines and the report
//! functions. NOT ported (interactive viewer driven by the pad over linked static nodes with raw
//! console memory dumps; never reached on PC): `hsd_80394E8C`, `hsd_80394F48`, `hsd_80395550`,
//! `hsd_80395644`, `hsd_803956D8`, `hsd_803957C0`, `hsd_80395970`, `hsd_80395A78`,
//! `hsd_80395D88`, `hsd_80396130`, `hsd_80396188`, `hsd_803962A8`, `hsd_803966A0`,
//! `hsd_80396868`, `hsd_80396884`, `hsd_80396A20`, `hsd_80396C78`, `hsd_80396E40`,
//! `fn_803970D8`..`fn_80397108`, `hsd_80397110`, `fn_80397374`, `hsd_80397520`,
//! `hsd_803975D4`, `fn_80397814` ([`ext`] hooks named like the C stand in for them).
//!
//! Native shape: console addresses (framebuffers, font atlas, stack frames) are GameCube `u32`
//! addresses; reading or writing them goes through the [`ext`] hooks (`gc_read_u32` /
//! `gc_read_u8` / `gc_write_u8`). The draw routines `hsd_803921B8` / `hsd_803922FC` take the
//! glyph address and the colour-table address (0 = none; the tables below sit at their
//! console addresses 0x8040AB00 / AB20 / AB40, see [`ColorSel`]).
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, clippy::too_many_arguments)]

use melee_types::OSContext;

pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
                }
            } )*
        };
    }
    ext_fns! {
    fn OSReport(s: &str);
    fn OSDisableInterrupts() -> u32;
    fn OSRestoreInterrupts(level: u32);
    fn OSGetCurrentContext() -> *mut OSContext;
    fn OSSetCurrentContext(ctx: *mut OSContext);
    fn OSClearContext(ctx: *mut OSContext);
    fn OSGetPhysicalMemSize() -> u32;
    // OSCreateThread(thread, fn_80397814, ctx, stack = hsd_804CF8E8 + 0xFFC, 0x1000, 0, 1) + OSResumeThread
    fn OSCreateAndResumeDebugThread(ctx: *mut OSContext);
    fn gc_read_u32(addr: u32) -> u32;
    fn gc_read_u8(addr: u32) -> u8;
    fn gc_write_u8(addr: u32, v: u8);
    // video / font (hsd_3915.c, hsd_393C.c)
    fn hsd_803941E8(x24: &mut i32, x2c: &mut i32) -> i32;
    fn hsd_803921B8(glyph: u32, x: i32, y: i32, xfb: i32, w: i32, h: i32, stride: i32, color: u32);
    fn hsd_803922FC(glyph: u32, x: i32, y: i32, interlace: i32, xfb: i32, w: i32, h: i32, stride: i32, color: u32);
    fn hsd_80393E68(col: u32, row: u32);
    fn hsd_80394068() -> u8;
    // HSD_VIData: u16 at byte offset 4 and 8 (the xfb size), its address, and the font atlas address
    fn HSD_VIData_u16(byte_offset: i32) -> u16;
    fn HSD_VIData_addr() -> u32;
    fn HSD_DebugFontAtlas_addr() -> u32;
    }
}

/// `hsd_804D78CC` and `hsd_804D78C8`.
pub static mut hsd_804D78CC: u32 = 0;
pub static mut hsd_804D78C8: i32 = 0;

/// The three console colour tables' addresses (`lbl_8040AB00`, `AB20`, `AB40`); 0 is NULL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSel {
    None,
    Ab00,
    Ab20,
    Ab40,
}

impl ColorSel {
    /// The address the draw routines receive.
    pub fn addr(self) -> u32 {
        match self {
            ColorSel::None => 0,
            ColorSel::Ab00 => 0x8040AB00,
            ColorSel::Ab20 => 0x8040AB20,
            ColorSel::Ab40 => 0x8040AB40,
        }
    }
}

/// `struct lbl_8040AB00_t`: four `{ colour word, hsd_80392194 }` pairs (the function is the
/// per-pixel colour callback `hsd_80392194`, kept by address).
#[derive(Clone, Copy, Debug)]
pub struct ColorTable {
    pub colors: [u32; 4],
    pub func: u32,
}

pub const lbl_8040AB00: ColorTable = ColorTable { colors: [0x10808000, 0x46808000, 0x7C808000, 0xB3808000], func: 0x80392194 };
pub const lbl_8040AB20: ColorTable = ColorTable { colors: [0xB3808000, 0x7C808000, 0x46808000, 0x10808000], func: 0x80392194 };
pub const lbl_8040AB40: ColorTable = ColorTable { colors: [0x10808000; 4], func: 0x80392194 };

/// `struct SPREntry`: SPR number, name, description, handler (by symbol name).
#[derive(Clone, Copy, Debug)]
pub struct SprEntry {
    pub spr: u32,
    pub name: Option<&'static str>,
    pub description: Option<&'static str>,
    pub callback: Option<&'static str>,
}

/// `lbl_8040ADD8[]`: the SPR table, including the all-NULL separator entry between the user and
/// the supervisor SPRs.
pub const lbl_8040ADD8: [SprEntry; 69] = [
    SprEntry { spr: 0x00000009, name: Some("CTR"), description: Some("Count Register"), callback: None },
    SprEntry { spr: 0x000003F5, name: Some("DABR"), description: Some("Data Address Breakpoint"), callback: None },
    SprEntry { spr: 0x00000013, name: Some("DAR"), description: Some("Data Address Register"), callback: None },
    SprEntry { spr: 0x00000219, name: Some("DBAT0L"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000218, name: Some("DBAT0U"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x0000021B, name: Some("DBAT1L"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x0000021A, name: Some("DBAT1U"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x0000021D, name: Some("DBAT2L"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x0000021C, name: Some("DBAT2U"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x0000021F, name: Some("DBAT3L"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x0000021E, name: Some("DBAT3U"), description: Some("Data Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000016, name: Some("DEC"), description: Some("Descrimenter"), callback: None },
    SprEntry { spr: 0x00000012, name: Some("DSISR"), description: Some("Data Access/Alignment Exception"), callback: Some("fn_803970D8") },
    SprEntry { spr: 0x0000011A, name: Some("EAR"), description: Some("External Access"), callback: None },
    SprEntry { spr: 0x00000211, name: Some("IBAT0L"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000210, name: Some("IBAT0U"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000213, name: Some("IBAT1L"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000212, name: Some("IBAT1U"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000215, name: Some("IBAT2L"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000214, name: Some("IBAT2U"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000217, name: Some("IBAT3L"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000216, name: Some("IBAT3U"), description: Some("Inst Block Address Translation"), callback: Some("hsd_80396E40") },
    SprEntry { spr: 0x00000008, name: Some("LR"), description: Some("Link Register"), callback: None },
    SprEntry { spr: 0x0000011F, name: Some("PVR"), description: Some("Processor Version"), callback: None },
    SprEntry { spr: 0x000003BF, name: Some("SDA"), description: Some("Sample Data Address"), callback: None },
    SprEntry { spr: 0x00000019, name: Some("SDR1"), description: Some("Page Table Format"), callback: None },
    SprEntry { spr: 0x00000110, name: Some("SPRG0"), description: Some("For Operation System"), callback: None },
    SprEntry { spr: 0x00000111, name: Some("SPRG1"), description: Some("For Operation System"), callback: None },
    SprEntry { spr: 0x00000112, name: Some("SPRG2"), description: Some("For Operation System"), callback: None },
    SprEntry { spr: 0x00000113, name: Some("SPRG3"), description: Some("For Operation System"), callback: None },
    SprEntry { spr: 0x0000001A, name: Some("SRR0"), description: Some("Machine Status Save/Restore"), callback: Some("fn_803970DC") },
    SprEntry { spr: 0x0000001B, name: Some("SRR1"), description: Some("Machine Status Save/Restore"), callback: Some("fn_803970DC") },
    SprEntry { spr: 0x000003AF, name: Some("USDA"), description: Some("Sample Data Address"), callback: None },
    SprEntry { spr: 0x00000001, name: Some("XER"), description: Some("?"), callback: Some("fn_803970E0") },
    SprEntry { spr: 0x00000000, name: None, description: None, callback: None },
    SprEntry { spr: 0x0000039B, name: Some("DMA_L"), description: Some("Direct Memory Access"), callback: Some("fn_803970E4") },
    SprEntry { spr: 0x0000039A, name: Some("DMA_U"), description: Some("Direct Memory Access"), callback: Some("fn_803970E4") },
    SprEntry { spr: 0x00000390, name: Some("GQR0"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x00000391, name: Some("GQR1"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x00000392, name: Some("GQR2"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x00000393, name: Some("GQR3"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x00000394, name: Some("GQR4"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x00000395, name: Some("GQR5"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x00000396, name: Some("GQR6"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x00000397, name: Some("GQR7"), description: Some("Graphics Quantization"), callback: Some("fn_803970E8") },
    SprEntry { spr: 0x000003F0, name: Some("HID0"), description: Some("Hardware Implementation Depend"), callback: Some("fn_803970EC") },
    SprEntry { spr: 0x000003F1, name: Some("HID1"), description: Some("Hardware Implementation Depend"), callback: Some("fn_803970EC") },
    SprEntry { spr: 0x00000398, name: Some("HID2"), description: Some("Hardware Implementation Depend"), callback: Some("fn_803970EC") },
    SprEntry { spr: 0x000003F2, name: Some("IABR"), description: Some("Instruction Breakpoint"), callback: Some("fn_803970F0") },
    SprEntry { spr: 0x000003FB, name: Some("ICTC"), description: Some("Instruction Cache Throttle Control"), callback: Some("fn_803970F4") },
    SprEntry { spr: 0x000003F9, name: Some("L2CR"), description: Some("L2 Cache Control"), callback: Some("fn_803970F8") },
    SprEntry { spr: 0x000003B8, name: Some("MMCR0"), description: Some("Perf. Moniter Mode Control"), callback: Some("fn_803970FC") },
    SprEntry { spr: 0x000003BC, name: Some("MMCR1"), description: Some("Perf. Monitor Mode Control"), callback: Some("fn_803970FC") },
    SprEntry { spr: 0x000003B9, name: Some("PMC1"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003BA, name: Some("PMC2"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003BD, name: Some("PMC3"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003BE, name: Some("PMC4"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003BB, name: Some("SIA"), description: Some("Sample Instruction Address"), callback: None },
    SprEntry { spr: 0x000003FC, name: Some("THRM1"), description: Some("Thermal Managiment"), callback: Some("fn_80397104") },
    SprEntry { spr: 0x000003FD, name: Some("THRM2"), description: Some("Thermal Managiment"), callback: Some("fn_80397104") },
    SprEntry { spr: 0x000003FE, name: Some("THRM3"), description: Some("Thermal Managiment"), callback: Some("fn_80397104") },
    SprEntry { spr: 0x000003A8, name: Some("UMMCR0"), description: Some("Perf. Moniter Mode Control"), callback: Some("fn_803970FC") },
    SprEntry { spr: 0x000003AC, name: Some("UMMCR1"), description: Some("Perf. Moniter Mode Control"), callback: Some("fn_803970FC") },
    SprEntry { spr: 0x000003A9, name: Some("UPMC1"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003AA, name: Some("UPMC2"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003AD, name: Some("UPMC3"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003AE, name: Some("UPMC4"), description: Some("Performance Monitor Counter"), callback: Some("fn_80397100") },
    SprEntry { spr: 0x000003AB, name: Some("USIA"), description: Some("Sample Instruction Address"), callback: None },
    SprEntry { spr: 0x00000399, name: Some("WPAR"), description: Some("Write Pipe Address"), callback: Some("fn_80397108") },
];

/// `struct ParticleScreenState` (`hsd_804CF810`), native: the console's cursor, geometry and
/// framebuffer bookkeeping. Bit-field `x0` is three bools (`x0_b7`, `x0_b6`, `x0_b5`).
#[derive(Clone, Debug)]
pub struct ParticleScreenState {
    pub x0_b7: bool,
    pub x0_b6: bool,
    pub x0_b5: bool,
    pub x4: i32,
    pub x8: i32,
    pub x0C: i32,
    pub x10: i32,
    pub x14: i32,
    pub x18: i32,
    pub x1C: i32,
    pub x20: i32,
    /// First framebuffer (`x24`), second (`x28`), and the copy source (`x2C`).
    pub x24: i32,
    pub x28: u32,
    pub x2C: i32,
    /// `&HSD_VIData`.
    pub x30: u32,
    pub x34: i32,
    pub x38: i32,
    pub x3C: i32,
    pub x40: i32,
    pub x44: i32,
    pub x48: i32,
    /// Font atlas address.
    pub x4C: u32,
    pub x50: ColorSel,
    pub xBC: u32,
    pub xC0: u32,
    pub xC4: i32,
    pub xC8: i32,
    pub xCC: i32,
    /// Head of the node list (opaque handle of the interactive viewer, 0 = none).
    pub xD0: u32,
    pub xD4: *mut OSContext,
}

impl Default for ParticleScreenState {
    fn default() -> Self {
        ParticleScreenState {
            x0_b7: false, x0_b6: false, x0_b5: false, x4: 0, x8: 0, x0C: 0, x10: 0, x14: 0, x18: 0, x1C: 0, x20: 0,
            x24: 0, x28: 0, x2C: 0, x30: 0, x34: 0, x38: 0, x3C: 0, x40: 0, x44: 0, x48: 0, x4C: 0, x50: ColorSel::None,
            xBC: 0, xC0: 0, xC4: 0, xC8: 0, xCC: 0, xD0: 0, xD4: core::ptr::null_mut(),
        }
    }
}

impl ParticleScreenState {
    /// `(&sp->x24)[sp->x34]`: the framebuffer being drawn (x24 or x28).
    fn draw_xfb(&self) -> i32 {
        if self.x34 == 0 { self.x24 } else { self.x28 as i32 }
    }

    /// `hsd_80394314`: reset the console state and size it to the video mode.
    pub fn hsd_80394314(&mut self) {
        *self = ParticleScreenState::default();
        ext::hsd_803941E8(&mut self.x24, &mut self.x2C);

        self.x38 = if self.x28 != 0 { 2 } else { 1 };

        self.x34 = 0;
        self.x30 = ext::HSD_VIData_addr();
        self.x3C = ext::HSD_VIData_u16(4) as i32;
        self.x40 = ext::HSD_VIData_u16(8) as i32;

        self.x44 = ((((self.x3C as u16) as i32) + 15) * 2) & 0x1FFE0;

        self.x48 = self.x44.wrapping_mul(self.x40);
        self.x4 = 0;
        self.x8 = self.x40;
        self.x18 = 0;
        self.x14 = 0;
        self.x20 = ((self.x3C.wrapping_sub(0x28)) as u32 / 11) as i32;
        self.x1C = ((self.x40.wrapping_sub(0x50)) as u32 / 14) as i32;
        self.x4C = ext::HSD_DebugFontAtlas_addr();
        self.x50 = ColorSel::None;
        self.xC4 = 0;
    }

    /// One glyph draw at (`x`, `y`) on the current framebuffer (the shared body of the C's
    /// `mode != 0` ? interlaced : plain pair).
    fn draw_glyph(&self, glyph: u32, x: i32, y: i32) {
        if self.x0_b7 {
            ext::hsd_803922FC(glyph, x, y, self.x0_b6 as i32, self.draw_xfb(), self.x3C, self.x40, self.x44, self.x50.addr());
        } else {
            ext::hsd_803921B8(glyph, x, y, self.draw_xfb(), self.x3C, self.x40, self.x44, self.x50.addr());
        }
    }

    /// `hsd_80394434`: print a NUL-free byte string at the cursor (`\n` wraps to the left margin
    /// and draws that cell too, as the C falls through; `\r` is skipped).
    pub fn hsd_80394434(&mut self, text: &[u8]) {
        let mut x = self.x4;
        let mut y = self.x8;
        let left = self.x4;
        for &b in text {
            let c = b as i8;
            if c == 0 {
                break;
            }
            match c {
                13 => continue,
                10 => {
                    x = left;
                    y += 14;
                }
                _ => {}
            }
            self.draw_glyph(self.x4C.wrapping_add(((c as i32 & 0x7F) * 0x38) as u32), x, y);
            x += 11;
        }
    }
}

/// `hsd_80394544`: draw a `num_cols` x `num_rows` window of the console text buffer onto an
/// arbitrary framebuffer.
pub fn hsd_80394544(
    sp: &ParticleScreenState,
    col: i32,
    row: i32,
    num_cols: u32,
    num_rows: u32,
    x: i32,
    mut y: i32,
    xfb_buf: i32,
    xfb_w: i32,
    xfb_h: i32,
    xfb_stride: i32,
    font_data: u32,
    color_data: u32,
) {
    let mode = sp.x0_b7;
    let interlace = sp.x0_b6 as i32;
    let mut r: u32 = 0;

    while r < num_rows {
        ext::hsd_80393E68(col as u32, row.wrapping_add(r as i32) as u32);
        y -= 14;

        let mut c: u32 = 0;
        while c < num_cols {
            let ch = ext::hsd_80394068();
            if ch == 0 {
                break;
            }
            let glyph = font_data.wrapping_add(((ch & 0x7F) as u32) * 0x38);
            let gx = x.wrapping_add((c as i32).wrapping_mul(11));
            if mode {
                ext::hsd_803922FC(glyph, gx, y, interlace, xfb_buf, xfb_w, xfb_h, xfb_stride, color_data);
            } else {
                ext::hsd_803921B8(glyph, gx, y, xfb_buf, xfb_w, xfb_h, xfb_stride, color_data);
            }
            c += 1;
        }
        r += 1;
    }
}

/// `hsd_80394668`: redraw the console: copy the saved screen with the luma lowered, or, with no
/// saved copy, clear the text grid with blank cells in the AB40 colours.
pub fn hsd_80394668(sp: &mut ParticleScreenState) {
    let src = sp.x2C as u32;
    if src != 0 {
        // Copy XFB data with brightness adjustment.
        let dst = if sp.x34 == 0 { sp.x24 as u32 } else { sp.x28 };
        let size = sp.x48 as u32;
        let mut pos: u32 = 0;
        while pos < size {
            let s0 = ext::gc_read_u8(src.wrapping_add(pos));
            let s1 = ext::gc_read_u8(src.wrapping_add(pos).wrapping_add(1));
            ext::gc_write_u8(dst.wrapping_add(pos), ((((s0 as i32) - 0x10) & 0xFF_FFFF) + 0x10) as u8);
            ext::gc_write_u8(dst.wrapping_add(pos).wrapping_add(1), s1);
            pos += 2;
        }
    } else {
        // Draw console text to the framebuffer: a blank cell (glyph 0x700) at every grid position.
        let saved_color = sp.x50;
        sp.x50 = ColorSel::Ab40;
        let mut row = 0;
        while row <= sp.x1C {
            let y_off = (row + 1) * 14;
            let mut cur_x = 0x14;
            let mut col = 0;
            while col < sp.x20 {
                sp.x4 = cur_x;
                sp.x8 = (sp.x40 - 0x28) - y_off;
                sp.draw_glyph(sp.x4C.wrapping_add(0x700), sp.x4, sp.x8);
                cur_x += 11;
                col += 1;
            }
            row += 1;
        }
        sp.x50 = saved_color;
    }
}

/// C `%e` of a double: `d.dddddde+XX`.
fn fmt_e(v: f64) -> String {
    if v.is_nan() {
        return if v.is_sign_negative() { "-nan".into() } else { "nan".into() };
    }
    if v.is_infinite() {
        return if v < 0.0 { "-inf".into() } else { "inf".into() };
    }
    let s = format!("{:.6e}", v);
    let (m, e) = s.split_once('e').unwrap();
    let e: i32 = e.parse().unwrap();
    format!("{}e{}{:02}", m, if e < 0 { '-' } else { '+' }, e.abs())
}

/// `hsd_80394950`: dump the FPRs and the "PSF" rows of a saved context (the C reads the FPR words
/// again for the PSF rows, as floats).
///
/// # Safety
/// `ctx` points at a live context.
pub unsafe fn hsd_80394950(ctx: *mut OSContext) {
    const OS_CONTEXT_STATE_FPSAVED: u16 = 1;
    if (*ctx).state & OS_CONTEXT_STATE_FPSAVED == 0 {
        return;
    }

    let irq = ext::OSDisableInterrupts();
    let saved = ext::OSGetCurrentContext();
    let mut tmp = OSContext::zeroed();
    ext::OSClearContext(&mut tmp);
    ext::OSSetCurrentContext(&mut tmp);

    ext::OSReport("- FPR -----------------------------------------------\n");

    for i in 0..32 {
        let bits = (*ctx).fpr[i].to_bits();
        ext::OSReport(&format!(
            "R{:02}={:08X}:{:08X} ({})\n",
            i,
            (bits >> 32) as u32,
            bits as u32,
            fmt_e((*ctx).fpr[i])
        ));
    }

    ext::OSReport("- PSF -----------------------------------------------\n");

    for j in 0..32 {
        let bits = (*ctx).fpr[j].to_bits();
        let (val0, val1) = ((bits >> 32) as u32, bits as u32);
        ext::OSReport(&format!(
            "R{:02}={:08X}:{:08X} ({}, {})\n",
            j,
            val0,
            val1,
            fmt_e(f32::from_bits(val0) as f64),
            fmt_e(f32::from_bits(val1) as f64)
        ));
    }

    ext::OSClearContext(&mut tmp);
    ext::OSSetCurrentContext(saved);
    ext::OSRestoreInterrupts(irq);
}

/// `Exception_ReportStackTrace`: walk the back chain from r1 (`StackFrame { back_chain, saved_lr }`
/// in console memory).
///
/// # Safety
/// `ctx` points at a live context.
pub unsafe fn Exception_ReportStackTrace(ctx: *mut OSContext, max_depth: i32) {
    ext::OSReport("- STACK ---------------------------------------------\n");
    ext::OSReport(" Address:  Back Chain  LR Save\n");

    let mut frame: u32 = (*ctx).gpr[1];
    let mut i: u32 = 0;
    while frame != 0 && frame != 0xFFFF_FFFF && i < max_depth as u32 {
        if frame < 0x8000_0000 {
            break;
        }
        if frame as i64 >= ext::OSGetPhysicalMemSize() as i64 + 0x8_0000_0000 {
            break;
        }
        let back_chain = ext::gc_read_u32(frame);
        let saved_lr = ext::gc_read_u32(frame.wrapping_add(4));
        ext::OSReport(&format!("{:08X}:   {:08X}   {:08X}\n", frame, back_chain, saved_lr));
        frame = back_chain;
        i += 1;
    }
}

/// `Exception_ReportCodeline`.
///
/// # Safety
/// `ctx` points at a live context.
pub unsafe fn Exception_ReportCodeline(error: u16, dsisr: i32, dar: i32, ctx: *mut OSContext) {
    ext::OSReport("- UNHANDLED EXCEPTION -------------------------------\n");
    ext::OSReport(&format!("DSISR={:08X} DAR={:08X}\n", dsisr as u32, dar as u32));
    ext::OSReport(&format!("ERROR {}: ", error as i32));

    match error {
        0 => {
            ext::OSReport("(SYSTEM RESET)\n");
        }
        1 => {
            ext::OSReport("(MACHINE CHECK)\n");
        }
        2 => {
            ext::OSReport("(DSI)\n");
            ext::OSReport(&format!(
                "Instruction at {:08X} (read from SRR0) attempted\nto access invalid address {:08X} (read from DAR)\n",
                (*ctx).srr0,
                dar as u32
            ));
            if dsisr & 0x0200_0000 != 0 {
                ext::OSReport("Store operation.\n");
            } else {
                ext::OSReport("Load operation.\n");
            }
            if dsisr & 0x4000_0000 != 0 {
                ext::OSReport("No appropriate page entry or DBAT exist.\n");
            }
            if dsisr & 0x0800_0000 != 0 {
                ext::OSReport("Memory access is not permitted by the page or DBAT.\n");
            }
            if dsisr & 0x40_0000 != 0 {
                ext::OSReport("DABR match occurs.\n");
            }
        }
        3 => {
            ext::OSReport("(ISI)\n");
            ext::OSReport(&format!(
                "Attempted to fetch instruction from invalid address\n{:08X} (read from SRR0)\n",
                (*ctx).srr0
            ));
            if (*ctx).srr1 & 0x4000_0000 != 0 {
                ext::OSReport("No appropriate page entry or IBAT exist.\n");
            }
            if (*ctx).srr1 & 0x1000_0000 != 0 {
                ext::OSReport("Illegal address.\n");
            }
            if (*ctx).srr1 & 0x0800_0000 != 0 {
                ext::OSReport("Memory access is not permitted by the page or IBAT.\n");
            }
        }
        4 => {
            ext::OSReport("(EXTERNAL INTERRUPT)\n");
        }
        5 => {
            ext::OSReport("(ALIGNMNET)\n");
            ext::OSReport(&format!(
                "Instruction at {:08X} (read from SRR0) attempted\nto access unaligned address {:08X} (read from DAR)\n",
                (*ctx).srr0,
                dar as u32
            ));
        }
        6 => {
            ext::OSReport("(PROGRAM)\n");
            let srr1 = (*ctx).srr1;
            let around_or_at = if srr1 & 0x10000 != 0 { "around" } else { "at" };
            let exception_type = if srr1 & 0x10_0000 != 0 {
                "IEEE floating point enabled program exception"
            } else if srr1 & 0x8_0000 != 0 {
                "Illegal instruction"
            } else if srr1 & 0x4_0000 != 0 {
                "Privileged instruction"
            } else if srr1 & 0x2_0000 != 0 {
                "Trap program exception"
            } else {
                "Unknown type of program exception"
            };
            ext::OSReport(&format!("{} {} {:08X} (read from SRR0)\n", exception_type, around_or_at, (*ctx).srr0));
        }
        7 => {
            ext::OSReport("(FLOATING POINT)\n");
        }
        8 => {
            ext::OSReport("(DECREMENTER)\n");
        }
        9 => {
            ext::OSReport("(SYSTEM CALL)\n");
        }
        10 => {
            ext::OSReport("(TRACE)\n");
        }
        11 => {
            ext::OSReport("(PERFORMANCE MONITOR)\n");
        }
        12 => {
            ext::OSReport("(BREAKPOINT)\n");
        }
        13 => {
            ext::OSReport("(SYSTEM INTERRPUT)\n");
        }
        14 => {
            ext::OSReport("(THERMAL INTERRPUT)\n");
        }
        _ => {
            ext::OSReport("(UNKNOWN)\n");
        }
    }
}

/// `hsd_80397DA4`: run the debug console thread (`fn_80397814`) on `ctx`.
pub fn hsd_80397DA4(ctx: *mut OSContext) {
    ext::OSCreateAndResumeDebugThread(ctx);
}

/// `Exception_StoreDebugLevel`.
pub fn Exception_StoreDebugLevel(level: i32) {
    unsafe { hsd_804D78C8 = level };
}

/// `hsd_80397DFC`.
pub fn hsd_80397DFC(size: u32) {
    unsafe { hsd_804D78CC = size.wrapping_add(0xF) >> 4 };
}
