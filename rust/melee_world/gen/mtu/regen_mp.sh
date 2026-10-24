#!/bin/bash
# Regenerate rust/melee_world/src/tu/{mplib,mpcoll,mpisland}.rs from the game's C (libclang, see c2rs.py).
# Run from the worktree root. Hand-structured goto functions live in gen/mtu/hand/.
set -e
G=rust/melee_world/gen/mtu
SIB=mplib:src/melee/mp/mplib.c,mpisland:src/melee/mp/mpisland.c,mpcoll:src/melee/mp/mpcoll.c
for s in mplib mpcoll mpisland; do
  python $G/c2rs.py src/melee/mp/$s.c rust/melee_world/src/tu/$s.rs --head $G/mp/head_$s.rs --hand $G/hand --sib $SIB --check
done
