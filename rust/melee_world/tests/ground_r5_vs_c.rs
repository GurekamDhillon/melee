//! melee_world::gr::ground night-3 round-5 ports against the C (cref/gen_ground_r5_ref.sh ->
//! tests/data/ground_r5_ref.txt): Ground_MexParamStkind, Ground_801C2AD8/2AE8, GetMapGObj,
//! 801C2BBC/2BD4/2C8C/2CF4/2D0C, grRCruise_80201918, grBigBlue_801EF7D8, grIceMt_801FA728;
//! Ground_801C20D0, 801C205C, 801C2090, 801C498C, 801C1D38.
//! Built with the shipping build's FMA contraction (no fused sites after lowering).
use melee_types::*;
use melee_world::gr::ground::{ground_801c1d38, ground_801c498c};
use melee_world::gr::stage::StageWorld;
use std::cell::Cell;

thread_local! {
    static CB: Cell<(i64, *mut HSD_GObj)> = const { Cell::new((0, core::ptr::null_mut())) };
}
unsafe extern "C" fn cb(g: *mut HSD_GObj) {
    CB.with(|c| c.set((c.get().0 + 1, g)));
}

const DATA: &str = include_str!("data/ground_r5_ref.txt");

fn i(s: &str) -> i64 {
    s.parse().unwrap()
}
fn h(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn fl(s: &str) -> f32 {
    f32::from_bits(h(s))
}
fn bits(v: &Vec) -> [u32; 3] {
    [v.x.to_bits(), v.y.to_bits(), v.z.to_bits()]
}

#[test]
fn ground_r5_matches_c() {
    let mut w = StageWorld::default();
    let mut gp: Box<GroundParam> = Box::new(GroundParam::zeroed());
    let mut rows: std::vec::Vec<StageParam> = (0..8).map(|_| StageParam::zeroed()).collect();
    let mut gobjs: std::vec::Vec<HSD_GObj> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    let mut grs: std::vec::Vec<Ground> = (0..4).map(|_| unsafe { core::mem::zeroed() }).collect();
    let ids = [3usize, 34, 9, 10];
    let mut n = [0usize; 5];
    let mut fg: Box<HSD_GObj> = Box::new(unsafe { core::mem::zeroed() });
    let mut fog: Box<HSD_Fog> = Box::new(unsafe { core::mem::zeroed() });
    for (ln, line) in DATA.lines().enumerate() {
        let t: std::vec::Vec<&str> = line.split_whitespace().collect();
        match t[0] {
            "M" if t[1] == "null" => {
                w.stage_info.param = core::ptr::null_mut();
                assert_eq!(w.ground_mex_param_stkind(i(t[2]) as u32), i(t[3]) as u32, "line {ln}");
                n[0] += 1;
            }
            "M" => {
                gp.x68 = i(t[1]) as i16;
                gp.stage_param_count = i(t[2]) as i32;
                let nullp = t[3] == "1";
                gp.stage_params = if nullp { core::ptr::null_mut() } else { rows.as_mut_ptr() };
                // The driver randomises 8 rows and prints 6; count is at most 5.
                for k in 0..6 {
                    let (s, x) = t[4 + k].split_once(':').unwrap();
                    rows[k].stkind = i(s) as u32;
                    rows[k].x18 = i(x) as i16;
                }
                w.stage_info.param = &mut *gp;
                let q = i(t[10]) as u32;
                assert_eq!(w.ground_mex_param_stkind(q), i(t[11]) as u32, "line {ln}: {line}");
                match t[12] {
                    "-" => {}
                    "spin" => {
                        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| w.ground_801c2ae8(q)));
                        assert!(r.is_err(), "line {ln}: expected the not-found panic");
                    }
                    b => assert_eq!(w.ground_801c2ae8(q).to_bits(), h(b), "line {ln}: {line}"),
                }
                n[0] += 1;
            }
            "C" => {
                match t[1] {
                    "reset" => w.stage_info.x694 = [core::ptr::null_mut(); 4],
                    "a" => {
                        let v = i(t[2]) as usize as *mut core::ffi::c_void;
                        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| w.ground_801c2bd4(v)));
                        assert_eq!(r.is_ok(), t[3] == "ok", "line {ln}: {line}");
                        for k in 0..4 {
                            assert_eq!(w.stage_info.x694[k] as usize as i64, i(t[4 + k]), "line {ln}: {line}");
                        }
                    }
                    "q" => {
                        let v = i(t[2]) as usize as *mut core::ffi::c_void;
                        assert_eq!(w.ground_801c2c8c(v) as i64, i(t[3]), "line {ln}");
                    }
                    _ => panic!(),
                }
                n[1] += 1;
            }
            "X" => {
                let (ix, v, m, mv) = (i(t[1]) as i32, i(t[2]) as usize, i(t[4]) as i32, i(t[5]) as usize);
                w.ground_801c2d0c(ix, v as *mut HSD_JObj);
                // Ground_801C2BBC is static in the C; it is the setter GetMapGObj reads back.
                w.ground_801c2bbc(mv as *mut _, m);
                assert_eq!(w.ground_801c2cf4(ix) as usize as i64, i(t[3]), "line {ln}");
                assert_eq!(w.ground_get_map_gobj(m) as usize as i64, i(t[6]), "line {ln}");
                let p = w.ground_801c2ad8() as *mut i32;
                assert_eq!((p == &mut w.stage_info.xA0 as *mut i32) as i64, i(t[7]), "line {ln}");
                n[2] += 1;
            }
            "G" => {
                w.stage_info.map_gobjs = [core::ptr::null_mut(); 64];
                w.selected_stkind = i(t[5]) as u32;
                let x1c = Vec { x: fl(t[6]), y: fl(t[7]), z: fl(t[8]) };
                let x10 = fl(t[9]);
                for j in 0..4 {
                    let c = i(t[1 + j]);
                    grs[j] = unsafe { core::mem::zeroed() };
                    if j < 2 {
                        grs[j].u.scroll.x1C = x1c;
                    } else {
                        grs[j].u.icemt.x10 = x10;
                    }
                    gobjs[j].user_data = if c == 1 { core::ptr::null_mut() } else { &mut grs[j] as *mut Ground as *mut _ };
                    w.stage_info.map_gobjs[ids[j]] = if c == 0 { core::ptr::null_mut() } else { &mut gobjs[j] as *mut HSD_GObj as *mut _ };
                }
                let a0 = Vec { x: fl(t[10]), y: fl(t[11]), z: fl(t[12]) };
                assert_eq!(t[13], "|");
                let (mut a, mut b, mut d) = (a0, a0, a0);
                w.grrcruise_80201918(&mut a);
                w.grbigblue_801ef7d8(&mut b);
                w.gricemt_801fa728(&mut d);
                let o: std::vec::Vec<u32> = t[14..23].iter().map(|s| h(s)).collect();
                assert_eq!(bits(&a), [o[0], o[1], o[2]], "line {ln}: {line}");
                assert_eq!(bits(&b), [o[3], o[4], o[5]], "line {ln}: {line}");
                assert_eq!(bits(&d), [o[6], o[7], o[8]], "line {ln}: {line}");
                n[3] += 1;
            }
            "F" => {
                w.stage_info.cam_info.cam_vertical_tilt = fl(t[1]);
                let (fc, cn) = (i(t[2]), t[3] == "1");
                let col = |u: u32| -> GXColor { let b = u.to_le_bytes(); GXColor { r: b[0], g: b[1], b: b[2], a: b[3] } };
                let unc = |c: &GXColor| u32::from_le_bytes([c.r, c.g, c.b, c.a]);
                fog.color = col(h(t[4]));
                fg.hsd_obj = if fc == 1 { core::ptr::null_mut() } else { &mut *fog as *mut HSD_Fog as *mut _ };
                w.stage_info.x12C = if fc == 0 { core::ptr::null_mut() } else { &mut *fg };
                let inc = col(h(t[5]));
                assert_eq!(t[6], "|");
                assert_eq!(w.ground_801c20d0().to_bits(), h(t[7]), "line {ln}");
                w.ground_801c205c(if cn { None } else { Some(&inc) });
                assert_eq!(unc(&fog.color), h(t[8]), "line {ln}: {line}");
                let mut out = col(h(t[11]));
                let r = w.ground_801c2090(if cn { None } else { Some(&mut out) });
                assert_eq!(r as i64, i(t[9]), "line {ln}: {line}");
                assert_eq!(unc(&out), h(t[10]), "line {ln}: {line}");
                n[4] += 1;
            }
            "L" => {
                let cls: std::vec::Vec<u16> = t[1..].iter().take_while(|s| **s != "-1").map(|s| i(s) as u16).collect();
                let mut lg: std::vec::Vec<HSD_GObj> = (0..cls.len().max(1)).map(|_| unsafe { core::mem::zeroed() }).collect();
                let base = lg.as_mut_ptr();
                for (j, c) in cls.iter().enumerate() {
                    lg[j].classifier = *c;
                    lg[j].next = if j + 1 < cls.len() { unsafe { base.add(j + 1) } } else { core::ptr::null_mut() };
                }
                let head = if cls.is_empty() { core::ptr::null_mut() } else { base };
                let g = unsafe { ground_801c498c(head) };
                let idx = if g.is_null() { -1 } else { (unsafe { g.offset_from(base) }) as i64 };
                assert_eq!(idx, i(t[t.len() - 1]), "line {ln}: {line}");
                n[4] += 1;
            }
            "K" => {
                let set = t[1] == "1";
                let mut kgr: Ground = unsafe { core::mem::zeroed() };
                let mut kg: HSD_GObj = unsafe { core::mem::zeroed() };
                kgr.xC_callback = if set { Some(cb) } else { None };
                kg.user_data = &mut kgr as *mut Ground as *mut _;
                CB.with(|c| c.set((0, core::ptr::null_mut())));
                unsafe { ground_801c1d38(&mut kg) };
                let (hits, arg) = CB.with(|c| c.get());
                assert_eq!((hits, (arg == &mut kg as *mut HSD_GObj) as i64), (i(t[3]), i(t[4])), "line {ln}");
                n[4] += 1;
            }
            _ => panic!("bad tag {}", t[0]),
        }
    }
    assert_eq!(n, [6000, 4200, 2000, 4000, 5200]);
}
