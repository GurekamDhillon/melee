# Feature parity: C build vs `melee play` on the Rust engine

Read-only audit, 2026-11-02. Nothing was run; every cell comes from docs, the coverage TSVs and greps of `rust/`. Kit menus (`gmfrontend*`, gw_kit,
online lobby/settings kit pages) and the Atlas style are intentionally dropped and are NOT listed as gaps.

Legend. Rust column: OK = works and is evidenced, PART = partial / stand-in, MISSING = not present, BROKEN = present but wrong or crashes, UNVER = built but
nobody has verified it by eye/ear/hand (the docs say so). Effort: S = under a day, M = days, L = a week or more.
Doc paths are under `docs/rust-port/` unless they start with `rust/` or `pc/`. "STATUS" = STATUS.md.

Staleness warning: the per-scenario table in `notes/coverage.md` predates rounds 5 and 6; STATUS "Rust engine vs coverage goldens" is newer
(51 pass, 28 fail, 79 not wired, 10 no golden; round 6 then added about 12 passes). Coverage goldens run 3600 frames of idle/CPU pads, so a PASS
proves the sim for that opening minute, not every special move.

## 1. Boot and menus

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| Boot (card scene, region/logo, intro flow) | full retail flow | PART: `melee play` starts at Opening card; engine boots straight to a VS scene | rust/melee/src/play.rs:1-25 (header), notes/play.md | M |
| Opening movie (THP) | plays | MISSING: a stand-in card (`Scene::Opening`) | play.rs:643, notes/play.md | M |
| Title screen | retail MnTitle | OK: ported retail title (`retail/title.rs`), START exit and timeout to opening | rust/melee/src/retail/title.rs, play.rs:367 | - |
| Title Y/B -> port menu | kit menu | dropped on purpose (kit) | notes/menus.md | - |
| Main menu (1P / VS / Trophies / Options / Data) | retail MnMain | PART: ported (`retail/mainmenu.rs`) but description text, C-stick camera swing and sounds not done | mainmenu.rs:1-12 | M |
| Main menu leaf screens (Event, Rules, Name entry, Snapshots, Archives, Sound Test, Info, Records x3, Multi-Man, Rumble, Sound, Display, Language, Erase) | all work | MISSING: each returns `Exit::Screen(..)`; `play.rs` shows toast "IS NOT IN THE NATIVE PORT YET" | mainmenu.rs:731-763, play.rs:403-408 | L |
| Character select (CSS) | retail MnCharSel (26 chars, costumes, tags, CPU level, teams, handicap, token) | PART: ui.rs rectangles + bitmap font, 2 players, 7 fighters, no costumes/CPU/teams/tags | play.rs:46-54,463-500,720-757; notes/menus.md | L |
| Stage select (SSS) | retail MnStageSw (all stages, random, rules) | PART: stand-in list of 4 stages (FD, BF, Dream Land, Yoshi's Story) | play.rs:56-61,416-419 | L |
| Match loading / VS rules setup | rules from CSS/Rules menu | PART: rules fixed: time match, `--time-secs` (default 180), items off, p3/p4 none | play.rs:518-523 | M |
| Pause / resume / L+R+A+Start quit | works | OK: engine's own pause; quit combo in play.rs | play.rs:561-575, notes/play.md | - |
| Results screen | retail MnVsResult style screens | PART: ui.rs stand-in with winner and halt reason; not the retail results scene | play.rs:613-632,759; notes/audio.md (results audio proven against C, scene not) | M |
| Return to CSS and rematch | works, unlimited | BROKEN: engine statics boot once per process, a second match shows toast "ONE ENGINE MATCH PER RUN: RESTART" | play.rs:507-512 | M |
| Match unlock notices / secrets (Pichu, Mewtwo, stages unlock) | unlock flow | MISSING: no save, nothing is locked/unlocked | engine seam `CARDProbe = -3` (melee_engine/src/seams.rs:97) | M |
| Sound test, Trophies (gallery, lottery, collection), snapshots | work | MISSING: `Exit::Mode("ToyGallery"/"ToyLottery"/"ToyCollection")` falls to the toast | mainmenu.rs:744-756 | L |
| Name entry / tags | works | MISSING | mainmenu.rs:737 | M |

## 2. Game modes

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| VS Melee (time/stock, 2-4 players) | works | PART: 2 human players, time match only in `melee play`; engine VS scenarios with CPUs/4 players run in `melee golden` (not reachable from menus) | play.rs:518-523; STATUS "Rust engine vs coverage goldens" | M |
| VS stock matches / teams / handicap / rules page | works | UNVER: scenes with `stocks=`/`teams=` are "not wired" in the golden runner (not run at all); no menu | notes/coverage.md:53, scenarios.tsv (cpu_teams4, cpu_stock*) | M |
| Special Melee (Camera, Stamina, SSD, Giant, Tiny, Invisible, Single-Button, Lightning, ...) | works | MISSING: menu item goes to toast | mainmenu.rs:758-760 | M |
| Tournament mode | works | MISSING | mainmenu.rs:733 | M |
| 1P Classic | works | MISSING: the engine boots VS only; 11 classic scenarios not wired | coverage.md:53,59 (65 1p scenarios "not wired"), scenarios.tsv | L |
| 1P Adventure | works | MISSING: 12 scenarios not wired | same | L |
| 1P All-Star | works | MISSING: 13 scenarios not wired | same | L |
| Event Match | works | MISSING (menu toast; `event_match` has no golden either) | mainmenu.rs:731, STATUS "Blocked" | L |
| Stadium Target Test (26 chars) | works | MISSING: 26 scenarios not wired | scenarios.tsv, coverage.md:59 | L |
| Stadium Home-Run Contest | works | MISSING; no golden (retail-menu only) | STATUS "Blocked" | L |
| Stadium Multi-Man (10/100/3min/15min/endless/cruel) | works | MISSING; no golden | STATUS "Blocked" | L |
| Training mode | works | MISSING: `Exit::Mode("Training")` toast | mainmenu.rs:732 | L |
| Name / Records / Data (stats, records pages) | works | MISSING | mainmenu.rs:761-763 | M |
| Sudden death after tie | works | UNVER: audio proven vs C for the sudden-death scene, loop to it not driven by menus | notes/audio.md round 5 | S |

## 3. Characters (all 26) and specials

Sim parity is measured only by the 3600-frame coverage goldens (idle/CPU pads) plus 2 scripted pad goldens (Falcon v Fox FD, Marth v Sheik BF, full 1801).
Playable in the menu: 7 of 26 (Falcon, Fox, Falco, Marth, Sheik, Peach, Samus). Engine can load all 26 via `ck:N` scenes.

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| Menu roster | 26 characters (+ Master/Crazy Hand, Wireframes, Giga Bowser, Sandbag in modes) | PART: 7 selectable | play.rs:46-54 (ROSTER) | S |
| Passing coverage goldens (no sim divergence in 3600 frames) | n/a | OK for about 17: Fox, DK, G&W, Luigi, Link, Marth, Ice Climbers, Pikachu, Peach, Jigglypuff, Samus, Sheik, Falco, Dr. Mario, Ganondorf, Pichu (plus later passes: Mewtwo, Yoshi, Young Link, Captain Falcon, Bowser, Mario, Ness, Roy, Zelda per STATUS round 6) | coverage.md:~75-110, STATUS "cov3 round 6" | - |
| Kirby (copy abilities, hat DATs) | works | BROKEN: char_kirby diverges at frame 755 (JObj dirty flag / `lbCopyJObjSRT` count); Kirby hats decode via `dat_custom::kirbyhat` | STATUS "cluster A", triage.tsv char_kirby | M |
| Zelda / Sheik transform, Zelda | works | BROKEN: char_zelda diverges at 2446 (mover cur_pos.x off before `mpColl_80043754`); sheik_cpu_vs_zelda_cpu at 442; zelda_sheik_transform is not wired (Lua pad) | STATUS "Open: char_zelda"; scenarios.tsv | M |
| Bowser, Roy | works | BROKEN per last triage: char_bowser 791 (mplib RNG), char_roy 787 (stage slot proc) | triage.tsv | M |
| Ness, Mewtwo, Mario, Captain Falcon, Yoshi, Young Link | works | FIXED in cov3 round 6 per STATUS (union aliasing); re-verify, coverage.md rows still say FAIL | STATUS "Cluster C result" | S |
| Jigglypuff specials (`ftPr_*`, 80 callbacks) | works | BROKEN: callbacks unresolved ("no canonical"), so Puff's table has no callback on the engine; `ftData_SpecialS[15]` not set | rust/melee_link/retail_unresolved.txt (80 `ftPr_*` lines), STATUS blocker D | M |
| Crazy Hand (`ftCh_*` 40 callbacks + 12 `ftCo_*CrazyHand` capture callbacks) | boss in Classic / Event / Adventure | BROKEN: unresolved, fighter freezes | retail_unresolved.txt (52 lines), STATUS blocker D | M |
| Master Hand, Giga Bowser, Game&Watch demo, Wireframes: costume / demo-motion strings | work | BROKEN: 4 consts have unsupported type `[Option<&str>;N]` (ftMh, ftBo, ftGl CostumeStrings, ftGk DemoMotionFilenames); Master Hand fights untested | retail_unresolved.txt:1-4 | S |
| Panic stubs reached in play (Damage: lbBgFlash, efAsync, Camera quake; JumpAerial; ft_0c8c) | work | BROKEN-by-risk: direct `todo!` stubs "panic when reached (outside the render pass)"; many generated `todo!` are "installed by melee_link" so a raw grep over-counts | STATUS blocker E; rust/melee_fighter/src/tu/ftCo_Damage.rs | M |
| Peach dress dynamics | active | PART: engine runs `Dynamics P1 bone set 0`, the C leaves it inactive (diff only, unhashed) | STATUS blocker F | S |
| Hit corners (shield, clank, smash hitboxes) | works | PART: listed golden blocker 1 | STATUS "Current blockers (golden)" 1 | M |
| Unported fighter states (live free play) | works | UNVER: the old "sim halts at ~107 unported sites" note is from the deleted legacy model; the engine runs real C code, free play with 7 fighters is unmeasured | notes/play.md vs STATUS step 19 | S (measure) |

## 4. Stages, items, CPU

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| Menu stage list | all legal stages + random | PART: 4 | play.rs:56-61 | S |
| Passing stage goldens | n/a | OK (3600 frames) for Pokemon Stadium, Brinstar, Onett, Garden, Mute City, Kraid, Yoshi's Island, Fourside, Green Greens, Poke Floats, Big Blue, Flat Zone, DL64, YS64, KJ64, BF, FD, Castle and Jungle (fixed in round 5) | coverage.md:~90-110, STATUS round 5-6 | - |
| Dream Land (wind), Pokemon Stadium, Fountain `y` platforms vs console replays | match console | BROKEN vs Slippi replays only: platform y at Stadium (16 files), Fountain (8), DL wind (16) | STATUS "slpcheck" S1/S3 | M |
| Corneria | works | BROKEN: diverges at 955 (Arwing joint anim) and 512 owned-gobj cap | STATUS cov3 "stage_07_corneria" | M |
| Great Bay | works | BROKEN: Tingle balloon at 1623 | STATUS cov3 | M |
| Temple | works | BROKEN (maybe fixed): 2477 Sheik chain item state, listed in cov3 round 6 as passing in one line and failing in another | STATUS vs coverage.md | S (recheck) |
| Mushroom Kingdom II (Kyasarin) | works | BROKEN: 803/1254 then crash at 1455 (bad pointer) | STATUS cov3 | M |
| Ice Mountain | works | BROKEN: block-scroll wrap at 1472/1953 | STATUS cov3 | M |
| Rainbow Cruise, Venom, Yoshi's Story, Fountain | work | BROKEN/FIXED mix: Rainbow, Fountain pass (round 6); Venom 2777 hash divergence; YS 590 (Puff callbacks) | STATUS round 6 | M |
| Icetop (Ice Top hidden stage) | works in-game | N/A: C itself asserts (`bgm!=BGM_Undefined`), no golden | STATUS "Failed" | - |
| Akaneia (mod stage) | m-ex stage | N/A: absent from vanilla disc, C crashes | STATUS "Failed" | - |
| Stage hazards that need ported art: Randall (YS), Shy Guys, Green Greens star blocks | work | PART: Shy Guys `it_802D8618` was golden blocker 2 (now passes 3601 on peach_samus_ys); the *viewer* still lacks Randall and GG blocks | render.md round 4; STATUS "cpu_peach_samus_ys_items" | M |
| Items (all 40 kinds, spawner, Pokeball/Assist, Mr. Saturn, Hammer, etc.) | works | PART: items off in `melee play`; items scenarios 8 of 10 pass; item hash divergences: Mario/Yoshi stars (455), Mewtwo Shadow Ball (2116), Yoshi's Story hi (2805), Stadium (3529 `player_id` 192 garbage -> crash) | STATUS "Item hash divergences", triage.tsv | M |
| Item rules menu (frequency, toggles) | works | MISSING (Rules menu toast) | mainmenu.rs:736 | M |
| CPU levels 1-9 | works | OK for 11 of 12 cpulv scenarios; cpulv5_puff_link diverges at 673 (CPU AI output differs from identical state; Puff callbacks) | coverage.md cpulv rows | M |
| CPU in `melee play` | CSS CPU toggle + level | MISSING: stand-in CSS has humans only | play.rs:518-523 (p1/p2 `human`) | S |
| RNG pre-draws at stage init (BGM pick) | real | PART: "dummy draws" `MELEE_ENGINE_PRE_DRAWS` until the boot flow draws for real; per scenario tuned 0..3 | STATUS blocker 4, coverage.md:53 | M |

## 5. Audio

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| AX mixer, SFX, music streams (HPS) | works | OK: sample-exact against the C mix on six captures (opening+title+demo, CSS clicks, KO/GAME!/results, time-up -> sudden death, pause, front-end menu) | notes/audio.md rounds 1-6 | - |
| Engine-driven SFX in a live match | works | OK for Falcon v Fox and Marth (6106 / 843 sub-frames equal); other characters unproven | STATUS "Step 19 wip 2" (git 9bf137719) | S |
| Stage music for all stages | works | UNVER: `start_stage_bgm` exists; no per-stage audio capture beyond FD/BF/DL | play.rs:~535 | S |
| Menu sounds (CSS/SSS/main menu) | works | PART: main-menu streams proven; stand-in CSS/SSS use `css_pick` etc., SSS/trophy menus never driven | audio.md round 5-6 end notes | S |
| Announcer: GAME!, KO, Go, countdown | works | PART: engine issues the calls; timing of GAME!/stock-out unproven; sudden-death scene audio proven for the audio engine only | audio.md "Still open" | S |
| Positional/Doppler SFX, KO paths | works | UNVER: covered only where trace scenarios trigger them | audio.md round 2 | S |
| Volume / sound options (Sound menu: stereo/mono/surround, SFX/BGM balance) | works | PART: master volume only (`MELEE_VOLUME`/audio.cfg); sound options menu absent | audio.md round 3-4; mainmenu.rs:742 | S |
| Surround / Pro Logic II | works (stereo mix on PC) | N/A: C runs mono sound mode 0 and Rust copies it | audio.md round 1 | - |
| Listened-to-by-ear check | n/a | MISSING: "nothing was listened to" | STATUS platform "What only the user can check" | S |
| Music packs / custom BGM (C-build extra) | via mods | MISSING in Rust bin | pc/platform/gw_mods.c; notes/data.md | M |

## 6. Rendering

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| GX -> wgpu pipeline (TEV, fog, textures, lights) | Aurora/Dawn | OK: oracle-verified against Aurora code, engine bridge draws the real frame | notes/render.md steps 1-2, STATUS step 18 | - |
| Pixel parity vs C screenshots | n/a | OK on FD (mean abs error 2.4/255), BF 2.65, YS 3.2; GG 5.8; FoD/DL64 error 30/8; others unmeasured | render.md rounds 1-4 (these predate the engine cutover, so re-measure) | M |
| Fighter colour overlays (entry glow, rebirth ghost, intangible pulse) | works | OK: `colanim.rs` round 3 | render.md round 3 | - |
| Entry sequence (trophy stand, light pillars, PATH beam effect 36) | works | PART: PATH tracks now work for stages; bone-attached effects follow the root only | render.md rounds 2-4 | M |
| Particles (AppSRT quads/points, KO blast sprites) | works | BROKEN/OFF by default: oversized blocky quads, `MELEE_VIEW_APPSRT=1` to enable; check whether the engine path (not the viewer) fixed it | render.md "AppSRT particles ... OFF" | M |
| HUD KO pieces (exploding percent digits, SCORE -1, stock-loss anim, magnifier bubble) | works | PART: noted as not matched in viewer; engine now runs the real IfAll code, unmeasured | render.md round 4 "Not matched (a)" | S (re-measure) |
| Screen flashes (`lbBgFlash`, damage flash, final smash dim) | works | BROKEN risk: `lbBgFlash_80021C48` is a `todo!` in Damage/ft_0c8c | STATUS blocker E | S |
| Camera (cm/camera.c, zoom, KO quake) | works | OK on engine (real code); KO quake from non-ported death states not measured | render.md round 4.6 | S |
| Shadows, reflections (FoD water, Stadium) | works | UNVER: FD has none; FoD/DL64 reflection camera and water "not driven" | render.md round 4 (c) | M |
| Aspect ratio / widescreen 16:9 | 4:3 + `widescreen` setting (Hor+, 60/73 scale) | PART: play window is fixed 1280x960 (4:3); 16:9 projection only applied in headless golden mode; no widescreen setting | rust/melee/src/play.rs:1012, rust/melee_engine/src/engine.rs:126-136 | M |
| Fullscreen / vsync / resolution / MSAA / frame interpolation | display options | MISSING (only AutoVsync swapchain) | grep fullscreen = 0 hits in melee/platform/engine; render.md "Not ported" | M |
| Texture replacement packs, skins, stage skins, shader effects (`gw_fx*`, `gw_shader*`, `gw_skins*`, `gw_stageskin*`) | works | MISSING | pc/platform/gw_fx*.cpp, gw_shader_*.inc; render.md "Not ported / not verified" | L |
| PC palette / surface programs (`gw_surface`) | works | MISSING | render.md step 1 "Not ported" | L |
| Opening movie decode (THP/JPEG) | works | MISSING (card) | shard-945-952-jpeg.md, play.rs:643 | M |
| Screenshots (F-key / gd.screenshot) | works | PART: `--shots` option only | play.rs:905 | S |

## 7. Input

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| GameCube adapter (WinUSB) | works | OK: port of gc_adapter.c, oracle-checked; read a real WUP-028 once | notes/platform.md | - |
| Adapter HID transport | works | UNVER: not tried on hardware | platform.md "Remains" | S |
| Keyboard (WASD/IJKL/Space/Shift/Enter preset) | works | UNVER: unit tests only; interactive use by hand untried | platform.md "What only the user can check" | S |
| Gamepads (gilrs) | SDL gamepads, per-device mapping | PART: SDL-convention conversion, untried; no per-device mapping files | platform.md step 1 "Not ported" | M |
| Controller remapping (`gw_controls*`, profiles in settings.cfg, F9 recalibrate) | works | PART: F9 recal ported; remap profiles and `ctl_poll` remapping not ported; stick dead-zone setting not ported | platform.md step 1 | M |
| Rumble | adapter + Aurora SDL rumble | PART: adapter rumble yes; gamepad rumble not implemented; Rumble options screen toast | rust/melee_platform/src/pad_read.rs:219-224, mainmenu.rs:741 | S |
| Fighter-controls profiles (`gw_fighter_controls*`) | works | MISSING | pc/platform/gw_fighter_controls*.inc | M |
| Lua/live/bot pads (`gd.input`, gw_script_pad) | works | MISSING in bin; melee_script has the pad API but is not linked into the engine | rust/melee/Cargo.toml (no melee_script), notes/script.md | M |
| Linux adapter | works (Linux build) | MISSING (Windows target only) | platform.md step 1 | S |

## 8. Saves and options

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| Memory card / GCI save (unlocks, records, names, rules, snapshots) | works | MISSING in the game: engine seam answers `CARDProbe = -3` so no card; `melee_data::card` exists and is oracle-tested but unused by the bin | rust/melee_engine/src/seams.rs:97, notes/data.md:60 | M |
| Settings (`settings.cfg`: name, delay, widescreen, volume, mods) | works | PART: `cfg.rs` reads skip_intro, audio, mods, mods_dir only | notes/data.md round 3 | S |
| Options menu pages (Rumble, Sound, Display, Language, Erase Data) | retail screens | MISSING (toasts) | mainmenu.rs:741-745 | M |
| Custom Rules page, Names | retail screens | MISSING | mainmenu.rs:736-737 | M |
| Language (JP/US text) | works | MISSING | mainmenu.rs:744 | S |
| Crash logs / reports | works | OK: gw_log port and release crash hooks | notes/data.md:54, git 1a99407d5 | - |

## 9. Netplay and rollback

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| Rollback core (session, savestates, SyncTest) | works | OK: synctest depth 12, 1801 frames golden-equal, engine snapshot/rollback (step 17) | notes/net.md, STATUS step 17 | - |
| Direct host/join over UDP, delay, loss | works | OK on loopback with injected lag/loss (78 rollbacks, 0 desyncs) | net.md "netplay on the real game loop" | - |
| Lobby, room codes, random matchmaking server, STUN, hole-punch | works | OK on local server and loopback | net.md | - |
| Real internet play vs another machine | works | UNVER: never run | platform.md "What only the user can check" | S |
| Wired into `melee play` menus (Online screen) | kit online lobby | MISSING from play; only `melee netplay` command; kit lobby is intentionally dropped, so a retail-style entry is needed | rust/melee/src/netplay.rs, play.rs | M |
| UPnP, Envoy/stage-run/saved-run messages, mod sync, identity exchange | works | MISSING | net.md "Not ported" | M |
| Rematch / rejoin, CPUs and items in the scene, Turbo rules, non-zero controls words | works | MISSING | net.md "Not ported" | M |
| Characters/stages online | all | PART: scene parser handles `ck:N`; proven only Falcon/Fox FD (BF only in lobby) | net.md | S |
| Interop with the C build over the wire | works | N/A (dropped by decision); Rust host -> C guest desynced at index 211 on an effect RNG difference | STATUS decisions; net.md "Rust vs C" | - |
| Slippi online (gw_slippi_*: peer, wire, matchmaking, enet) | works | MISSING | pc/platform/gw_slippi_*.c; no hits in rust/melee_net | L |

## 10. Replays and Slippi

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| `.gdlp` record/playback (`gw_replay`) | works | PART: player + identity gate in `melee golden`; no recorder in the bin, no in-game replay viewer, no embedded-state restore, no CPU players | notes/data.md:60-61 | M |
| `.slp` playback of Falcon/Fox/Marth/Sheik on FD/BF | works | PART: ok on 38 of 125 external files (frame-exact); median first divergence frame 2322 | STATUS "slpcheck, port/replays" | M |
| `.slp` for other stages/characters | works (full roster) | PART: slpcheck now runs 125 of 125; `golden --slp` accepts only ck 0/2/9/19 on stages 31/32 | replays-ext.md; STATUS | M |
| Slippi gameplay codes (UCF 0.84, Whispy, 4 Stadium codes, FD BG seed, etc.) | handled by the Slippi mode | PART: UCF in; Stadium / Whispy / Neutral spawn codes are the open S1/S3 suspects | slpcheck.md; STATUS Leads 1-4 | M |
| Slippi mode (spectate, `gw_slippi_mode`, pad stream, .slp writer) | works | MISSING (reader only) | pc/platform/gw_slippi_mode.c; rust/melee_data/src/slippi_pad.rs, slp.rs | L |
| Rewind / history / savestates for the Lab (`gd.history`, `gd.state_*`) | works | PART: Lua halves ported in melee_script, game half not wired to the engine | notes/script.md round 2 | M |

## 11. Geno and mods

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| Geno fighter (attach registry, v1/v2/v5) | works | PART: registry parser and stable id port; native `define` entries (what shipped mods use) and fighter Lua NOT ported, so shipped geno.json parse to 0 entries | notes/script.md round 3 | L |
| Mods folder overlay (gdmods) | works | OK: `melee_data::modreg` + overlay on the DVD seam, `--mods` in `melee play` | notes/data.md:59, play.rs:525 | - |
| m-ex fighter/stage mods at runtime (PPC code, Gecko) | works (gw_ppc, gw_mex_*) | MISSING: `mex-import` packs files only; no PPC execution, no DOL/Gecko patches | notes/data.md:48 | L |
| Mod stage/fighter in SSS/CSS | works | MISSING (menus are stand-ins; scenes accept `ext:N`) | play.rs | M |
| Nucleus mod browser/installer (`gw_nucleus_*`) | works | MISSING | pc/platform/gw_nucleus_*.h | L |
| Roster registry (6-slot, `gw_roster_*`) | works | MISSING | pc/platform/gw_roster_*.inc | L |
| Lua scripting (`gd.*`, 349 names; scripts, mods) | works | PART: ~200 of 349 names in melee_script (mock-tested); crate is not linked by the `melee` binary | notes/script.md rounds 1-7; rust/melee/Cargo.toml | L |

## 12. pc-port extras

| feature | C build | Rust | evidence | effort |
|---|---|---|---|---|
| Training Lab (geno-lab, `lab.lua`, frame step, hitbox view) | works | MISSING in play: `gd.kit`-less headless run only; Training menu item toasts | notes/script.md round 7, mainmenu.rs:732 | L |
| Debug overlay / workbench / profiler / Tracy (`gw_overlay`, `gw_workbench`, `gw_profiler`) | works | MISSING (a `prof.rs` command exists for timing) | grep workbench = 0; rust/melee/src/prof.rs | M |
| Console (`gw_console.cpp`, `gd` commands) | works | PART: the command parser exists in melee_script, no console window | notes/script.md round 1 | M |
| Hitbox/hurtbox/joint debug draw | works | MISSING in play (Lua API only) | script.md | M |
| Display options: widescreen, safe area, stage/CSS skins | works | see 6 (MISSING) | - | M |
| Scene launcher (`MELEE_SCENE`) | works | OK: `Scenario::parse` ported | rust/melee_engine/src/scenario.rs | - |
| Hang watchdog (`gw_hang`) | works | MISSING | pc/platform/gw_hang.c | S |
| Self-tests (`gw_test*`) | work | OK in other form: cargo tests + gate.sh | STATUS Commands | - |
| Unlocked-launcher integration (melee-unlocked 3rd engine) | n/a | planned, not built | notes/unlocked-survey.md section 1 | L |

## Counts per area

Rows by Rust status (OK / PART / MISSING / BROKEN / UNVER / N/A-or-dropped). Counts are rows, not features, and BROKEN means "known wrong",
so the BROKEN count mixes real bugs with possibly already-fixed items (marked).

| area | rows | OK / PART / MISSING / BROKEN / UNVER / N/A / other |
|---|---|---|
| Boot and menus | 15 | 2 / 6 / 5 / 1 / 0 / 1 / 0 |
| Game modes | 14 | 0 / 1 / 11 / 0 / 2 / 0 / 0 |
| Characters (all 26) and specials | 13 | 1 / 3 / 0 / 8 / 1 / 0 / 0 |
| Stages, items, CPU | 17 | 2 / 4 / 2 / 7 / 0 / 2 / 0 |
| Audio | 10 | 2 / 3 / 2 / 0 / 2 / 1 / 0 |
| Rendering | 15 | 4 / 4 / 4 / 2 / 1 / 0 / 0 |
| Input | 9 | 1 / 3 / 3 / 0 / 2 / 0 / 0 |
| Saves and options | 6 | 1 / 1 / 4 / 0 / 0 / 0 / 0 |
| Netplay and rollback | 10 | 3 / 1 / 4 / 0 / 1 / 1 / 0 |
| Replays and Slippi | 6 | 0 / 5 / 1 / 0 / 0 / 0 / 0 |
| Geno and mods | 7 | 1 / 2 / 4 / 0 / 0 / 0 / 0 |
| pc-port extras | 9 | 2 / 1 / 4 / 0 / 0 / 0 / 2 |
| total | 131 | 19 / 34 / 44 / 18 / 9 / 5 / 2 |

