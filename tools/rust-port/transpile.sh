#!/usr/bin/env bash
# Raw C -> Rust with c2rust, one TU at a time, into rust/<crate>/src/raw/<path>.rs
#
#   tools/rust-port/transpile.sh [--crate NAME] [-j N] <paths...>
#
# <paths>: worktree-relative .c files, directories or globs (quote globs). Directories and globs
# are filtered to the game TU list (docs/rust-port/game_tus.txt). The crate is picked from the path
# (see crate_for below) unless --crate is given. Output mirrors the C path with `src/` dropped:
#   src/melee/cm/camera.c -> rust/melee_world/src/raw/melee/cm/camera.rs
# Raw files are NOT compiled by `cargo build` (they are not `mod`-declared); they are input for the
# hand port. Per-TU logs go to rust/target/transpile-logs/ (gitignored); a summary is printed.
#
# Runs from Windows (Git Bash / PowerShell via `bash`) by re-executing itself inside WSL Debian.
# Requires tools/rust-port/setup_wsl.sh to have been run once.
set -uo pipefail
if [ "$(uname -s)" != Linux ]; then
  self=$(cd "$(dirname "$0")" && pwd -W 2>/dev/null || pwd)
  drv=$(echo "${self:0:1}" | tr A-Z a-z); self="/mnt/$drv${self:2}/transpile.sh"
  MSYS_NO_PATHCONV=1 exec wsl -d "${WSL_DISTRO:-Debian}" -- bash "$self" "$@"
fi
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
. "$HOME/opt/c2rust-env.sh" || { echo "run tools/rust-port/setup_wsl.sh first" >&2; exit 2; }
# Generated font .inc files (build/GALE01/include) are not in this worktree; read the main checkout's.
GALE=${GALE_INCLUDE:-}
for c in "$ROOT/build/GALE01/include" "$ROOT/../../melee/build/GALE01/include"; do
  [ -z "$GALE" ] && [ -d "$c/sysdolphin" ] && GALE=$(cd "$c" && pwd)
done
[ -n "$GALE" ] || { echo "no build/GALE01/include found; set GALE_INCLUDE" >&2; exit 2; }

CRATE=""; JOBS=$(nproc)
while [ $# -gt 0 ]; do case "$1" in
  --crate) CRATE=$2; shift 2;; -j) JOBS=$2; shift 2;; --) shift; break;; -*) echo "bad flag $1" >&2; exit 2;; *) break;;
esac; done
[ $# -gt 0 ] || { sed -n '2,15p' "$0"; exit 2; }

crate_for() {
  case "$1" in
    src/melee/ft/chara/ftCommon/*) echo melee_fighter;;
    src/melee/ft/chara/*) echo melee_chara;;
    src/melee/ft/*|src/melee/pl/*) echo melee_fighter;;
    src/melee/mp/*|src/melee/cm/*|src/melee/ef/*|src/melee/gr/*|src/melee/if/*) echo melee_world;;
    src/melee/it/*|src/melee/ty/*|src/melee/vi/*) echo melee_items;;
    src/melee/gm/*|src/melee/mn/*|pc/gameworld/script_game.c|pc/geno/*|pc/tests/*) echo melee_modes;;
    src/sysdolphin/*|src/melee/lb/*|src/melee/db/*|src/melee/sfx/*|src/MSL/*|libs/*|pc/gameworld/gekko_fp.c|pc/gameworld/mtx_pc.c) echo melee_runtime;;
    *) echo "";;
  esac
}

WORK=$(mktemp -d /tmp/rp-transpile.XXXXXX); trap 'rm -rf "$WORK"' EXIT
LOGS="$ROOT/rust/target/transpile-logs"; mkdir -p "$LOGS"
cd "$ROOT"
python3 tools/rust-port/gen_cc.py --root "$ROOT" --gale-include "$GALE" --out "$WORK/all.json" "$@" >/dev/null || exit 2
python3 tools/rust-port/gen_cc.py --root "$ROOT" --gale-include "$GALE" --out /dev/null --list "$@" > "$WORK/tus.txt" || exit 2

one() {  # $1 = TU path
  tu=$1; crate=${CRATE:-$(crate_for "$tu")}
  [ -n "$crate" ] || { echo "NOCRATE $tu"; return; }
  key=$(echo "$tu" | tr / _); d="$WORK/$key"; mkdir -p "$d/db"
  python3 -c 'import json,sys; db=json.load(open(sys.argv[1])); json.dump([e for e in db if e["file"]==sys.argv[2]], open(sys.argv[3],"w"))' \
    "$WORK/all.json" "$tu" "$d/db/compile_commands.json"
  log="$LOGS/$key.log"
  c2rust transpile "$d/db/compile_commands.json" -o "$d/out" --emit-modules --overwrite-existing > "$log" 2>&1
  rs=$(find "$d/out" -name "$(basename "${tu%.c}").rs" 2>/dev/null | head -1)
  if [ -n "$rs" ] && [ -s "$rs" ]; then
    rel=${tu#src/}; dst="$ROOT/rust/$crate/src/raw/${rel%.c}.rs"; mkdir -p "$(dirname "$dst")"
    { echo "// RAW c2rust output of $tu (c2rust $(c2rust --version | cut -d' ' -f2), native x86_64 -DTARGET_PC)."
      echo "// Not compiled. Regenerate: tools/rust-port/transpile.sh $tu"; cat "$rs"; } > "$dst"
    w=$(grep -c "warning" "$log"); echo "OK $tu -> rust/$crate/src/raw/${rel%.c}.rs ($(wc -l < "$dst") lines, $w warnings)"
  else
    echo "FAIL $tu ($(grep -m1 -E 'panicked|error' "$log" | cut -c1-160)) log: rust/target/transpile-logs/$key.log"
  fi
}
export -f one crate_for; export WORK LOGS ROOT CRATE
xargs -a "$WORK/tus.txt" -P "$JOBS" -I{} bash -c 'one "$@"' _ {} | tee "$WORK/res.txt"
echo "transpile: $(grep -c ^OK "$WORK/res.txt") ok, $(grep -c ^FAIL "$WORK/res.txt") failed, $(grep -c ^NOCRATE "$WORK/res.txt") without crate"
