Goal: the whole codebase (decomp C, the 10 PowerPC `.s` files, `pc/` platform layer) in Rust, with game logic
bit-identical to the C build so rollback netplay and Slippi replays still agree.

**Start at `docs/rust-port/STATUS.md`** (current decisions, rules, crate map, commands, golden status, blockers, topic index).
The long history and topic detail that used to live here is in `docs/rust-port/notes/<topic>.md` (moved verbatim, 2026-10-30).

**Adding notes (merge-friendly):** write the detail in the matching `docs/rust-port/notes/<topic>.md` and add ONE line below
(`- YYYY-MM-DD branch: what changed -> notes/<topic>.md`), appended at the end. Do not paste sections into this file.

## Changelog (one line per entry; entries before 2026-10-30 are the old section headings, in file order)
- 2026-10-2x Oracle (already in the C build) -> notes/toolchain.md
- 2026-10-2x Phases -> notes/toolchain.md
- 2026-10-2x Toolchain -> notes/toolchain.md
- 2026-10-2x Inventory (docs/rust-port/inventory.md) -> notes/toolchain.md
- 2026-10-21 Translation toolchain -> notes/toolchain.md
- 2026-10-2x Cargo workspace (`rust/`) -> notes/toolchain.md
- 2026-10-2x Golden corpus (phase 1) -> notes/golden.md
- 2026-10-21 Float contraction -> notes/fma.md
- 2026-10-2x Group 0: melee_types (types + GC layout table) -> notes/types.md
- 2026-10-21 Group 3: fighter engine + ftCommon + pl -> notes/groups.md
- 2026-10-21 Integration, night 1 -> notes/integration.md
- 2026-10-2x Night 2: next steps, in order -> notes/integration.md
- 2026-10-21 Night 2 -> notes/integration.md
- 2026-10-21 Platform track, step 1: melee_platform -> notes/platform.md
- 2026-10-21 Platform track, step 2: melee_gx -> notes/render.md
- 2026-10-21 Platform track, night 3 step 1: shim_gx + swapchain/present + gx_viewer -> notes/render.md
- 2026-10-21 Platform track, night 3 step 2: gw_rollback.c session + SyncTest -> notes/net.md
- 2026-10-21 Platform track, night 3 step 4: melee_audio connected -> notes/audio.md
- 2026-10-21 Platform track, step 5: mex-import -> notes/data.md
- 2026-10-21 Sim round 2: pad_falcon_fox_fd to frame 103 -> notes/golden.md
- 2026-10-21 Sim round 1: pad_falcon_fox_fd through the entry sequence -> notes/golden.md
- 2026-10-21 Native golden run: pad_falcon_fox_fd -> notes/golden.md
- 2026-10-21 End of night 2: summary -> notes/integration.md
- 2026-10-2x Night 3: plan, in order -> notes/integration.md
- 2026-10-2x melee_modes::tu: whole-TU ports with a stateful C oracle (shard 465-472) -> notes/oracles-mtu.md
- 2026-10-2x melee_world::tu: whole-TU gr ports with a stateful C oracle (shard 553-560) -> notes/oracles-mtu.md
- 2026-10-2x melee_world::tu: gr/grt*.c set B (shard 601-608) -> notes/oracles-mtu.md
- 2026-10-2x melee_world::tu: gr/grt*.c set C (shard 609-616) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds item-kind ports with a C state oracle (shard 657-664) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds itpeach*, itpikachu*, itpippi (shard 769-776) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set (shard 673-680) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it TUs with a stateful C oracle (shard 649-656) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set (shard 681-688) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set (shard 705-712) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set 721-728 (itkyasarinegg itleadead itlgun itlgunbeam itlgunray itlikelike itlinkarrow itlinkbomb) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds itluigifireball, itmaril, itmariocape, itmariofireball, itmarumine, itmasterhandbullet, itmasterhandlaser, itmatadogas (shar -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds nokonoko ... peachexplode (shard 761-768) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set 2 (shard 777-784) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set 2 (shard 785-792) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set B (shard 793-800) -> notes/oracles-mtu.md
- 2026-10-2x melee_items::tu: it/kinds set 2 (shard 809-816) -> notes/oracles-mtu.md
- 2026-10-2x Shard 881-888: pl/player.c plattack.c plbonus.c plbonuslib.c, mn/mn_22EC.c, mp/mpisland.c mplib.c mpcoll.c -> notes/oracles-mtu.md
- 2026-10-24 Status recount and next plan -> notes/integration.md
- 2026-10-2x melee_items::tu: it/kinds set (shard 665-672) -> notes/oracles-mtu.md
- 2026-10-24 Merge of port/shard-665-672, failing-target fixes, stub audit -> notes/integration.md
- 2026-10-24 melee_chara stub fill -> notes/integration.md
- 2026-10-24 melee_fighter stub fill -> notes/integration.md
- 2026-10-24 Golden round 1: pad_falcon_fox_fd matches in full -> notes/golden.md
- 2026-10-24 Golden round 2: pad_marth_sheik_bf wired, frames 0..268 match -> notes/golden.md
- 2026-10-24 Golden round 3: pad_marth_sheik_bf matches in full -> notes/golden.md
- 2026-10-24 Golden round 4: cpu_fox_falco_dl wired, frames 0..63 match -> notes/golden.md
- 2026-10-24 Status after the stub-fill merges and golden rounds 1-4 -> notes/golden.md
- 2026-10-25 Golden round 5: cpu_fox_falco_dl frames 0..91 match -> notes/golden.md
- 2026-10-25 Golden round 6: cpu_fox_falco_dl 0..117, cpu_peach_samus_ys_items wired, 0..94 -> notes/golden.md
- 2026-10-25 Golden round 7: cpu_fox_falco_dl 0..139, cpu_peach_samus_ys_items 0..119 -> notes/golden.md
- 2026-10-25 Status: golden frames matching per scenario, and blockers -> notes/golden.md
- 2026-10-2x State snapshots (resume the golden sim mid-match) -> notes/snapshots.md
- 2026-10-2x melee_world::tu::efn: ef/eflib.c, efasync.c, efdata.c via c2rs + state oracle (pre-melee_world) -> notes/oracles-mtu.md
- 2026-10-24 melee_chara trace verification -> notes/traces.md
- 2026-10-24 melee_world trace verification -> notes/traces.md
- 2026-10-25 Trace status regenerated after the pre-melee_* merges -> notes/traces.md
- 2026-10-24 Platform track: `melee play` -> notes/play.md
- 2026-10-24 Platform track, audio proof -> notes/audio.md
- 2026-10-24 Platform track: scripting, `melee_script` -> notes/script.md
- 2026-10-24 Platform track: netplay on the real game loop -> notes/net.md
- 2026-10-24 Rust vs C build over the wire -> notes/net.md
- 2026-10-24 Platform track: saves, settings, mods, replays (melee_data), 2026-10-24 -> notes/data.md
- 2026-10-25 Platform track, audio round 2: pause and time-up proven against the C mix -> notes/audio.md
- 2026-10-25 Platform track, audio round 5: menu / title / results / sudden death replayed against the C mix -> notes/audio.md
- 2026-10-25 Platform track, audio round 6: retail main menu, stream block refills chunk by chunk -> notes/audio.md
- 2026-10-25 Platform track, render proof -> notes/render.md
- 2026-10-25 Platform -> notes/platform.md
- 2026-10-25 Platform track, audio round 7: driving past the main menu (no new capture) -> notes/audio.md
- 2026-10-30 port/tooling: `melee diff` field-level C-vs-Rust diff; RUST_PORT.md split into STATUS.md + notes/ -> notes/diff.md
- 2026-10-25 melee_fighter trace verification (port/pre-melee_fighter) -> notes/traces.md
- 2026-10-30 Trace status after all six pre-melee_* merges -> notes/traces.md
- 2026-10-30 rust-port: consolidation plan (real GObj loop on tu/ ports, vs_match stand-in inventory, dedupe list, diff gate, logic lint) -> notes/consolidation.md
