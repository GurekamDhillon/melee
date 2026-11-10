//! src/tu/ftCo_09f4.rs (the fighter light overlay) vs the C (gen/cref/tu/ftCo_09F4.c: the whole TU,
//! shipping contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_09F4`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use core::ffi::c_void;
use melee_fighter::tu::ftCo_09f4 as m;
use melee_fighter::types::*;
use tu_ftco_harness::*;

static mut L0: *mut HSD_LObj = core::ptr::null_mut();
static mut L1: *mut HSD_LObj = core::ptr::null_mut();
static mut LF: *mut HSD_LObj = core::ptr::null_mut();
static mut LS: *mut HSD_LObj = core::ptr::null_mut();
static mut LN: *mut HSD_LObj = core::ptr::null_mut();
static mut CO: *mut HSD_CObj = core::ptr::null_mut();
static mut OV: *mut ColorOverlay = core::ptr::null_mut();
static mut JO: *mut HSD_JObj = core::ptr::null_mut();
static mut LG: *mut HSD_GObj = core::ptr::null_mut();
static mut GRL: [*mut LightList; 2] = [core::ptr::null_mut(); 2];

fn ep<T>(p: *const T) -> u32 {
    let a = p as usize;
    unsafe {
        if a == 0 { 0 }
        else if a == L0 as usize { 1 }
        else if a == L1 as usize { 2 }
        else if a == LF as usize { 3 }
        else if a == LS as usize { 4 }
        else if a == LN as usize { 5 }
        else if a == CO as usize { 6 }
        else if a == LG as usize { 7 }
        else if a == GO as usize { 8 }
        else if a == JO as usize { 9 }
        else if a == (&raw const GRL) as usize { 10 }
        else if a == (&raw const m::node2) as usize { 11 }
        else { 0xFFFF }
    }
}
fn tv(v: &Vec) { t(b(v.x)); t(b(v.y)); t(b(v.z)); }

#[test]
fn ftCo_09f4_matches_c() {
    init();
    unsafe {
        macro_rules! leak { ($($n:ident),*) => { $( $n = Box::leak(Box::new(core::mem::zeroed())); )* } }
        leak!(L0, L1, LF, LS, LN, CO, OV, JO, LG);
        use m::ext::hook as hk;
        hk::GObj_Create = Some(|c, p, pr| { t(1); t(c as u32); t(p as u32); t(pr as u32); LG });
        hk::Ground_801C49B4 = Some(|| { t(2); (&raw mut GRL) as *mut *mut LightList });
        hk::lb_80011AC4 = Some(|l| {
            t(3);
            t(ep(l));
            if l as usize == (&raw mut m::node2) as usize {
                let n1 = *l;
                let d = (*n1).desc;
                let w = (*d).position;
                t(((*n1).desc as usize == (&raw mut m::node0) as usize) as u32);
                t((*n1).anims.is_null() as u32);
                t((*l.add(1)).is_null() as u32);
                t((*d).flags as u32); t((*d).attnflags as u32);
                t((*d).color.r as u32); t((*d).color.g as u32); t((*d).color.b as u32); t((*d).color.a as u32);
                t((*d).class_name.is_null() as u32); t((*d).next.is_null() as u32); t((*d).interest.is_null() as u32);
                // the C reads `floats[0..5]` through the record: class_name word, pos, robjdesc word
                t((*w).class_name as usize as u32); t(b((*w).pos.x)); t(b((*w).pos.y)); t(b((*w).pos.z)); t((*w).robjdesc as usize as u32);
                return LF;
            }
            LS
        });
        hk::lb_8000CDC0 = Some(|l| { t(4); t(ep(l)); LN });
        hk::Ground_801C2374 = Some(|l| { t(5); t(ep(l)); });
        hk::HSD_GObjObject_80390A70 = Some(|g, kind, o| { t(6); t(ep(g)); t(kind as u32); t(ep(o as *const c_void)); });
        hk::GObj_SetupGXLink = Some(|g, cb, gx, prio| { t(7); t(ep(g)); t((cb.map(|f| f as usize) == Some(m::ftCo_8009F54C as usize)) as u32); t(gx as u32); t(prio); });
        hk::HSD_GObj_SetupProc = Some(|g, f, prio| { t(8); t(ep(g)); t((f.map(|f| f as usize) == Some(m::ftCo_8009F480 as usize)) as u32); t(prio as u32); });
        hk::HSD_LObjAnimAll = Some(|l| { t(9); t(ep(l)); });
        hk::HSD_LObjReqAnimAll = Some(|l, f| { t(10); t(ep(l)); t(b(f)); });
        hk::HSD_LObj_803668EC = Some(|l| { t(11); t(ep(l)); });
        hk::HSD_LObjSetupInit = Some(|c| { t(12); t(ep(c)); });
        hk::HSD_CObjGetCurrent = Some(|| { t(13); CO });
        hk::HSD_LObjSetPosition = Some(|l, v| { t(14); t(ep(l)); tv(&*v); });
        hk::HSD_LObjSetColor = Some(|l, c| { t(15); t(ep(l)); t(c.r as u32); t(c.g as u32); t(c.b as u32); t(c.a as u32); });
        hk::HSD_LObjSetFlags = Some(|l, f| { t(16); t(ep(l)); t(f); });
        hk::HSD_LObjClearFlags = Some(|l, f| { t(17); t(ep(l)); t(f); });
        hk::HSD_LObjSetCurrentAll = Some(|l| { t(18); t(ep(l)); });
        hk::HSD_LObjAddCurrent = Some(|l| { t(19); t(ep(l)); });
        hk::ftCo_800C0658 = Some(|fp| { t(20); t((fp as usize == F as usize) as u32); OV });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_09F4.txt"));
    for op in 0..7u32 {
        for k in 0..3000u32 {
            case(op, k);
            unsafe {
                *OV = core::mem::zeroed();
                *JO = core::mem::zeroed();
                m::ext::HSD_GObj_LightKind = (rnd() % 20) as i8;
                m::lobj0 = if rnd() & 1 != 0 { L0 } else { core::ptr::null_mut() };
                m::lobj1 = if rnd() & 1 != 0 { L1 } else { core::ptr::null_mut() };
                (*GO).hsd_obj = JO as *mut c_void;
                (*GO).user_data = F as *mut c_void;
                (*F).gobj = GO;
                (*JO).rotate.y = rw();
                let f = f();
                f.x588 = if rnd() & 1 != 0 { LF } else { LS };
                f.set_x2221_b3((rnd() & 1) as _);
                (*OV).set_x7C_flag2((rnd() & 1) as _);
                (*OV).set_x7C_light_enable((rnd() & 1) as _);
                (*OV).x74_light_rot_x = rw();
                (*OV).x78_light_rot_yz = rw();
                (*OV).x50_light_color.r = rnd() as u8;
                (*OV).x50_light_color.g = rnd() as u8;
                (*OV).x50_light_color.b = rnd() as u8;
                (*OV).x50_light_color.a = rnd() as u8;
                match op {
                    0 => m::ftCo_8009F480(go()),
                    1 => m::ftCo_8009F4A4(),
                    2 => m::ftCo_8009F54C(go(), rnd() as isize),
                    3 => m::ftCo_8009F578(f),
                    4 => m::ftCo_8009F5AC(f),
                    5 => m::ftCo_8009F75C(f, rnd() & 1 != 0),
                    6 => m::ftCo_8009F7F8(f),
                    _ => unreachable!(),
                }
                let mut hh: u32 = 0x811C_9DC5;
                let f = &*F;
                for v in [ep(f.x588), f.x2221_b3() as u32, ep(m::lobj0), ep(m::lobj1)] {
                    hh = hh.wrapping_mul(0x0100_0193) ^ v;
                }
                c.line(op, k, 0, hh);
            }
        }
    }
    assert_eq!(c.done(), 7 * 3000);
}
