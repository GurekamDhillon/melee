#!/usr/bin/env bash
# Shared machine-wide build/test lock for agents. Up to N heavy processes at a time (N = contents of
# scratch/.build-slots, default 1). Slot 1 is always available; slots 2+ are only handed out while the
# machine has at least MIN_FREE_KB of free physical memory, so extra slots close by themselves under pressure.
#   buildlock.sh take <owner> [max_wait_seconds]   -> exit 0 when a slot is held by <owner>, 1 on timeout
#   buildlock.sh release <owner>                   -> frees the slot only if <owner> holds it
#   buildlock.sh status                            -> prints each slot's owner and age, or FREE
#   buildlock.sh run <owner> <command...>          -> take, run the command, always release; exits with the command's status
#   Light jobs: add --light before the owner (take|release|run --light <owner> ...) for cargo check, single-crate tests and
#   other small jobs. They use their own pool (scratch/.light-slots, default 2, lock dirs .cargo-build.lock.L1/.L2) so they
#   never wait behind a 40-minute capture, and only get a slot while at least 4 GB of memory is free. A light job does not
#   count against the heavy slots. `release` needs the same --light as the `take`.
#   One slot per lane: owner names that differ only by a "-b" / "-2" / "-b2" style suffix are one lane (genoboss and
#   genoboss-b). A lane that already holds a heavy slot is not given a second one: take returns 0 at once and shares the slot
#   (the second name's release is a no-op).
# Never remove the lock directories by hand. Never chain "mkdir ...; cmd; rm -rf ..." yourself.
set -u
S="${BUILDLOCK_DIR:-/e/Projects/Melee Workspace/scratch}"   # BUILDLOCK_DIR only for testing a copy
BASE="$S/.cargo-build.lock"
SLOTFILE="$S/.build-slots"
LOG="$S/buildslots.log"
MIN_FREE_KB=$((8 * 1024 * 1024))
LIGHTSLOTFILE="$S/.light-slots"
MIN_FREE_LIGHT_KB=$((4 * 1024 * 1024))
cmd="${1:-status}"; shift 2>/dev/null
light=0; [ "${1:-}" = "--light" ] && { light=1; shift; }
owner="${1:-}"
set -- "$cmd" "$owner" "${@:2}"   # keep the old positional layout below

nslots() { local n; n=$(cat "$SLOTFILE" 2>/dev/null | tr -dc '0-9'); [ -n "$n" ] && [ "$n" -ge 1 ] && [ "$n" -le 4 ] && echo "$n" || echo 1; }
slotdir() { if [ "$1" -eq 1 ]; then echo "$BASE"; else echo "$BASE.$1"; fi; }
holder() { cat "$(slotdir "$1")/owner" 2>/dev/null; }
memfree() { awk '/^MemFree:/{print $2}' /proc/meminfo 2>/dev/null || echo 0; }
note() { echo "$(date '+%F %T') $* memfree_mb=$(( $(memfree) / 1024 ))" >> "$LOG" 2>/dev/null; }

lane() { echo "$1" | sed -E 's/[-_](b|bb|[0-9]+|b[0-9]+)$//'; }
lnslots() { local n; n=$(cat "$LIGHTSLOTFILE" 2>/dev/null | tr -dc '0-9'); [ -n "$n" ] && [ "$n" -ge 1 ] && [ "$n" -le 4 ] && echo "$n" || echo 2; }
lslotdir() { echo "$BASE.L$1"; }
lholder() { cat "$(lslotdir "$1")/owner" 2>/dev/null; }
lmine() { local i; for i in 1 2 3 4; do [ "$(lholder $i)" = "$owner" ] && { echo $i; return 0; }; done; return 1; }
# a heavy slot held under another name of the same lane
lane_holds() { local i h; for i in 1 2 3 4; do h=$(holder $i); [ -n "$h" ] && [ "$h" != "$owner" ] && [ "$(lane "$h")" = "$(lane "$owner")" ] && { echo "$h"; return 0; }; done; return 1; }

take_light() {
  local max="${1:-7200}" waited=0 i n d
  while true; do
    lmine >/dev/null && return 0
    n=$(lnslots)
    if [ "$(memfree)" -ge "$MIN_FREE_LIGHT_KB" ]; then
      for i in $(seq 1 "$n"); do
        d=$(lslotdir "$i")
        if mkdir "$d" 2>/dev/null; then
          printf '%s
' "$owner" > "$d/owner"; date +%s > "$d/since"
          note "take light=$i owner=$owner waited=${waited}s"; return 0
        fi
      done
    fi
    if [ "$waited" -ge "$max" ]; then echo "buildlock: light job timed out waiting; held by $(lholder 1) $(lholder 2)" >&2; return 1; fi
    sleep 5; waited=$((waited+5))
  done
}

release_light() {
  local i d s
  if i=$(lmine); then
    d=$(lslotdir "$i"); s=$(cat "$d/since" 2>/dev/null || date +%s)
    rm -f "$d/owner" "$d/since"; rmdir "$d" 2>/dev/null
    note "release light=$i owner=$owner held=$(( $(date +%s) - s ))s"; return 0
  fi
  echo "buildlock: not released, '$owner' holds no light slot" >&2; return 1
}

mine() { local i; for i in 1 2 3 4; do [ "$(holder $i)" = "$owner" ] && { echo $i; return 0; }; done; return 1; }

take() {
  local max="${1:-7200}" waited=0 i n d o
  while true; do
    mine >/dev/null && return 0   # already ours
    if o=$(lane_holds); then note "share owner=$owner lane-holder=$o"; echo "buildlock: '$owner' shares the slot of '$o' (same lane)" >&2; return 0; fi
    n=$(nslots)
    for i in $(seq 1 "$n"); do
      if [ "$i" -gt 1 ] && [ "$(memfree)" -lt "$MIN_FREE_KB" ]; then break; fi
      d=$(slotdir "$i")
      if mkdir "$d" 2>/dev/null; then
        printf '%s\n' "$owner" > "$d/owner"
        date +%s > "$d/since"
        note "take slot=$i owner=$owner waited=${waited}s"
        return 0
      fi
    done
    if [ "$waited" -ge "$max" ]; then echo "buildlock: timed out waiting; held by $(holder 1) $(holder 2) $(holder 3)" >&2; return 1; fi
    sleep 10; waited=$((waited+10))
  done
}

release() {
  local i d
  if i=$(mine); then
    d=$(slotdir "$i"); s=$(cat "$d/since" 2>/dev/null || date +%s)
    rm -f "$d/owner" "$d/since"; rmdir "$d" 2>/dev/null
    note "release slot=$i owner=$owner held=$(( $(date +%s) - s ))s"
    return 0
  fi
  lane_holds >/dev/null && return 0   # we shared our lane's slot; its holder releases it
  [ -d "$BASE" ] || return 0
  echo "buildlock: not released, '$owner' holds no slot (slot 1 held by '$(holder 1)')" >&2; return 1
}

status() {
  local i d s any=0
  for i in 1 2 3 4; do
    d=$(slotdir "$i")
    if [ -d "$d" ]; then s=$(cat "$d/since" 2>/dev/null || echo 0); echo "slot $i held by $(holder $i) for $(( $(date +%s) - s ))s"; any=1; fi
  done
  [ "$any" -eq 1 ] || echo FREE
  for i in 1 2 3 4; do
    d=$(lslotdir "$i")
    if [ -d "$d" ]; then s=$(cat "$d/since" 2>/dev/null || echo 0); echo "light slot $i held by $(lholder $i) for $(( $(date +%s) - s ))s"; fi
  done
  echo "light slots allowed: $(lnslots) (need 4096 MB free)"
  echo "slots allowed: $(nslots), free memory: $(( $(memfree) / 1024 )) MB"
}

case "$cmd" in
  take)    [ -n "$owner" ] || { echo "owner required" >&2; exit 2; }; if [ $light = 1 ]; then take_light "${3:-7200}"; else take "${3:-7200}"; fi;;
  release) [ -n "$owner" ] || { echo "owner required" >&2; exit 2; }; if [ $light = 1 ]; then release_light; else release; fi;;
  status)  status;;
  run)     [ -n "$owner" ] || { echo "owner required" >&2; exit 2; }; shift 2
           if [ $light = 1 ]; then take_light 7200 || exit 1; else take 7200 || exit 1; fi
           "$@"; rc=$?
           if [ $light = 1 ]; then release_light; else release; fi; exit $rc;;
  *) echo "usage: buildlock.sh take|release|status|run [--light] <owner> ..." >&2; exit 2;;
esac
