# Mission folders

Offline standalone runtime; the editor remains unchanged. Enable this script mod,
start an offline match with P1 on FD, then `mission play first`.
Native round 3 verified retries, completion, borders, labels and parked CPUs.
Round 4 verified fix3 gameplay, camera following, fixed blast edges, reserves, C-stick
and restoration. Fix4 aspect-edge correction and camera cuts still need native acceptance. Engine batch-2 camera APIs are required.

The sample's `missions/first/models/` stays empty: the Blender exporter fills it
with `bf_floor_4m.gxmesh`, its collision sidecar and atlas. Until then `first` is
deliberately refused as an unknown folder part. Offline tests stub model loading.

## Folder contract

```text
missions/<name>/
  level.lua
  mission.lua             optional only when level.lua embeds mission
  models/                 local *.gxmesh, *.coll.json, textures
  chunks/<id>/            optional, described by root level.chunks
    level.lua
    models/
```

Files return plain Lua tables, evaluated with an empty environment. Folder, chunk,
part and marker names use letters, digits, underscores and hyphens, at most 64
bytes. No cross-folder paths. The catalogue is the folder's `models/*.gxmesh`
listing, with the extension removed. The engine resolves sidecars and textures.
If both mission forms are present, both are validated and `mission.lua` wins.

`level.lua` keeps the editor schema: `version=1|2`, `units=6.5`, a dense `parts`
list (max 128 per root/chunk). Each part has `part,x,y,z,rot,collision,floor_flags`
and optional `scale,scale_x,scale_y,scale_z`. `name` (then `label`) supplies
the model instance label, with `part` as fallback; printable labels are 1-80 bytes
and survive collision replacements. Rotation is -360..360; scale magnitude
is .001..100; floor flags are integer 0..3. V2 adds `camera` and `blast` rectangles
`{left,right,top,bottom}`, and `spawn={[slot]={x,y}}` for slots 0..7 or 127..146.
Root bounds/spawns govern the level; a chunk's descriptor supplies its respawn.

Optional `lines` is a dense list (max 768) of
`{x1,y1,x2,y2,kind,passthrough?,ledges?,draw?}`. Kinds and directions follow the
engine: floor left-to-right, ceiling right-to-left, right_wall downward,
left_wall upward. Model sidecar collision is enabled by each part's `collision`.
The native pools are shared; accepting a file is not a guarantee of free capacity.
Keep all transformed geometry inside +/-50,000; the editor's numeric validation
allows +/-100,000, but the game's projection assertion does not.

`mission.lua` returns exactly the existing pure machine's schema: `start`,
`enemies`, `checkpoints`, `goal`, `waves`, `triggers`, `objective`. See
`../map_editor/scripts/mission.lua` for its unchanged strict rules and caps.
The copied `scripts/mission.lua` is byte-identical; no mission logic is forked.
Lives count P1's falls; respawn slot is 4. Missing P1 during a KO retains the last
fall count and chunk window. Completion/failure cleans enemies but leaves geometry
available for a tour, restart or stop.

### Chunks

```lua
chunks={
  {id='room0',rect={left=0,right=130,bottom=0,top=104},spawn={x=20,y=10}},
  {id='room1',rect={left=130,right=260,bottom=0,top=104},spawn={x=150,y=10}},
}
```

Default chunks are 130x104 game units (20x16 whole metres at 6.5 units/metre).
Rectangles must be nonoverlapping, equal-sized, and on the grid defined by the
first rectangle. Containment is half-open (left/bottom included). This makes the
neighbourhood exactly the current cell and its eight neighbours; gaps are allowed.
Each child level uses **absolute level-space coordinates**, not chunk-local
offsets. Its catalogue inherits root models, which take precedence over same-name
chunk copies; unique chunk-local models remain available. Root-only model exports
may omit child model directories. Root mesh/sidecar/atlas paths let the engine
share one colour/glow atlas across chunks and unchanged reloads. An atlas alone
in root `models/` cannot override a chunk-only mesh's local atlas resolution with
the current engine API; export shared meshes to root too, or add engine atlas
fallback support. Root parts are always loaded. Every child
file is validated upfront, including distant unloaded chunks. Child mission tables
are refused. A descriptor's spawn must lie inside its rectangle. Entering a cell
sets slot 4 to its spawn; leaving the grid during a fall retains the last point.
Chunk respawn takes precedence over mission checkpoint respawn in streamed levels.
Chunk changes use an inclusive two-unit border dead band in both directions and log actual fighter coordinates.
Respawns are written only when they change or setup restores stage overrides.
Fly commands preload the destination alongside the player's actual 3x3 window
(up to 18 cells while travelling), retiring the extra window after arrival. A
distant preload does not select the player's current respawn chunk.

### Fighters

Root `level.lua` sets participating fighter ports with `fighters={[2]='fight'}`
(ports 2-6, CPUs only). Default `stand` means unused reserve: the runtime calls
`fighter_bench` so it is invisible, intangible, frozen and camera-excluded.
Owned reserves remain benched across reloads. A port explicitly changed to `fight`
is called at the current chunk spawn/checkpoint/start and gets fight AI; an
already-active fight port keeps its position. Humans and other scripts' reserves
are untouched. Pure mission enemies remain Adventure enemy actors, not fighter ports.

When bench is refused or unavailable, stand AI and spawn parking remain the
fallback; bench is retried every 30 logic frames and fallback placement every six.
The fallback remains visible/hittable and in the native camera subject list until
bench succeeds. Readiness refusals are quiet; successful benching logs once. Stop calls owned reserves at their saved
pre-mission positions, restoring the engine's saved flags/controller. A refused
release is logged; engine unload/scene cleanup also releases reserves. Fallback
stand ports return to fight (their original AI selection cannot be queried).

### Camera

V2 `camera` accepts a rectangle `{left,right,top,bottom}`, settings, or both in
one table. Rectangle keys describe outer level ends, never discard mode/settings.
Root settings merge with child settings; inherited validation includes distant
chunks. Explicit `ends`, the root rectangle or the chunk union supplies the outer
clamp. Without any of these, follow is unbounded.

```lua
camera={left=0,right=2600,bottom=0,top=312,mode='follow',
        window={w=250,h=180},leash=10,look_ahead=18,velocity_lead=6,
        track_smooth=1.8}
-- A chunk's level.lua can override root settings:
camera={mode='chunk',margin=4,frames=45,door_margin=24,safe_margin=12,fov=25}
-- Vertical sections:
camera={mode='shaft',x=65,window={w=150,h=260},vertical_leash=4,
        vertical_lead=6,vertical_max=48,fov=20}
```

Measured relation: `visible_height=2*distance*tan(fov/2)` and
`visible_width=visible_height*aspect`, viewport aspect read live: `gd.project` samples the camera-plane right/up basis
and `gd.safe_area()` normalizes screen coordinates. If projection is unavailable,
the live safe-area width/height ratio is used. An authored `aspect` no longer
overrides the actual view; resize/aspect changes recalculate framing.
Follow fits its desired 250x180 window at FOV 25: distance about 317.16,
actual footprint 250x140.625. Chunk mode fits height: default 130x104 room plus
four units per side gives distance 252.60, actual view 199.11x112. Neighbouring
rooms may show. Shaft fits a taller 260-unit height at FOV 20: distance 737.27,
view 462.22x260. These replace the old double-angle distance estimates.

Every mode clamps the real footprint to the level's outer edges. The native retail
camera uses its own fixed descriptor aspect, which can differ from the widened
projection. Native target bounds are therefore a 1x1 rectangle centred on the
stage origin, making its clamp centre the target irrespective of that descriptor.
The original level rectangle still supplies the outer ends. Before rendering,
three `gd.project` samples plus their depths invert the exact world-z=0 projection
at all four screen corners. If the visible rectangle crosses an outer edge, a
camera pose correction shifts eye and interest together back inside it. For an
outer level narrower/shorter than the view, centre the view; symmetric spill is
unavoidable at that zoom and is explicitly allowed instead of changing authored
zoom. The previous narrow-shaft distance reduction is superseded.

Default yaw/pitch gains and pan are zero, min_dist=max_depth, fixed_zoom 1,
track_smooth 1.8 and track_ratio 1. Standard camera following remains active.
A distant respawn/teleport uses `camera_set` to cut eye and interest on that same
frame. The cut stays detached until one completed native camera update consumes
the committed pose, then `camera_attach(0)` returns to the nearby normal pose.
Attaching in the same callback would restore the stale normal pose and undo the
cut. Native `camera_move` refuses zero frames, and track_smooth's allowed maximum
cannot make the native interest lerp instant. Boundary corrections use the same
one-update cut/release mechanism. Stop also releases any temporary camera owner.

Follow leads facing/motion by `facing*18 + vx*6` (movement direction wins over
facing), capped at 40 units, tracking that target with a 10-unit leash. For Mario
at vx=1.4 the steady origin should be roughly 16.4-36.4 ahead (player behind
centre), depending on the prior origin. Follow vertical lead is vy*3, capped at 24.
Shaft X is fixed, Y follows `player_y + vy*6` (cap 48) with a four-unit leash.
Look-ahead/leashes are constrained by the actual footprint to retain fast vertical
motion. A narrow level centres the view instead of reducing zoom distance. At outer ends the
no-void clamp takes priority over lead. Authored X in shaft mode must cover the route.

Chunk origin interpolates in Lua over 45 logic frames (authored 30-60), starting
when within 24 units of a border and moving toward its neighbour. A twelve-unit
interior safety margin biases the origin toward a player who would leave the
view during the tween. A missing/dead/respawning player snaps to the appropriate
respawn location; rebirth states use their actual position. Teleports farther than
one visible window also snap. All native origin calls have `frames=0`.

Every origin update writes the matching origin-relative blast rectangle in the
same Lua call, so authored world-space KO edges remain fixed. An update refusal
restores the old origin/edges. This necessarily uses the existing native blast
setter (which still logs/branches the rewind timeline); stationary cameras avoid
writes. No independent world-space blast API exists in this engine. Default host
blast edges are also preserved if the file omits `blast`.

Additional camera fields: `aspect` (legacy 0.5-4 value; live projection takes precedence), `leash`/`vertical_leash` (0-100),
`look_ahead` (0-100), `velocity_lead`/`vertical_lead` (0-30 frames),
`max_lead`/`vertical_max` (0-200), `door_margin`/`safe_margin` (0-100), along with
engine fields, `window`, `ends`, `x`, `margin` and `frames`. Zero is accepted.
No C-stick or pad calls are made. Temporary camera_set cuts are released after
one native update. Use LAB/VS/Training/Target Test
outside Classic/Adventure/All-Star to retain C-stick attacks. Stop/unload restores
camera parameters, origin, bounds and spawn overrides. Native framing and controller
acceptance remain the tester's job.

### Markers and progress

Default names are `start`, `enemy1`..N, `checkpoint1`..N, `trigger1`..N, `goal`.
Their route order is start first, then X in the direction from start to goal,
then Y and name. From-marker play clears waves whose enemies all precede the
marker, selects the most recent checkpoint (including the selected checkpoint),
and suppresses earlier once-trigger entries.

For vertical or nonlinear routes, provide explicit progress in root `level.lua`:

```lua
markers={{name='upper_room',x=20,y=200,
          cleared_waves={1,2},checkpoint=1,frames=600}}
```

These add named fly targets (or override a default name). `cleared_waves` explicitly
defines completed wave ids; `checkpoint` is an existing checkpoint index;
`frames` is elapsed logic frames (default 0). No changes to the pure machine's
schema are needed. Fly commands move only the fixture; they do not advance mission
progress as play-from does.

## Commands

| Command | Effect |
|---|---|
| `mission list` | Log folder names under `missions/` |
| `mission play <name>` | Validate/load folder, start mission at authored start |
| `mission play <name> from <marker>` | Load with the marker's earlier waves cleared and checkpoint selected |
| `mission reload` | Validate/stage replacement, retain fighter position, resume from last checkpoint if present |
| `mission restart` | Start again using the last play-from marker, or authored start |
| `mission stop` | Remove owned content/enemies, disarm flight, restore host stage/bounds/spawns |
| `mission fly next` / `mission fly prev` | Cycle marker targets |
| `mission fly <marker>` | Cancel clear/tour pursuit and fly to named marker, preloading its neighbourhood |
| `mission clear` | Pulse native fly hitbox (30 damage/radius), pursue the captured targets with a 1500-frame budget each, then release cursor |
| `mission drop` | `gd.fly(1,'place')`, disarm attack, cancel clear/tour, return to normal control |
| `mission tour` | Visit all markers and chunk spawns in order, one result line per destination |

For compatibility with older fly_target ranges, targets beyond +/-10,000 use `fly(true)` +
`teleport` as a logged fixture, only within the native +/-50,000 bound. This is
setup, never proof that a route can be traversed.

Play/reload/restart/fly/drop wait for a controllable P1 and retry every six logic
frames, for at most 600 frames (about ten seconds at 60 Hz). The old mission stays
active while waiting. A new command replaces a pending request; stop/match end
cancels it. Waiting and timeout get one clear log line each. Clear also waits and
re-arms after death, with a 600-frame bound on readiness/target refusal. Permanent
ownership errors are refused rather than retried. Tours retry transient refusals
within their existing 600-frame arrival bound.

Clear pursuit avoids duplicate fixed targets and retargets moving enemies at most
once per six frames. Temporary targeting refusals are retried; the fly hitbox
pulses 28 logic frames on / 12 off so hitlag and enemy death logic can finish.
After 600 target frames, pursuit alternates eight units left/right of the enemy.
At 1500 frames an undefeated target fails the entire command with its kind, state
and vulnerability in the log. The whole command also has a guarded `gd.deadline`
of 601 + 1501 times the captured target count (plus one diagnostic frame).
Completion, refusal, timeout, replacement, reload and stop close the deadline
and call `gd.fly_clear(1)` to release attack, targeting and script cursor ownership;
older APIs fall back to attack-off and fly-off. A completed tour also releases it.
`teleport` has no success return;
`fly('place')` returns false because flight has ended. Neither is asserted.

A tour waits for arrival within
1 world unit, or reports a refusal/600-frame timeout. Logs include loaded chunk
ids, seconds, and refusal reason. Normal controller input remains the way to judge
jumps, fairness, camera feel and ceiling/wall behavior.

## Reload and lifecycle

Both root file stamps are checked every 15 completed logic frames (not host ticks;
paused game logic does not poll). `mission reload` uses the same transaction.
Read errors, malformed tables, unstable root stamps, asset/instance refusals and
initial enemy refusals retain the running document and content. Candidate areas
are built before old areas are removed, so staging needs temporary spare capacity;
an at-capacity scene may refuse an otherwise valid replacement safely.

A successful reload keeps position, match and fighter, reuses engine asset handles
when the engine cache does, and restarts mission progression from the last
checkpoint. It does not patch ongoing enemies, elapsed time or death counts.
Changed models/chunks alone do not trigger the watcher: the exporter must write
models/chunks first and update a root file last. Stamps are checked before and after
reading to reject a changing export; this is not a cross-file filesystem transaction.
Content refresh/asset eviction depend on the engine packet.

Match end, unload, stop and loadstate clean the runtime. Savestates/rewind/netplay
are unsupported. Collision triggers replace nearby loaded parts in temporary
areas; those replacements are cleaned with their owning geometry.

## Development / offline verification

`main.lua` is generated, with module sources escaped as text; the engine's sandbox
`load` compiles them in this script's environment. Original module names/lines are
preserved for errors. There is no `require`. Every source module and the generated
entry have fewer than 400 physical lines and few top-level locals.

```text
python tools/port/missions_bundle.py
python tools/port/missions_bundle.py --check
lua melee/pc/tests/missions_runtime.lua
lua melee/pc/tests/missions_validation.lua
python -m unittest discover -s tools/port -p "test_missions.py"
```

Run `luac -p` on each Lua file separately. These tests use a stub engine; no
native visuals, combat, timing, streaming capacity or traversal are proven.
# Generated maze extension (2026-10-03)

Use `mission maze <seed> [size]` (size8..20, default12), `mission maze reroll`,
and `mission maze map`. Generation logs an ASCII map, installs through the normal
transactional loader and supports the existing fly/drop/tour commands. Reroll
advances the seed and retains size. Dead-end caches heal25 percent once per run.
Chunk recipes are data under `maze-chunks/`, using the fix3 default130x104.
The workspace's `tools/maze/generate.py --prepare-mod --kit <room-kit>` prepares
the entry, root kit assets and a portable generated mission. Runtime generations
use a read-only data overlay and last only for the current script session; offline
Python output is a real folder. Generator/checker source has no engine calls.

Camera cuts are triggered by load, respawn entry/fall count or a large player-position
jump, rather than distance from a clamped origin. Projection aspect changes below
0.0001 and world-coordinate differences below 0.01 units are treated as rounding
noise. Stationary boundary correction is not repeated on detach/attach.

Load reserves every CPU before changing the world (unsafe bench attempts refuse the
load before staging), holds P1 through host isolation, verifies the placement floor,
and restores P1's pre-load damage.
An optional root `level.lua` field `starting_percent` (integer 0-999) overrides that
snapshot, without changing the shared pure mission schema. Explicit `fight` CPUs
are called into the level only after staging succeeds.

`mission play`, `reload` and `restart` queue a frame-driven installation. CPU ports
are stood/benched at the first match pre-frame/start callback, before a play command.
P1 is reserved during validation and staging. Each frame retains one asset or
builds one parked, hidden area; the 3x3 chunk window is ordered nearest first.
Disk reads finish before the commit callback applies bounds, host hide, floor validation, placement,
visibility, mission start and camera changes together. The old mission stays
selected until commit. Staging failures roll back under fresh frame budgets;
stop cancels staging. Conflicting commands are refused until completion.

P1 remains reserved through all scene mutations. Before the single final call,
`floor_below(x,y+0.5,200)` must find live collision under the placement point;
otherwise commit refuses and rolls back. Entry and landing states wait before
staging; native reserve-readiness codes also gate the first staging step.
Commit and rollback call P1 with 60 intangible frames. Camera pose is held during
staging and released at commit/rollback; the fix5 event and tolerance guards remain.
Old parked areas and their asset references retire one per frame after commit.
This protects the script budget from aggregate disk/area work; a single native
asset read/build remains synchronous and still needs cold-cache native timing.

Install progress logs `mission: staging step <name> completed after N frames
(total M)` once per completed phase, asset/chunk, or rollback operation. A step
that cannot advance ends after 120 frames with `mission: refused staging step
<name>: <condition> not met after N frames`. Unsafe reserve readiness is read from
`fighter_benched` entity refusal codes without rewriting bounds/camera/hide while
waiting. A timed-out rollback keeps a safe recovery record; `mission stop`
explicitly retries cleanup. Other commands are refused until recovery completes.

`set_damage` returns no Lua values. Damage restoration compares the live percent,
calls it only when needed, and verifies synchronous readback. Rollback cursor
progress and tolerant native-state comparisons prevent repeated restoration writes.

Enemy escape policy: an enemy observed receiving a hit from P1
(`received > 0`, `last_attacker == 1`) earns defeat credit if it later leaves
the blast zone, even if a CPU hits it afterwards. This uses the normal defeat
event path; it does not force a native enemy death or synthesize item drops.
An untouched escape (including CPU-only damage) remains a lost enemy, with no
defeat credit. Interior removals retain the existing defeat classification.
Only hits observed before disappearance can be credited.

Load bookkeeping starts at completed handover: P1 stays reserved while geometry
and host isolation change, and its final call seeds collision against the mission
floor. The live fall count after that call becomes the mission baseline. Falls
before mission ownership are excluded; every later fall still counts normally.
This avoids attributing a stale pre-staging snapshot delta to a fresh mission.
The Lua checks do not establish that the native smoke-test KO is eliminated.

On settlement/stop, P1 enters airborne flight and immediately drops into the
normal Fall state while the old mission floor still exists, before collision is
retired. Unsafe transitions retain the world and retry every six frames for up
to 120 frames; then cleanup stops retrying and `mission stop` can retry explicitly.
Envoy keeps cleanup unpaused, blocks a scene launch until cleanup completes, and
clears old mission ownership on match end even during a requested relaunch.

## Room and doorway zones (fix9)

Room camera ownership uses P1's native `cur_pos` x/y, not feet or a predicted
position. Velocity and facing cannot choose another room. Membership is sampled
once per logic frame, published as `current.membership`, and passed to streaming,
respawn selection, camera, trigger observations and diagnostics. Contact traces
should read that published result rather than calculate another room. Lua AABB
membership is authoritative today; these volumes add no collision or physics.

Inside a transition zone, keep the committed room. Leaving it into exactly one
room zone commits that room. Returning to the source room costs no transition.
An ease finishes its 24-frame ownership lock before another room can commit;
if P1 is still genuinely in the return room afterwards, one return ease begins.
Returning into the doorway cancels a pending return. Upper-room entry while P1
is airborne waits for landing, so a jump excursion that falls back has no room
transition. Drops commit after leaving the lower edge of the vertical zone.

Defaults: horizontal commit distance 8 units per side, vertical 12, ease duration
24 logic frames, `smoothstep` curve, doorway view margin 12 units. The owner
should feel a body-width dead band, followed by one short, decisive slide. A
stationary doorway keeps its framing; turning cannot pull it toward another room.
Room ownership never anticipates. To keep P1 visible, doorway framing can shift
between the joined room centres only as far as the safe view margin requires.
During an ease, visibility takes priority over reaching the exact room centre.

Outside all zones, retain committed streaming/respawn ownership and use a bounded
follow camera with level-wide camera ends. Its origin moves at most 12 units per
axis per frame; it never makes an outside-zone cut. Re-entry commits normally.
A far debug teleport or KO can temporarily outrun that follow fallback: keeping
an arbitrarily teleported player instantly visible would require the forbidden
snap. Crossing/doorway fixtures retain the on-screen guarantee. One exit and one
re-entry log name the membership/committed room. `mission zones` lists volumes and
membership; the HUD adds room, transition and committed-room diagnostics.

```lua
camera={mode='chunk',commit_distance=8,vertical_commit_distance=12,
        ease_frames=24,curve='smoothstep',doorway_margin=12,
        doors={door_A_B={commit_distance=10,ease_frames=30,curve='linear'}}},
zones={
  {name='room_A',kind='room',room='A',
   rect={left=0,right=130,bottom=0,top=104}},
  {name='room_B',kind='room',room='B',
   rect={left=130,right=260,bottom=0,top=104}},
  {name='door_A_B',kind='transition',rooms={'A','B'},
   rect={left=120,right=140,bottom=0,top=32},
   camera={ease_frames=30,curve='linear',doorway_margin=10}},
}
```

Zones can live in root or chunk level files; coordinates are world game units,
and room IDs are chunk IDs. Room volumes cannot overlap each other (touching
edges are fine); transition volumes may overlap rooms and may join three or more
rooms. Transition membership has priority, with name-sorted deterministic priority
when multiple transitions overlap. A zone may span a T-junction. Its authored
extent defines hysteresis; commit-distance fields size *derived* zones rather
than secretly changing an authored volume. Root camera defaults, destination
chunk settings, per-door overrides and transition camera settings apply in that
order. `curve` accepts `linear` or `smoothstep`; `ease_frames` is 1..120. Legacy
`frames` and `door_margin` remain readable but no longer choose room ownership
or set the new room ease; use the new fields when tuning room feel.

Absent authored rooms, derive them from chunk rectangles. Absent authored doors,
derive connections from shared borders with the configured dead band. Maze
metadata restricts these to open exits, using horizontal slots y=0..32 and vertical
slots x=53..77 in each 130x104 cell. Sealed exits get no transition volume. Plain
legacy grids without exit metadata use the complete shared edge. If *any* authored
transition exists, it is the authoritative connection list: omitted connections
are not guessed. Missing areas then use the outside-zone fallback.

Exporter follow-up (not edited here): non-physical box markers should carry
`gd_zone=room|transition`, a unique `gd_zone_name`, `gd_room=<chunk id>` or
`gd_rooms=<comma-separated chunk ids>`, and optional camera tuning properties.
Export their transformed world AABBs as the `zones` data above; retain marker
names so validation can name both objects on an overlap. Apply the normal kit
6.5 scale conversion, and never generate a collider for a zone marker.

Maze-generator follow-up (not edited here): emit one named room zone per cell
and one transition zone per unsealed exit pair, joined by room IDs, with slot
extents and configurable dead-band depth. Deduplicate paired exits; generate an
explicit shared transition zone for authored T-junctions. Runtime derivation
already provides compatibility while those producers catch up.

Future native integration is isolated in `zones.source`: a guarded `zones_at(1)`
call is used only when `zones.native_source` supplies a registered-ID/name mapping.
That adapter must register these non-physical volumes with the final native zone
API and return the corresponding authored zone records. Unknown native handles
or an unregistered empty list never silently replace Lua membership. Engine
enter/exit events must update the same frame result, not run another ownership
state machine. The engine API was unavailable in this packet, so no guessed
`zone_add` signature or event payload is wired into gameplay.

Stable ownership does not issue spawn or unchanged camera parameter/bounds
setters. Stationary doorway/interior frames issue zero setters after settling.
Origin and its paired blast compensation necessarily update while an ease or a
visibility adjustment actually moves the framing; their world blast edges stay
fixed. Suppressing those writes on every unchanged-room frame would prevent
multi-frame easing, so the no-fighting rule is applied to ownership and unchanged
values rather than preventing required interpolation writes.

Fresh play from a marker inside a doorway has no crossing history: bootstrap from
the authored start room if that room is joined by the door, otherwise the first
room declared by the door. Reload preserves the existing committed room by ID.
An initial marker outside all authored volumes chooses no guessed room.
