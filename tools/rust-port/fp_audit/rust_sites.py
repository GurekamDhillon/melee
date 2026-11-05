#!/usr/bin/env python3
"""Index the Rust port: every `fn NAME` in rust/*/src with its file:line and the fused-op calls in its body
(fmadds / fmsubs / fnmsubs / fnmadds / fmsubs_neg / mul_add / fma helpers). Prints CSV
  func,file,line,fused
  python tools/rust-port/fp_audit/rust_sites.py OUT.csv
"""
import csv, re, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FUSED = re.compile(r"\b(fmadds|fmsubs|fnmsubs|fnmadds|fmsubs_neg|fmadd|fmsub|fnmsub|fnmadd|mul_add|fma|fmuladd\w*)\s*\(")
FN = re.compile(r"^\s*(?:pub(?:\([\w:]+\))?\s+)?(?:unsafe\s+)?(?:extern\s+\"C\"\s+)?fn\s+(\w+)")


def index():
    for p in sorted((ROOT / "rust").glob("*/src/**/*.rs")):
        lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        i = 0
        while i < len(lines):
            m = FN.match(lines[i])
            if not m:
                i += 1; continue
            depth, j, body, started = 0, i, [], False
            while j < len(lines):
                l = re.sub(r"//.*", "", lines[j])
                body.append(l)
                depth += l.count("{") - l.count("}")
                if "{" in l: started = True
                if started and depth <= 0: break
                if not started and l.rstrip().endswith(";"): break
                j += 1
            yield m.group(1), str(p.relative_to(ROOT)).replace("\\", "/"), i + 1, len(FUSED.findall("\n".join(body[1:] if len(body) > 1 else body)))
            i += 1


def main():
    with open(sys.argv[1], "w", newline="") as f:
        w = csv.writer(f); w.writerow(["func", "file", "line", "fused"])
        w.writerows(index())


if __name__ == "__main__":
    main()
