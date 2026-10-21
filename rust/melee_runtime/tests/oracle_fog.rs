//! Bit-for-bit check of the logic of baselib fog.c against the C. Vectors:
//! `tests/data/oracle_fog.txt`, written by `oracle/build_fog.sh` (fog.c through `helper_fog.c`,
//! over the real aobj.c / fobj.c, compiled with the shipping build's contraction + gwtool lowering;
//! generated inputs only). Each row replays one op on a persistent fog and compares the GX call
//! log and the whole fog state (fog, fog-adj incl. its 4x4 matrix, AObj). Row format: see
//! `oracle/helper_fog.c`.
use melee_runtime::aobj::{AObjGlobals, HsdAObjDesc};
use melee_runtime::fog::{self, FogGx, HsdFog, HsdFogAdj, HsdFogAdjDesc, HsdFogDesc};
use melee_runtime::hsd_util::GxColor;
use melee_runtime::mtx::Mtx44;

const DATA: &str = include_str!("data/oracle_fog.txt");

fn b(f: f32) -> u32 {
    if f.is_nan() { 0x7FC0_0000 } else { f.to_bits() }
}
fn col(c: GxColor) -> u32 {
    u32::from_be_bytes([c.r, c.g, c.b, c.a])
}
fn uncol(w: u32) -> GxColor {
    let [r, g, b, a] = w.to_be_bytes();
    GxColor { r, g, b, a }
}

#[derive(Default)]
struct Gx {
    log: Vec<u32>,
    vp: [f32; 6],
    pj: [f32; 7],
}
impl FogGx for Gx {
    type Table = (u16, u16);
    fn set_fog(&mut self, ty: u32, s: f32, e: f32, n: f32, f: f32, c: GxColor) {
        self.log.extend([1, ty, b(s), b(e), b(n), b(f), col(c)]);
    }
    fn get_viewport(&mut self) -> [f32; 6] {
        self.log.push(2);
        self.vp
    }
    fn get_projection(&mut self) -> [f32; 7] {
        self.log.push(3);
        self.pj
    }
    fn init_fog_adj_table(&mut self, w: u16, m: &Mtx44) -> (u16, u16) {
        self.log.extend([4, w as u32]);
        for r in m {
            for &x in r {
                self.log.push(b(x));
            }
        }
        (0xABCD, w)
    }
    fn set_fog_range_adj(&mut self, en: bool, c: u16, t: Option<&(u16, u16)>) {
        self.log.extend([5, en as u32, c as u32, t.is_some() as u32]);
        if let Some(t) = t {
            self.log.extend([t.0 as u32, t.1 as u32]);
        }
    }
}

struct In<'a>(std::str::SplitWhitespace<'a>);
impl In<'_> {
    fn u(&mut self) -> u32 {
        u32::from_str_radix(self.0.next().unwrap(), 16).unwrap()
    }
    fn f(&mut self) -> f32 {
        f32::from_bits(self.u())
    }
    fn view(&mut self, gx: &mut Gx) {
        for i in 0..6 {
            gx.vp[i] = self.f();
        }
        for i in 0..7 {
            gx.pj[i] = self.f();
        }
    }
    fn adesc(&mut self) -> HsdFogAdjDesc {
        let center = self.u() as u16;
        let width = self.u() as u16;
        let mut mtx = [[0.0; 4]; 4];
        for r in mtx.iter_mut() {
            for x in r.iter_mut() {
                *x = self.f();
            }
        }
        HsdFogAdjDesc { center, width, mtx }
    }
}

fn dump(fog: &Option<HsdFog<()>>, out: &mut Vec<u32>) {
    out.push(fog.is_some() as u32);
    let Some(f) = fog else { return };
    out.extend([f.ty, b(f.start), b(f.end), col(f.color), f.fog_adj.is_some() as u32]);
    if let Some(a) = &f.fog_adj {
        out.extend([a.center as i32 as u32, a.width as u32]);
        for r in &a.mtx {
            for &x in r {
                out.push(b(x));
            }
        }
    }
    out.push(f.aobj.is_some() as u32);
    if let Some(a) = &f.aobj {
        out.extend([a.flags, b(a.curr_frame), b(a.rewind_frame), b(a.end_frame), b(a.framerate)]);
    }
}

#[test]
fn fog_matches_c() {
    let mut fog: Option<HsdFog<()>> = None;
    let mut g = AObjGlobals::default();
    let mut gx = Gx::default();
    let mut rows = 0;
    let mut per_op = [0u32; 11];
    for (ln, line) in DATA.lines().enumerate() {
        let (lhs, rhs) = line.split_once(" | ").unwrap();
        let mut i = In(lhs.split_whitespace());
        assert_eq!(i.0.next(), Some("O"));
        let op = i.u();
        per_op[op as usize] += 1;
        gx.log.clear();
        match op {
            0 => {
                let t = i.u();
                let v = i.f();
                fog.as_mut().unwrap().update_func(t, v);
            }
            1 => {
                let has = i.u();
                i.view(&mut gx);
                if has != 0 {
                    let d = HsdFogDesc { ty: i.u(), start: i.f(), end: i.f(), color: uncol(i.u()), fogadjdesc: None };
                    fog.as_mut().unwrap().init(Some(&d), &mut gx);
                } else {
                    fog.as_mut().unwrap().init(None, &mut gx);
                }
            }
            2 => {
                let has = i.u();
                i.view(&mut gx);
                let d = i.adesc();
                if let Some(a) = fog.as_mut().unwrap().fog_adj.as_mut() {
                    a.init(if has != 0 { Some(&d) } else { None }, &mut gx);
                }
            }
            3 => {
                let nul = i.u();
                i.view(&mut gx);
                let near = i.f();
                let far = i.f();
                fog::fog_set(if nul != 0 { None } else { fog.as_ref() }, Some((near, far)), &mut gx);
            }
            4 => {
                let fl = i.u();
                let fr = i.f();
                fog::req_anim_by_flags(fog.as_mut(), fl, fr);
            }
            5 => {
                let fr = i.f();
                fog::req_anim(fog.as_mut(), fr);
            }
            6 => {
                for _ in 0..i.u() {
                    fog.as_mut().unwrap().interpret_anim(&mut g);
                }
            }
            7 => {
                let has = i.u();
                let d = HsdAObjDesc { flags: i.u(), end_frame: i.f(), fobjdesc: Vec::new(), obj_id: 0 };
                fog.as_mut().unwrap().add_anim(if has != 0 { Some(&d) } else { None }, |_| unreachable!());
            }
            8 => {
                let hasadj = i.u();
                i.view(&mut gx);
                let ad = i.adesc();
                let d = HsdFogDesc {
                    ty: i.u(),
                    start: i.f(),
                    end: i.f(),
                    color: uncol(i.u()),
                    fogadjdesc: if hasadj != 0 { Some(ad) } else { None },
                };
                fog = Some(HsdFog::load_desc(&d, &mut gx));
            }
            9 => {
                let has = i.u();
                i.view(&mut gx);
                let d = i.adesc();
                fog.as_mut().unwrap().fog_adj = Some(HsdFogAdj::load_desc(if has != 0 { Some(&d) } else { None }, &mut gx));
            }
            10 => {
                if i.u() == 0 {
                    fog::req_anim_by_flags::<()>(None, 0x200, 1.0);
                    fog::req_anim::<()>(None, 2.0);
                }
            }
            _ => unreachable!(),
        }
        assert!(i.0.next().is_none(), "line {}: inputs left", ln + 1);
        let mut out = vec![gx.log.len() as u32];
        out.extend(&gx.log);
        out.push(0);
        dump(&fog, &mut out);
        let want: Vec<u32> = rhs.split_whitespace().map(|s| u32::from_str_radix(s, 16).unwrap()).collect();
        assert_eq!(out, want, "line {} op {}", ln + 1, op);
        rows += 1;
    }
    assert_eq!(rows, 8000);
    assert!(per_op.iter().all(|&n| n > 600), "{per_op:?}");
}
