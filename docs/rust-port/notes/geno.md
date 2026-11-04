# Geno in the Rust engine (branch port/geno)

Geno is the fighter-extension engine (C: `pc/geno/`, `pc/platform/geno_registry.c`; design `docs/geno.md`,
`docs/specs/geno-v5.md`). Its Rust form:

| part | where |
|---|---|
| game half (geno_game.c + its .inc files) | `rust/melee_modes/src/tu/geno_game.rs` (translated), linked into the fighter code by `melee_link` rows |
| LAB mode (geno_lab_mode.c) | `rust/melee_modes/src/tu/geno_lab_mode.rs` |
| game-side test suite (geno_tests.c) | `rust/melee_modes/src/tu/geno_tests.rs`, run by `melee geno-tests` |
| registry (geno_registry.c, attach entries) | `rust/melee_script/src/geno/registry.rs` |
| the registry behind the game half's scalar API (`gw_Geno_*`) | `rust/melee_engine/src/geno_host.rs` |

## Running it

| what | how |
|---|---|
| test suite | `melee geno-tests [--filter SUB] [--scene S]` (needs GW_ISO_VANILLA). Boots a fighter-less VS match (the C suite runs before any scene), then runs each test in a child process (the tests are `extern "C"`: a panic aborts). TAP-like output, `TESTS: pass= fail=` |
| C reference | `_build/agents/sfxparity/melee-pc.exe --test` with `MELEE_TEST_FILTER=geno` (copy it into a private dir first). 64/64 of the game-side tests pass there |
| Geno fighters in a match | mount a mod with a `geno.json`: `MELEE_MODS_DIR=<mods dir>` (or `--mods DIR`, settings `mods_dir`). `melee_engine::geno_host::set_boot_mods` reads every mounted mod's `geno.json` in id order, as `gn_reg` does |
| the LAB | `MELEE_SCENE="mode=lab;p1=kirby/cpu9;p2=fox/cpu9;stage=fd" melee play ...` boots GM_LAB's match (`at=match` / no `at`). `mode=lab;at=css|sss` is refused with a log line: LAB's select screens are the port's kit screens, which the retail flow does not run. `melee golden --scenario geno_lab_cpu` boots it through `Engine::boot_vs`, which now takes the scene's mode (VS or LAB) |
| coverage | group `geno` in `docs/rust-port/golden/coverage/scenarios.json` (below) |

## Status (2026-11-03; base "none" defines 2026-11-04, below)

- **geno-tests: 50 / 64 pass.** Every test that does not need a native `define` passes. The 14 left
  (`geno_none_guard_contact`, the `geno_lua_*`, `geno_combo_*`, `geno_digest_coverage`, `geno_define_snapshot`,
  `geno_none_independence`, `geno_define_hit_tag`, `geno_define_state_hash`, `geno_define_article_digest`) install a
  `define` entry. The Rust registry skips `define` entries (and refuses an attach entry's lua / combo / meter block),
  so they stop at "the test define was not installed". Porting `geno_define_registry.inc`, `geno_define_records.inc`,
  `geno_lua_registry.inc`, `geno_combo_registry.inc` and the fighter Lua core is the next step for them. The attach
  half of `geno_define_hit_tag` and the RB_GameHash half of `geno_define_state_hash` already pass.
- **Coverage: 4 / 4 Geno scenarios match the C goldens hash for hash** (`run_rust.sh geno_kirby_bhv geno_fox_dash
  geno_cpu_kirby_fox geno_lab_cpu`). The goldens are from the sfxparity C lane (`record_coverage.sh --bin
  _build/agents/sfxparity`), each recorded twice and identical. Without the mod the engine diverges at the first special
  (frame 121-151), so the goldens do exercise Geno. The C log and the engine log show the same Geno event lines.

| scenario | what runs |
|---|---|
| `geno_kirby_bhv` | human Kirby (seq pad) with mod `geno-cov`: tornado (neutral B, ground and air, mashed), drill rush (side B, both directions), cape (down B, ground and air), a held midair jump and glide attack input. Fox (multi-dash profile) idle. FD, 2400 frames |
| `geno_fox_dash` | human Fox: the `geno.multi_dash` side special chained, jumps, Kirby (v5.1 profile) idle. Battlefield, 2400 frames |
| `geno_cpu_kirby_fox` | CPU9 Kirby and Fox with both profiles (the CPUs' specials enter Geno states; on_hit events). FD, 3600 frames |
| `geno_lab_cpu` | the same pair in GM_LAB (`mode=lab`): LAB rules (no timer, no stocks, respawn). FD, 3600 frames |

The mod is `tools/rust-port/golden/coverage/mods/geno-cov` (attach entries only, no disc data). A scenario's `env`
(`MELEE_MODS_DIR={wt}/...`) reaches both recorders: `scn.py tsv` expands `{wt}`, `run_rust.sh` exports it.
Glide: `geno_glide` (Kirby, geno-cov) covers entry path A, glide, glide end and glide attack. The trigger
(geno_glide_entry / GenoBhv_GlideEntryStep): a midair jump (jumpsUsed >= 2), the jump held for >= entry_hold (6) frames
AND vy <= entry_vy_max (0.6) while still held. Kirby's midair jump only falls to 0.56 on its 17th held frame, so
geno_kirby_bhv's 45-frame hold (released early by the seq timing) never entered it; geno_glide holds 80. Not covered, defines (native fighters: `vanilla-*` mods), fighter Lua, articles.

## Rules the translation needed (fixed on this branch)

1. **Script words are in the guest's byte order.** Pl*.dat scripts are big-endian and the engine's ftAction loop reads
   them so (`Command_09.id()` bitfields, `u32::from_be` for pointers). The translated geno_game read them natively.
   Now: the overlay pool is stored `to_be` (`geno_pool_sync`), `Geno_FtCmd` and `GenoGame_ScriptHasIasa` read
   `from_be`, and the tests' script builders store `to_be` (as the C says: "stored in the guest's byte order").
2. **No 32-bit pointer round trips for native memory.** `GenoGame_ScriptHasIasa` walked with `(pc as u32) + n`; native
   statics live above 4 GiB, so it now uses pointer arithmetic. Dat pointers (below 4 GiB, `lowmem.rs`) are unchanged.
   The overlay pool itself is an exe static: a `define` overlay's relative Subroutine/Goto operands (stored as 32-bit
   addresses into the pool) would truncate; it only matters once defines are hosted.
3. **One copy of each global.** The translation declared other TUs' globals as zeroed statics of its own
   (`p_ftCommonData`, `HSD_GObjPLinkHead`, `ftPartsTable`, `it_803F14C4`, `player_slots`, `gmVsMelee_StartData`,
   `Geno_ScriptPool` in the tests, ...). They are `pub use`s of the real ones now. Still duplicated: the
   `ft*_Init_MotionStateTable` / `ftData_MotionStateList` copies in geno_game (define-only code; the real tables are in
   melee_link, which melee_modes cannot name) and some in `script_game.rs` (`it_804A0E70`, `it_804D6D18`,
   `gm_804D6720`, `psFormGroupArray`, `gx_suppress_draws`: not Geno's, left alone).
4. **Dropped initialisers.** `geno_buttons` had lost `b = pressed ? pressed_buttons : held_buttons[0]` (every button
   condition read 0). A scan of every zero-initialised local against the C found no other.
5. Callees without a port are hooked in `geno_host::install_callees` (snap_open/save/load over `snapshot::Base`,
   `ftCommon_*`, memset/memcpy) and by name (`geno_host::by_name`, for script_game's `Geno_*` callees). Define-only
   queries answer "no define".

## LAB

The six `todo!`s are gone: `Netplay_Enabled` / `RB_Enabled` read `LAB_NETPLAY` / `LAB_ROLLBACK` (0 offline),
`Replay_Active` 0 (replays set it), `SceneLaunch_BootGameMode` / `SceneLaunch_ItemFreq` are the scene's
(`scenario::boot_game_mode`, `item_freq`), `Script_LabRequest` counts requests in `LAB_REQUESTS`. The LAB's Lua half
(the `geno-lab` mod's scripts, `gw_script.c`) needs the Lua script host, which exists as `rust/melee_script`
(`Engine`, `Host`, `lab_request`) but is not attached to the engine's frame loop yet; until it is, a LAB match runs the
mode's rules and Geno with no LAB UI or tools (the C goldens are recorded with `MELEE_SCRIPTS=0`, so they agree).

## Geno and the wasm mod host (rust/melee_modhost, notes/modhost.md)

They are separate systems that can run together:

| | Geno | modhost |
|---|---|---|
| what a mod ships | data: `geno.json` (attributes, states, behaviours, overlays, specials, articles), optional fighter Lua | code: a wasm module implementing mod-c-v1 |
| where it acts | inside the game's own functions: script opcode 59 (`Geno_FtCmd`), the fighter TUs' `Geno_*` call sites (reset, action change, collision, hit react, motion rows), Geno motion states >= `GENO_MOTION_BASE` (1024) | at hooks-v1 rows (`mod_hook!` at the top of a function): pre / replace / post |
| enabled by | a mounted mod with a `geno.json` (the gdmods folder) | `MELEE_WASM_MODS=<dir>` |
| determinism | game memory + snapshotted statics; the registry is read once at boot | wasmi with fuel; host state in the arena, snapshotted |

Interactions to keep in mind:
- A wasm **replace** of a row whose original contains a Geno call site skips Geno there unless the mod calls
  `gd_orig_call_*`. In melee_fighter's fighter.rs: `fighter.proc01` (`Fighter_8006A360`) runs `Geno_PreAnim` (the
  change-action checks), `fighter.proc06` (`Fighter_procMap`) runs `Geno_CollBegin`/`CollEnd` (ground edge, landing),
  and every `action.*.enter` row reaches `Fighter_ChangeMotionState`, which calls `Geno_OnActionChange` (state reset,
  overlays). `Geno_HitReact` sits in `Fighter_ProcessHit_8006D1EC`, which is not a wired row, so
  `damage.fighter_take_damage` does not bypass it. Pre and post hooks leave Geno intact.
- `gd_fighter_request_action_state` accepts only Wait / Squat / Fall states, so a wasm mod cannot put a fighter into a
  Geno state (>= 1024), and a Geno state is just another action id to the modhost's `action.*` rows (none of the 19
  wired rows is a Geno state's enter).
- Both change netplay identity in their own way: Geno salts a fighter's identity with its profile id
  (`gw_Geno_SaltForPlFile`): wired as `geno_host::netplay_identity`, folded into play_net's `mods_identity` handshake
  word (every attached Pl file's salt + every define's online id; 0 and so unchanged when no geno.json);
  modhost logs each module's FNV-1a-64. A netplay session must agree on both.
- Possible later convergence: Geno's fighter Lua (`geno_lua_*`, not ported) could be hosted on the modhost's sandbox
  rules (fuel, no I/O) instead of a second Lua VM, but Geno Lua is source-level Lua 5.4, not wasm; nothing here depends
  on that.

## Native defines (ported)

- `melee_script::geno::define` (catalog, define parse, presentation keys, audio states, Lua fold, bind to aliases
  127 downward skipping m-ex slots, the query API, records), `geno::lua` (geno_lua_core on the vendored Lua 5.4.7 via
  `mlua::ffi`; the closure scan is a C shim, `third_party/lua-src/extra/geno_lua_scan.c`), `geno::combo` (combo and
  meter blocks). `melee_engine::geno_host::install_define` sets every define hook (ftdata, fighter hooks, per-crate
  copies such as `ftmariospecialn::ext::GENO_DEFINE_BASE_KIND`, geno_game's MotionState table pointers, gm_1601,
  ifstock, lbaudio_ax). geno-tests 64/64.
- Coverage: `geno_vanilla_striker_cpu`, `geno_vanilla_charger`, `geno_vanilla_riposte`, `geno_vanilla_hero_caster`
  (scenario `"mods"` copies `pc/geno/mods/<id>` into `build/covmods/<name>`); C goldens, Rust PASS hash for hash.
- Base `"none"` defines (port/genonone): `geno::plan` (gn_plan_parse / gn_define_plan / DefinePlanInt / DefinePlanText:
  plan.json read from the define's mod `files/`), `ftdata::GenoDefine_BuildOwn` (+ JointFields, OwnTables, Census,
  PartsTable, the row remap, the demo rows as the own Wait row). Coverage: `mex_goku_cpu` (goku-xv2, 713 rows, 126
  joints) and `geno_courier_cpu` (vanilla-courier, 42 joints) PASS 3601/3601 against C goldens (sfxparity lane). Their
  mods are local: `tools/rust-port/golden/coverage/mex_mods.sh` assembles `_build/mex-mods/{goku,courier}` (courier =
  this checkout's text fixture + the original-art assets of `tools/geno/build_courier.sh`, default
  `_build/geno-slice4/courier`). Needed on the way: mod-only files get entry numbers past the FST (gcm.rs; the preload
  cache keeps them as s16, 0x100000 truncated to 0), the `_matanim` symbol suffix (dat_symbols.tsv), the define's
  stock icon (`DefineArtOpen` + a .gxtex handle table in geno_host, `uigen::UI_TextI4` over GDI for the initials
  fallback; render-only). Netplay identity: a base "none" define's resource files' bytes join its handshake part
  (mx_add_defines; a missing file drops it), test `geno_host::tests::identity_covers_base_none_files`.
- Not hosted: the controls schema, the records memory-card
  path (only `MELEE_GENO_RECORDS`), installed skins, the CSS/results art paths (`gm_1601` / gmresultplayer `UI_TextI4`
  and kit textures), the `cape.*` keys of goku-xv2 (the Rust registry logs them as unknown; Goku's CPU match still
  matches C hash for hash, so the C does not use them either there, but the C log was not compared line by line), define overlays using Subroutine/Goto (op 5/7 take a 32-bit pointer: needs the script pool
  below 4 GiB; no shipped mod uses them), Marth / Jigglypuff / Sheik templates of a base "none" define
  (`GenoDefine_ExtAttrBytes` stops).
- Gotcha: a define query hook left at the seams default (-1) silently changes gameplay (Mario-base define fired Dr.
  Mario pills: `Geno_DefineBaseKind` unset in ftmariospecialn). Set every per-crate copy.

## Next

1. Attach `melee_script` (the Lua host) to the engine loop so the `geno-lab` mod's UI runs in a LAB match.

## Define audio (port/audio2, 2026-11-04)
- `geno_host` slice 6: `DefineAudioPlay` (announcer, gm_80168C5C), `DefineVoicePlayOwner` (ft_0877 / ft_0881 voice calls, hook
  `melee_fighter::hooks::GENO_DEFINE_VOICE_PLAY_OWNER`, was unset), `DefineVoiceState` (state-exit / reset cuts), the .gnsnd
  decoder and the per-profile clip cache, played on `melee_audio::Ax`'s native clip slots (shim_ax.c gw_Audio_PlayOwnedClip).
  Same answers as gn_define_audio_play: declared-but-unusable = 1 (silent), stop id 540001, unmapped ids logged once.
  rb_active is taken as false (the engine hosts defines offline only).
- Tests: `geno_host::clip_tests` ports test_geno_gnsnd_audio and the audio-lifetime fixture of geno_define_tests.inc.
- End to end: a copy of vanilla-striker with an audio block (geno 10, synthesized tones; `build/covmods-audio`, not committed)
  in geno_vanilla_striker_cpu: C (sfxparity lane, turbo + MELEE_AUDIO_DUMP) and Rust (`golden --audio`) print the same 24
  define-audio log lines with the same counts (clip loads, 2 voice-line plays of sfx 1, the unmapped ids), and the Rust run
  still matches the hash golden 3600/3600. The WAVs were not compared sample for sample: the C dump includes the boot and
  load phase (14013 vs 12106 sub-frames) and no preroll alignment exists for a geno scenario; the vanilla mix itself is
  only proven bit-exact for pad_falcon_fox_fd. No shipped mod has an audio block, so no coverage scenario exercises clips.
