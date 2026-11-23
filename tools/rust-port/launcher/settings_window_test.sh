#!/usr/bin/env bash
# The launcher's Settings button on the Rust engine (rust/melee/src/settings_window.rs), hidden, no disc:
#   1. `melee --settings-window --settings-path <ini> --settings-tab audio` with a key script (two steps of the
#      master volume, Escape): exit 0, his file's volume changed, every other line of it kept, settings.cfg agrees;
#   2. standby: `--settings-standby <pid>` waits; the Show event opens it (script: Escape), exit 0;
#   3. standby: the launcher process ends -> exit 0 without a window.
# usage: settings_window_test.sh <melee.exe> [shot dir]   (with a shot dir: 720p and 1080p pictures)
set -u
EXE=${1:?melee.exe}
SHOTS_DIR=${2:-}
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
export MELEE_INPUT=n MELEE_PAD_IGNORE_ADAPTER=1 MELEE_WINDOW_HIDDEN=1 MELEE_VOLUME=3
export MELEE_SETTINGS_CFG="$T/settings.cfg" MELEE_AUDIO_CFG="$T/audio.cfg"
INI="$T/port-settings.ini"
fail() { echo "SETTINGS WINDOW FAIL: $*"; tail -20 "$T"/run*.txt 2>/dev/null; exit 1; }
win() { cygpath -w "$1"; }

printf '# test file\nfullscreen 0\nvolume 50\nmusic 80\ndlss_mode quality\n; a comment\nkey_A 88\npad1_A 4096\nvsync 1\n' > "$INI"
cp "$INI" "$T/before.ini"

# 1. open, change, close
MELEE_WINDOW_TITLE="agent musettings: settings window (open)" MELEE_SETTINGS_SCRIPT="right,right,esc" MELEE_SETTINGS_MAX_TICKS=600 \
  MELEE_LOG_FILE="$(win "$T/run1.txt")" timeout 60 "$EXE" --settings-window --settings-path "$(win "$INI")" --settings-tab audio 2> "$T/err1.txt"
rc=$?
[ $rc -eq 0 ] || { cat "$T/run1.txt" | tail -20; fail "open: exit $rc"; }
grep -q '^volume 60$' "$INI" || fail "volume not 60 in the ini: $(grep volume "$INI")"
diff <(grep -v '^volume' "$T/before.ini") <(grep -v '^volume' "$INI") > /dev/null || fail "other lines changed"
grep -q 'volume = 60' "$T/audio.cfg" || fail "audio.cfg: $(cat "$T/audio.cfg" 2>/dev/null)"
grep -q 'music=80' "$T/settings.cfg" || fail "settings.cfg not synced from the ini: $(cat "$T/settings.cfg" 2>/dev/null)"
echo "open: exit 0, volume 50 -> 60, other lines kept, settings.cfg / audio.cfg agree"

# 2. standby, shown by the launcher's event
sleep 60 & P=$!
WP=$(cat /proc/$P/winpid)
MELEE_WINDOW_TITLE="agent musettings: settings window (standby)" MELEE_SETTINGS_SCRIPT="esc" MELEE_SETTINGS_MAX_TICKS=600 \
  MELEE_LOG_FILE="$(win "$T/run2.txt")" timeout 60 "$EXE" --settings-window --settings-standby "$WP" --settings-path "$(win "$INI")" 2> "$T/err2.txt" & S=$!
for i in $(seq 1 50); do grep -q "standby for launcher" "$T/run2.txt" 2>/dev/null && break; sleep 0.2; done
powershell -NoProfile -Command "[System.Threading.EventWaitHandle]::new(\$false,'AutoReset','Local\\MeleeUnlockedSettingsShow-$WP').Set()" > /dev/null
wait $S; rc=$?
kill $P 2>/dev/null
[ $rc -eq 0 ] && grep -q "settings window: shown" "$T/run2.txt" && grep -q "settings window: closed" "$T/run2.txt" || { tail "$T/run2.txt"; fail "standby show: exit $rc"; }
echo "standby: shown by the event, closed, exit 0"

# 3. standby, launcher gone
sleep 60 & P=$!
WP=$(cat /proc/$P/winpid)
MELEE_LOG_FILE="$(win "$T/run3.txt")" timeout 30 "$EXE" --settings-window --settings-standby "$WP" --settings-path "$(win "$INI")" 2> "$T/err3.txt" & S=$!
sleep 2; kill $P
wait $S; rc=$?
[ $rc -eq 0 ] && grep -q "standby ended" "$T/run3.txt" || { tail "$T/run3.txt"; fail "standby parent exit: exit $rc"; }
echo "standby: launcher exit ends it, exit 0"

if [ -n "$SHOTS_DIR" ]; then
  mkdir -p "$SHOTS_DIR"
  for res in 1280x720 1920x1080; do
    for tab in video audio; do
      MELEE_WINDOW=$res MELEE_WINDOW_TITLE="agent musettings: settings window shot $res $tab" MELEE_SETTINGS_SHOT="$(win "$SHOTS_DIR/settings_${tab}_${res#*x}p.png")" \
        MELEE_LOG_FILE="$(win "$T/shots_${res}_${tab}.txt")" MELEE_SETTINGS_SCRIPT="down,esc" MELEE_SETTINGS_MAX_TICKS=600 timeout 60 "$EXE" --settings-window --settings-path "$(win "$INI")" --settings-tab $tab 2>> "$T/err_shots.txt" || fail "shot $res $tab"
    done
  done
  cat "$T"/shots_*.txt | grep "settings window: shot"
fi
echo "SETTINGS WINDOW OK"
