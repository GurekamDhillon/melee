//! All seven random.c definitions and random.h's empty-range wrapper, with the
//! native C state and trace after every call. Regenerate: oracle/build_random_tu.sh.
use melee_runtime::tu::random::*;
use std::cell::Cell;

thread_local! { static TRACE: Cell<[u32; 3]> = const { Cell::new([0; 3]) }; }
fn trace(seed: u32, stream: i32) {
    TRACE.with(|t| t.set([t.get()[0] + 1, seed, stream as u32]));
}
fn pointer_id(r: &HsdRand, p: *const u32) -> u32 {
    if p == &r.seed {
        1
    } else if p == &r.render_seed {
        2
    } else {
        3
    }
}

#[test]
fn every_random_definition_matches_c_state_and_return() {
    let mut coverage = [0; 8];
    for (row, line) in include_str!("data/oracle_random_tu.txt")
        .lines()
        .enumerate()
    {
        let v: Vec<u32> = line
            .split_whitespace()
            .map(|s| u32::from_str_radix(s, 16).unwrap())
            .collect();
        assert_eq!(v.len(), 18);
        let mut redirected = v[4];
        let mut r = HsdRand {
            seed: v[1],
            render_seed: v[2],
            render_depth: v[3],
            seed_ptr: if v[5] == 1 {
                SeedSlot::Default
            } else {
                SeedSlot::Redirected(&mut redirected)
            },
            trace: Some(trace),
        };
        TRACE.with(|t| t.set([0; 3]));
        let op = v[0] as usize;
        coverage[op] += 1;
        let rc = match op {
            0 => {
                HSD_RandRenderBegin(&mut r);
                0
            }
            1 => {
                HSD_RandRenderEnd(&mut r);
                0
            }
            2 => {
                let p = HSD_RandCurrentSeed(&mut r);
                pointer_id(&r, p)
            }
            3 => HSD_Rand(&mut r) as u32,
            4 => HSD_Randf(&mut r).to_bits(),
            5 => HSD_Randi(&mut r, v[6] as i32) as u32,
            6 => {
                let p = match r.seed_ptr {
                    SeedSlot::Default => &r.seed as *const u32,
                    SeedSlot::Redirected(p) => p,
                } as usize;
                let (lo, hi) = match v[7] {
                    0 => (p, p + 4),
                    1 => (p, p),
                    2 => (p + 4, p + 8),
                    3 => (p - 4, p),
                    4 => (p - 4, p + 4),
                    _ => unreachable!(),
                };
                _HSD_RandForgetMemory(
                    &mut r,
                    lo as *const u8,
                    hi as *const u8,
                );
                0
            }
            7 => randi(&mut r, v[6] as i32) as u32,
            _ => unreachable!(),
        };
        let current = HSD_RandCurrentSeed(&mut r);
        let tr = TRACE.with(Cell::get);
        let got = [
            rc,
            r.seed,
            r.render_seed,
            r.render_depth,
            redirected,
            if r.seed_ptr == SeedSlot::Default {
                1
            } else {
                3
            },
            pointer_id(&r, current),
            tr[0],
            tr[1],
            tr[2],
        ];
        assert_eq!(got, v[8..], "C state row {row}, op {op}");
    }
    assert_eq!(coverage, [1000; 8]);
}
