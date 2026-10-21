//! efLib_SetFlags / PauseAll / ResumeAll (ef::eflib) against the C
//! (cref/gen_trivial_r3_ref.sh -> tests/data/eflib_r3_ref.txt): 3000 random list pairs.
use melee_types::{EF_Effect, HSD_GObj};
use melee_world::ef::eflib::*;

const DATA: &str = include_str!("data/eflib_r3_ref.txt");

fn h(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

#[test]
fn eflib_r3_match_c() {
    let mut parents: Vec<HSD_GObj> = (0..3).map(|_| HSD_GObj::zeroed()).collect();
    let pp: Vec<*mut HSD_GObj> = parents.iter_mut().map(|p| p as *mut _).collect();
    let mut ops = [0; 3];
    for line in DATA.lines() {
        let mut it = line.split(" | ");
        let head: Vec<u32> = it.next().unwrap()[2..].split_whitespace().map(h).collect();
        let (op, arg, n) = (head[0], head[1] as i32, [head[2] as usize, head[3] as usize]);
        let mut desc = it.next().unwrap().split_whitespace();
        let want: Vec<u32> = it.next().unwrap_or("").split_whitespace().map(h).collect();
        let mut nodes: Vec<Vec<HSD_GObj>> = (0..2).map(|l| (0..n[l]).map(|_| HSD_GObj::zeroed()).collect()).collect();
        let mut effs: Vec<Vec<EF_Effect>> = (0..2).map(|l| (0..n[l]).map(|_| EF_Effect::zeroed()).collect()).collect();
        let mut has = vec![vec![false; 8]; 2];
        let mut heads = [core::ptr::null_mut::<HSD_GObj>(); 16];
        for l in 0..2 {
            for i in 0..n[l] {
                let p = desc.next().unwrap();
                if p != "n" {
                    has[l][i] = true;
                    let e = &mut effs[l][i];
                    e.parent_gobj = pp[h(p) as usize];
                    e.expire_flags = h(desc.next().unwrap()) as u8 as i8;
                    e.state_flags = h(desc.next().unwrap()) as u8;
                    nodes[l][i].user_data = e as *mut EF_Effect as *mut _;
                }
            }
            for i in (0..n[l]).rev() {
                let next = if i + 1 < n[l] { &mut nodes[l][i + 1] as *mut HSD_GObj } else { core::ptr::null_mut() };
                nodes[l][i].next = next;
            }
            heads[HSD_GOBJ_PLINK_EFFECT_UNK0 + l] = if n[l] > 0 { &mut nodes[l][0] } else { core::ptr::null_mut() };
        }
        unsafe {
            match op {
                0 => eflib_set_flags(&heads, pp[0], arg),
                1 => eflib_pause_all(&heads, pp[0]),
                _ => eflib_resume_all(&heads, pp[0]),
            }
        }
        let mut got = vec![];
        for l in 0..2 {
            for i in 0..n[l] {
                if has[l][i] {
                    got.push(effs[l][i].expire_flags as u8 as u32);
                    got.push(effs[l][i].state_flags as u32);
                }
            }
        }
        assert_eq!(got, want, "{line}");
        ops[op as usize] += 1;
    }
    assert!(ops.iter().all(|&c| c > 500), "{ops:?}");
}
