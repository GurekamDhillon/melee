#!/usr/bin/env python3
"""Drive drop_dups.py to a compiling tree: apply, `cargo check`, add the canonical names the compiler complains about to the
global skip list (`*<TAB>name` in skip.txt), restore rust/ from HEAD and repeat. Run from the repo root with a clean rust/."""
import json, re, subprocess, sys
from pathlib import Path
R = Path(__file__).resolve().parents[3]
skipf = R / 'tools/rust-port/dedupe/skip.txt'
extra = sys.argv[1:]
for it in range(12):
    subprocess.run(['git', 'checkout', 'rust'], cwd=R, check=True, capture_output=True)
    print(subprocess.run([sys.executable, 'tools/rust-port/dedupe/drop_dups.py', '--skip', str(skipf)] + extra, cwd=R, capture_output=True, text=True).stdout.strip())
    env = dict(__import__('os').environ, CARGO_PROFILE_DEV_DEBUG='0', CARGO_INCREMENTAL='0')
    p = subprocess.run(['cargo', 'check', '--workspace', '--message-format=json'], cwd=R / 'rust', capture_output=True, text=True, env=env, errors='replace')
    names = set(); nerr = 0
    for line in p.stdout.splitlines():
        try: m = json.loads(line)
        except Exception: continue
        if m.get('reason') != 'compiler-message': continue
        msg = m['message']
        if msg['level'] != 'error': continue
        nerr += 1
        txt = msg['message']
        mm = re.search(r'(?:function|struct|constant|static) `(\w+)` is private', txt)
        if mm: names.add(mm.group(1))
        mm = re.search(r'unresolved imports? `[\w:]*::(\w+)`', txt)
        if mm: names.add(mm.group(1))
        for ch in msg.get('children', []):
            if 'defined here' in ch['message']:
                for sp in ch['spans']:
                    try:
                        line_t = open(R / 'rust' / sp['file_name'].replace(chr(92), '/'), encoding='utf-8', errors='replace').read().split('\n')[sp['line_start'] - 1]
                        f = re.search(r'\bfn\s+(\w+)', line_t)
                        if f: names.add(f.group(1))
                    except OSError: pass
        for sp in msg['spans']:
            if sp.get('is_primary') and 'reexport' in txt: pass
    known = {l.split('\t')[1].lower() for l in skipf.read_text().splitlines() if l.startswith('*\t')}
    new = {n for n in names if n.lower() not in known}
    print('iteration', it, 'errors', nerr, 'new global skips', sorted(new))
    if nerr == 0: break
    if not new: print('stuck: remaining errors need a manual fix'); break
    with open(skipf, 'a') as fh:
        for n in sorted(new): fh.write('*\t%s\n' % n)
