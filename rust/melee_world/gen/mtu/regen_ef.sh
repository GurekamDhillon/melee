#!/bin/bash
# Regenerate rust/melee_world/src/tu/efn/{eflib,efasync,efdata,efalt,efsync}.rs from the game's C (libclang, see c2rs.py).
# Run from the worktree root. Hand-structured functions live in gen/mtu/ef/hand/. Afterwards run
# gen/rewire/rewire.py to wire the ext callees that have native ports.
set -e
G=rust/melee_world/gen/mtu
SIB=eflib:src/melee/ef/eflib.c,efasync:src/melee/ef/efasync.c,efdata:src/melee/ef/efdata.c,efalt:src/melee/ef/efalt.c,efsync:src/melee/ef/efsync.c
mkdir -p $G/ef/hand
for s in eflib efasync efdata efalt efsync; do
  python $G/c2rs.py src/melee/ef/$s.c rust/melee_world/src/tu/efn/$s.rs --head $G/ef/head_$s.rs --hand $G/ef/hand --sib $SIB --pub-all --all-extern --check
done
# wire the callees that have native ports (lbArchive_80017040: the hand-port variant takes different varargs)
python rust/melee_world/gen/rewire/rewire.py --skip=lbArchive_80017040 | grep -v ": 0 sites" || true
