//! The disc for the tests that need it (included with `#[path]`). A disc test used to return early and pass when
//! GW_ISO_VANILLA was unset, so a run without the disc reported them green: archive_host and scene_loop were broken for
//! weeks that way. Now a missing disc fails the test and says so, unless MELEE_SKIP_DISC_TESTS=1 opts out, which
//! prints a SKIPPED line (`cargo test -- --nocapture` shows it; tools/rust-port/oracle_tests.sh warns up front).
#![allow(dead_code)]

/// The disc image path, or None when the run opted out of disc tests (MELEE_SKIP_DISC_TESTS=1). Panics otherwise.
pub fn iso(test: &str) -> Option<String> {
    if let Ok(p) = std::env::var("GW_ISO_VANILLA") {
        return Some(p);
    }
    if std::env::var_os("MELEE_SKIP_DISC_TESTS").is_some() {
        eprintln!("SKIPPED {test}: no disc (GW_ISO_VANILLA unset, MELEE_SKIP_DISC_TESTS=1)");
        return None;
    }
    panic!("{test} needs the disc: set GW_ISO_VANILLA to the vanilla 1.02 ISO, or MELEE_SKIP_DISC_TESTS=1 to skip it");
}
