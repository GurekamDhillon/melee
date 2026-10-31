#!/usr/bin/env python3
"""Per-frame call-sequence diff between the C build and the Rust engine (consolidation step 1).

  C side:    MELEE_CALLTRACE_SEQ=c.seq [MELEE_CALLTRACE_FRAMES=0-300] <golden env, trace build>   (pc/platform/gw_calltrace.c)
  Rust side: melee golden ... --calltrace r.ct [--calltrace-frames 0-300]   (melee built with --features calltrace)
  python tools/rust-port/trace/seqdiff.py --c c.seq --map melee-pc.map --rust r.ct [--frames 0-300] \
         [--only-rust-names] [--filter REGEX] [--ignore-inlined LIST]

Both sides are reduced to a per-frame list of lower-cased function names. C names come from the map (rva -> symbol,
`gw_` prefix and leading underscores stripped, like calltrace.py). Rust names are the C names as the TU files spell
them (`ftcoll_8007ae80` and `ftColl_8007AE80` compare equal).

By default only functions that appear on BOTH sides anywhere in the compared frames are kept ("scope" = the
functions the Rust side has ported and instruments); `--only-rust-names` is the same thing, `--all` compares every
C call including ones the Rust engine does not run yet. `--filter` keeps only names matching a regex (a migrated
subsystem, e.g. `^(ft|Fighter)`). `--ignore-inlined FILE` drops names listed in FILE (one per line; functions the C
compiler inlines at -O2 are logged by the C hook but are not calls in Rust).

Output: for every frame up to the first mismatch the number of calls compared, then the first frame and position
where the sequences differ with 5 calls of context from each side. Exit status 1 when a difference is found.
No disc data is involved: function names and positions only."""
import argparse, re, sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))


def norm(n):
    n = re.sub(r'^_?gw_', '', n) if n.startswith(('_gw_', 'gw_')) else n.lstrip('_') or n
    return n.lower()


def read_c(path, mapfile):
    from calltrace import read_map
    syms = read_map(mapfile)
    fr = defaultdict(list)
    for line in Path(path).read_text().splitlines():
        if line.startswith('#'):
            continue
        f, rva, seq = line.split(',')
        name = syms.get(int(rva, 16), (None, None))[0]
        fr[int(f)].append((int(seq), norm(name) if name else 'rva_' + rva))
    return {f: [n for _, n in sorted(v)] for f, v in fr.items()}


def read_rust(path):
    fr = defaultdict(list)
    for line in Path(path).read_text().splitlines()[1:]:
        f, rest = line.split(',', 1)
        name, seq = rest.rsplit(',', 1)
        fr[int(f)].append((int(seq), norm(name)))
    return {f: [n for _, n in sorted(v)] for f, v in fr.items()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--c', required=True)
    ap.add_argument('--map', required=True)
    ap.add_argument('--rust', required=True)
    ap.add_argument('--frames', default=None, help='A-B (default: all frames both sides have)')
    ap.add_argument('--all', action='store_true', help='compare every C call, not only functions the Rust side ran')
    ap.add_argument('--only-rust-names', action='store_true', help='(default) restrict C to names the Rust side logged')
    ap.add_argument('--filter', default=None)
    ap.add_argument('--ignore-inlined', default=None)
    ap.add_argument('--auto-ignore', type=int, default=0, metavar='N', help='with --anchor: when exactly one call is extra on one side at the first '
                    'difference (a function one compiler inlines and the other does not), add it to the ignore set and go on, up to N times; '
                    'the names are printed so the list can be reviewed')
    ap.add_argument('--anchor', default=None, help='re-chunk both sides at each call of this function (frame phases differ: the C log '
                    'frame starts after render, the engine log at the pad renew); chunk k of C is compared with chunk k of Rust')
    a = ap.parse_args()
    c, r = read_c(a.c, a.map), read_rust(a.rust)
    lo, hi = (None, None)
    if a.frames:
        lo, hi = map(int, a.frames.split('-'))
    frames = sorted(f for f in set(c) & set(r) if f >= 0 and (lo is None or lo <= f <= hi))
    if not frames:
        print('no common frames', file=sys.stderr)
        return 2
    ign = set()
    if a.ignore_inlined:
        ign = {norm(l.strip()) for l in Path(a.ignore_inlined).read_text().splitlines() if l.strip()}
    rx = re.compile(a.filter, re.I) if a.filter else None
    scope = None if a.all else {n for f in frames for n in r[f]} & {n for f in frames for n in c[f]}

    def keep(n):
        return n not in ign and (scope is None or n in scope) and (rx is None or rx.search(n))

    total = 0
    if a.anchor:
        anc = norm(a.anchor)

        def chunks(d):
            out, cur, fid = [], None, None
            for f in frames:
                for n in d[f]:
                    if n == anc:
                        cur = []
                        out.append((f, cur))
                    if cur is not None and keep(n):
                        cur.append(n)
            return out
        cc, rc = chunks(c), chunks(r)
        auto = []
        for k in range(min(len(cc), len(rc))):
            (fc, cs), (fr, rs) = cc[k], rc[k]
            while cs != rs and len(auto) < a.auto_ignore:
                i = next((j for j in range(min(len(cs), len(rs))) if cs[j] != rs[j]), min(len(cs), len(rs)))
                if i < len(rs) and i + 1 <= len(cs) and rs[i + 1:i + 2] == cs[i:i + 1] and i < len(cs):
                    nm = rs[i]
                elif i < len(cs) and cs[i + 1:i + 2] == rs[i:i + 1] and i < len(rs):
                    nm = cs[i]
                else:
                    break
                auto.append(nm)
                ign.add(nm)
                cs = [n for n in cs if n != nm]
                rs = [n for n in rs if n != nm]
            if auto and k == 0:
                pass
            if cs != rs:
                if auto:
                    print('auto-ignored:', ' '.join(auto))
                i = next((j for j in range(min(len(cs), len(rs))) if cs[j] != rs[j]), min(len(cs), len(rs)))
                print(f'first difference: chunk {k} (C frame {fc}, Rust frame {fr}) position {i} (C {len(cs)} calls, Rust {len(rs)} calls)')
                print('  C   :', ' '.join(cs[max(0, i - 8):i + 8]))
                print('  Rust:', ' '.join(rs[max(0, i - 8):i + 8]))
                return 1
            total += len(cs)
        if auto:
            print('auto-ignored:', ' '.join(auto))
        print(f'anchored sequences identical over {min(len(cc), len(rc))} chunks (C {len(cc)}, Rust {len(rc)}), {total} calls')
        return 0
    for f in frames:
        cs = [n for n in c[f] if keep(n)]
        rs = [n for n in r[f] if keep(n)]
        if cs != rs:
            i = next((k for k in range(min(len(cs), len(rs))) if cs[k] != rs[k]), min(len(cs), len(rs)))
            print(f'first difference: frame {f} position {i} (C {len(cs)} calls, Rust {len(rs)} calls)')
            print('  C   :', ' '.join(cs[max(0, i - 5):i + 6]))
            print('  Rust:', ' '.join(rs[max(0, i - 5):i + 6]))
            print(f'{total} calls in {frames.index(f)} identical frames before it')
            return 1
        total += len(cs)
    print(f'sequences identical over {len(frames)} frames ({frames[0]}..{frames[-1]}), {total} calls, scope {len(scope) if scope is not None else "all"} functions')
    return 0


sys.exit(main())
