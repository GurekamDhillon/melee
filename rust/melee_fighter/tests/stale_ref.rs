//! stale.rs vs plstale.c / plattack.c / ft_0881.c staling (gen/cref/stale_ref.c; expected
//! output tests/data/stale.txt, regenerate with `python rust/melee_fighter/gen/cref/run.py stale`).
#![allow(static_mut_refs)]

use melee_fighter::stale as st;
use melee_fighter::types::Fighter;

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
                0 => st::init_attack_instance(),
                1 => st::reset_stale_move_table(slot),
                2 => ret = st::increment_attack_instance() as u32,
                3 => {
                    let same = r.rnd() % 4 == 0;
                    fp.x2068_attackID = (r.rnd() % 8) as i32;
                    fp.x206C_attack_instance = (r.rnd() % 5) as u16;
                    st::update_stale_moves_from_fighter(&fp, same);
                }
                4 => {
                    let is_fighter = r.rnd() & 1 != 0;
                    let id = (r.rnd() % 8) as i32;
                    let inst = (r.rnd() % 5) as u16;
                    st::update_stale_moves_from_item(if is_fighter { Some(&*fp) } else { None }, id, inst);
                }
                5 => {
                    let id = r.rnd() % 8;
                    st::new_attack(&mut fp, id)
                }
                6 => st::clear_attack_id(&mut fp),
                7 => st::renew_attack_instance(&mut fp),
                8 => {
                    let id = (r.rnd() % 8) as i32;
                    let dmg = r.rf() * 30.0;
                    ret = st::apply_staling(&fp, id, 0, dmg).to_bits();
                }
                9 => ret = st::attack_counter_next() as u32,
                10 => {
                    let s = st::action_stats(slot);
                    unsafe { core::ptr::write_bytes(s as *mut _ as *mut u8, 0xFF, core::mem::size_of_val(s)) };
                    st::reset_action_stats(slot as u32);
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
