//! Step 7: VS boot to frame 0 on the engine (needs GW_ISO_VANILLA; vacuous without).
#[cfg(feature = "guard")]
mod guard_alloc;
#[cfg(feature = "guard")]
#[global_allocator]
static A: guard_alloc::Guard = guard_alloc::Guard;

use melee_engine::{gcm::Gcm, scenario::Scenario, Engine};
use melee_platform::PadSystem;

fn boot(scene: &str) -> Option<Engine> {
    let iso = std::env::var("GW_ISO_VANILLA").ok()?;
    let disc = Gcm::open(&iso).expect("disc");
    let scn = Scenario::parse(scene).expect("scene");
    Some(Engine::boot_vs(disc, 64 << 20, PadSystem::headless(), scn).expect("boot_vs"))
}

#[test]
fn falcon_fox_fd_boots() {
    let Some(mut e) = boot("mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off") else { return };
    eprintln!("booted");
    assert!(e.tick());
}

fn golden_rows(name: &str) -> Vec<Vec<u32>> {
    let p = format!("{}/../../docs/rust-port/golden/{}.csv", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(p).unwrap().lines().skip(1).map(|l| {
        l.split(',').enumerate().map(|(i, v)| if i == 0 || i == 3 { v.parse().unwrap() } else { u32::from_str_radix(v, 16).unwrap() }).collect()
    }).collect()
}

#[test]
fn falcon_fox_fd_frame0() {
    let Some(mut e) = boot("mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off") else { return };
    let g = golden_rows("pad_falcon_fox_fd");
    let want = &g[0];
    // frame 0 = the state after the first scene tick; the seed (and so the hash) waits for the particle generators (step 10)
    assert!(e.tick());
    let r = e.neutral_row();
    assert_eq!(&r[4..16], &want[5..17], "fighter words s0..s11");
    assert_eq!((r[2], r[3]), (want[3], want[4]), "items");
}
