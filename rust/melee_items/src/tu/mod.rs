//! Whole-TU native ports of the item kinds' `.c` files (shard 657-664). Each file ports every function of one
//! `src/melee/it/kinds/<stem>.c` (statics and header inlines included) with the C's name and signature;
//! callees of other TUs go through the file's generated `ext` hooks (unset: panics "TODO"). Checked by
//! tests/kinds_<stem>.rs against the C (gen/kinds).
pub mod kinds_inl;
pub mod itarwinglaser;
pub mod itbat;
pub mod itcapsule;
pub mod itcerebi;
pub mod itchicorita;
pub mod itclimbersblizzard;
pub mod itbombhei;
pub mod itbox;
