#!/bin/bash
# SPDX-License-Identifier: MIT
# A batch of dolphin_peer.sh sets: our build vs the local-only Slippi Dolphin, every legal stage at three lag levels,
# with character pairs that cover projectiles (Young Link, Falco) and Ice Climbers, at most 2 pairs at a time.
#   dolphin_peer_batch.sh OUTROOT [--games 3] [--game-frames 2400] [--parallel 2] [--only N,M,...]
# Before each set it waits while the owner plays: any melee-slippi*.exe or "Slippi Dolphin.exe" (none of which this
# harness starts: it runs melee.exe and dolphin-emu-nogui.exe) or an owner-play hold on the build locks. Every process a
# set starts runs at below-normal priority (Dolphin) or as this harness's own child; only those are stopped.
# Output: OUTROOT/<n>-<scenario>/ per set and OUTROOT/results.tsv (one ROW per set, dolphin_peer_summary.py).
set -u
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ws="${GW_WORKSPACE:-E:/Projects/Melee Workspace}"
R=$1; shift; mkdir -p "$R"; R="$(cd "$R" && pwd)"
games=3; gframes=2400; par=2; only=""
while [ $# -gt 0 ]; do
  case "$1" in
    --games) games=$2; shift ;; --game-frames) gframes=$2; shift ;; --parallel) par=$2; shift ;; --only) only=",$2,"; shift ;;
    *) echo "dolphin_peer_batch: unknown option $1" >&2; exit 2 ;;
  esac; shift
done
[ "$par" -le 2 ] || par=2
stages=(0x20 0x1F 0x08 0x1C 0x02 0x03)   # FD, Battlefield, Yoshi's Story, Dream Land, Fountain of Dreams, Pokemon Stadium
lags=("0 4" "60 10" "120 20")            # round trip ms, jitter ms
pairs=("ylink falco" "falco ylink" "ics fox" "fox ics" "falco ics" "ics ylink")
owner_busy() {
  local p
  p=$(powershell -NoProfile -Command "Get-Process -ErrorAction SilentlyContinue | Where-Object { \$_.Name -like 'melee-slippi*' -or \$_.Name -eq 'Slippi Dolphin' } | ForEach-Object { \$_.Name + ':' + \$_.Id }" 2>/dev/null | tr -d '\r' | tr '\n' ' ')
  [ -n "${p// /}" ] && { echo "owner process: $p"; return 0; }
  bash "$ws/scratch/buildlock.sh" status 2>/dev/null | grep -q "owner-play" && { echo "owner-play lock held"; return 0; }
  return 1
}
free_gb() { df -BG /e 2>/dev/null | awk 'NR==2 {gsub("G","",$4); print $4}'; }
wait_owner() {
  local why
  while why=$(owner_busy); do echo "$(date +%T) paused: $why" | tee -a "$R/batch.log"; sleep 60; done
}
# DNS cache entries for Slippi / STUN / Google names before and after (Get-DnsClientCache needs no admin): a name the
# batch made this machine resolve would appear in "after" only
dnsnames() { powershell -NoProfile -Command "Get-DnsClientCache -ErrorAction SilentlyContinue | Where-Object { \$_.Entry -match 'slippi|appspot|stun|twilio|cloudflare|googleapis|google\.com' } | ForEach-Object { \$_.Entry } | Sort-Object -Unique" 2>/dev/null | tr -d '\r'; }
dnsnames > "$R/dns_before.txt"
n=0; running=()
: > "$R/results.tsv.tmp"
for li in 0 1 2; do
  for si in 0 1 2 3 4 5; do
    n=$((n + 1))
    [ -n "$only" ] && [[ "$only" != *",$n,"* ]] && continue
    read -r rtt jit <<< "${lags[$li]}"
    st=${stages[$si]}
    read -r ours theirs <<< "${pairs[$(( (si + li) % ${#pairs[@]} ))]}"
    dec=$([ $((n % 2)) = 0 ] && echo engine || echo dolphin)
    name=$(printf "%02d-%s-vs-%s-%s-rtt%s" $n "$ours" "$theirs" "$st" "$rtt")
    [ "$(free_gb)" -ge 20 ] || { echo "dolphin_peer_batch: under 20 GB free on E:, stopping" | tee -a "$R/batch.log"; break 2; }
    # at most $par sets at once
    while [ ${#running[@]} -ge "$par" ]; do
      alive=(); for p in "${running[@]}"; do kill -0 "$p" 2>/dev/null && alive+=("$p"); done; running=("${alive[@]}")
      [ ${#running[@]} -ge "$par" ] && sleep 5
    done
    wait_owner
    echo "$(date +%T) start $name (decider $dec)" | tee -a "$R/batch.log"
    bash "$here/dolphin_peer.sh" "$R/$name" --ours "$ours" --theirs "$theirs" --stage "$st" --rtt-ms "$rtt" --jitter-ms "$jit" \
      --games "$games" --game-frames "$gframes" --decider "$dec" --mm-port $((44000 + n * 4)) --seed $n --name "dp$n" \
      > "$R/$name.out" 2>&1 &
    running+=($!)
    sleep 20   # stagger the two pairs' boots
  done
done
wait
dnsnames > "$R/dns_after.txt"
echo "dns: $(wc -l < "$R/dns_before.txt") matching names before, $(wc -l < "$R/dns_after.txt") after; new: $(comm -13 "$R/dns_before.txt" "$R/dns_after.txt" | tr '\n' ' ')" | tee -a "$R/batch.log"
for d in "$R"/[0-9][0-9]-*/; do
  [ -f "$d/summary.txt" ] || continue
  printf "%s\t%s\n" "$(basename "$d")" "$(grep '^ROW' "$d/summary.txt" | cut -f2-)" >> "$R/results.tsv.tmp"
done
sort "$R/results.tsv.tmp" > "$R/results.tsv"; rm -f "$R/results.tsv.tmp"
cat "$R/results.tsv"
