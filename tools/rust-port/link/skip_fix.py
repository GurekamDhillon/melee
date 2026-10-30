#!/usr/bin/env python3
"""Run cargo build -p melee_link; every gen_table.rs line that fails the cast size check is added to link_skip.txt
(one hook path per line), then gen_link.py is rerun. Repeats until the build is clean."""
import subprocess, re, os, sys
here = os.path.dirname(os.path.abspath(__file__))
root = os.path.abspath(os.path.join(here, '..', '..', '..'))
env = dict(os.environ, CARGO_PROFILE_DEV_DEBUG='0', CARGO_INCREMENTAL='0')
skip = os.path.join(here, 'link_skip.txt')
for it in range(40):
    r = subprocess.run(['cargo', 'build', '-p', 'melee_link', '--message-format=short'], cwd=os.path.join(root, 'rust'), env=env, capture_output=True, text=True)
    out = r.stdout + r.stderr
    lines = sorted({int(m.group(1)) for m in re.finditer(r'gen_table\.rs:(\d+):', out)})
    errs = [l for l in out.splitlines() if 'error' in l and 'gen_table' not in l and 'cast' not in l and 'could not compile' not in l and 'aborting' not in l]
    print(f'iter {it}: {len(lines)} failing lines; other errors: {len(errs)}')
    if r.returncode == 0: break
    if not lines:
        print(out[-3000:]); sys.exit(1)
    gt = open(os.path.join(root, 'rust', 'melee_link', 'src', 'gen_table.rs'), encoding='utf8').read().splitlines()
    new = []
    for ln in lines:
        m = re.search(r'unsafe \{ (?:\w+)::(\S+) = Some', gt[ln-1])
        if m: new.append(m.group(1))
    if not new: print(out[-3000:]); sys.exit(1)
    with open(skip, 'a') as fh:
        for k in new: fh.write(k + '\n')
    subprocess.run([sys.executable, os.path.join(here, 'gen_link.py')], check=True)
