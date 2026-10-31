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
