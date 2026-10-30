#!/bin/bash
# Regenerate the ftcoll.c world-level state oracle (run from anywhere).
set -e
cd "$(dirname "$0")/../../../.."
SK=""
for n in EchoCapsuleCount EchoCapsule EchoIsCapsule EchoBlocked EchoSameGroup EchoTargetAllowed EchoConnected EchoPrepare EchoHitIndex EchoReportContext EchoCredit EchoSourceX; do SK="$SK --skip ScriptGame_$n"; done
python rust/melee_fighter/gen/mtu/fill.py ftcoll src/melee/ft/ftcoll.c HSD_GObj,HSD_JObj,Fighter,Item,HSD_DObj Fighter,HSD_JObj,HSD_GObj,ftCommonData
python rust/melee_fighter/gen/mtu/stubs.py ftcoll src/melee/ft/ftcoll.c rust/melee_fighter/src/tu/ftcoll.rs $SK --skip efSync_Spawn --skip atanf --skip atan2f --libhook ft_PlaySFX=PLAY_SFX
python rust/melee_fighter/gen/cref/tu/run_tu.py ftcoll_ops
