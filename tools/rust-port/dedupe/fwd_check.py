#!/usr/bin/env python3
"""cargo check loop for fwd_dups.py: every fn whose forwarder fails to compile is restored (and listed in fwd_skip.txt)."""
import os, sys, json, subprocess, re
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(here, '..'))
import lint_logic as L
os.chdir(L.REPO)
edits = json.load(open(os.environ.get('TEMP', '/tmp') + '/fwd_edits.json'))
skipf = L.REPO / 'tools/rust-port/dedupe/fwd_skip.txt'
env = dict(os.environ, CARGO_PROFILE_DEV_DEBUG='0', CARGO_INCREMENTAL='0')
def rd(p): return open(p, encoding='utf-8', errors='surrogateescape', newline='').read()
def wr(p, t): open(p, 'w', encoding='utf-8', errors='surrogateescape', newline='').write(t)
for it in range(40):
    r = subprocess.run(['cargo', 'check', '--workspace', '--message-format=json'], cwd='rust', env=env, capture_output=True, text=True, errors='replace')
    errs = []
    for line in r.stdout.split('\n'):
        if not line.startswith('{'): continue
        try: j = json.loads(line)
        except Exception: continue
        if j.get('reason') != 'compiler-message': continue
        m = j['message']
        if m['level'] != 'error': continue
        for sp in m['spans']:
            if sp.get('is_primary'):
                errs.append((sp['file_name'].replace(chr(92), '/'), sp['line_start'], m['message']))
    if not errs:
        print('check clean after', it, 'rounds'); break
    print('round', it, 'errors', len(errs))
    reverted = set()
    for f, ln, msg in errs:
        rel = 'rust/' + f if not f.startswith('rust/') else f
        if not os.path.exists(rel): continue
        t = rd(rel)
        for e in edits:
            if e[0] != rel or (e[0], e[1]) in reverted: continue
            i = t.find(e[3])
            if i < 0: continue
            l0 = t.count('\n', 0, i) + 1; l1 = l0 + e[3].count('\n')
            if l0 <= ln <= l1:
                t = t[:i] + e[2] + t[i + len(e[3]):]
                wr(rel, t); reverted.add((e[0], e[1]))
                open(skipf, 'a').write(e[0] + ':' + e[1] + '\n')
                break
        else:
            print('unattributed:', rel, ln, msg[:100])
    if not reverted: print('no progress'); break
