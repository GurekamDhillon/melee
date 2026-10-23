//! Whole-TU native ports of the item engine's `.c` files (shards 641-648). Each file ports every function of
//! one `src/melee/it/<stem>.c` (statics and inlines included) with the C's name and signature; callees of other
//! TUs go through the file's generated `ext` hooks (unset: panics "TODO"). Checked by tests/tu_<stem>.rs against
//! the C (gen/mtu).
pub mod ithitbox;
pub mod itdrop;
pub mod iteffect;
pub mod itdraw;
pub mod itcoll;
pub mod itgroundcoll;
pub mod itmaplib;
pub mod item;
pub mod itanimlist;

// Shard 657-664: whole-TU native ports of the item kinds' `.c` files (shard 657-664). Each file ports every function of one
// `src/melee/it/kinds/<stem>.c` (statics and header inlines included) with the C's name and signature;
// callees of other TUs go through the file's generated `ext` hooks (unset: panics "TODO"). Checked by
// tests/kinds_<stem>.rs against the C (gen/kinds).
pub mod kinds_inl;
pub mod itarwinglaser;
pub mod itbat;
pub mod itcapsule;
pub mod itcerebi;
pub mod itchicorita;
pub mod itclimbersblizzard;
pub mod itbombhei;
pub mod itbox;
// Shard 745-752:
pub mod itmato;
pub mod itmetalb;
pub mod itmetamon;
pub mod itmew;
pub mod itmewtwodisable;
pub mod itmball;
pub mod itmsbomb;
pub mod itmewtwoshadowball;
