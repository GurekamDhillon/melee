//! Step 6 smoke test: boot the engine on the disc (GW_ISO_VANILLA; without it the test passes vacuously) and tick 60 frames of an empty scene (no fighters, no stage)
//! through the real `gm_801A4D34` loop body, `HSD_GObj_RunProcs` and the pad feed, with no panic.
use melee_engine::{gcm::Gcm, Engine};
use melee_platform::PadSystem;

#[test]
fn empty_scene_runs_60_frames() {
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else { return };
    let disc = Gcm::open(&iso).expect("disc");
    let mut e = Engine::boot(disc, 64 << 20, PadSystem::headless()).expect("boot");
    for i in 0..60 {
        assert!(e.tick(), "scene ended at frame {i}");
    }
    assert_eq!(e.frames, 60);
    // the scene's frame counter (gm_80479D58.unk_0) advanced once per logic frame
    assert_eq!(unsafe { melee_modes::tu::gmscene::gm_80479D58.unk_0 }, 60);
}
