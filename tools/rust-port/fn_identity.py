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
The oracle harnesses key the rest: every `extern "C"` fn a test registers (`freg(m::f as usize)`, fnsym) gets the same
identity (REGISTERED_CRATES), so a pointer the code under test stored names one C function on every build. That
includes fns game code never takes as a value in Rust source: the item Logic*_Thrown / _ShieldBounced / ... callbacks
reach their tables through generated data, and the Windows kinds run misnamed 153 of them.
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
    # compared by address in game code
    ("melee_modes", "src/tu/geno_game.rs", r"geno_cb_nothing"),        # row.input_cb == geno_cb_nothing (empty body)
    ("melee_modes", "src/tu/geno_lab_mode.rs", r"fly_phys|fly_input"),  # phys_cb == fly_phys; fly_input is empty
    # HSD event list (hsd_3924 fn_80392480): a registration is found again by its address
    ("melee_runtime", "src/tu/hsd_3982.rs", r"fn_80392A3C_cb"),
    ("melee_modes", "src/tu/gm_1601.rs", r"fn_801652D8"),
    ("melee_modes", "src/tu/gmopening.rs", r"fn_801A9FCC"),
]
# crates whose src fns the C-oracle harnesses register (tests/**: freg)
REGISTERED_CRATES = ["melee_items", "melee_world", "melee_modes", "melee_runtime", "melee_fighter", "melee_chara"]
FN = (r'(?P<pre>(?:#\[[^\]]*\]\s*\n\s*|///[^\n]*\n\s*)*)(?P<indent>[ \t]*)pub unsafe extern "C" fn (?P<name>(?:{pat}))\b'
      r'\((?P<args>(?:[^()]|\([^()]*\))*)\)(?P<ret>[^{{;\n]*)\{{(?P<nl>\n?)')


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
        pub = m.start("indent") + len(m.group("indent"))
        indent = text[text.rfind("\n", 0, pub) + 1:pub]
        head = text[m.start():m.end()]
        if "#[inline(never)]" not in pre:
            k = pub - m.start()
            head = head[:k] + f"#[inline(never)]\n{indent}" + head[k:]
        rest_start = m.end()
        if m.group("nl"):
            line = f"{indent}    core::hint::black_box(0x{i:08X}u32); {MARK}\n"
            nxt = text[rest_start:rest_start + 200].split("\n", 1)[0]
            if MARK in nxt:
                rest_start += len(nxt) + 1
        else:  # one-line body: a block comment, the rest of the line is code
            line = f" core::hint::black_box(0x{i:08X}u32); /* fn identity */"
            if text.startswith(line, rest_start):
                rest_start += len(line)
        out.append(text[pos:m.start()] + head + line)
        pos = rest_start
        n += 1
    out.append(text[pos:])
    return "".join(out), n


def registered():
    """{src file: names}: the extern "C" fns a harness registers."""
    reg = set()
    for crate in REGISTERED_CRATES:
        for f in (RUST / crate / "tests").rglob("*.rs"):
            reg.update(m.split("::")[-1] for m in re.findall(r"\bfreg\(\s*([A-Za-z_][A-Za-z_0-9:]*)", f.read_text(encoding="utf-8", errors="replace")))
    srcs = {f: f.read_text(encoding="utf-8") for crate in REGISTERED_CRATES for f in sorted((RUST / crate / "src").rglob("*.rs"))}
    out = {}
    for f, s in srcs.items():
        names = {n for n in re.findall(r'pub unsafe extern "C" fn (\w+)\(', s) if n in reg}
        if names:
            out[f] = names
    return out


def main():
    check = "--check" in sys.argv
    ids, changed, total, deduped = {}, [], 0, 0
    pats = {}
    for crate, glob, pat in RULES:
        for f in sorted((RUST / crate).glob(glob)):
            pats.setdefault(f, []).append(pat)
    for f, names in registered().items():
        pats.setdefault(f, []).append("|".join(sorted(names)))
    for f in sorted(pats):
        crate = f.relative_to(RUST).parts[0]
        base = RUST / crate
        old = f.read_text(encoding="utf-8")
        new = old
        if f.name == "cb.rs" and f.parent.name == "chars":
            new, d = dedupe_cb(f, base / "src/tu")
            deduped += d
        mod = ".".join(f.relative_to(base / "src").with_suffix("").parts)
        new, n = mark(new, "|".join(pats[f]), f"{crate}::{mod}", ids)
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
