"""Extract Geno's production JSON reader for the controls standalone test."""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[2]
source = (root / "pc/platform/geno_registry.c").read_text()
start = source.index("enum { JN_NULL")
end = source.index("/* ---- hashing")
out = Path(sys.argv[1]).resolve()
out.write_text(source[start:end])
