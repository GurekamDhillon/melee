#!/bin/bash
# Regenerate the ftaction.c state oracle (run from anywhere).
set -e
cd "$(dirname "$0")/../../../.."
VIEWS=spawn_hitbox_0,spawn_hitbox_1,spawn_hitbox_2,spawn_hitbox_3,spawn_hitbox_4,spawn_hitbox_5,set_hitbox_x42_b57,set_throw_flags,set_jab_combo,set_throw_hitbox_0,set_throw_hitbox_1,set_throw_hitbox_2,unk27,set_tex_anim,part_anim,unk10,unk19
python rust/melee_fighter/gen/mtu/fill.py ftaction src/melee/ft/ftaction.c HSD_GObj,HSD_JObj,Fighter,Item,HSD_DObj Fighter,HSD_JObj,HSD_GObj,$VIEWS
python rust/melee_fighter/gen/mtu/stubs.py ftaction src/melee/ft/ftaction.c rust/melee_fighter/src/tu/ftaction.rs \
  --skip Command_Execute --skip Geno_FtCmd --ret "ftParts_GetBoneIndex=(int) (fv() % 16)|(fv() % 16) as i32"
python rust/melee_fighter/gen/cref/tu/run_tu.py ftaction_ops
