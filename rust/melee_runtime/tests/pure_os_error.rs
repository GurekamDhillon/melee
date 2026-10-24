//! `tu::os_error::OSReport` (the text it prints) vs the C `OSReport` (= `vprintf`) on the formats the
//! game uses, with random arguments. Regenerate the expected data with
//! `python rust/melee_runtime/gen/pure/run_pure.py os_error`.
mod pure_common;
use melee_runtime::tu::os_error::os_report_text;
use pure_common::*;
use std::ffi::CString;

#[test]
fn os_report_matches_c() {
    let strs: [CString; 4] = [
        CString::new("alpha").unwrap(),
        CString::new("Beta gamma").unwrap(),
        CString::new("").unwrap(),
        CString::new("a-much-longer-string").unwrap(),
    ];
    let mut r = Rng::new();
    let mut out = String::new();
    let mut put = |out: &mut String, fmt: &str, va: &[usize], tag: String| {
        let f = CString::new(fmt).unwrap();
        let t = unsafe { os_report_text(f.as_ptr(), va) };
        out.push_str(&String::from_utf8_lossy(&t));
        out.push_str(&format!("@@ {tag}\n"));
    };
    for k in 0..200 {
        let a = r.u() as u32 as i32;
        let b = (r.u() % 100000) as i32 - 50000;
        let u = r.u() as u32;
        let d = r.f() as f64;
        let e = r.f() as f64 * 1000.0;
        let s = strs[(r.u() % 4) as usize].as_ptr() as usize;
        let c = (b'a' + (r.u() % 26) as u8) as usize;
        let (ai, bi, ui) = (a as u32 as usize, b as u32 as usize, u as usize);
        let (db, eb) = (d.to_bits() as usize, e.to_bits() as usize);
        put(&mut out, "%s:%d: couldn t get gobj(id=%d)\n", &[s, bi, ai], format!("{k}.1"));
        put(&mut out, "talk no = %d\n", &[ai], format!("{k}.2"));
        put(&mut out, "*** Beam Damage! %f\n", &[db], format!("{k}.3"));
        put(&mut out, "pos = %f : tgrpos = %f\n", &[db, eb], format!("{k}.4"));
        put(&mut out, "oioi... %08x\n", &[ui], format!("{k}.5"));
        put(&mut out, "%3d|%-5d|%+d|%05d\n", &[bi, bi, bi, bi], format!("{k}.6"));
        put(&mut out, "%x %X %o %u %c\n", &[ui, ui, ui, ui, c], format!("{k}.7"));
        put(&mut out, "%.2f %8.3f %5.1f\n", &[db, db, eb], format!("{k}.8"));
        put(&mut out, "%s|%10s|%-10s|%.3s\n", &[s, s, s, s], format!("{k}.9"));
        put(&mut out, "no conversions here\n", &[], format!("{k}.10"));
    }
    compare("os_error", &out);
}
