# LAB theorycraft: next-generation training features on the deterministic engine (2026-11-08, port/labtheory)

Design exploration only. No code, no UI: behaviour and engine mapping. The user designs screens separately, one step at
a time. Input was a generic fighting-game list (Tekken-flavoured); this note translates it to Melee and to what this
engine measurably does. Everything "exists" below was checked in the repo at `pub/rust-port` db9d048e4; every number is
from `notes/perf.md`, `notes/wasm-ui-lab-api.md`, `notes/lab-ui-spec.md`, `notes/spectate-replay.md`,
`notes/slpcheck.md` and `STATUS.md`, with the date it was measured.

## 0. What we are building on (verified)

| capability | state | source |
|---|---|---|
| sim tick | 0.20 ms (pad_falcon_fox_fd) / 0.53 ms (cpu_fox_falco_dl), release, headless; so **30-80x real time** per core | perf.md 2026-11-01 |
| full-state snapshot | save 0.24-0.29 ms, load 0.35-0.44 ms (epoch pages, watched statics); page sharing between consecutive snapshots | perf.md 2026-11-05 |
| rollback | depth-7 resim 7 x 0.47 = 3.3 ms; whole depth-7 synctest tick 4.0-5.6 ms | perf.md |
| LAB ring | a snapshot every 8 frames, depth 600 frames, 58 MB, 1.25 ms per capture = ~112 us/frame amortised; slot save 1.8 ms, load 1.4 ms, rewind ~2.2 ms | lab-ui-spec.md status |
| determinism | bit-identical to the C build on the four goldens, seed included; synctest 0 mismatches; 38 of 125 external Slippi tournament replays frame-exact against the console's own post-frame data, the other 87 diverge for listed reasons (Stadium/Fountain platform state, DamageFlyRoll RNG draw count, hit-timing, Dream Land wind) | STATUS.md, slpcheck.md |
| `.slp` | `melee slpcheck` / `melee golden --slp`: Game Start converted to a scenario (any stage, ports kept, costumes, stocks, timer, UCF words, Frozen PS), inputs replayed, post-frame compared each frame. **Headless only, no window, no controls** | slpcheck.md, spectate-replay.md |
| `.mrpl` (our own replay) | `melee play --replay` with pause / step / back / speed 0.25-8 / seek; back-seek loads the nearest match snapshot (every 300 ticks) and resims | spectate-replay.md |
| mod host | wasmi 2.0 interpreter, fuel-metered (1 M per sim hook, 4 M per presentation call), fixed memory <= 1 MiB (16 pages), 16 KiB stack, imports only from `gd` / `gd_ui` / `gd_lab`; sim stores live in the arena (roll back), presentation stores are harness-owned (never snapshotted) | modhost.md, wasm-ui-lab-api.md section 0 |
| LAB mod today (`rust/mods/lab`, 2.6 k lines, no_std, no alloc) | freeze, step +-1/+-10, 600-frame scrubber with hit/save marks, take over (`go_live`), 16 slots, frame meter with stun hatch, observed move readout, state readout with action names, dummy panel, combo panel, input display, full menu (Dummy / Record / Display / HUD / Session), radial, command bar, two-player and online (`LabNetplay`, lab commands in the lockstep stream, 0 desyncs in `lab_pair.sh`) | lab-ui-spec.md status, lib.rs header |
| host side (`rust/melee/src/lab.rs`, `LabRun`) | sparse ring, record / play (loop, mirror, from-slot, ordered / random / weighted sets), dummy synthesiser (stand / crouch / jump / shield / CPU level / recording; reactions DI, SDI, tech, getup, ledge, shield-drop, OoS with delay, strength, chance, weights), events (hit, shield, tech, L-cancel, powershield, wavedash, ledgedash, SDI, ASDI, airdodge), frame-meter cells, observed frame data, overlay pass (hitbox, hurtbox, ECB, DI line) | wasm-ui-lab-api.md "half B" |
| CPU AI | the retail AI is ported (`melee_fighter/src/ftcpu.rs`, `tu/ftcpuattack.rs`); `CPU_LEVEL` sets the retail level and the port must be a CPU for it to drive | wasm-ui-lab-api.md |

Three facts shape every design below:

1. **One engine per process.** The snapshot rule says every writable static of the exe is sim state, and the arena
   belongs to the game thread (`arena::GAME_THREAD`). There is no second `Engine` to run a search on another thread.
   Lookahead must either (a) time-slice the one engine on the game thread between presented frames (save, simulate N
   frames, load back; the player sees nothing because presentation is harness-owned), or (b) run in **worker
   processes** that boot the same scenario and receive a frame number plus input log, or a serialised state
   (snapshots hold process addresses, spectate-replay.md records that they cannot cross processes; the deterministic
   boot plus input log is the portable snapshot).
2. **The Lab is a presentation mod**: it cannot write the sim except through `gd_lab_*` commands and the input boundary,
   it has 1 MiB of memory and 16 KiB of stack, and it is never rolled back. Anything that needs large tables (a ghost
   model, a search tree, a replay's frame list) or sim-side control (forking the state, driving a port's pad from a
   policy) is **host-side**, exposed through new `gd_lab_*` queries and commands. The mod stays the controller and the
   readout.
3. **Snapshots never leak.** Lab state (ring, slots, search scratch) is a field of the play flow on the system heap;
   the gate refuses `gd_lab_*` under `Netplay`, `Replay`, `Synctest` (returns `EOFFLINE`), and entering netplay ends the
   lab session. Every feature below keeps that: search scratch dies with the lab session, and lab-only writes
   (`set_pos`, `set_percent`, pad injection) stay inside `LabRun` and are logged as "forks the timeline".

## 1. Interactive replay takeover

### Melee terms
Open a Slippi `.slp` (your tournament set, a top player's) or our `.mrpl`, scrub to any frame, freeze, step, inspect,
then **take one port** and play from frame K against the recorded inputs of the other port (which keep playing as
recorded until they stop making sense, then hand over to a dummy policy), with auto-rewind to K after M frames or on
a condition (you got hit, you died, you landed the punish, the opponent reached the ledge, 1 stock lost).

### What exists
- `.mrpl`: playback with seek is live (`replay_ctl.rs`); snapshots every 300 ticks; take-over is not wired in the
  replay path but the lab's `rewind` / `go_live` / logged-future machinery (`LabRun::tick` step 2: "the logged pad
  while replaying after a rewind") is exactly the primitive: a replay is a 100 % logged future.
- `.slp`: `slpcheck::convert` builds the scenario from the Game Start block (stage, ports, costumes, stocks, timer,
  UCF, Frozen PS) and the per-frame pre-frame inputs feed the sim. The reader is `melee_data::slp`. Only headless today
  (`golden --slp`, `slpcheck`), and only 38 of 125 tournament files stay frame-exact to the end.
- Lab ring, slots, scrubber, step, take over, the frozen-frame readouts: in the mod and `LabRun` already.

### Mapping
Host-side (`LabRun` + a replay feed):
- A **replay feed** as a pad source: `LabRun::tick` step 2 gains "the replay's pad for this port and frame" ahead of
  the live pad; the lab's existing logged-future path then does scrub, step, rewind and `go_live` unchanged. Take-over
  of one port = `go_live` for that port only (today `go_live` is global; needs a port mask).
- **Keyframes**: the ring already is one (every 8 frames, 600 deep). A replay is finite (a 3-minute game is 10 800
  frames): a sparse keyframe set every 300 frames costs 36 snapshots; at 0.3 ms each and sharing unchanged pages they
  are cheap in time, and in memory about the ring's ratio (58 MB per 75 snapshots when 8 frames apart; 300 frames
  apart more pages change, so budget ~1-2 MB per keyframe = 40-70 MB per game; measure). Seeking to any frame then
  costs load (0.4 ms) + at most 299 resim ticks (60 ms) — well under one presented frame of perceived delay only if the
  seek runs unpaced on the game thread with presentation held; otherwise two frames. A denser ring around the current
  playhead (the existing 8-frame ring) gives frame-precise scrubbing locally.
- **Divergence handling**: once the human takes over, the recorded opponent's inputs are replayed verbatim, which is
  correct for a few frames (they were already committed) and then nonsense. Behaviour: the recorded inputs play for a
  configurable "commitment window" (e.g. until the opponent's next actionable frame as observed by `cell`), then the
  port switches to the dummy synthesiser (CPU level N, or a recording, or the ghost of section 2), with the switch
  frame marked on the scrubber.
- **Auto-rewind**: a lab command `OP_LOOP(anchor_slot, frames, cond)` applied in `LabRun::tick`: on frame anchor+M, or
  when `cond` fires (conditions evaluated from the published fighter structs and events: port X took damage / lost a
  stock / grabbed ledge / became actionable / entered action A), `load(anchor_slot)`. Conditions are the same event
  kinds the toasts already use. Online (LabNetplay) the loop command rides the lockstep stream like every other cmd.
- **Replay-exact vs lab-exact**: a `.slp` that diverges from the console at frame F (87 of 125 do) is still a valid
  lab situation up to F: the lab is playing our sim's rendition of the inputs. Expose the slpcheck divergence frame on
  the scrubber ("console-exact until frame F") so the user knows what they are labbing. For the 38 exact files
  everything is the real game.
Mod-side: scrubber marks for keyframes and the divergence frame, the loop setup rows, the condition picker, the
"recorded until / dummy from" marker. The mod's 1 MiB is fine: it holds no replay data; it queries
`gd_lab_events_range` as today.

### Cost
- Seek: load 0.4 ms + 0-299 resims at 0.2-0.5 ms = up to ~150 ms worst case (cpu scenario) once per seek; 8-frame ring
  around the playhead keeps scrubbing at ~2 ms. Per-frame steady state unchanged from the lab today (~112 us ring).
- Memory: keyframes 40-70 MB per game (estimate; measure), plus the 58 MB ring.
- Loading a `.slp` in the windowed flow: slpcheck boots the engine with the file's scenario in ~0.2 s (boot) and the
  VS load ~80 ms.

### Determinism / netplay
Offline only (or LabNetplay with both peers holding the same file: the replay would have to join the handshake
digest, as wasm mods do: `file#fnv32`). Nothing new touches the arena outside `LabRun`. A `.slp` with Gecko codes the
sim does not emulate stays a lab fork; the divergence frame is shown, never silently hidden.

### Risk: **medium**. The replay feed is a small change to `LabRun`; the windowed `.slp` path (today headless) is the
real work: hooking `slpcheck::convert` into `retail_flow`'s lab boot and the browser (`GD_UI_REQ_REPLAY_PLAY` already
relaunches `--replay` for `.mrpl`; `.slp` goes to `golden --slp`). Divergent files may feel "wrong" to a player who
expects the console game; the marker mitigates.

### Size: **M** (feed + port-masked go_live + loop command: S; windowed `.slp` boot + keyframes + markers: M).

## 2. Behavioural-cloning ghost and optimal-punish bot

### Melee terms
- **Ghost**: a dummy that plays like a particular player (you, your rival, a top player) learned from their `.slp`
  files: given the situation (both fighters' action state, frame in action, relative position, percents, stage zone,
  ledge/platform context), what does this player press? Melee-specific signals: tech choice distribution by percent
  and position, DI habits, ledge option mix, recovery mix, OoS option after a given move on shield, approach option
  from neutral at a given spacing. Those distributions are **already the lab's reaction tables** (tech weights, DI
  weights, ledge options with delay, OoS with delay): the simplest ghost is a tool that *fills those tables from
  replays* rather than a new AI.
- **Optimal-punish bot**: given a frozen state where the opponent is in a committed state (whiffed move, missed tech,
  shield-stunned, hit in hitstun), find the input sequence for our port that maximises damage / KO / positional gain
  under the opponent's best DI / tech / SDI. This is search, section 4, with the ghost (or uniform) as the opponent
  model.

### What exists
- `melee_data::slp` parses pre-frame inputs and post-frame state per fighter per frame: that is the training set,
  (state, input) pairs, 1.29 M frames in the 125-file corpus.
- Dummy synthesiser with weighted options and delays; `gd_lab_reaction_weights` (8 x 4-bit weights per kind); lab RNG
  for deterministic random draws.
- Recording slots: a recorded pad sequence is a "macro" the ghost can emit.
- Retail CPU AI (`ftcpu.rs`) as a fallback policy for frames the ghost has no opinion on.
- `cell` (idle / startup / active / recovery / stun / can-act) and action ids in `gd_lab_fighter_v1`: the natural
  state-signature features.

### Mapping
Three tiers, each useful alone:

**Tier A: replay-derived reaction tables (statistics, not AI).** An offline tool (`melee ghost --slp DIR --player TAG
--out ghost.toml`) runs slpcheck's reader (no sim needed) and counts, per situation bucket, what the player did: tech
option by (percent band, facing, distance to ledge), DI angle class by (move id that hit them, percent band), ledge
option by (opponent distance, opponent state), OoS option by (move on shield, shield stun), recovery mix by (height
band, distance). Output: the lab's existing reaction weights plus a few new reaction kinds (RECOVERY, APPROACH) and
delays as a profile the mod loads with one new command (`OP_PROFILE`, host reads the file; the mod only picks the
name). Sim-inert, deterministic, zero per-frame cost. Needs: a situation taxonomy in Melee terms (the hard,
design-heavy part: what counts as "the same situation"), and the labelling of inputs into options (tech in place vs
roll from a stick direction and the action-state transition that followed; the state transitions are in the replay,
so labelling is from post-frame action ids, which is robust).

**Tier B: a bucketed policy ghost (behavioural cloning).** Host-side `Policy` pad source for a port: each frame, hash a
state signature (own action/frame-bucket, opponent action/frame-bucket, quantised dx/dy, percent bands, grounded /
platform / ledge flags, facing parity) into a table of input-class histograms (stick sector x button set, 8-16 stick
sectors x ~20 button combinations), sample with the lab RNG, emit the pad. Learned from replays by the same tool. The
table is host memory (a 100 k-bucket table with 64 B histograms is 6 MB): nothing in wasm. Behaviour between the
buckets it has seen: fall through to Tier A tables, then to the retail CPU at level N. Quality is bounded: cloning
from a few hundred games gives a ghost that reproduces habits (option selection, timing distributions) and not
spacing precision; that is still the thing players want to lab ("how does this player tech at 60 %"). Per-frame cost:
one hash + one table lookup, microseconds. Deterministic given the seed.

**Tier C: the punish bot = search (section 4) with a ghost opponent model.** Nothing to build here beyond section 4; the
"bot" is "apply the best line found as this port's playback".

Also honest about the ceiling: a real learned policy (an NN over the fighter structs, as in Phillip / Slippi-AI) would be
host-side too, needs a training pipeline the project does not have, and inference per frame per port on the game
thread must fit the sub-millisecond budget; feasible for a small MLP but out of scope for a first pass. The bucketed
ghost and reaction profiles are the 80 % that costs 20 %.

### Cost
Tier A: offline minutes over a corpus; zero in game. Tier B: ~6 MB host table, microseconds per frame. Training needs
`.slp` files of the player (Slippi's own folder is a natural source); the engine is not needed to train Tier A/B
(post-frame data suffices), only to play.

### Determinism / netplay
A ghost is a pad source inside `LabRun` (like the synthesiser): deterministic from (profile, seed, frame). Online lab,
both peers need the same profile: add it to the LabNetplay digest, as mods are. Never in netplay proper.

### Risk: Tier A **low-medium** (taxonomy design); Tier B **medium** (quality may disappoint; must be positioned as
"habits", not "a copy of the player"). Size: Tier A **M**, Tier B **M-L**, Tier C **= section 4**.

## 3. Frame data and timeline from direct engine introspection

### Melee terms
Per frame, per fighter: startup / active / recovery / IASA, intangible (ledge, spot dodge, roll) and invincible frames,
hitlag, hitstun, shieldstun, landing lag (and L-cancelled), jumpsquat, grab release timing, ledge-grab window, shield
HP and shield push; frame advantage on hit / on shield / on whiff for the exchange that just happened; the "can act at
frame N" for both players; for a frozen frame, the full hitbox table of the current subaction (not just the active
ones this frame).

### What exists
- `gd_lab_fighter_v1` publishes action, anim frame, hitlag, hitstun, iasa, intangible, invincible, shield HP, jumps,
  ECB, `cell`, `action_frame`; `gd_lab_meter` per-frame cells; `gd_lab_hitboxes/hurtboxes` with state; events.
- The frame meter and move readout exist, but **observed**: `gd_lab_framedata` is "the observed timeline of the last
  run of an action, not parsed subaction scripts"; `frame_adv` is observed after an exchange once both players are
  actionable again (`lab.rs` ~1016, `b.act_at - a.act_at`), timing out after 300 frames.
- The engine runs the retail subaction event interpreter (it must, to be bit-exact), so the hitbox / intangibility /
  IASA event stream for any (character, subaction) is reachable by **executing the script against a scratch fighter**
  or by parsing the script bytes from the loaded Pl*AJ data, both host-side.

### Mapping
Three upgrades, in order:

**3a. Script-derived frame data (not observed).** Host-side parser of the subaction event stream for (character,
action) from the loaded character data: hitbox on/off frames, IASA, intangibility windows, throw frames, timers,
animation length and the landing-lag attribute. Fills `gd_lab_framedata` for *any* move, not only the last one seen,
and marks the meter's future cells (what *will* happen if nothing interrupts). Caveat: some timing depends on run-time
state (charge, aerial vs grounded, slope, Peach float), so the parse gives the nominal timeline and the observed one
corrects it. Melee's event interpreter is small and its opcodes are documented in the decomp (`ftCmd*`); a dry-run
executor over the opcodes (no fighter) is a few hundred lines. **S-M**, no determinism impact (read-only data).

**3b. Live frame advantage by headless lookahead ("if both do nothing from here").** On a frozen or live frame: save,
step the engine K frames with neutral pads on both ports (or "hold shield" for the defender), read the first
actionable frame of each from `cell`, load back. That is the true on-hit / on-shield advantage for *this* exchange,
including hitlag, shield push, landing lag and platform landings, before it happens. Cost per evaluation: save 0.3 ms +
load 0.4 ms + K ticks; K is bounded by the longest stun plus a recovery (~60 frames) = 12-30 ms, so **one evaluation
per exchange** (triggered by a hit / shield event) fits between frames in paced play (the lab runs headless-fast when
frozen anyway) and is instant when frozen. It must run on the game thread with presentation held (one engine). Online
lab: both peers would compute it, or it stays a local presentation-only readout (preferred: the result changes
nothing in the sim; the save/load is sim-neutral because the load restores every static). **Determinism check**: a
lookahead is save -> N ticks -> load; the ring already proves load restores to hash-equal (`MELEE_SNAP_CHECK`). The
only leak risk is a static written by the lookahead that the snapshot does not cover: by the snapshot rule there are
none, and `MELEE_SNAP_CHECK` after every lookahead is the test. **S**, the primitive is 20 lines in `LabRun`; its
value is large because the observed `frame_adv` arrives late and is often wrong when a player acts early.

**3c. Per-frame flag timeline from the engine, not the mod.** Extend the meter cell with a bit set rather than a
single kind: `ACTIVE_HITBOX | INTANGIBLE | INVINCIBLE | HITLAG | HITSTUN | SHIELDSTUN | LANDING_LAG | IASA | GRAB_RELEASE
| LEDGE_INTANGIBLE | AUTOCANCEL_WINDOW`, plus the hit frame's knockback vector and the DI'd result (the `DI_LINE`
overlay already computes the line; the knockback arc `KB_ARC` is accepted and not drawn). Keep the mod's query count
the same (one `gd_lab_meter` per present). **S**.

### Cost
3a: data parse once per character load, KB of host memory. 3b: ~15-30 ms per exchange, on demand. 3c: free.

### Risk: low (3a: run-time-dependent moves need the observed overlay; 3b: must not run during LabNetplay without being
a cmd, or peers would differ in wall time only, which is fine since presentation is never hashed; run it locally).

### Size: 3a **M**, 3b **S**, 3c **S**.

## 4. "Unlimited compute": search, adaptive trainer, freeze-frame volumes

### 4a. Background search for combos and punishes (BFS / beam / MCTS)

**Melee terms.** From the frozen state, enumerate our input sequences (macro-actions: a move, a jump, a wavedash, a
dash, a grab, a shield, a DI; not raw pads) against the opponent's responses (tech options, DI classes, SDI, no-tech,
jump out, airdodge) and return the lines that maximise damage, KO or ledge position, with the opponent's best
defence (minimax over defensive options) or the ghost's likely one (expectimax). The result is a **tree the user can
browse and play**: pick a line and the lab plays it as a recording on the port, with the opponent's chosen response.

**Engine mapping.** This is the one feature that wants the whole engine budget:
- Branching: with ~30 attacker macro-actions and ~8 defender options per decision point, each node costs save + the
  macro's length in ticks (10-40 frames = 2-20 ms) + load. Depth-3 exhaustive is 30^3 x 8^3 = 14 M nodes: impossible.
  Depth-3 with beam width 20 and 4 defender options: 20 x 30 x 4 x 3 ~ 7 k nodes x ~10 ms = 70 s on one engine. Beam /
  MCTS with a heuristic (damage, opponent in hitstun, position) and macro pruning by `cell` (only act on actionable
  frames: the first actionable frame is a single node, found by one neutral step) is where it becomes usable: a
  **punish search from a committed state** (missed tech, shield-stun, whiff) explores ~500-2000 nodes = 5-20 s
  single-engine, which is acceptable as a "think" the user triggers while frozen, with the best-so-far improving on
  screen (anytime search).
- **Parallelism needs processes**, not threads (one engine per process). A worker pool of N `melee` child processes
  (`melee search-worker`) each boots the scenario once (~0.3 s), then takes jobs over a pipe: "reach frame F by this
  input log (or load keyframe K from your own ring), then evaluate macro M against defence D for T ticks, return the
  fighter structs". Each worker keeps its own ring so a job is a load + resim, not a reboot. 8 workers on an 8-core
  box make the 5-20 s punish search 1-3 s. Memory per worker: the arena reservation is 4.8 GB virtual but committed
  pages are the game's working set plus a small ring; the perf notes do not give the committed figure, and this PC is
  memory-constrained: **measure before choosing N**, and make N a setting with a default of 2.
- Alternative without processes: time-slice on the game thread while frozen (the user is looking at a frozen frame;
  presentation keeps drawing at 60 Hz from the harness-owned side while the engine does 16 ms of search per present).
  Same throughput as one worker, zero new infrastructure, no memory multiplier. **Start here.**
- Macro-actions are the lab's recordings: a macro library per character (jab, dtilt, utilt, grab, nair, bair, dair,
  up-B, shine, wavedash L/R, dash L/R, SH, FH, shield, spot dodge, roll, DI classes) shipped as pad sequences, with
  "wait until actionable" semantics via `cell`.
- Evaluation and opponent model: damage dealt, KO (stock event), opponent on ledge/offstage, time to actionable; the
  defender's options from the dummy synthesiser's reaction tables (tech, DI, SDI, getup, ledge), uniform or from a
  ghost profile (section 2).
- Snapshot cost dominates at shallow depth: each node save + load is 0.7 ms; with page sharing between near-identical
  states it stays there. Ticks dominate at 10+ frames per node.

**Determinism / netplay.** Workers and time-sliced search only ever save / resim / load inside `LabRun`; a found line is
played back as a recording (deterministic). Offline only, or LabNetplay with the search result (the recording) as the
cmd, never the search itself. The search result's correctness relies on the lab's state being exactly the one searched:
freeze first, and invalidate the tree on any sim change (step, load, pad injection).

**Risk: high** (search quality and the UX of a tree are the hard part; the engine part is easy). **Size: L** (M for the
single-engine time-sliced punish finder with a fixed macro library; +M for the worker pool).

### 4b. Adaptive defence / mixup trainer

**Melee terms.** The dummy picks its option (tech, DI, ledge, OoS, recovery, approach) **against your history**: if you
cover tech-in-place 70 % of the time, it rolls more; weights update after every exchange toward what beats you (a
regret-matching / bandit update), or toward your own weaknesses ("you miss the roll-away punish most"). Variants:
"anti-habit" (punish your most common choice), "worst-case" (the option you handle worst), "level ramp" (reaction
delay shrinks as you succeed).

**Engine mapping.** Almost entirely exists: the reaction tables have weights, delays and the lab RNG, and events report
hits, techs and the frame advantage. Add host-side **outcome scoring per exchange** (did the attacker land a hit after
the dummy's option within N frames; was the dummy hit before acting) and a **weight updater** (`OP_ADAPT(kind, mode,
rate)`) that writes the existing weight tables. Mod-side: show the current weights and the success rate per option.
Deterministic: the update is a function of events and the lab RNG, so it is a cmd online. Cost zero.

**Risk: low. Size: S-M.**

### 4c. Freeze-frame scrubbing with hitbox / hurtbox / ECB volumes and launch / DI vectors

**Melee terms.** Frozen, step frame by frame, with every hitbox (active, with damage / angle / KB, and the interpolated
swept volume between frames), hurtbox state (normal / invincible / intangible), ECB with the previous-frame ECB
(the shape the game actually tests), ledge-grab boxes, shield bubble with HP, grab boxes; on a hit frame, the knockback
vector, hitstun length, and the fan of DI'd trajectories (best survival DI, best combo DI, the opponent's actual DI)
drawn as arcs, with the first frame of each arc at which the victim becomes actionable and whether it crosses a blast
zone (KO check at this percent, and the percent at which it would KO).

**What exists.** Freeze and step (also -1), hitbox / hurtbox / ECB / DI_LINE overlays drawn by the host from fighter data
(presentation only, hashes unchanged); `SWEPT`, `SHIELD`, `GRAB`, `BLASTZONE`, `KB_ARC`, `LEDGE_GRAB` bits accepted
but not drawn; hitbox `prev_x/prev_y` already published.

**Mapping.** Draw the accepted-but-undrawn bits (S). The **DI fan and KO check are lookahead** (3b's primitive): on a
hit frame, for each of ~9 DI classes, save, inject the DI stick for the victim, run until actionable or blast zone,
record the trajectory points, load. 9 x (0.7 ms + up to ~80 ticks x 0.3 ms) = ~0.2 s worst case, instant on a frozen
frame; cache per (hit frame, percent). The arc is then exact (the game's own gravity, fall speed, hitstun, SDI-free),
not a formula. "Percent at which this KOs" = bisection over `set_percent` + lookahead (8 steps = ~1.5 s at worst,
usually less; run it in the background slices). **M.**

Also the one thing the user asked for by name in a different form: **hit-frame "why"**: when a hit lands or whiffs by a
hair, the host can report the distance between the closest hitbox and hurtbox (volumes are known), and for a trade,
which hitbox hit first and the priority rule applied. **S**, pure read.

**Determinism / netplay.** Overlays are presentation-only. Lookahead is save / resim / load as in 3b. Offline or local
preview in LabNetplay.

**Risk: low-medium. Size: S (bits) + M (DI fan, KO check).**

## 5. Cross-cutting: what the wasm mod cannot do, and host support needed

| limit | consequence | host support |
|---|---|---|
| 1 MiB memory, 16 KiB stack, no alloc in the lab mod | no replay data, ghost tables, search trees or trajectory buffers in the mod | all of those are host objects referred to by handle, queried in pages (`gd_lab_*_range(from, to, out, max)` pattern already used by `events_range`) |
| 4 M fuel per present call | the mod must not iterate a search tree or a replay; it asks for the top-k lines, the current node's children | host-side ranking; the mod draws a page |
| presentation store is never rolled back | lab UI state survives loads (good); any per-frame cache keyed by match frame must be invalidated on load / rewind (the mod already does this via `gd_lab_status`) | expose a "timeline generation" counter that increments on every fork (load, rewind, set_pos, search result applied) |
| one engine per process | lookahead and search run on the game thread (time-sliced while frozen) or in worker processes | `LabRun::lookahead(pads, K) -> frame structs` primitive; later `melee search-worker` |
| the arena is the game thread's | worker processes, not threads; snapshots do not cross processes | portable state = scenario + input log (+ ring in each worker) |
| lab gate | all of this is `Offline` / `LabNetplay` only; `EOFFLINE` elsewhere | unchanged; new cmds join `lab cmd v1`'s digest version |

New host primitives, in dependency order (each small):

1. `lookahead(port pads as a closure or table, K, stop_cond) -> per-frame fighter structs` inside `LabRun`
   (save, tick K with injected pads, load; `MELEE_SNAP_CHECK` able). Used by 3b, 4a, 4c.
2. Per-port `go_live` and a replay pad feed (1).
3. `OP_LOOP(anchor, frames, cond)` auto-rewind (1, 4b drills).
4. Script-derived frame data (3a).
5. Macro library and outcome scoring (4a, 4b).
6. Ghost profiles: offline tool + `OP_PROFILE` (2A), policy pad source (2B).
7. Worker pool (4a scale-up).

## 6. Ranked shortlist: the best first features

1. **Headless lookahead primitive + true frame advantage + DI fan / KO check (3b, 3c, 4c).** Size S+S+M, risk low.
   One primitive (save, tick K, load: 0.7 ms + K x 0.2-0.5 ms) unlocks the three readouts players ask for most and
   that no other Melee trainer has exactly (UnclePunch computes KB from formulas; we simulate the real frames). It is
   also the foundation for every "unlimited compute" feature, so it gets built first regardless. Testable with
   `MELEE_SNAP_CHECK` and the existing lab-check.
2. **Replay takeover on `.mrpl` and `.slp` with auto-rewind (1).** Size M, risk medium. The lab already has scrub,
   step, take over, slots and a logged future; the missing pieces are a replay pad feed, per-port go-live, the loop
   command and booting a `.slp` scenario in the windowed lab (slpcheck has the converter). This is the feature with
   the most player value per line: "lab the situation from my last set". The 38/125 frame-exact figure and the
   divergence marker keep expectations honest.
3. **Adaptive mixup trainer (4b).** Size S-M, risk low. The weighted reaction tables, delays, lab RNG and events exist;
   add outcome scoring and a weight update rule. Deterministic, works online in LabNetplay as a cmd, and turns the
   existing dummy into a real training partner for tech chases, ledge coverage and OoS.
4. **Script-derived frame data (3a).** Size M, risk low. Replaces the observed-only `gd_lab_framedata` with the nominal
   timeline for any move, enabling a move library and future-cell prediction on the meter. Pure data, no sim impact.
5. **Replay-derived reaction profiles (2, Tier A).** Size M, risk low-medium. The "ghost" that costs little: an offline
   tool over `.slp` post-frame data fills the dummy's option tables with a real player's habits. Prefer it over the
   bucketed policy ghost until the taxonomy is proven on real files.

Deferred: punish/combo search (4a) until 1 exists and a macro library is in place; the bucketed policy ghost (2B) until
Tier A shows the buckets are right; the worker pool until single-engine time-slicing is measured as too slow on real
searches.

## 7. Open design questions for the user

1. **Lookahead while playing, or only when frozen?** Running 15-30 ms of lookahead between live frames is possible in
   paced play (the engine idles ~16 ms per frame) but shows as a frame-time spike on slow machines. Frozen-only is
   safe; live needs a budget setting.
2. **Replay takeover: when the recorded opponent stops making sense, what takes over by default?** Recorded inputs
   until next actionable frame, then CPU level 9? the lab's dummy settings? a ghost profile? Or freeze and ask.
3. **Which replays matter first: our `.mrpl` (always exact, seekable today) or Slippi `.slp` (the real tournament set,
   38/125 exact)?** This decides whether the first milestone is "wire the replay feed" or "boot `.slp` in the windowed
   lab".
4. **How honest should the divergence marker be?** Show the console-exact frame always, only when known, or hide it
   and treat every `.slp` as "our sim's rendition"?
5. **Search UX: anytime results that improve on screen, or a fixed "think for N seconds" with a progress toast?** And
   is a browsable tree desired, or only "top 3 lines, press to play"?
6. **Macro library: shipped per character by us (curated pad sequences), recorded by the user into slots, or both?**
   Shipped macros make search work out of the box; user recordings make it personal.
7. **Worker processes: acceptable on this memory-constrained PC?** Default N = 1 (time-sliced) with an opt-in pool,
   or no pool at all until a measurement says the single engine is too slow?
8. **Ghost taxonomy: which situations count?** A first list: tech (percent band x position), DI (move x percent),
   ledge option (opponent distance x state), OoS (move on shield), recovery (height x distance), neutral approach
   (spacing band). Which ones matter enough to design first?
9. **Online lab for the new features: local-only readouts (lookahead, DI fan), with only the applied result (a
   playback, a weight change) sent as a cmd?** That is the determinism-safe default; is a shared search view wanted?
10. **Auto-rewind conditions: which events are worth a condition?** took damage, lost stock, grabbed ledge, became
    actionable, entered action X, left a zone, N frames passed.
