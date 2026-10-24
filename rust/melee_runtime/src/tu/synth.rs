//! Native port of `baselib/synth.c`. AX/ARAM/DVD services are injectable hooks.
//! Disk records retain their 0x40-byte voice stride; native objects use generated layouts.
#![allow(non_snake_case, non_upper_case_globals, dead_code, static_mut_refs, clippy::too_many_arguments)]
use core::{ffi::c_void, ptr::{null_mut, write_unaligned}};
use melee_types::*;

pub type DevCallback = Option<unsafe extern "C" fn(i32, usize, *mut c_void, i32)>;
pub mod ext {
    use super::*;
    macro_rules! hooks { ($(fn $n:ident($($a:ident:$t:ty),*) $(->$r:ty)?;)*) => {
        pub mod hook { use super::*; $(pub static mut $n: Option<fn($($t),*) $(->$r)?> = None;)* }
        $(pub fn $n($($a:$t),*) $(->$r)? { unsafe { hook::$n.expect(concat!("TODO: ",stringify!($n)))($($a),*) } })*
    }}
    hooks! {
        fn OSAllocFromHeap(heap_id:i32,size:usize)->*mut c_void;
        fn OSFreeToHeap(heap_id:i32,p:*mut c_void);
        fn OSDisableInterrupts()->i32;
        fn OSRestoreInterrupts(enabled:i32)->i32;
        fn OSGetSoundMode()->u32;
        fn OSSetSoundMode(mode:u32);
        fn OSReport(message:&str,value:i32);
        fn DVDConvertPathToEntrynum(filename:*const i8)->i32;
        fn HSD_DevComRequest(file:i32,src:usize,dst:usize,size:usize,flags:u32,priority:i32,cb:DevCallback,args:usize)->i32;
        fn HSD_DevComCancelEx(req:i32,flags:u32,cb:DevCallback,args:usize)->i32;
        fn AXAcquireVoice(priority:u32,cb:Option<unsafe extern "C" fn(*mut c_void)>,context:u32)->*mut AXVPB;
        fn AXFreeVoice(voice:*mut AXVPB);
        fn AXSetVoicePriority(voice:*mut AXVPB,priority:u32);
        fn AXSetVoiceVe(voice:*mut AXVPB,ve:*mut AXPBVE);
        fn AXSetVoiceVeDelta(voice:*mut AXVPB,delta:i16);
        fn AXSetVoiceSrc(voice:*mut AXVPB,src:*mut AXPBSRC);
        fn AXSetVoiceSrcRatio(voice:*mut AXVPB,ratio:f32);
        fn AXSetVoiceAddr(voice:*mut AXVPB,addr:*mut AXPBADDR);
        fn AXSetVoiceAdpcm(voice:*mut AXVPB,adpcm:*mut AXPBADPCM);
        fn AXSetVoiceAdpcmLoop(voice:*mut AXVPB,adpcm:*mut AXPBADPCMLOOP);
        fn AXSetVoiceState(voice:*mut AXVPB,state:u16);
        fn AXSetVoiceMix(voice:*mut AXVPB,mix:*mut AXPBMIX);
        fn AXSetVoiceItdOn(voice:*mut AXVPB);
        fn AXSetVoiceItdTarget(voice:*mut AXVPB,l:u16,r:u16);
        fn AXSetVoiceEndAddr(voice:*mut AXVPB,addr:u32);
        fn AXSetVoiceCurrentAddr(voice:*mut AXVPB,addr:u32);
        fn AXSetVoiceLoopAddr(voice:*mut AXVPB,addr:u32);
        fn AXSetVoiceLoop(voice:*mut AXVPB,enabled:u16);
        fn AXInit();
        fn AXRegisterCallback(cb:Option<unsafe extern "C" fn()>);
        fn AISetDSPSampleRate(rate:u32);
        fn AISetStreamVolLeft(vol:u8);
        fn AISetStreamVolRight(vol:u8);
        fn ARAlloc(size:u32)->u32;
        fn wait_idle();
        fn watch_page(p:*mut c_void,size:u32);
    }
}
macro_rules! zero_statics { ($($n:ident:$t:ty),* $(,)?) => {$(pub static mut $n:$t=unsafe{core::mem::zeroed()};)*}; }
pub static mut HSD_Synth_804D6018:i32=-1;
pub static mut HSD_Synth_804D6030:f32=1.0;
zero_statics! {
    HSD_Synth_804D6028:[i32;2], voicelist:[*mut AXVPB;64], hsd_SynthSFXDataHash:[*mut c_void;32],
    HSD_Synth_804C2A60:[Anon_anon_49;6], hsd_SynthSFXLoadBuf:[u32;8], HSD_Synth_804C2AE0:[*mut AXVPB;64],
    hsd_SynthSFXBank:[i32;32], hsd_SynthSFXBankHead:[i32;33], hsd_SynthSFXNodes:[HSD_SynthSFXNode;64],
    voicelist_1784:[Anon_anon_64;16], voicelist_1844:[i32;256], pstHakoHeader:[Anon_anon_76;3],
    HSD_Synth_804D7720:i32,hsd_SynthSFXBankNum:i32,hsd_SynthSFXBankAREnd:u32,HSD_Synth_804D772C:i32,
    HSD_Synth_804D7730:*mut SfxLoadStreamNode,HSD_Synth_804D7734:*mut u32,HSD_Synth_804D7738:i32,
    sfxGroupDataReaddressCounter:i32,driverInactivatedCallback:Option<unsafe extern "C" fn(i32)>,
    driverMasterClockCallback:Option<unsafe extern "C" fn(i32)>,driverPauseCallback:Option<unsafe extern "C" fn(i32)>,
    HSD_Synth_804D774C:*mut HSD_SynthSFXNode,HSD_Synth_804D7750:i32,HSD_Synth_804D7754:u32,
    HSD_Synth_804D7758:u32,HSD_Synth_804D775C:i32,HSD_Synth_804D7760:i32,HSD_Synth_804D7764:i32,
    HSD_Synth_804D7768:u32,HSD_Synth_804D776C:u32,HSD_Synth_804D7770:u32,HSD_Synth_804D7774:u32,
    HSD_Synth_804D7778:u8,HSD_Synth_804D777C:i32,HSD_Synth_804D7780:u32,HSD_Synth_804D7784:u32,
    lbl_80407FB4:AXPBMIX
}
pub static mut HSD_Synth_80407FD8:AXPBSRC=AXPBSRC{ratioHi:1,ratioLo:0,currentAddressFrac:0,last_samples:[0;4]};

pub unsafe extern "C" fn HSD_AudioMalloc(size:usize)->*mut c_void { let p=ext::OSAllocFromHeap(HSD_Synth_804D6018,size); assert!(!p.is_null(),"audio heap overflow");p }
pub unsafe extern "C" fn HSD_AudioFree(p:*mut c_void){ext::OSFreeToHeap(HSD_Synth_804D6018,p);}
pub fn SfxLoadStreamDataSize(size:i32)->i32 {size.wrapping_add(8)}
pub unsafe fn hsd_SynthSFXNodeAt(index:i32)->*mut HSD_SynthSFXNode {
    let i=if !(0..64).contains(&index){ext::OSReport("[NODEIDX] bad index=%d",index);0}else{index};
    &raw mut hsd_SynthSFXNodes[i as usize]
}
pub unsafe fn getNode(id:i32)->*mut HSD_SynthSFXNode {let p=&raw mut hsd_SynthSFXNodes[(id&63) as usize];if id>0&&(*p).x0==id{p}else{null_mut()}}
pub unsafe extern "C" fn HSD_SynthSFXLoadNewProc(){if HSD_Synth_804D772C!=0 {let e=ext::OSDisableInterrupts();HSD_Synth_804D6028[0]=ext::HSD_DevComRequest(HSD_Synth_804C2A60[0].entrynum,0,(&raw mut hsd_SynthSFXLoadBuf) as usize,32,0x21,1,Some(HSD_SynthSFXHeaderLoadCallback),0);ext::OSRestoreInterrupts(e);}}
pub unsafe extern "C" fn HSD_SynthSFXLoad(filename:*const i8,bankID:i32,cb:Option<unsafe extern "C" fn(i32,i32)>,mode:i32)->i32 {
    assert!(bankID>=0&&bankID<hsd_SynthSFXBankNum);let entrynum=ext::DVDConvertPathToEntrynum(filename);
    ext::OSReport("synth: bank load entry",entrynum);
    while core::ptr::read_volatile(&raw const HSD_Synth_804D772C)>=6 {core::hint::spin_loop();}
    let e=ext::OSDisableInterrupts();let q=&mut HSD_Synth_804C2A60[HSD_Synth_804D772C as usize];q.entrynum=entrynum;q.bankID=bankID;q.x8=cb;q.xC=mode;
    HSD_Synth_804D772C+=1;if HSD_Synth_804D772C==1{HSD_SynthSFXLoadNewProc();}ext::OSRestoreInterrupts(e);entrynum
}
pub unsafe extern "C" fn HSD_SynthSFXWaitForLoadCompletion(cb:Option<unsafe extern "C" fn()>){while core::ptr::read_volatile(&raw const HSD_Synth_804D772C)!=0{cb.unwrap()();}}
pub unsafe extern "C" fn HSD_SynthSFXGetPendingLoadCount()->i32{HSD_Synth_804D772C.wrapping_sub(HSD_Synth_804D7738)}
pub unsafe extern "C" fn HSD_SynthSFXCancelLoad(entrynum:i32)->i32 {
    let mut result=0;let e=ext::OSDisableInterrupts();if HSD_Synth_804D772C!=0{
        if HSD_Synth_804C2A60[0].entrynum==entrynum&&HSD_Synth_804D7738==0{HSD_Synth_804D7738=1;for i in 0..2{ext::HSD_DevComCancelEx(HSD_Synth_804D6028[i],0,None,0);}result=1;}
        else {for i in 1..HSD_Synth_804D772C as usize{if HSD_Synth_804C2A60[i].entrynum==entrynum{for j in i..HSD_Synth_804D772C as usize-1{HSD_Synth_804C2A60[j]=HSD_Synth_804C2A60[j+1];}HSD_Synth_804D772C-=1;result=1;break;}}}
    }ext::OSRestoreInterrupts(e);result
}
pub unsafe extern "C" fn HSD_SynthSFXAllocateBank(size:i32){let n=hsd_SynthSFXBankNum as usize;let base=hsd_SynthSFXBankHead[n];hsd_SynthSFXBank[n]=base;hsd_SynthSFXBankHead[n+1]=size.wrapping_add(base);assert!((hsd_SynthSFXBankHead[n+1] as u32)<=hsd_SynthSFXBankAREnd);hsd_SynthSFXBankNum+=1;}
pub unsafe extern "C" fn HSD_SynthSFXGroupDataReaddressCallback(_:i32,_:usize,_:*mut c_void,_:i32){}
pub unsafe extern "C" fn HSD_SynthSFXBankDeflagSync(){}
pub unsafe extern "C" fn HSD_SynthGetSoundMode()->u32 {ext::OSGetSoundMode()}
pub unsafe extern "C" fn HSD_SynthSetSoundMode(mode:i32){HSD_Synth_804D7754=mode as u32;for i in 0..64{if hsd_SynthSFXNodes[i].x0>0{HSD_SynthSFXUpdateMix(&raw mut hsd_SynthSFXNodes[i],1);}}ext::OSSetSoundMode(mode as u32);}
pub unsafe fn freeVoices(node:*mut HSD_SynthSFXNode){for i in 0..(*node).voice_count as usize{let v=(*node).voice[i];ext::AXFreeVoice(v);(*hsd_SynthSFXNodeAt((*v).index as i32)).x0=0;}}
pub unsafe extern "C" fn HSD_SynthSFXStopNode(node:*mut HSD_SynthSFXNode){if (*node).flags&1==0&&(*node).x27==1{if let Some(f)=driverInactivatedCallback{f((*node).x0);}}freeVoices(node);}
pub unsafe extern "C" fn dropcallback(dropped:*mut c_void){let v=dropped.cast::<AXVPB>();let e=ext::OSDisableInterrupts();let mut n=hsd_SynthSFXNodeAt((*v).index as i32);for i in 0..HSD_Synth_804D7720 as usize{if voicelist[i]==v{voicelist[i]=null_mut();break;}}if (*n).x0==0{ext::OSRestoreInterrupts(e);return;}if (*n).x0 == -1{n=hsd_SynthSFXNodeAt((*(*n).voice[0]).index as i32);}if (*n).flags&1==0&&(*n).x27==1{if let Some(f)=driverInactivatedCallback{f((*n).x0);}}for i in 0..(*n).voice_count as usize{let p=(*n).voice[i];if p!=v&&HSD_Synth_804D7720<64{voicelist[HSD_Synth_804D7720 as usize]=p;HSD_Synth_804D7720+=1;}(*hsd_SynthSFXNodeAt((*p).index as i32)).x0=0;}ext::OSRestoreInterrupts(e);}
pub unsafe extern "C" fn HSD_SynthSFXKeyOff(id:i32){let n=getNode(id);if n.is_null(){return;}if (*n).flags&8!=0{HSD_SynthSFXStopNode(n);}else if (*n).flags&1==0{(*n).flags|=1;HSD_SynthSFXUpdateVolume(n);for i in 0..(*n).voice_count as usize{ext::AXSetVoicePriority((*n).voice[i],1);}}}
pub unsafe fn stopRange(lo:usize,hi:usize){for i in 0..64{let n=&raw mut hsd_SynthSFXNodes[i];if (*n).x0>0{let a=&(*(*n).voice[0]).pb.addr;let addr=((a.currentAddressHi as u32)<<16|a.currentAddressLo as u32) as usize;if addr>=lo&&addr<hi{HSD_SynthSFXStopNode(n);}}}}
pub unsafe extern "C" fn HSD_SynthSFXStopRange(bank:i32){let b=bank as usize;stopRange(hsd_SynthSFXBankHead[b].wrapping_mul(2) as u32 as usize,hsd_SynthSFXBankHead[b+1].wrapping_mul(2) as u32 as usize);}
pub unsafe extern "C" fn HSD_SynthSFXPause(id:i32){let n=getNode(id);if !n.is_null(){(*n).flags|=2;HSD_SynthSFXUpdateVolume(n);if (*n).flags&8!=0{(*n).flags|=6;}}}
pub unsafe extern "C" fn HSD_SynthSFXResume(id:i32){let n=getNode(id);if !n.is_null()&&(*n).flags&2!=0{(*n).flags&=!6;HSD_SynthSFXUpdatePitch(n);HSD_SynthSFXUpdateVolume(n);}}
pub unsafe extern "C" fn HSD_SynthSFXCheck(id:i32)->i32{let n=getNode(id);if !n.is_null(){if (*n).flags&1!=0{return -1;}for i in 0..(*n).voice_count as usize{if (*(*n).voice[i]).pb.state==0{return -1;}}return id;}-1}
pub unsafe extern "C" fn HSD_SynthSFXSetVolumeFade(id:i32,vol:u8,flag:i32){let n=getNode(id);if !n.is_null(){assert!((0..2).contains(&flag));(*n).user_vol[flag as usize].volume=((1.0/255.0f64)*vol as f64) as f32;(*n).user_vol[flag as usize].x4=1;HSD_SynthSFXUpdateVolume(n);}}
pub unsafe extern "C" fn HSD_SynthSFXSetPan(id:i32,vol:u8){let n=getNode(id);if !n.is_null(){(*n).user_vol[1].anon_8.x8=vol;HSD_SynthSFXUpdateMix(n,1);}}
pub unsafe extern "C" fn HSD_SynthSFXSetMix(id:i32,a:f32,b:f32,c:f32){let n=getNode(id);if !n.is_null(){(*n).x44=a;(*n).x48=b;(*n).x4C=c;HSD_SynthSFXUpdateMix(n,1);}}
pub unsafe extern "C" fn HSD_SynthSFXUpdatePitch(n:*mut HSD_SynthSFXNode){let r=if (*n).flags&4!=0{0.0}else{(*n).x14*(*n).x18[0]*(*n).x18[1]};if (*n).flags&8==0{for i in 0..(*n).voice_count as usize{ext::AXSetVoiceSrcRatio((*n).voice[i],r);}}}
pub unsafe extern "C" fn HSD_SynthSFXSetPitchRatio(id:i32,flag:i32,r:f32){let n=getNode(id);if !n.is_null()&&(*n).flags&1==0{assert!((0..2).contains(&flag));(*n).x18[flag as usize]=r;HSD_SynthSFXUpdatePitch(n);}}
pub unsafe extern "C" fn HSD_SynthSFXSetPriority(id:i32,p:i32){let n=getNode(id);if !n.is_null()&&(*n).flags&1==0{for i in 0..(*n).voice_count as usize{ext::AXSetVoicePriority((*n).voice[i],p as u32);}}}
pub fn user_vol_dst_offset(k:i32)->i32{k.wrapping_mul(3)}
pub fn user_vol_src_offset(k:i32)->i32{k.wrapping_mul(3)}
pub unsafe extern "C" fn HSD_SynthSFXUpdateVolume(n:*mut HSD_SynthSFXNode){let e=ext::OSDisableInterrupts();if (*n).volume_update_pending==0&&(*n).flags&8==0{(*n).volume_update_pending=1;(*n).x20=HSD_Synth_804D774C;HSD_Synth_804D774C=n;}ext::OSRestoreInterrupts(e);}
pub unsafe fn updateAllVolume(mask:u32){HSD_Synth_804D7758|=mask;for i in 0..64{let n=&raw mut hsd_SynthSFXNodes[i];if (*n).x0>0&&(1u32.checked_shl((*n).xB as u32).unwrap_or(0)&mask)!=0{HSD_SynthSFXUpdateVolume(n);}}}
pub unsafe extern "C" fn HSD_SynthSFXUpdateAllVolume(vol:i32,frames:u16,ch:i32){let v=&mut voicelist_1784[ch as usize];v.x1788=(1.0/255.0f32)*vol as f32;if v.x1784!=v.x1788{v.x178C=frames as i32;updateAllVolume(1u32.checked_shl(ch as u32).unwrap_or(0));}}
pub unsafe extern "C" fn HSD_SynthSFXSetDriverInactivatedCallback(cb:Option<unsafe extern "C" fn(i32)>){driverInactivatedCallback=cb;}
pub unsafe extern "C" fn HSD_SynthSFXSetDriverMasterClockCallback(cb:Option<unsafe extern "C" fn(i32)>){driverMasterClockCallback=cb;}
pub unsafe extern "C" fn HSD_SynthSFXSetDriverPauseCallback(cb:Option<unsafe extern "C" fn(i32)>){driverPauseCallback=cb;}

pub unsafe extern "C" fn HSD_SynthSFXVolumeEnvelope()->i32 {
    let intr=ext::OSDisableInterrupts();
    if HSD_Synth_804D7758!=0 {for ch in 0..16 {if HSD_Synth_804D7758&(1<<ch)!=0 {let v=&mut voicelist_1784[ch];let c=v.x178C;if c!=0 {v.x1784=(v.x1784*(c as f32-1.0))/c as f32+v.x1788/c as f32;v.x178C=c.wrapping_sub(1);}if v.x178C==0 {v.x1784=v.x1788;HSD_Synth_804D7758&=!(1<<ch);}}}}
    let mut link=&raw mut HSD_Synth_804D774C;
    while !(*link).is_null(){let n=*link;let mut active=false;if (*n).x0<=0{(*n).volume_update_pending=0;*link=(*n).x20;continue;}
        for k in 0..2{let c=(*n).user_vol[k].x4;if c!=0 {
            // user_vol[k].volume[-1] is unk28 for k=0, user_vol[0].x8_float for k=1.
            let cur=if k==0{&raw mut (*n).unk28}else{&raw mut (*n).user_vol[0].anon_8.x8_float};
            *cur=(*cur*(c as f32-1.0))/c as f32+(*n).user_vol[k].volume/c as f32;
            (*n).user_vol[k].x4=c.wrapping_sub(1);if (*n).user_vol[k].x4!=0{active=true;}
        }}
        if voicelist_1784[(*n).xB as usize].x178C!=0{active=true;}
        let vol=if (*n).flags&3==0{node_volume(n) as i32}else{active=false;0};
        let delta=(vol.wrapping_sub((*n).x24 as i32)/160).clamp(-20,20);
        for i in 0..(*n).voice_count as usize{ext::AXSetVoiceVeDelta((*n).voice[i],delta as i16);}
        (*n).x24=((*n).x24 as i32+delta*160) as u16;
        if delta==0&&(!active||vol as f32==0.0){(*n).x24=vol as u16;let mut ve=AXPBVE{currentVolume:vol as u16,currentDelta:0};
            for i in 0..(*n).voice_count as usize{(*(*n).voice[i]).sync&=0xfffffbff;ext::AXSetVoiceVe((*n).voice[i],&mut ve);}
            if !active{(*n).volume_update_pending=0;*link=(*n).x20;let f=(*n).flags;if f&1!=0{freeVoices(n);}else if f&6==2{(*n).flags=f|6;for _ in 0..(*n).voice_count{HSD_SynthSFXUpdatePitch(n);}if (*n).x27==1{if let Some(cb)=driverPauseCallback{cb((*n).x0);}}}continue;}
        }link=&raw mut (*n).x20;
    }ext::OSRestoreInterrupts(intr)
}
unsafe fn node_volume(n:*mut HSD_SynthSFXNode)->f32 {32767.0*((*n).user_vol[0].anon_8.x8_float*((*n).unk28*(HSD_Synth_804D6030*voicelist_1784[(*n).xB as usize].x1784)))}
pub unsafe fn my_memzero(dst:*mut c_void,size:usize){for i in (0..size).step_by(4){write_unaligned(dst.cast::<u8>().add(i).cast::<u32>(),0);}}
pub const lbl_8040806C:[u8;128]=[
0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13,14,14,
15,15,16,16,17,17,18,18,19,19,20,20,21,21,22,22,22,23,23,23,23,24,24,24,24,25,25,25,
25,25,26,26,26,26,26,26,27,27,27,27,27,27,27,28,28,28,28,28,28,28,28,29,29,29,29,29,29,
29,29,29,30,30,30,30,30,30,30,30,30,30,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31,31];
pub unsafe extern "C" fn HSD_SynthSFXUpdateMix(n:*mut HSD_SynthSFXNode,interpolate:i32){
    let pan=(*n).user_vol[1].anon_8.x8;
    let (mut l,r)=if HSD_Synth_804D7754!=0{(32767.0*crate::gekko::sqrtf_accurate(0.003921569*(255-pan) as f32),32767.0*crate::gekko::sqrtf_accurate(0.003921569*pan as f32))}else{(23169.768,23169.768)};
    let m=&raw mut lbl_80407FB4;
    if (*n).voice_count==1{(*m).vL=(l*(*n).x44) as i32 as u16;(*m).vR=(r*(*n).x44) as i32 as u16;(*m).vAuxAL=(l*(*n).x48) as i32 as u16;(*m).vAuxAR=(r*(*n).x48) as i32 as u16;(*m).vAuxBL=(l*(*n).x4C) as i32 as u16;(*m).vAuxBR=(r*(*n).x4C) as i32 as u16;ext::AXSetVoiceMix((*n).voice[0],m);}
    else if HSD_Synth_804D7754!=0{(*m).vL=(l*(*n).x44) as i32 as u16;(*m).vAuxAL=(l*(*n).x48) as i32 as u16;(*m).vAuxBL=(l*(*n).x4C) as i32 as u16;ext::AXSetVoiceMix((*n).voice[0],m);(*m).vAuxBL=0;(*m).vAuxAL=0;(*m).vL=0;(*m).vR=(r*(*n).x44) as i32 as u16;(*m).vAuxAR=(r*(*n).x48) as i32 as u16;(*m).vAuxBR=(r*(*n).x4C) as i32 as u16;ext::AXSetVoiceMix((*n).voice[1],m);}
    else{l/=2.0;(*m).vL=(l*(*n).x44) as i32 as u16;(*m).vR=(*m).vL;(*m).vAuxAL=(l*(*n).x48) as i32 as u16;(*m).vAuxAR=(*m).vAuxAL;(*m).vAuxBL=(l*(*n).x4C) as i32 as u16;(*m).vAuxBR=(*m).vAuxBL;ext::AXSetVoiceMix((*n).voice[0],m);ext::AXSetVoiceMix((*n).voice[1],m);}
    my_memzero(m.cast(),core::mem::size_of::<AXPBMIX>());
    let(mut sl,mut sr)=(0,0);if HSD_Synth_804D7754!=0{if pan<128{sl=lbl_8040806C[(127-pan) as usize] as u16;}else{sr=lbl_8040806C[(pan-128) as usize] as u16;}}
    for i in 0..(*n).voice_count as usize{let v=(*n).voice[i];if sl!=0||sr!=0||(*(*n).voice[0]).pb.itd.flag==1{if (*v).pb.itd.flag==0||(*v).sync&32!=0{ext::AXSetVoiceItdOn(v);if interpolate==0{(*v).pb.itd.shiftR=sr;(*v).pb.itd.shiftL=sl;}(*v).pb.itd.targetShiftR=sr;(*v).pb.itd.targetShiftL=sl;}else{ext::AXSetVoiceItdTarget(v,sl,sr);}}}
}
unsafe fn set_src_ratio(r:u32){HSD_Synth_80407FD8.ratioHi=(r>>16) as u16;HSD_Synth_80407FD8.ratioLo=r as u16;}
pub unsafe extern "C" fn HSD_Synth_80389334(id:i32,vol:u8,vol2:u8,pan:u8,priority:i32,itd:i32,p1:f32,p2:f32,a:f32,b:f32,c:f32)->i32 {
    let e=ext::OSDisableInterrupts();let mut entry=hsd_SynthSFXDataHash[(id&31) as usize].cast::<HSD_SynthVoiceEntry>();
    while !entry.is_null(){if (*entry).unk4==id{let mut voices=[null_mut();2];for i in 0..(*entry).unk8 as usize{voices[i]=ext::AXAcquireVoice(priority.wrapping_add(1) as u32,Some(dropcallback),0);if voices[i].is_null(){if !voices[0].is_null(){ext::AXFreeVoice(voices[0]);}ext::OSRestoreInterrupts(e);return -1;}}
        if (*entry).unk8==2&&voices[0]==voices[1]{ext::AXFreeVoice(voices[0]);ext::OSRestoreInterrupts(e);return -1;}
        if (*entry).unk8==2{let secondary=hsd_SynthSFXNodeAt((*voices[1]).index as i32);(*secondary).x0=-1;(*secondary).voice[0]=voices[0];}
        let idx=(*voices[0]).index;let n=hsd_SynthSFXNodeAt(idx as i32);(*n).x27=1;(*n).sfx_id=id;(*n).flags=0;(*n).voice_count=(*entry).unk8 as u8;(*n).xB=itd as u8;(*n).voice=voices;(*n).x14=0.00003125*(*entry).unkC as f32;(*n).x18=[p1,p2];
        let v=(1.0/255.0f32)*vol as f32;let v2=(1.0/255.0f32)*vol2 as f32;(*n).unk28=v;(*n).user_vol[0].volume=v;(*n).user_vol[0].x4=0;(*n).user_vol[0].anon_8.x8_float=v2;(*n).user_vol[1].volume=v2;(*n).user_vol[1].x4=0;(*n).user_vol[1].anon_8.x8=pan;(*n).x44=a;(*n).x48=b;(*n).x4C=c;
        HSD_SynthSFXUpdateVolume(n);HSD_SynthSFXUpdateMix(n,0);let mut ve=AXPBVE{currentVolume:node_volume(n) as i32 as u16,currentDelta:0};(*n).x24=ve.currentVolume;
        for i in 0..(*entry).unk8 as usize {let v=voices[i];ext::AXSetVoicePriority(v,priority as u32);ext::AXSetVoiceVe(v,&mut ve);set_src_ratio((65536.0*((*n).x18[1]*((*n).x14*(*n).x18[0]))) as u32);ext::AXSetVoiceSrc(v,&raw mut HSD_Synth_80407FD8);
            let block=entry.cast::<u8>().add(i*64).cast::<HSD_SynthVoiceEntry>();ext::AXSetVoiceAddr(v,&raw mut (*block).x10);ext::AXSetVoiceAdpcm(v,&raw mut (*block).x20);ext::AXSetVoiceAdpcmLoop(v,&raw mut (*block).x48);ext::AXSetVoiceState(v,1);
        }
        HSD_Synth_804D7750=HSD_Synth_804D7750.wrapping_add(64);if HSD_Synth_804D7750<0{HSD_Synth_804D7750=64;}(*n).x0=HSD_Synth_804D7750.wrapping_add(idx as i32);ext::OSRestoreInterrupts(e);return (*n).x0;
    }entry=(*entry).next.cast();}ext::OSRestoreInterrupts(e);-1
}
pub unsafe extern "C" fn HSD_SynthSFXPlayWithGroup(id:i32,vol:u8,vol2:u8,pan:u8,priority:i32,itd:i32,group:i32,p1:f32,p2:f32,a:f32,b:f32,c:f32)->i32{assert!((0..256).contains(&group));let g=group as usize;if g!=0{let old=voicelist_1844[g];if old>0{let n=getNode(old);if !n.is_null()&&(*n).flags!=1{if (*n).x27==1{if let Some(cb)=driverInactivatedCallback{cb((*n).x0);}}HSD_SynthSFXKeyOff(old);}}}let r=HSD_Synth_80389334(id,vol,vol2,pan,priority,itd,p1,p2,a,b,c);voicelist_1844[g]=r;r}

pub unsafe extern "C" fn HSD_SynthCallback(){let base=(HSD_Synth_804D775C%8*8) as usize;let e=ext::OSDisableInterrupts();loop{let old=HSD_Synth_804D7720;HSD_Synth_804D7720=old.wrapping_sub(1);if old<=0{break;}let v=voicelist[HSD_Synth_804D7720 as usize];if !v.is_null(){ext::AXFreeVoice(v);}}HSD_Synth_804D7720=0;for i in 0..8{let n=&raw mut hsd_SynthSFXNodes[base+i];if (*n).x0>0&&(*n).flags&8==0&&(*(*n).voice[0]).pb.state==0&&((*n).voice_count==1||(*(*n).voice[1]).pb.state==0){HSD_SynthSFXStopNode(n);}}HSD_SynthSFXVolumeEnvelope();if let Some(cb)=driverMasterClockCallback{cb(HSD_Synth_804D775C);}HSD_SynthPStreamMasterClockCallback();HSD_Synth_804D775C=HSD_Synth_804D775C.wrapping_add(1);ext::OSRestoreInterrupts(e);}
pub unsafe extern "C" fn HSD_SynthResetStreamCounters(_:i32,_:usize,_:*mut c_void,_:i32){HSD_Synth_804D776C=HSD_Synth_804D7768;HSD_Synth_804D7778=0;}
pub unsafe extern "C" fn HSD_SynthPStreamHakoHeaderCallback(_:i32,src:usize,_:*mut c_void,_:i32){ext::HSD_DevComRequest(HSD_Synth_804D7764,src,HSD_Synth_804D7780.wrapping_add(HSD_Synth_804D7768<<16) as usize,pstHakoHeader[HSD_Synth_804D7768 as usize].x0 as u32 as usize,0x23,0,Some(HSD_SynthResetStreamCounters),0);}
pub unsafe fn HSD_SynthPStreamMasterClockCallback_inline(pos:u32){if (HSD_Synth_804D776C+1)%3!=pos{let e=ext::OSDisableInterrupts();if HSD_Synth_804D7778!=0||HSD_Synth_804D7768!=HSD_Synth_804D776C{ext::OSRestoreInterrupts(e);return;}if !getNode(HSD_Synth_804D7760).is_null(){let src=pstHakoHeader[HSD_Synth_804D776C as usize].x8;if src == -1{HSD_Synth_804D776C=(HSD_Synth_804D776C+1)%3;}else{HSD_Synth_804D7768=(HSD_Synth_804D776C+1)%3;HSD_Synth_804D7778=1;ext::HSD_DevComRequest(HSD_Synth_804D7764,src as u32 as usize,(&raw mut pstHakoHeader[HSD_Synth_804D7768 as usize]) as usize,32,0x21,0,Some(HSD_SynthPStreamHakoHeaderCallback),src.wrapping_add(32) as u32 as usize);}}ext::OSRestoreInterrupts(e);}}
pub unsafe extern "C" fn HSD_SynthPStreamMasterClockCallback(){let n=getNode(HSD_Synth_804D7760);if n.is_null()||(*n).flags&8!=0{return;}
    let a=&(*(*n).voice[0]).pb.addr;let pos=(((a.currentAddressHi as u32)<<16|a.currentAddressLo as u32).wrapping_sub(HSD_Synth_804D7780.wrapping_mul(2)))>>17;if pos>=3{return;}
    if pos!=HSD_Synth_804D7774{HSD_Synth_804D7774=pos;for i in 0..(*n).voice_count as usize{ext::AXSetVoiceEndAddr((*n).voice[i],stream_addr(pos,i).wrapping_add(pstHakoHeader[pos as usize].x4 as u32));}}
    if pos==HSD_Synth_804D7770&&pos!=HSD_Synth_804D776C{if pstHakoHeader[HSD_Synth_804D7770 as usize].x8 == -1{HSD_Synth_804D7770=(HSD_Synth_804D7770+1)%3;for i in 0..(*n).voice_count as usize{ext::AXSetVoiceLoop((*n).voice[i],0);ext::AXSetVoiceLoopAddr((*n).voice[i],HSD_Synth_804D7784);}}
        else{HSD_Synth_804D7770=(HSD_Synth_804D7770+1)%3;for i in 0..(*n).voice_count as usize{ext::AXSetVoiceLoopAddr((*n).voice[i],stream_addr(HSD_Synth_804D7770,i).wrapping_add(2));let p=(&raw mut pstHakoHeader[HSD_Synth_804D7770 as usize]).cast::<u32>().add(i*2+3);ext::AXSetVoiceAdpcmLoop((*n).voice[i],p.cast());}}}
    HSD_SynthPStreamMasterClockCallback_inline(pos);
}
unsafe fn stream_addr(pos:u32,i:usize)->u32{HSD_Synth_804D7780.wrapping_add(pos<<16).wrapping_mul(2).wrapping_add((i as u32).wrapping_mul(pstHakoHeader[pos as usize].x0 as u32))}
pub unsafe extern "C" fn HSD_SynthPStreamFirstHakoDataCallback(){let n=getNode(HSD_Synth_804D7760);if !n.is_null(){let mut ve=AXPBVE{currentVolume:if (*n).flags&2==0{node_volume(n) as i32 as u16}else{0},currentDelta:0};(*n).x24=ve.currentVolume;for i in 0..(*n).voice_count as usize{let v=(*n).voice[i];ext::AXSetVoiceVe(v,&mut ve);set_src_ratio(if (*n).flags&4!=0{0}else{(65536.0*((*n).x14*(*n).x18[0]*(*n).x18[1])) as u32});ext::AXSetVoiceSrc(v,&raw mut HSD_Synth_80407FD8);let a=stream_addr(HSD_Synth_804D7768,i);ext::AXSetVoiceCurrentAddr(v,a.wrapping_add(2));ext::AXSetVoiceEndAddr(v,a.wrapping_add(pstHakoHeader[HSD_Synth_804D7768 as usize].x4 as u32));ext::AXSetVoiceLoopAddr(v,a.wrapping_add(2));ext::AXSetVoiceState(v,1);}(*n).flags&=!8;HSD_SynthSFXUpdateVolume(n);HSD_SynthSFXUpdateMix(n,0);}HSD_Synth_804D7778=0;}
pub unsafe extern "C" fn first_data_adapter(_:i32,_:usize,_:*mut c_void,_:i32){HSD_SynthPStreamFirstHakoDataCallback();}
pub unsafe extern "C" fn HSD_SynthPStreamFirstHakoHeaderCallback(_:i32,_:usize,_:*mut c_void,_:i32){ext::HSD_DevComRequest(HSD_Synth_804D7764,0xa0,HSD_Synth_804D7780.wrapping_add(HSD_Synth_804D7768<<16) as usize,pstHakoHeader[HSD_Synth_804D7768 as usize].x0 as u32 as usize,0x23,0,Some(first_data_adapter),0);}
pub unsafe extern "C" fn HSD_SynthPStreamHeaderCallback(_:i32,_:usize,buf:*mut c_void,_:i32){let entry=buf.cast::<u32>();let n=getNode(HSD_Synth_804D7760);if !n.is_null(){(*n).voice_count=*entry.add(3) as u8;if (*n).voice_count==2{(*n).voice[1]=ext::AXAcquireVoice(29,Some(dropcallback),0);assert!(!(*n).voice[1].is_null());}(*n).x14=0.00003125*(*entry.add(2) as f32);for i in 0..(*n).voice_count as usize{set_src_ratio((65536.0*(*n).x14) as u32);ext::AXSetVoiceAddr((*n).voice[i],entry.add(i*14+4).cast());ext::AXSetVoiceAdpcm((*n).voice[i],entry.add(i*14+8).cast());}HSD_Synth_804D7774=(HSD_Synth_804D7774+2)%3;HSD_Synth_804D776C=HSD_Synth_804D7774;HSD_Synth_804D7770=HSD_Synth_804D7774;HSD_Synth_804D7768=HSD_Synth_804D7774;ext::HSD_DevComRequest(HSD_Synth_804D7764,128,(&raw mut pstHakoHeader[HSD_Synth_804D7768 as usize]) as usize,32,0x21,0,Some(HSD_SynthPStreamFirstHakoHeaderCallback),0);}else{HSD_Synth_804D7778=0;}}
pub unsafe fn HSD_SynthPStreamStart_inline(){let n=getNode(HSD_Synth_804D7760);if n.is_null(){return;}if (*n).flags&8!=0{HSD_SynthSFXStopNode(n);}else if (*n).flags&1==0{(*n).flags|=1;HSD_SynthSFXUpdateVolume(n);for i in 0..(*n).voice_count as usize{ext::AXSetVoicePriority((*n).voice[i],1);}}}
pub unsafe extern "C" fn HSD_SynthPStreamStart(entry:i32,vol:u8,vol2:u8,ch:i32)->i32 {loop{ext::wait_idle();if core::ptr::read_volatile(&raw const HSD_Synth_804D7778)==0{break;}}HSD_Synth_804D7778=1;let e=ext::OSDisableInterrupts();if !getNode(HSD_Synth_804D7760).is_null(){HSD_SynthPStreamStart_inline();}HSD_Synth_804D7764=entry;let v=ext::AXAcquireVoice(29,Some(dropcallback),0);let idx=(*v).index;let n=hsd_SynthSFXNodeAt(idx as i32);HSD_Synth_804D7750=HSD_Synth_804D7750.wrapping_add(64);if HSD_Synth_804D7750<0{HSD_Synth_804D7750=64;}(*n).x27=2;HSD_Synth_804D7760=HSD_Synth_804D7750.wrapping_add(idx as i32);(*n).x0=HSD_Synth_804D7760;(*n).sfx_id=0;(*n).flags=8;(*n).voice_count=1;(*n).xB=ch as u8;(*n).voice[0]=v;(*n).x18=[1.0;2];(*n).unk28=0.003921569*vol as f32;(*n).user_vol[0].volume=(*n).unk28;(*n).user_vol[0].x4=0;(*n).user_vol[0].anon_8.x8_float=0.003921569*vol2 as f32;(*n).user_vol[1].volume=0.003921569*vol2 as f32;(*n).user_vol[1].x4=0;(*n).user_vol[1].anon_8.x8=128;(*n).x44=1.0;(*n).x48=0.0;(*n).x4C=0.0;ext::HSD_DevComRequest(entry,0,0,128,0x22,1,Some(HSD_SynthPStreamHeaderCallback),0);ext::OSRestoreInterrupts(e);HSD_Synth_804D7760}
pub unsafe extern "C" fn HSD_SynthStreamSetVolume(v:f32){HSD_Synth_804D6030=v;ext::AISetStreamVolLeft((v*HSD_Synth_804D777C as f32) as i32 as u8);ext::AISetStreamVolRight((v*HSD_Synth_804D777C as f32) as i32 as u8);updateAllVolume(65535);}
pub unsafe extern "C" fn HSD_SynthInit(_:i32,_:i32,_:i32,bank_size:i32){ext::AXInit();ext::AISetDSPSampleRate(0);HSD_Synth_804D7784=ext::ARAlloc(1280);ext::HSD_DevComRequest(0,0,HSD_Synth_804D7784 as usize,1280,3,0,None,0);HSD_Synth_804D7784=HSD_Synth_804D7784.wrapping_mul(2);hsd_SynthSFXBankHead[0]=ext::ARAlloc(bank_size as u32) as i32;ext::AXRegisterCallback(Some(HSD_SynthCallback));ext::watch_page((&raw mut hsd_SynthSFXNodes).cast(),core::mem::size_of_val(&hsd_SynthSFXNodes) as u32);HSD_Synth_804D777C=255;ext::AISetStreamVolLeft((HSD_Synth_804D6030*255.0) as i32 as u8);ext::AISetStreamVolRight((HSD_Synth_804D6030*255.0) as i32 as u8);for v in &mut voicelist_1784{v.x1784=1.0;v.x1788=1.0;v.x178C=0;}hsd_SynthSFXBankAREnd=hsd_SynthSFXBankHead[0].wrapping_add(bank_size) as u32;HSD_Synth_804D7780=ext::ARAlloc(0x30000);HSD_Synth_804D7754=ext::OSGetSoundMode();}

// SfxLoadStreamNode is the actual group object. The C calls it AXVPB to reuse
// the GC offsets; widened pointers make those aliases invalid on a native host.
pub unsafe fn HSD_SynthSFXGroupDataUnlink(vpb:*mut AXVPB){let g=vpb.cast::<SfxLoadStreamNode>();for i in 0..(*g).xC{HSD_SynthSFXDataUnlink((*g).x8.wrapping_add(i));}}
pub unsafe extern "C" fn HSD_SynthSFXUnloadBank(bank:i32){HSD_SynthSFXStopRange(bank);let h=&raw mut HSD_Synth_804C2AE0[bank as usize];while !(*h).is_null(){HSD_SynthSFXGroupDataUnlink(*h);let cur=(*h).cast::<SfxLoadStreamNode>();*h=(*cur).x0.cast();HSD_AudioFree(cur.cast());}hsd_SynthSFXBank[bank as usize]=hsd_SynthSFXBankHead[bank as usize];}
pub unsafe extern "C" fn HSD_SynthSFXDataUnlink(id:i32){let mut p=&raw mut hsd_SynthSFXDataHash[(id&31) as usize];while !(*p).is_null(){let cur=(*p).cast::<HSD_SynthVoiceEntry>();if (*cur).unk4==id{*p=(*cur).next;return;}p=&raw mut (*cur).next;}}
pub unsafe extern "C" fn HSD_SynthSFXGroupDataRemove(id:i32){for i in 0..32{let mut p=(&raw mut HSD_Synth_804C2AE0[i]).cast::<*mut SfxLoadStreamNode>();while !(*p).is_null(){let cur=*p;if (*cur).x4==id{HSD_SynthSFXGroupDataUnlink(cur.cast());*p=(*cur).x0;HSD_AudioFree(cur.cast());return;}p=&raw mut (*cur).x0;}}}
unsafe fn add_addr(a:*mut AXPBADDR,delta:u32,loop_test:bool){if !loop_test||(*a).loopFlag!=0{let v=(((*a).loopAddressHi as u32)<<16|(*a).loopAddressLo as u32).wrapping_add(delta);(*a).loopAddressHi=(v>>16) as u16;(*a).loopAddressLo=v as u16;}let v=(((*a).endAddressHi as u32)<<16|(*a).endAddressLo as u32).wrapping_add(delta);(*a).endAddressHi=(v>>16) as u16;(*a).endAddressLo=v as u16;let v=(((*a).currentAddressHi as u32)<<16|(*a).currentAddressLo as u32).wrapping_add(delta);(*a).currentAddressHi=(v>>16) as u16;(*a).currentAddressLo=v as u16;}
// Pointer+id grows by four bytes; round entries to native pointer alignment.
fn entry_size(voices:i32)->usize{(voices as usize*64+core::mem::offset_of!(HSD_SynthVoiceEntry,unk8)+8+7)&!7}
pub unsafe extern "C" fn HSD_SynthSFXGroupDataReaddress(vpb:*mut AXVPB,callback:*mut c_void){let g=vpb.cast::<SfxLoadStreamNode>();let dest=callback as usize as u32;ext::HSD_DevComRequest(0,(*g).x10 as u32 as usize,callback as usize,(*g).x14 as u32 as usize,0x1b,0,Some(HSD_SynthSFXGroupDataReaddressCallback),0);let delta=dest.wrapping_sub((*g).x10 as u32).wrapping_mul(2);let mut p=g.add(1).cast::<u8>();for _ in 0..(*g).xC{let entry=p.cast::<HSD_SynthVoiceEntry>();let count=(*entry).unk8;for j in 0..count as usize{let q=p.add(j*64).cast::<HSD_SynthVoiceEntry>();add_addr(&raw mut (*q).x10,delta,true);}p=p.add(entry_size(count));}(*g).x10=dest as i32;}
pub unsafe extern "C" fn HSD_SynthSFXBankDeflag(bank:i32){HSD_SynthSFXStopRange(bank);let mut g=HSD_Synth_804C2AE0[bank as usize].cast::<SfxLoadStreamNode>();let mut offset=hsd_SynthSFXBankHead[bank as usize] as u32;while !g.is_null(){if (*g).x10 as u32!=offset{HSD_SynthSFXGroupDataReaddress(g.cast(),offset as usize as *mut c_void);}offset=offset.wrapping_add((*g).x14 as u32);g=(*g).x0;}HSD_Synth_804C2AE0[bank as usize+32]=offset as usize as *mut AXVPB;}
pub unsafe extern "C" fn HSD_SynthSFXHeaderLoadCallback(_:i32,_:usize,_:*mut c_void,_:i32){if HSD_Synth_804D7738==0{let bank=HSD_Synth_804C2A60[0].bankID as usize;if hsd_SynthSFXBankHead[bank+1].wrapping_sub(hsd_SynthSFXBank[bank])<(hsd_SynthSFXLoadBuf[1] as i32){HSD_SynthSFXUnloadBank(bank as i32);}assert!((hsd_SynthSFXBankHead[bank+1].wrapping_sub(hsd_SynthSFXBank[bank]) as u32)>=hsd_SynthSFXLoadBuf[1]);let alloc=hsd_SynthSFXLoadBuf[2] as usize*16+core::mem::size_of::<SfxLoadStreamNode>();let header=hsd_SynthSFXLoadBuf[0] as usize;HSD_Synth_804D7730=HSD_AudioMalloc((alloc+header+31)&!31).cast();HSD_Synth_804D6028[1]=ext::HSD_DevComRequest(HSD_Synth_804C2A60[0].entrynum,32,HSD_Synth_804D7730 as usize,(header-16+31)&!31,0x21,1,None,0);HSD_Synth_804D6028[0]=ext::HSD_DevComRequest(HSD_Synth_804C2A60[0].entrynum,(header+16+31)&!31,hsd_SynthSFXBank[bank] as u32 as usize,hsd_SynthSFXLoadBuf[1] as usize,0x23,1,Some(HSD_SynthSFXSampleLoadCallback),0);}else{HSD_Synth_804D7730=null_mut();HSD_SynthSFXSampleLoadCallback(0,0,null_mut(),0);}}
pub unsafe extern "C" fn HSD_SynthSFXSampleLoadCallback(_:i32,_:usize,_:*mut c_void,_:i32){
    if HSD_Synth_804D7738==0{let header=hsd_SynthSFXLoadBuf[0] as usize;let data=header-16;let alloc=hsd_SynthSFXLoadBuf[2] as usize*16+core::mem::size_of::<SfxLoadStreamNode>();let total=(alloc+header+31)&!31;let mem=HSD_Synth_804D7730.cast::<u8>();core::ptr::copy(mem,mem.add(total-data),data);let dnw=total-header;for i in 0..4{write_unaligned(mem.add(dnw+i*4).cast::<u32>(),hsd_SynthSFXLoadBuf[4+i]);}HSD_Synth_804D7734=mem.add(dnw&!3).cast();
        let bank=HSD_Synth_804C2A60[0].bankID as usize;let mut link=(&raw mut HSD_Synth_804C2AE0[bank]).cast::<*mut SfxLoadStreamNode>();while !(*link).is_null(){link=&raw mut (**link).x0;}*link=HSD_Synth_804D7730;let g=HSD_Synth_804D7730;(*g).x0=null_mut();(*g).x4=HSD_Synth_804C2A60[0].entrynum;(*g).x10=hsd_SynthSFXBank[bank];(*g).x14=hsd_SynthSFXLoadBuf[1] as i32;let count=hsd_SynthSFXLoadBuf[2] as i32;let base=hsd_SynthSFXLoadBuf[3] as i32;(*g).x8=base;(*g).xC=count;HSD_Synth_804D7730=g.add(1);
        for i in 0..count{let n=*HSD_Synth_804D7734 as i32;let nbytes=SfxLoadStreamDataSize(n.wrapping_shl(6)) as usize;let entry=HSD_Synth_804D7730.cast::<HSD_SynthVoiceEntry>();core::ptr::copy(HSD_Synth_804D7734.cast::<u8>(),(&raw mut (*entry).unk8).cast::<u8>(),nbytes);for k in 0..n as usize{let q=entry.cast::<u8>().add(k*64).cast::<HSD_SynthVoiceEntry>();add_addr(&raw mut (*q).x10,(hsd_SynthSFXBank[bank] as u32).wrapping_mul(2),false);}let id=base.wrapping_add(i);(*entry).unk4=id;let bucket=&raw mut hsd_SynthSFXDataHash[(id&31) as usize];(*entry).next=*bucket;*bucket=entry.cast();HSD_Synth_804D7734=HSD_Synth_804D7734.add(nbytes/4);HSD_Synth_804D7730=entry.cast::<u8>().add(entry_size(n)).cast();}
        if let Some(cb)=HSD_Synth_804C2A60[0].x8{cb(HSD_Synth_804C2A60[0].entrynum,HSD_Synth_804C2A60[0].xC);}hsd_SynthSFXBank[bank]=hsd_SynthSFXBank[bank].wrapping_add(hsd_SynthSFXLoadBuf[1] as i32);
    }else{if !HSD_Synth_804D7730.is_null(){HSD_AudioFree(HSD_Synth_804D7730.cast());}HSD_Synth_804D7738=0;}
    let intr=ext::OSDisableInterrupts();HSD_Synth_804D772C-=1;for i in 0..HSD_Synth_804D772C as usize{HSD_Synth_804C2A60[i]=HSD_Synth_804C2A60[i+1];}HSD_SynthSFXLoadNewProc();ext::OSRestoreInterrupts(intr);
}
