//! `tu::fobj` (fobj.c on the GameCube-layout `HSD_FObj`, raw pointers) vs the C, bit for bit: the rows of
//! `tests/data/oracle_anim.txt` (written by `oracle/build_anim.sh` from the shipping-contraction C build; the
//! `C` / `F` rows are 1000 random keyframe streams stepped 60 times each with `HSD_FObjInterpretAnim`,
//! `HSD_FObjStopAnim` and `HSD_FObjReqAnimAll`). `parseFloat`, `FObjLoadData`, `FObjUpdateAnim` and the
//! per-op static helpers run inside `HSD_FObjInterpretAnim`; every call's callbacks and the whole
//! interpreter state (flags, op, op_intrp, nb_pack, fterm, time, p0, p1, d0, d1, ad offset) are compared.
use core::ffi::c_void;
use melee_runtime::tu::fobj::{self as m, ext};
use melee_types::{HSD_FObj, HSD_ObjAllocData, HSD_ObjData, HSD_ObjUpdateFunc, _HSD_FObjDesc};

const DATA: &str = include_str!("data/oracle_anim.txt");

fn hx(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}
static mut CB: Vec<(u32, u32)> = Vec::new();
unsafe extern "C" fn cb(_obj: *mut c_void, ty: i32, d: *mut HSD_ObjData) {
    (*(&raw mut CB)).push((ty as u32, (*d).iv as u32));
}
fn alloc_fobj(_data: *mut HSD_ObjAllocData) -> *mut c_void {
    Box::into_raw(Box::new(HSD_FObj::zeroed())) as *mut c_void
}
unsafe fn dump(f: *mut HSD_FObj) -> Vec<u32> {
    vec![
        (*f).flags as u32,
        (*f).op as u32,
        (*f).op_intrp as u32,
        (*f).nb_pack as u32,
        (*f).fterm as u32,
        (*f).time.to_bits(),
        (*f).p0.to_bits(),
        (*f).p1.to_bits(),
        (*f).d0.to_bits(),
        (*f).d1.to_bits(),
        (*f).ad.offset_from((*f).ad_head) as u32,
    ]
}

#[test]
fn fobj_raw_matches_c() {
    unsafe {
        ext::HSD_OBJALLOC = Some(alloc_fobj);
        let mut stream: Vec<u8> = Vec::new();
        let mut fobj: *mut HSD_FObj = core::ptr::null_mut();
        let (mut checks, mut stepped) = (0usize, 0usize);
        for (ln, line) in DATA.lines().enumerate() {
            let t: Vec<&str> = line.split_whitespace().collect();
            match t[0] {
                "C" => {
                    stream = bytes(t[6]);
                    stream.extend([0u8; 64]);
                    let mut d = _HSD_FObjDesc::zeroed();
                    d.length = hx(t[1]);
                    d.startframe = f32::from_bits(hx(t[2]));
                    d.r#type = hx(t[3]) as u8;
                    d.frac_value = hx(t[4]) as u8;
                    d.frac_slope = hx(t[5]) as u8;
                    d.ad = stream.as_mut_ptr();
                    fobj = m::HSD_FObjLoadDesc(&mut d);
                    assert_eq!((*fobj).startframe as u16 as u32, hx(t[8]), "line {} LoadDesc startframe", ln + 1);
                    m::HSD_FObjReqAnimAll(fobj, f32::from_bits(hx(t[7])));
                    checks += 1;
                }
                "F" => {
                    let rate = f32::from_bits(hx(t[1]));
                    let (stop, nocb) = (t[2] == "1", t[3] == "1");
                    let n: usize = t[4].parse().unwrap();
                    (*(&raw mut CB)).clear();
                    let upd: HSD_ObjUpdateFunc = if nocb { None } else { Some(cb) };
                    if stop {
                        m::HSD_FObjStopAnim(fobj, core::ptr::null_mut(), upd, rate);
                    } else {
                        m::HSD_FObjInterpretAnim(fobj, core::ptr::null_mut(), upd, rate);
                    }
                    let got: Vec<(u32, u32)> = (*(&raw mut CB)).clone();
                    let want: Vec<(u32, u32)> = (0..n).map(|k| (hx(t[5 + 2 * k]), hx(t[6 + 2 * k]))).collect();
                    assert_eq!(got, want, "line {} callbacks (stop={stop})", ln + 1);
                    let mut k = 5 + 2 * n;
                    let want: Vec<u32> = t[k..k + 11].iter().map(|s| hx(s)).collect();
                    assert_eq!(dump(fobj), want, "line {} state (stop={stop})", ln + 1);
                    k += 11;
                    if t.len() > k && t[k] == "R" {
                        m::HSD_FObjReqAnimAll(fobj, f32::from_bits(hx(t[k + 1])));
                        let want: Vec<u32> = t[k + 2..k + 13].iter().map(|s| hx(s)).collect();
                        assert_eq!(dump(fobj), want, "line {} ReqAnim state", ln + 1);
                    }
                    stepped += 1;
                }
                _ => {}
            }
        }
        assert!(stepped >= 50_000 && checks >= 1000, "{stepped} steps, {checks} streams");
        let _ = &stream;
    }
}

// The pool wrappers have no arithmetic to compare: their contract in fobj.c is the call into objalloc.c with
// `fobj_alloc_data` (sizeof(HSD_FObj) (native 0x40, GameCube 0x30), align 4), a zeroed record from Alloc and the tail-first free
// order of RemoveAll. The recording hooks stand in for objalloc.c (itself ported in tu::objalloc).
static mut LOG: Vec<String> = Vec::new();
fn log(s: String) {
    unsafe { (*(&raw mut LOG)).push(s) }
}
fn h_init(d: *mut HSD_ObjAllocData, size: u32, align: u32) {
    log(format!("init {} {size} {align}", d == unsafe { m::HSD_FObjGetAllocData() }));
}
fn h_alloc(d: *mut HSD_ObjAllocData) -> *mut c_void {
    log(format!("alloc {}", d == unsafe { m::HSD_FObjGetAllocData() }));
    let mut f = HSD_FObj::zeroed();
    f.op = 0xAA;
    f.p1 = 1.5;
    Box::into_raw(Box::new(f)) as *mut c_void
}
fn h_free(d: *mut HSD_ObjAllocData, p: *mut c_void) {
    log(format!("free {} {}", d == unsafe { m::HSD_FObjGetAllocData() }, unsafe { (*(p as *mut HSD_FObj)).obj_type }));
}

#[test]
fn fobj_pool_wrappers() {
    unsafe {
        ext::HSD_OBJALLOCINIT = Some(h_init);
        ext::HSD_OBJALLOC = Some(h_alloc);
        ext::HSD_OBJFREE = Some(h_free);
        (*(&raw mut LOG)).clear();
        m::HSD_FObjInitAllocData();
        let a = m::HSD_FObjAlloc();
        assert_eq!(((*a).op, (*a).p1.to_bits()), (0, 0), "Alloc must zero the record");
        let (b, c) = (m::HSD_FObjAlloc(), m::HSD_FObjAlloc());
        (*a).obj_type = 1;
        (*b).obj_type = 2;
        (*c).obj_type = 3;
        (*a).next = b;
        (*b).next = c;
        (*(&raw mut LOG)).clear();
        m::HSD_FObjRemoveAll(a);
        m::HSD_FObjRemoveAll(core::ptr::null_mut());
        m::HSD_FObjRemove(core::ptr::null_mut());
        assert_eq!(*(&raw const LOG), ["free true 3", "free true 2", "free true 1"]);
        m::HSD_FObjInitAllocData();
        assert_eq!((*(&raw const LOG)).last().unwrap(), &format!("init true {} 4", core::mem::size_of::<HSD_FObj>()));
        assert_eq!(<HSD_FObj as melee_core::gc::GcLayout>::GC_SIZE, 0x30);
    }
}
