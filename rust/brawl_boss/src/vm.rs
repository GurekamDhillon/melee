//! The enemy-command VM: runs Brawl's enemy / boss command scripts (`KK II nn` words) directly, for every Subspace
//! boss and enemy, from the player's own files. Semantics: `_build/boss-ir/engine-ops.md` and `enemies/engine.md`
//! (local notes, traced from the code; never committed), plus the Petey runner ([`crate::runner`]) for the shared
//! control commands.
//!
//! Two halves:
//! * [`Program`] (load time, allocates): the boss file and the shared script file (`adventure_common` leaf 333) are
//!   compiled into one flat bytecode array. Every script pointer becomes a code index, every `if` / `loop` / `switch`
//!   gets its jump targets precomputed, constant parameters are folded into the argument pool, and the rules are
//!   sorted by priority so a pass only visits the rules inside its window.
//! * [`Actor`] (run time): all state in fixed-size arrays, `Copy`, no pointers. A rollback snapshot is a plain copy
//!   of the struct; [`Actor::hash`] folds it into the netplay hash. [`step`] never allocates.
//!
//! Presentation commands (effects, sounds, hitbox definitions, partial motions, colours, rumble) are not interpreted:
//! they become [`Ev`] records naming the instruction, and the host reads the instruction's arguments itself. Commands
//! the VM neither runs nor forwards are listed by [`Program::unknown`]; the coverage test reads that list.

use crate::file::BossFile;
use crate::script::{Arg, Target, Var, MEM_PARAM, MEM_TEMP, MEM_WORK};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

// ------------------------------------------------------------------------------------------------ sizes

/// Work ints / floats per actor (ids above are a load diagnostic).
pub const NI: usize = 48;
pub const NF: usize = 48;
/// Work flags (bits).
pub const NB: usize = 64;
/// Temporaries (rule engine and condition scripts).
pub const TI: usize = 16;
pub const TF: usize = 16;
pub const TB: usize = 32;
/// Threads: 0 = status loop, 1..=9 = the motion's timelines (game / effect / sound x 3), 10 = the damage proc.
pub const THREADS: usize = 11;
const T_LOOP: usize = 0;
const T_DPROC: usize = 10;
/// Call depth and loop nesting per frame.
pub const FRAMES: usize = 6;
pub const LOOPS: usize = 4;
pub const MAX_RULES: usize = 128;
/// Kinetic slots (engine.md 6: up to 8).
pub const KIN: usize = 8;
/// Presentation events per frame.
pub const EVQ: usize = 64;
/// Instructions one thread may run in one frame (a runaway script is stopped, not hung).
const BUDGET: u32 = 4096;
const NONE: u32 = u32::MAX;
/// Sentinel instruction id: end of a script.
const END: u16 = 0xFFFF;

/// The "keep the current value" kinetic sentinel (35791.0) and its neighbours.
const KEEP_LO: f32 = 35789.0;
const KEEP_HI: f32 = 35791.0;

// ------------------------------------------------------------------------------------------------ program

/// One compiled argument.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum A {
    /// A constant (ints, scalars, bools and constant parameters).
    K(f32),
    /// Work int / float / flag.
    Wi(u8),
    Wf(u8),
    Wb(u8),
    /// Temp int / float / flag.
    Ti(u8),
    Tf(u8),
    Tb(u8),
    /// An engine value read at run time (memory 0, not a constant parameter).
    Eng(u32),
    /// A record parameter (44xxx): the index into [`Program::recs`].
    Rec(u16),
    /// A code index (script pointer).
    Code(u32),
    /// A requirement.
    Req { not: bool, id: u32 },
    /// Anything else, as stored.
    Raw(u32, u32),
}

/// One instruction. `j` / `k` are precomputed jumps (meaning per command, see `link_control`).
#[derive(Clone, Copy, Debug)]
pub struct Ins {
    pub id: u16,
    pub n: u8,
    pub a: u32,
    pub j: u32,
    pub k: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RuleP {
    pub prio: i32,
    pub mask: u32,
    pub cond: u32,
    pub action: u32,
    pub status: i32,
    pub on: bool,
    pub lock: bool,
}

/// A weighted list (switch 0010): entries (base, slope per L, condition code).
#[derive(Clone, Debug)]
pub struct WList {
    pub id: u32,
    pub e: Vec<(f32, f32, u32)>,
}

/// What the loader could not place (the coverage test prints these).
#[derive(Clone, Debug, Default)]
pub struct Diag {
    /// command id -> occurrences the VM neither runs nor forwards
    pub unknown_cmds: BTreeMap<u16, u32>,
    /// requirement ids the VM does not answer
    pub unknown_reqs: BTreeMap<u32, u32>,
    /// engine values (memory 0) the VM does not answer
    pub unknown_values: BTreeMap<u32, u32>,
    /// imports with no script in the shared file
    pub missing_imports: Vec<String>,
    /// variables outside the fixed arrays: (mem, kind, id)
    pub vars_out_of_range: Vec<(u8, u8, u32)>,
    /// parameter ids used but absent from the parameter map
    pub unmapped_params: BTreeMap<u32, u32>,
}

/// A compiled enemy / boss.
#[derive(Clone, Debug)]
pub struct Program {
    pub code: Vec<Ins>,
    pub args: Vec<A>,
    /// file offsets of the compiled scripts (main file), for the host's argument reads
    pub status_enter: Vec<u32>,
    pub status_loop: Vec<u32>,
    pub status_exit: Vec<u32>,
    /// `[motion][category * 3 + thread]`
    pub timelines: Vec<[u32; 9]>,
    pub motion_loop: Vec<bool>,
    /// frames per motion (the host fills these from the animation files; default 60)
    pub motion_len: Vec<u16>,
    pub motion_names: Vec<String>,
    pub rules: Vec<RuleP>,
    /// rule indices sorted by priority (stable: table order inside one priority)
    pub by_prio: Vec<u16>,
    pub init: u32,
    /// record parameters: (id, data offset)
    pub recs: Vec<(u32, u32)>,
    pub lists: Vec<WList>,
    /// the main file's data (record reads: 6507 / 6509 / 651E / 66xx)
    pub data: Vec<u8>,
    pub tuning: [f32; 44],
    /// data offset of the tuning block (record word 7)
    pub tuning_at: u32,
    /// built-in movement kinds 0-13 (param 43001, from the shared file's commonData): each a data offset (into the
    /// appended copies) of a 16-word record like 44000's, or 0 for none
    pub builtin: [u32; 14],
    /// code index of the shared statusAnimCmd_CommonCatchProcExit (0 = not used): a call to it sends EV_CATCH_EXIT
    pub catch_exit: u32,
    pub general: [f32; 44],
    /// param 43006 (the accessor 0x81A229F8: word 2 of the file's data root): a (list, count) of hurtbox indices,
    /// handed to emCollisionHitModuleImpl's constructor (0x81A2B8FC, from 0x81A00CC4). 6D00 sets them all
    pub hurt_list: Vec<u32>,
    /// engine value 21007 (0x81A22134): the placement's variant; the six Primids share one file and
    /// branch on it [L: the field's writer is not traced]. The host sets it.
    pub variant: i32,
    /// engine value 21009 (0x81A22118): the create record's word the enemy constructor keeps (0x81A003E8) [L: its
    /// meaning is not traced]. The host sets it; 0 by default
    pub place_word: i32,
    /// work-variable initial values the IR supplies (applied before the init script): (raw var, value)
    pub inits: Vec<(u32, f32)>,
    /// engine values 23000+ (the enemy's data-block words): (id, block 0 general / 1 tuning, offset), from the local
    /// value table [`block_values`]; an id absent from it reads 0
    pub block_values: Vec<(u32, u8, u32)>,
    /// the arena box the movement stays in: x min, x max, y min, y max. Reaching an edge stops the slots moving
    /// into it and finishes their records, the way the stage's walls and floor end a native move [L]. The host sets it
    /// from the arena; the default is a wide box.
    pub bounds: [f32; 4],
    pub diag: Diag,
}

/// The parameter map from the IR: (param id, offset inside the parameter block).
pub type ParamMap<'a> = &'a [(u32, u32)];

fn be32(d: &[u8], at: u32) -> u32 {
    let a = at as usize;
    if a + 4 > d.len() {
        return 0;
    }
    u32::from_be_bytes([d[a], d[a + 1], d[a + 2], d[a + 3]])
}
fn fbits(w: u32) -> f32 {
    let f = f32::from_bits(w);
    if f.is_nan() {
        0.0
    } else {
        f
    }
}

/// Every command id the VM runs itself.
pub const RUN_IDS: &[u16] = &[
    0x0001, 0x0002, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000A, 0x000B, 0x000C, 0x000D, 0x000E, 0x000F,
    0x0010, 0x0011, 0x0012, 0x0013, 0x0018, 0x0100, 0x0102, 0x0202, 0x0409, 0x0500, 0x0501, 0x0504, 0x0505, 0x0509,
    0x050A, 0x050B, 0x0605, 0x0607, 0x0808, 0x0900, 0x0C01, 0x0C02, 0x0C03, 0x0C04, 0x0D00, 0x1200, 0x1201, 0x1203,
    0x1205, 0x1206, 0x1207, 0x1208, 0x120A, 0x120B, 0x120F, 0x1210, 0x1212, 0x1400, 0x1401, 0x1500, 0x1600, 0x1900,
    0x1901, 0x1902, 0x1B02, 0x6500, 0x6504, 0x6505, 0x6507, 0x6509, 0x6514, 0x6516, 0x651E, 0x651F, 0x6523, 0x6600,
    0x6601, 0x6602, 0x6603, 0x6607, 0x6609, 0x660A, 0x660B, 0x660C, 0x6616, 0x6617, 0x661A, 0x661B, 0x6900, 0x6901, 0x6902, 0x6907, 0x6908, 0x6909, 0x690A, 0x6A00, 0x6A01,
    0x6A02, 0x6A03, 0x6B03, 0x1405, 0x6524, 0x6B01, 0x6604, 0x6605, 0x660E, 0x6905, 0x6906, 0x6520, 0x0502, 0x660F, 0x6506, 0x6510, 0x6511, 0x6515, 0x030A, 0x661C,
    0x661D, 0x661E, 0x6D00, 0x100D, 0x6508, 0x651B, 0x6521,
];

/// Commands forwarded to the host as presentation events, not interpreted: motion-module rate / partial-motion
/// commands (04, except 0409), attack / hitbox definitions (06, except the hurt states), ground-collision tuning
/// (08, except 0808), sounds (0A), damage-module reactions (0D, except 0D00), effects (11), the attack-group reset
/// (18), shakes / rumble (1A), hit colours (1C, 1D, 1E), the model-rotation module (6C: 6C00 starts a rotation of the
/// model toward a target angle, 0x81A2A888 [M]), the link events (6E), weapon / helper spawns (650E, 650F,
/// 651A), stage notices with a value (6513), the common effects (651D), and the kind-65 ids that act on other modules
/// or objects: 6502 (emInfo node query), 6517, 651C (an effect), 6525 (0x81A04EB8, a ground-module helper), 6528
/// (0x81A06F3C, creates a kind-0xB object), 6529 [L], 6527 flag (0x81A08E0C -> emGroundModuleImpl 0x81A2B630: walks the
/// ground module's shape pairs through 0x81A2B52C with the flag, in reverse when the module's float is 0 and work flag
/// 0x12000003 is clear: ground-collision shapes, the host's).
/// Links between actors, kept by the host: the link module's commands 0F00 / 0F01 / 0F02 (command kind 0x0F,
/// soLinkModuleImpl, isObserv 0x8106FD18) and 6519 var (0x81A09860 -> 0x81A0B438 -> 0x81A0B1E8: through the enemy
/// manager (0x81A0C804) copies the variable's value (int, float or flag by its kind) into the same variable of the
/// linked enemy). Also the model-node transforms 0302 / 0304 / 0306
/// (kind 0x03, the model-node module: a node's rotation / translation / scale from variables) [M from their use: set
/// every frame from random values, never read back].
pub fn forwarded(id: u16) -> bool {
    let m = id >> 8;
    (matches!(m, 0x04 | 0x06 | 0x08 | 0x0A | 0x0D | 0x11 | 0x18 | 0x1A | 0x0E | 0x14 | 0x1C | 0x1D | 0x1E | 0x6C | 0x6E) && !RUN_IDS.contains(&id))
        || matches!(id, 0x650E | 0x650F | 0x6513 | 0x651A | 0x651D | 0x6502 | 0x6517 | 0x651C | 0x6525 | 0x6528 | 0x6529 | 0x0302 | 0x0304 | 0x0306 | 0x6527 | 0x6519 | 0x0F00 | 0x0F01 | 0x0F02)
}

/// Requirement ids the VM answers.
pub const REQ_IDS: &[u32] = &[
    1, 2, 3, 4, 6, 7, 8, 9, 10, 12, 15, 16, 17, 18, 19, 20, 22, 47, 73, 10000, 10001, 10002, 10003, 10004, 10021, 10022, 10023, 37, 10005, 10008, 10009, 10010, 10012, 10013, 10014, 10015, 10016, 10017, 10018, 10019, 10020,
    10050, 10051, 10052, 10053, 10054, 10055, 10057, 10059, 56, 60, 75, 63, 64, 65, 66, 67, 68, 69, 10024, 10025, 10026, 10027, 10028, 10029, 10030, 10031, 10032, 10033, 10034, 10035,
];

/// Engine values (memory 0) the VM answers.
fn known_value(id: u32) -> bool {
    matches!(id, 3 | 4 | 8 | 9 | 15 | 21 | 20000 | 1020 | 1021 | 1022 | 1023 | 1024 | 1025 | 1026 | 1049 | 1050 | 1028 | 1029 | 3056 | 3057 | 20001 | 20003 | 21000 | 21001 | 21002 | 21004 | 21007 | 21008 | 21009 | 21010 | 41000)
        || (23000..=23099).contains(&id)
}

struct Builder<'a> {
    p: Program,
    pmap: ParamMap<'a>,
    pblock: u32,
    /// (file 0 main / 1 common, data offset) -> placeholder id (0x8000_0000 | queue index)
    at: BTreeMap<(u8, u32), u32>,
    /// the compile queue (append-only) and each entry's final code index
    queue: Vec<(u8, u32)>,
    done: Vec<u32>,
    /// common root name -> offset
    common_roots: BTreeMap<String, u32>,
}

impl Program {
    /// Compile a boss / enemy file. `common` is the shared script file (adventure_common leaf 333), parsed with
    /// [`BossFile::parse_raw`]; `params` maps parameter ids to offsets inside the parameter block (from the IR).
    pub fn build(main: &BossFile, common: Option<&BossFile>, params: ParamMap<'_>) -> Result<Program, &'static str> {
        let rec = main.root("data").ok_or("no data record")?;
        let w = |i: u32| rec + 4 * i;
        let mut b = Builder {
            p: Program {
                code: Vec::new(),
                args: Vec::new(),
                status_enter: Vec::new(),
                status_loop: Vec::new(),
                status_exit: Vec::new(),
                timelines: Vec::new(),
                motion_loop: Vec::new(),
                motion_len: Vec::new(),
                motion_names: Vec::new(),
                rules: Vec::new(),
                by_prio: Vec::new(),
                init: NONE,
                recs: Vec::new(),
                lists: Vec::new(),
                data: main.data().to_vec(),
                tuning: [0.0; 44],
                tuning_at: 0,
                builtin: [0; 14],
                catch_exit: 0,
                general: [0.0; 44],
                hurt_list: Vec::new(),
                variant: 0,
                place_word: 0,
                inits: Vec::new(),
                block_values: Vec::new(),
                bounds: [-250.0, 250.0, -100.0, 280.0],
                diag: Diag::default(),
            },
            pmap: params,
            pblock: main.local(w(8)).unwrap_or(0),
            at: BTreeMap::new(),
            common_roots: BTreeMap::new(),
            queue: Vec::new(),
            done: Vec::new(),
        };
        if let Some(c) = common {
            for r in &c.roots {
                b.common_roots.insert(r.name.clone(), r.at);
            }
        }
        // param 44000 (the movement records, block +0) is read by every 66xx start, referenced or not
        let pb = b.pblock;
        b.rec(44000, pb);
        // param 43001, the built-in kinds' records: the enemy accessor answers ids 43000-43039 (sora_enemy
        // 0x81A229F8) from the resource module's commonData root, the shared file's (emResourceModuleImpl's
        // constructor 0x81A1F57C finds it by name); 43001 is its word 2, an array of 14 record pointers (0 = none),
        // copied into the enemy at module init. The records are appended to the program's data.
        if let Some(c) = common {
            if let Some(root) = c.root("commonData") {
                if let Ok(arr) = c.local(root + 8) {
                    for k in 0..14u32 {
                        if let Ok(e) = c.local(arr + 4 * k) {
                            let cd = c.data();
                            if (e as usize + 64) <= cd.len() {
                                let at = b.p.data.len() as u32;
                                b.p.data.extend_from_slice(&cd[e as usize..e as usize + 64]);
                                b.p.builtin[k as usize] = at;
                            }
                        }
                    }
                }
            }
        }
        // the program starts with one END so "no script" can point somewhere harmless
        b.p.code.push(Ins { id: END, n: 0, a: 0, j: 0, k: 0 });
        for i in 0..44u32 {
            b.p.general[i as usize] = fbits(be32(main.data(), 4 * i));
        }
        if let Ok(h) = main.local(w(2)) {
            let n = be32(main.data(), h + 4) as i32;
            if let Ok(l) = main.local(h) {
                for i in 0..n.clamp(0, 64) as u32 {
                    b.p.hurt_list.push(be32(main.data(), l + 4 * i));
                }
            }
        }
        if let Ok(t) = main.local(w(7)) {
            b.p.tuning_at = t;
            for i in 0..44u32 {
                b.p.tuning[i as usize] = fbits(be32(main.data(), t + 4 * i));
            }
        }
        // motions
        let mt = main.local(w(0)).map_err(|_| "no motion table")?;
        let nm = ((main.next_object(mt) - mt) / 8).min(256);
        for i in 0..nm {
            let Ok(s) = main.local(mt + 8 * i + 4) else { break };
            let flags = be32(main.data(), mt + 8 * i);
            let tail = &main.data()[s as usize..];
            let end = tail.iter().position(|&c| c == 0).unwrap_or(0);
            b.p.motion_names.push(String::from_utf8_lossy(&tail[..end]).into_owned());
            b.p.motion_loop.push(flags & 0x0002_0000 != 0);
            b.p.motion_len.push(60);
        }
        let nm = b.p.motion_names.len() as u32;
        // timelines: record words 10..=18
        let mut tl = vec![[0u32; 9]; nm as usize];
        for ct in 0..9u32 {
            let tab = match main.ptr_field(w(10 + ct)) {
                Ok(Target::Local(o)) => o,
                _ => continue,
            };
            for m in 0..nm {
                let t = main.ptr_field(tab + 4 * m).unwrap_or(Target::None);
                tl[m as usize][ct as usize] = b.target(main, 0, t);
            }
        }
        b.p.timelines = tl;
        // status tables: 19 loop, 20 exit, 21 enter. Their length: up to the next object.
        for (word, which) in [(19u32, 0u8), (20, 1), (21, 2)] {
            let Ok(tab) = main.local(w(word)) else { continue };
            let n = ((main.next_object(tab) - tab) / 4).min(128);
            let mut v = Vec::with_capacity(n as usize);
            for s in 0..n {
                let t = main.ptr_field(tab + 4 * s).unwrap_or(Target::None);
                v.push(b.target(main, 0, t));
            }
            match which {
                0 => b.p.status_loop = v,
                1 => b.p.status_exit = v,
                _ => b.p.status_enter = v,
            }
        }
        let ns = b.p.status_enter.len().max(b.p.status_loop.len()).max(b.p.status_exit.len());
        for v in [&mut b.p.status_enter, &mut b.p.status_loop, &mut b.p.status_exit] {
            v.resize(ns, 0);
        }
        // rules
        if let Ok(rt) = main.local(w(22)) {
            let n = ((main.next_object(rt) - rt) / 4).min(MAX_RULES as u32);
            for i in 0..n {
                let Ok(at) = main.local(rt + 4 * i) else { break };
                let d = main.data();
                let cond = main.ptr_field(at + 8).unwrap_or(Target::None);
                let act = main.ptr_field(at + 12).unwrap_or(Target::None);
                let r = RuleP {
                    prio: be32(d, at) as i32,
                    mask: be32(d, at + 4),
                    cond: if cond == Target::None { NONE } else { b.target(main, 0, cond) },
                    action: b.target(main, 0, act),
                    status: be32(d, at + 16) as i32,
                    on: d.get(at as usize + 20).copied().unwrap_or(0) != 0,
                    lock: d.get(at as usize + 21).copied().unwrap_or(0) != 0,
                };
                b.p.rules.push(r);
            }
        }
        let mut order: Vec<u16> = (0..b.p.rules.len() as u16).collect();
        order.sort_by_key(|&i| b.p.rules[i as usize].prio);
        b.p.by_prio = order;
        let init = main.ptr_field(w(24)).unwrap_or(Target::None);
        b.p.init = b.target(main, 0, init);
        // weighted lists and record params come in as the scripts reference them
        let mut qi = 0;
        while qi < b.queue.len() {
            let (f, o) = b.queue[qi];
            let start = b.p.code.len() as u32;
            b.done.push(start);
            match (f, common) {
                (0, _) => b.compile(main, 0, o),
                (_, Some(c)) => b.compile(c, 1, o),
                _ => b.p.code.push(Ins { id: END, n: 0, a: 0, j: 0, k: 0 }),
            }
            qi += 1;
        }
        b.link();
        Ok(b.p)
    }

    /// Where a param id lives in the data (record params and values).
    pub fn rec_at(&self, rec: u16) -> u32 {
        self.recs.get(rec as usize).map_or(0, |r| r.1)
    }
    pub fn rec_f(&self, rec: u16, i: u32) -> f32 {
        fbits(be32(&self.data, self.rec_at(rec) + 4 * i))
    }
    pub fn rec_w(&self, rec: u16, i: u32) -> u32 {
        be32(&self.data, self.rec_at(rec) + 4 * i)
    }

    /// Commands, requirements and values the VM cannot place, as one list (empty = full coverage).
    pub fn unknown(&self) -> Vec<String> {
        use core::fmt::Write;
        let mut o = Vec::new();
        for (k, v) in &self.diag.unknown_cmds {
            let mut s = String::new();
            let _ = write!(s, "cmd {k:04X} x{v}");
            o.push(s);
        }
        for (k, v) in &self.diag.unknown_reqs {
            let mut s = String::new();
            let _ = write!(s, "req R{k} x{v}");
            o.push(s);
        }
        for (k, v) in &self.diag.unknown_values {
            let mut s = String::new();
            let _ = write!(s, "value {k} x{v}");
            o.push(s);
        }
        for n in &self.diag.missing_imports {
            let mut s = String::new();
            let _ = write!(s, "import {n}");
            o.push(s);
        }
        for (m, k, id) in &self.diag.vars_out_of_range {
            let mut s = String::new();
            let _ = write!(s, "var {m}.{k}.{id} out of range");
            o.push(s);
        }
        o
    }
}

impl<'a> Builder<'a> {
    /// The code index for a script pointer (queues the script for compilation).
    fn target(&mut self, f: &BossFile, file: u8, t: Target) -> u32 {
        match t {
            Target::Local(o) => self.entry(file, o),
            Target::Import(i) => {
                let name = f.import_name(i).unwrap_or("");
                match self.common_roots.get(name).copied() {
                    Some(o) => {
                        let c = self.entry(1, o);
                        if name == "statusAnimCmd_CommonCatchProcExit" {
                            self.p.catch_exit = c;
                        }
                        c
                    }
                    None => {
                        if !self.p.diag.missing_imports.iter().any(|n| n == name) {
                            self.p.diag.missing_imports.push(String::from(name));
                        }
                        0
                    }
                }
            }
            _ => 0,
        }
    }
    fn entry(&mut self, file: u8, o: u32) -> u32 {
        if let Some(&c) = self.at.get(&(file, o)) {
            return c;
        }
        let id = 0x8000_0000 | self.queue.len() as u32;
        self.queue.push((file, o));
        self.at.insert((file, o), id);
        id
    }

    fn param(&mut self, id: u32) -> A {
        if let Some(&(_, off)) = self.pmap.iter().find(|x| x.0 == id) {
            let at = self.pblock + off;
            let d = &self.p.data;
            return match id {
                24000..=24999 => A::K(be32(d, at) as i32 as f32),
                40000..=49999 => A::Rec(self.rec(id, at)),
                _ => A::K(fbits(be32(d, at))),
            };
        }
        if id == 44000 {
            let at = self.pblock;
            return A::Rec(self.rec(id, at));
        }
        if known_value(id) {
            return A::Eng(id);
        }
        if (4000..=4999).contains(&id) || (24000..=24999).contains(&id) || (40000..=49999).contains(&id) {
            *self.p.diag.unmapped_params.entry(id).or_default() += 1;
        } else {
            *self.p.diag.unknown_values.entry(id).or_default() += 1;
        }
        A::Eng(id)
    }
    fn rec(&mut self, id: u32, at: u32) -> u16 {
        if let Some(i) = self.p.recs.iter().position(|r| r.0 == id) {
            return i as u16;
        }
        self.p.recs.push((id, at));
        (self.p.recs.len() - 1) as u16
    }

    fn var(&mut self, v: Var) -> A {
        let id = v.id;
        let fits = |n: usize| (id as usize) < n;
        let a = match (v.mem, v.kind) {
            (MEM_PARAM, _) => return self.param(id),
            (MEM_WORK, 0) if fits(NI) => A::Wi(id as u8),
            (MEM_WORK, 1) if fits(NF) => A::Wf(id as u8),
            (MEM_WORK, 2) if fits(NB) => A::Wb(id as u8),
            (MEM_TEMP, 0) if fits(TI) => A::Ti(id as u8),
            (MEM_TEMP, 1) if fits(TF) => A::Tf(id as u8),
            (MEM_TEMP, 2) if fits(TB) => A::Tb(id as u8),
            _ => {
                let k = (v.mem, v.kind, id);
                if !self.p.diag.vars_out_of_range.contains(&k) {
                    self.p.diag.vars_out_of_range.push(k);
                }
                A::Raw(5, v.raw())
            }
        };
        a
    }

    fn compile(&mut self, f: &BossFile, file: u8, o: u32) {
        let mut at = o;
        let mut n_cmds = 0;
        loop {
            let (wd, a1) = (be32(f.data(), at), be32(f.data(), at + 4));
            if (wd == 0 && a1 == 0) || n_cmds > 4096 {
                break;
            }
            let Ok(cmd) = f.decode_cmd(at) else { break };
            let id = cmd.id();
            let a = self.p.args.len() as u32;
            for x in &cmd.args {
                let c = match *x {
                    Arg::Int(i) => A::K(i as f32),
                    Arg::Scalar(s) => A::K(s.raw() as f32 / 60000.0),
                    Arg::Bool(b) => A::K(b as i32 as f32),
                    Arg::Var(v) => self.var(v),
                    Arg::Ptr(t) => A::Code(self.target(f, file, t)),
                    Arg::Req { not, id } => {
                        if !REQ_IDS.contains(&id) {
                            *self.p.diag.unknown_reqs.entry(id).or_default() += 1;
                        }
                        A::Req { not, id }
                    }
                    other => {
                        let (t, v) = other.to_pair();
                        A::Raw(t, v)
                    }
                };
                self.p.args.push(c);
            }
            if !RUN_IDS.contains(&id) && !forwarded(id) {
                *self.p.diag.unknown_cmds.entry(id).or_default() += 1;
            }
            // weighted lists (0010 switch): the list param's entries {int base, float slope, ptr cond}
            if id == 0x0010 && cmd.args.len() == 3 {
                if let (Arg::Var(v), Arg::Int(cnt)) = (cmd.args[1], cmd.args[2]) {
                    self.list(f, v.id, cnt.max(0) as u32);
                }
            }
            self.p.code.push(Ins { id, n: cmd.args.len() as u8, a, j: 0, k: 0 });
            at += 8;
            n_cmds += 1;
        }
        self.p.code.push(Ins { id: END, n: 0, a: 0, j: 0, k: 0 });
    }

    fn list(&mut self, f: &BossFile, id: u32, n: u32) {
        if self.p.lists.iter().any(|l| l.id == id) {
            return;
        }
        let Some(&(_, off)) = self.pmap.iter().find(|x| x.0 == id) else {
            *self.p.diag.unmapped_params.entry(id).or_default() += 1;
            self.p.lists.push(WList { id, e: Vec::new() });
            return;
        };
        let at = self.pblock + off;
        let mut e = Vec::new();
        for i in 0..n.min(16) {
            let p = at + 12 * i;
            let base = be32(f.data(), p) as i32 as f32;
            let slope = fbits(be32(f.data(), p + 4));
            let c = f.ptr_field(p + 8).unwrap_or(Target::None);
            let c = if c == Target::None { NONE } else { self.target(f, 0, c) };
            e.push((base, slope, c));
        }
        self.p.lists.push(WList { id, e });
    }

    /// Resolve placeholder code indices and precompute control-flow jumps.
    fn link(&mut self) {
        let done = &self.done;
        let fix = |x: &mut u32| {
            if *x != NONE && *x & 0x8000_0000 != 0 {
                *x = done.get((*x & 0x7FFF_FFFF) as usize).copied().unwrap_or(0);
            }
        };
        for a in self.p.args.iter_mut() {
            if let A::Code(c) = a {
                fix(c);
            }
        }
        for v in [&mut self.p.status_enter, &mut self.p.status_loop, &mut self.p.status_exit] {
            v.iter_mut().for_each(fix);
        }
        for t in self.p.timelines.iter_mut() {
            t.iter_mut().for_each(fix);
        }
        for r in self.p.rules.iter_mut() {
            fix(&mut r.cond);
            fix(&mut r.action);
        }
        for l in self.p.lists.iter_mut() {
            for e in l.e.iter_mut() {
                fix(&mut e.2);
            }
        }
        fix(&mut self.p.init);
        fix(&mut self.p.catch_exit);
        link_control(&mut self.p.code);
    }
}

/// Jumps: `if` / `elif`: j = the next `elif` / `else` / `endif` of the same block, k = its `endif`; `else`: j = the
/// `endif`; `loop`: j = its `endloop`; `endloop`: j = its `loop`; `switch`: j = its `endswitch`; `break`: j = where it
/// lands, k = 1 when it leaves a loop.
fn link_control(code: &mut [Ins]) {
    let n = code.len();
    let mut i = 0;
    while i < n {
        match code[i].id {
            0x000A | 0x000D | 0x000E => {
                // next branch point and the endif at depth 0
                let mut d = 0i32;
                let mut next = NONE;
                let mut endif = NONE;
                let mut p = i + 1;
                while p < n && code[p].id != END {
                    match code[p].id {
                        0x000A => d += 1,
                        0x000F if d == 0 => {
                            endif = p as u32;
                            if next == NONE {
                                next = p as u32;
                            }
                            break;
                        }
                        0x000F => d -= 1,
                        0x000D | 0x000E if d == 0 && next == NONE => next = p as u32,
                        _ => {}
                    }
                    p += 1;
                }
                if endif == NONE {
                    endif = p as u32;
                    if next == NONE {
                        next = p as u32;
                    }
                }
                code[i].j = next;
                code[i].k = endif;
            }
            0x0004 => {
                let mut d = 0i32;
                let mut p = i + 1;
                while p < n && code[p].id != END {
                    match code[p].id {
                        0x0004 => d += 1,
                        0x0005 if d == 0 => break,
                        0x0005 => d -= 1,
                        _ => {}
                    }
                    p += 1;
                }
                code[i].j = p as u32;
                if p < n && code[p].id == 0x0005 {
                    code[p].j = i as u32;
                }
            }
            0x0010 => {
                let mut d = 0i32;
                let mut p = i + 1;
                while p < n && code[p].id != END {
                    match code[p].id {
                        0x0010 => d += 1,
                        0x0013 if d == 0 => break,
                        0x0013 => d -= 1,
                        _ => {}
                    }
                    p += 1;
                }
                code[i].j = p as u32;
                // a case / default reached while running the case before it ends that case: it jumps past the
                // endswitch (the scripts have no break between cases, e.g. Ridley's MenuA)
                let end = p as u32;
                let mut d2 = 0i32;
                for q in i + 1..p {
                    match code[q].id {
                        0x0010 => d2 += 1,
                        0x0013 => d2 -= 1,
                        0x0011 | 0x0012 if d2 == 0 => code[q].j = end,
                        _ => {}
                    }
                }
            }
            0x0018 => {
                let (mut sw, mut lp) = (0i32, 0i32);
                let mut p = i + 1;
                let mut k = 0;
                while p < n && code[p].id != END {
                    match code[p].id {
                        0x0010 => sw += 1,
                        0x0013 if sw == 0 => {
                            p += 1;
                            break;
                        }
                        0x0013 => sw -= 1,
                        0x0004 => lp += 1,
                        0x0005 if lp == 0 => {
                            p += 1;
                            k = 1;
                            break;
                        }
                        0x0005 => lp -= 1,
                        _ => {}
                    }
                    p += 1;
                }
                code[i].j = p as u32;
                code[i].k = k;
            }
            _ => {}
        }
        i += 1;
    }
}

// ------------------------------------------------------------------------------------------------ the files

/// FNV-1a 64 (identifies a leaf).
pub fn fnv(b: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for &x in b {
        h = (h ^ x as u64).wrapping_mul(0x0100_0000_01b3);
    }
    h
}
/// Every leaf of an archive (LZ77 and nested ARCs unpacked).
pub fn leaves(d: &[u8], out: &mut Vec<Vec<u8>>, depth: u32) {
    let Ok(u) = crate::archive::unpack(d) else { return };
    if crate::archive::is_arc(&u) {
        if depth > 6 {
            return;
        }
        if let Ok(es) = crate::archive::arc_entries(&u) {
            for e in es {
                leaves(e.body, out, depth + 1);
            }
        }
    } else {
        out.push(u.into_owned());
    }
}

/// Which leaf of which archive each boss / enemy is: (name, IR file stem, archive under `stage/adventure`, FNV-1a 64
/// of the leaf; 0 = the archive's first boss-shaped file). Hashes identify the player's files; they hold no data.
pub const FILES: &[(&str, &str, &str, u64)] = &[
    ("rayquaza", "rayquaza", "080001.pac", 0),
    ("ridley", "ridley", "280301.pac", 0),
    ("galleom", "galleom", "240101.pac", 0),
    ("duon", "duon", "350001.pac", 0),
    ("porky", "porky", "090101.pac", 0),
    ("metaridley", "metaridley", "370301.pac", 0),
    ("tabuu", "tabuu", "920801.pac", 0),
    ("petey", "petey", "030101.pac", 0),
    ("auroros", "enemies/auroros", "080101.pac", 0x230490a0a78dda00),
    ("big-primid", "enemies/big-primid", "adventure_common.arc", 0xf6347f3e7abf70a9),
    ("boom-primid", "enemies/boom-primid", "adventure_common.arc", 0x9facb83b00d641f0),
    ("borboras", "enemies/borboras", "070001.pac", 0x218b94f217a0c407),
    ("bucculus", "enemies/bucculus", "080103.pac", 0xda6de8025c5e01fe),
    ("bullet-bill", "enemies/bullet-bill", "060002.pac", 0x9e861d1729f0e894),
    ("feyesh", "enemies/feyesh", "050102.pac", 0xb0ec0ddbe98c5caf),
    ("fire-primid", "enemies/fire-primid", "adventure_common.arc", 0xc11dc3ff52950fba),
    ("giant-goomba", "enemies/giant-goomba", "060004.pac", 0x233667b991b9e3f6),
    ("glunder", "enemies/glunder", "040201.pac", 0xd5d0d599f7dad5e2),
    ("goomba", "enemies/goomba", "060001.pac", 0xd638dd98e5d4f009),
    ("hammer-bro", "enemies/hammer-bro", "060002.pac", 0x0df5bb27a1e31c1c),
    ("jyk", "enemies/jyk", "040201.pac", 0xbdbda8f02abcc879),
    ("koopa", "enemies/koopa", "060001.pac", 0x73934a0a1c7bbc53),
    ("metal-primid", "enemies/metal-primid", "adventure_common.arc", 0x8fd005bb5a024ef0),
    ("mite", "enemies/mite", "050103.pac", 0xe5a8fa68d3b41989),
    ("poppant", "enemies/poppant", "050102.pac", 0x0fe5db041804c0ca),
    ("primid", "enemies/primid", "adventure_common.arc", 0xed6f9aff9af14640),
    ("scope-primid", "enemies/scope-primid", "adventure_common.arc", 0xd1169965bbfb8773),
    ("spaak", "enemies/spaak", "030001.pac", 0xb23c2baba1e99553),
    ("sword-primid", "enemies/sword-primid", "adventure_common.arc", 0xb83a29ad82493522),
    ("ticken", "enemies/ticken", "070001.pac", 0x485808abe1397421),
    ("towtow", "enemies/towtow", "050102.pac", 0xb9a2609e929307c9),
    ("cymul", "enemies/cymul", "040101.pac", 0xd079052e6c1db35b),
    ("roturret", "enemies/roturret", "040201.pac", 0x7dc76c30e8ac5b11),
    ("buckot", "enemies/buckot", "090001.pac", 0x0ec6cacc57186d72),
    ("greap", "enemies/greap", "040101.pac", 0x26236da5486477ad),
    ("shaydas", "enemies/shaydas", "100001.pac", 0x4100e61b0aca15cd),
    ("roader", "enemies/roader", "100002.pac", 0xd5683323a8be9a40),
    ("bytan", "enemies/bytan", "090202.pac", 0x6e752c353b7509e7),
    ("floow", "enemies/floow", "090201.pac", 0xb54f9d7acbb75332),
    ("puppit", "enemies/puppit", "120001.pac", 0xf459072e0270883c),
    ("shellpod", "enemies/shellpod", "120003.pac", 0x9be1124947fc4b98),
    ("armight", "enemies/armight", "160301.pac", 0xc8b2aa6feda0b600),
    ("bombed", "enemies/bombed", "160201.pac", 0x955fb5966cc6b58f),
    ("nagagog", "enemies/nagagog", "100205.pac", 0xecd5d990ba4774c4),
    ("trowlon", "enemies/trowlon", "120002.pac", 0x251c2b8e436a9080),
    ("rob-blaster", "enemies/rob-blaster", "140005.pac", 0x8f5d20e3c9cd724b),
    ("rob-distance", "enemies/rob-distance", "140101.pac", 0x2aa5ccb166cb1e09),
    ("rob-launcher", "enemies/rob-launcher", "140105.pac", 0x1b9dfa7366df19e3),
    ("rob-sentry", "enemies/rob-sentry", "140005.pac", 0x0935ec98ddcefefa),
    ("autolance", "enemies/autolance", "100201.pac", 0x3dea716993ede337),
    ("armank", "enemies/armank", "240001.pac", 0x78f9f5138e22d68e),
    ("gamyga", "enemies/gamyga", "180001.pac", 0x2da882f42ef5f071),
    ("gamyga-base", "enemies/gamyga-base", "180001.pac", 0x57cb090faea1438d),
];
/// The shared script file (adventure_common leaf 333).
pub const COMMON_FNV: u64 = 0x0350bd0061e8208d;

/// `"raw_id": N, "location": "+0x..."` pairs from the IR's parameter list (no JSON parser needed).
pub fn param_map(ir: &str) -> Vec<(u32, u32)> {
    let mut v = Vec::new();
    let mut rest = ir;
    while let Some(i) = rest.find("\"raw_id\": ") {
        rest = &rest[i + 10..];
        let nd = rest.bytes().take_while(|c| c.is_ascii_digit()).count();
        let Ok(id) = rest[..nd].parse::<u32>() else { continue };
        let Some(l) = rest.find("\"location\": \"+0x") else { break };
        if l > 40 {
            continue;
        }
        let hx = &rest[l + 16..];
        let nh = hx.bytes().take_while(|c| c.is_ascii_hexdigit()).count();
        if let Ok(off) = u32::from_str_radix(&hx[..nh], 16) {
            v.push((id, off));
        }
    }
    v
}


/// The IR's variable initial values (`"raw_id": "vXXXXXXXX", "initial": N`): (raw var, value). Booleans are 0 / 1.
pub fn var_inits(ir: &str) -> Vec<(u32, f32)> {
    let mut v = Vec::new();
    let mut rest = ir;
    while let Some(i) = rest.find("\"raw_id\": \"v") {
        rest = &rest[i + 12..];
        let nh = rest.bytes().take_while(|c| c.is_ascii_hexdigit()).count();
        let Ok(raw) = u32::from_str_radix(&rest[..nh], 16) else { continue };
        let Some(k) = rest.find("\"initial\": ") else { break };
        // only when "initial" belongs to this variable (before the next raw_id)
        if rest.find("\"raw_id\"").is_some_and(|n| n < k) || k > 200 {
            continue;
        }
        let t = &rest[k + 11..];
        let x = if t.starts_with("true") {
            1.0
        } else if t.starts_with("false") {
            0.0
        } else {
            let n = t.bytes().take_while(|c| c.is_ascii_digit() || *c == b'-' || *c == b'.').count();
            t[..n].parse::<f32>().unwrap_or(0.0)
        };
        v.push((raw, x));
    }
    v
}

/// The local value table (`vm-values.txt` next to the IR, never committed): lines `id g|t 0xOFF`.
pub fn block_values(txt: &str) -> Vec<(u32, u8, u32)> {
    txt.lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            let id = it.next()?.parse::<u32>().ok()?;
            let b = match it.next()? {
                "g" => 0,
                "t" => 1,
                _ => return None,
            };
            let off = u32::from_str_radix(it.next()?.trim_start_matches("0x"), 16).ok()?;
            Some((id, b, off))
        })
        .collect()
}

/// The enemy class (`em<Class>ParamAccesser`) of each VM name, for the accessor-derived parameter maps.
pub fn class_of(name: &str) -> &'static str {
    match name {
        "rayquaza" => "Rayquaza",
        "ridley" => "Ridley",
        "galleom" => "Galleom",
        "duon" => "Duon",
        "porky" => "Porky",
        "metaridley" => "Metaridley",
        "tabuu" => "Taboo",
        "petey" => "Bosspackun",
        "auroros" => "Aroaros",
        "borboras" => "Boobas",
        "bucculus" => "Bucyulus",
        "bullet-bill" => "Killer",
        "feyesh" => "Pacci",
        "giant-goomba" => "Dekakuribo",
        "glunder" => "Galfire",
        "goomba" => "Kuribo",
        "hammer-bro" => "Hammerbros",
        "jyk" => "Jyakeel",
        "koopa" => "Patapata",
        "mite" => "Mite",
        "poppant" => "Popperam",
        "spaak" => "Spar",
        "ticken" => "Teckin",
        "towtow" => "Tautau",
        "cymul" => "Cymal",
        "roturret" => "Deathpod",
        "buckot" => "Botron",
        "greap" => "Gyraan",
        "shaydas" => "Siralamos",
        "roader" => "Roada",
        "bytan" => "Bitan",
        "floow" => "Flows",
        "puppit" => "Kokkon",
        "shellpod" => "Shelly",
        "armight" => "Jdus",
        "bombed" => "Bombhead",
        "nagagog" => "Ngagog",
        "trowlon" => "Faulong",
        "rob-blaster" => "Robobeam",
        "rob-distance" => "Robodistance",
        "rob-launcher" => "Robohoming",
        "rob-sentry" => "Robopunch",
        "autolance" => "Cataguard",
        "armank" => "Arman",
        "gamyga" => "Ghamgha",
        "gamyga-base" => "Ghamghabase",
        n if n.ends_with("primid") => "Prim",
        _ => "",
    }
}

/// The accessor map (`accmap.json`, local, from each class's getParamFloat / Int / Indefinite): `"id": "0xOFF"` pairs
/// of one class. Entries the IR already maps are kept from the IR by the caller.
pub fn acc_map(json: &str, class: &str) -> Vec<(u32, u32)> {
    let mut v = Vec::new();
    let key = alloc::format!("\"{class}\": {{");
    let Some(i) = json.find(&key) else { return v };
    let body = &json[i + key.len()..];
    let body = &body[..body.find('}').unwrap_or(body.len())];
    for item in body.split(',') {
        let mut parts = item.split(':');
        let (Some(k), Some(o)) = (parts.next(), parts.next()) else { continue };
        let k = k.trim().trim_matches('"');
        let o = o.trim().trim_matches('"').trim_start_matches("0x");
        if let (Ok(id), Ok(off)) = (k.parse::<u32>(), u32::from_str_radix(o, 16)) {
            v.push((id, off));
        }
    }
    v
}

/// The parameter map: the IR's entries, then the accessor map's for every id the IR leaves out.
pub fn merged_map(ir: &[(u32, u32)], acc: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut v = ir.to_vec();
    for &(id, off) in acc {
        if !v.iter().any(|x| x.0 == id) {
            v.push((id, off));
        }
    }
    v
}

include!("vm_run.rs");
