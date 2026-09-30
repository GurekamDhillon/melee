#!/bin/bash
set -euo pipefail
: "${GW_ROOT:?set GW_ROOT to the gdm checkout}"
exec bash "$GW_ROOT/tools/port/pipe_linux.sh" "$@"
