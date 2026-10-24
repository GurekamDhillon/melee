//! Synthetic vectors from the actual C definitions, including partial writes on exhaustion.
use melee_runtime::tu::{hsd_3B34 as enc, hsd_3B5C as dec};
use melee_types::{JpegOutput, JpegState, JpegWork,JpegComment,JpegHuffDc,JpegHuffAc};

fn num<T: std::str::FromStr>(it: &mut std::str::SplitWhitespace<'_>) -> T {
    it.next().unwrap().parse().ok().unwrap()
}
fn hex(it: &mut std::str::SplitWhitespace<'_>) -> u32 { u32::from_str_radix(it.next().unwrap(),16).unwrap() }
unsafe fn outinit(out: &mut [u8], cap:i32) {
    out.fill(0xcc); enc::hsd_804D79A0=out.as_mut_ptr(); enc::hsd_804D79A4=out.as_mut_ptr();
    enc::hsd_804D79A8=cap; enc::hsd_804D79AC=0; enc::hsd_804D79B0=[0;8];
}
unsafe fn ininit(src:&mut [u8],len:i32) {
    dec::hsd_804D79B8=src.as_mut_ptr(); dec::hsd_804D79BC=src.as_mut_ptr();
    dec::hsd_804D79C0=len; dec::hsd_804D79C4=0; dec::hsd_804D79C8=0;
}
fn check_bytes(it:&mut std::str::SplitWhitespace<'_>,actual:&[u8]) {
    for (i,&v) in actual.iter().enumerate() { assert_eq!(v,hex(it) as u8,"byte {i}"); }
}
fn check_ints(it:&mut std::str::SplitWhitespace<'_>,actual:&[i32]) {
    for (i,&v) in actual.iter().enumerate() { assert_eq!(v,num::<i32>(it),"word {i}"); }
}
#[test]
fn every_jpeg_definition_matches_native_c() { unsafe {
    enc::ext::HSD_803B3344=Some(enc::write_byte);
    enc::ext::HSD_803B3398=Some(enc::write_bulk);
    let mut out=vec![0;32768]; let mut image=[0u16;1024];
    for (line,row) in include_str!("data/oracle_shard_jpeg.txt").lines().enumerate() {
        let mut it=row.split_whitespace(); let kind=it.next().unwrap();
        match kind {
            "D" => {
                let mut b=[0;64]; for v in &mut b { *v=num(&mut it); }
                enc::fn_803B376C(b.as_mut_ptr().cast()); check_ints(&mut it,&b);
                dec::fn_803B61B4(b.as_mut_ptr()); check_ints(&mut it,&b);
            }
            "P" => {
                let w=num(&mut it);let x=num(&mut it);let y=num(&mut it);
                for v in &mut image { *v=hex(&mut it) as u16; }
                enc::hsd_803B3408(image.as_mut_ptr().cast(),x,y,w,32);
                let ew=&*core::ptr::addr_of!(enc::hsd_804D2648);
                check_ints(&mut it,&ew.x118);check_ints(&mut it,&ew.x518);check_ints(&mut it,&ew.x618);
                dec::hsd_804D2E70.work.luma=ew.x118;dec::hsd_804D2E70.work.cb=ew.x518;dec::hsd_804D2E70.work.cr=ew.x618;
                out.fill(0xcc);dec::fn_803B6820(out.as_mut_ptr(),x,y,w,32);
                check_ints(&mut it,&(*core::ptr::addr_of!(dec::hsd_804D2E70)).work.luma);check_bytes(&mut it,&out[..2048]);
            }
            "H" => {
                let v=num(&mut it);assert_eq!(enc::hsd_803B3CD8_bit_length(v),num::<i32>(&mut it));
                assert_eq!(enc::jpeg_run_bit_length(v),num::<i32>(&mut it));
                let f=num(&mut it);assert_eq!(dec::jpeg_clamp(f),num::<i32>(&mut it));
                let y=num(&mut it);let cb=num(&mut it);let cr=num(&mut it);let mut pixel=0;
                let mut o=JpegOutput{pixels:&mut pixel};dec::jpeg_store_rgb565(&mut o,0,y,cb,cr);assert_eq!(pixel,num::<u16>(&mut it));
            }
            "W" => {
                let which=num::<i32>(&mut it);let cap=num(&mut it);outinit(&mut out,cap);enc::hsd_803B5C2C(cap%14-2);
                let mut work=core::ptr::addr_of_mut!(enc::hsd_804D2648);
                let result=enc::jpeg_try(||{match which {
                    0=>enc::hsd_803B46D4(),1=>enc::hsd_803B4A2C(),2=>enc::hsd_803B4D64(0x1234,0x5678),
                    3=>enc::hsd_803B3CD8_write_byte(255,&mut work),4=>enc::hsd_803B3CD8_write_bits(-1,25,&mut work),
                    5=>enc::jpeg_write_run_payload(254,16,&mut work),6=>enc::hsd_803B3CD8_write_bits(0x1bcdef,25,&mut work),_=>unreachable!()};0});
                assert_eq!(result.is_none() as i32,num::<i32>(&mut it),"row {line}");
                assert_eq!(enc::hsd_804D79A0.offset_from(out.as_ptr()) as i32,num::<i32>(&mut it));
                assert_eq!({enc::hsd_804D79AC},num::<i32>(&mut it));assert_eq!(enc::hsd_804D79B0[0],num::<u8>(&mut it));check_bytes(&mut it,&out[..160]);
            }
            "C" => {
                let component=num(&mut it);enc::hsd_804D2648=JpegWork::zeroed();dec::hsd_804D2E70=JpegState::zeroed();outinit(&mut out,32768);
                for i in 0..64 {enc::hsd_804D2648.coef[i]=num(&mut it);}
                let(mut av,mut run,mut val,mut index)=(0,0,0,0);
                enc::jpeg_encode_component(component,&mut av,&mut run,&mut val,&mut index);
                for actual in [av,run,val,index,enc::hsd_804D79A0.offset_from(out.as_ptr()) as i32,enc::hsd_804D79AC,enc::hsd_804D79B0[0] as i32] {assert_eq!(actual,num::<i32>(&mut it),"row {line}");}
                check_bytes(&mut it,&out[..128]);let mut w=core::ptr::addr_of_mut!(enc::hsd_804D2648);enc::hsd_803B3CD8_write_bits(0,8,&mut w);
                let len=enc::hsd_804D79A0.offset_from(out.as_ptr()) as i32;ininit(&mut out,len);
                let fail=enc::jpeg_try(||{dec::hsd_803B5EA0(component);0}).is_none();assert_eq!(fail as i32,num::<i32>(&mut it));
                for actual in [dec::hsd_804D79B8.offset_from(out.as_ptr()) as i32,dec::hsd_804D79C4,dec::hsd_804D79C8 as i32] {assert_eq!(actual,num::<i32>(&mut it));}
                check_ints(&mut it,&(*core::ptr::addr_of!(dec::hsd_804D2E70)).work.coeff);
            }
            "R" => {
                let value=num::<u32>(&mut it);let bits=num(&mut it);let initial=num(&mut it);outinit(&mut out,32768);
                let mut w=core::ptr::addr_of_mut!(enc::hsd_804D2648);enc::hsd_803B3CD8_write_bits(value as i32,bits,&mut w);enc::hsd_803B3CD8_write_bits(0,8,&mut w);ininit(&mut out,32768);
                assert_eq!(dec::hsd_803B5C4C_read(initial,bits),num::<i32>(&mut it));
                for actual in [dec::hsd_804D79B8.offset_from(out.as_ptr()) as i32,dec::hsd_804D79C4,dec::hsd_804D79C8 as i32] {assert_eq!(actual,num::<i32>(&mut it));}
            }
            "E"=>{
                let width=num(&mut it);let height=num(&mut it);let cap=num(&mut it);let quality=num(&mut it);
                for v in &mut image {*v=hex(&mut it) as u16;}enc::hsd_803B5C2C(quality);outinit(&mut out,cap);
                let size=enc::hsd_803B51C8(image.as_mut_ptr().cast(),width,height,out.as_mut_ptr().cast(),cap);
                for actual in [size,enc::hsd_804D79A0.offset_from(out.as_ptr()) as i32,enc::hsd_804D79AC,enc::hsd_804D79B0[0] as i32] {assert_eq!(actual,num::<i32>(&mut it),"encode row {line}");}
                check_bytes(&mut it,&out[..4096]);let mut decoded=vec![0xcc;32768];let n=dec::hsd_803B6BE4(out.as_mut_ptr().cast(),size,decoded.as_mut_ptr().cast());assert_eq!(n,num::<i32>(&mut it));check_bytes(&mut it,&decoded[..2048]);
                let ew=&*core::ptr::addr_of!(enc::hsd_804D2648);let dw=&*core::ptr::addr_of!(dec::hsd_804D2E70.work);
                for a in [&ew.x118[..],&ew.x518[..],&ew.x618[..],&ew.coef[..],&ew.prev_dc[..],&dw.luma[..],&dw.cb[..],&dw.cr[..],&dw.coeff[..],&dw.prev_dc[..]] {check_ints(&mut it,a);}
                check_bytes(&mut it,&(*core::ptr::addr_of!(dec::QUANT)).luma);check_bytes(&mut it,&(*core::ptr::addr_of!(dec::QUANT)).chroma);
                for actual in [dec::hsd_804D79B8.offset_from(out.as_ptr()) as i32,dec::hsd_804D79C4,dec::hsd_804D79C8 as i32] {assert_eq!(actual,num::<i32>(&mut it));}
            }
            "I"=>{
                let cap=num(&mut it);outinit(&mut out,cap);
                let mut c=JpegComment{data:[0xdd_u8 as i8;21]};let mut dl=JpegHuffDc{data:[0xdd;28]};let mut dc=dl;let mut al=JpegHuffAc{data:[0xdd;178]};let mut ac=al;
                let n=enc::hsd_803B51C8_inline(image.as_mut_ptr().cast(),0,0,out.as_ptr().cast(),cap,&enc::lbl_803B9670,&mut c,&mut dl,&mut dc,&mut al,&mut ac);
                assert_eq!(n,num::<i32>(&mut it));assert_eq!(enc::hsd_804D79A0.offset_from(out.as_ptr()) as i32,num::<i32>(&mut it));
                for v in c.data {assert_eq!(v as u8,hex(&mut it) as u8);}for a in [&dl.data[..],&dc.data[..],&al.data[..],&ac.data[..]]{check_bytes(&mut it,a);}check_bytes(&mut it,&out[..700]);
            }
            "F"=>{
                let ac=num(&mut it);let component=num(&mut it);let code=num(&mut it);let len=num(&mut it);outinit(&mut out,32768);
                let mut w=core::ptr::addr_of_mut!(enc::hsd_804D2648);enc::hsd_803B3CD8_write_bits(code,len,&mut w);enc::hsd_803B3CD8_write_bits(0,16,&mut w);ininit(&mut out,32768);
                assert_eq!(dec::hsd_803B5D70(ac,component),num::<i32>(&mut it));
                for actual in [dec::hsd_804D79B8.offset_from(out.as_ptr()) as i32,dec::hsd_804D79C4,dec::hsd_804D79C8 as i32] {assert_eq!(actual,num::<i32>(&mut it));}
            }
            "B"=>{
                let test=num::<i32>(&mut it);out.fill(0);out[0]=if test&1!=0{255}else{0x93};out[1]=if test&2!=0{0}else{1};ininit(&mut out,test%4);
                let result=enc::jpeg_try(||dec::hsd_803B5C4C(test/4+1));assert_eq!(result.is_none() as i32,num::<i32>(&mut it));assert_eq!(result.unwrap_or(0),num::<i32>(&mut it));
                for actual in [dec::hsd_804D79B8.offset_from(out.as_ptr()) as i32,dec::hsd_804D79C4,dec::hsd_804D79C8 as i32] {assert_eq!(actual,num::<i32>(&mut it));}
            }
            _=>panic!("row {line}: {kind}"),
        }
        assert!(it.next().is_none(),"unconsumed row {line}");
    }
} }
