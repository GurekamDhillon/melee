"""Compile the real JPEG C definitions with native storage and gwtool FP semantics.

Only platform glue changes: native image pointer, contiguous GC globals, serialized
BE table/marker reads, and a host setjmp target separate from the GC layout buffer.
"""
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
OUT = ROOT / 'rust/target/oracle-shard-jpeg'
OUT.mkdir(parents=True, exist_ok=True)
CLANG = r'C:\Program Files\LLVM\bin\clang.exe'

def clean(s):
    return re.sub(r'^#include[^\n]*\n', '', s, flags=re.M)

enc = clean((ROOT/'src/sysdolphin/baselib/hsd_3B34.c').read_text())
dec = clean((ROOT/'src/sysdolphin/baselib/hsd_3B5C.c').read_text())
head = clean((ROOT/'src/sysdolphin/baselib/hsd_3B34.h').read_text()).replace('jmp_buf', 'gc_jmp_buf')
enc = enc.replace('hsd_803B51C8_inline(s32 image,', 'hsd_803B51C8_inline(u8* image,')
enc = enc.replace('hsd_803B51C8(s32 arg0,', 'hsd_803B51C8(u8* arg0,')
head = head.replace('hsd_803B51C8(s32,', 'hsd_803B51C8(u8*,')
# Those two source arrays are a single typed JpegEncodeTables in GC memory.
arrays = []
for name in ['lbl_80430C40', 'lbl_80430C80']:
    p = rf'static u8 {name}\[[^]]+\] = \{{(.*?)\}};'
    m = re.search(p, enc, re.S)
    arrays.extend(int(x, 0) for x in re.findall(r'0x[0-9A-Fa-f]+|\d+', m[1]))
    enc = re.sub(p, '', enc, flags=re.S)
enc = ('static u8 encode_storage[0x450] = {' + ','.join(map(str, arrays)) + '};\n'
       '#define lbl_80430C40 (encode_storage)\n#define lbl_80430C80 (encode_storage+0x40)\n' + enc)
dec = dec.replace('jmp_buf hsd_804D2E70;\nu8 hsd_804D2F68[0x70C];',
    'static struct { gc_jmp_buf buf; u8 data[0x710]; } decoder_storage;\n'
    '#define hsd_804D2E70 decoder_storage.buf')
dec = dec.replace('jmp_buf jmp;', 'gc_jmp_buf jmp;')
dec = dec.replace('*(u16*) hsd_804D79B8', 'read_be16(hsd_804D79B8)')
code = head + enc + dec
code = re.sub(r'\bsetjmp\([^\n]+?\)', 'setjmp(oracle_jmp)', code)
code = re.sub(r'\blongjmp\([^;]+?\);', 'longjmp(oracle_jmp, 1);', code)
preamble = '''#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
typedef uint8_t u8; typedef uint16_t u16; typedef uint32_t u32;
typedef int32_t s32; typedef float f32; typedef double f64;
typedef uint64_t gc_jmp_buf[35];
#define UNUSED
#define PAD_STACK(x)
static jmp_buf oracle_jmp;
static u16 read_be16(const u8 *p) { return (u16)((p[0]<<8)|p[1]); }
void hsd_803B3344(u8); void hsd_803B3398(void*, size_t);
'''
driver = (HERE/'driver_shard_jpeg.c').read_text()
src = OUT/'jpeg.c'
src.write_text(preamble + code + driver)
ll = OUT/'jpeg.ll'
subprocess.run([CLANG, '-std=c11', '-fno-strict-aliasing', '-fwrapv',
    '-ftrivial-auto-var-init=zero', '-ffp-contract=on', '-O2', '-Xclang', '-disable-llvm-passes',
    '-w', '-S', '-emit-llvm', str(src), '-o', str(ll)], check=True)
subprocess.run([sys.executable, str(HERE/'lower.py'), str(ll), str(OUT/'jpeg.lo.ll')], check=True)
subprocess.run([CLANG, '-O2', '-ffp-contract=off', str(OUT/'jpeg.lo.ll'), '-o', str(OUT/'jpeg.exe')], check=True)
with (HERE/'../tests/data/oracle_shard_jpeg.txt').open('w') as f:
    subprocess.run([str(OUT/'jpeg.exe')], stdout=f, check=True)
print('JPEG oracle complete:', ll)

