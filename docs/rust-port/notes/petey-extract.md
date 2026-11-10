# Petey Piranha: how the boss data was found and is loaded

Replaces the old owner-run extraction guide. The owner extracted their own Brawl disc (read-only) into a
git-ignored local folder outside this repository; nothing from it is in git, and no path to it is recorded here.

## Where Petey is

- Not the trophy BRRES under `toy/fig` and not `effect/enemy` (effects only). The boss rig lives in the adventure
  stage archive `stage/adventure/030101.pac` (ARC; its members are LZ77 type 0x10/0x11 compressed ARCs).
- Inside, one sub-archive holds the boss: a BRRES with MDL0 `EnmBosspackun` (v9; 31 bones, 3 polygons, three
  Lambert materials, textures `EnmBosspackunA` / `EnmBosspackunB`, 512x512 CMPR with mips), two more MDL0s
  `WpnBosspackunArmL` / `WpnBosspackunArmR` (the arms carrying the cages), the TEX0 set, and CHR0 animations
  `EnmBosspackun{Wait1,Wait2,Angry,BasketAttackL,BasketAttackR,DamageLeft,DamageRight,Down,JumpStart,JumpLoop,
  JumpAttack,JumpAttackLanding}` (plus the matching `WpnBosspackunArm*` ones and `...PeachOff`). Effects (REFF/REFT)
  sit beside them.
- A params blob named `BossPackun` (about 28 KB) is in the same archive; `module/sora_enemy.rel` has
  `emBosspackun` / `emBosspackunParamAccesser` (the code reading it) and `sora_adv_stage.rel` the
  `ENEMY_EVENT_BOSSPACKUN_DEAD_LEFT/RIGHT` events.

## Numbers (HP not confirmed)

`emBosspackunParamAccesser` was traced in `sora_enemy.rel` (2026-11-09): its type name is referenced from its RTTI
record, whose vtable lists three getters. Each fetches the boss's param block (the blob above, through the enemy's
resource handle, `+0x2C`) and switches on a numeric parameter id:

| getter | ids | reads |
|---|---|---|
| float | 0xFA0 | `+0x150` (0xFA1 / 0xFA2: a ratio computed from two live fighter fields, not the blob) |
| int | 0x5DC0, 0x5DC1, 0x5DC2 | `+0x144`, `+0x148`, `+0x14C` |
| sub-block pointer | 16 ids from 0xABE0 (jump table) | `+0x100`, `+0x124`, `+0x154`, ... `+0x314` |

None of these is named, and HP is not among them as far as this accessor shows: Brawl's enemies take HP from the
common enemy parameters read by the shared enemy code, which was not traced. So HP and cage HP are **not
confirmed** and the stand-ins stay (`petey.rs`: HP 300, cage 70).

## Loading (runtime, presentation only)

`rust/melee_modes/src/tu/petey_model.rs`: the folder is `MELEE_BRAWL_LOCAL`, else the first `_build/brawl` found
walking up from the working folder or the exe (`MELEE_PETEY_NO_MODEL` forces the placeholder). It reads the stage
archive on a helper thread (system heap, outside the rollback arena), unpacks ARC + LZ77 in memory, finds the BRRES
members by name, decodes the textures (CMPR and the plain formats) and the MDL0 display lists, and skins the
vertices per drawn frame from the CHR0 picked by the sim state (all twelve `<model><Name>` animations are kept,
Hermite keys and baked tracks both read): a vertex on one bone is stored in that bone's space, a
NodeMix vertex in model space (weights x world x inverse bind); the matrix slots come from the display list's
indexed loads (XF address in the low 12 bits). It keeps a triangle list with per-vertex colour (texel x vertex
colour x a fixed light). The draw hook (`petey::draw`) puts the body at Petey's sim position scaled to the placeholder height
and the arm models at the cage positions. Nothing feeds back into the sim, so replays / rollback are unaffected;
without the folder the capsule placeholder draws as before.

Not done: materials beyond one texture per polygon, effects.
