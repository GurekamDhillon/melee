#!/bin/bash
export GW_MELEE="E:/Projects/Melee Workspace/worktrees/plat-net"
python "E:/Projects/Melee Workspace/tools/netplay/server/gdmelee_server.py" --port $1 --bind 127.0.0.1 > "$TEMP/np/srv.log" 2>&1 &
echo $! > "$TEMP/np/srv.pid"
