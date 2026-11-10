//! Native port of `src/sysdolphin/baselib/card.c` (all definitions, including inlines).
//! Save headers remain byte-addressed and explicitly serialized; process pointers are native.
//! Async SDK callbacks must be deferred until the SDK call returns, as in card.h.
//! No floating-point arithmetic or fused sites occur in this translation unit.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments,
         unused_variables, unused_unsafe, unused_parens, static_mut_refs, dead_code)]
use core::ffi::c_void;
use melee_types::*;
use ext::*;

const running: i32 = 0;
const running_async: i32 = 1;
const idle: i32 = 2;
const BANNER_SMALL: i32 = 0xE00;
const BANNER_LARGE: i32 = 0x1800;
const CARD_STAT_ICON_MASK: i32 = 3;
const CARD_STAT_SPEED_MASK: i32 = 3;

trait CardArithmetic: Sized { fn card_div(self, d: Self) -> Self; fn card_rem(self, d: Self) -> Self; }
impl CardArithmetic for u32 {
    fn card_div(self, d: Self) -> Self { crate::cabi::udiv(self, d) }
    fn card_rem(self, d: Self) -> Self { crate::cabi::urem(self, d) }
}
impl CardArithmetic for i32 {
    fn card_div(self, d: Self) -> Self { crate::cabi::sdiv(self, d) }
    fn card_rem(self, d: Self) -> Self { crate::cabi::srem(self, d) }
}

unsafe fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    if n != 0 { core::ptr::copy_nonoverlapping(src.cast::<u8>(), dst.cast::<u8>(), n); }
    dst
}
unsafe fn memset(dst: *mut c_void, value: i32, n: usize) -> *mut c_void {
    if n != 0 { core::ptr::write_bytes(dst.cast::<u8>(), value as u8, n); }
    dst
}
unsafe fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32 {
    for i in 0..n {
        let x = *a.cast::<u8>().add(i); let y = *b.cast::<u8>().add(i);
        if x != y { return x as i32 - y as i32; }
    }
    0
}
// The already oracle-checked crypt.c ports supply these ABI adapters.
unsafe fn HSD_Checksum(src: *mut u8, len: i32, dest: *mut c_void) {
    let input = if len > 0 { core::slice::from_raw_parts(src, len as usize) } else { &[] };
    let digest = crate::hsd_util::checksum(input);
    core::ptr::copy_nonoverlapping(digest.as_ptr(), dest.cast::<u8>(), 16);
}
unsafe fn HSD_Encrypt(data: *mut u8, len: i32) -> i32 {
    crate::hsd_util::encrypt(core::slice::from_raw_parts_mut(data, len.max(0) as usize))
}
unsafe fn HSD_Decrypt(data: *mut u8, len: i32) -> i32 {
    crate::hsd_util::decrypt(core::slice::from_raw_parts_mut(data, len.max(0) as usize))
}

/// Platform CARD/OS integration points. Install callbacks before using this module.
/// TODO: connect these exact SDK symbols to the native platform crate.
pub mod ext {
    use super::*;
    macro_rules! hook {
        ($name:ident, $slot:ident, ($($arg:ident: $ty:ty),*) -> $out:ty) => {
            pub static mut $slot: Option<unsafe extern "C" fn($($ty),*) -> $out> = None;
            pub unsafe fn $name($($arg: $ty),*) -> $out {
                match $slot { Some(f) => f($($arg),*), None => todo!(stringify!($name)) }
            }
        };
    }
    hook!(OSDisableInterrupts, OSDisableInterrupts_hook, () -> i32);
    hook!(OSRestoreInterrupts, OSRestoreInterrupts_hook, (level: i32) -> i32);
    hook!(CARDCreateAsync, CARDCreateAsync_hook, (chan: i32, filename: *const i8, size: u32, info: *mut CARDFileInfo, callback: CARDCallback) -> i32);
    hook!(CARDFastOpen, CARDFastOpen_hook, (chan: i32, no: i32, info: *mut CARDFileInfo) -> i32);
    hook!(CARDOpen, CARDOpen_hook, (chan: i32, filename: *const i8, info: *mut CARDFileInfo) -> i32);
    hook!(CARDClose, CARDClose_hook, (info: *mut CARDFileInfo) -> i32);
    hook!(CARDGetXferredBytes, CARDGetXferredBytes_hook, (chan: i32) -> i32);
    hook!(CARDReadAsync, CARDReadAsync_hook, (info: *mut CARDFileInfo, buf: *mut c_void, len: i32, offset: i32, callback: CARDCallback) -> i32);
    hook!(CARDRead, CARDRead_hook, (info: *mut CARDFileInfo, buf: *mut c_void, len: i32, offset: i32) -> i32);
    hook!(CARDGetStatus, CARDGetStatus_hook, (chan: i32, no: i32, stat: *mut CARDStat) -> i32);
    hook!(CARDSetStatusAsync, CARDSetStatusAsync_hook, (chan: i32, no: i32, stat: *mut CARDStat, callback: CARDCallback) -> i32);
    hook!(CARDWriteAsync, CARDWriteAsync_hook, (info: *mut CARDFileInfo, buf: *mut c_void, len: i32, offset: i32, callback: CARDCallback) -> i32);
    hook!(CARDWrite, CARDWrite_hook, (info: *mut CARDFileInfo, buf: *mut c_void, len: i32, offset: i32) -> i32);
}

pub static mut hsd_804D79C8: u8 = 0;
pub static mut hsd_804D79C4: s32 = 0;
pub static mut hsd_804D79C0: s32 = 0;
pub static mut hsd_804D79BC: *mut u8 = ::core::ptr::null::<u8>() as *mut u8;
pub static mut hsd_804D79B8: *mut u8 = ::core::ptr::null::<u8>() as *mut u8;
pub static mut hsd_804D79B0: [u8; 8] = [0; 8];
pub static mut hsd_804D79AC: s32 = 0;
pub static mut hsd_804D79A8: s32 = 0;
pub static mut hsd_804D79A4: *mut u8 = ::core::ptr::null::<u8>() as *mut u8;
pub static mut hsd_804D79A0: *mut u8 = ::core::ptr::null::<u8>() as *mut u8;
pub static mut curr_head: s32 = 0;
pub static mut curr_tail: s32 = 0;
pub static mut curr_result: s32 = 0;
pub static mut hsd_804D798C: s32 = 0;
pub static mut hsd_804D7990: s32 = 0;
pub static mut hsd_804D7994: s32 = 0;
pub static mut hsd_804D7998: i32 = 0;
pub static mut hsd_804D799C: i32 = running;
pub static mut active_requests: CardActiveRequest = CardActiveRequest {
    r#type: CARD_ACTIVE_NONE,
    state: ::core::ptr::null::<CardState>() as *mut CardState,
    callback: None,
    callback_arg: 0,
};
pub static mut commands: [CardCmd; 128] = [CardCmd {
    r#type: CARD_CMD_NONE,
    state: ::core::ptr::null::<CardState>() as *mut CardState,
    x8: CardCmd_x8 {
        write: CardWriteArgs {
            file_idx: 0,
            phys: 0,
            block_id: 0,
            seq: 0,
            data: ::core::ptr::null::<c_void>() as *mut c_void,
            offset: 0,
            size: 0,
        },
    },
}; 128];
pub static mut requests: [CardRequest; 32] = [CardRequest {
    r#type: CARD_REQ_NONE,
    state: ::core::ptr::null::<CardState>() as *mut CardState,
    x8: CardRequest_x8 {
        file: CardFileReqArgs {
            file_idx: 0,
            buf: ::core::ptr::null::<u8>() as *mut u8,
        },
    },
    callback: None,
}; 32];
pub unsafe extern "C" fn checkOpen(mut state: *mut CardState) -> i32 {
    melee_core::ct!("checkOpen");
    let mut i: i32 = 0;
    let mut result: i32 = 0;
    i = 0 as i32;
    while i < 10 as i32 {
        result = CARDClose(&raw mut (*state).file_info) as i32;
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
pub unsafe extern "C" fn hsd_803A949C(mut chan: s32, mut card_result: s32) {
    melee_core::ct!("hsd_803A949C");
    let mut state: *mut CardState = ::core::ptr::null_mut::<CardState>();
    let mut block: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut result: s32 = 0;
    let mut banner_size: s32 = 0;
    let mut remaining: s32 = 0;
    let mut banner_size11: s32 = 0;
    let mut hdr_offset: s32 = 0;
    let mut hdr_offset13: s32 = 0;
    let mut icons_offset: s32 = 0;
    let mut icons_start: s32 = 0;
    let mut phys: s32 = 0;
    let mut i: s32 = 0;
    if hsd_804D799C as i32 != 1 as i32 {
        return;
    }
    state = commands[curr_head as usize].state;
    let mut current_block_238: u64;
    match commands[curr_head as usize].r#type as i32 {
        2 => {
            if card_result != 0 as i32 {
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else if commands[curr_head as usize].x8.read.phys
                < 0 as i32
            {
                if checkOpen(state) < 0 as i32 {
                    ::core::ptr::write_volatile(
                        &mut curr_result as *mut s32,
                        card_result,
                    );
                }
            } else {
                if commands[curr_head as usize].x8.read.phys
                    == 0 as i32
                {
                    hdr_offset = (*state)
                        .header_size
                        .wrapping_add(0x30 as u32)
                        .card_rem((*state).sector_size) as s32;
                    if commands[curr_head as usize].x8.read.size
                        > 0 as i32
                    {
                        if HSD_Decrypt(
                            (*state).sector_buf.offset(hdr_offset as isize),
                            (*state).sector_size.wrapping_sub(hdr_offset as u32)
                                as i32,
                        ) < 0 as i32
                        {
                            checkOpen(state);
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                -(0x105 as i32) as s32,
                            );
                            current_block_238 = 13475623910364476264;
                        } else {
                            if !commands[curr_head as usize]
                                .x8
                                .read
                                .data
                                .is_null()
                            {
                                let mut src: *mut CardBlockHeader = (*state).sector_buf.offset(hdr_offset as isize).cast::<CardBlockHeader>();
                                memcpy(
                                    commands[curr_head as usize].x8.read.data,
                                    &raw mut (*src).data as *mut u8
                                        as *const c_void,
                                    commands[curr_head as usize].x8.read.size
                                        as usize,
                                );
                            }
                            current_block_238 = 1109700713171191020;
                        }
                    } else {
                        current_block_238 = 1109700713171191020;
                    }
                    match current_block_238 {
                        13475623910364476264 => {}
                        _ => {
                            result = checkOpen(state) as s32;
                            current_block_238 = 17788412896529399552;
                        }
                    }
                } else if HSD_Decrypt(
                    (*state).sector_buf,
                    (*state).sector_size as i32,
                ) < 0 as i32
                {
                    checkOpen(state);
                    ::core::ptr::write_volatile(
                        &mut curr_result as *mut s32,
                        -(0x105 as i32) as s32,
                    );
                    current_block_238 = 13475623910364476264;
                } else {
                    if commands[curr_head as usize].x8.read.size
                        > 0 as i32
                        && !commands[curr_head as usize]
                            .x8
                            .read
                            .data
                            .is_null()
                    {
                        memcpy(
                            commands[curr_head as usize].x8.read.data,
                            &raw mut (*((*state).sector_buf as *mut CardBlockHeader))
                                .data as *mut u8 as *const c_void,
                            commands[curr_head as usize].x8.read.size
                                as usize,
                        );
                    }
                    result = checkOpen(state) as s32;
                    current_block_238 = 17788412896529399552;
                }
                match current_block_238 {
                    13475623910364476264 => {}
                    _ => {
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                        }
                    }
                }
            }
        }
        15 => {
            if card_result != 0 as i32 {
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else if HSD_Decrypt(
                (*state).sector_buf,
                (*state).sector_size as i32,
            ) < 0 as i32
            {
                checkOpen(state);
                ::core::ptr::write_volatile(
                    &mut curr_result as *mut s32,
                    -(0x105 as i32) as s32,
                );
            } else {
                result = checkOpen(state) as s32;
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                }
            }
        }
        5 => {
            if card_result != 0 as i32 {
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else if commands[curr_head as usize].x8.verify.block_id
                == 0 as i32
            {
                if commands[curr_head as usize].x8.verify.size
                    <= 0 as i32
                {
                    result = checkOpen(state) as s32;
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    }
                } else {
                    hdr_offset = (*state)
                        .header_size
                        .wrapping_add(0x30 as u32)
                        .card_rem((*state).sector_size) as s32;
                    if HSD_Decrypt(
                        (*state).sector_buf.offset(hdr_offset as isize),
                        (*state).sector_size.wrapping_sub(hdr_offset as u32)
                            as i32,
                    ) < 0 as i32
                    {
                        checkOpen(state);
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            2 as i32 as s32,
                        );
                    } else {
                        result = checkOpen(state) as s32;
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                        } else {
                            result = curr_head;
                            block = (*state).sector_buf.offset(hdr_offset as isize);
                            result = commands[result as usize]
                                .x8
                                .verify
                                .block_id;
                            if (*block.offset(0x10 as i32 as isize)
                                as i32) << 8 as i32
                                | *block.offset(0x11 as i32 as isize)
                                    as i32 != result
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    2 as i32 as s32,
                                );
                            } else if *block.offset(0x12 as i32 as isize)
                                as s32
                                != commands[curr_head as usize].x8.verify.seq
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    2 as i32 as s32,
                                );
                            } else if commands[curr_head as usize]
                                .x8
                                .verify
                                .size > 0 as i32
                                && {
                                    block = block.offset(0x20 as i32 as isize);
                                    memcmp(
                                        commands[curr_head as usize].x8.verify.data,
                                        block as *const c_void,
                                        commands[curr_head as usize].x8.verify.size
                                            as usize,
                                    ) != 0 as i32
                                }
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    2 as i32 as s32,
                                );
                            }
                        }
                    }
                }
            } else {
                let mut header: *mut CardBlockHeader = ::core::ptr::null_mut::<
                    CardBlockHeader,
                >();
                if HSD_Decrypt(
                    (*state).sector_buf,
                    (*state).sector_size as i32,
                ) < 0 as i32
                {
                    checkOpen(state);
                    ::core::ptr::write_volatile(
                        &mut curr_result as *mut s32,
                        2 as i32 as s32,
                    );
                } else {
                    result = checkOpen(state) as s32;
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        result = (((*((*state).sector_buf as *mut CardBlockHeader)).id_hi
                            as i32) << 8 as i32
                            | (*((*state).sector_buf as *mut CardBlockHeader)).id_lo
                                as i32) as s32;
                        header = (*state).sector_buf as *mut CardBlockHeader;
                        if result
                            != commands[curr_head as usize]
                                .x8
                                .verify
                                .block_id
                        {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                2 as i32 as s32,
                            );
                        } else if (*header).seq as s32
                            != commands[curr_head as usize].x8.verify.seq
                        {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                2 as i32 as s32,
                            );
                        } else if commands[curr_head as usize].x8.verify.size
                            > 0 as i32
                            && memcmp(
                                commands[curr_head as usize].x8.verify.data,
                                &raw mut (*header).data as *mut u8
                                    as *const c_void,
                                commands[curr_head as usize].x8.verify.size
                                    as usize,
                            ) != 0 as i32
                        {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                2 as i32 as s32,
                            );
                        }
                    }
                }
            }
        }
        10 => {
            if card_result != 0 as i32 {
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else {
                result = checkOpen(state) as s32;
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                } else {
                    match (*state).icon_info.banner_format as i32 {
                        2 => {
                            banner_size = BANNER_LARGE as s32;
                        }
                        1 => {
                            banner_size = BANNER_SMALL as s32;
                        }
                        _ => {
                            banner_size = 0 as i32 as s32;
                        }
                    }
                    if commands[curr_head as usize].x8.header.index
                        == 0 as i32
                    {
                        if memcmp(
                            (*state).sector_buf as *const c_void,
                            &raw mut (*state).comment as *mut i8
                                as *const c_void,
                            0x40 as usize,
                        ) != 0 as i32
                        {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                2 as i32 as s32,
                            );
                        } else if banner_size > 0 as i32
                            && memcmp(
                                (*state)
                                    .sector_buf
                                    .offset(0x40 as i32 as isize)
                                    as *const c_void,
                                commands[curr_head as usize].x8.header.banner,
                                banner_size as usize,
                            ) != 0 as i32
                        {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                2 as i32 as s32,
                            );
                        } else {
                            icons_start = (banner_size as i32
                                + 0x40 as i32) as s32;
                            if (*state).header_size > (*state).sector_size {
                                if memcmp(
                                    (*state).sector_buf.offset(icons_start as isize)
                                        as *const c_void,
                                    commands[curr_head as usize].x8.header.icons,
                                    (*state).sector_size.wrapping_sub(icons_start as u32)
                                        as usize,
                                ) != 0 as i32
                                {
                                    ::core::ptr::write_volatile(
                                        &mut curr_result as *mut s32,
                                        2 as i32 as s32,
                                    );
                                } else {
                                    HSD_Checksum(
                                        (*commands[curr_head as usize].state).sector_buf,
                                        (*commands[curr_head as usize].state).sector_size
                                            as i32,
                                        (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                            .offset(curr_head as isize))
                                            .state)
                                            .digest as *mut u8)
                                            .offset(
                                                ((*(&raw mut commands as *mut CardCmd)
                                                    .offset(curr_head as isize))
                                                    .x8
                                                    .header
                                                    .index as i32 * 0x10 as i32)
                                                    as isize,
                                            ) as *mut u8 as *mut c_void,
                                    );
                                }
                            } else if memcmp(
                                (*state).sector_buf.offset(icons_start as isize)
                                    as *const c_void,
                                commands[curr_head as usize].x8.header.icons,
                                (*state).header_size.wrapping_sub(icons_start as u32)
                                    as usize,
                            ) != 0 as i32
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    2 as i32 as s32,
                                );
                            } else {
                                HSD_Checksum(
                                    (*commands[curr_head as usize].state).sector_buf,
                                    (*commands[curr_head as usize].state).header_size
                                        as i32,
                                    (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                        .offset(curr_head as isize))
                                        .state)
                                        .digest as *mut u8)
                                        .offset(
                                            ((*(&raw mut commands as *mut CardCmd)
                                                .offset(curr_head as isize))
                                                .x8
                                                .header
                                                .index as i32 * 0x10 as i32)
                                                as isize,
                                        ) as *mut u8 as *mut c_void,
                                );
                                if memcmp(
                                    (*state).sector_buf.offset((*state).header_size as isize)
                                        as *const c_void,
                                    &raw mut (*(*(&raw mut commands as *mut CardCmd)
                                        .offset(curr_head as isize))
                                        .state)
                                        .digest as *mut u8 as *const c_void,
                                    0x30 as usize,
                                ) != 0 as i32
                                {
                                    ::core::ptr::write_volatile(
                                        &mut curr_result as *mut s32,
                                        2 as i32 as s32,
                                    );
                                }
                            }
                        }
                    } else {
                        icons_offset = (*state)
                            .sector_size
                            .wrapping_mul(
                                commands[curr_head as usize].x8.header.index
                                    as u32,
                            )
                            .wrapping_sub(0x40 as u32)
                            .wrapping_sub(banner_size as u32) as s32;
                        remaining = (*state)
                            .header_size
                            .wrapping_sub(
                                (*state)
                                    .sector_size
                                    .wrapping_mul(
                                        commands[curr_head as usize].x8.header.index
                                            as u32,
                                    ),
                            ) as s32;
                        if remaining as u32 > (*state).sector_size {
                            if memcmp(
                                (*state).sector_buf as *const c_void,
                                (commands[curr_head as usize].x8.header.icons
                                    as *mut u8)
                                    .offset(icons_offset as isize)
                                    as *const c_void,
                                (*state).sector_size as usize,
                            ) != 0 as i32
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    2 as i32 as s32,
                                );
                            } else {
                                HSD_Checksum(
                                    (*commands[curr_head as usize].state).sector_buf,
                                    (*commands[curr_head as usize].state).sector_size
                                        as i32,
                                    (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                        .offset(curr_head as isize))
                                        .state)
                                        .digest as *mut u8)
                                        .offset(
                                            ((*(&raw mut commands as *mut CardCmd)
                                                .offset(curr_head as isize))
                                                .x8
                                                .header
                                                .index as i32 * 0x10 as i32)
                                                as isize,
                                        ) as *mut u8 as *mut c_void,
                                );
                            }
                        } else if memcmp(
                            (*state).sector_buf as *const c_void,
                            (commands[curr_head as usize].x8.header.icons
                                as *mut u8)
                                .offset(icons_offset as isize)
                                as *const c_void,
                            remaining as usize,
                        ) != 0 as i32
                        {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                2 as i32 as s32,
                            );
                        } else {
                            HSD_Checksum(
                                (*commands[curr_head as usize].state).sector_buf,
                                remaining as i32,
                                (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8)
                                    .offset(
                                        ((*(&raw mut commands as *mut CardCmd)
                                            .offset(curr_head as isize))
                                            .x8
                                            .header
                                            .index as i32 * 0x10 as i32)
                                            as isize,
                                    ) as *mut u8 as *mut c_void,
                            );
                            if memcmp(
                                (*state).sector_buf.offset(remaining as isize)
                                    as *const c_void,
                                &raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8 as *const c_void,
                                0x30 as usize,
                            ) != 0 as i32
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    2 as i32 as s32,
                                );
                            }
                        }
                    }
                }
            }
        }
        11 => {
            if card_result != 0 as i32 {
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else {
                result = checkOpen(state) as s32;
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                } else {
                    match (*state).icon_info.banner_format as i32 {
                        2 => {
                            banner_size11 = BANNER_LARGE as s32;
                        }
                        1 => {
                            banner_size11 = BANNER_SMALL as s32;
                        }
                        _ => {
                            banner_size11 = 0 as i32 as s32;
                        }
                    }
                    if commands[curr_head as usize].x8.read_header.index
                        == 0 as i32
                    {
                        if !commands[curr_head as usize]
                            .x8
                            .read_header
                            .comment
                            .is_null()
                        {
                            memcpy(
                                commands[curr_head as usize]
                                    .x8
                                    .read_header
                                    .comment,
                                (*state).sector_buf as *const c_void,
                                0x40 as usize,
                            );
                        }
                        if banner_size11 > 0 as i32
                            && !commands[curr_head as usize]
                                .x8
                                .read_header
                                .banner
                                .is_null()
                        {
                            memcpy(
                                commands[curr_head as usize]
                                    .x8
                                    .read_header
                                    .banner,
                                (*state)
                                    .sector_buf
                                    .offset(0x40 as i32 as isize)
                                    as *const c_void,
                                banner_size11 as usize,
                            );
                        }
                        icons_start = (banner_size11 as i32
                            + 0x40 as i32) as s32;
                        if (*state).header_size > (*state).sector_size {
                            if !commands[curr_head as usize]
                                .x8
                                .read_header
                                .icons
                                .is_null()
                            {
                                memcpy(
                                    commands[curr_head as usize]
                                        .x8
                                        .read_header
                                        .icons,
                                    (*state).sector_buf.offset(icons_start as isize)
                                        as *const c_void,
                                    (*state).sector_size.wrapping_sub(icons_start as u32)
                                        as usize,
                                );
                            }
                            HSD_Checksum(
                                (*commands[curr_head as usize].state).sector_buf,
                                (*commands[curr_head as usize].state).sector_size
                                    as i32,
                                (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8)
                                    .offset(
                                        ((*(&raw mut commands as *mut CardCmd)
                                            .offset(curr_head as isize))
                                            .x8
                                            .read_header
                                            .index as i32 * 0x10 as i32)
                                            as isize,
                                    ) as *mut u8 as *mut c_void,
                            );
                        } else {
                            if !commands[curr_head as usize]
                                .x8
                                .read_header
                                .icons
                                .is_null()
                            {
                                memcpy(
                                    commands[curr_head as usize]
                                        .x8
                                        .read_header
                                        .icons,
                                    (*state).sector_buf.offset(icons_start as isize)
                                        as *const c_void,
                                    (*state).header_size.wrapping_sub(icons_start as u32)
                                        as usize,
                                );
                            }
                            HSD_Checksum(
                                (*commands[curr_head as usize].state).sector_buf,
                                (*commands[curr_head as usize].state).header_size
                                    as i32,
                                (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8)
                                    .offset(
                                        ((*(&raw mut commands as *mut CardCmd)
                                            .offset(curr_head as isize))
                                            .x8
                                            .read_header
                                            .index as i32 * 0x10 as i32)
                                            as isize,
                                    ) as *mut u8 as *mut c_void,
                            );
                            if memcmp(
                                (*state).sector_buf.offset((*state).header_size as isize)
                                    as *const c_void,
                                &raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8 as *const c_void,
                                0x30 as usize,
                            ) != 0 as i32
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    -(0x107 as i32) as s32,
                                );
                            }
                        }
                    } else {
                        icons_offset = (*state)
                            .sector_size
                            .wrapping_mul(
                                commands[curr_head as usize]
                                    .x8
                                    .read_header
                                    .index as u32,
                            )
                            .wrapping_sub(0x40 as u32)
                            .wrapping_sub(banner_size11 as u32) as s32;
                        chan = (*state)
                            .header_size
                            .wrapping_sub(
                                (*state)
                                    .sector_size
                                    .wrapping_mul(
                                        commands[curr_head as usize]
                                            .x8
                                            .read_header
                                            .index as u32,
                                    ),
                            ) as s32;
                        if chan as u32 > (*state).sector_size {
                            if !commands[curr_head as usize]
                                .x8
                                .read_header
                                .icons
                                .is_null()
                            {
                                memcpy(
                                    (commands[curr_head as usize]
                                        .x8
                                        .read_header
                                        .icons as *mut u8)
                                        .offset(icons_offset as isize) as *mut c_void,
                                    (*state).sector_buf as *const c_void,
                                    (*state).sector_size as usize,
                                );
                            }
                            HSD_Checksum(
                                (*commands[curr_head as usize].state).sector_buf,
                                (*commands[curr_head as usize].state).sector_size
                                    as i32,
                                (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8)
                                    .offset(
                                        ((*(&raw mut commands as *mut CardCmd)
                                            .offset(curr_head as isize))
                                            .x8
                                            .read_header
                                            .index as i32 * 0x10 as i32)
                                            as isize,
                                    ) as *mut u8 as *mut c_void,
                            );
                        } else {
                            if !commands[curr_head as usize]
                                .x8
                                .read_header
                                .icons
                                .is_null()
                            {
                                memcpy(
                                    (commands[curr_head as usize]
                                        .x8
                                        .read_header
                                        .icons as *mut u8)
                                        .offset(icons_offset as isize) as *mut c_void,
                                    (*state).sector_buf as *const c_void,
                                    chan as usize,
                                );
                            }
                            HSD_Checksum(
                                (*commands[curr_head as usize].state).sector_buf,
                                chan as i32,
                                (&raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8)
                                    .offset(
                                        ((*(&raw mut commands as *mut CardCmd)
                                            .offset(curr_head as isize))
                                            .x8
                                            .read_header
                                            .index as i32 * 0x10 as i32)
                                            as isize,
                                    ) as *mut u8 as *mut c_void,
                            );
                            if memcmp(
                                (*state).sector_buf.offset(chan as isize)
                                    as *const c_void,
                                &raw mut (*(*(&raw mut commands as *mut CardCmd)
                                    .offset(curr_head as isize))
                                    .state)
                                    .digest as *mut u8 as *const c_void,
                                0x30 as usize,
                            ) != 0 as i32
                            {
                                ::core::ptr::write_volatile(
                                    &mut curr_result as *mut s32,
                                    -(0x107 as i32) as s32,
                                );
                            }
                        }
                    }
                }
            }
        }
        1 => {
            if card_result != 0 as i32 {
                (*state)
                    .block_ids[commands[curr_head as usize].x8.write.phys
                    as usize] = -(0x7fff as i32) as s32;
                (*state)
                    .block_seqs[commands[curr_head as usize].x8.write.phys
                    as usize] = 0 as i32 as s32;
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else {
                if commands[curr_head as usize].x8.write.block_id
                    != 0xffff as i32
                {
                    (*state)
                        .block_ids[commands[curr_head as usize].x8.write.phys
                        as usize] = commands[curr_head as usize]
                        .x8
                        .write
                        .block_id;
                    (*state)
                        .block_seqs[commands[curr_head as usize]
                        .x8
                        .write
                        .phys as usize] = commands[curr_head as usize]
                        .x8
                        .write
                        .seq;
                } else {
                    (*state)
                        .block_ids[commands[curr_head as usize].x8.write.phys
                        as usize] = -(0x7fff as i32) as s32;
                    (*state)
                        .block_seqs[commands[curr_head as usize]
                        .x8
                        .write
                        .phys as usize] = 0 as i32 as s32;
                }
                result = checkOpen(state) as s32;
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                }
            }
        }
        16 => {
            if card_result != 0 as i32 {
                (*state)
                    .block_ids[commands[curr_head as usize].x8.sector.phys
                    as usize] = -(0x7fff as i32) as s32;
                (*state)
                    .block_seqs[commands[curr_head as usize].x8.sector.phys
                    as usize] = 0 as i32 as s32;
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else {
                if commands[curr_head as usize].x8.sector.block_id
                    != 0xffff as i32
                {
                    (*state)
                        .block_ids[commands[curr_head as usize]
                        .x8
                        .sector
                        .phys as usize] = commands[curr_head as usize]
                        .x8
                        .sector
                        .block_id;
                    (*state)
                        .block_seqs[commands[curr_head as usize]
                        .x8
                        .sector
                        .phys as usize] = commands[curr_head as usize]
                        .x8
                        .sector
                        .seq;
                } else {
                    (*state)
                        .block_ids[commands[curr_head as usize]
                        .x8
                        .sector
                        .phys as usize] = -(0x7fff as i32) as s32;
                    (*state)
                        .block_seqs[commands[curr_head as usize]
                        .x8
                        .sector
                        .phys as usize] = 0 as i32 as s32;
                }
                result = checkOpen(state) as s32;
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                }
            }
        }
        7 => {
            if card_result != 0 as i32 {
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else {
                i = 0 as i32 as s32;
                result = (*state).file_info.fileNo;
                while !(result != -(1 as i32)) {
                    i += 1;
                    if !(i < 10 as i32) {
                        break;
                    }
                }
                (*state).file_no = result;
                if (*state).file_no < 0 as i32 {
                    ::core::ptr::write_volatile(
                        &mut curr_result as *mut s32,
                        (*state).file_no,
                    );
                }
                result = checkOpen(state) as s32;
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                }
            }
        }
        8 => {
            if card_result != 0 as i32 {
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            }
        }
        9 => {
            if card_result != 0 as i32 {
                checkOpen(state);
                ::core::ptr::write_volatile(&mut curr_result as *mut s32, card_result);
            } else {
                result = checkOpen(state) as s32;
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                }
            }
        }
        13 => {
            phys = commands[curr_head as usize].x8.read.phys;
            checkOpen(state);
            if card_result != 0 as i32 {
                (*state).block_ids[phys as usize] = -(0x7fff as i32)
                    as s32;
                (*state).block_seqs[phys as usize] = 0 as i32 as s32;
            } else if phys == 0 as i32 {
                hdr_offset13 = (*state)
                    .header_size
                    .wrapping_add(0x30 as u32)
                    .card_rem((*state).sector_size) as s32;
                if HSD_Decrypt(
                    (*state).sector_buf.offset(hdr_offset13 as isize),
                    (*state).sector_size.wrapping_sub(hdr_offset13 as u32)
                        as i32,
                ) < 0 as i32
                {
                    (*state).block_ids[0 as i32 as usize] = -(0x7fff
                        as i32) as s32;
                    (*state).block_seqs[0 as i32 as usize] = 0
                        as i32 as s32;
                } else {
                    (*state).block_ids[phys as usize] = (*(*state)
                        .sector_buf
                        .offset(
                            (hdr_offset13 as i32
                                + 0x11 as i32) as isize,
                        ) as i32
                        + ((*(*state)
                            .sector_buf
                            .offset(
                                (hdr_offset13 as i32
                                    + 0x10 as i32) as isize,
                            ) as i32) << 8 as i32)) as s32;
                    if (*state).block_ids[phys as usize] != 0 as i32 {
                        (*state).block_ids[phys as usize] = -(0x7fff
                            as i32) as s32;
                        (*state).block_seqs[phys as usize] = 0 as i32
                            as s32;
                    } else {
                        (*state).block_seqs[phys as usize] = *(*state)
                            .sector_buf
                            .offset(
                                (hdr_offset13 as i32
                                    + 0x12 as i32) as isize,
                            ) as s32;
                        let mut src13: *mut CardBlockHeader = (*state).sector_buf.offset(hdr_offset13 as isize).cast::<CardBlockHeader>();
                        hsd_803AC558(state, &raw mut (*src13).file_table as *mut u8);
                    }
                }
            } else if HSD_Decrypt(
                (*state).sector_buf,
                (*state).sector_size as i32,
            ) < 0 as i32
            {
                (*state).block_ids[phys as usize] = -(0x7fff as i32)
                    as s32;
                (*state).block_seqs[phys as usize] = 0 as i32 as s32;
            } else {
                (*state).block_ids[phys as usize] = (*(*state)
                    .sector_buf
                    .offset(0x11 as i32 as isize) as i32
                    + ((*(*state).sector_buf.offset(0x10 as i32 as isize)
                        as i32) << 8 as i32)) as s32;
                if (*state).block_ids[phys as usize] < 0 as i32
                    || (*state).block_ids[phys as usize] > (*state).num_blocks
                {
                    (*state).block_ids[phys as usize] = -(0x7fff as i32)
                        as s32;
                    (*state).block_seqs[phys as usize] = 0 as i32 as s32;
                } else {
                    (*state).block_seqs[phys as usize] = *(*state)
                        .sector_buf
                        .offset(0x12 as i32 as isize) as s32;
                    hsd_803AC558(
                        state,
                        (*state).sector_buf.offset(0x13 as i32 as isize),
                    );
                }
            }
        }
        _ => {}
    }
    commands[curr_head as usize].r#type = CARD_CMD_NONE;
    hsd_804D799C = running;
    ::core::ptr::write_volatile(
        &mut curr_head as *mut s32,
        ((curr_head as i32 + 1 as i32)
            % 128 as i32) as s32,
    );
}
pub unsafe extern "C" fn fn_803AA790() -> i32 {
    melee_core::ct!("fn_803AA790");
    let mut entry: *mut CardRequest = ::core::ptr::null_mut::<CardRequest>();
    let mut result: s32 = 0;
    let mut state: *mut CardState = ::core::ptr::null_mut::<CardState>();
    entry = (&raw mut requests as *mut CardRequest).offset(hsd_804D7990 as isize)
        as *mut CardRequest;
    state = (*entry).state;
    hsd_804D7990 = ((hsd_804D7990 as i32 + 1 as i32)
        % 32 as i32) as s32;
    match (*entry).r#type as i32 {
        1 => {
            result = fn_803ADF90(
                (*entry).state,
                (*entry).x8.file.file_idx,
                (*entry).x8.file.buf,
                1 as i32,
                (*entry).callback,
            ) as s32;
            if result < 0 as i32 && (*entry).callback.is_some() {
                (*entry)
                    .callback
                    .expect(
                        "non-null function pointer",
                    )(
                    (*entry).x8.file.file_idx as i32,
                    result as i32,
                );
            }
            (*entry).r#type = CARD_REQ_NONE;
            return result as i32;
        }
        2 => {
            match (*state).file_flags[(*entry).x8.file.file_idx as usize] {
                0 => {
                    result = fn_803AE7F8(
                        (*entry).state,
                        (*entry).x8.file.file_idx,
                        (*entry).x8.file.buf,
                        1 as i32,
                        (*entry).callback,
                    ) as s32;
                }
                1 | 2 => {
                    result = fn_803AF3F0(
                        (*entry).state,
                        (*entry).x8.file.file_idx,
                        (*entry).x8.file.buf,
                        1 as i32,
                        (*entry).callback,
                    ) as s32;
                }
                3 => {
                    result = fn_803B0120(
                        (*entry).state,
                        (*entry).x8.file.file_idx,
                        (*entry).x8.file.buf,
                        1 as i32,
                        (*entry).callback,
                    ) as s32;
                }
                _ => {
                    result = -(0x101 as i32) as s32;
                }
            }
            if result < 0 as i32 && (*entry).callback.is_some() {
                (*entry)
                    .callback
                    .expect(
                        "non-null function pointer",
                    )(
                    (*entry).x8.file.file_idx as i32,
                    result as i32,
                );
            }
            (*entry).r#type = CARD_REQ_NONE;
            return result as i32;
        }
        3 => {
            result = fn_803B1F78(
                (*entry).state,
                (*entry).x8.create.filename,
                (*entry).x8.create.banner,
                (*entry).x8.create.icons,
                (*entry).callback,
            ) as s32;
            if result < 0 as i32 && (*entry).callback.is_some() {
                (*entry)
                    .callback
                    .expect(
                        "non-null function pointer",
                    )(0 as i32, result as i32);
            }
            (*entry).r#type = CARD_REQ_NONE;
            return result as i32;
        }
        4 => {
            result = fn_803B21E8(
                (*entry).state,
                (*entry).x8.status.banner,
                (*entry).x8.status.icons,
                (*entry).callback,
            ) as s32;
            if result < 0 as i32 && (*entry).callback.is_some() {
                (*entry)
                    .callback
                    .expect(
                        "non-null function pointer",
                    )(0 as i32, result as i32);
            }
            (*entry).r#type = CARD_REQ_NONE;
            return result as i32;
        }
        5 => {
            result = fn_803ADE4C(
                state,
                (*entry).x8.open.file_no as i32,
                (*entry).callback,
            ) as s32;
            if result < 0 as i32 && (*entry).callback.is_some() {
                (*entry)
                    .callback
                    .expect(
                        "non-null function pointer",
                    )(0 as i32, result as i32);
            }
            (*entry).r#type = CARD_REQ_NONE;
            return result as i32;
        }
        6 => {
            result = fn_803B26CC(
                state,
                (*entry).x8.header.comment,
                (*entry).x8.header.banner,
                (*entry).x8.header.icons,
                (*entry).callback,
            ) as s32;
            if result < 0 as i32 && (*entry).callback.is_some() {
                (*entry)
                    .callback
                    .expect(
                        "non-null function pointer",
                    )(0 as i32, result as i32);
            }
            (*entry).r#type = CARD_REQ_NONE;
            return result as i32;
        }
        _ => return -(0x101 as i32),
    };
}
#[inline]
pub unsafe extern "C" fn retryCardOpen(
    mut chan: s32,
    mut filename: *const i8,
    mut file_info: *mut CARDFileInfo,
) -> s32 {
    melee_core::ct!("retryCardOpen");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDOpen(chan, filename as *mut i8, file_info);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn retryCardFastOpen(
    mut chan: s32,
    mut file_no: s32,
    mut file_info: *mut CARDFileInfo,
) -> s32 {
    melee_core::ct!("retryCardFastOpen");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDFastOpen(chan, file_no, file_info);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn retryCardReadAsync(
    mut file_info: *mut CARDFileInfo,
    mut buffer: *mut c_void,
    mut length: s32,
    mut offset: s32,
    mut callback: CARDCallback,
) -> s32 {
    melee_core::ct!("retryCardReadAsync");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDReadAsync(file_info, buffer, length, offset, callback);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn retryCardWriteAsync(
    mut file_info: *mut CARDFileInfo,
    mut buffer: *mut c_void,
    mut length: s32,
    mut offset: s32,
    mut callback: CARDCallback,
) -> s32 {
    melee_core::ct!("retryCardWriteAsync");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDWriteAsync(file_info, buffer, length, offset, callback);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn retryCardClose(mut file_info: *mut CARDFileInfo) -> s32 {
    melee_core::ct!("retryCardClose");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDClose(file_info);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn retryCardCreateAsync(
    mut chan: s32,
    mut filename: *const i8,
    mut size: s32,
    mut file_info: *mut CARDFileInfo,
    mut callback: CARDCallback,
) -> s32 {
    melee_core::ct!("retryCardCreateAsync");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDCreateAsync(chan, filename, size as u32, file_info, callback);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn retryCardGetStatus(
    mut chan: s32,
    mut file_no: s32,
    mut stat: *mut CARDStat,
) -> s32 {
    melee_core::ct!("retryCardGetStatus");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDGetStatus(chan, file_no, stat);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn retryCardSetStatusAsync(
    mut chan: s32,
    mut file_no: s32,
    mut stat: *mut CARDStat,
    mut callback: CARDCallback,
) -> s32 {
    melee_core::ct!("retryCardSetStatusAsync");
    let mut i: s32 = 0;
    let mut result: s32 = 0;
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        result = CARDSetStatusAsync(chan, file_no, stat, callback);
        if result != -(1 as i32) {
            break;
        }
        i += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn setupCardIcons(mut cmd: *const CardCmd) -> i32 {
    melee_core::ct!("setupCardIcons");
    let mut bit: u32 = 0;
    let mut k: s32 = 0;
    bit = 0 as u32;
    k = bit as s32;
    while k < 8 as i32
        && (*(*cmd).state).icon_info.icon_speed[k as usize] as i32
            != 0 as i32
    {
        (*(*cmd).state).stat.iconFormat = ((*(*cmd).state).stat.iconFormat
            as i32 & !(CARD_STAT_ICON_MASK << bit)
            | <i32 as melee_core::ppc::Ppc>::pshl((*(*cmd).state).icon_info.icon_format[k as usize] as i32, (bit) as i64)) as u16;
        let fresh1 = k;
        k = k + 1;
        (*(*cmd).state).stat.iconSpeed = ((*(*cmd).state).stat.iconSpeed
            as i32 & !(CARD_STAT_SPEED_MASK << bit)
            | <i32 as melee_core::ppc::Ppc>::pshl((*(*cmd).state).icon_info.icon_speed[fresh1 as usize]
                as i32, (bit) as i64)) as u16;
        bit = bit.wrapping_add(2 as u32);
    }
    return k as i32;
}
#[inline]
pub unsafe extern "C" fn unpackCardStat(mut cmd: *const CardCmd, mut stat: *mut CARDStat) {
    melee_core::ct!("unpackCardStat");
    let mut k: s32 = 0;
    (*(*cmd).state).icon_info.banner_format = ((*stat).bannerFormat as i32
        & 3 as i32) as u8;
    k = 0 as i32 as s32;
    while k < 8 as i32 {
        (*(*cmd).state).icon_info.icon_format[k as usize] = (<i32 as melee_core::ppc::Ppc>::pshr((*stat).iconFormat
            as i32, (2 as s32 * k) as i64) & CARD_STAT_ICON_MASK) as u8;
        (*(*cmd).state).icon_info.icon_speed[k as usize] = (<i32 as melee_core::ppc::Ppc>::pshr((*stat).iconSpeed
            as i32, (2 as s32 * k) as i64) & CARD_STAT_SPEED_MASK) as u8;
        k += 1;
    }
    (*(*cmd).state).header_size = hsd_803AC340(&raw mut (*(*cmd).state).icon_info)
        as u32;
    let mut used: u32 = (*(*cmd).state)
        .header_size
        .wrapping_add((*(*cmd).state).sector_size);
    used = used.wrapping_add(0x2f as u32);
    (*(*cmd).state).num_blocks = (*stat)
        .length
        .card_div((*(*cmd).state).sector_size)
        .wrapping_sub(used.card_div((*(*cmd).state).sector_size)) as s32;
}
pub unsafe extern "C" fn rollbackCardCommands(mut snap: s32) {
    melee_core::ct!("rollbackCardCommands");
    let mut saved: s32 = snap;
    while saved != curr_tail {
        commands[saved as usize].r#type = CARD_CMD_NONE;
        saved = ((saved as i32 + 1 as i32)
            % 128 as i32) as s32;
    }
    ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
}
#[inline]
pub unsafe extern "C" fn initHeaderBlockCommand(
    mut buf: *mut CardCmd,
    mut state: *mut CardState,
    mut block: u32,
    mut comment: *mut c_void,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
) {
    melee_core::ct!("initHeaderBlockCommand");
    (*buf).r#type = CARD_CMD_READ_HEADER;
    (*buf).state = state;
    (*buf).x8.read_header.index = block as s32;
    (*buf).x8.read_header.comment = comment;
    (*buf).x8.read_header.banner = banner;
    (*buf).x8.read_header.icons = icons;
}
#[inline]
pub unsafe extern "C" fn queueHeaderBlock(
    mut state: *mut CardState,
    mut block: u32,
    mut comment: *mut c_void,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
) -> s32 {
    melee_core::ct!("queueHeaderBlock");
    let mut buf: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    initHeaderBlockCommand(&raw mut buf, state, block, comment, banner, icons);
    return fn_803AC168(&raw mut buf) as s32;
}
#[inline]
pub unsafe extern "C" fn queueHeaderBlocks(
    mut state: *mut CardState,
    mut comment: *mut c_void,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
) -> s32 {
    melee_core::ct!("queueHeaderBlocks");
    let mut i: u32 = 0;
    let mut result: i32 = 0;
    i = 0 as u32;
    while i
        < (0x2f as u32)
            .wrapping_add((*state).header_size)
            .wrapping_add((*state).sector_size)
            .card_div((*state).sector_size)
    {
        result = queueHeaderBlock(state, i, comment, banner, icons)
            as i32;
        if result < 0 as i32 {
            return result as s32;
        }
        i = i.wrapping_add(1);
    }
    return 0 as s32;
}
pub unsafe extern "C" fn hsd_803AAA48() {
    melee_core::ct!("hsd_803AAA48");
    let mut result: s32 = 0;
    let mut chan: s32 = 0;
    's_8: loop {
        let mut state_ptr: *mut *mut CardState = &raw mut active_requests.state;
        let mut cmd: *mut CardCmd = ::core::ptr::null_mut::<CardCmd>();
        let mut r#type: s32 = 0;
        let mut intr: BOOL = OSDisableInterrupts();
        let mut intr2: BOOL = 0;
        let mut busy: s32 = 0;
        if hsd_804D799C as i32 == 1 as i32 {
            busy = 1 as i32 as s32;
        } else {
            busy = 0 as i32 as s32;
        }
        OSRestoreInterrupts(intr);
        if busy != 0 {
            return;
        }
        if curr_result < 0 as i32 {
            while commands[curr_head as usize].r#type as i32
                != CARD_CMD_NONE as i32
            {
                commands[curr_head as usize].r#type = CARD_CMD_NONE;
                ::core::ptr::write_volatile(
                    &mut curr_head as *mut s32,
                    ((curr_head as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
            }
        }
        cmd = (&raw mut commands as *mut CardCmd).offset(curr_head as isize)
            as *mut CardCmd;
        r#type = (*cmd).r#type as s32;
        match r#type as u32 {
            0 => {
                if active_requests.r#type as i32
                    != CARD_ACTIVE_NONE as i32
                {
                    if active_requests.r#type as i32
                        == CARD_ACTIVE_WRITE_FILE_1_2 as i32
                    {
                        let mut file_idx: s32 = active_requests.callback_arg;
                        let mut blocks_before: s32 = fn_803AC6B8(*state_ptr, file_idx);
                        let mut file_blocks: s32 = fn_803AC634(*state_ptr, file_idx)
                            as s32;
                        let mut total: s32 = fn_803AC7DC(*state_ptr);
                        let mut map: [s32; 64] = [0; 64];
                        let mut j: s32 = 0;
                        j = 0 as i32 as s32;
                        while j < file_blocks {
                            map[j as usize] = -(1 as i32) as s32;
                            j += 1;
                        }
                        j = 1 as i32 as s32;
                        while j <= total {
                            let mut block_id: s32 = (**state_ptr).block_ids[j as usize];
                            if block_id >= 0 as i32 {
                                let mut logical: s32 = block_id - blocks_before;
                                if logical >= 0 as i32
                                    && logical < file_blocks
                                {
                                    if map[logical as usize] < 0 as i32 {
                                        map[logical as usize] = j;
                                    } else if fn_803ACB74(
                                        (**state_ptr).block_seqs[map[logical as usize] as usize],
                                        (**state_ptr).block_seqs[j as usize],
                                    ) < 0 as i32
                                    {
                                        (**state_ptr).block_ids[map[logical as usize] as usize]
                                            *= -(1 as i32);
                                        map[logical as usize] = j;
                                    } else {
                                        (**state_ptr).block_ids[j as usize]
                                            *= -(1 as i32);
                                    }
                                }
                            }
                            j += 1;
                        }
                    }
                    if active_requests.callback.is_some() {
                        active_requests
                            .callback
                            .expect(
                                "non-null function pointer",
                            )(
                            active_requests.callback_arg as i32,
                            curr_result as i32,
                        );
                    }
                    active_requests.r#type = CARD_ACTIVE_NONE;
                }
                ::core::ptr::write_volatile(
                    &mut curr_result as *mut s32,
                    0 as i32 as s32,
                );
                hsd_804D799C = idle;
                if hsd_804D7990 == hsd_804D7994
                    && requests[hsd_804D7990 as usize].r#type as i32
                        == CARD_REQ_NONE as i32
                {
                    return;
                }
                if fn_803AA790() < 0 as i32 {
                    return;
                }
                hsd_804D799C = running;
            }
            5 => {
                if curr_result == 2 as i32 {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                } else {
                    result = retryCardFastOpen(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).file_info,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        hsd_804D798C = CARDGetXferredBytes(
                            (*(*cmd).state).chan as i32,
                        ) as s32;
                        intr2 = OSDisableInterrupts();
                        result = retryCardReadAsync(
                            &raw mut (*(*cmd).state).file_info,
                            (*(*cmd).state).sector_buf as *mut c_void,
                            (*(*cmd).state).sector_size as s32,
                            (*cmd).x8.verify.offset,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                            retryCardClose(&raw mut (*(*cmd).state).file_info);
                        } else {
                            return
                        }
                    }
                }
            }
            11 => {
                result = retryCardFastOpen(
                    (*(*cmd).state).chan,
                    (*(*cmd).state).file_no,
                    &raw mut (*(*cmd).state).file_info,
                );
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                } else {
                    hsd_804D798C = CARDGetXferredBytes(
                        (*(*cmd).state).chan as i32,
                    ) as s32;
                    if (*cmd).x8.read_header.index == 0 as i32
                    {
                        memset(
                            &raw mut (*(*cmd).state).digest as *mut u8
                                as *mut c_void,
                            0 as i32,
                            0x30 as usize,
                        );
                    }
                    intr2 = OSDisableInterrupts();
                    result = retryCardReadAsync(
                        &raw mut (*(*cmd).state).file_info,
                        (*(*cmd).state).sector_buf as *mut c_void,
                        (*(*cmd).state).sector_size as s32,
                        ((*cmd).x8.read_header.index as u32)
                            .wrapping_mul((*(*cmd).state).sector_size) as s32,
                        Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                    );
                    hsd_804D799C = running_async;
                    OSRestoreInterrupts(intr2);
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                        hsd_804D799C = running;
                        retryCardClose(&raw mut (*(*cmd).state).file_info);
                    } else {
                        return
                    }
                }
            }
            10 => {
                if curr_result == 2 as i32 {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                } else {
                    result = retryCardFastOpen(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).file_info,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        hsd_804D798C = CARDGetXferredBytes(
                            (*(*cmd).state).chan as i32,
                        ) as s32;
                        if (*cmd).x8.header.index == 0 as i32
                        {
                            memset(
                                &raw mut (*(*cmd).state).digest as *mut u8
                                    as *mut c_void,
                                0 as i32,
                                0x30 as usize,
                            );
                        }
                        intr2 = OSDisableInterrupts();
                        result = retryCardReadAsync(
                            &raw mut (*(*cmd).state).file_info,
                            (*(*cmd).state).sector_buf as *mut c_void,
                            (*(*cmd).state).sector_size as s32,
                            ((*cmd).x8.header.index as u32)
                                .wrapping_mul((*(*cmd).state).sector_size) as s32,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                            retryCardClose(&raw mut (*(*cmd).state).file_info);
                        } else {
                            return
                        }
                    }
                }
            }
            6 => {
                if curr_result == 2 as i32 {
                    ::core::ptr::write_volatile(
                        &mut curr_result as *mut s32,
                        0 as i32 as s32,
                    );
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                } else {
                    ::core::ptr::write_volatile(
                        &mut curr_result as *mut s32,
                        1 as i32 as s32,
                    );
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            2 => {
                if curr_result != 1 as i32 {
                    result = retryCardFastOpen(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).file_info,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        hsd_804D798C = CARDGetXferredBytes(
                            (*(*cmd).state).chan as i32,
                        ) as s32;
                        intr2 = OSDisableInterrupts();
                        result = retryCardReadAsync(
                            &raw mut (*(*cmd).state).file_info,
                            (*(*cmd).state).sector_buf as *mut c_void,
                            (*(*cmd).state).sector_size as s32,
                            (*cmd).x8.read.offset,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                            retryCardClose(&raw mut (*(*cmd).state).file_info);
                        } else {
                            return
                        }
                    }
                } else {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            15 => {
                if curr_result != 1 as i32 {
                    result = retryCardFastOpen(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).file_info,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        hsd_804D798C = CARDGetXferredBytes(
                            (*(*cmd).state).chan as i32,
                        ) as s32;
                        intr2 = OSDisableInterrupts();
                        result = retryCardReadAsync(
                            &raw mut (*(*cmd).state).file_info,
                            (*(*cmd).state).sector_buf as *mut c_void,
                            (*(*cmd).state).sector_size as s32,
                            (*cmd).x8.sector.offset,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                            retryCardClose(&raw mut (*(*cmd).state).file_info);
                        } else {
                            return
                        }
                    }
                } else {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            4 => {
                if curr_result != 1 as i32 {
                    memset(
                        (*cmd).x8.clear.data,
                        0 as i32,
                        (*cmd).x8.clear.size as usize,
                    );
                }
                (*cmd).r#type = CARD_CMD_NONE;
                ::core::ptr::write_volatile(
                    &mut curr_head as *mut s32,
                    ((curr_head as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
            }
            1 => {
                if curr_result != 1 as i32 {
                    let mut hdr_offset: s32 = 0;
                    let mut rem: s32 = 0;
                    let mut size: i32 = 0;
                    result = retryCardFastOpen(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).file_info,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        if (*cmd).x8.write.block_id > 0 as i32
                        {
                            hdr_offset = 0 as i32 as s32;
                        } else {
                            hdr_offset = (*(*cmd).state)
                                .header_size
                                .wrapping_add(0x30 as u32)
                                .card_rem((*(*cmd).state).sector_size) as s32;
                        }
                        size = (*cmd).x8.write.size as i32;
                        if size > 0 as i32
                            && !(*cmd).x8.write.data.is_null()
                        {
                            memcpy(
                                (*(*cmd).state)
                                    .sector_buf
                                    .offset(
                                        (hdr_offset as i32
                                            + 0x20 as i32) as isize,
                                    ) as *mut u8 as *mut c_void,
                                (*cmd).x8.write.data,
                                size as usize,
                            );
                        }
                        rem = (*(*cmd).state)
                            .sector_size
                            .wrapping_sub(hdr_offset as u32)
                            .wrapping_sub(size as u32)
                            .wrapping_sub(0x20 as u32) as s32;
                        if rem != 0 as i32 {
                            memset(
                                (*(*cmd).state)
                                    .sector_buf
                                    .offset(
                                        (hdr_offset as i32 + size
                                            + 0x20 as i32) as isize,
                                    ) as *mut u8 as *mut c_void,
                                0 as i32,
                                rem as usize,
                            );
                        }
                        memset(
                            (*(*cmd).state).sector_buf.offset(hdr_offset as isize)
                                as *mut c_void,
                            0 as i32,
                            0x20 as usize,
                        );
                        *(*(*cmd).state)
                            .sector_buf
                            .offset(
                                (hdr_offset as i32
                                    + 0x10 as i32) as isize,
                            ) = ((*cmd).x8.write.block_id
                            >> 8 as i32) as u8;
                        *(*(*cmd).state)
                            .sector_buf
                            .offset(
                                (hdr_offset as i32
                                    + 0x11 as i32) as isize,
                            ) = (*cmd).x8.write.block_id as u8;
                        *(*(*cmd).state)
                            .sector_buf
                            .offset(
                                (hdr_offset as i32
                                    + 0x12 as i32) as isize,
                            ) = (*cmd).x8.write.seq as u8;
                        fn_803AC3F8(
                            (*cmd).state,
                            (*(*cmd).state)
                                .sector_buf
                                .offset(
                                    (hdr_offset as i32
                                        + 0x13 as i32) as isize,
                                ) as *mut u8,
                            (*cmd).x8.write.file_idx,
                        );
                        HSD_Encrypt(
                            (*(*cmd).state).sector_buf.offset(hdr_offset as isize),
                            (*(*cmd).state).sector_size.wrapping_sub(hdr_offset as u32)
                                as i32,
                        );
                        hsd_804D798C = CARDGetXferredBytes(
                            (*(*cmd).state).chan as i32,
                        ) as s32;
                        intr2 = OSDisableInterrupts();
                        result = retryCardWriteAsync(
                            &raw mut (*(*cmd).state).file_info,
                            (*(*cmd).state).sector_buf as *mut c_void,
                            (*(*cmd).state).sector_size as s32,
                            (*cmd).x8.write.offset,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                            retryCardClose(&raw mut (*(*cmd).state).file_info);
                        } else {
                            return
                        }
                    }
                } else {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            16 => {
                if curr_result != 1 as i32 {
                    result = retryCardFastOpen(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).file_info,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        HSD_Encrypt(
                            (*(*cmd).state).sector_buf,
                            (*(*cmd).state).sector_size as i32,
                        );
                        hsd_804D798C = CARDGetXferredBytes(
                            (*(*cmd).state).chan as i32,
                        ) as s32;
                        intr2 = OSDisableInterrupts();
                        result = retryCardWriteAsync(
                            &raw mut (*(*cmd).state).file_info,
                            (*(*cmd).state).sector_buf as *mut c_void,
                            (*(*cmd).state).sector_size as s32,
                            (*cmd).x8.sector.offset,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                            retryCardClose(&raw mut (*(*cmd).state).file_info);
                        } else {
                            return
                        }
                    }
                } else {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            7 => {
                intr2 = OSDisableInterrupts();
                result = retryCardCreateAsync(
                    (*(*cmd).state).chan,
                    (*cmd).x8.create.filename,
                    (*cmd).x8.create.size,
                    &raw mut (*(*cmd).state).file_info,
                    Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                );
                hsd_804D799C = running_async;
                OSRestoreInterrupts(intr2);
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                    hsd_804D799C = running;
                } else {
                    return
                }
            }
            8 => {
                if curr_result != 1 as i32 {
                    let mut k: i32 = 0;
                    result = retryCardGetStatus(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).stat,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        (*(*cmd).state).stat.commentAddr = 0 as u32;
                        (*(*cmd).state).stat.iconAddr = 0x40 as u32;
                        (*(*cmd).state).stat.bannerFormat = ((*(*cmd).state)
                            .stat
                            .bannerFormat as i32
                            & !(3 as i32)
                            | (*(*cmd).state).icon_info.banner_format
                                as i32) as u8;
                        k = setupCardIcons(cmd);
                        while k < 8 as i32 {
                            (*(*cmd).state).stat.iconFormat = ((*(*cmd).state)
                                .stat
                                .iconFormat as i32
                                & !(CARD_STAT_ICON_MASK << 2 as i32 * k))
                                as u16;
                            (*(*cmd).state).stat.iconSpeed = ((*(*cmd).state)
                                .stat
                                .iconSpeed as i32
                                & !(CARD_STAT_SPEED_MASK << 2 as i32 * k))
                                as u16;
                            k += 1;
                        }
                        intr2 = OSDisableInterrupts();
                        result = retryCardSetStatusAsync(
                            (*(*cmd).state).chan,
                            (*(*cmd).state).file_no,
                            &raw mut (*(*cmd).state).stat,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                        } else {
                            return
                        }
                    }
                } else {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            9 => {
                if curr_result != 1 as i32 {
                    let mut banner_size: s32 = 0;
                    let mut pos: s32 = 0;
                    result = retryCardFastOpen(
                        (*(*cmd).state).chan,
                        (*(*cmd).state).file_no,
                        &raw mut (*(*cmd).state).file_info,
                    );
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                    } else {
                        match (*(*cmd).state).icon_info.banner_format
                            as i32
                        {
                            2 => {
                                banner_size = BANNER_LARGE as s32;
                            }
                            1 => {
                                banner_size = BANNER_SMALL as s32;
                            }
                            _ => {
                                banner_size = 0 as i32 as s32;
                            }
                        }
                        if (*cmd).x8.header.index == 0 as i32
                        {
                            memcpy(
                                (*(*cmd).state).sector_buf as *mut c_void,
                                &raw mut (*(*cmd).state).comment as *mut i8
                                    as *const c_void,
                                0x40 as usize,
                            );
                            pos = 0x40 as i32 as s32;
                            if banner_size > 0 as i32 {
                                memcpy(
                                    (*(*cmd).state)
                                        .sector_buf
                                        .offset(0x40 as i32 as isize)
                                        as *mut c_void,
                                    (*cmd).x8.header.banner,
                                    banner_size as usize,
                                );
                                pos = (banner_size as i32
                                    + 0x40 as i32) as s32;
                            }
                            memset(
                                &raw mut (*(*cmd).state).digest as *mut u8
                                    as *mut c_void,
                                0 as i32,
                                0x30 as usize,
                            );
                            if (*(*cmd).state).header_size > (*(*cmd).state).sector_size
                            {
                                memcpy(
                                    (*(*cmd).state).sector_buf.offset(pos as isize) as *mut u8
                                        as *mut c_void,
                                    (*cmd).x8.header.icons,
                                    (*(*cmd).state).sector_size.wrapping_sub(pos as u32)
                                        as usize,
                                );
                                HSD_Checksum(
                                    (*(*cmd).state).sector_buf,
                                    (*(*cmd).state).sector_size as i32,
                                    (&raw mut (*(*cmd).state).digest as *mut u8)
                                        .offset(
                                            ((*cmd).x8.header.index as i32
                                                * 0x10 as i32) as isize,
                                        ) as *mut u8 as *mut c_void,
                                );
                            } else {
                                memcpy(
                                    (*(*cmd).state).sector_buf.offset(pos as isize) as *mut u8
                                        as *mut c_void,
                                    (*cmd).x8.header.icons,
                                    (*(*cmd).state).header_size.wrapping_sub(pos as u32)
                                        as usize,
                                );
                                HSD_Checksum(
                                    (*(*cmd).state).sector_buf,
                                    (*(*cmd).state).header_size as i32,
                                    (&raw mut (*(*cmd).state).digest as *mut u8)
                                        .offset(
                                            ((*cmd).x8.header.index as i32
                                                * 0x10 as i32) as isize,
                                        ) as *mut u8 as *mut c_void,
                                );
                                memcpy(
                                    (*(*cmd).state)
                                        .sector_buf
                                        .offset((*(*cmd).state).header_size as isize) as *mut u8
                                        as *mut c_void,
                                    &raw mut (*(*cmd).state).digest as *mut u8
                                        as *const c_void,
                                    0x30 as usize,
                                );
                            }
                        } else {
                            let mut remaining: u32 = (*(*cmd).state)
                                .header_size
                                .wrapping_sub(
                                    (*(*cmd).state)
                                        .sector_size
                                        .wrapping_mul((*cmd).x8.header.index as u32),
                                );
                            let mut icons_offset: s32 = (*(*cmd).state)
                                .sector_size
                                .wrapping_mul((*cmd).x8.header.index as u32)
                                .wrapping_sub(0x40 as u32)
                                .wrapping_sub(banner_size as u32) as s32;
                            if remaining > (*(*cmd).state).sector_size {
                                memcpy(
                                    (*(*cmd).state).sector_buf as *mut c_void,
                                    ((*cmd).x8.header.icons as *mut u8)
                                        .offset(icons_offset as isize)
                                        as *const c_void,
                                    (*(*cmd).state).sector_size as usize,
                                );
                                HSD_Checksum(
                                    (*(*cmd).state).sector_buf,
                                    (*(*cmd).state).sector_size as i32,
                                    (&raw mut (*(*cmd).state).digest as *mut u8)
                                        .offset(
                                            ((*cmd).x8.header.index as i32
                                                * 0x10 as i32) as isize,
                                        ) as *mut u8 as *mut c_void,
                                );
                            } else {
                                memcpy(
                                    (*(*cmd).state).sector_buf as *mut c_void,
                                    ((*cmd).x8.header.icons as *mut u8)
                                        .offset(icons_offset as isize)
                                        as *const c_void,
                                    remaining as usize,
                                );
                                HSD_Checksum(
                                    (*(*cmd).state).sector_buf,
                                    remaining as i32,
                                    (&raw mut (*(*cmd).state).digest as *mut u8)
                                        .offset(
                                            ((*cmd).x8.header.index as i32
                                                * 0x10 as i32) as isize,
                                        ) as *mut u8 as *mut c_void,
                                );
                                memcpy(
                                    (*(*cmd).state).sector_buf.offset(remaining as isize)
                                        as *mut u8 as *mut c_void,
                                    &raw mut (*(*cmd).state).digest as *mut u8
                                        as *const c_void,
                                    0x30 as usize,
                                );
                            }
                        }
                        intr2 = OSDisableInterrupts();
                        result = retryCardWriteAsync(
                            &raw mut (*(*cmd).state).file_info,
                            (*(*cmd).state).sector_buf as *mut c_void,
                            (*(*cmd).state).sector_size as s32,
                            (*(*cmd).state)
                                .sector_size
                                .wrapping_mul((*cmd).x8.header.index as u32)
                                as s32,
                            Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                        );
                        hsd_804D799C = running_async;
                        OSRestoreInterrupts(intr2);
                        if result < 0 as i32 {
                            hsd_804D799C = running;
                            retryCardClose(&raw mut (*(*cmd).state).file_info);
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                        } else {
                            return
                        }
                    }
                } else {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            12 => {
                let mut retries: s32 = 0;
                let mut stat: CARDStat = CARDStat {
                    fileName: [0; 32],
                    length: 0,
                    time: 0,
                    gameName: [0; 4],
                    company: [0; 2],
                    bannerFormat: 0,
                    iconAddr: 0,
                    iconFormat: 0,
                    iconSpeed: 0,
                    commentAddr: 0,
                    offsetBanner: 0,
                    offsetBannerTlut: 0,
                    offsetIcon: [0; 8],
                    offsetIconTlut: 0,
                    offsetData: 0,
                };
                let mut i: s32 = 0;
                let mut file_no: s32 = 0;
                (*(*cmd).state).file_no = (*cmd).x8.get_status.file_no;
                file_no = (*(*cmd).state).file_no;
                chan = (*(*cmd).state).chan;
                i = 0 as i32 as s32;
                while i < 10 as i32 {
                    result = CARDGetStatus(chan, file_no, &raw mut stat);
                    if result != -(1 as i32) {
                        break;
                    }
                    i += 1;
                }
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                    hsd_804D799C = running;
                } else {
                    unpackCardStat(cmd, &raw mut stat);
                    if stat.iconAddr != 0x40 as u32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            -(0x106 as i32) as s32,
                        );
                        hsd_804D799C = running;
                    } else {
                        (*cmd).r#type = CARD_CMD_NONE;
                        ::core::ptr::write_volatile(
                            &mut curr_head as *mut s32,
                            ((curr_head as i32 + 1 as i32)
                                % 128 as i32) as s32,
                        );
                    }
                }
            }
            13 => {
                result = retryCardFastOpen(
                    (*(*cmd).state).chan,
                    (*(*cmd).state).file_no,
                    &raw mut (*(*cmd).state).file_info,
                );
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                } else {
                    intr2 = OSDisableInterrupts();
                    result = retryCardReadAsync(
                        &raw mut (*(*cmd).state).file_info,
                        (*(*cmd).state).sector_buf as *mut c_void,
                        (*(*cmd).state).sector_size as s32,
                        (*cmd).x8.read.offset,
                        Some(hsd_803A949C as unsafe extern "C" fn(s32, s32) -> ()),
                    );
                    hsd_804D799C = running_async;
                    OSRestoreInterrupts(intr2);
                    if result < 0 as i32 {
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                        hsd_804D799C = running;
                        retryCardClose(&raw mut (*(*cmd).state).file_info);
                    } else {
                        return
                    }
                }
            }
            14 => {
                result = fn_803AD16C((*cmd).state);
                if result < 0 as i32 {
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                    hsd_804D799C = running;
                } else {
                    (*cmd).r#type = CARD_CMD_NONE;
                    ::core::ptr::write_volatile(
                        &mut curr_head as *mut s32,
                        ((curr_head as i32 + 1 as i32)
                            % 128 as i32) as s32,
                    );
                }
            }
            17 => {
                let mut blk: s32 = 0;
                fn_803AC2D4();
                result = fn_803ACF30(
                    (*cmd).state,
                    ::core::ptr::null_mut::<c_void>(),
                    ::core::ptr::null_mut::<c_void>(),
                    ::core::ptr::null_mut::<c_void>(),
                );
                if result < 0 as i32 {
                    fn_803AC2E0();
                    ::core::ptr::write_volatile(&mut curr_result as *mut s32, result);
                    hsd_804D799C = running;
                } else {
                    blk = 0 as i32 as s32;
                    while blk <= (*(*cmd).state).num_blocks {
                        result = fn_803AC258((*cmd).state, blk);
                        if result < 0 as i32 {
                            fn_803AC2E0();
                            ::core::ptr::write_volatile(
                                &mut curr_result as *mut s32,
                                result,
                            );
                            hsd_804D799C = running;
                            continue 's_8;
                        } else {
                            blk += 1;
                        }
                    }
                    result = fn_803AC2A4((*cmd).state);
                    if result < 0 as i32 {
                        fn_803AC2E0();
                        ::core::ptr::write_volatile(
                            &mut curr_result as *mut s32,
                            result,
                        );
                        hsd_804D799C = running;
                    } else {
                        fn_803AC334();
                        (*cmd).r#type = CARD_CMD_NONE;
                        ::core::ptr::write_volatile(
                            &mut curr_head as *mut s32,
                            ((curr_head as i32 + 1 as i32)
                                % 128 as i32) as s32,
                        );
                    }
                }
            }
            _ => return,
        }
    };
}
pub unsafe extern "C" fn fn_803AC168(mut cmd: *const CardCmd) -> i32 {
    melee_core::ct!("fn_803AC168");
    let mut intr: BOOL = 0;
    let mut mode: s32 = 0;
    let mut read_idx: s32 = 0;
    let mut head_type: s32 = 0;
    intr = OSDisableInterrupts();
    read_idx = curr_head;
    mode = hsd_804D799C as s32;
    head_type = commands[curr_head as usize].r#type as s32;
    OSRestoreInterrupts(intr);
    if mode != 2 as i32 {
        if curr_tail == read_idx {
            if mode != 0 as i32 || curr_tail != read_idx
                || head_type != 0 as i32
            {
                return -(265 as i32);
            }
        }
    }
    let mut idx: s32 = curr_tail;
    ::core::ptr::write_volatile(
        &mut curr_tail as *mut s32,
        ((curr_tail as i32 + 1 as i32)
            % 128 as i32) as s32,
    );
    memcpy(
        (&raw mut commands as *mut CardCmd).offset(idx as isize) as *mut CardCmd
            as *mut c_void,
        cmd as *const c_void,
        ::core::mem::size_of::<CardCmd>() as usize,
    );
    if mode == 2 as i32 {
        hsd_804D799C = running;
    }
    return 0 as i32;
}
pub unsafe extern "C" fn fn_803AC258(mut state: *mut CardState, mut block_idx: s32) -> s32 {
    melee_core::ct!("fn_803AC258");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    cmd.r#type = CARD_CMD_SCAN_BLOCK;
    cmd.state = state;
    cmd.x8.read.phys = block_idx;
    cmd.x8.read.xC = 0 as i32 as s32;
    cmd.x8.read.data = ::core::ptr::null_mut::<c_void>();
    cmd.x8.read.size = 0 as i32 as s32;
    cmd.x8.read.offset = fn_803ACBE8(state, block_idx);
    return fn_803AC168(&raw mut cmd) as s32;
}
pub unsafe extern "C" fn fn_803AC2A4(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803AC2A4");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    cmd.r#type = CARD_CMD_REPAIR;
    cmd.state = state;
    return fn_803AC168(&raw mut cmd) as s32;
}
pub unsafe extern "C" fn fn_803AC2D4() {
    melee_core::ct!("fn_803AC2D4");
    hsd_804D7998 = curr_tail as i32;
}
pub unsafe extern "C" fn fn_803AC2E0() {
    melee_core::ct!("fn_803AC2E0");
    let mut saved: s32 = 0;
    if hsd_804D7998 < 0 as i32 {
        return;
    }
    saved = hsd_804D7998 as s32;
    while saved != curr_tail {
        commands[saved as usize].r#type = CARD_CMD_NONE;
        saved = ((saved as i32 + 1 as i32)
            % 128 as i32) as s32;
    }
    ::core::ptr::write_volatile(&mut curr_tail as *mut s32, hsd_804D7998 as s32);
}
pub unsafe extern "C" fn fn_803AC334() {
    melee_core::ct!("fn_803AC334");
    hsd_804D7998 = -(1 as i32);
}
pub unsafe extern "C" fn hsd_803AC340(
    mut icon_info: *mut CardIconInfo,
) -> i32 {
    melee_core::ct!("hsd_803AC340");
    let mut data: *mut u8 = icon_info as *mut c_void as *mut u8;
    let mut banner_size: i32 = 0;
    let mut icon_size: i32 = 0;
    let mut has_c8: i32 = 0;
    let mut i: i32 = 0;
    match *data.offset(0 as i32 as isize) as i32 {
        2 => {
            banner_size = 0x1800 as i32;
        }
        1 => {
            banner_size = 0xe00 as i32;
        }
        _ => {
            banner_size = 0 as i32;
        }
    }
    has_c8 = 0 as i32;
    icon_size = 0 as i32;
    i = 0 as i32;
    while i < 8 as i32
        && *data.offset((i + 0xa as i32) as isize) as i32
            != 0 as i32
    {
        match *data.offset((i + 0x2 as i32) as isize)
            as i32
        {
            2 => {
                icon_size += 0x800 as i32;
            }
            1 => {
                has_c8 = 1 as i32;
                icon_size += 0x400 as i32;
            }
            _ => {}
        }
        i += 1;
    }
    if has_c8 != 0 as i32 {
        icon_size += 0x200 as i32;
    }
    icon_size = banner_size + icon_size;
    return icon_size + 0x40 as i32;
}
pub unsafe extern "C" fn hsd_803AC3E0(
    mut state: *mut CardState,
    mut file_idx: i32,
    mut file_size: i32,
    mut file_flags: i32,
    mut data: *mut u8,
) {
    melee_core::ct!("hsd_803AC3E0");
    (*state).file_flags[file_idx as usize] = file_flags;
    (*state).file_sizes[file_idx as usize] = file_size;
    (*state).file_data[file_idx as usize] = data;
}
// fn_803B1338's spare-block pass supplies file_idx == 9. The original
// three-row window then aliases file_flags[9] to file_sizes[0] and
// file_sizes[9] to file_data[0] at the GC offsets. Select those logical
// fields explicitly. This unused free-block word contains a process address;
// cross-process byte-identical save files need a canonical legacy address
// mapper here. The oracle compares this word as a pointer-valued observation.
pub unsafe extern "C" fn fn_803AC3F8(
    mut state: *mut CardState,
    mut data: *mut u8,
    mut file_idx: s32,
) {
    melee_core::ct!("fn_803AC3F8");
    let mut start: s32 = 0;
    let mut i: s32 = 0;
    if file_idx as i32 + 1 as i32
        >= 9 as i32
        || (*state)
            .file_sizes[(file_idx as i32 + 1 as i32)
            as usize] == 0 as i32
    {
        start = (file_idx as i32 - 2 as i32) as s32;
    } else {
        start = (file_idx as i32 - 1 as i32) as s32;
    }
    if start < 0 as i32 {
        start = 0 as i32 as s32;
    }
    i = 0 as i32 as s32;
    while i < 3 as i32 && i < 9 as i32 {
        let mut idx: s32 = start + i;
        let file_size = if idx == 9 { (*state).file_data[0] as usize as u32 as i32 } else { (*state).file_sizes[idx as usize] };
        let file_flags = if idx == 9 { (*state).file_sizes[0] } else { (*state).file_flags[idx as usize] };
        *data.offset(0 as i32 as isize) = idx as u8;
        *data.offset(1 as i32 as isize) = (file_size as u32 >> 0x10 as i32
            & 0x3f as u32
            | (file_flags << 6 as i32
                & 0xc0 as i32) as u32) as u8;
        *data.offset(2 as i32 as isize) = (file_size as u32 >> 8 as i32) as u8;
        *data.offset(3 as i32 as isize) = file_size as u8;
        data = data.offset(4 as i32 as isize);
        i += 1;
    }
    while i < 3 as i32 {
        *data.offset(0 as i32 as isize) = 0 as u8;
        *data.offset(1 as i32 as isize) = 0 as u8;
        *data.offset(2 as i32 as isize) = 0 as u8;
        *data.offset(3 as i32 as isize) = 0 as u8;
        data = data.offset(4 as i32 as isize);
        i += 1;
    }
}
pub unsafe extern "C" fn hsd_803AC558(mut state: *mut CardState, mut data: *mut u8) {
    melee_core::ct!("hsd_803AC558");
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < 3 as i32 {
        let fresh0 = data;
        data = data.offset(1);
        let mut file_idx: u8 = *fresh0;
        let mut byte1: u8 = *data.offset(0 as i32 as isize);
        let mut size: u32 = *data.offset(1 as i32 as isize) as u32
            | ((byte1 as i32) << 8 as i32
                & 0x3f00 as i32) as u32;
        let mut final_size: u32 = 0;
        let mut flags: u8 = (byte1 as i32 >> 6 as i32
            & 3 as i32) as u8;
        size <<= 8 as i32;
        final_size = size | *data.offset(2 as i32 as isize) as u32;
        data = data.offset(3 as i32 as isize);
        if final_size != 0 as u32
            && (*state).file_sizes[file_idx as usize] == 0 as i32
        {
            (*state).file_flags[file_idx as usize] = flags as i32;
            (*state).file_sizes[file_idx as usize] = final_size as i32;
        }
        i += 1;
    }
}
pub unsafe extern "C" fn fn_803AC634(mut state: *mut CardState, mut file_idx: s32) -> u32 {
    melee_core::ct!("fn_803AC634");
    if (*state).file_sizes[file_idx as usize] <= 0 as i32 {
        return 0 as u32;
    }
    if file_idx == 0 as i32 {
        let mut usable: u32 = 0;
        let mut sector_size: u32 = (*state).sector_size;
        let mut remaining: s32 = 0;
        remaining = (*state).file_sizes[0 as i32 as usize] as s32;
        usable = sector_size.wrapping_sub(0x20 as u32);
        remaining = remaining
            - usable
                .wrapping_sub(
                    (*state)
                        .header_size
                        .wrapping_add(48 as u32)
                        .card_rem(sector_size),
                ) as s32;
        if remaining <= 0 as i32 {
            return 1 as u32;
        }
        return (remaining as u32)
            .wrapping_add(sector_size)
            .wrapping_sub(0x21 as u32)
            .card_div(usable)
            .wrapping_add(1 as u32);
    }
    let mut sector_size_0: u32 = (*state).sector_size;
    return ((*state).file_sizes[file_idx as usize] as u32)
        .wrapping_add(sector_size_0)
        .wrapping_sub(0x21 as u32)
        .card_div(sector_size_0.wrapping_sub(0x20 as u32));
}
#[inline]
pub unsafe extern "C" fn fn_803AC6B8_first_block_count(mut state: *mut CardState) -> u32 {
    melee_core::ct!("fn_803AC6B8_first_block_count");
    if (*state).file_sizes[0 as i32 as usize] <= 0 as i32 {
        return 0 as u32;
    }
    let mut usable: u32 = 0;
    let mut sector_size: u32 = (*state).sector_size;
    let mut remaining: s32 = 0;
    remaining = (*state).file_sizes[0 as i32 as usize] as s32;
    usable = sector_size.wrapping_sub(0x20 as u32);
    remaining = remaining
        - usable
            .wrapping_sub(
                (*state).header_size.wrapping_add(48 as u32).card_rem(sector_size),
            ) as s32;
    if remaining <= 0 as i32 {
        return 1 as u32;
    }
    return (remaining as u32)
        .wrapping_add(sector_size)
        .wrapping_sub(0x21 as u32)
        .card_div(usable)
        .wrapping_add(1 as u32);
}
pub unsafe extern "C" fn fn_803AC6B8(mut state: *mut CardState, mut file_idx: s32) -> s32 {
    melee_core::ct!("fn_803AC6B8");
    let mut sector_size: u32 = 0;
    let mut remaining: s32 = 0;
    let mut usable: u32 = 0;
    let mut blocks: s32 = 0;
    let mut file_size: s32 = 0;
    let mut total: s32 = 0;
    let mut i: s32 = 0;
    if file_idx >= 9 as i32 {
        return 0 as s32;
    }
    if file_idx == 0 as i32 {
        return 0 as s32;
    }
    total = 1 as i32 as s32;
    if (*state).file_sizes[0 as i32 as usize] > 0 as i32 {
        total = (total as u32)
            .wrapping_add(fn_803AC6B8_first_block_count(state).wrapping_sub(1 as u32))
            as s32 as s32;
    }
    i = 1 as i32 as s32;
    while i < file_idx {
        file_size = (*state).file_sizes[i as usize] as s32;
        if file_size <= 0 as i32 {
            blocks = 0 as i32 as s32;
        } else if i == 0 as i32 {
            sector_size = (*state).sector_size;
            usable = sector_size.wrapping_sub(0x20 as u32);
            remaining = (*state).file_sizes[0 as i32 as usize]
                - usable
                    .wrapping_sub(
                        (*state)
                            .header_size
                            .wrapping_add(48 as u32)
                            .card_rem(sector_size),
                    ) as s32;
            if remaining <= 0 as i32 {
                blocks = 1 as i32 as s32;
            } else {
                blocks = (remaining as u32)
                    .wrapping_add(sector_size)
                    .wrapping_sub(0x21 as u32)
                    .card_div(usable)
                    .wrapping_add(1 as u32) as s32;
            }
        } else {
            sector_size = (*state).sector_size;
            blocks = (file_size as u32)
                .wrapping_add(sector_size)
                .wrapping_sub(0x21 as u32)
                .card_div(sector_size.wrapping_sub(0x20 as u32)) as s32;
        }
        total += blocks;
        i += 1;
    }
    return total;
}
#[inline]
pub unsafe extern "C" fn fn_803AC6B8_blocks_before(
    mut state: *mut CardState,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803AC6B8_blocks_before");
    let mut total: s32 = 0;
    let mut i: s32 = 0;
    if file_idx >= 9 as i32 {
        return 0 as s32;
    }
    if file_idx == 0 as i32 {
        return 0 as s32;
    }
    total = 1 as i32 as s32;
    if (*state).file_sizes[0 as i32 as usize] > 0 as i32 {
        total = (total as u32)
            .wrapping_add(fn_803AC634(state, 0 as s32).wrapping_sub(1 as u32)) as s32
            as s32;
    }
    i = 1 as i32 as s32;
    while i < file_idx {
        total = (total as u32).wrapping_add(fn_803AC634(state, i)) as s32 as s32;
        i += 1;
    }
    return total;
}
#[inline]
pub unsafe extern "C" fn fn_803AC7DC_block_count(
    mut state: *mut CardState,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803AC7DC_block_count");
    let mut blocks: s32 = 0;
    if (*state).file_sizes[file_idx as usize] <= 0 as i32 {
        blocks = 0 as i32 as s32;
    } else if file_idx == 0 as i32 {
        let mut usable: u32 = 0;
        let mut sector_size: u32 = (*state).sector_size;
        let mut remaining: s32 = 0;
        remaining = (*state).file_sizes[0 as i32 as usize] as s32;
        usable = sector_size.wrapping_sub(0x20 as u32);
        remaining = remaining
            - usable
                .wrapping_sub(
                    (*state)
                        .header_size
                        .wrapping_add(48 as u32)
                        .card_rem(sector_size),
                ) as s32;
        if remaining <= 0 as i32 {
            blocks = 1 as i32 as s32;
        } else {
            blocks = (remaining as u32)
                .wrapping_add(sector_size)
                .wrapping_sub(0x21 as u32)
                .card_div(usable)
                .wrapping_add(1 as u32) as s32;
        }
    } else {
        let mut sector_size_0: u32 = (*state).sector_size;
        blocks = ((*state).file_sizes[file_idx as usize] as u32)
            .wrapping_add(sector_size_0)
            .wrapping_sub(0x21 as u32)
            .card_div(sector_size_0.wrapping_sub(0x20 as u32)) as s32;
    }
    return blocks;
}
pub unsafe extern "C" fn fn_803AC7DC(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803AC7DC");
    let mut total: s32 = 0 as s32;
    let mut spare_blocks: s32 = 0 as s32;
    let mut i: s32 = 0;
    if (*state).file_sizes[0 as i32 as usize] > 0 as i32 {
        total = (fn_803AC7DC_block_count(state, 0 as s32) as i32
            - 1 as i32) as s32;
    }
    i = 1 as i32 as s32;
    while i < 9 as i32 {
        let mut size: s32 = (*state).file_sizes[i as usize];
        if !(size <= 0 as i32) {
            match (*state).file_flags[i as usize] {
                0 => {
                    total
                        += fn_803AC7DC_block_count(state, i) as i32
                            * 2 as i32;
                }
                1 => {
                    total += fn_803AC7DC_block_count(state, i);
                    if spare_blocks < fn_803AC7DC_block_count(state, i) {
                        spare_blocks = fn_803AC7DC_block_count(state, i);
                    }
                }
                2 => {
                    total += fn_803AC7DC_block_count(state, i);
                    if spare_blocks < 1 as i32 {
                        spare_blocks = 1 as i32 as s32;
                    }
                }
                3 => {
                    total += fn_803AC7DC_block_count(state, i);
                }
                _ => {}
            }
        }
        i += 1;
    }
    return total + spare_blocks;
}
#[inline]
pub unsafe extern "C" fn fn_803ACB74(mut seq_a: s32, mut seq_b: s32) -> s32 {
    melee_core::ct!("fn_803ACB74");
    if seq_a < 0 as i32 {
        return -(1 as s32);
    }
    if seq_b < 0 as i32 {
        return 1 as s32;
    }
    if seq_a == 0 as i32 && seq_b == 0xff as i32 {
        return 1 as s32;
    }
    if seq_a == 0xff as i32 && seq_b == 0 as i32 {
        return -(1 as s32);
    }
    if seq_a - seq_b > 0x80 as i32 {
        return -(1 as s32);
    }
    if seq_a - seq_b < -(0x80 as i32) {
        return 1 as s32;
    }
    return seq_a - seq_b;
}
pub unsafe extern "C" fn fn_803ACBE8(mut state: *mut CardState, mut block_idx: s32) -> s32 {
    melee_core::ct!("fn_803ACBE8");
    let mut sector_size: u32 = (*state).sector_size;
    let mut temp: u32 = (*state).header_size.wrapping_add(sector_size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    temp = num.card_div(sector_size);
    let mut sector: u32 = temp.wrapping_sub(1 as u32);
    sector = (block_idx as u32).wrapping_add(sector);
    return sector_size.wrapping_mul(sector) as s32;
}
pub unsafe extern "C" fn fn_803ACC0C(
    mut state: *mut CardState,
    mut block_idx: s32,
    mut block_id: s32,
    mut seq_num: s32,
    mut expected_data: *mut c_void,
    mut data_size: s32,
) -> s32 {
    melee_core::ct!("fn_803ACC0C");
    let mut result: s32 = 0;
    let mut retries: s32 = 0;
    let mut buf: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut sector_size: u32 = 0;
    let mut hdr: *mut CardBlockHeader = ::core::ptr::null_mut::<CardBlockHeader>();
    let mut read_offset: u32 = 0;
    let mut hdr_offset: s32 = 0;
    if data_size == 0 as i32 {
        return 0 as s32;
    }
    sector_size = (*state).sector_size;
    buf = (*state).sector_buf;
    let mut temp: u32 = (*state).header_size.wrapping_add(sector_size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    temp = num.card_div(sector_size);
    let mut idx: u32 = temp.wrapping_sub(1 as u32);
    idx = (block_idx as u32).wrapping_add(idx);
    read_offset = sector_size.wrapping_mul(idx);
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        let mut tmp: s32 = CARDRead(
            &raw mut (*state).file_info,
            buf as *mut c_void,
            sector_size as i32,
            read_offset as i32,
        ) as s32;
        result = tmp;
        if result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    if result < 0 as i32 {
        return result;
    }
    if block_idx == 0 as i32 {
        hdr_offset = (*state)
            .header_size
            .wrapping_add(48 as u32)
            .card_rem((*state).sector_size) as s32;
    } else {
        hdr_offset = 0 as i32 as s32;
    }
    if HSD_Decrypt(
        (*state).sector_buf.offset(hdr_offset as isize),
        (*state).sector_size.wrapping_sub(hdr_offset as u32) as i32,
    ) < 0 as i32
    {
        return 1 as s32;
    }
    hdr = (*state).sector_buf.offset(hdr_offset as isize) as *mut CardBlockHeader;
    if block_id
        != ((*hdr).id_hi as i32) << 8 as i32
            | (*hdr).id_lo as i32
    {
        return 1 as s32;
    }
    if seq_num != (*hdr).seq as i32 {
        return 1 as s32;
    }
    if memcmp(
        expected_data,
        &raw mut (*hdr).data as *mut u8 as *const c_void,
        data_size as usize,
    ) != 0 as i32
    {
        return 1 as s32;
    }
    return 0 as s32;
}
pub unsafe extern "C" fn fn_803ACD58(
    mut state: *mut CardState,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
) -> s32 {
    melee_core::ct!("fn_803ACD58");
    let mut banner_size: s32 = 0;
    let mut i: s32 = 0;
    let mut icons_start: s32 = 0;
    match (*state).icon_info.banner_format as i32 {
        2 => {
            banner_size = BANNER_LARGE as s32;
        }
        1 => {
            banner_size = BANNER_SMALL as s32;
        }
        _ => {
            banner_size = 0 as i32 as s32;
        }
    }
    icons_start = (banner_size as i32 + 0x40 as i32)
        as s32;
    let mut retries: s32 = 0;
    let mut buf: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut offset: s32 = 0;
    let mut size: s32 = 0;
    i = 0 as i32 as s32;
    loop {
        size = (*state).sector_size as s32;
        if !((i as u32)
            < (0x2f as u32)
                .wrapping_add((*state).header_size)
                .wrapping_add(size as u32)
                .card_div(size as u32))
        {
            break;
        }
        let mut result: s32 = 0;
        offset = i * size;
        buf = (*state).sector_buf;
        retries = 0 as i32 as s32;
        while retries < 10 as i32 {
            result = CARDRead(
                &raw mut (*state).file_info,
                buf as *mut c_void,
                size as i32,
                offset as i32,
            ) as s32;
            if result != -(1 as i32) {
                break;
            }
            retries += 1;
        }
        if result < 0 as i32 {
            return result;
        }
        if i == 0 as i32 {
            if memcmp(
                (*state).sector_buf as *const c_void,
                &raw mut (*state).comment as *mut i8
                    as *const c_void,
                0x40 as usize,
            ) != 0 as i32
            {
                return 1 as s32;
            }
            if banner_size > 0 as i32 {
                if memcmp(
                    (*state).sector_buf.offset(0x40 as i32 as isize)
                        as *const c_void,
                    banner,
                    banner_size as usize,
                ) != 0 as i32
                {
                    return 1 as s32;
                }
            }
            let mut header_size: u32 = (*state).header_size;
            let mut sector_size: u32 = (*state).sector_size;
            if header_size > sector_size {
                if memcmp(
                    (*state).sector_buf.offset(icons_start as isize)
                        as *const c_void,
                    icons,
                    sector_size.wrapping_sub(icons_start as u32) as usize,
                ) != 0 as i32
                {
                    return 1 as s32;
                }
            } else if memcmp(
                (*state).sector_buf.offset(icons_start as isize)
                    as *const c_void,
                icons,
                header_size.wrapping_sub(icons_start as u32) as usize,
            ) != 0 as i32
            {
                return 1 as s32
            }
        } else {
            let mut sector_size_0: u32 = (*state).sector_size;
            let mut remaining: u32 = (*state)
                .header_size
                .wrapping_sub((i as u32).wrapping_mul(sector_size_0));
            let mut icons_offset: s32 = (i as u32)
                .wrapping_mul(sector_size_0)
                .wrapping_sub(0x40 as u32)
                .wrapping_sub(banner_size as u32) as s32;
            if remaining > sector_size_0 {
                if memcmp(
                    (*state).sector_buf as *const c_void,
                    (icons as *mut u8).offset(icons_offset as isize)
                        as *const c_void,
                    sector_size_0 as usize,
                ) != 0 as i32
                {
                    return 1 as s32;
                }
            } else if memcmp(
                (*state).sector_buf as *const c_void,
                (icons as *mut u8).offset(icons_offset as isize)
                    as *const c_void,
                remaining as usize,
            ) != 0 as i32
            {
                return 1 as s32
            }
        }
        i += 1;
    }
    return 0 as s32;
}
pub unsafe extern "C" fn fn_803ACF30(
    mut state: *mut CardState,
    mut comment: *mut c_void,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
) -> s32 {
    melee_core::ct!("fn_803ACF30");
    let mut buf: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut ret: s32 = 0;
    let mut i: u32 = 0;
    i = 0 as u32;
    while i
        < (0x2f as u32)
            .wrapping_add((*state).header_size)
            .wrapping_add((*state).sector_size)
            .card_div((*state).sector_size)
    {
        buf.r#type = CARD_CMD_READ_HEADER;
        buf.state = state;
        buf.x8.read_header.index = i as s32;
        buf.x8.read_header.comment = comment;
        buf.x8.read_header.banner = banner;
        buf.x8.read_header.icons = icons;
        ret = fn_803AC168(&raw mut buf) as s32;
        if ret < 0 as i32 {
            return ret;
        }
        i = i.wrapping_add(1);
    }
    return 0 as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803ACFC0_header(
    mut state: *mut CardState,
    mut hdr_offset: s32,
) -> *mut u8 {
    melee_core::ct!("fn_803ACFC0_header");
    return (*state).sector_buf.offset(hdr_offset as isize);
}
#[inline]
pub unsafe extern "C" fn fn_803ACFC0_checksum_start(
    mut hdr_offset: s32,
    mut state: *mut CardState,
) -> *mut u8 {
    melee_core::ct!("fn_803ACFC0_checksum_start");
    return (*state).sector_buf.offset(hdr_offset as isize);
}
pub unsafe extern "C" fn fn_803ACFC0(
    mut state: *mut CardState,
    mut block_idx: s32,
    mut block_id: s32,
    mut seq_num: s32,
    mut payload: *mut c_void,
    mut payload_size: s32,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803ACFC0");
    let mut sector_size: u32 = 0;
    let mut retries: s32 = 0;
    let mut buf: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut offset: s32 = 0;
    let mut result: s32 = 0;
    let mut hdr_offset: s32 = 0;
    sector_size = (*state).sector_size;
    let mut data_start: u32 = (*state).header_size.wrapping_add(48 as u32);
    offset = sector_size
        .wrapping_mul(
            (block_idx as u32)
                .wrapping_add(
                    data_start
                        .wrapping_add(sector_size)
                        .wrapping_sub(1 as u32)
                        .card_div(sector_size),
                )
                .wrapping_sub(1 as u32),
        ) as s32;
    if block_idx == 0 as i32 {
        if block_id != 0 as i32 {
            return -(257 as s32);
        }
        buf = (*state).sector_buf;
        hdr_offset = data_start.card_rem(sector_size) as s32;
        retries = 0 as i32 as s32;
        while retries < 10 as i32 {
            result = CARDRead(
                &raw mut (*state).file_info,
                buf as *mut c_void,
                sector_size as i32,
                offset as i32,
            ) as s32;
            if result != -(1 as i32) {
                break;
            }
            retries += 1;
        }
        if result < 0 as i32 {
            return result;
        }
    } else {
        hdr_offset = 0 as i32 as s32;
    }
    if payload_size > 0 as i32 {
        memcpy(
            (fn_803ACFC0_header
                as unsafe extern "C" fn(
                    *mut CardState,
                    s32,
                ) -> *mut u8)(state, hdr_offset)
                .offset(0x20 as i32 as isize) as *mut u8
                as *mut c_void,
            payload,
            payload_size as usize,
        );
    }
    let mut remaining: s32 = (*state)
        .sector_size
        .wrapping_sub(hdr_offset as u32)
        .wrapping_sub(payload_size as u32)
        .wrapping_sub(0x20 as u32) as s32;
    if remaining != 0 as i32 {
        let mut off: s32 = payload_size + 0x20 as s32;
        
        memset(
            (*state).sector_buf.offset((hdr_offset + off) as isize).cast::<c_void>(),
            0 as i32,
            remaining as usize,
        );
    }
    memset(
        (*state).sector_buf.offset(hdr_offset as isize) as *mut c_void,
        0 as i32,
        0x20 as usize,
    );
    *(*state)
        .sector_buf
        .offset(hdr_offset as isize)
        .offset(0x10 as i32 as isize) = (block_id
        >> 8 as i32) as u8;
    *(*state)
        .sector_buf
        .offset(hdr_offset as isize)
        .offset(0x11 as i32 as isize) = block_id as u8;
    *(*state)
        .sector_buf
        .offset(hdr_offset as isize)
        .offset(0x12 as i32 as isize) = seq_num as u8;
    fn_803AC3F8(
        state,
        fn_803ACFC0_header(state, hdr_offset)
            .offset(0x13 as i32 as isize),
        file_idx,
    );
    HSD_Encrypt(
        fn_803ACFC0_checksum_start(hdr_offset, state),
        (*state).sector_size.wrapping_sub(hdr_offset as u32) as i32,
    );
    let mut write_retries: s32 = 0;
    let mut write_buf: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut write_size: u32 = 0;
    write_size = (*state).sector_size;
    write_retries = 0 as i32 as s32;
    write_buf = (*state).sector_buf;
    while write_retries < 10 as i32 {
        result = CARDWrite(
            &raw mut (*state).file_info,
            write_buf as *mut c_void,
            write_size as s32,
            offset,
        );
        if result != -(1 as i32) {
            break;
        }
        write_retries += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_total_blocks(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803AD16C_total_blocks");
    return (*state).num_blocks;
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_logical_index(
    mut blocks_before: s32,
    mut i: s32,
) -> s32 {
    melee_core::ct!("fn_803AD16C_logical_index");
    return blocks_before + i;
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_file_size(
    mut state: *mut CardState,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803AD16C_file_size");
    return (*state).file_sizes[file_idx as usize];
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_nonnegative(mut logical: s32) -> i32 {
    melee_core::ct!("fn_803AD16C_nonnegative");
    return (logical >= 0 as i32) as i32;
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_own(mut value: *mut c_void) { melee_core::ct!("fn_803AD16C_own");}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_queue_clear(
    mut state: *mut CardState,
    mut phys: s32,
    mut offset: s32,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803AD16C_queue_clear");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    cmd.r#type = CARD_CMD_WRITE_BLOCK;
    cmd.state = state;
    cmd.x8.write.phys = phys;
    cmd.x8.write.block_id = 0xffff as i32 as s32;
    cmd.x8.write.seq = 0 as i32 as s32;
    cmd.x8.write.data = ::core::ptr::null_mut::<c_void>();
    cmd.x8.write.size = 0 as i32 as s32;
    cmd.x8.write.offset = offset;
    cmd.x8.write.file_idx = file_idx;
    return fn_803AC168(&raw mut cmd) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_queue_read(
    mut state: *mut CardState,
    mut phys: s32,
) -> s32 {
    melee_core::ct!("fn_803AD16C_queue_read");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut size: u32 = (*state).sector_size;
    let mut temp: u32 = (*state).header_size.wrapping_add(size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    let mut idx: u32 = 0;
    temp = num.card_div(size);
    idx = temp.wrapping_sub(1 as u32);
    idx = (phys as u32).wrapping_add(idx);
    cmd.r#type = CARD_CMD_READ_SECTOR;
    cmd.state = state;
    cmd.x8.sector.phys = phys;
    cmd.x8.sector.offset = size.wrapping_mul(idx) as s32;
    return fn_803AC168(&raw mut cmd) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_queue_write(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
) -> s32 {
    melee_core::ct!("fn_803AD16C_queue_write");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut size: u32 = (*state).sector_size;
    let mut temp: u32 = (*state).header_size.wrapping_add(size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    let mut idx: u32 = 0;
    temp = num.card_div(size);
    idx = temp.wrapping_sub(1 as u32);
    idx = (phys as u32).wrapping_add(idx);
    cmd.r#type = CARD_CMD_WRITE_SECTOR;
    cmd.state = state;
    cmd.x8.sector.phys = phys;
    cmd.x8.sector.block_id = block_id;
    cmd.x8.sector.seq = seq;
    cmd.x8.sector.offset = size.wrapping_mul(idx) as s32;
    return fn_803AC168(&raw mut cmd) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803AD16C_queue_write_last(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
) -> s32 {
    melee_core::ct!("fn_803AD16C_queue_write_last");
    let mut tail: [s32; 2] = [0; 2];
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut size: u32 = (*state).sector_size;
    let mut temp: u32 = (*state).header_size.wrapping_add(size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    let mut idx: u32 = 0;
    temp = num.card_div(size);
    idx = temp.wrapping_sub(1 as u32);
    idx = (phys as u32).wrapping_add(idx);
    cmd.r#type = CARD_CMD_WRITE_SECTOR;
    cmd.state = state;
    cmd.x8.sector.phys = phys;
    cmd.x8.sector.block_id = block_id;
    cmd.x8.sector.seq = seq;
    cmd.x8.sector.offset = size.wrapping_mul(idx) as s32;
    fn_803AD16C_own(&raw mut tail as *mut s32 as *mut c_void);
    return fn_803AC168(&raw mut cmd) as s32;
}
pub unsafe extern "C" fn fn_803AD16C(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803AD16C");
    let mut ids: [s32; 64] = [0; 64];
    let mut seq: [s32; 64] = [0; 64];
    let mut newmap: [s32; 64] = [0; 64];
    let mut offset: s32 = 0;
    let mut filemap: [s32; 64] = [0; 64];
    let mut chosen: [s32; 64] = [0; 64];
    let mut result: s32 = 0;
    let mut file_idx: s32 = 0;
    let mut pass: s32 = 0;
    let mut i: s32 = 0;
    let mut phys: s32 = 0;
    let mut ofs: s32 = 0;
    let mut blocks_before: s32 = 0;
    let mut file_blocks: s32 = 0;
    result = 0 as i32 as s32;
    i = 0 as i32 as s32;
    while i <= (*state).num_blocks {
        newmap[i as usize] = -(1 as i32) as s32;
        ids[i as usize] = (*state).block_ids[i as usize];
        seq[i as usize] = (*state).block_seqs[i as usize];
        i += 1;
    }
    if (*state).num_blocks != fn_803AC7DC(state) {
        return -(257 as s32);
    }
    file_idx = 0 as i32 as s32;
    while file_idx < 9 as i32 {
        file_blocks = fn_803AC634(state, file_idx) as s32;
        if !(file_blocks <= 0 as i32) {
            blocks_before = fn_803AC6B8_blocks_before(state, file_idx);
            i = 0 as i32 as s32;
            while i < file_blocks {
                filemap[i as usize] = -(1 as i32) as s32;
                chosen[i as usize] = -(1 as i32) as s32;
                i += 1;
            }
            pass = 0 as i32 as s32;
            while pass <= (*state).num_blocks {
                let mut cur_seq: s32 = -(1 as s32);
                phys = 0 as i32 as s32;
                while phys <= fn_803AD16C_total_blocks(state) {
                    if ids[phys as usize] >= 0 as i32 {
                        let mut logical: s32 = ids[phys as usize] - blocks_before;
                        if logical >= 0 as i32 && logical < file_blocks {
                            if cur_seq == -(1 as i32)
                                || fn_803ACB74(cur_seq, seq[phys as usize])
                                    > 0 as i32
                            {
                                cur_seq = seq[phys as usize];
                            }
                        }
                    }
                    phys += 1;
                }
                if cur_seq < 0 as i32 {
                    break;
                }
                phys = 0 as i32 as s32;
                while phys <= (*state).num_blocks {
                    let mut logical_0: s32 = ids[phys as usize] - blocks_before;
                    if fn_803AD16C_nonnegative(logical_0) != 0 && logical_0 < file_blocks
                        && cur_seq == seq[phys as usize]
                    {
                        filemap[logical_0 as usize] = phys;
                    }
                    phys += 1;
                }
                i = 0 as i32 as s32;
                while i < file_blocks {
                    if filemap[i as usize] < 0 as i32
                        || cur_seq != seq[filemap[i as usize] as usize]
                    {
                        break;
                    }
                    i += 1;
                }
                if i == file_blocks {
                    i = 0 as i32 as s32;
                    while i < file_blocks {
                        chosen[i as usize] = filemap[i as usize];
                        i += 1;
                    }
                }
                i = 0 as i32 as s32;
                while i < file_blocks {
                    if filemap[i as usize] >= 0 as i32 {
                        ids[filemap[i as usize] as usize] = -(0x7fff
                            as i32) as s32;
                    }
                    i += 1;
                }
                pass += 1;
            }
            if chosen[0 as i32 as usize] >= 0 as i32 {
                i = 0 as i32 as s32;
                while i < file_blocks {
                    newmap[(blocks_before + i) as usize] = chosen[i as usize];
                    i += 1;
                }
            } else {
                i = 0 as i32 as s32;
                while i < file_blocks {
                    let mut tmp: s32 = filemap[i as usize];
                    if tmp < 0 as i32 {
                        result = -(0x103 as i32) as s32;
                    } else {
                        newmap[(blocks_before + i) as usize] = filemap[i as usize];
                        if result != -(0x103 as i32) {
                            result = -(0x104 as i32) as s32;
                        }
                    }
                    i += 1;
                }
            }
        }
        file_idx += 1;
    }
    file_idx = 0 as i32 as s32;
    while file_idx < 9 as i32 {
        if !(fn_803AD16C_file_size(state, file_idx) <= 0 as i32
            || (*state).file_flags[file_idx as usize] != 1 as i32
                && (*state).file_flags[file_idx as usize] != 2 as i32)
        {
            file_blocks = fn_803AC634(state, file_idx) as s32;
            blocks_before = fn_803AC6B8_blocks_before(state, file_idx);
            i = 0 as i32 as s32;
            while i < file_blocks {
                let mut block_id: s32 = fn_803AD16C_logical_index(blocks_before, i);
                if newmap[block_id as usize] >= 0 as i32 {
                    phys = 1 as i32 as s32;
                    while phys <= (*state).num_blocks {
                        if phys != newmap[block_id as usize]
                            && (*state).block_ids[phys as usize] == block_id
                        {
                            (*state).block_ids[phys as usize] = -block_id;
                        }
                        phys += 1;
                    }
                }
                i += 1;
            }
        }
        file_idx += 1;
    }
    file_idx = 0 as i32 as s32;
    while file_idx < 9 as i32 {
        if !((*state).file_sizes[file_idx as usize] <= 0 as i32
            || (*state).file_flags[file_idx as usize] != 0 as i32)
        {
            file_blocks = fn_803AC634(state, file_idx) as s32;
            blocks_before = fn_803AC6B8_blocks_before(state, file_idx);
            i = 0 as i32 as s32;
            while i < file_blocks {
                let mut block_id_0: s32 = blocks_before + i;
                let mut src: s32 = newmap[block_id_0 as usize];
                let mut dup: s32 = 0;
                let mut target_seq: s32 = 0;
                let mut ret: s32 = 0;
                if !(newmap[block_id_0 as usize] < 0 as i32) {
                    target_seq = (*state).block_seqs[src as usize];
                    dup = -(1 as i32) as s32;
                    phys = 1 as i32 as s32;
                    while phys <= (*state).num_blocks {
                        if phys != newmap[block_id_0 as usize]
                            && (*state).block_ids[phys as usize] == block_id_0
                        {
                            if dup < 0 as i32 {
                                dup = phys;
                            } else {
                                offset = fn_803ACBE8(state, phys);
                                if phys == 0 as i32 {
                                    ret = -(0x101 as i32) as s32;
                                } else {
                                    ret = fn_803AD16C_queue_clear(
                                        state,
                                        phys,
                                        offset,
                                        file_idx,
                                    );
                                }
                                if ret < 0 as i32
                                    && result == 0 as i32
                                {
                                    result = -(0x10b as i32) as s32;
                                }
                                (*state).block_ids[phys as usize] = -(0x7fff
                                    as i32) as s32;
                                (*state).block_seqs[phys as usize] = 0 as i32
                                    as s32;
                            }
                        }
                        phys += 1;
                    }
                    if dup == -(1 as i32) {
                        phys = (*state).num_blocks;
                        while phys > 0 as i32 {
                            if (*state).block_ids[phys as usize]
                                < 0 as i32
                            {
                                dup = phys;
                                break;
                            } else {
                                phys -= 1;
                            }
                        }
                        if dup >= 0 as i32 {
                            if newmap[block_id_0 as usize] == 0 as i32
                                || dup == 0 as i32
                            {
                                ret = -(0x101 as i32) as s32;
                            } else {
                                ret = fn_803AD16C_queue_read(
                                    state,
                                    newmap[block_id_0 as usize],
                                );
                                if ret < 0 as i32 {
                                    let _ = ret == -(0x101 as i32);
                                } else {
                                    ret = fn_803AD16C_queue_write(
                                        state,
                                        dup,
                                        block_id_0,
                                        target_seq,
                                    );
                                }
                            }
                            if ret < 0 as i32
                                && (result == 0 as i32
                                    || result == -(0x10b as i32))
                            {
                                result = -(0x102 as i32) as s32;
                            }
                        }
                    } else if (*state).block_seqs[dup as usize] != target_seq {
                        if newmap[block_id_0 as usize] == 0 as i32
                            || dup == 0 as i32
                        {
                            ret = -(0x101 as i32) as s32;
                        } else {
                            let mut cmd_result: s32 = fn_803AD16C_queue_read(
                                state,
                                newmap[block_id_0 as usize],
                            );
                            ret = cmd_result;
                            if ret < 0 as i32 {
                                let _ = ret == -(0x101 as i32);
                            } else {
                                ret = fn_803AD16C_queue_write_last(
                                    state,
                                    dup,
                                    block_id_0,
                                    target_seq,
                                );
                            }
                        }
                        if ret < 0 as i32
                            && (result == 0 as i32
                                || result == -(0x10b as i32))
                        {
                            result = -(0x102 as i32) as s32;
                        }
                    }
                }
                i += 1;
            }
        }
        file_idx += 1;
    }
    return result;
}
pub unsafe extern "C" fn fn_803ADE4C(
    mut state: *mut CardState,
    mut file_no: i32,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803ADE4C");
    let mut cmd_open: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut cmd_scan: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut result: s32 = 0;
    let mut saved: s32 = 0;
    let mut snap: s32 = 0;
    let mut saved1: s32 = 0;
    let mut snap1: s32 = 0;
    hsd_804D7998 = curr_tail as i32;
    cmd_open.r#type = CARD_CMD_GET_STATUS;
    cmd_open.state = state;
    cmd_open.x8.get_status.file_no = file_no as s32;
    result = fn_803AC168(&raw mut cmd_open) as s32;
    if result < 0 as i32 {
        snap1 = hsd_804D7998 as s32;
        if snap1 >= 0 as i32 {
            saved1 = snap1;
            while saved1 != curr_tail {
                commands[saved1 as usize].r#type = CARD_CMD_NONE;
                saved1 = ((saved1 as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap1);
        }
        return result as i32;
    }
    cmd_scan.r#type = CARD_CMD_SCAN_FILE;
    cmd_scan.state = state;
    result = fn_803AC168(&raw mut cmd_scan) as s32;
    if result < 0 as i32 {
        snap = hsd_804D7998 as s32;
        if snap >= 0 as i32 {
            saved = snap;
            while saved != curr_tail {
                commands[saved as usize].r#type = CARD_CMD_NONE;
                saved = ((saved as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
        }
        return result as i32;
    }
    active_requests.r#type = CARD_ACTIVE_OPEN_OR_READ_HEADER;
    active_requests.state = state;
    active_requests.callback = callback;
    active_requests.callback_arg = 0 as i32 as s32;
    hsd_804D7998 = -(1 as i32);
    return 0 as i32;
}
#[inline]
pub unsafe extern "C" fn queueCardReadCommand(
    mut state: *mut CardState,
    mut block: s32,
    mut data: *mut c_void,
    mut length: s32,
    mut offset: s32,
) -> s32 {
    melee_core::ct!("queueCardReadCommand");
    let mut command: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    command.r#type = CARD_CMD_READ_BLOCK;
    command.state = state;
    command.x8.read.phys = block;
    command.x8.read.xC = 0 as i32 as s32;
    command.x8.read.data = data;
    command.x8.read.size = length;
    command.x8.read.offset = offset;
    return fn_803AC168(&raw mut command) as s32;
}
#[inline]
pub unsafe extern "C" fn calculateDataBlockSize(
    mut state: *mut CardState,
    mut file_idx: s32,
    mut block_idx: s32,
) -> s32 {
    melee_core::ct!("calculateDataBlockSize");
    if file_idx == 0 as i32 && block_idx == 0 as i32 {
        return (*state)
            .sector_size
            .wrapping_sub(0x20 as u32)
            .wrapping_sub(
                (*state)
                    .header_size
                    .wrapping_add(0x30 as u32)
                    .card_rem((*state).sector_size),
            ) as s32;
    }
    return (*state).sector_size.wrapping_sub(0x20 as u32) as s32;
}
#[inline]
pub unsafe extern "C" fn calculateFileBlockCount(
    mut state: *mut CardState,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("calculateFileBlockCount");
    let mut file_size: s32 = (*state).file_sizes[file_idx as usize];
    if file_size <= 0 as i32 {
        return 0 as s32
    } else if file_idx == 0 as i32 {
        let mut sector_size: u32 = (*state).sector_size;
        let mut usable: u32 = 0;
        let mut rem: s32 = 0;
        rem = (*state).file_sizes[0 as i32 as usize] as s32;
        usable = sector_size.wrapping_sub(0x20 as u32);
        rem = rem
            - usable
                .wrapping_sub(
                    (*state)
                        .header_size
                        .wrapping_add(0x30 as u32)
                        .card_rem(sector_size),
                ) as s32;
        if rem <= 0 as i32 {
            return 1 as s32
        } else {
            return (rem as u32)
                .wrapping_add(sector_size)
                .wrapping_sub(0x21 as u32)
                .card_div(sector_size.wrapping_sub(0x20 as u32))
                .wrapping_add(1 as u32) as s32
        }
    } else {
        let mut sector_size_0: u32 = (*state).sector_size;
        return (file_size as u32)
            .wrapping_add(sector_size_0)
            .wrapping_sub(0x21 as u32)
            .card_div(sector_size_0.wrapping_sub(0x20 as u32)) as s32;
    };
}
#[inline]
pub unsafe extern "C" fn retryCardRead(
    mut info: *mut CARDFileInfo,
    mut buffer: *mut c_void,
    mut length: s32,
    mut offset: s32,
) -> s32 {
    melee_core::ct!("retryCardRead");
    let mut result: s32 = 0;
    let mut retries: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        result = CARDRead(
            info as *mut CARDFileInfo,
            buffer,
            length as i32,
            offset as i32,
        ) as s32;
        if result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn cancelQueuedCardCommands() {
    melee_core::ct!("cancelQueuedCardCommands");
    let mut snap: s32 = hsd_804D7998 as s32;
    if snap >= 0 as i32 {
        let mut saved: s32 = snap;
        let mut zero: s32 = 0;
        while saved != curr_tail {
            let mut queued: *mut CardCmd = (&raw mut commands as *mut CardCmd)
                .offset(saved as isize) as *mut CardCmd;
            saved = ((saved as i32 + 1 as i32)
                % 128 as i32) as s32;
            zero = 0 as i32 as s32;
            (*queued).r#type = zero as CardCmdType;
        }
        ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
    }
}
#[inline]
pub unsafe extern "C" fn queueCardClearCommand(
    mut state: *mut CardState,
    mut dst: *mut u8,
    mut size: s32,
) -> s32 {
    melee_core::ct!("queueCardClearCommand");
    let mut command: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    command.r#type = CARD_CMD_CLEAR_BUF;
    command.state = state;
    command.x8.clear.x8 = 0 as i32 as s32;
    command.x8.clear.xC = 0 as i32 as s32;
    command.x8.clear.data = dst as *mut c_void;
    command.x8.clear.size = size;
    command.x8.clear.x14 = 0 as i32 as s32;
    command.x8.clear.x0 = 0 as i32 as s32;
    return fn_803AC168(&raw mut command) as s32;
}
#[inline]
pub unsafe extern "C" fn queueClearDataBlock(
    mut state: *mut CardState,
    mut dst: *mut u8,
    mut size: s32,
) -> s32 {
    melee_core::ct!("queueClearDataBlock");
    return if size == 0 as i32 {
        0 as s32
    } else {
        queueCardClearCommand(state, dst, size)
    };
}
#[inline]
pub unsafe extern "C" fn cardDataBlockOffset(
    mut state: *const CardState,
    mut sector_size: u32,
    mut data_block: s32,
) -> s32 {
    melee_core::ct!("cardDataBlockOffset");
    let mut temp: u32 = (*state).header_size.wrapping_add(sector_size);
    let mut idx: u32 = 0;
    temp = temp.wrapping_add(0x2f as u32).card_div(sector_size);
    idx = temp.wrapping_sub(1 as u32);
    idx = (data_block as u32).wrapping_add(idx);
    return sector_size.wrapping_mul(idx) as s32;
}
#[inline]
pub unsafe extern "C" fn queueReadDataBlock(
    mut state: *mut CardState,
    mut block_idx: s32,
    mut dst: *mut u8,
    mut size: s32,
) -> i32 {
    melee_core::ct!("queueReadDataBlock");
    let mut sector_size: u32 = (*state).sector_size;
    let mut temp: u32 = (*state).header_size.wrapping_add(sector_size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    let mut idx: u32 = 0;
    temp = num.card_div(sector_size);
    idx = temp.wrapping_sub(1 as u32);
    idx = (block_idx as u32).wrapping_add(idx);
    return queueCardReadCommand(
        state,
        block_idx,
        dst as *mut c_void,
        size,
        sector_size.wrapping_mul(idx) as s32,
    ) as i32;
}
#[inline]
pub unsafe extern "C" fn readCardDataBlockFirst(
    mut state: *mut CardState,
    mut sector_size: u32,
    mut data_block: s32,
    mut dst: *mut u8,
    mut length: s32,
) -> s32 {
    melee_core::ct!("readCardDataBlockFirst");
    let mut offset: s32 = cardDataBlockOffset(state, sector_size, data_block);
    let mut buf: *mut u8 = (*state).sector_buf;
    let mut hdr_offset: s32 = 0;
    let mut result: i32 = retryCardRead(
        &raw mut (*state).file_info,
        buf as *mut c_void,
        sector_size as s32,
        offset,
    ) as i32;
    if result < 0 as i32 {
        return result as s32;
    }
    hdr_offset = (if data_block == 0 as i32 {
        (*state)
            .header_size
            .wrapping_add(0x30 as u32)
            .card_rem((*state).sector_size)
    } else {
        0 as u32
    }) as s32;
    if HSD_Decrypt(
        (*state).sector_buf.offset(hdr_offset as isize),
        (*state).sector_size.wrapping_sub(hdr_offset as u32) as i32,
    ) < 0 as i32
    {
        return -(0x105 as s32);
    }
    if length != 0 as i32 && !dst.is_null() {
        memcpy(
            dst as *mut c_void,
            (*state)
                .sector_buf
                .offset(
                    (hdr_offset as i32 + 0x20 as i32)
                        as isize,
                ) as *const c_void,
            length as usize,
        );
    }
    return 0 as s32;
}
#[inline]
pub unsafe extern "C" fn readCardDataBlockFinal(
    mut state: *mut CardState,
    mut sector_size: u32,
    mut data_block: s32,
    mut dst: *mut u8,
    mut length: s32,
) -> s32 {
    melee_core::ct!("readCardDataBlockFinal");
    let mut read_offset: u32 = 0;
    let mut buf: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut retries: s32 = 0;
    let mut hdr_offset: s32 = 0;
    let mut result: s32 = 0;
    retries = 0 as i32 as s32;
    buf = (*state).sector_buf;
    let mut temp: u32 = (*state).header_size.wrapping_add(sector_size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    temp = num.card_div(sector_size);
    let mut idx: u32 = temp.wrapping_sub(1 as u32);
    idx = (data_block as u32).wrapping_add(idx);
    read_offset = sector_size.wrapping_mul(idx);
    while retries < 10 as i32 {
        result = CARDRead(
            &raw mut (*state).file_info,
            buf as *mut c_void,
            sector_size as i32,
            read_offset as i32,
        ) as s32;
        if result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    if result < 0 as i32 {
        return result;
    }
    hdr_offset = (if data_block == 0 as i32 {
        (*state)
            .header_size
            .wrapping_add(0x30 as u32)
            .card_rem((*state).sector_size)
    } else {
        0 as u32
    }) as s32;
    if HSD_Decrypt(
        (*state).sector_buf.offset(hdr_offset as isize),
        (*state).sector_size.wrapping_sub(hdr_offset as u32) as i32,
    ) < 0 as i32
    {
        return -(0x105 as s32);
    }
    if length != 0 as i32 && !dst.is_null() {
        memcpy(
            dst as *mut c_void,
            (*state)
                .sector_buf
                .offset(
                    (hdr_offset as i32 + 0x20 as i32)
                        as isize,
                ) as *const c_void,
            length as usize,
        );
    }
    return 0 as s32;
}
pub unsafe extern "C" fn fn_803ADF90(
    mut state: *mut CardState,
    mut file_idx: s32,
    mut buf: *mut u8,
    mut async_0: i32,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803ADF90");
    let mut dst: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut remaining: s32 = 0;
    let mut block_map: [s32; 64] = [0; 64];
    let mut blocks_before: i32 = 0;
    let mut i: i32 = 0;
    let mut logical: i32 = 0;
    let mut scan_idx: i32 = 0;
    let mut result: i32 = 0;
    let mut file_blocks: i32 = 0;
    let mut chunk: s32 = 0;
    let mut total_blocks: s32 = 0;
    let mut file_size: s32 = 0;
    let mut status: s32 = 0;
    status = 0 as i32 as s32;
    if async_0 == 0 as i32 {
        let mut intr: BOOL = OSDisableInterrupts();
        let mut busy: s32 = 0;
        if hsd_804D799C as i32 == 1 as i32 {
            busy = 1 as i32 as s32;
        } else {
            busy = 0 as i32 as s32;
        }
        OSRestoreInterrupts(intr);
        if busy != 0 {
            return -(264 as i32);
        }
    }
    blocks_before = fn_803AC6B8_blocks_before(state, file_idx) as i32;
    file_size = (*state).file_sizes[file_idx as usize] as s32;
    file_blocks = calculateFileBlockCount(state, file_idx) as i32;
    total_blocks = fn_803AC7DC(state);
    i = 0 as i32;
    while i < file_blocks {
        block_map[i as usize] = -(1 as i32) as s32;
        i += 1;
    }
    chunk = -(1 as i32) as s32;
    scan_idx = 1 as i32;
    while scan_idx <= total_blocks {
        logical = (*state).block_ids[scan_idx as usize] - blocks_before;
        if logical >= 0 as i32 && logical < file_blocks {
            if chunk < 0 as i32 {
                chunk = (*state).block_seqs[scan_idx as usize];
            } else {
                if chunk != (*state).block_seqs[scan_idx as usize] {
                    status = -(260 as i32) as s32;
                }
                if fn_803ACB74(chunk, (*state).block_seqs[scan_idx as usize])
                    > 0 as i32
                {
                    chunk = (*state).block_seqs[scan_idx as usize];
                }
            }
            block_map[logical as usize] = scan_idx as s32;
        }
        scan_idx += 1;
    }
    if file_idx == 0 as i32
        && (*state).block_ids[0 as i32 as usize]
            == 0 as i32
    {
        block_map[0 as i32 as usize] = 0 as i32 as s32;
        if chunk >= 0 as i32
            && chunk != (*state).block_seqs[0 as i32 as usize]
        {
            status = -(260 as i32) as s32;
        }
    }
    i = 0 as i32;
    while i < file_blocks {
        if block_map[i as usize] < 0 as i32 {
            status = -(259 as i32) as s32;
            break;
        } else {
            i += 1;
        }
    }
    if status != 0 as i32 {
        return status as i32;
    }
    if async_0 != 0 as i32 {
        hsd_804D7998 = curr_tail as i32;
    } else {
        result = retryCardFastOpen(
            (*state).chan,
            (*state).file_no,
            &raw mut (*state).file_info,
        ) as i32;
        if result < 0 as i32 {
            return result;
        }
    }
    dst = buf;
    remaining = file_size;
    i = 0 as i32;
    while i < file_blocks && remaining > 0 as i32 {
        chunk = calculateDataBlockSize(state, file_idx, i as s32);
        if remaining > chunk {
            let mut phys: s32 = block_map[i as usize];
            let mut sector_size: u32 = 0;
            if phys >= 0 as i32 {
                if async_0 != 0 as i32 {
                    result = queueReadDataBlock(state, phys, dst, chunk);
                    if result < 0 as i32 {
                        cancelQueuedCardCommands();
                        return result;
                    }
                } else {
                    sector_size = (*state).sector_size;
                    result = readCardDataBlockFirst(state, sector_size, phys, dst, chunk)
                        as i32;
                    if result < 0 as i32 {
                        if result == -(0x105 as i32) {
                            status = -(259 as i32) as s32;
                        } else {
                            status = -(259 as i32) as s32;
                        }
                    }
                }
            } else if async_0 != 0 as i32 {
                result = queueClearDataBlock(state, dst, chunk) as i32;
                if result < 0 as i32 {
                    cancelQueuedCardCommands();
                    return result;
                }
            } else {
                memset(
                    dst as *mut c_void,
                    0 as i32,
                    chunk as usize,
                );
            }
            remaining -= chunk;
            dst = dst.offset(chunk as isize);
        } else {
            let mut phys_0: s32 = block_map[i as usize];
            let mut sector_size_0: u32 = 0;
            if phys_0 >= 0 as i32 {
                if async_0 != 0 as i32 {
                    result = queueReadDataBlock(state, phys_0, dst, remaining);
                    if result < 0 as i32 {
                        cancelQueuedCardCommands();
                        return result;
                    }
                } else {
                    sector_size_0 = (*state).sector_size;
                    result = readCardDataBlockFinal(
                        state,
                        sector_size_0,
                        phys_0,
                        dst,
                        remaining,
                    ) as i32;
                    if result < 0 as i32 {
                        if result == -(0x105 as i32) {
                            status = -(259 as i32) as s32;
                        } else {
                            status = -(259 as i32) as s32;
                        }
                    }
                }
            }
            remaining = 0 as i32 as s32;
        }
        i += 1;
    }
    if async_0 == 0 as i32 {
        result = retryCardClose(&raw mut (*state).file_info) as i32;
        if result < 0 as i32 {
            return -(267 as i32);
        }
    } else {
        active_requests.r#type = CARD_ACTIVE_READ_FILE;
        active_requests.state = state;
        active_requests.callback = callback;
        active_requests.callback_arg = file_idx;
        hsd_804D7998 = -(1 as i32);
    }
    return status as i32;
}
#[inline]
pub unsafe extern "C" fn fn_803AE7F8_rewind() {
    melee_core::ct!("fn_803AE7F8_rewind");
    let mut snap: s32 = hsd_804D7998 as s32;
    if snap >= 0 as i32 {
        let mut saved: s32 = snap;
        let mut zero: s32 = 0;
        while saved != curr_tail {
            let mut queued: *mut CardCmd = (&raw mut commands as *mut CardCmd)
                .offset(saved as isize) as *mut CardCmd;
            saved = ((saved as i32 + 1 as i32)
                % 128 as i32) as s32;
            zero = 0 as i32 as s32;
            (*queued).r#type = zero as CardCmdType;
        }
        ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
    }
}
#[inline]
pub unsafe extern "C" fn fn_803AE7F8_close(mut state: *mut CardState) {
    melee_core::ct!("fn_803AE7F8_close");
    let mut retries: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        if CARDClose(&raw mut (*state).file_info) != -(1 as i32) {
            break;
        }
        retries += 1;
    }
}
pub unsafe extern "C" fn fn_803AE7F8(
    mut state: *mut CardState,
    mut file_idx: s32,
    mut buf: *mut u8,
    mut async_0: i32,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803AE7F8");
    let mut current_block: u64;
    let mut st: *mut CardState = state;
    let mut block_map: [[s32; 64]; 3] = [[0; 64]; 3];
    let mut pad_block_map: [u8; 32] = [0; 32];
    let mut cmd_done: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut block_map_ptr: *mut [s32; 64] = ::core::ptr::null_mut::<[s32; 64]>();
    let mut blocks_before: s32 = 0;
    let mut current_seq: s32 = 0;
    let mut file_blocks: s32 = 0;
    let mut file_size: s32 = 0;
    let mut free_count: s32 = 0;
    let mut i: s32 = 0;
    let mut logical: s32 = 0;
    let mut repair_result: s32 = 0;
    let mut result: s32 = 0;
    let mut total_blocks: s32 = 0;
    let mut verify_failed: s32 = 0;
    let mut data: *mut u8 = ::core::ptr::null_mut::<u8>();
    repair_result = 0 as i32 as s32;
    verify_failed = 0 as i32 as s32;
    if async_0 == 0 as i32 {
        let mut intr: BOOL = OSDisableInterrupts();
        let mut busy: s32 = 0;
        if hsd_804D799C as i32 == 1 as i32 {
            busy = 1 as i32 as s32;
        } else {
            busy = 0 as i32 as s32;
        }
        OSRestoreInterrupts(intr);
        if busy != 0 {
            return -(264 as i32);
        }
    }
    blocks_before = fn_803AC6B8_blocks_before(state, file_idx);
    file_size = (*st).file_sizes[file_idx as usize] as s32;
    file_blocks = fn_803AC634(state, file_idx) as s32;
    total_blocks = fn_803AC7DC(state);
    i = 0 as i32 as s32;
    while i < file_blocks {
        block_map[0 as i32 as usize][i as usize] = -(1
            as i32) as s32;
        block_map[1 as i32 as usize][i as usize] = -(1
            as i32) as s32;
        i += 1;
    }
    free_count = 0 as i32 as s32;
    current_seq = -(1 as i32) as s32;
    i = 1 as i32 as s32;
    while i <= total_blocks {
        let mut block_id: s32 = (*st).block_ids[i as usize];
        if block_id < 0 as i32 {
            let fresh4 = free_count;
            free_count = free_count + 1;
            block_map[2 as i32 as usize][fresh4 as usize] = i;
        } else {
            logical = block_id - blocks_before;
            if 0 as i32 <= logical && logical < file_blocks {
                if current_seq == -(1 as i32)
                    || fn_803ACB74(current_seq, (*st).block_seqs[i as usize])
                        < 0 as i32
                {
                    current_seq = (*st).block_seqs[i as usize];
                }
                if block_map[0 as i32 as usize][logical as usize]
                    < 0 as i32
                {
                    block_map[0 as i32 as usize][logical as usize] = i;
                } else {
                    block_map[1 as i32 as usize][logical as usize] = i;
                }
            }
        }
        i += 1;
    }
    block_map_ptr = &raw mut block_map as *mut [s32; 64] as *mut [s32; 64];
    i = 0 as i32 as s32;
    while i < file_blocks {
        if (*block_map_ptr.offset(0 as i32 as isize))[i as usize]
            < 0 as i32
            || (*st)
                .block_seqs[(*block_map_ptr
                .offset(0 as i32 as isize))[i as usize] as usize]
                != current_seq
        {
            verify_failed = 1 as i32 as s32;
            break;
        } else if (*block_map_ptr.offset(1 as i32 as isize))[i as usize]
            < 0 as i32
            || (*st)
                .block_seqs[(*block_map_ptr
                .offset(1 as i32 as isize))[i as usize] as usize]
                != current_seq
        {
            verify_failed = 1 as i32 as s32;
            break;
        } else {
            i += 1;
        }
    }
    i = 0 as i32 as s32;
    while i < file_blocks {
        if (*block_map_ptr.offset(0 as i32 as isize))[i as usize]
            < 0 as i32
        {
            if free_count > 0 as i32 {
                free_count -= 1;
                (*block_map_ptr.offset(0 as i32 as isize))[i as usize] = block_map[2
                    as i32 as usize][free_count as usize];
            } else {
                return -(257 as i32)
            }
        }
        if (*block_map_ptr.offset(1 as i32 as isize))[i as usize]
            < 0 as i32
        {
            if free_count > 0 as i32 {
                free_count -= 1;
                (*block_map_ptr.offset(1 as i32 as isize))[i as usize] = block_map[2
                    as i32 as usize][free_count as usize];
            } else {
                repair_result = -(258 as i32) as s32;
            }
        }
        i += 1;
    }
    if async_0 != 0 as i32 {
        hsd_804D7998 = curr_tail as i32;
    } else {
        let mut chan: s32 = 0;
        let mut file_no: s32 = 0;
        let mut open_result: s32 = 0;
        let mut retries: s32 = 0;
        file_no = (*st).file_no;
        chan = (*st).chan;
        retries = 0 as i32 as s32;
        while retries < 10 as i32 {
            open_result = CARDFastOpen(chan, file_no, &raw mut (*st).file_info);
            if open_result != -(1 as i32) {
                break;
            }
            retries += 1;
        }
        if open_result < 0 as i32 {
            return open_result as i32;
        }
    }
    if verify_failed == 0 as i32 {
        let mut map: *mut [s32; 64] = ::core::ptr::null_mut::<[s32; 64]>();
        let mut pass: s32 = 0;
        pass = 0 as i32 as s32;
        map = block_map_ptr;
        's_395: loop {
            if !(pass < 2 as i32) {
                current_block = 12890877304563811856;
                break;
            }
            let mut remaining: s32 = file_size;
            data = buf;
            i = 0 as i32 as s32;
            while i < file_blocks && remaining > 0 as i32 {
                let mut phys: s32 = 0;
                let mut chunk: s32 = (*st).sector_size as s32;
                chunk -= 0x20 as i32;
                if remaining as u32 > chunk as u32 {
                    phys = (*map)[i as usize];
                    if phys >= 0 as i32 {
                        if async_0 != 0 as i32 {
                            let mut cmd_result: s32 = 0;
                            if chunk == 0 as i32 {
                                cmd_result = 0 as i32 as s32;
                            } else {
                                let mut ofs: s32 = fn_803ACBE8(state, phys);
                                let mut cmd: CardCmd = CardCmd {
                                    r#type: CARD_CMD_NONE,
                                    state: ::core::ptr::null::<CardState>() as *mut CardState,
                                    x8: CardCmd_x8 {
                                        write: CardWriteArgs {
                                            file_idx: 0,
                                            phys: 0,
                                            block_id: 0,
                                            seq: 0,
                                            data: ::core::ptr::null::<c_void>()
                                                as *mut c_void,
                                            offset: 0,
                                            size: 0,
                                        },
                                    },
                                };
                                cmd.r#type = CARD_CMD_VERIFY_BLOCK;
                                cmd.state = state;
                                cmd.x8.verify.block_id = blocks_before + i;
                                cmd.x8.verify.seq = current_seq;
                                cmd.x8.verify.data = data
                                    as *mut c_void;
                                cmd.x8.verify.size = chunk;
                                cmd.x8.verify.offset = ofs;
                                cmd_result = fn_803AC168(&raw mut cmd) as s32;
                            }
                            if cmd_result < 0 as i32 {
                                fn_803AE7F8_rewind();
                                return cmd_result as i32;
                            }
                        } else {
                            let mut verify_result: s32 = fn_803ACC0C(
                                state,
                                phys,
                                blocks_before + i,
                                current_seq,
                                data as *mut c_void,
                                chunk,
                            );
                            result = verify_result;
                            if result < 0 as i32 {
                                fn_803AE7F8_close(st);
                                return result as i32;
                            }
                            if result > 0 as i32 {
                                current_block = 2119837134665360504;
                                break 's_395;
                            }
                        }
                    }
                    remaining = (remaining as u32)
                        .wrapping_sub((*st).sector_size.wrapping_sub(0x20 as u32))
                        as s32 as s32;
                    data = data
                        .offset((*st).sector_size.wrapping_sub(0x20 as u32) as isize);
                } else {
                    phys = (*map)[i as usize];
                    if phys >= 0 as i32 {
                        if async_0 != 0 as i32 {
                            let mut cmd_result_0: s32 = 0;
                            if remaining == 0 as i32 {
                                cmd_result_0 = 0 as i32 as s32;
                            } else {
                                let mut ofs_0: s32 = fn_803ACBE8(state, phys);
                                let mut cmd_0: CardCmd = CardCmd {
                                    r#type: CARD_CMD_NONE,
                                    state: ::core::ptr::null::<CardState>() as *mut CardState,
                                    x8: CardCmd_x8 {
                                        write: CardWriteArgs {
                                            file_idx: 0,
                                            phys: 0,
                                            block_id: 0,
                                            seq: 0,
                                            data: ::core::ptr::null::<c_void>()
                                                as *mut c_void,
                                            offset: 0,
                                            size: 0,
                                        },
                                    },
                                };
                                cmd_0.r#type = CARD_CMD_VERIFY_BLOCK;
                                cmd_0.state = state;
                                cmd_0.x8.verify.block_id = blocks_before + i;
                                cmd_0.x8.verify.seq = current_seq;
                                cmd_0.x8.verify.data = data
                                    as *mut c_void;
                                cmd_0.x8.verify.size = remaining;
                                cmd_0.x8.verify.offset = ofs_0;
                                cmd_result_0 = fn_803AC168(&raw mut cmd_0) as s32;
                            }
                            if cmd_result_0 < 0 as i32 {
                                fn_803AE7F8_rewind();
                                return cmd_result_0 as i32;
                            }
                        } else {
                            result = fn_803ACC0C(
                                state,
                                phys,
                                blocks_before + i,
                                current_seq,
                                data as *mut c_void,
                                remaining,
                            );
                            if result < 0 as i32 {
                                fn_803AE7F8_close(st);
                                return result as i32;
                            }
                            if result > 0 as i32 {
                                current_block = 2119837134665360504;
                                break 's_395;
                            }
                        }
                    }
                    remaining = 0 as i32 as s32;
                }
                i += 1;
            }
            pass += 1;
            map = map.offset(1);
        }
        match current_block {
            2119837134665360504 => {}
            _ => {
                if async_0 != 0 as i32 {
                    let mut cmd_result_1: s32 = 0;
                    cmd_done.r#type = CARD_CMD_CHECK_VERIFIED;
                    cmd_done.state = state;
                    cmd_result_1 = fn_803AC168(&raw mut cmd_done) as s32;
                    if cmd_result_1 < 0 as i32 {
                        fn_803AE7F8_rewind();
                        return cmd_result_1 as i32;
                    }
                } else {
                    let mut retries_0: s32 = 0;
                    retries_0 = 0 as i32 as s32;
                    while retries_0 < 10 as i32 {
                        result = CARDClose(&raw mut (*st).file_info);
                        if result != -(1 as i32) {
                            break;
                        }
                        retries_0 += 1;
                    }
                    if result < 0 as i32 {
                        return -(267 as i32);
                    }
                    return 1 as i32;
                }
            }
        }
    }
    current_seq = (current_seq as i32 + 1 as i32
        & 0xff as i32) as s32;
    let mut pass_0: s32 = 0;
    let mut map_0: *mut [s32; 64] = block_map_ptr;
    pass_0 = 0 as i32 as s32;
    while pass_0 < 2 as i32 {
        let mut remaining_0: s32 = file_size;
        data = buf;
        i = 0 as i32 as s32;
        while i < file_blocks && remaining_0 > 0 as i32 {
            let mut phys_0: s32 = 0;
            let mut chunk_0: s32 = 0;
            chunk_0 = (*st).sector_size.wrapping_sub(0x20 as u32) as s32;
            if remaining_0 as u32 > chunk_0 as u32 {
                phys_0 = (*map_0)[i as usize];
                if phys_0 >= 0 as i32 {
                    if async_0 != 0 as i32 {
                        let mut current_block_174: u64;
                        let mut zero: s32 = 0;
                        let mut cmd_result_2: s32 = 0;
                        let mut init_cmd: CardCmd = CardCmd {
                            r#type: CARD_CMD_NONE,
                            state: ::core::ptr::null::<CardState>() as *mut CardState,
                            x8: CardCmd_x8 {
                                write: CardWriteArgs {
                                    file_idx: 0,
                                    phys: 0,
                                    block_id: 0,
                                    seq: 0,
                                    data: ::core::ptr::null::<c_void>()
                                        as *mut c_void,
                                    offset: 0,
                                    size: 0,
                                },
                            },
                        };
                        let mut ofs_1: s32 = 0;
                        let mut cmd_1: CardCmd = CardCmd {
                            r#type: CARD_CMD_NONE,
                            state: ::core::ptr::null::<CardState>() as *mut CardState,
                            x8: CardCmd_x8 {
                                write: CardWriteArgs {
                                    file_idx: 0,
                                    phys: 0,
                                    block_id: 0,
                                    seq: 0,
                                    data: ::core::ptr::null::<c_void>()
                                        as *mut c_void,
                                    offset: 0,
                                    size: 0,
                                },
                            },
                        };
                        ofs_1 = fn_803ACBE8(state, phys_0);
                        if phys_0 == 0 as i32 {
                            zero = 0 as i32 as s32;
                            if blocks_before + i != 0 as i32 {
                                cmd_result_2 = -(0x101 as i32) as s32;
                                current_block_174 = 10988931288923011180;
                            } else {
                                init_cmd.r#type = CARD_CMD_READ_BLOCK;
                                init_cmd.state = state;
                                init_cmd.x8.read.phys = zero;
                                init_cmd.x8.read.xC = zero;
                                init_cmd.x8.read.data = zero
                                    as *mut c_void;
                                init_cmd.x8.read.size = zero;
                                init_cmd.x8.read.offset = ofs_1;
                                fn_803AC168(&raw mut init_cmd);
                                current_block_174 = 4122836492991094814;
                            }
                        } else {
                            current_block_174 = 4122836492991094814;
                        }
                        match current_block_174 {
                            4122836492991094814 => {
                                cmd_1.r#type = CARD_CMD_WRITE_BLOCK;
                                cmd_1.state = state;
                                cmd_1.x8.write.phys = phys_0;
                                cmd_1.x8.write.block_id = blocks_before + i;
                                cmd_1.x8.write.seq = current_seq;
                                cmd_1.x8.write.data = data
                                    as *mut c_void;
                                cmd_1.x8.write.size = chunk_0;
                                cmd_1.x8.write.offset = ofs_1;
                                cmd_1.x8.write.file_idx = file_idx;
                                cmd_result_2 = fn_803AC168(&raw mut cmd_1) as s32;
                            }
                            _ => {}
                        }
                        if cmd_result_2 < 0 as i32 {
                            fn_803AE7F8_rewind();
                            return cmd_result_2 as i32;
                        }
                    } else {
                        result = fn_803ACFC0(
                            state,
                            phys_0,
                            blocks_before + i,
                            current_seq,
                            data as *mut c_void,
                            chunk_0,
                            file_idx,
                        );
                        if result < 0 as i32 {
                            (*st)
                                .block_ids[block_map[pass_0 as usize][i as usize]
                                as usize] = -(0x7fff as i32) as s32;
                            (*st)
                                .block_seqs[block_map[pass_0 as usize][i as usize]
                                as usize] = 0 as i32 as s32;
                            fn_803AE7F8_close(st);
                            return if pass_0 == 0 as i32 {
                                -(260 as i32)
                            } else {
                                -(258 as i32)
                            };
                        }
                        (*st).block_ids[(*map_0)[i as usize] as usize] = blocks_before
                            + i;
                        (*st).block_seqs[(*map_0)[i as usize] as usize] = current_seq;
                    }
                }
                remaining_0 = (remaining_0 as u32)
                    .wrapping_sub((*st).sector_size.wrapping_sub(0x20 as u32)) as s32
                    as s32;
                data = data
                    .offset((*st).sector_size.wrapping_sub(0x20 as u32) as isize);
            } else {
                phys_0 = (*map_0)[i as usize];
                if phys_0 >= 0 as i32 {
                    if async_0 != 0 as i32 {
                        let mut current_block_217: u64;
                        let mut zero_0: s32 = 0;
                        let mut cmd_result_3: s32 = 0;
                        let mut init_cmd_0: CardCmd = CardCmd {
                            r#type: CARD_CMD_NONE,
                            state: ::core::ptr::null::<CardState>() as *mut CardState,
                            x8: CardCmd_x8 {
                                write: CardWriteArgs {
                                    file_idx: 0,
                                    phys: 0,
                                    block_id: 0,
                                    seq: 0,
                                    data: ::core::ptr::null::<c_void>()
                                        as *mut c_void,
                                    offset: 0,
                                    size: 0,
                                },
                            },
                        };
                        let mut ofs_2: s32 = 0;
                        let mut cmd_2: CardCmd = CardCmd {
                            r#type: CARD_CMD_NONE,
                            state: ::core::ptr::null::<CardState>() as *mut CardState,
                            x8: CardCmd_x8 {
                                write: CardWriteArgs {
                                    file_idx: 0,
                                    phys: 0,
                                    block_id: 0,
                                    seq: 0,
                                    data: ::core::ptr::null::<c_void>()
                                        as *mut c_void,
                                    offset: 0,
                                    size: 0,
                                },
                            },
                        };
                        ofs_2 = fn_803ACBE8(state, phys_0);
                        if phys_0 == 0 as i32 {
                            zero_0 = 0 as i32 as s32;
                            if blocks_before + i != 0 as i32 {
                                cmd_result_3 = -(0x101 as i32) as s32;
                                current_block_217 = 2161511853102541895;
                            } else {
                                init_cmd_0.r#type = CARD_CMD_READ_BLOCK;
                                init_cmd_0.state = state;
                                init_cmd_0.x8.read.phys = zero_0;
                                init_cmd_0.x8.read.xC = zero_0;
                                init_cmd_0.x8.read.data = zero_0
                                    as *mut c_void;
                                init_cmd_0.x8.read.size = zero_0;
                                init_cmd_0.x8.read.offset = ofs_2;
                                fn_803AC168(&raw mut init_cmd_0);
                                current_block_217 = 2574664466675447602;
                            }
                        } else {
                            current_block_217 = 2574664466675447602;
                        }
                        match current_block_217 {
                            2574664466675447602 => {
                                cmd_2.r#type = CARD_CMD_WRITE_BLOCK;
                                cmd_2.state = state;
                                cmd_2.x8.write.phys = phys_0;
                                cmd_2.x8.write.block_id = blocks_before + i;
                                cmd_2.x8.write.seq = current_seq;
                                cmd_2.x8.write.data = data
                                    as *mut c_void;
                                cmd_2.x8.write.size = remaining_0;
                                cmd_2.x8.write.offset = ofs_2;
                                cmd_2.x8.write.file_idx = file_idx;
                                cmd_result_3 = fn_803AC168(&raw mut cmd_2) as s32;
                            }
                            _ => {}
                        }
                        if cmd_result_3 < 0 as i32 {
                            fn_803AE7F8_rewind();
                            return cmd_result_3 as i32;
                        }
                    } else {
                        result = fn_803ACFC0(
                            state,
                            phys_0,
                            blocks_before + i,
                            current_seq,
                            data as *mut c_void,
                            remaining_0,
                            file_idx,
                        );
                        if result < 0 as i32 {
                            (*st)
                                .block_ids[block_map[pass_0 as usize][i as usize]
                                as usize] = -(0x7fff as i32) as s32;
                            (*st)
                                .block_seqs[block_map[pass_0 as usize][i as usize]
                                as usize] = 0 as i32 as s32;
                            fn_803AE7F8_close(st);
                            return if pass_0 == 0 as i32 {
                                -(260 as i32)
                            } else {
                                -(258 as i32)
                            };
                        }
                        (*st).block_ids[(*map_0)[i as usize] as usize] = blocks_before
                            + i;
                        (*st).block_seqs[(*map_0)[i as usize] as usize] = current_seq;
                    }
                }
                remaining_0 = 0 as i32 as s32;
            }
            i += 1;
        }
        pass_0 += 1;
        map_0 = map_0.offset(1);
    }
    if async_0 == 0 as i32 {
        let mut retries_1: s32 = 0;
        retries_1 = 0 as i32 as s32;
        while retries_1 < 10 as i32 {
            result = CARDClose(&raw mut (*st).file_info);
            if result != -(1 as i32) {
                break;
            }
            retries_1 += 1;
        }
        if result < 0 as i32 {
            return if repair_result == 0 as i32 {
                -(267 as i32)
            } else {
                repair_result as i32
            };
        }
    } else {
        let mut active: *mut CardActiveRequest = &raw mut active_requests;
        (*active).r#type = CARD_ACTIVE_WRITE_FILE;
        (*active).state = state;
        (*active).callback = callback;
        (*active).callback_arg = file_idx;
        hsd_804D7998 = -(1 as i32);
    }
    return repair_result as i32;
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_chunk_size(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803AF3F0_chunk_size");
    return (*state).sector_size.wrapping_sub(0x20 as u32) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_queue_verify_first(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
    mut data: *mut c_void,
    mut size: s32,
) -> s32 {
    melee_core::ct!("fn_803AF3F0_queue_verify_first");
    if size == 0 as i32 {
        return 0 as s32;
    }
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut ofs: s32 = fn_803ACBE8(state, phys);
    cmd.r#type = CARD_CMD_VERIFY_BLOCK;
    cmd.state = state;
    cmd.x8.verify.block_id = block_id;
    cmd.x8.verify.seq = seq;
    cmd.x8.verify.data = data;
    cmd.x8.verify.size = size;
    cmd.x8.verify.offset = ofs;
    return fn_803AC168(&raw mut cmd) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_queue_verify_final(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
    mut data: *mut c_void,
    mut size: s32,
) -> s32 {
    melee_core::ct!("fn_803AF3F0_queue_verify_final");
    let mut result: s32 = 0;
    if size == 0 as i32 {
        return 0 as s32;
    }
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut ofs: s32 = fn_803ACBE8(state, phys);
    cmd.r#type = CARD_CMD_VERIFY_BLOCK;
    cmd.state = state;
    cmd.x8.verify.block_id = block_id;
    cmd.x8.verify.seq = seq;
    cmd.x8.verify.data = data;
    cmd.x8.verify.size = size;
    cmd.x8.verify.offset = ofs;
    result = fn_803AC168(&raw mut cmd) as s32;
    return result;
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_queue_write_first(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
    mut data: *mut c_void,
    mut size: s32,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803AF3F0_queue_write_first");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut ofs: s32 = fn_803ACBE8(state, phys);
    let mut init_cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    if phys == 0 as i32 {
        let mut zero: s32 = 0 as s32;
        if block_id != 0 as i32 {
            return -(0x101 as s32);
        }
        init_cmd.r#type = CARD_CMD_READ_BLOCK;
        init_cmd.state = state;
        init_cmd.x8.read.phys = zero;
        init_cmd.x8.read.xC = zero;
        init_cmd.x8.read.data = zero as *mut c_void;
        init_cmd.x8.read.size = zero;
        init_cmd.x8.read.offset = ofs;
        fn_803AC168(&raw mut init_cmd);
    }
    cmd.r#type = CARD_CMD_WRITE_BLOCK;
    cmd.state = state;
    cmd.x8.write.phys = phys;
    cmd.x8.write.block_id = block_id;
    cmd.x8.write.seq = seq;
    cmd.x8.write.data = data;
    cmd.x8.write.size = size;
    cmd.x8.write.offset = ofs;
    cmd.x8.write.file_idx = file_idx;
    return fn_803AC168(&raw mut cmd) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_queue_write_final(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
    mut data: *mut c_void,
    mut size: s32,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803AF3F0_queue_write_final");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut ofs: s32 = fn_803ACBE8(state, phys);
    let mut init_cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut result: s32 = 0;
    if phys == 0 as i32 {
        let mut zero: s32 = 0 as s32;
        if block_id != 0 as i32 {
            return -(0x101 as s32);
        }
        init_cmd.r#type = CARD_CMD_READ_BLOCK;
        init_cmd.state = state;
        init_cmd.x8.read.phys = zero;
        init_cmd.x8.read.xC = zero;
        init_cmd.x8.read.data = zero as *mut c_void;
        init_cmd.x8.read.size = zero;
        init_cmd.x8.read.offset = ofs;
        fn_803AC168(&raw mut init_cmd);
    }
    cmd.r#type = CARD_CMD_WRITE_BLOCK;
    cmd.state = state;
    cmd.x8.write.phys = phys;
    cmd.x8.write.block_id = block_id;
    cmd.x8.write.seq = seq;
    cmd.x8.write.data = data;
    cmd.x8.write.size = size;
    cmd.x8.write.offset = ofs;
    cmd.x8.write.file_idx = file_idx;
    result = fn_803AC168(&raw mut cmd) as s32;
    return result;
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_rewind() {
    melee_core::ct!("fn_803AF3F0_rewind");
    let mut snap: s32 = hsd_804D7998 as s32;
    if snap >= 0 as i32 {
        let mut saved: s32 = snap;
        let mut zero: s32 = 0;
        while saved != curr_tail {
            let mut queued: *mut CardCmd = (&raw mut commands as *mut CardCmd)
                .offset(saved as isize) as *mut CardCmd;
            saved = ((saved as i32 + 1 as i32)
                % 128 as i32) as s32;
            zero = 0 as i32 as s32;
            (*queued).r#type = zero as CardCmdType;
        }
        ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
    }
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_close(mut state: *mut CardState) {
    melee_core::ct!("fn_803AF3F0_close");
    let mut retries: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        if CARDClose(&raw mut (*state).file_info) != -(1 as i32) {
            break;
        }
        retries += 1;
    }
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_open(
    mut chan: s32,
    mut file_no: s32,
    mut info: *mut CARDFileInfo,
) -> s32 {
    melee_core::ct!("fn_803AF3F0_open");
    let mut retries: s32 = 0;
    let mut open_result: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        open_result = CARDFastOpen(chan, file_no, info);
        if open_result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    return open_result;
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_check_seq(
    mut state: *mut CardState,
    mut primary: *const s32,
    mut count: s32,
    mut current_seq: s32,
    mut needs_rewrite: *mut s32,
) {
    melee_core::ct!("fn_803AF3F0_check_seq");
    let mut i: s32 = 0;
    i = 0 as i32 as s32;
    while i < count {
        if *primary.offset(i as isize) < 0 as i32
            || (*state).block_seqs[*primary.offset(i as isize) as usize] != current_seq
        {
            *needs_rewrite = 1 as i32 as s32;
            break;
        } else {
            i += 1;
        }
    }
}
#[inline]
pub unsafe extern "C" fn fn_803AF3F0_calc_file_blocks(
    mut file_idx: s32,
    mut state: *mut CardState,
    mut file_blocks: *mut s32,
    mut total_blocks: *mut s32,
) {
    melee_core::ct!("fn_803AF3F0_calc_file_blocks");
    if (*state).file_sizes[file_idx as usize] <= 0 as i32 {
        *file_blocks = 0 as i32 as s32;
    } else if file_idx == 0 as i32 {
        let mut sector_size: u32 = (*state).sector_size;
        let mut usable: u32 = 0;
        let mut rem: s32 = 0;
        rem = (*state).file_sizes[0 as i32 as usize] as s32;
        usable = sector_size.wrapping_sub(0x20 as u32);
        rem = rem
            - usable
                .wrapping_sub(
                    (*state)
                        .header_size
                        .wrapping_add(0x30 as u32)
                        .card_rem(sector_size),
                ) as s32;
        if rem <= 0 as i32 {
            *file_blocks = 1 as i32 as s32;
        } else {
            *file_blocks = (rem as u32)
                .wrapping_add(sector_size)
                .wrapping_sub(0x21 as u32)
                .card_div(sector_size.wrapping_sub(0x20 as u32))
                .wrapping_add(1 as u32) as s32;
        }
    } else {
        let mut sector_size_0: u32 = (*state).sector_size;
        *file_blocks = ((*state).file_sizes[file_idx as usize] as u32)
            .wrapping_add(sector_size_0)
            .wrapping_sub(0x21 as u32)
            .card_div(sector_size_0.wrapping_sub(0x20 as u32)) as s32;
    }
    *total_blocks = fn_803AC7DC(state);
}
pub unsafe extern "C" fn fn_803AF3F0(
    mut state: *mut CardState,
    mut file_idx: s32,
    mut buf: *mut u8,
    mut async_0: i32,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803AF3F0");
    let mut current_block: u64;
    let mut block_map: [[s32; 64]; 3] = [[0; 64]; 3];
    let mut needs_rewrite: s32 = 0;
    let mut blocks_before: s32 = 0;
    let mut file_blocks: s32 = 0;
    let mut file_size: s32 = 0;
    let mut result: s32 = 0;
    let mut current_seq: s32 = 0;
    let mut secondary_count: s32 = 0;
    let mut free_count: s32 = 0;
    let mut i: s32 = 0;
    let mut remaining: s32 = 0;
    let mut total_blocks: s32 = 0;
    let mut data: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut free_blk: i32 = 0;
    let mut write_blk: i32 = 0;
    needs_rewrite = 0 as i32 as s32;
    if async_0 == 0 as i32 {
        let mut intr: BOOL = OSDisableInterrupts();
        let mut busy: s32 = 0;
        if hsd_804D799C as i32 == 1 as i32 {
            busy = 1 as i32 as s32;
        } else {
            busy = 0 as i32 as s32;
        }
        OSRestoreInterrupts(intr);
        if busy != 0 {
            return -(264 as i32);
        }
    }
    blocks_before = fn_803AC6B8_blocks_before(state, file_idx);
    file_size = (*state).file_sizes[file_idx as usize] as s32;
    fn_803AF3F0_calc_file_blocks(
        file_idx,
        state,
        &raw mut file_blocks,
        &raw mut total_blocks,
    );
    if async_0 != 0 as i32 {
        hsd_804D7998 = curr_tail as i32;
    } else {
        let mut open_result: s32 = fn_803AF3F0_open(
            (*state).chan,
            (*state).file_no,
            &raw mut (*state).file_info,
        );
        if open_result < 0 as i32 {
            return open_result as i32;
        }
    }
    i = 0 as i32 as s32;
    while i < file_blocks {
        block_map[0 as i32 as usize][i as usize] = -(1
            as i32) as s32;
        block_map[1 as i32 as usize][i as usize] = -(1
            as i32) as s32;
        i += 1;
    }
    current_seq = -(1 as i32) as s32;
    secondary_count = 0 as i32 as s32;
    free_count = 0 as i32 as s32;
    i = 1 as i32 as s32;
    while i <= total_blocks {
        let mut block_id: s32 = (*state).block_ids[i as usize];
        if block_id >= 0 as i32 {
            let mut logical: s32 = block_id - blocks_before;
            if 0 as i32 <= logical && logical < file_blocks {
                if fn_803ACB74(current_seq, (*state).block_seqs[i as usize])
                    < 0 as i32
                {
                    current_seq = (*state).block_seqs[i as usize];
                }
                block_map[0 as i32 as usize][logical as usize] = i;
            }
        } else {
            let mut logical_0: s32 = -(block_id + blocks_before);
            if 0 as i32 <= logical_0 && logical_0 < file_blocks {
                block_map[1 as i32 as usize][secondary_count as usize] = i;
                secondary_count += 1;
            } else {
                block_map[2 as i32 as usize][free_count as usize] = i;
                free_count += 1;
            }
        }
        i += 1;
    }
    fn_803AF3F0_check_seq(
        state,
        &raw mut *(&raw mut block_map as *mut [s32; 64])
            .offset(0 as i32 as isize) as *mut s32,
        file_blocks,
        current_seq,
        &raw mut needs_rewrite,
    );
    if needs_rewrite == 0 as i32 {
        remaining = file_size;
        data = buf;
        i = 0 as i32 as s32;
        loop {
            if !(i < file_blocks && remaining > 0 as i32) {
                current_block = 1352918242886884122;
                break;
            }
            if remaining as u32 > fn_803AF3F0_chunk_size(state) as u32 {
                if async_0 != 0 as i32 {
                    let mut block: s32 = block_map[0 as i32
                        as usize][i as usize];
                    let mut cmd_result: s32 = fn_803AF3F0_queue_verify_first(
                        state,
                        block,
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        fn_803AF3F0_chunk_size(state),
                    );
                    if cmd_result < 0 as i32 {
                        fn_803AF3F0_rewind();
                        return cmd_result as i32;
                    }
                } else {
                    let mut verify_result: s32 = fn_803ACC0C(
                        state,
                        block_map[0 as i32 as usize][i as usize],
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        fn_803AF3F0_chunk_size(state),
                    );
                    if verify_result < 0 as i32 {
                        fn_803AF3F0_close(state);
                        return verify_result as i32;
                    }
                    if verify_result > 0 as i32 {
                        current_block = 6970646637464534235;
                        break;
                    }
                }
                remaining -= fn_803AF3F0_chunk_size(state);
                data = data.offset(fn_803AF3F0_chunk_size(state) as isize);
            } else {
                if async_0 != 0 as i32 {
                    let mut cmd_result_0: s32 = fn_803AF3F0_queue_verify_final(
                        state,
                        block_map[0 as i32 as usize][i as usize],
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        remaining,
                    );
                    if cmd_result_0 < 0 as i32 {
                        fn_803AF3F0_rewind();
                        return cmd_result_0 as i32;
                    }
                } else {
                    let mut verify_result_0: s32 = fn_803ACC0C(
                        state,
                        block_map[0 as i32 as usize][i as usize],
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        remaining,
                    );
                    if verify_result_0 < 0 as i32 {
                        fn_803AF3F0_close(state);
                        return verify_result_0 as i32;
                    }
                    if verify_result_0 > 0 as i32 {
                        current_block = 6970646637464534235;
                        break;
                    }
                }
                remaining = 0 as i32 as s32;
            }
            i += 1;
        }
        match current_block {
            6970646637464534235 => {}
            _ => {
                if async_0 != 0 as i32 {
                    let mut cmd: CardCmd = CardCmd {
                        r#type: CARD_CMD_NONE,
                        state: ::core::ptr::null::<CardState>() as *mut CardState,
                        x8: CardCmd_x8 {
                            write: CardWriteArgs {
                                file_idx: 0,
                                phys: 0,
                                block_id: 0,
                                seq: 0,
                                data: ::core::ptr::null::<c_void>()
                                    as *mut c_void,
                                offset: 0,
                                size: 0,
                            },
                        },
                    };
                    let mut cmd_result_1: s32 = 0;
                    cmd.r#type = CARD_CMD_CHECK_VERIFIED;
                    cmd.state = state;
                    cmd_result_1 = fn_803AC168(&raw mut cmd) as s32;
                    if cmd_result_1 < 0 as i32 {
                        fn_803AF3F0_rewind();
                        return cmd_result_1 as i32;
                    }
                } else {
                    let mut retries: s32 = 0;
                    retries = 0 as i32 as s32;
                    while retries < 10 as i32 {
                        result = CARDClose(&raw mut (*state).file_info);
                        if result != -(1 as i32) {
                            break;
                        }
                        retries += 1;
                    }
                    if result < 0 as i32 {
                        return -(267 as i32);
                    }
                    return 1 as i32;
                }
            }
        }
    }
    if secondary_count > file_blocks {
        i = file_blocks;
        while i < secondary_count {
            if async_0 != 0 as i32 {
                let mut cmd_result_2: s32 = 0;
                free_blk = block_map[1 as i32 as usize][i as usize];
                cmd_result_2 = fn_803AF3F0_queue_write_final(
                    state,
                    free_blk,
                    0xffff as s32,
                    0 as s32,
                    ::core::ptr::null_mut::<c_void>(),
                    0 as s32,
                    file_idx,
                );
                if cmd_result_2 < 0 as i32 {
                    fn_803AF3F0_rewind();
                    return cmd_result_2 as i32;
                }
            } else {
                result = fn_803ACFC0(
                    state,
                    block_map[1 as i32 as usize][i as usize],
                    0xffff as s32,
                    0 as s32,
                    ::core::ptr::null_mut::<c_void>(),
                    0 as s32,
                    file_idx,
                );
                (*state)
                    .block_ids[block_map[1 as i32 as usize][i as usize]
                    as usize] = -(0x7fff as i32) as s32;
                (*state)
                    .block_seqs[block_map[1 as i32 as usize][i as usize]
                    as usize] = 0 as i32 as s32;
                if result < 0 as i32 {
                    fn_803AF3F0_close(state);
                    return result as i32;
                }
            }
            i += 1;
        }
    } else if secondary_count < file_blocks {
        while secondary_count < file_blocks && free_count > 0 as i32 {
            free_count -= 1;
            let fresh2 = secondary_count;
            secondary_count = secondary_count + 1;
            block_map[1 as i32 as usize][fresh2 as usize] = block_map[2
                as i32 as usize][free_count as usize];
        }
        if secondary_count < file_blocks {
            i = 0 as i32 as s32;
            while i < file_blocks && secondary_count < file_blocks {
                let mut primary: s32 = block_map[0 as i32
                    as usize][i as usize];
                if 0 as i32 <= primary {
                    let fresh3 = secondary_count;
                    secondary_count = secondary_count + 1;
                    block_map[1 as i32 as usize][fresh3 as usize] = primary;
                    block_map[0 as i32 as usize][i as usize] = -(1
                        as i32) as s32;
                }
                i += 1;
            }
            if secondary_count < file_blocks {
                if async_0 == 0 as i32 {
                    fn_803AF3F0_close(state);
                }
                return -(257 as i32);
            }
        }
    }
    let mut next: s32 = current_seq + 1 as s32;
    remaining = file_size;
    data = buf;
    current_seq = (next as i32 & 0xff as i32) as s32;
    i = 0 as i32 as s32;
    while i < file_blocks && remaining > 0 as i32 {
        let mut chunk: s32 = 0;
        chunk = fn_803AF3F0_chunk_size(state);
        if remaining as u32 > chunk as u32 {
            if async_0 != 0 as i32 {
                let mut cmd_result_3: s32 = 0;
                write_blk = block_map[1 as i32 as usize][i as usize];
                cmd_result_3 = fn_803AF3F0_queue_write_first(
                    state,
                    write_blk,
                    blocks_before + i,
                    current_seq,
                    data as *mut c_void,
                    chunk,
                    file_idx,
                );
                if cmd_result_3 < 0 as i32 {
                    fn_803AF3F0_rewind();
                    return cmd_result_3 as i32;
                }
            } else {
                let mut write_result: s32 = fn_803ACFC0(
                    state,
                    block_map[1 as i32 as usize][i as usize],
                    blocks_before + i,
                    current_seq,
                    data as *mut c_void,
                    fn_803AF3F0_chunk_size(state),
                    file_idx,
                );
                if write_result < 0 as i32 {
                    (*state)
                        .block_ids[block_map[1 as i32
                        as usize][i as usize] as usize] = -(0x7fff as i32)
                        as s32;
                    (*state)
                        .block_seqs[block_map[1 as i32
                        as usize][i as usize] as usize] = 0 as i32 as s32;
                    fn_803AF3F0_close(state);
                    return write_result as i32;
                }
                (*state)
                    .block_ids[block_map[1 as i32 as usize][i as usize]
                    as usize] = blocks_before + i;
                (*state)
                    .block_seqs[block_map[1 as i32 as usize][i as usize]
                    as usize] = current_seq;
            }
            remaining = (remaining as u32)
                .wrapping_sub((*state).sector_size.wrapping_sub(0x20 as u32)) as s32
                as s32;
            data = data.offset(fn_803AF3F0_chunk_size(state) as isize);
        } else {
            if async_0 != 0 as i32 {
                let mut block_0: s32 = block_map[1 as i32
                    as usize][i as usize];
                let mut cmd_result_4: s32 = fn_803AF3F0_queue_write_final(
                    state,
                    block_0,
                    blocks_before + i,
                    current_seq,
                    data as *mut c_void,
                    remaining,
                    file_idx,
                );
                if cmd_result_4 < 0 as i32 {
                    fn_803AF3F0_rewind();
                    return cmd_result_4 as i32;
                }
            } else {
                let mut write_result_0: s32 = fn_803ACFC0(
                    state,
                    block_map[1 as i32 as usize][i as usize],
                    blocks_before + i,
                    current_seq,
                    data as *mut c_void,
                    remaining,
                    file_idx,
                );
                if write_result_0 < 0 as i32 {
                    (*state)
                        .block_ids[block_map[1 as i32
                        as usize][i as usize] as usize] = -(0x7fff as i32)
                        as s32;
                    (*state)
                        .block_seqs[block_map[1 as i32
                        as usize][i as usize] as usize] = 0 as i32 as s32;
                    fn_803AF3F0_close(state);
                    return write_result_0 as i32;
                }
                (*state)
                    .block_ids[block_map[1 as i32 as usize][i as usize]
                    as usize] = blocks_before + i;
                (*state)
                    .block_seqs[block_map[1 as i32 as usize][i as usize]
                    as usize] = current_seq;
            }
            remaining = 0 as i32 as s32;
        }
        i += 1;
    }
    if async_0 == 0 as i32 {
        i = 0 as i32 as s32;
        while i < file_blocks {
            if block_map[0 as i32 as usize][i as usize]
                >= 0 as i32
            {
                (*state)
                    .block_ids[block_map[0 as i32 as usize][i as usize]
                    as usize] = -(*state)
                    .block_ids[block_map[0 as i32 as usize][i as usize]
                    as usize];
            }
            i += 1;
        }
        let mut retries_0: s32 = 0;
        let mut close_result: s32 = 0;
        retries_0 = 0 as i32 as s32;
        while retries_0 < 10 as i32 {
            close_result = CARDClose(&raw mut (*state).file_info);
            if close_result != -(1 as i32) {
                break;
            }
            retries_0 += 1;
        }
        if close_result < 0 as i32 {
            return -(267 as i32);
        }
    } else {
        let mut active: *mut CardActiveRequest = &raw mut active_requests;
        (*active).r#type = CARD_ACTIVE_WRITE_FILE_1_2;
        (*active).state = state;
        (*active).callback = callback;
        (*active).callback_arg = file_idx;
        hsd_804D7998 = -(1 as i32);
    }
    return 0 as i32;
}
#[inline]
pub unsafe extern "C" fn fn_803B0120_first_chunk(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803B0120_first_chunk");
    let mut sector_size: u32 = (*state).sector_size;
    return sector_size.wrapping_sub(0x20 as u32) as s32
        - (*state).header_size.wrapping_add(0x30 as u32).card_rem(sector_size)
            as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803B0120_rewind() {
    melee_core::ct!("fn_803B0120_rewind");
    let mut snap: s32 = hsd_804D7998 as s32;
    if snap >= 0 as i32 {
        let mut saved: s32 = snap;
        let mut zero: s32 = 0;
        while saved != curr_tail {
            let mut queued: *mut CardCmd = (&raw mut commands as *mut CardCmd)
                .offset(saved as isize) as *mut CardCmd;
            saved = ((saved as i32 + 1 as i32)
                % 128 as i32) as s32;
            zero = 0 as i32 as s32;
            (*queued).r#type = zero as CardCmdType;
        }
        ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
    }
}
#[inline]
pub unsafe extern "C" fn fn_803B0120_close(mut state: *mut CardState) {
    melee_core::ct!("fn_803B0120_close");
    let mut retries: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        if CARDClose(&raw mut (*state).file_info) != -(1 as i32) {
            break;
        }
        retries += 1;
    }
}
#[inline]
pub unsafe extern "C" fn fn_803B0120_close_result(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803B0120_close_result");
    let mut result: s32 = 0;
    let mut retries: s32 = 0;
    let mut ret: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        result = CARDClose(&raw mut (*state).file_info);
        if result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    ret = result;
    return ret;
}
#[inline]
pub unsafe extern "C" fn fn_803B0120_block_offset(
    mut state: *mut CardState,
    mut block_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803B0120_block_offset");
    let mut size: u32 = (*state).sector_size;
    let mut temp: u32 = (*state).header_size.wrapping_add(size);
    let mut num: u32 = temp.wrapping_add(0x2f as u32);
    let mut idx: u32 = num.card_div(size).wrapping_sub(1 as u32);
    let mut pos: u32 = (block_idx as u32).wrapping_add(idx);
    return size.wrapping_mul(pos) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803B0120_queue_verify(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
    mut data: *mut c_void,
    mut size: s32,
) -> s32 {
    melee_core::ct!("fn_803B0120_queue_verify");
    let mut result: s32 = 0;
    if size == 0 as i32 {
        return 0 as s32;
    }
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut ofs: s32 = fn_803B0120_block_offset(state, phys);
    cmd.r#type = CARD_CMD_VERIFY_BLOCK;
    cmd.state = state;
    cmd.x8.verify.block_id = block_id;
    cmd.x8.verify.seq = seq;
    cmd.x8.verify.data = data;
    cmd.x8.verify.size = size;
    cmd.x8.verify.offset = ofs;
    result = fn_803AC168(&raw mut cmd) as s32;
    return result;
}
#[inline]
pub unsafe extern "C" fn fn_803B0120_queue_write(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut seq: s32,
    mut data: *mut c_void,
    mut size: s32,
    mut file_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803B0120_queue_write");
    let mut cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut ofs: s32 = fn_803ACBE8(state, phys);
    let mut init_cmd: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut result: s32 = 0;
    if phys == 0 as i32 {
        let mut zero: s32 = 0 as s32;
        if block_id != 0 as i32 {
            return -(0x101 as s32);
        }
        init_cmd.r#type = CARD_CMD_READ_BLOCK;
        init_cmd.state = state;
        init_cmd.x8.read.phys = zero;
        init_cmd.x8.read.xC = zero;
        init_cmd.x8.read.data = zero as *mut c_void;
        init_cmd.x8.read.size = zero;
        init_cmd.x8.read.offset = ofs;
        fn_803AC168(&raw mut init_cmd);
    }
    cmd.r#type = CARD_CMD_WRITE_BLOCK;
    cmd.state = state;
    cmd.x8.write.phys = phys;
    cmd.x8.write.block_id = block_id;
    cmd.x8.write.seq = seq;
    cmd.x8.write.data = data;
    cmd.x8.write.size = size;
    cmd.x8.write.offset = ofs;
    cmd.x8.write.file_idx = file_idx;
    result = fn_803AC168(&raw mut cmd) as s32;
    return result;
}
pub unsafe extern "C" fn fn_803B0120(
    mut state: *mut CardState,
    mut file_idx: s32,
    mut buf: *mut u8,
    mut async_0: i32,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803B0120");
    let mut current_block: u64;
    let mut block_map: [[s32; 64]; 3] = [[0; 64]; 3];
    let mut needs_rewrite: s32 = 0;
    let mut blocks_before: s32 = 0;
    let mut file_blocks: s32 = 0;
    let mut file_size: s32 = 0;
    let mut total_blocks: s32 = 0;
    let mut current_seq: s32 = 0;
    let mut block_id: s32 = 0;
    let mut seq: s32 = 0;
    let mut i: s32 = 0;
    let mut secondary_count: s32 = 0;
    let mut free_count: s32 = 0;
    let mut logical: s32 = 0;
    let mut remaining: s32 = 0;
    let mut result: s32 = 0;
    let mut data: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut j: s32 = 0;
    needs_rewrite = 0 as i32 as s32;
    if async_0 == 0 as i32 {
        let mut intr: BOOL = OSDisableInterrupts();
        let mut busy: s32 = 0;
        if hsd_804D799C as i32 == 1 as i32 {
            busy = 1 as i32 as s32;
        } else {
            busy = 0 as i32 as s32;
        }
        OSRestoreInterrupts(intr);
        if busy != 0 {
            return -(264 as i32);
        }
    }
    blocks_before = fn_803AC6B8_blocks_before(state, file_idx);
    let mut size: s32 = (*state).file_sizes[file_idx as usize];
    file_size = size;
    file_blocks = calculateFileBlockCount(state, file_idx);
    total_blocks = fn_803AC7DC(state);
    if async_0 != 0 as i32 {
        hsd_804D7998 = curr_tail as i32;
    } else {
        let mut open_result: s32 = fn_803AF3F0_open(
            (*state).chan,
            (*state).file_no,
            &raw mut (*state).file_info,
        );
        if open_result < 0 as i32 {
            return open_result as i32;
        }
    }
    j = 0 as i32 as s32;
    while j < file_blocks {
        block_map[0 as i32 as usize][j as usize] = -(1
            as i32) as s32;
        block_map[1 as i32 as usize][j as usize] = -(1
            as i32) as s32;
        j += 1;
    }
    current_seq = -(1 as i32) as s32;
    if file_idx == 0 as i32 {
        if (*state).block_ids[0 as i32 as usize]
            == 0 as i32
        {
            current_seq = (*state).block_seqs[0 as i32 as usize];
        } else {
            needs_rewrite = 1 as i32 as s32;
        }
        block_map[0 as i32 as usize][0 as i32 as usize] = 0
            as i32 as s32;
    }
    secondary_count = 0 as i32 as s32;
    free_count = 0 as i32 as s32;
    i = 1 as i32 as s32;
    while i <= total_blocks {
        block_id = (*state).block_ids[i as usize];
        if block_id >= 0 as i32 {
            logical = block_id - blocks_before;
            if file_idx == 0 as i32 && logical == 0 as i32
            {
                block_map[1 as i32 as usize][secondary_count as usize] = i;
                secondary_count += 1;
            } else if logical >= 0 as i32 && logical < file_blocks {
                seq = (*state).block_seqs[i as usize];
                if fn_803ACB74(current_seq, seq) < 0 as i32 {
                    current_seq = seq;
                }
                block_map[0 as i32 as usize][logical as usize] = i;
            }
        } else {
            logical = -(block_id + blocks_before);
            if logical >= 0 as i32 && logical < file_blocks {
                block_map[1 as i32 as usize][secondary_count as usize] = i;
                secondary_count += 1;
            } else {
                block_map[2 as i32 as usize][free_count as usize] = i;
                free_count += 1;
            }
        }
        i += 1;
    }
    i = 0 as i32 as s32;
    while i < file_blocks {
        if block_map[0 as i32 as usize][i as usize]
            < 0 as i32
        {
            needs_rewrite = 1 as i32 as s32;
            if secondary_count > 0 as i32 {
                secondary_count -= 1;
                block_map[0 as i32 as usize][i as usize] = block_map[1
                    as i32 as usize][secondary_count as usize];
                block_map[1 as i32 as usize][secondary_count as usize] = -(1
                    as i32) as s32;
            } else if free_count > 0 as i32 {
                free_count -= 1;
                block_map[0 as i32 as usize][i as usize] = block_map[2
                    as i32 as usize][free_count as usize];
                block_map[2 as i32 as usize][free_count as usize] = -(1
                    as i32) as s32;
            } else {
                if async_0 == 0 as i32 {
                    fn_803B0120_close(state);
                }
                return -(257 as i32);
            }
        }
        i += 1;
    }
    if needs_rewrite == 0 as i32 {
        i = 0 as i32 as s32;
        while i < file_blocks {
            if block_map[0 as i32 as usize][i as usize]
                < 0 as i32
                || (*state)
                    .block_seqs[block_map[0 as i32 as usize][i as usize]
                    as usize] != current_seq
            {
                needs_rewrite = 1 as i32 as s32;
                break;
            } else {
                i += 1;
            }
        }
    }
    if needs_rewrite == 0 as i32 {
        remaining = file_size;
        data = buf;
        i = 0 as i32 as s32;
        loop {
            if !(i < file_blocks && remaining > 0 as i32) {
                current_block = 13505557363059842426;
                break;
            }
            let mut block_id_0: s32 = blocks_before + i;
            let mut chunk: s32 = 0;
            if block_id_0 == 0 as i32 {
                chunk = fn_803B0120_first_chunk(state);
            } else {
                chunk = (*state).sector_size.wrapping_sub(0x20 as u32) as s32;
            }
            if remaining > chunk {
                if async_0 != 0 as i32 {
                    let mut cmd_result: s32 = fn_803B0120_queue_verify(
                        state,
                        block_map[0 as i32 as usize][i as usize],
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        chunk,
                    );
                    if cmd_result < 0 as i32 {
                        fn_803B0120_rewind();
                        return cmd_result as i32;
                    }
                } else {
                    let mut verify_result: s32 = fn_803ACC0C(
                        state,
                        block_map[0 as i32 as usize][i as usize],
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        chunk,
                    );
                    result = verify_result;
                    if result < 0 as i32 {
                        fn_803B0120_close(state);
                        return result as i32;
                    }
                    if result > 0 as i32 {
                        current_block = 7293850626974290116;
                        break;
                    }
                }
                remaining -= chunk;
                data = data.offset(chunk as isize);
            } else {
                if async_0 != 0 as i32 {
                    let mut cmd_result_0: s32 = fn_803B0120_queue_verify(
                        state,
                        block_map[0 as i32 as usize][i as usize],
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        remaining,
                    );
                    if cmd_result_0 < 0 as i32 {
                        fn_803B0120_rewind();
                        return cmd_result_0 as i32;
                    }
                } else {
                    result = fn_803ACC0C(
                        state,
                        block_map[0 as i32 as usize][i as usize],
                        blocks_before + i,
                        current_seq,
                        data as *mut c_void,
                        remaining,
                    );
                    if result < 0 as i32 {
                        fn_803B0120_close(state);
                        return result as i32;
                    }
                    if result > 0 as i32 {
                        current_block = 7293850626974290116;
                        break;
                    }
                }
                remaining = 0 as i32 as s32;
            }
            i += 1;
        }
        match current_block {
            7293850626974290116 => {}
            _ => {
                if async_0 != 0 as i32 {
                    let mut cmd: CardCmd = CardCmd {
                        r#type: CARD_CMD_NONE,
                        state: ::core::ptr::null::<CardState>() as *mut CardState,
                        x8: CardCmd_x8 {
                            write: CardWriteArgs {
                                file_idx: 0,
                                phys: 0,
                                block_id: 0,
                                seq: 0,
                                data: ::core::ptr::null::<c_void>()
                                    as *mut c_void,
                                offset: 0,
                                size: 0,
                            },
                        },
                    };
                    let mut cmd_result_1: s32 = 0;
                    cmd.r#type = CARD_CMD_CHECK_VERIFIED;
                    cmd.state = state;
                    cmd_result_1 = fn_803AC168(&raw mut cmd) as s32;
                    if cmd_result_1 < 0 as i32 {
                        fn_803B0120_rewind();
                        return cmd_result_1 as i32;
                    }
                } else {
                    let mut retries: s32 = 0;
                    retries = 0 as i32 as s32;
                    while retries < 10 as i32 {
                        result = CARDClose(&raw mut (*state).file_info);
                        if result != -(1 as i32) {
                            break;
                        }
                        retries += 1;
                    }
                    if result < 0 as i32 {
                        return -(267 as i32);
                    }
                    return 1 as i32;
                }
            }
        }
    }
    while secondary_count > 0 as i32 {
        secondary_count -= 1;
        if async_0 != 0 as i32 {
            let mut block: s32 = block_map[1 as i32
                as usize][secondary_count as usize];
            let mut cmd_result_2: s32 = fn_803B0120_queue_write(
                state,
                block,
                0xffff as s32,
                0 as s32,
                ::core::ptr::null_mut::<c_void>(),
                0 as s32,
                file_idx,
            );
            if cmd_result_2 < 0 as i32 {
                fn_803B0120_rewind();
                return cmd_result_2 as i32;
            }
        } else {
            let mut secondary: *mut s32 = &raw mut *(&raw mut block_map
                as *mut [s32; 64])
                .offset(1 as i32 as isize) as *mut s32;
            let mut block_0: *mut s32 = secondary.offset(secondary_count as isize)
                as *mut s32;
            result = fn_803ACFC0(
                state,
                *block_0,
                0xffff as s32,
                0 as s32,
                ::core::ptr::null_mut::<c_void>(),
                0 as s32,
                file_idx,
            );
            (*state).block_ids[*block_0 as usize] = -(0x7fff as i32)
                as s32;
            (*state).block_seqs[*block_0 as usize] = 0 as i32 as s32;
            if result < 0 as i32 {
                fn_803B0120_close(state);
            }
            return result as i32;
        }
    }
    let mut next: s32 = current_seq + 1 as s32;
    remaining = file_size;
    data = buf;
    seq = (next as i32 & 0xff as i32) as s32;
    i = 0 as i32 as s32;
    while i < file_blocks && remaining > 0 as i32 {
        let mut block_id_1: s32 = blocks_before + i;
        let mut chunk_0: s32 = 0;
        if block_id_1 == 0 as i32 {
            chunk_0 = fn_803B0120_first_chunk(state);
        } else {
            chunk_0 = (*state).sector_size.wrapping_sub(0x20 as u32) as s32;
        }
        if remaining > chunk_0 {
            if async_0 != 0 as i32 {
                let mut block_1: s32 = block_map[0 as i32
                    as usize][i as usize];
                let mut cmd_result_3: s32 = fn_803B0120_queue_write(
                    state,
                    block_1,
                    block_id_1,
                    seq,
                    data as *mut c_void,
                    chunk_0,
                    file_idx,
                );
                if cmd_result_3 < 0 as i32 {
                    fn_803B0120_rewind();
                    return cmd_result_3 as i32;
                }
            } else {
                let mut write_result: s32 = fn_803ACFC0(
                    state,
                    block_map[0 as i32 as usize][i as usize],
                    blocks_before + i,
                    seq,
                    data as *mut c_void,
                    chunk_0,
                    file_idx,
                );
                result = write_result;
                if result < 0 as i32 {
                    (*state)
                        .block_ids[block_map[0 as i32
                        as usize][i as usize] as usize] = -(0x7fff as i32)
                        as s32;
                    (*state)
                        .block_seqs[block_map[0 as i32
                        as usize][i as usize] as usize] = 0 as i32 as s32;
                    fn_803B0120_close(state);
                    return result as i32;
                }
                (*state)
                    .block_ids[block_map[0 as i32 as usize][i as usize]
                    as usize] = blocks_before + i;
                (*state)
                    .block_seqs[block_map[0 as i32 as usize][i as usize]
                    as usize] = seq;
            }
            remaining -= chunk_0;
            data = data.offset(chunk_0 as isize);
        } else {
            if async_0 != 0 as i32 {
                let mut block_2: s32 = block_map[0 as i32
                    as usize][i as usize];
                let mut cmd_result_4: s32 = fn_803B0120_queue_write(
                    state,
                    block_2,
                    block_id_1,
                    seq,
                    data as *mut c_void,
                    remaining,
                    file_idx,
                );
                if cmd_result_4 < 0 as i32 {
                    fn_803B0120_rewind();
                    return cmd_result_4 as i32;
                }
            } else {
                let mut write_result_0: s32 = fn_803ACFC0(
                    state,
                    block_map[0 as i32 as usize][i as usize],
                    blocks_before + i,
                    seq,
                    data as *mut c_void,
                    remaining,
                    file_idx,
                );
                result = write_result_0;
                if result < 0 as i32 {
                    (*state)
                        .block_ids[block_map[0 as i32
                        as usize][i as usize] as usize] = -(0x7fff as i32)
                        as s32;
                    (*state)
                        .block_seqs[block_map[0 as i32
                        as usize][i as usize] as usize] = 0 as i32 as s32;
                    fn_803B0120_close(state);
                    return result as i32;
                }
                (*state)
                    .block_ids[block_map[0 as i32 as usize][i as usize]
                    as usize] = blocks_before + i;
                (*state)
                    .block_seqs[block_map[0 as i32 as usize][i as usize]
                    as usize] = seq;
            }
            remaining = 0 as i32 as s32;
        }
        i += 1;
    }
    if async_0 == 0 as i32 {
        result = fn_803B0120_close_result(state);
        if result < 0 as i32 {
            return -(267 as i32);
        }
    } else {
        active_requests.r#type = CARD_ACTIVE_WRITE_FILE_3;
        active_requests.state = state;
        active_requests.callback = callback;
        active_requests.callback_arg = file_idx;
        hsd_804D7998 = -(1 as i32);
    }
    let mut __0: [u8; 4] = [0; 4];
    return 0 as i32;
}
#[inline]
pub unsafe extern "C" fn queueVerifyCardHeader(
    mut state: *mut CardState,
    mut cmd: *mut CardCmd,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
) -> s32 {
    melee_core::ct!("queueVerifyCardHeader");
    let mut block_idx: u32 = 0;
    let mut result: s32 = 0;
    block_idx = 0 as u32;
    while block_idx
        < (0x2f as u32)
            .wrapping_add((*state).header_size)
            .wrapping_add((*state).sector_size)
            .card_div((*state).sector_size)
    {
        (*cmd).r#type = CARD_CMD_VERIFY_HEADER;
        (*cmd).state = state;
        (*cmd).x8.header.banner = banner;
        (*cmd).x8.header.icons = icons;
        (*cmd).x8.header.index = block_idx as s32;
        result = fn_803AC168(cmd) as s32;
        if result < 0 as i32 {
            return result;
        }
        block_idx = block_idx.wrapping_add(1);
    }
    return 0 as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803B0E9C_write_block(
    mut state: *mut CardState,
    mut block_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803B0E9C_write_block");
    let mut write_size: u32 = (*state).sector_size;
    let mut offset: s32 = write_size.wrapping_mul(block_idx as u32) as s32;
    let mut buf: *mut u8 = (*state).sector_buf;
    let mut retries: s32 = 0;
    let mut result: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        result = CARDWrite(
            &raw mut (*state).file_info,
            buf as *mut c_void,
            write_size as s32,
            offset,
        );
        if result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn fn_803B0E9C_write_block_final(
    mut state: *mut CardState,
    mut block_idx: s32,
) -> s32 {
    melee_core::ct!("fn_803B0E9C_write_block_final");
    let mut write_size: u32 = (*state).sector_size;
    let mut retries: s32 = 0;
    let mut result: s32 = 0;
    let mut offset: s32 = write_size.wrapping_mul(block_idx as u32) as s32;
    let mut buf: *mut u8 = (*state).sector_buf;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        result = CARDWrite(
            &raw mut (*state).file_info,
            buf as *mut c_void,
            write_size as s32,
            offset,
        );
        if result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    return result;
}
#[inline]
pub unsafe extern "C" fn fn_803B0E9C_read_first(
    mut state: *mut CardState,
    mut sector_size: u32,
) -> s32 {
    melee_core::ct!("fn_803B0E9C_read_first");
    let mut read_size: u32 = sector_size;
    let mut buf: *mut u8 = (*state).sector_buf;
    let mut retries: s32 = 0;
    let mut result: s32 = 0;
    retries = 0 as i32 as s32;
    while retries < 10 as i32 {
        result = CARDRead(
            &raw mut (*state).file_info,
            buf as *mut c_void,
            read_size as i32,
            0 as i32,
        ) as s32;
        if result != -(1 as i32) {
            break;
        }
        retries += 1;
    }
    return result;
}
pub unsafe extern "C" fn fn_803B0E9C(
    mut state: *mut CardState,
    mut banner: *mut c_void,
    mut icons: *mut u8,
    mut is_new: i32,
    mut async_0: i32,
) -> i32 {
    melee_core::ct!("fn_803B0E9C");
    let mut digest: [u8; 48] = [0; 48];
    let mut cmd_done: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut cmd_clear: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut pad_cmd_clear: [u8; 4] = [0; 4];
    let mut cmd_patch: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut pad_cmd_patch: [u8; 8] = [0; 8];
    let mut cmd_write_header: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut pad_cmd_write_header: [u8; 16] = [0; 16];
    let mut cmd_verify_header: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut pad_cmd_verify_header: [u8; 8] = [0; 8];
    let mut remaining: s32 = 0;
    let mut payload_pos: s32 = 0;
    let mut block_idx: s32 = 0;
    let mut result: s32 = 0;
    let mut has_blocks: s32 = 0;
    let mut sector_size: u32 = 0;
    (*state).header_size = hsd_803AC340(&raw mut (*state).icon_info) as u32;
    if is_new == 0 as i32 {
        if async_0 != 0 as i32 {
            result = queueVerifyCardHeader(
                state,
                &raw mut cmd_verify_header,
                banner,
                icons as *mut c_void,
            );
            if result < 0 as i32 {
                return result as i32;
            }
            cmd_done.r#type = CARD_CMD_CHECK_VERIFIED;
            cmd_done.state = state;
            result = fn_803AC168(&raw mut cmd_done) as s32;
            if result < 0 as i32 {
                return result as i32;
            }
        } else {
            result = fn_803ACD58(state, banner, icons as *mut c_void);
            if result < 0 as i32 {
                return result as i32;
            }
            if result == 0 as i32 {
                return 1 as i32;
            }
        }
    }
    if async_0 != 0 as i32 {
        async_0 = 0 as i32;
        loop {
            sector_size = (*state).sector_size;
            if !((async_0 as u32)
                < (0x2f as u32)
                    .wrapping_add((*state).header_size)
                    .wrapping_add(sector_size)
                    .card_div(sector_size))
            {
                break;
            }
            if (async_0 + 1 as i32) as u32
                == (0x2f as u32)
                    .wrapping_add((*state).header_size)
                    .wrapping_add(sector_size)
                    .card_div(sector_size)
            {
                if is_new != 0 as i32 {
                    let mut clear_buf: *mut c_void = (*state).sector_buf
                        as *mut c_void;
                    let mut zero: s32 = 0 as s32;
                    cmd_clear.r#type = CARD_CMD_CLEAR_BUF;
                    cmd_clear.state = state;
                    cmd_clear.x8.clear.x8 = zero;
                    cmd_clear.x8.clear.xC = zero;
                    cmd_clear.x8.clear.data = clear_buf;
                    cmd_clear.x8.clear.size = sector_size as s32;
                    cmd_clear.x8.clear.x14 = zero;
                    cmd_clear.x8.clear.x0 = zero;
                    result = fn_803AC168(&raw mut cmd_clear) as s32;
                } else {
                    if (*state).file_sizes[0 as i32 as usize]
                        > 0 as i32
                    {
                        has_blocks = 1 as i32 as s32;
                    } else {
                        has_blocks = 0 as i32 as s32;
                    }
                    cmd_patch.r#type = CARD_CMD_READ_BLOCK;
                    cmd_patch.state = state;
                    cmd_patch.x8.read.phys = -(1 as i32)
                        as s32;
                    cmd_patch.x8.read.xC = 0 as i32 as s32;
                    cmd_patch.x8.read.data = ::core::ptr::null_mut::<
                        c_void,
                    >();
                    cmd_patch.x8.read.size = has_blocks;
                    cmd_patch.x8.read.offset = (async_0 as u32)
                        .wrapping_mul(sector_size) as s32;
                    result = fn_803AC168(&raw mut cmd_patch) as s32;
                }
                if result < 0 as i32 {
                    return result as i32;
                }
            }
            cmd_write_header.r#type = CARD_CMD_WRITE_HEADER;
            cmd_write_header.state = state;
            cmd_write_header.x8.header.index = async_0 as s32;
            cmd_write_header.x8.header.banner = banner;
            cmd_write_header.x8.header.icons = icons
                as *mut c_void;
            result = fn_803AC168(&raw mut cmd_write_header) as s32;
            if result < 0 as i32 {
                return result as i32;
            }
            async_0 += 1;
        }
        return 0 as i32;
    }
    sector_size = (*state).sector_size;
    if (*state).header_size.wrapping_add(0x30 as u32) < sector_size {
        if is_new != 0 as i32 {
            memset(
                (*state).sector_buf as *mut c_void,
                0 as i32,
                sector_size as usize,
            );
        } else {
            result = fn_803B0E9C_read_first(state, sector_size);
            if result < 0 as i32 {
                return result as i32;
            }
        }
    }
    memcpy(
        (*state).sector_buf as *mut c_void,
        &raw mut (*state).comment as *mut i8
            as *const c_void,
        ::core::mem::size_of::<[i8; 64]>() as usize,
    );
    payload_pos = 0x40 as i32 as s32;
    match (*state).icon_info.banner_format as i32 {
        2 => {
            let mut dst: *mut c_void = (*state)
                .sector_buf
                .offset(0x40 as i32 as isize) as *mut c_void;
            memcpy(dst, banner, BANNER_LARGE as usize);
            payload_pos = (BANNER_LARGE as usize)
                .wrapping_add(
                    ::core::mem::size_of::<[i8; 64]>() as usize,
                ) as s32;
        }
        1 => {
            let mut dst_0: *mut c_void = (*state)
                .sector_buf
                .offset(0x40 as i32 as isize) as *mut c_void;
            memcpy(dst_0, banner, BANNER_SMALL as usize);
            payload_pos = (BANNER_SMALL as usize)
                .wrapping_add(
                    ::core::mem::size_of::<[i8; 64]>() as usize,
                ) as s32;
        }
        _ => {}
    }
    memset(
        &raw mut digest as *mut u8 as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[u8; 48]>() as usize,
    );
    block_idx = 0 as i32 as s32;
    remaining = (*state).header_size.wrapping_sub(payload_pos as u32) as s32;
    while remaining >= 0 as i32 {
        sector_size = (*state).sector_size;
        if (payload_pos + remaining) as u32 > sector_size {
            memcpy(
                (*state).sector_buf.offset(payload_pos as isize)
                    as *mut c_void,
                icons as *const c_void,
                sector_size.wrapping_sub(payload_pos as u32) as usize,
            );
            sector_size = (*state).sector_size;
            icons = icons
                .offset(sector_size.wrapping_sub(payload_pos as u32) as isize);
            remaining = (remaining as u32)
                .wrapping_sub(sector_size.wrapping_sub(payload_pos as u32)) as s32
                as s32;
            HSD_Checksum(
                (*state).sector_buf,
                sector_size as i32,
                (&raw mut digest as *mut u8)
                    .offset(
                        (block_idx as i32 * 0x10 as i32)
                            as isize,
                    ) as *mut u8 as *mut c_void,
            );
            result = fn_803B0E9C_write_block(state, block_idx);
            if result < 0 as i32 {
                return result as i32;
            }
            payload_pos = 0 as i32 as s32;
            block_idx += 1;
            if (remaining as i32 + 0x30 as i32)
                < (*state).header_size as s32
            {
                if is_new != 0 as i32 {
                    memset(
                        (*state).sector_buf as *mut c_void,
                        0 as i32,
                        (*state).sector_size as usize,
                    );
                } else {
                    let mut read_size: u32 = (*state).sector_size;
                    let mut retries2: s32 = 0;
                    let mut buf: *mut u8 = (*state).sector_buf;
                    retries2 = 0 as i32 as s32;
                    while retries2 < 10 as i32 {
                        result = CARDRead(
                            &raw mut (*state).file_info,
                            buf as *mut c_void,
                            read_size as i32,
                            read_size.wrapping_mul(block_idx as u32)
                                as i32,
                        ) as s32;
                        if result != -(1 as i32) {
                            break;
                        }
                        retries2 += 1;
                    }
                    if result < 0 as i32 {
                        return result as i32;
                    }
                }
            }
        } else {
            memcpy(
                (*state).sector_buf.offset(payload_pos as isize)
                    as *mut c_void,
                icons as *const c_void,
                remaining as usize,
            );
            payload_pos += remaining;
            HSD_Checksum(
                (*state).sector_buf,
                payload_pos as i32,
                (&raw mut digest as *mut u8)
                    .offset(
                        (block_idx as i32 * 0x10 as i32)
                            as isize,
                    ) as *mut u8 as *mut c_void,
            );
            memcpy(
                (*state).sector_buf.offset(payload_pos as isize)
                    as *mut c_void,
                &raw mut digest as *mut u8 as *const c_void,
                0x30 as usize,
            );
            remaining = -(1 as i32) as s32;
            result = fn_803B0E9C_write_block_final(state, block_idx);
            if result < 0 as i32 {
                return result as i32;
            }
            block_idx += 1;
        }
    }
    return 0 as i32;
}
#[inline]
pub unsafe extern "C" fn fn_803B1338_queue_write(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut data: *mut c_void,
    mut size: s32,
    mut file_idx: s32,
    mut init_buf: *mut CardCmdBuf,
    mut buf: *mut CardCmdBuf,
) -> s32 {
    melee_core::ct!("fn_803B1338_queue_write");
    let mut ofs: s32 = fn_803ACBE8(state, phys);
    let mut result: s32 = 0;
    if phys == 0 as i32 {
        if block_id != 0 as i32 {
            return -(0x101 as s32);
        }
        (*init_buf).cmd.r#type = CARD_CMD_READ_BLOCK;
        (*init_buf).cmd.state = state;
        (*init_buf).cmd.x8.read.phys = 0 as i32 as s32;
        (*init_buf).cmd.x8.read.xC = 0 as i32 as s32;
        (*init_buf).cmd.x8.read.data = ::core::ptr::null_mut::<
            c_void,
        >();
        (*init_buf).cmd.x8.read.size = 0 as i32 as s32;
        (*init_buf).cmd.x8.read.offset = ofs;
        fn_803AC168(&raw mut (*init_buf).cmd);
    }
    (*buf).cmd.r#type = CARD_CMD_WRITE_BLOCK;
    (*buf).cmd.state = state;
    (*buf).cmd.x8.write.phys = phys;
    (*buf).cmd.x8.write.block_id = block_id;
    (*buf).cmd.x8.write.seq = 0 as i32 as s32;
    (*buf).cmd.x8.write.data = data;
    (*buf).cmd.x8.write.size = size;
    (*buf).cmd.x8.write.offset = ofs;
    (*buf).cmd.x8.write.file_idx = file_idx;
    result = fn_803AC168(&raw mut (*buf).cmd) as s32;
    return result;
}
#[inline]
pub unsafe extern "C" fn fn_803B1338_data_at(
    mut file_payload: *mut u8,
    mut offset: s32,
) -> *mut u8 {
    melee_core::ct!("fn_803B1338_data_at");
    return file_payload.offset(offset as isize);
}
#[inline]
pub unsafe extern "C" fn fn_803B1338_data_size(mut state: *mut CardState) -> s32 {
    melee_core::ct!("fn_803B1338_data_size");
    return (*state).sector_size.wrapping_sub(0x20 as u32) as s32;
}
#[inline]
pub unsafe extern "C" fn fn_803B1338_queue_write_data(
    mut state: *mut CardState,
    mut phys: s32,
    mut block_id: s32,
    mut fdata: *mut u8,
    mut offset: s32,
    mut file_idx: s32,
    mut init_buf: *mut CardCmdBuf,
    mut buf: *mut CardCmdBuf,
) -> s32 {
    melee_core::ct!("fn_803B1338_queue_write_data");
    let mut ofs: s32 = fn_803ACBE8(state, phys);
    let mut data: *mut u8 = fn_803B1338_data_at(fdata, offset);
    let mut size: s32 = fn_803B1338_data_size(state);
    let mut result: s32 = 0;
    if phys == 0 as i32 {
        if block_id != 0 as i32 {
            return -(0x101 as s32);
        }
        (*init_buf).cmd.r#type = CARD_CMD_READ_BLOCK;
        (*init_buf).cmd.state = state;
        (*init_buf).cmd.x8.read.phys = 0 as i32 as s32;
        (*init_buf).cmd.x8.read.xC = 0 as i32 as s32;
        (*init_buf).cmd.x8.read.data = ::core::ptr::null_mut::<
            c_void,
        >();
        (*init_buf).cmd.x8.read.size = 0 as i32 as s32;
        (*init_buf).cmd.x8.read.offset = ofs;
        fn_803AC168(&raw mut (*init_buf).cmd);
    }
    (*buf).cmd.r#type = CARD_CMD_WRITE_BLOCK;
    (*buf).cmd.state = state;
    (*buf).cmd.x8.write.phys = phys;
    (*buf).cmd.x8.write.block_id = block_id;
    (*buf).cmd.x8.write.seq = 0 as i32 as s32;
    (*buf).cmd.x8.write.data = data as *mut c_void;
    (*buf).cmd.x8.write.size = size;
    (*buf).cmd.x8.write.offset = ofs;
    (*buf).cmd.x8.write.file_idx = file_idx;
    result = fn_803AC168(&raw mut (*buf).cmd) as s32;
    return result;
}
pub unsafe extern "C" fn fn_803B1338(
    mut state: *mut CardState,
    mut async_0: i32,
) -> i32 {
    melee_core::ct!("fn_803B1338");
    let mut cmd0: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd1: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd2: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd3: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd4: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd5: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd6: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd7: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd8: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd9: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd10: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd11: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd12: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd13: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd14: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd15: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd16: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut cmd17: CardCmdBuf = CardCmdBuf {
        cmd: CardCmd {
            r#type: CARD_CMD_NONE,
            state: ::core::ptr::null::<CardState>() as *mut CardState,
            x8: CardCmd_x8 {
                write: CardWriteArgs {
                    file_idx: 0,
                    phys: 0,
                    block_id: 0,
                    seq: 0,
                    data: ::core::ptr::null::<c_void>()
                        as *mut c_void,
                    offset: 0,
                    size: 0,
                },
            },
        },
        pad_24: [0; 4],
    };
    let mut result: s32 = 0;
    let mut phys: s32 = 0;
    let mut spare_blocks: s32 = 0;
    let mut file_blocks: s32 = 0;
    let mut index: s32 = 0;
    let mut block_id: s32 = 0;
    let mut fdata: *mut u8 = ::core::ptr::null_mut::<u8>();
    let mut __0: [u8; 40] = [0; 40];
    spare_blocks = 0 as i32 as s32;
    block_id = 1 as i32 as s32;
    phys = 1 as i32 as s32;
    if fn_803AC634(state, 0 as s32) as s32 != 0 as i32 {
        let mut i: s32 = 0;
        let mut offset: s32 = 0;
        let mut hdr_offset: s32 = 0;
        let mut data_start: u32 = 0;
        let mut sector_size: u32 = 0;
        file_blocks = fn_803AC634(state, 0 as s32) as s32;
        offset = 0 as i32 as s32;
        sector_size = (*state).sector_size;
        data_start = (*state).header_size.wrapping_add(0x30 as u32);
        hdr_offset = data_start.card_rem(sector_size) as s32;
        if async_0 != 0 as i32 {
            fdata = (*state).file_data[0 as i32 as usize];
            if fdata.is_null() {
                let mut ofs0: s32 = sector_size
                    .wrapping_mul(
                        data_start
                            .wrapping_add(sector_size)
                            .wrapping_sub(1 as u32)
                            .card_div(sector_size)
                            .wrapping_sub(1 as u32),
                    ) as s32;
                cmd0.cmd.r#type = CARD_CMD_READ_BLOCK;
                cmd0.cmd.state = state;
                cmd0.cmd.x8.read.phys = 0 as i32 as s32;
                cmd0.cmd.x8.read.xC = 0 as i32 as s32;
                cmd0.cmd.x8.read.data = ::core::ptr::null_mut::<
                    c_void,
                >();
                cmd0.cmd.x8.read.size = 0 as i32 as s32;
                cmd0.cmd.x8.read.offset = ofs0;
                fn_803AC168(&raw mut cmd0.cmd);
                cmd1.cmd.r#type = CARD_CMD_WRITE_BLOCK;
                cmd1.cmd.state = state;
                cmd1.cmd.x8.write.phys = 0 as i32 as s32;
                cmd1.cmd.x8.write.block_id = 0 as i32 as s32;
                cmd1.cmd.x8.write.seq = 0 as i32 as s32;
                cmd1.cmd.x8.write.data = ::core::ptr::null_mut::<
                    c_void,
                >();
                cmd1.cmd.x8.write.size = 0 as i32 as s32;
                cmd1.cmd.x8.write.offset = ofs0;
                cmd1.cmd.x8.write.file_idx = 0 as i32 as s32;
                result = fn_803AC168(&raw mut cmd1.cmd) as s32;
            } else {
                let mut ofs0_0: s32 = sector_size
                    .wrapping_mul(
                        data_start
                            .wrapping_add(sector_size)
                            .wrapping_sub(1 as u32)
                            .card_div(sector_size)
                            .wrapping_sub(1 as u32),
                    ) as s32;
                cmd2.cmd.r#type = CARD_CMD_READ_BLOCK;
                cmd2.cmd.state = state;
                cmd2.cmd.x8.read.phys = 0 as i32 as s32;
                cmd2.cmd.x8.read.xC = 0 as i32 as s32;
                cmd2.cmd.x8.read.data = ::core::ptr::null_mut::<
                    c_void,
                >();
                cmd2.cmd.x8.read.size = 0 as i32 as s32;
                cmd2.cmd.x8.read.offset = ofs0_0;
                fn_803AC168(&raw mut cmd2.cmd);
                cmd3.cmd.r#type = CARD_CMD_WRITE_BLOCK;
                cmd3.cmd.state = state;
                cmd3.cmd.x8.write.phys = 0 as i32 as s32;
                cmd3.cmd.x8.write.block_id = 0 as i32 as s32;
                cmd3.cmd.x8.write.seq = 0 as i32 as s32;
                cmd3.cmd.x8.write.data = fdata as *mut c_void;
                cmd3.cmd.x8.write.size = sector_size
                    .wrapping_sub(hdr_offset as u32)
                    .wrapping_sub(0x20 as u32) as s32;
                cmd3.cmd.x8.write.offset = ofs0_0;
                cmd3.cmd.x8.write.file_idx = 0 as i32 as s32;
                result = fn_803AC168(&raw mut cmd3.cmd) as s32;
                offset = (*state)
                    .sector_size
                    .wrapping_sub(hdr_offset as u32)
                    .wrapping_sub(0x20 as u32) as s32;
            }
            if result < 0 as i32 {
                return result as i32;
            }
        } else {
            fdata = (*state).file_data[0 as i32 as usize];
            if fdata.is_null() {
                result = fn_803ACFC0(
                    state,
                    0 as s32,
                    0 as s32,
                    0 as s32,
                    ::core::ptr::null_mut::<c_void>(),
                    0 as s32,
                    0 as s32,
                );
            } else {
                result = fn_803ACFC0(
                    state,
                    0 as s32,
                    0 as s32,
                    0 as s32,
                    fdata as *mut c_void,
                    sector_size
                        .wrapping_sub(hdr_offset as u32)
                        .wrapping_sub(0x20 as u32) as s32,
                    0 as s32,
                );
                offset = (*state)
                    .sector_size
                    .wrapping_sub(hdr_offset as u32)
                    .wrapping_sub(0x20 as u32) as s32;
            }
            if result < 0 as i32
                && result != -(0x105 as i32)
            {
                (*state).block_ids[0 as i32 as usize] = -(0x7fff
                    as i32) as s32;
                (*state).block_seqs[0 as i32 as usize] = 0
                    as i32 as s32;
                return result as i32;
            }
        }
        i = 1 as i32 as s32;
        while i < file_blocks {
            if async_0 != 0 as i32 {
                fdata = (*state).file_data[0 as i32 as usize];
                if fdata.is_null() {
                    result = fn_803B1338_queue_write(
                        state,
                        phys,
                        block_id,
                        ::core::ptr::null_mut::<c_void>(),
                        0 as s32,
                        0 as s32,
                        &raw mut cmd4,
                        &raw mut cmd5,
                    );
                } else {
                    let mut chunk: s32 = 0;
                    let mut wdata: *mut u8 = ::core::ptr::null_mut::<u8>();
                    wdata = fn_803B1338_data_at(fdata, offset);
                    chunk = fn_803B1338_data_size(state);
                    result = fn_803B1338_queue_write(
                        state,
                        phys,
                        block_id,
                        wdata as *mut c_void,
                        chunk,
                        0 as s32,
                        &raw mut cmd6,
                        &raw mut cmd7,
                    );
                    offset = (*state).sector_size.wrapping_add(offset as u32) as s32;
                    offset -= 0x20 as i32;
                }
                if result < 0 as i32 {
                    return result as i32;
                }
            } else {
                fdata = (*state).file_data[0 as i32 as usize];
                if fdata.is_null() {
                    result = fn_803ACFC0(
                        state,
                        phys,
                        block_id,
                        0 as s32,
                        ::core::ptr::null_mut::<c_void>(),
                        0 as s32,
                        0 as s32,
                    );
                } else {
                    result = fn_803ACFC0(
                        state,
                        phys,
                        block_id,
                        0 as s32,
                        fdata.offset(offset as isize) as *mut c_void,
                        (*state).sector_size.wrapping_sub(0x20 as u32) as s32,
                        0 as s32,
                    );
                    offset = (*state).sector_size.wrapping_add(offset as u32) as s32;
                    offset -= 0x20 as i32;
                }
                if result < 0 as i32 {
                    (*state).block_ids[phys as usize] = -(0x7fff as i32)
                        as s32;
                    (*state).block_seqs[phys as usize] = 0 as i32 as s32;
                    return result as i32;
                }
                (*state).block_ids[phys as usize] = block_id;
                (*state).block_seqs[phys as usize] = 0 as i32 as s32;
            }
            phys += 1;
            block_id += 1;
            i += 1;
        }
    }
    index = 1 as i32 as s32;
    while index < 9 as i32 {
        let mut j: s32 = 0;
        let mut offset_0: s32 = 0;
        file_blocks = fn_803AC7DC_block_count(state, index);
        offset_0 = 0 as i32 as s32;
        if file_blocks > 0 as i32 {
            j = 0 as i32 as s32;
            while j < file_blocks {
                if async_0 != 0 as i32 {
                    fdata = (*state).file_data[index as usize];
                    if fdata.is_null() {
                        result = fn_803B1338_queue_write(
                            state,
                            phys,
                            block_id,
                            ::core::ptr::null_mut::<c_void>(),
                            0 as s32,
                            index,
                            &raw mut cmd8,
                            &raw mut cmd9,
                        );
                    } else {
                        result = fn_803B1338_queue_write_data(
                            state,
                            phys,
                            block_id,
                            fdata,
                            offset_0,
                            index,
                            &raw mut cmd10,
                            &raw mut cmd11,
                        );
                        offset_0 = (*state).sector_size.wrapping_add(offset_0 as u32)
                            as s32;
                        offset_0 -= 0x20 as i32;
                    }
                    if result < 0 as i32 {
                        return result as i32;
                    }
                } else {
                    fdata = (*state).file_data[index as usize];
                    if fdata.is_null() {
                        result = fn_803ACFC0(
                            state,
                            phys,
                            block_id,
                            0 as s32,
                            ::core::ptr::null_mut::<c_void>(),
                            0 as s32,
                            index,
                        );
                    } else {
                        result = fn_803ACFC0(
                            state,
                            phys,
                            block_id,
                            0 as s32,
                            fdata.offset(offset_0 as isize) as *mut c_void,
                            (*state).sector_size.wrapping_sub(0x20 as u32) as s32,
                            index,
                        );
                        offset_0 = (*state).sector_size.wrapping_add(offset_0 as u32)
                            as s32;
                        offset_0 -= 0x20 as i32;
                    }
                    if result < 0 as i32 {
                        (*state).block_ids[phys as usize] = -(0x7fff
                            as i32) as s32;
                        (*state).block_seqs[phys as usize] = 0 as i32
                            as s32;
                        return result as i32;
                    }
                    (*state).block_ids[phys as usize] = block_id;
                    (*state).block_seqs[phys as usize] = 0 as i32 as s32;
                }
                phys += 1;
                block_id += 1;
                j += 1;
            }
            match (*state).file_flags[index as usize] {
                1 => {
                    if spare_blocks < file_blocks {
                        spare_blocks = file_blocks;
                    }
                }
                2 => {
                    if spare_blocks < 1 as i32 {
                        spare_blocks = 1 as i32 as s32;
                    }
                }
                0 | 3 | _ => {}
            }
        }
        index += 1;
    }
    let mut i_0: s32 = 0;
    i_0 = 0 as i32 as s32;
    while i_0 < spare_blocks {
        if async_0 != 0 as i32 {
            result = fn_803B1338_queue_write(
                state,
                phys,
                0xffff as s32,
                ::core::ptr::null_mut::<c_void>(),
                0 as s32,
                index,
                &raw mut cmd12,
                &raw mut cmd13,
            );
            if result < 0 as i32 {
                return result as i32;
            }
        } else {
            result = fn_803ACFC0(
                state,
                phys,
                0xffff as s32,
                0 as s32,
                ::core::ptr::null_mut::<c_void>(),
                0 as s32,
                index,
            );
            (*state).block_ids[phys as usize] = -(0x7fff as i32) as s32;
            (*state).block_seqs[phys as usize] = 0 as i32 as s32;
            if result < 0 as i32 {
                return result as i32;
            }
        }
        block_id += 1;
        phys += 1;
        i_0 += 1;
    }
    let mut block_idx: s32 = 0;
    let mut file_id: s32 = 0;
    index = 1 as i32 as s32;
    if fn_803AC634(state, 0 as s32) as s32 != 0 as i32 {
        index = (index as u32)
            .wrapping_add(fn_803AC634(state, 0 as s32).wrapping_sub(1 as u32)) as s32
            as s32;
    }
    file_id = 1 as i32 as s32;
    while file_id < 9 as i32 {
        let mut offset_1: s32 = 0;
        file_blocks = fn_803AC634(state, file_id) as s32;
        offset_1 = 0 as i32 as s32;
        if file_blocks > 0 as i32 {
            match (*state).file_flags[file_id as usize] {
                0 => {
                    block_idx = 0 as i32 as s32;
                    while block_idx < file_blocks {
                        if async_0 != 0 as i32 {
                            fdata = (*state).file_data[file_id as usize];
                            if fdata.is_null() {
                                result = fn_803B1338_queue_write(
                                    state,
                                    phys,
                                    index,
                                    ::core::ptr::null_mut::<c_void>(),
                                    0 as s32,
                                    file_id,
                                    &raw mut cmd14,
                                    &raw mut cmd15,
                                );
                            } else {
                                result = fn_803B1338_queue_write_data(
                                    state,
                                    phys,
                                    index,
                                    fdata,
                                    offset_1,
                                    file_id,
                                    &raw mut cmd16,
                                    &raw mut cmd17,
                                );
                                offset_1 = (*state)
                                    .sector_size
                                    .wrapping_add(offset_1 as u32) as s32;
                                offset_1 -= 0x20 as i32;
                            }
                            if result < 0 as i32 {
                                return result as i32;
                            }
                        } else {
                            fdata = (*state).file_data[file_id as usize];
                            if fdata.is_null() {
                                result = fn_803ACFC0(
                                    state,
                                    phys,
                                    index,
                                    0 as s32,
                                    ::core::ptr::null_mut::<c_void>(),
                                    0 as s32,
                                    file_id,
                                );
                            } else {
                                result = fn_803ACFC0(
                                    state,
                                    phys,
                                    index,
                                    0 as s32,
                                    fdata.offset(offset_1 as isize) as *mut c_void,
                                    (*state).sector_size.wrapping_sub(0x20 as u32) as s32,
                                    file_id,
                                );
                                offset_1 = (*state)
                                    .sector_size
                                    .wrapping_add(offset_1 as u32) as s32;
                                offset_1 -= 0x20 as i32;
                            }
                            if result < 0 as i32 {
                                (*state).block_ids[phys as usize] = -(0x7fff
                                    as i32) as s32;
                                (*state).block_seqs[phys as usize] = 0 as i32
                                    as s32;
                                return result as i32;
                            }
                            (*state).block_ids[phys as usize] = index;
                            (*state).block_seqs[phys as usize] = 0 as i32
                                as s32;
                        }
                        phys += 1;
                        index += 1;
                        block_idx += 1;
                    }
                }
                1 | 2 | 3 => {
                    index += file_blocks;
                }
                _ => {}
            }
        }
        file_id += 1;
    }
    return 0 as i32;
}
pub unsafe extern "C" fn fn_803B1F78(
    mut state: *mut CardState,
    mut filename: *const i8,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803B1F78");
    let mut cmd_create: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut cmd_set_status: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut result: s32 = 0;
    let mut file_size: s32 = 0;
    let mut saved: s32 = 0;
    let mut snap: s32 = 0;
    let mut saved1: s32 = 0;
    let mut snap1: s32 = 0;
    let mut saved2: s32 = 0;
    let mut snap2: s32 = 0;
    let mut saved3: s32 = 0;
    let mut snap3: s32 = 0;
    let mut saved4: s32 = 0;
    let mut snap4: s32 = 0;
    hsd_804D7998 = curr_tail as i32;
    file_size = (*state).sector_size.wrapping_mul(hsd_803B2674(state) as u32) as s32;
    cmd_create.r#type = CARD_CMD_CREATE_FILE;
    cmd_create.state = state;
    cmd_create.x8.create.filename = filename;
    cmd_create.x8.create.size = file_size;
    result = fn_803AC168(&raw mut cmd_create) as s32;
    if result < 0 as i32 {
        snap1 = hsd_804D7998 as s32;
        if snap1 >= 0 as i32 {
            saved1 = snap1;
            while saved1 != curr_tail {
                commands[saved1 as usize].r#type = CARD_CMD_NONE;
                saved1 = ((saved1 as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap1);
        }
        return result as i32;
    }
    result = fn_803B0E9C(
        state,
        banner,
        icons as *mut u8,
        1 as i32,
        1 as i32,
    ) as s32;
    if result < 0 as i32 {
        snap2 = hsd_804D7998 as s32;
        if snap2 >= 0 as i32 {
            saved2 = snap2;
            while saved2 != curr_tail {
                commands[saved2 as usize].r#type = CARD_CMD_NONE;
                saved2 = ((saved2 as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap2);
        }
        return result as i32;
    }
    result = fn_803B1338(state, 1 as i32) as s32;
    if result < 0 as i32 {
        snap3 = hsd_804D7998 as s32;
        if snap3 >= 0 as i32 {
            saved3 = snap3;
            while saved3 != curr_tail {
                commands[saved3 as usize].r#type = CARD_CMD_NONE;
                saved3 = ((saved3 as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap3);
        }
        return result as i32;
    }
    cmd_set_status.r#type = CARD_CMD_SET_STATUS;
    cmd_set_status.state = state;
    result = fn_803AC168(&raw mut cmd_set_status) as s32;
    if result < 0 as i32 {
        snap = hsd_804D7998 as s32;
        if snap >= 0 as i32 {
            saved = snap;
            while saved != curr_tail {
                commands[saved as usize].r#type = CARD_CMD_NONE;
                saved = ((saved as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
        }
        return result as i32;
    }
    active_requests.r#type = CARD_ACTIVE_CREATE_FILE;
    active_requests.state = state;
    active_requests.callback = callback;
    result = 0 as i32 as s32;
    active_requests.callback_arg = result;
    if result < 0 as i32 {
        snap4 = hsd_804D7998 as s32;
        if snap4 >= 0 as i32 {
            saved4 = snap4;
            while saved4 != curr_tail {
                commands[saved4 as usize].r#type = CARD_CMD_NONE;
                saved4 = ((saved4 as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap4);
        }
        return 0 as i32;
    }
    hsd_804D7998 = -(1 as i32);
    return 0 as i32;
}
pub unsafe extern "C" fn fn_803B21E8(
    mut state: *mut CardState,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803B21E8");
    let mut cmd_set_status: CardCmd = CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        x8: CardCmd_x8 {
            write: CardWriteArgs {
                file_idx: 0,
                phys: 0,
                block_id: 0,
                seq: 0,
                data: ::core::ptr::null::<c_void>()
                    as *mut c_void,
                offset: 0,
                size: 0,
            },
        },
    };
    let mut result: s32 = 0;
    let mut saved: s32 = 0;
    let mut snap: s32 = 0;
    let mut saved1: s32 = 0;
    let mut snap1: s32 = 0;
    let mut saved2: s32 = 0;
    let mut snap2: s32 = 0;
    hsd_804D7998 = curr_tail as i32;
    result = fn_803B0E9C(
        state,
        banner,
        icons as *mut u8,
        0 as i32,
        1 as i32,
    ) as s32;
    if result < 0 as i32 {
        snap1 = hsd_804D7998 as s32;
        if snap1 >= 0 as i32 {
            saved1 = snap1;
            while saved1 != curr_tail {
                commands[saved1 as usize].r#type = CARD_CMD_NONE;
                saved1 = ((saved1 as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap1);
        }
        return result as i32;
    }
    cmd_set_status.r#type = CARD_CMD_SET_STATUS;
    cmd_set_status.state = state;
    result = fn_803AC168(&raw mut cmd_set_status) as s32;
    if result < 0 as i32 {
        snap2 = hsd_804D7998 as s32;
        if snap2 >= 0 as i32 {
            saved2 = snap2;
            while saved2 != curr_tail {
                commands[saved2 as usize].r#type = CARD_CMD_NONE;
                saved2 = ((saved2 as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap2);
        }
        return result as i32;
    }
    active_requests.r#type = CARD_ACTIVE_SET_STATUS;
    active_requests.state = state;
    active_requests.callback = callback;
    result = 0 as i32 as s32;
    active_requests.callback_arg = result;
    if result < 0 as i32 {
        snap = hsd_804D7998 as s32;
        if snap >= 0 as i32 {
            saved = snap;
            while saved != curr_tail {
                commands[saved as usize].r#type = CARD_CMD_NONE;
                saved = ((saved as i32 + 1 as i32)
                    % 128 as i32) as s32;
            }
            ::core::ptr::write_volatile(&mut curr_tail as *mut s32, snap);
        }
        return 0 as i32;
    }
    hsd_804D7998 = -(1 as i32);
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_803B2374() {
    melee_core::ct!("hsd_803B2374");
    let mut i: i32 = 0;
    hsd_804D7990 = 0 as i32 as s32;
    hsd_804D7994 = 0 as i32 as s32;
    memset(
        &raw mut requests as *mut CardRequest as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[CardRequest; 32]>() as usize,
    );
    ::core::ptr::write_volatile(
        &mut curr_head as *mut s32,
        0 as i32 as s32,
    );
    ::core::ptr::write_volatile(
        &mut curr_tail as *mut s32,
        0 as i32 as s32,
    );
    hsd_804D799C = idle;
    i = 0 as i32;
    while i < 128 as i32 {
        commands[i as usize].r#type = CARD_CMD_NONE;
        i += 1;
    }
    ::core::ptr::write_volatile(
        &mut curr_result as *mut s32,
        0 as i32 as s32,
    );
}
pub unsafe extern "C" fn hsd_803B24E4(
    mut state: *mut CardState,
    mut chan: i32,
    mut sector_size: i32,
    mut work_buf: *mut c_void,
) {
    melee_core::ct!("hsd_803B24E4");
    memset(
        state as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<CardState>() as usize,
    );
    (*state).file_no = -(1 as i32) as s32;
    (*state).chan = chan as s32;
    (*state).sector_size = sector_size as u32;
    (*state).sector_buf = work_buf as *mut u8;
}
pub unsafe extern "C" fn hsd_803B2550(
    mut state: *mut CardState,
    mut filename: *const i8,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("hsd_803B2550");
    let mut file_no_copy: s32 = 0;
    let mut chan: s32 = (*state).chan;
    let mut i: s32 = 0;
    let mut file_no_copy2: s32 = 0;
    let mut write_idx: s32 = 0;
    let mut result: s32 = 0;
    let mut file_no: s32 = 0;
    result = retryCardOpen(chan, filename, &raw mut (*state).file_info);
    if result < 0 as i32 {
        return result as i32;
    }
    result = 0 as i32 as s32;
    let mut tmp: s32 = (*state).file_info.fileNo;
    while !(tmp != -(1 as i32)) {
        result += 1;
        if !(result < 10 as i32) {
            break;
        }
    }
    file_no_copy2 = (*state).file_info.fileNo;
    file_no_copy = file_no_copy2;
    file_no = file_no_copy;
    if tmp < 0 as i32 {
        return file_no_copy as i32;
    }
    i = 0 as i32 as s32;
    while i < 10 as i32 {
        if CARDClose(&raw mut (*state).file_info) != -(1 as i32) {
            break;
        }
        i += 1;
    }
    let mut read_idx: s32 = hsd_804D7990;
    write_idx = hsd_804D7994;
    if read_idx == write_idx {
        if requests[read_idx as usize].r#type as i32
            != CARD_REQ_NONE as i32
        {
            return -(265 as i32);
        }
    }
    let mut next: s32 = write_idx + 1 as s32;
    requests[write_idx as usize].r#type = CARD_REQ_OPEN_FILE;
    requests[write_idx as usize].state = state;
    requests[write_idx as usize].x8.open.file_no = file_no;
    requests[write_idx as usize].callback = callback;
    hsd_804D7994 = (next as i32 % 32 as i32) as s32;
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_803B2674(mut state: *mut CardState) -> i32 {
    melee_core::ct!("hsd_803B2674");
    let mut blocks: s32 = 0;
    (*state).header_size = hsd_803AC340(&raw mut (*state).icon_info) as u32;
    blocks = (0x2f as u32)
        .wrapping_add((*state).header_size)
        .wrapping_add((*state).sector_size)
        .card_div((*state).sector_size) as s32;
    blocks += fn_803AC7DC(state);
    return blocks as i32;
}
pub unsafe extern "C" fn fn_803B26CC(
    mut state: *mut CardState,
    mut comment: *mut c_void,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("fn_803B26CC");
    let mut result: s32 = 0;
    (*state).header_size = hsd_803AC340(&raw mut (*state).icon_info) as u32;
    hsd_804D7998 = curr_tail as i32;
    result = queueHeaderBlocks(state, comment, banner, icons);
    if result < 0 as i32 {
        if hsd_804D7998 >= 0 as i32 {
            rollbackCardCommands(hsd_804D7998 as s32);
        }
        return result as i32;
    }
    active_requests.r#type = CARD_ACTIVE_OPEN_OR_READ_HEADER;
    active_requests.state = state;
    active_requests.callback = callback;
    active_requests.callback_arg = 0 as i32 as s32;
    hsd_804D7998 = -(1 as i32);
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_803B27F4(
    mut state: *mut CardState,
    mut comment: *mut c_void,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("hsd_803B27F4");
    let mut read_idx: s32 = hsd_804D7990;
    let mut write_idx: s32 = hsd_804D7994;
    if read_idx == write_idx {
        if requests[read_idx as usize].r#type as i32
            != CARD_REQ_NONE as i32
        {
            return -(265 as i32);
        }
    }
    let mut next: s32 = write_idx + 1 as s32;
    requests[write_idx as usize].r#type = CARD_REQ_READ_HEADER;
    requests[write_idx as usize].state = state;
    requests[write_idx as usize].x8.header.comment = comment;
    requests[write_idx as usize].x8.header.banner = banner;
    requests[write_idx as usize].x8.header.icons = icons;
    requests[write_idx as usize].callback = callback;
    hsd_804D7994 = (next as i32 % 32 as i32) as s32;
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_803B286C(
    mut state: *mut CardState,
    mut filename: *const i8,
    mut comment: *const i8,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("hsd_803B286C");
    memcpy(
        &raw mut (*state).comment as *mut i8
            as *mut c_void,
        comment as *const c_void,
        64 as usize,
    );
    let mut write_idx: s32 = 0;
    let mut read_idx: s32 = hsd_804D7990;
    write_idx = hsd_804D7994;
    if read_idx == write_idx {
        if requests[read_idx as usize].r#type as i32
            != CARD_REQ_NONE as i32
        {
            return -(265 as i32);
        }
    }
    requests[write_idx as usize].r#type = CARD_REQ_CREATE_FILE;
    requests[write_idx as usize].state = state;
    requests[write_idx as usize].x8.create.filename = filename;
    requests[write_idx as usize].x8.create.banner = banner;
    requests[write_idx as usize].x8.create.icons = icons;
    requests[write_idx as usize].callback = callback;
    hsd_804D7994 = ((write_idx as i32 + 1 as i32)
        % 32 as i32) as s32;
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_803B2928(
    mut state: *mut CardState,
    mut comment: *const i8,
    mut banner: *mut c_void,
    mut icons: *mut c_void,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("hsd_803B2928");
    memcpy(
        &raw mut (*state).comment as *mut i8
            as *mut c_void,
        comment as *const c_void,
        64 as usize,
    );
    let mut write_idx: s32 = 0;
    let mut read_idx: s32 = hsd_804D7990;
    write_idx = hsd_804D7994;
    if read_idx == write_idx {
        if requests[read_idx as usize].r#type as i32
            != CARD_REQ_NONE as i32
        {
            return -(265 as i32);
        }
    }
    requests[write_idx as usize].r#type = CARD_REQ_SET_STATUS;
    requests[write_idx as usize].state = state;
    requests[write_idx as usize].x8.status.banner = banner;
    requests[write_idx as usize].x8.status.icons = icons;
    requests[write_idx as usize].callback = callback;
    hsd_804D7994 = ((write_idx as i32 + 1 as i32)
        % 32 as i32) as s32;
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_803B29D8(
    mut state: *mut CardState,
    mut file_idx: i32,
    mut buf: *mut u8,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("hsd_803B29D8");
    let mut read_idx: s32 = hsd_804D7990;
    let mut write_idx: s32 = hsd_804D7994;
    if read_idx == write_idx {
        if requests[read_idx as usize].r#type as i32
            != CARD_REQ_NONE as i32
        {
            return -(265 as i32);
        }
    }
    let mut next: s32 = write_idx + 1 as s32;
    requests[write_idx as usize].r#type = CARD_REQ_READ_FILE;
    requests[write_idx as usize].state = state;
    requests[write_idx as usize].x8.file.file_idx = file_idx as s32;
    requests[write_idx as usize].x8.file.buf = buf;
    requests[write_idx as usize].callback = callback;
    hsd_804D7994 = (next as i32 % 32 as i32) as s32;
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_803B2A4C(
    mut state: *mut CardState,
    mut file_idx: i32,
    mut buf: *mut u8,
    mut callback: CardCallback,
) -> i32 {
    melee_core::ct!("hsd_803B2A4C");
    let mut read_idx: s32 = 0;
    let mut write_idx: s32 = 0;
    if (*state).file_sizes[file_idx as usize] <= 0 as i32 {
        return -(257 as i32);
    }
    read_idx = hsd_804D7990;
    write_idx = hsd_804D7994;
    if read_idx == write_idx {
        if requests[read_idx as usize].r#type as i32
            != CARD_REQ_NONE as i32
        {
            return -(265 as i32);
        }
    }
    let mut next: s32 = write_idx + 1 as s32;
    requests[write_idx as usize].r#type = CARD_REQ_WRITE_FILE;
    requests[write_idx as usize].state = state;
    requests[write_idx as usize].x8.file.file_idx = file_idx as s32;
    requests[write_idx as usize].x8.file.buf = buf;
    requests[write_idx as usize].callback = callback;
    hsd_804D7994 = (next as i32 % 32 as i32) as s32;
    return 0 as i32;
}
pub unsafe extern "C" fn hsd_SetCardIconInfo(
    mut state: *mut CardState,
    mut icon_info: *mut CardIconInfo,
) -> i32 {
    melee_core::ct!("hsd_SetCardIconInfo");
    memcpy(
        &raw mut (*state).icon_info as *mut c_void,
        icon_info as *const c_void,
        ::core::mem::size_of::<CardIconInfo>() as usize,
    );
    (*state).header_size = hsd_803AC340(&raw mut (*state).icon_info) as u32;
    return 0 as i32;
}
pub const NULL: i32 = 0 as i32;
