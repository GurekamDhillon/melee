# Envoy co-op (offline, two local players, one team)

Off by default. `envoy coop [fighter1] [fighter2] [p1=cpu] [p2=cpu] [seed=N] [loops=N]`, or Setup > Mode (cycles to "co-op" once the engine
offers what it needs; entry 1 then reads "Begin coop as ..."). `envoy coop stop|status|record|rows|ux|tuning [name value|reset]`. A one-player run's
rules and screens are untouched (all pre-existing tests pass unchanged); see "What the campaigns found and fixed" below for the internals both share (items 4-8 and 12 reach a one-player run too).

## What it is
A sequence of Versus team stages (`gd.scene_launch`: `mode=vs;teams=1;items=off;time=0;enemy_team_colors=1`, P1+P2 on team0 with a CPU level
and stock count per slot, the opponents on team1, ports 3..6). The plan of a stage (stage, opponents' fighters, count, CPU level) is a pure
function of (run seed, stage, loop). The run, not the engine, ends the stage: `gd.match_end_hold('envoy-coop')` for the whole stage, a
stage-clear is read from stocks, then `gd.pause` and each player's reward screen in turn (P1 then P2, own offers from a seat-salted seed, the
screen wears the port colour and reads that player's controller; while one screen is up the other seat's strip and the opponent plate are not
drawn), then the next `scene_launch`. After the last stage of a pass (default 8, the final: a pick of three rare/unique) the loop counter goes up
(New Game+), the build carries over, depth jumps as in Classic.

Built by generalising, not forking: one `run_host` per SEAT (`run_host.new(g,mods,retail,seat)`), each with its own `drive_lab` (bag, slots,
keystones, starter, strip, screens, input port) over ONE shared `mod_lab` (one engine: builds, statuses, hit rules are per port 1..6; one ground).
Seat 1 is the lead (opponents, drops, match hold); further seats are followers. No seat = the old one-player host, byte for byte.
Floor drives are the `drive_coop` item (`items/drive_coop/item.json`, ports mask 3): the solo `drive` item only answers port 1, so with it seat 2 could never
touch a drop (found by the campaigns; `tools/port/envoy_bundle.py` keeps both item files in step).

## Rule table (every default is a named value: `envoy coop tuning`)
| question | default | alternatives | what the campaigns showed (2026-10-05, second lane; numbers below) |
|---|---|---|---|
| builds, bags, keystones | one each, nothing shared; different starting keystone | | each seat's strip/inventory separate and readable |
| reward screen | in turn, P1 then P2, own offers | simultaneous (online) | works in a real-time window; the other strip stays hidden (verified, see below) |
| floor drops | `drop_owner=first` (first touch) | `causer`: the drive goes to whoever last hit that opponent (handed over with a card); `both`: duplicated, one each | Nobody starves under any of the three. Fly cursor, seeds 321-323 through NG+2 (72 stages each): floor drives collected P1 / P2 = first 20 / 23, causer 24 / 22, both 43 / 46 (the duplicate rule doubles the supply); the weaker seat's peak build in NG+2 20.7 / 20.7 / 13.1, team strength 24.2 / 24.8 / 20.8, median clear 390-800 frames, win 100% in all. With three seeds the rules do not separate: builds diverge by seed (reward and keystone picks) far more than by ownership. Before this lane seat 2 could not collect at all (solo item ports mask). Kept `first`; `causer` gave the most even split (24 / 22) and no stalls; `both` is the generous option. |
| opponents scale to | `team_formula=max_share` (1+max excess+0.5*other excess) | `sum_dim`, `max` | All three finished every fly run through NG+2. Median clear L0 / L1 / L2 (fly, 3 seeds): max_share 466 / 798 / 591, sum_dim 445 / 476 / 387, max 533 / 662 / 530; opponent strength about 1.2x the team's under the default. Assisted CPU pairs (3 seeds each) cleared 29 (max_share), 32 (sum_dim) and 37 (max) stages before both were out, every run ending at NG+1: the gentler the formula, the further, and none removed the NG+1 wall for CPUs on 3 stocks (they finish all four passes only with `stocks_player=99`). Kept `max_share`: it keeps a two-player team challenged (opponents above the team's strength, clears of 500-800 frames) without a wall; `sum_dim` is slightly kinder when one build is far ahead. |
| opponent count | 2 + 1 per 3 stages + 1 per loop, cap 4 (free ports) | `foes_base/step/loop` | cap 4 reached at NG+1; no run was too crowded to finish |
| stocks | 3 each player, 1 each opponent | `stocks_player/foe` | assisted CPUs do not survive NG+1 on 3 stocks (see below); `stocks_player=99` lets them finish all four passes |
| a player out of stocks | `down_rule=spectate` (back next stage) | `end` | engine keeps the match going while a teammate lives |
| both out | the run ends | `retry_on_loss=1`: stage repeats once | the cursor-flown pair never lost in the 5 default runs of the final code; the assisted CPU pairs lost 8 of 8 default runs, at NG+0 stage 7 to NG+1 stage 4 |
| rewards | every 2nd stage and the final (`reward_every=2`) | | |
| cross-player synergy | ON: statuses are per victim in the engine, so P2's rules read statuses P1 applied | cannot be switched off without an engine status field | Measured in the game (5 seeds x 5 conditions, `synth gear`): the mechanism works across seats and is attributed to the right pair: in the split runs all 24 team chains the engine recorded were cross-seat, every one `plague_bearer>malice` (P1's keystone applies Burn, P2's Malice reads it), counted on the payoff seat, and the synergy visual drew 72 cross-seat link flashes (applier seat to payoff seat, after this lane's fix; before it the flash ran from the payoff seat to the nearest fighter, usually an opponent). Outcome is not measurably better: split vs together vs two self-contained builds vs neutral gave team damage per stage 619 / 620 / 575 / 615 and clears of 176 / 186 / 188 / 160 frames (the synthetic firepower dominates Burn's 3 a second, and Plague Bearer burns its holder). |
| friendly fire | off (teams=1) | | the debug cursor's capsule still hits a teammate (hitlag, no damage): a synthetic-driver issue, see below |

`envoy coop tuning reset` returns every value to its default (verified in the game: `drop_owner both` then `reset` printed `now first`).

## Campaigns (how it was tested without people)
Both seats are driven through the engine's own debug calls and the mod's own reward calls: `envoy coop ... seed=N loops=3` plus `synth start seeded`
(`scripts/coop_synth.lua`, offline only; never part of a normal run). Two policies:
* **fly**: each seat is a debug-flown fighter (`gd.fly_target`, `gd.fly_attack` in burst mode). The seats split the opponents (a seat with no foe of its
  own hovers 70 units above, clear of the other seat's capsule, which also hits teammates), stand 4 units off the target on the stage-centre side,
  arm the burst once (every call re-arms the cycle at phase 0, which is the old every-frame attack), back off for 1.5 s when a foe has taken no
  damage for 8 s, and fly to floor drives (each drive claimed by one seat, the seat with first pick alternating by stage). Firepower stands in for a
  team's offence: 18 damage per burst +4 per loop (cap 30), a burst every 27 frames, every 15 from NG+2.
* **cpu**: both seats are assisted CPUs (`gd.cpu_assist`), level 9.
Reward and keystone choices are seeded (`synth start seeded`). A run is a full pass plus three New Game+ loops (`loops=3`: 32 stages; the rule comparisons ran `loops=2` where noted). Games ran on the
fast virtual clock on the frozen build (about 1000 frames a second); `envoy coop rows` prints one line per stage from the run's own memory (the game's log
drops lines under load).

### Per-loop results


#### fly cursor, default rules, seeds 321-325  (5 runs; complete:32, complete:32, complete:32, complete:32, complete:32)
| loop | stages | win % | median frames/stage | team str | peak build P1 / P2 | opponent str | dealt P1/P2 | taken P1/P2 | floor drops P1/P2 |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 40 | 100 | 474 | 1.5 | 1.5 / 1.8 | 1.5 | 115 / 112 | 5 / 7 | 20 / 19 |
| 1 | 40 | 100 | 574 | 4.0 | 4.0 / 3.4 | 4.5 | 145 / 176 | 19 / 23 | 13 / 13 |
| 2 | 40 | 100 | 552 | 18.5 | 14.8 / 16.7 | 23.4 | 253 / 257 | 53 / 53 | 5 / 5 |
| 3 | 40 | 100 | 305 | 96.1 | 92.2 / 110.8 | 121.3 | 161 / 188 | 67 / 63 | 5 / 5 |

#### assisted CPU, default rules  (3 runs; loss:8, loss:9, loss:12)
| loop | stages | win % | median frames/stage | team str | peak build P1 / P2 | opponent str | dealt P1/P2 | taken P1/P2 | floor drops P1/P2 |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 24 | 96 | 2829 | 1.7 | 1.5 / 2.1 | 1.7 | 203 / 196 | 117 / 106 | 3 / 5 |
| 1 | 5 | 60 | 4322 | 4.2 | 1.5 / 3.7 | 4.9 | 249 / 260 | 402 / 373 | 1 / 2 |

#### assisted CPU, stocks_player=99  (5 runs; complete:32, complete:32, complete:32, complete:32, complete:32)
| loop | stages | win % | median frames/stage | team str | peak build P1 / P2 | opponent str | dealt P1/P2 | taken P1/P2 | floor drops P1/P2 |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 40 | 100 | 2868 | 2.0 | 2.3 / 1.8 | 2.0 | 231 / 200 | 132 / 110 | 0 / 0 |
| 1 | 40 | 100 | 5034 | 7.2 | 8.9 / 4.8 | 8.6 | 353 / 318 | 706 / 698 | 0 / 0 |
| 2 | 40 | 100 | 4199 | 24.1 | 26.5 / 14.1 | 31.3 | 636 / 448 | 1004 / 1097 | 0 / 0 |
| 3 | 40 | 100 | 2754 | 93.5 | 151.5 / 61.1 | 118.0 | 803 / 608 | 806 / 694 | 0 / 0 |

#### drop ownership (fly cursor, seeds 321-323)
| setting | runs | reached NG+2 | stages (loops 0-2) | win % | median frames/stage L0 / L1 / L2 | P1 / P2 floor drops (loops 0-2) | peak build P1 / P2 in loop 2 | team str L2 |
|---|---|---|---|---|---|---|---|---|
| first (default) | 3 | 3 | 72 | 100 | 466 / 798 / 591 | 20 / 23 | 21.5 / 20.7 | 24.2 |
| causer | 3 | 3 | 72 | 100 | 401 / 566 / 436 | 24 / 22 | 20.7 / 23.8 | 24.8 |
| both | 3 | 3 | 72 | 100 | 390 / 548 / 414 | 43 / 46 | 22.2 / 13.1 | 20.8 |

#### team scaling, fly cursor, seeds 321-323
| setting | runs | reached NG+2 | stages (loops 0-2) | win % | median frames/stage L0 / L1 / L2 | P1 / P2 floor drops (loops 0-2) | peak build P1 / P2 in loop 2 | team str L2 |
|---|---|---|---|---|---|---|---|---|
| max_share (default) | 3 | 3 | 72 | 100 | 466 / 798 / 591 | 20 / 23 | 21.5 / 20.7 | 24.2 |
| sum_dim | 3 | 3 | 72 | 100 | 445 / 476 / 387 | 23 / 24 | 21.8 / 22.1 | 24.1 |
| max | 3 | 3 | 72 | 100 | 533 / 662 / 530 | 25 / 28 | 20.3 / 21.6 | 20.8 |

#### team scaling, assisted CPUs, seeds 321-323
| setting | runs | reached NG+2 | stages (loops 0-2) | win % | median frames/stage L0 / L1 / L2 | P1 / P2 floor drops (loops 0-2) | peak build P1 / P2 in loop 2 | team str L2 |
|---|---|---|---|---|---|---|---|---|
| max_share (default) | 3 | 0 | 29 | 90 | 2829 / 4322 / - | 4 / 7 | 0.0 / 0.0 | 0.0 |
| sum_dim | 3 | 0 | 32 | 91 | 3448 / 4172 / - | 1 / 1 | 0.0 / 0.0 | 0.0 |
| max | 3 | 0 | 37 | 92 | 2761 / 4361 / - | 0 / 0 | 0.0 / 0.0 | 0.0 |

#### Cross-player synergy (both seats flown, 4 stages a run, firepower 6 so Burn damage over time shows; seeds 341-345)
| condition | runs | stages | frames/stage | team damage/stage | P1 / P2 dealt per stage | build strength P1 / P2 | chains fired by P1/P2 (of those across seats) | chain pairs |
|---|---|---|---|---|---|---|---|---|
| S split: P1 keystone Plague Bearer (applies Burn) | P2 payoffs Pyre + Malice | 5 | 21 | 619 | 176 | 97 / 80 | 1.32 / 1.09 | 24 (24) | plague_bearer>malice=24 |
| T together: P1 Plague Bearer + Pyre + Malice | P2 neutral | 5 | 20 | 620 | 186 | 104 / 82 | 1.40 / 1.18 | 2 (0) | - |
| C control: no Burn applier anywhere (P2 holds Pyre + Malice) | 5 | 20 | 607 | 166 | 84 / 82 | 1.26 / 1.06 | 7 (3) | plague_bearer>malice=3 |
| D two self-contained Burn builds | 5 | 20 | 575 | 188 | 95 / 92 | 1.42 / 1.26 | 2 (0) | - |
| E neutral | neutral | 5 | 20 | 615 | 160 | 77 / 83 | 1.28 / 1.26 | 0 (0) | - |

#### Script cost of the rule host's frame (one game, fast virtual clock, fly policy, seeds 301 and 302, `loops=3`; ms per frame)
| loop | mean before / after | p95 before / after | max before / after |
|---|---|---|---|
| NG+0 | 0.66 / 0.35 | 1.7 / 1.2 | 38.8 / 8.6 |
| NG+1 | 1.44 / 0.95 | 4.8 / 4.8 | 48.4 / 19.2 |
| NG+2 | 1.60 / 1.25 | 5.7 / 5.9 | 38.3 / 28.5 |
| NG+3 | 1.95 / 1.53 | 6.1 / 6.8 | 32.1 / 210.8 (one wall-clock outlier in one run) |
"before" is the committed tree (624cb0cf8) with the corrected synthetic driver; "after" is this tree before its last small edits. p95 is the p95 of per-2-second windows (about 120 frames each), averaged over the two runs. The before runs hit, in the same game
session, the failures listed above (16 KiB checkpoint refusals, "ran too long", `invalid reserved capacity`, the crit refusal): their loop 3 numbers are for a run that had already lost builds. A deep-loop profile
after the changes (wall clock, with the profiler's own overhead roughly doubling every figure) put a frame at 1.5-2.7 ms: `lab.export` 0.9 (the whole state encoded, 28 `mod_codec.encode` calls a frame, down
from 40-70), opponent rolls 1.9 ms on the 1 frame in 4 that advances one, technique operations 0.15-0.3.


## What the campaigns found and fixed (every item has a Lua test in `envoy_coop_campaign.lua` or `envoy_coop.lua`)
Synthetic driver (not part of the game):
1. Re-calling `gd.fly_attack` every frame re-arms the burst at phase 0, i.e. the old every-frame attack: a foe is hit again the frame its hitlag ends and
   never leaves the stage (the previous lane's stuck runs at 999%). The driver arms once and re-arms only when `gd.fly_state(p).attacking` shows the engine
   disarmed the cursor (a stock lost).
2. Two cursors on one foe (or a cursor next to its teammate) chain hitlag; a cursor 13 units from a tiny hurtbox (Pichu) never connects. Fixed with
   the foe split, the hover, stand-off 4 and capsule radius 14.
3. Foes at depth take about 13% of a hit: firepower scales with the loop.
Mod (these reach a real player, solo or co-op):
4. **"modifier checkpoint exceeds 16 KiB"** (native cap on `sim_commit`'s blob): four opponents' build records (1 KB each) and plates (1 KB each), plus
   status origin lists (5 KB at depth), overflowed from NG+1. A run's checkpoint now leaves out the opponent records and plates and keeps the first six
   origin lines of each status (a run never restores; the LAB keeps everything). Residual risk: none seen in the final waves; `techprobe size` prints the parts.
5. **"sim_commit journal memory budget exhausted"** after about 3500 frames in a stage with six fighters: every frame re-sent twelve overlay and hit-rule
   operations (2.4 KB each, kept for the rewind journal under a 128 MiB budget). A run now sends them only when they change (cheap signatures, a full refresh
   every 120 frames) and rebuilds the blob every tenth frame; the LAB is unchanged.
6. **"ran too long" in a publication** (2 M instructions per callback (and 500 ms of wall time; the wall limit was 50 ms when this was written); 20 errors switch the whole script off, which hung a co-op run at NG+3): probe
   engines re-derived every build from scratch. The derivation memo is now shared between engines (content key, pool, context), `mod_budget.build/values`
   have a bounded content memo, the equipped part of the memo key is cached; a publication that still runs out of budget is **deferred and retried**
   (log line + on-screen "Build update delayed"), a refused one is retried twice then dropped **loudly** (log + toast), and a hosted run republishes from its bags.
   Never silent any more (a checkpoint refusal toasts as well).
7. `invalid reserved capacity` (a floor drive on a full bag): a run may have a drop on a full bag (the pickup asks which to give up), so the LAB's space
   reservation no longer refuses it in a run (it disabled the mod for good before).
8. `crit multiplier_max is below multiplier`: a forced crit above x4 left the multiplier above the (capped) maximum; clamped.
9. Seat 2 could not touch a floor drive (`drive` item ports mask 1): `drive_coop`.
10. The opponent plate showed through the reward grid when a stage cleared inside its four seconds: it now waits while a screen is up.
11. Seen once in the campaigns and not reproduced: `invalid snapshot status` in the probe's import (the message now names the status and the field).
12. **A second run in one game session gave no floor drive at all**: `run_host` never cleared its stage attempt counts or given-drop table at `run_begin`, so every stage the
   previous run had played counted as a retry whose drop was already given (solo too). Cleared at run begin; `envoy_coop.lua` plays two runs in a row.
13. The synergy visual's chain link: a chain between two teammates' records flashes from the applier's seat to the payoff's seat (`synergy_fx.lua`; solo unchanged).

## Netplay readiness
Seeds: `seed_for(seed,stage,loop,k)` only, k = salt per seat. No wall clock in a rule except the reward screen's existing countdown
(`run_screen` `g.time`, to be replaced by lobby ticks online). Every player decision is an event (`take_offer`, `keystone`, `replace`,
`left_behind`, `gain`, `player_out`, `stage_clear`, `screen_done`, `new_game_plus`, `engine_ended`...) in `coop.events`; `envoy coop record` prints
the record and a 64-bit digest (two FNV-1a words over the canonical text and the builds through the sorted codec). `coop.lua` `C.cheats` lists
each place one machine sees both players' choices at once and how online differs.

## Online co-op, what stage 5 delivered (2026-10-08) and what is still stage 6
Delivered (native, `pc/platform/gw_netplay.c`, `gw_netrun.h`; details and API in `ONLINE.md`, "Stage 5"):
* **A lobby mode flag.** The host chooses the room's mode, Versus set or co-op run (`gd.netplay_act("envoymode", "coop")`, `MELEE_NETPLAY_ENVOY=coop`). The co-op mode
  word rides in the handshake's existing Envoy field (protocol 5, no new bytes); both clients report it (`gd.netplay().envoy.mode`). A Versus-only (stage 3) client
  refuses a co-op room at the first packet.
* **A run record both peers keep and compare.** `RN2` carries the seed, the depth (stage), the loop, the score, the mode, the stage order and the build word, and an
  optional `x`: the co-op director's `C.digest` of `C:record()` (seed, every player decision in order, both builds through the sorted codec) goes there with
  `gd.netplay_act("rnote", digest)`, and is then digest-checked by the same machinery that checks a Versus set. The call exists and is tested; nothing in `coop.lua` calls it yet (stage 6 calls it at each stage boundary). A co-op room begins
  its record at once (no first game is needed), resumes it peer to peer (equal digests, or the later one wins), abandons it on both sides, and keeps it when a peer is lost.
* **The lobby refuses to start the match** ("Envoy co-op cannot be played online yet"): the room, the record and the digest check work; the match does not.

Still stage 6 (unchanged from the list below): CPU opponents in an online match, two human slots plus CPUs in the scene builder, seat-to-local-pad mapping, team stage
results synchronised, drops decided at the stage boundary (items are off online), the floor-drive item's port mask set from the lobby's seats.

The checklist `C.cheats` becomes, online: (1) each peer sees only its own offer and the pick is a message the host validates (the stage-3 reward phase already does
this for a Versus set); (2) the countdown is lobby ticks (done for Versus); (3) both builds come from the record (`set_build` is the model); (4) a floor drive's touch is a
game event both peers see on one frame (stage 6); (5) the run seed is the host's (the lobby host chooses it); (6) one process per peer.

## Engine capabilities online (and offline) co-op needs
1. `gd.match_end_hold` applies only while the human in SLOT 0 has a stock. With CPU-assisted players, or P1 out, the engine ends the match
   under the run. Needed: hold while ANY fighter of a named team (or any reason holder) is alive, independent of slot 0 being human.
   Workaround built: `C:scene_ended` reads the outcome from the last frames seen.
2. After a VS match ends the engine re-enters the same seeded scene by itself (a rematch of the `scene_launch` config). Needed: a one-shot
   scene launch (seed consumed at first use) or `on_match_end` that lets the script launch before the re-seed. Workaround: log + `engine_restart` event.
3. A stage-result event for VS (winner team, stocks left per player) instead of polling `gd.player(p).stocks`.
4. Debug attack: `gd.fly_target` raises when the fighter cannot fly (dead, respawning); the burst attack now has real knockback (the 999% problem is gone) but
   (a) the cursor's capsule hits TEAMMATES (hitlag on both, no damage) although `teams=1` has friendly fire off, and (b) every `fly_attack` call re-arms the cycle
   at phase 0, so a per-frame call silently becomes the every-frame attack: a "set once" call that does not restart a running cycle is wanted.
5. Afterimages: `afterimage_add` and `echo_afterimage` refuse with "paired afterimages must share surface and blend for warm traversal" when two
   builds each want a picture. Needed: allow differing surface/blend per fighter. Mitigation built: earned-picture retries every 90 frames.
6. **Per-callback script budget** (2 M instructions, 500 ms wall backstop since 2026-10-06, 50 ms before; 20 errors unload the script): a co-op frame hook does two seats' work plus four opponents' rolls and a
   publication. Needed: a budget the host script may raise for its own callbacks, or a yield point for long checks, and a way to learn the remaining budget (the
   mod cannot count instructions: no `debug`, no `os`). Built around it: shared memo, deferral and retry.
7. **`sim_commit` limits**: the blob is capped at 16 KiB and the journal costs 2.4 KB per operation per frame under a 128 MiB budget. A run has no rewind but pays for
   one. Needed: a "no rewind" commit (operations only, nothing journaled), or unchanged operations skipped by the engine, or a larger blob for two builds plus four
   opponents. Built around it: changed-only operations and a blob every tenth frame, hosted runs only.
8. Online co-op additionally needs (beyond the scoping study): two human slots plus CPUs in the scene builder, seat-to-local-pad mapping
   (a peer's seat 2 is its local pad 1), team stage results synchronised, drops decided at the stage boundary (items are off online), and the floor-drive item's port
   mask set from the lobby's seats rather than fixed at 3.

## Files
`scripts/coop.lua` (director, plan, rules, record, stage rows), `scripts/coop_synth.lua` (synthetic players: `synth start [first|seeded|best] [cpuonly]`, `synth wait <ticks>`,
`synth damage <n>`, `synth gear <seat> <id>...`, `synth chains`, `synth trace <frame>`, `synth refusals`), seat support in `drive_lab`, `mod_lab` (`add_seat`, `techprobe size|ops|prof|cost`),
`run_host`, `run_hud`, `run_screen`, `menu_input`, `items/drive_coop/item.json`; tests `melee/pc/tests/envoy_coop.lua` and `envoy_coop_campaign.lua`.

The unified RN2 record now digest-carries the shared pair stock pool (`stocks`), the single remaining
run-level continue token (`continues`) and a lost-stage flag (`lost`), as well as seed, depth, loop and
both seats' deterministic build history. A lost stage ends both players; the token is spent only by
both seats agreeing. Native record helpers enforce these transitions; stage 7 must wire them into its
director. A disconnect preserves the stage-start record for replay and does not spend the token.
The stage-5 unit tests cover resource round trips, digest differences, strict bounds, single-token
agreement, interruption/resume and abandon. Its two-client proof is owed. Arbitrary co-op drop bags
are not recoverable from the optional `x` digest, and co-op continue-offline remains gated on the director.
