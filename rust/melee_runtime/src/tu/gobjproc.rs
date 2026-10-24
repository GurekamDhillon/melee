//! `gobjproc.c`: re-export the existing, C-oracle-verified arena port.
//!
//! `HSD_GObjProc_QueueProc` = `GObjSystem::queue_proc`;
//! `HSD_GObjProc_UnqueueProc` = `unqueue_proc`;
//! `HSD_GObjProc_UnlinkProcFromGObj` = `unlink_proc_from_gobj`;
//! `HSD_GObj_SetupProc` = `setup_proc`; `HSD_GObjProc_RemoveProc` = `remove_proc`;
//! `HSD_GObjProc_RemoveAllProcs` = `remove_all_procs`.
pub use crate::gobj::{GObjId, GObjSystem, HsdGObjProc, ProcId};
