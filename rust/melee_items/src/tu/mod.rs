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

// Shard 649-656 (generator copy under gen/mtu649).
pub mod itmaterial;
pub mod itspawn;
pub mod itzako;
pub mod it_26B1;
pub mod it_2725;
pub mod it_279C;
pub mod it_3F14;
pub mod it_3F2F;
pub mod itfoxblaster;
pub mod itfoxillusion;
pub mod itfoxlaser;
pub mod itfreeze;
pub mod itfreezer;
pub mod itfushigibana;
pub mod itgamewatchbreath;
pub mod itgamewatchchef;

// Shard 697-704.

pub mod inl;
pub mod itheart;
pub mod itgreatfoxlaser;
pub mod ithammerhead;
pub mod ithammer;
pub mod itharisen;
pub mod itgshell;
pub mod ithassam;
pub mod itheiho;
