#!/bin/bash
# SPDX-License-Identifier: MIT
# One Unranked set of our build against a real (local-only) Slippi Dolphin on this Windows machine, through the CSS
# between games, paired by the loopback mock matchmaking (mock_mm.py), with lag and jitter from the mock's relay.
#
#   dolphin_peer.sh OUTDIR [--exe MELEE.EXE] [--dolphin-dir D] [--ours CHAR] [--theirs CHAR] [--stage 0x20]
#                   [--rtt-ms N] [--jitter-ms N] [--loss-pct P] [--games N] [--game-frames N] [--decider engine|dolphin]
#                   [--mm-port P] [--seed S] [--name TAG]
#
# --rtt-ms is the round trip the relay adds (half of it each way); --jitter-ms adds 0..N ms per datagram, each way (it
# reorders datagrams); --game-frames N with --end sd (default): after N in-game frames of random play Dolphin's player
# runs off the stage until it is out of stocks (a normal game end, a few stocks lost in seconds); --end lras: it holds
# L+R+A+Start instead (Slippi's client pause quit-out, which disconnects); 0 frames: play each game out. --decider: the mock makes the first queued player the host (Slippi's decider).
# Characters are dolphin_drive.py CSS names (falco, ylink, ics, fox, marth, ...); stage an external stage id (the mock's
# stage list holds only it, so the Unranked random stage is that one).
#
# Isolation (docs/rust-port/notes/slippi-online.md "Real Slippi Dolphin peer"):
# - Dolphin is the local-only build (_build/dolphin-oracle branch gd/dolphinpeer, GD_DOLPHINPEER): it forces
#   SLIPPI_LOCAL_TEST (web API, rank, game reports refused before DNS; no STUN; no browser; netplay bound to 127.0.0.1),
#   only a ws://127.0.0.1 matchmaking URL, and refuses every C++ HTTP request. Its user dir is made fresh per run here:
#   a throwaway test profile (fake uid, no play key, code DPEER#001), never a real user.json.
# - our build talks only to the loopback mock (MELEE_SLIPPI_MM_URL; its client refuses other addresses without --mm-real).
# - netwatch.txt: every 2 s, every TCP connection and UDP endpoint owned by this run's Dolphin and game processes;
#   the summary fails the run if any is not loopback.
# Input: Dolphin's pads come from its named pipe (\\.\pipe\<tag>slippibot1, dolphin_drive.py); ours from MELEE_PAD_SCRIPT
# and the generated in-game pads (MELEE_SLIPPI_NP_PADGEN). No OS input, no focus, no adapter, no controller device.
# Output in OUTDIR: summary.txt (one row, dolphin_peer_summary.py), ours.log, d_*.txt (Dolphin's checksum, peer and state
# logs), dolphin.log, drive.log, mm.log, ours*.csv, ours*.slp, dslp/*.slp, netwatch.txt. Only processes started here
# are stopped.
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"; repo="$(cd "$here/../../.." && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
O=$1; shift; mkdir -p "$O"; O="$(cd "$O" && pwd)"
exe="${EXE:-$ws/_build/dolphinpeer/melee.exe}"; ddir="${DOLPHIN_DIR:-$ws/_build/dolphin-peer}"
ours=fox; theirs=falco; stage=0x20; rtt=0; jit=0; loss=0; games=3; gframes=0; decider=dolphin; mmp=43500; seed=1; tag=""; endhow=sd
while [ $# -gt 0 ]; do
  case "$1" in
    --exe) exe=$2; shift ;; --dolphin-dir) ddir=$2; shift ;; --ours) ours=$2; shift ;; --theirs) theirs=$2; shift ;;
    --stage) stage=$2; shift ;; --rtt-ms) rtt=$2; shift ;; --jitter-ms) jit=$2; shift ;; --loss-pct) loss=$2; shift ;;
    --games) games=$2; shift ;; --game-frames) gframes=$2; shift ;; --decider) decider=$2; shift ;;
    --mm-port) mmp=$2; shift ;; --seed) seed=$2; shift ;; --name) tag=$2; shift ;; --end) endhow=$2; shift ;;
    *) echo "dolphin_peer: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
tag=${tag:-dp$mmp}
echo $$ > "$O/harness.pid"
dexe="$ddir/dolphin-emu-nogui.exe"
[ -x "$exe" ] || { echo "dolphin_peer: no game exe at $exe" >&2; exit 2; }
[ -x "$dexe" ] || { echo "dolphin_peer: no Dolphin at $dexe" >&2; exit 2; }
iso=$(sed -nE 's/^(export )?GW_ISO_VANILLA=//p' "$ws/.env" | head -1 | tr -d '"\r')
[ -n "$iso" ] || { echo "dolphin_peer: GW_ISO_VANILLA is not in .env" >&2; exit 2; }
half=$((rtt / 2))
scen="$ours vs Dolphin $theirs, stage $stage, rtt $rtt ms, jitter $jit ms, $games games"
echo "dolphin_peer: $scen (decider $decider, mm port $mmp)" | tee "$O/args.txt"
echo "exe $exe" >> "$O/args.txt"; echo "dolphin $dexe" >> "$O/args.txt"
rm -rf "$O"/user "$O"/dslp "$O"/desyncs "$O"/*.csv "$O"/*.slp "$O"/d_*.txt "$O"/*.log "$O"/netwatch.txt "$O"/summary.txt
mkdir -p "$O/dslp"

# --- the throwaway Dolphin user dir: no controller device but the pipe, no adapter, no audio, no video
U="$O/user"; mkdir -p "$U/Config" "$U/Slippi" "$U/Logs"
Uw=$(cygpath -w "$U"); Ow=$(cygpath -w "$O")
cat > "$U/Config/Dolphin.ini" <<EOF
[Core]
SIDevice0 = 6
SIDevice1 = 0
SIDevice2 = 0
SIDevice3 = 0
EmulationSpeed = 1.00000000
EnableCheats = True
GFXBackend = Null
CPUThread = True
[DSP]
Backend = No Audio Output
Volume = 0
[Input]
BackgroundInput = True
[Interface]
ConfirmStop = False
UsePanicHandlers = False
OnScreenDisplayMessages = False
[Display]
Fullscreen = False
RenderToMain = False
[Analytics]
Enabled = False
PermissionAsked = True
[AutoUpdate]
UpdateTrack =
[Slippi]
OnlineDelay = 2
EnableSpectator = False
SaveReplays = True
ReplayDir = $Ow\\dslp
ReplayMonthlyFolders = False
EnableJukebox = False
EnableQuickChat = 0
InputSuggestions = False
ForceNetplayPort = False
ShowLocalRankInfo = False
ShowOpponentRankInfo = False
EOF
# SIDevice0 = 6 is a standard controller fed by Dolphin's emulated pad 1 (below: the pipe), not the GC adapter (12)
cat > "$U/Config/Logger.ini" <<EOF
[Options]
Verbosity = 4
WriteToFile = True
WriteToConsole = False
[Logs]
SLIPPI = True
SLIPPI_ONLINE = True
COMMON = True
EOF
{
  echo "[GCPad1]"
  echo "Device = Pipe/0/slippibot1"
  for b in A B X Y Z L R; do echo "Buttons/$b = \`Button $b\`"; done
  echo "Buttons/Start = \`Button START\`"
  echo "Main Stick/Up = \`Axis MAIN Y +\`"; echo "Main Stick/Down = \`Axis MAIN Y -\`"
  echo "Main Stick/Left = \`Axis MAIN X -\`"; echo "Main Stick/Right = \`Axis MAIN X +\`"
  echo "Main Stick/Dead Zone = 0.0"
  echo "C-Stick/Up = \`Axis C Y +\`"; echo "C-Stick/Down = \`Axis C Y -\`"
  echo "C-Stick/Left = \`Axis C X -\`"; echo "C-Stick/Right = \`Axis C X +\`"
  echo "C-Stick/Dead Zone = 0.0"
  echo "Triggers/L = \`Button L\`"; echo "Triggers/R = \`Button R\`"
  echo "Triggers/L-Analog = \`Axis L -+\`"; echo "Triggers/R-Analog = \`Axis R -+\`"
  for d in Up:UP Down:DOWN Left:LEFT Right:RIGHT; do echo "D-Pad/${d%%:*} = \`Button D_${d#*:}\`"; done
  for p in 2 3 4; do echo "[GCPad$p]"; echo "Device = "; done
} > "$U/Config/GCPadNew.ini"
# a fake test identity: no real account, no play key
printf '{"uid":"dolphinpeer-local-%s","playKey":"","connectCode":"DPEER#001","displayName":"Peer Test","latestVersion":"0.0.0"}\n' "$tag" > "$U/Slippi/user.json"

# --- the mock matchmaking server and relay (loopback only)
first=$([ "$decider" = engine ] && echo ours || echo dolphin)
python -I "$here/mock_mm.py" --port $mmp --stages "$stage" --lag-ms $half --jitter-ms $jit --loss-pct $loss > "$O/mm.log" 2>&1 < /dev/null &
mm=$!
sleep 1

# --- process bookkeeping: only what this run starts
dpid=""; gpids=""
# our game processes: this exe's name with this run's directory (either slash form) on the command line
ours_pids() { DP_N="$(basename "$exe")" DP_D="$O" DP_DW="$Ow" powershell -NoProfile -Command 'Get-CimInstance Win32_Process | Where-Object { $_.Name -eq $env:DP_N -and $_.CommandLine -and ($_.CommandLine.Contains($env:DP_D) -or $_.CommandLine.Contains($env:DP_DW)) } | ForEach-Object { $_.ProcessId }' 2>/dev/null | tr -d '\r' | tr '\n' ' '; }
stop_all() {
  [ -n "$watch" ] && kill $watch 2>/dev/null
  for p in $(ours_pids) $dpid; do taskkill //F //PID $p > /dev/null 2>&1; done
  [ -n "${drv:-}" ] && kill $drv 2>/dev/null
  kill $mm 2>/dev/null
}
watch=""
trap 'stop_all; exit 143' INT TERM

start_dolphin() {
  SLIPPI_LOCAL_TEST=1 SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" SLIPPI_PIPE_PREFIX="$tag" \
  SLIPPI_CHECKSUM_LOG="$Ow\\d_checksums.txt" SLIPPI_PEER_LOG="$Ow\\d_peer.txt" \
  MELEE_DOLPHIN_STATELOG="$Ow\\d_state.txt" MELEE_DOLPHIN_STATELOG_ADDRS="804a04f0,804d6640,804d6cf4,80479d60,804c1fac,804a0bc0,81155a8c,81155a90" \
  DP_EXE="$(cygpath -w "$dexe")" DP_ARGS="-u \"$Uw\" -p headless -v Null -e \"$iso\"" DP_OUT="$Ow" \
  powershell -NoProfile -Command '$p = Start-Process -FilePath $env:DP_EXE -ArgumentList $env:DP_ARGS -PassThru -WindowStyle Hidden -RedirectStandardError (Join-Path $env:DP_OUT "d_stderr.txt") -RedirectStandardOutput (Join-Path $env:DP_OUT "d_stdout.txt"); try { $p.PriorityClass = "BelowNormal" } catch {}; $p.Id' > "$O/d_pid.txt" 2>&1 < /dev/null
}
start_ours() {
  python -I "$here/dolphin_peer_pads.py" "$ours" "$O" || exit 2
  local setpads=(); for g in $(seq 2 $((games + 1))); do setpads+=("MELEE_SLIPPI_SET_PAD_SCRIPT_G$g=$O/pads_gn.txt"); done
  ( cd "$repo/rust" && env "${setpads[@]}" GW_ISO_VANILLA="$iso" MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_WINDOW_UNFOCUSED=1 MELEE_VOLUME=3 \
    MELEE_KEYBOARD=0 MELEE_RETAIL_FIRST=1 MELEE_CARD=0 MELEE_WINDOW_TITLE="dolphinpeer agent: $scen" \
    MELEE_PAD_SCRIPT="$O/pads_g1.txt" \
    MELEE_SLIPPI_SET_GAMES=$((games + 1)) MELEE_SLIPPI_MM_URL="ws://127.0.0.1:$mmp/v1/queue" MELEE_SLIPPI_NP_PORT=$((mmp + 100)) \
    MELEE_SLIPPI_TEST_CODE="OURS#001" MELEE_SLIPPI_NP_OUT="$O/ours.csv" MELEE_SLIPPI_NP_MAX_SECONDS=900 MELEE_SLIPPI_NP_PADGEN=$seed \
    MELEE_DESYNC_DIR="$O/desyncs" \
    timeout $((games * 700 + 300)) "$exe" play --skip-intro --slippi-menus on --slp "$O/ours.slp" > "$O/ours.log" 2>&1 < /dev/null ) &
  gw=$!
}
netwatch() {  # every TCP connection and UDP endpoint of this run's processes, every 2 s
  while true; do
    local ps; ps="$(ours_pids) $dpid"
    ps=$(echo $ps | tr ' ' ',')
    [ -n "$ps" ] && powershell -NoProfile -Command "\$t=Get-Date -Format HH:mm:ss; foreach (\$id in @($ps)) { Get-NetTCPConnection -OwningProcess \$id -ErrorAction SilentlyContinue | ForEach-Object { \"\$t pid \$id TCP \$(\$_.LocalAddress):\$(\$_.LocalPort) -> \$(\$_.RemoteAddress):\$(\$_.RemotePort) \$(\$_.State)\" }; Get-NetUDPEndpoint -OwningProcess \$id -ErrorAction SilentlyContinue | ForEach-Object { \"\$t pid \$id UDP \$(\$_.LocalAddress):\$(\$_.LocalPort)\" } }" 2>/dev/null | tr -d '\r'
    sleep 2
  done
}

if [ "$first" = ours ]; then
  start_ours
  for i in $(seq 240); do grep -q "join from OURS" "$O/mm.log" && break; sleep 0.5; done
fi
# not $(...): Dolphin inherits the pipe and the substitution would wait for it to exit
start_dolphin &
for i in $(seq 100); do [ -s "$O/d_pid.txt" ] && break; sleep 0.2; done
dpid=$(tr -dc '0-9' < "$O/d_pid.txt")
echo "dolphin pid $dpid" >> "$O/args.txt"
netwatch > "$O/netwatch.txt" 2>&1 &
watch=$!
python -I "$here/dolphin_drive.py" --name "$tag" --state "$Ow\\d_state.txt" --mode unranked --play "$theirs" --games $games \
  --pads $seed --frames $([ "$gframes" -gt 0 ] && echo $gframes || echo 40000) $([ "$gframes" -gt 0 ] && echo --$endhow) \
  --timeout 600 --css-timeout 240 > "$O/drive.log" 2>&1 &
drv=$!
if [ "$first" = dolphin ]; then
  for i in $(seq 600); do grep -q "join from DPEER" "$O/mm.log" && break; kill -0 $drv 2>/dev/null || break; sleep 0.5; done
  start_ours
fi
wait $drv; drc=$?
echo "driver rc $drc" >> "$O/args.txt"
# our side: after the last game it is back on the select for the next one (MELEE_SLIPPI_SET_GAMES = games + 1); give it
# time to log the game's end and the select, then stop this run's processes
for i in $(seq 60); do
  n=$(grep -ac "final checksums" "$O/ours.log" 2>/dev/null); [ "${n:-0}" -ge "$games" ] && break; sleep 2
done
sleep 8
stop_all
wait $gw 2>/dev/null
cp "$U/Logs/dolphin.log" "$O/dolphin.log" 2>/dev/null
python -I "$here/dolphin_peer_summary.py" "$O" --games $games | tee "$O/summary.txt"
