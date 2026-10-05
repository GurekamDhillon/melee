# Crit impact frames on Fox's drill

An experiment: every hit of Fox's down air (the drill) plays an anime-style impact sequence, as if that hit were a
crit. It is the toned-down, strength-scaled cousin of the clank impact frames (`experiment/clank_impact`); the
original look is kept here as the `clank` preset for comparison. Visual only. It changes no gameplay unless
`impact freeze` is set above 0 (default 0).

## Run it

Offline vanilla LAB, Final Destination, P1 Fox, P2 anything. Mount or load this folder
(`load <absolute-path-to-this-folder>`), or use the workspace's `_build/audit-20261003/impact-drill/play.sh`.
Short hop over P2 and down-air (C-stick down). Each hit of the drill fires.

## How the hit is detected

`on_hit(attacker, victim, info)` is the engine's collision-origin hit event. Its `info.attacker_action` is the
attacker's action state at the collision, and Fox's down air is `AttackAirLw`, action **69** (`ftCo_MS_AttackAirLw`;
confirmed in game: `info.attacker_action=69`, `move_tag="aerial"`, while P1's nair reads 65). The mod fires when
`attacker == 1` and `attacker_action == 69`. The drill's hits are separate hit events (3 per drill against a grounded
Falco, 5 logic frames apart), and each one fires the sequence separately. The contact point is `info.x/y/z`.
`impact who p1all` widens it to every hit P1 lands; `alldrill` fires for any fighter's down air (any port).

## What the sequence is

One full-screen pass (`shaders/crit.wgsl`), centred on the contact point projected to screen each frame, never
leaving a pass behind. Layers: a one-to-two frame hard two-tone "impact frame" (inverted from medium up), then
speed lines, a radial blur, a decaying chromatic aberration, a shockwave ring and a contrast push, all with an
ease-out. One `strength` (0..1) scales everything:

| layer | at strength s |
|---|---|
| total length | 0.16 + 0.34 s seconds (light 0.25 s, heavy 0.45 s) |
| hard frame | 1 logic frame below s 0.45, 2 above; cut strength 0.30 + 0.70 s; radius 0.30 + 1.60 s (whole screen from about 0.9) |
| inversion | only above s 0.55, up to 90% |
| speed lines | none below s 0.15, up to 0.9 |
| blur / aberration / ring / contrast | each rises with s (see `look()` in `scripts/main.lua`) |

Presets: `light` s=0.25 (default; a small two-tone bubble, faint lines, a little aberration), `medium` 0.55,
`heavy` 0.85, `clank` (the original over-the-top v3 look, 1.18 s, `shaders/clank.wgsl`).

## Repeated hits

The drill hits again every ~83 ms, so the sequence **restarts** on each hit: the live pass is removed and a new one
added (from the already-compiled shader), so at most one pass of this mod exists and nothing stacks or grows. To
avoid a flash train, a hit that starts within 0.30 s of the previous sequence keeps everything except the hard
frame, which becomes a small bubble with no inversion. The first hit of a burst gets the full hard frame.
Shaders are drawn once, invisibly, at match start (and when `clank` is selected) so the first hit has no compile hitch.

## Modes and commands

Console: `impact preset light|medium|heavy|clank`, `impact strength 0..1`, `impact ramp on|off`,
`impact chance 0..100`, `impact who p1drill|p1all|alldrill`, `impact test` (fires at screen centre),
`impact off|on`, `impact hud on|off`, `impact freeze 0..12`, `impact seed N`, `impact log on|off`, `impact state`.

- **ramp**: hit n of a drill plays at 30% of the strength rising to 100% by hit 3 (`RAMP_HITS`, the drill length
  measured here); a drill ends after 24 logic frames without a hit.
- **chance N**: only N% of hits are crits; the draw is a fixed hash of (seed, draw number), so the same seed repeats
  the same pattern (`impact seed N` resets it). The others play nothing.
- **freeze**: extra freeze frames at strength 1 (scaled by the hit's strength) through `gd.hitstop`: offline only,
  default 0. This is the only setting that changes gameplay timing.
- **HUD**: a one-line status at the top-left, hide with `impact hud off`.
- **Shortcuts**: hold L+R (shield, which cannot taunt) and tap the D-pad: left/right cycles presets, up/down changes
  strength by 0.1. Keyboard: F6 preset, F7 on/off, F8 HUD. The C-stick is never read.

## Limits of the existing API

There is no true per-hit "crit" event; this is an `on_hit` plus action-state test. Hit location is read from
`info.x/y/z`. A freeze per hit uses `gd.hitstop` (a host gate, not the fighters' own hitlag), is refused online,
and each hit's request replaces the last.

## Credit

The clank look in `shaders/clank.wgsl` is GD's own "Haki Impact V3" reference, ported in
`experiment/clank_impact`. `crit.wgsl` is original, written against the same pass API. The visual language
(impact frames, speed lines, radial blur, aberration, shockwave rings) is the common anime/manga hit-stop
vocabulary; no third-party source or assets were consulted or copied.

Verification: ran in the frozen vanilla build (2026-10-05): detection, restart count, ramp, chance determinism,
who modes, cost and screenshots. The L+R D-pad chord was exercised by injected input. The HUD line drew without errors but
does not appear in screenshots, so its look is unchecked.
