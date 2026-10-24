// Generated fixture mirror of oracle/card_driver.c by run_card.py --mirror.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments, unused_variables, unused_unsafe, unused_parens, static_mut_refs, dead_code, clippy::all)]
use core::ffi::c_void;
use melee_types::*;
use melee_runtime::tu::card::*;
use melee_runtime::tu::card as m;
const idle: i32 = 2;
const running_async: i32 = 1;
const running: i32 = 0;
extern "C" {
    fn memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn memset(d: *mut c_void, v: i32, n: usize) -> *mut c_void;
}
unsafe fn HSD_Encrypt(p: *mut u8, n: i32) -> i32 { melee_runtime::hsd_util::encrypt(core::slice::from_raw_parts_mut(p,n as usize)) }
unsafe fn HSD_Decrypt(p: *mut u8, n: i32) -> i32 { melee_runtime::hsd_util::decrypt(core::slice::from_raw_parts_mut(p,n as usize)) }
static mut card_oracle_state: CardState = CardState {
    sector_buf: ::core::ptr::null::<u8>() as *mut u8,
    chan: 0,
    sector_size: 0,
    file_info: CARDFileInfo {
        chan: 0,
        fileNo: 0,
        offset: 0,
        length: 0,
        iBlock: 0,
    },
    file_no: 0,
    header_size: 0,
    file_flags: [0; 9],
    file_sizes: [0; 9],
    file_data: [::core::ptr::null::<u8>() as *mut u8; 9],
    pad_94: [0; 220],
    block_ids: [0; 64],
    block_seqs: [0; 64],
    comment: [0; 64],
    icon_info: CardIconInfo {
        banner_format: 0,
        unused: 0,
        icon_format: [0; 8],
        icon_speed: [0; 8],
    },
    stat: CARDStat {
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
    },
    digest: [0; 48],
    num_blocks: 0,
};

static mut card_oracle_cmd: CardCmd = CardCmd {
    r#type: CARD_CMD_NONE,
    state: ::core::ptr::null::<CardState>() as *mut CardState,
    anon_8: CardCmd_anon_193 {
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
};

static mut card_oracle_cmd_buf: [CardCmdBuf; 2] = [CardCmdBuf {
    cmd: CardCmd {
        r#type: CARD_CMD_NONE,
        state: ::core::ptr::null::<CardState>() as *mut CardState,
        anon_8: CardCmd_anon_193 {
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
}; 2];

static mut card_oracle_stat: CARDStat = CARDStat {
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

static mut card_oracle_icon: CardIconInfo = CardIconInfo {
    banner_format: 0,
    unused: 0,
    icon_format: [0; 8],
    icon_speed: [0; 8],
};

static mut card_oracle_sector: [u8; 32768] = [0; 32768];

static mut card_oracle_data: [u8; 65536] = [0; 65536];

static mut card_oracle_banner: [u8; 8192] = [0; 8192];

static mut card_oracle_icons: [u8; 32768] = [0; 32768];

static mut card_oracle_comment: [u8; 64] = [0; 64];

static mut card_oracle_disk: [u8; 262144] = [0; 262144];

static mut card_oracle_primary: [s32; 64] = [0; 64];

static mut card_oracle_filename: [i8; 16] = unsafe {
    ::core::mem::transmute::<
        [u8; 16],
        [i8; 16],
    >(*b"native-card\0\0\0\0\0")
};

static mut card_oracle_calls: u32 = 0;

static mut card_oracle_trace: u32 = 0;

static mut card_oracle_hash: u32 = 0;

static mut card_oracle_busy: i32 = 0;

static mut card_oracle_result: i32 = 0;

static mut card_oracle_k: i32 = 0;

static mut card_oracle_op: i32 = 0;

static mut card_oracle_out1: i32 = 0;

static mut card_oracle_out2: i32 = 0;

static mut card_oracle_pending: CARDCallback = None;

static mut card_oracle_pending_chan: i32 = 0;

static mut card_oracle_hit: [u32; 110] = [0; 110];

unsafe extern "C" fn card_oracle_h(mut v: u32) {
    card_oracle_hash = card_oracle_hash.wrapping_mul(0x1000193 as u32) ^ v;
}

unsafe extern "C" fn card_oracle_t(mut v: u32) {
    card_oracle_trace = card_oracle_trace.wrapping_mul(31 as u32).wrapping_add(v);
}

unsafe extern "C" fn card_oracle_ptr(mut p: *const c_void) -> u32 {
    let mut a: u64 = p as u64;
    if p.is_null() {
        return 0 as u32;
    }
    if a >= &raw mut card_oracle_state as u64
        && (a as usize)
            < (&raw mut card_oracle_state as usize)
                .wrapping_add(::core::mem::size_of::<CardState>() as usize)
    {
        return ((1 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_state as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_cmd as u64
        && (a as usize)
            < (&raw mut card_oracle_cmd as usize)
                .wrapping_add(::core::mem::size_of::<CardCmd>() as usize)
    {
        return ((2 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_cmd as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_cmd_buf as u64
        && (a as usize)
            < (&raw mut card_oracle_cmd_buf as usize)
                .wrapping_add(::core::mem::size_of::<[CardCmdBuf; 2]>() as usize)
    {
        return ((3 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_cmd_buf as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_stat as u64
        && (a as usize)
            < (&raw mut card_oracle_stat as usize)
                .wrapping_add(::core::mem::size_of::<CARDStat>() as usize)
    {
        return ((4 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_stat as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_icon as u64
        && (a as usize)
            < (&raw mut card_oracle_icon as usize)
                .wrapping_add(::core::mem::size_of::<CardIconInfo>() as usize)
    {
        return ((5 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_icon as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_sector as u64
        && (a as usize)
            < (&raw mut card_oracle_sector as usize)
                .wrapping_add(::core::mem::size_of::<[u8; 32768]>() as usize)
    {
        return ((6 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_sector as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_data as u64
        && (a as usize)
            < (&raw mut card_oracle_data as usize)
                .wrapping_add(::core::mem::size_of::<[u8; 65536]>() as usize)
    {
        return ((7 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_data as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_banner as u64
        && (a as usize)
            < (&raw mut card_oracle_banner as usize)
                .wrapping_add(::core::mem::size_of::<[u8; 8192]>() as usize)
    {
        return ((8 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_banner as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_icons as u64
        && (a as usize)
            < (&raw mut card_oracle_icons as usize)
                .wrapping_add(::core::mem::size_of::<[u8; 32768]>() as usize)
    {
        return ((9 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_icons as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_comment as u64
        && (a as usize)
            < (&raw mut card_oracle_comment as usize)
                .wrapping_add(::core::mem::size_of::<[u8; 64]>() as usize)
    {
        return ((10 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_comment as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_disk as u64
        && (a as usize)
            < (&raw mut card_oracle_disk as usize)
                .wrapping_add(::core::mem::size_of::<[u8; 262144]>() as usize)
    {
        return ((11 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_disk as u64)
                as u32;
    }
    if a >= &raw mut card_oracle_primary as u64
        && (a as usize)
            < (&raw mut card_oracle_primary as usize)
                .wrapping_add(::core::mem::size_of::<[s32; 64]>() as usize)
    {
        return ((12 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw mut card_oracle_primary as u64)
                as u32;
    }
    if a >= &raw const card_oracle_filename as u64
        && (a as usize)
            < (&raw const card_oracle_filename as usize)
                .wrapping_add(
                    ::core::mem::size_of::<[i8; 16]>() as usize,
                )
    {
        return ((13 as i32) << 24 as i32) as u32
            | a.wrapping_sub(&raw const card_oracle_filename as u64)
                as u32;
    }
    return 0xffffffff as u32;
}

unsafe extern "C" fn card_oracle_status() -> i32 {
    card_oracle_calls = card_oracle_calls.wrapping_add(1);
    if card_oracle_busy > 0 as i32 {
        card_oracle_busy -= 1;
        return -(1 as i32);
    }
    return card_oracle_result;
}

unsafe extern "C" fn card_oracle_callback(
    mut f: i32,
    mut r: i32,
) {
    card_oracle_t(900 as u32);
    card_oracle_t(f as u32);
    card_oracle_t(r as u32);
}

pub unsafe extern "C" fn card_oracle_OSDisableInterrupts() -> BOOL {
    card_oracle_t(1 as u32);
    return 1 as BOOL;
}

pub unsafe extern "C" fn card_oracle_OSRestoreInterrupts(mut level: BOOL) -> BOOL {
    card_oracle_t(2 as u32);
    card_oracle_t(level as u32);
    return level;
}

pub unsafe extern "C" fn card_oracle_CARDOpen(
    mut chan: s32,
    mut filename: *const i8,
    mut info: *mut CARDFileInfo,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(3 as u32);
    card_oracle_t(chan as u32);
    card_oracle_t(card_oracle_ptr(filename as *const c_void));
    card_oracle_t(card_oracle_ptr(info as *const c_void));
    r = card_oracle_status();
    if r == 0 {
        (*info).chan = chan;
        (*info).fileNo = 7 as i32 as s32;
    }
    return r as s32;
}

pub unsafe extern "C" fn card_oracle_CARDFastOpen(
    mut chan: s32,
    mut no: s32,
    mut info: *mut CARDFileInfo,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(4 as u32);
    card_oracle_t(chan as u32);
    card_oracle_t(no as u32);
    card_oracle_t(card_oracle_ptr(info as *const c_void));
    r = card_oracle_status();
    if r == 0 {
        (*info).chan = chan;
        (*info).fileNo = no;
    }
    return r as s32;
}

pub unsafe extern "C" fn card_oracle_CARDClose(mut info: *mut CARDFileInfo) -> s32 {
    card_oracle_t(5 as u32);
    card_oracle_t(card_oracle_ptr(info as *const c_void));
    return card_oracle_status() as s32;
}

pub unsafe extern "C" fn card_oracle_CARDGetXferredBytes(
    mut chan: i32,
) -> i32 {
    card_oracle_t(6 as u32);
    card_oracle_t(chan as u32);
    return (768 as i32 + card_oracle_k) as i32;
}

pub unsafe extern "C" fn card_oracle_CARDRead(
    mut info: *mut CARDFileInfo,
    mut buf: *mut c_void,
    mut len: i32,
    mut off: i32,
) -> i32 {
    let mut r: i32 = 0;
    card_oracle_t(7 as u32);
    card_oracle_t(card_oracle_ptr(info as *const c_void));
    card_oracle_t(card_oracle_ptr(buf));
    card_oracle_t(len as u32);
    card_oracle_t(off as u32);
    r = card_oracle_status();
    if r == 0 && off >= 0 as i32 && len >= 0 as i32
        && (off + len) as usize <= ::core::mem::size_of::<[u8; 262144]>() as usize
    {
        memcpy(
            buf,
            (&raw mut card_oracle_disk as *mut u8).offset(off as isize)
                as *const c_void,
            len as usize,
        );
    }
    return r as i32;
}

pub unsafe extern "C" fn card_oracle_CARDWrite(
    mut info: *mut CARDFileInfo,
    mut buf: *mut c_void,
    mut len: s32,
    mut off: s32,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(8 as u32);
    card_oracle_t(card_oracle_ptr(info as *const c_void));
    card_oracle_t(card_oracle_ptr(buf));
    card_oracle_t(len as u32);
    card_oracle_t(off as u32);
    r = card_oracle_status();
    if r == 0 && off >= 0 as i32 && len >= 0 as i32
        && (off + len) as usize <= ::core::mem::size_of::<[u8; 262144]>() as usize
    {
        memcpy(
            (&raw mut card_oracle_disk as *mut u8).offset(off as isize)
                as *mut c_void,
            buf,
            len as usize,
        );
    }
    return r as s32;
}

pub unsafe extern "C" fn card_oracle_CARDReadAsync(
    mut info: *mut CARDFileInfo,
    mut buf: *mut c_void,
    mut len: s32,
    mut off: s32,
    mut cb: CARDCallback,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(9 as u32);
    card_oracle_t(cb.is_some() as i32 as u32);
    r = card_oracle_CARDRead(
        info,
        buf,
        len as i32,
        off as i32,
    ) as i32;
    if r == 0 {
        card_oracle_pending = cb;
        card_oracle_pending_chan = (*info).chan as i32;
    }
    return r as s32;
}

pub unsafe extern "C" fn card_oracle_CARDWriteAsync(
    mut info: *mut CARDFileInfo,
    mut buf: *mut c_void,
    mut len: s32,
    mut off: s32,
    mut cb: CARDCallback,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(10 as u32);
    card_oracle_t(cb.is_some() as i32 as u32);
    r = card_oracle_CARDWrite(info, buf, len, off) as i32;
    if r == 0 {
        card_oracle_pending = cb;
        card_oracle_pending_chan = (*info).chan as i32;
    }
    return r as s32;
}

pub unsafe extern "C" fn card_oracle_CARDCreateAsync(
    mut chan: s32,
    mut filename: *const i8,
    mut size: u32,
    mut info: *mut CARDFileInfo,
    mut cb: CARDCallback,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(11 as u32);
    card_oracle_t(chan as u32);
    card_oracle_t(card_oracle_ptr(filename as *const c_void));
    card_oracle_t(size);
    card_oracle_t(card_oracle_ptr(info as *const c_void));
    card_oracle_t(cb.is_some() as i32 as u32);
    r = card_oracle_status();
    if r == 0 {
        (*info).chan = chan;
        (*info).fileNo = 7 as i32 as s32;
        (*info).length = size as s32;
        card_oracle_pending = cb;
        card_oracle_pending_chan = chan as i32;
    }
    return r as s32;
}

pub unsafe extern "C" fn card_oracle_CARDGetStatus(
    mut chan: s32,
    mut no: s32,
    mut stat: *mut CARDStat,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(12 as u32);
    card_oracle_t(chan as u32);
    card_oracle_t(no as u32);
    card_oracle_t(card_oracle_ptr(stat as *const c_void));
    r = card_oracle_status();
    if r == 0 {
        *stat = card_oracle_stat;
    }
    return r as s32;
}

pub unsafe extern "C" fn card_oracle_CARDSetStatusAsync(
    mut chan: s32,
    mut no: s32,
    mut stat: *mut CARDStat,
    mut cb: CARDCallback,
) -> s32 {
    let mut r: i32 = 0;
    card_oracle_t(13 as u32);
    card_oracle_t(chan as u32);
    card_oracle_t(no as u32);
    card_oracle_t(card_oracle_ptr(stat as *const c_void));
    card_oracle_t(cb.is_some() as i32 as u32);
    r = card_oracle_status();
    if r == 0 {
        card_oracle_stat = *stat;
        card_oracle_pending = cb;
        card_oracle_pending_chan = chan as i32;
    }
    return r as s32;
}

unsafe extern "C" fn card_oracle_bytes(mut p: *const u8, mut n: i32) {
    let mut i: i32 = 0;
    i = 0 as i32;
    while i < n {
        card_oracle_h(*p.offset(i as isize) as u32);
        i += 1;
    }
}

unsafe extern "C" fn card_oracle_card_bytes(
    mut p: *const u8,
    mut n: i32,
) {
    let mut block: [u8; 8192] = [0; 8192];
    let mut at: i32 = 0;
    let mut i: i32 = 0;
    if card_oracle_op != 96 as i32 {
        card_oracle_bytes(p, n);
        return;
    }
    at = 0 as i32;
    while at < n {
        let mut encrypted: i32 = 0;
        let mut changed: i32 = 0 as i32;
        memcpy(
            &raw mut block as *mut u8 as *mut c_void,
            p.offset(at as isize) as *const c_void,
            8192 as usize,
        );
        encrypted = (HSD_Decrypt(&raw mut block as *mut u8, 8192 as i32)
            == 0 as i32) as i32;
        if encrypted == 0 {
            memcpy(
                &raw mut block as *mut u8 as *mut c_void,
                p.offset(at as isize) as *const c_void,
                8192 as usize,
            );
        }
        if block[16 as i32 as usize] as i32
            == 255 as i32
            && block[17 as i32 as usize] as i32
                == 255 as i32
        {
            i = 19 as i32;
            while i < 31 as i32 {
                if block[i as usize] as i32 == 9 as i32 {
                    let mut got: u32 = ((block[(i + 1 as i32) as usize]
                        as i32 & 63 as i32)
                        << 16 as i32
                        | (block[(i + 2 as i32) as usize]
                            as i32) << 8 as i32
                        | block[(i + 3 as i32) as usize]
                            as i32) as u32;
                    let mut live: u32 = card_oracle_state
                        .file_data[0 as i32 as usize]
                        as u64 as u32;
                    let mut token: u32 = card_oracle_ptr(
                        card_oracle_state.file_data[0 as i32 as usize]
                            as *const c_void,
                    );
                    card_oracle_h(
                        (got == live & 0x3fffff as i32 as u32)
                            as i32 as u32,
                    );
                    block[(i + 1 as i32) as usize] = ((block[(i
                        + 1 as i32) as usize] as i32
                        & 192 as i32) as u32
                        | token >> 16 as i32 & 63 as u32) as u8;
                    block[(i + 2 as i32) as usize] = (token
                        >> 8 as i32) as u8;
                    block[(i + 3 as i32) as usize] = token as u8;
                    changed = 1 as i32;
                }
                i += 4 as i32;
            }
        }
        if changed != 0 && encrypted != 0 {
            HSD_Encrypt(&raw mut block as *mut u8, 8192 as i32);
        }
        if changed != 0 {
            card_oracle_bytes(&raw mut block as *mut u8, 8192 as i32);
        } else {
            card_oracle_bytes(p.offset(at as isize), 8192 as i32);
        }
        at += 8192 as i32;
    }
}

unsafe extern "C" fn card_oracle_hash_cmd(mut c: *const CardCmd) {
    card_oracle_h((*c).r#type as u32);
    card_oracle_h(card_oracle_ptr((*c).state as *const c_void));
    match (*c).r#type as i32 {
        1 | 2 | 5 | 4 | 13 | 15 | 16 => {
            card_oracle_h((*c).anon_8.write.file_idx as u32);
            card_oracle_h((*c).anon_8.write.phys as u32);
            card_oracle_h((*c).anon_8.write.block_id as u32);
            card_oracle_h((*c).anon_8.write.seq as u32);
            card_oracle_h(card_oracle_ptr((*c).anon_8.write.data));
            card_oracle_h((*c).anon_8.write.offset as u32);
            card_oracle_h((*c).anon_8.write.size as u32);
        }
        7 => {
            card_oracle_h(
                card_oracle_ptr(
                    (*c).anon_8.create.filename as *const c_void,
                ),
            );
            card_oracle_h((*c).anon_8.create.size as u32);
        }
        9 | 10 => {
            card_oracle_h((*c).anon_8.header.index as u32);
            card_oracle_h(card_oracle_ptr((*c).anon_8.header.banner));
            card_oracle_h(card_oracle_ptr((*c).anon_8.header.icons));
        }
        11 => {
            card_oracle_h((*c).anon_8.read_header.index as u32);
            card_oracle_h(card_oracle_ptr((*c).anon_8.read_header.comment));
            card_oracle_h(card_oracle_ptr((*c).anon_8.read_header.banner));
            card_oracle_h(card_oracle_ptr((*c).anon_8.read_header.icons));
        }
        12 => {
            card_oracle_h((*c).anon_8.get_status.file_no as u32);
        }
        _ => {}
    };
}

unsafe extern "C" fn card_oracle_hash_req(mut r: *const CardRequest) {
    card_oracle_h((*r).r#type as u32);
    card_oracle_h(card_oracle_ptr((*r).state as *const c_void));
    card_oracle_h((*r).callback.is_some() as i32 as u32);
    match (*r).r#type as i32 {
        1 | 2 => {
            card_oracle_h((*r).anon_8.file.file_idx as u32);
            card_oracle_h(
                card_oracle_ptr(
                    (*r).anon_8.file.buf as *const c_void,
                ),
            );
        }
        3 => {
            card_oracle_h(
                card_oracle_ptr(
                    (*r).anon_8.create.filename as *const c_void,
                ),
            );
            card_oracle_h(card_oracle_ptr((*r).anon_8.create.banner));
            card_oracle_h(card_oracle_ptr((*r).anon_8.create.icons));
        }
        4 => {
            card_oracle_h((*r).anon_8.status.x0 as u32);
            card_oracle_h(card_oracle_ptr((*r).anon_8.status.banner));
            card_oracle_h(card_oracle_ptr((*r).anon_8.status.icons));
        }
        5 => {
            card_oracle_h((*r).anon_8.open.file_no as u32);
        }
        6 => {
            card_oracle_h(card_oracle_ptr((*r).anon_8.header.comment));
            card_oracle_h(card_oracle_ptr((*r).anon_8.header.banner));
            card_oracle_h(card_oracle_ptr((*r).anon_8.header.icons));
        }
        _ => {}
    };
}

unsafe extern "C" fn card_oracle_hash_CardState(mut p: *const CardState) {
    let mut i0: i32 = 0;
    let mut i1: i32 = 0;
    let mut i2: i32 = 0;
    let mut j0: i32 = 0;
    let mut j1: i32 = 0;
    let mut j2: i32 = 0;
    let mut j3: i32 = 0;
    card_oracle_h(card_oracle_ptr((*p).sector_buf as *const c_void));
    card_oracle_h((*p).chan as u32);
    card_oracle_h((*p).sector_size);
    card_oracle_h((*p).file_info.chan as u32);
    card_oracle_h((*p).file_info.fileNo as u32);
    card_oracle_h((*p).file_info.offset as u32);
    card_oracle_h((*p).file_info.length as u32);
    card_oracle_h((*p).file_info.iBlock as u32);
    card_oracle_h((*p).file_no as u32);
    card_oracle_h((*p).header_size);
    i0 = 0 as i32;
    while i0 < 9 as i32 {
        card_oracle_h((*p).file_flags[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 9 as i32 {
        card_oracle_h((*p).file_sizes[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 9 as i32 {
        card_oracle_h(
            card_oracle_ptr((*p).file_data[i0 as usize] as *const c_void),
        );
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 220 as i32 {
        card_oracle_h((*p).pad_94[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 64 as i32 {
        card_oracle_h((*p).block_ids[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 64 as i32 {
        card_oracle_h((*p).block_seqs[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 64 as i32 {
        card_oracle_h((*p).comment[i0 as usize] as u32);
        i0 += 1;
    }
    card_oracle_h((*p).icon_info.banner_format as u32);
    card_oracle_h((*p).icon_info.unused as u32);
    i0 = 0 as i32;
    while i0 < 8 as i32 {
        card_oracle_h((*p).icon_info.icon_format[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 8 as i32 {
        card_oracle_h((*p).icon_info.icon_speed[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 32 as i32 {
        card_oracle_h((*p).stat.fileName[i0 as usize] as u32);
        i0 += 1;
    }
    card_oracle_h((*p).stat.length);
    card_oracle_h((*p).stat.time);
    i0 = 0 as i32;
    while i0 < 4 as i32 {
        card_oracle_h((*p).stat.gameName[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 2 as i32 {
        card_oracle_h((*p).stat.company[i0 as usize] as u32);
        i0 += 1;
    }
    card_oracle_h((*p).stat.bannerFormat as u32);
    card_oracle_h((*p).stat.iconAddr);
    card_oracle_h((*p).stat.iconFormat as u32);
    card_oracle_h((*p).stat.iconSpeed as u32);
    card_oracle_h((*p).stat.commentAddr);
    card_oracle_h((*p).stat.offsetBanner);
    card_oracle_h((*p).stat.offsetBannerTlut);
    i0 = 0 as i32;
    while i0 < 8 as i32 {
        card_oracle_h((*p).stat.offsetIcon[i0 as usize]);
        i0 += 1;
    }
    card_oracle_h((*p).stat.offsetIconTlut);
    card_oracle_h((*p).stat.offsetData);
    i0 = 0 as i32;
    while i0 < 48 as i32 {
        card_oracle_h((*p).digest[i0 as usize] as u32);
        i0 += 1;
    }
    card_oracle_h((*p).num_blocks as u32);
}

unsafe extern "C" fn card_oracle_hash_CARDStat(mut p: *const CARDStat) {
    let mut i0: i32 = 0;
    let mut i1: i32 = 0;
    let mut i2: i32 = 0;
    let mut j0: i32 = 0;
    let mut j1: i32 = 0;
    let mut j2: i32 = 0;
    let mut j3: i32 = 0;
    i0 = 0 as i32;
    while i0 < 32 as i32 {
        card_oracle_h((*p).fileName[i0 as usize] as u32);
        i0 += 1;
    }
    card_oracle_h((*p).length);
    card_oracle_h((*p).time);
    i0 = 0 as i32;
    while i0 < 4 as i32 {
        card_oracle_h((*p).gameName[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 2 as i32 {
        card_oracle_h((*p).company[i0 as usize] as u32);
        i0 += 1;
    }
    card_oracle_h((*p).bannerFormat as u32);
    card_oracle_h((*p).iconAddr);
    card_oracle_h((*p).iconFormat as u32);
    card_oracle_h((*p).iconSpeed as u32);
    card_oracle_h((*p).commentAddr);
    card_oracle_h((*p).offsetBanner);
    card_oracle_h((*p).offsetBannerTlut);
    i0 = 0 as i32;
    while i0 < 8 as i32 {
        card_oracle_h((*p).offsetIcon[i0 as usize]);
        i0 += 1;
    }
    card_oracle_h((*p).offsetIconTlut);
    card_oracle_h((*p).offsetData);
}

unsafe extern "C" fn card_oracle_hash_CardIconInfo(mut p: *const CardIconInfo) {
    let mut i0: i32 = 0;
    let mut i1: i32 = 0;
    let mut i2: i32 = 0;
    let mut j0: i32 = 0;
    let mut j1: i32 = 0;
    let mut j2: i32 = 0;
    let mut j3: i32 = 0;
    card_oracle_h((*p).banner_format as u32);
    card_oracle_h((*p).unused as u32);
    i0 = 0 as i32;
    while i0 < 8 as i32 {
        card_oracle_h((*p).icon_format[i0 as usize] as u32);
        i0 += 1;
    }
    i0 = 0 as i32;
    while i0 < 8 as i32 {
        card_oracle_h((*p).icon_speed[i0 as usize] as u32);
        i0 += 1;
    }
}

unsafe extern "C" fn card_oracle_hash_all() {
    let mut i: i32 = 0;
    card_oracle_hash_CardState(&raw mut card_oracle_state);
    card_oracle_hash_CARDStat(&raw mut card_oracle_stat);
    card_oracle_hash_CardIconInfo(&raw mut card_oracle_icon);
    card_oracle_hash_cmd(&raw mut card_oracle_cmd);
    i = 0 as i32;
    while i < 2 as i32 {
        card_oracle_hash_cmd(
            &raw mut (*(&raw mut card_oracle_cmd_buf as *mut CardCmdBuf)
                .offset(i as isize))
                .cmd,
        );
        card_oracle_bytes(
            &raw mut (*(&raw mut card_oracle_cmd_buf as *mut CardCmdBuf)
                .offset(i as isize))
                .pad_24 as *mut i8 as *mut u8,
            4 as i32,
        );
        i += 1;
    }
    i = 0 as i32;
    while i < 128 as i32 {
        card_oracle_hash_cmd(
            (&raw mut commands as *mut CardCmd).offset(i as isize) as *mut CardCmd,
        );
        i += 1;
    }
    i = 0 as i32;
    while i < 32 as i32 {
        card_oracle_hash_req(
            (&raw mut requests as *mut CardRequest).offset(i as isize)
                as *mut CardRequest,
        );
        i += 1;
    }
    card_oracle_h(curr_head as u32);
    card_oracle_h(curr_tail as u32);
    card_oracle_h(curr_result as u32);
    card_oracle_h(hsd_804D798C as u32);
    card_oracle_h(hsd_804D7990 as u32);
    card_oracle_h(hsd_804D7994 as u32);
    card_oracle_h(hsd_804D7998 as u32);
    card_oracle_h(hsd_804D799C as u32);
    card_oracle_h(active_requests.r#type as u32);
    card_oracle_h(card_oracle_ptr(active_requests.state as *const c_void));
    card_oracle_h(active_requests.callback.is_some() as i32 as u32);
    card_oracle_h(active_requests.callback_arg as u32);
    card_oracle_card_bytes(
        &raw mut card_oracle_sector as *mut u8,
        ::core::mem::size_of::<[u8; 32768]>() as i32,
    );
    card_oracle_bytes(
        &raw mut card_oracle_data as *mut u8,
        ::core::mem::size_of::<[u8; 65536]>() as i32,
    );
    card_oracle_bytes(
        &raw mut card_oracle_banner as *mut u8,
        ::core::mem::size_of::<[u8; 8192]>() as i32,
    );
    card_oracle_bytes(
        &raw mut card_oracle_icons as *mut u8,
        ::core::mem::size_of::<[u8; 32768]>() as i32,
    );
    card_oracle_bytes(
        &raw mut card_oracle_comment as *mut u8,
        ::core::mem::size_of::<[u8; 64]>() as i32,
    );
    card_oracle_card_bytes(
        &raw mut card_oracle_disk as *mut u8,
        ::core::mem::size_of::<[u8; 262144]>() as i32,
    );
    card_oracle_h(card_oracle_out1 as u32);
    card_oracle_h(card_oracle_out2 as u32);
    card_oracle_h(card_oracle_calls);
    card_oracle_h(card_oracle_trace);
    card_oracle_h(card_oracle_pending.is_some() as i32 as u32);
}

unsafe extern "C" fn card_oracle_setup(mut k: i32) {
    let mut s: *mut CardState = &raw mut card_oracle_state;
    let mut i: i32 = 0;
    let mut j: i32 = 0;
    let mut n: i32 = 0;
    let mut logical: i32 = 0 as i32;
    memset(
        s as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<CardState>() as usize,
    );
    memset(
        &raw mut commands as *mut CardCmd as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[CardCmd; 128]>() as usize,
    );
    memset(
        &raw mut requests as *mut CardRequest as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[CardRequest; 32]>() as usize,
    );
    memset(
        &raw mut active_requests as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<CardActiveRequest>() as usize,
    );
    memset(
        &raw mut card_oracle_cmd as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<CardCmd>() as usize,
    );
    memset(
        &raw mut card_oracle_cmd_buf as *mut CardCmdBuf as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[CardCmdBuf; 2]>() as usize,
    );
    memset(
        &raw mut card_oracle_stat as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<CARDStat>() as usize,
    );
    memset(
        &raw mut card_oracle_icon as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<CardIconInfo>() as usize,
    );
    memset(
        &raw mut card_oracle_sector as *mut u8 as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[u8; 32768]>() as usize,
    );
    memset(
        &raw mut card_oracle_disk as *mut u8 as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[u8; 262144]>() as usize,
    );
    i = 0 as i32;
    while (i as usize) < ::core::mem::size_of::<[u8; 65536]>() as usize {
        card_oracle_data[i as usize] = (i * 13 as i32
            + k * 7 as i32 & 255 as i32) as u8;
        i += 1;
    }
    i = 0 as i32;
    while (i as usize) < ::core::mem::size_of::<[u8; 8192]>() as usize {
        card_oracle_banner[i as usize] = (i * 3 as i32 + k
            & 255 as i32) as u8;
        i += 1;
    }
    i = 0 as i32;
    while (i as usize) < ::core::mem::size_of::<[u8; 32768]>() as usize {
        card_oracle_icons[i as usize] = (i * 17 as i32
            + k * 9 as i32 & 255 as i32) as u8;
        i += 1;
    }
    i = 0 as i32;
    while i < 64 as i32 {
        card_oracle_comment[i as usize] = (i * 3 as i32 + k
            & 255 as i32) as u8;
        card_oracle_primary[i as usize] = i as s32;
        i += 1;
    }
    (*s).sector_buf = &raw mut card_oracle_sector as *mut u8;
    (*s).sector_size = 8192 as u32;
    (*s).chan = (k & 1 as i32) as s32;
    (*s).file_no = 7 as i32 as s32;
    (*s).file_info.chan = (*s).chan;
    (*s).file_info.fileNo = 7 as i32 as s32;
    (*s).file_info.length = (8192 as i32 * 20 as i32)
        as s32;
    (*s).icon_info.banner_format = (k % 3 as i32) as u8;
    i = 0 as i32;
    while i < k % 9 as i32 {
        (*s).icon_info.icon_speed[i as usize] = (1 as i32
            + (i + k) % 3 as i32) as u8;
        (*s).icon_info.icon_format[i as usize] = (1 as i32
            + i % 2 as i32) as u8;
        i += 1;
    }
    (*s).header_size = hsd_803AC340(&raw mut (*s).icon_info) as u32;
    memcpy(
        &raw mut (*s).comment as *mut i8 as *mut c_void,
        &raw mut card_oracle_comment as *mut u8 as *const c_void,
        64 as usize,
    );
    card_oracle_icon = (*s).icon_info;
    card_oracle_stat.bannerFormat = (*s).icon_info.banner_format;
    i = 0 as i32;
    while i < 8 as i32 {
        card_oracle_stat.iconFormat = (card_oracle_stat.iconFormat as i32
            | ((*s).icon_info.icon_format[i as usize] as i32)
                << i * 2 as i32) as u16;
        card_oracle_stat.iconSpeed = (card_oracle_stat.iconSpeed as i32
            | ((*s).icon_info.icon_speed[i as usize] as i32)
                << i * 2 as i32) as u16;
        i += 1;
    }
    card_oracle_stat.length = (8192 as i32 * 20 as i32)
        as u32;
    (*s).stat = card_oracle_stat;
    i = 0 as i32;
    while i < 3 as i32 {
        (*s).file_sizes[i as usize] = 128 as i32
            + (k + i * 7 as i32) % 3 as i32
                * 8192 as i32;
        (*s).file_flags[i as usize] = (k + i) % 4 as i32;
        (*s).file_data[i as usize] = (&raw mut card_oracle_data as *mut u8)
            .offset((i * 20000 as i32) as isize);
        i += 1;
    }
    (*s).num_blocks = fn_803AC7DC(s);
    i = 0 as i32;
    while i < 64 as i32 {
        (*s).block_ids[i as usize] = -(0x7fff as i32) as s32;
        (*s).block_seqs[i as usize] = 0 as i32 as s32;
        i += 1;
    }
    (*s).block_ids[0 as i32 as usize] = 0 as i32 as s32;
    (*s).block_seqs[0 as i32 as usize] = (k & 255 as i32)
        as s32;
    n = 1 as i32;
    i = 0 as i32;
    while i < 3 as i32 {
        let mut count: i32 = fn_803AC634(s, i as s32)
            as i32;
        let mut start: i32 = if i == 0 as i32 {
            1 as i32
        } else {
            0 as i32
        };
        j = start;
        while j < count {
            (*s).block_ids[n as usize] = (logical + j) as s32;
            let fresh7 = n;
            n = n + 1;
            (*s).block_seqs[fresh7 as usize] = (k & 255 as i32) as s32;
            j += 1;
        }
        logical += count;
        i += 1;
    }
    i = 0 as i32;
    while i <= (*s).num_blocks && i < 25 as i32 {
        let mut offset: i32 = fn_803ACBE8(s, i as s32)
            as i32;
        let mut hdr: i32 = (if i == 0 as i32 {
            (*s).header_size.wrapping_add(48 as u32).wrapping_rem(8192 as u32)
        } else {
            0 as u32
        }) as i32;
        let mut b: *mut u8 = (&raw mut card_oracle_disk as *mut u8)
            .offset(offset as isize)
            .offset(hdr as isize);
        *b.offset(16 as i32 as isize) = ((*s).block_ids[i as usize]
            >> 8 as i32) as u8;
        *b.offset(17 as i32 as isize) = (*s).block_ids[i as usize]
            as u8;
        *b.offset(18 as i32 as isize) = (*s).block_seqs[i as usize]
            as u8;
        fn_803AC3F8(s, b.offset(19 as i32 as isize), i as s32 % 3 as s32);
        j = 32 as i32;
        while j < 8192 as i32 - hdr {
            *b.offset(j as isize) = card_oracle_data[((i * 8176 as i32 + j
                - 32 as i32) % 65536 as i32) as usize];
            j += 1;
        }
        HSD_Encrypt(b, 8192 as i32 - hdr);
        i += 1;
    }
    memcpy(
        &raw mut card_oracle_sector as *mut u8 as *mut c_void,
        &raw mut card_oracle_disk as *mut u8 as *const c_void,
        8192 as usize,
    );
    ::core::ptr::write_volatile(
        &mut curr_tail as *mut s32,
        (k % 128 as i32) as s32,
    );
    ::core::ptr::write_volatile(
        &mut curr_head as *mut s32,
        ::core::ptr::read_volatile::<s32>(&curr_tail as *const s32),
    );
    ::core::ptr::write_volatile(
        &mut curr_result as *mut s32,
        0 as i32 as s32,
    );
    hsd_804D798C = 0 as i32 as s32;
    hsd_804D7994 = (k % 32 as i32) as s32;
    hsd_804D7990 = hsd_804D7994;
    hsd_804D7998 = -(1 as i32);
    hsd_804D799C = idle;
    card_oracle_pending = None;
    card_oracle_pending_chan = 0 as i32;
    card_oracle_hash = 0 as u32;
    card_oracle_trace = card_oracle_hash;
    card_oracle_calls = card_oracle_trace;
    card_oracle_busy = k % 13 as i32;
    card_oracle_result = if k % 5 as i32 == 0 as i32 {
        -(3 as i32)
    } else if k % 7 as i32 == 0 as i32 {
        -(5 as i32)
    } else {
        0 as i32
    };
    card_oracle_out2 = 0 as i32;
    card_oracle_out1 = card_oracle_out2;
    card_oracle_k = k;
    card_oracle_cmd.state = s;
    card_oracle_cmd.r#type = CARD_CMD_CLEAR_BUF;
    card_oracle_cmd.anon_8.clear.data = &raw mut card_oracle_data as *mut u8
        as *mut c_void;
    card_oracle_cmd.anon_8.clear.size = (16 as i32
        + k % 128 as i32) as s32;
    memset(
        &raw mut card_oracle_hit as *mut u32 as *mut c_void,
        0 as i32,
        ::core::mem::size_of::<[u32; 110]>() as usize,
    );
}

unsafe extern "C" fn card_oracle_run(
    mut op: i32,
    mut k: i32,
) -> u32 {
    let mut s: *mut CardState = &raw mut card_oracle_state;
    let mut c: *mut CardCmd = &raw mut card_oracle_cmd;
    let mut r: i32 = 0 as i32;
    let mut i: i32 = 0;
    let mut b: i32 = k % 3 as i32;
    let mut f: i32 = k % 3 as i32;
    let mut len: i32 = if k % 4 as i32
        == 0 as i32
    {
        0 as i32
    } else {
        16 as i32 + k % 128 as i32
    };
    card_oracle_setup(k);
    card_oracle_op = op;
    if k >= 48 as i32 {
        hsd_804D799C = running;
        commands[curr_head as usize] = *c;
        requests[hsd_804D7990 as usize].r#type = CARD_REQ_READ_FILE;
        requests[hsd_804D7990 as usize].state = s;
        requests[hsd_804D7990 as usize].anon_8.file.file_idx = f as s32;
        requests[hsd_804D7990 as usize].anon_8.file.buf = &raw mut card_oracle_data
            as *mut u8;
    }
    match op {
        0 => {
            r = checkOpen(s);
        }
        1 => {
            (*c).r#type = (k % 18 as i32) as CardCmdType;
            memset(
                &raw mut (*c).anon_8.write as *mut c_void,
                0 as i32,
                ::core::mem::size_of::<CardWriteArgs>() as usize,
            );
            if (*c).r#type as i32
                == CARD_CMD_CREATE_FILE as i32
            {
                (*c).anon_8.create.filename = &raw const card_oracle_filename
                    as *const i8;
                (*c).anon_8.create.size = 8192 as i32 as s32;
            } else if (*c).r#type as i32
                == CARD_CMD_WRITE_HEADER as i32
                || (*c).r#type as i32
                    == CARD_CMD_VERIFY_HEADER as i32
            {
                (*c).anon_8.header.index = 0 as i32 as s32;
                (*c).anon_8.header.banner = &raw mut card_oracle_banner
                    as *mut u8 as *mut c_void;
                (*c).anon_8.header.icons = &raw mut card_oracle_icons
                    as *mut u8 as *mut c_void;
            } else if (*c).r#type as i32
                == CARD_CMD_READ_HEADER as i32
            {
                (*c).anon_8.read_header.index = 0 as i32 as s32;
                (*c).anon_8.read_header.comment = &raw mut card_oracle_comment
                    as *mut u8 as *mut c_void;
                (*c).anon_8.read_header.banner = &raw mut card_oracle_banner
                    as *mut u8 as *mut c_void;
                (*c).anon_8.read_header.icons = &raw mut card_oracle_icons
                    as *mut u8 as *mut c_void;
            } else {
                (*c).anon_8.write.file_idx = f as s32;
                (*c).anon_8.write.phys = b as s32;
                (*c).anon_8.write.block_id = b as s32;
                (*c).anon_8.write.seq = (k & 255 as i32) as s32;
                (*c).anon_8.write.data = &raw mut card_oracle_data as *mut u8
                    as *mut c_void;
                (*c).anon_8.write.size = len as s32;
                (*c).anon_8.write.offset = fn_803ACBE8(s, b as s32);
            }
            commands[curr_head as usize] = *c;
            ::core::ptr::write_volatile(
                &mut curr_tail as *mut s32,
                ((curr_head as i32 + 1 as i32)
                    % 128 as i32) as s32,
            );
            active_requests.state = s;
            active_requests.r#type = (k % 8 as i32) as CardActiveType;
            active_requests.callback = Some(
                card_oracle_callback
                    as unsafe extern "C" fn(i32, i32) -> (),
            ) as CardCallback;
            hsd_804D799C = running_async;
            hsd_803A949C(
                k as s32 & 1 as s32,
                if k % 3 as i32 == 0 as i32 {
                    -(3 as s32)
                } else {
                    0 as s32
                },
            );
        }
        2 => {
            let mut req: *mut CardRequest = (&raw mut requests as *mut CardRequest)
                .offset(hsd_804D7990 as isize) as *mut CardRequest;
            (*req).r#type = (k % 7 as i32) as CardRequestType;
            (*req).state = s;
            (*req).callback = Some(
                card_oracle_callback
                    as unsafe extern "C" fn(i32, i32) -> (),
            ) as CardCallback;
            if (*req).r#type as i32
                == CARD_REQ_CREATE_FILE as i32
            {
                (*req).anon_8.create.filename = &raw const card_oracle_filename
                    as *const i8;
                (*req).anon_8.create.banner = &raw mut card_oracle_banner
                    as *mut u8 as *mut c_void;
                (*req).anon_8.create.icons = &raw mut card_oracle_icons
                    as *mut u8 as *mut c_void;
            } else if (*req).r#type as i32
                == CARD_REQ_SET_STATUS as i32
            {
                (*req).anon_8.status.banner = &raw mut card_oracle_banner
                    as *mut u8 as *mut c_void;
                (*req).anon_8.status.icons = &raw mut card_oracle_icons
                    as *mut u8 as *mut c_void;
            } else if (*req).r#type as i32
                == CARD_REQ_READ_HEADER as i32
            {
                (*req).anon_8.header.comment = &raw mut card_oracle_comment
                    as *mut u8 as *mut c_void;
                (*req).anon_8.header.banner = &raw mut card_oracle_banner
                    as *mut u8 as *mut c_void;
                (*req).anon_8.header.icons = &raw mut card_oracle_icons
                    as *mut u8 as *mut c_void;
            } else {
                (*req).anon_8.file.file_idx = f as s32;
                (*req).anon_8.file.buf = &raw mut card_oracle_data as *mut u8;
            }
            hsd_804D7994 = ((hsd_804D7990 as i32
                + 1 as i32) % 32 as i32) as s32;
            r = fn_803AA790();
        }
        3 => {
            r = retryCardOpen(
                k as s32 & 1 as s32,
                &raw const card_oracle_filename as *const i8,
                &raw mut (*s).file_info,
            ) as i32;
        }
        4 => {
            r = retryCardFastOpen(k as s32 & 1 as s32, 7 as s32, &raw mut (*s).file_info)
                as i32;
        }
        5 => {
            r = retryCardReadAsync(
                &raw mut (*s).file_info,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                k as s32 % 128 as s32,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            ) as i32;
        }
        6 => {
            r = retryCardWriteAsync(
                &raw mut (*s).file_info,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                k as s32 % 128 as s32,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            ) as i32;
        }
        7 => {
            r = retryCardClose(&raw mut (*s).file_info) as i32;
        }
        8 => {
            r = retryCardCreateAsync(
                k as s32 & 1 as s32,
                &raw const card_oracle_filename as *const i8,
                len as s32,
                &raw mut (*s).file_info,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            ) as i32;
        }
        9 => {
            r = retryCardGetStatus(
                k as s32 & 1 as s32,
                7 as s32,
                &raw mut card_oracle_stat,
            ) as i32;
        }
        10 => {
            r = retryCardSetStatusAsync(
                k as s32 & 1 as s32,
                7 as s32,
                &raw mut card_oracle_stat,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            ) as i32;
        }
        11 => {
            r = setupCardIcons(c);
        }
        12 => {
            unpackCardStat(c, &raw mut card_oracle_stat);
        }
        13 => {
            hsd_804D7998 = curr_tail as i32;
            i = 0 as i32;
            while i < 5 as i32 {
                commands[curr_tail as usize].r#type = CARD_CMD_CLEAR_BUF;
                ::core::ptr::write_volatile(
                    &mut curr_tail as *mut s32,
                    ((curr_tail as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
                i += 1;
            }
            rollbackCardCommands(curr_tail);
        }
        14 => {
            initHeaderBlockCommand(
                c,
                s,
                b as u32,
                &raw mut card_oracle_comment as *mut u8 as *mut c_void,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
            );
        }
        15 => {
            r = queueHeaderBlock(
                s,
                b as u32,
                &raw mut card_oracle_comment as *mut u8 as *mut c_void,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
            ) as i32;
        }
        16 => {
            r = queueHeaderBlocks(
                s,
                &raw mut card_oracle_comment as *mut u8 as *mut c_void,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
            ) as i32;
        }
        17 => {
            (*c).r#type = (k % 18 as i32) as CardCmdType;
            memset(
                &raw mut (*c).anon_8.write as *mut c_void,
                0 as i32,
                ::core::mem::size_of::<CardWriteArgs>() as usize,
            );
            if (*c).r#type as i32
                == CARD_CMD_CREATE_FILE as i32
            {
                (*c).anon_8.create.filename = &raw const card_oracle_filename
                    as *const i8;
                (*c).anon_8.create.size = 8192 as i32 as s32;
            } else if (*c).r#type as i32
                == CARD_CMD_WRITE_HEADER as i32
                || (*c).r#type as i32
                    == CARD_CMD_VERIFY_HEADER as i32
            {
                (*c).anon_8.header.index = 0 as i32 as s32;
                (*c).anon_8.header.banner = &raw mut card_oracle_banner
                    as *mut u8 as *mut c_void;
                (*c).anon_8.header.icons = &raw mut card_oracle_icons
                    as *mut u8 as *mut c_void;
            } else if (*c).r#type as i32
                == CARD_CMD_READ_HEADER as i32
            {
                (*c).anon_8.read_header.index = 0 as i32 as s32;
                (*c).anon_8.read_header.comment = &raw mut card_oracle_comment
                    as *mut u8 as *mut c_void;
                (*c).anon_8.read_header.banner = &raw mut card_oracle_banner
                    as *mut u8 as *mut c_void;
                (*c).anon_8.read_header.icons = &raw mut card_oracle_icons
                    as *mut u8 as *mut c_void;
            } else {
                (*c).anon_8.write.file_idx = f as s32;
                (*c).anon_8.write.phys = b as s32;
                (*c).anon_8.write.block_id = b as s32;
                (*c).anon_8.write.seq = (k & 255 as i32) as s32;
                (*c).anon_8.write.data = &raw mut card_oracle_data as *mut u8
                    as *mut c_void;
                (*c).anon_8.write.size = len as s32;
                (*c).anon_8.write.offset = fn_803ACBE8(s, b as s32);
            }
            commands[curr_head as usize] = *c;
            ::core::ptr::write_volatile(
                &mut curr_tail as *mut s32,
                ((curr_head as i32 + 1 as i32)
                    % 128 as i32) as s32,
            );
            active_requests.state = s;
            active_requests.r#type = (k % 8 as i32) as CardActiveType;
            active_requests.callback = Some(
                card_oracle_callback
                    as unsafe extern "C" fn(i32, i32) -> (),
            ) as CardCallback;
            hsd_804D799C = running;
            hsd_803AAA48();
        }
        18 => {
            r = fn_803AC168(c);
        }
        19 => {
            r = fn_803AC258(s, b as s32) as i32;
        }
        20 => {
            r = fn_803AC2A4(s) as i32;
        }
        21 => {
            fn_803AC2D4();
        }
        22 => {
            hsd_804D7998 = curr_tail as i32;
            i = 0 as i32;
            while i < 5 as i32 {
                commands[curr_tail as usize].r#type = CARD_CMD_CLEAR_BUF;
                ::core::ptr::write_volatile(
                    &mut curr_tail as *mut s32,
                    ((curr_tail as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
                i += 1;
            }
            fn_803AC2E0();
        }
        23 => {
            fn_803AC334();
        }
        24 => {
            r = hsd_803AC340(&raw mut card_oracle_icon);
        }
        25 => {
            hsd_803AC3E0(
                s,
                f,
                8192 as i32 + len,
                k % 4 as i32,
                &raw mut card_oracle_data as *mut u8,
            );
        }
        26 => {
            fn_803AC3F8(s, &raw mut card_oracle_data as *mut u8, f as s32);
        }
        27 => {
            fn_803AC3F8(s, &raw mut card_oracle_data as *mut u8, f as s32);
            memset(
                &raw mut (*s).file_sizes as *mut i32
                    as *mut c_void,
                0 as i32,
                ::core::mem::size_of::<[i32; 9]>() as usize,
            );
            hsd_803AC558(s, &raw mut card_oracle_data as *mut u8);
        }
        28 => {
            r = fn_803AC634(s, f as s32) as i32;
        }
        29 => {
            r = fn_803AC6B8_first_block_count(s) as i32;
        }
        30 => {
            r = fn_803AC6B8(s, f as s32) as i32;
        }
        31 => {
            r = fn_803AC6B8_blocks_before(s, f as s32) as i32;
        }
        32 => {
            r = fn_803AC7DC_block_count(s, f as s32) as i32;
        }
        33 => {
            r = fn_803AC7DC(s) as i32;
        }
        34 => {
            r = fn_803ACB74(
                k as s32 * 17 as s32 % 258 as s32 - 1 as s32,
                k as s32 * 31 as s32 % 258 as s32 - 1 as s32,
            ) as i32;
        }
        35 => {
            r = fn_803ACBE8(s, b as s32) as i32;
        }
        36 => {
            r = fn_803ACC0C(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
            ) as i32;
        }
        37 => {
            r = fn_803ACD58(
                s,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
            ) as i32;
        }
        38 => {
            r = fn_803ACF30(
                s,
                &raw mut card_oracle_comment as *mut u8 as *mut c_void,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
            ) as i32;
        }
        39 => {
            r = card_oracle_ptr(
                fn_803ACFC0_header(s, k as s32 % 64 as s32) as *const c_void,
            ) as i32;
        }
        40 => {
            r = card_oracle_ptr(
                fn_803ACFC0_checksum_start(k as s32 % 64 as s32, s)
                    as *const c_void,
            ) as i32;
        }
        41 => {
            r = fn_803ACFC0(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                f as s32,
            ) as i32;
        }
        42 => {
            r = fn_803AD16C_total_blocks(s) as i32;
        }
        43 => {
            r = fn_803AD16C_logical_index(k as s32 % 9 as s32, k as s32 % 7 as s32)
                as i32;
        }
        44 => {
            r = fn_803AD16C_file_size(s, f as s32) as i32;
        }
        45 => {
            r = fn_803AD16C_nonnegative(k as s32 % 9 as s32 - 4 as s32);
        }
        46 => {
            fn_803AD16C_own(
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
            );
        }
        47 => {
            r = fn_803AD16C_queue_clear(s, b as s32, k as s32 % 128 as s32, f as s32)
                as i32;
        }
        48 => {
            r = fn_803AD16C_queue_read(s, b as s32) as i32;
        }
        49 => {
            r = fn_803AD16C_queue_write(s, b as s32, b as s32, k as s32 & 255 as s32)
                as i32;
        }
        50 => {
            r = fn_803AD16C_queue_write_last(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
            ) as i32;
        }
        51 => {
            r = fn_803AD16C(s) as i32;
        }
        52 => {
            r = fn_803ADE4C(
                s,
                7 as i32,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        53 => {
            r = queueCardReadCommand(
                s,
                b as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                k as s32 % 128 as s32,
            ) as i32;
        }
        54 => {
            r = calculateDataBlockSize(s, f as s32, b as s32) as i32;
        }
        55 => {
            r = calculateFileBlockCount(s, f as s32) as i32;
        }
        56 => {
            r = retryCardRead(
                &raw mut (*s).file_info,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                k as s32 % 128 as s32,
            ) as i32;
        }
        57 => {
            hsd_804D7998 = curr_tail as i32;
            i = 0 as i32;
            while i < 5 as i32 {
                commands[curr_tail as usize].r#type = CARD_CMD_CLEAR_BUF;
                ::core::ptr::write_volatile(
                    &mut curr_tail as *mut s32,
                    ((curr_tail as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
                i += 1;
            }
            cancelQueuedCardCommands();
        }
        58 => {
            r = queueCardClearCommand(
                s,
                &raw mut card_oracle_data as *mut u8,
                len as s32,
            ) as i32;
        }
        59 => {
            r = queueClearDataBlock(
                s,
                &raw mut card_oracle_data as *mut u8,
                len as s32,
            ) as i32;
        }
        60 => {
            r = cardDataBlockOffset(s, (*s).sector_size, b as s32) as i32;
        }
        61 => {
            r = queueReadDataBlock(
                s,
                b as s32,
                &raw mut card_oracle_data as *mut u8,
                len as s32,
            );
        }
        62 => {
            r = readCardDataBlockFirst(
                s,
                (*s).sector_size,
                b as s32,
                &raw mut card_oracle_data as *mut u8,
                len as s32,
            ) as i32;
        }
        63 => {
            r = readCardDataBlockFinal(
                s,
                (*s).sector_size,
                b as s32,
                &raw mut card_oracle_data as *mut u8,
                len as s32,
            ) as i32;
        }
        64 => {
            r = fn_803ADF90(
                s,
                f as s32,
                &raw mut card_oracle_data as *mut u8,
                k & 1 as i32,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        65 => {
            hsd_804D7998 = curr_tail as i32;
            i = 0 as i32;
            while i < 5 as i32 {
                commands[curr_tail as usize].r#type = CARD_CMD_CLEAR_BUF;
                ::core::ptr::write_volatile(
                    &mut curr_tail as *mut s32,
                    ((curr_tail as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
                i += 1;
            }
            fn_803AE7F8_rewind();
        }
        66 => {
            fn_803AE7F8_close(s);
        }
        67 => {
            r = fn_803AE7F8(
                s,
                f as s32,
                &raw mut card_oracle_data as *mut u8,
                k as i32 & 1 as i32,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        68 => {
            r = fn_803AF3F0_chunk_size(s) as i32;
        }
        69 => {
            r = fn_803AF3F0_queue_verify_first(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
            ) as i32;
        }
        70 => {
            r = fn_803AF3F0_queue_verify_final(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
            ) as i32;
        }
        71 => {
            r = fn_803AF3F0_queue_write_first(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                f as s32,
            ) as i32;
        }
        72 => {
            r = fn_803AF3F0_queue_write_final(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                f as s32,
            ) as i32;
        }
        73 => {
            hsd_804D7998 = curr_tail as i32;
            i = 0 as i32;
            while i < 5 as i32 {
                commands[curr_tail as usize].r#type = CARD_CMD_CLEAR_BUF;
                ::core::ptr::write_volatile(
                    &mut curr_tail as *mut s32,
                    ((curr_tail as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
                i += 1;
            }
            fn_803AF3F0_rewind();
        }
        74 => {
            fn_803AF3F0_close(s);
        }
        75 => {
            r = fn_803AF3F0_open(k as s32 & 1 as s32, 7 as s32, &raw mut (*s).file_info)
                as i32;
        }
        76 => {
            fn_803AF3F0_check_seq(
                s,
                &raw mut card_oracle_primary as *mut s32,
                3 as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_out1,
            );
        }
        77 => {
            fn_803AF3F0_calc_file_blocks(
                f as s32,
                s,
                &raw mut card_oracle_out1,
                &raw mut card_oracle_out2,
            );
        }
        78 => {
            r = fn_803AF3F0(
                s,
                f as s32,
                &raw mut card_oracle_data as *mut u8,
                k as i32 & 1 as i32,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        79 => {
            r = fn_803B0120_first_chunk(s) as i32;
        }
        80 => {
            hsd_804D7998 = curr_tail as i32;
            i = 0 as i32;
            while i < 5 as i32 {
                commands[curr_tail as usize].r#type = CARD_CMD_CLEAR_BUF;
                ::core::ptr::write_volatile(
                    &mut curr_tail as *mut s32,
                    ((curr_tail as i32 + 1 as i32)
                        % 128 as i32) as s32,
                );
                i += 1;
            }
            fn_803B0120_rewind();
        }
        81 => {
            fn_803B0120_close(s);
        }
        82 => {
            r = fn_803B0120_close_result(s) as i32;
        }
        83 => {
            r = fn_803B0120_block_offset(s, b as s32) as i32;
        }
        84 => {
            r = fn_803B0120_queue_verify(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
            ) as i32;
        }
        85 => {
            r = fn_803B0120_queue_write(
                s,
                b as s32,
                b as s32,
                k as s32 & 255 as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                f as s32,
            ) as i32;
        }
        86 => {
            r = fn_803B0120(
                s,
                f as s32,
                &raw mut card_oracle_data as *mut u8,
                k as i32 & 1 as i32,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        87 => {
            r = queueVerifyCardHeader(
                s,
                c,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
            ) as i32;
        }
        88 => {
            r = fn_803B0E9C_write_block(s, b as s32) as i32;
        }
        89 => {
            r = fn_803B0E9C_write_block_final(s, b as s32) as i32;
        }
        90 => {
            r = fn_803B0E9C_read_first(s, (*s).sector_size) as i32;
        }
        91 => {
            r = fn_803B0E9C(
                s,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8,
                k >> 1 as i32 & 1 as i32,
                k & 1 as i32,
            );
        }
        92 => {
            r = fn_803B1338_queue_write(
                s,
                b as s32,
                b as s32,
                &raw mut card_oracle_data as *mut u8 as *mut c_void,
                len as s32,
                f as s32,
                (&raw mut card_oracle_cmd_buf as *mut CardCmdBuf)
                    .offset(0 as i32 as isize) as *mut CardCmdBuf,
                (&raw mut card_oracle_cmd_buf as *mut CardCmdBuf)
                    .offset(1 as i32 as isize) as *mut CardCmdBuf,
            ) as i32;
        }
        93 => {
            r = card_oracle_ptr(
                fn_803B1338_data_at(
                    &raw mut card_oracle_data as *mut u8,
                    k as s32 % 128 as s32,
                ) as *const c_void,
            ) as i32;
        }
        94 => {
            r = fn_803B1338_data_size(s) as i32;
        }
        95 => {
            r = fn_803B1338_queue_write_data(
                s,
                b as s32,
                b as s32,
                &raw mut card_oracle_data as *mut u8,
                k as s32 % 128 as s32,
                f as s32,
                (&raw mut card_oracle_cmd_buf as *mut CardCmdBuf)
                    .offset(0 as i32 as isize) as *mut CardCmdBuf,
                (&raw mut card_oracle_cmd_buf as *mut CardCmdBuf)
                    .offset(1 as i32 as isize) as *mut CardCmdBuf,
            ) as i32;
        }
        96 => {
            r = fn_803B1338(s, k & 1 as i32);
        }
        97 => {
            r = fn_803B1F78(
                s,
                &raw const card_oracle_filename as *const i8,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        98 => {
            r = fn_803B21E8(
                s,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        99 => {
            hsd_803B2374();
        }
        100 => {
            hsd_803B24E4(
                s,
                k & 1 as i32,
                (*s).sector_size as i32,
                &raw mut card_oracle_sector as *mut u8 as *mut c_void,
            );
        }
        101 => {
            r = hsd_803B2550(
                s,
                &raw const card_oracle_filename as *const i8,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        102 => {
            r = hsd_803B2674(s);
        }
        103 => {
            r = fn_803B26CC(
                s,
                &raw mut card_oracle_comment as *mut u8 as *mut c_void,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        104 => {
            r = hsd_803B27F4(
                s,
                &raw mut card_oracle_comment as *mut u8 as *mut c_void,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        105 => {
            r = hsd_803B286C(
                s,
                &raw const card_oracle_filename as *const i8,
                &raw mut card_oracle_comment as *mut u8 as *const i8,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        106 => {
            r = hsd_803B2928(
                s,
                &raw mut card_oracle_comment as *mut u8 as *const i8,
                &raw mut card_oracle_banner as *mut u8 as *mut c_void,
                &raw mut card_oracle_icons as *mut u8 as *mut c_void,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        107 => {
            r = hsd_803B29D8(
                s,
                f,
                &raw mut card_oracle_data as *mut u8,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        108 => {
            r = hsd_803B2A4C(
                s,
                f,
                &raw mut card_oracle_data as *mut u8,
                Some(
                    card_oracle_callback
                        as unsafe extern "C" fn(
                            i32,
                            i32,
                        ) -> (),
                ),
            );
        }
        109 => {
            r = hsd_SetCardIconInfo(s, &raw mut card_oracle_icon);
        }
        _ => {}
    }
    card_oracle_h(r as u32);
    card_oracle_hash_all();
    return card_oracle_hash;
}
#[test]
fn all_card_definitions_match_native_c() { unsafe {
    m::ext::OSDisableInterrupts_hook = Some(card_oracle_OSDisableInterrupts);
    m::ext::OSRestoreInterrupts_hook = Some(card_oracle_OSRestoreInterrupts);
    m::ext::CARDCreateAsync_hook = Some(card_oracle_CARDCreateAsync);
    m::ext::CARDFastOpen_hook = Some(card_oracle_CARDFastOpen);
    m::ext::CARDOpen_hook = Some(card_oracle_CARDOpen);
    m::ext::CARDClose_hook = Some(card_oracle_CARDClose);
    m::ext::CARDGetXferredBytes_hook = Some(card_oracle_CARDGetXferredBytes);
    m::ext::CARDReadAsync_hook = Some(card_oracle_CARDReadAsync);
    m::ext::CARDRead_hook = Some(card_oracle_CARDRead);
    m::ext::CARDGetStatus_hook = Some(card_oracle_CARDGetStatus);
    m::ext::CARDSetStatusAsync_hook = Some(card_oracle_CARDSetStatusAsync);
    m::ext::CARDWriteAsync_hook = Some(card_oracle_CARDWriteAsync);
    m::ext::CARDWrite_hook = Some(card_oracle_CARDWrite);
    let mut count = [0u32; 110];
    for line in include_str!("data/tu_card.txt").lines() {
        let p: std::vec::Vec<_> = line.split_whitespace().collect();
        let op: i32 = p[0].parse().unwrap(); let k: i32 = p[1].parse().unwrap();
        let expected = u32::from_str_radix(p[2], 16).unwrap();
        let got = card_oracle_run(op,k);
        assert_eq!(got, expected, "card.c op={op}, case={k}");
        count[op as usize] += 1;
    }
    assert!(count.iter().all(|&n| n >= 96));
} }
