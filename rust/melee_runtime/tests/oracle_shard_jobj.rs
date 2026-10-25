//! State transcript from the real jobj.c, compiled -ffp-contract=on and lower.py.
#![allow(non_snake_case,static_mut_refs)]
use melee_runtime::{tu::jobj::*,mtx};
use melee_types::*;
use std::{ptr::null_mut,fmt::Write};
static mut EV:u32=0;
static mut STATUS:i32=0;
static mut OBJECTS:std::vec::Vec<*mut HSD_JObj>=std::vec::Vec::new();
static mut IDS:std::vec::Vec<(usize,*mut HSD_JObj)>=std::vec::Vec::new();
static mut PARENT:_HSD_ClassInfo=unsafe{std::mem::zeroed()};
static mut CAMERA:HSD_CObj=unsafe{std::mem::zeroed()};
unsafe fn event(n:u32){EV=EV.wrapping_mul(33).wrapping_add(n)}
fn word(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619)}
unsafe fn pid(j:*mut HSD_JObj)->u32{if j.is_null(){0}else{OBJECTS.iter().position(|&p|p==j).map_or(99,|i|i as u32+1)}}
unsafe extern "C" fn parent_init(_o:*mut _HSD_Class)->i32{event(1);STATUS}
unsafe extern "C" fn parent_release(_o:*mut _HSD_Class){event(2)}
unsafe extern "C" fn parent_amnesia(_i:*mut _HSD_ClassInfo){event(3)}
unsafe extern "C" fn destroy(_o:*mut _HSD_Class){event(4)}
unsafe fn init_info(i:*mut _HSD_ClassInfo,p:*mut _HSD_ClassInfo,_l:*const i8,_n:*const i8,is:i32,os:i32){event(5);(*i).head.parent=p;(*i).head.info_size=is as i16;(*i).head.obj_size=os as i16;(*i).destroy=Some(destroy)}
unsafe fn descendant(_i:*mut _HSD_ClassInfo,_p:*mut _HSD_ClassInfo)->bool{event(6);true}
unsafe fn search(n:*mut i8)->*mut _HSD_ClassInfo{event(7);if *n==b'x' as i8{(&raw mut hsdJObj).cast()}else{null_mut()}}
unsafe fn new(i:*mut _HSD_ClassInfo)->*mut HSD_JObj{let j=Box::into_raw(Box::new(HSD_JObj::zeroed()));OBJECTS.push(j);(*j).object.parent.class_info=i;((*i).init.unwrap())(j.cast());j}
unsafe fn delete(j:*mut HSD_JObj){let i=(*j).object.parent.class_info;((*i).release.unwrap())(j.cast());((*i).destroy.unwrap())(j.cast())}
unsafe fn dload(d:*mut HSD_DObjDesc)->*mut HSD_DObj{event(8+(!d.is_null() as u32));if d.is_null(){null_mut()}else{Box::into_raw(Box::new(HSD_DObj::zeroed()))}}
unsafe fn rload(d:*mut HSD_RObjDesc)->*mut HSD_RObj{event(10+(!d.is_null() as u32));if d.is_null(){null_mut()}else{Box::into_raw(Box::new(HSD_RObj::zeroed()))}}
unsafe fn dresolve(j:*mut HSD_DObj,_d:*mut HSD_DObjDesc){event(12+(!j.is_null() as u32))}
unsafe fn rresolve(j:*mut HSD_RObj,_d:*mut HSD_RObjDesc){event(14+(!j.is_null() as u32))}
unsafe fn dremove(_j:*mut HSD_DObj){event(16)}
unsafe fn rremove(_j:*mut HSD_RObj){event(17)}
unsafe fn aremove(_j:*mut HSD_AObj){event(18)}
unsafe fn insert(_t:*mut std::ffi::c_void,id:usize,j:*mut HSD_JObj){event(19);if let Some(p)=IDS.iter_mut().find(|p|p.0==id){p.1=j}else{IDS.push((id,j))}}
unsafe fn get(_t:*mut std::ffi::c_void,id:usize,_s:*mut i32)->*mut HSD_JObj{event(20);IDS.iter().find(|p|p.0==id).map_or(null_mut(),|p|p.1)}
unsafe fn remove(_t:*mut std::ffi::c_void,id:usize){event(21);IDS.retain(|p|p.0!=id)}
unsafe fn camera()->*mut HSD_CObj{event(22);&raw mut CAMERA}
unsafe fn display(j:*mut HSD_JObj,m:&Mtx,f:u32,r:u32){event(23+pid(j));for row in m{for x in row{event(x.to_bits())}}event(f);event(r)}
unsafe fn dump(seed:u32,stage:u32,out:&mut String){let mut h=2166136261;word(&mut h,EV);word(&mut h,OBJECTS.len() as u32);word(&mut h,pid(current_jobj));word(&mut h,!default_class.is_null() as u32);word(&mut h,!ufc_callbacks.is_null() as u32);
 for &p in &OBJECTS{let j=&*p;for n in [pid(j.next),pid(j.parent),pid(j.child),j.flags,j.object.ref_count as u32,j.object.ref_count_individual as u32]{word(&mut h,n)}
 for f in [j.rotate.x,j.rotate.y,j.rotate.z,j.rotate.w,j.scale.x,j.scale.y,j.scale.z,j.translate.x,j.translate.y,j.translate.z]{word(&mut h,f.to_bits())}for r in j.mtx{for f in r{word(&mut h,f.to_bits())}}
 word(&mut h,!j.scl.is_null() as u32);word(&mut h,!j.envelopemtx.is_null() as u32);if !j.envelopemtx.is_null(){for r in *(j.envelopemtx as *const Mtx){for f in r{word(&mut h,f.to_bits())}}}word(&mut h,!j.u.dobj.is_null() as u32);word(&mut h,!j.robj.is_null() as u32);word(&mut h,!j.aobj.is_null() as u32);}
 writeln!(out,"{seed:x} {stage:x} {h:08x}").unwrap();
}
unsafe fn run(seed:u32,out:&mut String){
 EV=0;STATUS=0;OBJECTS.clear();IDS.clear();hsdJObj=HSD_JObjInfo::zeroed();default_class=null_mut();current_jobj=null_mut();ufc_callbacks=null_mut();
 PARENT.init=Some(parent_init);PARENT.release=Some(parent_release);PARENT.amnesia=Some(parent_amnesia);ext::hsdObj=PARENT;JObjInfoInit();HSD_JObjSetDefaultClass((&raw mut hsdJObj).cast());
 let mut d=[HSD_Joint::zeroed();4];let mut s=[_HSD_SList::zeroed();2];let mut m=mtx::identity();let mut local=HSD_JObj::zeroed();let mut r=HSD_RObj::zeroed();
 for(i,x)in d.iter_mut().enumerate(){x.flags=(1<<18)|(1<<28);x.scale=Vec{x:1.+(seed%3) as f32,y:2.,z:3.};x.rotation=Vec{x:seed as f32*0.001,y:i as f32*0.1,z:-0.25};x.position=Vec{x:i as f32,y:seed as f32,z:-2.};}
 d[0].child=&mut d[1];d[1].next=&mut d[2];d[0].next=&mut d[3];d[3].child=&mut d[1];d[3].flags|=1<<12;d[1].class_name=if seed&1!=0{c"x".as_ptr().cast_mut()}else{c"z".as_ptr().cast_mut()};d[2].flags|=match seed%3{0=>1<<5,1=>1<<14,_=>0};
 s[0].next=&mut s[1];s[0].data=0x123usize as *mut _;s[1].data=0x456usize as *mut _;
 match seed%3{0=>d[2].u.ptcl=s.as_mut_ptr(),1=>d[2].u.spline=0x100usize as *mut _,_=>d[2].u.dobjdesc=0x200usize as *mut _}
 d[0].robjdesc=0x300usize as *mut _;m[0][3]=seed as f32;d[0].mtx=m.as_mut_ptr();CAMERA.view_mtx=mtx::identity();
 let j=HSD_JObjLoadJoint(d.as_mut_ptr());dump(seed,0,out);
 let mut h=2166136261;word(&mut h,(hsdJObj.load.unwrap() as usize==JObjLoad as usize) as u32);word(&mut h,hsdJObj.make_mtx.is_some() as u32);word(&mut h,(hsdJObj.release_child.unwrap() as usize==JObjReleaseChild as usize) as u32);word(&mut h,has_scl(null_mut()) as u32);word(&mut h,has_scl(j) as u32);word(&mut h,(robj_set_next(null_mut(),&mut r)==&raw mut r) as u32);word(&mut h,(robj_set_next(&mut r,null_mut())==&raw mut r) as u32);word(&mut h,r.next.is_null() as u32);if seed%3==0{word(&mut h,s[0].data as usize as u32);word(&mut h,s[1].data as usize as u32)}writeln!(out,"{seed:x} 1 {h:08x}").unwrap();
 local.object.parent.class_info=(&raw mut hsdJObj).cast();STATUS=-3;word(&mut h,JObjInit((&raw mut local).cast()) as u32);STATUS=0;word(&mut h,JObjInit((&raw mut local).cast()) as u32);word(&mut h,local.flags);word(&mut h,local.scale.x.to_bits());writeln!(out,"{seed:x} 2 {h:08x}").unwrap();
 HSD_JObjSetCurrent(j);HSD_JObjSetCurrent(OBJECTS[3]);HSD_JObjSetCurrent(null_mut());dump(seed,3,out);
 for(i,&p)in OBJECTS.iter().enumerate(){(*p).flags&=!(1<<6);(*p).mtx[0][3]=i as f32;}
 HSD_JObjDispAll(j,&m,1,seed);HSD_JObjDispAll(OBJECTS[3],&m,1,seed);(*OBJECTS[3]).flags|=1<<4;HSD_JObjDispAll(OBJECTS[3],&m,1,seed);dump(seed,4,out);
 (*OBJECTS[2]).aobj=Box::into_raw(Box::new(HSD_AObj::zeroed()));JObjReleaseChild(OBJECTS[2]);dump(seed,5,out);
 for i in 0..4{local.object.ref_count=if i==3{65535}else{i};local.object.ref_count_individual=(seed%3) as u16;local.child=null_mut();local.parent=null_mut();local.robj=null_mut();local.aobj=null_mut();local.u.dobj=null_mut();local.scl=null_mut();local.envelopemtx=null_mut();local.id=0;
 HSD_JObjUnref(&mut local);h=2166136261;word(&mut h,EV);word(&mut h,local.object.ref_count as u32);word(&mut h,local.object.ref_count_individual as u32);HSD_JObjUnrefThis(&mut local);word(&mut h,EV);word(&mut h,local.object.ref_count as u32);word(&mut h,local.object.ref_count_individual as u32);writeln!(out,"{seed:x} {:x} {h:08x}",6+i).unwrap();}
 ufc_callbacks=s.as_mut_ptr();current_jobj=j;JObjAmnesia((&raw mut hsdJObj).cast());dump(seed,10,out);JObjRelease(j.cast());dump(seed,11,out);
}
#[test]
fn shard_jobj_native_state_oracle(){unsafe{
 use ext::hook::*;use std::ffi::c_void;melee_runtime::tu::jobj::use_ext_data=true;
 hsdNew=Some(|i|new(i).cast::<c_void>());hsdSearchClassInfo=Some(|n|search(n as *mut i8));hsdIsDescendantOf=Some(|i,p|descendant(i.cast(),p.cast()) as i32);hsdInitClassInfo=Some(|i,p,l,n,is,os|init_info(i,p,l,n as *const i8,is,os));
 HSD_DObjLoadDesc=Some(|d|dload(d));HSD_RObjLoadDesc=Some(|d|rload(d));HSD_DObjResolveRefsAll=Some(|a,b|dresolve(a,b));HSD_RObjResolveRefsAll=Some(|a,b|rresolve(a,b));HSD_DObjRemoveAll=Some(|a|dremove(a));HSD_RObjRemoveAll=Some(|a|rremove(a));HSD_AObjRemove=Some(|a|aremove(a));
 HSD_IDInsertToTable=Some(|t,id,j|insert(t.cast(),id as usize,j.cast()));HSD_IDGetDataFromTable=Some(|t,id,s|get(t.cast(),id as usize,s).cast::<c_void>());HSD_IDRemoveByIDFromTable=Some(|t,id|remove(t.cast(),id as usize));
 HSD_CObjGetCurrent=Some(||camera());HSD_JObjDisp=Some(|j,m,f,r|display(j,&*(m as *const Mtx),f,r));HSD_MtxFree=Some(|_|{});HSD_VecFree=Some(|_|{});HSD_MtxAlloc=Some(||Box::into_raw(Box::new([[0f32;4];3])).cast::<c_void>());
 let mut actual=String::new();for seed in 0..128{run(seed,&mut actual)}
 let expected=include_str!("data/oracle_shard_jobj.txt");for (i,(a,e))in actual.lines().zip(expected.lines()).enumerate(){assert_eq!(a,e,"oracle row {i}");}assert_eq!(actual.lines().count(),expected.lines().count());
}}
