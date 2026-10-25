//! Yoshi's Story per-frame stage procs (`src/melee/gr/grstory.c`): Randall's puff timer
//! (`grStory_801E33E0` -> `grStory_801E366C`, map gobj 2) and the Shy Guys' spawn timer
//! (`grStory_801E3334` -> `grStory_801E3418`, map gobj 3).
//!
//! Both run from the first frame (the gobj procs of s_link 4; unlike Green Greens nothing waits for GO).
//! Not ported (each returns `Err("... not ported")` where reached): the Shy Guys themselves
//! (`it_802D8618`, `It_Kind_Heiho`), Randall's collision movement (`Ground_UpdateMapColl`: the moving
//! platform's lines stay where `mpLibLoad` put them) and the map animations of gobjs 1..3 (their joint
//! tracks; only the particle keys are run, by `dptcl`).

use super::*;

pub struct Story {
    stage: std::sync::Arc<DatArchive>,
    /// `gp->u.randall.timer` of map gobj 2.
    randall_timer: i32,
    /// `gp->u.randall.jobj` = `Ground_801C3FA4(gobj, 1)`: the stage joint the puff follows.
    randall_joint: Option<JObjRef>,
    /// `gp->u.shyguys.timer` of map gobj 3 (the C overwrites the drawn value with 120).
    shy_timer: i32,
    /// `gp->u.shyguys.pattern`.
    shy_pattern: i32,
    /// `grStory_YakumonoParam`: timer_min, timer_rand, spawnmany_rarity, vpos[6].
    yak: Vec<u32>,
}

impl Story {
    pub fn new(stage: std::sync::Arc<DatArchive>, joint_mtx: &mut HashMap<JObjRef, Mtx>) -> Result<Story, String> {
        let yak = crate::stage::yakumono_words(&stage, 9).ok_or("GrSt.dat: no yakumono_param")?;
        // grStory_801E3370: `Ground_801C3FA4(gobj, 1)`, the second JObj of Randall's tree.
        let randall_joint = crate::stage::map_joint_chain(&stage, 2, 1).map(|(j, chain)| {
            joint_mtx.insert(j, crate::stage::bind_pose_matrix(&stage.data, &chain));
            j
        });
        Ok(Story { stage, randall_timer: 0, randall_joint, shy_timer: 120, shy_pattern: 0, yak })
    }
}

impl VsMatch {
    /// The stage gobjs' procs at s_link 4: Randall (gobj 2), then the Shy Guys (gobj 3).
    pub(super) fn story_procs(&mut self) -> Result<(), String> {
        let Some(s) = &mut self.story else { return Ok(()) };
        // Map gobj 3 (created before gobj 2, so its proc runs first): grStory_801E3334 -> grStory_801E3418, no
        // spawn while a Shy Guy exists (none can yet).
        if s.shy_timer != 0 {
            s.shy_timer -= 1;
        } else {
            let _ = (&s.stage, &s.yak, s.shy_pattern);
            return Err("grStory_801E3418: the Shy Guys' spawn (it_802D8618, It_Kind_Heiho) is not ported".into());
        }
        // grStory_801E33E0 -> grStory_801E366C: `if (timer-- >= 0) return;`.
        let t = s.randall_timer;
        s.randall_timer -= 1;
        if t < 0 {
            if let Some(j) = s.randall_joint {
                // grLib_801C97DC(0x2C, 0, jobj) -> hsd_8039F6CC(0, 0, 0x2C, jobj).
                let mut host = Host { rand: &mut self.rand, joint_mtx: &self.joint_mtx, appsrt_hook_lists: &self.appsrt_hook_lists };
                self.ps.gen_create_on_jobj_once(&mut host, 0, 0, 0x2C, Some(j));
                s.randall_timer = self.rand.randi(20) + 10;
            }
        }
        Ok(())
    }
}
