//! `gobjuserdata.c`: re-export the existing, C-oracle-verified arena port.
//!
//! `GObj_InitUserData` = `GObjSystem::init_user_data`;
//! `GObj_RemoveUserData` = `GObjSystem::remove_user_data`.
pub use crate::gobj::{GObjHost, GObjId, GObjSystem, HsdGObj};
