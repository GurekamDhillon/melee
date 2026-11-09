//! `itacc` (the plain Item accessors) vs the verbatim C definitions over a random `Item` record
//! (`melee_runtime/gen/pure/drv_itacc.c`). Regenerate the expected data with
//! `python rust/melee_runtime/gen/pure/run_pure.py itacc`.
#![allow(static_mut_refs)]
#[path = "../../melee_runtime/tests/pure_common/mod.rs"]
mod pure_common;
use melee_types::*;
use melee_world::itacc::*;
use pure_common::*;

static mut MEXCALLS: u32 = 0;

struct Hs(u64);
impl Hs {
    fn w(&mut self, w: u32) {
        self.bytes(&w.to_le_bytes());
    }
    fn bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.0 ^= x as u64;
            self.0 = self.0.wrapping_mul(0x100000001B3);
        }
    }
}

#[test]
fn itacc_matches_c() {
    let mut out = format!("sizeof_Item {}\n", core::mem::size_of::<Item>());
    unsafe {
        MEX_INDEX_STAGE_ITEM = Some(|_a, idx| MEXCALLS += 1 + idx as u32);
    }
    let mut r = Rng::new();
    let mut g: Box<[HSD_GObj; 6]> = Box::new(unsafe { core::mem::zeroed() });
    let mut items: Box<[Item; 6]> = Box::new(unsafe { core::mem::zeroed() });
    let mut arts: Box<[Article; 3]> = Box::new(unsafe { core::mem::zeroed() });
    let mut heads = [core::ptr::null_mut::<HSD_GObj>(); 64];
    unsafe {
        melee_runtime::tu::gobj_native::HSD_GObjPLinkHead = heads.as_mut_ptr();
    }
    for op in 0..20 {
        for k in 0..200 {
            let mut h = Hs(0xCBF29CE484222325);
            for i in 0..6 {
                let p = &mut items[i] as *mut Item as *mut u8;
                for j in 0..core::mem::size_of::<Item>() {
                    unsafe { *p.add(j) = r.u() as u8 };
                }
                items[i].grab_victim = if r.u() & 1 != 0 { core::ptr::null_mut() } else { 0x1000 as *mut HSD_GObj };
                items[i].owner = if r.u() & 1 != 0 { core::ptr::null_mut() } else { 0x2000 as *mut HSD_GObj };
                items[i].xAC8_hurtboxNum = (r.u() % 3) as _;
                // bitfields are set logically: the native C and the GameCube-ordered Rust records number their bits differently
                items[i].xDC8_word.flags.set_x13((r.u() & 1) as _);
                items[i].xDC8_word.flags.set_xC((r.u() & 1) as _);
                unsafe {
                    items[i].xDCD_flag.anon_0.set_b7((r.u() & 1) as _);
                    items[i].xDAA.xDAA_flag.x0.set_b1((r.u() & 1) as _);
                }
                for m in 0..2 {
                    items[i].xACC_itemHurtbox[m].set_skip_update_pos((r.u() & 1) as _);
                }
                items[i].kind = (r.u() % 5) as _;
                g[i].user_data = &mut items[i] as *mut Item as *mut core::ffi::c_void;
                g[i].next = if i < 5 { &mut g[i + 1] } else { core::ptr::null_mut() };
            }
            heads[HSD_GOBJ_PLINK_ITEM as usize] = &mut g[0];
            unsafe {
                for i in 0..30 {
                    it_804A0F60[i] = if r.u() & 1 != 0 { core::ptr::null_mut() } else { &mut arts[0] };
                }
                MEXCALLS = 0;
            }
            let mut pos = Vec { x: 0.0, y: 0.0, z: 0.0 };
            let mut kind = -1i32;
            let mut ret = 0i32;
            let a = r.u() as u32;
            let f = r.f();
            let sel = (r.u() % 6) as usize;
            let gp: *mut HSD_GObj = &mut g[sel];
            unsafe {
                match op {
                    0 => ret = itGetKind(gp) as i32,
                    1 => it_8026B294(gp, &mut pos),
                    2 => it_8026B718(gp, f),
                    3 => ret = it_8026C1E8(gp) as i32,
                    4 => ret = it_8026B3C0((a % 5) as _),
                    5 => it_ResetStageArticleTable(),
                    6 => it_8026B40C(&mut arts[(a % 3) as usize], (It_Kind_Old_Kuri as i32 + ((a >> 4) % 30) as i32) as i32),
                    7 => it_8026B40C(&mut arts[(a % 3) as usize], 5000 + (a % 7) as i32),
                    8 => it_80275414(gp),
                    9 => it_802756D0(gp),
                    10 => it_802756E0(gp),
                    11 => it_802714C0(gp),
                    12 => it_80274C60(gp),
                    13 => it_802F2014(gp, a as i32),
                    14 => ret = it_802F2020(gp),
                    15 => ret = it_802ECC8C(gp),
                    16 => it_802ECC98(gp, f),
                    17 => it_802ECCA4(gp, &mut kind, &mut pos),
                    18 => it_802ECD1C(gp, &mut pos),
                    _ => {}
                }
            }
            h.w(ret as u32);
            h.w(kind as u32);
            h.w(unsafe { MEXCALLS });
            h.w(pos.x.to_bits());
            h.w(pos.y.to_bits());
            h.w(pos.z.to_bits());
            for i in 0..6 {
                let it = &items[i];
                h.w(it.kind as u32);
                h.w(it.pos.x.to_bits());
                h.w(it.pos.y.to_bits());
                h.w(it.pos.z.to_bits());
                h.w(it.xCBC_hitlagFrames.to_bits());
                h.w(!it.grab_victim.is_null() as u32);
                h.w(!it.owner.is_null() as u32);
                unsafe {
                    h.w(it.xDC8_word.flags.x13() as u32);
                    h.w(it.xDC8_word.flags.xC() as u32);
                    h.w(it.xDCD_flag.anon_0.b7() as u32);
                    h.w(it.xDAA.xDAA_flag.x0.b1() as u32);
                }
                h.w(it.xD0C as u32);
                h.w(it.xAC8_hurtboxNum as u32);
                for m in 0..2 {
                    h.w(it.xACC_itemHurtbox[m].skip_update_pos() as u32);
                }
                h.w(it.x378_itemColl.joint_id_only as u32);
                unsafe {
                    h.w(it.xDD4_itemVar.coin.x0 as u32);
                    h.w(it.xDD4_itemVar.tincle.x20 as u32);
                    h.w(it.xDD4_itemVar.tincle.x50.to_bits());
                    h.w(it.xDD4_itemVar.tincle.x54.to_bits());
                }
                h.w(it.msid as u32);
            }
            for i in 0..30 {
                let p = unsafe { it_804A0F60[i] };
                h.w(if p.is_null() { 0 } else { ((p as usize - &arts[0] as *const Article as usize) / core::mem::size_of::<Article>()) as u32 + 1 });
            }
            out += &line(&format!("op{op}"), k, h.0);
        }
    }
    compare_runtime_data("itacc", &out);
}
