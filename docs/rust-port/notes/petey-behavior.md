# Petey: Brawl behaviour, measured (clean room)

Branch port/petey. Clean-room rules: the Brawl side is read as data (file formats only) and, where possible, by
watching the game. No Brawl code was decompiled, disassembled or read by this side; constants that come from a
code reading arrive only as prose in the reader agent's spec (git-ignored, `_build/petey-spec/spec.md`) and are
marked [spec] below. Nothing from the disc is committed: the numbers here are derived (lengths, counts, sizes).

## Method

1. Data (a). `petey_model.rs` reads the owner's local extracted files at runtime. Formats: ARC (nested, LZ77
   0x10/0x11), BRRES groups, MDL0 v9 bones, CHR0 v4/v5 (BrawlBox-documented layouts), and the boss's enemy param
   file: a relocatable section (header: size, data size, relocation count, root counts; data at +0x20; root table
   of (data offset, name offset) pairs). Its named roots include `HitDataSet`, `wpnHitDataSet_ArmLeft/Right`,
   `cameraRangeSet`, `CameraClipSphereData`, `s_wpnAnmCmdDataSet` and AI script names. Hurtbox records use the
   BrawlBox HurtBox layout: offset xyz, stretch xyz, radius, flag word with the bone index in bits 23-31.
   A local-only probe (`petey_model::probe::petey_probe`, `#[ignore]`) prints the derived numbers.
2. Observation (b). Not done: see "Dolphin" below.

## Data findings

Animations (body, both arms share every clip and length; no loop flag set, looping is the game's choice):

| clip | frames | read as |
|---|---|---|
| Wait1 | 271 | idle |
| Wait2 | 160 | idle (second) |
| BasketAttackL | 160 | cage swing with the left arm |
| BasketAttackR | 160 | cage swing with the right arm |
| JumpStart | 20 | jump take-off |
| JumpLoop | 31 | airborne loop |
| JumpAttack | 90 | the falling body slam |
| JumpAttackLanding | 71 | landing / recovery |
| DamageLeft / DamageRight | 30 | flinch |
| Angry | 70 | the rage (phase change) |
| Down | 721 | defeated / long down |

No spin or bite clip exists: the stand-in's spin and bite were invented and are removed.

AI script names in the param file: `AIExecAttackSelect1`, `AIExecAttackSelect2` (two attack-selection tables:
before and after the rage), `AIExecAttackSwing_NearArm` (the basket swing uses the arm nearer the target),
`AIExecAngryToNext` (rage, then the next table), `JumpAttack` (with an `Init`).

Bones: body 31 (NeckAN, HeadN, Attach01N/02N for the arms, Collision01-03N, mouth / petal / tongue);
left arm carries `PeachBodyM` (Peach's cage) and `LWeakN`; right arm carries `ZeldaBodyM` and `RWeakN`.
The cages move only by the CHR0 (bones LIronNa/Nb, IronchainNa/Nb): canned animation, no physics.

Hurtboxes (param file):

| set | bone (index) | offset | stretch | radius |
|---|---|---|---|---|
| body | UppermouthN (28) | (0, 0, 10) | (0, 0, 10) | 39 |
| cage L | LIronNb (9) | (0, 0, 0) | (0, -20, 0) | 11 |
| cage R | RIronNb (9) | (0, 0, 0) | (0, 20, 0) | 11 |

So the body's one hurtbox is a big sphere-capsule on the head / mouth; each cage is a 40-long capsule on its iron.

Camera: `cameraRangeSet` (40, -40, 80, -10 after two words) and `CameraClipSphereData` centre (0, 25, 0)
radius 30 (read as the boss's camera framing; units Brawl).

Size: the body is 102 units tall, 77 wide in its idle pose (Brawl units; ours are the same order).

## Dolphin (observation)

Not reached. There is no Brawl save, save state or input movie on this machine, and Petey is fought at the end of
an Subspace Emissary stage that is only reachable by playing the earlier stages from a new file. Doing that without
OS input means authoring a frame-exact input movie blind through the menus and two platforming stages; by the rules
this is reported, not worked around. Every "observed" column below is therefore empty; what the user can supply to
unlock it: a Dolphin save state at the start of the fight (or a Brawl save with the stage cleared), after which a
.dtm from that state plus frame dumps measure everything listed.

## Derived from the animated bones (local probe, `petey::probe::petey_tracks_probe`)

- Body hurtbox centre y: 60-65 in the waits, 43-70 in the baskets, up to 159 in JumpAttack, 28-64 landing.
- The jump height is in **JumpAttack's** clip, not JumpLoop's: the root carries the body from y 52 up to about 162
  between frames 5 and 85, then down. JumpStart / JumpLoop stay on the ground (a crouch held for the timeout T).
  (Settles spec §11 "how jump height comes from the root".)
- BasketAttackL swings the **left** cage (model +x, x 50 at frame 69) across to x -58 by frame 92-114, at y 8-13;
  BasketAttackR is the mirror (-55 to +62). So the swing goes toward the -x side for L and the +x side for R. With
  rule 24 "target on side 1 -> right basket" this means side 1 = target at +x of the boss (the swing travels toward
  the player). Used as the sign for side 1 (spec §11 item 2): from data, not observation.
- Down: the body settles by frame ~400 (y 30) and holds to 721; the motion holds its last frame (spec §11 item 6).
- Landing boxes on TopN: centre (0, 31), sides (+-29, 0): TopN does not move in the landing clip.

## Our implementation (petey.rs) vs the spec

Statuses 1-15, rules 0-30 (as a direct state machine with the same priorities), the L tables, wait / jump timeouts,
weighted picks, cage HP 999, body damage forwarded to both cages, death on a hit with the lower cage <= 10%,
cage break -> Down + partner freed, phase-up at 50% (Angry) and 25%, flinch from the waits, kinetic kinds 14/15/16
(x0.75), dive aim snapped to +-x, every hitbox of §8 with its frames, bone, offset and level scaling, hurtboxes on
the bones (UppermouthN r39; LIronNb / RIronNb capsules r11). Shapes follow the bones when the local model is present
(tracks built at load, in memory only); otherwise a stand-in of the same sizes and frames. Approach chosen: generate
locally at load, fall back to the stand-in (no disc-derived tables committed). The two can differ, so a replay must
be played with the same data presence it was recorded with.

Camera: the carrier's camera subject is moved onto Petey every frame with the cameraRangeSet box (+-80 across,
+40 / -40), so the stage camera frames P1 and Petey instead of the invisible carrier.

## Feel comparison (Brawl vs ours)

Brawl column = spec / data (no observation possible). "ours" = this build.

| aspect | Brawl (spec / data) | ours | gap |
|---|---|---|---|
| movement speed / distance | still except the dive: 0.2 start, +(0.015+0.003L)/f, max 1.4+0.03L, x0.75 [spec M/L] | same; clamped to x +-55 (FD) | arena width: Brawl's arena is wider, the dive stops at our edge |
| attack startup / active / recovery | basket: cage boxes f70-114 of 160; landing centre f2-4, sides f3-4 of 71; jump: 20 + T(60-5.4L) + 90 + 71 | same frames | none known |
| turning / facing | never turns; cages fixed (L = Peach, model +x) | same | the model's on-screen orientation is as authored, unverified |
| hitstop / knockback when he hits | per §8 (dmg 10+2.7L, 145 deg, KBG 44, BKB 105-2L; hitlag x1 / x0.5) | dmg / angle / KBG / BKB as §8; hitlag multipliers not applied (Melee default); angle relative to the swing direction [Q2] | hitlag multiplier, angle reference |
| hitstop when a cage / body is hit | no knockback; a scripted flinch (30 f) from the waits only | same; the attacker gets Melee hitlag | Brawl's own hitlag formula not reproduced |
| cage swing | baked animation, no physics | same (bones) | none |
| camera | stage camera with the boss's range box and clip sphere | range box fed into Melee's camera | the clip sphere and the stage's scripted shots (events 5-9) not used |
| idle between attacks | Wait A 100-150 (-9L/-12L), Wait B 2-20 | same | none |
| difficulty | L from strength and the game's difficulty index | index 5 (fresh Normal) | Envoy has no SSE tier / continues |
| size | model 102 tall, no placement scale | scale 1.0 | none |
| end | stage cutscene (events 9 / 1000 / 1001) frees a princess | Down / dying, then the carrier's KO ends the fight; the broken cage's model hides | cutscene not reproduced |

Measured in our build (MELEE_PETEY_LOG, idle P1, phase 0, L 1.7): Wait A 95-120 f, basket 159 f, Wait B 1-9 f,
jump chain 19 + 51 + 89 + 70 f (T = 50.8 at L 1.7), the dive reaching the arena edge, matching the spec's numbers.

## Reader answers applied (2026-11-09, second pass)

- Scale 1.0 confirmed (0.75 is movement only). Difficulty index 5 (a fresh Normal game; Brawl: 3 tier + 2 - min(retries,
  5)), so L = 2.5 / 3.3 / 4.1 by phase.
- Side 1 = target x < boss x (screen-left) -> right basket (Zelda's cage, model -x). This replaces the data-based guess
  above (the swing direction is not what decides it).
- Basket 145 degrees in the cage's own facing: left cage +1, right cage -1 (up and backwards). Landing centre -90 =
  straight down (our 270).
- The +-68 is an absolute world goal x set by rule 13 (target left -> -68, else +68); each frame after movement, within
  0.1 + |vx| of it, x snaps to it and holds. Our clamp is now FD's edges (+-85) so the goals are reachable.

## Questions for the reader (answered; kept for the record)

Q1 model / hitbox scale and the difficulty index per SSE setting; Q2 the reference for the basket's 145 degrees and
whether -90 is a straight-down launch; Q3 whether the +-68 lead offset affects the dive at all.
