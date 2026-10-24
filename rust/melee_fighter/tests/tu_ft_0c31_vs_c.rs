//! src/tu/ft_0c31.rs (match-start entry) vs the C (gen/cref/tu/ft_0c31.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ft_0c31`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use melee_fighter::hooks as h;
use melee_fighter::stale::PLAYER_SLOTS;
use melee_fighter::tu::ft_0c31 as m;
use melee_fighter::types::*;
use tu_ftco_harness::*;

type G = *mut HSD_GObj;
static mut J: *mut HSD_JObj = core::ptr::null_mut();
static mut A: *mut HSD_JObj = core::ptr::null_mut();
static mut JOINT: *mut HSD_Joint = core::ptr::null_mut();

fn ep<T>(p: *const T) -> u32 {
    let a = p as usize;
    if a == 0 {
        return 0;
    }
    unsafe {
        let (fb, f2b) = (F as usize, F2 as usize);
        if a >= fb && a < fb + core::mem::size_of::<Fighter>() {
            return 0x1000 + (a - fb) as u32;
        }
        if a >= f2b && a < f2b + core::mem::size_of::<Fighter>() {
            return 0x2000 + (a - f2b) as u32;
        }
        if a == J as usize { return 0x3000; }
        if a == A as usize { return 0x4000; }
        if a == GO as usize { return 0x5000; }
        if a == GO2 as usize { return 0x6000; }
    }
    0xFFFF
}
fn hv(v: &Vec3) { hm(b(v.x)); hm(b(v.y)); hm(b(v.z)); }
fn hj(j: &HSD_JObj) {
    hv(&j.scale);
    hv(&j.translate);
    hm(b(j.rotate.x)); hm(b(j.rotate.y)); hm(b(j.rotate.z)); hm(b(j.rotate.w));
    hm(j.flags);
}
static mut HH: u32 = 0;
fn hm(v: u32) { unsafe { if DBGT { println!("H {}", v); } HH = HH.wrapping_mul(0x0100_0193) ^ v } }
fn cbid(cb: fn(G)) -> u32 {
    let a = cb as usize;
    if a == m::fn_800C63BC as fn(G) as usize { 1 } else if a == m::fn_800C63E0 as fn(G) as usize { 2 } else { 3 }
}
fn bx(bx: &ftCollisionBox) {
    t(b(bx.top)); t(b(bx.bottom)); t(b(bx.right.x)); t(b(bx.right.y)); t(b(bx.left.x)); t(b(bx.left.y));
}

#[test]
fn ft_0c31_matches_c() {
    init();
    unsafe {
        J = Box::leak(Box::new(core::mem::zeroed()));
        A = Box::leak(Box::new(core::mem::zeroed()));
        JOINT = Box::leak(Box::new(core::mem::zeroed()));
        (*GO).hsd_obj = J as *mut core::ffi::c_void;
        m::ext::FIGHTER_804D6514 = Some(|| JOINT);
        m::ext::FIGHTER_CHANGE_MOTION_STATE = Some(|g, ms, fl, a, bb, c, o| { t(4); t(ep(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(ep(o)); });
        m::ext::HSD_JOBJ_SET_MTX_DIRTY_SUB = Some(|j| { t(5); t(ep(j)); });
        m::ext::FTCOMMON_SET_ACCESSORY = Some(|fp, j| { t(6); t(ep(fp as *const Fighter)); t((j == JOINT) as u32); fp.x20A0_accessory = A; });
        m::ext::EFASYNC_SPAWN = Some(|g, q, kind, id, j, v| { t(7); t(ep(g)); t(ep(q)); t(kind); t(id); t(ep(j)); t(ep(v)); });
        m::ext::LBAUDIOAX_80024304 = Some(|id| { t(8); t(id as u32); 0 });
        m::ext::FT_80083E64 = Some(|g, bb, cb| { t(9); t(ep(g)); t(ep(bb as *const ftCollisionBox)); bx(bb); t(cbid(cb)); });
        m::ext::FT_800846B0 = Some(|g, bb, cb| { t(10); t(ep(g)); t(ep(bb as *const ftCollisionBox)); bx(bb); t(cbid(cb)); });
        m::ext::FTCOLL_8007B760 = Some(|g, n| { t(11); t(ep(g)); t(n as u32); });
        h::FTCO_800BFFD0 = Some(|fp, k, bb| { t(12); t(ep(fp as *const Fighter)); t(k as u32); t(bb as u32); false });
        h::FTCO_FALL_ENTER = Some(|g| { t(13); t(ep(g)); });
        h::FT_8008A2BC = Some(|g| { t(14); t(ep(g)); });
    }
    const FL: [u32; 6] = [0, 1 << 6, 1 << 25, (1 << 6) | (1 << 25), (1 << 23) | (1 << 6), 1 << 23];
    let mut c = Check::new(include_str!("data/tu_ft_0c31.txt"));
    for op in 0..16u32 {
        for k in 0..1500u32 {
            case(op, k);
            unsafe { DBGT = std::env::var("TUDBG").map(|v| v == format!("{op}:{k}")).unwrap_or(false); }
            unsafe {
                HH = 0x811C_9DC5;
                *J = core::mem::zeroed();
                *A = core::mem::zeroed();
                (*J).flags = FL[(rnd() % 6) as usize];
                (*A).flags = FL[(rnd() % 6) as usize];
                (*J).scale.x = rw(); (*J).scale.y = rw(); (*J).scale.z = rw();
                (*A).translate.x = rw();
                (*A).rotate.y = rw();
                let cdv = cd();
                cdv.x6BC = (rnd() % 9) as i32 - 1;
                cdv.x6C0 = (rnd() % 9) as i32;
                cdv.x6C4 = rw();
                cdv.x6C8 = (rnd() % 50) as i32;
                let u4c = (rnd() % 5) as i8;
                let bit4 = (rnd() & 1) as u8;
                let f = f();
                f.player_id = (rnd() % 3) as u8;
                f.cur_pos.x = rw(); f.cur_pos.y = rw(); f.cur_pos.z = rw();
                f.facing_dir = rg(); f.x34_scale.y = rg(); f.x34_scale.x = rg(); f.x40 = rw();
                f.co_attrs.trophy_scale = rw();
                f.ground_or_air = (rnd() & 1) as _;
                f.set_is_sub_fighter((rnd() & 1) as _);
                let _ = rnd();
                f.x20A0_accessory = A;
                f.coll_data.ecb.top.y = rw(); f.coll_data.ecb.bottom.y = rw();
                f.coll_data.ecb.right.x = rw(); f.coll_data.ecb.right.y = rw();
                f.coll_data.ecb.left.x = rw(); f.coll_data.ecb.left.y = rw();
                let e = &mut f.mv.co.entry;
                e.timer = (rnd() % 7) as i32 - 1;
                e.x4 = rw(); e.x8.x = rw(); e.x8.y = rw(); e.x8.z = rw();
                e.x14.x = rw(); e.x14.y = rw(); e.x14.z = rw();
                e.x20 = rw(); e.x24 = rw(); e.x28 = rw();
                e.x2C.top = rw(); e.x2C.bottom = rw();
                e.x2C.right.x = rw(); e.x2C.right.y = rw();
                e.x2C.left.x = rw(); e.x2C.left.y = rw();
                let f2 = f2();
                f2.cur_pos.y = rw(); f2.mv.co.entry.x28 = rw(); f2.set_x221F_b3((rnd() & 1) as _);
                let ent1 = if rnd() & 1 != 0 { GO2 } else { core::ptr::null_mut() };
                for s in 0..6 {
                    PLAYER_SLOTS[s] = core::mem::zeroed();
                    PLAYER_SLOTS[s].transformed = [0, 1];
                    PLAYER_SLOTS[s].player_entity = [GO2, ent1];
                    PLAYER_SLOTS[s].unk4C = u4c;
                    PLAYER_SLOTS[s].flags.set_b4(bit4);
                }
                let g = go();
                match op {
                    0 => m::ftCo_800C61B0(g),
                    1 => m::ftCo_Entry_Anim(g),
                    2 => m::ftCo_800C6408(g),
                    3 => m::ftCo_EntryStart_Anim(g),
                    4 => m::ftCo_EntryStart_Phys(g),
                    5 => m::ftCo_EntryStart_Coll(g),
                    6 => m::fn_800C69F4(g),
                    7 => m::HSD_JObjSetMtxDirty(if rnd() & 3 != 0 { J } else { core::ptr::null_mut() }),
                    8 => m::ftCo_800C6B6C(g),
                    9 => m::ftCo_EntryEnd_Anim(g),
                    10 => m::ftCo_EntryEnd_Phys(g),
                    11 => m::ftCo_EntryEnd_Coll(g),
                    12 => m::fn_800C6F34(g),
                    13 => { m::ftCo_Entry_IASA(g); m::ftCo_Entry_Phys(g); m::ftCo_Entry_Coll(g); }
                    14 => m::ftCo_EntryStart_IASA(g),
                    15 => m::ftCo_EntryEnd_IASA(g),
                    _ => unreachable!(),
                }
                let f = &*F;
                let f2 = &*F2;
                hv(&f.cur_pos); hv(&f2.cur_pos);
                hm(f.invisible() as u32); hm(f.x2219_b1() as u32); hm(f.x221E_b1() as u32); hm(f.x221E_b2() as u32); hm(f.x221F_b1() as u32);
                let cb = f.accessory1_cb.map(|x| x as usize).unwrap_or(0);
                hm(if cb == m::fn_800C69F4_cb as usize { 1 } else if cb == m::fn_800C6F34_cb as usize { 2 } else if cb != 0 { 3 } else { 0 });
                let e = &f.mv.co.entry;
                hm(e.timer as u32); hm(b(e.x4)); hv(&e.x8); hv(&e.x14);
                hm(b(e.x20)); hm(b(e.x24)); hm(b(e.x28));
                hm(b(e.x2C.top)); hm(b(e.x2C.bottom)); hm(b(e.x2C.right.x));
                hm(b(e.x2C.right.y)); hm(b(e.x2C.left.x)); hm(b(e.x2C.left.y));
                hm(b(f.mv.co.common.x4.x));
                hm(ep(f.x20A0_accessory));
                hj(&*J);
                hj(&*A);
                c.line(op, k, 0, HH);
            }
        }
    }
    assert_eq!(c.done(), 16 * 1500);
}
