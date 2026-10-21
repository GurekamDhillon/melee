//! Port of `src/sysdolphin/baselib/generator.c`: the particle generators (emitters). Each frame
//! `hsd_8039EE24` advances every generator's `count` by its rate (`random`, drawn from
//! `HSD_Randf` when positive) and `hsd_8039DAD4` emits one particle per whole unit of `count`
//! through `hsd_80398F0C` (particle.c, the host), drawing the emission point and direction from
//! `HSD_Randf` per shape (`type & 0xF`: 0 point/disc, 1 line, 2 tornado, 3/4 filled disc,
//! 5 rect, 6 cone, 7 cylinder, 8 sphere, 9+ the host's shape hook).
//!
//! Native shape: generators live in the [`GeneratorSystem`] arena and are named by [`GenId`];
//! the C's intrusive list (`hsd_804D78FC` head, `next`) is kept as is, because the order
//! (including `hsd_8039D9C8`'s insert-after-the-cursor's-next and the `hsd_804D78F8` cursor
//! that `hsd_8039D3AC` / `hsd_8039D4DC` leave behind) decides the order of HSD_Randf draws.
//! A freed slot is reused last-freed first, as the HSD object allocator does. `hsd_804D78F8` (a
//! `u32` holding a generator pointer) is `cursor: Option<GenId>`; `hsd_804D78F4` (an `HSD_SList`
//! of generators queued by `hsd_8039F6CC`) is a [`SList`]. The JObj is the host's handle `J`;
//! `cmdList` and `userfunc` are opaque `u32` handles. The `aux` union is `aux: [f32; 12]` (the
//! float words at 0x60..0x90) plus `aux_flag` (`aux.rect.flag`, 0x90); the `AUX_*` constants
//! name the members. GcLayout: `melee_types::HSD_Generator` (0x94).
//!
//! Globals of particle.c that generator.c reads (`psCamera`, the hooks `hsd_804D78E8` /
//! `hsd_804D78EC`, `ptclref_804D0E5C`, `psCmdListArray`, `psTexGroupArray`) and the calls out
//! (hsd_80398F0C, hsd_8039D0A0, JObj ref / unref / SetupMatrix, the TARGET_PC ScriptGame
//! stage-slot hooks) go through [`GeneratorHost`]. The generator counters `hsd_804D78E0` (live)
//! and `hsd_804D78DA` (peak) are particle.c globals only generator.c writes; they live here.
//!
//! Float rule (`fma_sites.py`, all ported with `fmadds`): hsd_8039D71C 12 sites (the two 3-term
//! rotations), hsd_8039DAD4 22, hsd_8039EE24 1 (`count += random * Randf()`), hsd_8039F05C 3
//! (the sphere speed and xz length). `M_PI` / `M_PI_2` arithmetic is f64 as in the C. The
//! `(f32)(s32)count` conversion is gwtool's saturating fptosi.
use crate::cabi::f2i32;
use crate::gekko::{cosf, sinf, sqrtf};
use crate::hsd_mtx::{fabsf_bitwise, get_scale};
use crate::lbtrigf::atan2f;
use crate::list::SList;
use crate::mtx::{self, Mtx, Vec3};
use crate::psappsrt::{AppSrtHost, AppSrtPool, SrtId};
use melee_core::fp::fmadds;
use std::f64::consts::{FRAC_PI_2, PI};

/// Index of a generator in the [`GeneratorSystem`] arena.
pub type GenId = u32;

const FLT_MIN: f32 = 1.1754944e-38;
const FLT_EPSILON: f32 = 1.1920929e-7;
const HALF_PI_F: f32 = 1.5707964;

/// `aux.disc.minAngle` / `aux.cone.minAngle` / `aux.line.x2` / `aux.tornado.vel` /
/// `aux.rect.x` / `aux.sphere.speed`.
pub const AUX_MIN_ANGLE: usize = 0;
/// `aux.disc.maxAngle` / `aux.cone.maxAngle` / `aux.line.y2` / `aux.rect.y` / `aux.sphere.latMid`.
pub const AUX_MAX_ANGLE: usize = 1;
/// `aux.cone.height` / `aux.line.z2` / `aux.rect.z` / `aux.sphere.latRange`.
pub const AUX_HEIGHT: usize = 2;
/// `aux.sphere.lonMid` (= `aux.rect.xx`).
pub const AUX_LON_MID: usize = 3;
/// `aux.sphere.lonRange` (= `aux.rect.xy`).
pub const AUX_LON_RANGE: usize = 4;
/// `aux.rect.xx` .. `aux.rect.zz` start (xx 3, xy 4, xz 5, yx 6, yy 7, yz 8, zx 9, zy 10, zz 11).
pub const AUX_RECT_XX: usize = 3;

/// `HSD_Generator`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HsdGenerator<J> {
    /// 0x00
    pub next: Option<GenId>,
    /// 0x04
    pub kind: u32,
    /// 0x08
    pub random: f32,
    /// 0x0C
    pub count: f32,
    /// 0x10
    pub jobj: Option<J>,
    /// 0x14
    pub gen_life: u16,
    /// 0x16
    pub type_: u16,
    /// 0x18
    pub bank: u8,
    /// 0x19
    pub link_no: u8,
    /// 0x1A
    pub tex_group: u8,
    /// 0x1B
    pub dummy: u8,
    /// 0x1C
    pub idnum: u16,
    /// 0x1E
    pub life: u16,
    /// 0x20 `cmdList` (opaque handle).
    pub cmd_list: u32,
    /// 0x24
    pub pos: Vec3,
    /// 0x30
    pub vel: Vec3,
    /// 0x3C
    pub grav: f32,
    /// 0x40
    pub fric: f32,
    /// 0x44
    pub size: f32,
    /// 0x48
    pub radius: f32,
    /// 0x4C
    pub angle: f32,
    /// 0x50
    pub num_child: u32,
    /// 0x54
    pub appsrt: Option<SrtId>,
    /// 0x58 `userfunc` (opaque handle; None = NULL).
    pub userfunc: Option<u32>,
    /// 0x5C `callback` (opaque handle; never called by generator.c).
    pub callback: Option<u32>,
    /// 0x60..0x90: the float words of the `aux` union.
    pub aux: [f32; 12],
    /// 0x90 `aux.rect.flag`.
    pub aux_flag: u16,
}

impl<J> melee_core::gc::GcLayout for HsdGenerator<J> {
    const GC_NAME: &'static str = <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_NAME;
    const GC_SIZE: u32 = <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_SIZE;
    const GC_ALIGN: u32 = <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_ALIGN;
    const GC_FIELDS: &'static [melee_core::gc::GcField] =
        <melee_types::HSD_Generator as melee_core::gc::GcLayout>::GC_FIELDS;
}

/// The fields of `HSD_PSCmdList` (0x40 + the command bytes) that `hsd_8039F05C` reads.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PsCmdList {
    pub type_: u16,
    pub tex_group: u16,
    pub gen_life: u16,
    pub life: u16,
    pub kind: u32,
    pub grav: f32,
    pub fric: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub radius: f32,
    pub angle: f32,
    pub random: f32,
    pub size: f32,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
    /// `&cmdList[0]` (opaque handle stored in `HSD_Generator.cmdList`).
    pub cmd_list: u32,
}

/// The arguments of one `hsd_80398F0C` call (particle spawn).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PsSpawn {
    pub link_no: i32,
    pub bank: i32,
    pub kind: i32,
    pub tex_group: u16,
    pub cmd_list: u32,
    pub life: i32,
    pub arg6: i32,
    pub gen: GenId,
    pub pos: Vec3,
    pub vel: Vec3,
    pub size: f32,
    pub grav: f32,
    pub fric: f32,
}

/// What generator.c reads or calls outside itself.
pub trait GeneratorHost<J: Copy + PartialEq>: AppSrtHost {
    /// `HSD_Randf()`.
    fn randf(&mut self) -> f32;
    /// `HSD_JObjSetupMatrix(jobj)` (the inline: SetupMatrixSub when the matrix is dirty).
    fn jobj_setup_matrix(&mut self, j: J);
    /// `jobj->mtx`.
    fn jobj_mtx(&self, j: J) -> Mtx;
    /// `ref_INC(jobj)`.
    fn jobj_ref_inc(&mut self, j: J);
    /// `HSD_JObjUnref(jobj)`.
    fn jobj_unref(&mut self, j: J);
    /// `jobj->flags`.
    fn jobj_flags(&self, j: J) -> u32;
    /// `jobj->child`.
    fn jobj_child(&self, j: J) -> Option<J>;
    /// `jobj->next`.
    fn jobj_next(&self, j: J) -> Option<J>;
    /// `psCamera` as `(psCamera->eyepos->pos, HSD_CObjGetUpVector(psCamera))`; None = NULL
    /// (hsd_8039DAD4 asserts).
    fn ps_camera(&mut self) -> Option<(Vec3, Vec3)>;
    /// `hsd_80398F0C(...)` (particle.c: spawn a particle).
    fn spawn_particle(&mut self, p: &PsSpawn);
    /// `hsd_8039D0A0(gen)` (particle.c).
    fn particle_8039d0a0(&mut self, gen: GenId, g: &HsdGenerator<J>);
    /// `hsd_804D78E8(gen, rot_mtx)` for a shape > 8 (no-op when NULL).
    fn shape_emit(&mut self, gen: GenId, g: &mut HsdGenerator<J>, rot: &Mtx);
    /// `hsd_804D78EC(gen)` for a shape > 8 in hsd_8039F05C (no-op when NULL).
    fn shape_init(&mut self, gen: GenId, g: &mut HsdGenerator<J>);
    /// `hsd_804D7900(gen)` (no-op when NULL).
    fn on_create(&mut self, gen: GenId, g: &mut HsdGenerator<J>);
    /// `psCmdListArray[bank]`.
    fn cmd_list_count(&self, bank: i32) -> i32;
    /// `ptclref_804D0E5C[bank][idx]` (None = NULL).
    fn cmd_list(&self, bank: i32, idx: i32) -> Option<PsCmdList>;
    /// `psTexGroupArray[bank][tex_group]->palflag` (None = NULL group).
    fn tex_group_palflag(&self, bank: i32, tex_group: u8) -> Option<u16>;
    /// hsd_8039D354's reset of the particle.c globals `psCamera`, `hsd_804D78E8`, `hsd_804D78EC`.
    fn reset_ps_globals(&mut self);
    /// TARGET_PC `ScriptGame_StageSlotNativeContext()`.
    fn stage_slot_native_context(&mut self) -> bool;
    /// TARGET_PC `ScriptGame_StageSlotParticleBegin(linkNo)`.
    fn stage_slot_particle_begin(&mut self, link_no: i32) -> bool;
    /// TARGET_PC `ScriptGame_StageSlotProcEnd()`.
    fn stage_slot_proc_end(&mut self);
    /// TARGET_PC `HSD_StageSlotParticleLinkClear()`.
    fn stage_slot_particle_link_clear(&mut self);
}

/// generator.c's state (plus psappsrt.c's pool, which the generators own a share of).
#[derive(Clone, Debug)]
pub struct GeneratorSystem<J> {
    pub slots: Vec<Option<HsdGenerator<J>>>,
    free: Vec<GenId>,
    /// `hsd_804D78FC`: the list head.
    pub head: Option<GenId>,
    /// `hsd_804D78F8`: the cursor (the generator before the one being processed).
    pub cursor: Option<GenId>,
    /// `hsd_804D78F4`: generators queued by hsd_8039F6CC for one hsd_8039D71C.
    pub queue: SList<GenId>,
    /// `lbl_804D6368`: the last idnum handed out.
    pub id_counter: u16,
    /// `hsd_804D78E0`: live generators.
    pub live: u16,
    /// `hsd_804D78DA`: peak of `live`.
    pub peak: u16,
    pub srts: AppSrtPool,
}

impl<J> Default for GeneratorSystem<J> {
    fn default() -> Self {
        GeneratorSystem {
            slots: Vec::new(),
            free: Vec::new(),
            head: None,
            cursor: None,
            queue: SList::new(),
            id_counter: 0x100,
            live: 0,
            peak: 0,
            srts: AppSrtPool::default(),
        }
    }
}

/// `hsd_8039D1E4`: `gen->userfunc = userfunc`.
pub fn set_userfunc<J>(gen: &mut HsdGenerator<J>, userfunc: Option<u32>) {
    gen.userfunc = userfunc;
}

fn col(m: &Mtx, c: usize) -> Vec3 {
    Vec3 { x: m[0][c], y: m[1][c], z: m[2][c] }
}

impl<J: Copy + PartialEq> GeneratorSystem<J> {
    pub fn gen(&self, id: GenId) -> &HsdGenerator<J> {
        self.slots[id as usize].as_ref().expect("freed generator")
    }
    pub fn gen_mut(&mut self, id: GenId) -> &mut HsdGenerator<J> {
        self.slots[id as usize].as_mut().expect("freed generator")
    }
    fn next_of(&self, id: GenId) -> Option<GenId> {
        self.gen(id).next
    }

    /// `hsd_8039D1EC`: the next idnum (>= 0x100).
    pub fn next_idnum(&mut self) -> u16 {
        self.id_counter = self.id_counter.wrapping_add(1);
        if self.id_counter < 256 {
            self.id_counter = 256;
        }
        self.id_counter
    }

    /// The common head of hsd_8039D214 and hsd_8039D71C: SetupMatrix, then copy the JObj's
    /// translation into the generator / its own app SRT. Returns the matrix (None: no JObj).
    fn follow_jobj<H: GeneratorHost<J>>(&mut self, id: GenId, host: &mut H) -> Option<Mtx> {
        let g = self.gen(id);
        if g.type_ & 0x100 == 0 {
            return None;
        }
        let j = g.jobj?;
        host.jobj_setup_matrix(j);
        let m = host.jobj_mtx(j);
        let (type_, srt) = (g.type_, g.appsrt);
        if type_ & 0x200 != 0 {
            self.gen_mut(id).pos = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
        }
        let owned = srt.filter(|&s| self.srts.get(s).gp == Some(id));
        if type_ & 0x800 != 0 {
            if let Some(s) = owned {
                self.srts.get_mut(s).translate = Vec3 { x: m[0][3], y: m[1][3], z: m[2][3] };
            }
        }
        if type_ & 0x1000 != 0 {
            if let Some(s) = owned {
                self.srts.get_mut(s).scale = get_scale(&m);
            }
        }
        Some(m)
    }

    /// `hsd_8039D214`: follow the attached JObj (position / SRT translate / SRT scale).
    pub fn follow(&mut self, id: GenId, host: &mut impl GeneratorHost<J>) {
        self.follow_jobj(id, host);
    }

    /// `hsd_8039D354`: reset (the C re-inits the allocator; live generators are leaked there,
    /// dropped here).
    pub fn init(&mut self, host: &mut impl GeneratorHost<J>) {
        self.slots.clear();
        self.free.clear();
        self.head = None;
        self.live = 0;
        self.peak = 0;
        self.queue = SList::new();
        self.cursor = None;
        host.reset_ps_globals();
    }

    /// `hsd_8039D3AC(gen, prev)`: unlink and free `gen` unless it still has children or its
    /// app SRT is shared (then `random = 0`, `genLife = 1` and it stays). Returns `gen` when it
    /// stayed, else `prev`.
    pub fn remove(&mut self, id: GenId, prev: Option<GenId>, host: &mut impl GeneratorHost<J>) -> Option<GenId> {
        if self.gen(id).type_ & 0x80 != 0 {
            host.particle_8039d0a0(id, self.gen(id));
        }
        let g = self.gen(id);
        let mut keep = g.num_child != 0;
        if !keep && g.type_ & 0x100 != 0 && g.jobj.is_some() && g.type_ & 0x1800 != 0 {
            if let Some(s) = g.appsrt {
                let srt = self.srts.get(s);
                if srt.gp == Some(id) && srt.used_count != 1 {
                    keep = true;
                }
            }
        }
        if keep {
            let g = self.gen_mut(id);
            g.random = 0.0;
            g.gen_life = 1;
            return Some(id);
        }
        let next = self.gen(id).next;
        match prev {
            None => self.head = next,
            Some(p) => self.gen_mut(p).next = next,
        }
        if self.gen(id).appsrt.is_some() {
            let mut slot = self.gen(id).appsrt;
            self.srts.remove_generator(id, &mut slot, host);
            self.gen_mut(id).appsrt = slot;
        }
        if let Some(j) = self.gen(id).jobj {
            host.jobj_unref(j);
            self.gen_mut(id).jobj = None;
        }
        self.slots[id as usize] = None;
        self.free.push(id);
        self.live = self.live.wrapping_sub(1);
        if self.queue.iter().any(|&q| q == id) {
            // The C leaves the queued pointer dangling (hsd_8039EE24 then runs hsd_8039D71C on
            // freed memory); natively the entry is dropped so a reused slot is never touched.
            self.requeue_without(id);
        }
        prev
    }

    /// `hsd_8039D4DC`: remove `gen` (found by walking the list) and leave the cursor on the last
    /// generator.
    pub fn kill(&mut self, id: GenId, host: &mut impl GeneratorHost<J>) {
        let mut cur = self.head;
        self.cursor = None;
        while let Some(c) = cur {
            if c == id {
                self.cursor = self.remove(id, self.cursor, host);
                if let Some(mut k) = self.cursor {
                    while let Some(n) = self.next_of(k) {
                        k = n;
                    }
                    self.cursor = Some(k);
                } else if let Some(mut k) = self.head {
                    while let Some(n) = self.next_of(k) {
                        k = n;
                    }
                    self.cursor = Some(k);
                }
                return;
            }
            self.cursor = Some(c);
            cur = self.next_of(c);
        }
    }

    fn kill_attached(&mut self, j: J, host: &mut impl GeneratorHost<J>) {
        let mut cur = self.head;
        while let Some(c) = cur {
            let next = self.next_of(c);
            if self.gen(c).jobj == Some(j) {
                self.kill(c, host);
            }
            cur = next;
        }
    }

    /// `hsd_8039D580`: kill every generator attached to `jobj` (None: no-op).
    pub fn kill_by_jobj(&mut self, jobj: Option<J>, host: &mut impl GeneratorHost<J>) {
        if let Some(j) = jobj {
            self.kill_attached(j, host);
        }
    }

    /// `hsd_8039D5DC`: hsd_8039D580 over `jobj` and, unless it is an instance (flag 0x1000),
    /// its whole subtree.
    pub fn kill_by_jobj_tree(&mut self, jobj: Option<J>, host: &mut impl GeneratorHost<J>) {
        let Some(j) = jobj else { return };
        self.kill_attached(j, host);
        if host.jobj_flags(j) & 0x1000 != 0 {
            return;
        }
        let mut c = host.jobj_child(j);
        while let Some(k) = c {
            self.kill_by_jobj_tree(Some(k), host);
            c = host.jobj_next(k);
        }
    }

    /// `hsd_8039D688(jobj, unused, unused)`: remove every generator attached to `jobj` (None
    /// matches the generators with no JObj), marking those with an app SRT and a followed JObj
    /// 0x80 first.
    pub fn remove_by_jobj(&mut self, jobj: Option<J>, host: &mut impl GeneratorHost<J>) {
        let mut cur = self.head;
        self.cursor = None;
        while let Some(c) = cur {
            let next = self.next_of(c);
            if self.gen(c).jobj == jobj {
                let g = self.gen_mut(c);
                if g.appsrt.is_some() && g.type_ & 0x100 != 0 {
                    g.type_ |= 0x80;
                }
                self.cursor = self.remove(c, self.cursor, host);
            } else {
                self.cursor = Some(c);
            }
            cur = next;
        }
    }

    /// `hsd_8039D71C`: follow the JObj, then rotate `vel` (and a line's end) into its frame.
    pub fn attach_transform(&mut self, id: GenId, host: &mut impl GeneratorHost<J>) {
        let Some(m) = self.follow_jobj(id, host) else { return };
        let c0 = mtx::vec_normalize(&col(&m, 0));
        let c1 = mtx::vec_normalize(&col(&m, 1));
        let c2 = mtx::vec_normalize(&col(&m, 2));
        let g = self.gen_mut(id);
        let v = g.vel;
        g.vel.x = fmadds(c2.x, v.z, fmadds(c0.x, v.x, c1.x * v.y));
        g.vel.y = fmadds(c2.y, v.z, fmadds(c0.y, v.x, c1.y * v.y));
        g.vel.z = fmadds(c2.z, v.z, fmadds(c0.z, v.x, c1.z * v.y));
        if g.type_ & 0xF == 1 {
            let (ax, ay, az) = (g.aux[0], g.aux[1], g.aux[2]);
            for r in 0..3 {
                g.aux[r] = fmadds(m[r][2], az, fmadds(m[r][0], ax, m[r][1] * ay));
            }
        }
    }

    /// `hsd_8039D9C8`: allocate a zeroed generator, link it after the cursor's next (or after
    /// the head) and give it the next idnum.
    pub fn alloc(&mut self) -> Option<GenId> {
        let g = HsdGenerator::<J> {
            next: None,
            kind: 0,
            random: 0.0,
            count: 0.0,
            jobj: None,
            gen_life: 0,
            type_: 0,
            bank: 0,
            link_no: 0,
            tex_group: 0,
            dummy: 0,
            idnum: 0,
            life: 0,
            cmd_list: 0,
            pos: Vec3::default(),
            vel: Vec3::default(),
            grav: 0.0,
            fric: 0.0,
            size: 0.0,
            radius: 0.0,
            angle: 0.0,
            num_child: 0,
            appsrt: None,
            userfunc: None,
            callback: None,
            aux: [0.0; 12],
            aux_flag: 0,
        };
        let id = if let Some(id) = self.free.pop() {
            self.slots[id as usize] = Some(g);
            id
        } else {
            self.slots.push(Some(g));
            (self.slots.len() - 1) as u32
        };
        self.live = self.live.wrapping_add(1);
        if self.live > self.peak {
            self.peak = self.live;
        }
        let after = match self.cursor {
            Some(c) if self.next_of(c).is_some() => self.next_of(c),
            _ => self.head,
        };
        match after {
            None => {
                self.gen_mut(id).next = None;
                self.head = Some(id);
            }
            Some(a) => {
                let n = self.next_of(a);
                self.gen_mut(id).next = n;
                self.gen_mut(a).next = Some(id);
            }
        }
        let idnum = self.next_idnum();
        let g = self.gen_mut(id);
        g.idnum = idnum;
        g.appsrt = None;
        g.userfunc = None;
        Some(id)
    }

    /// `hsd_8039EE24(mask)`: run the queued attach transforms, then advance every generator
    /// whose link bit (`1 << (linkNo + 16)`) is not in `mask` and that is not paused
    /// (kind 0x800): follow its JObj, add its rate, emit, and age it.
    pub fn update(&mut self, mask: u32, host: &mut impl GeneratorHost<J>) {
        while let Some(&gp) = self.queue.iter().next() {
            let owned = host.stage_slot_particle_begin(self.gen(gp).link_no as i32);
            self.attach_transform(gp, host);
            self.queue.remove();
            if let Some(j) = self.gen(gp).jobj {
                host.jobj_unref(j);
                self.gen_mut(gp).jobj = None;
            }
            if owned {
                host.stage_slot_proc_end();
            }
        }
        let mut cur = self.head;
        self.cursor = None;
        while let Some(c) = cur {
            let (link_no, kind, next) = (self.gen(c).link_no, self.gen(c).kind, self.gen(c).next);
            if mask & crate::cabi::shl(1, link_no as u32 + 16) != 0 || kind & 0x800 != 0 {
                self.cursor = Some(c);
                cur = next;
                continue;
            }
            let owned = host.stage_slot_particle_begin(link_no as i32);
            self.follow_jobj(c, host);
            {
                let random = self.gen(c).random;
                if random < 0.0 {
                    self.gen_mut(c).count -= random;
                } else {
                    let r = host.randf();
                    let g = self.gen_mut(c);
                    g.count = fmadds(random, r, g.count);
                }
            }
            if self.gen(c).count >= 1.0 {
                let mut g = self.slots[c as usize].take().unwrap();
                g.count = emit(&mut g, c, host);
                self.slots[c as usize] = Some(g);
            }
            let g = self.gen_mut(c);
            if g.gen_life != 0 {
                g.gen_life -= 1;
                if g.gen_life == 0 {
                    self.cursor = self.remove(c, self.cursor, host);
                    cur = match self.cursor {
                        Some(k) => self.next_of(k),
                        None => self.head,
                    };
                    if owned {
                        host.stage_slot_proc_end();
                    }
                    continue;
                }
            }
            self.cursor = Some(c);
            cur = self.next_of(c);
            if owned {
                host.stage_slot_proc_end();
            }
        }
    }

    /// `hsd_8039EFAC(linkNo, bank, gfx_id, jobj)`: hsd_8039F05C, then attach `jobj` (ref'd) and
    /// follow it (type 0x500, plus 0x200 unless the generator owns an app SRT).
    pub fn spawn_on_jobj(&mut self, link_no: i32, bank: i32, gfx_id: i32, jobj: Option<J>, host: &mut impl GeneratorHost<J>) -> Option<GenId> {
        let id = self.spawn(link_no, bank, gfx_id, host)?;
        self.attach_new(id, jobj, host);
        Some(id)
    }

    fn attach_new(&mut self, id: GenId, jobj: Option<J>, host: &mut impl GeneratorHost<J>) {
        self.gen_mut(id).jobj = jobj;
        if let Some(j) = jobj {
            host.jobj_ref_inc(j);
        }
        let g = self.gen_mut(id);
        g.type_ |= if g.kind & 0x20000 != 0 { 0x500 } else { 0x700 };
    }

    /// `hsd_8039F6CC`: hsd_8039EFAC, and queue the generator for one hsd_8039D71C (which then
    /// drops the JObj).
    pub fn spawn_on_jobj_once(&mut self, link_no: i32, bank: i32, gfx_id: i32, jobj: Option<J>, host: &mut impl GeneratorHost<J>) -> Option<GenId> {
        let id = self.spawn(link_no, bank, gfx_id, host)?;
        self.attach_new(id, jobj, host);
        self.queue.alloc_and_append(id);
        Some(id)
    }

    /// `hsd_8039F05C(linkNo, bank, idx)`: create a generator from `ptclref_804D0E5C[bank][idx]`.
    pub fn spawn(&mut self, link_no: i32, bank: i32, idx: i32, host: &mut impl GeneratorHost<J>) -> Option<GenId> {
        let mut link_no = link_no;
        if host.stage_slot_native_context() {
            link_no = 3;
        }
        if bank >= 0x41 || link_no >= 8 || idx >= host.cmd_list_count(bank) {
            return None;
        }
        let c = host.cmd_list(bank, idx)?;
        let id = self.alloc()?;
        let g = self.slots[id as usize].as_mut().unwrap();
        g.type_ = c.type_;
        g.bank = bank as u8;
        g.link_no = link_no as u8;
        g.kind = c.kind;
        g.tex_group = c.tex_group as u8;
        g.life = c.life;
        g.gen_life = c.gen_life;
        g.pos = Vec3::default();
        g.vel = Vec3 { x: c.vx, y: c.vy, z: c.vz };
        g.grav = c.grav;
        g.fric = c.fric;
        g.size = c.size;
        g.cmd_list = c.cmd_list;
        g.radius = c.radius;
        g.angle = c.angle;
        g.random = c.random;
        if g.kind & 0x100 != 0 {
            g.count = if g.random < 0.0 {
                if 1.0 + g.random > FLT_EPSILON { 1.0 } else { 0.0 }
            } else {
                0.9999999
            };
        } else if g.random < 0.0 {
            g.count = 0.0;
        } else {
            g.count = host.randf();
        }
        if let Some(pal) = host.tex_group_palflag(bank, g.tex_group) {
            if pal != 0 {
                g.kind |= 0x10;
            }
        }
        g.jobj = None;
        g.num_child = 0;
        match g.type_ & 0xF {
            0 | 3 | 4 | 6 | 7 => {
                if c.param1 == 0.0 && c.param2 == 0.0 {
                    g.aux[AUX_MIN_ANGLE] = 0.0;
                    g.aux[AUX_MAX_ANGLE] = 6.2831855;
                } else {
                    g.aux[AUX_MIN_ANGLE] = c.param1;
                    g.aux[AUX_MAX_ANGLE] = c.param2;
                }
                if g.type_ & 0xF >= 6 {
                    g.aux[AUX_HEIGHT] = c.param3;
                }
            }
            1 => {
                g.aux[0] = c.param1;
                g.aux[1] = c.param2;
                g.aux[2] = c.param3;
            }
            2 => {}
            5 => {
                g.aux[0] = c.param1;
                g.aux[AUX_RECT_XX] = c.param1;
                g.aux[1] = c.param2;
                g.aux[AUX_RECT_XX + 4] = c.param2;
                g.aux[2] = c.param3;
                g.aux[AUX_RECT_XX + 8] = c.param3;
                for k in [7usize, 6, 5, 3, 2, 1] {
                    g.aux[AUX_RECT_XX + k] = 0.0;
                }
                g.aux_flag = 0;
                if c.param1 < 0.0 {
                    g.aux_flag |= 1;
                }
                if c.param2 < 0.0 {
                    g.aux_flag |= 2;
                }
                if c.param3 < 0.0 {
                    g.aux_flag |= 4;
                }
            }
            8 => {
                let v = g.vel;
                g.aux[0] = sqrtf(fmadds(v.z, v.z, fmadds(v.x, v.x, v.y * v.y)));
                let mag = sqrtf(fmadds(v.x, v.x, v.z * v.z));
                g.aux[1] = if mag < FLT_MIN {
                    if v.y >= 0.0 { HALF_PI_F } else { -HALF_PI_F }
                } else {
                    atan2f(v.y, mag)
                };
                g.aux[AUX_LON_MID] = if fabsf_bitwise(v.x) < FLT_MIN {
                    if v.z >= 0.0 { HALF_PI_F } else { -HALF_PI_F }
                } else {
                    atan2f(v.z, v.x)
                };
                g.aux[2] = c.param1;
                if g.aux[2] < 0.0 {
                    g.aux[2] = -g.aux[2];
                    g.aux[0] = -g.aux[0];
                }
                g.aux[AUX_LON_RANGE] = c.param2;
            }
            _ => {
                let mut g = self.slots[id as usize].take().unwrap();
                host.shape_init(id, &mut g);
                self.slots[id as usize] = Some(g);
            }
        }
        if self.gen(id).kind & 0x20000 != 0 {
            let idnum = self.gen(id).idnum;
            self.gen_mut(id).type_ |= 0x800;
            let mut slot = None;
            self.srts.add_begin(&mut slot, 0, idnum);
            self.gen_mut(id).appsrt = slot;
            if let Some(s) = slot {
                let srt = self.srts.get_mut(s);
                srt.xa2 = 1;
                srt.gp = Some(id);
            }
        }
        let mut g = self.slots[id as usize].take().unwrap();
        host.on_create(id, &mut g);
        self.slots[id as usize] = Some(g);
        Some(id)
    }

    fn requeue_without(&mut self, id: GenId) {
        let kept: Vec<GenId> = self.queue.iter().copied().filter(|&g| g != id).collect();
        let mut q = SList::new();
        for g in kept.into_iter().rev() {
            q.alloc_and_prepend(g);
        }
        self.queue = q;
    }

    /// TARGET_PC `HSD_StageSlotParticlesClear`: drop the queued and live generators of link 3.
    pub fn stage_slot_particles_clear(&mut self, host: &mut impl GeneratorHost<J>) {
        let kept: Vec<GenId> = self.queue.iter().copied().filter(|&g| self.gen(g).link_no != 3).collect();
        let mut q = SList::new();
        for g in kept.into_iter().rev() {
            q.alloc_and_prepend(g);
        }
        self.queue = q;
        host.stage_slot_particle_link_clear();
        let mut cur = self.head;
        let mut previous = None;
        while let Some(c) = cur {
            let next = self.next_of(c);
            if self.gen(c).link_no == 3 {
                if self.remove(c, previous, host) == Some(c) {
                    previous = Some(c);
                }
            } else {
                previous = Some(c);
            }
            cur = next;
        }
        // The C leaves hsd_804D78F8 alone; if it named a freed generator, the next
        // hsd_8039D9C8 reads freed memory. Natively the cursor is dropped.
        if let Some(k) = self.cursor {
            if self.slots[k as usize].is_none() {
                self.cursor = None;
            }
        }
    }
}

fn spawn_args<J>(g: &HsdGenerator<J>, id: GenId, kind: u32, pos: Vec3, vel: Vec3, size: f32, grav: f32, fric: f32) -> PsSpawn {
    PsSpawn {
        link_no: g.link_no as i32,
        bank: g.bank as i32,
        kind: kind as i32,
        tex_group: g.tex_group as u16,
        cmd_list: g.cmd_list,
        life: g.life as i32,
        arg6: 0,
        gen: id,
        pos,
        vel,
        size,
        grav,
        fric,
    }
}

fn snap(v: f32, half: f32) -> f32 {
    if v > half { 1.0 } else { 0.0 }
}

fn atan2_or_pole(y: f32, x: f32, sign_of: f32) -> f32 {
    if fabsf_bitwise(x) < FLT_MIN {
        if sign_of >= 0.0 { HALF_PI_F } else { -HALF_PI_F }
    } else {
        atan2f(y, x)
    }
}

#[allow(unused_assignments)]
/// `hsd_8039DAD4(gen)`: emit one particle per whole unit of `count`; returns the remainder.
pub fn emit<J: Copy + PartialEq, H: GeneratorHost<J>>(gen: &mut HsdGenerator<J>, id: GenId, host: &mut H) -> f32 {
    let mut angle3 = 0.0f32;
    let mut angle1 = 0.0f32;
    let (mut sin_az, mut cos_az, mut sin_el) = (0.0f32, 0.0f32, 0.0f32);
    let (mut angle_step, mut cur_angle, mut cone_angle, mut elevation, mut radius) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    if gen.count < 1.0 {
        return gen.count;
    }
    let vel_copy = gen.vel;
    let vel_mag_sq = sqrtf(fmadds(vel_copy.z, vel_copy.z, fmadds(vel_copy.x, vel_copy.x, vel_copy.y * vel_copy.y)));
    let mut rot = mtx::identity();
    let shape = gen.type_ & 0xF;

    if gen.type_ & 0x100 != 0 && gen.type_ & 0x400 != 0 && gen.kind & 0x30000 == 0 {
        if let Some(j) = gen.jobj {
            let jm = host.jobj_mtx(j);
            for c in 0..3 {
                let v = mtx::vec_normalize(&col(&jm, c));
                rot[0][c] = v.x;
                rot[1][c] = v.y;
                rot[2][c] = v.z;
            }
            rot[2][3] = 0.0;
            rot[1][3] = 0.0;
            rot[0][3] = 0.0;
        }
    }

    if gen.kind & 0x10000 != 0 {
        let (eye, up) = host.ps_camera().expect("hsd_8039DAD4: psCamera (generator.c:677)");
        let look = mtx::vec_normalize(&Vec3 { x: eye.x - gen.pos.x, y: eye.y - gen.pos.y, z: eye.z - gen.pos.z });
        let up = mtx::vec_normalize(&up);
        let cross1 = mtx::vec_cross(&up, &look);
        let up = mtx::vec_cross(&look, &cross1);
        for (c, v) in [cross1, up, look].iter().enumerate() {
            rot[0][c] = v.x;
            rot[1][c] = v.y;
            rot[2][c] = v.z;
        }
    }

    if shape != 1 && vel_mag_sq > FLT_EPSILON {
        let vn = mtx::vec_normalize(&gen.vel);
        sin_az = atan2_or_pole(vn.y, vn.z, vn.y);
        cos_az = sinf(sin_az);
        sin_az = cosf(sin_az);
        let projected_z = fmadds(vn.y, cos_az, vn.z * sin_az);
        elevation = atan2_or_pole(vn.x, projected_z, vn.x);
        sin_el = sinf(elevation);
        let cos_el = cosf(elevation);
        let neg_cos_az = -cos_az;
        let trig: Mtx = [
            [cos_el, 0.0, sin_el, 0.0],
            [neg_cos_az * sin_el, sin_az, cos_az * cos_el, 0.0],
            [-sin_az * sin_el, neg_cos_az, sin_az * cos_el, 0.0],
        ];
        rot = mtx::concat(&rot, &trig);
    }
    elevation = vel_mag_sq;

    if shape == 2 {
        angle1 = atan2_or_pole(rot[1][2], rot[2][2], rot[1][2]);
        let first = rot[2][2] * cosf(angle1);
        let comb = fmadds(rot[1][2], sinf(angle1), first);
        angle3 = atan2_or_pole(rot[0][2], comb, rot[0][2]);
    }

    if gen.angle < 0.0 {
        let n = f2i32(gen.count as f64);
        match shape {
            0 | 3 | 4 | 6 | 7 => {
                let min_a = gen.aux[AUX_MIN_ANGLE];
                let rnd = host.randf();
                angle_step = (gen.aux[AUX_MAX_ANGLE] - min_a) / n as f32;
                cur_angle = fmadds(angle_step, rnd, min_a);
            }
            _ => {
                let rnd = host.randf();
                cur_angle = ((PI * rnd as f64) * 2.0) as f32;
                angle_step = ((2.0 * PI) / n as f64) as f32;
            }
        }
    }

    let eps = 0.001f32 as f64;
    while gen.count >= 1.0 {
        match shape {
            0 | 3 | 4 | 6 | 7 => {
                if gen.radius < 0.0 {
                    sin_az = -gen.radius;
                    radius = 1.0;
                } else {
                    radius = host.randf();
                    if shape == 3 || shape == 4 {
                        radius = sqrtf(radius);
                    }
                    sin_az = radius * gen.radius;
                }
                match shape {
                    6 => {
                        let h = gen.aux[AUX_HEIGHT];
                        if gen.angle < 0.0 {
                            cur_angle += angle_step;
                            cone_angle = if fabsf_bitwise(sin_az) < FLT_MIN {
                                if h >= 0.0 { -gen.angle } else { (PI - gen.angle as f64) as f32 }
                            } else {
                                ((FRAC_PI_2 - atan2f(h, sin_az) as f64) - gen.angle as f64) as f32
                            };
                        } else {
                            cur_angle = gen.aux[AUX_MIN_ANGLE];
                            let rnd = host.randf();
                            cone_angle = gen.aux[AUX_MAX_ANGLE] - cur_angle;
                            cur_angle = fmadds(cone_angle, rnd, cur_angle);
                            cone_angle = if fabsf_bitwise(sin_az) < FLT_MIN {
                                if h >= 0.0 { gen.angle } else { (PI + gen.angle as f64) as f32 }
                            } else {
                                (gen.angle as f64 + (FRAC_PI_2 - atan2f(h, sin_az) as f64)) as f32
                            };
                        }
                    }
                    7 => {
                        if gen.angle < 0.0 {
                            cone_angle = (FRAC_PI_2 - gen.angle as f64) as f32;
                            cur_angle += angle_step;
                        } else {
                            cur_angle = gen.aux[AUX_MIN_ANGLE];
                            let rnd = host.randf();
                            cone_angle = gen.aux[AUX_MAX_ANGLE] - cur_angle;
                            cur_angle = fmadds(cone_angle, rnd, cur_angle);
                            cone_angle = (FRAC_PI_2 + gen.angle as f64) as f32;
                        }
                    }
                    _ => {
                        if gen.angle < 0.0 {
                            cone_angle = -gen.angle;
                            cur_angle += angle_step;
                            cone_angle = radius * cone_angle;
                        } else {
                            cur_angle = gen.aux[AUX_MIN_ANGLE];
                            let rnd = host.randf();
                            cone_angle = radius * gen.angle;
                            sin_el = gen.aux[AUX_MAX_ANGLE] - cur_angle;
                            cur_angle = fmadds(sin_el, rnd, cur_angle);
                        }
                    }
                }
                cos_az = cur_angle;
                let mut emit_pos = Vec3 { x: sin_az * cosf(cos_az), y: sin_az * sinf(cos_az), z: 0.0 };
                if shape == 6 || shape == 7 {
                    emit_pos.z = host.randf();
                    if shape == 6 {
                        emit_pos.x *= 1.0 - emit_pos.z;
                        emit_pos.y *= 1.0 - emit_pos.z;
                    }
                    emit_pos.z *= gen.aux[AUX_HEIGHT];
                }
                let sin_ca = elevation * sinf(cone_angle);
                let mut vec = Vec3 { x: sin_ca * cosf(cos_az), y: sin_ca * sinf(cos_az), z: elevation * cosf(cone_angle) };
                if shape == 3 {
                    vec.x *= radius;
                    vec.y *= radius;
                    vec.z *= radius;
                }
                let mut emit_pos = mtx::mult_vec(&rot, &emit_pos);
                emit_pos.x += gen.pos.x;
                emit_pos.y += gen.pos.y;
                emit_pos.z += gen.pos.z;
                let vec = mtx::mult_vec(&rot, &vec);
                cur_angle = cos_az;
                host.spawn_particle(&spawn_args(gen, id, gen.kind, emit_pos, vec, gen.size, gen.grav, gen.fric));
            }
            1 => {
                let rnd = host.randf();
                let e = Vec3 { x: rnd * gen.aux[0], y: rnd * gen.aux[1], z: rnd * gen.aux[2] };
                let mut e = mtx::mult_vec(&rot, &e);
                e.x += gen.pos.x;
                e.y += gen.pos.y;
                e.z += gen.pos.z;
                let vec = mtx::mult_vec(&rot, &vel_copy);
                host.spawn_particle(&spawn_args(gen, id, gen.kind, e, vec, gen.size, gen.grav, gen.fric));
            }
            2 => {
                sin_az = if gen.radius < 0.0 { 1.0 } else { host.randf() };
                if gen.angle < 0.0 {
                    cur_angle += angle_step;
                } else {
                    let rnd = host.randf();
                    cur_angle = (2.0 * (PI * rnd as f64)) as f32;
                }
                gen.aux[0] = elevation;
                let vel = Vec3 { x: cur_angle, y: sin_az, z: 0.0 };
                host.spawn_particle(&spawn_args(gen, id, gen.kind | 4, Vec3::default(), vel, gen.size, angle1, angle3));
            }
            5 => {
                let mut e = Vec3 { x: host.randf(), y: 0.0, z: 0.0 };
                e.y = host.randf();
                e.z = host.randf();
                let a = &gen.aux;
                match gen.aux_flag {
                    1 => e.x = snap(e.x, 0.5),
                    2 => e.y = snap(e.y, 0.5),
                    3 => {
                        let rnd = host.randf();
                        let a2 = a[0];
                        if rnd > a2 / (a2 + a[1]) {
                            e.y = snap(e.y, 0.5);
                        } else {
                            e.x = snap(e.x, 0.5);
                        }
                    }
                    4 => e.z = snap(e.z, 0.5),
                    5 => {
                        let rnd = host.randf();
                        let a2 = a[0];
                        if rnd > a2 / (a2 + a[2]) {
                            e.z = snap(e.z, 0.5);
                        } else {
                            e.x = snap(e.x, 0.5);
                        }
                    }
                    6 => {
                        let rnd = host.randf();
                        let a2 = a[1];
                        if rnd > a2 / (a2 + a[2]) {
                            e.z = snap(e.z, 0.5);
                        } else {
                            e.y = snap(e.y, 0.5);
                        }
                    }
                    7 => {
                        let rnd = host.randf();
                        let (a2, b2, c2) = (a[2], a[1], a[0]);
                        let r0 = 1.0 / fmadds(c2, b2 + a2, b2 * a2);
                        if rnd < r0 * (c2 * b2) {
                            e.z = snap(e.z, 0.5);
                        } else if rnd > fmadds(-r0, c2 * a2, 1.0) {
                            e.y = snap(e.y, 0.5);
                        } else {
                            e.x = snap(e.x, 0.5);
                        }
                    }
                    _ => {}
                }
                e.x -= 0.5;
                e.y -= 0.5;
                e.z -= 0.5;
                let r = &gen.aux[AUX_RECT_XX..];
                let (xx, xy, xz, yx, yy, yz, zx, zy, zz) = (r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8]);
                let t = Vec3 {
                    x: fmadds(zx, e.z, fmadds(xx, e.x, yx * e.y)),
                    y: fmadds(zy, e.z, fmadds(xy, e.x, yy * e.y)),
                    z: fmadds(zz, e.z, fmadds(xz, e.x, yz * e.y)),
                };
                let mut e = mtx::mult_vec(&rot, &t);
                e.x += gen.pos.x;
                e.y += gen.pos.y;
                e.z += gen.pos.z;
                let mag = sqrtf(fmadds(zz, zz, fmadds(zx, zx, zy * zy)));
                let scale = elevation / mag;
                let vec = mtx::mult_vec(&rot, &Vec3 { x: zx * scale, y: zy * scale, z: zz * scale });
                host.spawn_particle(&spawn_args(gen, id, gen.kind, e, vec, gen.size, gen.grav, gen.fric));
            }
            8 => {
                let r0 = gen.aux[AUX_HEIGHT];
                if r0 == 0.0 || (r0 as f64 - PI).abs() < eps {
                    radius = sqrtf(host.randf());
                    radius = (FRAC_PI_2 * radius as f64) as f32;
                    let rnd2 = host.randf();
                    if rnd2 < 0.5 {
                        radius = (PI - radius as f64) as f32;
                    }
                } else {
                    radius = sqrtf(host.randf());
                    radius = gen.aux[AUX_HEIGHT] * radius;
                }
                cone_angle = host.randf();
                cone_angle = (2.0 * (PI * cone_angle as f64)) as f32;
                let r2 = gen.radius;
                if r2 < 0.0 {
                    sin_az = -r2;
                } else {
                    sin_az = sqrtf(host.randf());
                    sin_az = r2 * sin_az;
                }
                let cos_r = cosf(cone_angle);
                cos_az = sinf(radius);
                sin_el = sinf(cone_angle);
                let sin_r = sinf(radius);
                let v = Vec3 { x: cos_az * cos_r, y: sin_r * sin_el, z: cosf(radius) };
                let ep = mtx::mult_vec(&rot, &v);
                let s = gen.aux[0];
                let mut vec = Vec3 { x: ep.x * s, y: ep.y * s, z: ep.z * s };
                if gen.radius >= 0.0 && s < 0.0 {
                    let scale = sin_az / gen.radius;
                    vec.x *= scale;
                    vec.y *= scale;
                    vec.z *= scale;
                }
                let e = Vec3 {
                    x: fmadds(sin_az, ep.x, gen.pos.x),
                    y: fmadds(sin_az, ep.y, gen.pos.y),
                    z: fmadds(sin_az, ep.z, gen.pos.z),
                };
                host.spawn_particle(&spawn_args(gen, id, gen.kind, e, vec, gen.size, gen.grav, gen.fric));
            }
            _ => host.shape_emit(id, gen, &rot),
        }
        gen.count -= 1.0;
    }
    gen.count
}
