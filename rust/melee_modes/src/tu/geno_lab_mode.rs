#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_variables, unused_imports, clippy::all)]
//! Native port of pc/geno/geno_lab_mode.c. Unsafe entry points require live native game records and serialized access to the TU state.
use melee_types::*;
pub use melee_types::FlyCursor;
use core::ffi::c_void;
use core::ptr::{null_mut,null};
pub const GM_LAB:u32=GM_COUNT+2;
pub static mut lab_vs:VsModeData=unsafe {core::mem::zeroed()};
pub static mut lab_in_match:i32=0;
pub static mut lab_next:i32=0;
pub static mut fly_speed:f32=2.;
pub static mut fly_solid:i32=0;
pub static mut fly_cursor:[FlyCursor;6]=unsafe {core::mem::zeroed()};
pub static mut hold_on:[i32;6]=[0;6];
pub static mut hold_action:[i32;6]=[0;6];
pub static mut hold_frame:[i32;6]=[0;6];
pub static mut hold_frozen:[i32;6]=[0;6];
pub static mut hold_tick:[i32;6]=[0;6];
pub static mut hold_mask:[i32;6]=[0;6];
pub static mut hold_hits:[i32;6]=[0;6];
/// Out-of-TU calls: existing ports by default, named TODOs where integration is pending.
/// Hooks let the C oracle observe the same dependency boundary and side effects.
pub mod ext {
    use super::*;
    use crate::tu::ext_fns_c;
    ext_fns_c! {
        Fighter_ChangeMotionState(a0: *mut HSD_GObj, a1: i32, a2: u32, a3: f32, a4: f32, a5: f32, a6: *mut HSD_GObj) = |a0, a1, a2, a3, a4, a5, a6| unsafe { melee_fighter::tu::fighter::Fighter_ChangeMotionState(a0, a1, a2, a3, a4, a5, a6) };
        Netplay_Enabled() -> i32 = || todo!("TODO(port): Netplay_Enabled");
        Player_GetEntity(a0: i32) -> *mut HSD_GObj = |a0| melee_fighter::player::get_entity(a0);
        RB_Enabled() -> i32 = || todo!("TODO(port): RB_Enabled");
        Replay_Active() -> i32 = || todo!("TODO(port): Replay_Active");
        SceneLaunch_BootGameMode() -> i32 = || todo!("TODO(port): SceneLaunch_BootGameMode");
        SceneLaunch_ItemFreq() -> i32 = || todo!("TODO(port): SceneLaunch_ItemFreq");
        SceneLaunch_SeedPlayers(a0: *mut VsModeData, a1: i32) -> i32 = |a0, a1| todo!("TODO(port): SceneLaunch_SeedPlayers");
        SceneLaunch_SeedVs(a0: *mut VsModeData, a1: i32) = |a0, a1| todo!("TODO(port): SceneLaunch_SeedVs");
        ScriptGame_HitRuleCreate(a0: *mut Fighter, a1: *mut HitCapsule, a2: i32) = |a0, a1, a2| todo!("TODO(port): ScriptGame_HitRuleCreate");
        Script_LabRequest() = || todo!("TODO(port): Script_LabRequest");
        ftAnim_IsFramesRemaining(a0: *mut HSD_GObj) -> i32 = |a0| unsafe { melee_fighter::tu::ftanim::ftAnim_IsFramesRemaining(a0) as i32 };
        ftAnim_SetAnimRate(a0: *mut HSD_GObj, a1: f32) = |a0, a1| unsafe { melee_fighter::tu::ftanim::ftAnim_SetAnimRate(a0, a1) };
        ftCo_Fall_Enter(a0: *mut HSD_GObj) = |a0| melee_fighter::tu::ftCo_Fall::ftCo_Fall_Enter(a0);
        gmFrontend_BeginLoading() = || unsafe { crate::tu::gmfrontend::gmFrontend_BeginLoading() };
        gmFrontend_ModeSelect(a0: *mut GameModeState, a1: i32, a2: *const i8) = |a0, a1, a2| unsafe { crate::tu::gmfrontend::gmFrontend_ModeSelect(a0, a1, a2) };
        gmVsMelee_EnterCss(a0: *mut GameModeState, a1: *mut VsModeData, a2: u32) = |a0, a1, a2| unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterCss(a0, a1, a2) };
        gmVsMelee_EnterSss(a0: *mut GameModeState, a1: *mut VsModeData) = |a0, a1| unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterSss(a0, a1) };
        gmVsMelee_EnterVs(a0: *mut GameModeState, a1: *mut VsModeData, a2: Option<unsafe extern "C" fn(*mut StartMeleeData, *mut StartMeleeData)>, a3: Option<unsafe extern "C" fn(*mut PlayerInitData, *mut PlayerInitData)>) = |a0, a1, a2, a3| unsafe { crate::tu::gmvsmelee::gmVsMelee_EnterVs(a0, a1, a2, a3) };
        gmVsMelee_ExitCss(a0: *mut GameModeState, a1: *mut VsModeData) = |a0, a1| unsafe { crate::tu::gmvsmelee::gmVsMelee_ExitCss(a0, a1) };
        gmVsMelee_ExitSss(a0: *mut GameModeState, a1: *mut VsModeData, a2: u8) = |a0, a1, a2| unsafe { crate::tu::gmvsmelee::gmVsMelee_ExitSss(a0, a1, a2) };
        gmVs_EndMatch(a0: i32) = |a0| unsafe { crate::tu::gmvs::gmVs_EndMatch(a0) };
        gm_ChangeGameModeAfterCurrentScene(a0: i32) = |a0| crate::tu::gm_1A3F::gm_ChangeGameModeAfterCurrentScene(a0);
        gm_GetCurrentGameMode() -> u8 = || crate::tu::gm_1A3F::gm_GetCurrentGameMode();
        gm_GetGameModeStateEnterData(a0: *mut GameModeState) -> *mut c_void = |a0| unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateEnterData(a0) };
        gm_GetGameModeStateExitData(a0: *mut GameModeState) -> *mut c_void = |a0| unsafe { crate::tu::gm_1A3F::gm_GetGameModeStateExitData(a0) };
        gm_GetNumCostumesForCKind(a0: u8) -> u8 = |a0| unsafe { crate::tu::gm_1601::gm_GetNumCostumesForCKind(a0) };
        gm_InitVsMode(a0: *mut VsModeData) = |a0| unsafe { crate::tu::gm_1601::gm_InitVsMode(a0) };
        gm_SetNextGameModeStateId(a0: u8) = |a0| crate::tu::gm_1A3F::gm_SetNextGameModeStateId(a0);
        gm_SetupAllPlayerDefaults(a0: *mut PlayerInitData) = |a0| unsafe { crate::tu::gm_1601::gm_SetupAllPlayerDefaults(a0) };
        lbColl_80008440(a0: *mut HitCapsule) = |a0| unsafe { crate::lb_ptr::lbColl_80008440(&mut *a0) };
        mpCheckFloor(a0: f32, a1: f32, a2: f32, a3: f32, a4: f32, a5: *mut Vec, a6: *mut i32, a7: *mut u32, a8: *mut Vec, a9: i32, a10: i32, a11: i32, a12: Option<unsafe extern "C" fn(*mut HSD_GObj, i32) -> i32>, a13: *mut HSD_GObj) -> i32 = |a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13| todo!("TODO(port): mpCheckFloor");
        mpColl_80043680(a0: *mut CollData, a1: *mut Vec) = |a0, a1| todo!("TODO(port): mpColl_80043680");
        plStale_IncrementAttackInstance() -> u16 = || melee_fighter::stale::increment_attack_instance();
    }
    pub static mut HSD_GObjPLinkHead: *mut *mut HSD_GObj = null_mut();
    pub use crate::tu::gmvsmelee::{gmVsMelee_CssData, gmVsMelee_SssData, gmVsMelee_StartData, gmVsMelee_VsExitInfo};
    pub fn OSReport(message: &str) { eprint!("{message}"); }
}

pub const GENO_LAB_TO_CSS: i32 = 0;
pub const GENO_LAB_TO_SSS: i32 = 1;
pub const GENO_LAB_TO_MENU: i32 = 2;
pub const GENO_LAB_TO_MATCH: i32 = 3;
pub const GENO_FLY_OFF: i32 = 0;
pub const GENO_FLY_ON: i32 = 1;
pub const GENO_FLY_PLACE: i32 = 2;
const LAB_STATE_CSS: u8 = 0;
const LAB_STATE_SSS: u8 = 1;
const LAB_STATE_MATCH: u8 = 2;
const LAB_STATE_LOADING: u8 = 3;

pub static mut gm_Mode_Lab_States: [GameModeState; 5] = {
    use super::support::{state, end};
    [
        state(LAB_STATE_CSS, 3, Some(lab_enter_css), Some(lab_exit_css), GS_CSS,
              (&raw mut ext::gmVsMelee_CssData).cast(), (&raw mut ext::gmVsMelee_CssData).cast()),
        state(LAB_STATE_SSS, 3, Some(lab_enter_sss), Some(lab_exit_sss), GS_SSS,
              (&raw mut ext::gmVsMelee_SssData).cast(), (&raw mut ext::gmVsMelee_SssData).cast()),
        state(LAB_STATE_MATCH, 3, Some(lab_enter_match), Some(lab_exit_match), GS_VS,
              (&raw mut ext::gmVsMelee_StartData).cast(), (&raw mut ext::gmVsMelee_VsExitInfo).cast()),
        state(LAB_STATE_LOADING, 3, Some(lab_enter_loading), Some(lab_exit_loading), GS_COUNT + 1, null_mut(), null_mut()),
        end(),
    ]
};

pub unsafe extern "C" fn lab_enter_css(state: *mut GameModeState) {
    ext::gmVsMelee_EnterCss(state, &raw mut lab_vs, VS_MELEE);
    ext::gmFrontend_ModeSelect(state, 0, c"LAB".as_ptr());
}
pub unsafe extern "C" fn lab_exit_css(state: *mut GameModeState) {
    ext::gmVsMelee_ExitCss(state, &raw mut lab_vs);
}
pub unsafe extern "C" fn lab_enter_sss(state: *mut GameModeState) {
    ext::gmVsMelee_EnterSss(state, &raw mut lab_vs);
    ext::gmFrontend_ModeSelect(state, 1, c"LAB".as_ptr());
}
pub unsafe extern "C" fn lab_exit_sss(state: *mut GameModeState) {
    ext::gmVsMelee_ExitSss(state, &raw mut lab_vs, LAB_STATE_CSS);
    if (*(ext::gm_GetGameModeStateExitData(state) as *mut SSSData)).start_game != 0 {
        ext::gm_SetNextGameModeStateId(LAB_STATE_LOADING);
    }
}
pub unsafe extern "C" fn lab_enter_loading(_: *mut GameModeState) { ext::gmFrontend_BeginLoading(); }
pub unsafe extern "C" fn lab_exit_loading(_: *mut GameModeState) { ext::gm_SetNextGameModeStateId(LAB_STATE_MATCH); }

pub unsafe extern "C" fn GenoLab_ApplyRules(start: *mut StartMeleeData) {
    let rules = &mut (*start).rules;
    rules.set_match_kind(MatchKind_Time as _);
    rules.set_is_stock(0);
    rules.set_timer_enabled(0);
    rules.set_timer_counts_up(0);
    rules.time_limit = 0;
    // The C evaluates the getter twice when items are configured.
    rules.item_freq = if ext::SceneLaunch_ItemFreq() >= 0 { ext::SceneLaunch_ItemFreq() as i8 } else { -1 };
    rules.set_disable_pausing(1);
    rules.set_x5_0(0);
    rules.set_x4_2(0);
    for player in &mut (*start).players { player.stocks = 0; }
}
pub unsafe extern "C" fn lab_start_cb(start: *mut StartMeleeData, _: *mut StartMeleeData) { GenoLab_ApplyRules(start); }
pub unsafe extern "C" fn lab_player_cb(player: *mut PlayerInitData, _: *mut PlayerInitData) { (*player).stocks = 0; }
pub unsafe extern "C" fn lab_enter_match(state: *mut GameModeState) {
    ext::gmVsMelee_EnterVs(state, &raw mut lab_vs, Some(lab_start_cb), Some(lab_player_cb));
    // EnterVs copies players after the callbacks, so stocks must be cleared again.
    GenoLab_ApplyRules(ext::gm_GetGameModeStateEnterData(state).cast());
    lab_in_match = 1;
    lab_next = GENO_LAB_TO_CSS;
    ext::Script_LabRequest();
    ext::OSReport("geno lab: LAB match - no timer, no stocks, infinite respawn\n");
}
pub unsafe extern "C" fn GenoLab_NextAfterMatch() -> i32 { lab_next }
pub unsafe extern "C" fn lab_exit_match(_: *mut GameModeState) {
    lab_in_match = 0;
    if (lab_vs.start.players[4].slot_type != Gm_PKind_NA as u8 || lab_vs.start.players[5].slot_type != Gm_PKind_NA as u8)
        && lab_next != GENO_LAB_TO_MATCH && lab_next != GENO_LAB_TO_MENU {
        ext::OSReport("geno lab: six-slot match cannot return through CSS/SSS; going to menus\n");
        lab_next = GENO_LAB_TO_MENU;
    }
    match lab_next {
        GENO_LAB_TO_SSS => ext::gm_SetNextGameModeStateId(LAB_STATE_SSS),
        GENO_LAB_TO_MENU => ext::gm_ChangeGameModeAfterCurrentScene(GM_MENU as i32),
        GENO_LAB_TO_MATCH => ext::gm_SetNextGameModeStateId(LAB_STATE_LOADING),
        _ => ext::gm_SetNextGameModeStateId(LAB_STATE_CSS),
    }
    ext::OSReport(&format!("geno lab: LAB match over -> {}\n", match lab_next {
        GENO_LAB_TO_SSS => "stage select", GENO_LAB_TO_MENU => "menus",
        GENO_LAB_TO_MATCH => "the same match", _ => "character select",
    }));
    lab_next = GENO_LAB_TO_CSS;
}
pub unsafe extern "C" fn gm_Mode_Lab_OnInit() { ext::gm_InitVsMode(&raw mut lab_vs); }
pub unsafe extern "C" fn gm_Mode_Lab_OnLoad() {
    lab_in_match = 0;
    lab_next = GENO_LAB_TO_CSS;
    ext::Script_LabRequest();
    if ext::SceneLaunch_BootGameMode() == GM_LAB as i32 { ext::SceneLaunch_SeedVs(&raw mut lab_vs, 0); }
}
pub unsafe extern "C" fn GenoLab_ModeActive() -> i32 { (ext::gm_GetCurrentGameMode() as u32 == GM_LAB) as i32 }
pub unsafe extern "C" fn GenoLab_InMatch() -> i32 { (ext::gm_GetCurrentGameMode() as u32 == GM_LAB && lab_in_match != 0) as i32 }
pub unsafe extern "C" fn GenoLab_Leave(r#where: i32) -> i32 {
    if ext::gm_GetCurrentGameMode() as u32 != GM_LAB || lab_in_match == 0 { return 0; }
    if (lab_vs.start.players[4].slot_type != Gm_PKind_NA as u8 || lab_vs.start.players[5].slot_type != Gm_PKind_NA as u8)
        && r#where != GENO_LAB_TO_MATCH && r#where != GENO_LAB_TO_MENU {
        ext::OSReport("geno lab: six-slot selection refused; restart the match or leave to menus\n");
        return 0;
    }
    lab_next = match r#where { GENO_LAB_TO_SSS | GENO_LAB_TO_MENU | GENO_LAB_TO_MATCH => r#where, _ => GENO_LAB_TO_CSS };
    ext::gmVs_EndMatch(OUTCOME_NO_CONTEST as i32);
    1
}

pub unsafe extern "C" fn fly_boundary_blocked(netplay: i32, rollback: i32, replay: i32) -> i32 {
    (netplay != 0 || rollback != 0 || replay != 0) as i32
}
pub unsafe extern "C" fn GenoFly_OfflineAllowed() -> i32 {
    (fly_boundary_blocked(ext::Netplay_Enabled(), ext::RB_Enabled(), ext::Replay_Active()) == 0) as i32
}
pub unsafe extern "C" fn fly_cursor_for(fp: *mut Fighter) -> *mut FlyCursor {
    let slot = (*fp).player_id as usize;
    if slot < 6 && fly_cursor[slot].fighter == fp { &raw mut fly_cursor[slot] } else { null_mut() }
}
pub unsafe extern "C" fn fly_cursor_pulse(fp: *mut Fighter, c: *mut FlyCursor) {
    for h in &mut (*fp).x914 { h.state = HitCapsule_Disabled; }
    let h = &raw mut (*fp).x914[0];
    // A new attack instance must erase BOTH victim histories and all flags.
    core::ptr::write_bytes(h, 0, 1);
    (*h).state = HitCapsule_Enabled;
    (*h).damage = (*c).damage as f32;
    (*h).unk_count = (*c).damage as u32;
    (*h).scale = (*c).radius;
    (*h).kb_angle = if (*c).every_frame != 0 { 45 } else { (*c).angle };
    (*h).x24 = if (*c).every_frame != 0 { 70 } else { (*c).kbg as u32 };
    (*h).x2C = if (*c).every_frame != 0 { 20 } else { (*c).kb_base as u32 };
    (*h).set_x40_b0(1); (*h).set_x40_b2(1); (*h).set_x40_b3(1);
    (*h).set_x42_b5(1); (*h).set_x42_b7(1);
    (*h).b_offset = (*fp).cur_pos;
    (*h).x4C = (*fp).cur_pos;
    (*h).x58 = (*fp).cur_pos;
    ext::ScriptGame_HitRuleCreate(fp, h, 0);
    (*fp).x206C_attack_instance = ext::plStale_IncrementAttackInstance();
    if (*c).pulses < i32::MAX { (*c).pulses += 1; }
}
pub unsafe extern "C" fn fly_cursor_step(fp: *mut Fighter, c: *mut FlyCursor) {
    let cycle = (*c).active.wrapping_add((*c).gap);
    if (*c).every_frame != 0 || (*c).active < 1 || cycle < 1 { fly_cursor_pulse(fp,c); return; }
    if (*c).phase >= cycle { (*c).phase = 0; }
    if (*c).phase == 0 { fly_cursor_pulse(fp,c); }
    else if (*c).phase == (*c).active { (*fp).x914[0].state = HitCapsule_Disabled; }
    (*c).phase = (*c).phase.wrapping_add(1);
}
pub unsafe extern "C" fn fly_anim(gobj: *mut HSD_GObj) {
    let fp = (*gobj).user_data as *mut Fighter;
    if (*fp).motion_id != ftCo_MS_Wait as i32 || ext::ftAnim_IsFramesRemaining(gobj) != 0 { return; }
    ext::Fighter_ChangeMotionState(gobj, ftCo_MS_Wait as i32, 0, 0., 1., 0., null_mut());
    hold_callbacks(fp);
}
pub unsafe extern "C" fn fly_input(_: *mut HSD_GObj) {}
pub unsafe extern "C" fn fly_phys(gobj: *mut HSD_GObj) {
    let fp = (*gobj).user_data as *mut Fighter;
    let mut sx = (*fp).input.lstick[0].x;
    let mut sy = (*fp).input.lstick[0].y;
    let mut v = fly_speed;
    let cursor = fly_cursor_for(fp);
    if GenoFly_OfflineAllowed() == 0 { GenoFly_Watch(); return; }
    if (*fp).input.held_buttons[0] & 0x100 != 0 { v *= 0.25; }
    if (*fp).input.held_buttons[0] & 0x200 != 0 { v *= 4.; }
    if sx < 0.1 && sx > -0.1 { sx = 0.; }
    if sy < 0.1 && sy > -0.1 { sy = 0.; }
    (*fp).self_vel.x = sx * v;
    (*fp).self_vel.y = sy * v;
    if !cursor.is_null() && (*cursor).target != 0 {
        let dx = (*cursor).x - (*fp).cur_pos.x;
        let dy = (*cursor).y - (*fp).cur_pos.y;
        // Shipping PPC IR contracts dx*dx + dy*dy, with the left multiply fused.
        let distance = melee_runtime::gekko::sqrtf(melee_core::fp::fmadds(dx, dx, dy * dy));
        let fraction = if distance > fly_speed { fly_speed / distance } else { 1. };
        (*fp).self_vel.x = dx * fraction;
        (*fp).self_vel.y = dy * fraction;
    }
    (*fp).self_vel.z = 0.;
    (*fp).x8c_kb_vel = Vec3::zeroed();
    if sx > 0. { (*fp).facing_dir = 1.; } else if sx < 0. { (*fp).facing_dir = -1.; }
    (*fp).dmg.x1910 = 0;
    (*fp).x1988 = if fly_solid != 0 { 0 } else { 2 };
    // Pulse before hold_step: holding clears the attacker's hitlag.
    if !cursor.is_null() && (*cursor).attack != 0 && (*fp).dmg.x195c_hitlag_frames <= 0. { fly_cursor_step(fp, cursor); }
    hold_step(gobj);
}
pub unsafe extern "C" fn fly_coll(gobj: *mut HSD_GObj) {
    let fp = (*gobj).user_data as *mut Fighter;
    let cursor = fly_cursor_for(fp);
    ext::mpColl_80043680(&raw mut (*fp).coll_data, &raw mut (*fp).cur_pos);
    (*fp).coll_data.env_flags = 0;
    if !cursor.is_null() && (*cursor).attack != 0 && (*fp).x914[0].state != HitCapsule_Disabled {
        (*fp).x914[0].b_offset = (*fp).cur_pos;
        (*fp).x914[0].x4C = (*fp).cur_pos;
        (*fp).x914[0].x58 = (*fp).cur_pos;
    }
}
pub unsafe extern "C" fn hold_callbacks(fp: *mut Fighter) {
    (*fp).anim_cb=Some(fly_anim); (*fp).input_cb=Some(fly_input);
    (*fp).phys_cb=Some(fly_phys); (*fp).coll_cb=Some(fly_coll);
}
pub unsafe extern "C" fn hold_live(fp: *mut Fighter) -> i32 {
    (*fp).x914.iter().any(|h| h.state == HitCapsule_Enabled) as i32
}
pub unsafe extern "C" fn hold_step(gobj: *mut HSD_GObj) {
    let fp = (*gobj).user_data as *mut Fighter;
    let slot=(*fp).player_id as usize;
    if slot>=6 || hold_on[slot]==0 { return; }
    if (*fp).motion_id!=hold_action[slot] || hold_frozen[slot]<0 {
        ext::Fighter_ChangeMotionState(gobj,hold_action[slot],0,0.,1.,0.,null_mut());
        hold_callbacks(fp); hold_frozen[slot]=0; hold_tick[slot]=0; return;
    }
    if hold_frozen[slot]==0 {
        let mut mask=0;
        for i in 0..4 { if (*fp).x914[i].state==HitCapsule_Enabled { mask |= 1<<i; } }
        if mask!=0 && (*fp).cur_anim_frame>=hold_frame[slot] as f32 {
            hold_frozen[slot]=1; hold_mask[slot]=mask;
            ext::OSReport(&format!("geno hold: P{} frozen in motion {} at frame {:.1}, hitboxes {:x}\n",slot+1,(*fp).motion_id,(*fp).cur_anim_frame,mask));
        } else if ext::ftAnim_IsFramesRemaining(gobj)==0 { hold_frozen[slot]=-1; return; }
    }
    if hold_frozen[slot]==1 {
        ext::ftAnim_SetAnimRate(gobj,0.);
        (*fp).dmg.x195c_hitlag_frames=0.;
        for i in 0..4 { if hold_mask[slot] & (1<<i)!=0 && (*fp).x914[i].state!=HitCapsule_Enabled { (*fp).x914[i].state=HitCapsule_Enabled; } }
        hold_tick[slot]=hold_tick[slot].wrapping_add(1);
        if hold_tick[slot]%8==0 {
            hold_hits[slot]=hold_hits[slot].wrapping_add(1);
            for i in 0..4 { ext::lbColl_80008440(&raw mut (*fp).x914[i]); }
        }
    }
}
pub unsafe extern "C" fn GenoFly_HoldHitbox(slot:i32,on:i32,action:i32,frame:i32)->i32 {
    let gobj=fly_gobj(slot);
    if gobj.is_null() { return -1; }
    let fp=(*gobj).user_data as *mut Fighter;
    if GenoFly_Fighter(fp)==0 { return -3; }
    let slot=slot as usize;
    if on==0 {
        if hold_on[slot]!=0 {
            hold_on[slot]=0;
            ext::Fighter_ChangeMotionState(gobj,ftCo_MS_Wait as i32,0,0.,1.,0.,null_mut());
            hold_callbacks(fp); (*fp).self_vel=Vec3::zeroed();
        }
        return 0;
    }
    const CODES:[i32;6]=[ftCo_MS_AttackAirN,ftCo_MS_AttackAirF,ftCo_MS_AttackAirB,ftCo_MS_AttackAirHi,ftCo_MS_AttackAirLw,ftCo_MS_Attack11];
    hold_on[slot]=1; hold_hits[slot]=0;
    hold_action[slot]=if (0..6).contains(&action) { CODES[action as usize] as i32 } else { action };
    hold_frame[slot]=frame; hold_tick[slot]=0; hold_frozen[slot]=-1;
    0
}
pub unsafe extern "C" fn GenoFly_HoldHits(slot:i32)->i32 { if (0..6).contains(&slot) {hold_hits[slot as usize]} else {0} }
pub unsafe extern "C" fn GenoFly_Holding(slot:i32)->i32 { ((0..6).contains(&slot) && hold_on[slot as usize]!=0) as i32 }
pub unsafe extern "C" fn GenoFly_Fighter(fp:*mut Fighter)->i32 {
    (!fp.is_null() && (*fp).phys_cb.is_some_and(|f| core::ptr::fn_addr_eq(f,fly_phys as unsafe extern "C" fn(*mut HSD_GObj)))) as i32
}
pub unsafe extern "C" fn fly_gobj(slot:i32)->*mut HSD_GObj {
    if !(0..6).contains(&slot) || ext::HSD_GObjPLinkHead.is_null() { return null_mut(); }
    let want=ext::Player_GetEntity(slot);
    let mut g=*ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    while !g.is_null() { if g==want {return g;} g=(*g).next; }
    null_mut()
}
pub unsafe extern "C" fn GenoFly_Clear(slot:i32) {
    if !(0..6).contains(&slot) { return; }
    let c=&raw mut fly_cursor[slot as usize];
    let gobj=fly_gobj(slot);
    // Stale cursors must not disable hitboxes owned by a replacement action.
    if (*c).attack!=0 && !gobj.is_null() && (*gobj).user_data.cast::<Fighter>()==(*c).fighter && GenoFly_Fighter((*c).fighter)!=0 {
        (*(*c).fighter).x914[0].state=HitCapsule_Disabled;
    }
    core::ptr::write_bytes(c,0,1);
}
pub unsafe extern "C" fn GenoFly_Reset() { for i in 0..6 {GenoFly_Clear(i);} }
pub unsafe extern "C" fn fly_cursor_actor(slot:i32)->*mut Fighter {
    let g=fly_gobj(slot);
    let fp=if g.is_null(){null_mut()}else{(*g).user_data as *mut Fighter};
    if !fp.is_null() && (*fp).is_sub_fighter()==0 && GenoFly_Fighter(fp)!=0 {fp}else{null_mut()}
}
pub unsafe extern "C" fn GenoFly_Target(slot:i32,x_bits:i32,y_bits:i32)->i32 {
    let fp=fly_cursor_actor(slot);
    let x=f32::from_bits(x_bits as u32); let y=f32::from_bits(y_bits as u32);
    if fp.is_null(){return -1;}
    if !(x>=-49000. && x<=49000. && y>=-49000. && y<=49000.){return -2;}
    let c=&mut fly_cursor[slot as usize]; c.fighter=fp; c.target=1; c.x=x; c.y=y;
    0
}
pub unsafe extern "C" fn GenoFly_AttackConfig(slot:i32,flags:i32,damage:i32,radius_bits:i32,active_gap:i32,kb_kbg:i32,angle:i32)->i32 {
    let fp=fly_cursor_actor(slot);
    let active=active_gap&0xffff; let gap=(active_gap>>16)&0xffff;
    let kb=kb_kbg&0xffff; let kbg=(kb_kbg>>16)&0xffff;
    if !(0..6).contains(&slot){return -1;}
    let c=&mut fly_cursor[slot as usize];
    if flags&1==0 {
        if !fp.is_null() && c.fighter==fp && c.attack!=0 {(*fp).x914[0].state=HitCapsule_Disabled;}
        c.attack=0; return 0;
    }
    if fp.is_null(){return -1;}
    let radius=f32::from_bits(radius_bits as u32);
    if damage<1 || damage>30 || !(radius>=1. && radius<=30.){return -2;}
    if active>600 || gap>3000 || kb>2000 || kbg>2000 || angle<0 || angle>361 {return -2;}
    c.fighter=fp; c.attack=1; c.damage=damage; c.radius=radius; c.every_frame=(flags>>1)&1;
    c.active=if active!=0{active}else{3}; c.gap=if active!=0 || gap!=0{gap}else{24};
    c.kb_base=if kb!=0{kb}else{30}; c.kbg=if kbg!=0{kbg}else{100}; c.angle=if angle!=0{angle}else{45};
    c.phase=0; 0
}
pub unsafe extern "C" fn GenoFly_AttackSet(slot:i32,on:i32,damage:i32,radius_bits:i32)->i32 {GenoFly_AttackConfig(slot,(on!=0) as i32,damage,radius_bits,0,0,0)}
pub unsafe extern "C" fn GenoFly_State(slot:i32,field:i32)->i32 {
    if !(0..6).contains(&slot){return -1;}
    if fly_cursor_actor(slot).is_null(){return 0;}
    let c=&raw const fly_cursor[slot as usize];
    if (*c).fighter!=fly_cursor_actor(slot){return 0;}
    match field {0=>(*c).attack,1=>(*c).target,2=>(*c).damage,3=>(*c).radius.to_bits() as i32,4=>(*c).x.to_bits() as i32,5=>(*c).y.to_bits() as i32,
        6=>(*c).pulses,7=>(*c).every_frame,8=>(*c).active,9=>(*c).gap,10=>(*c).kb_base,11=>(*c).kbg,12=>(*c).angle,13=>(*c).phase,_=>-1}
}
pub unsafe extern "C" fn GenoFly_Pulses(slot:i32)->i32 {GenoFly_State(slot,6)}
pub unsafe extern "C" fn fly_refused(fp:*mut Fighter)->i32 {
    if (*fp).x221F_b3()!=0 || (*fp).kind==Ft_Kind_MasterH || (*fp).kind==Ft_Kind_CrezyH {return 1;}
    let motion=(*fp).motion_id;
    for (lo,hi) in [(ftCo_MS_DeadDown,ftCo_MS_RebirthWait),(ftCo_MS_CapturePulledHi,ftCo_MS_CaptureFoot),(ftCo_MS_ThrownF,ftCo_MS_ThrownlwWomen),(ftCo_MS_ShoulderedWait,ftCo_MS_ThrownCrazyHand)] {
        if motion>=lo as i32 && motion<=hi as i32 {return 1;}
    }
    (!(*fp).victim_gobj.is_null() || !(*fp).x1A5C.is_null()) as i32
}
pub unsafe extern "C" fn fly_drop(gobj:*mut HSD_GObj) {
    let fp=(*gobj).user_data as *mut Fighter;
    (*fp).self_vel=Vec3::zeroed(); (*fp).x8c_kb_vel=Vec3::zeroed(); (*fp).prev_pos=(*fp).cur_pos;
    ext::mpColl_80043680(&raw mut (*fp).coll_data,&raw mut (*fp).cur_pos);
    ext::ftCo_Fall_Enter(gobj); (*fp).x1988=0;
}
pub unsafe extern "C" fn fly_boundary_fighter(gobj:*mut HSD_GObj,blocked:i32,drop:Option<unsafe extern "C" fn(*mut HSD_GObj)>)->i32 {
    let fp=(*gobj).user_data as *mut Fighter;
    if blocked==0 || GenoFly_Fighter(fp)==0{return 0;}
    let c=fly_cursor_for(fp);
    if !c.is_null(){if (*c).attack!=0{(*fp).x914[0].state=HitCapsule_Disabled;} core::ptr::write_bytes(c,0,1);}
    drop.expect("fly boundary drop callback")(gobj); 1
}
pub unsafe extern "C" fn GenoFly_Watch()->i32 {
    if GenoFly_OfflineAllowed()!=0{return 0;}
    if !ext::HSD_GObjPLinkHead.is_null(){
        let mut g=*ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
        while !g.is_null(){fly_boundary_fighter(g,1,Some(fly_drop));g=(*g).next;}
    }
    GenoFly_Reset(); 1
}
pub unsafe extern "C" fn GenoFly_Set(slot:i32,mode:i32)->i32 {
    let gobj=fly_gobj(slot); if gobj.is_null(){return -1;}
    let fp=(*gobj).user_data as *mut Fighter;
    if mode==GENO_FLY_ON {
        if GenoFly_Fighter(fp)!=0{return 0;}
        GenoFly_Clear(slot); if fly_refused(fp)!=0{return -2;}
        fly_drop(gobj);
        if (*fp).ground_or_air==GA_Air {
            ext::Fighter_ChangeMotionState(gobj,ftCo_MS_Wait as i32,0,0.,1.,0.,null_mut());
            (*fp).self_vel=Vec3::zeroed();
        }
        hold_callbacks(fp); if fly_solid==0{(*fp).x1988=2;} return 0;
    }
    GenoFly_Clear(slot); if GenoFly_Fighter(fp)==0{return 0;}
    hold_on[slot as usize]=0;
    if mode==GENO_FLY_PLACE {
        let mut floor=Vec3::zeroed();
        if ext::mpCheckFloor((*fp).cur_pos.x,(*fp).cur_pos.y,(*fp).cur_pos.x,(*fp).cur_pos.y-100000.,0.,&mut floor,null_mut(),null_mut(),null_mut(),-1,-1,-1,None,null_mut())!=0 {
            (*fp).cur_pos.x=floor.x; (*fp).cur_pos.y=floor.y+0.5; fly_drop(gobj); return 0;
        }
        fly_drop(gobj); return 1;
    }
    fly_drop(gobj); 0
}
pub unsafe extern "C" fn GenoFly_Get(slot:i32)->i32 {let g=fly_gobj(slot); if g.is_null(){-1}else{GenoFly_Fighter((*g).user_data.cast())}}
pub unsafe extern "C" fn GenoFly_Any()->i32 {
    if ext::HSD_GObjPLinkHead.is_null(){return 0;}
    let mut g=*ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    while !g.is_null(){if GenoFly_Fighter((*g).user_data.cast())!=0{return 1;}g=(*g).next;}
    0
}
pub unsafe extern "C" fn GenoFly_CameraSubject(subject:*mut c_void)->i32 {
    if ext::HSD_GObjPLinkHead.is_null(){return 0;}
    let mut g=*ext::HSD_GObjPLinkHead.add(HSD_GOBJ_PLINK_FIGHTER as usize);
    while !g.is_null(){let fp=(*g).user_data as *mut Fighter; if GenoFly_Fighter(fp)!=0 && (*fp).x890_cameraBox.cast::<c_void>()==subject{return 1;}g=(*g).next;}
    0
}
pub unsafe extern "C" fn GenoFly_Teleport(slot:i32,x_bits:i32,y_bits:i32)->i32 {
    let g=fly_gobj(slot); if g.is_null(){return -1;}
    let fp=(*g).user_data as *mut Fighter;
    if GenoFly_Fighter(fp)==0 && fly_refused(fp)!=0{return -2;}
    (*fp).cur_pos.x=f32::from_bits(x_bits as u32); (*fp).cur_pos.y=f32::from_bits(y_bits as u32); (*fp).prev_pos=(*fp).cur_pos;
    if GenoFly_Fighter(fp)!=0 {ext::mpColl_80043680(&raw mut (*fp).coll_data,&raw mut (*fp).cur_pos);}else{fly_drop(g);}
    0
}
pub unsafe extern "C" fn GenoFly_SetSpeed(speed_bits:i32) {let s=f32::from_bits(speed_bits as u32); fly_speed=if s<0.05{0.05}else if s>200.{200.}else{s};}
pub unsafe extern "C" fn GenoFly_Speed()->f32 {fly_speed}
pub unsafe extern "C" fn GenoFly_SetSolid(solid:i32) {fly_solid=(solid!=0) as i32;}
pub unsafe extern "C" fn GenoFly_Solid()->i32 {fly_solid}
pub unsafe extern "C" fn GenoFly_TestArm(fp:*mut Fighter) {hold_callbacks(fp);}
pub unsafe extern "C" fn fly_boundary_test_drop(gobj:*mut HSD_GObj) {let fp=(*gobj).user_data as *mut Fighter; (*fp).phys_cb=None;(*fp).x1988=0;}

/// The C headless cursor fixture, including its saved/restored TU state.
pub unsafe extern "C" fn GenoFly_CursorTest() -> i32 {
    static mut fp: Fighter = unsafe { core::mem::zeroed() };
    static mut gobj: HSD_GObj = unsafe { core::mem::zeroed() };
    let saved=fly_cursor[0]; let speed=fly_speed; let solid=fly_solid;
    let mut rc=0;
    core::ptr::write_bytes(&raw mut fp,0,1); core::ptr::write_bytes(&raw mut gobj,0,1);
    gobj.user_data=(&raw mut fp).cast(); fp.player_id=0;
    GenoFly_TestArm(&raw mut fp); fly_speed=2.; fly_solid=1;
    core::ptr::write_bytes(&raw mut fly_cursor[0],0,1);
    fly_cursor[0].fighter=&raw mut fp; fly_cursor[0].target=1;
    fly_cursor[0].x=3.; fly_cursor[0].y=4.; fly_cursor[0].attack=1;
    fly_cursor[0].damage=3; fly_cursor[0].radius=6.; fly_cursor[0].every_frame=1;
    fp.x914[0].x44=12; fp.x914[0].x45=12;
    fly_phys(&raw mut gobj);
    if fp.self_vel.x<1.199 || fp.self_vel.x>1.201 || fp.self_vel.y<1.599 || fp.self_vel.y>1.601 || fp.x1988!=0
        || fp.x914[0].x44!=0 || fp.x914[0].x45!=0 || fp.x914[0].damage!=3. || fp.x914[0].scale!=6.
        || fp.x914[0].x42_b5()==0 || fp.x914[0].x42_b7()==0 || fp.x914[0].x2C!=20 {rc=1;}
    let mut instance=fp.x206C_attack_instance;
    fp.cur_pos.x+=fp.self_vel.x; fp.cur_pos.y+=fp.self_vel.y;
    fp.x914[0].x44=1; fp.x914[0].x45=1;
    fp.x914[0].victims_1[0].victim=(&raw mut gobj).cast(); fp.x914[0].victims_2[0].victim=(&raw mut gobj).cast();
    fly_coll(&raw mut gobj);
    if fp.x914[0].b_offset.x!=fp.cur_pos.x || fp.x914[0].b_offset.y!=fp.cur_pos.y || fp.x914[0].x4C.x!=fp.cur_pos.x || fp.x914[0].x58.y!=fp.cur_pos.y
        || fp.x206C_attack_instance!=instance || fly_cursor[0].pulses!=1 || fp.x914[0].x44!=1 || fp.x914[0].x45!=1
        || fp.x914[0].victims_1[0].victim!=(&raw mut gobj).cast() || fp.x914[0].victims_2[0].victim!=(&raw mut gobj).cast() {rc=1;}
    fp.x914[0].x44=1; fp.x914[0].x45=1;
    fp.x914[0].victims_1[0].victim=(&raw mut gobj).cast(); fp.x914[0].victims_2[0].victim=(&raw mut gobj).cast();
    fp.cur_pos.x=2.9; fp.cur_pos.y=3.9; fly_phys(&raw mut gobj);
    if fp.x206C_attack_instance==instance || fp.x914[0].x44!=0 || fp.x914[0].x45!=0 || !fp.x914[0].victims_1[0].victim.is_null() || !fp.x914[0].victims_2[0].victim.is_null()
        || fp.self_vel.x<0.099 || fp.self_vel.x>0.101 || fp.self_vel.y<0.099 || fp.self_vel.y>0.101 || fly_cursor[0].pulses!=2 {rc=1;}
    fp.dmg.x195c_hitlag_frames=2.; fly_phys(&raw mut gobj);
    if fly_cursor[0].pulses!=2{rc=1;}
    instance=fp.x206C_attack_instance; fp.x914[0].x44=1; fp.x914[0].x45=1;
    fp.x914[0].victims_1[0].victim=(&raw mut gobj).cast(); fp.x914[0].victims_2[0].victim=(&raw mut gobj).cast();
    fp.cur_pos.x=10.; fp.cur_pos.y=20.; fly_coll(&raw mut gobj);
    if fp.x914[0].b_offset.x!=10. || fp.x914[0].x4C.y!=20. || fp.x914[0].x58.y!=20. || fp.x206C_attack_instance!=instance || fly_cursor[0].pulses!=2
        || fp.x914[0].x44!=1 || fp.x914[0].x45!=1 || fp.x914[0].victims_1[0].victim!=(&raw mut gobj).cast() || fp.x914[0].victims_2[0].victim!=(&raw mut gobj).cast(){rc=1;}
    if fly_boundary_blocked(0,0,0)!=0 || fly_boundary_blocked(1,0,0)==0 || fly_boundary_blocked(0,1,0)==0 || fly_boundary_blocked(0,0,1)==0{rc=1;}
    if fly_boundary_fighter(&raw mut gobj,0,Some(fly_boundary_test_drop))!=0 || GenoFly_Fighter(&raw mut fp)==0 || fly_cursor[0].attack==0 || fp.x914[0].state==HitCapsule_Disabled{rc=1;}
    if fly_boundary_fighter(&raw mut gobj,1,Some(fly_boundary_test_drop))==0 || GenoFly_Fighter(&raw mut fp)!=0 || fp.x1988!=0 || fly_cursor[0].attack!=0 || fly_cursor[0].target!=0 || fp.x914[0].state!=HitCapsule_Disabled{rc=1;}
    GenoFly_TestArm(&raw mut fp);
    fly_cursor[0].fighter=null_mut(); fp.x914[0].state=HitCapsule_Disabled; fp.dmg.x195c_hitlag_frames=0.; fly_phys(&raw mut gobj);
    if fp.x914[0].state!=HitCapsule_Disabled{rc=1;}
    fp.dmg.x195c_hitlag_frames=0.; core::ptr::write_bytes(&raw mut fly_cursor[0],0,1);
    fly_cursor[0].fighter=&raw mut fp; fly_cursor[0].attack=1; fly_cursor[0].damage=3; fly_cursor[0].radius=6.;
    fly_cursor[0].active=2; fly_cursor[0].gap=3; fly_cursor[0].kb_base=33; fly_cursor[0].kbg=77; fly_cursor[0].angle=55;
    let mut pulses0=fly_cursor[0].pulses; let mut enabled_frames=0; let mut pulse_frames=[0;16]; let mut np=0;
    for k in 0..12 {
        let before=fly_cursor[0].pulses; fly_phys(&raw mut gobj);
        if fly_cursor[0].pulses!=before && np<16{pulse_frames[np]=k;np+=1;}
        if fp.x914[0].state==HitCapsule_Enabled{enabled_frames+=1;}
        if k==0 && (fp.x914[0].x2C!=33 || fp.x914[0].x24!=77 || fp.x914[0].kb_angle!=55){rc=1;}
        if k==1{fp.dmg.x195c_hitlag_frames=2.;fly_phys(&raw mut gobj);fp.dmg.x195c_hitlag_frames=0.;}
    }
    if np!=3 || pulse_frames[0]!=0 || pulse_frames[1]!=5 || pulse_frames[2]!=10 || enabled_frames!=6 || fly_cursor[0].pulses-pulses0!=3{rc=1;}
    fly_cursor[0].every_frame=1; pulses0=fly_cursor[0].pulses;
    for _ in 0..4{fly_phys(&raw mut gobj);}
    if fly_cursor[0].pulses-pulses0!=4{rc=1;}
    fly_cursor[0]=saved; fly_speed=speed; fly_solid=solid;
    rc
}

pub unsafe extern "C" fn GenoLab_SixSeedTest(six:i32)->i32 {
    static mut fixture:VsModeData=unsafe{core::mem::zeroed()};
    core::ptr::write_bytes(&raw mut fixture,0,1);
    ext::gm_SetupAllPlayerDefaults((&raw mut fixture.start.players).cast());
    if six==0 {
        fixture.start.players[4].ckind=2; fixture.start.players[5].ckind=2;
        fixture.start.players[4].slot_type=Gm_PKind_Cpu as u8; fixture.start.players[5].slot_type=Gm_PKind_Cpu as u8;
        ext::SceneLaunch_SeedPlayers(&raw mut fixture,0);
        return (fixture.start.players[4].ckind!=ChKind_None as i8 || fixture.start.players[5].ckind!=ChKind_None as i8
            || fixture.start.players[4].slot_type!=Gm_PKind_NA as u8 || fixture.start.players[5].slot_type!=Gm_PKind_NA as u8) as i32;
    }
    if ext::SceneLaunch_SeedPlayers(&raw mut fixture,0)!=6{return 1;}
    for i in 0..6 {
        let p=&fixture.start.players[i];
        if p.slot!=i as u8+1 || p.ckind!=2 || p.slot_type!=if i!=0{Gm_PKind_Cpu}else{Gm_PKind_Human} as u8
            || p.team!=if i!=0{1}else{0} || p.stocks!=i as i8+1 || p.color>=ext::gm_GetNumCostumesForCKind(2)
            || (i>=4 && (p.spawn_pos!= -1 || p.attack_ratio!=1. || p.defense_ratio!=1.)){return 1;}
    }
    0
}
