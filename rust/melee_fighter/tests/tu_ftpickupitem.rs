//! src/tu/ftpickupitem.rs vs the C (gen/cref/tu/ftpickupitem.c: the whole TU with TARGET_PC,
//! shipping contraction; regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftpickupitem`).
#![allow(static_mut_refs, non_snake_case, unused_unsafe)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftpickupitem as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut IG: *mut [HSD_GObj; 3] = core::ptr::null_mut();
static mut PLHEAD: *mut HSD_GObj = core::ptr::null_mut();
static mut D6524_X8: f32 = 0.0;
static mut FD: *mut ftData = core::ptr::null_mut();
static mut FX8: *mut ftData_x8 = core::ptr::null_mut();
static mut SFX: *mut FtSFX = core::ptr::null_mut();

const KINDS: [i32; 13] = [
    It_Kind_Heart as i32, It_Kind_Tomato as i32, It_Kind_Foods as i32, It_Kind_Lucky_Egg as i32,
    It_Kind_WhispyHealApple as i32, It_Kind_WStar as i32, It_Kind_Hammer as i32, It_Kind_RabbitC as i32,
    It_Kind_MetalB as i32, It_Kind_Spycloak as i32, It_Kind_Coin as i32, 0, 50,
];

fn ig(i: usize) -> *mut HSD_GObj { unsafe { &mut (*IG)[i] as *mut HSD_GObj } }
fn iid(g: *mut HSD_GObj) -> u32 {
    for i in 0..3 { if g == ig(i) { return 1 + i as u32; } }
    if g == go() { return 10; }
    if g.is_null() { 0 } else { 99 }
}
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == f() as *const Fighter) as u32 }
fn r10() -> f32 { rg() * 10.0 / 3.0 }
fn q(v: &mut Quaternion) { v.x = r10(); v.y = r10(); v.z = r10(); v.w = r10(); }
unsafe extern "C" fn pickup_ext(g: *mut HSD_GObj, a: i32) { t(25); t(iid(g)); t(a as u32) }

#[test]
fn ftpickupitem_matches_c() {
    init();
    unsafe {
        IG = Box::leak(Box::new([HSD_GObj::zeroed(), HSD_GObj::zeroed(), HSD_GObj::zeroed()]));
        FD = Box::leak(Box::new(ftData::zeroed()));
        FX8 = Box::leak(Box::new(ftData_x8::zeroed()));
        SFX = Box::leak(Box::new(FtSFX::zeroed()));

        hk::Item_IsGrabbable = Some(|i| { t(1); t(iid(i)); rnd() % 4 != 0 });
        hk::it_8026B47C = Some(|i| { t(2); t(iid(i)); (rnd() % 3) as i32 });
        hk::it_8026B4F0 = Some(|i| { t(3); t(iid(i)); rnd() & 1 != 0 });
        hk::itIsHeavy = Some(|i| { t(4); t(iid(i)); rnd() % 3 == 0 });
        hk::it_8026B344 = Some(|i, p| { t(5); t(iid(i)); p.x = rg() * 10.0 / 3.0; p.y = rg() * 10.0 / 7.0; p.z = rg(); });
        hk::itGetGrabRangeX = Some(|i| { t(6); t(iid(i)); rg() * 5.0 });
        hk::itGetGrabRangeY = Some(|i| { t(7); t(iid(i)); rg() * 5.0 });
        hk::ftCo_IsCpuControlled = Some(|fr| { t(8); t(isf(fr)); rnd() & 1 != 0 });
        hk::HSD_GObjPLinkHead = Some(|l| { assert_eq!(l, HSD_GOBJ_PLINK_ITEM); PLHEAD });
        hk::Geno_ItemPressDist = Some(|i, fr| {
            t(9); t(iid(i)); t(isf(fr));
            match rnd() % 4 { 0 => rg() * 100.0, 1 => rg() * 100000.0, _ => -1.0 }
        });
        hk::it_8026B30C = Some(|i| { t(10); t(iid(i)); if rnd() % 3 != 0 { 5 } else { (rnd() % 7) as i32 } });
        hk::Geno_ItemFamilyEvent = Some(|i, a, p| { t(11); t(iid(i)); t(a as u32); t(p as u32) });
        hk::itGetKind = Some(|i| { t(12); t(iid(i)); KINDS[(rnd() % 13) as usize] });
        hk::Fighter_8006CF5C = Some(|fr, v| { t(13); t(isf(fr)); t(v as u32) });
        hk::Item_8026A8EC = Some(|i| { t(14); t(iid(i)) });
        hk::ftCo_800C4724 = Some(|g| { t(15); t(iid(g)) });
        hk::ftCo_800C52F4 = Some(|g| { t(16); t(iid(g)) });
        hk::ftCommon_8007FA58 = Some(|g, i| { t(17); t(iid(g)); t(iid(i)) });
        hk::ftLib_ApplyMetalBox = Some(|g, i| { t(18); t(iid(g)); t(iid(i)) });
        hk::ftCo_800C88D4 = Some(|g, v, bb| { t(19); t(iid(g)); t(v as u32); t(bb as u32) });
        hk::ftCo_CalcYScaledKnockback = Some(|a, bb, c| { t(20); t(b(a)); t(b(bb)); t(b(c)); rw() });
        hk::Fighter_804D6524_x8 = Some(|| D6524_X8);
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(21); t(iid(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| {
            t(22); t(iid(g));
            if rnd() % 3 == 0 { f().anon_2210.anon_0.set_throw_flags_b3(1); }
        });
        hk::ftCo_800974C4 = Some(|_| t(23));
        hk::pl_8003E17C = Some(|s, sub, i| { t(24); t(s as u32); t(sub as u32); t(iid(i)) });
        hk::ftData_OnItemPickupExt = Some(|k| if k == 0 || k == 2 { Some(pickup_ext as unsafe extern "C" fn(*mut HSD_GObj, i32)) } else { None });
        hk::Mex_OnItemPickupDispatch = Some(|k, g, i, v| { t(26); t(k as u32); t(iid(g as _)); t(iid(i as _)); t(v.is_null() as u32) });
        hk::pl_8003E854 = Some(|s, sub, i| { t(27); t(s as u32); t(sub as u32); t(iid(i)) });
        hk::Item_8026AB54 = Some(|i, o, p| { t(28); t(iid(i)); t(iid(o)); t(p as u32) });
        hk::ft_800881D8 = Some(|fr, id, v, p| { t(29); t(isf(fr)); t(id as u32); t(v as u32); t(p as u32) });
        hk::Geno_ItemPressCollect = Some(|i, p| { t(30); t(iid(i)); t(p as u32) });
        hk::ftAnim_IsFramesRemaining = Some(|g| { t(31); t(iid(g)); sr(31) });
        hk::ftDk_MS_341_800DF980 = Some(|g| { t(32); t(iid(g)) });
        hk::ftCo_80096D9C = Some(|g| { t(33); t(iid(g)) });
        hk::ft_8008A2BC = Some(|g| { t(34); t(iid(g)) });
        hk::ft_80084F3C = Some(|g| { t(35); t(iid(g)) });
        hk::ft_800841B8 = Some(|g, cb| { t(36); t(iid(g)); t((cb as usize == m::ftpickupitem_80094D90 as usize) as u32) });
        hk::Item_8026ABD8 = Some(|i, p, x| { t(37); t(iid(i)); t(b(p.x)); t(b(p.y)); t(b(p.z)); t(b(x)) });
        hk::ftCo_800C5284 = Some(|g| { t(38); t(iid(g)) });
        hk::ftCo_8009750C = Some(|g| { t(39); t(iid(g)) });
        hk::ftCo_Fall_Enter = Some(|g| { t(40); t(iid(g)) });
    }
    let mut c = Check::new(include_str!("data/tu_ftpickupitem.txt"));
    for op in 0..15u32 {
        for k in 0..2000u32 { unsafe {
            case(op, k);
            for i in 0..3 { (*IG)[i] = HSD_GObj::zeroed(); }
            *FD = ftData::zeroed(); *FX8 = ftData_x8::zeroed(); *SFX = FtSFX::zeroed();
            (*FD).x8 = FX8; (*FD).x4C_sfx = SFX;
            f().ft_data = FD;
            let n = (rnd() % 4) as usize;
            for i in 0..n { (*IG)[i].next = if i + 1 < n { ig(i + 1) } else { core::ptr::null_mut() }; }
            PLHEAD = if n != 0 { ig(0) } else { core::ptr::null_mut() };
            D6524_X8 = rw();
            (*FX8).x10 = (rnd() % 8) as u8; (*FX8).x11 = (rnd() % 8) as u8;
            (*SFX).x2C = (rnd() % 100000) as i32;
            cd().x7CC = (rnd() % 100) as i32;
            let fr = f();
            fr.kind = (rnd() % 4) as _;
            fr.player_id = (rnd() % 4) as u8;
            fr.set_is_sub_fighter((rnd() & 1) as u8);
            fr.item_gobj = if rnd() & 1 != 0 { ig(1) } else { core::ptr::null_mut() };
            fr.x1978 = if rnd() & 1 != 0 { ig(2) } else { core::ptr::null_mut() };
            fr.ground_or_air = (rnd() & 1) as _;
            fr.facing_dir = rg();
            fr.cur_pos.x = r10(); fr.cur_pos.y = r10();
            q(&mut fr.x294_itPickup.gr_light_offset); q(&mut fr.x294_itPickup.gr_heavy_offset); q(&mut fr.x294_itPickup.air_light_offset);
            fr.set_x2222_b4((rnd() & 1) as u8); fr.set_x2222_b0((rnd() & 1) as u8);
            fr.cpu.kind = (if rnd() & 1 != 0 { 28 } else { 3 }) as _;
            fr.motion_id = (if rnd() & 1 != 0 { ftCo_MS_LightGet } else { ftCo_MS_HeavyGet }) as _;
            fr.x34_scale.y = rw();
            fr.mv.co.itemget.x0 = (rnd() & 1) as _;
            let tf = (rnd() & 0xFF) as u8;
            {
                let a = &mut fr.anon_2210.anon_0;
                a.set_throw_flags_b0(tf & 1); a.set_throw_flags_b1((tf >> 1) & 1);
                a.set_throw_flags_b2((tf >> 2) & 1); a.set_throw_flags_b3((tf >> 3) & 1);
                a.set_throw_flags_b4((tf >> 4) & 1); a.set_throw_flags_b5((tf >> 5) & 1);
                a.set_throw_flags_b6((tf >> 6) & 1); a.set_throw_flags_b7((tf >> 7) & 1);
            }
            let g = go();
            let mut ret = 0u32;
            match op {
                0 => ret = m::ftpickupitem_80094150(g, ig(0)) as u32,
                1 => ret = iid(m::ftpickupitem_800942A0(g, rnd() % 4)),
                2 => ret = m::ftpickupitem_8009447C(g, ig(0)) as u32,
                3 => {
                    let ms = if rnd() & 1 != 0 { ftCo_MS_LightGet } else { ftCo_MS_HeavyGet } as i32;
                    m::ftpickupitem_80094694(g, ms, rnd() & 1 != 0)
                }
                4 => ret = m::ftpickupitem_80094790(g) as u32,
                5 => m::ftpickupitem_80094818(g, rnd() & 1 != 0),
                6 => m::ftpickupitem_800948A8(g, ig(0)),
                7 => m::ftpickupitem_Anim(g),
                8 => m::ftpickupitem_IASA(g),
                9 => m::ftpickupitem_Phys(g),
                10 => m::ftpickupitem_Coll(g),
                11 => m::ftpickupitem_80094B6C(g, ig(0)),
                12 => m::ftpickupitem_80094D90(g),
                13 => m::ftpickupitem_80094DF8(g),
                _ => m::geno_consumable_collect(ig(0), (rnd() % 4) as i32),
            }
            let fr = f();
            let a = &fr.anon_2210.anon_0;
            let mut h = Hs::new();
            h.m(iid(fr.item_gobj)); h.m(iid(fr.x1978));
            h.m(a.throw_flags_b0() as u32); h.m(a.throw_flags_b1() as u32); h.m(a.throw_flags_b2() as u32); h.m(a.throw_flags_b3() as u32);
            h.m(a.throw_flags_b4() as u32); h.m(a.throw_flags_b5() as u32); h.m(a.throw_flags_b6() as u32); h.m(a.throw_flags_b7() as u32);
            h.m(fr.mv.co.itemget.x0 as u32);
            h.m(match fr.take_dmg_cb {
                None => 0,
                Some(p) if p as usize == m::ftCo_800974C4_cb as usize => 1,
                Some(p) if p as usize == m::ftpickupitem_80094DF8_cb as usize => 2,
                _ => 3,
            });
            c.line(op, k, ret, h.0);
        } }
    }
    assert_eq!(c.done(), 30000);
}
