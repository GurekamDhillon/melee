# Per-move script overrides (`script_overrides`), Rust only

Companion to `anim-overrides.md` (per-clip animation overrides; that note lives on `port/animloop`. Fold this one into
it as a "Script overrides" section when the branches meet). Code: `rust/melee_fighter/src/script_override.rs`, the
hook `ftdata::row_script`, manifest parsing in `melee_data::modreg`, loading in `melee::mods::Mods::script_overrides`.

## Manifest

Same shape as `anim_overrides`; `file` holds the replacement action (subaction) script of the move:

```json
"anim_overrides":   [{"fighter": "marth", "clip": {"row": 55}, "file": "anim/ftilt.dat"}],
"script_overrides": [{"fighter": "marth", "clip": {"row": 55}, "file": "scripts/ftilt.words"}]
```

- `clip`: `{"row": N}` = one row of the fighter's motion (subaction) table; a string = every row whose FigaTree symbol
  is that name (as for clips). Matching, mount order (later mod wins) and the Nana/Popo rule are the clip overrides'
  (shared code: `ftdata::match_row`).
- `file`: `*.words` = text, one 32-bit word per line (`0x..` or decimal, `#` comments), what
  `python -m tools.geno.script asm X.genoasm --words` writes; any other extension = raw big-endian words.
- Each entry enters the mod's netplay digest (pseudo path `script_override:<fighter>/<clip>=<file>` + file size).

## Mechanism and why

The engine binds a row's script in two places: `Fighter_ChangeMotionState` (`fp->x3E4.u = row->xC`) and the wait-anim
switch (`ftwaitanim`). Both now ask `row_script`, which returns the override's buffer for a matched row and `row->xC`
otherwise. With no override installed it is one test of a static: unmodded play is unchanged.

The accepted design said "opcode-59 overlay": a Geno `subactions` overlay (geno-v5 4.3) that writes a pool pointer into
the loaded Pl file's subaction table at spawn. Not used here because it needs a `geno.json` define for the fighter (a
Geno profile: profile index, content id, the snapshotted `Geno_ScriptPool`), which changes netplay identity and layout,
and vanilla fighters such as Marth have no ported Geno template on this build (marth-test README: a Marth define panics
in `GenoDefine_ExtAttrBytes`). The direct substitution touches no match state beyond the command pointer the fighter
already holds, which is not in the rollback hash; the buffers are heap boxes installed before boot and never moved.

## Validation (boot stops with the mod and file named; never a panic)

Unknown fighter, negative row, path outside the mod, unreadable file, non-word text, empty, over 4096 words, odd byte
length; then a command walk with the engine's own word-count table: a command overrunning the file, no End (opcode 0)
before the end, opcode > 59 (no handler: the C would index past its table), opcode 59 (Geno escape, needs a profile),
Subroutine / Return / Goto (5/6/7: 32-bit addresses into the Pl file), Execute Loop without Set Loop, loops nested over 4.
One End word is appended after the file's words.

Not checked: whether command arguments make sense (hitbox bone ids past the fighter's bones, GFX/SFX ids). Those fail
as they would in a retail script.

## What depends on a clip's LENGTH

| depends on the clip length / frame numbers | where | covered by a script override? |
|---|---|---|
| hitbox on/off frames, damage/angle per window | script (hitbox, hitboxes_clear, wait/wait_until) | yes |
| IASA (can act) frame | script `iasa` (allow_interrupt) | yes |
| sword trail, GFX, SFX, rumble, voice | script | yes |
| body state (intangible/invincible frames), hurtbox states | script | yes |
| smash charge frame, jab follow-up windows, auto-cancel window, aerial "landing lag applies" flag | script flags (charge command, cmd vars / throw_flags) | yes |
| throw release frame | script (throw command) | yes |
| state end (most moves, rolls, getups, dashes, landings) | `!ftAnim_IsFramesRemaining` at clip end (162 call sites) | follows the new clip automatically; not a script matter |
| translation (TransN root motion) | the clip | follows the clip |
| aerial landing lag (LandingAirN/F/B/Hi/Lw) | attributes `landingair*_lag`; the landing clip is rate-scaled to last `lag` frames (`(clip len + 0.1) / lag`) | no: lag is a fighter attribute (Pl file; a Geno define can set it). A new landing clip of any length still lasts `lag` frames |
| normal landing lag, squat/jumpsquat | attributes `normal_landing_lag`, `jump_startup_time` (KneeBend ends at that frame or clip end, whichever first) | no (attributes) |
| standing turn | attribute `standing_turn_frames` | no |
| walk / run cycle speed | `run_animation_scaling`, walk speed attributes | no |
| move-specific frame numbers in C state code (specials compare `cur_anim_frame` to constants, e.g. charge loops, multi-hit loops, special fall timers) | fighter TUs | no: code; a clip with different timing desyncs these |
| thrown victim's animation | the thrower's table (`Thrown*` rows) | an override on the thrower's row applies (match by table owner) |
| CPU behaviour (move choice windows) | AI code | no |
| LAB `gd.timeline` and other static readers of `row->xC` | read the table, not the bound pointer | no: they show the retail script for an overridden row |

## Not done / limits

- No hot reload of scripts (the clip overrides have one on port/animloop): fighters hold pointers into the buffers,
  so a reload would have to keep the old buffer alive until no fighter points into it.
- Static readers of the table (LAB timeline, Geno's ORIG lookup) do not see the override.
