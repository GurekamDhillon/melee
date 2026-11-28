#!/bin/bash
# SPDX-License-Identifier: MIT
# Stop every process of one dolphin_peer.sh run (or a batch): those whose command line names DIR, the harness's own
# bash included, never anything else (not this script, not PowerShell).
#   dolphin_peer_stop.sh DIR
D=$(cd "$1" && pwd); Dw=$(cygpath -w "$D")
me=$$
DP_D="$D" DP_DW="$Dw" powershell -NoProfile -Command 'Get-CimInstance Win32_Process | Where-Object { $_.Name -ne "powershell.exe" -and $_.CommandLine -and ($_.CommandLine.Contains($env:DP_D) -or $_.CommandLine.Contains($env:DP_DW)) -and -not $_.CommandLine.Contains("dolphin_peer_stop") } | ForEach-Object { $_.ProcessId }' | tr -d '\r' | while read -r p; do
  [ -n "$p" ] && [ "$p" != "$me" ] && taskkill //F //PID "$p" > /dev/null 2>&1 && echo "stopped $p"
done
