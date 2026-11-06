#!/usr/bin/env python3
"""Console arithmetic lint for the game crates (docs/rust-port/notes/semantics.md). Run by gate.sh.

  python tools/rust-port/semantics_lint.py            exit 1 on any violation not in semantics_allow.tsv
  python tools/rust-port/semantics_lint.py --list     print every violation (allowed ones marked)
  python tools/rust-port/semantics_lint.py --force    ignore the up-to-date stamp

Rules (reported as `rule<TAB>file:line<TAB>detail`):
  M1 disallowed method   clippy::disallowed_methods with rust/clippy.toml: f32/f64::mul_add, host libm (sqrt, sin, cos,
                         atan2, powf, exp, ln, ...), float min/max/clamp/round/rem_euclid. Use melee_core::fp /
                         the melee_runtime MSL ports / the C comparison.
  I1 int div/rem         `/` or `%` on integers with a non-literal (or 0 / -1) divisor: melee_core::ppc::Ppc::pdiv/prem
  I2 int shift           `<<` / `>>` with a non-literal amount: Ppc::pshl/pshr (slw/srw/sraw, low 6 bits of rB)
  I3 float -> int        any `as` cast from f32/f64 to an integer: ppc::fctiwz_f32 / fctiwz / fcvt_u32 (all widths,
                         u32 included, so every conversion site is explicit)
I1-I3 come from the crates' MIR (exact spans, literal operands already folded), via int_audit.mir_sites; M1 from
clippy's JSON output. Both run on the nightly toolchain (SEMLINT_TOOLCHAIN, default nightly-2026-02-14; clippy and
-Zunpretty=mir) in their own target subdirectory ($CARGO_TARGET_DIR/semlint), so they never disturb the stable build.
Scope: every file of CRATES below except raw/, c2r/, tests/, melee_core/src/{ppc,fp}.rs (the helpers), and the
host files listed in semantics_allow.tsv (`<rule or *>\t<path glob>\t<reason>`; one row per intentional exception).
A stamp (hash of the scoped sources, clippy.toml, this file and the allow-list) skips an unchanged tree."""
import argparse, fnmatch, hashlib, json, os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
RUST = REPO / "rust"
sys.path.insert(0, str(HERE))
import int_audit  # noqa: E402

CRATES = ["melee_core", "melee_types", "melee_runtime", "melee_world", "melee_fighter", "melee_chara", "melee_items",
          "melee_modes"]
HELPERS = {"rust/melee_core/src/ppc.rs", "rust/melee_core/src/fp.rs"}
ALLOW = HERE / "semantics_allow.tsv"


def rel(f):
    f = f.replace("\\", "/")
    i = f.find("melee")
    # spans are relative to rust/ (cargo cwd) or absolute
    p = Path(f) if os.path.isabs(f) else RUST / f
    try:
        return p.resolve().relative_to(REPO).as_posix()
    except ValueError:
        return f[i:] if i >= 0 else f


def in_scope(r):
    if not r.startswith("rust/"):
        return False
    parts = r.split("/")
    if len(parts) < 3 or parts[1] not in CRATES or parts[2] != "src":
        return False
    if any(x in ("raw", "c2r", "tests") for x in parts) or r in HELPERS:
        return False
    return True


def load_allow():
    rows = []
    if ALLOW.exists():
        for line in ALLOW.read_text().splitlines():
            if not line.strip() or line.startswith("#"):
                continue
            rule, glob, *_ = line.split("\t") + [""]
            rows.append((rule.strip(), glob.strip()))
    return rows


_LINES = {}


def marked(path, line):
    """`// semantics: allow(...)` on the flagged line (or the line above it)."""
    if path not in _LINES:
        try:
            _LINES[path] = (REPO / path).read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            _LINES[path] = []
    L = _LINES[path]
    return any(0 <= k < len(L) and "semantics: allow" in L[k] for k in (line - 1, line - 2))


def allowed(rule, path, allow, line=0):
    if line and marked(path, line):
        return True
    return any((r == "*" or r == rule) and fnmatch.fnmatch(path, g) for r, g in allow)


def stamp():
    h = hashlib.sha256()
    for f in [HERE / "semantics_lint.py", ALLOW, RUST / "clippy.toml", HERE / "int_audit.py"]:
        h.update(f.read_bytes() if f.exists() else b"-")
    for c in CRATES:
        for p in sorted((RUST / c).rglob("*.rs")):
            if "target" in p.parts:
                continue
            h.update(str(p.relative_to(RUST)).encode()); h.update(p.read_bytes())
    return h.hexdigest()


def env():
    e = dict(os.environ)
    td = Path(e.get("CARGO_TARGET_DIR", str(REPO / "build/cargo")))
    e["CARGO_TARGET_DIR"] = str(td / "semlint")
    e["RUSTC_BOOTSTRAP"] = "1"
    e.setdefault("CARGO_INCREMENTAL", "0")
    e.setdefault("CARGO_PROFILE_DEV_DEBUG", "0")
    return e, td


def tc():
    return "+" + os.environ.get("SEMLINT_TOOLCHAIN", "nightly-2026-02-14")


def clippy_sites(e):
    cmd = ["cargo", tc(), "clippy", "--message-format=json", "--profile", "fast"]
    for c in CRATES:
        cmd += ["-p", c]
    cmd += ["--lib", "--", "-A", "clippy::all", "-W", "clippy::disallowed_methods"]
    p = subprocess.run(cmd, cwd=RUST, env=e, capture_output=True, text=True, errors="replace")
    out = []
    for line in p.stdout.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        if (msg.get("code") or {}).get("code") != "clippy::disallowed_methods":
            continue
        sp = next((s for s in msg["spans"] if s["is_primary"]), None)
        if sp:
            out.append(("M1", rel(sp["file_name"]), sp["line_start"], msg["message"].split("\n")[0]))
    if p.returncode != 0 and not out:
        sys.stderr.write(p.stderr[-4000:])
        raise SystemExit("semantics_lint: clippy failed")
    return out


def mir_sites(e):
    out = []
    e = dict(e)
    e.pop("RUSTC_WRAPPER", None)  # sccache cannot cache an -Zunpretty run (no rlib to zip)
    for c in CRATES:
        cmd = ["cargo", tc(), "rustc", "-p", c, "--lib", "--profile", "fast", "--", "-Zunpretty=mir",
               "-Zmir-include-spans=on",
               "-Zinline-mir=no"]
        p = subprocess.run(cmd, cwd=RUST, env=e, capture_output=True, text=True, errors="replace")
        if p.returncode != 0:
            sys.stderr.write(p.stderr[-4000:])
            raise SystemExit(f"semantics_lint: MIR dump of {c} failed")
        for fn, f, l1, c1, l2, c2, cls, detail in mir_all(p.stdout):
            rule = {"div": "I1", "rem": "I1", "shl": "I2", "shr": "I2", "fptoi": "I3"}[cls]
            out.append((rule, rel(f), l1, f"{cls} {detail} in {fn}"))
            RAW.append((c, fn, rel(f), l1, c1, l2, c2, cls, detail))
    return out


RAW = []  # int_audit rmir rows (with spans) for --fix
U32CAST = re.compile(r"\s+(_\d+|\(.*\)) = (.*) as (u32|u64|usize) \(FloatToInt\);\s*$")


def mir_all(text):
    """int_audit.mir_sites plus the float -> u32/u64/usize casts it leaves out. Const / static item bodies are dropped:
    they are evaluated at compile time (a shift there has a constant amount; an out-of-range one is a compile error)."""
    keep, skip = [], False
    for line in text.splitlines():
        if not skip and (line.startswith("const ") or line.startswith("static ") or line.startswith("promoted[")):
            skip = True
        if not skip:
            keep.append(line)
        elif line == "}":
            skip = False
    text = chr(10).join(keep)
    rows = list(int_audit.mir_sites(text))
    for line in text.splitlines():
        sm = int_audit.SPAN.search(line)
        if not sm:
            continue
        m = U32CAST.match(line.split("//")[0])
        if m:
            f = sm.group(1).replace("\\", "/")
            ts, _ = int_audit.operand_type(m.group(2), {})
            rows.append(("?", f, int(sm.group(2)), int(sm.group(3)), int(sm.group(4)), int(sm.group(5)), "fptoi",
                         f"{ts if ts in ('f32', 'f64') else 'f32'}->{m.group(3)}"))
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--fix", action="store_true", help="rewrite the I1-I3 violations with int_audit's fixer")
    a = ap.parse_args()
    e, td = env()
    st = td / "semlint.stamp"
    s = stamp()
    if not a.force and not a.list and st.exists() and st.read_text() == s:
        print("ok   semantics lint: sources unchanged since the last clean pass")
        return 0
    allow = load_allow()
    sites = sorted(set(clippy_sites(e) + mir_sites(e)))
    bad = 0
    counts = {}
    for rule, f, line, d in sites:
        if not in_scope(f):
            continue
        ok = allowed(rule, f, allow, line)
        counts[(rule, ok)] = counts.get((rule, ok), 0) + 1
        if not ok:
            bad += 1
        if a.list or not ok:
            print(f"{rule}\t{f}:{line}\t{d}{'  [allowed]' if ok else ''}")
    if a.fix:
        import csv, tempfile, types
        keep = [r for r in RAW if in_scope(r[2]) and not allowed({"div": "I1", "rem": "I1", "shl": "I2", "shr": "I2",
                                                                  "fptoi": "I3"}[r[7]], r[2], allow, r[3])]
        with tempfile.NamedTemporaryFile("w", suffix=".csv", delete=False, newline="") as fo:
            w = csv.writer(fo)
            w.writerow(["crate", "func", "file", "l1", "c1", "l2", "c2", "class", "detail", "scope"])
            w.writerows([*r, "game"] for r in sorted(set(keep)))
        int_audit.fix(types.SimpleNamespace(csv=fo.name, dry=False))
        return 0
    summary = ", ".join(f"{r}{'' if not ok else ' allowed'} {n}" for (r, ok), n in sorted(counts.items())) or "no sites"
    if bad:
        print(f"FAIL semantics lint: {bad} violation(s) ({summary})")
        return 1
    td.mkdir(parents=True, exist_ok=True)
    st.write_text(s)
    print(f"ok   semantics lint: 0 violations ({summary})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
