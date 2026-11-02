#!/bin/bash
# build_trace.sh for a fresh private root WITHOUT rebuilding Aurora: reuse the libraries of an existing root that has `ax86m` built.
#   bash tools/rust-port/trace/build_trace_reuse.sh [SRC_ROOT] [NEW_ROOT]
# SRC_ROOT defaults to _build/agents/cov-trace (any root with ax86m/aurora_*.lib); NEW_ROOT to _build/agents/covfix-ct2.
# The stock build fails in a fresh root (Aurora's CMake needs vswhere and a matching source path). Here: copy the .lib files, junction the
# 3 GB `_deps` tree, and run build_trace.sh with a sitecustomize that turns the CMake step of tools/port/aurora_build.py into a no-op
# (the stamp is then written for this checkout). Needs build/GALE01/include/sysdolphin/baselib/*.inc (copy from <workspace>/melee/build).
# Afterwards: MELEE_CALLTRACE_SEQ=<file> MELEE_CALLTRACE_FRAMES=a-b bash tools/rust-port/golden/coverage/record_coverage.sh --force \
#   --bin <NEW_ROOT> NAME   (COVERAGE_AGENT=<run root name>), then seqdiff.py (notes/coverage.md "C call trace").
set -euo pipefail
ws="${GW_ROOT:-E:/Projects/Melee Workspace}"
src="${1:-$ws/_build/agents/cov-trace}"; dst="${2:-$ws/_build/agents/covfix-ct2}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$dst/ax86m/extern"
cp -n "$src"/ax86m/*.lib "$src"/ax86m/*.pdb "$dst/ax86m/" 2>/dev/null || true
cp -rn "$src"/ax86m/extern/*.lib "$dst/ax86m/extern/"
for d in _deps bin lib; do
  [ -e "$dst/ax86m/$d" ] || powershell -NoProfile -Command "New-Item -ItemType Junction -Path '$(cygpath -w "$dst/ax86m/$d")' -Target '$(cygpath -w "$src/ax86m/$d")' | Out-Null"
done
hook="$(mktemp -d)"
cat > "$hook/sitecustomize.py" <<'PY'
import subprocess
_orig = subprocess.run
def run(cmd, *a, **k):
    if isinstance(cmd, (list, tuple)) and any('build_aurora_windows' in str(c) for c in cmd):
        print("sitecustomize: aurora cmake build skipped (libs reused)", flush=True)
        class R: returncode = 0
        return R()
    return _orig(cmd, *a, **k)
subprocess.run = run
PY
PYTHONPATH="$hook" GW_BUILD_ROOT="$dst" bash "$here/build_trace.sh"
