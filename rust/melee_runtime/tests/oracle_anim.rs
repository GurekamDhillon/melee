//! Bit-for-bit check of baselib fobj.c (the keyframe interpreter), util.c and crypt.c against
//! the C. Vectors: `tests/data/oracle_anim.txt`, written by `oracle/build_anim.sh` (C TUs
//! compiled with the shipping build's contraction + gwtool lowering; generated inputs only).
use std::sync::Arc;

use melee_runtime::fobj::{HsdFObj, HsdFObjDesc};
use melee_runtime::hsd_util::{self as hu, GxColor};

const DATA: &str = include_str!("data/oracle_anim.txt");

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
fn dump(f: &HsdFObj) -> Vec<u32> {
    vec![
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
    ]
}

#[test]
fn fobj_util_crypt_match_c_build() {
    let mut fobj: Option<HsdFObj> = None;
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
    for (ln, line) in DATA.lines().enumerate() {
        let t: Vec<&str> = line.split_whitespace().collect();
        match t[0] {
            "C" => {
                let desc = HsdFObjDesc {
                    length: hx(t[1]),
                    startframe: f32::from_bits(hx(t[2])),
                    type_: hx(t[3]) as u8,
                    frac_value: hx(t[4]) as u8,
                    frac_slope: hx(t[5]) as u8,
                    ad: Arc::from(bytes(t[6])),
                };
                let mut f = HsdFObj::load_desc(&desc);
                eq("LoadDesc startframe", ln, vec![f.startframe as u16 as u32], vec![hx(t[8])]);
                f.req_anim(f32::from_bits(hx(t[7])));
                fobj = Some(f);
            }
            "F" => {
                let f = fobj.as_mut().unwrap();
                let rate = f32::from_bits(hx(t[1]));
                let (stop, nocb) = (t[2] == "1", t[3] == "1");
                let n: usize = t[4].parse().unwrap();
                let mut got = Vec::new();
                {
                    let mut cb = |ty: u8, v: f32| got.extend([ty as u32, v.to_bits()]);
                    let mut upd: melee_runtime::fobj::ObjUpdate<'_> = if nocb { None } else { Some(&mut cb) };
                    if stop {
                        f.stop_anim(&mut upd, rate);
                    } else {
                        f.interpret_anim(&mut upd, rate);
                    }
                }
                let want: Vec<u32> = t[5..5 + 2 * n].iter().map(|s| hx(s)).collect();
                eq(if stop { "StopAnim callbacks" } else { "InterpretAnim callbacks" }, ln, got, want);
                let mut k = 5 + 2 * n;
                let want: Vec<u32> = t[k..k + 11].iter().map(|s| hx(s)).collect();
                eq(if stop { "StopAnim state" } else { "InterpretAnim state" }, ln, dump(f), want);
                k += 11;
                if t.len() > k && t[k] == "R" {
                    f.req_anim(f32::from_bits(hx(t[k + 1])));
                    let want: Vec<u32> = t[k + 2..k + 13].iter().map(|s| hx(s)).collect();
                    eq("ReqAnim state", ln, dump(f), want);
                }
            }
            "U" => {
                let c = |s: &str| {
                    let v = bytes(s);
                    GxColor { r: v[0], g: v[1], b: v[2], a: v[3] }
                };
                let r = hu::mul_color(&c(t[1]), &c(t[2]));
                eq("HSD_MulColor", ln, vec![r.r as u32, r.g as u32, r.b as u32, r.a as u32], bytes(t[3]).iter().map(|&b| b as u32).collect());
                eq("HSD_GetNbBits", ln, vec![hu::get_nb_bits(hx(t[4]))], vec![hx(t[5])]);
                eq("HSD_Index2PosNrmMtx", ln, vec![hu::index_to_pos_nrm_mtx(hx(t[6])) as u32], vec![hx(t[7])]);
            }
            "E" => {
                let mut d = bytes(t[2]);
                let e = hu::encrypt(&mut d);
                eq("HSD_Encrypt", ln, vec![e as u32], vec![t[3].parse::<i32>().unwrap() as u32]);
                eq("HSD_Encrypt data", ln, d.iter().map(|&b| b as u32).collect(), bytes(t[4]).iter().map(|&b| b as u32).collect());
                let mut d = bytes(t[5]);
                let e = hu::decrypt(&mut d);
                eq("HSD_Decrypt", ln, vec![e as u32], vec![t[6].parse::<i32>().unwrap() as u32]);
                eq("HSD_Decrypt data", ln, d.iter().map(|&b| b as u32).collect(), bytes(t[7]).iter().map(|&b| b as u32).collect());
            }
            _ => panic!("bad row {}", ln + 1),
        }
    }
    eprintln!("oracle_anim: {checks} checks, {failed} failed, per function {per:?}");
    assert!(failed == 0, "{failed}/{checks} mismatches:\n{}", msgs.join("\n"));
    assert!(checks > 100_000);
}
