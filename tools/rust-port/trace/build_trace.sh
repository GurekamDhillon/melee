#!/bin/bash
# Build the call-tracing melee-pc.exe in a PRIVATE build root (default _build/agents/trace):
# every game TU goes through gwtool_trace.sh (entry hook in every function) and the link list gets
# the tracer shim pc/platform/gw_calltrace.c. The normal build is untouched.
#   bash tools/rust-port/trace/build_trace.sh
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -W)"
export GW_ROOT="${GW_ROOT:-E:/Projects/Melee Workspace}"
export GW_MELEE="${GW_MELEE:-$(cd "$here/../../.." && pwd -W)}"
export GW_BUILD_ROOT="${GW_BUILD_ROOT:-$GW_ROOT/_build/agents/trace}"
export GW_GWTOOL="$here/gwtool_trace.sh"
export GW_LINK_OBJECTS="$GW_BUILD_ROOT/melee_link_objects_trace.rsp"
mkdir -p "$GW_BUILD_ROOT/masstest/out" "$GW_BUILD_ROOT/masstest/shimobj" "$GW_BUILD_ROOT/runs"
# curated baseline list rewritten for this root, plus the tracer shim
python - <<PY
import sys
sys.path.insert(0, r"$GW_ROOT/tools/port")
import link_response
from pathlib import Path
t = link_response.response_text(Path(r"$GW_ROOT"), Path(r"$GW_BUILD_ROOT").resolve())
t += '"' + (Path(r"$GW_BUILD_ROOT").resolve() / "masstest/shimobj/gw_calltrace.obj").as_posix() + '"\n'
Path(r"$GW_LINK_OBJECTS").write_text(t)
PY
cd "$GW_MELEE"
exec bash "$GW_ROOT/tools/port/build.sh" "$@"
