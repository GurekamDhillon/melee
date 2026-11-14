# Envoy body-group tints

Each held item lights the part of the fighter it belongs to. A hand item tints the arms, a movement item the legs,
and a defence item the torso. The tint is subtle while you play, and that body group flares when the item procs.
This builds envoy-items-design.md section 11.12.

## What the C-build roguelite did

Two pieces in the C build did this. `pc/scripts/examples/roguelite/visuals.lua` (with `bindings.lua`) and the
LAB's `pc/gameworld/script_game.c` (`ScriptParts_*`, now `melee_modes::tu::script_game`).

- **Groups.** An offline "part lab" measured each costume's DObjs over several poses. It bound them to three roles:
  assault (arms and hands), traversal (legs and feet) and guard (torso and head). `part_joint_region` walked a
  joint's parents to the nearest bone named in `ftPartsTable`, so the same names worked for every fighter. That bone
  gave one of ten regions: head/neck, bust/waist/hip, each arm and shoulder, each hand, each leg and knee, each foot.
  For skinned meshes, `part_envelopes` let each envelope vote by weight.
- **Tint.** `gd.dobj_tint(port, dobj, colour)` used `ScriptParts_Tint`. It appended one TEV stage after the
  material's: `CPREV * KONST`, with K taken from a konst register the material does not use. The colour was solid,
  set per DObj, in three phases (equipped, charging, ready). Each element had its own palette (cinder, rime). The Lua
  side re-resolved the bindings every 30 frames.

## The Envoy version

- **Groups, from the joint tree.** `melee_modes::tu::envoy_tint` maps the same named bones (`ftPartsTable`
  `part_to_joint`) to six groups: head, torso, arms, hands, legs and feet. Every m-ex or ACE slot that has a parts
  table gets groups the same way as a vanilla fighter. The group is decided per **PObj**, not per DObj:
  - a skinned PObj takes the group with the most envelope weight;
  - a rigid or skinned-to-joint PObj takes its joint's nearest named ancestor.

  So one big skinned DObj still splits into its parts. The table is built once per model (root joint and kind) and
  rebuilt when either changes. A frame only does one binary search per drawn PObj.
- **Draw.** `hsd::HSD_DObjDisp` has a `BODY_TINT` hook (melee_runtime `dobj.rs`). For a fighter in its main model
  pass (`ftdrawcommon::DRAW_FIGHTER`), the hook adds one TEV stage, `colour = CPREV + K`, clamped, with alpha
  untouched. Each PObj then sets its own group's K colour.
  - The stage goes after every stage the renderer has live (`hsd::GX_NUM_TEV`, installed by melee_engine), not
    after HSD's committed count. That way the fighter colour overlay's stage (hit flashes, ftMaterial) is kept. The
    first build overwrote it.
  - The K register is the highest one the material leaves free; the overlay takes the lowest
    (lbGetFreeColorRegister).
  - The GX calls go through `lbspdisplay::ext`, the path the engine wires to the renderer. script_game's own GX
    entries are stubs in the port.
  - It is additive, so the costume and player colours stay readable under it. Shadows, reflections and the LAB's
    own tints are left alone.
  - The hook is installed only on the first present that queues a tint. Until then, and in every non-Envoy mode,
    the draw path is unchanged.
- **API (reusable).** `envoy_tint::{begin, add(port, mask, rgb, strength), commit}` work per present, like
  drive_draw. Wasm UI mods call the host import `gd_ui_body_tint(port, mask, rgb, strength)`; `mask` has one bit per
  group (0 head, 1 torso, 2 arms, 3 hands, 4 legs, 5 feet). Calls on the same group add up, so the rarest-item aura
  (port/envoy-pickup) can call it with `mask = 0x3F` and a low strength alongside the item groups.
- **Families** (`envoy_data::items::body_groups`):

  | Item trigger or kind | Body group | Items |
  | --- | --- | --- |
  | Hits and shots from the hands (nair, jab, laser, smash, whiff, the nair ring) | Arms and hands | Spark Ring, Brimstone Jab, Laser Prism, Smash Quake |
  | Shot modifiers | Arms and hands | Chain Spark, Ice Glass, Homing Ink, Twin Shot, Bouncy |
  | Movement (wavedash, short hop, ledge, roll, airdodge, dash) and Demon Wings | Legs and feet | |
  | Defence and the rest (shine, powershield, shield, hits taken, crouch cancel, KO, pickup, burst modifiers) and Halo | Torso | |
  | Devil deals | Head as well | |

- **Look** (`envoy_ui::items_ui::body`).
  - Every group that holds an item gets 12% of that item's colour; the head gets 10%. When a group holds more than
    one item, the highest quality item gives the colour.
  - A proc, read from the sim's burst ring, flares the item's groups at 60%, fading to zero over 12 frames.
  - The Envoy effect-intensity setting scales all of it.
- **Determinism.** Everything reads the view the sim publishes and writes only draw state. Nothing goes back into a
  sim, so rollback and synctest hashes are unaffected.
- **Checks.**
  - `MELEE_BODY_TINT=0` turns the draw off, for before/after comparisons.
  - `MELEE_BODY_TINT=k` runs the stage with K = 0. Its frames are pixel-identical to the off frames (checked on
    Marth, frames 702 and 984), so the stage changes nothing on its own.
  - `MELEE_BODY_TINT_PERF=1` logs the hook's cost every 600 presents.
- **Measured** (Falcon, Marth, Peach and Fox; fight 1; 1280x720; hidden window):
  - Hook cost per present, all fighters together: 0.011-0.013 ms on average, 0.15 ms at worst.
  - Groups per model, as head/torso/arms/hands/legs/feet PObj counts. Falcon: 21/51/2/22/16/8. Fox: 38/29/6/23/10/8.
    Peach: 30/45/18/42/6/1. Marth: 21/61/8/23/18/5. No PObj was left without a group.
  - A tag partner shares the player's port, so it shows the same tints.
