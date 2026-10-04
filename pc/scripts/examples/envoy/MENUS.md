# Envoy retail 1P menu contract - 2026-10-04 fix1

This retail contract supersedes the default maze/campaign flow below. The old
mission flow remains parked for diagnostics; it is not extended by this packet.

- Asset-free menu and setup default to Classic; Adventure is the second run choice.
  Fighter, difficulty and stocks carry into each NG+ playthrough.
- A selects a drive reward after each cleared retail stage. Three coloured/size
  choices appear in an existing kit panel; the final clear has a larger reward.
  Each choice states its concrete percentage effect. The selected reward plays
  existing drive pickup sounds/FX and animates before/after growth across level-ups.
  It applies after the atomic save succeeds. Coloured
  drive glyphs use existing kit text; retained 3D models have no kit preview API. B cannot
  bypass the reward. A bounded engine hold keeps a broken panel from stranding
  retail progression.
- Opponent leaning-stat tags appear at stage start. Retail teams, giant/metal
  handicaps and AI levels retain their normal behavior; drive multipliers stack
  with those handicaps. Physical in-stage drive drops default off.
- The HUD retains Power, Speed, Guard and Jump bars. Jump changes ground/air jump
  height and air speed; Guard reduces damage/knockback and briefly flashes on a hit.
- Results return to a working menu without assets. The walkable garden appears
  only after its model resolves; failed spawning returns to the menu.
- Master Hand completion (Adventure: its own final boss) evolves a young
  companion, then NG+ continues with the same stats and age. Continue retries use
  the same seeded opponent spread. Game over settles growth and returns to results.
- Older engines show an unavailable-mode message; they must never silently start
  a maze in response to Classic or Adventure.

See the Classic packet report for exact engine API, ending/record policy and
source-only validation. Controller feel and retail presentation need native play.

## Parked mission contract (historical)

# Envoy slices 2?4 menu and garden contract

Supersedes the slice1 menu contract. All panels, text and lists use existing
`gd.kit` components. No new art. Safe-area layout covers 640, 853 and 1140px
canvases. A selects, B returns, START pauses; C-stick retains attacks.

| Surface | Behavior |
| --- | --- |
| Title/Profile | Explicitly open Envoy; Continue enters the physical garden. Corrupt profiles refuse without replacing the file. |
| Garden | Walk freely on the authored mission level. Move near a station and tap A. START opens the garden menu. |
| Run exit, x=-180 | Opens setup; selected fighter launches an offline LAB scene, then the seeded campaign starts after readiness. |
| Fighter, x=-90 | Existing fighter selection list; selection is used at the next run launch. |
| Companion, x=0 | Power, Speed, Guard and Jump grades, nonlinear bars, age/remaining runs, named passive, colours and DNA. The tinted kit placeholder has an Envoy id/type label. |
| Records, x=90 | Runs started, settled runs, wins, best winning time, companions raised and highest observed grade. Unknown legacy history is labelled. |
| Nest, x=180 | Present with a slice5 closed label; no breeding. |
| Playing | Power, Speed, Guard and Jump bars fill with collected points. Receiving bar flashes; LEVEL UP lasts 48 logic frames. START pauses. |
| Interlude | Current drive and level gains, next theme. A explicitly continues; B cannot skip or advance. |
| Pause | Resume, Companion, Abandon Run with safe-default confirmation. |
| Results | Before/after grades, levels and points; boss outcome; evolution/type/passive moment. A/B returns to the garden after cleanup/settlement. |
| Garden return | Sixth settlement shows the reincarnated egg with a notice; the next started run hatches it. |

Menus own only their own pause and D-pad mask. Garden walking and staged cleanup
remain unpaused. A station opens once per press within 22 units horizontally;
release A before activating another. Screens opened by a station return to the
physical garden. Closing Envoy disposes its garden and restores host ownership.
Normal matches remain untouched until the player explicitly enters Envoy.

`envoy hub`, `menu`, `menudump`, `dump`, `status`, `start`, `retry`, `give`, `stop`
and `reset-profile confirm` provide diagnostics. `retry` reuses the last saved
campaign seed; a saved interrupted start resumes its pending seed without aging
again. It restarts traversal, rather than restoring unsaved room/item state.
