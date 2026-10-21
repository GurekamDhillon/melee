//! Synthetic-bank checks of the particle port (no disc data). The FD background emitter shape is
//! reproduced with made-up numbers: a type-8 generator whose particle list draws four times at
//! birth (opcode 0xBA) and then dies.
use super::*;
use crate::random::HsdRand;

struct Host {
    rand: HsdRand,
    draws: u32,
}

impl PsHost for Host {
    fn randf(&mut self) -> f32 {
        self.draws += 1;
        self.rand.randf()
    }
    fn jobj_setup_matrix(&mut self, _j: JObjRef) -> Mtx {
        mtx::identity()
    }
}

fn push32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}

/// A version-0x42 cmd bank with one list at index `base`, followed by a one-group tex bank.
fn bank(base: u32, type_: u16, life: u16, gen_life: u16, kind: u32, floats: [f32; 12], cmd: &[u8]) -> (Arc<[u8]>, u32, u32) {
    let mut d = Vec::new();
    d.extend_from_slice(&0x0042_0000u32.to_be_bytes());
    push32(&mut d, base);
    push32(&mut d, 1);
    push32(&mut d, 0x10); // entry 0 -> offset 0x10
    d.extend_from_slice(&type_.to_be_bytes());
    d.extend_from_slice(&0u16.to_be_bytes());
    d.extend_from_slice(&gen_life.to_be_bytes());
    d.extend_from_slice(&life.to_be_bytes());
    push32(&mut d, kind);
    for f in floats {
        push32(&mut d, f.to_bits());
    }
    d.extend_from_slice(cmd);
    while d.len() % 4 != 0 {
        d.push(0);
    }
    let tex = d.len() as u32;
    push32(&mut d, 1); // one group
    push32(&mut d, 8); // at tex+8
    for w in [1u32, 0, 0, 8, 8] {
        push32(&mut d, w);
    }
    push32(&mut d, 0); // palnum, palflag
    push32(&mut d, 0x40); // texTable[0] non-null
    (Arc::from(d), 0, tex)
}

const FD_LIKE_CMD: &[u8] = &[
    0x40, 0x00, 0xCF, 0x00, 0x3D, 0x3D, 0x3D, 0x64, 0xBA, 0x64, 0x64, 0x64, 0x19, 0xCF, 0x05, 0x3D, 0x3D, 0x3D, 0x00, 0x04, 0xFF,
];

#[test]
fn locate_rewrites_kind_bits() {
    let (d, c, t) = bank(30000, 8, 4, 3999, 0x0640_0000, [0.0; 12], FD_LIKE_CMD);
    let b = PsBankData::locate(&d, c, t).unwrap();
    assert_eq!((b.base, b.end), (30000, 30001));
    assert_eq!(b.cmd(30000).unwrap().kind, 0x0840_0000);
    assert!(b.cmd(30001).is_none());
}

#[test]
fn sphere_emitter_draw_counts() {
    // grav fric vx vy vz radius angle random size p1 p2 p3
    let fl = [0.0, 0.0, 0.0, 0.0, -0.01, -4000.0, 0.087_266_46, 10.0, 30.0, 0.663_225, 0.0, 0.0];
    let (d, c, t) = bank(30000, 8, 4, 3999, 0x0040_0000, fl, FD_LIKE_CMD);
    let mut ps = PsSystem::new();
    ps.load_bank(0x1E, Arc::new(PsBankData::locate(&d, c, t).unwrap()), None);
    let mut h = Host { rand: HsdRand::default(), draws: 0 };
    h.rand.set_logic_seed(12345);
    let g = ps.gen_create_on_jobj(&mut h, 0, 0x1E, 30000, Some(7)).unwrap();
    assert_eq!(h.draws, 1, "count = HSD_Randf()");
    assert_eq!(ps.gen(g).type_, 8 | 0x700);
    let mut prev_live = 0;
    for frame in 0..8 {
        let before = h.draws;
        let c0 = ps.gen(g).count;
        ps.particles_update_all(&mut h, 0x60000);
        assert_eq!(h.draws, before, "the list draws only at birth");
        ps.gen_update_all(&mut h, 0x60000);
        let emitted = (h.draws - before - 1) / 6;
        assert_eq!((h.draws - before - 1) % 6, 0, "2 emitter + 4 bytecode draws per particle");
        assert!(emitted <= 11 && ps.gen(g).count < 1.0, "frame {frame}: c0 {c0}");
        // life 4: a particle lives its birth frame plus four updates.
        let live = ps.particles(0).len();
        assert!(live >= emitted as usize && live <= prev_live + emitted as usize);
        prev_live = live;
        assert_eq!(ps.gen(g).num_child as usize, live);
    }
    let p = ps.particles(0)[0];
    assert!(ps.part(p).kind & KIND_DISP_TEXTURE != 0, "opcode 0x40 found texTable[0]");
    assert_eq!(ps.part(p).prim_col_count, 5);
}

#[test]
fn particle_dies_after_life() {
    let (d, c, t) = bank(0, 0, 2, 1, 0, [0.0; 12], &[0x01, 0xFF]);
    let mut ps = PsSystem::new();
    ps.load_bank(0, Arc::new(PsBankData::locate(&d, c, t).unwrap()), None);
    let mut h = Host { rand: HsdRand::default(), draws: 0 };
    let cl = ps.cmd_list(0, 0).unwrap();
    let p = ps
        .generate_particle0(&mut h, Head::List, 0, 0, 0, 0, Some(cl.cmd_list.clone()), 2, 0, Vec3::default(), Vec3 { x: 1.0, y: 0.0, z: 0.0 }, 1.0, 0.0, 1.0, None, true)
        .unwrap();
    // Birth: wait 1 -> life 3 -> 2, moved once.
    assert_eq!(ps.part(p).life, 2);
    assert_eq!(ps.part(p).pos.x, 1.0);
    ps.particles_update_all(&mut h, 0);
    // Wait expired: 0xFF sets life 1, then life 0 -> deleted.
    assert!(ps.heads[0].is_none());
    assert_eq!(ps.num_parts, 0);
}
