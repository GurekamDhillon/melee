"""Adapt the card.c c2rust reference to native generated types (developer helper).

The resulting Rust is reviewed/checked in; this script never runs in Cargo.
Native-pointer repairs mirror the four documented oracle source repairs.
"""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[3]
raw = (ROOT / 'rust/melee_runtime/src/raw/sysdolphin/baselib/card.rs').read_text()
body = raw[raw.index('#[no_mangle]\npub static mut hsd_804D79C8'):]
body = body.replace('#[no_mangle]\n', '')
body = re.sub(r'    let mut _: \[[^;]+; \d+\] = \[0; \d+\];\n', '', body)
body = re.sub(r'(?m)^(unsafe extern "C" fn|static mut) ', r'pub \1 ', body)
for old, new in {
    '::core::ffi::c_int':'i32', '::core::ffi::c_uint':'u32',
    '::core::ffi::c_long':'i32', '::core::ffi::c_ulong':'usize',
    '::core::ffi::c_char':'i8', '::core::ffi::c_uchar':'u8',
    '::core::ffi::c_void':'c_void', 'u8_0':'u8', 'u16_0':'u16',
    'u32_0':'u32', 'bool_0':'i32', 'gc_size_t':'usize',
    'C2RustUnnamed_0':'CardCmd_anon_193',
    'C2RustUnnamed_1':'CardRequest_anon_258', 'C2RustUnnamed_2':'i32',
    'type_0':'r#type', 'c2rust_unnamed':'anon_8',
}.items():
    body = body.replace(old, new)
# These one-word local wrappers only force the original compiler's register allocation.
body = re.sub(r'let mut (free_blk|write_blk): C2RustUnnamed_[34] = C2RustUnnamed_[34] \{ v: 0 \};', r'let mut \1: i32 = 0;', body)
body = re.sub(r'\b(free_blk|write_blk)\.v\b', r'\1', body)
# Four guest-era pointer-through-int operations must retain the full native address.
body, n = re.subn(r'\((hdr_offset(?:13)?) as u32\)\s*\.wrapping_add\(\(\*state\).sector_buf as u32\)\s*as \*mut CardBlockHeader',
                 r'(*state).sector_buf.offset(\1 as isize).cast::<CardBlockHeader>()', body)
assert n == 2
body = body.replace('(hdr_offset + (*state).sector_buf as s32) as *mut u8', '(*state).sector_buf.offset(hdr_offset as isize)')
body = body.replace('let mut addr: s32 = (*state).sector_buf as s32;', '')
body = body.replace('(hdr_offset + off + addr) as *mut u8 as *mut c_void', '(*state).sector_buf.offset((hdr_offset + off) as isize).cast::<c_void>()')
# Rust wrapping_div still traps on zero. Card sizes are normally valid, but the shipping
# arithmetic semantics specify zero divisors too.
body = body.replace('.wrapping_div(', '.card_div(').replace('.wrapping_rem(', '.card_rem(')
body = body.replace('ret == -(0x101 as i32);', 'let _ = ret == -(0x101 as i32);')
# fn_803B1338 passes index 9 while writing spare blocks. Select the GC record
# aliases explicitly instead of reading native padding or a widened pointer.
lo = body.index('pub unsafe extern "C" fn fn_803AC3F8(')
hi = body.index('pub unsafe extern "C" fn hsd_803AC558(',lo)
chunk = body[lo:hi]
chunk = re.sub(r'\(\*state\)\s*\.file_sizes\[idx as usize\]', 'file_size', chunk)
chunk = re.sub(r'\(\*state\)\s*\.file_flags\[idx as usize\]', 'file_flags', chunk)
chunk = chunk.replace('let mut idx: s32 = start + i;', '''let mut idx: s32 = start + i;
        let file_size = if idx == 9 { (*state).file_data[0] as usize as u32 as i32 } else { (*state).file_sizes[idx as usize] };
        let file_flags = if idx == 9 { (*state).file_sizes[0] } else { (*state).file_flags[idx as usize] };''')
chunk = chunk.replace('pub unsafe extern "C" fn fn_803AC3F8(', '''// fn_803B1338's spare-block pass supplies file_idx == 9. The original
// three-row window then aliases file_flags[9] to file_sizes[0] and
// file_sizes[9] to file_data[0] at the GC offsets. Select those logical
// fields explicitly. This unused free-block word contains a process address;
// cross-process byte-identical save files need a canonical legacy address
// mapper here. The oracle compares this word as a pointer-valued observation.
pub unsafe extern "C" fn fn_803AC3F8(''')
body = body[:lo] + chunk + body[hi:]
head = '''//! Native port of `src/sysdolphin/baselib/card.c` (all definitions, including inlines).
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

'''
out = ROOT / 'rust/melee_runtime/src/tu/card.rs'
out.write_text(head + body, encoding='utf-8', newline='\n')
print(f'{out}: {len(re.findall(r"(?:pub )?unsafe extern .C. fn ", body))} C definitions')
