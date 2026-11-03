#!/bin/bash
# Run a command at idle priority (Windows): start it, lower its class, wait for it. Exit code = the command's.
"$@" &
pid=$!
wp=$(cat /proc/$pid/winpid 2>/dev/null)
[ -n "$wp" ] && powershell -NoProfile -Command "try { (Get-Process -Id $wp).PriorityClass = 'Idle' } catch {}" >/dev/null 2>&1
wait $pid
