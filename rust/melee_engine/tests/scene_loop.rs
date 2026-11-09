//! Step 6 smoke test: boot the engine on the disc (GW_ISO_VANILLA; test_support/disc.rs when it is missing) and tick
//! 60 frames through the real `gm_801A4D34` loop body, `HSD_GObj_RunProcs` and the pad feed, with no panic. It boots the
//! way the engine's runs do (`Engine::boot_vs`: main's mode init, the mode's load, the scene entry; Mario v Fox on
//! Battlefield, vs_boot covers Falcon v Fox on FD): the old empty-scene `Engine::boot` skipped the mode's load, and the
//! scene init's LbRb.dat no longer parsed there.
#[path = "../../melee_core/test_support/disc.rs"]
mod disc;
use melee_engine::{gcm::Gcm, scenario::Scenario, Engine};
use melee_platform::PadSystem;

#[test]
fn vs_scene_runs_60_frames() {
    let Some(iso) = disc::iso("scene_loop") else { return };
    let disc = Gcm::open(&iso).expect("disc");
    let scn = Scenario::parse("mode=vs;at=match;p1=mario/human;p2=fox/human;p3=none;p4=none;stage=battlefield;match=time;minutes=0;items=off")
        .expect("scene");
    let mut e = Engine::boot_vs(disc, 64 << 20, PadSystem::headless(), scn).expect("boot");
    let mut scenes = 0;
    for _ in 0..60 {
        if e.tick_modes() {
            scenes += 1;
        }
    }
    assert_eq!(e.frames, 60, "{scenes} scene changes");
    // while the first scene runs, its frame counter (gm_80479D58.unk_0) advances once per logic frame
    if scenes == 0 {
        assert_eq!(unsafe { melee_modes::tu::gmscene::gm_80479D58.unk_0 }, 60);
    }
}
