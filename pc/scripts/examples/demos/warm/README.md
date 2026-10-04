# Warm before drawing

Load this single-feature demo in an offline vanilla Final Destination match.
It declares all seven enemy kinds, shows descriptor/pipeline progress and spawns
one Goomba only after `gd.warm_done` reports success. No menu action is needed.

`gd.warm` returns immediately. One descriptor is discovered per engine tick;
background workers compile its material pipelines. Keep polling rather than
assuming a frame count. Failure is displayed and never treated as readiness.
Owned warm and enemy handles are released on unload. Start a fresh match to
repeat. Cold-cache GPU/native acceptance remains pending; Lua stub checks are
ordering and cleanup evidence only.
