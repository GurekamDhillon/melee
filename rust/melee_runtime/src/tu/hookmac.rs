#![allow(unused_macros, unused_imports)]
//! The hook macros of the `ext` modules (callees outside the ported TU). One line per hook, so
//! gen/oracle/oracle.py can read the signatures: `hook!(STATIC, c_name, (a: T, ..) -> R);` is a
//! TODO stub (an unset hook panics), `hook_or!(STATIC, c_name, (a: T, ..) -> R, port);` runs the
//! native port `port` when unset. The oracle installs a logging closure into every hook.

macro_rules! hook {
    ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?) => {
        pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
        #[allow(non_snake_case)]
        pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { $hook } {
                Some(f) => f($($a),*),
                None => panic!(concat!("TODO: ", stringify!($name), " is not ported")),
            }
        }
    };
}

macro_rules! hook_or {
    ($hook:ident, $name:ident, ($($a:ident: $t:ty),*) $(-> $r:ty)?, $port:expr) => {
        pub static mut $hook: Option<fn($($t),*) $(-> $r)?> = None;
        #[allow(non_snake_case)]
        pub fn $name($($a: $t),*) $(-> $r)? {
            match unsafe { $hook } {
                Some(f) => f($($a),*),
                None => $port($($a),*),
            }
        }
    };
}

pub(crate) use hook;
pub(crate) use hook_or;
