//! Rollback support for the native VS match: save / load of the dynamic state, the whole-state
//! hash and the neutral (curated) hash `melee_net::session::RbGame` needs, and the raw-pad feed
//! (`VsMatch::pad_feed`) the session's inputs go through.
//!
//! The state is exactly the set of components `VsMatch::state_dump` writes (the component list
//! the C-snapshot loader of snapshot.rs overlays): per player the Fighter record and every
//! `Player` field that changes after the match is built, the HSD pad status, the logic RNG, the
//! particle system, mp collision, the live effects / items / stage particle tracks, the
//! StaticPlayer table and the staling counters. Everything else in `VsMatch` (disc files, banks,
//! stage tables, animation rows loaded on first use) is immutable after construction and shared.
//!
//! Fighters are restored IN PLACE (the Box stays where it is, the record is overwritten), like
//! MEM1 in the C: pointers into the record stay valid.

use super::*;
use melee_net::pad::RbInput;
use melee_types::neutral_hash::nh_game;

/// A `Player`'s dynamic fields.
#[derive(Clone)]
struct PlayerDyn {
    fp: Fighter,
    animating: bool,
    entry: EntryVars,
    walk: WalkVars,
    turn: TurnVars,
    landing_allow_interrupt: bool,
    fall_smid: i32,
    fall_x4: f32,
    ca_lw_x0: f32,
    ca_lw_friction: f32,
    script: u32,
    script_sp: usize,
    script_ret: [u32; 8],
    x221d_b3: bool,
    input_locked: bool,
    hud_randomize: bool,
    dead_timer: i32,
    guard: GuardVars,
    jump: JumpVars,
    dash: dashrun::DashVars,
    run: dashrun::RunVars,
    fx: FoxVars,
    attack1_x0: bool,
    rb: PlayerRespawn,
    hitboxes: [Option<Hitbox>; 4],
    item_cmds: Vec<ItemCmd>,
    ef_queue: Vec<QueuedEffect>,
    ef_ops: Vec<EfOp>,
    keep_gfx: bool,
    blend_zero: bool,
    skel: Skeleton,
}

impl PlayerDyn {
    fn save(p: &Player) -> Self {
        PlayerDyn {
            fp: *p.fp,
            animating: p.animating,
            entry: p.entry,
            walk: p.walk,
            turn: p.turn,
            landing_allow_interrupt: p.landing_allow_interrupt,
            fall_smid: p.fall_smid,
            fall_x4: p.fall_x4,
            ca_lw_x0: p.ca_lw_x0,
            ca_lw_friction: p.ca_lw_friction,
            script: p.script,
            script_sp: p.script_sp,
            script_ret: p.script_ret,
            x221d_b3: p.x221d_b3,
            input_locked: p.input_locked,
            hud_randomize: p.hud_randomize,
            dead_timer: p.dead_timer,
            guard: p.guard,
            jump: p.jump,
            dash: p.dash,
            run: p.run,
            fx: p.fx,
            attack1_x0: p.attack1_x0,
            rb: p.rb,
            hitboxes: p.hitboxes,
            item_cmds: p.item_cmds.clone(),
            ef_queue: p.ef_queue.clone(),
            ef_ops: p.ef_ops.clone(),
            keep_gfx: p.keep_gfx,
            blend_zero: p.blend_zero,
            skel: p.skel.clone(),
        }
    }

    fn load(&self, p: &mut Player) {
        *p.fp = self.fp; // in place
        p.animating = self.animating;
        p.entry = self.entry;
        p.walk = self.walk;
        p.turn = self.turn;
        p.landing_allow_interrupt = self.landing_allow_interrupt;
        p.fall_smid = self.fall_smid;
        p.fall_x4 = self.fall_x4;
        p.ca_lw_x0 = self.ca_lw_x0;
        p.ca_lw_friction = self.ca_lw_friction;
        p.script = self.script;
        p.script_sp = self.script_sp;
        p.script_ret = self.script_ret;
        p.x221d_b3 = self.x221d_b3;
        p.input_locked = self.input_locked;
        p.hud_randomize = self.hud_randomize;
        p.dead_timer = self.dead_timer;
        p.guard = self.guard;
        p.jump = self.jump;
        p.dash = self.dash;
        p.run = self.run;
        p.fx = self.fx;
        p.attack1_x0 = self.attack1_x0;
        p.rb = self.rb;
        p.hitboxes = self.hitboxes;
        p.item_cmds.clone_from(&self.item_cmds);
        p.ef_queue.clone_from(&self.ef_queue);
        p.ef_ops.clone_from(&self.ef_ops);
        p.keep_gfx = self.keep_gfx;
        p.blend_zero = self.blend_zero;
        p.skel.clone_from(&self.skel);
    }
}

/// `gw_snap_save`'s result for the native match.
#[derive(Clone)]
pub struct RbSnapshot {
    frame: i32,
    stopped: Option<String>,
    pads: [input::PadStatus; 4],
    rand: HsdRand,
    ps: PsSystem,
    ground: GroundColl,
    cg: CollGlobals,
    players: Vec<PlayerDyn>,
    last_bg: Option<LastBg>,
    dptcl: Vec<DptclAnim>,
    aobj_globals: AObjGlobals,
    effects: Vec<Effect>,
    next_effect_instance: u32,
    joint_mtx: HashMap<JObjRef, Mtx>,
    fighter_jobjs: Vec<(JObjRef, usize, usize)>,
    items: Vec<SimItem>,
    respawn_busy: [i32; 6],
    static_players: Vec<melee_types::StaticPlayer>,
    stale_attack_instance: u16,
    attack_counter: u16,
}

impl VsMatch {
    /// Save the dynamic state.
    pub fn rb_save(&self) -> RbSnapshot {
        assert!(self.players.iter().all(|p| p.mex.is_none()), "rollback snapshots do not cover m-ex fighter state");
        RbSnapshot {
            frame: self.frame,
            stopped: self.stopped.clone(),
            pads: self.pads,
            rand: self.rand.clone(),
            ps: self.ps.clone(),
            ground: self.ground.clone(),
            cg: self.cg.clone(),
            players: self.players.iter().map(PlayerDyn::save).collect(),
            last_bg: self.last_bg.clone(),
            dptcl: self.dptcl.clone(),
            aobj_globals: self.aobj_globals.clone(),
            effects: self.effects.clone(),
            next_effect_instance: self.next_effect_instance,
            joint_mtx: self.joint_mtx.clone(),
            fighter_jobjs: self.fighter_jobjs.clone(),
            items: self.items.clone(),
            respawn_busy: self.respawn_busy,
            static_players: (0..6).map(|s| *stale::player(s)).collect(),
            stale_attack_instance: unsafe { stale::STALE_ATTACK_INSTANCE },
            attack_counter: unsafe { stale::ATTACK_COUNTER },
        }
    }

    /// Restore a saved state (the match must be the one it was saved from).
    pub fn rb_load(&mut self, s: &RbSnapshot) {
        self.frame = s.frame;
        self.stopped = s.stopped.clone();
        self.pads = s.pads;
        // TEST-ONLY negative controls: leave one component out of the restore (SyncTest must fail).
        let neg = std::env::var("MELEE_RB_NEGCTL").unwrap_or_default();
        if neg != "rand" {
            self.rand = s.rand.clone();
        }
        self.ps = s.ps.clone();
        self.ground = s.ground.clone();
        self.cg = s.cg.clone();
        for (p, d) in self.players.iter_mut().zip(&s.players) {
            d.load(p);
        }
        self.last_bg = s.last_bg.clone();
        self.dptcl = s.dptcl.clone();
        self.aobj_globals = s.aobj_globals.clone();
        self.effects = s.effects.clone();
        self.next_effect_instance = s.next_effect_instance;
        self.joint_mtx = s.joint_mtx.clone();
        self.fighter_jobjs = s.fighter_jobjs.clone();
        self.items = s.items.clone();
        self.respawn_busy = s.respawn_busy;
        if neg != "players" {
            for (i, p) in s.static_players.iter().enumerate() {
                *stale::player(i as i32) = *p;
            }
        }
        unsafe {
            stale::STALE_ATTACK_INSTANCE = s.stale_attack_instance;
            stale::ATTACK_COUNTER = s.attack_counter;
        }
    }

    /// `gw_Snap_Checksum`: a hash of the whole canonical state text (valid within one process).
    pub fn rb_state_hash(&self) -> u32 {
        let h = super::snapshot::fnv(&self.state_dump());
        (h ^ (h >> 32)) as u32
    }

    /// `gw_RB_GameHashForFrame`: the neutral hash of the current frame (what two peers compare).
    pub fn rb_game_hash(&self) -> u32 {
        let w = self.words();
        nh_game(w.seed, &w.slots, w.items, w.itemsum)
    }

    /// The neutral hash with the seed word replaced (the C's online checksum seed rule).
    pub fn rb_game_hash_seeded(&self, seed: u32) -> u32 {
        let w = self.words();
        nh_game(seed, &w.slots, w.items, w.itemsum)
    }

    /// Force the logic RNG (gw_Replay_ResyncSeed in live mode).
    pub fn rb_force_seed(&mut self, seed: u32) {
        self.rand.set_logic_seed(seed);
    }

    /// Feed one frame's inputs (one per port; `None`: neutral) to the next step.
    pub fn set_pads(&mut self, inputs: &[Option<RbInput>; 4]) {
        let mut feed = [input::RawPad::default(); 4];
        for (f, i) in feed.iter_mut().zip(inputs) {
            if let Some(i) = i {
                *f = raw_pad(i);
            }
        }
        self.pad_feed = Some(feed);
    }
}

/// A session input (`GwRbInput`, raw controller form) as the HSD pad layer's raw sample.
pub fn raw_pad(i: &RbInput) -> input::RawPad {
    input::RawPad {
        button: i.buttons & 0xFFFF,
        stick_x: i.raw[0],
        stick_y: i.raw[1],
        sub_x: i.raw[2],
        sub_y: i.raw[3],
        analog_l: i.pad_l,
        analog_r: i.pad_r,
    }
}

/// The scenario script's sample as a session input (the "controller" of a scripted player).
pub fn script_input(port: usize, g: i32) -> RbInput {
    let r = input::pad_fixed(port, g);
    RbInput {
        present: 1,
        is_raw: 1,
        confirmed: 1,
        buttons: r.button & 0xFFFF,
        raw: [r.stick_x, r.stick_y, r.sub_x, r.sub_y],
        pad_l: r.analog_l,
        pad_r: r.analog_r,
        ..RbInput::default()
    }
}
