# Gameplay codes (branch port/slpcodes, 2026-11-04)

Opt-in Slippi / community gameplay codes for the Rust build, as match rules. They are our own reimplementations of
each code's documented behaviour. Hero's `scratch/ext/melee-unlocked` was read as reference only (his Injection
lists and the shape of his native sites); no code was copied.

## Using them
`melee play|golden ... --codes LIST` (or `MELEE_CODES=LIST`). LIST is comma-separated: `ucf`, `frozen-ps`,
`frozen-stages`, `no-wobble`, `lgl` / `lgl:N`, `neutral-spawn`, `skip-results`, `handwarmers`, `freeze-fix`,
`stealth`. There are also presets: `tournament` = neutral-spawn + freeze-fix, `stealth` = tournament + stealth. `none`
means nothing on. The log prints `gw: gameplay codes: ...`. An unknown name exits with code 2.

- **Off by default.** Without `--codes` the word is 0, and every hook site runs the game's own code.
- **UCF is the exception.** This build already shipped UCF 0.84 on by default offline (`Ucf_OfflineMode`: the
  `ucf` setting, default 84, `MELEE_UCF`), the same as the C build that the goldens come from. Changing that default
  would move the pad goldens, so it stays. Once `--codes` is given, the `ucf` bit decides (`--codes none` = vanilla
  1.02 controls).
- **Deterministic.** The word is one process-wide atomic, set before boot and constant during a match. Per-match
  state (the hand-warmer decision, ledge-grab counters, wobble counters) lives in statics. A rollback load rewrites
  the statics, so they roll back with the game, and the match-start function resets them. Replays re-run that
  function.
- **Netplay.** `play_net::with_codes` folds the word into the handshake's global-data digest (next to the mods).
  Peers whose codes differ are refused with `different global game data; host has: ...; gameplay codes X`. With no
  codes the digest is unchanged, so vanilla peers still match older builds.
- **Replays.** The `.slp` export writes these fields:
  - the Game Start per-port dashback / shield-drop words (0x141 / 0x145 + 8*port) = 1 when UCF 0.84 is active. Before
    this they were always 0, although the engine played with UCF.
  - the Frozen PS byte (0x1A2).
  - metadata `gameplayCodes` (`<hex word>[:<lgl limit>]`) and `gameplayCodesNames`.

  Not done: playback reading `gameplayCodes` back. UCF and Frozen PS come back through the Slippi fields that
  melee_data already honours. For the other codes, playing our own file needs the same `--codes`.

## The codes, with the behaviour each one reproduces

| code | bit | sites | source of the behaviour |
|---|---|---|---|
| `ucf` | 0x1 | existing ftUcf sites (`ftCo_Turn_Ucf084Dashback`, Guard / Pass / Escape / Damage / DamageFall) via `Ucf_OfflineMode` | UCF 0.84 (slippi-ssbm-asm `External/UCF 0.84/*`, Slippi InjectionLists `list_console_UCF_084.json`: dashback 800C9A44, shield drop 800998A4 / 8009A0B8, pad buffer + 1.0 cardinals, SDI, shield SDI, tumble). |
| `frozen-ps` | 0x2 | `grpstadium` transformation decision (the `Slippi_FrozenStadium` hook) | Slippi "Frozen Pokemon Stadium" (`External/Frozen PS/Core/FreezePokemon.asm` @ 801D45FC + the toggle @ 801D457C): no transformation. |
| `frozen-stages` | 0x4 | Frozen PS; `grStory_801E3334` skips the Shy Guy spawner `grStory_801E3418`; `grLast_8021AAB0` skips the backdrop think `grLast_8021B2E8`; `grOldPupupu` advance never selects state 2 (wind) | Slippi "Frozen Stages" (`list_console_stages_all.json`, `External/Frozen All/Core/1-4.asm`: 803E67E0 = `grOp_803E67D8[2]`, 8021AAE4 in `grLast_8021AAB0`, 801D1548 in `grStadium_801D1520`, 801E3348 in `grStory_801E3334`). Hero's TE text gives the same list in words ("Stadium base form, Dream Land no wind, Yoshi's Story no Shy Guys, FD background still"). |
| `no-wobble` | 0x8 | `ftCo_8008EC90` grabbed-hit path (`inlineB2_wobble`), reset in `fn_800DB790` / `fn_800DBAE4` | Slippi "Disable Wobbling" (`External/PreventWobbling/*.asm` @ 800DB880, 800DBBD4, 8008F090): the grabbed fighter counts distinct attacks from the grabbing Ice Climber's partner, and the 4th breaks the grab (`ftCo_800DA698`). Not in teams (`gm_8016B168`). |
| `lgl` | 0x10 | count in `ftcliffcommon_80081298` (main fighter, successful CliffCatch); ranking in `fn_80165AC0` | Slippi "Ledge Grab Limit" (`list_console_gameplay_lgl.json`: patches in `fn_80165AC0` @ 80165C48 and the sudden-death setup 801A5E90 / 8016EBD8 / 801B04xx): on a time-out, a player over the limit loses instead of going to sudden death. |
| `neutral-spawn` | 0x20 | the existing m-ex / replay Neutral Spawn branch in `gmvs` spawn placement | Slippi `External/NeutralSpawn/NeutralSpawn.asm` @ 8016E510 (in the tournament / stealth / friendlies sets). |
| `skip-results` | 0x40 | `gmVsMelee_ExitVs` (`id0 = 0`), next to m-ex `skip_result_screen` | Slippi's "Show results screen" off and the community "Skip Results" code: an offline VS match goes back to character select. Netplay keeps its results. |
| `handwarmers` | 0x80 | `match_begin` decides; intangible (`x198C = 2` after `ftColl_800764DC`), no push (`ftcommon_8007dd7c`), shared ledges (`ftcliffcommon_80081298`) | 20XX TE hand-warmer mode (TE's menu text, as Hero lists it: "in 1-minute time matches fighters pass through each other and cannot hit each other"). Only in a timed, non-stock, 60-second match. |
| `freeze-fix` | 0x100 | `ftNn_Init_80123954` keeps `x1A5C` | Slippi `External/FreezeGlitchFix/FreezeGlitchFix.asm` @ 801239A8 (the Ice Climbers freeze glitch); already applied for Slippi replays. |
| `stealth` | 0x200 | none (recorded only) | Slippi "Mods: Stealth" / "UCF stealth": the same gameplay as tournament with no on-screen indicator. This build draws no code indicator, so the bit only marks the handshake and the replay. |

### Where the documented behaviour was unclear (the documented version is implemented)
- **LGL limit.** The default is 45, the number I remember from Slippi's LGL option; some rulesets use 60. Use
  `lgl:N` to set another. The real code also edits the sudden-death setup. Here the ranking change alone removes
  the tie, so no sudden death starts when exactly one tied player is over the limit. Two tied players who are both
  over still go to sudden death. Only the main fighter's grabs count (Nana's do not).
- **Wobbling.** Only hits from the partner fighter count, not items or the grabber's own pummels. Nana is not given
  the extra release motion that Hero's native code applies.
- **Frozen Dream Land.** Slippi rewrites the state table entry. I map state 2 to 0 when it is read, which assumes the
  patched value is 0.
- **Hand-warmer skip.** I took "hand-warmers" to be 20XX TE's hand-warmer mode. Nothing named "hand-warmer skip"
  was found in the reference.
- Not done: Kongo Jungle 64 without the barrel (TE's frozen list only); the cosmetic CSS / nametag codes of the
  tournament sets.

## Checks
- `bash tools/rust-port/codes/run_codes.sh [filter]` runs each scenario headless through `melee golden` (the table
  is `tools/rust-port/codes/codes.tsv`, read through the new `MELEE_SCENARIO_TSV`). It compares the off and on hash
  CSVs. Result (2026-11-04): all PASS.
  - Codes that changed the run:

    | scenario | code | first different frame |
    |---|---|---|
    | Stadium | `frozen-ps`, `frozen-stages` | 3744 |
    | Yoshi's Story | `frozen-stages` | 121 |
    | Dream Land 64 | `frozen-stages` | 1149 |
    | FD | `frozen-stages` | 1886 |
    | BF, P1 + P3 | `neutral-spawn`, `tournament` | 1 |
    | BF, 2 minutes | `stealth` | 1 |
    | BF, 1 minute | `handwarmers` | 113 |
    | FD, scripted dashback pad (`ucf_dashback.lua`) | `none` (UCF off, compared with the UCF default) | 862 |

  - Runs equal to the code-off run, where they should be: `frozen-ps` on Yoshi's Story and FD; `handwarmers` in a
    2-minute match; `lgl:0`, `skip-results`, and all the codes together except neutral spawn, on a match without a
    time-out; `--codes ucf` against the default.
  - A second `handwarmers` run equals the first (determinism).
  - `no-wobble`: the CPU Ice Climbers never wobble, so that scenario is equal. The wobble path is not exercised.
- `cargo test -p melee_modes --test codes_lgl`: the LGL ranking, off versus on, at the limit, and without a
  time-out. `cargo test -p melee_core codes`: parse and encode round trip.
- Not verified: wobbling in a real grab; skip-results through the menus; the netplay refusal between two peers with
  different codes (only built); a Slippi viewer reading the new metadata.
