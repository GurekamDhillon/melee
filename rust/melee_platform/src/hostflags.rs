//! Cross-thread handshake flags that must never be part of the game state. A rollback save copies the game crates'
//! statics and a load writes them back (melee_engine's snapshot.rs); melee_platform is a host-only crate in the
//! statics mask (melee_engine's snapmask.rs), so flags defined here are neither copied, restored nor checked.
//! Defined here rather than beside their users because their users live in melee_engine, a game crate.
use core::sync::atomic::AtomicBool;

/// melee_engine::arena: snapshot work (save / load / hash) is running; the background threads hold off.
pub static BG_PAUSE: AtomicBool = AtomicBool::new(false);
/// melee_engine::arena: the arena's background thread is inside its critical part.
pub static BG_BUSY: AtomicBool = AtomicBool::new(false);
/// melee_engine::arena: the GX decode thread is inside a job. Set and cleared by that thread at any time, including
/// while a load runs (its begin raises it before it can see `BG_PAUSE`), so it cannot live in a game static: a load
/// would write it back under that thread, and MELEE_SNAP_CHECK saw the flip as a differing statics page (page 138).
pub static JOB_BUSY: AtomicBool = AtomicBool::new(false);
