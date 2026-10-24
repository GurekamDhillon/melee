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

pub mod itegg;
pub mod itentei;
pub mod itevyoshiegg;
pub mod itfflower;
pub mod itfflowerflame;
pub mod itfoods;
pub mod itflipper;
pub mod itfire;
pub mod itgamewatchfire;
pub mod itgamewatchgreenhouse;
pub mod itgamewatchjudge;
pub mod itgamewatchmanhole;
pub mod itgamewatchpanic;
pub mod itgamewatchparachute;
pub mod itgamewatchrescue;
pub mod itgamewatchturtle;

// Shard 897-904: whole-TU native ports of the vi (visual scenes) and ty/tylist TUs.
pub mod vi;
pub mod vi0102;
pub mod vi0401;
pub mod vi0402;
pub mod vi0501;
pub mod vi0502;
pub mod vi0601;
pub mod tylist;
