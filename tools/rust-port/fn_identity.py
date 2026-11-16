"""Give every C-callback entry fn of the port one address that no build can fold (docs/rust-port/notes/fn-identity.md).

The game names a callback by its address: it stores `fp->accessory4_cb = X` and later tests `cb == X`, and the
oracle harnesses turn stored pointers back into C names the same way. An optimized Rust build breaks that in two
ways, both seen in chars_ref on release (accessory4_cb of ftSk_SpecialS_80111440):
  - copies: rustc treats a small fn as inlinable across crates and emits a local copy in every codegen unit that
    takes its address, so one fn has several addresses (`chars::cb::ftseakspecials__ftSk_SpecialS_8011097C`
    twice in one binary);
  - folding: two entries with one body (two Rust wrappers of one C callback, or distinct C callbacks with
    identical code, the empty procs) are merged by LLVM's MergeFunctions or a linker's ICF (lld --icf,
    MSVC /OPT:ICF), so a store of one reads back as the other.
A profile flag only protects builds that use it, so the identity goes in the source:
  - `#[inline(never)]` on each entry: never inlinable, so one copy;
  - `core::hint::black_box(ID)` as its first statement, ID a 32-bit FNV-1a of its path, unique by construction
    (checked here): no two entries have one body, so nothing can fold them;
  - where `chars::cb` wraps the same port as a TU's own `cb_X`, the cb name re-exports `cb_X` (one fn per C
    callback, so both stores compare equal); `chars::cb::CALLBACKS` is a static (one table).
Idempotent: run it after any generator that rewrites these files.
    python tools/rust-port/fn_identity.py [--check]
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUST = ROOT / "rust"
MARK = "// fn identity (tools/rust-port/fn_identity.py)"
# (crate dir, glob, entry-fn name pattern): the fns stored in callback fields / tables and compared or keyed
RULES = [
    ("melee_chara", "src/chars/cb.rs", r"\w+"),
    ("melee_chara", "src/tu/*.rs", r"cb_\w+"),
]
FN = r'(?P<pre>(?:#\[[^\]]*\]\s*\n\s*|///[^\n]*\n\s*)*)pub unsafe extern "C" fn (?P<name>{pat})\((?P<args>[^)]*)\)(?P<ret>[^{{]*)\{{\n'


def fnv32(s):
    h = 0x811C9DC5
    for b in s.encode():
        h = ((h ^ b) * 0x01000193) & 0xFFFFFFFF
    return h


def dedupe_cb(cb_rs, tu_dir):
    """chars::cb wrappers whose body calls the very port a TU's `cb_X` calls become `pub use` of that `cb_X`."""
    tu_cb = {}
    for f in sorted(tu_dir.glob("*.rs")):
        for n, body in re.findall(r'pub unsafe extern "C" fn cb_(\w+)\(g: \*mut HSD_GObj\) \{\n(.*?)\n\}', f.read_text(encoding="utf-8"), re.S):
            calls = re.findall(r"\b(\w+)\(g\b", body)
            if calls == [n]:
                tu_cb[(f.stem, n)] = True
    chars = cb_rs.parent

    def target(call):
        m = re.search(r"crate::tu::(\w+)::(\w+)\(g\)", call)
        if m:
            return m.groups()
        m = re.search(r"crate::chars::(r\d+)::(\w+)::\w+::(\w+)\(g\)", call)
        if m:
            r, ch, fn = m.groups()
            p = chars / r / f"{ch}.rs"
            if p.exists():
                for mm in re.finditer(r"pub use crate::tu::(\w+)::(?:\{([^}]*)\}|(\w+));", p.read_text(encoding="utf-8")):
                    if fn in [x.strip() for x in (mm.group(2) or mm.group(3)).split(",")]:
                        return (mm.group(1), fn)
        return None

    s = cb_rs.read_text(encoding="utf-8")
    n = 0

    def sub(m):
        nonlocal n
        tg = target(m.group("body"))
        if tg and tg in tu_cb:
            n += 1
            return f"pub use crate::tu::{tg[0]}::cb_{tg[1]} as {m.group('name')};\n"
        return m.group(0)

    s = re.sub(r'(?:///[^\n]*\n)*(?:#\[inline\(never\)\][^\n]*\n)?pub unsafe extern "C" fn (?P<name>\w+?__\w+)\(g: \*mut HSD_GObj\) \{\n'
               r'(?:\s*core::hint::black_box[^\n]*\n)?\s*(?P<body>[^\n]*)\n\}\n', sub, s)
    s = s.replace("pub const CALLBACKS:", "pub static CALLBACKS:")
    return s, n


def mark(text, pat, path_prefix, ids):
    out, n = [], 0
    pos = 0
    for m in re.finditer(FN.format(pat=pat), text):
        name = m.group("name")
        key = f"{path_prefix}::{name}"
        i = fnv32(key)
        if i in ids and ids[i] != key:
            raise SystemExit(f"fn_identity: id clash {ids[i]} / {key}")
        ids[i] = key
        pre = m.group("pre")
        head = text[m.start():m.end()]
        if "#[inline(never)]" not in pre:
            indent = re.match(r"\s*", text[m.start() + len(pre):]).group(0) if pre else ""
            head = head.replace(f"{pre}pub unsafe", f"{pre}#[inline(never)]\n{indent}pub unsafe", 1)
        line = f"    core::hint::black_box(0x{i:08X}u32); {MARK}\n"
        rest_start = m.end()
        nxt = text[rest_start:rest_start + 200].split("\n", 1)[0]
        if MARK in nxt:
            rest_start += len(nxt) + 1
        out.append(text[pos:m.start()] + head + line)
        pos = rest_start
        n += 1
    out.append(text[pos:])
    return "".join(out), n


def main():
    check = "--check" in sys.argv
    ids, changed, total, deduped = {}, [], 0, 0
    for crate, glob, pat in RULES:
        base = RUST / crate
        for f in sorted(base.glob(glob)):
            old = f.read_text(encoding="utf-8")
            new = old
            if f.name == "cb.rs" and f.parent.name == "chars":
                new, d = dedupe_cb(f, base / "src/tu")
                deduped += d
            mod = ".".join(f.relative_to(base / "src").with_suffix("").parts)
            new, n = mark(new, pat, f"{crate}::{mod}", ids)
            total += n
            if new != old:
                changed.append(f)
                if not check:
                    f.write_text(new, encoding="utf-8", newline="\n")
    print(f"fn_identity: {total} entry fns, {deduped} chars::cb wrappers re-exported, {len(changed)} files "
          f"{'would change' if check else 'changed'}")
    if check and changed:
        for f in changed:
            print("  ", f.relative_to(ROOT))
        raise SystemExit(1)


if __name__ == "__main__":
    main()
