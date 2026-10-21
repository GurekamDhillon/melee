//! Bit-for-bit check of baselib aobj.c (over the fobj.c port), id.c and list.c against the C.
//! Vectors: `tests/data/oracle_aobj.txt`, written by `oracle/build_aobj.sh` (C TUs compiled with
//! the shipping build's contraction + gwtool lowering; generated inputs only).
use std::sync::Arc;

use melee_runtime::aobj::{AObjGlobals, HsdAObj, HsdAObjDesc};
use melee_runtime::fobj::{HsdFObj, HsdFObjDesc, ObjUpdate};
use melee_runtime::id::IdTable;
use melee_runtime::list::SList;

const DATA: &str = include_str!("data/oracle_aobj.txt");

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn hx64(s: &str) -> u64 {
    u64::from_str_radix(s, 16).unwrap()
}
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
fn f(s: &str) -> f32 {
    f32::from_bits(hx(s))
}
fn dump_fobj(f: &HsdFObj, out: &mut Vec<u32>) {
    out.extend([
        f.flags as u32,
        f.op as u32,
        f.op_intrp as u32,
        f.nb_pack as u32,
        f.fterm as u32,
        f.time.to_bits(),
        f.p0.to_bits(),
        f.p1.to_bits(),
        f.d0.to_bits(),
        f.d1.to_bits(),
        f.ad as u32,
    ]);
}
fn dump(a: &HsdAObj<()>) -> Vec<u32> {
    let mut v = vec![
        a.flags,
        a.curr_frame.to_bits(),
        a.rewind_frame.to_bits(),
        a.end_frame.to_bits(),
        a.framerate.to_bits(),
        melee_runtime::aobj::get_flags(Some(a)),
        a.fobj.len() as u32,
    ];
    a.fobj.iter().for_each(|f| dump_fobj(f, &mut v));
    v
}
/// The C prints `ad` as ffffffff while it is NULL (before the first ReqAnim); the Rust keeps an
/// offset of 0 there. Those slots are not compared.
fn want_state(t: &[&str], got: &[u32]) -> Vec<u32> {
    t.iter()
        .enumerate()
        .map(|(i, s)| {
            if *s == "ffffffff" && i >= 7 && (i - 7) % 11 == 10 {
                got.get(i).copied().unwrap_or(0)
            } else {
                hx(s)
            }
        })
        .collect()
}

#[test]
fn aobj_id_list_match_c_build() {
    let (mut checks, mut failed) = (0usize, 0usize);
    let mut per = std::collections::BTreeMap::<&str, usize>::new();
    let mut msgs = Vec::new();
    let mut eq = |what: &'static str, ln: usize, got: Vec<u32>, want: Vec<u32>| {
        checks += 1;
        if got != want {
            failed += 1;
            *per.entry(what).or_default() += 1;
            if msgs.len() < 30 {
                msgs.push(format!("line {} {what}: got {got:x?} want {want:x?}", ln + 1));
            }
        }
    };
    let mut aobj: Option<HsdAObj<()>> = None;
    let mut descs: Vec<HsdFObjDesc> = Vec::new();
    let mut g = AObjGlobals::default();
    let mut tables: [IdTable<u64>; 2] = [IdTable::new(), IdTable::new()];
    let mut list: SList<u32> = SList::new();
    for (ln, line) in DATA.lines().enumerate() {
        let t: Vec<&str> = line.split_whitespace().collect();
        let bar = t.iter().position(|s| *s == "|");
        match t[0] {
            "A" => {
                let nf: usize = t[3].parse().unwrap();
                descs = (0..nf)
                    .map(|k| {
                        let b = 4 + 6 * k;
                        HsdFObjDesc {
                            length: hx(t[b]),
                            startframe: f(t[b + 1]),
                            type_: hx(t[b + 2]) as u8,
                            frac_value: hx(t[b + 3]) as u8,
                            frac_slope: hx(t[b + 4]) as u8,
                            ad: Arc::from(bytes(t[b + 5])),
                        }
                    })
                    .collect();
                let d = HsdAObjDesc { flags: hx(t[1]), end_frame: f(t[2]), fobjdesc: descs.clone(), obj_id: 0 };
                let a = HsdAObj::<()>::load_desc(&d, |_| unreachable!());
                let got = dump(&a);
                eq("AObjLoadDesc", ln, got.clone(), want_state(&t[bar.unwrap() + 1..], &got));
                if let Some(old) = aobj.replace(a) {
                    old.remove();
                }
            }
            "O" => {
                let a = aobj.as_mut().unwrap();
                let (op, u, fv, nocb) = (t[1].parse::<u32>().unwrap(), hx(t[2]), f(t[3]), t[4] == "1");
                let ncb: usize = t[5].parse().unwrap();
                let mut cbs = Vec::new();
                {
                    let mut cb = |ty: u8, v: f32| cbs.extend([ty as u32, v.to_bits()]);
                    let mut upd: ObjUpdate = if nocb { None } else { Some(&mut cb) };
                    match op {
                        0 => a.req_anim(fv),
                        1 => a.set_flags(u),
                        2 => a.clear_flags(u),
                        3 => a.set_rate(fv),
                        4 => a.set_rewind_frame(fv),
                        5 => a.set_end_frame(fv),
                        6 => a.set_current_frame(fv),
                        7 => a.stop_anim(&mut upd),
                        8 => {
                            g.init_end_callback();
                            g.invoke_callbacks();
                        }
                        9 => a.set_fobj(if u == 1 { vec![HsdFObj::load_desc(&descs[0])] } else { Vec::new() }),
                        _ => a.interpret_anim(&mut g, &mut upd),
                    }
                }
                let what: &'static str = match op {
                    0 => "AObjReqAnim",
                    1 => "AObjSetFlags",
                    2 => "AObjClearFlags",
                    3 => "AObjSetRate",
                    4 => "AObjSetRewindFrame",
                    5 => "AObjSetEndFrame",
                    6 => "AObjSetCurrentFrame",
                    7 => "AObjStopAnim",
                    8 => "AObjInitEndCallBack",
                    9 => "AObjSetFObj",
                    _ => "AObjInterpretAnim",
                };
                eq(what, ln, cbs, t[6..6 + 2 * ncb].iter().map(|s| hx(s)).collect());
                let got = dump(a);
                eq(what, ln, got.clone(), want_state(&t[bar.unwrap() + 1..], &got));
            }
            "I" => tables[t[1].parse::<usize>().unwrap()].insert(hx64(t[2]), hx64(t[3])),
            "D" => tables[t[1].parse::<usize>().unwrap()].remove(hx64(t[2])),
            "Z" => tables[0].clear(),
            "G" => {
                let r = tables[t[1].parse::<usize>().unwrap()].get(hx64(t[2]));
                let ok = if t[3] == "1" { r.is_some() as u32 } else { 7 };
                let v = r.copied().unwrap_or(0);
                let w = hx64(t[4]);
                eq(
                    "IDGetDataFromTable",
                    ln,
                    vec![v as u32, (v >> 32) as u32, ok],
                    vec![w as u32, (w >> 32) as u32, t[5].parse().unwrap()],
                );
            }
            "L" => {
                match t[1] {
                    "0" => list.alloc_and_append(hx(t[2])),
                    "1" => list.alloc_and_prepend(hx(t[2])),
                    _ => {
                        list.remove();
                    }
                }
                eq("SList", ln, list.items.clone(), t[4..].iter().map(|s| hx(s)).collect());
            }
            _ => panic!("bad row {}", ln + 1),
        }
    }
    eprintln!("oracle_aobj: {checks} checks, {failed} failed, per function {per:?}");
    assert!(failed == 0, "{failed}/{checks} mismatches:\n{}", msgs.join("\n"));
    assert!(checks > 30_000);
}
