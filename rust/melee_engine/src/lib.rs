//! Engine host (consolidation plan step 5): the OS heaps, the DVD seam and the archive images the game's own
//! loaders (`lbFile`, `lbArchive`, `HSD_ArchiveParse`) run on. No game logic lives here.
#![allow(static_mut_refs, non_snake_case, clippy::missing_safety_doc)]
pub mod archive;
pub mod dat_types;
pub mod dvd;
pub mod engine;
pub mod gcm;
pub mod host;
pub mod install;
pub mod lowmem;
pub mod os;
pub mod scenario;
pub mod seams;
pub mod seams_gen;

pub use engine::Engine;
pub use host::Host;
pub use install::install;
