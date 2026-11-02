//! C string and string-record materialisation for the generated retail tables (`retail.rs`): the tu ports keep file
//! names and costume strings as Rust `&str` consts; the fighter data tables hold NUL terminated `char*` and records of
//! them. Each call leaks one copy (the tables live for the whole run).

use melee_types::{Fighter_CostumeStrings, Fighter_DemoStrings};

/// A leaked, NUL terminated copy of `s`.
pub fn cstr(s: &str) -> *mut i8 {
    let mut v = Vec::with_capacity(s.len() + 1);
    v.extend_from_slice(s.as_bytes());
    v.push(0);
    Box::leak(v.into_boxed_slice()).as_mut_ptr() as *mut i8
}

/// `Fighter_CostumeStrings[]`: `{dat, joint}` or `{dat, joint, matanim}` per costume.
pub fn costume_strings<const N: usize>(rows: &[[&str; N]]) -> *mut Fighter_CostumeStrings {
    let mut v: Vec<Fighter_CostumeStrings> = Vec::with_capacity(rows.len());
    for r in rows {
        let mut c = Fighter_CostumeStrings::zeroed();
        c.dat_filename = cstr(r[0]);
        c.joint_name = cstr(r[1]);
        if N > 2 {
            c.matanim_joint_name = cstr(r[2]);
        }
        v.push(c);
    }
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}

/// `Fighter_DemoStrings`: result, intro, ending, vi-wait file names.
pub fn demo_strings<const N: usize>(names: &[&str; N]) -> *mut Fighter_DemoStrings {
    let mut d = Fighter_DemoStrings::zeroed();
    d.result_filename = cstr(names[0]);
    d.intro_filename = cstr(names[1]);
    d.ending_filename = cstr(names[2]);
    d.vi_wait_filename = cstr(names[3]);
    Box::leak(Box::new(d))
}

/// `cstr` of an optional string: `None` is the C initialiser's NULL.
fn cstr_opt(s: Option<&str>) -> *mut i8 {
    s.map_or(core::ptr::null_mut(), cstr)
}

/// `costume_strings` for rows with NULL members (`{ dat, joint, NULL }`: Master Hand, the wireframes).
pub fn costume_strings_opt<const N: usize>(rows: &[[Option<&str>; N]]) -> *mut Fighter_CostumeStrings {
    let mut v: Vec<Fighter_CostumeStrings> = Vec::with_capacity(rows.len());
    for r in rows {
        let mut c = Fighter_CostumeStrings::zeroed();
        c.dat_filename = cstr_opt(r[0]);
        c.joint_name = cstr_opt(r[1]);
        if N > 2 {
            c.matanim_joint_name = cstr_opt(r[2]);
        }
        v.push(c);
    }
    Box::leak(v.into_boxed_slice()).as_mut_ptr()
}

/// `demo_strings` with NULL members (`Fighter_DemoStrings ftGk_Init_DemoMotionFilenames = { 0 }`).
pub fn demo_strings_opt<const N: usize>(names: &[Option<&str>; N]) -> *mut Fighter_DemoStrings {
    let mut d = Fighter_DemoStrings::zeroed();
    d.result_filename = cstr_opt(names[0]);
    d.intro_filename = cstr_opt(names[1]);
    d.ending_filename = cstr_opt(names[2]);
    d.vi_wait_filename = cstr_opt(names[3]);
    Box::leak(Box::new(d))
}
