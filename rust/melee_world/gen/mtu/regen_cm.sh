#!/bin/bash
# Regenerate rust/melee_world/src/tu/cmcamera.rs from the game's C (libclang, see c2rs.py). Run from the
# worktree root. Hand-structured functions live in gen/mtu/cm/hand/. Afterwards run
# gen/rewire/rewire.py to wire the ext callees that have native ports.
set -e
G=rust/melee_world/gen/mtu
python $G/c2rs.py src/melee/cm/camera.c rust/melee_world/src/tu/cmcamera.rs --head $G/cm/head_camera.rs --hand $G/cm/hand --pub-all --check
