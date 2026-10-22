//! One module per ported C TU (docs/rust-port/game_tus.txt order shards).
//!
//! `ext_fns!` declares a TU's calls out of the TU: `ext::hook::<name>` overrides, unset = the
//! default (the port, or a TODO panic naming the C function).
macro_rules! ext_fns {
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
pub(crate) use ext_fns;

pub mod gmtoygallery;
pub mod gmtoylottery;
pub mod globals;
pub mod gmtrainingmode;
pub mod gmvsmelee;
pub mod gmvsmode;
pub mod gm_16A2;
