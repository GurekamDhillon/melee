#![allow(non_snake_case,non_upper_case_globals,dead_code,unused_imports,static_mut_refs)]
pub use melee_runtime::{cabi,gekko};
#[path="../src/tu/synth.rs"] mod synth;
use synth::*;use melee_types::*;use core::ffi::c_void;
static mut seed:u32=0;static mut hh:u32=0;static mut th:u32=0;static mut acq:u32=0;static mut acquire_mode:u32=0;
static mut voices:[AXVPB;64]=unsafe{core::mem::zeroed()};
static mut heap:[u64;1024]=[0;1024];static mut sample:[u64;32]=[0;32];static mut group:[u64;64]=[0;64];static mut header:[u32;64]=[0;64];
unsafe fn rnd()->u32{seed^=seed<<13;seed^=seed>>17;seed^=seed<<5;seed}
unsafe fn rf()->f32{((rnd()%4096) as i32-1024) as f32/1024.0}
unsafe fn h(v:u32){hh=(hh^v).wrapping_mul(16777619);}unsafe fn t(v:u32){th=(th^v).wrapping_mul(16777619);}
unsafe extern "C" fn cb1(v:i32){t(101);t(v as u32);}unsafe extern "C" fn cb2(v:i32){t(102);t(v as u32);}unsafe extern "C" fn cb3(v:i32){t(103);t(v as u32);}
unsafe extern "C" fn load_cb(a:i32,b:i32){t(104);t(a as u32);t(b as u32);}unsafe extern "C" fn wait_cb(){t(105);HSD_Synth_804D772C-=1;}
unsafe fn pi(v:usize)->u32{
let b=(&raw const hsd_SynthSFXNodes) as usize;if v>=b&&v<b+core::mem::size_of_val(&hsd_SynthSFXNodes){return 0x100000+(v-b) as u32;}
let b=(&raw const voices) as usize;if v>=b&&v<b+core::mem::size_of_val(&voices){return 0x200000+(v-b) as u32;}
let b=(&raw const heap) as usize;if v>=b&&v<b+core::mem::size_of_val(&heap){return 0x300000+(v-b) as u32;}
let b=(&raw const sample) as usize;if v>=b&&v<b+core::mem::size_of_val(&sample){return 0x400000+(v-b) as u32;}
let b=(&raw const group) as usize;if v>=b&&v<b+core::mem::size_of_val(&group){return 0x500000+(v-b) as u32;}
let b=(&raw const header) as usize;if v>=b&&v<b+core::mem::size_of_val(&header){return 0x600000+(v-b) as u32;}
let b=(&raw const hsd_SynthSFXLoadBuf) as usize;if v>=b&&v<b+core::mem::size_of_val(&hsd_SynthSFXLoadBuf){return 0x700000+(v-b) as u32;}
let b=(&raw const pstHakoHeader) as usize;if v>=b&&v<b+core::mem::size_of_val(&pstHakoHeader){return 0x800000+(v-b) as u32;}
v as u32}
unsafe fn fi(v:usize)->u32{
if v==HSD_SynthSFXHeaderLoadCallback as *const () as usize{return 1;}
if v==HSD_SynthSFXSampleLoadCallback as *const () as usize{return 2;}
if v==HSD_SynthSFXGroupDataReaddressCallback as *const () as usize{return 3;}
if v==dropcallback as *const () as usize{return 4;}
if v==HSD_SynthCallback as *const () as usize{return 5;}
if v==HSD_SynthResetStreamCounters as *const () as usize{return 6;}
if v==HSD_SynthPStreamHakoHeaderCallback as *const () as usize{return 7;}
if v==HSD_SynthPStreamFirstHakoHeaderCallback as *const () as usize{return 9;}
if v==HSD_SynthPStreamHeaderCallback as *const () as usize{return 10;}
if v==cb1 as *const () as usize{return 11;}
if v==cb2 as *const () as usize{return 12;}
if v==cb3 as *const () as usize{return 13;}
if v==load_cb as *const () as usize{return 14;}
if v==wait_cb as *const () as usize{return 15;}
if v==synth::first_data_adapter as *const () as usize{return 8;}
if v==0{0}else{999}}
unsafe fn hash_AXVPB(p:*const AXVPB){
h(pi(((*p).next) as usize));
h(pi(((*p).prev) as usize));
h(pi(((*p).next1) as usize));
h(((*p).priority) as u32);
h(fi(((*p).callback).map_or(0,|f|f as usize)));
h(((*p).userContext) as u32);
h(((*p).index) as u32);
h(((*p).sync) as u32);
h(((*p).depop) as u32);
h(((*p).updateMS) as u32);
h(((*p).updateCounter) as u32);
h(((*p).updateTotal) as u32);
h(pi(((*p).updateWrite) as usize));
for j0 in 0..128 {
h(((*p).updateData[j0]) as u32);
}
h(pi(((*p).itdBuffer) as usize));
h(((*p).pb.nextHi) as u32);
h(((*p).pb.nextLo) as u32);
h(((*p).pb.currHi) as u32);
h(((*p).pb.currLo) as u32);
h(((*p).pb.srcSelect) as u32);
h(((*p).pb.coefSelect) as u32);
h(((*p).pb.mixerCtrl) as u32);
h(((*p).pb.state) as u32);
h(((*p).pb.mix.vL) as u32);
h(((*p).pb.mix.vDeltaL) as u32);
h(((*p).pb.mix.vR) as u32);
h(((*p).pb.mix.vDeltaR) as u32);
h(((*p).pb.mix.vAuxAL) as u32);
h(((*p).pb.mix.vDeltaAuxAL) as u32);
h(((*p).pb.mix.vAuxAR) as u32);
h(((*p).pb.mix.vDeltaAuxAR) as u32);
h(((*p).pb.mix.vAuxBL) as u32);
h(((*p).pb.mix.vDeltaAuxBL) as u32);
h(((*p).pb.mix.vAuxBR) as u32);
h(((*p).pb.mix.vDeltaAuxBR) as u32);
h(((*p).pb.mix.vAuxBS) as u32);
h(((*p).pb.mix.vDeltaAuxBS) as u32);
h(((*p).pb.mix.vS) as u32);
h(((*p).pb.mix.vDeltaS) as u32);
h(((*p).pb.mix.vAuxAS) as u32);
h(((*p).pb.mix.vDeltaAuxAS) as u32);
h(((*p).pb.itd.flag) as u32);
h(((*p).pb.itd.bufferHi) as u32);
h(((*p).pb.itd.bufferLo) as u32);
h(((*p).pb.itd.shiftL) as u32);
h(((*p).pb.itd.shiftR) as u32);
h(((*p).pb.itd.targetShiftL) as u32);
h(((*p).pb.itd.targetShiftR) as u32);
for j0 in 0..5 {
h(((*p).pb.update.updNum[j0]) as u32);
}
h(((*p).pb.update.dataHi) as u32);
h(((*p).pb.update.dataLo) as u32);
h(((*p).pb.dpop.aL) as u32);
h(((*p).pb.dpop.aAuxAL) as u32);
h(((*p).pb.dpop.aAuxBL) as u32);
h(((*p).pb.dpop.aR) as u32);
h(((*p).pb.dpop.aAuxAR) as u32);
h(((*p).pb.dpop.aAuxBR) as u32);
h(((*p).pb.dpop.aS) as u32);
h(((*p).pb.dpop.aAuxAS) as u32);
h(((*p).pb.dpop.aAuxBS) as u32);
h(((*p).pb.ve.currentVolume) as u32);
h(((*p).pb.ve.currentDelta) as u32);
h(((*p).pb.fir.numCoefs) as u32);
h(((*p).pb.fir.coefsHi) as u32);
h(((*p).pb.fir.coefsLo) as u32);
h(((*p).pb.addr.loopFlag) as u32);
h(((*p).pb.addr.format) as u32);
h(((*p).pb.addr.loopAddressHi) as u32);
h(((*p).pb.addr.loopAddressLo) as u32);
h(((*p).pb.addr.endAddressHi) as u32);
h(((*p).pb.addr.endAddressLo) as u32);
h(((*p).pb.addr.currentAddressHi) as u32);
h(((*p).pb.addr.currentAddressLo) as u32);
for j0 in 0..8 {
for j1 in 0..2 {
h(((*p).pb.adpcm.a[j0][j1]) as u32);
}
}
h(((*p).pb.adpcm.gain) as u32);
h(((*p).pb.adpcm.pred_scale) as u32);
h(((*p).pb.adpcm.yn1) as u32);
h(((*p).pb.adpcm.yn2) as u32);
h(((*p).pb.src.ratioHi) as u32);
h(((*p).pb.src.ratioLo) as u32);
h(((*p).pb.src.currentAddressFrac) as u32);
for j0 in 0..4 {
h(((*p).pb.src.last_samples[j0]) as u32);
}
h(((*p).pb.adpcmLoop.loop_pred_scale) as u32);
h(((*p).pb.adpcmLoop.loop_yn1) as u32);
h(((*p).pb.adpcmLoop.loop_yn2) as u32);
for j0 in 0..3 {
h(((*p).pb.pad[j0]) as u32);
}
}
unsafe fn hash_HSD_SynthSFXNode(p:*const HSD_SynthSFXNode){
h(((*p).x0) as u32);
h(((*p).sfx_id) as u32);
h(((*p).pad8) as u32);
h(((*p).flags) as u32);
h(((*p).voice_count) as u32);
h(((*p).xB) as u32);
for j0 in 0..2 {
h(pi(((*p).voice[j0]) as usize));
}
h(((*p).x14).to_bits());
for j0 in 0..2 {
h(((*p).x18[j0]).to_bits());
}
h(pi(((*p).x20) as usize));
h(((*p).x24) as u32);
h(((*p).volume_update_pending) as u32);
h(((*p).x27) as u32);
h(((*p).unk28).to_bits());
for j0 in 0..2 {
h(((*p).user_vol[j0].volume).to_bits());
h(((*p).user_vol[j0].x4) as u32);
h((*p).user_vol[j0].x8_u.x8_float.to_bits());
}
h(((*p).x44).to_bits());
h(((*p).x48).to_bits());
h(((*p).x4C).to_bits());
}
unsafe fn hash_AXPBMIX(p:*const AXPBMIX){
h(((*p).vL) as u32);
h(((*p).vDeltaL) as u32);
h(((*p).vR) as u32);
h(((*p).vDeltaR) as u32);
h(((*p).vAuxAL) as u32);
h(((*p).vDeltaAuxAL) as u32);
h(((*p).vAuxAR) as u32);
h(((*p).vDeltaAuxAR) as u32);
h(((*p).vAuxBL) as u32);
h(((*p).vDeltaAuxBL) as u32);
h(((*p).vAuxBR) as u32);
h(((*p).vDeltaAuxBR) as u32);
h(((*p).vAuxBS) as u32);
h(((*p).vDeltaAuxBS) as u32);
h(((*p).vS) as u32);
h(((*p).vDeltaS) as u32);
h(((*p).vAuxAS) as u32);
h(((*p).vDeltaAuxAS) as u32);
}
unsafe fn hash_AXPBVE(p:*const AXPBVE){
h(((*p).currentVolume) as u32);
h(((*p).currentDelta) as u32);
}
unsafe fn hash_AXPBSRC(p:*const AXPBSRC){
h(((*p).ratioHi) as u32);
h(((*p).ratioLo) as u32);
h(((*p).currentAddressFrac) as u32);
for j0 in 0..4 {
h(((*p).last_samples[j0]) as u32);
}
}
unsafe fn hash_AXPBADDR(p:*const AXPBADDR){
h(((*p).loopFlag) as u32);
h(((*p).format) as u32);
h(((*p).loopAddressHi) as u32);
h(((*p).loopAddressLo) as u32);
h(((*p).endAddressHi) as u32);
h(((*p).endAddressLo) as u32);
h(((*p).currentAddressHi) as u32);
h(((*p).currentAddressLo) as u32);
}
unsafe fn hash_AXPBADPCM(p:*const AXPBADPCM){
for j0 in 0..8 {
for j1 in 0..2 {
h(((*p).a[j0][j1]) as u32);
}
}
h(((*p).gain) as u32);
h(((*p).pred_scale) as u32);
h(((*p).yn1) as u32);
h(((*p).yn2) as u32);
}
unsafe fn hash_AXPBADPCMLOOP(p:*const AXPBADPCMLOOP){
h(((*p).loop_pred_scale) as u32);
h(((*p).loop_yn1) as u32);
h(((*p).loop_yn2) as u32);
}
unsafe fn hash_SfxLoadStreamNode(p:*const SfxLoadStreamNode){
h(pi(((*p).x0) as usize));
h(((*p).x4) as u32);
h(((*p).x8) as u32);
h(((*p).xC) as u32);
h(((*p).x10) as u32);
h(((*p).x14) as u32);
}
unsafe fn hash_HSD_SynthVoiceEntry(p:*const HSD_SynthVoiceEntry){
h(pi(((*p).next) as usize));
h(((*p).unk4) as u32);
h(((*p).unk8) as u32);
h(((*p).unkC) as u32);
h(((*p).x10.loopFlag) as u32);
h(((*p).x10.format) as u32);
h(((*p).x10.loopAddressHi) as u32);
h(((*p).x10.loopAddressLo) as u32);
h(((*p).x10.endAddressHi) as u32);
h(((*p).x10.endAddressLo) as u32);
h(((*p).x10.currentAddressHi) as u32);
h(((*p).x10.currentAddressLo) as u32);
for j0 in 0..8 {
for j1 in 0..2 {
h(((*p).x20.a[j0][j1]) as u32);
}
}
h(((*p).x20.gain) as u32);
h(((*p).x20.pred_scale) as u32);
h(((*p).x20.yn1) as u32);
h(((*p).x20.yn2) as u32);
h(((*p).x48.loop_pred_scale) as u32);
h(((*p).x48.loop_yn1) as u32);
h(((*p).x48.loop_yn2) as u32);
}
unsafe fn trace_AXVPB(p:*const AXVPB){let save=hh;hh=th;hash_AXVPB(p);th=hh;hh=save;}
unsafe fn trace_HSD_SynthSFXNode(p:*const HSD_SynthSFXNode){let save=hh;hh=th;hash_HSD_SynthSFXNode(p);th=hh;hh=save;}
unsafe fn trace_AXPBMIX(p:*const AXPBMIX){let save=hh;hh=th;hash_AXPBMIX(p);th=hh;hh=save;}
unsafe fn trace_AXPBVE(p:*const AXPBVE){let save=hh;hh=th;hash_AXPBVE(p);th=hh;hh=save;}
unsafe fn trace_AXPBSRC(p:*const AXPBSRC){let save=hh;hh=th;hash_AXPBSRC(p);th=hh;hh=save;}
unsafe fn trace_AXPBADDR(p:*const AXPBADDR){let save=hh;hh=th;hash_AXPBADDR(p);th=hh;hh=save;}
unsafe fn trace_AXPBADPCM(p:*const AXPBADPCM){let save=hh;hh=th;hash_AXPBADPCM(p);th=hh;hh=save;}
unsafe fn trace_AXPBADPCMLOOP(p:*const AXPBADPCMLOOP){let save=hh;hh=th;hash_AXPBADPCMLOOP(p);th=hh;hh=save;}
unsafe fn trace_SfxLoadStreamNode(p:*const SfxLoadStreamNode){let save=hh;hh=th;hash_SfxLoadStreamNode(p);th=hh;hh=save;}
unsafe fn trace_HSD_SynthVoiceEntry(p:*const HSD_SynthVoiceEntry){let save=hh;hh=th;hash_HSD_SynthVoiceEntry(p);th=hh;hh=save;}
fn stub_OSAllocFromHeap(heap_id:i32,size:usize)->*mut c_void{unsafe{t(1);t(heap_id as u32);t(size as u32);heap.as_mut_ptr().cast()}}
fn stub_OSFreeToHeap(heap_id:i32,p:*mut c_void)->(){unsafe{t(2);t(heap_id as u32);t(pi(p as usize));}}
fn stub_OSDisableInterrupts()->i32{unsafe{t(3);rnd() as _}}
fn stub_OSRestoreInterrupts(enabled:i32)->i32{unsafe{t(4);t(enabled as u32);rnd() as _}}
fn stub_OSGetSoundMode()->u32{unsafe{t(5);rnd() as _}}
fn stub_OSSetSoundMode(mode:u32)->(){unsafe{t(6);t(mode as u32);}}
fn stub_OSReport(_m:&str,_v:i32){}
fn stub_DVDConvertPathToEntrynum(filename:*const i8)->i32{unsafe{t(8);let mut q=filename.cast::<u8>();while *q!=0{t(*q as u32);q=q.add(1);}rnd() as _}}
fn stub_HSD_DevComRequest(file:i32,src:usize,dst:usize,size:usize,flags:u32,priority:i32,cb:DevCallback,args:usize)->i32{unsafe{t(9);t(file as u32);t(pi(src));t(pi(dst));t(size as u32);t(flags as u32);t(priority as u32);t(fi(cb.map_or(0,|f|f as usize)));t(pi(args));rnd() as _}}
fn stub_HSD_DevComCancelEx(req:i32,flags:u32,cb:DevCallback,args:usize)->i32{unsafe{t(10);t(req as u32);t(flags as u32);t(fi(cb.map_or(0,|f|f as usize)));t(args as u32);rnd() as _}}
fn stub_AXAcquireVoice(priority:u32,cb:Option<unsafe extern "C" fn(*mut c_void)>,context:u32)->*mut AXVPB{unsafe{t(11);t(priority as u32);t(fi(cb.map_or(0,|f|f as usize)));t(context as u32);let call=acq;acq+=1;if acquire_mode==1||(acquire_mode==2&&call==1){core::ptr::null_mut()}else{&raw mut voices[8+if acquire_mode==3{0}else{(call%2) as usize}]} }}
fn stub_AXFreeVoice(voice:*mut AXVPB)->(){unsafe{t(12);t(pi(voice as usize));}}
fn stub_AXSetVoicePriority(voice:*mut AXVPB,priority:u32)->(){unsafe{t(13);t(pi(voice as usize));t(priority as u32);}}
fn stub_AXSetVoiceVe(voice:*mut AXVPB,ve:*mut AXPBVE)->(){unsafe{t(14);t(pi(voice as usize));trace_AXPBVE(ve);}}
fn stub_AXSetVoiceVeDelta(voice:*mut AXVPB,delta:i16)->(){unsafe{t(15);t(pi(voice as usize));t(delta as u32);}}
fn stub_AXSetVoiceSrc(voice:*mut AXVPB,src:*mut AXPBSRC)->(){unsafe{t(16);t(pi(voice as usize));trace_AXPBSRC(src);}}
fn stub_AXSetVoiceSrcRatio(voice:*mut AXVPB,ratio:f32)->(){unsafe{t(17);t(pi(voice as usize));t(ratio.to_bits());}}
fn stub_AXSetVoiceAddr(voice:*mut AXVPB,addr:*mut AXPBADDR)->(){unsafe{t(18);t(pi(voice as usize));trace_AXPBADDR(addr);}}
fn stub_AXSetVoiceAdpcm(voice:*mut AXVPB,adpcm:*mut AXPBADPCM)->(){unsafe{t(19);t(pi(voice as usize));trace_AXPBADPCM(adpcm);}}
fn stub_AXSetVoiceAdpcmLoop(voice:*mut AXVPB,adpcm:*mut AXPBADPCMLOOP)->(){unsafe{t(20);t(pi(voice as usize));trace_AXPBADPCMLOOP(adpcm);}}
fn stub_AXSetVoiceState(voice:*mut AXVPB,state:u16)->(){unsafe{t(21);t(pi(voice as usize));t(state as u32);}}
fn stub_AXSetVoiceMix(voice:*mut AXVPB,mix:*mut AXPBMIX)->(){unsafe{t(22);t(pi(voice as usize));trace_AXPBMIX(mix);}}
fn stub_AXSetVoiceItdOn(voice:*mut AXVPB)->(){unsafe{t(23);t(pi(voice as usize));}}
fn stub_AXSetVoiceItdTarget(voice:*mut AXVPB,l:u16,r:u16)->(){unsafe{t(24);t(pi(voice as usize));t(l as u32);t(r as u32);}}
fn stub_AXSetVoiceEndAddr(voice:*mut AXVPB,addr:u32)->(){unsafe{t(25);t(pi(voice as usize));t(addr as u32);}}
fn stub_AXSetVoiceCurrentAddr(voice:*mut AXVPB,addr:u32)->(){unsafe{t(26);t(pi(voice as usize));t(addr as u32);}}
fn stub_AXSetVoiceLoopAddr(voice:*mut AXVPB,addr:u32)->(){unsafe{t(27);t(pi(voice as usize));t(addr as u32);}}
fn stub_AXSetVoiceLoop(voice:*mut AXVPB,enabled:u16)->(){unsafe{t(28);t(pi(voice as usize));t(enabled as u32);}}
fn stub_AXInit()->(){unsafe{t(29);}}
fn stub_AXRegisterCallback(cb:Option<unsafe extern "C" fn()>)->(){unsafe{t(30);t(fi(cb.map_or(0,|f|f as usize)));}}
fn stub_AISetDSPSampleRate(rate:u32)->(){unsafe{t(31);t(rate as u32);}}
fn stub_AISetStreamVolLeft(vol:u8)->(){unsafe{t(32);t(vol as u32);}}
fn stub_AISetStreamVolRight(vol:u8)->(){unsafe{t(33);t(vol as u32);}}
fn stub_ARAlloc(size:u32)->u32{unsafe{t(34);t(size as u32);rnd() as _}}
fn stub_wait_idle()->(){unsafe{t(35);HSD_Synth_804D7778=0;}}
fn stub_watch_page(p:*mut c_void,size:u32)->(){unsafe{t(36);t(pi(p as usize));t(size as u32);}}
unsafe fn setup(k:u32){HSD_Synth_804D6028=core::mem::zeroed();voicelist=core::mem::zeroed();hsd_SynthSFXDataHash=core::mem::zeroed();HSD_Synth_804C2A60=core::mem::zeroed();hsd_SynthSFXLoadBuf=core::mem::zeroed();HSD_Synth_804C2AE0=core::mem::zeroed();hsd_SynthSFXBank=core::mem::zeroed();hsd_SynthSFXBankHead=core::mem::zeroed();hsd_SynthSFXNodes=core::mem::zeroed();voicelist_1784=core::mem::zeroed();voicelist_1844=core::mem::zeroed();pstHakoHeader=core::mem::zeroed();HSD_Synth_804D7720=0;hsd_SynthSFXBankNum=0;hsd_SynthSFXBankAREnd=0;HSD_Synth_804D772C=0;HSD_Synth_804D7730=core::ptr::null_mut();HSD_Synth_804D7734=core::ptr::null_mut();HSD_Synth_804D7738=0;sfxGroupDataReaddressCounter=0;driverInactivatedCallback=None;driverMasterClockCallback=None;driverPauseCallback=None;HSD_Synth_804D774C=core::ptr::null_mut();HSD_Synth_804D7750=0;HSD_Synth_804D7754=0;HSD_Synth_804D7758=0;HSD_Synth_804D775C=0;HSD_Synth_804D7760=0;HSD_Synth_804D7764=0;HSD_Synth_804D7768=0;HSD_Synth_804D776C=0;HSD_Synth_804D7770=0;HSD_Synth_804D7774=0;HSD_Synth_804D7778=0;HSD_Synth_804D777C=0;HSD_Synth_804D7780=0;HSD_Synth_804D7784=0;lbl_80407FB4=core::mem::zeroed();HSD_Synth_804D6018=0;HSD_Synth_804D6030=0.0;HSD_Synth_80407FD8=core::mem::zeroed();voices=core::mem::zeroed();heap=[0;1024];sample=[0;32];group=[0;64];header=[0;64];seed=k+1;acq=0;acquire_mode=0;HSD_Synth_804D6018=3;HSD_Synth_804D6030=rf();HSD_Synth_80407FD8.ratioHi=1;hsd_SynthSFXBankNum=2;hsd_SynthSFXBankAREnd=0x100000;for i in 0..33{hsd_SynthSFXBankHead[i]=0x4000+i as i32*0x10000;}for i in 0..32{hsd_SynthSFXBank[i]=hsd_SynthSFXBankHead[i]+0x800;}for i in 0..64{voices[i].index=i as u32;voices[i].sync=rnd();voices[i].pb.state=(rnd()%2) as u16;voices[i].pb.itd.flag=(rnd()%2) as u16;let a=0x8000+(k%4)*0x20000;voices[i].pb.addr.currentAddressHi=(a>>16) as u16;voices[i].pb.addr.currentAddressLo=a as u16;}for i in 0..8{let n=&raw mut hsd_SynthSFXNodes[i];(*n).x0=64+i as i32;(*n).sfx_id=100+i as i32;(*n).flags=(rnd()%16) as u8;(*n).voice_count=(1+rnd()%2) as u8;(*n).xB=(rnd()%16) as u8;(*n).x14=rf();(*n).x18[0]=rf();(*n).x18[1]=rf();(*n).x24=rnd() as u16;(*n).x27=(1+rnd()%2) as u8;(*n).unk28=rf();(*n).x44=rf();(*n).x48=rf();(*n).x4C=rf();(*n).voice[0]=&raw mut voices[i];(*n).voice[1]=&raw mut voices[i+16];(*n).x20=if i<7{&raw mut hsd_SynthSFXNodes[i+1]}else{core::ptr::null_mut()};(*n).volume_update_pending=(k%2) as u8;(*n).user_vol[0].volume=rf();(*n).user_vol[0].x4=(rnd()%4) as i32;(*n).user_vol[1].volume=rf();(*n).user_vol[1].x4=(rnd()%4) as i32;(*n).user_vol[0].x8_u.x8_float=rf();(*n).user_vol[1].x8_u.x8=rnd() as u8;}HSD_Synth_804D774C=if k%2!=0{&raw mut hsd_SynthSFXNodes[0]}else{core::ptr::null_mut()};for i in 0..16{voicelist_1784[i].x1784=rf();voicelist_1784[i].x1788=rf();voicelist_1784[i].x178C=(rnd()%5) as i32;}HSD_Synth_804D7758=rnd()&65535;HSD_Synth_804D7754=k%2;HSD_Synth_804D7750=if k%2!=0{0x7fffffc0}else{128};HSD_Synth_804D775C=(k%128) as i32;HSD_Synth_804D7760=if k%5==0{-1}else{64};HSD_Synth_804D7764=200;HSD_Synth_804D7768=k%3;HSD_Synth_804D776C=k%3;HSD_Synth_804D7770=(k+1)%3;HSD_Synth_804D7774=(k+2)%3;HSD_Synth_804D7778=(k%2) as u8;HSD_Synth_804D777C=255;HSD_Synth_804D7780=0x4000;HSD_Synth_804D7784=0x1234;HSD_Synth_804D772C=(1+k%4) as i32;HSD_Synth_804D7738=(k%2) as i32;for i in 0..6{HSD_Synth_804C2A60[i].entrynum=100+i as i32;HSD_Synth_804C2A60[i].bankID=0;HSD_Synth_804C2A60[i].x8=Some(load_cb);HSD_Synth_804C2A60[i].xC=i as i32*5;}for i in 0..3{pstHakoHeader[i].x0=0x200;pstHakoHeader[i].x4=0x100;pstHakoHeader[i].x8=if k%3==0{-1}else{0x300+i as i32*32};}driverInactivatedCallback=Some(cb1);driverMasterClockCallback=Some(cb2);driverPauseCallback=Some(cb3);voicelist_1844[1]=64;HSD_Synth_804D7720=2;voicelist[0]=&raw mut voices[0];voicelist[1]=&raw mut voices[16];let s=sample.as_mut_ptr().cast::<HSD_SynthVoiceEntry>();(*s).unk4=100;(*s).unk8=(1+k%2) as i32;(*s).unkC=32000;(*s).x10.loopFlag=(k%2) as u16;(*s).x10.loopAddressLo=0x6000;(*s).x10.endAddressLo=0x7000;(*s).x10.currentAddressLo=0x6002;hsd_SynthSFXDataHash[100&31]=s.cast();let g=group.as_mut_ptr().cast::<SfxLoadStreamNode>();(*g).x4=400;(*g).x8=100;(*g).xC=1;(*g).x10=0x4800;(*g).x14=0x200;core::ptr::copy_nonoverlapping(s,g.add(1).cast::<HSD_SynthVoiceEntry>(),1);HSD_Synth_804C2AE0[0]=g.cast();header[2]=32000;header[3]=1+k%2;hsd_SynthSFXLoadBuf[0]=0x60;hsd_SynthSFXLoadBuf[1]=0x100;hsd_SynthSFXLoadBuf[2]=1;hsd_SynthSFXLoadBuf[3]=200;hsd_SynthSFXLoadBuf[4]=1;hsd_SynthSFXLoadBuf[5]=32000;HSD_Synth_804D7730=heap.as_mut_ptr().cast();th=2166136261;hh=2166136261;}
unsafe fn snapshot(){for j0 in 0..2 {h((HSD_Synth_804D6028[j0]) as u32);}for j0 in 0..64 {h(pi((voicelist[j0]) as usize));}for j0 in 0..32 {h(pi((hsd_SynthSFXDataHash[j0]) as usize));}for j0 in 0..6 {h((HSD_Synth_804C2A60[j0].entrynum) as u32);h((HSD_Synth_804C2A60[j0].bankID) as u32);h(fi((HSD_Synth_804C2A60[j0].x8).map_or(0,|f|f as usize)));h((HSD_Synth_804C2A60[j0].xC) as u32);}for j0 in 0..8 {h((hsd_SynthSFXLoadBuf[j0]) as u32);}for j0 in 0..64 {h(pi((HSD_Synth_804C2AE0[j0]) as usize));}for j0 in 0..32 {h((hsd_SynthSFXBank[j0]) as u32);}for j0 in 0..33 {h((hsd_SynthSFXBankHead[j0]) as u32);}for j0 in 0..64 {h((hsd_SynthSFXNodes[j0].x0) as u32);h((hsd_SynthSFXNodes[j0].sfx_id) as u32);h((hsd_SynthSFXNodes[j0].pad8) as u32);h((hsd_SynthSFXNodes[j0].flags) as u32);h((hsd_SynthSFXNodes[j0].voice_count) as u32);h((hsd_SynthSFXNodes[j0].xB) as u32);for j1 in 0..2 {h(pi((hsd_SynthSFXNodes[j0].voice[j1]) as usize));}h((hsd_SynthSFXNodes[j0].x14).to_bits());for j1 in 0..2 {h((hsd_SynthSFXNodes[j0].x18[j1]).to_bits());}h(pi((hsd_SynthSFXNodes[j0].x20) as usize));h((hsd_SynthSFXNodes[j0].x24) as u32);h((hsd_SynthSFXNodes[j0].volume_update_pending) as u32);h((hsd_SynthSFXNodes[j0].x27) as u32);h((hsd_SynthSFXNodes[j0].unk28).to_bits());for j1 in 0..2 {h((hsd_SynthSFXNodes[j0].user_vol[j1].volume).to_bits());h((hsd_SynthSFXNodes[j0].user_vol[j1].x4) as u32);h(hsd_SynthSFXNodes[j0].user_vol[j1].x8_u.x8_float.to_bits());}h((hsd_SynthSFXNodes[j0].x44).to_bits());h((hsd_SynthSFXNodes[j0].x48).to_bits());h((hsd_SynthSFXNodes[j0].x4C).to_bits());}for j0 in 0..16 {h((voicelist_1784[j0].x1784).to_bits());h((voicelist_1784[j0].x1788).to_bits());h((voicelist_1784[j0].x178C) as u32);}for j0 in 0..256 {h((voicelist_1844[j0]) as u32);}for j0 in 0..3 {h((pstHakoHeader[j0].x0) as u32);h((pstHakoHeader[j0].x4) as u32);h((pstHakoHeader[j0].x8) as u32);for j1 in 0..20 {h((pstHakoHeader[j0].pad[j1]) as u32);}}h((HSD_Synth_804D7720) as u32);h((hsd_SynthSFXBankNum) as u32);h((hsd_SynthSFXBankAREnd) as u32);h((HSD_Synth_804D772C) as u32);h(pi((HSD_Synth_804D7730) as usize));h(pi((HSD_Synth_804D7734) as usize));h((HSD_Synth_804D7738) as u32);h((sfxGroupDataReaddressCounter) as u32);h(fi((driverInactivatedCallback).map_or(0,|f|f as usize)));h(fi((driverMasterClockCallback).map_or(0,|f|f as usize)));h(fi((driverPauseCallback).map_or(0,|f|f as usize)));h(pi((HSD_Synth_804D774C) as usize));h((HSD_Synth_804D7750) as u32);h((HSD_Synth_804D7754) as u32);h((HSD_Synth_804D7758) as u32);h((HSD_Synth_804D775C) as u32);h((HSD_Synth_804D7760) as u32);h((HSD_Synth_804D7764) as u32);h((HSD_Synth_804D7768) as u32);h((HSD_Synth_804D776C) as u32);h((HSD_Synth_804D7770) as u32);h((HSD_Synth_804D7774) as u32);h((HSD_Synth_804D7778) as u32);h((HSD_Synth_804D777C) as u32);h((HSD_Synth_804D7780) as u32);h((HSD_Synth_804D7784) as u32);h((lbl_80407FB4.vL) as u32);h((lbl_80407FB4.vDeltaL) as u32);h((lbl_80407FB4.vR) as u32);h((lbl_80407FB4.vDeltaR) as u32);h((lbl_80407FB4.vAuxAL) as u32);h((lbl_80407FB4.vDeltaAuxAL) as u32);h((lbl_80407FB4.vAuxAR) as u32);h((lbl_80407FB4.vDeltaAuxAR) as u32);h((lbl_80407FB4.vAuxBL) as u32);h((lbl_80407FB4.vDeltaAuxBL) as u32);h((lbl_80407FB4.vAuxBR) as u32);h((lbl_80407FB4.vDeltaAuxBR) as u32);h((lbl_80407FB4.vAuxBS) as u32);h((lbl_80407FB4.vDeltaAuxBS) as u32);h((lbl_80407FB4.vS) as u32);h((lbl_80407FB4.vDeltaS) as u32);h((lbl_80407FB4.vAuxAS) as u32);h((lbl_80407FB4.vDeltaAuxAS) as u32);h((HSD_Synth_804D6018) as u32);h((HSD_Synth_804D6030).to_bits());h((HSD_Synth_80407FD8.ratioHi) as u32);h((HSD_Synth_80407FD8.ratioLo) as u32);h((HSD_Synth_80407FD8.currentAddressFrac) as u32);for j0 in 0..4 {h((HSD_Synth_80407FD8.last_samples[j0]) as u32);}for i in 0..64{hash_AXVPB(&raw const voices[i]);}hash_HSD_SynthVoiceEntry(sample.as_ptr().cast());hash_SfxLoadStreamNode(group.as_ptr().cast());hash_HSD_SynthVoiceEntry(group.as_ptr().cast::<SfxLoadStreamNode>().add(1).cast());for i in 0..64{h(header[i]);}{hash_SfxLoadStreamNode(heap.as_ptr().cast());hash_HSD_SynthVoiceEntry(heap.as_ptr().cast::<SfxLoadStreamNode>().add(1).cast());}}
#[test]fn synth_matches_native_c(){unsafe{ext::hook::OSAllocFromHeap=Some(stub_OSAllocFromHeap);ext::hook::OSFreeToHeap=Some(stub_OSFreeToHeap);ext::hook::OSDisableInterrupts=Some(stub_OSDisableInterrupts);ext::hook::OSRestoreInterrupts=Some(stub_OSRestoreInterrupts);ext::hook::OSGetSoundMode=Some(stub_OSGetSoundMode);ext::hook::OSSetSoundMode=Some(stub_OSSetSoundMode);ext::hook::OSReport=Some(stub_OSReport);ext::hook::DVDConvertPathToEntrynum=Some(stub_DVDConvertPathToEntrynum);ext::hook::HSD_DevComRequest=Some(stub_HSD_DevComRequest);ext::hook::HSD_DevComCancelEx=Some(stub_HSD_DevComCancelEx);ext::hook::AXAcquireVoice=Some(stub_AXAcquireVoice);ext::hook::AXFreeVoice=Some(stub_AXFreeVoice);ext::hook::AXSetVoicePriority=Some(stub_AXSetVoicePriority);ext::hook::AXSetVoiceVe=Some(stub_AXSetVoiceVe);ext::hook::AXSetVoiceVeDelta=Some(stub_AXSetVoiceVeDelta);ext::hook::AXSetVoiceSrc=Some(stub_AXSetVoiceSrc);ext::hook::AXSetVoiceSrcRatio=Some(stub_AXSetVoiceSrcRatio);ext::hook::AXSetVoiceAddr=Some(stub_AXSetVoiceAddr);ext::hook::AXSetVoiceAdpcm=Some(stub_AXSetVoiceAdpcm);ext::hook::AXSetVoiceAdpcmLoop=Some(stub_AXSetVoiceAdpcmLoop);ext::hook::AXSetVoiceState=Some(stub_AXSetVoiceState);ext::hook::AXSetVoiceMix=Some(stub_AXSetVoiceMix);ext::hook::AXSetVoiceItdOn=Some(stub_AXSetVoiceItdOn);ext::hook::AXSetVoiceItdTarget=Some(stub_AXSetVoiceItdTarget);ext::hook::AXSetVoiceEndAddr=Some(stub_AXSetVoiceEndAddr);ext::hook::AXSetVoiceCurrentAddr=Some(stub_AXSetVoiceCurrentAddr);ext::hook::AXSetVoiceLoopAddr=Some(stub_AXSetVoiceLoopAddr);ext::hook::AXSetVoiceLoop=Some(stub_AXSetVoiceLoop);ext::hook::AXInit=Some(stub_AXInit);ext::hook::AXRegisterCallback=Some(stub_AXRegisterCallback);ext::hook::AISetDSPSampleRate=Some(stub_AISetDSPSampleRate);ext::hook::AISetStreamVolLeft=Some(stub_AISetStreamVolLeft);ext::hook::AISetStreamVolRight=Some(stub_AISetStreamVolRight);ext::hook::ARAlloc=Some(stub_ARAlloc);ext::hook::wait_idle=Some(stub_wait_idle);ext::hook::watch_page=Some(stub_watch_page);
let names=["HSD_AudioMalloc","HSD_AudioFree","SfxLoadStreamDataSize","hsd_SynthSFXNodeAt","HSD_SynthSFXSampleLoadCallback","HSD_SynthSFXHeaderLoadCallback","HSD_SynthSFXLoadNewProc","HSD_SynthSFXBankDeflagSync","HSD_SynthCallback","HSD_SynthPStreamMasterClockCallback","HSD_SynthPStreamFirstHakoDataCallback","HSD_SynthPStreamStart_inline","HSD_SynthSFXLoad","HSD_SynthSFXWaitForLoadCompletion","HSD_SynthSFXGetPendingLoadCount","HSD_SynthSFXCancelLoad","HSD_SynthSFXAllocateBank","HSD_SynthSFXGroupDataUnlink","HSD_SynthSFXUnloadBank","HSD_SynthSFXDataUnlink","HSD_SynthSFXGroupDataRemove","HSD_SynthSFXGroupDataReaddressCallback","HSD_SynthSFXGroupDataReaddress","HSD_SynthSFXBankDeflag","HSD_SynthGetSoundMode","HSD_SynthSetSoundMode","HSD_SynthSFXStopNode","freeVoices","HSD_SynthSFXUpdatePitch","HSD_SynthSFXUpdateVolume","dropcallback","HSD_Synth_80389334","HSD_SynthSFXPlayWithGroup","getNode","HSD_SynthSFXKeyOff","HSD_SynthSFXPause","HSD_SynthSFXResume","HSD_SynthSFXCheck","stopRange","HSD_SynthSFXStopRange","HSD_SynthSFXSetVolumeFade","HSD_SynthSFXSetPan","HSD_SynthSFXSetMix","HSD_SynthSFXSetPitchRatio","HSD_SynthSFXSetPriority","user_vol_dst_offset","user_vol_src_offset","HSD_SynthSFXVolumeEnvelope","my_memzero","HSD_SynthSFXUpdateMix","updateAllVolume","HSD_SynthSFXUpdateAllVolume","HSD_SynthSFXSetDriverInactivatedCallback","HSD_SynthSFXSetDriverMasterClockCallback","HSD_SynthSFXSetDriverPauseCallback","HSD_SynthResetStreamCounters","HSD_SynthPStreamFirstHakoHeaderCallback","HSD_SynthPStreamHakoHeaderCallback","HSD_SynthPStreamMasterClockCallback_inline","HSD_SynthPStreamHeaderCallback","HSD_SynthPStreamStart","HSD_SynthStreamSetVolume","HSD_SynthInit"];
for row in include_str!("data/tu_synth.txt").lines(){let w:std::vec::Vec<&str>=row.split_whitespace().collect();let op:usize=w[0].parse().unwrap();let k:u32=w[1].parse().unwrap();setup(k);let mut result=0u32;match op{0=>{result=pi(HSD_AudioMalloc(128) as usize);},1=>{HSD_AudioFree(heap.as_mut_ptr().cast());},2=>{result=(SfxLoadStreamDataSize(k.wrapping_mul(0x1020304) as i32)) as u32;},3=>{result=pi(hsd_SynthSFXNodeAt((k%68) as i32-2) as usize);},4=>{HSD_SynthSFXSampleLoadCallback(0,0,core::ptr::null_mut(),0);},5=>{if k%4==0{hsd_SynthSFXBank[0]=hsd_SynthSFXBankHead[1]-64;}HSD_SynthSFXHeaderLoadCallback(0,0,core::ptr::null_mut(),0);},6=>{HSD_SynthSFXLoadNewProc();},7=>{HSD_SynthSFXBankDeflagSync();},8=>{HSD_SynthCallback();},9=>{HSD_SynthPStreamMasterClockCallback();},10=>{HSD_SynthPStreamFirstHakoDataCallback();},11=>{HSD_SynthPStreamStart_inline();},12=>{result=(HSD_SynthSFXLoad(c"fixture".as_ptr(),0,Some(load_cb),3)) as u32;},13=>{HSD_SynthSFXWaitForLoadCompletion(Some(wait_cb));},14=>{result=(HSD_SynthSFXGetPendingLoadCount()) as u32;},15=>{result=(HSD_SynthSFXCancelLoad(100+(k%6) as i32)) as u32;},16=>{HSD_SynthSFXAllocateBank(0x2000+k as i32);},17=>{HSD_SynthSFXGroupDataUnlink(group.as_mut_ptr().cast());},18=>{HSD_SynthSFXUnloadBank(0);},19=>{HSD_SynthSFXDataUnlink(100+(k%2) as i32);},20=>{HSD_SynthSFXGroupDataRemove(400+(k%2) as i32);},21=>{HSD_SynthSFXGroupDataReaddressCallback(0,0,core::ptr::null_mut(),0);},22=>{HSD_SynthSFXGroupDataReaddress(group.as_mut_ptr().cast(),0x5000usize as *mut _);},23=>{HSD_SynthSFXBankDeflag(0);},24=>{result=(HSD_SynthGetSoundMode()) as u32;},25=>{HSD_SynthSetSoundMode((k%2) as i32);},26=>{HSD_SynthSFXStopNode(&raw mut hsd_SynthSFXNodes[0]);},27=>{freeVoices(&raw mut hsd_SynthSFXNodes[0]);},28=>{HSD_SynthSFXUpdatePitch(&raw mut hsd_SynthSFXNodes[0]);},29=>{HSD_SynthSFXUpdateVolume(&raw mut hsd_SynthSFXNodes[0]);},30=>{dropcallback((&raw mut voices[if k%2!=0{0}else{16}]).cast());},31=>{acquire_mode=k%4;result=(HSD_Synth_80389334(100,k as u8,k.wrapping_mul(3) as u8,k.wrapping_mul(5) as u8,20,(k%16) as i32,0.75,1.25,0.5,0.25,0.125)) as u32;},32=>{acquire_mode=k%4;result=(HSD_SynthSFXPlayWithGroup(100,k as u8,k.wrapping_mul(3) as u8,k.wrapping_mul(5) as u8,20,(k%16) as i32,(k%2) as i32,0.75,1.25,0.5,0.25,0.125)) as u32;},33=>{result=pi(getNode((63+k%10) as i32) as usize);},34=>{HSD_SynthSFXKeyOff((63+k%10) as i32);},35=>{HSD_SynthSFXPause((63+k%10) as i32);},36=>{HSD_SynthSFXResume((63+k%10) as i32);},37=>{result=(HSD_SynthSFXCheck((63+k%10) as i32)) as u32;},38=>{stopRange(0x8000,0x28000);},39=>{HSD_SynthSFXStopRange(0);},40=>{HSD_SynthSFXSetVolumeFade(64,k as u8,(k%2) as i32);},41=>{HSD_SynthSFXSetPan(64,k as u8);},42=>{HSD_SynthSFXSetMix(64,0.25,0.5,0.75);},43=>{HSD_SynthSFXSetPitchRatio(64,(k%2) as i32,k as f32/128.0);},44=>{HSD_SynthSFXSetPriority(64,(k%32) as i32);},45=>{result=(user_vol_dst_offset(k.wrapping_mul(0x1020304) as i32)) as u32;},46=>{result=(user_vol_src_offset(k.wrapping_mul(0x1020304) as i32)) as u32;},47=>{if k%4==0{hsd_SynthSFXNodes[0].x24=0;hsd_SynthSFXNodes[0].flags=(k%16) as u8;hsd_SynthSFXNodes[0].user_vol[0].x8_u.x8_float=0.0;hsd_SynthSFXNodes[0].user_vol[0].x4=0;hsd_SynthSFXNodes[0].user_vol[1].x4=0;voicelist_1784[hsd_SynthSFXNodes[0].xB as usize].x178C=0;hsd_SynthSFXNodes[0].x20=core::ptr::null_mut();HSD_Synth_804D774C=&raw mut hsd_SynthSFXNodes[0];}result=(HSD_SynthSFXVolumeEnvelope()) as u32;},48=>{my_memzero(header.as_mut_ptr().cast(),(k%32) as usize);},49=>{HSD_SynthSFXUpdateMix(&raw mut hsd_SynthSFXNodes[0],(k%2) as i32);},50=>{updateAllVolume(k.wrapping_mul(0x1020304));},51=>{HSD_SynthSFXUpdateAllVolume(k as i32,(k%10) as u16,(k%16) as i32);},52=>{HSD_SynthSFXSetDriverInactivatedCallback(Some(cb3));},53=>{HSD_SynthSFXSetDriverMasterClockCallback(Some(cb3));},54=>{HSD_SynthSFXSetDriverPauseCallback(Some(cb3));},55=>{HSD_SynthResetStreamCounters(0,0x500,core::ptr::null_mut(),0);},56=>{HSD_SynthPStreamFirstHakoHeaderCallback(0,0x500,core::ptr::null_mut(),0);},57=>{HSD_SynthPStreamHakoHeaderCallback(0,0x500,core::ptr::null_mut(),0);},58=>{HSD_SynthPStreamMasterClockCallback_inline(k%3);},59=>{HSD_SynthPStreamHeaderCallback(0,0,header.as_mut_ptr().cast(),0);},60=>{result=(HSD_SynthPStreamStart(500,k as u8,k.wrapping_mul(3) as u8,(k%16) as i32)) as u32;},61=>{HSD_SynthStreamSetVolume(k as f32/256.0);},62=>{HSD_SynthInit(0,0,0,0x40000);},_=>unreachable!()};hh=2166136261;snapshot();let expected=[u32::from_str_radix(w[2],16).unwrap(),u32::from_str_radix(w[3],16).unwrap(),u32::from_str_radix(w[4],16).unwrap()];assert_eq!([result,th,hh],expected,"{} seed {}",names[op],k);}
}}
