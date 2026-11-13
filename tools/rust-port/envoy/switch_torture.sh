#!/bin/bash
# Envoy stage-switch torture (the owner's "died falling through the stage / warped onto the blast zone at a switch"):
# (log line counts are per log line; golden logs each sim line twice) the envoy_sim test knob `torture` forces a stage switch every 240 live frames, walking every ordered pair of the
# rotation (36 BF, 10, 28, 12, 37), while CPU fighters on both sides play through the awkward states (grounded,
# side platforms, high and low in the air, offstage, on the ledge, in hitstun, grabs). Per run it checks:
#   * no KO is caused by a switch: no fighter enters a KO (motion 0-10) and no stock is lost within 120 frames of a
#     switch unless it was hit or grabbed first after the switch (that one is a combat KO, counted, allowed; placed
#     fighters are intangible for 60 frames);
#   * 45 frames after each switch every fighter in play is inside the stage's box (|x| < 200, y > -5 below the main
#     floor of every stage in the rotation, y < 180) and not in a KO;
#   * the states fighters were in at the switches (the coverage table).
#   GW_ISO_VANILLA=... bash tools/rust-port/envoy/switch_torture.sh
#   OUT=dir; FRAMES (default 14400); FIGHTS (start fights, default "1 2 3c 5"; a trailing c = co-op);
#   OLD=1 runs the pre-fix placement (MELEE_ENVOY_SWITCH_OLD=1), which must FAIL;
#   RUN_TIMEOUT=secs per fight (default FRAMES/2+600; a depth-7 14400-frame run under load takes 70-85 min): the game's own watchdog, plus an outer kill 30 s later.
#   DEPTH=4 / 7 runs it as a synctest (0 mismatches required; log lines then repeat once per rollback pass).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"; cd "$repo"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
if [ -z "${GW_ISO_VANILLA:-}" ] && [ -f "$ws/.env" ]; then set -a; . "$ws/.env"; set +a; fi
[ -n "${GW_ISO_VANILLA:-}" ] || { echo "switch_torture: GW_ISO_VANILLA not set"; exit 2; }
command -v python >/dev/null 2>&1 || python() { python3 "$@"; }
exe="${EXE:-$repo/build/cargo/release/melee}"; [ -e "$exe" ] || exe="$exe.exe"
out="${OUT:-${TEMP:-/tmp}/switch_torture}"; mkdir -p "$out"
frames="${FRAMES:-14400}"; tmo="${RUN_TIMEOUT:-$((frames / 2 + 600))}"; fights="${FIGHTS:-1 2 3c 5}"; old="${OLD:-0}"; depth="${DEPTH:-0}"
mods="$repo/build/mods-torture"; rm -rf "$mods"; mkdir -p "$mods"
cp "$repo/build/mods/envoy_sim.wasm" "$mods/" || { echo "switch_torture: build the mods first"; exit 2; }
fail=0
for f in $fights; do
  tag="f$f$([ "$old" = 1 ] && echo -old)$([ "$depth" != 0 ] && echo "-d$depth")"
  coop=(); case "$f" in *c) coop=(MELEE_ENVOY_COOP=1 MELEE_PAD_CHANNELS=3); f=${f%c} ;; esac
  if [ "$depth" = 0 ]; then mode=(golden --out "$out/$tag.csv"); else mode=(synctest --depth "$depth" --timeout 3000); fi
  env "${coop[@]}" MELEE_VOLUME=3 MELEE_FULLSCREEN=off MELEE_WASM_MODS="$mods" MELEE_ENVOY_ARM=1 MELEE_ENVOY_TEST_P1CPU=1 \
    MELEE_ENVOY_START_FIGHT="$f" MELEE_ENVOY_TEST_DRIVES="torture" MELEE_ENVOY_SWITCH_OLD="$old" \
    MELEE_PAD_SCRIPT="$here/drives_pad.txt" MELEE_WINDOW_TITLE="switch torture $tag d$depth (${BRANCH_TAG:-envoy})" \
    timeout -k 15 $((tmo + 30)) "$exe" synctest --engine tu --iso "$GW_ISO_VANILLA" --scenario pad_falcon_fox_fd \
    --depth "$depth" --frames "$frames" --timeout "$tmo" >"$out/$tag.log" 2>&1
  rc=$?; if [ $rc = 124 ] || [ $rc = 137 ]; then echo "melee: switch_torture killed the run after $((tmo + 30)) s (hung)" >>"$out/$tag.log"; fi
  python - "$out/$tag.log" "$tag" <<'PY' || fail=1
import re, sys, collections
log, tag = sys.argv[1], sys.argv[2]
L = open(log, encoding='utf-8', errors='replace').read().splitlines()
sync = next((l for l in L if 'engine synctest' in l), '')
sw = [l for l in L if 'envoy: stage transition' in l]
kos = [l for l in L if 'KO right after a stage switch' in l or 'stock lost soon after a stage switch' in l]
combat = sum('combat KO after a stage switch' in l for l in L)
pat = re.compile(r'envoy: torture (at|after) switch s(\d) (motion|x10|y10)\D*?(-?\d+)\s*$')
snap = collections.defaultdict(dict); n = {'at': 0, 'after': 0}; last = None
for l in L:
    m = pat.search(l)
    if not m: continue
    when, s, k, v = m.group(1), int(m.group(2)), m.group(3), int(m.group(4))
    if when == 'at' and s == 0 and k == 'motion': n['at'] += 1
    if when == 'after' and k == 'motion' and (s, n['after']) != last: pass
    key = (when, n['at'], s)
    snap[key][k] = v
def cls(d):
    m, x, y = d.get('motion', -1), d.get('x10', 0) / 10, d.get('y10', 0) / 10
    if 0 <= m <= 10: return 'dying'
    if 11 <= m <= 13: return 'respawn'
    if 252 <= m <= 263: return 'ledge'
    if 75 <= m <= 91 or 156 <= m <= 157 or 183 <= m <= 199: return 'hitstun/down'
    if 211 <= m <= 251: return 'grab/throw'
    if abs(x) > 90: return 'offstage'
    if y > 60: return 'air high'
    if y < -5: return 'air low'
    if 20 < y <= 60: return 'platform/air'
    return 'centre'
cov = collections.Counter(cls(d) for (w, i, s), d in snap.items() if w == 'at')
bad = []
for (w, i, s), d in sorted(snap.items()):
    if w != 'after' or 'motion' not in d: continue
    x, y, m = d.get('x10', 0) / 10, d.get('y10', 0) / 10, d['motion']
    if 0 <= m <= 10 or abs(x) >= 200 or y <= -5 or y >= 180:
        bad.append(f'switch {i} slot {s}: motion {m} at ({x:.1f},{y:.1f})')
# every run is a synctest (depth 0 too): no result line means it never finished (a stall, a crash, the timeout)
ok = ' 0 MISMATCHES' in sync
if not sync: sync = 'NO SYNCTEST RESULT LINE (the run did not finish)'
res = 'ok  ' if not kos and not bad and ok else 'FAIL'
print(f'{res} {tag}: {len(sw)} switches, {len(kos)} switch KOs, {len(bad)} misplaced, {combat} combat KOs (hit after the switch: allowed); {sync.strip()}')
print('     states at the switches: ' + ', '.join(f'{k} {v}' for k, v in cov.most_common()))
for l in kos[:8]: print('     ' + l.strip())
for b in bad[:8]: print('     ' + b)
sys.exit(0 if res == 'ok  ' else 1)
PY
done
echo "switch_torture: $([ $fail = 0 ] && echo PASS || echo FAIL) (logs: $out)"
exit $fail
