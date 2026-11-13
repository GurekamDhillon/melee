#!/bin/bash
# Sourced by the Envoy check scripts: pick the exe under test and say which build it is.
#   exe_pick NAME [DEFAULT_EXE_BASE]   sets $exe from EXE (when set) else DEFAULT (build/cargo/release/melee[.exe]);
#                                      exits 2 loudly when the file does not exist.
#   exe_banner NAME                    prints "NAME: exe <path> (build <where> <hash>)", read from the exe's embedded
#                                      window-title string ("melee [<where> <hash>]"; a dirty build adds a suffix after it); call at the start of each row.
exe_pick() {
  local who=$1 def=${2:-$repo/build/cargo/release/melee}
  exe="${EXE:-$def}"; [ -f "$exe" ] || exe="$exe.exe"
  [ -f "$exe" ] || { echo "$who: FAIL exe not found: ${EXE:-$def} (set EXE=path, or build it)" >&2; exit 2; }
}
exe_banner() {
  local id; id=$(grep -a -o 'melee \[[A-Za-z0-9_./-]* [0-9a-f]\{7,12\}' "$exe" 2>/dev/null | head -1 | cut -c8-)
  echo "$1: exe $exe (build ${id:-unknown})"
}
