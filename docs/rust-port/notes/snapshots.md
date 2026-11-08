# State snapshots

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 1239-1244 -->
## State snapshots (resume the golden sim mid-match)
`MELEE_SNAP_DUMP=<frame>[,..]:<file>` makes the C build dump its gameplay state after a match frame;
`melee golden --from-snapshot FILE --start N` resumes the native sim from it and compares the hash from N
on. Dump/resume/proof commands, what is loaded from where, and the model differences found:
`tools/rust-port/snap/README.md`. Proven on `pad_falcon_fox_fd` at frames 0 and 300.

## Harness counters stay off the statics (MELEE_SNAP_CHECK in slippi-netplay)
The snapshot saves and restores every writable static of the exe, so a static is sim state by rule. A harness
value that describes something the snapshot does not restore (the system heap, sockets, threads) must not live in a
static: a load would rewind it while the thing it counts stays as it is. `melee::SYS_LIVE` (system-heap live bytes,
bumped by the global allocator on every thread) was one: with port/npping's socket reader threads allocating during a
load, MELEE_SNAP_CHECK=1 failed after the first rollback, "range 2 page 120", 2 bytes at .data+0x78f08 (one 8-byte
.data contribution of the melee crate, no PDB symbol; found by the RIP-relative `addq` to it in `sys_add`). It was never
read by the sim (only `play`'s "mem after" line, which rollbacks/seeks also made wrong), so no desync. Fix: the counter
lives in a system-heap cell; the static only holds its address, set once at startup before any snapshot base.
MELEE_SNAP_CHECK now prints the differing bytes and the RVA of the first one; map it with
`llvm-pdbutil dump -section-contribs melee.pdb` (.data starts at the section's RVA) and `llvm-objdump -d` for writers.
