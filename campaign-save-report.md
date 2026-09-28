# campaign-save

Code-only batch contribution. No build, game launch, or commit was performed.

## Changes

- `pc/scripts/lib/campaign_save.lua`: bundleable `CampaignSave` library and
  `codex/gamemode-kit2` adapter. No `require`, filesystem escape, or executable save data.
- `pc/platform/gw_script_campaign.inc`: isolated native storage implementation.
  `gw_script.c` only includes it after data storage and registers one API entry.
  `script_game.c`, snapshots, rollback, the build list, and the sibling branch are untouched.
- `pc/tests/campaign-save.lua`: ready-to-launch, generated, self-checking LAB test.
  Sources: `campaign-save-body.lua` and the library; regenerate with
  `python pc/tests/bundle_campaign_save.py` (text concatenation, not a build).
- `pc/tests/campaign-save-unit.lua`: host Lua checks using in-memory storage and mocked
  scenes, with optional checks against the actual sibling framework implementation.

## Storage and schema

`gd.campaign_storage()` checks offline/gameplay access. `(name)` reads from the caller's
existing `scripts-data/<script-id>/` namespace. `(name, bytes)` returns true after a checked
`WriteFile`, `FlushFileBuffers`, close and same-directory `MoveFileExA` replacement;
errors raise without advancing Lua's committed state. It retains the existing data API's
name restrictions and 1 MiB limit. No simulation memory is changed and no rewind branch
is created. Netplay and rollback reject reads and writes, including rollback-safe scripts.

`CampaignSave.new{id=..., version=1, slot='campaign', defaults={area='forest', ...}}`
uses `campaign.1.gdc` and `campaign.2.gdc`. The two slots alternate monotonically numbered
generations. The inactive slot is written through a `.tmp` file, so the previous valid slot
remains available. Loading checks both records and chooses the newest valid generation.
Interrupted/truncated/invalid records fall back to the other slot, then caller defaults.
Orphan `.tmp` files are ignored. No automatic repair writes occur on load.

The `GDCP1` envelope contains a checksum (FNV-1a, corruption detection, not authentication),
mode id, mode-definition version, generation, schema and payload. The bounded, length-prefixed
codec never evaluates Lua; it rejects cycles, duplicate keys, excessive nesting, oversized
payloads and trailing bytes. Payload limit is 64 KiB including envelope fields. Integer,
string, boolean and table values are supported; floats/functions/userdata are not.

Schema **2** carries `area`, `entry`, `checkpoint`, `party`, `unlocks`, `collectibles`,
`progress`, and `cleared`. `area` identifies the current room; resume is at `entry`, not a
mid-frame position. Party is a dense list of up to 16 stable character IDs; unlocks map IDs
to booleans; collectibles map IDs to nonnegative integer counts. `progress` and `cleared`
retain framework-owned authored state. Checkpoint contains a copy of the durable fields.
Schema **1** migrates `room` to `area` (also in its checkpoint), filling optional defaults.
Migration is in memory; the next explicit commit writes schema 2. Unknown schemas or different
mode-definition versions block commits, preserving those records rather than downgrading them.

One writer per script/slot is required. Different scripts have separate namespaces; keep the
owning script ID stable across upgrades. This is local persistence, not cross-process locking
or cloud synchronization. Atomic replacement and a flushed file do not guarantee survival
of every possible filesystem/device power-loss failure; the second valid slot is the fallback.

## Commit contract

Call `save:scene()` from the owning scene-boundary hook before consuming progress; it returns
a copied state and `primary`, `fallback`, or `defaults`. Use `save:get()` for an independent
copy while playing. Call `save:commit(reason, data)` explicitly:

| Reason | Durable result |
| --- | --- |
| `checkpoint` | Commit all progress and replace the checkpoint with that complete durable state. |
| `death` | Ignore pending data; commit the last **disk-committed checkpoint**, discarding later party/progress/unlock/collectible changes. |
| `room_exit` | Commit the supplied destination room/entry and all earned progress; preserve the committed checkpoint. Call after a successful transition, or supply the validated destination before launching the new scene. |
| `quit` | Commit current room/entry and all earned progress; preserve the checkpoint. Call before `gd.quit()` or an authored leave action. |

Encounter handles, health, stocks, timers, waves, camera state and boss ownership are never
saved by the adapter. Resume reconstructs a fresh encounter. Abrupt termination keeps the last
successful commit; OS window-close/crash is not an implicit quit commit. `on_unload` may be
used for an explicit owner policy, but unloading is not assumed to mean quitting a campaign.
Do not call this library from `on_loadstate`, rewind, or resimulation hooks. A savestate load
does not load, undo, or automatically commit campaign files.

## Game-mode integration

Read `../codex-gamemode/pc/scripts/lib/gamemode.lua` and `gamemode.md` without editing them.
Bundle that library and `campaign_save.lua` before the owner's entry, as the framework does
for its own demos. There is no runtime library loader or shared global between scripts.

```lua
local mode = Gamemode.new(def)
local save = CampaignSave.new{
    id=def.id, version=def.version, slot='campaign',
    defaults={area=def.start, entry='start', party={'fox'}}
}
local campaign = CampaignSave.adapter(mode, save)
function on_scene() campaign:scene() end
function on_match_start()
    local ok, why = campaign:start()
    if not ok then gd.log('campaign start failed: ' .. tostring(why)) end
end
function on_match_end() mode:stop(true) end
-- Keep the framework's normal frame/draw/enemy/target/boss forwarding.
-- After an authored checkpoint has been accepted:
-- campaign:commit('checkpoint', current_party)
-- After a successful room transition:
-- campaign:commit('room_exit', current_party)
-- Before an explicit leave/quit:
-- campaign:commit('quit', current_party); gd.quit()
```

`adapter:scene()` validates the saved area/entry and returns the resume DTO. `adapter:start()`
requires no active mode, claims an empty native mode blob, seeds saved progress, constructs
only the saved area's fresh encounter, and restores the campaign checkpoint. It intentionally
uses the framework **v1 construction seam** (`_safe`, `_save`, `_enter`) and its schema-1
state shape; update this adapter if that private contract changes. It does not use `Mode:load`,
which restores live snapshot handles. Failures use the framework's ordinary error cleanup.
The adapter reads `state.progress.unlocks` and `.collectibles`; optional party arguments replace
the saved party. The owner applies the returned party to its scene roster as needed.

For death, call `campaign:commit('death')`, then restart/load at the next scene boundary and
call `campaign:start()` once the fighter is live. This restores the durable checkpoint; it
does not resurrect a dead fighter or assume that the framework's most recent transient
checkpoint was committed. Scene routing and death detection remain the owning mode's policy.

## Integration-lane test launch

From the workspace root in Git Bash, after that lane's single combined build, with its own
`GW_MELEE`, `GW_BUILD_ROOT` and `MELEE_ISO`/disc configuration already set:

```bash
MELEE_SCENE='mode=lab;p1=fox;p2=falco/cpu;stage=fd' \
MELEE_PAD_SCRIPT="$GW_MELEE/pc/tests/campaign-save.lua" \
MELEE_PAD_IGNORE_ADAPTER=1 MELEE_SCRIPTS=0 MELEE_CARD=0 \
bash tools/port/run.sh campaign-save --iso "$MELEE_ISO"
```

Use an absolute Windows-style `GW_MELEE` as the workspace tools expect. The ready-to-run test
is under `pc/tests/`; it needs no additional Lua module loading. It claims neutral input and
pad-scripts the memory-card prompt if present. It saves in Final Destination LAB, launches
Battlefield LAB, observes a new scene epoch and changed stage, loads at that boundary, checks
all fields, corrupts the newest slot, checks fallback, exercises death/quit, corrupts both
slots, checks defaults, and checks migration and invalid-state rejection. Every check logs
`TEST campaign-save section <n>: PASS|FAIL <detail>`; success and caught failure both call
`gd.quit()`. A host-tick timeout fails a stalled transition. Accept only a complete run with
no FAIL lines and the `TEST campaign-save complete` marker. The test deliberately overwrites
its own `campaign-test.*.gdc` files, never a production slot.

## Verification and limits

Performed without building or running the game:

- Lua 5.4.7 parse checks (`luac -p`) of library, body, bundle and host checks.
- `lua pc/tests/campaign-save-unit.lua ../codex-gamemode/pc/scripts/lib/gamemode.lua`:
  PASS with mocked storage/scenes; covers the LAB body, failed-write state preservation,
  truncated-record recovery, future-schema write refusal, offline-load refusal, cycle rejection,
  and the actual sibling framework's fresh-room resume/party commit via mocked game APIs.
- `git diff --check`.

Attempted PowerPC syntax-only check of changed C using the workspace LLVM executable and the
AGENTS.md flags. The sandbox returned **permission denied executing clang.exe**, so C syntax
has not been verified. `gw_script.c` is a Windows-native TU; full native syntax/link validation
also remains for the integration lane. No game-side C was changed.

Untested: native Windows I/O success/failure/atomic-replace behavior, disk-full and power-loss
recovery, actual engine scene timing, native online gating, gameplay budget at maximum payload
size, real framework resource allocation, and the LAB runtime script. No exe-string, screen,
or game-log verification is claimed. No builds, game runs, commits, or sibling edits.
