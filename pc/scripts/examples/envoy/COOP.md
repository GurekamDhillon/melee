# Envoy co-op (offline, two local players, one team)

Off by default. `envoy coop [fighter1] [fighter2] [p1=cpu] [p2=cpu] [seed=N] [loops=N]`, or Setup > Mode (cycles to "co-op" once the engine
offers what it needs). `envoy coop stop|status|record|tuning [name value|reset]`. A one-player run is untouched (all pre-existing tests pass unchanged).

## What it is
A sequence of Versus team stages (`gd.scene_launch`: `mode=vs;teams=1;items=off;time=0;enemy_team_colors=1`, P1+P2 on team0 with a CPU level
and stock count per slot, the opponents on team1, ports 3..6). The plan of a stage (stage, opponents' fighters, count, CPU level) is a pure
function of (run seed, stage, loop). The run, not the engine, ends the stage: `gd.match_end_hold('envoy-coop')` for the whole stage, a
stage-clear is read from stocks, then `gd.pause` and each player's reward screen in turn (P1 then P2, own offers from a seat-salted seed, the
screen wears the port colour and reads that player's controller), then the next `scene_launch`. After the last stage of a pass (default 8,
the final: a pick of three rare/unique) the loop counter goes up (New Game+), the build carries over, depth jumps as in Classic.

Built by generalising, not forking: one `run_host` per SEAT (`run_host.new(g,mods,retail,seat)`), each with its own `drive_lab` (bag, slots,
keystones, starter, strip, screens, input port) over ONE shared `mod_lab` (one engine: builds, statuses, hit rules are per port 1..6; one ground).
Seat 1 is the lead (opponents, drops, match hold); further seats are followers. No seat = the old one-player host, byte for byte.

## Rule table (every default is a named value: `envoy coop tuning`)
| question | default | alternatives | what the synthetic runs showed |
|---|---|---|---|
| builds, bags, keystones | one each, nothing shared; different starting keystone | | each seat's strip/inventory separate and readable (captures/) |
| reward screen | in turn, P1 then P2, own offers | simultaneous (online) | works; second seat's offers differ (salt) |
| floor drops | `drop_owner=first` (first touch) | `causer`: the drive goes to whoever last hit that opponent (handed over with a card); `both`: duplicated, one each | see RESULTS section |
| opponents scale to | `team_formula=max_share` (1+max excess+0.5*other excess) | `sum_dim`, `max` | see RESULTS section |
| opponent count | 2 + 1 per 3 stages + 1 per loop, cap 4 (free ports) | `foes_base/step/loop` | |
| stocks | 3 each player, 1 each opponent | `stocks_player/foe` | |
| a player out of stocks | `down_rule=spectate` (back next stage) | `end` | engine keeps the match going while a teammate lives |
| both out | the run ends | `retry_on_loss=1`: stage repeats once | |
| rewards | every 2nd stage and the final (`reward_every=2`) | | |
| cross-player synergy | ON: statuses are per victim in the engine, so P2's rules read statuses P1 applied (Lua test: Icebound by P1, Brittle by P2) | cannot be switched off without an engine status field | balance effect: a second build's payoff drives find their primer for free |
| friendly fire | off (teams=1) | | |

## Netplay readiness
Seeds: `seed_for(seed,stage,loop,k)` only, k = salt per seat. No wall clock in a rule except the reward screen's existing countdown
(`run_screen` `g.time`, to be replaced by lobby ticks online). Every player decision is an event (`take_offer`, `keystone`, `replace`,
`left_behind`, `gain`, `player_out`, `stage_clear`, `screen_done`, `new_game_plus`, `engine_ended`...) in `coop.events`; `envoy coop record` prints
the record and a 64-bit digest (two FNV-1a words over the canonical text and the builds through the sorted codec). `coop.lua` `C.cheats` lists
each place one machine sees both players' choices at once and how online differs.

## Engine capabilities missing (specified exactly)
1. `gd.match_end_hold` applies only while the human in SLOT 0 has a stock. With CPU-assisted players, or P1 out, the engine ends the match
   under the run. Needed: hold while ANY fighter of a named team (or any reason holder) is alive, independent of slot 0 being human.
   Workaround built: `C:scene_ended` reads the outcome from the last frames seen.
2. After a VS match ends the engine re-enters the same seeded scene by itself (a rematch of the `scene_launch` config). Needed: a one-shot
   scene launch (seed consumed at first use) or `on_match_end` that lets the script launch before the re-seed. Workaround: log + `engine_restart` event.
3. A stage-result event for VS (winner team, stocks left per player) instead of polling `gd.player(p).stocks`.
4. `gd.fly_target` raises when the fighter cannot fly (dead, respawning): an uncaught error in a frame hook disabled the hook after about 20 frames
   (both games). Synthetic-only; wrapped in pcall. `fly_attack` carries no knockback: a foe at 999% never leaves; the synthetic driver uses `gd.hit`.
5. Afterimages: `afterimage_add` and `echo_afterimage` refuse with "paired afterimages must share surface and blend for warm traversal" when two
   builds each want a picture (the engine's per-callback watchdog then reports on_frame as failing). Needed: allow differing surface/blend per fighter.
   Mitigation built: earned-picture retries every 90 frames instead of every frame.
6. Online co-op additionally needs (beyond the scoping study): two human slots plus CPUs in the scene builder, seat-to-local-pad mapping
   (a peer's seat 2 is its local pad 1), team stage results synchronised, drops decided at the stage boundary (items are off online).

## Files
`scripts/coop.lua` (director, plan, rules, record), `scripts/coop_synth.lua` (synthetic players: `synth start [first|seeded|best] [cpuonly]`,
`synth wait <ticks>`, `synth refusals`), seat support in `drive_lab`, `mod_lab` (`add_seat`), `run_host`, `run_hud`, `run_screen`, `menu_input`;
test `melee/pc/tests/envoy_coop.lua`.
