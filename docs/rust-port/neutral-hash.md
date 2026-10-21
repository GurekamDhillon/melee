# Byte-order-neutral gameplay hash (format NHV1)

The golden oracle for the native Rust port. The C build writes it with
`MELEE_NEUTRAL_HASHLOG=<csv>` (one row per live match frame, from the same hook as
`MELEE_TURBO_HASHLOG`, `gw_script.c`, after the frame's logic and script tick). The code is
`RB_NeutralHash` / `RB_NeutralHashWord` in `src/melee/ft/fighter.c` (under `TARGET_PC`).

## Why a second hash

`MELEE_TURBO_HASHLOG` hashes the raw snapshot bytes (big-endian guest memory, heap addresses,
render state): a native little-endian port can never reproduce it. `RB_GameHash` (the rollback
checksum) reads logical values but also mixes PC-only add-ons and, for CPUs, four raw bytes at
`cpu+0xF8`. The neutral hash is the RB_GameHash field set with every word defined by a C
*field value*, so a Rust port reading its own native structs produces the identical number.

Rules every word follows:
- Integers: the field's value converted to `u32` (C conversion: signed values are two's
  complement, e.g. `-1 -> 0xFFFFFFFF`; `u8`/`s8` zero/sign-extend per their C type first).
- Floats: the IEEE-754 bit pattern of the `f32` (`f32::to_bits`). No conversion, no rounding.
- No pointers, no addresses, no struct offsets, no raw memory.
- Bitfields: packed by the rule below, independent of the compiler's bitfield layout.

## Mixing function

```
mix(h, v):  h ^= v;  h = h * 0x01000193 (mod 2^32);  h ^= h >> 15;  return h
```
(All `u32`, wrapping. The same function as `ftRb_Mix` in `fighter.c`.)

## Game word (`hash` column)

```
h = 0x811C9DC5
h = mix(h, 0x4E485631)                 "NHV1": format version
h = mix(h, seed)                       *HSD_RandSeedPtr (the HSD_Rand state), read at log time
for player i in 0..6, for j in 0..2 (j=1: Ice Climbers' follower):
    if Player_GetEntityAtIndex(i, j) has a fighter:  h = mix(h, fighter_word(fp, slot = i*2 + j))
h = mix(h, 0x4954434E)                 "ITCN"
h = mix(h, item_count)
h = mix(h, item_sum)
```

## Fighter word (`s0`..`s11` columns), in this exact order

GC offsets are of `struct Fighter` (`src/melee/ft/types.h`) in the GameCube layout, recorded for
the m-ex offset map.

| # | value | C field | GC offset | type |
|---|---|---|---|---|
| - | start | `h = 0x811C9DC5` | | |
| 1 | `slot + 0x100` | (player*2 + follower) | | u32 |
| 2 | motion id | `motion_id` | +0x10 | FtMotionId (enum, int) |
| 3-5 | position x, y, z | `cur_pos` | +0xB0 | f32 x3 |
| 6-7 | self velocity x, y | `self_vel` | +0x80 | f32 x2 |
| 8 | percent | `dmg.x1830_percent` | +0x1830 | f32 |
| 9 | facing | `facing_dir` | +0x2C | f32 |
| 10 | ground/air | `ground_or_air` | +0xE0 | GroundOrAir (enum, int) |
| 11-12 | knockback velocity x, y | `x8c_kb_vel` | +0x8C | f32 x2 |
| 13 | ground velocity | `gr_vel` | +0xEC | f32 |
| 14 | hitlag frames | `dmg.x195c_hitlag_frames` | +0x195C | f32 |
| 15 | hitlag flag | `x2219_b5` | +0x2219 bit 5 (MSB=bit 0) | 1-bit |
| 16 | shield health | `shield_health` | +0x1998 | f32 |
| 17 | action frame counter | `x3E4_fighterCmdScript.frame_count` | +0x3E8 | f32 |
| 18 | jumps used | `x1968_jumpsUsed` | +0x1968 | u8 |
| 19 | L/R timer | `x67F` | +0x67F | u8 |
| 20 | L/R timer | `x680` | +0x680 | u8 |
| 21 | stocks | `Player_GetStocks(fp->player_id)` | (player table) | s32 |

Only when `ftCo_IsCpuControlled(fp)`, continuing the same `h` (`fp->cpu`, `struct CpuFighter`
at Fighter +0x1A88; offsets below are inside `CpuFighter`):

| # | value | C field | GC offset | type |
|---|---|---|---|---|
| 22 | `0x43505541` "CPUA" | | | |
| 23 | buttons | `buttons` | +0x0 | u32 |
| 24 | sticks | `(u8)lstick.x<<24 \| (u8)lstick.y<<16 \| (u8)cstick.x<<8 \| (u8)cstick.y` | +0x4, +0x6 | s8 x4 |
| 25 | triggers | `ltrigger<<8 \| rtrigger` | +0x8, +0x9 | u8 x2 |
| 26 | kind | `kind` | +0xC | CpuKind (enum, int) |
| 27 | level | `level` | +0x10 | int |
| 28-36 | `x14 x18 x1C x20 x24 x28 x2C x30 x34` | | +0x14..+0x34 | int x9 |
| 37-39 | `x38 x3C x40` | | +0x38..+0x40 | f32 x3 |
| 40-42 | `x7C x80 x84` | | +0x7C..+0x84 | int x3 |
| 43 | defend queue count | `xC8` | +0xC8 | u8 |
| 44 | attack queue count | `xEC` | +0xEC | u8 |
| 45 | command duration | `command_duration` | +0x44C | u32 |
| 46 | script read offset | `csP - buffer` (0 if outside `buffer[0x100]`) | +0x450 / +0x454 | u32 |
| 47 | flag word | see below | +0xF8..+0xFB | 32 bits |

Flag word: start `w = 0`; for each bitfield in declaration order
`xF8_b0:1 xF8_b12:2 xF8_b34:2 xF8_b5:1 xF8_b6:1 xF8_b7:1 xF9_b0..xF9_b7 (8 x :1)
xFA_b0:1 xFA_b1:1 xFA_b2:1 xFA_b34:2 xFA_b5:1 xFA_b6:1 xFA_b7:1 xFB_b0..xFB_b7 (8 x :1)`,
`w = (w << width) | (value & mask)`. (By construction this equals the GameCube big-endian
word at +0xF8, since the PPC ABI allocates bitfields MSB first.)

## Item sum (`items`, `itemsum` columns)

For each item on `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` with non-null `user_data`
(`struct Item`, `src/melee/it/types.h`):

```
ih = mix(0x4954454D, kind)            kind   +0x10  ItemKind
ih = mix(ih, msid)                    msid   +0x24  enum_t
ih = mix(ih, bits(pos.x/y/z))         pos    +0x4C  f32 x3
ih = mix(ih, bits(x40_vel.x/y/z))     x40_vel +0x40 f32 x3
item_sum += ih (wrapping); item_count += 1
```
The sum makes the word independent of list order. Raw `kind` is used (RB_GameHash's Geno stable
kind is a PC-only mod feature and is 0 for vanilla items anyway).

## Left out of RB_GameHash (PC-only add-ons; zero/absent in a vanilla match)

`ScriptGame_IntrWinHashWord` (Turbo window), `GenoDefine_StateDigest`,
`ScriptGame_BuildHashWord` (Envoy builds), `Geno_ArtStableKind`, the legacy-mode switch. If the
Rust port grows these features it should add them as a new format version (`NHV2`).

## CSV

`frame,hash,seed,items,itemsum,s0,...,s11` - `frame` is the script layer's `gs.match_frame`
(decimal), the rest 8-digit upper-case hex except `items` (decimal). An absent fighter slot is
`00000000`. A Rust frame driver must log at the same point: after the frame's logic, once per
live match frame.

## Caveats

- Not a full-state proof: it covers what decides the next frame for fighters and items, the same
  set rollback trusts. Stage state, camera and particles are not in it.
- The seed is read at log time, after the frame's render-side draws; a Rust port must therefore
  reproduce render-side `HSD_Rand` draws too, or this column (and `hash`) will diverge while the
  `sN` columns stay equal. That localises the problem, it does not hide it.
