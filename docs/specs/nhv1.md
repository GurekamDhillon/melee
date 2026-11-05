<!-- SPDX-License-Identifier: MIT
     Copyright (c) 2026 GurekamDhillon -->
# NHV1: the byte-order-neutral per-frame gameplay hash

Spec version: NHV1 (format tag `0x4E485631`), document revision 2026-11-02 (rev 2: melee-unlocked's answers folded in). Derived from our source only:
`src/melee/ft/fighter.c` (`nh_*`, `RB_NeutralHash`), `src/melee/gm/gmscene.c` (`gm_801A4D34`),
`pc/platform/gw_script.c` (the CSV writer), `rust/melee_types/src/neutral_hash.rs`,
`tools/rust-port/compare_golden.py`, `docs/rust-port/golden/`. It supersedes `docs/rust-port/neutral-hash.md`
for anything the two disagree on (that file keeps its stale "render draws" caveat; section 8 here is current).
Reference code: `docs/specs/tools/nhv1_ref.py` (pure Python; self-test recomputes all 10,804 golden `hash` cells and
the 8 C-oracle vectors, all pass).

## 0. Agreed with melee-unlocked

hero88go (maintainer of melee-unlocked, a decomp-faithful C/C++ source port) accepted NHV1 as written. Rows he has agreed are marked
`agreed (melee-unlocked)`; the rest keep their earlier status.

| # | item | decision | status |
|---|---|---|---|
| 1 | hash function, field set, field order, CPU block, item sum | unchanged from NHV1 (sections 4-6) | agreed (melee-unlocked) |
| 2 | CSV layout | exactly section 7; `tools/rust-port/compare_golden.py` reads either build's file unchanged | agreed (melee-unlocked) |
| 3 | log point | in `gm_801A4D34`, immediately after `HSD_GObj_RunProcs()` returns and before `gm_80479D58.unk_0++` (section 2) | agreed (melee-unlocked): he hooks `gmscene` at the same place |
| 4 | `frame` column | `gm_80479D58.unk_0` (scene frame counter) at the log point, minus its value at the first logged row (so row 0 is frame 0) (section 3) | agreed (melee-unlocked): his H is 0, because he has no held iterations (he will confirm with the probe). OPEN: our build's H not measured |
| 5 | `seed` column | `*HSD_RandSeedPtr` at the log point (end of logic); unchanged number in our goldens | agreed (melee-unlocked) |
| 6 | second seed column `seed_post` | the seed after the frame's render/draw pass (= start of the next iteration), appended after `s11`; optional (section 8) | proposed; melee-unlocked's render draws advance the shared seed, so his `seed_post != seed` whenever a frame drew (section 8) |
| 7 | render-side `HSD_Rand` draws | our build isolates them on a separate stream; a hardware-faithful build does not. `seed_post` makes the difference visible (section 8) | agreed (melee-unlocked): his build has hardware semantics (render draws advance the shared seed, as on console); ours is the divergence, and his Dolphin oracle will match HIS seed column, not ours. OPEN row 16 (section 14) asks whether we add a hardware-faithful seed mode |
| 8 | cross-build input | `.slp` replays for non-CPU scenarios, pad scripts for `cpu_*` scenarios; Dolphin memory read is a third oracle (section 9) | agreed in principle |
| 9 | `compare_golden.py` | gains `--common` (compare the columns both files have, by name) so `seed_post` or NHV2 columns do not break a comparison | done (this commit) |
| 10 | NHV2 stage state | candidate fields in section 12; versioned separately, not part of NHV1 | proposed |
| 11 | line terminator | writers: LF or CRLF; readers must accept both (our C build on Windows writes CRLF, the Rust writer LF) | agreed (melee-unlocked) |
| 12 | first exchange | `pad_falcon_fox_fd` frame 0 `hash` = `FC467705`, which is our golden's value (section 11 recomputes it) | agreed (melee-unlocked): his first exchange |
| 13 | Geno state digest | optional trailing columns `geno0`..`geno3` (section 8.1); implemented in our C build behind `MELEE_NEUTRAL_HASHLOG` (+ `MELEE_NEUTRAL_HASHLOG_GENO=1`) | proposed (asked for by melee-unlocked) |

## 1. What it is

One 32-bit word per live match frame (`hash`) plus its parts: seed, item count, item sum and one word per fighter
slot. The parts let a comparison name the first divergent frame and the fighter (or items, or the RNG) that diverged.
Everything is a logical C field value; no pointer, address, struct offset or raw memory byte is hashed, so a build
with any struct layout or byte order produces the same numbers if its field values are equal.

## 2. Log point

| | where | before / after |
|---|---|---|
| decomp boundary (normative) | `gm_801A4D34` (the scene loop, `src/melee/gm/gmscene.c`), once per logic iteration that runs the fighters | **after** `HSD_GObj_RunProcs()` returns (all fighter, item, stage, camera procs of this frame have run); **before** the render pass and **before** `gm_80479D58.unk_0++` |
| our C build today | `gw_Script_FramePost()` (`pc/platform/gw_script.c`), called from `Script_FramePost()` right after `HSD_GObj_RunProcs()` and four port-only calls (`Netplay_Background`, `Camera_RefreshViewingMtx`, `ifMagnify_UpdateLogicOffscreen`, `HSD_ObjAllocTopUp`); the CSV row is written at the end of it | the same boundary. The port-only calls touch no hashed field. The script layer's `on_frame` hooks run between the boundary and the sample; in every golden run `MELEE_SCRIPTS=0` leaves only the pad script, which writes inputs, never hashed state |
| our Rust engine today | the `gmscene::ext::FRAME_POST` hook (`rust/melee_engine/src/engine.rs`: `compute_row`), the port of the same call site | the same boundary |

So the decomp-exact definition needs no script layer: sample directly after `HSD_GObj_RunProcs()` in `gm_801A4D34`.
Nothing is hashed in iterations that do not run the fighters: our loading-screen hold (`mnLoadScreen_Frame`, port only)
skips `Script_FramePost`, `Replay_Tick` and the log; the decomp has no such hold.

## 3. Frame numbering

| column value | definition |
|---|---|
| `frame` (proposed canonical) | `gm_80479D58.unk_0` (`struct gm_80479D58_t`, `u32`, reset to 0 at `gm_801A4D34` entry, incremented once at the end of every iteration, saturating guard `!= -2`) sampled at the log point, minus its value at the first logged row |
| `frame` (our column today) | `gs.match_frame` (`gw_script.c`): 0 on the first `Script_FramePost` that finds a fighter present (sets `match_active`), +1 on every later one |
| relation | `match_frame = unk_0 - H`, H = number of iterations before the first row, i.e. port-only held iterations (`unk_0` still increments in them) plus iterations with no fighter. In a decomp-exact build H = 0 and the two are equal. **OPEN:** H for our build was not measured (needs a one-line probe of `unk_0` at the first row) |
| under a replay | `Replay_Tick` also runs only in non-held iterations and makes the Slippi frame -123 on the first one, so `frame = slippi_frame + 123` |
| `frame` 0 | the first iteration in which fighters exist: spawn, before the ready/go animation has ended (the fighters are still falling in; the Hero fixture shows y = 10 at frame 0 and landing near frame 90). It is **not** the timer start |

A row exists for every frame 0..N with no gaps; a duplicate frame number is a writer error (`compare_golden.py` keeps both and reports).

## 4. Primitive rules

| rule | detail |
|---|---|
| word | everything is mixed as a `u32` |
| integers | the field's C value converted to `u32`: signed values two's complement (`-1 -> 0xFFFFFFFF`), `u8`/`s8`/`s16` zero- or sign-extended by their C type first. Exception: the stick bytes in the CPU block are taken `(u8)` |
| floats | the IEEE-754 binary32 bit pattern of the `f32` (`ftRb_Bits`, a union read). **No canonicalisation**: `-0.0` hashes as `0x80000000` (not `0`), a NaN hashes its exact payload, denormals as they are. The worked example has two `-0.0` words |
| float arithmetic | the values must be bit-exact results of Melee's f32 arithmetic: no fused multiply-add, no x87 extended precision, no fast-math (our C build compiles with `-ffp-contract=off`; see `docs/rust-port/notes/fma.md`). A one-ulp difference in position is a divergence |
| byte order | none: words are values, never bytes. In the CSV they are printed as hex numbers (section 7) |
| bitfields | packed by the rule in 5.3, never read as raw memory |
| pointers | never hashed; the only pointer-derived word is the CPU script offset (5.3 word 46) |

## 5. Hash

### 5.1 Function

```
mix(h, v):  h ^= v;  h = h * 0x01000193 (mod 2^32);  h ^= h >> 15;  return h       (u32, wrapping)
```
Seeds: fighter word starts at `0x811C9DC5`; item word starts at `0x4954454D` ("ITEM"); game word starts at `0x811C9DC5`.

### 5.2 Game word (`hash` column)

```
h = 0x811C9DC5
h = mix(h, 0x4E485631)                       "NHV1"
h = mix(h, seed)                             seed = *HSD_RandSeedPtr (the pointer's target; a stage may redirect it)
for i in 0..5 (player slot), j in 0..1 (j=1: follower, e.g. Nana):
    g = Player_GetEntityAtIndex(i, j)        = player_slots[i].player_entity[player_slots[i].transformed[j]]
    if g != NULL and g->user_data != NULL:   h = mix(h, fighter_word(g->user_data, slot = i*2 + j))
h = mix(h, 0x4954434E)                       "ITCN"
h = mix(h, item_count)
h = mix(h, item_sum)
```
Absent slots are skipped (nothing is mixed for them). `Player_GetEntityAtIndex` returns the active transformed form
(Zelda/Sheik). `player_slots` is 0x80453080 on the GameCube (`config/GALE01/symbols.txt`), 6 slots.

### 5.3 Fighter word (`s0`..`s11`), in this exact order

`h = 0x811C9DC5`, then `h = mix(h, word)` for each word. GC offsets are of `struct Fighter` (`src/melee/ft/types.h`).

| # | word | C field | GC offset | type |
|---|---|---|---|---|
| 1 | `slot + 0x100` | slot = player*2 + follower | | u32 |
| 2 | motion id | `motion_id` | +0x10 | int enum |
| 3-5 | position x, y, z | `cur_pos` | +0xB0 | f32 x3 |
| 6-7 | self velocity x, y | `self_vel` | +0x80 | f32 x2 |
| 8 | percent | `dmg.x1830_percent` | +0x1830 | f32 |
| 9 | facing | `facing_dir` | +0x2C | f32 |
| 10 | ground/air | `ground_or_air` | +0xE0 | int enum |
| 11-12 | knockback velocity x, y | `x8c_kb_vel` | +0x8C | f32 x2 |
| 13 | ground velocity | `gr_vel` | +0xEC | f32 |
| 14 | hitlag frames | `dmg.x195c_hitlag_frames` | +0x195C | f32 |
| 15 | hitlag flag | `x2219_b5` | +0x2219 bit 5 (MSB = bit 0) | 1 bit |
| 16 | shield health | `shield_health` | +0x1998 | f32 |
| 17 | script frame counter | `x3E4_fighterCmdScript.frame_count` | +0x3E8 | f32 |
| 18 | jumps used | `x1968_jumpsUsed` | +0x1968 | u8 |
| 19 | L/R timer | `x67F` | +0x67F | u8 |
| 20 | L/R timer | `x680` | +0x680 | u8 |
| 21 | stocks | `Player_GetStocks(fp->player_id)` | player table | s32 |

Only when `ftCo_IsCpuControlled(fp)`, continuing the same `h` (`fp->cpu`, `struct CpuFighter` at Fighter +0x1A88):

| # | word | C field | offset in CpuFighter | type |
|---|---|---|---|---|
| 22 | `0x43505541` ("CPUA") | | | |
| 23 | buttons | `buttons` | +0x0 | u32 |
| 24 | sticks | `(u8)lstick.x<<24 \| (u8)lstick.y<<16 \| (u8)cstick.x<<8 \| (u8)cstick.y` | +0x4, +0x6 | s8 x4 |
| 25 | triggers | `ltrigger<<8 \| rtrigger` | +0x8, +0x9 | u8 x2 |
| 26 | kind | `kind` | +0xC | int enum |
| 27 | level | `level` | +0x10 | int |
| 28-36 | `x14 x18 x1C x20 x24 x28 x2C x30 x34` | | +0x14..+0x34 | int x9 |
| 37-39 | `x38 x3C x40` | | +0x38..+0x40 | f32 x3 |
| 40-42 | `x7C x80 x84` | | +0x7C..+0x84 | int x3 |
| 43 | defend queue count | `xC8` | +0xC8 | u8 |
| 44 | attack queue count | `xEC` | +0xEC | u8 |
| 45 | command duration | `command_duration` | +0x44C | u32 |
| 46 | script read offset | `csP - buffer` if `buffer <= csP <= buffer + 0x100` (one past the end allowed), else 0 | +0x450 / +0x454 | u32 |
| 47 | flag word | below | +0xF8..+0xFB | 32 bits |

Flag word: `w = 0`; for each bitfield in this declaration order, `w = (w << width) | (value & ((1<<width)-1))`:
`xF8_b0:1 xF8_b12:2 xF8_b34:2 xF8_b5:1 xF8_b6:1 xF8_b7:1`, `xF9_b0..xF9_b7` (8 x :1),
`xFA_b0:1 xFA_b1:1 xFA_b2:1 xFA_b34:2 xFA_b5:1 xFA_b6:1 xFA_b7:1`, `xFB_b0..xFB_b7` (8 x :1). Total 32 bits; equals the
GameCube big-endian word at +0xF8 (PPC allocates bitfields MSB first).

### 5.4 Item word and sum (`items`, `itemsum`)

For each item on `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]` (when the head is non-null) whose `user_data` is non-null
(`struct Item`, `src/melee/it/types.h`):

```
ih = mix(0x4954454D, kind)            kind    +0x10   ItemKind (raw; Geno article kinds are NOT remapped)
ih = mix(ih, msid)                    msid    +0x24
ih = mix(ih, bits(pos.x))  ih = mix(ih, bits(pos.y))  ih = mix(ih, bits(pos.z))        pos     +0x4C
ih = mix(ih, bits(x40_vel.x))  ... y  ... z                                              x40_vel +0x40
item_sum = (item_sum + ih) mod 2^32;   item_count += 1
```
The sum makes the result independent of list order. `items` is `item_count` (decimal in the CSV).

## 6. Left out (and why)

| not hashed | reason |
|---|---|
| stage state, camera, particles, sound | NHV1 covers what decides the next frame for fighters and items; stage state is proposed as NHV2 (section 12) |
| PC-only add-ons of `RB_GameHash` (Turbo window word, `GenoDefine_StateDigest`, Envoy build word, Geno article stable kinds, legacy-mode switch) | zero or absent in a vanilla match; a build that grows them must define a new version, not extend NHV1 |
| projectile items that are not on the item p-link | not in the sum |

## 7. CSV layout (exact)

```
frame,hash,seed,items,itemsum,s0,s1,s2,s3,s4,s5,s6,s7,s8,s9,s10,s11
0,FC467705,9D38619F,0,00000000,992E57A6,00000000,818E79DE,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000,00000000
```

| column | value | format |
|---|---|---|
| `frame` | section 3 | decimal, may be negative only if a build logs pre-match frames (ours does not) |
| `hash` | game word 5.2 | 8 upper-case hex digits, no prefix |
| `seed` | `*HSD_RandSeedPtr` at the log point | 8 upper-case hex |
| `items` | `item_count` | decimal |
| `itemsum` | `item_sum` | 8 upper-case hex |
| `s0`..`s11` | fighter word of slot k = player*2 + follower | 8 upper-case hex; an absent slot is `00000000`. A present fighter whose word is exactly 0 is indistinguishable from absent (probability 2^-32; the Rust reader treats 0 as absent) |

Header: exactly the first line above; readers reject a file whose first two header cells are not `frame,hash`
(`compare_golden.py` exit 2). Separator `,`; no quoting; no trailing comma; one row per frame, ascending; terminator LF or CRLF
(accept both); a UTF-8 BOM is not written. Optional trailing columns (section 8, 12) are appended after `s11` and
named; a comparison of files with different headers needs `compare_golden.py --common`.

## 8. Seed, pre-draws, render draws

| term | definition |
|---|---|
| `seed` (column) | `*HSD_RandSeedPtr` at the log point: the logic stream after this frame's logic |
| `seed_post` (proposed column, after `s11`) | the same pointer's target at the **start of the next iteration of `gm_801A4D34`, before `Replay_Tick`/`Replay_ResyncSeed`** (i.e. after this frame's render/draw pass); on the last row, at the end of the scene. In our C build that point is the `Replay_NoteSeed(*HSD_RandSeedPtr)` call, so the data is already sampled there |
| render-side draws | `src/sysdolphin/baselib/random.c` (commit `0f3376ee3`, 2026-10-09): inside `HSD_RandRenderBegin/End` (`gobj.c` render brackets; `HSD_JObjDisp` creating `SPtcl` generators) draws use a private `render_seed` copied from the logic seed and zeroed at the end. **They never advance `*HSD_RandSeedPtr`.** On hardware, and in a decomp-faithful build without that change, they do. The goldens (2026-10-20) were recorded with the isolation |
| consequence | in our build `seed_post == seed` by construction (**OPEN**: not yet confirmed by a probe, because the port-only calls between the boundary and the next iteration were not audited for draws). In a hardware-semantics build `seed_post != seed` whenever the frame's render pass drew. Emitting both columns separates "the simulation agrees" (`s0..s11`, `items`, `itemsum`) from "render draws shifted the stream" (`seed`, `seed_post`, `hash`) |
| pre-draws | logic-stream `HSD_Rand` draws made after scene start and before the first log point, by subsystems the Rust engine does not run yet (the memory-card trophy draw, stage init: `grLast_8021AC30` x4 on FD, `grBattle_BG_Callback0` x1 on BF, the stage BGM pick `Stage_80225074 -> Ground_801C24F8` x1). They are inside `seed` at frame 0. `tools/rust-port/engine/pre_draws.tsv` (FD 1, BF 1 today) makes the Rust engine do that many dummy draws before its first tick so the `seed` column compares. In column terms: `pre_draws` = the number of LCG steps (`s = s*214013 + 2531011`) between the seed the build holds at the scene start and `seed` at frame 0, minus the draws the build makes itself. Match setup also draws (FoD: `grIzumi_801CC358` x2, `ftCo_800A101C` x2, `ftCo_800B9704` x2): Slippi restores the Game Start seed after them (`Replay_ResyncSeed`, `gw_replay.c`) |
| replays | a replay forces the seed every frame (`seed + ((frame+123)<<16)` online, the file's Frame Start seed offline) before the logic runs, so `seed` and `hash` cannot show RNG-count divergence; compare with `--ignore-seed` (skips `hash` and `seed`) |
| netplay variant | `RB_NeutralHashForFrame(frame)` hashes with the seed the frame will consume (`Replay_ChecksumSeed`); not a CSV column |

**Hardware semantics and our divergence (decided with melee-unlocked, 2026-11-02).** melee-unlocked's render draws advance the shared seed exactly as on a console, so in
his build `seed_post != seed` whenever a frame drew, and a Dolphin memory oracle will match HIS `seed` column, not ours. Our render-draw isolation (private stream since `0f3376ee3`)
is therefore a **divergence from hardware**, recorded here as ours (not a defect of his). The simulation is not affected (render draws feed no logic in either build);
only the number in `seed`, and so `hash` (which mixes `seed`), differs after the first frame that drew.

### Seed comparison across builds

| situation | do this |
|---|---|
| two builds with the same render-draw semantics | compare normally; `seed`, `hash` and `seed_post` must all agree |
| ours vs melee-unlocked (or Dolphin), any scenario | `python tools/rust-port/compare_golden.py ours.csv theirs.csv --common --ignore-seed` (skips `hash` and `seed`; `s0..s11`, `items`, `itemsum` still decide) |
| ours vs melee-unlocked, wanting `seed`/`hash` too | compare those two columns only at frames with no render draws (in the hardware-semantics file: rows where nothing drew before the log point, e.g. the first row, which precedes any render pass) |
| `--common` | compares the columns both files have, by name, so a `seed_post` or `geno0..3` column on one side does not break the comparison |

If a simulation column disagrees while `seed` also disagrees, first rule out a pre-draw count (the `pre_draws` row above) before suspecting the simulation.

### 8.1 Optional trailing columns `geno0`..`geno3` (Geno variable digest)

Trailing-column rules are those of section 7: appended after `s11` (after `seed_post` when both are present, then NHV2 `st*`), named in the header, compared with `--common`.
Our C build emits them only when `MELEE_NEUTRAL_HASHLOG_GENO=1` is set next to `MELEE_NEUTRAL_HASHLOG`, so the four committed goldens (headers ending at `s11`) are unchanged.
One column per player port 0..3 (`geno<port>`), 8 upper-case hex digits, sampled at the same log point as the rest of the row (section 2).

The word of port `i` comes from the Geno state block of the fighter object `Player_GetEntityAtIndex(i, 0)` (the main fighter in its active transformed form; a follower such as Nana is not covered):

```
fighter absent:                                                       word = 0
block.profile < 0 AND (block.flags & 1) == 0   (GENO_SF_SCRIPT):       word = 0     (inert: no Geno profile and the script escape never ran)
h = 0x811C9DC5
h = mix(h, 0x474E5631)                      "GNV1"      (mix is NHV1's: h ^= v; h *= 0x01000193; h ^= h >> 15)
for k in 0..63: h = mix(h, la_i[k])         long-term ints,    s32 taken as u32
for k in 0..63: h = mix(h, bits(la_f[k]))   long-term floats,  binary32 bit pattern, no canonicalisation (NHV1 section 4)
for k in 0..63: h = mix(h, ra_i[k])         per-action ints
for k in 0..63: h = mix(h, bits(ra_f[k]))   per-action floats
for k in 0..7:  h = mix(h, move_i[k])       behaviour per-move ints   (MOVE_I0..7)
for k in 0..7:  h = mix(h, bits(move_f[k])) behaviour per-move floats (MOVE_F0..7)
h = mix(h, action_time)                     ACTION_FRAME, s32 (1 on the first frame of an action)
word = h
```
Not hashed: the diagnostics (`resets`, `hook_calls`, `changes`, `state_entries`, `art_spawned`, `counters`, `motion_land`), the change-action checks, the rehit/link/stun tables and the root-motion scratch. Only the variable banks and the action clock.
Block offsets (`docs/specs/geno-v5.md` 8.2): `la_i` +16, `ra_i` +272, `la_f` +528, `ra_f` +784, `move_i` +1564, `move_f` +1596, `action_time` +1048.
An active fighter whose word is exactly 0 reads as inert (probability 2^-32). Geno state is not part of the `hash` column (section 6). Reference code: `docs/specs/tools/nhv1_ref.py` `geno_digest()`. Checked: `docs/specs/fixtures/geno-v5/nhv1_hero_jab_geno.csv` (the hero-jab run with `MELEE_NEUTRAL_HASHLOG_GENO=1`; columns up to `s11` equal `nhv1_hero_jab.csv`, `compare_golden.py --common`) has all 301 `geno0` words equal to `geno_digest_zero(action_time)` for a plausible action clock; `geno1..3` are `00000000` (no Geno fighter on those ports).

## 9. Replays and pad scenarios

| scenario kind | input | used for | frame mapping |
|---|---|---|---|
| `pad_*` | a pad script, `tools/rust-port/golden/pad_fixed.lua`, frame-indexed (`gd.input`, 2-frame holds: a sample at match frame n is read on frame n+1) | the committed goldens `docs/rust-port/golden/pad_*.csv` | the pad script's counter `n` advances once per `on_frame_pre` with the match active, which first happens in the iteration after row 0; so script sample `n` is acted on in CSV frame `n+1` (a 2-frame hold, see the comment in `pad_fixed.lua`) and the state the script reads at call `n` is the end-of-logic state of CSV frame `n-1` (inferred from where `match_active` is set; not separately probed) |
| `cpu_*` | no human input: CPUs run the AI (`pad_idle.lua` only quits at frame 3600) | `cpu_fox_falco_dl`, `cpu_peach_samus_ys_items`; items/AI words (CPU block) | none needed. **A replay cannot replace these**: it drives CPU slots with recorded inputs and never runs the AI, so the CPU block (5.3 words 22-47) would be empty |
| `.slp` replay | Slippi post-frame inputs (processed sticks/buttons) per port, restored at `Fighter_Spaghetti_8006AD10` (the point `RestoreGameFrame.asm` writes), plus Frame Start seed per frame | the cross-build input for human scenarios (ours, melee-unlocked, and Dolphin) | `slippi_frame = frame - 123` (first fighter-running frame is -123); frames before it do not exist in the file |

Our replay path today: `MELEE_SLP=<file> MELEE_SLP_RESYNC=1 MELEE_NEUTRAL_HASHLOG=<csv>` (C) and `melee golden --slp <file> --out <csv>` (Rust,
`rust/melee/src/main.rs: slp_open`: Falcon/Fox/Marth/Sheik on FD/BF, two humans). `tools/rust-port/golden/record_slp.sh`
records a pad scenario to `.slp` (`MELEE_SLP_RECORD`) and plays it back (`<name>.play.csv`); `check_slp.sh` runs Rust over
`docs/rust-port/golden/slp/pad_falcon_fox_fd.slp` and its whole-match `.gdlp` and compares with `compare_golden.py --allow-short --ignore-seed`.
Measured facts: the committed `.slp` plays back in the C build with the same `s0..s11` as the pad run at frame 0 (`ABDB2EDD` vs `FC467705` differ only because the
seed is forced), and the control run is identical over 1801 frames. External tournament files (Slippi 3.17, UCF dashback and
shield drop on in every file) have 0 of 5 matching: port layout (the C hash keeps the file's port as the slot, the Rust player compacts to s0/s2), rules/costume/stocks not
applied from the file, and UCF is not in the vanilla sim (`docs/rust-port/notes/replays-ext.md`). A replay used as a cross-build input must therefore be recorded
from a vanilla (non-UCF) run of one of the two builds.

Third oracle (Dolphin): read the same fields from GameCube memory at the offsets in 5.3 and 5.4 after each frame's `HSD_GObj_RunProcs` equivalent
(an emulator breakpoint at that point, or the Slippi frame-end hook) and run the same `nhv1_ref.py` words over them. Offsets: `player_slots` 0x80453080, then `player_entity[]`/`transformed[]` to the
fighter GObj, `user_data` to `struct Fighter`; items from `HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]`. Built 2026-11-05: `tools/rust-port/dolphin/oracle.sh`, docs/rust-port/notes/dolphin-oracle.md (an HLE hook at 0x801A4FA4, the instruction after `bl HSD_GObj_RunProcs` in `gm_801A4D34`).

## 10. Comparing two runs

`python tools/rust-port/compare_golden.py A.csv B.csv [--ignore-seed] [--allow-short] [--from N] [--common] [--max-report N]`

| step | rule |
|---|---|
| load | first two header cells must be `frame,hash`; rows keyed by frame |
| headers | must be identical, or `--common` (compare the columns both have, by name) |
| per frame | all columns after `frame` compared as strings; `--ignore-seed` skips `hash` and `seed` (replays) |
| first divergence | the lowest frame whose row differs; the report lists the differing columns: `seed`, `items`/`itemsum`, or slot `sN` (N = player*2 + follower), then each column's two values |
| localise the field | `hash` differs but every `sN`, `items`, `itemsum` equal: the seed changed (`seed`). An `sN` differs: recompute that slot's 21 (+27) words in each build (`nhv1_ref.py fighter_inputs`, or the Rust `MELEE_NH_DUMP=<slot>` / C `rp_ecb_log` dump of the words) and compare word by word; the first differing word names the field (table 5.3) |
| exit | 0 identical; 1 mismatch; 2 bad input. `--print-count` prints how many frames match consecutively from the first row |

## 11. Worked example

Computed by `python docs/specs/tools/nhv1_ref.py --worked`; the fighter and item words equal the C oracle
(`rust/melee_types/tests/data/nh_expected.txt`, produced by compiling `fighter.c`'s own `nh_*` functions), and the game word is
the committed golden's frame 0.

Fighter, oracle input k = 0 (slot 0, not CPU, stocks 3; position (0, **-0.0**, 0), velocity (0, -1), kb velocity (0, **-0.0**), shield 60, `x67F` 1, `x680` 2):

| word | value | h after |
|---|---|---|
| start | | 811C9DC5 |
| slot+0x100 | 00000100 | 050AC00A |
| motion_id | 00000010 | 09EC7B36 |
| pos.x | 00000000 | D5445C89 |
| pos.y | **80000000** (-0.0, not canonicalised) | C39C2C90 |
| pos.z | 00000000 | 7EDADB04 |
| vel.x | 00000000 | B687AA41 |
| vel.y | BF800000 | 7811F471 |
| percent | 00000000 | 75432764 |
| facing | 3F800000 | 15372802 |
| ground_or_air | 00000000 | 67D33481 |
| kb.x | 00000000 | F27A43E4 |
| kb.y | **80000000** | 1A78EB1D |
| gr_vel | 00000000 | C95B8C13 |
| hitlag | 00000000 | 0E1D9DD2 |
| x2219_b5 | 00000000 | 0A9F64A8 |
| shield | 42700000 | 78DB85CE |
| cmd frame_count | 00000000 | 0F93BC6D |
| jumps used | 00000000 | F2907AB4 |
| x67F | 00000001 | 8E70360D |
| x680 | 00000002 | 49A58AD7 |
| stocks | 00000003 | **C3980C8F** = fighter word (oracle `C3980C8F`) |

Item, oracle k = 0 (kind 0, msid 3, pos (0, -0.0, 0), vel (0, -0.0, 0)): start 4954454D; kind 00000000 -> BCA86165; msid 00000003 -> 631195B0; pos.x -> A4AFED4D; pos.y 80000000 -> 0DF28BD2; pos.z -> C6D39632; vel.x -> 3115129C; vel.y 80000000 -> 602C8BCC; vel.z -> **32207664** (oracle `32207664`).

Game word, `docs/rust-port/golden/pad_falcon_fox_fd.csv` frame 0 (seed 9D38619F, s0 992E57A6, s2 818E79DE, no items):

| step | value | h after |
|---|---|---|
| NHV1 tag | 4E485631 | 567DBDE6 |
| seed | 9D38619F | 77FAFD8F |
| s0 | 992E57A6 | 21C79D04 |
| s2 | 818E79DE | 2E531F88 |
| ITCN | 4954434E | F696E69C |
| items | 00000000 | CB8C908E |
| itemsum | 00000000 | **FC467705** = the CSV `hash` |

Frame-0 fighter words are the same for every fighter on a stage (spawn position, entry motion, 0 percent, 3 stocks): the Hero fixture's `s0` at frame 0 equals Falcon's.

## 12. NHV2 (proposed: stage state; versioned separately, nothing here is emitted today)

A new tag (`"NHV2"` = `0x4E485632`) and new trailing CSV columns `st0..stK`; NHV1 columns and words are unchanged. One word per stage
object, mixed with the same `mix`, in this order: stage, then objects in the order below. Candidate decomp fields (names from `src/melee/gr/types.h`; **unverified**: field meanings were read from names and
usage, not run):

| stage | state | candidate fields (GC offset) |
|---|---|---|
| all | stage kind, camera-independent | `stage_info.grkind` (+0x88) |
| Fountain of Dreams | the two side platforms and the water level | `gp->u.izumi3`: `xC4` (s16 phase), `xC6` (s16 timer), `xC8` (s16), `xCC` (JObj: hash its `translate` x/y/z), `xD0` `xD4` `xD8` `xDC` (f32 heights/velocity/target), logic in `grizumi.c` ~lines 460-700; `HSD_Randf` in the logic means the platform path is seed-dependent |
| Yoshi's Story | Randall the cloud | `gp->u.randall.timer` (s16, gp+0xC4), `gp->u.randall.jobj` (+0xC8): its moving collision box position (`translate`), `grstory.c` lines 198-230 |
| Pokemon Stadium | transform phase | `gp->u.stadium.xC4_b0` / `xC4_b1` (transform flags, `grpstadium.c` ~294-362), the transform timer and the active map id `gp->map_id` |
| any stage with moving collision | platform positions | the map-collision joint translations updated by `Ground_UpdateMapColl` / `Ground_UpdateWindAndMapColl` (hash JObj `translate` bits) |
| hazards | timers | stage-specific `s16`/`f32` timers in `gp->u.*` (Randall above, Pokemon Stadium, Brinstar acid, Great Bay) |

Open: which fields are decision-bearing for a later fighter frame (only those should be hashed); a list to be agreed per stage before any
word is emitted.

## 13. Interop proposal

| what | ours (C / Rust) | melee-unlocked (C) |
|---|---|---|
| CSV | section 7 header, `frame` per section 3, plus optional `seed_post` | the same header and word order; `seed_post` if its render draws consume the shared seed |
| log point | after `HSD_GObj_RunProcs()` in `gm_801A4D34` | same |
| scenarios | the four goldens (`pad_*` via pad script, `cpu_*`) and `.slp` playback of `docs/rust-port/golden/slp/*.slp` with `--ignore-seed` | the same replays; for `cpu_*` the same scene + AI seed |
| first run | exchange `pad_falcon_fox_fd.csv` frame 0 (`hash` FC467705): it depends on nothing but stage, fighters, seed 12345 and the pre-draws | |
| diff | `compare_golden.py ours.csv theirs.csv [--common]` | |

**Loader-off vs empty-mod-loader digest check.** Run one scenario twice with the mod loader (and, for us, Geno) disabled and enabled-but-empty;
both NHV1 CSVs must be byte-identical to each other and to the golden. Rationale: a loader that merely being present perturbs a retail
fighter (an extra RNG draw, a changed attribute copy, a Geno hook that is not inert) is the first thing to rule out before comparing mod fighters.
Ours: `MELEE_MODS_DIR` = an empty directory (loader off: no mod mounts) vs a directory with one mod `{mod.json, geno.json = {"geno":5,"fighters":[]}}` (loader on, empty). Result on `pad_falcon_fox_fd`
(`MELEE_TEST_SEED=12345`, `pad_fixed.lua`, exe sha256 `9cca2879...`): both modes reproduce the committed golden exactly (section 14). Each build states the same two modes in its own terms; the digest to compare is the SHA-256
of the CSV body (lines after the header, LF-normalised).

## 14. Verification and OPEN items

| item | status |
|---|---|
| `nhv1_ref.py` reproduces 8 C-oracle vectors and 10,804 golden `hash` cells | done, passing |
| loader-off vs empty-loader on `pad_falcon_fox_fd` | done 2026-11-02: both runs 1801 frames, `compare_golden.py` OK against the committed golden, and byte-identical to each other (our log shows `geno: 1 geno.json file(s), 0 fighter entries` in the empty-loader run). Body SHA-256 of the golden: `d17699ab75fdcb47dfd839a32d6fa1333d501ac2f39b627f5d5f6b1fbf4165ee` |
| `frame` offset H to `unk_0` | melee-unlocked: H = 0 (he will confirm with the probe). Ours: OPEN, not measured |
| `seed_post == seed` in our build | OPEN |
| render-draw semantics of melee-unlocked's build | closed: hardware semantics; ours is the divergence (section 8) |
| same log point in melee-unlocked's loop | closed: he hooks `gmscene` at the same place (row 3) |
| 16: optional hardware-faithful seed mode in our builds (render draws on the shared stream, behind a switch) | OPEN: do we add it? It would need its own golden set (ours are recorded with isolation); benefit: `seed`/`hash` comparable with melee-unlocked and Dolphin without `--ignore-seed` |
| Geno digest `geno0..3` (8.1) | implemented in our C build; the Rust engine has no Geno runtime; melee-unlocked has not implemented it |
| NHV2 field list | proposed |
| Dolphin memory oracle | built 2026-11-05 (docs/rust-port/notes/dolphin-oracle.md, tools/rust-port/dolphin/oracle.sh): patched Slippi Dolphin, hook after `HSD_GObj_RunProcs` at 0x801A4FA4, words by `nhv1_ref.py`. No tournament replay matches all the way: the first NHV1 difference in each of the 6 slpcheck-exact files tried is `x8c_kb_vel.y` -0.0 (console) vs +0.0 (ours) |
| UCF / non-vanilla replays | not comparable to the vanilla sim |
