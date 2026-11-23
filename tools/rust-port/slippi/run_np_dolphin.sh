#!/bin/bash
# SPDX-License-Identifier: MIT
# Linux counterpart of run_np_dolphin.ps1: start one local-test Slippi Dolphin (build_dolphin_linux.sh) for the
# loopback engine-vs-Dolphin harness. Headless, Null video, no audio, no window, no OS input: pads come from the FIFO
# <userdir>/Pipes/slippibot1 (dolphin_drive.py --pipe). Matchmaking goes to the local mock (SLIPPI_MM_URL);
# SLIPPI_LOCAL_TEST=1 refuses the web API, game reports, STUN and browser pages and binds every netplay socket to
# 127.0.0.1 (np_dolphin_pair.sh also runs everything in a private network namespace with loopback only).
#   run_np_dolphin.sh NAME OUT MMPORT CODE OPP_CODE [ADDRS]
# The user dir OUT/user_NAME is made fresh each run: a throwaway test profile (fake uid, no play key, CODE), never a
# real Slippi user.json. NP_REPLAY_DIR=D: Dolphin saves its .slp there (latestVersion must parse as semver: an empty one throws in prepareOnlineStatus). The disc comes from GW_ISO_VANILLA and is never printed. Prints the PID and returns at once.
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
N=$1; O=$2; MMP=$3; CODE=$4; OPP=$5
ADDRS=${6:-804a04f0,804d6640,804d6cf4,80479d60,804c1fac,804a0bc0,81155a8c,81155a90}
BIN=${DOLPHIN_BIN:-$HOME/melee_out/deciderharness/dolphin-build/Binaries}
[ -n "$GW_ISO_VANILLA" ] || { echo "run_np_dolphin: GW_ISO_VANILLA is not set" >&2; exit 2; }
U=$O/user_$N
rm -rf "$U"; mkdir -p "$U/Config" "$U/Slippi" "$U/Pipes" "$U/Logs"
mkfifo "$U/Pipes/slippibot1"
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
[Slippi]
EnableSpectator = False
SaveReplays = $([ -n "$NP_REPLAY_DIR" ] && echo True || echo False)
ReplayDir = ${NP_REPLAY_DIR:-$O/replays}
ReplayMonthlyFolders = False
EnableJukebox = False
EnableQuickChat = 0
InputSuggestions = False
ForceNetplayPort = False
EOF
cat > "$U/Config/Logger.ini" <<EOF
[Options]
Verbosity = 4
WriteToFile = True
WriteToConsole = False
[Logs]
SLIPPI = True
SLIPPI_ONLINE = True
BOOT = False
EOF
{
  echo "[GCPad1]"
  echo "Device = Pipe/0/slippibot1"
  for b in A B X Y Z L R; do echo "Buttons/$b = \`Button $b\`"; done
  echo "Buttons/Start = \`Button START\`"
  echo "Main Stick/Up = \`Axis MAIN Y +\`"; echo "Main Stick/Down = \`Axis MAIN Y -\`"
  echo "Main Stick/Left = \`Axis MAIN X -\`"; echo "Main Stick/Right = \`Axis MAIN X +\`"
  echo "C-Stick/Up = \`Axis C Y +\`"; echo "C-Stick/Down = \`Axis C Y -\`"
  echo "C-Stick/Left = \`Axis C X -\`"; echo "C-Stick/Right = \`Axis C X +\`"
  echo "Triggers/L = \`Button L\`"; echo "Triggers/R = \`Button R\`"
  echo "Triggers/L-Analog = \`Axis L -+\`"; echo "Triggers/R-Analog = \`Axis R -+\`"
  for d in Up:UP Down:DOWN Left:LEFT Right:RIGHT; do echo "D-Pad/${d%%:*} = \`Button D_${d#*:}\`"; done
} > "$U/Config/GCPadNew.ini"
printf '{"uid":"localtest%s","playKey":"","connectCode":"%s","displayName":"Test %s","latestVersion":"0.0.0"}\n' \
  "$N" "$CODE" "$N" > "$U/Slippi/user.json"
printf '[{"connectCode":"%s","lastPlayed":%d}]\n' "$OPP" "$(date +%s)" > "$U/Slippi/direct-codes.json"
export SLIPPI_LOCAL_TEST=1 SLIPPI_MM_URL="ws://127.0.0.1:$MMP/v1/queue" SLIPPI_PIPE_PREFIX=$N
export SLIPPI_CHECKSUM_LOG=$O/checksums_$N.txt MELEE_DOLPHIN_STATELOG=$O/state_$N.txt MELEE_DOLPHIN_STATELOG_ADDRS=$ADDRS
unset SLIPPI_MM_ENET SLIPPI_REAL_NET
# a distinctive process name (comm): deciderharness:dolphin
ln -sf dolphin-emu-nogui "$BIN/dh-dolphin"
nohup nice -n 5 "$BIN/dh-dolphin" -u "$U" -p headless -v Null -e "$GW_ISO_VANILLA" > "$O/stdout_$N.txt" 2> "$O/stderr_$N.txt" &
echo $!
