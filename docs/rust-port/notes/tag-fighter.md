# Tag fighter (prototype, port/tagfight)

Two-character tag teams for the Rust engine, as wasm mods. The idea comes from Joyastick's melee-pc-TagFighter
(README only). This is a clean-room build: no code or assets from that project.

## Play

| what | how |
|---|---|
| mods | `bash tools/rust-port/modhost/build_mods.sh`, then put `build/mods/tag_fighter.wasm` (sim) and `tag_fighter_ui.wasm` (cinematic) in the `MELEE_WASM_MODS` directory |
| teams | `MELEE_TAG_PARTNER=marth` gives port 1 a partner. `MELEE_TAG_PARTNER=marth,falco` sets ports 1 and 2 (empty or `none` skips a port). Names are `melee_engine::scenario::ckind_by_name` |
| tag | press **D-pad down** while grounded and actionable (Wait, walk, dash, run, turn, squat), holding no item and not grabbed. Each port then has a 120-frame cooldown |
| skip | press D-pad down again during the cinematic, from its 6th frame on |

## How it works

| piece | where | what |
|---|---|---|
| spawn | `melee_fighter::tu::player::tag_apply_partner`, called from `Player_80031AD0` | `Player_MexSetPartner(ckind, partner)` before the spawn. `Player_80031AD0` then creates the partner as the sub-fighter with `has_transformation`, so `Fighter_Create` puts it to sleep (`ftCo_800BFD04`). This is the Zelda / Sheik path |
| swap | `melee_modes::tu::tag_swap::swap` | `ftCommon_8007EFC8` (what `ftZd_SpecialLw` / `ftSk_SpecialLw` call) with `ft_8008A2BC` (Wait) as the entry. It carries position, facing, velocity and camera. Afterwards the incoming fighter's own percent is restored (`Player_SetHPByIndex`). The team shares the player's stocks |
| freeze | `tag_swap::freeze_begin` | `gm_SetDbPauseFlag(7)`. `gm_803DA888[7]` (unused by retail) is set to the Start pause's proc mask, so fighters, items and stage freeze while the camera keeps running, and a Start press cannot clear the freeze |
| order | `melee_modhost` frame hook | REQUESTED (`gd_tag_begin`) -> FREEZE (66 frames) -> SWAP + RELEASE in one step after the `frame.run_procs` post chain, between logic frames (the same point where stage_switch applies its change) |
| record | `tag_swap::TRANSITION` (static, so it is in the snapshot) | phase, kind, slot, frame, len, from/to kind, skipped, release clock |
| sim imports | `gd_tag_partner/check/begin/swap`, `gd_transition_phase/frame/slot`, `gd_pad_pressed` | `gd_pad_pressed` reads `HSD_PadMasterStatus.trigger`, the synced sim pad, which stays valid while frozen |
| UI import | `gd_ui_transition(out)` | a read-only 10-word view of the record for presentation mods. The cinematic in `tag_fighter_ui` is driven by the sim clock in it and never writes back |

Shared with the stage switch: `freeze_begin(KIND_STAGE, ...)` and `gd_ui_transition` are kind-neutral. If
stage_switch freezes, applies its `switch_to` at the release, and uses a UI mod reading `gd_ui_transition`, it gets the
same cover and cinematic. The two can then merge into one transition module.

## Checks

`EXE=... bash tools/rust-port/tag/check.sh` runs `pad_falcon_fox_fd` with `MELEE_TAG_PARTNER=marth` and the P1 pad
script `tools/rust-port/tag/tag_p1.txt` (three tags, one skipped). It checks:

- two golden runs give byte-identical CSVs;
- synctest at depth 4 reports 0 mismatches;
- synctest with `MELEE_SNAP_CHECK=1` passes.

`MELEE_PAD_SCRIPT` (txt) now feeds the headless `golden --engine tu` and `synctest` pads (`pads::script_pad`).

## Gaps

- No camera push: the cinematic is letterbox, flash, name card and a slanted wipe.
- The partner is set per character kind. If two ports pick the same main, the later spawn's partner wins the
  mapping (each port's own sub-fighter is still the right one).
- Zelda, Sheik, the Ice Climbers and Geno define pairs cannot be a main or a partner, because they already own a
  sub-fighter.
- `MELEE_TAG_PARTNER` is a local env var and is not in the netplay handshake. Both peers must set the same value.
- The cinematic draws only in `melee play` (the presentation host). Headless runs have no UI.
- No CSS UI for picking a partner.
