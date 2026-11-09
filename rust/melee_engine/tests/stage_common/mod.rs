//! Shared by stage_coll_fd.rs / stage_coll_bf.rs (one engine boot per process: a second boot in the same process crashes).
//! Step 9: the real stage collision on the engine (needs GW_ISO_VANILLA; vacuous without). A free `CollData` with a fixed ECB
//! box (`mpColl_8004730C`, no fighter, no joints) is dropped onto the stage that `StartMelee` loaded from the disc, so the
//! `mpLib` line data decoded from the DAT, `mpColl_80043754` stepping and the floor callback are exercised alone.
#[path = "../../../melee_core/test_support/disc.rs"]
mod disc;
use melee_engine::{gcm::Gcm, scenario::Scenario, Engine};
use melee_platform::PadSystem;
use melee_types::{CollData, ftCollisionBox, Vec2};
use melee_world::tu::mpcoll;

pub fn boot(stage: &str) -> Option<Engine> {
    let iso = disc::iso("stage_coll")?;
    let disc = Gcm::open(&iso).expect("disc");
    let scn = Scenario::parse(&format!("mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage={stage};match=time;minutes=0;items=off")).expect("scene");
    let mut e = Engine::boot_vs(disc, 64 << 20, PadSystem::headless(), scn).expect("boot_vs");
    for _ in 0..3 {
        assert!(e.tick());
    }
    Some(e)
}

/// Drop a 8-high, 4-wide box from (x, y0) to (x, y1) in one collision step; returns (cur y, floor line, env flags).
pub unsafe fn drop_box(x: f32, y0: f32, y1: f32) -> (f32, i32, i32) {
    let mut cd: CollData = core::mem::zeroed();
    mpcoll::mpColl_80041EE4(&mut cd);
    let mut bx = ftCollisionBox { top: 8.0, bottom: 0.0, left: Vec2 { x: -2.0, y: 4.0 }, right: Vec2 { x: 2.0, y: 4.0 } };
    // settle the ECB in the air first (the box becomes cd.ecb), then move
    cd.cur_pos.x = x;
    cd.cur_pos.y = y0;
    cd.last_pos = cd.cur_pos;
    cd.prev_pos = cd.cur_pos;
    mpcoll::mpColl_8004730C(&mut cd, &mut bx);
    cd.last_pos = cd.cur_pos;
    cd.cur_pos.y = y1;
    mpcoll::mpColl_8004730C(&mut cd, &mut bx);
    (cd.cur_pos.y, cd.floor.index, cd.env_flags)
}

