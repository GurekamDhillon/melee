//! One module per ported C TU (docs/rust-port/game_tus.txt order shards).
//!
//! `ext_fns!` declares a TU's calls out of the TU: `ext::hook::<name>` overrides, unset = the
//! default (the port, or a TODO panic naming the C function).
macro_rules! ext_fns_c {
    ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? = $def:expr;)*) => {
        pub mod hook {
            #[allow(unused_imports)] use super::*;
            $( #[allow(non_upper_case_globals)] pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
        }
        $( #[allow(non_snake_case)] pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { hook::$name } { Some(f) => f($($a),*), None => { let d: fn($($t),*) $(-> $r)? = $def; d($($a),*) } }
        } )*
    };
}
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
pub mod adapt;
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
pub mod gmfrontend_atlas_toy;
pub mod gmmode_init;
pub mod gmscenelaunch;
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
pub(crate) use ext_fns_c;

pub mod gmtoygallery;
pub mod gmtoylottery;
pub mod globals;
pub mod gmtrainingmode;
pub mod gmvsmelee;
pub mod gmvsmode;
pub mod gm_16A2;
pub mod gm_1601;
pub mod gmvs;
pub mod gm_1A36;
pub mod gm_1B03;
pub mod gm_1ADD;
pub mod gmscmemcard;
pub mod gm_1A33;
pub mod gm_1A9B;
pub mod gm_1A7A;
pub mod gm_1A3F;
pub mod gmtitle;
pub mod gmtitlemode;
pub mod gmtou_0;
pub mod gmtou_1;
pub mod gmtou_2;
pub mod gmtoulib;
pub mod gmtoumode;
pub mod gmtoycollection;
pub mod gm_1BFA;
pub mod lblanguage;
pub mod lbgx;
pub mod lbext;
pub mod lbfile;
pub mod lbheap;
pub mod lbmemory;
pub mod mndeflicker;
pub mod mncount;
pub mod mngallery;
pub mod mninl;
pub mod mndiagram;
pub mod mndiagram2;
pub mod mndiagram3;
pub mod mnevent;
pub mod mndatadel;
pub mod lb_013B;
pub mod lb_0146;
pub mod lb_0192;
pub mod lb_0195;
pub mod lb_01F8;
pub mod lb_0219;
pub mod mncharsel;
pub mod lb_020A;
pub mod mnhyaku;
pub mod mninfo;
pub mod mninfobonus;
pub mod mnitemsw;
pub mod mnlanguage;
pub mod mnmain;
pub mod mnmainrule;
pub mod mnname;
pub mod mnsound;
pub mod mnstagesw;
pub mod mnruleplus;
pub mod mnsoundtest;
pub mod mnvibration;
pub mod mnsnap;
pub mod mnnamenew;
pub mod mnstagesel;
pub mod mn_22EC;
pub mod geno_lab_mode;
pub mod geno_game;
pub mod mex_tests;
pub mod geno_tests;
pub mod script_game;
