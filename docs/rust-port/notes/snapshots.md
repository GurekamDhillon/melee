# State snapshots

Moved verbatim from RUST_PORT.md (original lines noted per block). Current truth: docs/rust-port/STATUS.md. Append new notes here, not in RUST_PORT.md.

<!-- RUST_PORT.md lines 1239-1244 -->
## State snapshots (resume the golden sim mid-match)
`MELEE_SNAP_DUMP=<frame>[,..]:<file>` makes the C build dump its gameplay state after a match frame;
`melee golden --from-snapshot FILE --start N` resumes the native sim from it and compares the hash from N
on. Dump/resume/proof commands, what is loaded from where, and the model differences found:
`tools/rust-port/snap/README.md`. Proven on `pad_falcon_fox_fd` at frames 0 and 300.
