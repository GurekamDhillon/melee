#!/usr/bin/env python3
"""Point chars::cb entries (src/chars/cb.rs) at native ports.

  python rust/melee_chara/gen/oracle/cbedit.py <cb fn name>=<rust path> [...]

e.g. ftyoshispecialn__fn_8012D128=crate::tu::ftyoshispecialn::fn_8012D128. The entry's body (a
`panic!(TODO...)` or an older call) is replaced by one call. cb.rs is CRLF in the working copy; its
line endings are kept.
"""
import re, sys
from pathlib import Path

CB = Path(__file__).resolve().parents[2] / "src/chars/cb.rs"


def main():
    s = CB.read_text(encoding="utf-8")  # universal newlines
    crlf = b"\r\n" in CB.read_bytes()
    for a in sys.argv[1:]:
        name, path = a.split("=", 1)
        pat = re.compile(r'(pub unsafe extern "C" fn ' + re.escape(name) + r'\(g: \*mut HSD_GObj\) \{\n)(.*?)(\n\})', re.S)
        m = pat.search(s)
        if not m:
            raise SystemExit(f"cbedit: no entry {name}")
        s = s[:m.start()] + m.group(1) + f"    {path}(g);" + m.group(3) + s[m.end():]
        print(f"cbedit: {name} -> {path}")
    CB.write_bytes((s.replace("\n", "\r\n") if crlf else s).encode("utf-8"))


if __name__ == "__main__":
    main()
