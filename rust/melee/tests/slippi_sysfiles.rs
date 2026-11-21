//! Disc-gated, network-using check of the Slippi system-file layer (plan step 4): fetch Slippi's public files (hash
//! checked), build the 13 menu files from the player's disc and print each size + fnv1a (Hero's log line format).
//! Run: `GW_ISO_VANILLA=... cargo test -p melee --test slippi_sysfiles -- --ignored --nocapture`.
//! Nothing it reads or builds is written outside the cache dir (a temp dir unless MELEE_SLIPPI_SYS_DIR is set).

#[test]
#[ignore = "needs the player's disc and network"]
fn builds_all_system_files_from_disc() {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        eprintln!("skip: GW_ISO_VANILLA not set");
        return;
    };
    let cache = std::env::var_os("MELEE_SLIPPI_SYS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("melee-slippi-sys-test"));
    let mut gcm = melee_engine::gcm::Gcm::open(&iso).expect("open disc");
    let mut disc = |n: &str| gcm.read(n);
    let mut fetch = |u: &str| melee_data::sysfiles::fetch_curl(u);
    let s = melee_data::sysfiles::SysFiles::build(&cache, &mut disc, &mut fetch).expect("build");
    for (n, size, h) in &s.built {
        println!("slippi menus: /{} {size} {h:08x}", n.to_ascii_lowercase());
    }
    assert!(s.len() >= 12);
}
