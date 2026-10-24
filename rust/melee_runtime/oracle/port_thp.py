"""Reproducible adaptation of c2rust reference to the audited native THP port.

The reference types are discarded. Native fields come exclusively from melee_types;
the six shipping fmuladd sites and native pointer/asset representations are explicit.
"""
from pathlib import Path
import re
ROOT = Path(__file__).resolve().parents[3]
raw = (ROOT/'rust/melee_runtime/src/raw/libs/dolphin/src/dolphin/thp/THPDec.rs').read_text()
s = raw[raw.index('static mut __THP420Error'):]
s = s.replace('#[no_mangle]\n', '')
for a,b in {'::core::ffi::c_int':'i32','::core::ffi::c_uint':'u32','::core::ffi::c_long':'i32',
            '::core::ffi::c_char':'i8','::core::ffi::c_float':'f32','::core::ffi::c_double':'f64',
            'u8_0':'u8','u16_0':'u16','u32_0':'u32','f32_0':'f32','f64_0':'f64',
            'gc_size_t':'usize','uintptr_t':'usize'}.items():
    s = s.replace(a,b)
s = s.replace('i32long', 'i64').replace('FALSE', '0').replace('TRUE', '1')
s = re.sub(r'(?m)^unsafe extern "C" fn ', 'pub unsafe extern "C" fn ', s)
s = re.sub(r'\b(\d+|0x[0-9a-f]+) as i32 as (usize|isize)', r'\1 as \2', s)
s = re.sub(r'\b(\d+|0x[0-9a-f]+) as i32\b', r'\1i32', s)
# Native pointers must not pass through the old 32-bit int parameter.
s = s.replace('mut huffmanBits: i32', 'mut huffmanBits: usize')
s = s.replace('huffmanBits as i32', 'huffmanBits as usize')
for ch,i in [('Y',0),('U',1),('V',2)]:
    s = s.replace(f'&raw mut (*(info as *mut THPFileInfoDCTComp{ch}View)).predDC',
                  f'&raw mut (*info).components[{i}].predDC')
s = s.replace('0x920 as u32', 'native_work_prefix() as u32').replace('0x920 as isize','native_work_prefix() as isize')
s = re.sub(r'p = \(\(\(\*info\)\.scratch as u32\).*?as \*mut u8;',
           'p = (((*info).scratch as usize).wrapping_add(31) & !31usize) as *mut u8;',s,flags=re.S)
# Stream words are file bytes, not native scalar state.
s = s.replace('(*info).currByte = *p;', '(*info).currByte = core::ptr::read_unaligned(p).to_be();')
s = s.replace('return *p;', 'return core::ptr::read_unaligned(p).to_be();')
s = s.replace('(*info).currByte = *ptr;', '(*info).currByte = core::ptr::read_unaligned(ptr).to_be();')
s = s.replace('*((*info).file as *const u32).offset(1 as isize)',
              'core::ptr::read_unaligned(((*info).file as *const u32).offset(1 as isize)).to_be()')
# The AAN IDCT has exactly six contracted expressions in shipping LLVM IR.
s = s.replace('(t1 - t3) * 1.414213562f32 - t13', 'fmadds(t1 - t3, 1.414213562f32, -t13)')
s = s.replace('1.082392200f32 * z12 - z5', 'fmadds(1.082392200f32, z12, -z5)')
s = s.replace('-2.613125930f32 * z10 + z5', 'fmadds(-2.613125930f32, z10, z5)')
s = re.sub(r'(\(\*cp.offset\(16 as isize\)\s*- \*cp.offset\(48 as isize\)\))\s*\* 1.414213562f32\s*- t13_0',
           r'fmadds(\1, 1.414213562f32, -t13_0)', s)
s = s.replace('1.082392200f32 * z12_0 - z5_0', 'fmadds(1.082392200f32, z12_0, -z5_0)')
s = s.replace('-2.613125930f32 * z10_0 + z5_0', 'fmadds(-2.613125930f32, z10_0, z5_0)')
assert s.count('fmadds(') == 6
# Code tables immediately follow byte-sized length tables and need unaligned access.
s = re.sub(r'\*(\(\*info\)\.huffmanTabs\[tab_index as usize\]\s*\.codeTab\s*\.offset\(fresh13 as isize\)) = code;',
           r'core::ptr::write_unaligned(\1, code);', s)
s = s.replace('*(*h).codeTab.offset(p as isize)', 'core::ptr::read_unaligned((*h).codeTab.offset(p as isize))')
s = re.sub(r'\*\(\*h\)\s*\.codeTab\s*\.offset\(\(p as i32 - 1i32\) as isize\)',
           'core::ptr::read_unaligned((*h).codeTab.offset((p - 1) as isize))', s)
s += '\n/// The TARGET_PC body has no Gekko quantisation registers to initialise.\npub unsafe extern "C" fn OSInitFastCast() {}\n'
prefix = '''//! Native TARGET_PC THP decoder. All C definitions, including the bitstream and AAN helpers.
//! Asset words stay big endian; game state and generated records are native little endian.
//! Native-width scratch/predictor access replaces the original fixed GameCube offset views.
#![allow(non_snake_case, non_upper_case_globals, unused_mut, unused_assignments,
         unused_variables, unused_unsafe, unused_parens, static_mut_refs, dead_code)]
use melee_types::*;
use melee_core::fp::fmadds;
use core::ffi::c_void;
use ext::*;

const fn native_work_prefix() -> usize { (core::mem::size_of::<THPFileInfo>() + 31) & !31 }
unsafe fn memset(dst: *mut c_void, v: i32, n: usize) -> *mut c_void {
    core::ptr::write_bytes(dst.cast::<u8>(), v as u8, n); dst
}

/// Platform cache and diagnostic hooks, retaining the C symbol names.
pub mod ext {
    use core::ffi::c_void;
    pub static mut DCZeroRange_HOOK: Option<unsafe fn(*mut c_void, u32)> = None;
    pub static mut LCStoreData_HOOK: Option<unsafe fn(*mut c_void, *mut c_void, u32) -> u32> = None;
    pub static mut LCQueueWait_HOOK: Option<unsafe fn(u32)> = None;
    pub static mut OSReport_HOOK: Option<unsafe fn(*mut i8)> = None;
    pub unsafe fn DCZeroRange(p: *mut c_void, n: u32) {
        (DCZeroRange_HOOK.expect("TODO: DCZeroRange"))(p,n)
    }
    pub unsafe fn LCStoreData(d: *mut c_void, s: *mut c_void, n: u32) -> u32 {
        (LCStoreData_HOOK.expect("TODO: LCStoreData"))(d,s,n)
    }
    pub unsafe fn LCQueueWait(n: u32) { (LCQueueWait_HOOK.expect("TODO: LCQueueWait"))(n) }
    pub unsafe fn OSReport(s: *mut i8) { (OSReport_HOOK.expect("TODO: OSReport"))(s) }
}
'''
out = ROOT/'rust/melee_runtime/src/tu/THPDec.rs'
out.write_text(prefix+s+'\n#[cfg(test)]\n#[path = "../../tests/thp_state/mod.rs"]\nmod tests;\n')
print('THP native adaptation:', len(re.findall(r'pub unsafe extern "C" fn ',s)), 'definitions')
