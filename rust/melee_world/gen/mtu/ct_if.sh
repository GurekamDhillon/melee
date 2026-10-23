#!/usr/bin/env bash
# Builds and runs one tu_<stem> oracle test, printing only errors / the result:
#   bash rust/melee_world/gen/mtu/ct_if.sh <stem>        (worktree root)
cd "$(dirname "$0")/../../.." || exit 1
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p melee_world --test "tu_$1" 2>&1 \
  | grep -E "^error|test result|panicked|assertion|op [0-9]+ case" -A12 | head -${2:-80}
