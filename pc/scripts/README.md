# Scripted hit for gameplay tests

`gd.hit(port, {damage=, angle=, kbg=, bkb=, from=})` applies a hit to a live
fighter through Melee's collision damage result and fighter hit processing.
Ports are one-based. `from` is an optional attacker port; omit it or set it to
`nil` for an anonymous hit. All four hit values are required integers. Damage
is 0–500, angle 0–361, and KBG/BKB 0–1000. The call returns `true` when a
live victim (and, if specified, live attacker) accepted the hit, or `false`
when a port has no live fighter. It is available to offline gameplay scripts
and the console; gameplay scripts need `"gameplay": true` in `mod.json`.

```lua
-- In an offline match, hit the boss on port 2 from port 1.
-- The fighter's damage, hitlag, reaction and stamina HP use the game path.
gd.hit(2, {damage=30, angle=45, kbg=100, bkb=20, from=1})
```

Unlike `gd.set_damage`, this also invokes the victim's damage state. For a
stamina boss, an HP-depleting hit can therefore lead to `on_boss_defeated`
through the ordinary KO flow. `gd.set_damage` is useful for preparing the
fighter's percent/HP before the final hit.
