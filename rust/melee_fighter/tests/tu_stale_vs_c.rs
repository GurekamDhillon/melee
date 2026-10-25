//! The C-named ports (tu::plstale, tu::plattack, tu::ft_0881, tu::player) vs plstale.c / plattack.c / ft_0881.c staling (gen/cref/stale_ref.c; expected
//! output tests/data/stale.txt, regenerate with `python rust/melee_fighter/gen/cref/run.py stale`).
#![allow(static_mut_refs)]

use melee_fighter::stale as st;
use melee_fighter::tu::{ft_0881 as f81, plattack as pa, plstale as ps};
use melee_fighter::types::{Fighter, HSD_GObj, Item};

struct Rng(u32);
impl Rng {
    fn rnd(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rf(&mut self) -> f32 {
        (self.rnd() % 2001) as i32 as f32 / 9000.0
    }
}

#[test]
fn staling_matches_c() {
    let mut lines = include_str!("data/stale.txt").lines();
    let mut dec = [0f32; 9];
    let mut n = 0;
    for k in 0..300u32 {
        unsafe {
            st::PLAYER_SLOTS = core::mem::zeroed();
        }
        let mut fp: Box<Fighter> = Box::new(Fighter::zeroed());
        let mut it: Box<Item> = unsafe { Box::new(core::mem::zeroed()) };
        let mut fg: Box<HSD_GObj> = unsafe { Box::new(core::mem::zeroed()) };
        let mut ig: Box<HSD_GObj> = unsafe { Box::new(core::mem::zeroed()) };
        fg.user_data = &mut *fp as *mut Fighter as *mut core::ffi::c_void;
        ig.user_data = &mut *it as *mut Item as *mut core::ffi::c_void;
        let (fgp, igp): (*mut HSD_GObj, *mut HSD_GObj) = (&mut *fg, &mut *ig);
        let mut r = Rng(0xC0FFEEu32.wrapping_add(k.wrapping_mul(2654435761)));
        unsafe {
            st::STALE_ATTACK_INSTANCE = r.rnd() as u16;
            st::ATTACK_COUNTER = r.rnd() as u16;
        }
        for d in dec.iter_mut() {
            *d = r.rf();
        }
        unsafe {
            st::STALE_DECREMENTS = dec.as_ptr();
            st::DB_LEVEL = if r.rnd() % 8 == 0 { 3 } else { 0 };
        }
        for step in 0..40u32 {
            let slot = (r.rnd() % 6) as i32;
            let op = r.rnd() % 11;
            let mut ret: u32 = 0;
            fp.player_id = slot as u8;
            match op {
                0 => ps::plStale_InitAttackInstance(),
                1 => ps::plStale_ResetStaleMoveTableForPlayer(slot),
                2 => ret = ps::plStale_IncrementAttackInstance() as u32,
                3 => {
                    let same = r.rnd() % 4 == 0;
                    fp.x2068_attackID = (r.rnd() % 8) as i32;
                    fp.x206C_attack_instance = (r.rnd() % 5) as u16;
                    unsafe { ps::plStale_UpdateStaleMovesFromFighter(fgp, if same { fgp } else { core::ptr::null_mut() }) };
                }
                4 => {
                    let is_fighter = r.rnd() & 1 != 0;
                    let id = (r.rnd() % 8) as i32;
                    let inst = (r.rnd() % 5) as u16;
                    fg.classifier = if is_fighter { 4 } else { 0 };
                    it.owner = fgp;
                    it.xD88_attackID = id;
                    it.xD8C_attack_instance = inst;
                    unsafe { ps::plStale_UpdateStaleMovesFromItem(igp, core::ptr::null_mut()) };
                }
                5 => {
                    let id = r.rnd() % 8;
                    f81::ft_800890D0(&mut fp, id as i32)
                }
                6 => f81::ft_800890BC(&mut fp),
                7 => f81::ft_800892A0(fgp),
                8 => {
                    let id = (r.rnd() % 8) as i32;
                    let dmg = r.rf() * 30.0;
                    ret = f81::ft_80089228(&fp, id, 0, dmg).to_bits();
                }
                9 => ret = pa::plAttack_80037B08() as u32,
                10 => {
                    let s = st::action_stats(slot);
                    unsafe { core::ptr::write_bytes(s as *mut _ as *mut u8, 0xFF, core::mem::size_of_val(s)) };
                    pa::plAttack_8003759C(slot as u32);
                    let s = st::action_stats(slot);
                    ret = s
                        .attacks
                        .total
                        .wrapping_add(s.hits.by_attack_counts[5])
                        .wrapping_add(unsafe { s.anon_358.x358_hits.thrown_item_count })
                        .wrapping_add(s.x504[3] as u32)
                        .wrapping_add(s.x598[7])
                        .wrapping_add(s.x5B8[1] as u32)
                        .wrapping_add(
                            ((s.x5BC_b0() as u32) << 3) | ((s.x5BC_b1() as u32) << 2) | ((s.x5BC_b2() as u32) << 1) | s.x5BC_b3() as u32,
                        );
                }
                _ => unreachable!(),
            }
            let t = st::stale_table(slot);
            let mut got = format!("{k} {step} {op} {slot} {ret:08X} {}", t.current_index);
            for m in t.StaleMoves.iter() {
                got += &format!(" {}:{}", m.move_id, m.attack_instance);
            }
            unsafe {
                got += &format!(" {} {} {} {}", st::STALE_ATTACK_INSTANCE, st::ATTACK_COUNTER, fp.x2068_attackID, fp.x206C_attack_instance);
            }
            assert_eq!(got, lines.next().expect("C reference too short"), "case {k} step {step}");
            n += 1;
        }
    }
    assert_eq!(n, 12000);
}

#[test]
fn plattack_80037590_resets_counter() {
    unsafe { st::ATTACK_COUNTER = 0x1234 };
    pa::plAttack_80037590();
    assert_eq!(unsafe { st::ATTACK_COUNTER }, 1); // `unk_804D6480 = 1;` in plattack.c
}
