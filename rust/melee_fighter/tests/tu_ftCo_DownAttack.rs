//! src/tu/ftCo_DownAttack.rs vs the C (gen/cref/tu/ftCo_DownAttack.c, shipping contraction;
//! regenerate with `python rust/melee_fighter/gen/cref/tu/run_tu.py ftCo_DownAttack`).
#![allow(static_mut_refs, non_snake_case)]
mod tu_ftco_harness;
use tu_ftco_harness::*;
use melee_fighter::tu::ftCo_DownAttack as m;
use m::ext::hook as hk;
use melee_fighter::types::*;

static mut PLH: G0 = core::ptr::null_mut();
type G0 = *mut HSD_GObj;
fn isf(fr: &Fighter) -> u32 { (fr as *const Fighter == unsafe { F } as *const Fighter) as u32 }

#[test]
fn ftCo_DownAttack_matches_c() {
    init();
    unsafe {
        hk::ftCo_800DF644 = Some(|fr| { t(3); t(isf(fr)); sr(2) });
        hk::Fighter_ChangeMotionState = Some(|g, ms, fl, a, bb, c, o| {
            t(4); t(is_go(g)); t(ms as u32); t(fl); t(b(a)); t(b(bb)); t(b(c)); t(o.is_null() as u32)
        });
        hk::ftAnim_8006EBA4 = Some(|g| { t(5); t(is_go(g)) });
        hk::ftAnim_IsFramesRemaining = Some(|_| { t(7); sr(3) });
        hk::ft_8008A2BC = Some(|g| { t(8); t(is_go(g)) });
        hk::ft_80084F3C = Some(|g| { t(9); t(is_go(g)) });
        hk::ft_80084104 = Some(|g| { t(10); t(is_go(g)) });
        hk::HSD_GObjPLinkHead = Some(|l| { assert_eq!(l, HSD_GOBJ_PLINK_FIGHTER as i32); PLH });
        hk::ftCo_800C5240 = Some(|g| { t(12); t(is_go(g)); sr(4) });
        hk::ftCo_800987D0 = Some(|g| { t(13); t(is_go(g)) });
    }
    let ms = [ftCo_MS_DownWaitU, ftCo_MS_DownWaitD, 0];
    let mut trk: Box<Item_FtTrack> = Box::new(Item_FtTrack::zeroed());
    let mut c = Check::new(include_str!("data/tu_ftCo_DownAttack.txt"));
    for op in 0..9u32 {
        for k in 0..2000u32 {
            case(op, k);
            *trk = Item_FtTrack::zeroed();
            let d = cd();
            d.x250 = rw(); d.x1C = (rnd() % 20) as i32;
            let fr = f();
            fr.input.pressed_buttons = rnd() & 0x3FF;
            fr.motion_id = ms[(rnd() % 3) as usize] as _;
            fr.x680 = (rnd() % 20) as u8; fr.x684 = (rnd() % 20) as u8;
            fr.cur_pos.x = rw(); fr.cur_pos.y = rw(); fr.cur_pos.z = rw();
            let f2 = f2();
            f2.cur_pos.x = rw(); f2.cur_pos.y = rw(); f2.cur_pos.z = rw();
            unsafe {
                match rnd() % 3 {
                    0 => PLH = core::ptr::null_mut(),
                    1 => { PLH = go(); (*go()).next = core::ptr::null_mut(); }
                    _ => { PLH = go2(); (*go2()).next = go(); (*go()).next = core::ptr::null_mut(); }
                }
            }
            let g = go();
            let ret = match op {
                0 => m::ftCo_800984D4(g) as u32,
                1 => { m::ftCo_8009856C(g, (rnd() % 400) as i32); 0 }
                2 => { m::ftCo_DownAttack_Anim(g); 0 }
                3 => { m::ftCo_DownAttack_IASA(g); 0 }
                4 => { m::ftCo_DownAttack_Phys(g); 0 }
                5 => { m::ftCo_DownAttack_Coll(g); 0 }
                6 => { m::ftCo_80098634(&mut trk); 0 }
                7 => m::ftCo_800986B0(g) as u32,
                _ => m::ftCo_8009872C(g) as u32,
            };
            let mut h = Hs::new();
            h.m(f().motion_id as u32);
            h.m(trk.count as u32);
            for i in 0..11 {
                let e = &trk.ecb_offset_arr[i];
                h.m(b(e.top)); h.m(b(e.bottom)); h.m(b(e.right)); h.m(b(e.left));
                let p = &trk.ft_pos_arr[i];
                h.m(b(p.x)); h.m(b(p.y)); h.m(b(p.z));
            }
            c.line(op, k, ret, h.0);
        }
    }
    assert_eq!(c.done(), 18000);
}
