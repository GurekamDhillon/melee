#!/usr/bin/env python3
"""Insert (or remove) the call-trace entry marker `melee_core::ct!("name");` as the first statement of every
top-level `pub`/`#[no_mangle]` fn in rust/*/src/tu/**.rs (consolidation step 1; rust/melee_core/src/ct.rs).

  python tools/rust-port/trace/insert_ct.py            insert (idempotent)
  python tools/rust-port/trace/insert_ct.py --remove   strip every marker
  python tools/rust-port/trace/insert_ct.py --check    exit 1 if any eligible fn lacks a marker

Skipped: `const fn` (the marker is not const), declarations ending in `;`, fns inside macro_rules!/extern blocks
(only column-0 `fn` lines are looked at). Without the `calltrace` feature the marker expands to nothing, so the
instrumented source builds and behaves exactly as before."""
import re, sys, glob, os

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', '..', 'rust')
FN = re.compile(r'^(pub(\([a-z]+\))?\s+)?(unsafe\s+)?(extern\s+"C"\s+)?fn\s+([A-Za-z_0-9]+)')
MARK = re.compile(r'^\s*melee_core::ct!\("[^"]*"\);\n', re.M)


def body_open(s, i):
    """Index just after the `{` that opens the body of the fn whose `fn` keyword is at s[i:], or -1 (declaration)."""
    j = s.index('(', i)
    depth = 0
    n = len(s)
    while j < n:
        c = s[j]
        if c == '(':
            depth += 1
        elif c == ')':
            depth -= 1
            if depth == 0:
                break
        j += 1
    # after the args: return type / where clause, up to `{` or `;` (angle brackets can hold no braces here)
    k = j + 1
    while k < n and s[k] not in '{;':
        k += 1
    return k + 1 if k < n and s[k] == '{' else -1


def process(path, mode):
    src = open(path, encoding='utf-8', newline='').read()
    if mode == 'remove':
        out = MARK.sub('', src)
        if out != src:
            open(path, 'w', encoding='utf-8', newline='').write(out)
        return 0, 0
    lines = src.split('\n')
    pos = 0
    edits = []  # (offset after '{', name)
    missing = 0
    attrs_nm = False
    for ln in lines:
        m = FN.match(ln)
        if m and 'const fn' not in ln[:40] and (m.group(1) or attrs_nm):
            off = pos + m.start(0)
            ob = body_open(src, off + (m.start(0) - m.start(0)))
            if ob > 0:
                rest = src[ob:ob + 120]
                if not re.match(r'\s*melee_core::ct!\(', rest):
                    edits.append((ob, m.group(5)))
        attrs_nm = ln.startswith('#[no_mangle]') or (attrs_nm and ln.startswith('#['))
        pos += len(ln) + 1
    if mode == 'check':
        return 0, len(edits)
    if not edits:
        return 0, 0
    out = []
    last = 0
    for ob, name in edits:
        out.append(src[last:ob])
        # keep the body's first line break: `{\n    body` becomes `{\n    ct;\n    body`; one-liners get a space.
        nl = src[ob:ob + 2]
        if nl.startswith('\r\n') or nl.startswith('\n'):
            eol = '\r\n' if nl.startswith('\r\n') else '\n'
            line_start = ob + len(eol)
            m = re.match(r'[ \t]*', src[line_start:])
            ind = m.group(0) or '    '
            out.append(eol + ind + 'melee_core::ct!("%s");' % name)
        else:
            out.append(' melee_core::ct!("%s");' % name)
        last = ob
    out.append(src[last:])
    open(path, 'w', encoding='utf-8', newline='').write(''.join(out))
    return len(edits), 0


def main():
    mode = 'remove' if '--remove' in sys.argv else 'check' if '--check' in sys.argv else 'insert'
    files = sorted(glob.glob(os.path.join(ROOT, '*', 'src', 'tu', '**', '*.rs'), recursive=True))
    tot = miss = 0
    for f in files:
        a, b = process(f, mode)
        tot += a
        miss += b
    print(f'{mode}: {len(files)} files, {tot} markers inserted, {miss} missing')
    if mode == 'check' and miss:
        sys.exit(1)


main()
