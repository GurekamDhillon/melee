from pathlib import Path
r=Path.cwd(); d=r/'rust/melee_runtime/gen/sis_sobj/tu'
ops=[
('HSD_SObjLib_803A44A4();','m::HSD_SObjLib_803A44A4();'),
('HSD_SObjLib_803A44D4(&G,&S[ix],pr);','m::HSD_SObjLib_803A44D4(&raw mut G,&raw mut S[ix],pr);'),
('HSD_SObjLib_803A466C(k%5==0?0:&S[ix]);','m::HSD_SObjLib_803A466C(if k%5==0 {core::ptr::null_mut()} else {&raw mut S[ix]});'),
('HSD_SObjLib_803A4740((HSD_SObj*)G.hsd_obj);','m::HSD_SObjLib_803A4740(G.hsd_obj.cast());'),
('ret=PI(HSD_SObjLib_803A477C(&G,&D.desc,ws,wt,pr,k%2));','ret=pi(m::HSD_SObjLib_803A477C(&raw mut G,&raw mut D.desc,ws,wt,pr,(k%2) as u8));'),
('HSD_SObjLib_803A49E0(&G,ix);','m::HSD_SObjLib_803A49E0(&raw mut G,ix as i32);'),
('HSD_SObjLib_803A4A68(&S[ix]);','m::HSD_SObjLib_803A4A68(&raw mut S[ix]);'),
('HSD_SObjLib_803A54EC(&G,ix);','m::HSD_SObjLib_803A54EC(&raw mut G,ix as i32);'),
('HSD_SObjLib_803A55DC(&G,ww,hh,pr);','m::HSD_SObjLib_803A55DC(&raw mut G,ww,hh,pr as i32);'),
('GXColor c=discard_color(S[ix].x38_color); ret=RH(&c,sizeof c);','let c=m::discard_color(S[ix].anon_38.x38_color); ret=rh(&c);'),
('ret=PI(HSD_GObjGetHSDObj(&G)); GXEnd();','ret=pi(m::HSD_GObjGetHSDObj(&raw mut G));m::GXEnd();'),
]
c='''#include TU_SRC
#include "mtu_common.h"
static HSD_SObj S[4]; static HSD_GObj G; static HSD_SObjDesc2 D; static HSD_ImageDesc IM[2]; static HSD_Tlut TL; static HSD_CObj CAM;
static void callback(HSD_SObj* s) { T(2000); T(PI(s)); s->x48++; }
#include "sobjlib_gen.h"
int main(void) {
 preg(S,sizeof S); preg(&G,sizeof G); preg(&D,sizeof D); preg(IM,sizeof IM); preg(&TL,sizeof TL); preg(&CAM,sizeof CAM); preg(&HSD_SObjLib_804D10E0,sizeof HSD_SObjLib_804D10E0);
 freg_all();freg(callback);
 for(int op=0;op<11;op++) for(int k=0;k<500;k++) {
 tu_case(op,k); u32 ret=0;
 for(int i=0;i<4;i++) fill_HSD_SObj(&S[i]); fill_HSD_GObj(&G); for(int i=0;i<2;i++) fill_HSD_ImageDesc(&IM[i]); fill_HSD_Tlut(&TL); fill_HSD_ObjAllocData(&HSD_SObjLib_804D10E0);
 int n=rnd()%4; unsigned ix=rnd()%4; u8 pr=fv(); u32 ws=fv()%3,wt=fv()%3;u16 ww=fv(),hh=fv();
 for(int i=0;i<4;i++){S[i].x0=0;S[i].gobj=&G;S[i].prev=(i>0&&i<n)?&S[i-1]:0;S[i].next=(i+1<n)?&S[i+1]:0;S[i].x44=i*60;S[i].x40=fv()%32;S[i].x18=(float)((int)(fv()%101)-50)/8;S[i].x4C_callback=(fv()%2)?callback:0;}
 HSD_SObjLib_804D7960=23;G.obj_kind=k%7?23:24; G.hsd_obj=n?S:0;
 D.desc.image=&IM[0];D.desc.tlut=k%2?&TL:0;D.image2=&IM[1]; IM[0].image_ptr=0;IM[1].image_ptr=0;TL.lut=0;
 switch(op){
'''
rs='''#![allow(non_snake_case,static_mut_refs,unused_imports,dead_code)]
#[path="../../melee_world/tests/mtu_harness/mod.rs"] mod mtu_harness;
use mtu_harness::*; use melee_types::*; use melee_runtime::tu::sobjlib as m;
#[path="mtu/sobjlib_gen.rs"] mod g;
static mut S:[HSD_SObj;4]=unsafe{core::mem::zeroed()}; static mut G:HSD_GObj=unsafe{core::mem::zeroed()};static mut D:HSD_SObjDesc2=unsafe{core::mem::zeroed()};static mut IM:[HSD_ImageDesc;2]=unsafe{core::mem::zeroed()};static mut TL:HSD_Tlut=unsafe{core::mem::zeroed()};static mut CAM:HSD_CObj=unsafe{core::mem::zeroed()};
unsafe extern "C" fn callback(s:*mut HSD_SObj){t(2000);t(pi(s));(*s).x48=(*s).x48.wrapping_add(1);}
#[test] fn all_functions_vs_c(){unsafe{
 preg(&raw const S,core::mem::size_of_val(&S));preg(&raw const G,core::mem::size_of_val(&G));preg(&raw const D,core::mem::size_of_val(&D));preg(&raw const IM,core::mem::size_of_val(&IM));preg(&raw const TL,core::mem::size_of_val(&TL));preg(&raw const CAM,core::mem::size_of_val(&CAM));preg(&raw const m::HSD_SObjLib_804D10E0,core::mem::size_of_val(&m::HSD_SObjLib_804D10E0));
 g::install();g::freg_all();freg(callback as *const () as usize);let mut check=Check::new(include_str!("data/mtu_sobjlib.txt"));
 for op in 0..11 {for k in 0..500 {case(op,k); let mut ret=0;
 for i in 0..4 {g::fill_HSD_SObj(&mut S[i]);}g::fill_HSD_GObj(&mut G);for i in 0..2 {g::fill_HSD_ImageDesc(&mut IM[i]);}g::fill_HSD_Tlut(&mut TL);g::fill_HSD_ObjAllocData(&mut m::HSD_SObjLib_804D10E0);
 let n=(rnd()%4) as usize;let ix=(rnd()%4) as usize;let pr=fv() as u8;let ws=fv()%3;let wt=fv()%3;let ww=fv() as u16;let hh=fv() as u16;
 for i in 0..4 {S[i].x0=core::ptr::null_mut();S[i].gobj=&raw mut G;S[i].prev=if i>0&&i<n {&raw mut S[i-1]}else{core::ptr::null_mut()};S[i].next=if i+1<n {&raw mut S[i+1]}else{core::ptr::null_mut()};S[i].x44=(i*60) as u8;S[i].x40=fv()%32;S[i].x18=((fv()%101) as i32-50) as f32/8.0;S[i].x4C_callback=if fv()%2!=0 {Some(callback)}else{None};}
 m::HSD_SObjLib_804D7960=23;G.obj_kind=if k%7!=0{23}else{24};G.hsd_obj=if n!=0{(&raw mut S[0]).cast()}else{core::ptr::null_mut()};
 D.desc.image=&raw mut IM[0];D.desc.tlut=if k%2!=0{&raw mut TL}else{core::ptr::null_mut()};D.image2=&raw mut IM[1];IM[0].image_ptr=core::ptr::null_mut();IM[1].image_ptr=core::ptr::null_mut();TL.lut=core::ptr::null_mut();
 match op {
'''
for i,(a,b) in enumerate(ops):c+=f'case {i}:{{{a}}}break;\n';rs+=f'{i}=>{{{b}}},\n'
c+='''}
 for(int i=0;i<4;i++)hash_HSD_SObj(&S[i]);hash_HSD_GObj(&G);for(int i=0;i<2;i++)hash_HSD_ImageDesc(&IM[i]);hash_HSD_Tlut(&TL);hash_HSD_ObjAllocData(&HSD_SObjLib_804D10E0);
 printf("%d %d %08X %08X %08X\\n",op,k,ret,H,TR);
 }return 0;}
'''
rs+='''_=>unreachable!(),}
 for i in 0..4 {g::hash_HSD_SObj(&S[i]);}g::hash_HSD_GObj(&G);for i in 0..2 {g::hash_HSD_ImageDesc(&IM[i]);}g::hash_HSD_Tlut(&TL);g::hash_HSD_ObjAllocData(&m::HSD_SObjLib_804D10E0);check.line(op,k,ret);
 }} assert_eq!(check.done(),5500);}}
'''
(d/'sobjlib.c').write_text(c);(r/'rust/melee_runtime/tests/tu_sobjlib.rs').write_text(rs)
