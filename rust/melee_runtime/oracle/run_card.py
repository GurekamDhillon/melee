#!/usr/bin/env python3
"""Native card.c oracle: python rust/melee_runtime/oracle/run_card.py [--mirror].

Reuses melee_modes/gen/cref's native flags, AST traversal and record hashing, and
runtime/oracle/lower.py for shipping arithmetic. --mirror regenerates the Rust
fixture from the fixture C only via c2rust in WSL. No production implementation
is copied into the Rust test. Four pointer-int repairs are the complete C diff.
"""
from pathlib import Path
import json, os, re, subprocess, sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
BUILD = ROOT / 'rust/target/card-oracle'
BUILD.mkdir(parents=True, exist_ok=True)
os.chdir(ROOT)
os.environ['GALE_INCLUDE'] = str(ROOT / 'rust/target/empty-include')
sys.path.insert(0, str(ROOT / 'rust/melee_modes/gen/cref'))
import gen_tu as g
import run_tu as runner

source = (ROOT / 'src/sysdolphin/baselib/card.c').read_text()
native = source
for old, new in [
    ('(hdr_offset +\n                                            (u32) state->sector_buf)', '(state->sector_buf + hdr_offset)'),
    ('(hdr_offset13 +\n                                            (u32) state->sector_buf)', '(state->sector_buf + hdr_offset13)'),
    ('(u8*) (hdr_offset + (s32) state->sector_buf)', '(state->sector_buf + hdr_offset)'),
    ('s32 addr = (s32) state->sector_buf;\n            memset((u8*) (hdr_offset + off + addr), 0, remaining);',
     'u8* addr = state->sector_buf;\n            memset(addr + hdr_offset + off, 0, remaining);'),
]:
    assert native.count(old) == 1, old
    native = native.replace(old, new)
# Native layout adds padding before file_data and widens its pointers. The
# spare-block table's index-9 window is a GC alias, not native array padding.
begin = native.index('void fn_803AC3F8(CardState* state, u8* data, s32 file_idx)\n{')
end = native.index('void hsd_803AC558(',begin)
part = native[begin:end].replace('state->file_sizes[idx]', '(idx == 9 ? (s32)(unsigned long long)state->file_data[0] : state->file_sizes[idx])')
part = part.replace('state->file_flags[idx]', '(idx == 9 ? state->file_sizes[0] : state->file_flags[idx])')
native = native[:begin]+part+native[end:]
(BUILD / 'card_native.c').write_text(native, newline='\n')
tu = g.Tu('src/sysdolphin/baselib/card.c')
names = list(tu.defs)
assert len(names) == 110, len(names)
ported = re.findall(r'^pub unsafe extern "C" fn (\w+)', (ROOT/'rust/melee_runtime/src/tu/card.rs').read_text(), re.M)
assert set(names) == set(ported), (set(names)-set(ported), set(ported)-set(names))
walker = g.Walk(tu)
hashes = []
for tn in ['CardState', 'CARDStat', 'CardIconInfo']:
    c, _ = walker.gen(tn)
    c = c[c.index('static void hash_'):]
    c = c.replace('hash_', 'card_oracle_hash_').replace('HM(', 'card_oracle_h(').replace('PI(', 'card_oracle_ptr(')
    hashes.append(c)
(BUILD/'card_hash.inc').write_text('\n'.join(hashes), newline='\n')

def arg(fn, p):
    name = p.spelling
    typ = p.type.spelling
    if 'CardCmdBuf' in typ: return '&card_oracle_cmd_buf[' + ('0' if name=='init_buf' else '1') + ']'
    if 'CardCmd' in typ: return 'c'
    if 'CardState' in typ: return 's'
    if 'CARDFileInfo' in typ: return '&s->file_info'
    if 'CARDStat' in typ: return '&card_oracle_stat'
    if 'CardIconInfo' in typ: return '&card_oracle_icon'
    if 'Callback' in typ: return 'card_oracle_callback'
    values = {
        'filename':'card_oracle_filename', 'comment':'card_oracle_comment',
        'banner':'card_oracle_banner', 'icons':'card_oracle_icons',
        'data':'card_oracle_data', 'expected_data':'card_oracle_data', 'payload':'card_oracle_data',
        'fdata':'card_oracle_data', 'file_payload':'card_oracle_data', 'dst':'card_oracle_data',
        'buf':'card_oracle_data', 'buffer':'card_oracle_data', 'work_buf':'card_oracle_sector',
        'value':'card_oracle_data', 'chan':'k & 1', 'card_result':'k % 3 == 0 ? -3 : 0',
        'file_no':'7', 'file_idx':'f', 'file_size':'8192 + len', 'file_flags':'k % 4',
        'block_idx':'b', 'block':'b', 'phys':'b', 'block_id':'b',
        'seq':'k & 255', 'seq_num':'k & 255', 'seq_a':'(k * 17) % 258 - 1',
        'seq_b':'(k * 31) % 258 - 1', 'snap':'curr_tail',
        'size':'len', 'length':'len', 'data_size':'len', 'payload_size':'len',
        'hdr_offset':'k % 64', 'offset':'k % 128', 'blocks_before':'k % 9', 'i':'k % 7',
        'logical':'k % 9 - 4', 'sector_size':'s->sector_size', 'data_block':'b',
        'async':'k & 1', 'is_new':'(k >> 1) & 1', 'primary':'card_oracle_primary',
        'count':'3', 'current_seq':'k & 255', 'needs_rewrite':'&card_oracle_out1',
        'file_blocks':'&card_oracle_out1', 'total_blocks':'&card_oracle_out2',
    }
    if name not in values: raise RuntimeError((fn, name, typ))
    return values[name]

cases=['switch (op) {']
for op, (name, fn) in enumerate(tu.defs.items()):
    pre = ''
    if name in ['rollbackCardCommands','fn_803AC2E0','cancelQueuedCardCommands','fn_803AE7F8_rewind','fn_803AF3F0_rewind','fn_803B0120_rewind']:
        pre = 'hsd_804D7998 = curr_tail; for (i = 0; i < 5; i++) { commands[curr_tail].type = CARD_CMD_CLEAR_BUF; curr_tail = (curr_tail + 1) % 128; }'
    if name == 'hsd_803AC558':
        pre = 'fn_803AC3F8(s, card_oracle_data, f); memset(s->file_sizes, 0, sizeof s->file_sizes);'
    if name in ['hsd_803AAA48', 'hsd_803A949C']:
        pre = '''c->type = k % 18; memset(&c->write, 0, sizeof c->write);
        if (c->type == CARD_CMD_CREATE_FILE) { c->create.filename = card_oracle_filename; c->create.size = 8192; }
        else if (c->type == CARD_CMD_WRITE_HEADER || c->type == CARD_CMD_VERIFY_HEADER) { c->header.index = 0; c->header.banner = card_oracle_banner; c->header.icons = card_oracle_icons; }
        else if (c->type == CARD_CMD_READ_HEADER) { c->read_header.index = 0; c->read_header.comment = card_oracle_comment; c->read_header.banner = card_oracle_banner; c->read_header.icons = card_oracle_icons; }
        else { c->write.file_idx = f; c->write.phys = b; c->write.block_id = b; c->write.seq = k & 255; c->write.data = card_oracle_data; c->write.size = len; c->write.offset = fn_803ACBE8(s, b); }
        commands[curr_head] = *c; curr_tail = (curr_head + 1) % 128;
        active_requests.state = s; active_requests.type = k % 8; active_requests.callback = card_oracle_callback;
        hsd_804D799C = ''' + ('1;' if name=='hsd_803A949C' else '0;')
    if name == 'fn_803AA790':
        pre = '''CardRequest* req = &requests[hsd_804D7990]; req->type = k % 7; req->state = s; req->callback = card_oracle_callback;
        if (req->type == CARD_REQ_CREATE_FILE) { req->create.filename = card_oracle_filename; req->create.banner = card_oracle_banner; req->create.icons = card_oracle_icons; }
        else if (req->type == CARD_REQ_SET_STATUS) { req->status.banner = card_oracle_banner; req->status.icons = card_oracle_icons; }
        else if (req->type == CARD_REQ_READ_HEADER) { req->header.comment = card_oracle_comment; req->header.banner = card_oracle_banner; req->header.icons = card_oracle_icons; }
        else { req->file.file_idx = f; req->file.buf = card_oracle_data; }
        hsd_804D7994 = (hsd_804D7990 + 1) % 32;'''
    call = name + '(' + ', '.join(arg(name,p) for p in fn.get_arguments()) + ')'
    rtype = g.canon(fn.result_type).kind
    if rtype == g.T.VOID: stmt = call + ';'
    elif rtype == g.T.POINTER: stmt = 'r = card_oracle_ptr(' + call + ');'
    else: stmt = 'r = ' + call + ';'
    cases.append(f'case {op}: {{ /* {name} */ {pre}\n {stmt} break; }}')
cases.append('}')
(BUILD/'card_cases.inc').write_text('\n'.join(cases), newline='\n')
(HERE/'card_functions.txt').write_text('\n'.join(f'{i} {n}' for i,n in enumerate(names))+'\n', newline='\n')

def run(cmd):
    p = subprocess.run([str(a) for a in cmd], cwd=ROOT, text=True, capture_output=True)
    if p.returncode:
        print(p.stdout, p.stderr); raise SystemExit(p.returncode)
    return p.stdout

flags = runner.FLAGS + ['-I', str(BUILD), '-I', str(ROOT/'src/sysdolphin/baselib')]
ll = BUILD/'card.ll'; lower = BUILD/'card.lo.ll'; exe = BUILD/'card.exe'
run([runner.CLANG, *flags, HERE/'card_driver.c', '-o', ll])
print('card.c float fusion sites:', len(re.findall(r'call .*@llvm.fmuladd',ll.read_text())))
run([sys.executable, runner.LOWER, ll, lower])
run([runner.CLANG, '-O1', '-w', lower, '-o', exe, '-Wl,/OPT:NOICF', '-fuse-ld=lld'])
out = run([exe])
(ROOT/'rust/melee_runtime/tests/data/tu_card.txt').write_text(out, newline='\n')
print(f'card.c oracle: {len(out.splitlines())} rows / {len(names)} definitions')

if '--mirror' in sys.argv:
    # c2rust is reference-only for the fixture too: extract only card_oracle_* items.
    wroot = '/mnt/' + ROOT.drive[0].lower() + ROOT.as_posix()[2:]
    sys.path.insert(0,str(ROOT/'tools/rust-port'))
    import gen_cc
    wf = [wroot+'/rust/target/empty-include' if x=='@GALE@' else x for x in gen_cc.FLAGS]
    wf += ['-I.', '-Irust/target/card-oracle', '-Isrc/sysdolphin/baselib']
    file = 'rust/melee_runtime/oracle/card_driver.c'
    (BUILD/'compile_commands.json').write_text(json.dumps([{'directory':wroot,'file':file,'arguments':['clang',*wf,'-c',file]}]))
    command = f'. "$HOME/opt/c2rust-env.sh" && cd "{wroot}" && c2rust transpile rust/target/card-oracle/compile_commands.json -o rust/target/card-oracle/mirror --emit-modules --overwrite-existing'
    run(['wsl','-d','Debian','--','bash','-c',command])
    raw = (BUILD/'mirror/src/card_driver.rs').read_text()
    items=[]
    for m in re.finditer(r'(?m)^(?:(?:pub )?unsafe extern "C" fn|(?:pub )?static mut) (card_oracle_\w+)\b',raw):
        start=m.start(); pos=start; depth=0; isfn='fn ' in m.group(0)
        if isfn:
            pos=raw.index('{',m.end()); depth=1; pos+=1
            while depth:
                depth += (raw[pos]=='{')-(raw[pos]=='}'); pos+=1
        else:
            while True:
                ch=raw[pos]; depth += (ch in '([{')-(ch in ')]}')
                pos+=1
                if ch==';' and depth==0: break
        items.append(raw[start:pos])
    text='\n\n'.join(items)
    repl = {'::core::ffi::c_int':'i32','::core::ffi::c_longlong':'i64','::core::ffi::c_ulonglong':'u64','::core::ffi::c_long':'i32','::core::ffi::c_ulong':'usize','::core::ffi::c_uchar':'u8','::core::ffi::c_uint':'u32','::core::ffi::c_char':'i8','::core::ffi::c_void':'c_void','u8_0':'u8','u16_0':'u16','u32_0':'u32','bool_0':'i32','gc_size_t':'usize','type_0':'r#type','c2rust_unnamed':'anon_8'}
    for old in sorted(repl,key=len,reverse=True): text=text.replace(old,repl[old])
    # c2rust assigns anonymous names by encounter order, so read the owner fields.
    unions = {re.search(r'pub struct '+owner+r' \{.*?pub c2rust_unnamed: (\w+)',raw,re.S)[1]:target for owner,target in [('CardCmd','CardCmd_anon_193'),('CardRequest','CardRequest_anon_258')]}
    text = re.sub(r'\bC2RustUnnamed(?:_\d+)?\b', lambda m: unions.get(m[0],'i32'), text)
    pre='''// Generated fixture mirror of oracle/card_driver.c by run_card.py --mirror.
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
'''
    hooks = re.findall(r'hook!\((\w+), (\w+)',(ROOT/'rust/melee_runtime/src/tu/card.rs').read_text())
    test='\n#[test]\nfn all_card_definitions_match_native_c() { unsafe {\n'
    for name,slot in hooks: test+=f'    m::ext::{slot} = Some(card_oracle_{name});\n'
    test+='''    let mut count = [0u32; 110];
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
'''
    (ROOT/'rust/melee_runtime/tests/tu_card.rs').write_text(pre+text+test,newline='\n')
    print('card.c fixture Rust mirror regenerated')
