# SSE placed objects: inventory (port/sse-full)

Every placed object kind in every Subspace Emissary archive of the owner's Brawl files (270 files), counted by
the local-only `_build/boss-ir/tools/inv/inventory.py`. Per-file and per-room detail is local only
(`_build/boss-ir/sse-inventory.md`). Kind names only here, no offsets or values. Meanings are from the public
BrawlCrate categories (`sse-audit.md` section 1); "?" kinds are not identified yet.

Port column: **supported** = loaded and live; **static** = its collision is in the room but held still (no
switch / path logic yet); **dropped** = not loaded at all.

Enemies (2392 generator points): 635 are kinds with a hand-made actor of their own (`subspace.rs`: Goomba, Jyk,
Giant Goomba, Bullet Bill, Hammer Bro, Primid, Koopa); 1489 are run by the enemy-command VM (`explore_vm.rs`, Brawl's
own enemy scripts compiled from the owner's files: Mite, Bucculus, Roturret, Floow, Bytan, Spaak, Feyesh, Glunder,
the R.O.B.s and the rest with a VM program); 170 (the Primid variants without a program, kinds 28/31/33/34) are
logged and not spawned; 98 are boss parts (their own boss fights). `MELEE_SSE_STANDINS=1` puts the nearest hand-made
actor in place of the 170; `MELEE_SSE_VM=0` turns the VM lane off. Both are boot options that both netplay peers
must set alike.

The VM lane's host (ours, not Brawl data): the ground / air switch (a floor within 4 units below holds a walker,
walking off one drops it), a fall of 0.1 a frame up to 2.5 in falling / damage motions (the VM's own gravity is 0),
a hurt capsule from the model's rest extent, the start status 0 (Brawl's start status comes from the generator
record, not read yet). Checked: Lake 3 (Mite, Bucculus) and level 22 with synctest depth 4 / 7, 0 mismatches;
enemies are hit and beaten by fox, and level 22's enemies' attack boxes land on him.

| kind | what | placed | files | port |
|---|---|---|---|---|
| GEG1 | enemy generators | 2392 | 162 | 635 with an actor; 1489 more by the VM, 170 listed |
| GET2 | area triggers (2) | 1175 | 119 | dropped |
| GFG1 | fighter spawns (ally/CPU fights) | 720 | 44 | dropped |
| GET1 | area triggers (1) | 564 | 162 | dropped |
| GMOV | moving map parts / lifts | 545 | 117 | mode 1 supported (314); mode 2 static (231) |
| EPSP | enemy spawn points | 417 | 134 | dropped |
| GDOR | doors | 305 | 140 | supported |
| GFT2 | triggers (GFT2) | 191 | 53 | dropped |
| GIB2 | item boxes / crates | 180 | 106 | supported (Melee container) |
| GTIK | GTIK (other) | 179 | 15 | dropped |
| GASD | GASD trigger | 153 | 81 | dropped |
| GNDV | GNDV graphics | 148 | 11 | dropped |
| GMOT | model motion | 136 | 56 | dropped |
| GBLK | breakable blocks | 131 | 53 | supported (collision breaks; model stays) |
| GEPT | enemy-pattern triggers | 119 | 41 | dropped |
| GLAD | ladders | 111 | 44 | dropped |
| GMWA | walls (switch gated) | 107 | 36 | static |
| GSPR | springs | 100 | 15 | dropped |
| GMTA | GMTA objects | 99 | 25 | dropped |
| GFPR | fire pillars | 95 | 10 | dropped |
| GSBK | switch blocks | 93 | 22 | static |
| GMCA | GMCA hazards | 91 | 22 | dropped |
| GITM | placed items | 86 | 51 | dropped |
| GDBF | Halberd/factory doors | 81 | 47 | dropped |
| EPBM | boss marks | 76 | 51 | dropped |
| GLOK | camera/area locks | 73 | 49 | dropped |
| GMWO | walls (switch open) | 70 | 26 | static |
| GWAT | water | 66 | 23 | dropped |
| GEFF | effects | 64 | 24 | dropped |
| GQUK | quakes | 56 | 25 | dropped |
| GFRT | triggers (GFRT) | 51 | 12 | dropped |
| GBC2 | cannons (2) | 51 | 6 | dropped |
| GMPS | moving platforms | 50 | 11 | static |
| GDMD | GDMD (other) | 49 | 42 | dropped |
| GSTG | GSTG objects | 47 | 7 | dropped |
| GMIN | GMIN (other) | 46 | 10 | dropped |
| GFSR | camera scroll ranges | 40 | 28 | static (Brawl camera range used) |
| GMVA | moving parts (GMVA) | 31 | 5 | dropped |
| GCAT | GCAT objects | 29 | 13 | dropped |
| GSHT | shutters | 29 | 5 | static |
| GENC | GENC hazards | 28 | 14 | dropped |
| GBC1 | cannons (1) | 27 | 10 | dropped |
| GFRK | falling rocks | 26 | 8 | dropped |
| GSND | sounds | 23 | 16 | dropped |
| GMV4 | moving parts (GMV4) | 23 | 9 | dropped |
| GMOA | GMOA triggers | 22 | 10 | dropped |
| GWND | wind | 22 | 14 | dropped |
| GBLT | conveyor belts | 22 | 2 | dropped |
| GLK2 | locks (2) | 21 | 13 | dropped |
| GELA | elevators (A) | 21 | 7 | static |
| GTRR | trains | 20 | 2 | dropped |
| GMSJ | level jumps / exits | 18 | 18 | supported |
| GTRC | tracks | 16 | 8 | dropped |
| GBRF | GBRF objects | 13 | 6 | dropped |
| GELE | elevators | 13 | 10 | static |
| GITR | item rolls | 12 | 9 | dropped |
| GMEA | GMEA hazards | 12 | 2 | dropped |
| GWAP | warps | 12 | 1 | dropped |
| GWD2 | wind (2) | 11 | 11 | dropped |
| GFIM | GFIM objects | 10 | 1 | dropped |
| GSAV | save points | 9 | 9 | dropped |
| GMRL | GMRL objects | 8 | 1 | dropped |
| GHPE | GHPE hazards | 7 | 2 | dropped |
| GSCN | scene changes | 5 | 5 | dropped |
| GPS2 | moving platforms (2) | 4 | 3 | static |
| GCMP | camera points | 4 | 2 | dropped |
| GDTP | GDTP objects | 3 | 2 | dropped |
| GFIN | GFIN (other) | 3 | 3 | dropped |
| GCAM | cameras | 2 | 1 | dropped |
| GKNG | GKNG | 1 | 1 | dropped |
| GDBS | GDBS doors | 1 | 1 | dropped |

| enemy | placed | port |
|---|---|---|
| Primid | 204 | actor |
| Mite | 140 | VM (explore_vm) when it has a program, else listed |
| Sword Primid | 103 | VM (explore_vm) when it has a program, else listed |
| Jyk | 102 | actor |
| Koopa | 97 | actor |
| Feyesh | 93 | VM (explore_vm) when it has a program, else listed |
| Goomba | 93 | actor |
| Bullet Bill | 84 | actor |
| Bytan | 74 | VM (explore_vm) when it has a program, else listed |
| Auroros | 71 | VM (explore_vm) when it has a program, else listed |
| Floow | 68 | VM (explore_vm) when it has a program, else listed |
| Primid variant 34 | 67 | VM (explore_vm) when it has a program, else listed |
| R.O.B. Sentry | 65 | VM (explore_vm) when it has a program, else listed |
| Boomerang Primid | 62 | VM (explore_vm) when it has a program, else listed |
| Spaak | 60 | VM (explore_vm) when it has a program, else listed |
| Buckot | 56 | VM (explore_vm) when it has a program, else listed |
| Puppit | 56 | VM (explore_vm) when it has a program, else listed |
| Primid variant 33 | 55 | VM (explore_vm) when it has a program, else listed |
| Borboras | 50 | VM (explore_vm) when it has a program, else listed |
| Hammer Bro | 44 | actor |
| Bombed | 43 | VM (explore_vm) when it has a program, else listed |
| Ticken | 40 | VM (explore_vm) when it has a program, else listed |
| Glunder 44 | 38 | VM (explore_vm) when it has a program, else listed |
| Glunder 45 | 37 | VM (explore_vm) when it has a program, else listed |
| Glunder 43 | 36 | VM (explore_vm) when it has a program, else listed |
| Shaydas | 35 | VM (explore_vm) when it has a program, else listed |
| Cymul | 32 | VM (explore_vm) when it has a program, else listed |
| R.O.B. Launcher | 32 | VM (explore_vm) when it has a program, else listed |
| Roader | 29 | VM (explore_vm) when it has a program, else listed |
| Trowlon | 29 | VM (explore_vm) when it has a program, else listed |
| R.O.B. Blaster | 28 | VM (explore_vm) when it has a program, else listed |
| Armight | 27 | VM (explore_vm) when it has a program, else listed |
| Primid variant 28 | 26 | VM (explore_vm) when it has a program, else listed |
| Bucculus | 25 | VM (explore_vm) when it has a program, else listed |
| Shellpod | 25 | VM (explore_vm) when it has a program, else listed |
| Towtow | 22 | VM (explore_vm) when it has a program, else listed |
| Primid variant 31 | 22 | VM (explore_vm) when it has a program, else listed |
| Poppant | 20 | VM (explore_vm) when it has a program, else listed |
| Roturret | 17 | VM (explore_vm) when it has a program, else listed |
| Nagagog | 17 | VM (explore_vm) when it has a program, else listed |
| Autolance | 16 | VM (explore_vm) when it has a program, else listed |
| R.O.B. Distance | 15 | VM (explore_vm) when it has a program, else listed |
| Greap | 11 | VM (explore_vm) when it has a program, else listed |
| Giant Goomba | 11 | actor |
| Gamyga | 10 | VM (explore_vm) when it has a program, else listed |
| Gamyga base | 10 | VM (explore_vm) when it has a program, else listed |
| Gamyga base | 10 | VM (explore_vm) when it has a program, else listed |
| Gamyga base | 10 | VM (explore_vm) when it has a program, else listed |
| Gamyga base | 10 | VM (explore_vm) when it has a program, else listed |
| Galleom (boss) | 9 | boss: own boss fight |
| Master Hand (boss) | 8 | boss: own boss fight |
| Armank | 7 | VM (explore_vm) when it has a program, else listed |
| Crazy Hand (boss) | 6 | boss: own boss fight |
| Petey (boss) | 5 | boss: own boss fight |
| Rayquaza (boss) | 5 | boss: own boss fight |
| Porky (boss) | 5 | boss: own boss fight |
| Ridley (boss) | 5 | boss: own boss fight |
| Duon (boss) | 5 | boss: own boss fight |
| Meta Ridley (boss) | 5 | boss: own boss fight |
| Falcon Flyer (boss) | 3 | boss: own boss fight |
| Tabuu (boss) | 2 | boss: own boss fight |
