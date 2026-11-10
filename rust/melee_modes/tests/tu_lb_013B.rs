//! src/tu/lb_013B.rs vs the C (gen/cref/tu/lb_013B.c: the whole TU, shipping contraction;
//! regenerate with `python rust/melee_modes/gen/cref/run_tu.py lb_013B`).
#![allow(static_mut_refs, non_snake_case)]
mod mtu_harness;
use melee_modes::tu::lb_013B as m;
use melee_types::*;
use mtu_harness::*;
#[path = "mtu/lb_013B_gen.rs"]
mod g;

static mut CO: ColorOverlay = unsafe { core::mem::zeroed() };
static mut W: [ColorOverlay_x8_t; 128] = unsafe { core::mem::zeroed() };
static mut RUM: [Fighter_804D653C_t; 8] = unsafe { core::mem::zeroed() };
static mut G: HSD_GObj = unsafe { core::mem::zeroed() };
static mut WN: usize = 0;

unsafe fn wcolor() {
    let c = &mut W[WN].light_color;
    c.r = fv() as u8;
    c.g = fv() as u8;
    c.b = fv() as u8;
    c.a = fv() as u8;
    WN += 1;
}
unsafe fn put_cmd(op: u32) {
    match op {
        0xB | 0xF | 0x13 => {
            W[WN].unk.set_unk(op);
            W[WN].unk.set_timer(fv() & 0x3FF_FFFF);
            WN += 1;
            if op != 0xB {
                wcolor();
            }
        }
        0xD => {
            W[WN].unk.set_unk(op);
            W[WN].light_rot2.set_light_enable(fv() & 1);
            W[WN].light_rot2.set_x((fv() & 0xFFF) as i32 - 0x800);
            W[WN].light_rot2.set_yz((fv() & 0xFFF) as i32 - 0x800);
            WN += 1;
            wcolor();
        }
        0x10 => {
            W[WN].unk.set_unk(op);
            W[WN].light_rot1.set_x((fv() & 0x1FFF) as i32 - 0x1000);
            W[WN].light_rot1.set_yz((fv() & 0x1FFF) as i32 - 0x1000);
            WN += 1;
        }
        0xE | 0x12 => {
            W[WN].unk.set_unk(op);
            WN += 1;
            wcolor();
        }
        _ => {
            W[WN].unk.set_unk(op);
            WN += 1;
        }
    }
}
unsafe fn build_stream(forced: i32) {
    const TBL: [u32; 16] = [0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x20, 0x3F, 0x05, 0x00, 0xA];
    W = core::mem::zeroed();
    WN = 0;
    if forced >= 0 {
        put_cmd(forced as u32);
        return;
    }
    let n = 1 + rnd() % 10;
    for _ in 0..n {
        put_cmd(TBL[(rnd() % 16) as usize]);
    }
    put_cmd(0xA);
}
unsafe extern "C" fn cmdcb(g: *mut Fighter_GObj, i: *mut CommandInfo, op: i32) {
    let c = i as *mut ColorOverlay;
    t(7001);
    t(pi(g));
    t(pi(i));
    t(op as u32);
    (*c).x8_ptr1 = (*c).x8_ptr1.add(1);
    if rnd() % 4 == 0 {
        (*c).x0_timer = (rnd() % 3) as i32;
    }
}

fn reg<T>(p: *const T) {
    preg(p, core::mem::size_of::<T>());
}

#[test]
fn lb_013B_matches_c() {
    unsafe {
        g::install();
        reg(&raw const CO);
        reg(&raw const W);
        reg(&raw const RUM);
        reg(&raw const G);
        g::freg_all();
        let mut c = Check::new(include_str!("data/mtu_lb_013B.txt"));
        for op in 0..18u32 {
            for k in 0..300u32 {
                let mut ret = 0u32;
                case(op, k);
                CO = core::mem::zeroed();
                RUM = core::mem::zeroed();
                g::fill_ColorOverlay(&mut CO);
                for i in 0..8 {
                    g::fill_Fighter_804D653C_t(&mut RUM[i]);
                    RUM[i].unk = ((&raw mut W[i * 3]) as *mut core::ffi::c_void) as *mut _;
                }
                let w0 = (&raw mut W) as *mut ColorOverlay_x8_t;
                match op {
                    0 => { build_stream(0xA); CO.x8_ptr1 = w0; ret = m::lb_80013BB0(&raw mut CO) as u32 }
                    1 => { build_stream(0xB); CO.x8_ptr1 = w0; ret = m::lb_80013BB8(&raw mut CO) as u32 }
                    2 => { build_stream(0xC); CO.x8_ptr1 = w0; ret = m::lb_80013BE4(&raw mut CO) as u32 }
                    3 => { build_stream(0xD); CO.x8_ptr1 = w0; ret = m::lb_80013C18(&raw mut CO) as u32 }
                    4 => { build_stream(0xE); CO.x8_ptr1 = w0; ret = m::lb_80013D68(&raw mut CO) as u32 }
                    5 => { build_stream(0xF); CO.x8_ptr1 = w0; ret = m::lb_80013E3C(&raw mut CO) as u32 }
                    6 => { build_stream(0x10); CO.x8_ptr1 = w0; ret = m::lb_80013F78(&raw mut CO) as u32 }
                    7 => { build_stream(0x11); CO.x8_ptr1 = w0; ret = m::lb_80013FF0(&raw mut CO) as u32 }
                    8 => { build_stream(0x12); CO.x8_ptr1 = w0; ret = m::lb_80014014(&raw mut CO) as u32 }
                    9 => { build_stream(0x13); CO.x8_ptr1 = w0; ret = m::lb_800140F8(&raw mut CO) as u32 }
                    10 => { build_stream(0x14); CO.x8_ptr1 = w0; ret = m::lb_80014234(&raw mut CO) as u32 }
                    11 => {
                        build_stream(-1);
                        CO.x8_ptr1 = if rnd() % 8 == 0 { core::ptr::null_mut() } else { w0 };
                        ret = m::lb_80014258(&raw mut G, (&raw mut CO) as *mut _, Some(cmdcb)) as u32;
                    }
                    12 => { build_stream(-1); CO.x8_ptr1 = w0; m::lb_80014498(&raw mut CO) }
                    13 => {
                        build_stream(-1);
                        CO.x28_colanim.i = (rnd() % 8) as i32;
                        let a2 = (rnd() % 8) as i32;
                        let a3 = fv() as i32;
                        ret = m::lb_800144C8(&raw mut CO, (&raw mut RUM) as *mut _, a2, a3) as u32;
                    }
                    14 => { m::lb_804D63C0 = core::ptr::null_mut(); m::lb_80014534() }
                    15 => {
                        m::lb_804D63C0 = (&raw mut RUM) as *mut _;
                        let a0 = fv() as u8;
                        let a1 = fv() as i32;
                        let a2 = (rnd() % 8) as i32;
                        let a3 = fv() as i32;
                        m::lb_80014574(a0, a1, a2, a3)
                    }
                    16 => m::lb_800145C0(fv() as u8),
                    _ => m::lb_800145F4(),
                }
                if TRACEON { for i in 0..31 { println!("D {:08X}", *((&raw const CO) as *const u32).add(i)); } }
                g::hash_ColorOverlay(&CO);
                for i in 0..8 {
                    g::hash_Fighter_804D653C_t(&RUM[i]);
                }
                hm(pi(m::lb_804D63C0));
                c.line(op, k, ret);
            }
        }
        assert_eq!(c.done(), 18 * 300);
    }
}
