EnemyGenes is a pure Lua adapter bundled beside Core; it does not require modules.

`EnemyGenes.new(Core, gd, {get_run=function() return current_run end,
on_tell=function(state,event) end,
on_player_hit=function(handle,from_port,damage) end,
target_port=1,target_host='player'})` returns an adapter.

Call `:attach(handle,{host='enemy_room_spawn',family='cinder'|'rime',slot='assault'})`
after native spawning. Stable host IDs reuse the existing equipped instance on
re-entry. Attach authors +1 potency and -2 capacity encounter modifiers through
Core; the original gene stays immutable. Other placements require a future
custom controller and are rejected. The same Core resolver supplies cost,
cooldown, damage, reach, marks and Thermal Shock as for fighters.

Call `:tick()` once after `Core.tick(run,1)` during unpaused gameplay. It never
advances the run clock. Natural item contact activation IDs charge the gene;
ready signatures show a 24-frame telegraph, then perform a native owned strike.
Native refusal restores charge, cooldown and target mark, with a 20-frame retry
wait. Scripted reaction strikes do not count as natural charge contacts.

`:states()` returns sorted draw data: handle, host, family, slot, x, y, facing,
phase, charge, cost, ability and windup_until. on_tell receives that state plus
attached/charging/ready/telegraph/recovery/release/refused/detached. No callbacks
fire for every unchanged frame. `:detach(handle)` and `:clear()` remove adapter
attachments, not native actors or permanent run hosts. Root owns native removal.
Keeping enemy instances equipped prevents player fusion/export borrowing.

Native `on_enemy_hit(event)` is the authoritative incoming contact stream:
`{handle,from=one_based_port,damage,reaction=false}` is queued before item damage
callbacks and delivered before the resulting defeat, even after destruction.
Subfighters and scripted gene damage do not emit it. Runtime uses its genuine
player move activation ID for Core charging, rather than inventing one per hit.
Adapter on_player_hit polling is disabled by default; explicitly setting
poll_player_hits=true enables the diagnostic fallback, which can miss lethal
or multiple contacts between ticks. Do not combine it with authoritative events.
Missing/stale handles detach without attacking or inventing defeat events.

Adapter tests execute actual Lua with a simulated native boundary; they verify
shared rules, refusal rollback, deduplication, stable host reuse and fresh run
references. They do not replace native tests or normal-speed live acceptance.
