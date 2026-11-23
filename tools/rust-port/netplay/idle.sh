#!/bin/bash
# Run a command at idle priority (Windows): start it, lower its class, wait for it. Exit code = the command's.
# Only the agent harness's own peers go Idle: a peer being measured (MELEE_PROFILE=1 in its environment) or one with
# NP_PRIO=normal keeps normal priority (an Idle game thread loses to every other process and its frame times are noise).
"$@" &
pid=$!
if [ "${MELEE_PROFILE:-}" != 1 ] && [ "${NP_PRIO:-idle}" != normal ]; then
  wp=$(cat /proc/$pid/winpid 2>/dev/null)
  [ -n "$wp" ] && powershell -NoProfile -Command "try { (Get-Process -Id $wp).PriorityClass = 'Idle' } catch {}" >/dev/null 2>&1
fi
wait $pid
