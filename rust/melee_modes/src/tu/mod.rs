// Group 6 per-TU ports (gmapproach, gmboot, gmcamera; tests/tu_<stem>.rs) plus the shard-465-472 ports below.
pub mod gmapproach;
pub mod gmboot;
pub mod gmcamera;
// Per-TU native ports of the game's gm/mn/lb/if translation units (one module per `.c` file,
// `pub mod <tu_stem>;`). Each module's `ext` lists the callees of unported TUs as settable
// hooks named like the C functions (unset: panics "TODO"); the oracle tests in
// `tests/tu_<stem>.rs` install recording hooks and compare against the C
// (`gen/cref/tu/<stem>.c`, the whole TU, shipping contraction).
pub mod support;
pub mod gmcameramode;
pub mod gmclassic;
pub mod gmdebugmode;
pub mod gmevent;
pub mod gmfixedcamera;
pub mod gmgiant;
pub mod gmgover;
pub mod gmhanyucss;
pub mod gmhanyusss;
pub mod gmhomerun;
pub mod gmhowto;
pub mod gminvisible;
pub mod gmlightning;
pub mod gmmain;
pub mod gmmain_lib;
pub mod gmmenu;
pub mod gmomake15;
pub mod gmpause;
pub mod gmprogressive;
pub mod gmmovieend;
pub mod gmopening;
pub mod gmmenumode;
pub mod gmopeningmode;
pub mod gmmultiman;
pub mod gm_180A;
pub mod gm_186E;
pub mod gm_181A;
pub mod gm_1832;
pub mod gm_1879;
pub mod gm_1884;
pub mod gm_19EF;
pub mod gm_18A1;
pub mod ext_hook;
pub mod gm_1736;
pub mod gm_17ad;
pub mod gm_17ba;
pub mod gm_17e4;
pub mod gm_17eb;
pub mod gm_16f1;
pub mod gm_1798;
pub mod gm_17c0;
// One file per game translation unit (port rounds "shard"): src/melee/gm/<stem>.c -> tu/<stem>.rs.
// Callees of unported TUs are `ext` stubs in each file; every file's functions are checked by the
// state oracle (`crate::oracle`, `gen/tu/run.py`) against the C compiled natively.
pub mod gmfrontend;
pub mod gmprogressivemode;
pub mod gmregcommon;
pub mod gmregtyfall;

pub mod gmregclear;
pub mod gmregenddisp;
pub mod gmresult;
pub mod gmresultplayer;
pub mod hsd;
pub mod gmscdata;
pub mod gmscene;
pub mod gmsinglebutton;
pub mod gmslomo;
pub mod gmstaffroll;
pub mod gmstamina;
pub mod gmsupersudden;
pub mod gmtiny;
