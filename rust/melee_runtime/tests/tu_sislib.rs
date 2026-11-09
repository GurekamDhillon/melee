#![allow(non_snake_case,static_mut_refs,unused_imports,dead_code)]
#[path="../../melee_world/tests/mtu_harness/mod.rs"]mod mtu_harness;
use mtu_harness::*;use melee_types::*;use melee_runtime::tu::sislib as m;
#[path="mtu/sislib_gen.rs"]mod g;
static mut ARENA:[u64;1024]=[0;1024];static mut TEX:[HSD_Text;3]=unsafe{core::mem::zeroed()};static mut E:[sislib_UnkAlloc3;3]=unsafe{core::mem::zeroed()};static mut BL:[SisBlock;9]=unsafe{core::mem::zeroed()};static mut G:[HSD_GObj;4]=unsafe{core::mem::zeroed()};static mut STR:[[i8;32];3]=[[0;32];3];static mut SI:[SIS;5]=unsafe{core::mem::zeroed()};static mut TABLE:[*mut SIS;5]=[core::ptr::null_mut();5];static mut AR:[u64;8]=[0;8];static mut CAM:HSD_CObj=unsafe{core::mem::zeroed()};static mut KCASE:u32=0;
#[test]fn all_functions_vs_c(){unsafe{
 preg(&raw const ARENA,8192);preg(&raw const TEX,core::mem::size_of_val(&TEX));preg(&raw const E,core::mem::size_of_val(&E));preg(&raw const BL,core::mem::size_of_val(&BL));preg(&raw const G,core::mem::size_of_val(&G));preg(&raw const STR,core::mem::size_of_val(&STR));preg(&raw const SI,core::mem::size_of_val(&SI));preg(&raw const TABLE,core::mem::size_of_val(&TABLE));preg(&raw const AR,core::mem::size_of_val(&AR));preg(&raw const CAM,core::mem::size_of_val(&CAM));g::install();g::freg_all();let mut check=Check::new(include_str!("data/mtu_sislib.txt"));
 for op in 0..22{for k in 0..400{case(op,k);KCASE=k;let mut ret=0;
 for i in 0..8192{*((&raw mut ARENA).cast::<u8>().add(i))=fv() as u8;}
 for i in 0..3{g::fill_HSD_Text(&mut TEX[i]);g::fill_sislib_UnkAlloc3(&mut E[i]);}for i in 0..9{g::fill_SisBlock(&mut BL[i]);}for i in 0..4{g::fill_HSD_GObj(&mut G[i]);}
 m::HSD_SisLib_803A6048(8192);TR=0;
 for i in 0..3{
 TEX[i].next=if i<2{&raw mut TEX[i+1]}else{core::ptr::null_mut()};TEX[i].entity=if fv()%2!=0{&raw mut G[i]}else{core::ptr::null_mut()};TEX[i].font_idx=(i%2) as u8;TEX[i].alloc_data=core::ptr::null_mut();TEX[i].state_stack=if fv()%2!=0{STR[i].as_mut_ptr() as *mut u8}else{core::ptr::null_mut()};TEX[i].render_callback=None;
 E[i].x0=if i<2{&raw mut E[i+1]}else{core::ptr::null_mut()};E[i].x4=if fv()%2!=0{&raw mut G[i]}else{core::ptr::null_mut()};E[i].xA=(i%2) as u16;E[i].xE%=16;
 for j in 0..32{STR[i][j]=fv() as i8;}
 BL[i].data=(&raw mut TEX[i]).cast();BL[i+3].data=(&raw mut E[i]).cast();BL[i+6].data=STR[i].as_mut_ptr().cast();
 }
 for i in 0..9{BL[i].next=if i<8{&raw mut BL[i+1]}else{core::ptr::null_mut()};BL[i].size=32;}
 m::used_head=&raw mut BL[0];m::HSD_SisLib_804D7978=&raw mut TEX[0];m::HSD_SisLib_804D797C=&raw mut E[0];
 for i in 0..5{TABLE[i]=&raw mut SI[i];m::HSD_SisLib_804D1124[i]=if k%3!=0{TABLE.as_mut_ptr().cast()}else{core::ptr::null_mut()};m::HSD_SisLib_804D1110[i]=if k%2!=0{AR.as_mut_ptr().cast()}else{core::ptr::null_mut()};}
 let ix=(rnd()%4) as usize;let font=(rnd()%3) as i32;let sz=(8*(1+rnd()%16)) as i32;let fx=rw();let fy=rw();let fz=rw();let fw=rw();let fh=rw();let cl=fv() as u16;let pl=fv() as u8;let pp=fv() as u8;let gl=fv() as u8;let gp=fv() as u8;let rp=fv();
 match op{
0=>{ret=pi(m::HSD_SisLib_Alloc(sz));},
1=>{m::HSD_SisLib_Free(if ix<3 {(&raw mut TEX[ix]).cast()}else{core::ptr::null_mut()});},
2=>{m::HSD_SisLib_803A5A2C(if ix<3 {(&raw mut TEX[ix]).cast()}else{core::ptr::null_mut()});},
3=>{ret=pi(m::HSD_SisLib_803A5ACC(0,k as i32%3-1,fx,fy,fz,fw,fh));},
4=>{m::HSD_SisLib_803A5CC4(if ix<3 {&raw mut TEX[ix]}else{core::ptr::null_mut()});},
5=>{m::HSD_SisLib_803A5D30();},
6=>{m::HSD_SisLib_803A5DA0_inline0(font);},
7=>{m::HSD_SisLib_803A5DA0(font);},
8=>{m::HSD_SisLib_803A5E70();},
9=>{m::HSD_SisLib_803A5F50(font);},
10=>{m::HSD_SisLib_803A5FBC();},
11=>{m::HSD_SisLib_803A6048(8192);},
12=>{m::fn_803A60EC((&raw mut G[ix]).cast());},
13=>{m::sislib_WideCamera(&raw mut G[ix],font as isize);},
14=>{m::HSD_SisLib_SetWideCanvas(font,(k%4) as i32);},
15=>{ret=m::HSD_SisLib_803A611C(font,if k%2!=0{&raw mut G[0]}else{core::ptr::null_mut()},cl,pl,pp,gl,gp,rp) as u32;},
16=>{m::HSD_SisLib_803A62A0(font,c"archive".as_ptr() as *mut _,c"symbol".as_ptr() as *mut _);},
17=>{m::HSD_SisLib_803A6368(&raw mut TEX[ix%3],(k%5) as i32);},
18=>{m::used_head=core::ptr::null_mut();let a=m::HSD_SisLib_Alloc(sz);let b=m::HSD_SisLib_Alloc(64);m::HSD_SisLib_Free(b);m::HSD_SisLib_Free(a);},
19=>{BL[7].next=core::ptr::null_mut();BL[8].next=core::ptr::null_mut();BL[8].size=32;(*m::free_head).next=&raw mut BL[8];ret=pi(m::HSD_SisLib_Alloc(sz));},
20=>{ret=pi(m::HSD_SisLib_Alloc(sz-3));},
21=>{m::used_head=core::ptr::null_mut();let a=m::HSD_SisLib_Alloc(sz-7);let b=m::HSD_SisLib_Alloc(16);m::HSD_SisLib_Free(b);m::HSD_SisLib_Free(a);},
_=>unreachable!(),}
 for i in 0..3{g::hash_HSD_Text(&TEX[i]);g::hash_sislib_UnkAlloc3(&E[i]);}for i in 0..9{g::hash_SisBlock(&BL[i]);}for i in 0..4{g::hash_HSD_GObj(&G[i]);}
 hm(pi(m::used_head));hm(pi(m::free_head));hm(m::HSD_SisLib_804D7968);hm(pi(m::HSD_SisLib_804D796C));hm(pi(m::HSD_SisLib_804D7978));hm(pi(m::HSD_SisLib_804D797C));
 for i in 0..5{hm(pi(m::HSD_SisLib_804D1110[i]));hm(pi(m::HSD_SisLib_804D1124[i]));}
 let mut b=m::used_head;while !b.is_null(){g::hash_SisBlock(&*b);b=(*b).next;}b=m::free_head;while !b.is_null(){g::hash_SisBlock(&*b);b=(*b).next;}
 let mut t=m::HSD_SisLib_804D7978;while !t.is_null(){g::hash_HSD_Text(&*t);if op==17&&t==&raw mut TEX[ix%3]{for j in 0..16{hm(*(*t).state_stack.add(j) as u8 as u32);}}t=(*t).next;}
 let mut e=m::HSD_SisLib_804D797C;while !e.is_null(){g::hash_sislib_UnkAlloc3(&*e);e=(*e).x0;}
 check.line(op,k,ret);
 }}assert_eq!(check.done(),8800);}}
