//! melee_world::tu::efalt: 0x49D's offset and the M_PI_2 narrowing bit-exact vs the C
//! (cref/efalt_ref.c via cref/gen_efalt_ref.sh), plus efAlt_Spawn smoke tests.
use core::ptr::null_mut;
use melee_types::{EF_Effect, Fighter, FighterBone, HSD_GObj, HSD_Generator, HSD_JObj, Vec3};
use melee_world::tu::efalt::*;
use melee_world::tu::efasync::{facing_rot, EfAsyncState, EfHost, VaArg, VaList, EF_LOADKIND_SYNC};
use melee_world::tu::efdata::EfData;

const REF: &str = include_str!("data/tu_efalt_ref.txt");

#[test]
fn offset_and_rot_bit_exact() {
    let mut n = 0;
    for line in REF.lines() {
        let t: Vec<u32> = line.split(' ').map(|s| u32::from_str_radix(s, 16).unwrap()).collect();
        let z = f32::from_bits(t[0]);
        assert_eq!(offset_z(z).to_bits(), t[1], "{line}");
        assert_eq!(facing_rot(z).to_bits(), t[2], "{line}");
        n += 1;
    }
    assert_eq!(n, 2000);
}

struct Mock {
    ef: EfData,
    st: EfAsyncState,
    log: Vec<String>,
    keep: Vec<Box<[u8]>>,
}
impl Mock {
    fn alloc<T>(&mut self) -> *mut T {
        let mut b = vec![0u8; core::mem::size_of::<T>() + 16].into_boxed_slice();
        let p = b.as_mut_ptr();
        self.keep.push(b);
        let a = core::mem::align_of::<T>();
        (((p as usize) + a - 1) & !(a - 1)) as *mut T
    }
    unsafe fn effect(&mut self) -> *mut EF_Effect {
        let j: *mut HSD_JObj = self.alloc();
        let c: *mut HSD_JObj = self.alloc();
        (*j).child = c;
        (*j).translate.z = 10.0;
        let g: *mut HSD_GObj = self.alloc();
        (*g).hsd_obj = j.cast();
        let e: *mut EF_Effect = self.alloc();
        (*e).gobj = g;
        e
    }
}
unsafe impl EfHost for Mock {
    fn ef_data(&mut self) -> &mut EfData { &mut self.ef }
    fn ef_async(&mut self) -> &mut EfAsyncState { &mut self.st }
    unsafe fn efLib_Create_Attach(&mut self, id: u32, _g: *mut HSD_GObj, _j: *mut HSD_JObj) -> *mut EF_Effect {
        self.log.push(format!("Attach {id:#x}"));
        self.effect()
    }
    unsafe fn efLib_Create_AttachChild(&mut self, id: u32, _g: *mut HSD_GObj, _j: *mut HSD_JObj) -> *mut EF_Effect {
        self.log.push(format!("Child {id:#x}"));
        self.effect()
    }
    unsafe fn efLib_Create_Attach_Pos(&mut self, id: u32, _g: *mut HSD_GObj, _p: *mut Vec3) -> *mut EF_Effect {
        self.log.push(format!("Pos {id:#x}"));
        self.effect()
    }
    unsafe fn hsd_8039EFAC(&mut self, _l: i32, b: i32, id: i32, _j: *mut HSD_JObj) -> *mut HSD_Generator {
        self.log.push(format!("Efac {b} {id:#x}"));
        null_mut()
    }
    unsafe fn HSD_JObjSetMtxDirtySub(&mut self, _j: *mut HSD_JObj) {}
    unsafe fn HSD_JObjAnimAll(&mut self, _j: *mut HSD_JObj) {}
}
unsafe impl EfAltHost for Mock {}

fn mock() -> Mock {
    Mock { ef: EfData::default(), st: EfAsyncState::default(), log: vec![], keep: vec![] }
}

#[test]
fn spawn_49d_reads_two_floats_and_offsets_child() {
    let mut m = mock();
    unsafe {
        let gj: *mut HSD_JObj = m.alloc();
        (*gj).scale.y = 1.25;
        let g: *mut HSD_GObj = m.alloc();
        (*g).hsd_obj = gj.cast();
        let mut pos = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
        let mut dir = -1.0f32;
        let mut z = 0.5f32;
        let args = [VaArg::Vec3(&mut pos), VaArg::F32(&mut dir), VaArg::F32(&mut z)];
        let mut vl = VaList::new(&args);
        let e = efAlt_Spawn(&mut m, 0x49D, g, &mut vl) as *mut EF_Effect;
        let j = (*(*e).gobj).hsd_obj as *mut HSD_JObj;
        assert_eq!((*j).scale.x, 1.25);
        assert_eq!((*(*j).child).translate.z, 4.0);
        assert_eq!((*j).rotate.y, facing_rot(-1.0));
        assert_eq!((*j).rotate.z, -0.5);
        assert_eq!(vl.pos, 3);
        assert!((*e).update.is_some());
        assert_eq!(m.ef.load_kind, EF_LOADKIND_SYNC);
    }
}

#[test]
fn spawn_494_chains_three_and_47a_spawns_generator() {
    let mut m = mock();
    unsafe {
        let parts: *mut FighterBone = m.alloc::<[FighterBone; 64]>().cast();
        let fp: *mut Fighter = m.alloc();
        (*fp).parts = parts;
        let g: *mut HSD_GObj = m.alloc();
        (*g).user_data = fp.cast();
        let e = efAlt_Spawn(&mut m, 0x494, g, &mut VaList::new(&[])) as *mut EF_Effect;
        assert_eq!(m.log, vec!["Child 0x1388", "Attach 0x1389", "Attach 0x138a"]);
        assert_eq!((*e).lifetime, 0x41);
        assert!((*e).user_data.is_null());
        assert_eq!((*(*e).next).user_data, fp.cast());
        assert_eq!((*(*(*e).next).next).user_data, fp.cast());

        m.log.clear();
        let mut j: HSD_JObj = core::mem::zeroed();
        let mut dir = 2.0f32;
        let args = [VaArg::JObj(&mut j), VaArg::F32(&mut dir)];
        efAlt_Spawn(&mut m, 0x47A, null_mut(), &mut VaList::new(&args));
        assert_eq!(m.log, vec!["Attach 0x3e8", "Efac 1 0x3e9"]);

        m.log.clear();
        assert!(efAlt_Spawn(&mut m, 0x4FF, null_mut(), &mut VaList::new(&[])).is_null());
        assert!(m.log.is_empty());
    }
}
