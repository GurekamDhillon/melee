//! C functions covered (C name = Rust implementation the tu::jobj / tu::robj wrappers delegate to): JObjAnimAll = JObjRt::anim_all_sub (op 7),
//! HSD_JObjRemoveAnimAllByFlags (op 10); the RObj legs HSD_RObjReqAnimAllByFlags / HSD_RObjRemoveAnimAllByFlags run inside those walkers.
//! Actual jobj.c animation walkers over actual aobj.c/fobj.c; DObj calls logged by value.
use melee_runtime::{jobj::*,jobj_rt::*,robj::RObjHost,aobj::*,fobj::*,mtx::Vec3};
use std::fmt::Write;
#[derive(Default)]struct Host(u32);
impl Host{fn ev(&mut self,n:u32){self.0=self.0.wrapping_mul(33).wrapping_add(n)}}
impl RObjHost for Host{
 fn jobj_ref_this(&mut self,_:JObjId){}fn jobj_unref_this(&mut self,_:Option<JObjId>){}
 fn byte_code_eval(&mut self,_:Option<&[u8]>,_:&[f32],_:u32)->f32{panic!("unexpected bytecode")}
}
impl JObjExt for Host{
 type D=u32;type MatAnim=u32;type ShapeAnim=u32;
 fn dobj_add_anim_all(&mut self,d:&[u32],m:Option<&u32>,s:Option<&u32>){self.ev(1);self.ev(d.first().copied().unwrap_or(0));self.ev(m.copied().unwrap_or(0));self.ev(s.copied().unwrap_or(0));}
 fn dobj_anim_all(&mut self,d:&[u32]){self.ev(2);self.ev(d.first().copied().unwrap_or(0))}
 fn dobj_remove_anim_all_by_flags(&mut self,d:&[u32],f:u32){self.ev(3);self.ev(d.first().copied().unwrap_or(0));self.ev(f)}
 fn dobj_req_anim_all_by_flags(&mut self,d:&[u32],x:f32,f:u32){self.ev(4);self.ev(d.first().copied().unwrap_or(0));self.ev(x.to_bits());self.ev(f)}
 fn spline_point(&mut self,_:JObjId,_:f32)->Vec3{panic!("unexpected path")}
 fn aobj_resolve_obj(&mut self,_:u32)->Option<JObjId>{None}
 fn jobj_ref(&mut self,_:Option<JObjId>){}fn jobj_unref(&mut self,_:Option<JObjId>){}
}
fn word(h:&mut u32,n:u32){*h=(*h^n).wrapping_mul(16777619)}
fn anim(seed:u32,i:u32)->HsdAnimJoint{HsdAnimJoint{flags:(seed+i)&1,aobjdesc:Some(HsdAObjDesc{flags:if seed&8!=0{1<<29}else{0},end_frame:(3+i) as f32,fobjdesc:vec![],obj_id:0}),..Default::default()}}
fn desc(seed:u32)->HsdAnimJoint{let mut root=anim(seed,0);let mut child=anim(seed,1);child.next=Some(Box::new(anim(seed,2)));child.child=Some(Box::new(anim(seed,3)));root.child=Some(Box::new(child));root}
fn sub(i:u32)->HsdSubAnimJoint<u32>{HsdSubAnimJoint{child:None,next:None,anim:Some(i)}}
fn subdesc(offset:u32)->HsdSubAnimJoint<u32>{let mut root=sub(offset);let mut child=sub(offset+1);child.next=Some(Box::new(sub(offset+2)));child.child=Some(Box::new(sub(offset+3)));root.child=Some(Box::new(child));root}
#[test]fn shard_jobj_animation_state_oracle(){
 let mut out=String::new();
 for seed in 0u32..128{
  let mut t=JObjTree::<u32,u32>::new();let mut rt=JObjRt::new(Host::default());for i in 0..5{let j=t.alloc();t.get_mut(j).dobj=vec![i+1];}
  t.get_mut(JObjId(0)).child=Some(JObjId(1));t.get_mut(JObjId(1)).parent=Some(JObjId(0));t.get_mut(JObjId(1)).next=Some(JObjId(2));t.get_mut(JObjId(2)).parent=Some(JObjId(0));t.get_mut(JObjId(1)).child=Some(JObjId(3));t.get_mut(JObjId(3)).parent=Some(JObjId(1));t.get_mut(JObjId(3)).next=Some(JObjId(4));t.get_mut(JObjId(4)).parent=Some(JObjId(1));
  if seed&1!=0{t.get_mut(JObjId(1)).flags|=JOBJ_INSTANCE}if seed&2!=0{t.get_mut(JObjId(2)).flags|=JOBJ_PTCL}if seed&4!=0{t.get_mut(JObjId(4)).flags|=JOBJ_SPLINE}
  let a=desc(seed);let m=subdesc(1);let s=subdesc(9);
  let mut sort=HsdAObj::<JObjId>::alloc();for i in 0..5{sort.fobj.push(HsdFObj::load_desc(&HsdFObjDesc{length:0,startframe:0.,type_:if i==seed%6{12}else{5+i as u8},frac_value:0,frac_slope:0,ad:std::sync::Arc::from([])}));}
  sort_anim::<JObjId>(None);sort_anim(Some(&mut sort));let mut h=2166136261;for f in sort.fobj{word(&mut h,f.obj_type as u32)}writeln!(out,"{seed:x} 0 {h:08x}").unwrap();
  let mut x=seed+12345;
  for k in 0..78{
   let op=k%13;x=x.wrapping_mul(1664525).wrapping_add(1013904223);let idx=(x>>16)%6;x=x.wrapping_mul(1664525).wrapping_add(1013904223);let flags=(x>>16)&0x7ff;let frame=((x&31) as i32-8) as f32;let j=(idx<5).then_some(JObjId(idx));
   match op{0=>rt.add_anim(&mut t,j,Some(&a),Some(&m),Some(&s)),1=>rt.add_anim_all(&mut t,j,Some(&a),if seed&16!=0{None}else{Some(&m)},if seed&32!=0{None}else{Some(&s)}),2=>rt.req_anim_by_flags(&mut t,j,flags,frame),3=>rt.req_anim_all_by_flags(&mut t,j,flags,frame),4=>rt.req_anim(&mut t,j,frame),5=>rt.req_anim_all(&mut t,j,frame),6=>rt.anim(&mut t,j),7=>rt.anim_all_sub(&mut t,j),8=>rt.anim_all(&mut t,j),9=>rt.remove_anim_by_flags(&mut t,j,flags),10=>rt.remove_anim_all_by_flags(&mut t,j,flags),11=>rt.remove_anim(&mut t,j),_=>rt.remove_anim_all(&mut t,j)}
   h=2166136261;word(&mut h,rt.ext.borrow().0);for o in &t.nodes{word(&mut h,o.flags);word(&mut h,o.aobj.is_some() as u32);if let Some(a)=&o.aobj{word(&mut h,a.flags);for f in [a.curr_frame,a.rewind_frame,a.end_frame,a.framerate]{word(&mut h,f.to_bits())}}}writeln!(out,"{seed:x} {:x} {h:08x}",k+1).unwrap();
  }
 }
 let expected=include_str!("data/oracle_shard_jobj_anim.txt");for(i,(a,e))in out.lines().zip(expected.lines()).enumerate(){assert_eq!(a,e,"oracle row {i}")}assert_eq!(out.lines().count(),expected.lines().count());
}
