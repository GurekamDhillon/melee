//! C functions covered (C name = Rust implementation the tu:: wrapper delegates to): hsd_8039D214 = GeneratorSystem::follow (op 21),
//! hsd_8039D9C8 = alloc (op 10), hsd_8039DAD4 = generator::emit (op 23), hsd_8039D3AC = remove (via remove_by_jobj), psAttachParticleAppSRT and
//! psAttachGeneratorAppSRT = AppSrtPool::attach, psRemoveGeneratorSRT = remove_generator.
//! Bit-for-bit check of baselib generator.c + psappsrt.c against the C. Vectors:
//! `tests/data/oracle_generator.txt`, written by `oracle/build_generator.sh` (both TUs through
//! `helper_generator.c`, over the real list.c / random.c / mtx.c / mtx_pc.c / gekko_fp.c /
//! lbtrigf.c, compiled with the shipping build's contraction + gwtool lowering; generated inputs
//! only). Row format: see `oracle/helper_generator.c`. Each op row checks an FNV-1a hash of every
//! host call the op made (particle spawns with all their arguments, unrefs, SetupMatrix,
//! callbacks, stage-slot hooks), its result, and the whole generator / app SRT / JObj state.
use melee_runtime::generator::{GenId, GeneratorHost, GeneratorSystem, HsdGenerator, PsCmdList, PsSpawn};
use melee_runtime::mtx::{Mtx, Vec3};
use melee_runtime::psappsrt::{AppSrtHost, HsdPsAppSrt, SrtId};
use melee_runtime::random::HsdRand;

const DATA: &str = include_str!("data/oracle_generator.txt");
const NBANK: usize = 3;
const NREC: usize = 6;
const NTG: usize = 4;
const NJ: usize = 6;
const NPART: usize = 4;
const DIRTY: u32 = 1 << 6;
const USER_DEF: u32 = 1 << 23;

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn b(f: f32) -> u32 {
    if f.is_nan() { 0x7FC0_0000 } else { f.to_bits() }
}
fn ix(o: Option<u32>) -> u32 {
    o.unwrap_or(u32::MAX)
}

#[derive(Default, Clone)]
struct J {
    flags: u32,
    mtx: Mtx,
    child: Option<usize>,
    next: Option<usize>,
    refc: u32,
}

#[derive(Default)]
struct Host {
    rand: HsdRand,
    hooks_on: bool,
    hooks: bool,
    nc: u32,
    pb: u32,
    counts: [i32; NBANK],
    recs: [[Option<PsCmdList>; NREC]; NBANK],
    tgs: [[Option<u16>; NTG]; NBANK],
    jobjs: Vec<J>,
    parts: [(Option<SrtId>, u16); NPART],
    eye: Vec3,
    up: Vec3,
    log: Vec<u32>,
    spawns: usize,
}

impl AppSrtHost for Host {
    fn free_srt(&mut self, tag: u32, id: SrtId, _: &HsdPsAppSrt) {
        self.log.extend([5, tag, id]);
    }
}

impl GeneratorHost<usize> for Host {
    fn randf(&mut self) -> f32 {
        self.rand.randf()
    }
    fn jobj_setup_matrix(&mut self, j: usize) {
        let o = &mut self.jobjs[j];
        if o.flags & USER_DEF == 0 && o.flags & DIRTY != 0 {
            self.log.extend([4, j as u32]);
            self.jobjs[j].flags &= !DIRTY;
        }
    }
    fn jobj_mtx(&self, j: usize) -> Mtx {
        self.jobjs[j].mtx
    }
    fn jobj_ref_inc(&mut self, j: usize) {
        self.jobjs[j].refc += 1;
    }
    fn jobj_unref(&mut self, j: usize) {
        self.log.extend([3, j as u32]);
    }
    fn jobj_flags(&self, j: usize) -> u32 {
        self.jobjs[j].flags
    }
    fn jobj_child(&self, j: usize) -> Option<usize> {
        self.jobjs[j].child
    }
    fn jobj_next(&self, j: usize) -> Option<usize> {
        self.jobjs[j].next
    }
    fn ps_camera(&mut self) -> Option<(Vec3, Vec3)> {
        Some((self.eye, self.up))
    }
    fn spawn_particle(&mut self, p: &PsSpawn) {
        self.spawns += 1;
        self.log.extend([1, p.link_no as u32, p.bank as u32, p.kind as u32, p.tex_group as u32, p.cmd_list, p.life as u32, p.arg6 as u32, p.gen]);
        self.log.extend([p.pos.x, p.pos.y, p.pos.z, p.vel.x, p.vel.y, p.vel.z, p.size, p.grav, p.fric].map(b));
    }
    fn particle_8039d0a0(&mut self, gen: GenId, _: &HsdGenerator<usize>) {
        self.log.extend([2, gen]);
    }
    fn shape_emit(&mut self, gen: GenId, _: &mut HsdGenerator<usize>, rot: &Mtx) {
        if self.hooks {
            self.log.extend([6, gen]);
            self.log.extend(rot.iter().flatten().map(|f| b(*f)));
        }
    }
    fn shape_init(&mut self, gen: GenId, _: &mut HsdGenerator<usize>) {
        if self.hooks {
            self.log.extend([7, gen]);
        }
    }
    fn on_create(&mut self, gen: GenId, _: &mut HsdGenerator<usize>) {
        if self.hooks {
            self.log.extend([8, gen]);
        }
    }
    fn cmd_list_count(&self, bank: i32) -> i32 {
        self.counts[bank as usize]
    }
    fn cmd_list(&self, bank: i32, idx: i32) -> Option<PsCmdList> {
        self.recs[bank as usize][idx as usize]
    }
    fn tex_group_palflag(&self, bank: i32, tg: u8) -> Option<u16> {
        self.tgs[bank as usize][tg as usize]
    }
    fn reset_ps_globals(&mut self) {
        self.hooks = false;
    }
    fn stage_slot_native_context(&mut self) -> bool {
        self.log.extend([12, self.nc]);
        self.nc != 0
    }
    fn stage_slot_particle_begin(&mut self, link: i32) -> bool {
        let r = (link as u32 & 1) ^ self.pb;
        self.log.extend([9, link as u32, r]);
        r != 0
    }
    fn stage_slot_proc_end(&mut self) {
        self.log.push(10);
    }
    fn stage_slot_particle_link_clear(&mut self) {
        self.log.push(11);
    }
}

fn dump(s: &GeneratorSystem<usize>, h: &Host, v: &mut Vec<u32>) {
    v.extend([h.rand.seed, s.live as u32, s.peak as u32, s.id_counter as u32, ix(s.head), ix(s.cursor)]);
    v.extend([s.srts.live as u32, s.srts.peak as u32]);
    v.extend(s.queue.iter().copied());
    v.push(0xEEEE);
    for g in &s.slots {
        let Some(g) = g else {
            v.push(0xDEAD);
            continue;
        };
        v.extend([ix(g.next), g.kind, b(g.random), b(g.count), g.jobj.map_or(u32::MAX, |j| j as u32)]);
        v.extend([g.gen_life, g.type_].map(u32::from));
        v.extend([g.bank, g.link_no, g.tex_group, g.dummy].map(u32::from));
        v.extend([g.idnum as u32, g.life as u32, g.cmd_list]);
        v.extend([g.pos.x, g.pos.y, g.pos.z, g.vel.x, g.vel.y, g.vel.z, g.grav, g.fric, g.size, g.radius, g.angle].map(b));
        v.extend([g.num_child, ix(g.appsrt), g.userfunc.unwrap_or(0), g.callback.unwrap_or(0)]);
        v.extend(g.aux.map(b));
        v.push(g.aux_flag as u32);
    }
    v.push(0xEEEF);
    for t in &s.srts.slots {
        let Some(t) = t else {
            v.push(0xDEAD);
            continue;
        };
        v.push(ix(t.gp));
        v.extend([t.translate.x, t.translate.y, t.translate.z, t.rot.x, t.rot.y, t.rot.z, t.rot.w, t.scale.x, t.scale.y, t.scale.z].map(b));
        v.extend([t.status as u32, t.frame_num as u32, t.used_count as u32, t.freefunc.unwrap_or(0), t.idnum as u32, t.xa2 as u32]);
    }
    v.push(0xEEF0);
    for j in &h.jobjs {
        v.extend([j.refc, j.flags]);
    }
    for p in &h.parts {
        v.push(ix(p.0));
    }
}

fn fnv(ws: &[u32]) -> u32 {
    let mut h = 2166136261u32;
    for w in ws {
        for i in 0..4 {
            h ^= (w >> (8 * i)) & 0xFF;
            h = h.wrapping_mul(16777619);
        }
    }
    h
}

fn setup(t: &[u32]) -> Host {
    let mut it = t.iter().copied();
    let mut n = || it.next().unwrap();
    let mut h = Host::default();
    h.rand.seed = n();
    h.hooks_on = n() != 0;
    h.nc = n();
    h.pb = n();
    for bk in 0..NBANK {
        h.counts[bk] = n() as i32;
        for tg in 0..NTG {
            let present = n() != 0;
            let pal = n() as u16;
            h.tgs[bk][tg] = present.then_some(pal);
        }
        for r in 0..NREC {
            let present = n() != 0;
            let mut c = PsCmdList { type_: n() as u16, tex_group: n() as u16, gen_life: n() as u16, life: n() as u16, kind: n(), ..Default::default() };
            let mut f = || f32::from_bits(n());
            c.grav = f();
            c.fric = f();
            c.vx = f();
            c.vy = f();
            c.vz = f();
            c.radius = f();
            c.angle = f();
            c.random = f();
            c.size = f();
            c.param1 = f();
            c.param2 = f();
            c.param3 = f();
            c.cmd_list = (bk * 16 + r + 1) as u32;
            h.recs[bk][r] = present.then_some(c);
        }
    }
    h.jobjs = vec![J::default(); NJ];
    for i in 0..NJ {
        let p = n();
        h.jobjs[i].flags = n();
        for r in 0..3 {
            for c in 0..4 {
                h.jobjs[i].mtx[r][c] = f32::from_bits(n());
            }
        }
        if p != u32::MAX {
            let p = p as usize;
            match h.jobjs[p].child {
                None => h.jobjs[p].child = Some(i),
                Some(mut c) => {
                    while let Some(nx) = h.jobjs[c].next {
                        c = nx;
                    }
                    h.jobjs[c].next = Some(i);
                }
            }
        }
    }
    for i in 0..NPART {
        h.parts[i] = (None, n() as u16);
    }
    let mut f = || f32::from_bits(n());
    h.eye = Vec3 { x: f(), y: f(), z: f() };
    h.up = Vec3 { x: f(), y: f(), z: f() };
    h
}

fn reset(s: &mut GeneratorSystem<usize>, h: &mut Host) {
    s.init(h);
    s.srts.init(16, 0xA4);
    h.hooks = h.hooks_on;
}

fn run_op(a: &[u32], s: &mut GeneratorSystem<usize>, h: &mut Host) -> u32 {
    let gid = |k: u32| (k != u32::MAX).then_some(k);
    let job = |k: u32| (k != u32::MAX).then_some(k as usize);
    let i = |k: u32| k as i32;
    let r = |o: Option<u32>| ix(o);
    match a[0] {
        0 => r(s.spawn(i(a[1]), i(a[2]), i(a[3]), h)),
        1 => r(s.spawn_on_jobj(i(a[1]), i(a[2]), i(a[3]), job(a[4]), h)),
        2 => r(s.spawn_on_jobj_once(i(a[1]), i(a[2]), i(a[3]), job(a[4]), h)),
        3 => {
            s.update(a[1], h);
            0
        }
        4 => {
            if let Some(g) = gid(a[1]) {
                s.kill(g, h);
            }
            0
        }
        5 => {
            s.kill_by_jobj(job(a[1]), h);
            0
        }
        6 => {
            s.kill_by_jobj_tree(job(a[1]), h);
            0
        }
        7 => {
            s.remove_by_jobj(job(a[1]), h);
            0
        }
        8 => {
            if let Some(g) = gid(a[1]) {
                melee_runtime::generator::set_userfunc(s.gen_mut(g), (a[2] != 0).then_some(a[2]));
            }
            0
        }
        9 => s.next_idnum() as u32,
        10 => r(s.alloc()),
        11 => {
            if let Some(g) = gid(a[1]) {
                let g = s.gen_mut(g);
                let v = a[3];
                let fv = f32::from_bits(v);
                match a[2] {
                    0 => g.num_child = v,
                    1 => g.random = fv,
                    2 => g.radius = fv,
                    3 => g.angle = fv,
                    4 => g.type_ ^= v as u16,
                    5 => g.kind ^= v,
                    6 => g.count = fv,
                    7 => g.aux_flag = v as u16,
                    9 => {
                        for (i, k) in [4usize, 5, 6, 8, 9, 10].into_iter().enumerate() {
                            g.aux[k] = f32::from_bits(a[3 + i]);
                        }
                    }
                    10 => g.pos = Vec3 { x: fv, y: f32::from_bits(a[4]), z: f32::from_bits(a[5]) },
                    _ => g.aux[a[4] as usize] = fv,
                }
            }
            0
        }
        12 => {
            let j = &mut h.jobjs[a[1] as usize];
            if a[2] != 0 {
                j.flags |= DIRTY;
            }
            for k in 0..12 {
                j.mtx[k / 4][k % 4] = f32::from_bits(a[3 + k]);
            }
            0
        }
        13 => match gid(a[1]) {
            Some(g) => {
                let GeneratorSystem { slots, srts, .. } = s;
                let slot = &mut slots[g as usize].as_mut().unwrap().appsrt;
                srts.attach(Some(slot), gid(a[2])) as u32
            }
            None => -9i32 as u32,
        },
        14 => match gid(a[1]) {
            Some(g) => {
                let mut slot = s.gen(g).appsrt;
                let c = s.srts.remove_generator(g, &mut slot, h);
                s.gen_mut(g).appsrt = slot;
                c as u32
            }
            None => -9i32 as u32,
        },
        15 => {
            let p = a[1] as usize;
            let mut slot = h.parts[p].0;
            let id = s.srts.add_begin(&mut slot, a[2] as i32, h.parts[p].1);
            h.parts[p].0 = slot;
            r(id)
        }
        16 => {
            let p = a[1] as usize;
            let mut slot = h.parts[p].0;
            let c = s.srts.attach(Some(&mut slot), gid(a[2]));
            h.parts[p].0 = slot;
            c as u32
        }
        17 => {
            let p = a[1] as usize;
            let mut slot = h.parts[p].0;
            let c = s.srts.remove_particle(&mut slot, h);
            h.parts[p].0 = slot;
            c as u32
        }
        18 => {
            if let Some(t) = gid(a[1]) {
                s.srts.get_mut(t).freefunc = Some(0x77);
            }
            0
        }
        19 => {
            s.stage_slot_particles_clear(h);
            0
        }
        20 => {
            reset(s, h);
            for p in h.parts.iter_mut() {
                p.0 = None;
            }
            0
        }
        21 => {
            if let Some(g) = gid(a[1]) {
                s.follow(g, h);
            }
            0
        }
        22 => {
            if let Some(g) = gid(a[1]) {
                s.attach_transform(g, h);
            }
            0
        }
        23 => {
            if let Some(g) = gid(a[1]) {
                let mut gen = s.slots[g as usize].take().unwrap();
                gen.count = f32::from_bits(a[2]);
                let rem = melee_runtime::generator::emit(&mut gen, g, h);
                s.slots[g as usize] = Some(gen);
                h.log.push(b(rem));
            }
            0
        }
        o => panic!("op {o}"),
    }
}

#[test]
fn generator_matches_c() {
    let mut sys = GeneratorSystem::<usize>::default();
    let mut host = Host::default();
    let (mut rows, mut fails, mut spawns) = (0usize, 0usize, 0usize);
    for (ln, line) in DATA.lines().enumerate() {
        let t: Vec<&str> = line.split_whitespace().collect();
        if t[0] == "S" {
            spawns += host.spawns;
            let w: Vec<u32> = t[1..].iter().map(|s| hx(s)).collect();
            host = setup(&w);
            // lbl_804D6368 is a static nothing resets (not even hsd_8039D354).
            let idc = sys.id_counter;
            sys = GeneratorSystem::default();
            sys.id_counter = idc;
            reset(&mut sys, &mut host);
            continue;
        }
        let bar = t.iter().position(|s| *s == "|").unwrap();
        let args: Vec<u32> = t[1..bar].iter().map(|s| hx(s)).collect();
        let want = hx(t[bar + 1]);
        host.log.clear();
        let ret = run_op(&args, &mut sys, &mut host);
        let mut ws = std::mem::take(&mut host.log);
        ws.push(ret);
        dump(&sys, &host, &mut ws);
        rows += 1;
        if fnv(&ws) != want {
            fails += 1;
            if fails <= 5 {
                eprintln!("line {}: op {:?}: hash {:08x} != {:08x}\n  words: {}", ln + 1, args, fnv(&ws), want,
                    ws.iter().map(|w| format!("{w:08x}")).collect::<Vec<_>>().join(" "));
            }
            if fails >= 50 {
                break;
            }
        }
    }
    spawns += host.spawns;
    eprintln!("generator oracle: {rows} rows, {fails} failed, {spawns} particle spawns");
    assert_eq!(fails, 0);
}
