// The VM's run-time half: [`Actor`] (fixed-size state) and [`step`]. Included by `vm.rs`.

// ------------------------------------------------------------------------------------------------ actor state

/// One call frame: pc and the open loops (body start, iterations left; -1 = forever).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Fr {
    pub pc: u32,
    pub nl: u32,
    pub lb: [u32; LOOPS],
    pub ll: [i32; LOOPS],
}
const FR0: Fr = Fr { pc: 0, nl: 0, lb: [0; LOOPS], ll: [0; LOOPS] };

/// A thread: a call stack and the frame it resumes on.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Th {
    pub nf: u32,
    /// resume when tick >= resume (fractional waits keep their fraction)
    pub resume: f32,
    pub fr: [Fr; FRAMES],
}
pub const TH0: Th = Th { nf: 0, resume: 0.0, fr: [FR0; FRAMES] };

/// A kinetic slot.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Kin {
    /// -1 = off
    pub kind: i32,
    /// direction in degrees; `dir_set` = 0 means "along the facing"
    pub ang: f32,
    pub dir_set: u32,
    pub spd: f32,
    pub acc: f32,
    pub max: f32,
    /// 1 = the record finished (decayed to zero / reached its limit)
    pub done: u32,
    /// 1 = paused by 6605, resumed by 6604
    pub paused: u32,
    /// the engine kind (0-24; for a record unit its table entry's word 0): the per-frame update branches on it
    /// (0x81A1D1F8, kind table): 0 / 8-13 directed, 1 / 14-19 / 23 / 24 the motion's root movement, 2-7, 20,
    /// 21 / 22 their own updates
    pub ek: i32,
    /// the lower speed limit (record word 5; f32::MIN = none)
    pub min: f32,
    /// 1 = started from a file record (44000, or 43001 for a built-in kind)
    pub rec: u32,
    /// the record's upper-limit level coefficient (word 8), for 661A's new limit
    pub max_l: f32,
    /// 1 = heading for the placement point (6602): stops on it
    /// the unit's facing reference (record word 10; set by 6616 / 6617): the rotation kinds compare it with the
    /// facing [L: kept, not read; the VM's rotation kinds use the facing directly]
    pub face: f32,
    pub seek: u32,
    /// kinds 21 / 22: the frame on the placement path (unit [off]) and the record's path mode (word 12, unit
    /// [off]: 1 = loop, else the path's end finishes the unit)
    pub pf: f32,
    pub pmode: i32,
    /// kinds 21 / 22: the start's second path word (unit [off], read by 6613): a 6602 / 6603's argument 6 as an
    /// integer (default 0); another start of a path kind gives 0 [M: their start vectors are the module's constants]
    pub pn: f32,
}
pub const KIN0: Kin = Kin { kind: -1, ang: 0.0, dir_set: 0, spd: 0.0, acc: 0.0, max: f32::MAX, done: 0, paused: 0, ek: -1, min: f32::MIN, rec: 0, max_l: 0.0, face: 0.0, seek: 0, pf: 0.0, pmode: 0, pn: 0.0 };

/// The placement path a Subspace placement hands its enemy (create parameter [off], stored at Enemy [off] and read
/// as value 41000): a stage gimmick of class grGimmickMotionPath. `frames` is its frame count ([off]); `pos(id, f)`
/// is its position at frame f (0x81C6EB5C sets the frame, 0x81C6EB6C reads the animated node's position). The host
/// owns the stage data; `id` is its handle.
#[derive(Clone, Copy, Debug)]
pub struct Path {
    pub id: usize,
    pub frames: u32,
    pub pos: fn(usize, f32) -> [f32; 3],
}

/// A presentation event for the host: `ins` is the instruction (its id and arguments are in the program).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Ev {
    pub kind: u32,
    pub ins: u32,
    pub v: i32,
}
/// A forwarded command (`ins`).
pub const EV_CMD: u32 = 1;
/// Status changed to `v`.
pub const EV_STATUS: u32 = 2;
/// Motion changed to `v`.
pub const EV_MOTION: u32 = 3;
/// Stage event `v` (6505).
pub const EV_STAGE: u32 = 4;
/// The enemy removed itself (6500).
pub const EV_REMOVE: u32 = 5;
/// A hurt state changed (0605 / 0607 / 0C02 / 0C03 / 1400 / 1401).
pub const EV_HURT: u32 = 6;
/// A weapon / article create (650E rec 0 [joint]): `v` = the record's param id as the argument resolves it; `ins` is the
/// 650E (the host reads its joint argument from the program, -1 when it has none). The host builds the projectile from
/// its local record data keyed by that id.
pub const EV_WEAPON: u32 = 7;
/// A stand-in fired (diagnostic only, no game effect): `v` = 1 a set goal lies beyond the arena box or a wall on axis
/// `ins` (0 x, 1 y), so arrival is taken at the limit (see `arrived`).
pub const EV_NOTE: u32 = 8;
/// A status exit called the shared statusAnimCmd_CommonCatchProcExit (the catch ends: the host drops a held victim
/// unless a throw is pending).
pub const EV_CATCH_EXIT: u32 = 9;
/// 661E: turn a joint about z to the units' movement angle: `ins` = the instruction (the host reads the joint argument
/// from the program), `v` = the angle in degrees as f32 bits.
pub const EV_AIM: u32 = 10;
/// A rule fired (only while `Input::trace`): `ins` = the rule index, `v` = its slot (-1: none).
pub const EV_RULE: u32 = 11;

/// Everything that changes during a fight. Plain data, fixed size, `Copy`: a snapshot is a copy.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Actor {
    pub live: u32,
    pub tick: u32,
    pub status: i32,
    pub prev_status: i32,
    pub status_t0: u32,
    pub motion: i32,
    pub motion_t0: u32,
    /// the second animated part's motion (0408 part motion blend; -1 none) and its start tick
    pub part_motion: i32,
    pub part_t0: u32,
    /// the placement point (engine vector 41000, the enemy's create parameter): the host's, else where the actor
    /// stood on its first frame; `home_set` = 1 once taken
    pub home: [f32; 2],
    pub home_set: u32,
    pub gen_status: u32,
    pub gen_motion: u32,
    pub pending: i32,
    pub strength: i32,
    pub difficulty: i32,
    pub rng: u32,
    pub lock: i32,
    pub last_slot: i32,
    pub eval_slot: i32,
    pub pos: [f32; 3],
    pub lr: f32,
    /// goal x / y; `goal_set` bit 0 = x set, bit 1 = y set
    pub goal: [f32; 2],
    pub goal_set: u32,
    /// whole-body hurt state (0 normal, 2 intangible), per-hurtbox intangible mask, hurtbox groups off
    pub hurt: u32,
    pub hurt_mask: u32,
    pub hurt_groups_off: u32,
    pub shadow: u32,
    pub active: u32,
    pub situation: i32,
    pub search_reset: u32,
    pub dproc: u32,
    pub rules_on: [u64; 2],
    /// the targets' positions recorded when found (R10009 / R10013 / 6908 / 690A)
    pub found: [[f32; 2]; 4],
    pub wb: u64,
    pub tb: u32,
    pub wi: [i32; NI],
    pub wf: [f32; NF],
    pub ti: [i32; TI],
    pub tf: [f32; TF],
    pub kin: [Kin; KIN],
    pub th: [Th; THREADS],
    /// the enemy's four frame timers: start (NONE = never started), pause frame (0 = running),
    /// frames spent paused
    pub timers: [[u32; 3]; 4],
    /// how far the last frame's movement carried the actor (engine value 1050)
    pub moved: f32,
    /// the model's rotation the rotation units (kinds 2-7) turn, degrees about x / y / z, for the host to draw;
    /// it does not move the actor
    pub rot: [f32; 3],
    pub ev_n: u32,
    pub ev: [Ev; EVQ],
}

pub const OFF: Actor = Actor {
    live: 0,
    tick: 0,
    status: -1,
    prev_status: -1,
    status_t0: 0,
    motion: -1,
    motion_t0: 0,
    part_motion: -1,
    part_t0: 0,
    home: [0.0; 2],
    home_set: 0,
    gen_status: 0,
    gen_motion: 0,
    pending: -1,
    strength: 1,
    difficulty: 5,
    rng: 1,
    lock: -1,
    last_slot: -1,
    eval_slot: -1,
    pos: [0.0; 3],
    lr: 1.0,
    goal: [0.0; 2],
    goal_set: 0,
    hurt: 0,
    hurt_mask: 0,
    hurt_groups_off: 0,
    shadow: 1,
    active: 1,
    situation: 0,
    search_reset: 0,
    dproc: NONE,
    rules_on: [0; 2],
    found: [[0.0; 2]; 4],
    wb: 0,
    tb: 0,
    wi: [0; NI],
    wf: [0.0; NF],
    ti: [0; TI],
    tf: [0.0; TF],
    kin: [KIN0; KIN],
    th: [TH0; THREADS],
    timers: [[NONE, 0, 0]; 4],
    moved: 0.0,
    rot: [0.0; 3],
    ev_n: 0,
    ev: [Ev { kind: 0, ins: 0, v: 0 }; EVQ],
};

/// The host's view of the world for one frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    /// debug: report every rule that fires (EV_RULE); presentation only
    pub trace: bool,
    /// target candidates (players): live position, None = absent
    pub targets: [Option<[f32; 2]>; 4],
    /// damage taken this frame (0 = not hit) and the reaction frames the hit's knockback gives
    pub damage: f32,
    pub reaction: i32,
    /// on the ground; the floor height under the actor (0808)
    pub ground: bool,
    pub floor_y: f32,
    /// ground-collision mask (R12), territory-edge mask (R16), no floor ahead per side (R22)
    pub col: u32,
    pub territory: u32,
    pub cliff: [bool; 2],
    pub offscreen: bool,
    pub out_of_area: bool,
    /// the ground module's surface queries R63-R69 (bit n = R(63 + n)), answered by the host's collision (ground-module
    /// vfuncs, positions in the local engine-ops.md; sora_melee
    /// requirement switch). Their arguments (a surface kind, a line id, a side mask) are in the instruction
    pub ground_q: u32,
    /// the host's stage collision around the actor: a floor under it (`floor_y` is then its height) and the nearest
    /// wall each side (`wall[0]` left, `wall[1]` right, valid when bit 0 / 1 of `wall_on` is set). Movement stops on
    /// them like on the arena box
    pub floor_on: bool,
    pub wall: [f32; 2],
    pub wall_on: u32,
    /// this frame's root movement of the boss's playing motion (model units, unmirrored), when the host has the
    /// animation: units of the root-motion kinds move by it (rotated by the unit's angle and mirrored by the facing)
    pub root_on: bool,
    /// the host caught a fighter this frame (the catch callback): the actor's work flag 6 ("caught") is set
    pub caught: bool,
    /// a victim is held (R60, the catch module's query)
    pub holding: bool,
    /// the actor was reflected (R75, the reflect module's "reflector id set" query): from the frame a reflector's hit
    /// lands until the module is reset
    pub reflected: bool,
    /// the placement point (a Subspace spawn point); None: the actor's first-frame position
    pub home: Option<[f32; 2]>,
    /// the placement path (value 41000 as a path object); None: kinds 21 / 22 head straight for `home` [L]
    pub path: Option<Path>,
    pub root_move: [f32; 2],
    /// the model's scale (the model module's scale query); 0 = 1. Kind 23 multiplies its move by it
    pub root_scale: f32,
    /// kind 24's source: the motion module's second translation query (in place of the one kind 1 reads), e.g. a
    /// partial / added motion's root movement; same frame and units
    pub root_move2: [f32; 2],
    /// the area module's contacts (R10055): bit n = the actor touches area n of the area kind R10055 asks about (13)
    pub area: u32,
    /// the article manager's free room per article kind 0..3 (100D): the kind's maximum count less the live ones
    pub article_room: [i32; 4],
    /// the link module has a partner (R56)
    pub linked: bool,
    /// the enemy manager has a free enemy slot (0x81A0CEF0: 0x81A0CE10 finds an entry whose first byte is 0)
    pub enemy_room: bool,
    /// the stage-wide group manager (sora_melee 0x812331C4 count / 0x81233208 entry) as R10052 asks it: per group id,
    /// its entry count and a bit per entry the manager reports present. A group not listed has no entries
    pub groups: [Group; 4],
    /// the actor's linked enemy (the one the enemy manager holds under the actor's link id) exists and its work flag 0
    /// (dead, 0x12000000) is clear (R10053)
    pub partner_alive: bool,
    /// per target (player slot 0..3): its facing (the posture module's lr; 0 = +1) and its model scale (0 = 1), the
    /// per-target factors 6901 / 6902 apply to their (x, y) offset
    pub target_lr: [f32; 4],
    pub target_scale: [f32; 4],
    /// R10059: a flag byte of a global sora_melee object (the one 0x81C21B30 creates and clears), set by sora_melee
    /// 0x81C51C04 when one of that object's timers reaches its value [L: meaning not traced; a VS fight leaves it clear]
    pub global_flag: bool,
    /// R10054: the enemy manager's live entries (0x81A0CF18: entries whose first byte is set)
    pub enemy_live: u32,
    /// R10054: per value a, the live entries whose enemy's id word equals a (0x81A0CF8C) [L: the word's meaning is
    /// not traced]; `gated` is the static object's answer (its vfunc 0x0C on a) that forces the count to 0. An a not
    /// listed counts 0
    pub enemy_ids: [EnemyCount; 4],
    /// R10057: a live enemy (in the manager, its enemy present, work flag 0 clear) is linked to the actor (its link id
    /// is the actor's id in the enemy manager)
    pub child_alive: bool,
    /// model nodes' global rotations (R10024-R10047, the model module's getNodeGlobalMtx); a node not listed turns by
    /// the facing [L]
    pub nodes: [NodeRot; 2],
    /// the enemy's centre (getCenterPos: the global position of the model node param 0x59E6 names), for 1051 / 1052
    /// and R10024-R10047; None: the actor's position [L]
    pub center: Option<[f32; 2]>,
    /// the article manager's live articles, in its list's order (soGenerateArticleManageModuleImpl, accessor slot
    /// [off]): R21 and R24 walk it
    pub articles: [Article; 8],
    /// R28: the enemy class's article table (the per-class object the Enemy constructor takes, e.g. an
    /// soArticleMediatorImpl over its weapon pools) per article kind: bit a = its vfunc [off] answers true for kind a
    /// (a pooled instance of the kind answers 0 to its vfunc [off]) [M: read as "a free instance"]. Kinds at or past
    /// the table's count are clear (the null table has none)
    pub article_pool: u32,
    /// 21012: the linked enemy's status (the actor's manager entry's link handle, its object's work-module status
    /// getter); None: the chain is broken (value -1)
    pub partner_status: Option<i32>,
    /// 6519 var (0x81A09860 -> 0x81A0B438 -> 0x81A0B1E8, through the enemy manager 0x81A0C804): the linked enemy
    /// whose variable the command copies into this actor's same variable at once (int, float or flag by its kind), so
    /// the condition that follows reads the copied value (the Falcon Flyer's rule 3 reads Meta Ridley's attack id).
    /// [M from use: the copy direction, partner -> actor, is the only one the two files' scripts work with; the ship's
    /// file has the 6519 and Meta Ridley's writes the variable.] None: no linked enemy, the command only forwards.
    pub partner_vars: Option<PartnerVars>,
}

/// The linked enemy's work variables as 6519 reads them (the host copies them from its actor before this step).
#[derive(Clone, Copy, Debug)]
pub struct PartnerVars {
    pub wi: [i32; NI],
    pub wf: [f32; NF],
    pub wb: u64,
}

impl PartnerVars {
    pub fn of(a: &Actor) -> Self {
        PartnerVars { wi: a.wi, wf: a.wf, wb: a.wb }
    }
}

/// One live article for R21 / R24: `id` its vfunc [off] answer (an article id [L]), `q` its vfunc [off] with 0
/// answers 1 [L: meaning not traced].
#[derive(Clone, Copy, Debug, Default)]
pub struct Article {
    pub on: bool,
    pub id: i32,
    pub q: bool,
}

/// One model node's global rotation (rows of the 3x3 part of its matrix) for R10024-R10047; `id` is the node id the
/// script names (for the by-name ids, the argument's value).
#[derive(Clone, Copy, Debug, Default)]
pub struct NodeRot {
    pub on: bool,
    pub id: i32,
    pub rot: [[f32; 3]; 3],
}

/// One R10054 count (0x81A0CF8C).
#[derive(Clone, Copy, Debug, Default)]
pub struct EnemyCount {
    pub id: i32,
    pub live: u32,
    pub gated: bool,
}

/// One group of the stage-wide group manager (R10052).
#[derive(Clone, Copy, Debug, Default)]
pub struct Group {
    pub id: i32,
    pub count: u32,
    /// bit n: entry n is present
    pub present: u32,
}

// work variables the engine itself maintains (engine.md 5 / engine-ops.md)
const WF_HP: usize = 1;
const WF_HP_MAX: usize = 3;
const WF_L: usize = 12;
const WB_DEAD: u32 = 0;
const WB_IN_DAMAGE: u32 = 4;
/// work flag 6: a fighter was caught (set by the catch callback; the scripts' "if caught" reads it)
const WB_CAUGHT: u32 = 6;
const WB_FIGURE_DEAD: u32 = 15;
const WB_FIGURE_MODE: u32 = 18;
/// work flag 17: R10057 sets it, a live enemy linked to the actor clears it (0x81A06EB4 / 0x81A06DDC)
const WB_NO_CHILD: u32 = 17;
const TB_REACTION_END: u32 = 0;
const TB_DAMAGED: u32 = 2;
const TB_TRUE: u32 = 4;
const TB_OUT_OF_AREA: u32 = 8;

fn level_f(strength: i32, difficulty: i32) -> f32 {
    crate::level::level(strength, difficulty).raw() as f32 / 60000.0
}

impl Actor {
    /// FNV-1a over the simulation state (the event queue excluded: presentation, rebuilt every frame).
    pub fn vm_hash(&self) -> u32 {
        let mut h: u32 = 0x811C_9DC5;
        let mut mix = |v: u32| {
            for b in v.to_le_bytes() {
                h ^= b as u32;
                h = h.wrapping_mul(0x0100_0193);
            }
        };
        let a = self;
        for v in [a.live, a.tick, a.status as u32, a.prev_status as u32, a.status_t0, a.motion as u32, a.motion_t0, a.part_motion as u32, a.part_t0, a.home[0].to_bits(), a.home[1].to_bits(), a.home_set] {
            mix(v);
        }
        for v in [a.pending as u32, a.strength as u32, a.difficulty as u32, a.rng, a.lock as u32, a.last_slot as u32] {
            mix(v);
        }
        for v in a.pos {
            mix(v.to_bits());
        }
        mix(a.moved.to_bits());
        for v in a.rot {
            mix(v.to_bits());
        }
        for v in [a.lr.to_bits(), a.goal[0].to_bits(), a.goal[1].to_bits(), a.goal_set, a.hurt, a.hurt_mask] {
            mix(v);
        }
        for v in [a.hurt_groups_off, a.shadow, a.active, a.situation as u32, a.dproc, a.wb as u32, (a.wb >> 32) as u32] {
            mix(v);
        }
        for r in a.rules_on {
            mix(r as u32);
            mix((r >> 32) as u32);
        }
        for f in a.found {
            mix(f[0].to_bits());
            mix(f[1].to_bits());
        }
        mix(a.tb);
        a.timers.iter().flatten().for_each(|&v| mix(v));
        a.wi.iter().for_each(|&v| mix(v as u32));
        a.wf.iter().for_each(|v| mix(v.to_bits()));
        a.ti.iter().for_each(|&v| mix(v as u32));
        a.tf.iter().for_each(|v| mix(v.to_bits()));
        for k in &a.kin {
            for v in [k.kind as u32, k.ang.to_bits(), k.dir_set, k.spd.to_bits(), k.acc.to_bits(), k.max.to_bits(), k.done, k.paused, k.pf.to_bits(), k.pmode as u32, k.pn.to_bits()] {
                mix(v);
            }
        }
        for t in &a.th {
            mix(t.nf);
            mix(t.resume.to_bits());
            for f in &t.fr[..t.nf as usize] {
                mix(f.pc);
                mix(f.nl);
                for i in 0..f.nl as usize {
                    mix(f.lb[i]);
                    mix(f.ll[i] as u32);
                }
            }
        }
        h
    }

    fn wb_get(&self, i: u32) -> bool {
        i < 64 && self.wb >> i & 1 != 0
    }
    fn wb_set(&mut self, i: u32, on: bool) {
        if i < 64 {
            if on {
                self.wb |= 1 << i
            } else {
                self.wb &= !(1 << i)
            }
        }
    }
    fn tb_get(&self, i: u32) -> bool {
        i < 32 && self.tb >> i & 1 != 0
    }
    fn tb_set(&mut self, i: u32, on: bool) {
        if i < 32 {
            if on {
                self.tb |= 1 << i
            } else {
                self.tb &= !(1 << i)
            }
        }
    }
    fn rule_on(&self, i: usize) -> bool {
        i < MAX_RULES && self.rules_on[i >> 6] >> (i & 63) & 1 != 0
    }
    fn set_rule(&mut self, i: usize, on: bool) {
        if i < MAX_RULES {
            if on {
                self.rules_on[i >> 6] |= 1 << (i & 63)
            } else {
                self.rules_on[i >> 6] &= !(1 << (i & 63))
            }
        }
    }
    pub fn level(&self) -> f32 {
        self.wf[WF_L]
    }
    pub fn hp(&self) -> f32 {
        self.wf[WF_HP]
    }
    pub fn dead(&self) -> bool {
        self.wb_get(WB_DEAD)
    }
    /// HP percent, engine value 0x402: trunc(100 * HP / max HP).
    pub fn hp_pct(&self) -> i32 {
        let m = self.wf[WF_HP_MAX];
        if m <= 0.0 {
            100
        } else {
            (100.0 * self.wf[WF_HP] / m) as i32
        }
    }
    pub fn events(&self) -> &[Ev] {
        &self.ev[..(self.ev_n as usize).min(EVQ)]
    }
}

// ------------------------------------------------------------------------------------------------ runtime

struct Cx<'a> {
    p: &'a Program,
    a: &'a mut Actor,
    i: &'a Input,
    depth: u32,
}

/// Start an actor: L from (strength, difficulty), HP = tuning 0 + tuning 1 * L, the init script, then `status`.
/// `figure` arms the threshold ("Figure") death bosses use (work flag 18).
pub fn begin(p: &Program, a: &mut Actor, strength: i32, difficulty: i32, seed: u32, status: i32, figure: bool) {
    *a = OFF;
    a.live = 1;
    a.strength = strength;
    a.difficulty = difficulty;
    a.rng = seed | 1;
    a.wf[WF_L] = level_f(strength, difficulty);
    let hp = p.tuning[0] + p.tuning[1] * a.wf[WF_L];
    a.wf[WF_HP] = hp;
    a.wf[WF_HP_MAX] = hp;
    a.wb_set(WB_FIGURE_MODE, figure);
    for (i, r) in p.rules.iter().enumerate() {
        a.set_rule(i, r.on);
    }
    for &(raw, x) in &p.inits {
        let v = crate::script::Var::from_raw(raw);
        let id = v.id as usize;
        match (v.mem, v.kind) {
            (1, 0) if id < NI => a.wi[id] = x as i32,
            (1, 1) if id < NF => a.wf[id] = x,
            (1, 2) => a.wb_set(id as u32, x != 0.0),
            _ => {}
        }
    }
    let inp = Input::default();
    let mut cx = Cx { p, a, i: &inp, depth: 0 };
    cx.run_now(p.init);
    cx.change_status(status);
}

/// Start movement record `kind` on `slot` as 6600 does (no angle), outside a frame: for tests of the record start.
#[doc(hidden)]
pub fn start_unit(p: &Program, a: &mut Actor, i: &Input, slot: i32, kind: i32) {
    let mut cx = Cx { p, a, i, depth: 0 };
    cx.kin_start(slot, kind, None);
}

/// One frame. Never allocates.
pub fn step(p: &Program, a: &mut Actor, i: &Input) {
    if a.live == 0 {
        return;
    }
    a.ev_n = 0;
    let mut cx = Cx { p, a, i, depth: 0 };
    cx.input();
    cx.ai();
    cx.run_thread(T_LOOP);
    for t in 1..=9 {
        cx.run_thread(t);
    }
    if cx.a.dproc != NONE {
        cx.run_thread(T_DPROC);
    }
    cx.kinetics();
    cx.a.tick = cx.a.tick.wrapping_add(1);
}

impl<'a> Cx<'a> {
    /// Queue an event. The queue is sized above the worst frame measured over all files (vm_events_max), so nothing is
    /// dropped; `ev_n` counts every event, and an overflow is a debug assertion.
    fn ev(&mut self, kind: u32, ins: u32, v: i32) {
        let n = self.a.ev_n as usize;
        debug_assert!(n < EVQ, "vm event queue overflow ({} events in one frame)", n + 1);
        if n < EVQ {
            self.a.ev[n] = Ev { kind, ins, v };
        }
        self.a.ev_n += 1;
    }
    fn rnd(&mut self, n: u32) -> u32 {
        self.a.rng = self.a.rng.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (self.a.rng >> 8) % n.max(1)
    }

    /// A float draw in [0, 1) from the VM's seeded generator (the engine's randf, 8003FB64, for 35790).
    fn rndf(&mut self) -> f32 {
        self.a.rng = self.a.rng.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (self.a.rng >> 8) as f32 / 16_777_216.0
    }

    fn input(&mut self) {
        let i = *self.i;
        if i.damage > 0.0 {
            self.a.tb_set(TB_DAMAGED, true);
            self.a.wi[0] = i.reaction;
            let hp_max = self.a.wf[WF_HP_MAX];
            if self.a.wf[WF_HP] >= 0.0 {
                let hp = (self.a.wf[WF_HP] - i.damage).max(0.0);
                self.a.wf[WF_HP] = hp;
                if self.a.wb_get(WB_FIGURE_MODE) && hp_max > 0.0 {
                    // fn 0x81A11BCC: the HP percent at or below the general block's threshold is the death
                    if (100.0 * hp / hp_max) <= self.p.general[0x9C / 4] {
                        self.a.wf[WF_HP] = 0.0;
                        self.a.wb_set(WB_FIGURE_DEAD, true);
                        self.a.wb_set(WB_DEAD, true);
                    }
                } else if hp <= 0.0 {
                    self.a.wb_set(WB_DEAD, true);
                }
            }
        }
        self.a.tb_set(TB_OUT_OF_AREA, i.out_of_area);
        if self.a.home_set == 0 {
            self.a.home = i.home.unwrap_or([self.a.pos[0], self.a.pos[1]]);
            self.a.home_set = 1;
        }
        if i.caught {
            self.a.wb_set(WB_CAUGHT, true);
        }
        if i.ground {
            self.a.situation = 0;
        }
    }

    /// The AI evaluation (fn 0x81A2B1B0): the -100 pre-pass, then the damage window or the default window; the
    /// AreaOut rule (-300) when the host reports the actor out of the area.
    fn ai(&mut self) {
        if self.i.out_of_area {
            self.evaluate(-300, -300);
        }
        let s = self.a.gen_status;
        self.evaluate(-100, -100);
        if self.a.gen_status != s {
            return;
        }
        if self.a.wb_get(WB_IN_DAMAGE) || self.a.tb_get(TB_REACTION_END) {
            self.evaluate(-250, -150);
        } else {
            self.evaluate(-50, 250);
        }
    }

    /// One pass over the rules inside [lo, hi] (a slice of the priority-sorted index: rules outside the window are
    /// never visited). Every enabled rule's condition runs; the lowest priority among the true ones wins.
    fn evaluate(&mut self, lo: i32, hi: i32) {
        let p = self.p;
        let from = p.by_prio.partition_point(|&r| p.rules[r as usize].prio < lo);
        let to = p.by_prio.partition_point(|&r| p.rules[r as usize].prio <= hi);
        if from >= to {
            return;
        }
        let mut best = i32::MAX;
        let mut hits = [(0u16, -1i8); 16];
        let mut nh = 0usize;
        for &ri in &p.by_prio[from..to] {
            let r = &p.rules[ri as usize];
            if !self.a.rule_on(ri as usize) {
                continue;
            }
            if r.mask & 0x0F != 0 {
                for s in 0..4i8 {
                    if r.mask & (1 << s) != 0 && self.i.targets[s as usize].is_some() && self.check(r.cond, s as i32) {
                        if r.prio < best {
                            best = r.prio;
                            nh = 0;
                        }
                        if r.prio == best && nh < hits.len() {
                            hits[nh] = (ri, s);
                            nh += 1;
                        }
                    }
                }
            } else if self.check(r.cond, -1) {
                if r.prio < best {
                    best = r.prio;
                    nh = 0;
                }
                if r.prio == best && nh < hits.len() {
                    hits[nh] = (ri, -1);
                    nh += 1;
                }
            }
        }
        self.a.eval_slot = -1;
        if nh == 0 {
            return;
        }
        // several winning slots: draw one by the tuning block's slot weights (words 2-5), the last slot cut by the
        // repeat factor (word 29)
        let mut slots = [false; 4];
        for h in &hits[..nh] {
            if h.1 >= 0 {
                slots[h.1 as usize] = true;
            }
        }
        let n_slots = slots.iter().filter(|&&x| x).count();
        let chosen = if n_slots > 1 {
            let mut w = [0u32; 4];
            let mut total = 0u32;
            for s in 0..4 {
                if slots[s] {
                    let mut x = p.tuning[2 + s];
                    if self.a.last_slot == s as i32 {
                        x *= p.tuning[29];
                    }
                    w[s] = (x.max(0.0) * 1000.0) as u32;
                    total += w[s];
                }
            }
            let mut r = self.rnd(total.max(1));
            let mut pick = -1;
            for s in 0..4 {
                if w[s] > r {
                    pick = s as i8;
                    break;
                }
                r -= w[s];
            }
            pick
        } else {
            -1
        };
        let h = hits[..nh].iter().find(|h| h.1 < 0 || chosen < 0 || h.1 == chosen).copied().unwrap_or(hits[0]);
        self.fire(h.0 as usize, h.1 as i32);
    }

    fn check(&mut self, cond: u32, slot: i32) -> bool {
        if cond == NONE {
            return true;
        }
        self.a.tb_set(TB_TRUE, false);
        self.a.eval_slot = slot;
        if slot >= 0 {
            self.a.ti[0] = slot;
        }
        self.run_now(cond);
        self.a.tb_get(TB_TRUE)
    }

    fn fire(&mut self, ri: usize, slot: i32) {
        if self.i.trace {
            self.ev(EV_RULE, ri as u32, slot);
        }
        let r = self.p.rules[ri];
        if slot >= 0 {
            self.a.ti[0] = slot;
            self.a.last_slot = slot;
        }
        self.a.eval_slot = slot;
        self.a.pending = -1;
        self.run_now(r.action);
        if r.lock && slot >= 0 {
            self.lock(slot);
        }
        self.a.eval_slot = -1;
        if r.status != -1 {
            self.a.pending = r.status;
        }
        self.apply_pending();
    }

    fn lock(&mut self, s: i32) {
        self.a.lock = s;
        if let Some(t) = self.i.targets.get(s as usize).copied().flatten() {
            self.a.found[s as usize & 3] = t;
        }
    }

    fn apply_pending(&mut self) {
        let s = self.a.pending;
        if s >= 0 {
            self.a.pending = -1;
            self.change_status(s);
        }
    }

    fn change_status(&mut self, s: i32) {
        if self.depth > 8 {
            return;
        }
        self.depth += 1;
        let p = self.p;
        // the status kind changes before the old status's exit runs: an exit reading the current status (value
        // 20001) sees the new one (Tabuu's ChainStart exit: "if the status is not 37, CommonCatchProcExit")
        let old = self.a.status;
        self.a.prev_status = old;
        self.a.status = s;
        if old >= 0 {
            let ex = p.status_exit.get(old as usize).copied().unwrap_or(0);
            self.run_now(ex);
        }
        self.a.status = s;
        self.a.status_t0 = self.a.tick;
        self.a.gen_status = self.a.gen_status.wrapping_add(1);
        self.a.th[T_LOOP] = TH0;
        self.a.pending = -1;
        self.ev(EV_STATUS, 0, s);
        let en = p.status_enter.get(s as usize).copied().unwrap_or(0);
        self.run_now(en);
        let lp = p.status_loop.get(s as usize).copied().unwrap_or(0);
        self.a.th[T_LOOP] = thread_at(lp, self.a.tick);
        let pend = self.a.pending;
        if pend >= 0 {
            self.a.pending = -1;
            self.change_status(pend);
        }
        self.depth -= 1;
    }

    fn change_motion(&mut self, m: i32) {
        self.a.motion = m;
        self.a.motion_t0 = self.a.tick;
        self.a.gen_motion = self.a.gen_motion.wrapping_add(1);
        self.ev(EV_MOTION, 0, m);
        let tl = self.p.timelines.get(m as usize).copied().unwrap_or([0; 9]);
        for t in 0..9 {
            self.a.th[1 + t] = thread_at(tl[t], self.a.tick);
        }
    }

    /// (frame in the motion, ended)
    fn motion_frame(&self) -> (i32, bool) {
        let m = self.a.motion;
        let el = self.a.tick.wrapping_sub(self.a.motion_t0) as i32;
        let len = self.p.motion_len.get(m as usize).copied().unwrap_or(60).max(1) as i32;
        if self.p.motion_loop.get(m as usize).copied().unwrap_or(false) {
            let f = el % len;
            (f, f == len - 1)
        } else {
            (el.min(len - 1), el >= len - 1)
        }
    }

    // ---------------------------------------------------------------- values

    fn val(&self, x: A) -> f32 {
        let a = &*self.a;
        match x {
            A::K(k) => k,
            A::Wi(i) => a.wi[i as usize] as f32,
            A::Wf(i) => a.wf[i as usize],
            A::Wb(i) => a.wb_get(i as u32) as i32 as f32,
            A::Ti(i) => a.ti[i as usize] as f32,
            A::Tf(i) => a.tf[i as usize],
            A::Tb(i) => a.tb_get(i as u32) as i32 as f32,
            A::Eng(id) => self.engine(id),
            A::Rec(r) => self.p.recs.get(r as usize).map_or(0.0, |x| x.0 as f32),
            _ => 0.0,
        }
    }
    fn engine(&self, id: u32) -> f32 {
        let a = &*self.a;
        match id {
            3 => a.pos[0],
            // 8 / 9 (value switch sora_melee 0x8108C0D8): the posture module's facing, 9 negated
            8 => a.lr,
            9 => -a.lr,
            // 15: model query, the same accessor 21 reads
            15 => a.pos[2],
            4 => a.pos[1],
            // 0x15: model query, the z of every 0509 [read as the posture z]
            21 => a.pos[2],
            1026 => a.hp_pct() as f32,
            // 1049 (0x81A17018): tuning word 18 (value 3013, the enemy's data block the value switch reads first)
            // scaled by (90 - t) / 90, t = the angle in degrees between the move and the ground (kinetic module,
            // written at 0x81A16B8C); 1 outside 0..90. Read as flat ground, t = 0 [L: no slope is modelled]
            1049 => self.p.tuning[18],
            // 1050 (value switch case): the distance between the ground module's two positions (current and
            // previous): how far the last frame moved the actor
            1050 => a.moved,
            // 1003 (0x81A216CC -> 0x81A161B4 on unit 3): unit 3's angle as 0x81A1EC18 reads it (as 6614's term: a stored
            // 35790 reads as 270; the markers 35782-35787 [L: not traced, read as stored])
            1003 => {
                let x = a.kin[3].ang;
                if x as i32 == 35790 { 270.0 } else { x }
            }
            // 1051 / 1052 (0x81A22060 / 0x81A22098): the enemy's centre (getCenterPos 0x81A0A88C: the model module's
            // getNodeGlobalPosition, vfunc 0x98, on the node of param 0x59E6), x / y; the host's `center`
            1051 => self.center()[0],
            1052 => self.center()[1],
            // 1020-1025 (value switch 0x81A214C8, cases -> 0x81A16234 -> 0x81A1E6A4): unit 2-7's speed (the record's
            // speed word, which 660E negates), 0 for an off unit; units of modes 2-4 are signed by the facing. Read here
            // as: a unit without a direction moves along the facing, so its speed is signed by it
            1020..=1025 => {
                let k = &a.kin[(id - 1018) as usize & (KIN - 1)];
                if k.kind < 0 {
                    0.0
                } else if k.dir_set == 0 && k.kind >= 14 {
                    a.lr * k.spd
                } else {
                    k.spd
                }
            }
            1028 => a.goal[0],
            1029 => a.goal[1],
            3056 => self.p.general[2],
            3057 => self.p.general[0x9C / 4],
            20001 => a.status as f32,
            20003 => a.prev_status as f32,
            // 20000 (sora_melee value switch 0x8108BC8C): the motion module's current motion; read
            // while a status's enter runs it is still the motion the last status played
            20000 => a.motion as f32,
            // 21000-21002 / 21008: timer 0-2 / 3 (0x81A0A468: now - start - paused). A timer no 6504 has started
            // reads the frames in the status [L: Rayquaza's 21002 rule has no 6504 in its file]
            21000 | 21001 | 21002 | 21008 => {
                let n = if id == 21008 { 3 } else { (id - 21000) as usize };
                let t = a.timers[n];
                if t[0] == NONE {
                    a.tick.wrapping_sub(a.status_t0) as f32
                } else {
                    let end = if t[1] != 0 { t[1] } else { a.tick };
                    end.wrapping_sub(t[0]).wrapping_sub(t[2]) as f32
                }
            }
            21004 => {
                if a.lock >= 0 {
                    a.lock as f32
                } else {
                    a.eval_slot as f32
                }
            }
            21007 => self.p.variant as f32,
            // 21009 / 21010 (value switch 0x81A22118, cases 0x5211 / 0x5212): enemy fields the constructor copies from
            // the create record (0x81A003E8: a word of it; 0x81A003F8: a byte). The byte is the strength (the level
            // getter 0x81A03678 reads it 1-5 with the difficulty; 6523's setter 0x81A037D8 writes it)
            21009 => self.p.place_word as f32,
            21010 => a.strength as f32,
            21012 => self.i.partner_status.map_or(-1.0, |s| s as f32),
            // 23000-23033 (0x81A227C0): words of the enemy's data blocks, placed by the local value table
            // (`Program::block_values`) [L: which block the second pointer is was not traced]. A word below 0x10000 is
            // an int, else a float.
            23000..=23099 => match self.p.block_values.iter().find(|v| v.0 == id) {
                Some(&(_, t, off)) => {
                    let w = be32(&self.p.data, if t == 0 { 0 } else { self.p.tuning_at } + off);
                    if w < 0x10000 {
                        w as f32
                    } else {
                        fbits(w)
                    }
                }
                None => 0.0,
            },
            _ => 0.0,
        }
    }
    fn set(&mut self, x: A, v: f32) {
        let a = &mut *self.a;
        match x {
            A::Wi(i) => a.wi[i as usize] = v as i32,
            A::Wf(i) => a.wf[i as usize] = v,
            A::Wb(i) => a.wb_set(i as u32, v != 0.0),
            A::Ti(i) => a.ti[i as usize] = v as i32,
            A::Tf(i) => a.tf[i as usize] = v,
            A::Tb(i) => a.tb_set(i as u32, v != 0.0),
            _ => {}
        }
    }
    fn arg(&self, ins: &Ins, k: u32) -> A {
        if k < ins.n as u32 {
            self.p.args[(ins.a + k) as usize]
        } else {
            A::K(0.0)
        }
    }
    fn av(&self, ins: &Ins, k: u32) -> f32 {
        self.val(self.arg(ins, k))
    }

    // ---------------------------------------------------------------- requirements

    fn slot_arg(&self, v: f32) -> i32 {
        let s = v as i32;
        if (0..4).contains(&s) {
            s
        } else if self.a.lock >= 0 {
            self.a.lock
        } else {
            self.a.eval_slot
        }
    }
    /// The point the search module's helpers build for slot `s` (0x81A25C60 / 0x81A25DF0, also 0x81A25A50 /
    /// 0x81A25268): the slot's target (`found`: the search snapshot, 0x81A25524; else the target now, 0x81A23A0C)
    /// plus the offset (x * ax, y) * scale (0x81A24278, rotation 0). ax is the target's facing when `flag` is set,
    /// else 1. Slot 0..3: a player (absent: point 0, ax 0, scale 1); 4..7: point 0, factors 1 [L: kinds 4..6 not
    /// modelled]; any other slot: point 0 and, for the snapshot, factors -1 (else ax 0, scale 1)
    fn slot_point(&self, s: i32, found: bool, x: f32, y: f32, flag: bool) -> [f32; 2] {
        let (t, ax, sc) = match s {
            0..=3 => {
                let i = s as usize;
                let live = self.target(s);
                let p = if found { live.map(|_| self.a.found[i]) } else { live };
                match p {
                    Some(p) => {
                        let lr = if self.i.target_lr[i] == 0.0 { 1.0 } else { self.i.target_lr[i] };
                        let sc = if self.i.target_scale[i] == 0.0 { 1.0 } else { self.i.target_scale[i] };
                        (p, lr, sc)
                    }
                    None => ([0.0; 2], 0.0, 1.0),
                }
            }
            4..=7 => ([0.0; 2], 1.0, 1.0),
            _ if found => ([0.0; 2], -1.0, -1.0),
            _ => ([0.0; 2], 0.0, 1.0),
        };
        let ax = if flag { ax } else { 1.0 };
        [t[0] + x * ax * sc, t[1] + y * sc]
    }

    /// The rotation part of a model node's global matrix (rows; column n is the node's axis n): the host's `nodes`
    /// entry for the node, else a turn about y by the facing (lr * 90 degrees) [L: no skeleton]
    /// The enemy's centre (getCenterPos 0x81A0A88C: a model node's global position): the host's `center`, else the
    /// actor's position [L: no skeleton]
    fn center(&self) -> [f32; 2] {
        self.i.center.unwrap_or([self.a.pos[0], self.a.pos[1]])
    }

    fn node_rot(&self, node: i32) -> [[f32; 3]; 3] {
        if let Some(n) = self.i.nodes.iter().find(|n| n.on && n.id == node) {
            return n.rot;
        }
        let s = self.a.lr;
        [[0.0, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, 0.0]]
    }

    fn target(&self, s: i32) -> Option<[f32; 2]> {
        if s < 0 {
            return None;
        }
        self.i.targets.get(s as usize).copied().flatten()
    }

    /// The requirement at args[k..] (the first is the `Req`).
    fn cond(&mut self, ins: &Ins, k: u32) -> bool {
        let A::Req { not, id } = self.arg(ins, k) else { return false };
        let x = |j: u32| self.av(ins, k + 1 + j);
        let a = &*self.a;
        let r = match id {
            1 | 2 => self.motion_frame().1,
            // R37 part: the second part's motion ended (AICheckBustPartsMotionEnd) [L: one part]
            37 => {
                let m = a.part_motion;
                m >= 0 && {
                    let len = self.p.motion_len.get(m as usize).copied().unwrap_or(60).max(1) as i32;
                    a.tick.wrapping_sub(a.part_t0) as i32 >= len - 1
                }
            }
            3 => a.situation == 0,
            // R6 (sora_melee 0x81078644): the situation module's state is 0 and the ground module
            // reports contact: standing on a floor [M]
            6 => a.situation == 0 && self.i.ground,
            4 => a.situation != 0,
            7 => {
                let (l, op, r) = (x(0), x(1) as i32, x(2));
                match op {
                    0 => l < r,
                    1 => l <= r,
                    2 => l == r,
                    3 => l != r,
                    4 => l >= r,
                    5 => l > r,
                    _ => false,
                }
            }
            8 => x(0) != 0.0,
            9 => a.lr == 1.0,
            10 => a.lr == -1.0,
            12 => self.i.col & x(0) as u32 != 0,
            15 => true,
            16 => self.i.territory & x(0) as u32 != 0,
            17 => false,
            // a player inside a search unit: read as "present in the slot" [L: the unit shapes are not modelled]
            18 => self.target(self.slot_arg(x(1))).is_some(),
            19 => self.target(self.slot_arg(x(1))).is_none(),
            20 => a.pending < 0,
            22 => self.i.cliff[(x(0) as i32 & 1) as usize],
            47 => self.i.offscreen,
            // R60 (sora_melee requirement switch, case 0x3C -> 0x81079E68): vfunc 0x20 of module accessor [off]. The
            // accessor constructor (0x81A6821C, called from 0x81A80E38 with in-place null modules) fills [off] from
            // soCatchModuleNull and [off] from soCaptureModuleNull, so [off] is the catch (grabber) module.
            // soCatchModuleImpl's vfunc 0x20 (0x81A64F80): true when its catch flag ([off]) is set and the link
            // module (accessor [off]) reports a link (vfunc 0x30 or 0x2C): a victim is held. soCaptureModuleImpl's
            // own vfunc 0x20 (0x81A66284) asks the same of the catch module. Tabuu's chain grab goes on to its attack
            // on "caught and R60"; the sora_enemy damage path (0x81A12380) asks it before accepting a hit
            60 => self.i.holding,
            // R75 (case 0x4B -> 0x81A7AA6C): vfunc 0x3C of accessor [off] through its vtable at [off]. The accessor
            // build at 0x81A80C24 puts soReflectModuleNull there (typeinfo name string; vfunc 0x3C 0x81A7BB24 = 0), so
            // [off] is the reflect module. soReflectModuleImpl's 0x3C (0x81A64044): its [off] is not -1. [off] is the
            // reflector's id, stored by the hit callback 0x81A640AC when a kind-3 (reflector) collision lands and the
            // power check passes, and reset to -1 by vfuncs 0x20 / 0x24: the actor was reflected
            75 => self.i.reflected,
            // R56 (sora_melee requirement switch, case 0x38 -> 0x8107A918): the link module (accessor [off]) vfunc
            // 0x168; emLinkModuleImpl's (0x81A28530): its partner id is not -1: linked to another actor
            56 => self.i.linked,
            // R21 a (generic switch case 0x15): an int argument only (any other type: false); the article manager
            // (accessor [off]) vfunc [off]: some live entry of its list answers a to its vfunc [off]
            21 => match self.arg(ins, k + 1) {
                A::K(v) => self.i.articles.iter().any(|t| t.on && t.id == v as i32),
                _ => false,
            },
            // R24 a [b] (case 0x18): an int a only; the manager's vfunc [off] gathers the live articles answering a
            // (b = 0: all, 1: the first, 2: the last, else none), true when one answers 1 to its vfunc [off] with 0
            24 => match self.arg(ins, k + 1) {
                A::K(v) => {
                    let mut m = self.i.articles.iter().filter(|t| t.on && t.id == v as i32);
                    match x(1) as i32 {
                        0 => m.any(|t| t.q),
                        1 => m.next().is_some_and(|t| t.q),
                        2 => m.last().is_some_and(|t| t.q),
                        _ => false,
                    }
                }
                _ => false,
            },
            // R28 a (case 0x1C): the manager's vfunc [off]: a below the class table's count asks that table's vfunc
            // [off] (an extra table the manager can hold is never set by the enemy constructor [M]), else false
            28 => match self.arg(ins, k + 1) {
                A::K(v) => (0..32).contains(&(v as i32)) && self.i.article_pool & (1 << v as i32) != 0,
                _ => false,
            },
            // R39 n (case 0x27): the motion module's vfunc [off] on motion slot n: false when the slot is flagged,
            // else its frame is at or past its last frame (the clip's frame count - 1). Slot 0 is the main motion;
            // the others and the slot flags are not modelled [L]
            39 => x(0) as i32 == 0 && {
                let len = self.p.motion_len.get(a.motion as usize).copied().unwrap_or(60).max(1) as i32;
                self.motion_frame().0 >= len - 1
            },
            63..=69 => self.i.ground_q & (1 << (id - 63)) != 0,
            73 => a.lock >= 0,
            10004 => a.kin.iter().any(|k| k.kind >= 0 && k.done != 0),
            10005 => a.kin.iter().any(|k| k.kind >= 0 && k.spd != 0.0),
            10008 | 10009 => {
                let s = self.slot_arg(x(0));
                let t = if id == 10008 { self.target(s) } else if s >= 0 { Some(a.found[s as usize & 3]) } else { None };
                match t {
                    Some(t) => {
                        let side = if t[0] >= a.pos[0] { 0 } else { 1 };
                        let front = if a.lr > 0.0 { 0 } else { 1 };
                        match x(1) as i32 {
                            0 => side == 0,
                            1 => side == 1,
                            2 => side == front,
                            _ => side != front,
                        }
                    }
                    None => false,
                }
            }
            // energy-slot queries (enemy requirement switch, table at sora_enemy s5 [off]: 10000 -> 0x81A1794C, 10001 ->
            // 0x81A179E0): the slot (-1 = every slot) is off or ...
            // - 10000: not running (0x81A1C9C4: the unit's running bit, set by the record start 0x81A1BE44 and cleared
            //   by the kinetic stop points 0x81A1DCF0-0x81A1E298). A path rider's end (0x81A1DFCC-0x81A1E0F8) only marks
            //   it finished (0x81A1C110) and keeps it running, looping or not, so a done path unit counts as running.
            //   Other done units are read as stopped [M: the VM keeps one finish flag]
            // - 10001: held (0x81A1CA10: the unit's pause byte, the VM's `paused`: 6604 / 6605)
            10001 => {
                let s = x(0) as i32;
                let off_or_held = |k: &Kin| k.kind < 0 || k.paused != 0;
                if s < 0 {
                    a.kin.iter().all(off_or_held)
                } else {
                    off_or_held(&a.kin[s as usize & (KIN - 1)])
                }
            }
            10000 => {
                let s = x(0) as i32;
                let off_or_done = |k: &Kin| k.kind < 0 || (k.done != 0 && !(self.i.path.is_some() && matches!(k.ek, 21 | 22)));
                if s < 0 {
                    a.kin.iter().all(off_or_done)
                } else {
                    off_or_done(&a.kin[s as usize & (KIN - 1)])
                }
            }
            // 10002 / 10003 slot (0x81A17A74 / 0x81A17B08): the unit is on, its upper (10002, 0x81A1CA50) / lower
            // (10003, 0x81A1CAE4) limit is set (not the 35791 / 35795 "none", the VM's f32::MAX / MIN) and its speed
            // has reached it: speed >= upper / speed <= lower. Slot -1: every unit that is on passes; a slot outside
            // 0..7 reads unit 0 (0x81A15FD4)
            // 10006 / 10007 node sel angle (0x81A13C14 -> 0x81A181D8; it needs three arguments): the model node (10006: the id
            // x(0); 10007: by name, vfunc 0x88) gives its global matrix (vfunc 0xA8); its rotation turns the axis sel
            // picks (35782 / 35785: z -1 / +1, 35783 / 35786: y, 35784 / 35787: x; any other: the zero vector) and
            // the angle in degrees between that and world up (8003DF9C: 0 for a zero vector) is at most x(2)
            10006 | 10007 => {
                let s = match x(1) as i32 {
                    35782 => [0.0, 0.0, -1.0],
                    35783 => [0.0, -1.0, 0.0],
                    35784 => [-1.0, 0.0, 0.0],
                    35785 => [0.0, 0.0, 1.0],
                    35786 => [0.0, 1.0, 0.0],
                    35787 => [1.0, 0.0, 0.0],
                    _ => [0.0f32; 3],
                };
                let r = self.node_rot(x(0) as i32);
                let d: [f32; 3] = core::array::from_fn(|i| r[i][0] * s[0] + r[i][1] * s[1] + r[i][2] * s[2]);
                let sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                let ang = if sq == 0.0 { 0.0 } else { (libm::acos((d[1] / libm::sqrtf(sq)).clamp(-1.0, 1.0) as f64) as f32) * 57.29578 };
                ang <= x(2)
            }
            // R23 d [flag] (generic switch case 0x17 -> the ground module's vfunc 0x1F8, emGroundModuleImpl ->
            // soGroundModuleImpl, sora_melee offset 0x2A578; true when its result is >= 0): from the posture position
            // (soPostureModuleImpl vfunc 0x18) a stage ray straight down by |d| (0x81C27CC4): d <= 0 or no hit gives
            // -1, a hit its distance. Answered from the host's floor under the actor [M: the nearest floor below; the
            // flag passed to the ray is not modelled]
            23 => {
                let d = x(0);
                let drop = a.pos[1] - self.i.floor_y;
                d > 0.0 && self.i.floor_on && drop >= 0.0 && drop <= libm::fabsf(d)
            }
            10002 | 10003 => {
                let hit = |k: &Kin| {
                    k.kind >= 0
                        && if id == 10002 { k.max != f32::MAX && !(k.spd < k.max) } else { k.min != f32::MIN && !(k.spd > k.min) }
                };
                let s = x(0) as i32;
                if s == -1 {
                    a.kin.iter().filter(|k| k.kind >= 0).all(hit)
                } else {
                    hit(&a.kin[if (0..KIN as i32).contains(&s) { s as usize } else { 0 }])
                }
            }
            // 10023 n (0x81A17B9C): unit n is on with speed 0 (0x81A1CB78); n = -1: every unit off or at speed 0 [M]
            10023 => {
                let s = x(0) as i32;
                let stopped = |k: &Kin| k.kind >= 0 && k.spd == 0.0;
                if s < 0 {
                    a.kin.iter().all(|k| k.kind < 0 || stopped(k))
                } else {
                    stopped(&a.kin[s as usize & (KIN - 1)])
                }
            }
            // 10021 / 10022 (0x81A17CAC): the slot's current kind equals the argument [M]
            10021 | 10022 => {
                let s = x(0) as i32;
                s >= 0 && a.kin[s as usize & (KIN - 1)].kind == x(1) as i32
            }
            // 10014 / 10015 (0x81A24BAC / 0x81A257E0 on the live / found position) [L: read as "the slot's target
            // exists"]; 10016 (0x81A2A404): bit n of the rotation module's flag word [L: no rotation state is kept,
            // false]; 10017-10020 (0x81A26360 / 26394 on a search unit): a player inside / not inside the unit [M, as
            // R18 / R19]
            10014 | 10015 | 10017 | 10019 => self.target(self.slot_arg(x(1))).is_some(),
            10018 | 10020 => self.target(self.slot_arg(x(1))).is_none(),
            10016 => false,
            // 10024-10047 node slot x y angle flag (0x81A145F0): the model module's node (10024-10026, 10030-10032,
            // 10036-10038, 10042-10044: the node id x(0); the others: the node named by x(0), vfunc 0x88 getNodeId)
            // gives its global matrix (vfunc 0xA8 getNodeGlobalMtx); its rotation turns a unit axis (x / y / z by id,
            // +1 for 10024-10035, -1 for 10036-10047; the table at the switch's float constants) into P (8003E3BC,
            // rotation only), and (P.x, P.y) is the direction. True when the angle (degrees) between it and the vector
            // from the actor's centre to the slot's point (0x81A25268 / 0x81A25A50: as 6902 / 6901, the target now for
            // 10024-10029 and 10036-10041 (0x81A2536C), the search snapshot for the others (0x81A25B54)) is within
            // |angle| (0x81A25138: a zero P gives 3 degrees; a zero product of the lengths 0, 8003DDD0). The centre
            // is the enemy's centre (`center`); the node rotation is the host's `nodes`, else a turn about y by the
            // facing [L]
            10024..=10047 => {
                let k = id - 10024;
                let axis = (k % 3) as usize;
                let sign = if k < 12 { 1.0 } else { -1.0 };
                let r = self.node_rot(x(0) as i32);
                let px = r[0][axis] * sign;
                let py = r[1][axis] * sign;
                let found = (k / 6) % 2 == 1;
                let t = self.slot_point(x(1) as i32, found, x(2), x(3), x(5) != 0.0);
                let c = self.center();
                let ang = cone_angle([px, py], [t[0] - c[0], t[1] - c[1]]);
                libm::fabsf(ang) <= libm::fabsf(x(4))
            }
            10010 => a.lock >= 0 && self.target(a.lock).is_some(),
            10012 | 10013 => {
                let s = self.slot_arg(x(0));
                let t = if id == 10012 { self.target(s) } else if s >= 0 { Some(a.found[s as usize & 3]) } else { None };
                match t {
                    Some(t) => self.band(t) == x(1) as i32,
                    None => false,
                }
            }
            10050 => self.target(self.slot_arg(x(0))).is_some(),
            10051 => self.arrived(),
            // 10055 n (0x81A15078; n a variable (argument type 5, read by sora_melee 0x8108D170) or a constant): the
            // actor's area module (module accessor [off], cast to emAreaModuleImpl) answers 0x81A27688: false when its
            // contact count (0x810A53D0, list vfunc 0x14) is 0 or area n's kind (0x810A4DC0 then 0x810A8024: byte 1
            // of the area's record) is not 13; else true when a contact (0x810A53E4, list vfunc 0x0C) has id n. The
            // host reports the kind-13 contacts by id
            10055 => self.i.area & (1u32 << (x(0) as i32 as u32 & 31)) != 0,
            // 10052 a b (0x81A14D38, two arguments, each a variable or a constant -> 0x81A0AEC8): false unless the
            // enemy manager has a free slot (0x81A0CEF0); then the stage-wide group manager's count of group a
            // (sora_melee 0x812331C4): false when 0; b = -1: every entry 0..count of group a is present (0x81233208);
            // else entry b is present
            10052 => {
                let (ga, gb) = (x(0) as i32, x(1) as i32);
                let g = self.i.groups.iter().find(|g| g.id == ga && g.count > 0);
                self.i.enemy_room
                    && match g {
                        None => false,
                        Some(g) if gb == -1 => (0..g.count).all(|n| n < 32 && g.present & (1 << n) != 0),
                        Some(g) => (0..32).contains(&gb) && g.present & (1 << gb) != 0,
                    }
            }
            // 10053 (0x81A14E70, no arguments -> 0x81A0B34C on the actor itself, the 0x803F0F44 call being a
            // __dynamic_cast of the actor): the enemy manager reads the link id of the actor's own entry (0x81A0D044,
            // none when unset or not live), then (0x81A0D0A8) true when that entry is absent or has no enemy, else
            // when that enemy's work flag 0 (0x12000000, work vfunc 0x4C) is set: the partner is gone
            10053 => !self.i.partner_alive,
            // 10054 a b f (0x81A14EB0, each a variable or a constant -> 0x81A0B5EC): X = the enemy manager's live
            // entries, Y = those whose enemy's id word is a (0 when the static object gates a); d = b - (X - Y) * f
            // in single precision (b and X - Y converted unsigned); true when d > 0 and d > Y
            10054 => {
                let (ga, gb, f) = (x(0) as i32, x(1) as i32, x(2));
                let xl = self.i.enemy_live;
                let y = match self.i.enemy_ids.iter().find(|e| e.id == ga) {
                    Some(e) if !e.gated => e.live,
                    _ => 0,
                };
                let d = gb as u32 as f32 - xl.wrapping_sub(y) as f32 * f;
                !(d <= 0.0) && !(d <= y as f32)
            }
            // 10057 (0x81A15204, no arguments -> 0x81A06EB4 on the actor itself): sets the actor's work flag 17, then
            // the enemy manager walks its other live entries (0x81A0CC44) and the callback (0x81A06DDC) clears the
            // flag for one linked to the actor (its link id is the actor's id, 0x81A0CB10) whose enemy exists and
            // whose work flag 0 (dead) is clear; the result is the flag. The write is applied below
            10057 => !self.i.child_alive,
            // 10059 (0x81A15284): no arguments (else false); the global object's flag byte
            10059 => self.i.global_flag,
            _ => false,
        };
        if id == 10057 {
            self.a.wb_set(WB_NO_CHILD, r);
        }
        r != not
    }

    /// Distance band (0x81A242EC): the first i with d <= threshold i (tuning words 14-16), else 3.
    fn band(&self, t: [f32; 2]) -> i32 {
        let dx = t[0] - self.a.pos[0];
        let dy = t[1] - self.a.pos[1];
        let d = libm::sqrtf(dx * dx + dy * dy);
        for i in 0..3 {
            if d <= self.p.tuning[14 + i] {
                return i as i32;
            }
        }
        3
    }
    /// The movement limits on an axis: the arena box, narrowed by the host's walls (x) or floor (y).
    fn limits(&self, ax: usize) -> (f32, f32) {
        let (mut lo, mut hi) = (self.p.bounds[2 * ax], self.p.bounds[2 * ax + 1]);
        if ax == 0 {
            if self.i.wall_on & 1 != 0 && self.i.wall[0] > lo {
                lo = self.i.wall[0];
            }
            if self.i.wall_on & 2 != 0 && self.i.wall[1] < hi {
                hi = self.i.wall[1];
            }
        } else if self.i.floor_on && self.i.floor_y > lo {
            lo = self.i.floor_y;
        }
        (lo, hi)
    }

    /// At the goal: within 0.1 on each set axis, or held at a limit with the goal beyond it [L: a stand-in. The
    /// boss files' goals assume their own arenas (Galleom's charge aims at +-225); in a smaller stage the box or a
    /// wall stops the move first, and the status waiting for arrival would never end. In the bosses' real arenas the
    /// goal is always inside and this never fires; when it does, kinetics sends EV_NOTE 1].
    fn arrived(&self) -> bool {
        let a = &*self.a;
        let at = |ax: usize| {
            let (p, g) = (a.pos[ax], a.goal[ax]);
            let (lo, hi) = self.limits(ax);
            libm::fabsf(p - g) <= 0.1 || (g > hi && p >= hi - 0.1) || (g < lo && p <= lo + 0.1)
        };
        (a.goal_set & 1 == 0 || at(0)) && (a.goal_set & 2 == 0 || at(1))
    }

    /// `if` / `elif` with the `and` / `or` commands after it, left to right: (value, pc after the chain).
    fn chain(&mut self, at: u32) -> (bool, u32) {
        let p: &'a Program = self.p;
        let c = &p.code;
        let mut v = self.cond(&c[at as usize], 0);
        let mut pc = at + 1;
        loop {
            let ins = &c[pc as usize];
            match ins.id {
                0x000B => v = self.cond(ins, 0) && v,
                0x000C => v = self.cond(ins, 0) || v,
                _ => return (v, pc),
            }
            pc += 1;
        }
    }

    // ---------------------------------------------------------------- threads

    fn run_now(&mut self, code: u32) {
        if code == NONE || code == 0 {
            return;
        }
        let mut t = thread_at(code, self.a.tick);
        self.run(&mut t, true);
    }

    fn run_thread(&mut self, k: usize) {
        if self.a.th[k].nf == 0 {
            return;
        }
        let (gs, gm) = (self.a.gen_status, self.a.gen_motion);
        let mut t = self.a.th[k];
        self.run(&mut t, false);
        // a status change replaced the loop thread / a motion change replaced the timelines: keep the new ones
        let replaced = match k {
            T_LOOP => self.a.gen_status != gs,
            1..=9 => self.a.gen_motion != gm,
            _ => false,
        };
        if !replaced {
            self.a.th[k] = t;
        }
    }

    fn run(&mut self, t: &mut Th, immediate: bool) {
        let mut budget = BUDGET;
        let now = self.a.tick as f32;
        while t.nf > 0 && t.resume <= now {
            budget -= 1;
            if budget == 0 {
                t.nf = 0;
                return;
            }
            let fi = t.nf as usize - 1;
            let pc = t.fr[fi].pc;
            let ins = self.p.code[pc as usize];
            t.fr[fi].pc = pc + 1;
            match ins.id {
                END | 0x0008 => t.nf -= 1,
                0x0001 => {
                    let n = self.av(&ins, 0);
                    if immediate && n > 0.0 {
                        t.nf = 0;
                        return;
                    }
                    t.resume = now + n;
                }
                0x0002 => {
                    let at = self.a.motion_t0 as f32 + self.av(&ins, 0);
                    if immediate && at > now {
                        t.nf = 0;
                        return;
                    }
                    t.resume = at;
                }
                0x0004 => {
                    let n = self.av(&ins, 0) as i32;
                    let f = &mut t.fr[fi];
                    if n == 0 || f.nl as usize >= LOOPS {
                        f.pc = ins.j + 1;
                    } else {
                        f.lb[f.nl as usize] = pc + 1;
                        f.ll[f.nl as usize] = n;
                        f.nl += 1;
                    }
                }
                0x0005 => {
                    let f = &mut t.fr[fi];
                    if f.nl > 0 {
                        let l = f.nl as usize - 1;
                        if f.ll[l] < 0 || f.ll[l] > 1 {
                            if f.ll[l] > 0 {
                                f.ll[l] -= 1;
                            }
                            f.pc = f.lb[l];
                        } else {
                            f.nl -= 1;
                        }
                    }
                }
                // 0006: leave the innermost loop
                0x0006 => {
                    let f = &mut t.fr[fi];
                    if f.nl > 0 {
                        let l = f.nl as usize - 1;
                        let lp = f.lb[l] - 1;
                        f.pc = self.p.code[lp as usize].j + 1;
                        f.nl -= 1;
                    }
                }
                0x0007 => {
                    if let A::Code(c) = self.arg(&ins, 0) {
                        if c != 0 && c == self.p.catch_exit {
                            self.ev(EV_CATCH_EXIT, pc, 0);
                        }
                        if c != 0 && (t.nf as usize) < FRAMES {
                            t.fr[t.nf as usize] = Fr { pc: c, ..FR0 };
                            t.nf += 1;
                        }
                    }
                }
                // 0009: go to (the frame continues in the target script)
                0x0009 => {
                    if let A::Code(c) = self.arg(&ins, 0) {
                        t.fr[fi] = Fr { pc: c, ..FR0 };
                    }
                }
                0x000A => {
                    let (v, after) = self.chain(pc);
                    t.fr[fi].pc = if v { after } else { self.skip_false(ins.j) };
                }
                0x000D | 0x000E => t.fr[fi].pc = ins.k + 1,
                0x0011 | 0x0012 => {
                    if ins.j != 0 {
                        t.fr[fi].pc = ins.j + 1;
                    }
                }
                0x000B | 0x000C | 0x000F | 0x0013 | 0x0100 | 0x0102 | 0x1B02 => {}
                0x0010 => {
                    let pick = self.switch_pick(&ins);
                    t.fr[fi].pc = self.case_pc(pc, ins.j, pick);
                }
                0x0018 => {
                    let f = &mut t.fr[fi];
                    f.pc = ins.j;
                    if ins.k == 1 && f.nl > 0 {
                        f.nl -= 1;
                    }
                }
                _ => self.exec(&ins, pc),
            }
        }
    }

    /// After a false `if` / `elif` with branch point `j`: the next `elif` that holds, the `else`, or past `endif`.
    fn skip_false(&mut self, mut j: u32) -> u32 {
        loop {
            let ins = &self.p.code[j as usize];
            match ins.id {
                0x000D => {
                    let (v, after) = self.chain(j);
                    if v {
                        return after;
                    }
                    j = ins.j;
                }
                0x000E | 0x000F => return j + 1,
                _ => return j,
            }
        }
    }

    /// A unit's velocity this frame, as the kinetic update gives it (0 for an off / paused / rotation unit).
    fn unit_vel(&self, s: usize) -> [f32; 2] {
        let k = &self.a.kin[s & (KIN - 1)];
        let lr = self.a.lr;
        if k.rec == 0 || k.paused != 0 || (2..=7).contains(&k.ek) || k.ek == 20 {
            return [0.0; 2];
        }
        if matches!(k.ek, 1 | 14..=19 | 23 | 24) && self.i.root_on {
            let rm = if k.ek == 24 { self.i.root_move2 } else { self.i.root_move };
            return [rm[0] * lr, rm[1]];
        }
        let r = k.ang * lr * (core::f32::consts::PI / 180.0);
        [lr * k.spd * libm::cosf(r), lr * k.spd * libm::sinf(r)]
    }

    fn switch_pick(&mut self, ins: &Ins) -> i32 {
        // switch kinds 1001 / 1002 n (0x81A1FC5C -> 0x81A17FBC / 0x81A1805C): the sign of unit n's velocity along x
        // / y: 2 positive, 3 negative, -1 still
        let ty = self.av(ins, 0) as i32;
        if ty == 1001 || ty == 1002 {
            let v = self.unit_vel(self.av(ins, 1) as usize);
            let c = if ty == 1001 { v[0] } else { v[1] };
            return if v[0] * v[0] + v[1] * v[1] <= 0.0 || c == 0.0 {
                -1
            } else if c > 0.0 {
                2
            } else {
                3
            };
        }
        let A::Rec(r) = self.arg(ins, 1) else { return -1 };
        let id = self.p.recs[r as usize].0;
        let n = self.av(ins, 2) as usize;
        let p = self.p;
        let Some(list) = p.lists.iter().find(|l| l.id == id) else { return -1 };
        let l = self.a.level();
        let mut w = [0u32; 16];
        let mut total = 0u32;
        for (k, e) in list.e.iter().take(n.min(16)).enumerate() {
            let mut x = e.0 + e.1 * l;
            if e.2 != NONE {
                self.a.ti[6] = k as i32;
                self.a.tf[0] = x;
                self.a.tb_set(TB_TRUE, false);
                self.run_now(e.2);
                x = if self.a.tb_get(TB_TRUE) { self.a.tf[0] } else { 0.0 };
            }
            w[k] = if x > 0.0 { x as u32 } else { 0 };
            total += w[k];
        }
        if total == 0 {
            return -1;
        }
        let mut r = self.rnd(total);
        for k in 0..n.min(16) {
            if w[k] > r {
                return k as i32;
            }
            r -= w[k];
        }
        -1
    }

    fn case_pc(&self, sw: u32, end: u32, pick: i32) -> u32 {
        let c = &self.p.code;
        let mut d = 0;
        let mut default = NONE;
        for pc in sw + 1..end {
            let ins = &c[pc as usize];
            match ins.id {
                0x0010 => d += 1,
                0x0013 => d -= 1,
                0x0011 if d == 0 && self.av(ins, 0) as i32 == pick => return pc + 1,
                0x0012 if d == 0 && default == NONE => default = pc,
                _ => {}
            }
        }
        if default != NONE {
            default + 1
        } else {
            end + 1
        }
    }

    // ---------------------------------------------------------------- commands

    fn exec(&mut self, ins: &Ins, pc: u32) {
        let v = |s: &Self, k: u32| s.av(ins, k);
        match ins.id {
            0x0202 => {
                if self.cond(ins, 2) {
                    self.a.pending = v(self, 1) as i32;
                }
            }
            0x0409 => self.change_motion(v(self, 0) as i32),
            // 0408 part motion blend: a second animated part (Porky's bust) plays a motion of the same list; the host
            // draws it from the forwarded command [L: one part]
            0x0408 => {
                self.a.part_motion = v(self, 1) as i32;
                self.a.part_t0 = self.a.tick;
                self.ev(EV_CMD, pc, 0);
            }
            // 030A node x y z: a model node's world position into three variables (kind 0x03, the model-node module:
            // isObserv sora_melee 0x8100E5D0, handler next to it, scene-graph node calls) [M from its uses, which
            // subtract the actor's x right after]. The VM has no skeleton: the actor's position is written and the
            // host may refine it from the event
            0x030A => {
                let p = self.a.pos;
                for k in 0..3 {
                    self.set(self.arg(ins, 1 + k), p[k as usize]);
                }
                self.ev(EV_CMD, pc, 0);
            }
            0x0500 => self.a.lr = -self.a.lr,
            // 0504 (posture module, sora_melee 0x81023C14 by the animcmd table): the posture's turn field = a constant x
            // the facing, i.e. the model is turned to the current facing; the facing itself is unchanged (0500 is the
            // flip: setLr(-lr), 0x81025068) [M]
            0x0504 => {}
            // 0502: face the goal x [L: from its use right after a goal side is picked]
            0x0502 => {
                if self.a.goal_set & 1 != 0 && self.a.goal[0] != self.a.pos[0] {
                    self.a.lr = if self.a.goal[0] > self.a.pos[0] { 1.0 } else { -1.0 };
                }
            }
            0x0501 => self.a.lr = if v(self, 0) < 0.0 { -1.0 } else { 1.0 },
            // 0505 var: the facing into a variable [L: from its use right before an lr compare]
            0x0505 => {
                let x = self.arg(ins, 0);
                let lr = self.a.lr;
                self.set(x, lr);
            }
            0x0509 | 0x050A => {
                for k in 0..3 {
                    self.a.pos[k] = v(self, k as u32);
                }
            }
            0x050B => {
                for k in 0..3 {
                    self.a.pos[k] += v(self, k as u32);
                }
            }
            0x0605 => {
                self.a.hurt = v(self, 0) as u32;
                self.ev(EV_HURT, pc, self.a.hurt as i32);
            }
            0x0607 => {
                let (i, s) = (v(self, 0) as u32 & 31, v(self, 1) as i32);
                if s == 2 {
                    self.a.hurt_mask |= 1 << i;
                } else {
                    self.a.hurt_mask &= !(1 << i);
                }
                self.ev(EV_HURT, pc, s);
            }
            // 6D00 s (emCollisionHitModuleImpl, command kind 0x6D, 0x81A2BC5C -> 0x81A2BA44): every hurtbox of the
            // file's list (param 43006) gets status s through the hit module's vfunc 0x44 (i, s, 0), the call 0607 makes
            0x6D00 => {
                let s = v(self, 0) as i32;
                for &i in &self.p.hurt_list {
                    let i = i & 31;
                    if s == 2 {
                        self.a.hurt_mask |= 1 << i;
                    } else {
                        self.a.hurt_mask &= !(1 << i);
                    }
                }
                self.ev(EV_HURT, pc, s);
            }
            // 100D kind var (soGenerateArticleManageModuleImpl, command kind 0x10, case 13 at 0x81097190): var = the
            // module's vfunc 0x64 (0x810977A0): the article manager's maximum count for the kind (vfunc 0x60) less
            // its live count (vfunc 0x5C), the room for more; the host keeps the articles and reports the room
            0x100D => {
                let k = v(self, 0) as i32;
                let room = if (0..4).contains(&k) { self.i.article_room[k as usize] } else { 0 };
                self.set(self.arg(ins, 1), room as f32);
            }
            0x0808 => {
                if self.i.ground {
                    self.a.pos[1] = self.i.floor_y;
                }
                self.ev(EV_CMD, pc, 0);
            }
            0x0900 => self.a.situation = v(self, 0) as i32,
            0x0C01 | 0x0C04 => {}
            0x0C02 => {
                self.a.active = 0;
                self.a.hurt = 2;
                self.ev(EV_HURT, pc, 2);
            }
            0x0C03 => {
                self.a.active = 1;
                self.a.hurt = 0;
                self.ev(EV_HURT, pc, 0);
            }
            0x0D00 => {
                if let A::Code(c) = self.arg(ins, 1) {
                    self.a.dproc = c;
                    self.a.th[T_DPROC] = thread_at(c, self.a.tick);
                }
            }
            0x1200 | 0x1206 => {
                let x = v(self, 0);
                self.set(self.arg(ins, 1), x);
            }
            // 1201 a var: var += a (1207 the float form)
            0x1201 | 0x1207 => {
                let t = self.arg(ins, 1);
                let x = self.val(t) + v(self, 0);
                self.set(t, x);
            }
            0x1203 => {
                let t = self.arg(ins, 0);
                let x = self.val(t) + 1.0;
                self.set(t, x);
            }
            // 1205 lo hi var: a random int in [lo, hi] [M from its uses: 0 1]
            0x1205 => {
                let (lo, hi) = (v(self, 0) as i32, v(self, 1) as i32);
                let n = (hi - lo + 1).max(1) as u32;
                let x = lo + self.rnd(n) as i32;
                self.set(self.arg(ins, 2), x as f32);
            }
            0x1208 => {
                let t = self.arg(ins, 1);
                let x = self.val(t) - v(self, 0);
                self.set(t, x);
            }
            0x120A => self.set(self.arg(ins, 0), 1.0),
            0x120B => self.set(self.arg(ins, 0), 0.0),
            0x120F => {
                let t = self.arg(ins, 1);
                let x = self.val(t) * v(self, 0);
                self.set(t, x);
            }
            // 1210 a var: var /= a [M]; 1212 a var: var := |a| [L: always the same var twice]
            0x1210 => {
                let t = self.arg(ins, 1);
                let d = v(self, 0);
                let x = if d != 0.0 { self.val(t) / d } else { 0.0 };
                self.set(t, x);
            }
            0x1212 => {
                let x = libm::fabsf(v(self, 0));
                self.set(self.arg(ins, 1), x);
            }
            0x1400 => {
                self.a.hurt_groups_off &= !(1 << (v(self, 0) as u32 & 31));
                self.ev(EV_HURT, pc, 0);
            }
            0x1401 => {
                self.a.hurt_groups_off |= 1 << (v(self, 0) as u32 & 31);
                self.ev(EV_HURT, pc, 0);
            }
            // 1405 b: the body's hit collision on / off after a death [L]; the host reads the argument
            0x1405 => self.ev(EV_HURT, pc, v(self, 0) as i32),
            // 6524: capture released (CommonCaptureProcExit) [L]: nothing VM-side
            0x6524 => {}
            // 6B01: the damage turn without rotation (CommonAIExecDamageTurnNoRot): face the other way [M]
            0x6B01 => self.a.lr = -self.a.lr,
            // 1500: apply a pending facing change [L]; nothing is pending in this model
            0x1500 => {}
            0x1600 => {
                self.a.search_reset = self.a.search_reset.wrapping_add(1);
                for s in 0..4 {
                    if let Some(t) = self.target(s) {
                        self.a.found[s as usize] = t;
                    }
                }
            }
            0x1900 => self.a.shadow = 1,
            0x1901 => self.a.shadow = 0,
            0x1902 => {}
            0x6500 => {
                self.a.live = 0;
                self.ev(EV_REMOVE, pc, 0);
            }
            // 6504 n: timer n restarted (0x81A0A3C4); 6510 n: paused (0x81A0A418); 6511 n: started if never started, else
            // resumed with the paused frames added (0x81A0A440)
            0x6504 | 0x6510 | 0x6511 => {
                let n = v(self, 0) as i32;
                if (0..4).contains(&n) {
                    let now = self.a.tick;
                    let t = &mut self.a.timers[n as usize];
                    match ins.id {
                        0x6504 => *t = [now, 0, 0],
                        0x6510 => t[1] = now,
                        _ => {
                            if t[0] == NONE {
                                t[0] = now;
                            } else if t[1] != 0 {
                                t[2] = t[2].wrapping_add(now.wrapping_sub(t[1]));
                                t[1] = 0;
                            }
                        }
                    }
                }
            }
            // 6506 rec var: a random int in [a + b L, c + d L] (0x81A067C0: the int form of 6507; a slope of exactly
            // the sentinel is skipped)
            0x6506 => {
                if let A::Rec(r) = self.arg(ins, 0) {
                    let l = self.a.level();
                    let p = self.p;
                    let lo = (p.rec_w(r, 0) as i32 as f32 + p.rec_f(r, 1) * l) as i32;
                    let hi = ((p.rec_w(r, 2) as i32 as f32 + p.rec_f(r, 3) * l) as i32).max(lo);
                    let x = lo + self.rnd((hi - lo + 1) as u32) as i32;
                    self.set(self.arg(ins, 1), x as f32);
                }
            }
            // 6515: the goal y (0x81A08EBC)
            0x6515 => {
                self.a.goal[1] = v(self, 0);
                self.a.goal_set |= 2;
            }
            0x6505 => self.ev(EV_STAGE, pc, v(self, 0) as i32),
            0x6507 => {
                if let A::Rec(r) = self.arg(ins, 0) {
                    let l = self.a.level();
                    let p = self.p;
                    let lo = p.rec_f(r, 0) + p.rec_f(r, 1) * l;
                    let hi = (p.rec_f(r, 2) + p.rec_f(r, 3) * l).max(lo);
                    let u = self.rnd(1 << 16) as f32 / 65536.0;
                    self.set(self.arg(ins, 1), lo + (hi - lo) * u);
                }
            }
            0x6509 => {
                if let A::Rec(r) = self.arg(ins, 0) {
                    let x = self.p.rec_f(r, 0) + self.p.rec_f(r, 1) * self.a.level();
                    self.set(self.arg(ins, 1), x);
                }
            }
            // 651B a b var flag (0x81A0872C): a percentage of a, with the module's constants 0 and 100: t = 100 - b;
            // flag set: var = 0 when t <= 0, else a * t / 100; flag clear: var = a when t <= 0, else a * b / 100.
            // Stored through the work module's float setter
            0x651B => {
                let (a, b) = (v(self, 0), v(self, 1));
                let t = 100.0 - b;
                let x = if v(self, 3) != 0.0 {
                    if t <= 0.0 { 0.0 } else { a * t / 100.0 }
                } else if t <= 0.0 {
                    a
                } else {
                    a * b / 100.0
                };
                self.set(self.arg(ins, 2), x);
            }
            // 6521 node vx vy vz (0x81A089DC -> 0x81A06C44): a model node's position (the motion module's node
            // transform, vfunc 0x98, through the enemy's matrix); a node past the part count gives (0, 0, 0). Stored
            // through the work module's float setter [L: no skeleton: the node is the actor's position, as 10024]
            0x6521 => {
                if v(self, 0) as i32 != -1 {
                    for k in 0..3u32 {
                        let x = self.a.pos[k as usize];
                        self.set(self.arg(ins, 1 + k), x);
                    }
                }
            }
            // 6508 rec var (0x81A08444 -> 0x81A06660): the integer form of 6509: n = trunc(word 0); when word 1 is
            // not 0, n = trunc(n + word 1 * level); stored through the work module's int setter
            0x6508 => {
                if let A::Rec(r) = self.arg(ins, 0) {
                    let mut n = self.p.rec_f(r, 0) as i32;
                    let k = self.p.rec_f(r, 1);
                    if k != 0.0 {
                        n = (n as f32 + k * self.a.level()) as i32;
                    }
                    self.set(self.arg(ins, 1), n as f32);
                }
            }
            0x651F => {
                if let A::Rec(r) = self.arg(ins, 0) {
                    let x = self.p.rec_f(r, 0);
                    self.set(self.arg(ins, 1), x);
                }
            }
            // 6520 rec var: the record's second float: 651F and 6520 share handler 0x81A078EC, which stores word +0
            // for id 0x1F and word +4 for id 0x20
            0x6520 => {
                if let A::Rec(r) = self.arg(ins, 0) {
                    let x = self.p.rec_f(r, 1);
                    self.set(self.arg(ins, 1), x);
                }
            }
            0x6514 => {
                self.a.goal[0] = v(self, 0);
                self.a.goal_set |= 1;
            }
            0x6516 => self.a.goal_set = 0,
            0x651E => {
                if let A::Rec(r) = self.arg(ins, 0) {
                    self.a.pos[0] = self.p.rec_f(r, 0);
                    self.a.pos[1] = self.p.rec_f(r, 1);
                    self.a.lr = if self.p.rec_f(r, 2) < 0.0 { -1.0 } else { 1.0 };
                }
            }
            0x6523 => {
                self.a.strength = v(self, 0) as i32;
                self.a.wf[WF_L] = level_f(self.a.strength, self.a.difficulty);
            }
            0x6600 | 0x6601 | 0x6609 | 0x660A | 0x660B => {
                let (slot, kind) = (v(self, 0) as i32, v(self, 1) as i32);
                let ang = if ins.id == 0x6600 && ins.n >= 3 { Some(v(self, 2)) } else { None };
                self.kin_start(slot, kind, ang);
            }
            0x6607 => {
                let s = v(self, 0) as i32;
                for k in 0..KIN {
                    if s < 0 || s == k as i32 {
                        self.a.kin[k] = KIN0;
                    }
                }
            }
            // 6604 / 6605 n: unit n (-1 = all) resumed / paused (0x81A1BF64, 0x81A1C038);
            // 660E n: unit n reversed (0x81A1F060: speed negated; a directed unit's direction turned round)
            // 660F n (0x81A1A388 -> 0x81A1CF08): the unit's direction is re-derived from the posture's facing; read
            // as mirroring a directed unit for the current facing [L]
            // 660F: the unit's angle is kept relative to the facing (the directed update turns by angle x facing),
            // so re-deriving it from the facing changes nothing here
            0x660F => {}
            0x6604 | 0x6605 | 0x660E => {
                let s = v(self, 0) as i32;
                for k in 0..KIN {
                    if s < 0 || s == k as i32 {
                        let u = &mut self.a.kin[k];
                        match ins.id {
                            0x6604 => u.paused = 0,
                            0x6605 => u.paused = 1,
                            _ => {
                                if u.dir_set != 0 {
                                    u.ang += 180.0;
                                } else {
                                    u.spd = -u.spd;
                                }
                            }
                        }
                    }
                }
            }
            // 661C a nx ny out (kind-66 case 0x1C, 0x81A1A6B4 -> 0x81A15A00): the angle in degrees of the vector
            // (nx, ny) from the axis (a, 0): acos of the normalized dot product, 360 minus it when ny < 0; written to
            // `out` (it feeds 660C, the unit's direction)
            0x661C => {
                let d = vec_angle(v(self, 0), [v(self, 1), v(self, 2)]);
                self.set(self.arg(ins, 3), d);
            }
            // 661D slot kind [n] [flag] (0x81A19684 -> 0x81A16CFC; n default -1, flag default 0): the move vectors of
            // every live unit (see unit_move for the gate 0x81A1C1D4)
            // are added up; units of category 0 (0x81A1AE30) are ended (0x81A1C2BC with 1, then
            // 0x81A1CC18). If the sum is non-zero, the slot is off or finished (0x81A1794C) and the kind's record
            // (0x81A15964) is a directed one (kinds 0 / 8-13) whose start speed (word 1) holds one of the markers
            // 35788 / 35789 / 35791: the direction is the sum reversed (flag 0, 0x81A108CC by -1) or the sum with its
            // larger axis reversed (flag 1; x on a tie), the other units are ended again with that direction
            // (0x81A1C414), the slot is cleared (0x81A17C30), its speed becomes the direction's length (0x81A1C610)
            // and the record starts (0x81A1627C) at the record's angle, or, for word 9 = 35789, at the direction's
            // angle for the facing (0x81A15A00). A bounce off what stopped the units. n goes to the record start
            // [L: its use there is not read]
            0x661D => {
                let (slot, kind) = (v(self, 0) as i32, v(self, 1) as i32);
                let flag = ins.n > 3 && v(self, 3) != 0.0;
                let mut sum = [0.0f32; 2];
                for k in 0..KIN {
                    let m = self.unit_move(k);
                    sum[0] += m[0];
                    sum[1] += m[1];
                    let u = &mut self.a.kin[k];
                    // category 0 (0x81A1AE30: every kind but 2-7 and 20): 0x81A1C2BC returns 1, the unit ends;
                    // category 1 returns 0; category 2 (kind 20) is not followed [L]
                    if u.kind >= 0 && u.paused == 0 && !(2..=7).contains(&u.ek) && u.ek != 20 {
                        *u = KIN0;
                    }
                }
                let si = slot as usize & (KIN - 1);
                let free = |k: &Kin| k.kind < 0 || k.done != 0;
                let ok = if slot < 0 { self.a.kin.iter().all(free) } else { free(&self.a.kin[si]) };
                let len = libm::sqrtf(sum[0] * sum[0] + sum[1] * sum[1]);
                let rec = self.kin_rec(kind);
                if len != 0.0 && ok && rec != 0 && slot >= 0 {
                    let p = self.p;
                    let w = |i: u32| f32::from_bits(be32(&p.data, rec + 4 * i));
                    let ek = be32(&p.data, rec) as i32;
                    let w1 = w(1);
                    if (ek == 0 || (8..=13).contains(&ek)) && (w1 == 35791.0 || w1 == 35789.0 || w1 == 35788.0) {
                        let mut d = sum;
                        if !flag {
                            d = [-d[0], -d[1]];
                        } else if libm::fabsf(d[0]) >= libm::fabsf(d[1]) {
                            d[0] = -d[0];
                        } else {
                            d[1] = -d[1];
                        }
                        let ang = if w(9) == 35789.0 { Some(vec_angle(self.a.lr, d)) } else { None };
                        self.a.kin[si] = KIN0;
                        self.kin_start(slot, kind, ang);
                        self.a.kin[si].spd = libm::sqrtf(d[0] * d[0] + d[1] * d[1]);
                    }
                }
            }
            // 661E joint (0x81A1AA1C -> 0x81A1F3B8): when the units' summed move vector (the kinetic module's vfunc
            // 0x40 with -1) is longer than 1e-5, the joint (a number, or a node name looked up in the model) is turned
            // about z to the vector's angle for the facing (0x81A15A00), negated when the posture's yaw is -90
            // (0x81A1E768 kind 2, posture vfunc 0x44). The VM has no per-joint rotation: the host gets EV_AIM with
            // `ins` = this instruction (the joint argument read from the program) and `v` = the angle's f32 bits
            0x661E => {
                let mut sum = [0.0f32; 2];
                for k in 0..KIN {
                    let m = self.unit_move(k);
                    sum[0] += m[0];
                    sum[1] += m[1];
                }
                if libm::sqrtf(sum[0] * sum[0] + sum[1] * sum[1]) > 1e-5 {
                    let lr = self.a.lr;
                    let a = vec_angle(lr, sum) * if lr < 0.0 { -1.0 } else { 1.0 };
                    self.ev(EV_AIM, pc, a.to_bits() as i32);
                }
            }
            // 6616 / 6617 n v (0x81A19D54 -> 0x81A1F2C8): unit n's facing reference = v (6617: + its current one)
            0x6616 | 0x6617 => {
                let n = v(self, 0) as i32;
                if n >= 0 {
                    let x = v(self, 1);
                    let u = &mut self.a.kin[n as usize & (KIN - 1)];
                    u.face = if ins.id == 0x6617 { u.face + x } else { x };
                }
            }
            // 661A / 661B n v (0x81A19BD0 -> 0x81A1F348, 0x81A1B044): unit n's upper speed limit = v (661B: + the
            // current limit) + the record's level coefficient x L; a "keep" v takes the current speed [M]
            0x661A | 0x661B => {
                let n = v(self, 0) as i32;
                if n >= 0 {
                    let l = self.a.level();
                    let x = v(self, 1);
                    let u = &mut self.a.kin[n as usize & (KIN - 1)];
                    let base = if ins.id == 0x661B { u.max + x } else { x };
                    u.max = if (KEEP_LO..=KEEP_HI).contains(&x) { u.spd } else if sentinel(x) { f32::MAX } else { base + u.max_l * l };
                }
            }
            // 6602 / 6603 slot kind path flag [frame] [n] (0x81A18500): argument 2 is value 41000, the placement's
            // path object (Enemy [off], from the create parameter [off]: a grGimmickMotionPath), which the record
            // start (0x81A1B0C0) keeps at unit [off]. Kinds 21 / 22 ride it (Input::path). Without a host path the
            // unit heads straight for Actor::home and stops there [L: a stand-in for hosts with no stage paths]
            0x6602 | 0x6603 => {
                let (slot, kind) = (v(self, 0) as i32, v(self, 1) as i32);
                let t = self.a.home;
                let (dx, dy) = (t[0] - self.a.pos[0], t[1] - self.a.pos[1]);
                let lr = self.a.lr;
                let ang = lr * libm::atan2f(dy, lr * dx) * (180.0 / core::f32::consts::PI);
                let si = slot as usize & (KIN - 1);
                let prev = self.a.kin[si].pf;
                self.kin_start(slot, kind, Some(ang));
                let snap = ins.n > 3 && v(self, 3) != 0.0;
                let f = if ins.n > 5 { libm::truncf(v(self, 5)) } else { -1.0 };
                // argument 6 (integer, default 0): the path word 6613 reads
                let pn = if ins.n > 6 { libm::truncf(v(self, 6)) } else { 0.0 };
                let (g0, g1) = (self.p.general[22], self.p.general[23]);
                let path = self.i.path;
                let u = &mut self.a.kin[si];
                u.seek = 1;
                if let (Some(pa), true) = (path, matches!(u.ek, 21 | 22)) {
                    // the path movers (record start 0x81A1BC40): the frame is argument 5 when above -1 (default
                    // -1, 0x81A18944), else 0 when argument 3 (the snap flag) is set, else kept; with the flag the
                    // actor is put on the path point + the offset (values 3054 / 3055) at once
                    u.seek = 0;
                    u.pn = pn;
                    u.pf = if f > -1.0 {
                        f
                    } else if snap {
                        0.0
                    } else {
                        prev
                    };
                    let pf = u.pf;
                    if snap {
                        let q = (pa.pos)(pa.id, pf);
                        self.a.pos[0] = q[0] + g0;
                        self.a.pos[1] = q[1] + g1;
                    }
                }
            }
            // 6614 slot value (kinetic switch case 0x14, shared with 0x0C at 0x81A19ED4): exactly two arguments; the
            // unit's angle becomes value + its current angle as 0x81A1EC18 reads it (slot -1: + 0; a stored 35790
            // reads as 270; the markers 35782-35787 read the posture [L: not traced, read as stored]), written by
            // the angle setter 0x81A1EF60 (slot -1: every unit that is on; one slot: written even when off)
            0x6614 => {
                if ins.n == 2 {
                    let n = v(self, 0) as i32;
                    let unit = |n: i32| if (0..KIN as i32).contains(&n) { n as usize } else { 0 };
                    let f = if n == -1 {
                        0.0
                    } else {
                        let x = self.a.kin[unit(n)].ang;
                        if x as i32 == 35790 { 270.0 } else { x }
                    };
                    let w = v(self, 1) + f;
                    if n == -1 {
                        for s in 0..KIN {
                            if self.a.kin[s].ek != -1 {
                                kin_set_ang(&mut self.a.kin[s], w);
                            }
                        }
                    } else {
                        kin_set_ang(&mut self.a.kin[unit(n)], w);
                    }
                }
            }
            // 6612 slot var (kinetic switch case 0x12, 0x81A1A554 -> 0x81A17CF0 -> 0x81A1F130): exactly two arguments,
            // the second a variable; var = the unit's placement-path frame when its engine kind is 21 / 22, else 0
            // (an off unit too; slot -1 gives 0; a slot outside 0..7 reads unit 0, 0x81A15FD4)
            0x6612 => {
                if ins.n == 2 {
                    let s = v(self, 0) as i32;
                    let f = if s == -1 {
                        0.0
                    } else {
                        let k = &self.a.kin[if (0..KIN as i32).contains(&s) { s as usize } else { 0 }];
                        if k.kind >= 0 && (k.ek == 21 || k.ek == 22) { k.pf } else { 0.0 }
                    };
                    self.set(self.arg(ins, 1), f);
                }
            }
            // 6613 slot var (same case as 6612): the unit's second path word (`Kin::pn`) for engine kinds 21 / 22, else 0
            0x6613 => {
                if ins.n == 2 {
                    let s = v(self, 0) as i32;
                    let f = if s == -1 {
                        0.0
                    } else {
                        let k = &self.a.kin[if (0..KIN as i32).contains(&s) { s as usize } else { 0 }];
                        if k.kind >= 0 && (k.ek == 21 || k.ek == 22) { k.pn } else { 0.0 }
                    };
                    self.set(self.arg(ins, 1), f);
                }
            }
            // 0200: no module observes command kind 0x02 [M: the two enemy files' 0200s are records read as scripts]
            0x0200 => {}
            0x660C => {
                let s = v(self, 0) as usize & (KIN - 1);
                self.a.kin[s].ang = v(self, 1);
                self.a.kin[s].dir_set = 1;
            }
            0x6900 => {
                let s = if self.a.eval_slot >= 0 { self.a.eval_slot } else { self.a.lock };
                if s >= 0 {
                    self.lock(s);
                }
            }
            // 6901 / 6902 slot x y out flag (search-module switch 0x81A26510, sub-op 1 / 2; 0x81A25C60 / 0x81A25DF0):
            // the angle (degrees, [0, 360), 0x81A25BD0 -> 0x81A15A00) from the actor to a point = the slot's target
            // plus the offset (x * ax, y) * scale (0x81A24278; its rotation is by 0). ax is the target's facing when
            // the flag is set, else 1; the angle is measured from the actor's facing (the posture lr passed in f1).
            // 6901 reads the search snapshot (the target as last found), 6902 the target now. Slot 0..3: a player
            // (absent: point 0, ax 0, scale 1); 4..7: other target kinds (4 / 5 linked objects, 6 a module point,
            // 7 none) give point 0 and factors 1 here [L: 4..6 not modelled]; any other slot: point 0 and, for
            // 6901, factors -1 (6902: ax 0, scale 1). Only a variable out is written; the actor point is the actor's
            // position (the node the code reads; no skeleton)
            0x6901 | 0x6902 => {
                let p = self.slot_point(v(self, 0) as i32, ins.id == 0x6901, v(self, 1), v(self, 2), v(self, 4) != 0.0);
                let ang = vec_angle(self.a.lr, [p[0] - self.a.pos[0], p[1] - self.a.pos[1]]);
                self.set(self.arg(ins, 3), ang);
            }
            // 6905 / 6906 slot var: the distance from the actor to the target's live / found position (0x81A244DC)
            0x6905 | 0x6906 => {
                let s = self.slot_arg(v(self, 0));
                let t = if ins.id == 0x6906 { (s >= 0).then(|| self.a.found[s as usize & 3]) } else { self.target(s) };
                if let Some(t) = t {
                    let (dx, dy) = (t[0] - self.a.pos[0], t[1] - self.a.pos[1]);
                    self.set(self.arg(ins, 1), libm::sqrtf(dx * dx + dy * dy));
                }
            }
            0x6907 | 0x6908 | 0x6909 | 0x690A => {
                let s = self.slot_arg(v(self, 0));
                let found = matches!(ins.id, 0x6908 | 0x690A);
                let c = if matches!(ins.id, 0x6907 | 0x6908) { 0 } else { 1 };
                let t = if found { (s >= 0).then(|| self.a.found[s as usize & 3]) } else { self.target(s) };
                if let Some(t) = t {
                    self.set(self.arg(ins, 1), t[c]);
                }
            }
            0x6A00 => self.a.set_rule(v(self, 0) as usize, true),
            0x6A01 => {
                let n = v(self, 0) as i32;
                if n < 0 {
                    self.a.rules_on = [0; 2];
                } else {
                    self.a.set_rule(n as usize, false);
                }
            }
            0x6A02 => {
                for (i, r) in self.p.rules.iter().enumerate() {
                    self.a.set_rule(i, r.on);
                }
            }
            0x6A03 => self.a.lock = -1,
            0x6B03 => {
                if v(self, 1) <= v(self, 0) {
                    self.a.wf[WF_HP] = 0.0;
                    self.a.wb_set(WB_DEAD, true);
                    self.a.tb_set(TB_TRUE, true);
                }
            }
            0x650E => {
                let r = v(self, 0) as i32;
                self.ev(EV_WEAPON, pc, r);
            }
            0x6519 => {
                if let Some(pa) = self.i.partner_vars {
                    for k in 0..ins.n as u32 {
                        let x = self.arg(ins, k);
                        let v = match x {
                            A::Wi(i) => pa.wi[i as usize] as f32,
                            A::Wf(i) => pa.wf[i as usize],
                            A::Wb(i) => (pa.wb >> (i as u32 & 63) & 1) as f32,
                            _ => continue,
                        };
                        self.set(x, v);
                    }
                }
                self.ev(EV_CMD, pc, 0);
            }
            id if forwarded(id) => self.ev(EV_CMD, pc, 0),
            _ => {}
        }
    }

    // ---------------------------------------------------------------- kinetics

    /// Start a movement record on a slot. Kinds >= 14 are the file's records (param 44000, 16 words each); below
    /// that, engine built-ins (0 gravity; the damage-flight kinds are the host's) [M, engine.md 6].
    fn kin_start(&mut self, slot: i32, kind: i32, ang: Option<f32>) {
        let s = slot as usize & (KIN - 1);
        let mut k = KIN0;
        k.kind = kind;
        if let Some(a) = ang {
            if a == 35791.0 || a == 35789.0 {
                // the angle setter (0x81A1EF60, from the record start 0x81A1B0C0 via [off]) skips these two markers:
                // the slot's unit keeps its current angle
                k.ang = self.a.kin[s].ang;
                k.dir_set = self.a.kin[s].dir_set;
            } else {
                k.ang = a;
                k.dir_set = 1;
            }
        }
        let l = self.a.level();
        let p = self.p;
        // the record (0x81A15964): kinds >= 14 from param 44000, below that the built-ins of param 43001
        let at = self.kin_rec(kind);
        if at != 0 && (at as usize + 64) <= p.data.len() {
            let wd = |i: u32| be32(&p.data, at + 4 * i);
            let w = |i: u32| f32::from_bits(wd(i));
            let keep = |x: f32| (KEEP_LO..=KEEP_HI).contains(&x);
            // every energy type reads the same words: speed w1 + w2 L (keep = the slot's current speed),
            // acceleration w3 + w4 L (negative = decay toward zero), limit w7 + w8 L [M: from the records' use]
            // the engine kind's category (0x81A1AE30, table s5+2A38: kinds 2-7 one, 20 another, the rest a third)
            let cat = |k: i32| match k {
                2..=7 => 1,
                20 => 2,
                _ => 0,
            };
            // the record start (0x81A1B0C0) copies the unit's old words before it loads the record, so "previous"
            // is the old unit's engine kind (unit [off]), and the new one is the record's word 0, not the kind
            // numbers. For a start speed whose integer part is 35791 / 35789 / 35788 (0x81A1B330), 0x81A1B370 ->
            // 0x81A1B39C give: 0 when the old engine kind is unset (0x81A1AE10) or its category differs; 0 when a
            // directed old kind (0 / 8-13) goes to a root-movement one (1 / 14-19 / 21-24); else 35791 the old
            // speed, but the length of the old move vector (0x81A10830) when a root-movement old kind goes to a
            // directed one; 35789 / 35788 the vector's x / y. [M: the unit's vector ([off]) is the VM's move vector;
            // 35790 is the random speed below]
            let (pek, nek) = (self.a.kin[s].ek, wd(0) as i32);
            let directed = |e: i32| e == 0 || (8..=13).contains(&e);
            let rooted = |e: i32| matches!(e, 1 | 14..=19 | 21..=24);
            let w1 = w(1);
            let carried = if !(0..25).contains(&pek) || cat(pek) != cat(nek) || (directed(pek) && rooted(nek)) {
                0.0
            } else {
                let mv = self.unit_move(s);
                let from_vec = rooted(pek) && directed(nek);
                if w1 == 35789.0 {
                    mv[0]
                } else if w1 == 35788.0 {
                    mv[1]
                } else if from_vec {
                    libm::sqrtf(mv[0] * mv[0] + mv[1] * mv[1])
                } else {
                    self.a.kin[s].spd
                }
            };
            // 35790 (0x81A1B564): a random speed between the raw limits, Y = word 5 and X = word 7: 0 when X < Y, X
            // when they are equal, else Y + r (X - Y + the smallest normal float) with one draw r in [0, 1) (randf,
            // here the VM's seeded generator). Then for every start (0x81A1B614): the level term w2 L is added only
            // when the speed is not 0
            let base = if w1 as i32 == 35790 {
                let (x, y) = (w(7), w(5));
                if x < y {
                    0.0
                } else if x == y {
                    x
                } else {
                    let span = x - y + f32::MIN_POSITIVE;
                    y + span * self.rndf()
                }
            } else if keep(w1) || w1 == 35788.0 {
                carried
            } else {
                w1
            };
            k.spd = if base != 0.0 { base + w(2) * l } else { base };
            k.acc = if keep(w(3)) { 0.0 } else { w(3) + w(4) * l };
            // a "keep" limit keeps the slot's current limit; a slot with none caps at its start speed (no
            // unbounded acceleration) [L: the energy classes' keep handling is stripped]
            k.ek = wd(0) as i32;
            // the lower limit, word 5 (+ word 6 L), unless it holds the "none" marker
            k.min = if keep(w(5)) || sentinel(w(5)) { f32::MIN } else { w(5) + w(6) * l };
            let prev = self.a.kin[s].max;
            k.max = if !keep(w(7)) {
                w(7) + w(8) * l
            } else if prev < f32::MAX {
                prev
            } else {
                libm::fabsf(k.spd)
            };
            k.rec = 1;
            if kind < 14 && sentinel(w(7)) {
                // a built-in record's "none" upper limit (0x81A1B044: 35791 / 35795 leave it unset) [M]
                k.max = f32::MAX;
            }
            k.max_l = w(8);
            k.face = w(10);
            k.pmode = wd(12) as i32;
        }
        self.a.kin[s] = k;
    }

    /// The record of a kinetic kind (0x81A15964): kinds >= 14 from param 44000, below that the built-ins of param
    /// 43001; 0 = none.
    fn kin_rec(&self, kind: i32) -> u32 {
        let p = self.p;
        let at = if kind >= 14 {
            p.recs.iter().position(|r| r.0 == 44000).map_or(0, |r| p.rec_at(r as u16) + 64 * (kind as u32 - 14))
        } else if kind >= 0 {
            p.builtin.get(kind as usize).copied().unwrap_or(0)
        } else {
            0
        };
        if at != 0 && (at as usize + 64) <= p.data.len() { at } else { 0 }
    }

    /// A live unit's move vector for this frame (the unit's vfunc 0x10), as `kinetics_move` applies it to a directed
    /// unit (kinds 0 / 8-13: (speed x facing, 0) turned by angle x facing; kind 0 straight down) [L: the other kinds'
    /// vectors (root movement, path) are read as 0]
    fn unit_move(&self, s: usize) -> [f32; 2] {
        let k = &self.a.kin[s];
        // the gate (0x81A1C1D4): on ([off] not -1), running (0x81A1C9C4: a flag bit), not held (0x81A1CA10: [off]).
        // The VM's `done` (a limit reached, the path's end: [off], 0x81A1C110) is not that bit, and a done unit
        // still moves here, so only off and paused units are skipped [M]
        if k.kind < 0 || k.paused != 0 {
            return [0.0, 0.0];
        }
        if k.kind == 0 {
            // built-in kind 0: straight down (kinetics_move)
            return [0.0, -k.spd];
        }
        if let (Some(pa), true) = (self.i.path, matches!(k.ek, 21 | 22)) {
            // a path rider's last step: the path point it went to minus the one before (the frame has already
            // advanced by the speed) [L: the unit's own vector is not read; wraps are not followed]
            let (a, b) = ((pa.pos)(pa.id, k.pf - k.spd), (pa.pos)(pa.id, k.pf - 2.0 * k.spd));
            return [a[0] - b[0], a[1] - b[1]];
        }
        if !(k.ek == 0 || (8..=13).contains(&k.ek)) || k.seek != 0 {
            return [0.0, 0.0];
        }
        let lr = self.a.lr;
        let r = k.ang * lr * (core::f32::consts::PI / 180.0);
        [lr * k.spd * libm::cosf(r), lr * k.spd * libm::sinf(r)]
    }

    fn kinetics(&mut self) {
        let p0 = self.a.pos;
        self.kinetics_move();
        let (dx, dy) = (self.a.pos[0] - p0[0], self.a.pos[1] - p0[1]);
        self.a.moved = libm::sqrtf(dx * dx + dy * dy);
    }

    fn kinetics_move(&mut self) {
        let mut d = [0.0f32; 2];
        let mut turns = [(-1i32, 0.0f32); KIN];
        let lr = self.a.lr;
        for s in 0..KIN {
            let k = &mut self.a.kin[s];
            if k.kind < 0 || k.paused != 0 {
                continue;
            }
            if k.kind == 0 && k.rec == 0 {
                // kind 0 without a record (no 43001 in the file): a plain fall [L: not a Brawl case]
                k.spd = (k.spd + 0.1).min(3.0);
                d[1] -= k.spd;
                continue;
            }
            if k.kind == 0 {
                // built-in kind 0 (jump table 0x81A1D280 -> 0x81A1D298, the directed-unit group with kinds 8-13):
                // speed += acceleration (43001 word 3), limits skipped on the "none" marker (0x81A1E60C /
                // 0x81A1E658); its angle is word 9's marker 35790, which 0x81A1EC18 turns into 270 degrees with no
                // mirroring, then 0x81A1E5B0 turns (speed x facing, 0) by 270 x facing: straight down either way
                let before = k.spd;
                k.spd += k.acc;
                if k.min > f32::MIN && k.spd < k.min {
                    // 0x81A1E658 clamps to the lower limit whatever the speed was before
                    k.spd = k.min;
                    if before >= k.min {
                        k.done = 1;
                    }
                }
                if k.spd >= k.max {
                    k.spd = k.max;
                    k.done = 1;
                }
                d[1] -= k.spd;
                continue;
            }
            if k.rec == 0 {
                // a built-in kind the file gives no record: the host's
                continue;
            }
            // 0x81A1D1F8: speed += acceleration, then the upper limit and the lower limit (0x81A1E60C / 0x81A1E658)
            // each skipped when it holds the "none" marker; a limit reached ends the record
            let before = k.spd;
            k.spd += k.acc;
            if k.min > f32::MIN && k.spd < k.min {
                // 0x81A1E658: the speed is raised to the lower limit whatever it was before (a unit that starts
                // under its floor, e.g. at 0 with a decay, moves at the floor); only crossing it ends the record
                k.spd = k.min;
                if before >= k.min {
                    k.done = 1;
                }
            } else if k.min == f32::MIN && k.acc < 0.0 && before > 0.0 && k.spd <= 0.0 {
                // no lower limit read: a decay stops at zero [L]
                k.spd = 0.0;
                k.done = 1;
            }
            if k.spd >= k.max {
                k.spd = k.max;
                k.done = 1;
            }
            if k.ek == 20 {
                // kind 20 (0x81A1D764): turns another unit (its index came as the start's angle argument, when
                // word 9 holds the "none" marker): that unit's angle += speed x the model's scale, wrapped to
                // [0, 360)
                turns[s] = (k.ang as i32, k.spd);
                continue;
            }
            if (2..=4).contains(&k.ek) {
                // rotation units, kinds 2-4 (0x81A1D6AC -> 0x81A1E768): the posture's rotation turned about the
                // world z / x / y axis (kind 2 / 3 / 4) by the speed in degrees, mirrored by the facing; no
                // translation [L: which facing the yaw test picks is read as lr, as in 1020-1025]
                let ax = [2usize, 0, 1][(k.ek - 2) as usize];
                let rot = self.a.rot;
                self.a.rot = turn(rot, ax, k.spd * lr, true);
                continue;
            }
            if (5..=7).contains(&k.ek) {
                // kinds 5-7 (the routine's other branch): the rotation becomes the speed in degrees about x / y / z
                // alone (a rotation matrix about that axis read back as angles; the offset to it is applied)
                let mut r = [0.0f32; 3];
                r[(k.ek - 5) as usize] = k.spd;
                self.a.rot = turn(r, 0, 0.0, true);
                continue;
            }
            if let (Some(pa), true) = (self.i.path, matches!(k.ek, 21 | 22)) {
                // kinds 21 / 22 on the placement path (0x81A1D2D4): the move is the path's point at the unit's
                // frame + the offset (values 3054 / 3055: general words 22 / 23, 0x81A226F4) - the position; then
                // (0x81A1DFCC) frame += speed: past the end (frames - 1 in loop mode 1, frames otherwise; below 0,
                // or below -frames for a backward non-loop unit) loop mode wraps it by the frame count, and either
                // way the unit is marked finished (0x81A1C110). Kind 22's turn along the path is not modelled [L]
                let q = (pa.pos)(pa.id, k.pf);
                d[0] += q[0] + self.p.general[22] - self.a.pos[0] - d[0];
                d[1] += q[1] + self.p.general[23] - self.a.pos[1] - d[1];
                k.pf += k.spd;
                let n = pa.frames as i32;
                let (mut hi, mut lo) = (n, 0);
                if k.pmode != 0 {
                    hi = n - 1;
                } else if k.spd < 0.0 {
                    lo = -hi;
                    hi = 0;
                }
                if k.pf > hi as f32 || k.pf < lo as f32 {
                    if k.pmode == 1 {
                        if k.pf > hi as f32 {
                            k.pf -= n as f32;
                        }
                        if k.pf < lo as f32 {
                            k.pf += n as f32;
                        }
                    }
                    k.done = 1;
                }
                continue;
            }
            let root = matches!(k.ek, 1 | 14..=19 | 23 | 24);
            if k.ek == 23 && self.i.root_on {
                // kind 23 (0x81A1D4C4): the root movement turned by the model's rotation (0x81A06270: the posture's
                // rotation, whose yaw is the facing) and scaled by the model's scale, added to the position
                // directly (no unit angle, no speed). The model's pitch / roll are not modelled [L]
                let sc = if self.i.root_scale != 0.0 { self.i.root_scale } else { 1.0 };
                d[0] += self.i.root_move[0] * lr * sc;
                d[1] += self.i.root_move[1] * sc;
            } else if root && self.i.root_on {
                // the motion's root movement, mirrored by the facing and turned by the unit's angle (x lr); kind 24
                // reads the second translation
                let rm = if k.ek == 24 { self.i.root_move2 } else { self.i.root_move };
                let (mx, my) = (rm[0] * lr, rm[1]);
                let r = k.ang * lr * (core::f32::consts::PI / 180.0);
                let (c, sn) = (libm::cosf(r), libm::sinf(r));
                d[0] += mx * c - my * sn;
                d[1] += mx * sn + my * c;
            } else if root && s == 5 {
                // no animation from the host: the vertical root move read as the unit's speed [L]
                d[1] += k.spd;
            } else if k.seek != 0 {
                // heading for the placement point (6602): straight at it, stopping on it [L]
                let h = self.a.home;
                let (dx, dy) = (h[0] - self.a.pos[0] - d[0], h[1] - self.a.pos[1] - d[1]);
                let dist = libm::sqrtf(dx * dx + dy * dy);
                let sp = libm::fabsf(k.spd);
                if dist <= sp || dist == 0.0 {
                    d[0] += dx;
                    d[1] += dy;
                    k.spd = 0.0;
                    k.done = 1;
                    k.seek = 0;
                } else {
                    d[0] += dx / dist * sp;
                    d[1] += dy / dist * sp;
                }
            } else {
                // a directed unit (0x81A1E5B0): (speed x facing, 0) turned by angle x facing
                let r = k.ang * lr * (core::f32::consts::PI / 180.0);
                d[0] += lr * k.spd * libm::cosf(r);
                d[1] += lr * k.spd * libm::sinf(r);
            }
        }
        let sc = if self.i.root_scale != 0.0 { self.i.root_scale } else { 1.0 };
        for (t, v) in turns {
            if (0..KIN as i32).contains(&t) {
                let u = &mut self.a.kin[t as usize];
                let mut g = u.ang + v * sc;
                while g >= 360.0 {
                    g -= 360.0;
                }
                while g < 0.0 {
                    g += 360.0;
                }
                u.ang = g;
                u.dir_set = 1;
            }
        }
        // the goal snap: a move that reaches or passes a set goal axis stops on it (the core's goal-x snap)
        for (ax, bit) in [(0usize, 1u32), (1, 2)] {
            let before = self.a.pos[ax];
            let after = before + d[ax];
            let g = self.a.goal[ax];
            if self.a.goal_set & bit != 0 && d[ax] != 0.0 && (before - g) * (after - g) <= 0.0 {
                self.a.pos[ax] = g;
            } else {
                self.a.pos[ax] = after;
            }
            // the arena box (and the host's walls and floor inside it): clamp, and every moving slot's record ends at
            // the edge
            let (lo, hi) = self.limits(ax);
            if self.a.pos[ax] < lo || self.a.pos[ax] > hi {
                let g = self.a.goal[ax];
                let inside = (lo..=hi).contains(&before);
                if inside && self.a.goal_set & bit != 0 && (g > hi || g < lo) {
                    self.ev(EV_NOTE, ax as u32, 1);
                }
                let floor = ax == 1 && self.a.pos[1] < lo && lo > self.p.bounds[2];
                self.a.pos[ax] = self.a.pos[ax].clamp(lo, hi);
                if floor {
                    // landing: only the fall ends (a walk along the floor goes on)
                    for k in self.a.kin.iter_mut() {
                        if k.kind == 0 {
                            k.spd = 0.0;
                        }
                    }
                    continue;
                }
                for k in self.a.kin.iter_mut() {
                    if k.kind >= 0 && k.spd != 0.0 {
                        k.spd = 0.0;
                        k.done = 1;
                    }
                }
            }
        }
    }
}

/// A limit word holding the engine's "none" marker (its integer part 35791 or 35795, as 0x81A1E60C tests).
/// 0x81A15A00: the angle in degrees of `n` from the axis (a, 0): acos of the normalized dot product, 360 minus it
/// when n.y < 0; 0 for a zero vector.
fn vec_angle(ax: f32, n: [f32; 2]) -> f32 {
    let l = libm::sqrtf(n[0] * n[0] + n[1] * n[1]);
    let mut d = 0.0;
    if l > 0.0 {
        let c = (if ax < 0.0 { -n[0] } else { n[0] } / l).clamp(-1.0, 1.0);
        d = libm::acosf(c) * (180.0 / core::f32::consts::PI);
        if n[1] < 0.0 {
            d = 360.0 - d;
        }
    }
    d
}

/// The unit angle setter (0x81A1EF60): nothing for a category-1 engine kind (2-7, 0x81A1AE30) or the markers 35791 /
/// 35789; 35782-35787 become 0 for a category-2 kind (20), and for a category-0 kind with a negative mode word [L: the
/// mode word is not kept: kept as given]; 35781 becomes 0. Else the angle is set.
fn kin_set_ang(k: &mut Kin, v: f32) {
    if (2..=7).contains(&k.ek) || v == 35791.0 || v == 35789.0 {
        return;
    }
    let marker = (35782.0..=35787.0).contains(&v) && v == libm::truncf(v);
    let v = if v == 35781.0 || (marker && k.ek == 20) { 0.0 } else { v };
    k.ang = v;
    k.dir_set = 1;
}

/// 0x81A25138 from P on: a zero P (0x81A10830: length, 0 at or under the smallest normal) gives 3; else the angle
/// between P and d (8003DDD0: 0 when the product of the squared lengths is 0, else acos of the clamped cosine) times
/// 57.29578
fn cone_angle(p: [f32; 2], d: [f32; 2]) -> f32 {
    let sq = p[0] * p[0] + p[1] * p[1];
    let len = if libm::fabsf(sq) <= f32::MIN_POSITIVE { 0.0 } else { sq * (1.0 / libm::sqrtf(sq)) };
    if len == 0.0 {
        return 3.0;
    }
    let prod = sq * (d[0] * d[0] + d[1] * d[1]);
    if prod == 0.0 {
        return 0.0;
    }
    let c = ((p[0] * d[0] + p[1] * d[1]) * (1.0 / libm::sqrtf(prod))).clamp(-1.0, 1.0);
    (libm::acos(c as f64) as f32) * 57.29578
}

fn sentinel(x: f32) -> bool {
    let i = x as i32;
    i == 35791 || i == 35795
}

/// Euler angles (degrees, applied x then y then z: R = Rz Ry Rx) turned about axis `ax` by `deg` (the world axis:
/// R' = Raxis R), back to angles, as the kinetic module's matrix / angle conversions do.
fn turn(e: [f32; 3], ax: usize, deg: f32, world: bool) -> [f32; 3] {
    use libm::{atan2f, cosf, sinf, sqrtf};
    let k = core::f32::consts::PI / 180.0;
    let (sx, cx, sy, cy, sz, cz) = (sinf(e[0] * k), cosf(e[0] * k), sinf(e[1] * k), cosf(e[1] * k), sinf(e[2] * k), cosf(e[2] * k));
    let m = [
        [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz],
        [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz],
        [-sy, sx * cy, cx * cy],
    ];
    let (s, c) = (sinf(deg * k), cosf(deg * k));
    let r = match ax {
        0 => [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]],
        1 => [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]],
        _ => [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]],
    };
    let (a, b) = if world { (r, m) } else { (m, r) };
    let mut o = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            o[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    let y = atan2f(-o[2][0], sqrtf(o[0][0] * o[0][0] + o[1][0] * o[1][0]));
    let x = atan2f(o[2][1], o[2][2]);
    let z = atan2f(o[1][0], o[0][0]);
    [x / k, y / k, z / k]
}

fn thread_at(code: u32, tick: u32) -> Th {
    if code == NONE || code == 0 {
        return TH0;
    }
    let mut t = TH0;
    t.nf = 1;
    t.fr[0].pc = code;
    t.resume = tick as f32;
    t
}
