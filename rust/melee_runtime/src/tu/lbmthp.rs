//! src/melee/lb/lbmthp.c: the THP movie player (one static `MoviePlayer` decoder): a ring of
//! DevCom-fed frame buffers, a 60 Hz alarm that advances the displayed frame, and the render
//! callback that decodes the current frame and draws it as three I8 textures through an SObj.
//!
//! Byte order: the THP header and every frame's `next_packed_size` are big-endian in the file; the
//! native structs are little-endian, so the port swaps those words where it reads them (after the
//! 0x40-byte header read in `fn_8001EB14`, and at each `next_packed_size` read). The frame payload
//! is handed to the decoder untouched. The frame-buffer pointer table is sized with the native
//! pointer width (`ALIGN_32(count * 4)` of the GameCube build).
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs)]

use core::ffi::{c_char, c_void};
use core::ptr::{addr_of_mut, null_mut};
use melee_types::*;
use super::lbext::OS_BUS_CLOCK;

/// Callees in TUs that are not ported (dolphin thp/gx/os/dvd, baselib sobjlib/video/memory).
/// Each is a settable hook; unset it panics "TODO: <name> is not ported".
pub mod ext {
    use super::*;
    use crate::tu::lbext::lb_hook;
    pub use crate::tu::lbext::{OSReport, OsArg, __assert};
    pub use crate::tu::lbfile::ext::DVDConvertPathToEntrynum;
    pub use crate::tu::lbheap::ext::{HSD_Free, HSD_MemAlloc};
    pub use crate::tu::lbmemory::ext::{HSD_DevComRequest, OSCreateAlarm, OSDisableInterrupts, OSRestoreInterrupts};
    lb_hook!(H_THPInit, THPInit, () = || unsafe { crate::tu::THPDec::THPInit() });
    lb_hook!(H_THPDec_8032FD40, THPDec_8032FD40, (a: *mut THPDec_8032FD40_Data, b: u16) -> i32 = |a, b| unsafe { crate::tu::THPDec::THPDec_8032FD40(a, b) as _ });
    lb_hook!(H_THPVideoDecode, THPVideoDecode, (header: *mut c_void, status: *mut c_void, work: *mut _THPFileInfo, data: *mut c_void, desc: *mut THPDec_8032FD40_Data) -> *mut _THPFileInfo = |header, status, work, data, desc| unsafe { crate::tu::THPDec::THPVideoDecode(header as _, status as _, work as _, data as _, desc) as _ });
    lb_hook!(H_THPDec_80331340, THPDec_80331340, (a: *mut _THPFileInfo, y: *mut c_void, u: *mut c_void, v: *mut c_void) = |a, y, u, v| unsafe { crate::tu::THPDec::THPDec_80331340(a as _, y as _, u as _, v as _) });
    lb_hook!(H_THPDec_803313D0, THPDec_803313D0, (a: *mut _THPFileInfo, y: *mut c_void, u: *mut c_void, v: *mut c_void, width: u32) = |a, y, u, v, width| unsafe { crate::tu::THPDec::THPDec_803313D0(a as _, y as _, u as _, v as _, width) });
    lb_hook!(H_DCInvalidateRange, DCInvalidateRange, (addr: *mut c_void, n: u32));
    lb_hook!(H_DCStoreRange, DCStoreRange, (addr: *mut c_void, n: u32));
    lb_hook!(H_GXInitTexObj, GXInitTexObj, (obj: *mut _GXTexObj, image: *mut c_void, w: u16, h: u16, fmt: i32, wrap_s: i32, wrap_t: i32, mipmap: u8));
    lb_hook!(H_GXInitTexObjLOD, GXInitTexObjLOD, (obj: *mut _GXTexObj, min: i32, mag: i32, min_lod: f32, max_lod: f32, bias: f32, bias_clamp: u8, edge_lod: u8, aniso: i32));
    lb_hook!(H_GXLoadTexObj, GXLoadTexObj, (obj: *mut _GXTexObj, id: i32));
    lb_hook!(H_HSD_SObjLib_803A477C, HSD_SObjLib_803A477C, (g: *mut HSD_GObj, d: *mut HSD_SObjDesc, s: i32, t: i32, a: u8, b: u8) -> *mut HSD_SObj);
    lb_hook!(H_HSD_SObjLib_803A49E0, HSD_SObjLib_803A49E0, (g: *mut HSD_GObj, a: i32) = |g, a| unsafe { crate::tu::sobjlib::HSD_SObjLib_803A49E0(g, a as _) });
    lb_hook!(H_HSD_VIWaitXFBFlush, HSD_VIWaitXFBFlush, () = || unsafe { crate::tu::video::HSD_VIWaitXFBFlush() });
    lb_hook!(H_OSGetTick, OSGetTick, () -> u32);
    lb_hook!(H_OSSetPeriodicAlarm, OSSetPeriodicAlarm, (a: *mut OSAlarm, start: i64, period: i64, h: OSAlarmHandler));
    lb_hook!(H_OSCancelAlarm, OSCancelAlarm, (a: *mut OSAlarm));
}

const FILE: &str = "src/melee/lb/lbmthp.c";

pub use melee_types::{THPDecComp, THPFrameBuffer};

/// `static THPDecComp MoviePlayer` (zero initialised).
pub static mut MoviePlayer: THPDecComp = unsafe { core::mem::zeroed() };

/// `lbl_803BAFE8`: the image descriptor of the movie SObj.
pub static mut lbl_803BAFE8: HSD_ImageDesc = HSD_ImageDesc {
    image_ptr: null_mut(),
    width: 0x280,
    height: 0x1E0,
    format: 6,
    mipmap: 0,
    minLOD: 0.0,
    maxLOD: 0.0,
};
/// `lbl_804D3834`: the SObj descriptor over `lbl_803BAFE8`.
pub static mut lbl_804D3834: HSD_SObjDesc = HSD_SObjDesc { image: null_mut(), tlut: null_mut() };

#[inline]
fn mp() -> &'static mut THPDecComp {
    unsafe { &mut *addr_of_mut!(MoviePlayer) }
}

#[inline]
fn align32(x: u32) -> u32 {
    x.wrapping_add(0x1F) & !0x1F
}

/// `OSSecondsToTicks(1.0f / 60)`: a float product, truncated to `OSTime`.
fn frame_ticks() -> i64 {
    ((1.0f32 / 60.0) * ((OS_BUS_CLOCK / 4) as f32)) as i64
}

/// `next_packed_size` of a frame buffer (big-endian in the file).
unsafe fn packed_size(fb: *const THPFrameBuffer) -> u32 {
    u32::from_be((*fb).next_packed_size)
}

/// `fn_8001E910`: the DevCom completion callback of a frame read; advances the file position,
/// records the read latency, and queues the next frame read when the ring has room.
pub unsafe extern "C" fn fn_8001E910(_arg0: i32, _arg1: usize, _arg2: *mut c_void, cancelflag: i32) {
    let sp = mp();
    if cancelflag != 0 {
        ext::__assert(FILE, 104, "!cancelflag");
    }

    let tick_diff = ext::OSGetTick().wrapping_sub(sp.unk_13C) as i32;
    sp.unk_134 = tick_diff;
    sp.unk_130 = tick_diff >> 0x1F;
    sp.unk_108 = sp.unk_108.wrapping_add(1);
    if sp.unk_74 != 0 {
        sp.curr_file_offset = sp.curr_file_offset.wrapping_add(sp.currPackedSize);
    } else {
        sp.curr_file_offset = sp.first_frame;
    }
    let var_r0 = if sp.unk_8C == 0 { sp.unk_104.wrapping_sub(1) } else { sp.unk_8C.wrapping_sub(1) };
    sp.currPackedSize = packed_size(*sp.frame_buffers.add(var_r0 as usize));
    if sp.unk_90 != sp.unk_8C && sp.unk_70 != 0 {
        let intr = ext::OSDisableInterrupts();
        sp.unk_13C = ext::OSGetTick();
        let mut did_request = 0i32;
        sp.unk_138 = did_request as u32;
        if sp.unk_74 != sp.unk_40 || (sp.unk_74 == sp.unk_40 && did_request != 0) {
            if sp.currPackedSize == 0 {
                ext::OSReport(
                    "filnum = %d, ofs = %d, by sugano.",
                    &[ext::OsArg::I(sp.file_entrynum), ext::OsArg::I(sp.curr_file_offset as i32)],
                );
                ext::__assert(FILE, 131, "(u32)streamPlayer->currPackedSize != 0");
            }
            ext::HSD_DevComRequest(
                sp.file_entrynum,
                sp.curr_file_offset as usize,
                *sp.frame_buffers.add(sp.unk_8C as usize) as usize,
                (sp.currPackedSize.wrapping_add(0x1F) & 0xFFFF_FFE0) as usize,
                0x21,
                1,
                Some(fn_8001E910),
                0,
            );
            sp.unk_74 = sp.unk_74.wrapping_add(1);
            if sp.unk_74 == sp.unk_40 && sp.unk_68 != 0 {
                sp.unk_74 = 0;
            }
            let mut var_r3 = sp.unk_8C.wrapping_add(1);
            if var_r3 >= sp.unk_104 {
                var_r3 = 0;
            }
            sp.unk_8C = var_r3;
            did_request = 1;
            sp.unk_110 = did_request;
        }
        if did_request == 0 {
            sp.unk_110 = 0;
        }
        ext::OSRestoreInterrupts(intr);
        return;
    }
    sp.unk_110 = 0;
}

/// `fn_8001EB14`: reads the 0x40-byte THP header of `path` into `data` and sets up the player.
unsafe fn fn_8001EB14(data: *mut THPDecComp, path: *const c_char) -> i32 {
    ext::THPInit();
    (*data).file_entrynum = ext::DVDConvertPathToEntrynum(path);
    crate::tu::lbfile::lbFile_800161C4((*data).file_entrynum, 0, data as usize, 0x40, 0x21, 1);
    // The header words (version .. first_frame_size) are big-endian in the file.
    (*data).version = u32::from_be((*data).version);
    (*data).buf_size = u32::from_be((*data).buf_size);
    (*data).x_size = u32::from_be((*data).x_size);
    (*data).y_size = u32::from_be((*data).y_size);
    (*data).frame_rate = u32::from_be((*data).frame_rate);
    (*data).num_frames = u32::from_be((*data).num_frames);
    (*data).first_frame = u32::from_be((*data).first_frame);
    (*data).frame_offsets = u32::from_be((*data).frame_offsets);
    (*data).first_frame_size = u32::from_be((*data).first_frame_size);

    (*data).unk_40 = (*data).num_frames;
    (*data).width = (*data).x_size;
    (*data).height = (*data).y_size;
    (*data).unk_100 = (*data).buf_size;

    if (*data).frame_offsets != 0 {
        ext::OSReport("Warning : frame offsets not supported\n", &[]);
    }
    if (*data).version > 2 {
        ext::OSReport("Warning : file format is newer than player\n", &[]);
    }

    (*data).unk_11C = 1;
    (*data).unk_6C = 1;
    (*data).unk_110 = 0;
    (*data).unk_70 = 1;
    (*data).unk_134 = 0;
    (*data).unk_130 = 0;
    1
}

/// `fn_8001EBF0`: sizes the decoder work memory (frame ring, Y/U/V planes, decode area, pointer
/// and rate tables) and resets the player counters. Returns the memory required.
///
/// # Safety
/// `data` must be valid for reads and writes.
pub unsafe fn fn_8001EBF0(data: *mut THPDecComp) -> usize {
    (*data).unk_104 = 0x20;

    let unk_104_val = (*data).unk_104;
    let aligned_100 = align32((*data).unk_100);
    let mut size = aligned_100.wrapping_mul(unk_104_val) as i32;
    let width = (*data).width;
    let height = (*data).height;
    let wh = width.wrapping_mul(height);
    (*data).unk_9C.val1 = width as u16;
    let height = (*data).height;
    (*data).unk_9C._pad = height as u16;
    (*data).unk_9C.val2 = 4;
    (*data).unk_9C.val0 = 0;

    size = size.wrapping_add(wh as i32);
    let wh_div4 = wh >> 2;
    size = size.wrapping_add(wh_div4 as i32);
    size = size.wrapping_add(wh_div4 as i32);

    size = size.wrapping_add(ext::THPDec_8032FD40(&mut (*data).unk_9C, (*data).height as u16));

    (*data).unk_7C = 0;
    (*data).unk_78 = 0;
    (*data).unk_84 = 0;
    (*data).unk_80 = 0;
    (*data).unk_90 = 0;
    (*data).unk_88 = 0;
    (*data).unk_94 = -1;
    (*data).unk_68 = 0;

    (*data).unk_A8 = (*data).width as u16;
    (*data).unk_AA = (*data).height as u16;
    (*data).unk_AC = 0;

    let ptr = core::mem::size_of::<*mut THPFrameBuffer>() as u32;
    size = size.wrapping_add(align32((*data).unk_104.wrapping_mul(ptr)) as i32);
    size = size.wrapping_add(align32((*data).unk_40.wrapping_mul(ptr)) as i32);

    size as usize
}

/// `fn_8001ECF4`: lays out the frame-pointer table, the preloaded frame ring and the YUV planes in
/// `buf`, and preloads the first frames from disc.
unsafe fn fn_8001ECF4(data: *mut THPDecComp, buf: *mut c_void) {
    let width = (*data).width;
    let height = (*data).height;
    let y_size = width.wrapping_mul(height);
    (*data).frame_buffers = buf as *mut *mut THPFrameBuffer;
    let count = (*data).unk_104;
    (*data).unk_64 = 0;
    let uv_size = width.wrapping_mul(height) >> 2;
    let ptr = core::mem::size_of::<*mut THPFrameBuffer>() as u32;
    let mut var_r29 = (buf as *mut u8).add(align32(count.wrapping_mul(ptr)) as usize);
    if (*data).unk_6C != 0 && (*data).unk_11C != 0 {
        let mut var_r24 = (*data).first_frame_size;
        let mut csizep = addr_of_mut!((*data).first_frame_size) as *mut u8;
        let mut var_r25 = 0u32;
        (*data).curr_file_offset = (*data).first_frame;
        while var_r25 < (*data).unk_104 {
            *(*data).frame_buffers.add(var_r25 as usize) = var_r29 as *mut THPFrameBuffer;
            if var_r24 == 0 {
                use ext::OsArg::{I, S, U};
                ext::OSReport("by sugano & yoshiki.\n", &[]);
                ext::OSReport("base %x\n", &[U(var_r29 as usize as u32)]);
                ext::OSReport("size %d\n", &[I(var_r24 as i32)]);
                ext::OSReport("count %d\n", &[I(var_r25 as i32)]);
                ext::OSReport("csizep %x\n", &[U(csizep as usize as u32)]);
                ext::OSReport("[LbMthp] magic = %s\n", &[S(data as *const c_char)]);
                ext::OSReport("[LbMthp] version = %d\n", &[I((*data).version as i32)]);
                ext::OSReport("[LbMthp] bufSize = %d\n", &[I((*data).buf_size as i32)]);
                ext::OSReport("[LbMthp] xSize = %d\n", &[I((*data).x_size as i32)]);
                ext::OSReport("[LbMthp] ySize = %d\n", &[I((*data).y_size as i32)]);
                ext::OSReport("[LbMthp] framerate = %d\n", &[I((*data).frame_rate as i32)]);
                ext::OSReport("[LbMthp] numFrames = %d\n", &[I((*data).num_frames as i32)]);
                ext::OSReport("[LbMthp] firstFrame = %d\n", &[I((*data).first_frame as i32)]);
                ext::OSReport("[LbMthp] frameOffsets = %d\n", &[I((*data).frame_offsets as i32)]);
                ext::OSReport("[LbMthp] firstFrameSize = %d\n", &[I((*data).first_frame_size as i32)]);
                ext::__assert(FILE, 294, "0");
            }
            crate::tu::lbfile::lbFile_800161C4(
                (*data).file_entrynum,
                (*data).curr_file_offset as usize,
                var_r29 as usize,
                (var_r24.wrapping_add(0x1F) & 0xFFFF_FFE0) as usize,
                0x21,
                1,
            );
            csizep = var_r29;
            (*data).curr_file_offset = (*data).curr_file_offset.wrapping_add(var_r24);
            var_r24 = packed_size(var_r29 as *const THPFrameBuffer);
            var_r29 = var_r29.add((*data).unk_100 as usize);
            var_r25 += 1;
        }
        (*data).unk_74 = var_r25;
        (*data).currPackedSize = var_r24;
        if (*data).unk_74 >= (*data).unk_40 {
            (*data).unk_74 = 0;
        }
        (*data).unk_8C = 0;
        let temp = (*data).unk_104.wrapping_sub(1) as i32;
        (*data).unk_108 = temp;
        (*data).unk_10C = temp;
    }
    (*data).unk_50 = var_r29 as *mut c_void;
    ext::DCInvalidateRange(var_r29 as *mut c_void, y_size);
    var_r29 = var_r29.add(y_size as usize);
    (*data).unk_54 = var_r29 as *mut c_void;
    ext::DCInvalidateRange(var_r29 as *mut c_void, uv_size);
    var_r29 = var_r29.add(uv_size as usize);
    (*data).unk_58 = var_r29 as *mut c_void;
    ext::DCInvalidateRange(var_r29 as *mut c_void, uv_size);
    var_r29 = var_r29.add(uv_size as usize);
    (*data).unk_98 = var_r29 as *mut _THPFileInfo;
}

/// `fn_8001EF5C`: decodes the frame at `unk_90` into the YUV planes when it is not already
/// decoded, then tries to queue the next read. Returns `unk_94` (the decoded frame index).
unsafe fn fn_8001EF5C(data: *mut THPDecComp) -> i32 {
    let mut sp_c: i32 = 0;
    if (*data).unk_94 as u32 != (*data).unk_90 {
        let intr = ext::OSDisableInterrupts();
        let fb = *(*data).frame_buffers.add((*data).unk_90 as usize);
        (*data).unk_98 = ext::THPVideoDecode(
            addr_of_mut!((*data).unk_A8) as *mut c_void,
            &mut sp_c as *mut i32 as *mut c_void,
            (*data).unk_98,
            (*fb).data.as_mut_ptr() as *mut c_void,
            &mut (*data).unk_9C,
        );
        ext::OSRestoreInterrupts(intr);

        if (*data).width == 0x280 {
            ext::THPDec_80331340((*data).unk_98, (*data).unk_50, (*data).unk_54, (*data).unk_58);
            ext::DCStoreRange((*data).unk_50, (*data).width.wrapping_mul((*data).height));
            ext::DCStoreRange((*data).unk_54, (*data).width.wrapping_mul((*data).height) >> 2);
            ext::DCStoreRange((*data).unk_58, (*data).width.wrapping_mul((*data).height) >> 2);
        } else {
            ext::THPDec_803313D0((*data).unk_98, (*data).unk_50, (*data).unk_54, (*data).unk_58, (*data).width);
        }

        let intr = ext::OSDisableInterrupts();
        (*data).unk_94 = (*data).unk_90 as i32;
        ext::OSRestoreInterrupts(intr);
        return fn_8001F13C(data);
    }
    (*data).unk_94
}

/// `fn_8001F06C`: advances the decode frame (`unk_78`) and the ring read position (`unk_88`),
/// holding at the last frame when not looping. Always returns 1.
unsafe fn fn_8001F06C(data: *mut THPDecComp) -> i32 {
    let intr = ext::OSDisableInterrupts();

    'end: {
        if (*data).unk_108 > 0 || (*data).unk_6C == 0 {
            (*data).unk_78 = (*data).unk_78.wrapping_add(1);

            if (*data).unk_78 == (*data).unk_40 {
                if (*data).unk_68 != 0 {
                    (*data).unk_78 = 0;
                } else {
                    (*data).unk_78 = (*data).unk_78.wrapping_sub(1);
                    break 'end;
                }
            }

            (*data).unk_88 = (*data).unk_88.wrapping_add(1);
            (*data).unk_108 = (*data).unk_108.wrapping_sub(1);

            if (*data).unk_88 >= (*data).unk_104 {
                (*data).unk_88 = 0;
            }
        }

        if (*data).unk_108 < (*data).unk_10C {
            (*data).unk_10C = (*data).unk_108;
        }
    }

    ext::OSRestoreInterrupts(intr);
    1
}

/// `fn_8001F13C`: queues the next frame read when the ring has a free slot and none is in flight.
/// Returns the result of `OSRestoreInterrupts`.
///
/// # Safety
/// `stream_player` must be valid for reads and writes.
pub unsafe fn fn_8001F13C(stream_player: *mut THPDecComp) -> i32 {
    let sp = &mut *stream_player;
    let intr = ext::OSDisableInterrupts();
    if sp.unk_90 != sp.unk_8C && sp.unk_110 == 0 && sp.unk_70 != 0 {
        sp.unk_13C = ext::OSGetTick();
        sp.unk_138 = 0;
        if sp.unk_74 != sp.unk_40 {
            if sp.currPackedSize == 0 {
                ext::OSReport(
                    "filnum = %d, ofs = %d, by sugano.",
                    &[ext::OsArg::I(sp.file_entrynum), ext::OsArg::I(sp.curr_file_offset as i32)],
                );
                ext::__assert(FILE, 408, "(u32)streamPlayer->currPackedSize != 0");
            }
            ext::HSD_DevComRequest(
                sp.file_entrynum,
                sp.curr_file_offset as usize,
                *sp.frame_buffers.add(sp.unk_8C as usize) as usize,
                align32(sp.currPackedSize) as usize,
                0x21,
                1,
                Some(fn_8001E910),
                0,
            );
            sp.unk_74 = sp.unk_74.wrapping_add(1);
            if sp.unk_74 == sp.unk_40 && sp.unk_68 != 0 {
                sp.unk_74 = 0;
            }
            let mut next = sp.unk_8C.wrapping_add(1);
            if next >= sp.unk_104 {
                next = 0;
            }
            sp.unk_8C = next;
            sp.unk_110 = 1;
        }
    }
    ext::OSRestoreInterrupts(intr)
}

/// `fn_8001F294`: whether a frame read is in flight (`unk_110`).
pub fn fn_8001F294() -> i32 {
    mp().unk_110
}

/// `lbMthp_GetFrame`: the movie frame shown after `counter` display ticks, through the optional
/// table of (frame count, ticks per frame) runs.
unsafe fn lbMthp_GetFrame(rate_table: *mut *mut u32, mut counter: u32) -> u32 {
    use crate::cabi::udiv;
    let mut rate_ptr = *rate_table;
    let mut frame = 0u32;
    if !rate_ptr.is_null() {
        loop {
            let count = *rate_ptr;
            let total = count.wrapping_mul(*rate_ptr.add(1));
            if counter >= total {
                frame = frame.wrapping_add(count);
                counter = counter.wrapping_sub(total);
                rate_ptr = rate_ptr.add(2);
                continue;
            } else {
                frame = frame.wrapping_add(udiv(counter, *rate_ptr.add(1)));
                break;
            }
        }
    } else {
        frame = counter;
    }
    frame
}

/// `fn_8001F2A4`: the 60 Hz alarm handler; advances the display counter `unk_80` through the rate
/// table, stops at the end (or wraps when looping), and steps the decoder when the frame changes.
pub unsafe extern "C" fn fn_8001F2A4(_alarm: *mut OSAlarm, _context: *mut OSContext) {
    let sp = addr_of_mut!(MoviePlayer);
    let rate_table = addr_of_mut!((*sp).rate_table);

    let mut frame = lbMthp_GetFrame(rate_table, (*sp).unk_80);

    if (*sp).unk_78 == frame {
        (*sp).unk_80 = (*sp).unk_80.wrapping_add(1);

        frame = lbMthp_GetFrame(rate_table, (*sp).unk_80);

        if (*sp).unk_40 == frame {
            if (*sp).unk_68 != 0 {
                (*sp).unk_80 = 0;
            } else {
                (*sp).unk_80 = (*sp).unk_80.wrapping_sub(1);
                (*sp).unk_144 = 1;
            }
        }
    }

    frame = lbMthp_GetFrame(rate_table, (*sp).unk_80);

    if (*sp).unk_78 != frame {
        fn_8001F06C(sp);
    }
}

/// `lbMthp_8001F410(filename, rate_table, buf, heap_size, loop)`: starts the movie `filename`
/// (decoding into `buf`, or into a fresh allocation when `buf` is null) and arms the 60 Hz alarm.
/// A movie missing from the disc is reported finished instead (PC behaviour).
///
/// # Safety
/// `filename` must be a NUL-terminated string; `buf`, when non-null, must hold `heap_size` bytes.
pub unsafe fn lbMthp_8001F410(filename: *const c_char, rate_table: *mut u32, mut buf: *mut c_void, heap_size: usize, loop_: i32) {
    let sp = addr_of_mut!(MoviePlayer);

    if (*sp).power != 0 {
        ext::__assert(FILE, 528, "!MoviePlayer.power");
    }
    // TARGET_PC: a movie that is not on this disc is skipped (the title-screen demo would crash).
    if ext::DVDConvertPathToEntrynum(filename) < 0 {
        ext::OSReport("lbMthp: %s is not on this disc - skipping the movie\n", &[ext::OsArg::S(filename)]);
        (*sp).unk_94 = (*sp).unk_90 as i32;
        (*sp).unk_148 = 0;
        (*sp).unk_144 = 1;
        return;
    }
    (*sp).power = 1;
    fn_8001EB14(sp, filename);
    (*sp).rate_table = rate_table;
    let memory_required = fn_8001EBF0(sp);
    if !buf.is_null() {
        if heap_size < memory_required {
            ext::__assert(FILE, 549, "heap_size >= memoryRequired");
        }
        (*sp).unk_140 = null_mut();
    } else {
        buf = ext::HSD_MemAlloc(memory_required as i32);
        (*sp).unk_140 = buf;
    }
    (*sp).unk_68 = loop_;
    fn_8001ECF4(sp, buf);
    (*sp).unk_144 = 0;
    (*sp).unk_148 = 1;
    ext::OSCreateAlarm(addr_of_mut!((*sp).alarm));
    ext::OSSetPeriodicAlarm(addr_of_mut!((*sp).alarm), frame_ticks(), frame_ticks(), Some(fn_8001F2A4));
}

/// `lbMthp_8001F578`: publishes the decode position to the render side (`unk_90/7C/84`).
pub fn lbMthp_8001F578() {
    let intr = ext::OSDisableInterrupts();
    let sp = mp();
    sp.unk_90 = sp.unk_88;
    sp.unk_7C = sp.unk_78;
    sp.unk_84 = sp.unk_80;
    ext::OSRestoreInterrupts(intr);
}

/// `lbMthp_8001F5C4`: the published display counter (`unk_84`).
pub fn lbMthp_8001F5C4() -> i32 {
    mp().unk_84 as i32
}

/// `lbMthp_8001F5D4`: the last read latency in ticks (`unk_134`).
pub fn lbMthp_8001F5D4() -> u32 {
    mp().unk_134 as u32
}

/// `lbMthp_8001F5E4`: the number of buffered frames (`unk_108`).
pub fn lbMthp_8001F5E4() -> i32 {
    mp().unk_108
}

/// `lbMthp_8001F5F4`: the buffered-frame low-water mark (`unk_10C`).
pub fn lbMthp_8001F5F4() -> i32 {
    mp().unk_10C
}

/// `lbMthp_8001F604`: nonzero once the movie has finished (`unk_144`).
pub fn lbMthp_8001F604() -> i32 {
    mp().unk_144
}

/// `lbMthp_8001F614`: shows (nonzero) or hides the movie (`unk_148`).
pub fn lbMthp_8001F614(arg0: i32) {
    mp().unk_148 = arg0;
}

/// `lbMthp_8001F624(gobj, width, height)`: creates the movie SObj over `lbl_803BAFE8` resized to
/// `width` x `height`, with the 0x10 flag set.
///
/// # Safety
/// `gobj` must be valid; the SObj library must return a valid SObj.
pub unsafe fn lbMthp_8001F624(gobj: *mut HSD_GObj, width: i32, height: i32) -> *mut HSD_SObj {
    lbl_803BAFE8.image_ptr = null_mut();
    lbl_803BAFE8.width = width as u16;
    lbl_803BAFE8.height = height as u16;
    lbl_804D3834.image = addr_of_mut!(lbl_803BAFE8);
    let sobj = ext::HSD_SObjLib_803A477C(gobj, addr_of_mut!(lbl_804D3834), 0, 0, 0x80, 0);
    (*sobj).x40 |= 0x10;
    sobj
}

/// `lbMthp_8001F67C(gobj, arg1)`: decodes the current frame and, when shown, binds the Y/U/V
/// planes as three I8 textures (map 0 at full size, maps 1 and 2 at half size) and draws the SObj.
///
/// # Safety
/// `gobj` must be the movie GObj.
pub unsafe fn lbMthp_8001F67C(gobj: *mut HSD_GObj, arg1: i32) {
    let sp = addr_of_mut!(MoviePlayer);

    fn_8001EF5C(sp);
    if (*sp).unk_148 != 0 {
        let (w, h) = ((*sp).width, (*sp).height);
        ext::GXInitTexObj(addr_of_mut!((*sp).unk_178), (*sp).unk_50, w as u16, h as u16, GX_TF_I8 as i32, GX_CLAMP as i32, GX_CLAMP as i32, 0);
        ext::GXInitTexObjLOD(addr_of_mut!((*sp).unk_178), GX_NEAR as i32, GX_NEAR as i32, 0.0, 0.0, 0.0, 0, 0, GX_ANISO_1 as i32);
        ext::GXLoadTexObj(addr_of_mut!((*sp).unk_178), GX_TEXMAP0 as i32);

        ext::GXInitTexObj(addr_of_mut!((*sp).unk_198), (*sp).unk_54, (w / 2) as u16, (h / 2) as u16, GX_TF_I8 as i32, GX_CLAMP as i32, GX_CLAMP as i32, 0);
        ext::GXInitTexObjLOD(addr_of_mut!((*sp).unk_198), GX_NEAR as i32, GX_NEAR as i32, 0.0, 0.0, 0.0, 0, 0, GX_ANISO_1 as i32);
        ext::GXLoadTexObj(addr_of_mut!((*sp).unk_198), GX_TEXMAP1 as i32);

        ext::GXInitTexObj(addr_of_mut!((*sp).unk_1B8), (*sp).unk_58, (w / 2) as u16, (h / 2) as u16, GX_TF_I8 as i32, GX_CLAMP as i32, GX_CLAMP as i32, 0);
        ext::GXInitTexObjLOD(addr_of_mut!((*sp).unk_1B8), GX_NEAR as i32, GX_NEAR as i32, 0.0, 0.0, 0.0, 0, 0, GX_ANISO_1 as i32);
        ext::GXLoadTexObj(addr_of_mut!((*sp).unk_1B8), GX_TEXMAP2 as i32);

        ext::HSD_SObjLib_803A49E0(gobj, arg1);
    }
}

/// `lbMthp_8001F800`: stops the movie: waits for the in-flight read, cancels the alarm, waits for
/// the XFB flush and frees the decode memory.
pub fn lbMthp_8001F800() {
    let sp = mp();
    if sp.power != 0 {
        sp.unk_70 = 0;

        while fn_8001F294() != 0 {}

        ext::OSCancelAlarm(addr_of_mut!(sp.alarm));
        ext::HSD_VIWaitXFBFlush();

        if !sp.unk_140.is_null() {
            ext::HSD_Free(sp.unk_140);
        }

        sp.power = 0;
    }
}

/// `lbMthp_8001F87C`: marks the player stopped without cleanup.
pub fn lbMthp_8001F87C() {
    mp().power = 0;
}

/// `MoviePlayer.power` (test access to the player state the stop calls clear).
#[doc(hidden)]
pub fn test_power() -> i32 {
    mp().power as i32
}
#[doc(hidden)]
pub fn test_set_power(v: i32) {
    mp().power = v as _;
}
