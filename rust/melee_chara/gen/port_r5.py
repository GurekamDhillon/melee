#!/usr/bin/env python3
"""Round-5 porting aid: port_r4.py's translator with more group-3 calls stubbed.

  python rust/melee_chara/gen/port_r5.py      (from the worktree root; Windows, LLVM clang, cargo)

Adds to the round-4 subset (see port_r4.py / port_r3.py / port_r2.py):
  - calls to 16 more group-3 functions, declared in crate::stubs as TODO(group3) hooks
    (NEW_STUBS below), three group-3 functions group 3 has already ported (8007E2F4, 8007E2FC, UpdateFacing),
    and three ft/inlines.h inlines written out in crate::stubs
    (Fighter_OnKnockbackEnter/Exit, Fighter_ClearCmdVars);
  - argument kinds: `c` an HSD_GObjEvent argument (a function name -> its chars::cb entry,
    NULL -> None), `h` / `b` an s16 / u8 constant (clang evaluates `(s16)(expr)` / `(u8)(expr)`);
  - calls into round-4 ports and this round's (fixpoint).
Output: src/chars/r5/<Char>.rs, gen/ported_r5.txt, gen/callbacks_r5.txt (new callback keys), and
src/chars/cb.rs rewritten over rounds 3-5. Verified by gen/chref.py. Computed from C source only.
"""
import re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import port_r2 as r2  # noqa: E402
import port_r3 as r3  # noqa: E402
import port_r4 as r4  # noqa: E402
from port_r2 import Skip  # noqa: E402

ROOT, CRATE = r2.ROOT, r2.CRATE
BUILD = ROOT / "rust/target/port_r5"
OUT = CRATE / "src/chars/r5"

# name -> (rust fn in crate::stubs, kinds, ret)
NEW_STUBS = {
    "ft_80082C74": ("ft_80082C74", "gc", "v"), "ft_8008403C": ("ft_8008403C", "gc", "v"),
    "ft_800841B8": ("ft_800841B8", "gc", "v"), "ft_80084104": ("ft_80084104", "g", "v"),
    "ftParts_80074A4C": ("ftParts_80074A4C", "gii", "v"), "ftParts_80074B0C": ("ftParts_80074B0C", "gii", "v"),
    "ft_CheckGroundAndLedge": ("ft_CheckGroundAndLedge", "gi", "i"), "ftCommon_8007DB24": ("ftCommon_8007DB24", "g", "v"),
    "ftCommon_8007D92C": ("ftCommon_8007D92C", "g", "v"), "ftCo_Wait_IASA": ("ftCo_Wait_IASA", "g", "v"),
    "ftCommon_8007EBAC": ("ftCommon_8007EBAC", "Fuu", "v"),
    "ftAnim_SetAnimRate": ("ftAnim_SetAnimRate", "gf", "v"),
    "ftAnim_800704F0": ("ftAnim_800704F0", "gif", "v"),
    "ftCo_LandingFallSpecial_Enter": ("ftCo_LandingFallSpecial_Enter", "gif", "v"),
    "Fighter_UnkSetFlag_8006CFBC": ("Fighter_UnkSetFlag_8006CFBC", "g", "v"),
    "ftCo_800957F4": ("ftCo_800957F4", "gi", "v"),
}
NEW_HELPERS = {
    # group 3 has ported these (melee_fighter::ftcommon / ftcommon2); the oracle runs the C cut
    "ftCommon_8007E2F4": ("ftCommon_8007E2F4", "Fh", "v"), "ftCommon_8007E2FC": ("ftCommon_8007E2FC", "g", "v"),
    "ftCommon_UpdateFacing": ("ftCommon_UpdateFacing", "F", "v"),
    "Fighter_OnKnockbackEnter": ("Fighter_OnKnockbackEnter", "gi", "v"),
    "Fighter_OnKnockbackExit": ("Fighter_OnKnockbackExit", "gi", "v"),
    "Fighter_ClearCmdVars": ("Fighter_ClearCmdVars", "F", "v"),
}
NEW = {**NEW_STUBS, **NEW_HELPERS}
r2.CALLS.update(NEW)  # Fn.call reads the module dict


def known_calls(extra):
    calls = r4.known_calls(set())
    for tu, name in r3.read_list("ported_r4.txt"):
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r4::{d}::{stem}::{name}(gobj)", "v", False)
    for (tu, name) in extra:
        d, stem = Path(tu).parent.name, Path(tu).stem
        calls[(tu, name)] = (f"crate::chars::r5::{d}::{stem}::{name}(gobj)", "v", False)
    return calls


class Fn5(r4.Fn4):
    def arg(self, a, k):
        a = a.strip()
        if k == "c":
            if a in ("NULL", "0", "nullptr"):
                return "None"
            a = re.sub(r"^&\s*", "", a)
            if not re.fullmatch(r"[A-Za-z_]\w*", a):
                raise Skip("callback arg")
            key = r3.def_tu(self.tu.path, a)
            self.cbs.add((self.tu.path, key, a))
            return f"Some(crate::chars::cb::{r3.cb_name(key, a)})"
        if k in "hb":
            if re.search(r"\b(fp|gobj)\b|->", a):
                raise Skip("non-constant narrow arg")
            cty, rty = {"h": ("s16", "i16"), "b": ("u8", "u8")}[k]
            return f"({self.tu.const(f'({cty}) ({a})', 'i')} as {rty})"
        return super().arg(a, k)

    def call(self, text):
        m = re.fullmatch(r"(\w+)\s*\((.*)\)", text.strip(), re.S)
        if m and m.group(1) in NEW:
            self.used_port = True
        return super().call(text)


def scan(drop, extra):
    ported = set()
    for lst in ("ported.txt", "ported_r2.txt", "ported_r3.txt", "ported_r4.txt"):
        ported |= {n for _, n in r3.read_list(lst)}
    Fn5.CALLS3 = known_calls(extra)
    tus = {}
    for f in sorted((ROOT / "src/melee/ft/kinds").glob("*/*.c")):
        if f.parent.name == "ftCommon":
            continue
        src = f.read_text(encoding="utf-8", errors="replace")
        rel = f.relative_to(ROOT).as_posix()
        tu = r4.TU4(rel)
        fns = []
        for m in re.finditer(r"^(static\s+)?(?:inline\s+)?void\s+(\w+)\s*\(\s*(?:HSD_GObj|Fighter_GObj)\s*\*\s*gobj\s*\)\s*\{", src, re.M):
            name = m.group(2)
            if name in ported or name in drop:
                continue
            i, d = m.end(), 1
            while d and i < len(src):
                d += src[i] == "{"; d -= src[i] == "}"; i += 1
            try:
                body = r4.normalise(r2.strip(src[m.end():i - 1]), tu)
                fn = Fn5(tu, name, body)
                fn.cbs = set()
                fn.static = bool(m.group(1))
                fn.used_port = fn.assigned = False
                rest = fn.block(body, "    ")
                if rest:
                    raise Skip("trailing")
                if not (fn.used_port or fn.assigned):
                    raise Skip("nothing new")
                fns.append(fn)
            except Skip:
                continue
        if fns:
            tus[rel] = (tu, fns)
    return tus


def write_cb(r5list, r5cbs):
    """cb.rs over rounds 3-5: port_r4.write_cb for 3+4 (+5's keys), then entries whose target
    is a round-5 port call it. callbacks_r3/r4.txt are kept; new keys go to callbacks_r5.txt."""
    r3cbs, r4cbs = r4.read_callbacks("callbacks_r3.txt"), r4.read_callbacks("callbacks_r4.txt")
    keep4 = (HERE / "callbacks_r4.txt").read_bytes()
    r4.write_cb([tuple(x) for x in r3.read_list("ported_r4.txt")], r4cbs | r5cbs)
    (HERE / "callbacks_r4.txt").write_bytes(keep4)
    rs = (CRATE / "src/chars/cb.rs").read_text(encoding="utf-8")
    for tu, name in r5list:
        d, stem = Path(tu).parent.name, Path(tu).stem
        key = r3.cb_name(tu, name)
        rs = re.sub(r"(fn %s\(g: \*mut HSD_GObj\) \{\n    )panic!\([^\n]*\);" % re.escape(key),
                    lambda m: m.group(1) + f"crate::chars::r5::{d}::{stem}::{name}(g);", rs)
    rs = rs.replace("gen/port_r3.py + gen/port_r4.py", "gen/port_r3.py + gen/port_r4.py + gen/port_r5.py", 1)
    (CRATE / "src/chars/cb.rs").write_text(rs, encoding="utf-8", newline="\n")
    out = sorted(" ".join(c) for c in r5cbs - r3cbs - r4cbs)
    (HERE / "callbacks_r5.txt").write_text(
        "# <setter tu> <defining tu | ext> <callback> stored or passed by round-5 ports (new ones only): gen/port_r5.py\n"
        + "".join(l + "\n" for l in out), encoding="utf-8", newline="\n")


def emit(tus):
    by_char = {}
    for rel, (tu, fns) in sorted(tus.items()):
        by_char.setdefault(Path(rel).parent.name, []).append((rel, tu, fns))
    if OUT.exists():
        for p in OUT.glob("*.rs"):
            p.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    listing = ["# Round-5 character functions (gen/port_r5.py output): <tu> <function>, checked by gen/chref.py"]
    linemap, dropped = {}, set()
    r3.CALLBACKS.clear()
    cbs = set()
    for ch, items in sorted(by_char.items()):
        rs = [f"//! @generated by rust/melee_chara/gen/port_r5.py: round-5 callbacks of {ch}.",
              "//! Do not edit by hand: change the translator or move a function to a hand-written module.",
              "#![allow(non_snake_case, unused_unsafe, unused_variables, unused_parens, unused_comparisons, clippy::all)]", ""]
        for rel, tu, fns in items:
            body = []
            for fn in fns:
                before = set(r3.CALLBACKS)
                try:
                    out = []
                    for l in fn.lines:
                        l = re.sub(r"@C(\d+)@", lambda m: r2.rust_const(tu, int(m.group(1))), l)
                        l = re.sub(r"@K(\d+)@", lambda m: r4.rust_cmp(tu, int(m.group(1))), l)
                        m = re.search(r"@A(\d+)@", l)
                        if m:
                            l = l[:m.start()] + r4.rust_assign(tu, int(m.group(1)))
                        l = l.replace("st::ftCommon_GroundToAirStateChange(gobj, fp,", "st::ftCommon_GroundToAirStateChange(gobj, &mut *fp,")
                        l = l.replace("st::ftCommon_AirToGroundStateChange(gobj, fp,", "st::ftCommon_AirToGroundStateChange(gobj, &mut *fp,")
                        out.append(r4.da_rs(tu, l))
                except Skip:
                    dropped.add(fn.name)
                    r3.CALLBACKS.clear(); r3.CALLBACKS.update(before)
                    continue
                cbs |= fn.cbs
                body.append((fn, out))
            if not body:
                continue
            stem = Path(rel).stem
            rs.append(f"pub mod {stem} {{")
            rs += ["    #[allow(unused_imports)]", "    use crate::stubs as st;", "    #[allow(unused_imports)]", "    use melee_types::{Fighter, HSD_GObj};", ""]
            for fn, out in body:
                rs.append(f"    /// `{fn.name}`{' (static)' if fn.static else ''} ({rel}).")
                rs.append(f"    pub fn {fn.name}(gobj: *mut HSD_GObj) {{")
                rs.append("        unsafe {")
                for l in out:
                    rs.append("        " + l)
                    linemap[(ch, len(rs))] = fn.name
                rs.append("        }")
                rs += ["    }", ""]
                listing.append(f"{rel} {fn.name}")
            rs += ["}", ""]
        (OUT / f"{ch}.rs").write_text("\n".join(rs), encoding="utf-8", newline="\n")
    mods = ["//! @generated by gen/port_r5.py: round-5 translated character callbacks, one file per character.",
            *[f"pub mod {p.stem};" for p in sorted(OUT.glob("*.rs")) if p.stem != "mod"], ""]
    (OUT / "mod.rs").write_text("\n".join(mods), encoding="utf-8", newline="\n")
    (HERE / "ported_r5.txt").write_text("\n".join(listing) + "\n", encoding="utf-8", newline="\n")
    write_cb([l.split() for l in listing[1:]], set(r3.CALLBACKS) | cbs)
    return linemap, dropped, len(listing) - 1


def cargo_errors(linemap):
    r = subprocess.run(["cargo", "build", "-p", "melee_chara", "--message-format=short"],
                       cwd=ROOT / "rust", capture_output=True, text=True)
    bad = set()
    for m in re.finditer(r"src[\\/]chars[\\/]r5[\\/](\w+)\.rs:(\d+):\d+: (?:error|warning: unreachable)", r.stderr):
        ln = int(m.group(2))
        name = linemap.get((m.group(1), ln))
        if name is None:
            cands = [n for (c, l), n in linemap.items() if c == m.group(1) and l >= ln]
            name = cands[0] if cands else None
        if name:
            bad.add(name)
    if r.returncode and not bad:
        sys.stderr.write(r.stderr[-4000:])
        raise SystemExit("cargo build failed outside round-5 functions")
    return bad


def main():
    drop, extra = set(), set()
    while True:
        tus = scan(drop, extra)
        names = {(rel, fn.name) for rel, (tu, fns) in tus.items() for fn in fns}
        if names - extra:
            extra |= names
            continue
        r2.BUILD = r3.BUILD = r4.BUILD = BUILD
        r2.eval_consts(tus)
        r4.probe_assigns(tus)
        r4.probe_cmps(tus)
        linemap, dropped, n = emit(tus)
        bad = dropped | cargo_errors(linemap)
        if not bad:
            break
        print(f"port_r5: dropping {len(bad)}: {sorted(bad)[:8]}...", flush=True)
        drop |= bad
        extra = {k for k in extra if k[1] not in drop}
    print(f"port_r5: {n} functions in {len(tus)} TUs")


if __name__ == "__main__":
    main()
