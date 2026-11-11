# Envoy V3 UI: the Party pack look and the way in (port/v3ui, 2026-11-11)

Built overnight from `menu/out_envoy/UI-SPEC.md` (the authority) and its reference screens, on the V3 rules of
`port/v3sim` (merged at e77bbca20; `envoy-v3-view.md` is the data contract, `envoy-v3-rules.md` the rules).

## What is in

| piece | where | notes |
|---|---|---|
| **Envoy on the main menu** | `melee_modes::tu::mnmain` (`envoy_main6`, `ENVOY_SLOT`, `ENVOY_ORDER`, `ENVOY_LIGHT`) | a real sixth row between Vs. Mode and Trophies. Selection 5 is virtual and shown in display slot 2 (retail rows 2-4 move down one); the disc's own 6-row layout frame (`CountUnlockedSelections` = 6) and a sixth `MenMainCursor` instance; up / down walk 1-P, Vs., Envoy, Trophies, Options, Data; A opens the camp (`envoy::door_main_open`), or with the Brawl files missing shows the "needs your Brawl disc" message and never loads the camp. While it is hovered the retail right-hand panel (tree joint 14) is hidden and the retail description text too; the mod draws the stickers and its own description. The light is sunny. Everything is behind `envoy::DOOR_ON`: with no Envoy mod the menu is the retail five rows, byte for byte (every helper returns the retail value) |
| removed | `mnmain` 1-P think, `melee/src/browser.rs` | the phantom row under the 1-P menu and the browser's Envoy row |
| host: rotation | `melee_gx::ui` (`UiQuad.rot/piv`, WGSL), `melee_modhost::ui::cmd` (`OP_ROT` 11), `gd_ui_enc::Enc::rot` | quads rotate about a pivot; everything after `rot(a, x, y)` until `rot(0, ..)`. `UiQuad` is 80 bytes |
| host: fonts | `melee_gx::ui_font` (`FACE_DISPLAY`, `FACE_BODY`), style bits `TEXT_DISPLAY` (1 << 11) and `TEXT_BODY` (1 << 12) | Bricolage Grotesque ExtraBold (a static instance of the variable font, wght 800 / opsz 24) and Source Sans 3 Bold, both SIL OFL, licences beside them in `rust/melee_gx/assets/fonts` |
| host: atlas | `melee_gx::ui::ATLAS` 2048 | the stickers need room; `gd_ui_tex_load` takes PNGs up to 1024 and logs when the atlas is full |
| host: co-op join | `gd_ui_envoy_coop(on)` (UI), `envoy::COOP_JOIN`, `gd_envoy_coop` | player 2 presses Start at the camp to join (hold B to leave); a run started after is co-op. `MELEE_ENVOY_COOP=1` still forces it |
| host: state | `gd_ui_envoy_state` bits | 2 = the main menu is up (was: the 1-P menu), 4 = its cursor is on Envoy, 64 = A was pressed there without the Brawl files |
| sticker kit | `tools/rust-port/envoy/bake_stickers.py` -> `rust/mods/envoy_ui/art/*.png` (77 sprites); `envoy_ui/src/kit.rs` | baked with headless Chromium from the design generators' own SVG (original art): sticker faces as 48 px 9-slices (white, sunny, coral, lagoon, faded, lilac, ink, cream-dashed, hovered), discs, stocks, coin, stars, spark, lock, bursts, rays, four flames, pointers, D-pad, emblem, no-disc mark, swap, head, the menu label in three states, 8 families x 3 tiers of badges, 6 keystone emblems, the wedges, two backdrops (half size). The kit draws faces, rotated stickers, outlined text (eight ink copies), charge rings (dots), the strip and the hint line, the "enter" and burst motions |
| main menu stickers | `envoy_ui/src/party.rs` `menu` | saved run (run card with each fighter's chips, punch card, stocks, keystone pills), no run yet (with a "New!" burst), Brawl disc missing (dashed card with the three steps; A shows the message) |
| camp C1 | `envoy_ui/src/camp.rs` | one sticker list, one cursor (P1 reads its own pad: `gd_ui_pad`): Continue / Start run, New run (asks to abandon a saved run), Fighters, Run length, Team attack, NG+ level, Collection, Back; the status board (fighters, P2 seat, run strip, stocks, keystones, "set for this run"); Start starts or continues from any row; P2 Start joins with a "P2 joined" burst |
| the road (map N2) | `camp.rs` `road` | after a fight the camp scene (which replaces the VS character select) is the road: the run strip with the pointer on the next stop, the next stop's poster (kind, fight n of 5, what a win gives), the party, stocks, keystones; A "Start the fight" goes to the character select and the stage select as before. A run that ended shows "Run cleared!" (rays, a Cleared burst) or "The run ended" |
| fight HUD (H2 + M1) | `envoy_ui/src/hud.rs` | per player above the retail percent: tag, fighter, the bench sticker (fills while the tag cools, D-pad down), the drive row (badge, charge ring, level pips, a burst when it fires), the active drive (Volley: square badge, pips, D-pad right, a burst when ready), the Heat bar (from `P_HEAT` / `P_HEAT_CAP` / `P_HEAT_MARKS`: fill in the state colour, the three marks, a burst on a mark crossed, the flame growing with the state, a sunny ring at the cap, no digits). Top left: keystones as emblems, the purse, the stock pool (the last stock coral on a pulsing burst). Top right: foe plates (elite / boss with their drives and charge, Burning / Chilled / Marked). Petey: the boss bar cut in quarters with the cages. Moments from the event ring: "Cashed out!" with coins flying to the purse, "+n" coins, technique sparks, "Blazing!", "Break!" |
| pick of three (R2) | `envoy_ui/src/pick.rs` | three cards (badge with its tier rim, name, family, the rule from the rules data, what it does for each fighter: new / level n / row full), then the two fighter stickers with the pointer; the full-row step (`P_STEP` 2) rings the slot that will be replaced and asks "Replace X with Y for good?"; "Level up!" burst on a duplicate; "Set!" when done. Co-op: two halves, each with its own cursor |
| keystone pick (K1) | `pick.rs` | solo: three wedges (ink, hovered sunny and level), emblem on the top edge, name, rule, price; the others swing out and fade when taken, "Set!". Co-op uses the card halves |
| shop (S1) | `envoy_ui/src/shop.rs` | "Rest stop" with the purse; the list (three drives with badge and price, level up, restore a stock, swap the drives on sale, reorder, back on the road), prices you cannot pay in deep coral; the hovered drive's tag; the fighter the shop acts on with the slot ring in the slot steps; the last result as a sticker. Co-op: two lists |
| after a pick | `hud.rs` `picked` | a sticker: "Ember goes to Fox", "Plague Bearer is the rule" |

## Cut, and why

- **The retail percent and stock icons stay.** Our player corners sit above them. Hiding the retail HUD means
  touching game objects that the gate and the replays hash; not worth the risk overnight. So the corner has no
  percent of its own and the stock dots are at the top left, not bottom centre.
- **No character select between fights is not done**: the road (N2) is drawn in the camp scene that replaces the
  VS character select, and A goes on to the retail character select and stage select (bannered by the old V2
  banner). Driving the retail scene chain from the UI straight into the next match is the V2 finding and still
  stands; it is more than a few hours.
- **Pause and results** are retail (Start pauses with the retail pause; the retail results screen runs). The run's
  end is on the road ("Run cleared!" / "The run ended").
- **Second Wind, the Fighters sheet, Collection, NG+ ladder**: the rules have none of them; the camp rows say so
  ("soon", "0 only", "next screen").
- **Sticker pop-in scaling**: stickers enter staggered with a fade; the 0.85 -> 1.04 scale is only on bursts.
- **The keystone pick after the drive pick** (rules-side cut) and the pick timer offline (the sim still runs it;
  the timer disc is drawn when the sim sends one).

## Decisions made overnight

- The Envoy label is ours, drawn on the disc's plate over a pill in the plate's colour (the plate's own label art
  could not be hidden: the label joints are animated). It reads as the plate's face.
- Envoy's menu light is sunny (#FFD84A).
- Families: V2 / V3 families map to the spec's badges RED fire, GREEN wind, BLUE clay hexagon (defence), YELLOW
  lightning, PURPLE frost, WHITE utility; the tier (common / rare / epic) is the rim.
- Twin Soul and Echo Oath share the two-ring emblem; Overclock uses the fuse emblem.
- After a fight the road ignores A for 40 frames, so an A mashed through the results cannot skip it.
- P2's Start joins only when P1's Start is not pressed on the same frame (a mirrored controller cannot join).

## Known rough spots

- Text on stickers tilted under 1.3 degrees can shimmer at 1x.
- Long fighter names cut at the sticker edge.
- The HUD corner for P2 is anchored to the right edge; in a 4-player co-op fight it can sit over a CPU's retail
  percent.
- `MELEE_ENVOY_HIDEJ` (a comma list of cursor joint indices to hide on the Envoy plate) is a leftover check knob;
  no index hid the label.

## How to run

`E:\Projects\Melee Workspace\scratch\envoy-demo\play-v3-solo.cmd` (solo) or `play-v3.cmd` (P2 can join at the camp
with Start). Title, main menu, Down twice to Envoy, A, Start.

Checks: `tools/rust-port/envoy/camp_run.sh` (title > main menu > Envoy row > camp > Start run > three fights, picks
between); `tools/rust-port/envoy/v2_drives.sh`, `v3_slice.sh`; the gate and the replay sets.
Sprites: `python tools/rust-port/envoy/bake_stickers.py` (needs the design folder and playwright).
