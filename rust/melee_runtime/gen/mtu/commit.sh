#!/bin/bash
# bash gen/mtu/commit.sh "<msg>" : stage everything under rust/melee_runtime except the WIP class files and commit
cd "$(dirname "$0")/../../../.."
git add -A rust/melee_runtime docs 2>/dev/null
git reset -q rust/melee_runtime/gen/mtu/lbspecs_class.py rust/melee_runtime/gen/mtu/tu/class_* rust/melee_runtime/tests/mtu/class_gen.rs rust/melee_runtime/src/tu/class.rs 2>/dev/null
git commit -q -m "$1

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" 2>&1 | grep -v warning
git log --oneline -1
