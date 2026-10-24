use super::*;
use std::{alloc::{alloc_zeroed, dealloc, Layout}, ffi::{c_void, CString}, path::PathBuf};
include!("fields.rs");

#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryW(path: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const i8) -> *mut c_void;
}
struct Api(*mut c_void);
impl Api {
    unsafe fn open() -> Self {
        let root=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let output=std::process::Command::new("python").arg(root.join("rust/melee_runtime/oracle/build_thp.py"))
            .env("GALE_INCLUDE",root.join("rust/target/empty-include")).output().unwrap();
        assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
        let path=root.join("rust/target/shard985-thp/thp_oracle.dll");
        let w:std::vec::Vec<u16>=path.as_os_str().to_string_lossy().encode_utf16().chain([0]).collect();
        let p=LoadLibraryW(w.as_ptr()); assert!(!p.is_null(),"load THP C oracle"); Self(p)
    }
    unsafe fn get<T:Copy>(&self,name:&str)->T {
        let n=CString::new(name).unwrap(); let p=GetProcAddress(self.0,n.as_ptr());
        assert!(!p.is_null(),"C oracle symbol {name}"); core::mem::transmute_copy(&p)
    }
}
macro_rules! call {
    ($native:expr,$api:expr,$name:ident,$ty:ty $(,$arg:expr)* $(,)?)=>{{
        let f:$ty=if $native { $api.get(concat!("thp_ref_",stringify!($name))) } else { super::$name };
        f($($arg),*)
    }};
}
fn rnd(s:&mut u32)->u32 { *s^=*s<<13; *s^=*s>>17; *s^=*s<<5; *s }
struct Fixture { p:*mut u8 }
impl Drop for Fixture { fn drop(&mut self) { unsafe { dealloc(self.p,Layout::from_size_align(0x80000,32).unwrap()); } } }
impl Fixture {
    unsafe fn new(seed:u32)->Self {
        let p=alloc_zeroed(Layout::from_size_align(0x80000,32).unwrap()); assert!(!p.is_null());
        let f=Self{p}; let i=&mut *f.info(); let mut s=seed|1;
        i.file=f.at(0x20000); i.dataStart=i.file; i.scanStart=i.file; i.scratch=f.at(0x1000);
        i.cnt=1; i.xSize=32; i.ySize=16; i.xPixelSize=32; i.yPixelSize=16;
        i.samplingHMax=2; i.samplingVMax=2; i.nComponents=3; i.x8D4=16; i.decompressedY=16;
        i.MCUsPerRow=2; i.nMCU=1; i.currMCU=1; i.RST=(seed%2) as u8;
        i.tileY=f.at(0x30000);i.tileU=f.at(0x40000);i.tileV=f.at(0x50000);
        for (n,c) in i.components.iter_mut().enumerate() {
            c.componentID=n as u8+1; c.samplingH=if n==0{2}else{1}; c.samplingV=c.samplingH;
            c.quantizationTableSelector=n as u8; c.predDC=(rnd(&mut s)%255) as i16-127;
            c.x08=if n==0{32}else{16}; c.x0C=if n==0{16}else{8};
        }
        for n in 0..16 { i.mcuBuffer[n]=f.at(0x1000+n*128).cast(); }
        for q in &mut i.quantTabs { for v in q { *v=((rnd(&mut s)%1000) as f32+1.0)*0.03125; } }
        for (n,h) in i.huffmanTabs.iter_mut().enumerate() {
            let off=0x22000+n*0x1000;h.bits=f.at(off);h.Vij=f.at(off+32);h.sizeTab=f.at(off+512).cast();h.codeTab=f.at(off+1024).cast();
            h.quick.fill(0);h.increment.fill(1);h.maxCode.fill(-1);h.maxCode[1]=1;h.maxCode[17]=0xfffff;
            h.valPtr.fill(0);*h.bits=2; *h.sizeTab=1;*h.sizeTab.add(1)=1;*h.codeTab=0;*h.codeTab.add(1)=1;
            h.numCodes=2;h.numVij=2;
        }
        i.validHuffmanTabs=15;i.validQuantTabs=7;
        f
    }
    unsafe fn at(&self,n:usize)->*mut u8 {self.p.add(n)}
    fn info(&self)->*mut THPFileInfo {self.p.cast()}
    unsafe fn bytes(&self,n:usize,data:&[u8]) {core::ptr::copy_nonoverlapping(data.as_ptr(),self.at(n),data.len());}
    unsafe fn bind(&self,api:&Api,native:bool) {
        if native {
            let f:unsafe extern "C" fn(*mut THPFileInfo,*mut f32,*mut u8,u32)=api.get("thp_ref_bind");
            f(self.info(),(*self.info()).quantTabs[0].as_mut_ptr(),self.at(0x30000),32);
        } else {
            Gq.value=(*self.info()).quantTabs[0].as_mut_ptr();Gbase.value=self.at(0x30000);Gwid.value=32;
            Ydchuff.value=&raw mut (*self.info()).huffmanTabs[0]; Udchuff.value=Ydchuff.value;Vdchuff.value=Ydchuff.value;
            Yachuff.value=&raw mut (*self.info()).huffmanTabs[1]; Uachuff.value=Yachuff.value;Vachuff.value=Yachuff.value;
        }
    }
    unsafe fn compare(&self,b:&Self,op:usize,seed:u32) {
        let a=state_fields(&*self.info(),self.p as usize);let bb=state_fields(&*b.info(),b.p as usize);
        for ((name,x),(_,y)) in a.iter().zip(bb.iter()) {assert_eq!(x,y,"op {op}, seed {seed}, field {name}");}
        for (start,end) in [(native_work_prefix(),0x10000),(0x20000,0x60000)] {
            let aa=core::slice::from_raw_parts(self.at(start),end-start);let bb=core::slice::from_raw_parts(b.at(start),end-start);
            if let Some(n)=aa.iter().zip(bb).position(|(a,b)|a!=b) {panic!("op {op} seed {seed} byte {:#x}: {} != {}",start+n,aa[n],bb[n]);}
        }
    }
}
unsafe fn zero(p:*mut c_void,n:u32) {core::ptr::write_bytes(p.cast::<u8>(),0,n as usize)}
unsafe fn copy(d:*mut c_void,s:*mut c_void,n:u32)->u32 {core::ptr::copy_nonoverlapping(s.cast::<u8>(),d.cast::<u8>(),n as usize);0}
unsafe fn wait(_:u32){}
unsafe fn report(_: *mut i8){}

unsafe fn run(api:&Api,native:bool,f:&Fixture,op:usize,seed:u32)->u64 {
    f.bind(api,native); let i=f.info(); let p=(*i).file; let block=(*i).mcuBuffer[0];
    type I=unsafe extern "C" fn(*mut THPFileInfo)->u8;
    type V=unsafe extern "C" fn(*mut THPFileInfo);
    type B=unsafe extern "C" fn(*mut THPFileInfo,*mut i16);
    let mut s=seed|1;
    if (7..=12).contains(&op) {
        for n in 0..1024 { *p.add(n)=rnd(&mut s) as u8; }
        (*i).currByte=u32::from_be_bytes(core::ptr::read_unaligned(p.cast()));
        if op<=8 {
            (*i).huffmanTabs[0].quick.fill(255); (*i).huffmanTabs[0].maxCode[6]=63;
            for n in 0..64 { *(*i).huffmanTabs[0].Vij.add(n)=rnd(&mut s) as u8; }
        } else {
            (*i).huffmanTabs[0].quick.fill((seed%12) as u8);
            (*i).huffmanTabs[1].quick.fill([0,0xf0,1][seed as usize%3]);
        }
    }
    match op {
        0=> {for n in 0..128 {let v=rnd(&mut s);assert_eq!(super::__THPSlw(v,n),call!(true,api,__THPSlw,unsafe extern "C" fn(u32,u32)->u32,v,n));assert_eq!(super::__THPSrw(v,n),call!(true,api,__THPSrw,unsafe extern "C" fn(u32,u32)->u32,v,n));}0}
        1=> {let t=seed%25+1;let v=rnd(&mut s)&((1<<t)-1);call!(native,api,__THPExtend,unsafe extern "C" fn(u32,u32)->i32,v,t) as u64}
        2=> {for n in 0..128 {*p.add(n)=rnd(&mut s) as u8;}call!(native,api,__THPNextWord,unsafe extern "C" fn(*mut THPFileInfo)->u32,i) as u64}
        3..=5=> {for n in 0..128 {*p.add(n)=rnd(&mut s) as u8;}(*i).currByte=rnd(&mut s);(*i).cnt=seed%33+1;
            match op {3=>call!(native,api,__THPGetBits,unsafe extern "C" fn(*mut THPFileInfo,u32)->u32,i,seed%25+1) as u64,
            4=>call!(native,api,__THPPeek5,unsafe extern "C" fn(*const THPFileInfo)->u32,i) as u64,
            _=>{call!(native,api,__THPSkipBits,unsafe extern "C" fn(*mut THPFileInfo,u32),i,seed%25+1);0}}}
        6=> {(*i).file=p.add((seed%4) as usize);(*i).cnt=33;call!(native,api,__THPPrepBitStream,V,i);0}
        7=>call!(native,api,__THPHuffDecodePC,unsafe extern "C" fn(*mut THPFileInfo,*const THPHuffmanTab)->i32,i,&raw const (*i).huffmanTabs[0]) as u64,
        8=>call!(native,api,__THPHuffDecodeTab,unsafe extern "C" fn(*mut THPFileInfo,*mut THPHuffmanTab)->i32,i,&raw mut (*i).huffmanTabs[0]) as u64,
        9=> {call!(native,api,__THPHuffDecodeBlockPC,unsafe extern "C" fn(*mut THPFileInfo,*mut i16,*const THPHuffmanTab,*const THPHuffmanTab,*mut i16),i,block,&raw const (*i).huffmanTabs[0],&raw const (*i).huffmanTabs[1],&raw mut (*i).components[0].predDC);0}
        10=> {call!(native,api,__THPHuffDecodeDCTCompY,B,i,block);0}
        11=> {call!(native,api,__THPHuffDecodeDCTCompU,B,i,block);0}
        12=> {call!(native,api,__THPHuffDecodeDCTCompV,B,i,block);0}
        13=> {let v=match seed%8 {0=>f32::NAN,1=>f32::INFINITY,2=>f32::NEG_INFINITY,_=>(rnd(&mut s) as i32) as f32*0.0001};call!(native,api,__THPStoreSample,unsafe extern "C" fn(*mut u8,f32),f.at(0x30000),v);0}
        14..=16=> {for n in 0..64 {*block.add(n)=if seed%4==0 && n%8!=0{0}else{(rnd(&mut s)%1023) as i16-511};}
            match op {14=>call!(native,api,__THPInverseDCTPC,unsafe extern "C" fn(*const i16,u32,u32),block,8,0),15=>call!(native,api,__THPInverseDCTNoYPos,unsafe extern "C" fn(*mut i16,u32),block,8),_=>call!(native,api,__THPInverseDCTY8,unsafe extern "C" fn(*mut i16,u32),block,8)};0}
        17=> {(*i).scratch=f.at(0x1000+(seed%32) as usize);call!(native,api,THPDec_803300E0,V,i);0}
        18=> {let mut h=[0,17,8,0,16,0,32,3,1,0x22,0,2,0x11,1,3,0x11,2];h[2]=if seed%5==0{12}else{8};h[4]=(seed%63+1) as u8;h[6]=(seed%64+1) as u8;f.bytes(0x20000,&h);call!(native,api,__THPReadFrameHeader,I,i) as u64}
        19=> {f.bytes(0x20000,&[0,67,(seed%3) as u8]);for n in 0..64{*p.add(n+3)=rnd(&mut s) as u8;}call!(native,api,__THPReadQuantizationTable,I,i) as u64}
        20=> {f.bytes(0x20000,&[0,21,0,2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1]);call!(native,api,__THPReadHuffmanTableSpecification,I,i) as u64}
        21=> {call!(native,api,__THPHuffGenerateSizeTable,unsafe extern "C" fn(*mut THPFileInfo,u8,usize)->u8,i,0,(*i).huffmanTabs[0].bits as usize) as u64}
        22=>call!(native,api,__THPHuffGenerateCodeTable,unsafe extern "C" fn(*mut THPFileInfo,u8)->u8,i,0) as u64,
        23=>call!(native,api,__THPHuffGenerateDecoderTables,unsafe extern "C" fn(*mut THPFileInfo,u8)->i32,i,0) as u64,
        24=>call!(native,api,THPDec_803310CC,I,i) as u64,
        25=> {f.bytes(0x20000,&[0,12,3,1,0,2,0,3,0,0,63,0]);call!(native,api,__THPReadScaneHeader,I,i) as u64}
        26=> {f.bytes(0x20000,&[0,4,(seed>>8) as u8,seed as u8]);call!(native,api,__THPRestartDefinition,I,i) as u64}
        27=> {f.bytes(0x20000,&[0,16,b'J',b'F',b'I',b'F',0,1,2,0,0,1,0,1,0,0]);(*i).dataStart=p.sub(4);if seed%3==0{*p.add(14)=1;}call!(native,api,THPDec_80330158,I,i) as u64}
        28=> {f.bytes(0x20000,&[0,(seed%254+2) as u8]);call!(native,api,THPDec_803302EC,unsafe extern "C" fn(*mut *mut u8)->i32,&raw mut (*i).file) as u64}
        29=> {let d=f.at(0x28000).cast::<THPDec_8032FD40_Data>();(*d).val0=rnd(&mut s) as i32;(*d).val1=(seed%2048) as u16;(*d).val2=if seed%4==0{2}else{4};call!(native,api,THPDec_8032FD40,unsafe extern "C" fn(*mut THPDec_8032FD40_Data,u16)->i32,d,(seed%1024) as u16) as u64}
        30=> {f.bytes(0x20000,&[255,216,255,192,0,17,8,0,16,0,32,3,1,0x22,0,2,0x11,1,3,0x11,2,255,218]);if seed%3==0{*p=0;}let d=f.at(0x28000).cast();call!(native,api,THPDec_8032F8D4,unsafe extern "C" fn(*mut u8,*mut THPDec_8032FD40_Data)->i32,p,d) as u64}
        31=> {
            let mut data=vec![255,216];
            if seed%3==1 {
                for table in 0..3 {data.extend([255,219,0,67,table]);for _ in 0..64{data.push((rnd(&mut s)%255+1) as u8);}}
                for table in [0,16,1,17] {data.extend([255,196,0,21,table,2]);data.extend([0;15]);data.extend([0,0]);}
            }
            data.extend([255,192,0,17,8,0,16,0,32,3,1,0x22,0,2,0x11,1,3,0x11,2]);
            if seed%3==1 {data.extend([255,218,0,12,3,1,0,2,0,3,0,0,63,0]);}else{data.extend([255,217]);}
            f.bytes(0x20000,&data);if seed%3==0{*p=0;}
            let h=f.at(0x28000).cast::<THPVideoDecodeHeader>();(*h).xSize=32;(*h).ySize=16;
            let r=call!(native,api,THPVideoDecode,unsafe extern "C" fn(*mut c_void,*mut c_void,*mut THPFileInfo,*mut c_void,*mut THPDec_8032FD40_Data)->*mut THPFileInfo,h.cast(),f.at(0x28010).cast(),i,p.cast(),core::ptr::null_mut());
            assert_eq!(!r.is_null(),seed%3==1,"THPVideoDecode fixture");(!r.is_null()) as u64
        }
        32..=35=> {
            call!(native,api,THPInit,unsafe extern "C" fn());
            let mut work=[core::ptr::null_mut();3];if native {let w:unsafe extern "C" fn(*mut *mut u8)=api.get("thp_ref_work");w(work.as_mut_ptr());}else{work=__THPLCWork672;}
            for (n,&wp) in work.iter().enumerate(){core::ptr::write_bytes(wp,0,if n==0{0x2a00}else{0xa80});}
            (*i).MCUsPerRow=if op==32||op==34{40}else{2};
            match op {32=>call!(native,api,__THPDecompressiMCURow640x480,V,i),33=>call!(native,api,__THPDecompressiMCURowNxN,unsafe extern "C" fn(*mut THPFileInfo,u32),i,32),34=>call!(native,api,THPDec_80331340,unsafe extern "C" fn(*mut THPFileInfo,*mut c_void,*mut c_void,*mut c_void),i,f.at(0x30000).cast(),f.at(0x40000).cast(),f.at(0x50000).cast()),_=>call!(native,api,THPDec_803313D0,unsafe extern "C" fn(*mut THPFileInfo,*mut c_void,*mut c_void,*mut c_void,u32),i,f.at(0x30000).cast(),f.at(0x40000).cast(),f.at(0x50000).cast(),32)};0
        }
        36=> {call!(native,api,OSInitFastCast,unsafe extern "C" fn());call!(native,api,THPInit,unsafe extern "C" fn());let mut w=[core::ptr::null_mut();3];if native {let get:unsafe extern "C" fn(*mut *mut u8)=api.get("thp_ref_work");get(w.as_mut_ptr());}else{w=__THPLCWork672;}assert_eq!(w[1].offset_from(w[0]),0x2a00);assert_eq!(w[2].offset_from(w[1]),0xa80);0}
        _=>unreachable!()
    }
}

#[test]
fn all_thp_definitions_match_native_c_state() { unsafe {
    let api=Api::open();ext::DCZeroRange_HOOK=Some(zero);ext::LCStoreData_HOOK=Some(copy);ext::LCQueueWait_HOOK=Some(wait);ext::OSReport_HOOK=Some(report);
    for op in 0..37 {for seed in 1..=64 {
        if std::env::var_os("THP_TRACE").is_some() {eprintln!("THP op {op} seed {seed}");}
        let a=Fixture::new(seed);let b=Fixture::new(seed);
        let x=run(&api,false,&a,op,seed);
        if std::env::var_os("THP_TRACE").is_some() {eprintln!("THP C");}
        let y=run(&api,true,&b,op,seed);
        assert_eq!(x,y,"op {op} seed {seed} return"); a.compare(&b,op,seed);
    }}
} }

#[test]
fn thp_shift_semantics() {
    unsafe {
        assert_eq!(__THPSlw(0x8000_0001, 32), 0);
        assert_eq!(__THPSlw(0x8000_0001, 64), 0x8000_0001);
    }
}
