//! The script host's engine side (melee_engine::script_host::EngineGame) on a live match: every query forwarder answers what
//! the transpiled script_game.c function answers, and the slice / C-string / guest-memory calls follow gw_script.c's
//! conversions. Needs the disc (GW_ISO_VANILLA; MELEE_SKIP_DISC_TESTS=1 skips it, loudly).
use melee_engine::{gcm::Gcm, scenario::Scenario, script_host::EngineGame, Engine};
use melee_modes::tu::script_game as sg;
use melee_platform::PadSystem;
use melee_script::game::{ScriptGame, SI_ACTION, SI_CHAR, SI_PRESENT};

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The `melee` binary's process allocator, its arena part (melee/src/main.rs DebugAlloc without the diagnostics): the
/// game scope allocates from the arena, which is what makes a snapshot base possible in this test binary.
struct ArenaAlloc;
unsafe impl std::alloc::GlobalAlloc for ArenaAlloc {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        if melee_engine::arena::active() {
            let p = melee_engine::arena::alloc(l);
            if !p.is_null() {
                return p;
            }
            melee_engine::arena::note_miss(l);
        }
        std::alloc::System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        if melee_engine::arena::contains(p) {
            return melee_engine::arena::free(p, l);
        }
        // a rollback run inside the game scope: a snapshot may still point at this system block (main.rs rollback_pinned)
        if melee_engine::arena::active() && melee_engine::cow::active() {
            return;
        }
        std::alloc::System.dealloc(p, l)
    }
    unsafe fn alloc_zeroed(&self, l: std::alloc::Layout) -> *mut u8 {
        if melee_engine::arena::active() {
            let p = melee_engine::arena::alloc_zeroed(l);
            if !p.is_null() {
                return p;
            }
        }
        std::alloc::System.alloc_zeroed(l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: std::alloc::Layout, n: usize) -> *mut u8 {
        if melee_engine::arena::contains(p) || melee_engine::arena::active() {
            let q = self.alloc(std::alloc::Layout::from_size_align_unchecked(n, l.align()));
            if !q.is_null() {
                core::ptr::copy_nonoverlapping(p, q, l.size().min(n));
                self.dealloc(p, l);
            }
            return q;
        }
        std::alloc::System.realloc(p, l, n)
    }
}
#[global_allocator]
static GLOBAL: ArenaAlloc = ArenaAlloc;

fn boot(frames: u32) -> Option<(std::sync::MutexGuard<'static, ()>, Engine)> {
    let g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(iso) = std::env::var("GW_ISO_VANILLA") else {
        if std::env::var_os("MELEE_SKIP_DISC_TESTS").is_some() {
            eprintln!("SKIPPED script_host: no disc (GW_ISO_VANILLA unset, MELEE_SKIP_DISC_TESTS=1)");
            return None;
        }
        panic!("script_host needs the disc: set GW_ISO_VANILLA, or MELEE_SKIP_DISC_TESTS=1 to skip it");
    };
    let disc = Gcm::open(&iso).expect("disc");
    let scn = Scenario::parse("mode=vs;at=match;p1=falcon/human;p2=fox/human;p3=none;p4=none;stage=fd;match=time;minutes=0;items=off")
        .expect("scene");
    let mut e = Engine::boot_vs(disc, 64 << 20, PadSystem::headless(), scn).expect("boot_vs");
    for _ in 0..frames {
        e.tick();
    }
    Some((g, e))
}

/// Every query (`&self`) forwarder against the direct call, at the first slot / index / field: catches a wrong argument
/// order or cast in the forwarders, and that each runs on a live match.
#[test]
fn queries_equal_the_game_side() {
    let Some((_g, _e)) = boot(120) else { return };
    let g = EngineGame;
    assert_eq!(g.fighter_i(0, 0), unsafe { sg::ScriptGame_FighterI(0, 0) } as i32, "fighter_i");
    assert_eq!(g.fighter_f(0, 0).to_bits(), (unsafe { sg::ScriptGame_FighterF(0, 0) } as f32).to_bits(), "fighter_f");
    assert_eq!(g.lab_i(0, 0), unsafe { sg::ScriptGame_LabI(0, 0) } as i32, "lab_i");
    assert_eq!(g.lab_f(0, 0).to_bits(), (unsafe { sg::ScriptGame_LabF(0, 0) } as f32).to_bits(), "lab_f");
    assert_eq!(g.hit_i(0, 0, 0), unsafe { sg::ScriptGame_HitI(0, 0, 0) } as i32, "hit_i");
    assert_eq!(g.hit_f(0, 0, 0).to_bits(), (unsafe { sg::ScriptGame_HitF(0, 0, 0) } as f32).to_bits(), "hit_f");
    assert_eq!(g.hurt_i(0, 0, 0), unsafe { sg::ScriptGame_HurtI(0, 0, 0) } as i32, "hurt_i");
    assert_eq!(g.hurt_f(0, 0, 0).to_bits(), (unsafe { sg::ScriptGame_HurtF(0, 0, 0) } as f32).to_bits(), "hurt_f");
    assert_eq!(g.joint_f(0, 0, 0).to_bits(), (unsafe { sg::ScriptGame_JointF(0, 0, 0) } as f32).to_bits(), "joint_f");
    assert_eq!(g.joint_parent(0, 0), unsafe { sg::ScriptGame_JointParent(0, 0) } as i32, "joint_parent");
    assert_eq!(g.lab_tobj_f(0, 0, 0, 0).to_bits(), (unsafe { sg::ScriptGame_LabTObjF(0, 0, 0, 0) } as f32).to_bits(), "lab_tobj_f");
    assert_eq!(g.camera_f(0).to_bits(), (unsafe { sg::ScriptGame_CameraF(0) } as f32).to_bits(), "camera_f");
    assert_eq!(g.lab_anim_end(0).to_bits(), (unsafe { sg::ScriptGame_LabAnimEnd(0) } as f32).to_bits(), "lab_anim_end");
    assert_eq!(g.lab_attr_count(), unsafe { sg::ScriptGame_LabAttrCount() } as i32, "lab_attr_count");
    assert_eq!(g.lab_attr_f(0, 0).to_bits(), (unsafe { sg::ScriptGame_LabAttrF(0, 0) } as f32).to_bits(), "lab_attr_f");
    assert_eq!(g.virtual_pad_seen(0, 0), unsafe { sg::ScriptGame_VirtualPadSeen(0, 0) } as i32, "virtual_pad_seen");
    assert_eq!(g.cpu_mode_info(0), unsafe { sg::ScriptGame_CpuModeInfo(0) } as i32, "cpu_mode_info");
    assert_eq!(g.lab_common_count(0), unsafe { sg::ScriptGame_LabCommonCount(0) } as i32, "lab_common_count");
    assert_eq!(g.lab_motion_anim(0, 0), unsafe { sg::ScriptGame_LabMotionAnim(0, 0) } as i32, "lab_motion_anim");
    assert_eq!(g.lab_row_count(0), unsafe { sg::ScriptGame_LabRowCount(0) } as i32, "lab_row_count");
    assert_eq!(g.stage_slots_active(), unsafe { sg::ScriptGame_StageSlotsActive() } != 0, "stage_slots_active");
    assert_eq!(g.lab_common_f(0).to_bits(), (unsafe { sg::ScriptGame_LabCommonF(0) } as f32).to_bits(), "lab_common_f");
    assert_eq!(g.lab_floor_y(0.0, 0.0, 0.0).to_bits(), (unsafe { sg::ScriptGame_LabFloorY(0, 0, 0) } as f32).to_bits(), "lab_floor_y");
    assert_eq!(g.stage_kind(), unsafe { sg::ScriptGame_StageKind() } as i32, "stage_kind");
    assert_eq!(g.game_mode(), unsafe { sg::ScriptGame_GameMode() } as i32, "game_mode");
    assert_eq!(g.rng_seed(), unsafe { sg::ScriptGame_RngSeed() } as u32, "rng_seed");
    assert_eq!(g.mode_active(), unsafe { sg::ScriptGame_ModeActive() } != 0, "mode_active");
    assert_eq!(g.lab_knockback(0, 0, 0, 0, 0, 0, 0, 0).to_bits(), (unsafe { sg::ScriptGame_LabKnockback(0, 0, 0, 0, 0, 0, 0, 0) } as f32).to_bits(), "lab_knockback");
    assert_eq!(g.lab_kb_level(0), unsafe { sg::ScriptGame_LabKbLevel(0) } as i32, "lab_kb_level");
    assert_eq!(g.lab_flight_f(0, 0).to_bits(), (unsafe { sg::ScriptGame_LabFlightF(0, 0) } as f32).to_bits(), "lab_flight_f");
    assert_eq!(g.arena_owner(), unsafe { sg::ScriptGame_ArenaOwner() } as i32, "arena_owner");
    assert_eq!(g.arena_spawn_pos(0, 0, 0.0).to_bits(), (unsafe { sg::ScriptGame_ArenaSpawnPos(0, 0, 0.0) } as f32).to_bits(), "arena_spawn_pos");
    assert_eq!(g.arena_read(0), unsafe { sg::ScriptGame_ArenaRead(0) } as i32, "arena_read");
    assert_eq!(g.arena_group(0, 0), unsafe { sg::ScriptGame_ArenaGroup(0, 0) } as i32, "arena_group");
    assert_eq!(g.map_bound(0, 0), unsafe { sg::ScriptGame_MapBound(0, 0) } as i32, "map_bound");
    assert_eq!(g.caps_movement_read(0, 0), unsafe { sg::ScriptGame_CapsMovementRead(0, 0) } as i32, "caps_movement_read");
    assert_eq!(g.caps_effect_read(0, 0, 0), unsafe { sg::ScriptGame_CapsEffectRead(0, 0, 0) } as i32, "caps_effect_read");
    assert_eq!(g.caps_armour_read(0, 0), unsafe { sg::ScriptGame_CapsArmourRead(0, 0) } as i32, "caps_armour_read");
    assert_eq!(g.caps_opponent(0, 0, 0, 0), unsafe { sg::ScriptGame_CapsOpponent(0, 0, 0, 0) } as i32, "caps_opponent");
    assert_eq!(g.caps_status_read(0, 0, 0), unsafe { sg::ScriptGame_CapsStatusRead(0, 0, 0) } as i32, "caps_status_read");
    assert_eq!(g.intr_win_read(0, 0), unsafe { sg::ScriptGame_IntrWinRead(0, 0) } as i32, "intr_win_read");
    assert_eq!(g.armor_read(0, 0, 0), unsafe { sg::ScriptGame_ArmorRead(0, 0, 0) } as i32, "armor_read");
    assert_eq!(g.crit_read(0, 0), unsafe { sg::ScriptGame_CritRead(0, 0) } as i32, "crit_read");
    assert_eq!(g.fighter_bench_check(0, 0, 0), unsafe { sg::ScriptGame_FighterBenchCheck(0, 0, 0) } as i32, "fighter_bench_check");
    assert_eq!(g.fighter_benched(0), unsafe { sg::ScriptGame_FighterBenched(0) } as i32, "fighter_benched");
    assert_eq!(g.fighter_reserve_read(0, 0, 0), unsafe { sg::ScriptGame_FighterReserveRead(0, 0, 0) } as i32, "fighter_reserve_read");
    assert_eq!(g.action_category(0), unsafe { sg::ScriptGame_ActionCategory(0) } as i32, "action_category");
    assert_eq!(g.zone_field(0, 0), unsafe { sg::ScriptGame_ZoneField(0, 0) } as i32, "zone_field");
    assert_eq!(g.zone_text(0, 0, 0), unsafe { sg::ScriptGame_ZoneText(0, 0, 0) } as i32, "zone_text");
    assert_eq!(g.zone_entity(0, 0, 0), unsafe { sg::ScriptGame_ZoneEntity(0, 0, 0) } as i32, "zone_entity");
    assert_eq!(g.zone_event(0, 0), unsafe { sg::ScriptGame_ZoneEvent(0, 0) } as i32, "zone_event");
    assert_eq!(g.zone_event_text(0, 0, 0), unsafe { sg::ScriptGame_ZoneEventText(0, 0, 0) } as i32, "zone_event_text");
    assert_eq!(g.zone_event_count(), unsafe { sg::ScriptGame_ZoneEventCount() } as i32, "zone_event_count");
    assert_eq!(g.skill_row_count(), unsafe { sg::ScriptGame_SkillRowCount() } as i32, "skill_row_count");
    assert_eq!(g.skill_row(0, 0), unsafe { sg::ScriptGame_SkillRow(0, 0) } as i32, "skill_row");
    assert_eq!(g.skill_state(0, 0), unsafe { sg::ScriptGame_SkillState(0, 0) } as i32, "skill_state");
    assert_eq!(g.skill_const(0), unsafe { sg::ScriptGame_SkillConst(0) } as i32, "skill_const");
    assert_eq!(g.shock_read(0, 0), unsafe { sg::ScriptGame_ShockRead(0, 0) } as i32, "shock_read");
    assert_eq!(g.contact_field(0, 0, 0, 0), unsafe { sg::ScriptGame_ContactField(0, 0, 0, 0) } as i32, "contact_field");
    assert_eq!(g.contact_flags(0), unsafe { sg::ScriptGame_ContactFlags(0) } as i32, "contact_flags");
    assert_eq!(g.contact_cross(0, 0), unsafe { sg::ScriptGame_ContactCross(0, 0) } as i32, "contact_cross");
    assert_eq!(g.camera_param_read(0), unsafe { sg::ScriptGame_CameraParamRead(0) } as i32, "camera_param_read");
    assert_eq!(g.camera_param_owner(), unsafe { sg::ScriptGame_CameraParamOwner() } as i32, "camera_param_owner");
    assert_eq!(g.fighter_history_depth(), unsafe { sg::ScriptGame_FighterHistoryDepth() } as i32, "fighter_history_depth");
    assert_eq!(g.fighter_history_read(0, 0, 0, 0), unsafe { sg::ScriptGame_FighterHistoryRead(0, 0, 0, 0) } as i32, "fighter_history_read");
    assert_eq!(g.echo_read(0, 0, 0), unsafe { sg::ScriptGame_EchoRead(0, 0, 0) } as i32, "echo_read");
    assert_eq!(g.echo_owner(0), unsafe { sg::ScriptGame_EchoOwner(0) } as i32, "echo_owner");
    assert_eq!(g.echo_next_handle(), unsafe { sg::ScriptGame_EchoNextHandle() } as i32, "echo_next_handle");
    assert_eq!(g.echo_state(0, 0, 0), unsafe { sg::ScriptGame_EchoState(0, 0, 0) } as i32, "echo_state");
    assert_eq!(g.cpu_attr(0, 0), unsafe { sg::ScriptGame_CpuAttr(0, 0) } as i32, "cpu_attr");
    assert_eq!(g.parts_i(0, 0, 0), unsafe { sg::ScriptGame_PartsI(0, 0, 0) } as i32, "parts_i");
    assert_eq!(g.parts_f(0, 0, 0).to_bits(), (unsafe { sg::ScriptGame_PartsF(0, 0, 0) } as f32).to_bits(), "parts_f");
    assert_eq!(g.parts_joint(0, 0), unsafe { sg::ScriptGame_PartsJoint(0, 0) } as i32, "parts_joint");
    assert_eq!(g.fighter_mod_owner(0), unsafe { sg::ScriptGame_FighterModOwner(0) } as i32, "fighter_mod_owner");
    assert_eq!(g.fighter_mod_read(0, 0), unsafe { sg::ScriptGame_FighterModRead(0, 0) } as i32, "fighter_mod_read");
    assert_eq!(g.entity_field(0, 0, 0), unsafe { sg::ScriptGame_EntityField(0, 0, 0) } as i32, "entity_field");
    assert_eq!(g.entity_epoch(), unsafe { sg::ScriptGame_EntityEpoch() } as i32, "entity_epoch");
    assert_eq!(g.entity_address_field(0, 0, 0), unsafe { sg::ScriptGame_EntityAddressField(0, 0, 0) } as i32, "entity_address_field");
    assert_eq!(g.hit_rules_owner(0), unsafe { sg::ScriptGame_HitRulesOwner(0) } as i32, "hit_rules_owner");
    assert_eq!(g.hit_rules_count(0), unsafe { sg::ScriptGame_HitRulesCount(0) } as i32, "hit_rules_count");
    assert_eq!(g.fighter_status(0), unsafe { sg::ScriptGame_FighterStatus(0) } as i32, "fighter_status");
    assert_eq!(g.hit_rule_read(0, 0, 0), unsafe { sg::ScriptGame_HitRuleRead(0, 0, 0) } as i32, "hit_rule_read");
}

/// What the match is: two fighters in slots 0 and 1, an animation playing, its subaction script readable as guest words.
#[test]
fn fighters_animation_and_guest_memory() {
    let Some((_g, _e)) = boot(120) else { return };
    let g = EngineGame;
    assert_eq!((g.fighter_i(0, SI_PRESENT), g.fighter_i(1, SI_PRESENT)), (1, 1));
    assert_eq!((g.fighter_i(2, SI_PRESENT), g.fighter_i(3, SI_PRESENT)), (0, 0));
    assert_ne!(g.fighter_i(0, SI_CHAR), g.fighter_i(1, SI_CHAR), "Falcon and Fox");
    assert!(g.fighter_i(0, SI_ACTION) >= 0);
    let sym = g.lab_anim_symbol(0).expect("an animation is playing");
    assert!(sym.starts_with("PlyCaptain"), "{sym}");
    // the playing animation's subaction script: a guest address inside game memory, its words readable big-endian
    let script = g.lab_script(0, -1).expect("Falcon's current animation has a script");
    assert!(g.guest_ok(script, 4));
    let w0 = g.guest_word(script);
    assert!(w0 >> 26 < 64, "a command opcode: {w0:08x}");
    assert_eq!(w0, u32::from_be_bytes(unsafe { *(script as usize as *const [u8; 4]) }));
    // outside game memory: not readable, reads 0
    assert!(!g.guest_ok(0x10, 4));
    assert_eq!(g.guest_word(0x10), 0);
}

/// The scripted camera (cm/camera.c's Camera_Script* through gw_script.c's calls): detach captures the game camera's pose,
/// a set moves the detached pose, attach(frames) eases back (state 2) and attach(0) / reset snaps back (state 0). And the
/// match-side Geno / stage queries on a vanilla match: no profile, no combo graph, scripts are their own originals.
#[test]
fn camera_script_and_match_queries() {
    let Some((_g, mut e)) = boot(120) else { return };
    let mut g = EngineGame;
    assert_eq!(g.camera_script_state(), 0, "attached");
    let eye: Vec<i32> = (0..8).map(|f| g.camera_script_get_bits(f)).collect();
    assert!(g.camera_script_detach());
    assert_eq!(g.camera_script_state(), 1, "detached");
    assert_eq!((0..8).map(|f| g.camera_script_get_bits(f)).collect::<Vec<_>>(), eye, "detach keeps the game camera's pose");
    g.camera_script_set(6, 45.0f32.to_bits() as i32);
    g.camera_script_set(0, 12.5f32.to_bits() as i32);
    assert_eq!((g.camera_script_get_bits(6), g.camera_script_get_bits(0)), (45.0f32.to_bits() as i32, 12.5f32.to_bits() as i32));
    e.tick();
    assert_eq!(g.camera_script_get_bits(6), 45.0f32.to_bits() as i32, "a detached pose survives a frame");
    g.camera_script_attach(10);
    assert_eq!(g.camera_script_state(), 2, "attaching");
    g.camera_script_reset();
    assert_eq!(g.camera_script_state(), 0, "reset snaps back");
    // a set while attached is ignored (Camera_ScriptSet: state != 1 returns)
    let fov = g.camera_script_get_bits(6);
    g.camera_script_set(6, 10.0f32.to_bits() as i32);
    assert_eq!(g.camera_script_get_bits(6), fov);
    assert_eq!(g.camera_script_mode(), unsafe { melee_world::tu::camera::Camera_ScriptMode() });
    assert_eq!(g.camera_script_completion(), unsafe { melee_world::tu::camera::Camera_ScriptCompletion() });

    assert_eq!(g.stage_external(), unsafe { melee_world::tu::stage::Stage_80225194() } as i32);
    assert_eq!(g.stage_external(), 32, "Final Destination's external id");
    let kind = g.lab_i(0, melee_script::game::lab::I_NAME_KIND);
    assert_eq!((g.geno_profile_for_kind(kind), g.geno_state_count(-1), g.mex_move_logic_entries(kind)), (-1, 0, 0));
    assert!(g.combo_info(0, 3) < 0, "Falcon has no combo graph");
    assert_eq!(g.combo_info(0, 3), unsafe { melee_modes::tu::geno_game::GenoGame_ComboInfo(0, 3) });
    assert!(g.combo_set_ki(0, 50) < 0);
    let script = g.lab_script(0, -1).expect("a script");
    // geno_game.c: an address that is not a Geno overlay slot answers 0 (gw_script.c only asks at a Geno ORIG)
    assert_eq!(g.geno_script_original(script), 0, "no Geno overlay on a vanilla fighter");
    assert_eq!(g.geno_lab_reload(), 0, "no Geno fighter to re-apply");
}

/// The LAB fly mode, CPU skills, stage bounds and the release / peek / resimulation calls, on a VS match (not the LAB):
/// each forwarder answers what the game function answers, and the C's clamps and refusals hold.
#[test]
fn fly_cpu_bounds_and_releases() {
    use melee_modes::tu::geno_lab_mode as lab;
    let Some((_g, _e)) = boot(120) else { return };
    let mut g = EngineGame;
    // fly: slot 9 has no fighter; speed clamps to [0.05, 200] (GenoFly_SetSpeed); solid round-trips
    assert_eq!((g.geno_fly_get(0), g.geno_fly_get(9)), unsafe { (lab::GenoFly_Get(0), lab::GenoFly_Get(9)) });
    assert_eq!((g.geno_fly_get(9), g.geno_fly_set(9, 1)), (-1, -1));
    g.geno_fly_set_speed(500.0f32.to_bits() as i32);
    assert_eq!(g.geno_fly_speed(), 200.0);
    g.geno_fly_set_speed(0.01f32.to_bits() as i32);
    assert_eq!(g.geno_fly_speed(), 0.05);
    g.geno_fly_set_speed(1.5f32.to_bits() as i32);
    assert_eq!(g.geno_fly_speed(), unsafe { lab::GenoFly_Speed() });
    for v in [true, false] {
        g.geno_fly_set_solid(v);
        assert_eq!(g.geno_fly_solid(), v);
    }
    assert_eq!((g.geno_fly_holding(0), g.geno_fly_hold_hits(0), g.geno_fly_hold_hits(9)), (false, 0, 0));
    for f in 0..14 {
        assert_eq!(g.geno_fly_state(0, f), unsafe { lab::GenoFly_State(0, f) }, "fly state {f}");
    }
    assert_eq!(g.geno_fly_teleport(9, 0, 0), unsafe { lab::GenoFly_Teleport(9, 0, 0) });
    assert_eq!(g.geno_fly_target(9, 0, 0), unsafe { lab::GenoFly_Target(9, 0, 0) });
    assert_eq!(g.geno_fly_attack_config(9, 1, 5, 4.0f32.to_bits() as i32, 2, 0, 45), unsafe {
        lab::GenoFly_AttackConfig(9, 1, 5, 4.0f32.to_bits() as i32, 2, 0, 45)
    });
    assert_eq!(g.geno_fly_hold_hitbox(9, true, 0, 0), unsafe { lab::GenoFly_HoldHitbox(9, 1, 0, 0) });
    g.geno_fly_clear(0);
    assert_eq!(g.geno_fly_state(0, 0), 0, "cleared: not attacking");
    // not the LAB: leaving is refused and changes nothing
    assert!(!g.geno_lab_mode_active());
    assert!(!g.geno_lab_leave(0));

    // CPU skills on human fighters: nothing enabled; the controller has no CPU fighter (GCC_G_ACTIVE -1)
    for f in 0..18 {
        assert_eq!(g.cpu_assist_state(0, 1, f), unsafe { melee_fighter::tu::fighter::ScriptGame_CpuAssistState(0, 1, f) }, "assist {f}");
        assert_eq!(g.cpu_technical_state(0, 1, f), unsafe { melee_fighter::tu::fighter::ScriptGame_CpuTechnicalState(0, 1, f) }, "technical {f}");
    }
    assert_eq!(g.cpu_assist_state(0, 1, 0), 0, "assist off");
    assert_eq!(g.cpu_assist_clear_slot(0, 1), unsafe { melee_fighter::tu::fighter::ScriptGame_CpuAssistClearSlot(0, 1) });
    assert_eq!(g.cpu_ctl_read(0, 0, 100, 0), -1, "no CPU fighter in slot 0");
    assert_eq!(g.cpu_ctl_read(0, 0, 100, 0), unsafe { sg::ScriptGame_CpuCtl(0, 0, 100, 0) } as i32);

    // stage bounds as float bits; FD has a main floor (field 12)
    for f in 0..14 {
        assert_eq!(g.stage_bounds_i(f), unsafe { sg::ScriptGame_StageBounds(f) } as i32, "bounds {f}");
    }
    assert_eq!(g.stage_bounds_i(12), 1);
    assert!(f32::from_bits(g.stage_bounds_i(0) as u32) != 0.0, "a blast zone");

    // a script owner that holds nothing: the releases leave the match as it is
    let before = (g.fighter_i(0, SI_ACTION), g.fighter_i(1, SI_ACTION), g.echo_next_handle());
    g.caps_release(1);
    g.owner_release(1);
    g.arena_release(1);
    g.mods_game_event(0, 0, 0, 0, 0);
    assert_eq!((g.fighter_i(0, SI_ACTION), g.fighter_i(1, SI_ACTION), g.echo_next_handle()), before);

    // lab_peek: the bytes of game memory as stored (big-endian); outside it, None
    let script = g.lab_script(0, -1).expect("a script");
    let b = g.mem1_read(script, 8).expect("game memory");
    assert_eq!(u32::from_be_bytes(b[0..4].try_into().unwrap()), g.guest_word(script));
    assert_eq!(u32::from_be_bytes(b[4..8].try_into().unwrap()), g.guest_word(script + 4));
    assert_eq!((g.mem1_read(0x10, 4), g.mem1_read(u32::MAX - 2, 4)), (None, None));

    // the rollback's flag (gw_Snap_SessionResim), as the play loop sets it
    assert!(!g.resimulating());
    melee_engine::snap_host::session_resim(true);
    assert!(g.resimulating());
    melee_engine::snap_host::session_resim(false);
    assert!(!g.resimulating());
}

/// gw_snap.c's machinery through the script methods (melee_engine::snap_host, the one base): slots by index, the
/// long rewind's keyframes, the exactness self-test, savestate files, the rollback ring. "The same state" is the game
/// state's page hash (the OS heap, what rollback and synctest compare).
#[test]
fn snapshot_slots_rewind_files_rbviz() {
    use melee_engine::snap_host as sh;
    let Some((_g, mut e)) = boot(60) else { return };
    let mut g = EngineGame;
    let hash = || sh::with_base(|b| b.page_hash());

    // slots: reserve opens the machinery and caps at GW_SNAP_MAX_SLOTS; a never-saved or missing slot does not load
    assert_eq!(g.snap_reserve(4), 4);
    assert_eq!(g.snap_reserve(100), sh::MAX_SLOTS as i32);
    assert!(g.snap_slot_bytes() > 1 << 20);
    g.snap_save_index(0, -1);
    let h0 = hash();
    for _ in 0..30 {
        e.tick();
    }
    assert_ne!(hash(), h0, "30 frames changed the game");
    assert_eq!(g.snap_load_index(0), 0);
    assert_eq!(hash(), h0, "slot 0 put the state back");
    assert_eq!((g.snap_load_index(1), g.snap_load_index(99), g.snap_load_index(-1)), (-1, -1, -1));

    // the rewind: a base and keyframes with rising tags, each with its user block
    assert_eq!(g.rw_begin(10, b"base"), 0);
    e.tick();
    assert_eq!(g.rw_save(11, b"k11"), 0);
    let h11 = hash();
    e.tick();
    assert_eq!(g.rw_save(12, b"k12"), 0);
    assert_eq!(g.rw_save(12, b"again"), -1, "a tag not above the latest");
    assert_eq!((g.rw_count(), g.rw_base_tag(), g.rw_latest()), (3, 10, 12));
    assert_eq!((g.rw_find_le(11), g.rw_find_le(100), g.rw_find_le(9)), (Some(11), Some(12), None));
    assert!(g.rw_delta_bytes() > 0.0 && g.rw_bytes() >= g.rw_delta_bytes());
    let u = g.rw_load(11).expect("keyframe 11");
    assert_eq!((&u[..3], u.len()), (&b"k11"[..], sh::RW_USER));
    assert_eq!(hash(), h11, "keyframe 11 put back");
    assert!(g.rw_last_load_pages() > 0);
    assert_eq!(g.rw_load(13), None);
    assert_eq!(&g.rw_load(10).expect("the base")[..4], b"base");
    assert_eq!(g.rw_drop_from(10), -1, "the base cannot go");
    assert_eq!(g.rw_drop_from(12), 0);
    assert_eq!((g.rw_count(), g.rw_latest()), (2, 11));
    g.rw_trim(11);
    assert_eq!((g.rw_count(), g.rw_base_tag()), (1, 11));
    let ht = hash();
    assert_eq!((g.rw_find_le(10), hash()), (None, ht), "trim folded 10 away and left the live state alone");
    g.rw_end();
    assert_eq!((g.rw_count(), g.rw_bytes()), (0, 0.0));

    // the exactness self-test: equal right after the capture, different a frame later; the compare frees the capture
    assert_eq!(g.rw_compare(), None);
    assert_eq!(g.rw_capture(), 0);
    let (d, h) = g.rw_compare().expect("a capture");
    assert_eq!((d[1], h[0] == h[1]), (0.0, true));
    assert_eq!(g.rw_capture(), 0);
    e.tick();
    let (d, h) = g.rw_compare().expect("a capture");
    assert!(d[1] > 0.0 && d[0] >= d[1] && h[0] != h[1], "{d:?}");
    assert_eq!(g.rw_compare(), None);

    // savestate files: the header alone, renamed in place, a whole load; damage refused before anything is touched
    let dir = std::env::temp_dir().join(format!("snaphost-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("s.gdst");
    let p = path.to_str().unwrap();
    let hf = hash();
    assert_eq!(g.snap_file_save(p, b"HEAD0001"), 0);
    assert_eq!(g.snap_file_header(p, 64).as_deref(), Some(&b"HEAD0001"[..]));
    assert_eq!(g.snap_file_header(p, 4), None, "longer than the caller's buffer");
    assert_eq!(g.snap_file_rewrite_header(p, b"HEAD0002"), 0);
    assert_eq!(g.snap_file_rewrite_header(p, b"short"), -1);
    for _ in 0..10 {
        e.tick();
    }
    assert_ne!(hash(), hf);
    assert_eq!(g.snap_file_load(p), 0);
    assert_eq!(hash(), hf, "the file put the state back");
    assert_eq!(g.snap_file_header(p, 64).as_deref(), Some(&b"HEAD0002"[..]));
    let mut bad = std::fs::read(&path).unwrap();
    let n = bad.len();
    bad[n / 2] ^= 0xFF;
    let badp = dir.join("bad.gdst");
    std::fs::write(&badp, &bad).unwrap();
    let hb = hash();
    assert_eq!(g.snap_file_load(badp.to_str().unwrap()), -3, "damaged");
    assert_eq!(hash(), hb, "a refused file touches nothing");
    std::fs::write(&badp, b"not a state").unwrap();
    assert_eq!((g.snap_file_load(badp.to_str().unwrap()), g.snap_file_header(badp.to_str().unwrap(), 64)), (-1, None));
    assert_eq!(g.snap_file_load(dir.join("missing").to_str().unwrap()), -1);
    let _ = std::fs::remove_dir_all(&dir);

    // the rollback ring: newest first, a mismatch marks the newest, the first desync is kept
    g.rbviz_reset();
    assert_eq!((g.rbviz_total(), g.rbviz_get(0).is_none()), (0, true));
    sh::rbviz_push(100, 98, 2, 1, 0.5, 0);
    sh::rbviz_push(101, 100, 1, 2, 0.25, 1);
    sh::rbviz_mark_mismatch();
    let r = g.rbviz_get(0).unwrap();
    assert_eq!((g.rbviz_total(), r.frame, r.depth, r.mismatch, g.rbviz_get(1).unwrap().mismatch), (2, 101, 1, true, false));
    assert_eq!(g.rbviz_first_mismatch().unwrap().frame, -1);
    sh::rbviz_desync(101, 0x1234, 0x5678);
    sh::rbviz_desync(102, 1, 2);
    let m = g.rbviz_first_mismatch().unwrap();
    assert_eq!((m.frame, m.count, m.place.as_str()), (101, 2, "netplay checksum: local 00001234, peer 00005678"));
    g.rbviz_reset();
    assert_eq!((g.rbviz_total(), g.rbviz_first_mismatch().unwrap().frame), (0, -1));

    // the Lab's silent re-simulation flag, while the machinery is open
    g.snap_lab_resim(true);
    assert!(g.resimulating());
    g.snap_lab_resim(false);
    assert!(!g.resimulating());
    // the base belongs to this engine: let it go before the next test boots another
    sh::release();
    assert!(!sh::enabled());
}
