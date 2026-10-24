//! `src/MSL/ansi_files.c` contains no function definitions: only the three MSL
//! `FILE` initializers for stdin/stdout/stderr. Native Rust I/O uses the host's
//! standard streams, as specified by inventory.md section 1 (MSL becomes Rust core).
//! The legacy FILE record and its GameCube metadata remain in `melee_types`.
