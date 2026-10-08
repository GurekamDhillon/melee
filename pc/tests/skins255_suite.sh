#!/bin/bash
# Run all registered native cases against the selected checkout; keep complete logs.
set -u
cd "$(dirname "$0")/../../../.."
source .env
export GW_MELEE="E:/Projects/Melee Workspace/worktrees/skins255"
export GW_BUILD_ROOT="E:/Projects/Melee Workspace/_build/agents/skins255"
phase=${1:-final}
logs="$GW_BUILD_ROOT/skins255-logs"
mkdir -p "$logs"
python -c 'import re; s=open("tools/port/native_test.sh").read(); print("\n".join(n for x in re.findall(r"^([a-z0-9|_-]+)\)",s,re.M) for n in x.split("|")))' > "$logs/names.txt"
while read -r name; do
    name=${name%$'\r'}
    bash tools/port/build.sh --native-test "$name" > "$logs/$phase-$name.log" 2>&1
    echo "$name $?"
done < "$logs/names.txt"
