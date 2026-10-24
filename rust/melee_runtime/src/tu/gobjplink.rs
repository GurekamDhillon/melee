//! `gobjplink.c`: re-export the existing, C-oracle-verified arena port.
//!
//! `GObj_PReorder` = `GObjSystem::p_reorder`; `CreateGObj` = `create_gobj`;
//! `GObj_Create` = `gobj_create`; `HSD_GObjFree` = `free`;
//! `HSD_GObjPLink_ChangeGObjPri_Unk` = `change_gobj_pri`. The two static
//! priority walkers remain private implementation details of that same port.
pub use crate::gobj::{GObjHost, GObjId, GObjSystem, HsdGObj};
