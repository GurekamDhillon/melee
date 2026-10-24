//! The respawn after a KO: `ftCo_800BFD9C` (Dead -> Sleep) -> `gm_80167320` -> `fn_8016719C`
//! (the player's respawn data) -> `Player_80032070` -> `ftCo_800D4FF4` (Rebirth), then Rebirth /
//! RebirthWait (ft_0D4D.c) and the random idle animation (`ftCo_8008A7A8`).
//!
//! Not modelled: the respawn platform's model (`x20A0_accessory` and its animation), the fighter
//! camera box, the colour animation (`ftCo_800BFFD0`), the intangibility window
//! (`ftColl_8007B7A4`) and the HUD (`ifStatus_802F6508`): none is hashed, none draws.

use super::*;

pub const FTCO_MS_SLEEP: i32 = 11;
pub const FTCO_MS_REBIRTH: i32 = 12;
pub const FTCO_MS_REBIRTH_WAIT: i32 = 13;
/// `ftCo_SM_Wait1_0`: the animation row Rebirth / RebirthWait play.
const FTCO_SM_WAIT1_0: u32 = 2;

/// `StaticPlayer` data the respawn path reads and writes.
#[derive(Clone, Copy, Debug)]
pub struct PlayerRespawn {
    /// `player_poses.byIndex[0]` (`Player_80032768` / `Player_LoadPlayerCoords`).
    pub coords: Vec3,
    pub facing: f32,
    /// `Player_SetSomePos`.
    pub some_pos: Vec3,
    /// `unk4E` (`Player_80032FA4` / `Player_80032F30`).
    pub unk4e: i32,
    /// `mv.co.common.x0` (the Rebirth / RebirthWait countdown) and `x4` (its target).
    pub timer: i32,
    pub target: Vec3,
    /// `Player_SetSpawnPlatformPos`.
    pub platform: Vec3,
}

impl Default for PlayerRespawn {
    fn default() -> Self {
        let z = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        // StaticPlayer.unk4E starts at -1 (no stage-specific respawn point).
        PlayerRespawn { coords: z, facing: 0.0, some_pos: z, unk4e: -1, timer: 0, target: z, platform: z }
    }
}

/// What the respawn needs from the match.
pub struct RespawnCtx<'a> {
    pub ground: &'a mut GroundColl,
    pub cg: &'a mut CollGlobals,
    pub rand: &'a mut HsdRand,
    /// `Stage_80224E38(&v, n)` = the stage's point `n + 4`.
    pub respawn_points: &'a [Vec3],
    /// `Stage_GetCamBoundsTopOffset()`.
    pub cam_top: f32,
    /// `scene_state->fighters[i].x8`: the respawn spots' countdowns (`fn_8016758C`).
    pub busy: &'a mut [i32; 6],
    pub start_shield: f32,
}

fn stage_point(ctx: &RespawnCtx<'_>, n: i32) -> Result<Vec3, String> {
    ctx.respawn_points.get(n as usize).copied().ok_or(format!("Stage_80224E38({n}): no such respawn point"))
}

/// `ftCo_800BFD9C`: DeadDown ran out; become Sleep, then respawn.
pub fn dead_over(p: &mut Player, ctx: &mut RespawnCtx<'_>) -> Result<(), String> {
    if p.fp.x2222_b5() != 0 {
        return Err("ftCo_800BFD9C: the follower of a pair not ported".into());
    }
    // ftCo_800BFD04: ChangeMotionState(Sleep) + invisible, inactive camera box, x221F_b3 / b1.
    change_motion_state(p, FTCO_MS_SLEEP, None, ctx.rand)?;
    {
        let fp = &mut *p.fp;
        fp.set_invisible(1);
        fp.set_x221E_b1(1);
        fp.set_x221E_b2(1);
        fp.set_x2219_b1(1);
        fp.set_x221F_b3(1);
        fp.set_x221F_b1(1);
    }
    // gm_80167320(player_id, is_sub_fighter): a time match, so no stock check -> fn_8016719C.
    // (`Player_GetFlagsBit1` clear; the 1P / event branches are not this match.)
    fn_8016719c(p, ctx)
}

/// `fn_8016719C(slot, subchar)` (gm_1601.c) up to `Player_80032070`.
fn fn_8016719c(p: &mut Player, ctx: &mut RespawnCtx<'_>) -> Result<(), String> {
    // fn_80167638: get_idx() = the first free respawn spot; Stage_80224E38(&pos, 0); offset.x = 16 * sp[idx].
    let idx = ctx.busy.iter().position(|&b| b == 0).unwrap_or(0);
    let sp: [f32; 6] = [0.0, 1.0, -1.0, 2.0, 0.0, 0.0];
    let mut respawn = stage_point(ctx, 0)?;
    let offset = Vec3 { x: 16.0 * sp[idx], y: 0.0, z: 0.0 };
    ctx.busy[idx] = 0x90;
    respawn.x += offset.x;
    // Player_SetSpawnPlatformPos; stage_info.unk8C.b5 is set on Final Destination, so
    // Player_80032FA4 / Player_SetSomePos are skipped (unk4E stays -1).
    p.rb.platform = respawn;
    // respawn.y = Stage_GetCamBoundsTopOffset(); Player_80032768; Player_SetFacingDirection.
    p.rb.coords = Vec3 { x: respawn.x, y: ctx.cam_top, z: respawn.z };
    p.rb.facing = if respawn.x >= 0.0 { -1.0 } else { 1.0 };
    // Player_80032070(slot, 0) -> ftCo_800D4FF4.
    rebirth_enter(p, ctx)
}

/// `Fighter_UnkProcessDeath_80068354` (the parts a hash or this sim's state can see).
fn fighter_unk_process_death(p: &mut Player, ctx: &mut RespawnCtx<'_>) -> Result<(), String> {
    let rb = p.rb;
    {
        let fp = &mut *p.fp;
        // Fighter_UnkInitReset_80067C98.
        fp.facing_dir = rb.facing;
        let x = melee_core::fp::fmadds(fp.facing_dir, fp.x40 * fp.x34_scale.x, rb.coords.x);
        fp.cur_pos = melee_types::Vec { x, y: rb.coords.y, z: rb.coords.z };
        fp.prev_pos = fp.cur_pos;
        fp.facing_dir1 = fp.facing_dir;
        fp.x34_scale.y = fp.x34_scale.x;
        fp.pos_delta = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.cur_anim_frame = 0.0;
        fp.x898_unk = 0.0;
        fp.frame_speed_mul = 1.0;
        fp.x8A0_unk = 1.0;
        fp.dmg.kb_applied = 0.0;
        fp.dmg.x18A4_knockbackMagnitude = 0.0;
        fp.dmg.x18A8 = 0.0;
        fp.dmg.x18ac_time_since_hit = -1;
        fp.dmg.armor0 = 0.0;
        fp.dmg.armor1 = 0.0;
        fp.x1968_jumpsUsed = 0;
        fp.x1969_walljumpUsed = 0;
        fp.hitlag_mul = 0.0;
        fp.x2064_ledgeCooldown = 0;
        // dmg.x1830_percent = Player_GetDamage(player_id): the player's damage was reset by the KO.
        fp.dmg.x1830_percent = 0.0;
        fp.dmg.x1838_percentTemp = 0.0;
        fp.dmg.x18c4_source_ply = 6;
        fp.dmg.x18C8 = -1;
        fp.dmg.x195c_hitlag_frames = 0.0;
        fp.dmg.x1954 = 0.0;
        fp.dmg.x189C_unk_num_frames = 0.0;
        fp.set_x2219_b1(0);
        fp.set_x2219_b2(0);
        fp.set_x2219_b3(0);
        fp.set_x2219_b4(0);
        fp.set_x2219_b5(0);
        fp.set_x2219_b6(0);
        fp.set_x2219_b7(0);
        fp.set_allow_sdi(0);
        fp.set_x221A_b1(0);
        fp.set_x221A_b3(0);
        fp.xF8_playerNudgeVel = melee_types::Vec2 { x: 0.0, y: 0.0 };
        fp.x100 = -1.0;
        fp.set_fall_fast(0);
        fp.set_x2219_b0(0);
        fp.anon_2210.throw_flags = 0;
        fp.item_gobj = core::ptr::null_mut();
        fp.x197C = core::ptr::null_mut();
        fp.x1980 = core::ptr::null_mut();
        fp.victim_gobj = core::ptr::null_mut();
        fp.shield_health = ctx.start_shield;
        fp.set_x221A_b7(0);
        fp.set_x221C_b1(0);
        fp.set_x221C_b2(0);
        fp.set_x221C_b3(0);
        fp.x19A0_shieldDamageTaken = 0;
        fp.x19A4 = 0;
        fp.lightshield_amount = 0.0;
        fp.set_reflecting(0);
        fp.x68C_transNPos = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.x6A4_transNOffset = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.lstick_angle = 0.0;
        fp.x6C0 = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.x6D8 = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.wall_jump_input_timer = 254;
        fp.x2180 = 6;
        fp.set_x2224_b2(0);
        fp.set_x2224_b3(0);
        fp.set_is_metal(0);
        // ftCommon_8007E2FC: every velocity cleared.
        fp.xE4_ground_accel_1 = 0.0;
        fp.xE8_ground_accel_2 = 0.0;
        fp.x74_self_accel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.gr_vel = 0.0;
        fp.self_vel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.xF0_ground_kb_vel = 0.0;
        fp.x8c_kb_vel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.xF4_ground_attacker_shield_kb_vel = 0.0;
        fp.x98_atk_shield_kb = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
        fp.xD4_unk_vel = melee_types::Vec { x: 0.0, y: 0.0, z: 0.0 };
    }
    p.hitboxes = [None; 4];
    p.guard = GuardVars::default();
    // The skeleton follows cur_pos (HSD_JObjSetTranslate).
    let root = p.skel.root;
    let c = p.fp.cur_pos;
    p.skel.set_translate(root, Vec3 { x: c.x, y: c.y, z: c.z });
    // ft_80081B38: CollData init with the bone ECB source.
    let ecb_desc = p.ecb_desc;
    {
        let fp = &mut *p.fp;
        fp.ecb_lock = 0;
        fp.coll_data.cur_pos = fp.cur_pos;
        fp.coll_data.last_pos = fp.coll_data.cur_pos;
        mc::mp_coll_80041ee4(&mut fp.coll_data, ctx.cg);
        fp.coll_data.x34_flags.set_b1234(1);
        mc::mp_coll_set_ecb_source_jobj(&mut fp.coll_data, ecb_desc.unk_c * fp.x34_scale.y);
        let t = fp.x34_scale.y;
        mc::mp_coll_set_ledge_snap(&mut fp.coll_data, ecb_desc.ledge_snap_x * t, ecb_desc.ledge_snap_y * t, ecb_desc.ledge_snap_height * t);
        fp.coll_data.x50 = fp.co_attrs.weight;
        let v = 10.0f32 * fp.x34_scale.y;
        fp.coll_data.ecb_source.x128 = v;
        fp.coll_data.ecb_source.x12C = v;
    }
    // smash_attrs.x2135 == -1 (never set before the first respawn): ft_80082A68, the floor probe
    // 10 units either side of the new position; a floor there is a grounded start.
    if p.fp.smash_attrs.x2135 == -1 {
        let fp = &mut *p.fp;
        fp.coll_data.cur_pos = fp.cur_pos;
        fp.coll_data.last_pos = fp.coll_data.cur_pos;
        fp.coll_data.last_pos.y += 10.0;
        fp.coll_data.cur_pos.y -= 10.0;
        let joints = ecb_joints(&mut p.skel, &ecb_desc);
        let fp = &mut *p.fp;
        let hit = mc::mp_coll_800471f8(ctx.ground, ctx.cg, &mut fp.coll_data, &joints).map_err(|e| format!("respawn probe: {} not ported", e.0))?;
        if hit {
            return Err("respawn probe found a floor: ftCommon_8007D6A4 (grounded start) not wired".into());
        }
    }
    melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp);
    // ftCo_800A101C(fp, Player_GetCpuType, Player_GetCpuLevel, 0): two HSD_Randf.
    {
        let slot = p.slot;
        let cpu_type = melee_fighter::player::get_cpu_type(slot);
        let cpu_level = melee_fighter::player::get_cpu_level(slot);
        melee_fighter::ftcpu::ftco_800a101c(&mut p.fp, cpu_type, cpu_level, 0, None, None, &mut || ctx.rand.randf());
    }
    p.ef_queue.clear();
    Ok(())
}

/// `ftCo_800D4FF4`: Rebirth.
fn rebirth_enter(p: &mut Player, ctx: &mut RespawnCtx<'_>) -> Result<(), String> {
    fighter_unk_process_death(p, ctx)?;
    p.fp.smash_attrs.x2135 = p.rb.unk4e as i8;
    if p.fp.smash_attrs.x2135 == -1 {
        // Player_GetSpawnPlatformPos: the target the fighter eases to (mv.co.turn.facing_after, x8).
        let sp44 = p.rb.platform;
        let fp = &*p.fp;
        let x = melee_core::fp::fmadds(fp.facing_dir, fp.x40 * fp.x34_scale.x, sp44.x);
        p.rb.target = Vec3 { x, y: sp44.y, z: 0.0 };
    } else {
        return Err("ftCo_800D4FF4: a stage-specific respawn point (x2135 != -1) not ported".into());
    }
    melee_fighter::ftcommon2::ftcommon_8007d5d4(&mut p.fp);
    p.rb.timer = melee_fighter::hooks::common_data().rebirth_countdown;
    let row = load_row(p, FTCO_SM_WAIT1_0)?;
    p.keep_gfx = true;
    p.blend_zero = true;
    change_motion_state(p, FTCO_MS_REBIRTH, row.as_ref(), ctx.rand)?;
    let fp = &mut *p.fp;
    fp.set_x221E_b2(1);
    fp.set_x2219_b1(1);
    fp.set_x221E_b1(1);
    fp.set_x2222_b7(1);
    fp.set_x221D_b5(1);
    // ftCommon_SetAccessory (the platform) is not modelled; ftCommon_8007EFC0(fp, 1).
    melee_fighter::ftcommon2::ftcommon_8007efc0(fp, 1);
    Ok(())
}

/// `ftCo_Rebirth_Phys` / `ftCo_RebirthWait_Phys`.
pub fn phys(p: &mut Player, _ctx: &mut RespawnCtx<'_>) -> Result<(), String> {
    if p.fp.smash_attrs.x2135 != -1 {
        return Err("ftCo_Rebirth_Phys: a stage-specific respawn point not ported".into());
    }
    let fp = &mut *p.fp;
    // ftCommon_8007F8B4: cur_pos + xD4_unk_vel.
    let (cx, cy) = (fp.cur_pos.x + fp.xD4_unk_vel.x, fp.cur_pos.y + fp.xD4_unk_vel.y);
    let inv = 1.0f32 / p.rb.timer as f32;
    fp.self_vel.x = (p.rb.target.x - cx) * inv;
    fp.self_vel.y = (p.rb.target.y - cy) * inv;
    Ok(())
}

/// `ftCo_Rebirth_Anim` / `ftCo_RebirthWait_Anim`.
pub fn anim(p: &mut Player, ctx: &mut RespawnCtx<'_>) -> Result<(), String> {
    ft_8008a7a8(p, ctx.rand)?;
    p.rb.timer -= 1;
    if p.rb.timer == 0 {
        if p.fp.motion_id == FTCO_MS_REBIRTH {
            // ftCo_800D5600: RebirthWait (mpColl_80043680 only for x2135 != -1).
            p.fp.self_vel.y = 0.0;
            p.rb.timer = melee_fighter::hooks::common_data().rebirth_wait;
            let row = load_row(p, FTCO_SM_WAIT1_0)?;
            p.keep_gfx = true;
            change_motion_state(p, FTCO_MS_REBIRTH_WAIT, row.as_ref(), ctx.rand)?;
            let fp = &mut *p.fp;
            fp.set_x221E_b2(1);
            fp.set_x2219_b1(1);
            fp.set_x221E_b1(1);
            fp.set_x221D_b5(1);
        } else {
            // ftColl_8007B7A4 (intangibility window, not modelled), then Fall.
            fall_enter(p, ctx.rand)?;
        }
    }
    Ok(())
}

/// `ftCo_RebirthWait_IASA` (not a follower): any input leaves the platform.
pub fn wait_iasa(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if !p.fp.item_gobj.is_null() {
        return Err("ftCo_RebirthWait_IASA with an item not ported".into());
    }
    // The priority group (special, grab, air dodge, aerial, double jump): refused when it would fire.
    if special_air_check(p, rand)? {
        return Ok(());
    }
    {
        let fp = &*p.fp;
        let i = &fp.input;
        if i.pressed_buttons & !(PAD_STICK_DIRS | input::HSD_PAD_XY | input::HSD_PAD_DPADUP) != 0 || i.cstick[0].x != 0.0 || i.cstick[0].y != 0.0 {
            return Err(format!("player {}: ftCo_RebirthWait_IASA with a press / C-stick: not ported", p.slot + 1));
        }
    }
    if ftco_800cb870(p, rand)? {
        return Ok(());
    }
    let cd = melee_fighter::hooks::common_data();
    let fp = &*p.fp;
    let i = &fp.input;
    // ftCo_80091A2C, ftCo_800DE9B8, fn_800D5F84, ftCo_800C97A8, ftWalkCommon_800DFC70.
    if i.held_buttons[0] & input::HSD_PAD_LR != 0
        || i.pressed_buttons & input::HSD_PAD_DPADUP != 0
        || i.lstick[0].y < -cd.x90
        || i.lstick[0].x * fp.facing_dir <= cd.x34
        || i.lstick[0].x * fp.facing_dir >= cd.walk_stick_threshold
    {
        fall_enter(p, rand)?;
    }
    Ok(())
}

/// `ftCo_8008A7A8(gobj, ft_data->x24)`: when the animation has ended, a new random idle one.
pub fn ft_8008a7a8(p: &mut Player, rand: &mut HsdRand) -> Result<(), String> {
    if p.skel.frames_remaining() {
        return Ok(());
    }
    if p.fp.item_gobj != core::ptr::null_mut() {
        return Err("ftCo_8008A7A8 with an item not ported".into());
    }
    if p.wait_anims.is_empty() {
        return Err("ftCo_8008A7A8: no wait animation table".into());
    }
    let table = p.wait_anims.clone();
    let mut anim_id;
    loop {
        // getAnimID: HSD_Randi(100) + 1 against the running sum of the weights.
        let max = rand.randi(100) + 1;
        let mut count = 0;
        anim_id = -1;
        for &(id, weight) in &table {
            count += weight;
            if max <= count {
                anim_id = id;
                break;
            }
        }
        if anim_id == -1 {
            return Err("wait anim data illegal".into());
        }
        // while (!inlineA0(fp) && fp->anim_id == temp)
        let cur = p.fp.anim_id;
        if cur == 2 || cur == 31 || cur != anim_id {
            break;
        }
    }
    // ftCo_8008A6D8 (inlined): ftData_80085CD8, the script, ftAnim_8006EBE8(0, 1, blend), ftAnim_8006EBA4.
    let (row, tree) = load_row(p, anim_id as u32)?.ok_or(format!("wait anim {anim_id} has no figatree"))?;
    p.fp.anim_id = anim_id;
    p.script = row.script;
    p.script_sp = 0;
    p.fp.anon_594.x594_s32 = i32::from_ne_bytes(row.flags.to_be_bytes());
    p.skel.attach(&tree, 0.0, 1.0, row.looping(), row.blend as f32);
    p.animating = true;
    p.fp.x3E4_fighterCmdScript.timer = 0.0;
    anim_step(p, rand)
}

/// `ftCo_Rebirth_Coll`: `ft_80083DCC` when the respawn has no stage-specific point.
pub fn coll(p: &mut Player, ctx: &mut RespawnCtx<'_>) -> Result<(), String> {
    if p.fp.smash_attrs.x2135 != -1 {
        return Ok(());
    }
    let ecb_desc = p.ecb_desc;
    let joints = ecb_joints(&mut p.skel, &ecb_desc);
    let fp = &mut *p.fp;
    fp.coll_data.last_pos = fp.coll_data.cur_pos;
    fp.coll_data.cur_pos = fp.cur_pos;
    mc::mp_coll_800478f4(ctx.ground, ctx.cg, &mut fp.coll_data, &joints).map_err(|e| format!("{} not ported", e.0))?;
    fp.cur_pos = fp.coll_data.cur_pos;
    ft_80081a00(fp);
    Ok(())
}
