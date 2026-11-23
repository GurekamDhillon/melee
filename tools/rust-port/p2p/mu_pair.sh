#!/bin/bash
# mu_pair.sh OUTDIR [FRAMES] [extra args for both peers...]: two headless `melee mu-netplay` peers (P2P Direct,
# mu_net) on loopback, hidden, scripted pads. Pass when both exit 0 (no desync, checksums compared, RESULT digests
# agree). Run it under the game slots: ~/melee_out/locks/game.sh 2 tools/rust-port/p2p/mu_pair.sh OUT 1200
# Env: MELEE (the binary; default the p2pdirect cargo dir), GW_ISO_VANILLA (found in ~/melee_linux_test if unset).
# Never prints the disc path.
set -u
OUT=${1:?outdir}; FRAMES=${2:-1200}; shift; shift || true
mkdir -p "$OUT"
MELEE=${MELEE:-$HOME/melee_rust/build/cargo-p2pdirect/release/melee}
if [ -z "${GW_ISO_VANILLA:-}" ]; then
  GW_ISO_VANILLA=$(find "$HOME/melee_linux_test" -maxdepth 1 -name '*Smash*.iso' | head -1)
fi
export GW_ISO_VANILLA
export MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_VOLUME=3
PA=$((40000 + RANDOM % 10000)); PB=$((PA + 1))
SEED=$(printf '%08X' $((RANDOM * 65536 + RANDOM)))
common=(--p2p-chars 2/0:9/1 --p2p-stage 31 --p2p-seed "$SEED" --p2p-delay 2 --p2p-names A:B --p2p-connect-seconds 30 --frames "$FRAMES")
IA="$OUT/id_a.json"; IB="$OUT/id_b.json"
# identities first, so each side can expect the other's key (as the lobby passes --p2p-expect)
"$MELEE" mu-netplay --p2p-port 1 --p2p-peer 127.0.0.1:1 --p2p-identity "$IA" --p2p-connect-seconds 1 >/dev/null 2>"$OUT/ida.log"
"$MELEE" mu-netplay --p2p-port 1 --p2p-peer 127.0.0.1:1 --p2p-identity "$IB" --p2p-connect-seconds 1 >/dev/null 2>"$OUT/idb.log"
KA=$(grep -o 'identity [0-9a-f]\{64\}' "$OUT/ida.log" | head -1 | cut -d' ' -f2)
KB=$(grep -o 'identity [0-9a-f]\{64\}' "$OUT/idb.log" | head -1 | cut -d' ' -f2)
MELEE_WINDOW_TITLE="p2pdirect A" "$MELEE" mu-netplay --p2p-port $PA --p2p-peer 127.0.0.1:$PB --p2p-slot 0 --p2p-identity "$IA" --p2p-expect "$KB" \
  "${common[@]}" --out "$OUT/a.csv" "$@" >"$OUT/a.out" 2>"$OUT/a.log" &
A=$!
MELEE_WINDOW_TITLE="p2pdirect B" "$MELEE" mu-netplay --p2p-port $PB --p2p-peer 127.0.0.1:$PA --p2p-slot 1 --p2p-identity "$IB" --p2p-expect "$KA" \
  "${common[@]}" --out "$OUT/b.csv" "$@" >"$OUT/b.out" 2>"$OUT/b.log" &
B=$!
wait $A; RA=$?
wait $B; RB=$?
grep -h "^p2p: slot" "$OUT/a.log" "$OUT/b.log" | grep -E "final checksums|DESYNC|failed|connected" 
echo "mu_pair: exit A=$RA B=$RB"
[ $RA -eq 0 ] && [ $RB -eq 0 ]
