//! src/tu/ftCo_Attack100.rs vs the C (gen/cref/tu/ftCo_Attack100.c: the whole TU, shipping
//! contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_Attack100`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
#[macro_use]
mod tu_ftco_attack;
use tu_ftco_harness::*;
use tu_ftco_attack::*;
use melee_fighter::tu::ftCo_Attack100 as m;
use melee_fighter::types::*;
use m::ext::hook as hk;

unsafe extern "C" fn vanilla_fn(_: *mut HSD_GObj) { t(99) }
fn cb_fn(g: *mut HSD_GObj) { t(98); t(is_go(g)) }
fn tab(bit: u32) -> HSD_GObjEvent { if srv(1) & bit != 0 { Some(vanilla_fn) } else { None } }
fn van(v: HSD_GObjEvent) -> u32 { match v { None => 0, Some(p) if p as usize == vanilla_fn as usize => 1, _ => 9 } }
fn item(sel: bool) -> *mut Item_GObj { (if sel { igo() } else { igo2() }) as *mut Item_GObj }

#[test]
fn ftCo_Attack100_matches_c() {
    init();
    af_init();
    unsafe {
        hk::ScriptGame_CapsActionAllowed = Some(|fr, fl| { t(1); t((fr as *const Fighter == F as *const Fighter) as u32); t(fl as u32); (chk(1) | chk(7)) as i32 });
        hk::ftData_SpecialN = Some(|_| tab(1));
        hk::ftData_SpecialLw = Some(|_| tab(2));
        hk::ftData_SpecialHi = Some(|_| tab(4));
        hk::ftData_SpecialAirHi = Some(|_| tab(8));
        hk::Geno_SpecialEnter = Some(|g, w| { t(2); t(is_go(g)); t(w as u32); chk(2) as i32 });
        hk::Mex_SpecialNDispatch = Some(|k, g, v| { t(3); t(k as u32); t(is_go(g)); t(van(v)) });
        hk::Mex_SpecialLwDispatch = Some(|k, g, v| { t(4); t(k as u32); t(is_go(g)); t(van(v)) });
        hk::Mex_SpecialHiDispatch = Some(|k, g, v| { t(5); t(k as u32); t(is_go(g)); t(van(v)) });
        hk::Mex_SpecialHiAirDispatch = Some(|k, g, v| { t(6); t(k as u32); t(is_go(g)); t(van(v)) });
        hk::ftGw_Attack100Start_Enter = vrec!(8);
        hk::ftpickupitem_80094790 = chkrec!(9);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(10); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = vrec!(11);
        hk::ftAnim_IsFramesRemaining = chkrec!(12);
        hk::ft_800892A0 = vrec!(13);
        hk::ft_80089824 = vrec!(14);
        hk::ft_8008A2BC = vrec!(15);
        hk::ft_80084FA8 = vrec!(16);
        hk::ft_80084104 = vrec!(17);
        hk::Geno_ItemPressDist = Some(|i, fr| {
            t(18); t(af_item(i)); t((fr as *const Fighter == F as *const Fighter) as u32);
            if sr(5) { if sr(21) { 0.0 } else { 3.5 } } else { -1.0 }
        });
        hk::Geno_ItemPressCollect = Some(|i, p| { t(19); t(af_item(i)); t(p as u32) });
        hk::ftpickupitem_80094818 = Some(|g, bb| { t(20); t(is_go(g)); t(bb as u32) });
        hk::itGetKind = Some(|i| { t(21); t(af_item(i)); if sr(6) { It_Kind_WStar as i32 } else { (srv(7) % 50) as i32 } });
        hk::pl_8003E854 = Some(|a, bb, i| { t(22); t(a as u32); t(bb as u32); t(af_item(i)) });
        hk::Item_8026AB54 = Some(|i, o, p| { t(23); t(af_item(i)); t(is_go(o)); t(p as u32) });
        hk::ftpickupitem_8009447C = Some(|g, i| { t(24); t(is_go(g)); t(af_item(i)); chk(24) });
        hk::efAsync_Spawn = Some(|g, q, k, id, j| {
            t(25); t(is_go(g)); t((q == &mut f().x60C as *mut _ as *mut core::ffi::c_void) as u32); t(k); t(id); t(af_jobj(j))
        });
        hk::ftpickupitem_800942A0 = Some(|g, fl| {
            t(26); t(is_go(g)); t(fl);
            if sr(8) { core::ptr::null_mut() } else { item(sr(24)) }
        });
        hk::ftKb_MexKirbyLike = Some(|k| { t(27); t(k as u32); chk(27) as i32 });
        hk::ftKb_SpecialN_800F1CD8 = Some(|g| { t(28); t(is_go(g)); (srv(9) % 7) as i32 });
    }
    let mut c = Check::new(include_str!("data/tu_ftCo_Attack100.txt"));
    for op in 0..26u32 {
        for k in 0..2000u32 {
            case(op, k);
            af_fill();
            let ms = (rnd() % 400) as i32;
            let g = go();
            let ret = match op {
                0 => m::ftCo_800D6824(g) as u32,
                1 => m::ftCo_800D688C(f()) as u32,
                2 => m::ftCo_800D68C0(g) as u32,
                3 => m::ftCo_800D6928(f()) as u32,
                4 => m::ftCo_Attack100_CheckInput(g) as u32,
                5 => m::ftCo_800D69C4(g) as u32,
                6 => m::ftCo_Attack_800D6A50(g) as u32,
                7 => { m::fn_800D6AC4(g); 0 }
                8 => { m::ftCo_800D6B00(g, ms); 0 }
                9 => { m::fn_800D6B8C(g); 0 }
                10 => { m::ftCo_Attack100Start_Anim(g); 0 }
                11 => { m::ftCo_800D6C60(g, cb_fn); 0 }
                12 => { m::ftCo_Attack100Loop_Anim(g); 0 }
                13 => { m::ftCo_Attack100End_Anim(g); 0 }
                14 => { m::ftCo_Attack100Loop_IASA(g); 0 }
                15 => { m::ftCo_Attack100Start_Phys(g); 0 }
                16 => { m::ftCo_Attack100Loop_Phys(g); 0 }
                17 => { m::ftCo_Attack100End_Phys(g); 0 }
                18 => { m::ftCo_Attack100Start_Coll(g); 0 }
                19 => { m::ftCo_Attack100Loop_Coll(g); 0 }
                20 => { m::ftCo_Attack100End_Coll(g); 0 }
                21 => { m::fn_800D6F58(g, item(ms & 1 != 0)); 0 }
                22 => m::ftCo_800D705C(g) as u32,
                23 => m::ftCo_800D7100(g) as u32,
                24 => { m::ftCo_800D71D8(g); 0 }
                _ => m::ftCo_800D7268(f()) as u32,
            };
            c.line(op, k, ret, af_hash());
        }
    }
    assert_eq!(c.done(), 52000);
}
