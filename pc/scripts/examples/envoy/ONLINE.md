# Envoy online (first slices, 2026-10-05)

Status: stages 0 to 3 and 5 of `_research/envoy-netplay-scoping-2026-10-05.md` in the workspace (stage 5, 2026-10-08: the run record, resume, abandon, continue
offline and the co-op mode flag; see "Stage 5" below). An ADVERSARIAL Versus set between two humans over the
port's own rollback netplay, private rooms only, passive builds only. The stages 0?3 proofs below were run on one machine over loopback (two real clients and a
local copy of the matchmaking server). Stage 5 has native/Lua unit proof; its two-client reconnect proof is still owed; nothing was run over the internet and nobody has looked at the menus.

## What a set is

1. The host turns on **VERSUS > ONLINE > Envoy** (SETTINGS > ONLINE > Envoy is the same switch, saved as `envoy_online`) and hosts a room. Random
   Opponent never carries it. The Envoy mod must be installed on both PCs.
2. The handshake (netplay protocol 5) carries the Envoy MODE word (`gw_matchbuild.h`, `GW_ENVOY_MODE_V1`). A guest takes the host's word; a client that
   insists on another, or a protocol-4 client, is refused at the first packet.
3. In the lobby the host's run seed (1..2147483646) travels in an `E` message. Both clients compute BOTH players' builds from it
   (`mod_progression.lua`, `set_starter`, `set_offers`, `set_apply`, `set_build`, `set_stage`: pure functions): one starter drive and one keystone each
   (online-safe records only, so they differ by seat), later picks added at tier 1 or one tier up.
4. Each client stages both builds natively (`gd.netbuild_stage`) and reports a word (`W`) over both players' record digests and compiled ops. The host
   will not start a game unless the words agree ("Envoy builds differ (host H, guest G)"; the guest checks the match's `envoy=` token again).
5. Before frame 0 the native side applies the staged ops once (the rollback session's open, `gw_Script_NetBuildApply`) through the same setters a script
   uses offline, under owner 100. The game-side table (`pc/gameworld/script_build.inc`) records what was applied, lives in snapshotted memory and
   contributes ONE word to `RB_GameHash` (0 when no build is applied), computed from the stored digests AND the live fighter values and hit rules.
6. Between games (before game 2 and later) the lobby opens a reward pick: each player picks 0..2 (an offer) or 3 (keep) with `gd.netplay_act("rpick", i)`,
   the host validates and counts 1800 lobby ticks; no pick takes offer 0; the resolved picks of every game are kept in the lobby (`history`) and both
   clients restage. If a peer leaves, the set is no longer live, but the RUN is kept (stage 5, below): the same two players can resume it, either can abandon
   it, and the survivor can continue it offline.

## Stage 5: the run record, resume, abandon, continue offline, the co-op flag

What two peers must agree on to carry a run is ONE record, `RN2` (`pc/platform/gw_netrun.h`, the same family as the stage-7 stage-run record `RN1`, whose seven
fields it keeps in order): `RN2|<seed>|<stage>|<loop>|<score0>|<score1>|<flags>|<ext>|<mode>|<round>|<winner>|<picks>|<stages>|<bw>|<x>|<stocks>|<continues>|<lost>|<digest>`. In an Envoy set
`stage` is the DEPTH (game number - 1), `round` the last game whose reward resolved, `picks` both players' reward picks (with the seed they regenerate the
BAGS and KEYSTONES: `mod_progression.set_build`), `stages` the stage of every game played (the STAGE ORDER, as content-identity words), `bw` the build word both
sides verified at the start of the latest game, `x` an optional digest a script supplies (the co-op director's record digest, `gd.netplay_act("rnote", hex)`), and
`stocks` is the shared pair stock pool, `continues` the remaining single run-level token (0/1), `lost` a lost-stage flag (0/1). All three join the digest;
The host sends the pool in its existing lobby state message so the guest's own stock menu setting cannot change it.
A lost stage sets shared stocks to zero and ends the pair, and only both seats agreeing can spend the token. Stage 7 wires these record transitions to gameplay.
The digest is two salted FNV-1a words, the same function as `mod_codec.digest64(body, "netrun:")` (a literal is checked on both sides). Integers only, so the text is
byte-identical on Windows and Linux. It adds no simulation state: nothing is in the snapshot or the rollback hash.

* Both clients write it to `settings.cfg` (`envoy_run`, `envoy_run_state`, `envoy_run_me`) at the same boundaries: a game starts (the first one begins the record),
  a game ends, a reward resolves. The state (`active`, `interrupted`, `abandoned`, `continued`) and the seat are kept beside it, not in the digest.
* **Resume.** On entering the lobby of a set that has not begun, the guest reports its saved run (`U`), the host compares: both resumable, same mode, the right seats.
  Equal digests: the run continues (the host sends the record in `T` chunks and `Y 1 <digest>`; the guest applies it and says whether its own copy was the same).
  One record is the other plus later boundaries (same seed, every shared pick and stage equal): the later one wins and the lagging side adopts it. Anything else
  (a different run, a record on one side only, another mode, an abandoned run): a NEW run, the older record kept as `envoy_run_prev`, and the host may choose with
  `gd.netplay_act("rresume", 1|2)` (its own / the guest's) after two different runs. READY waits for the check (an older guest that reports nothing delays it by 3 s).
  The game that was being played when the connection broke is replayed: the record is at the START of that game, with its build word.
* **Abandon.** `gd.netplay_act("rabandon")` (either player; the guest asks, the host decides): the host transfers its exact record first, both mark it `abandoned` with the SAME digest, clear the run
  and the host starts a new one (another seed) in the same room. An abandoned run is never offered again.
* **A lost peer** (a blackout, a quit, a killed process): the live set ends but the record becomes `interrupted` on the survivor (and stays `active` on a side that
  never learned), and the lobby note says so. A connection lost IN a match is found at the next lobby: the game started and never ended, so it is replayed on resume and the run is
  interrupted there.
* **Continue offline.** `gd.netplay_run("saved")` returns the record parsed (offline too); the console command `envoy continue [classic|adventure] [fighter]` turns the
  survivor's build into an offline run carrying the recorded depth and loop, seeded with the set's seed (`mod_progression.set_carry`: each held record becomes a common white drive with that modifier at its
  tier, the keystones go in the keystone list; `run_host:carry_in`) and marks the record `continued`. A record whose depth floor is deeper than its online tier (pyre, cinder,
  shatter, keen, brutal, ruthless, finishing) carries at the floor's tier: the offline bag cannot hold a drive below its depth tier.
* **Co-op flag.** The host picks the room's mode: `gd.netplay_act("envoymode", "off"|"versus"|"coop")` or `MELEE_NETPLAY_ENVOY=coop`. The co-op mode word
  (`GW_ENVOY_MODE_COOP` 0x45560002) rides in the same protocol-5 handshake field as the Versus word: no new bytes, no protocol change; a stage-3 client refuses it at
  the first packet. A co-op room has the lobby, the run record, resume, abandon and the digest check, but READY is refused with "Envoy co-op cannot be played online yet"
  (the match needs CPU opponents in an online match: stage 6). The menu row is still a Versus on/off switch: the co-op choice is a script action for now (owed).

Calls added: `gd.netplay().envoy.mode` (`off|versus|coop`), `.run` (`status` none|checking|resumed|fresh|conflict|abandoned|interrupted, `pending`, `conflict`, `live`, `resumed`,
`abandoned`, `interrupted`, `note`, `note_seq`, `record`), `gd.netplay_run([live|saved|previous])`, `gd.netplay_act("rabandon" | "rresume", n | "rnote", hex | "rstate", "continued" |
"envoymode", mode)`, console `envoynet run`. Env: `MELEE_NETPLAY_PORT=<n>` (the UDP port a menu-hosted room opens; two lanes on one machine must not fight over 51500).
Tests: native `netplay_envoyrun_record|events|resume|abandon|coop`; Lua `pc/tests/envoy_online_run.lua`; standalone native resources `pc/tests/envoy_netrun_test.c`; the two-client proof `tools/netplay/np_envoy_resume.py` in the workspace (owed).
The earlier unpublished 16-field RN2 draft is refused; both peers must use this updated build.
Arbitrary future co-op drop inventories cannot be recovered from `x`: it is a digest only. Co-op offline continuation needs the stage-7 director and is currently refused.

## API (all in `gd`)

| call | meaning |
|---|---|
| `gd.netbuild_stage(slot, record, ops)` | stage one player's build for the next online match. slot 1 = host, 2 = guest. `record`: `mod_codec.build_record` text (`EB1\|...`). `ops`: the sim_commit op list from `engine:passive_ops(port, slot)`; only `fighter_mod`, `hit_rules`, `fighter_caps`, `fighter_armor`, `crit`, only the slot's own port. Refused during an online match. |
| `gd.netbuild_clear([slot])` | forget one or both |
| `gd.netbuild()` | READ-ONLY, works online: `{staged, word, applied, slots={ {staged, record, digest, ops, nops, active, applied_record, applied_ops, applied_nops, live, values, rules, rules_word} }}`; `values` and `rules` are read back from the game's own tables |
| `gd.netplay().envoy` | `{on, seed, open, round, left, picks, history, refused, fail, word, peer_word, peer_reported}` (lobby ticks, not milliseconds) |
| `gd.netplay_act("rpick", i)` / `("envoy", on)` | the reward pick; the host's Envoy preference (the menu row's own setter) |
| console `envoynet status\|auto <0..3\|x>\|pick <i>\|tamper` | `auto` picks for a driver; `tamper` is a TEST hook that stages a different build on this client |

Environment (scripted runs): `MELEE_NETPLAY_ENVOY=on\|off` (direct-connect path: the host's choice, or what a guest insists on),
`MELEE_ENVOY_POISON=1` (test: flip one bit of slot 1's record word after the apply, so the rollback hash must report a desync).

## Which records are online-safe

Pool: 86 records, 38 passive (`equip`), 48 triggered. ONLINE-SAFE NOW: the 33 passive records whose effects are all `value`, `convert`, `versus-status`,
`crit`, `armor`, `air_jumps`, `restrict` (`engine:online_safe(rule)` says why a record is not). NOT online-safe: the 5 passive echo records (`trailing`,
`echoes`, `echo_heart`, `echo_oath`, `echo_weaver`: they need the echo journal and its presentation) and all 48 triggered records (statuses, stacks,
heals, interrupts, `crit_next`: they need the native evaluator, stage 4 of the study). `lingering` is online-safe but inert in a passive-only build
(it only lengthens statuses).

## Files

Lua: `mod_codec.lua` (record, digests, seeds), `mod_engine.lua` (`passive_ops`, `online_safe`), `mod_progression.lua` (the set), `mod_lab.lua`
(`net_sync`, `net_draw`: the lobby glue and the reward screen), `menu_input.lua`/`drive_lab.lua` (no input mask or chord online: the engine refuses
them, and refusals disabled the script). Native: `pc/platform/gw_matchbuild.h`, `gw_script_netbuild.inc`, `gw_netplay.c` (mode word, lobby reward phase,
`E`/`W`/`X` messages, `envoy=` token), `gw_net.c/.h` (protocol 5), `pc/gameworld/script_build.inc`, `src/melee/ft/fighter.c` (the hash word),
`src/melee/gm/gmfrontend*.c/.inc` (the toggle and the lobby's one instruction line).
Tests: `pc/tests/envoy_online.lua`, `envoy_online_build.lua`, `net_proc_helper.lua`; native `netbuild`, `net_*envoy*`, `netplay_lobby_envoy`.

## Online Classic director (stage 7a/7b, 2026-10-08)

Host a private room with `MELEE_NETPLAY_RUN=classic` (or `gd.netplay_act("run", "classic")`). The handshake uses the additive run mode word
`0x45560003` in protocol 5; no transport packet or protocol version changes. Envoy builds and CPU opponents are not part of this slice.
Both humans play ordinary GS_VS placeholders. The Classic plan retains the retail battle rows, difficulty, matchup stages and opponent kinds as data;
bonus rows and Adventure special stages are skipped. Played arenas come from the seeded agreed lobby stage list.

Each save advertises unlocked fighters, mapped through content identities and intersected with installed common fighters. Selection is seeded, avoids
repeat opponents and the humans where possible, and never relaxes the unlock mask. An intersection that cannot fill a battle row refuses READY.
The boss row remains data for stage 6. A reordered mod fighter installation can refuse the plan digest; vanilla IDs are the supported proof target.

The pair has a shared pool (default 12), charged for both humans' lost stocks at each stage boundary. A cancellation or no clear loses the stage for
both. An exhausted pool holds the failed stage and offers one agreed continue: both READY votes spend it and refill the original pool; another loss
ends the run for both. RN2 carries the pool, remaining token, loss flag, packed director state, chosen stage identity and Classic-plan digest.

Before scene teardown both peers compare epoch, exit frame, a final checksum of the last simulated frame, winner, remaining stocks, stage and pool.
Missing checksum, tuple mismatch or timeout aborts without advancing the director. The lobby shows the next stage, opponents, pool, continue or
terminal outcome, replacing retail results/sudden death and retail save side effects. The blocking exit barrier cannot apply a pending rollback
correction: it safely aborts; a future frame-loop exit gate and real-client lag/loss proof are still needed.

`gd.netplay().run` adds `kind`, `len`, `pool`, `pool_start`, `cont`, `cont_used`, `over`, `asking`, `ends`, `each`, `mask`, `plan`, `last`, `block`,
`title` and `line`. `over` is empty while live, then `stocks`, `cleared` or `aborted`. `record` remains the digest-checked RN2 string.
Test overrides: `MELEE_NETPLAY_RUN_LEN`, `_POOL`, `_CONT` (0 or 1), `_STOCKS`; `MELEE_NETRUN_UNLOCK` (hex mask), `MELEE_NETRUN_POISON=1|mask|barrier`.
Native: `build.sh --native-test netdir`, `--native-test envoy-netrun`; integrated headless: `run.sh --test`.
