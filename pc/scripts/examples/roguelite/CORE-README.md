# Pure roguelite core

`core.lua` returns a table without calling `gd`, requiring libraries or performing
IO. The installer must bundle it before the runtime; the in-engine sandbox has no
`require`/`dofile`. All times are simulation frames. The module emits actions; it
does not implement damage, movement, visuals, menus, room generation or AI.

## Runtime contract

- `new_profile(seed)` creates three authored starter individuals, including two
  contrasting compatible Cinders. Seeds are integers 1..2147483646.
- `new_run(profile, {seed=, stocks=})` reserves the next run ID and makes independent
  copies. Persist the profile reservation before the run. `run.owner` binds the run
  to `profile.id`; profile identity derives from its initial seed in this prototype,
  so independent profiles must use different seeds.
  Generate and resume the dungeon from immutable `run.world_seed`; `run.seed` is
  the mutable acquisition/fusion random stream and must not regenerate room graphs.
- `equip(run, host, slot, gene_id)` assigns one gene to `assault`, `traversal` or
  `guard`; nil unequips. Each individual can be equipped once, on player or enemy.
  Model mesh count never grants slots. Moving requires unequip first; its charge,
  cooldown and deduplication history remain with the individual. Slot modifiers
  clear on equip changes. Unequipped genes remain in the inventory.
- `resolve(run, host, slot)` computes bounded values from base, upgrades and active
  modifiers. It never rewrites base traits. `apply_modifier` takes
  `{id=, stat=, add=, expires=absolute_frame}`; `remove_modifier` removes its ID.
  `tick(run, frames)` advances simulation time, expires penalties/marks and clamps
  charge. Stat modifiers are additive; cooldown resolves to integer frames.
- `ability(run, host, slot)` returns category/name/family/action/effect recipe,
  charge/readiness, cooldown remaining, cost, reach, damage and knockback metadata.
  Categories support the runtime's three-branch recursive command tree; this core
  does not handle D-pad input or taunts.
- `on_event(run, {host=, kind=, move_id=, reaction=, lineage=})` accepts `direct_hit`,
  `defend` or `move`. The engine adapter must verify genuine hits/defenses/movement,
  provide a globally unique move-instance ID and distinguish shield contact.
  One instance earns charge once per equipped gene. Reaction/non-direct lineage
  events cannot earn charge. History is bounded to 512 IDs per individual and
  expires after 600 frames while equipped; this expiry is not permission to reuse
  move-instance identifiers.
- `activate(run, host, slot, {target=})` consumes the entire charge and starts
  cooldown only when ready. Failure returns `nil, reason` without changes. A Rime
  mark requires a stable target ID; a subsequent charged fire action consumes the
  mark for one Thermal Shock. Returned action has `source`, `target` and
  `lineage='reaction'`; any resulting engine hit must retain that lineage. The
  runtime chooses legal targets/range and translates `step`, `glide`, `counter`,
  `eruption` and `mark` into authored engine behavior. Effect recipes are existing
  package names, not newly created art or proof of implemented visuals.
- `reward(run, id, stat, delta)` makes a bounded run improvement/tradeoff;
  `acquire(run, kind)` adds an authored individual. `lock(container,id,stat,bool)`
  protects inherited base traits. `breed(profile,a,b)` requires compatible distinct
  parents, preserves originals and creates deterministic offspring.
  `fuse(run,a,b)` requires unequipped compatible parents and consumes them; their
  bounded run upgrades are averaged separately from inherited base. Conflicting
  locked base traits reject the transaction without mutations.
- `finish(profile,run,'success',gene_id)` exports one individual, retaining base,
  ancestry and locks but discarding run upgrades. `'failure'` exports nothing and
  preserves collection genes. The profile's finished ledger makes repeated finish
  calls idempotent, including a crash after profile write but before run write.
  Success with nil gene explicitly skips export, including when the collection is
  full; an attempted export into a full collection fails without mutations.
  Store the profile first, then the updated run; IO is owned by the runtime.
- `run.progress={room='entry',cleared={},claimed={},supplies=2}` is the canonical
  traversal/reward checkpoint. The runtime may update it, then validate with
  `snapshot`. Room/flag names are bounded, flags are booleans and supplies are 0..9.
- `snapshot(profile_or_run)` and `restore(text)` return `value` or `nil,reason`.
  The bounded JSON object codec parses data directly without Lua `load`, rejects
  unknown schema fields/versions, duplicate keys, nonfinite values and invalid
  references, and normalizes ancestry indices. No executable input is evaluated.
  Save size is at most 256 KiB; strings at most 256 bytes; recursion at most 16.

Collection size is capped at 128 and live hosts at 32. The finish ledger is a small
prototype ledger constrained by the save object's 512-field limit; long-lived
production profiles will need a versioned bounded export ledger policy. Success
export/failure and nonconsuming breeding are reversible prototype economy defaults.
Callers must not mutate definitions, base genes or internal event state directly;
these are ordinary Lua data tables, not a security boundary against trusted scripts.

Run `python3 tools/roguelite/test_core.py` from the repository root. It invokes the
actual Lua interpreter to verify source isolation, modifiers, placement behavior,
player/enemy activation, reaction gating, cooldown retention, breeding/fusion,
progress validation, malformed saves, profile ownership and crash-safe export.
These tests do not establish playable engine integration or visual correctness.

Native `gd.data_write` currently truncates the destination and does not establish
atomic replacement or check short writes. Persistence integration must retain a
previous validated checkpoint (for example alternating complete checkpoint slots
with generation/readback validation) or use an audited atomic native wrapper.
Separate profile/run writes alone do not protect against a truncated profile file.
