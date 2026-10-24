"""Build the actual TARGET_PC THP C as a test-only DLL, with shipping FP lowering.

Only representation adaptations are made: pointer-sized Huffman addresses, native
scratch placement, predictor field access instead of GC-offset alias views, and
big-endian file-word decoding. No decoder algorithms are replaced.
"""
from pathlib import Path
import json, os, re, subprocess, sys

ROOT = Path(__file__).resolve().parents[3]
os.chdir(ROOT)
BUILD = ROOT / 'rust/target/shard985-thp'
CLANG = r'C:\Program Files\LLVM\bin\clang.exe'
sys.path.insert(0, str(ROOT / 'rust/melee_types/gen'))
import gen_types as g

def adapted_source():
    s = (ROOT / 'libs/dolphin/src/dolphin/thp/THPDec.c').read_text()
    s = s.replace('int huffmanBits', 'uintptr_t huffmanBits')
    s = s.replace('(int) huffmanBits', '(uintptr_t) huffmanBits')
    s = s.replace('DCZeroRange(info, 0x920);', 'DCZeroRange(info, (sizeof(*info) + 31) & ~31u);')
    s = s.replace('info->scratch += 0x920;', 'info->scratch += (sizeof(*info) + 31) & ~31u;')
    s = s.replace('p = (u8*) OSRoundUp32B(info->scratch);',
                  'p = (u8*) (((uintptr_t) info->scratch + 31) & ~(uintptr_t) 31);')
    # On PPC all C loads decode these compressed bytes as big endian.
    s = s.replace('info->currByte = *p;', 'info->currByte = __builtin_bswap32(*p);')
    s = s.replace('return *p;', 'return __builtin_bswap32(*p);')
    s = s.replace('((const u32*) info->file)[1]', '__builtin_bswap32(((const u32*) info->file)[1])')
    s = s.replace('info->currByte = *ptr;', 'info->currByte = __builtin_bswap32(*ptr);')
    for ch, i in [('Y', 0), ('U', 1), ('V', 2)]:
        # These legacy view structs contain fixed GC padding, so native offsets differ.
        s = re.sub(r'&\(\((?:struct )?THPFileInfoDCTComp'+ch+r'View\*\) info\)->predDC',
                   f'&info->components[{i}].predDC', s)
    return s

def main():
    BUILD.mkdir(parents=True, exist_ok=True)
    s = adapted_source()
    # The one forward declaration in the SDK header also needs the native-width ABI.
    header = (ROOT / 'libs/dolphin/include/dolphin/thp/thp.h').read_text().replace(
        '(THPFileInfo*, u8, int)', '(THPFileInfo*, u8, uintptr_t)')
    inc = BUILD / 'dolphin/thp'; inc.mkdir(parents=True, exist_ok=True)
    (inc / 'thp.h').write_text(header)
    src = BUILD / 'thp_native.c'; src.write_text(s)
    args = [a for a in g.NAT if a != '-include']
    # Use the same source type parser as the one type generator.
    tu = g.parse(['-I' + str(BUILD), *g.NAT], src)
    wrappers = []
    names = []
    for c in tu.cursor.get_children():
        if c.kind != g.K.FUNCTION_DECL or not c.is_definition() or Path(c.location.file.name).resolve() != src:
            continue
        params = list(c.get_arguments())
        signature = ', '.join(p.type.spelling + ' ' + p.spelling for p in params) or 'void'
        call = c.spelling + '(' + ', '.join(p.spelling for p in params) + ')'
        wrappers.append('__declspec(dllexport) ' + c.result_type.spelling + ' thp_ref_' + c.spelling + '(' + signature + ') { ' +
                        ('return ' if c.result_type.kind != g.T.VOID else '') + call + '; }')
        names.append(c.spelling)
    wrappers.append('''
void DCZeroRange(void* p, u32 n) { memset(p, 0, n); }
u32 LCStoreData(void* d, void* s, u32 n) { memcpy(d,s,n); return 0; }
void LCQueueWait(u32 n) { (void)n; }
void OSReport(char* format, ...) { (void)format; }
/* math_ppc.h emits an unused out-of-line sqrtf; the THP decoder has no sqrt calls. */
double __frsqrte(double x) { (void)x; return 0; }
__declspec(dllexport) void thp_ref_bind(THPFileInfo* info, float* q, u8* out, u32 width) {
    Gq.value=q; Gbase.value=out; Gwid.value=width;
    Ydchuff.value=Udchuff.value=Vdchuff.value=&info->huffmanTabs[0];
    Yachuff.value=Uachuff.value=Vachuff.value=&info->huffmanTabs[1];
}
__declspec(dllexport) void thp_ref_work(u8** out) {
    out[0]=__THPLCWork672[0]; out[1]=__THPLCWork672[1]; out[2]=__THPLCWork672[2];
}
''')
    driver = BUILD / 'driver.c'; driver.write_text('#include "thp_native.c"\n' + '\n'.join(wrappers))
    ll, lo = BUILD / 'thp.ll', BUILD / 'thp.lo.ll'
    flags = ['-I' + str(BUILD), *g.NAT, '-ffp-contract=on', '-fno-fast-math', '-ftrivial-auto-var-init=zero',
             '-O2', '-Xclang', '-disable-llvm-passes', '-S', '-emit-llvm']
    subprocess.run([CLANG, *flags, str(driver), '-o', str(ll)], cwd=ROOT, check=True)
    subprocess.run([sys.executable, str(ROOT/'rust/melee_runtime/oracle/lower.py'), str(ll), str(lo)], check=True)
    subprocess.run([CLANG, '-shared', '-O0', str(lo), '-fuse-ld=lld', '-o', str(BUILD/'thp_oracle.dll')], check=True)
    (BUILD/'functions.json').write_text(json.dumps(names, indent=2))
    # A field-by-field walker over the same generated record types used by both states.
    info = next(c for c in tu.cursor.get_children() if c.spelling == '_THPFileInfo' and c.is_definition())
    lines = ['// Generated from THPFileInfo by oracle/build_thp.py.',
             'unsafe fn state_fields(info: &THPFileInfo, base: usize) -> std::vec::Vec<(String,u64)> {',
             'let mut out = std::vec::Vec::new();']
    def walk(ty, expr, label, depth=0):
        t = ty.get_canonical()
        if t.kind == g.T.CONSTANTARRAY:
            i=f'i{depth}'; lines.append(f'for {i} in 0..{t.element_count} {{')
            walk(t.element_type, f'{expr}[{i}]', label+'[]', depth+1); lines.append('}')
        elif t.kind == g.T.RECORD:
            for f in t.get_fields():
                walk(f.type, expr+'.'+f.spelling, label+'.'+f.spelling, depth)
        else:
            if t.kind == g.T.POINTER:
                value=f'if {expr}.is_null() {{0}} else {{({expr} as usize).wrapping_sub(base) as u64}}'
            elif t.kind == g.T.FLOAT: value=f'{expr}.to_bits() as u64'
            elif label.endswith('.x10') and 'components' in label:
                value=f'if {expr} == 0 {{0}} else {{{expr}.wrapping_sub(base as u32) as u64}}'
            else: value=f'{expr} as u64'
            lines.append(f'out.push(("{label}".into(), {value}));')
    walk(info.type, '(*info)', 'THPFileInfo')
    lines += ['out', '}']
    (ROOT/'rust/melee_runtime/tests/thp_state/fields.rs').write_text('\n'.join(lines)+'\n')
    print(f'THP oracle: {len(names)} actual C definitions, {ll.read_text().count("call float @llvm.fmuladd")} f32 fused sites')

if __name__ == '__main__': main()
