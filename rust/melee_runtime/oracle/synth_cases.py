"""C/Rust paired fixtures for synth_oracle.py (executed in its generator context)."""
initc=[];initr=[]
def ini(c,r):initc.append(c);initr.append(r)
def assign(c,r,cv,rv=None):ini(c+'='+cv+';',r+'='+ (cv if rv is None else rv)+';')
for n,t in globals:
    if t=='f32':assign(n,n,'0.0f','0.0')
    elif t.startswith('Option'):assign(n,n,'0','None')
    elif t.startswith('*'):assign(n,n,'0','core::ptr::null_mut()')
    elif t in ('i32','u32','u8'):assign(n,n,'0')
    else:ini(f'memset(&{n},0,sizeof {n});',f'{n}=core::mem::zeroed();')
ini('memset(voices,0,sizeof voices);memset(heap,0,sizeof heap);memset(sample,0,sizeof sample);memset(group,0,sizeof group);memset(header,0,sizeof header);',
    'voices=core::mem::zeroed();heap=[0;1024];sample=[0;32];group=[0;64];header=[0;64];')
ini('seed=k+1;acq=0;acquire_mode=0;','seed=k+1;acq=0;acquire_mode=0;')
assign('HSD_Synth_804D6018','HSD_Synth_804D6018','3')
assign('HSD_Synth_804D6030','HSD_Synth_804D6030','rf()')
assign('HSD_Synth_80407FD8.ratioHi','HSD_Synth_80407FD8.ratioHi','1')
assign('hsd_SynthSFXBankNum','hsd_SynthSFXBankNum','2')
assign('hsd_SynthSFXBankAREnd','hsd_SynthSFXBankAREnd','0x100000')
ini('for(int i=0;i<33;i++){hsd_SynthSFXBankHead[i]=0x4000+i*0x10000;}','for i in 0..33{hsd_SynthSFXBankHead[i]=0x4000+i as i32*0x10000;}')
ini('for(int i=0;i<32;i++){hsd_SynthSFXBank[i]=hsd_SynthSFXBankHead[i]+0x800;}','for i in 0..32{hsd_SynthSFXBank[i]=hsd_SynthSFXBankHead[i]+0x800;}')
ini('for(int i=0;i<64;i++){voices[i].index=i;voices[i].sync=rnd();voices[i].pb.state=rnd()%2;voices[i].pb.itd.flag=rnd()%2;native_set_pair(&voices[i].pb.addr.currentAddressHi,0x8000+(k%4)*0x20000);}',
    'for i in 0..64{voices[i].index=i as u32;voices[i].sync=rnd();voices[i].pb.state=(rnd()%2) as u16;voices[i].pb.itd.flag=(rnd()%2) as u16;let a=0x8000+(k%4)*0x20000;voices[i].pb.addr.currentAddressHi=(a>>16) as u16;voices[i].pb.addr.currentAddressLo=a as u16;}')
ini('for(int i=0;i<8;i++){struct HSD_SynthSFXNode*n=&hsd_SynthSFXNodes[i];','for i in 0..8{let n=&raw mut hsd_SynthSFXNodes[i];')
for f,cv,rv in [('x0','64+i','64+i as i32'),('sfx_id','100+i','100+i as i32'),('flags','rnd()%16','(rnd()%16) as u8'),('voice_count','1+rnd()%2','(1+rnd()%2) as u8'),('xB','rnd()%16','(rnd()%16) as u8'),('x14','rf()','rf()'),('x18[0]','rf()','rf()'),('x18[1]','rf()','rf()'),('x24','rnd()','rnd() as u16'),('x27','1+rnd()%2','(1+rnd()%2) as u8'),('unk28','rf()','rf()'),('x44','rf()','rf()'),('x48','rf()','rf()'),('x4C','rf()','rf()')]:assign('n->'+f,'(*n).'+f,cv,rv)
assign('n->voice[0]','(*n).voice[0]','&voices[i]','&raw mut voices[i]')
assign('n->voice[1]','(*n).voice[1]','&voices[i+16]','&raw mut voices[i+16]')
assign('n->x20','(*n).x20','i<7?&hsd_SynthSFXNodes[i+1]:0','if i<7{&raw mut hsd_SynthSFXNodes[i+1]}else{core::ptr::null_mut()}')
assign('n->volume_update_pending','(*n).volume_update_pending','k%2','(k%2) as u8')
for j in range(2):
    assign(f'n->user_vol[{j}].volume',f'(*n).user_vol[{j}].volume','rf()')
    assign(f'n->user_vol[{j}].x4',f'(*n).user_vol[{j}].x4','rnd()%4','(rnd()%4) as i32')
assign('n->user_vol[0].x8_float','(*n).user_vol[0].anon_8.x8_float','rf()')
assign('n->user_vol[1].x8','(*n).user_vol[1].anon_8.x8','rnd()','rnd() as u8')
ini('}','}')
assign('HSD_Synth_804D774C','HSD_Synth_804D774C','k%2?&hsd_SynthSFXNodes[0]:0','if k%2!=0{&raw mut hsd_SynthSFXNodes[0]}else{core::ptr::null_mut()}')
ini('for(int i=0;i<16;i++){voicelist_1784[i].x1784=rf();voicelist_1784[i].x1788=rf();voicelist_1784[i].x178C=rnd()%5;}',
    'for i in 0..16{voicelist_1784[i].x1784=rf();voicelist_1784[i].x1788=rf();voicelist_1784[i].x178C=(rnd()%5) as i32;}')
for n,cv,rv in [('HSD_Synth_804D7758','rnd()&65535','rnd()&65535'),('HSD_Synth_804D7754','k%2','k%2'),('HSD_Synth_804D7750','k%2?0x7fffffc0:128','if k%2!=0{0x7fffffc0}else{128}'),('HSD_Synth_804D775C','k%128','(k%128) as i32'),('HSD_Synth_804D7760','k%5==0?-1:64','if k%5==0{-1}else{64}'),('HSD_Synth_804D7764','200','200'),('HSD_Synth_804D7768','k%3','k%3'),('HSD_Synth_804D776C','k%3','k%3'),('HSD_Synth_804D7770','(k+1)%3','(k+1)%3'),('HSD_Synth_804D7774','(k+2)%3','(k+2)%3'),('HSD_Synth_804D7778','k%2','(k%2) as u8'),('HSD_Synth_804D777C','255','255'),('HSD_Synth_804D7780','0x4000','0x4000'),('HSD_Synth_804D7784','0x1234','0x1234'),('HSD_Synth_804D772C','1+k%4','(1+k%4) as i32'),('HSD_Synth_804D7738','k%2','(k%2) as i32')]:assign(n,n,cv,rv)
ini('for(int i=0;i<6;i++){HSD_Synth_804C2A60[i].entrynum=100+i;HSD_Synth_804C2A60[i].bankID=0;HSD_Synth_804C2A60[i].x8=load_cb;HSD_Synth_804C2A60[i].xC=i*5;}',
    'for i in 0..6{HSD_Synth_804C2A60[i].entrynum=100+i as i32;HSD_Synth_804C2A60[i].bankID=0;HSD_Synth_804C2A60[i].x8=Some(load_cb);HSD_Synth_804C2A60[i].xC=i as i32*5;}')
ini('for(int i=0;i<3;i++){pstHakoHeader[i].x0=0x200;pstHakoHeader[i].x4=0x100;pstHakoHeader[i].x8=k%3==0?-1:0x300+i*32;}',
    'for i in 0..3{pstHakoHeader[i].x0=0x200;pstHakoHeader[i].x4=0x100;pstHakoHeader[i].x8=if k%3==0{-1}else{0x300+i as i32*32};}')
for n,f in [('driverInactivatedCallback','cb1'),('driverMasterClockCallback','cb2'),('driverPauseCallback','cb3')]:assign(n,n,f,f'Some({f})')
assign('voicelist_1844[1]','voicelist_1844[1]','64')
assign('HSD_Synth_804D7720','HSD_Synth_804D7720','2')
ini('voicelist[0]=&voices[0];voicelist[1]=&voices[16];','voicelist[0]=&raw mut voices[0];voicelist[1]=&raw mut voices[16];')
ini('struct foo *s=(struct foo*)sample;s->unk4=100;s->unk8=1+k%2;s->unkC=32000;s->x10.loopFlag=k%2;native_set_pair(&s->x10.loopAddressHi,0x6000);native_set_pair(&s->x10.endAddressHi,0x7000);native_set_pair(&s->x10.currentAddressHi,0x6002);hsd_SynthSFXDataHash[100&31]=s;',
    'let s=sample.as_mut_ptr().cast::<HSD_SynthVoiceEntry>();(*s).unk4=100;(*s).unk8=(1+k%2) as i32;(*s).unkC=32000;(*s).x10.loopFlag=(k%2) as u16;(*s).x10.loopAddressLo=0x6000;(*s).x10.endAddressLo=0x7000;(*s).x10.currentAddressLo=0x6002;hsd_SynthSFXDataHash[100&31]=s.cast();')
ini('struct SfxLoadStreamNode*g=(struct SfxLoadStreamNode*)group;g->x4=400;g->x8=100;g->xC=1;g->x10=0x4800;g->x14=0x200;memcpy(g+1,s,sizeof(*s));HSD_Synth_804C2AE0[0]=(AXVPB*)g;',
    'let g=group.as_mut_ptr().cast::<SfxLoadStreamNode>();(*g).x4=400;(*g).x8=100;(*g).xC=1;(*g).x10=0x4800;(*g).x14=0x200;core::ptr::copy_nonoverlapping(s,g.add(1).cast::<HSD_SynthVoiceEntry>(),1);HSD_Synth_804C2AE0[0]=g.cast();')
ini('header[2]=32000;header[3]=1+k%2;hsd_SynthSFXLoadBuf[0]=0x60;hsd_SynthSFXLoadBuf[1]=0x100;hsd_SynthSFXLoadBuf[2]=1;hsd_SynthSFXLoadBuf[3]=200;hsd_SynthSFXLoadBuf[4]=1;hsd_SynthSFXLoadBuf[5]=32000;HSD_Synth_804D7730=(void*)heap;',
    'header[2]=32000;header[3]=1+k%2;hsd_SynthSFXLoadBuf[0]=0x60;hsd_SynthSFXLoadBuf[1]=0x100;hsd_SynthSFXLoadBuf[2]=1;hsd_SynthSFXLoadBuf[3]=200;hsd_SynthSFXLoadBuf[4]=1;hsd_SynthSFXLoadBuf[5]=32000;HSD_Synth_804D7730=heap.as_mut_ptr().cast();')
ini('th=2166136261U;hh=2166136261U;','th=2166136261;hh=2166136261;')

# Each function has its own operation. All seeds reset the entire synthetic world.
# Targeted invalid handles and transition variants are mixed with active nodes.
ops=[]
def op(name,c,r=None,ret='void'):
    if r is None:r=c
    ops.append((name,c,r,ret))
op('HSD_AudioMalloc','HSD_AudioMalloc(128)','HSD_AudioMalloc(128)','ptr')
op('HSD_AudioFree','HSD_AudioFree(heap)','HSD_AudioFree(heap.as_mut_ptr().cast())')
op('SfxLoadStreamDataSize','SfxLoadStreamDataSize((int)(k*0x1020304))','SfxLoadStreamDataSize(k.wrapping_mul(0x1020304) as i32)','int')
op('hsd_SynthSFXNodeAt','hsd_SynthSFXNodeAt((int)(k%68)-2)','hsd_SynthSFXNodeAt((k%68) as i32-2)','ptr')
op('HSD_SynthSFXSampleLoadCallback','HSD_SynthSFXSampleLoadCallback(0,0,0,0)','HSD_SynthSFXSampleLoadCallback(0,0,core::ptr::null_mut(),0)')
op('HSD_SynthSFXHeaderLoadCallback','HSD_SynthSFXHeaderLoadCallback(0,0,0,0)','HSD_SynthSFXHeaderLoadCallback(0,0,core::ptr::null_mut(),0)')
for n in ['HSD_SynthSFXLoadNewProc','HSD_SynthSFXBankDeflagSync','HSD_SynthCallback','HSD_SynthPStreamMasterClockCallback','HSD_SynthPStreamFirstHakoDataCallback','HSD_SynthPStreamStart_inline']:op(n,n+'()')
op('HSD_SynthSFXLoad','HSD_SynthSFXLoad("fixture",0,load_cb,3)','HSD_SynthSFXLoad(c"fixture".as_ptr(),0,Some(load_cb),3)','int')
op('HSD_SynthSFXWaitForLoadCompletion','HSD_SynthSFXWaitForLoadCompletion(wait_cb)','HSD_SynthSFXWaitForLoadCompletion(Some(wait_cb))')
op('HSD_SynthSFXGetPendingLoadCount','HSD_SynthSFXGetPendingLoadCount()',ret='int')
op('HSD_SynthSFXCancelLoad','HSD_SynthSFXCancelLoad(100+k%6)','HSD_SynthSFXCancelLoad(100+(k%6) as i32)','int')
op('HSD_SynthSFXAllocateBank','HSD_SynthSFXAllocateBank(0x2000+k)','HSD_SynthSFXAllocateBank(0x2000+k as i32)')
op('HSD_SynthSFXGroupDataUnlink','HSD_SynthSFXGroupDataUnlink((AXVPB*)group)','HSD_SynthSFXGroupDataUnlink(group.as_mut_ptr().cast())')
op('HSD_SynthSFXUnloadBank','HSD_SynthSFXUnloadBank(0)')
op('HSD_SynthSFXDataUnlink','HSD_SynthSFXDataUnlink(100+k%2)','HSD_SynthSFXDataUnlink(100+(k%2) as i32)')
op('HSD_SynthSFXGroupDataRemove','HSD_SynthSFXGroupDataRemove(400+k%2)','HSD_SynthSFXGroupDataRemove(400+(k%2) as i32)')
op('HSD_SynthSFXGroupDataReaddressCallback','HSD_SynthSFXGroupDataReaddressCallback(0,0,0,0)','HSD_SynthSFXGroupDataReaddressCallback(0,0,core::ptr::null_mut(),0)')
op('HSD_SynthSFXGroupDataReaddress','HSD_SynthSFXGroupDataReaddress((AXVPB*)group,(void*)0x5000)','HSD_SynthSFXGroupDataReaddress(group.as_mut_ptr().cast(),0x5000usize as *mut _)')
op('HSD_SynthSFXBankDeflag','HSD_SynthSFXBankDeflag(0)')
op('HSD_SynthGetSoundMode','HSD_SynthGetSoundMode()',ret='int')
op('HSD_SynthSetSoundMode','HSD_SynthSetSoundMode(k%2)','HSD_SynthSetSoundMode((k%2) as i32)')
for n in ['HSD_SynthSFXStopNode','freeVoices','HSD_SynthSFXUpdatePitch','HSD_SynthSFXUpdateVolume']:op(n,n+'(&hsd_SynthSFXNodes[0])',n+'(&raw mut hsd_SynthSFXNodes[0])')
op('dropcallback','dropcallback(&voices[k%2?0:16])','dropcallback((&raw mut voices[if k%2!=0{0}else{16}]).cast())')
op('HSD_Synth_80389334','HSD_Synth_80389334(100, k, k*3, k*5, 20, k%16, .75f,1.25f,.5f,.25f,.125f)','HSD_Synth_80389334(100,k as u8,k.wrapping_mul(3) as u8,k.wrapping_mul(5) as u8,20,(k%16) as i32,0.75,1.25,0.5,0.25,0.125)','int')
op('HSD_SynthSFXPlayWithGroup','HSD_SynthSFXPlayWithGroup(100,k,k*3,k*5,20,k%16,k%2,.75f,1.25f,.5f,.25f,.125f)','HSD_SynthSFXPlayWithGroup(100,k as u8,k.wrapping_mul(3) as u8,k.wrapping_mul(5) as u8,20,(k%16) as i32,(k%2) as i32,0.75,1.25,0.5,0.25,0.125)','int')
op('getNode','getNode(63+k%10)','getNode((63+k%10) as i32)','ptr')
for n in ['HSD_SynthSFXKeyOff','HSD_SynthSFXPause','HSD_SynthSFXResume','HSD_SynthSFXCheck']:op(n,n+'(63+k%10)',n+'((63+k%10) as i32)','int' if n.endswith('Check') else 'void')
op('stopRange','stopRange(0x8000,0x28000)')
op('HSD_SynthSFXStopRange','HSD_SynthSFXStopRange(0)')
op('HSD_SynthSFXSetVolumeFade','HSD_SynthSFXSetVolumeFade(64,k,k%2)','HSD_SynthSFXSetVolumeFade(64,k as u8,(k%2) as i32)')
op('HSD_SynthSFXSetPan','HSD_SynthSFXSetPan(64,k)','HSD_SynthSFXSetPan(64,k as u8)')
op('HSD_SynthSFXSetMix','HSD_SynthSFXSetMix(64,.25f,.5f,.75f)','HSD_SynthSFXSetMix(64,0.25,0.5,0.75)')
op('HSD_SynthSFXSetPitchRatio','HSD_SynthSFXSetPitchRatio(64,k%2,(float)k/128)','HSD_SynthSFXSetPitchRatio(64,(k%2) as i32,k as f32/128.0)')
op('HSD_SynthSFXSetPriority','HSD_SynthSFXSetPriority(64,k%32)','HSD_SynthSFXSetPriority(64,(k%32) as i32)')
for n in ['user_vol_dst_offset','user_vol_src_offset']:op(n,n+'((int)(k*0x1020304))',n+'(k.wrapping_mul(0x1020304) as i32)','int')
op('HSD_SynthSFXVolumeEnvelope','HSD_SynthSFXVolumeEnvelope()',ret='int')
op('my_memzero','my_memzero(header,k%32)','my_memzero(header.as_mut_ptr().cast(),(k%32) as usize)')
op('HSD_SynthSFXUpdateMix','HSD_SynthSFXUpdateMix(&hsd_SynthSFXNodes[0],k%2)','HSD_SynthSFXUpdateMix(&raw mut hsd_SynthSFXNodes[0],(k%2) as i32)')
op('updateAllVolume','updateAllVolume(k*0x1020304)','updateAllVolume(k.wrapping_mul(0x1020304))')
op('HSD_SynthSFXUpdateAllVolume','HSD_SynthSFXUpdateAllVolume(k,k%10,k%16)','HSD_SynthSFXUpdateAllVolume(k as i32,(k%10) as u16,(k%16) as i32)')
for n in ['HSD_SynthSFXSetDriverInactivatedCallback','HSD_SynthSFXSetDriverMasterClockCallback','HSD_SynthSFXSetDriverPauseCallback']:op(n,n+'(cb3)',n+'(Some(cb3))')
for n in ['HSD_SynthResetStreamCounters','HSD_SynthPStreamFirstHakoHeaderCallback','HSD_SynthPStreamHakoHeaderCallback']:op(n,n+'(0,0x500,0,0)',n+'(0,0x500,core::ptr::null_mut(),0)')
op('HSD_SynthPStreamMasterClockCallback_inline','HSD_SynthPStreamMasterClockCallback_inline(k%3)')
op('HSD_SynthPStreamHeaderCallback','HSD_SynthPStreamHeaderCallback(0,0,header,0)','HSD_SynthPStreamHeaderCallback(0,0,header.as_mut_ptr().cast(),0)')
op('HSD_SynthPStreamStart','HSD_SynthPStreamStart(500,k,k*3,k%16)','HSD_SynthPStreamStart(500,k as u8,k.wrapping_mul(3) as u8,(k%16) as i32)','int')
op('HSD_SynthStreamSetVolume','HSD_SynthStreamSetVolume((float)k/256)','HSD_SynthStreamSetVolume(k as f32/256.0)')
op('HSD_SynthInit','HSD_SynthInit(0,0,0,0x40000)')

cpre='''#include "synth_native.c"
#undef size_t
extern int printf(const char*,...);
extern void exit(int);
void __assert(const char*a,u32 b,const char*c){printf("ASSERT %s %d %s",a,b,c);exit(2);}
static u32 seed,hh,th,acq,acquire_mode;
static AXVPB voices[64];static unsigned long long heap[1024],sample[32],group[64];static u32 header[64];
static u32 rnd(void){seed^=seed<<13;seed^=seed>>17;seed^=seed<<5;return seed;}
static float rf(void){return ((int)(rnd()%4096)-1024)/1024.0f;}
static u32 fb(float f){union{float f;u32 u;}v;v.f=f;return v.u;}
static void h(u32 v){hh=(hh^v)*16777619U;}static void t(u32 v){th=(th^v)*16777619U;}
static void cb1(int v){t(101);t(v);}static void cb2(int v){t(102);t(v);}static void cb3(int v){t(103);t(v);}
static void load_cb(int a,int b){t(104);t(a);t(b);}static void wait_cb(void){t(105);HSD_Synth_804D772C--;}
static u32 pi(const void*p){unsigned long long v=(unsigned long long)p;
'''
rpre='''#![allow(non_snake_case,non_upper_case_globals,dead_code,unused_imports,static_mut_refs)]
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
'''
for rid,n in enumerate(['hsd_SynthSFXNodes','voices','heap','sample','group','header','hsd_SynthSFXLoadBuf','pstHakoHeader'],1):
    cpre+=f'if(v>=(unsigned long long)&{n} && v<(unsigned long long)&{n}+sizeof {n})return 0x{rid}00000+(u32)(v-(unsigned long long)&{n});\n'
    rpre+=f'let b=(&raw const {n}) as usize;if v>=b&&v<b+core::mem::size_of_val(&{n}){{return 0x{rid}00000+(v-b) as u32;}}\n'
cpre+='return (u32)v;}\nstatic u32 fi(void*p){\n'
rpre+='v as u32}\nunsafe fn fi(v:usize)->u32{\n'
for i,n in enumerate(cbnames,1):
    cpre+=f'if(p==(void*){n})return {i};\n'
    if n=='HSD_SynthPStreamFirstHakoDataCallback':continue
    rpre+=f'if v=={n} as *const () as usize{{return {i};}}\n'
# Adapter has the C callback's identity (it adapts the historical extra args ABI).
rpre+='if v==synth::first_data_adapter as *const () as usize{return 8;}\n'
cpre+='return p?999:0;}\n'
rpre+='if v==0{0}else{999}}\n'
tracec=[];tracer=[]
for ty in types:
    ct=cnames.get(ty,ty)
    tracec.append(f'static void trace_{ty}(const {ct}*p){{u32 save=hh;hh=th;hash_{ty}(p);th=hh;hh=save;}}')
    tracer.append(f'unsafe fn trace_{ty}(p:*const {ty}){{let save=hh;hh=th;hash_{ty}(p);th=hh;hh=save;}}')
statec+=['for(int i=0;i<64;i++)h(header[i]);','{hash_SfxLoadStreamNode((void*)heap);hash_HSD_SynthVoiceEntry((struct foo*)((struct SfxLoadStreamNode*)heap+1));}']
stater+=['for i in 0..64{h(header[i]);}','{hash_SfxLoadStreamNode(heap.as_ptr().cast());hash_HSD_SynthVoiceEntry(heap.as_ptr().cast::<SfxLoadStreamNode>().add(1).cast());}']
ccases=[];rcases=[]
for i,(n,c,r,ret) in enumerate(ops):
    if ret=='ptr':c=f'result=pi({c});';r=f'result=pi({r} as usize);'
    elif ret=='int':c=f'result=(u32){c};';r=f'result=({r}) as u32;'
    else:c+=';';r+=';'
    if n in ('HSD_Synth_80389334','HSD_SynthSFXPlayWithGroup'):
        c='acquire_mode=k%4;'+c;r='acquire_mode=k%4;'+r
    if n=='HSD_SynthSFXHeaderLoadCallback':
        c='if(k%4==0)hsd_SynthSFXBank[0]=hsd_SynthSFXBankHead[1]-64;'+c
        r='if k%4==0{hsd_SynthSFXBank[0]=hsd_SynthSFXBankHead[1]-64;}'+r
    if n=='HSD_SynthSFXVolumeEnvelope':
        # Force completion, pause and release branches, besides ordinary ramps.
        c='if(k%4==0){hsd_SynthSFXNodes[0].x24=0;hsd_SynthSFXNodes[0].flags=k%16;hsd_SynthSFXNodes[0].user_vol[0].x8_float=0;hsd_SynthSFXNodes[0].user_vol[0].x4=0;hsd_SynthSFXNodes[0].user_vol[1].x4=0;voicelist_1784[hsd_SynthSFXNodes[0].xB].x178C=0;hsd_SynthSFXNodes[0].x20=0;HSD_Synth_804D774C=&hsd_SynthSFXNodes[0];}'+c
        r='if k%4==0{hsd_SynthSFXNodes[0].x24=0;hsd_SynthSFXNodes[0].flags=(k%16) as u8;hsd_SynthSFXNodes[0].user_vol[0].anon_8.x8_float=0.0;hsd_SynthSFXNodes[0].user_vol[0].x4=0;hsd_SynthSFXNodes[0].user_vol[1].x4=0;voicelist_1784[hsd_SynthSFXNodes[0].xB as usize].x178C=0;hsd_SynthSFXNodes[0].x20=core::ptr::null_mut();HSD_Synth_804D774C=&raw mut hsd_SynthSFXNodes[0];}'+r
    ccases.append(f'case {i}: {c}break;')
    rcases.append(f'{i}=>{{{r}}},')
cmain='''
static void setup(unsigned k){'''+''.join(initc)+'''}
static void snapshot(void){'''+''.join(statec)+'''}
int main(void){for(unsigned op=0;op<'''+str(len(ops))+''';op++)for(unsigned k=0;k<512;k++){u32 result=0;setup(k);switch(op){'''+''.join(ccases)+'''}hh=2166136261U;snapshot();printf("%u %u %08x %08x %08x\\n",op,k,result,th,hh);}return 0;}
'''
rmain='''
unsafe fn setup(k:u32){'''+''.join(initr)+'''}
unsafe fn snapshot(){'''+''.join(stater)+'''}
#[test]fn synth_matches_native_c(){unsafe{'''+''.join(install)+'''
let names=['''+','.join('"'+n+'"' for n,*_ in ops)+'''];
for row in include_str!("data/tu_synth.txt").lines(){let w:Vec<&str>=row.split_whitespace().collect();let op:usize=w[0].parse().unwrap();let k:u32=w[1].parse().unwrap();setup(k);let mut result=0u32;match op{'''+''.join(rcases)+'''_=>unreachable!()};hh=2166136261;snapshot();let expected=[u32::from_str_radix(w[2],16).unwrap(),u32::from_str_radix(w[3],16).unwrap(),u32::from_str_radix(w[4],16).unwrap()];assert_eq!([result,th,hh],expected,"{} seed {}",names[op],k);}
}}
'''
rmain=rmain.replace('let w:Vec<&str>','let w:std::vec::Vec<&str>')
(BUILD/'driver.c').write_text(cpre+'\n'.join(walk_c+tracec+stubs_c)+cmain)
(ROOT/'rust/melee_runtime/tests/tu_synth.rs').write_text(rpre+'\n'.join(walk_r+tracer+stubs_r)+rmain)
flags=['--target=x86_64-pc-windows-msvc','-std=c99','-nostdinc','-fno-builtin','-DLINT','-DTARGET_PC','-fno-short-enums','-fsigned-char','-mlong-double-64','-fno-strict-aliasing','-fwrapv','-fcommon','-fgnu89-inline','-ffp-contract=on','-O2','-Xclang','-disable-llvm-passes','-ftrivial-auto-var-init=zero','-S','-emit-llvm','-w','-D_Static_assert(a,b)=','-Dsize_t=gc_size_t','-Isrc','-Ipc','-Ipc/gameworld','-Isrc/sysdolphin/baselib','-isystem','src/MSL','-isystem','libs/dolphin/include','-isystem','libs/dolphin/src','-include','src/MSL/math_ppc.h']
lls=[]
for source in [BUILD/'driver.c',ROOT/'pc/gameworld/gekko_fp.c']:
    ll=BUILD/(source.stem+'.ll');lo=BUILD/(source.stem+'.lo.ll')
    run([CL,*flags,source,'-o',ll]);os.environ['GW_NARROW_FPTOUI_AS_SI']='1'
    print(run([sys.executable,HERE/'lower.py',ll,lo]).strip());lls.append(lo)
exe=BUILD/'synth.exe';run([CL,'-O0','-w',*lls,'-o',exe,'-fuse-ld=lld','-Wl,/OPT:NOICF'])
output=run([exe]);(ROOT/'rust/melee_runtime/tests/data/tu_synth.txt').write_text(output)
print(len(ops),'functions;',len(output.splitlines()),'oracle cases')
