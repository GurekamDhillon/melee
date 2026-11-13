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
# pad_for_disc FILE OUTDIR [SKIP]: echo the pad script to use. With ACE=1 (an m-ex disc such as ACE, which boots
# straight to the title: no memcard / opening scenes, the main menu about 200 frames sooner), a copy without the
# script's second Start (lines 3-4: the "200 idle, Start" that the vanilla boot needs) is written; a script that has
# no boot presses (SKIP=n: its first idle shortened by n frames instead).
pad_for_disc() {
  local f=$1 o=$2 skip=${3:-}
  if [ "${ACE:-0}" != 1 ]; then echo "$f"; return; fi
  local c="$o/$(basename "$f" .txt)_ace.txt"
  if [ -n "$skip" ]; then
    awk -v s="$skip" 'BEGIN{d=0} /^#/||NF==0{print;next} d==0{$1=$1-s; d=1} {print}' "$f" > "$c"
  else
    awk 'BEGIN{n=0} /^#/||NF==0{print;next} {n++; if(n==3||n==4) next; print}' "$f" > "$c"
  fi
  echo "$c"
}
