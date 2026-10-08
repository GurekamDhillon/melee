#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/../../../.."
source .env
export GW_MELEE="E:/Projects/Melee Workspace/worktrees/skins255"
export GW_BUILD_ROOT="E:/Projects/Melee Workspace/_build/agents/skins255"
case ${1:-build} in
build) bash tools/port/build.sh ;;
baseline) bash tools/port/run.sh --test skins255-baseline ;;
test) bash tools/port/run.sh --test skins255-final ;;
fixture64) MELEE_SKINS255_TEST_COUNT=64 bash tools/port/run.sh --test skins255-fixture64 ;;
fixture255) MELEE_SKINS255_TEST_COUNT=255 bash tools/port/run.sh --test skins255-fixture255 ;;
esac
