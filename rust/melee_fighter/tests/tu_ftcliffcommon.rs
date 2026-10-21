//! src/tu/ftcliffcommon.rs vs the C (gen/cref/tu/ftcliffcommon.c, shipping contraction),
//! 8000 cases over CliffCatch Phys / Coll / Anim / IASA with recording stubs.
#![allow(static_mut_refs)]
mod tu_s25;
use melee_fighter::hooks;
use melee_fighter::tu::ftcliffcommon as c;
use melee_fighter::types::*;
use tu_s25::*;

static mut LV: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
static mut SRV: [u32; 16] = [0; 16];
static mut GOP: usize = 0;
fn sr(i: usize) -> bool { unsafe { SRV[i] & 1 != 0 } }

fn m_ed8(id: i32) -> bool { t(1); t(id as u32); sr(0) }
fn m_ecc(id: i32, v: &mut Vec3) { t(2); t(id as u32); unsafe { *v = LV } }
fn m_da4(id: i32, v: &mut Vec3) { t(3); t(id as u32); unsafe { LV.x += 1.0; *v = LV } }
fn fall(g: *mut HSD_GObj) { t(4); t((g as usize == unsafe { GOP }) as u32); }
fn ifr(_g: *mut HSD_GObj) -> bool { t(5); sr(1) }
fn a804(_g: *mut HSD_GObj) { t(6); }
fn b1dc(_g: *mut HSD_GObj) -> bool { t(7); sr(2) }
fn b1c(_g: *mut HSD_GObj) { t(8); }
fn ef68(_g: *mut HSD_GObj) -> bool { t(9); sr(3) }

#[test]
fn ftcliffcommon_vs_c() {
    let exp = expected("tu_ftcliffcommon");
    let mut cd: Box<ftCommonData> = zeroed();
    unsafe {
        hooks::P_FT_COMMON_DATA = &*cd;
        c::ext::MPLIB_80054ED8 = Some(m_ed8);
        c::ext::MPLIB_80053ECC_FLOOR = Some(m_ecc);
        c::ext::MPLIB_80053DA4_FLOOR = Some(m_da4);
        c::ext::FTCO_8009A804 = Some(a804);
        c::ext::FT_800821DC = Some(b1dc);
        c::ext::FT_80082B1C = Some(b1c);
        c::ext::FTCO_8009EF68 = Some(ef68);
        hooks::FTCO_FALL_ENTER = Some(fall);
        hooks::FTANIM_IS_FRAMES_REMAINING = Some(ifr);
    }
    let mut n = 0;
    for op in 0..4u32 {
        for k in 0..2000u32 {
            let mut h = H::case(op, k);
            unsafe { TR = 0; SRV = h.srv; }
            let mut f: Box<Fighter> = zeroed();
            let mut go: Box<HSD_GObj> = zeroed();
            unsafe { *cd = core::mem::zeroed(); hooks::P_FT_COMMON_DATA = &*cd; }
            unsafe { LV.x = h.rw(); LV.y = h.rw(); LV.z = h.rw(); }
            f.x68C_transNPos.x = h.rw(); f.x68C_transNPos.y = h.rw(); f.x68C_transNPos.z = h.rw();
            f.facing_dir = h.rg();
            f.cur_pos.x = h.rw(); f.cur_pos.y = h.rw();
            f.mv.co.cliff.ledge_id = (h.rnd() % 100) as _;
            cd.ledge_cooldown = (h.rnd() % 60) as _;
            f.x2064_ledgeCooldown = (h.rnd() % 60) as _;
            go.user_data = &mut *f as *mut Fighter as *mut _;
            let g: *mut HSD_GObj = &mut *go;
            unsafe { GOP = g as usize };
            match op {
                0 => c::ftco_cliff_catch_phys(g),
                1 => c::ftco_cliff_catch_coll(g),
                2 => c::ftco_cliff_catch_anim(g),
                _ => c::ftco_cliff_catch_iasa(g),
            }
            h.h = 0x811C9DC5;
            h.hv(&f.cur_pos);
            h.hm(f.x2064_ledgeCooldown as u32);
            let got = format!("{} {} {:08X} {:08X}", op, k, h.h, unsafe { TR });
            assert_eq!(got, exp[n], "case {op} {k}");
            n += 1;
        }
    }
    assert_eq!(n, exp.len());
}
