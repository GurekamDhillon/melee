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

## Dolphin (observation, 2026-11-10, port/petey5)

The user supplied a Dolphin save state taken at the start of the fight (git-ignored copy under `_build/petey-spec/states/`).
Method: our own Dolphin instance (separate user dir, audio 0, emulation 0.5x, frame dump on), loaded with `-e <disc> -s
<state>`, P1 Kirby idle (no device bound: no OS input). A read-only RAM logger (ReadProcessMemory on that Dolphin's MEM1;
values only, no code) recorded every frame (7200 frames, none dropped, keyed on a 60 Hz counter): Petey's root x / y,
his motion frame counter (frozen frames = hitlag), Kirby's position. Petey reads Normal: the dive's acceleration is
exactly 0.015 + 0.003L at L = 2.5. The idle Kirby lost his last stock about 2740 frames in, ending the fight. Our build
ran the same scenario: Kirby P1 idle, Envoy fight 5, `MELEE_PETEY_LOG=1`. Screenshots:
`scratch/envoy-demo/screenshots/petey5/side_by_side.png` (top row Brawl, bottom row ours).

Not observable with an idle player: cage HP per hit, phase changes, Angry, flinch (Damage), Down, death. Those need
player attacks, i.e. an authored .dtm from the state. Not done this round.

## Feel comparison (Brawl vs ours)

| aspect | observed in Brawl | ours before (petey4) | ours now (petey5) |
|---|---|---|---|
| move order | Wait A -> basket -> Wait B -> menu (basket or jump); jump chain -> Wait A. Sequence seen: B, WB 5, B, WB 4, J, WA 112, B, WB 14, J, WA 89, B, WB 15, B, WB 12, J, WA 111, B, WB 2, B, WB 4, J, WA 119, B, WB 15, J | same rules | same (WA 88, B, WB 5, J, WA 85, B, WB 4, J ...) |
| Wait A / Wait B | 89-119 f / 2-15 f | in range | in range |
| basket | 160 f (+ hitlag), boxes hit at motion f 79-86 | 160 f | 160 f |
| jump chain | JumpStart 19-20, hover 47 (= T 46.5 at L 2.5), JumpAttack 89-90, landing 70-71 | same | same |
| jump height | root y peaks at +94.3 over the ground at JumpAttack frame ~54 | from the bones | from the bones (unchanged) |
| dive speed | first step 0.2225, +0.0225 / f, max 1.475: the spec's kind 16 **without** the x0.75 | x0.75 (max 1.11) | no scale: identical |
| dive distance | -68 -> +26.3 in the dive, landing slide (decay 0.0725 / f = 0.06 + 0.005L) to +42.06 | 72 + 11 | -68 -> +27.8, slide to +42.1 |
| dive goal | stops exactly on +-68 when reached (0 -> -68, snaps, holds); unreachable goal (+68 from -68) has no effect | same | same |
| turn / facing | never turns | same | same |
| hitlag (Petey frozen) | basket 9 frozen frames (15 dmg), landing side 4 (13 dmg, x0.5) | none: Petey kept moving | carrier hitlag freezes Petey: basket 8-9, landing 3-4 (x0.5 applied to attacker and victim) |
| hits per attack | one: a victim takes one basket or one landing hit | two (box A then B; landing centre then side) | one (an attack's boxes share victims) |
| basket knockback | L cage: 146.8 deg (-x), R cage: 33.3 deg (+x), first-frame speed 3.41 on Kirby 0 -> 15% | wrong side for the L cage (Melee launches from the carrier's x) | L 147 deg (-x), R 33 deg (+x), speed 3.36 |
| landing side knockback | 151.8 deg outward (left side), speed 3.64 (Kirby 15 -> 28%) / 4.14 (30 -> 43%) | 152 deg, 3.86 | same rule (angle mirrored per side) |
| basket damage | **15** (Kirby 0 -> 15) | 16 (10 + 2.7 x 2.5) | 15 since port/petey6: the cage uses trunc(L) (Q4) |
| landing side damage | 13 (= 8 + 2L) | 13 | 13 |
| camera | close stage camera: Petey fills about half the screen height and leaves the top of the frame in the jump | wide FD camera framing Petey with his range box | port/petey6: player-only follow camera (Q6) |
| cage HP / phase / Down / stun | not observed (idle player) | - | - |
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
(raw values; 0.75 is a bounce factor, Q5), dive aim snapped to +-x, every hitbox of §8 with its frames, bone, offset and level scaling, hurtboxes on
the bones (UppermouthN r39; LIronNb / RIronNb capsules r11). Shapes follow the bones when the local model is present
(tracks built at load, in memory only); otherwise a stand-in of the same sizes and frames. Approach chosen: generate
locally at load, fall back to the stand-in (no disc-derived tables committed). The two can differ, so a replay must
be played with the same data presence it was recorded with.

Camera (port/petey6, replaces the cameraRangeSet subject box): see "Reader answers Q4-Q6" below.

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

## Reader answers Q4-Q6 applied (2026-11-10, port/petey6)

- Q4, cage damage: the cages take the boss's L through link 5112 as an integer, so cage L = trunc(L): 2 / 3 / 4 on
  Normal by phase, basket damage 15 / 18 / 20 and basket knockback growth 105 - 2 trunc(L). Body contact and landing
  keep the float L (landing side 13 at L 2.5). Checked in our build: idle Kirby 0 -> 15 from the first basket (f183).
- Q5, param 3056 (0.75): a fallback bounce factor on wall/floor contact, never a speed scale. Petey never bounces and
  is never launched, so it is unused; the old KIN_SCALE constant is gone and the kinds run at their raw values.
- Q6, camera: Brawl's adventure follow camera, player only (Petey's range box no longer feeds the camera). Fixed eye
  distance 260 approached at 5% of the gap per frame (the centre eases at the same 5%), tilt -1 degree (eye above),
  visible rectangle plus 10 units kept inside x -180..180, y -70..120. FOV 30 degrees, Melee's own default camera FOV:
  2 x 260 x tan 15 = 139.3 visible units of height. The width for the clamp uses the camera's authored aspect
  (1.2173), so the logic does not depend on the window; a 16:9 window shows a little more at the sides than Brawl's
  clamp assumed. Driven through Melee's scripted (boss intro, mode 6) camera with fixed interest and eye; the state is
  in `petey::B` and `game_camera` (statics, in the snapshot): synctest depth 4 over 3000 frames, 0 mismatches.
  Comparison: `scratch/envoy-demo/screenshots/petey6/camera_compare.png` (top Brawl, bottom ours).

## Knockback conversion for Melee fighters (Nov 13, owner: "a large tendency to get one shot")
Petey's boxes keep Brawl's damage, angles and knockback formula inputs, and the launch speeds matched Brawl
(basket 3.41 measured in Brawl, 3.36 here). The one-shots come from the arena: a Melee stage's blast zones are
much closer to his +-68 landing spots than his Brawl arena's. Brawl and Melee use the same knockback formula
and the same 0.03 / 0.051 speed constants, so nothing was double-scaled. The fresh run's fight 5 is difficulty 5
(Normal, L 2.5), and Envoy adds no damage or knockback multiplier to enemy hits.
**Rule:** growth and base knockback of every Petey box x0.8 (`KB_SCALE_PM`), so knockback x0.8 and travel
x0.64; set (weight-based) knockback unchanged. It is the largest 0.05 step at which, in a launch model on FD
(Melee gravity and fall speed, grounded spikes bounce at 0.8, launched from x 68, no DI), no Petey attack KOs
Mario, Kirby or Fox from 0, 50 or 100%. Unscaled, the landing side box KOed Kirby and Fox at 50%.
