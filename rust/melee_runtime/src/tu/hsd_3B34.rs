//! Native JPEG encoder. Serialized table words are converted once; working state is native.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]
use melee_types::{JpegWork,JpegEncodeTables,JpegMetadata,JpegComment,JpegHuffDc,JpegHuffAc};
use melee_core::fp::fmadds;
use crate::cabi::{sdiv,shl};
use super::hsd_3B5C::{lbl_80431638,lbl_80431678,lbl_80431690,lbl_8043169C,lbl_804316B4};
pub static TABLES:JpegEncodeTables=JpegEncodeTables{quant_luma:[16,11,10,16,24,40,51,61,12,12,14,19,26,58,60,55,14,13,16,24,40,57,69,56,14,17,22,29,51,87,80,62,18,22,37,56,68,109,103,77,24,35,55,64,81,104,113,92,49,64,78,87,103,121,120,101,72,92,95,98,112,100,103,99],quant_chroma:[17,18,24,47,99,99,99,99,18,21,26,66,99,99,99,99,24,26,56,99,99,99,99,99,47,66,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99],ac_code_luma:[10,0,1,4,11,26,120,248,1014,65410,65411,12,27,121,502,2038,65412,65413,65414,65415,65416,28,249,1015,4084,65417,65418,65419,65420,65421,65422,58,503,4085,65423,65424,65425,65426,65427,65428,65429,59,1016,65430,65431,65432,65433,65434,65435,65436,65437,122,2039,65438,65439,65440,65441,65442,65443,65444,65445,123,4086,65446,65447,65448,65449,65450,65451,65452,65453,250,4087,65454,65455,65456,65457,65458,65459,65460,65461,504,32704,65462,65463,65464,65465,65466,65467,65468,65469,505,65470,65471,65472,65473,65474,65475,65476,65477,65478,506,65479,65480,65481,65482,65483,65484,65485,65486,65487,1017,65488,65489,65490,65491,65492,65493,65494,65495,65496,1018,65497,65498,65499,65500,65501,65502,65503,65504,65505,2040,65506,65507,65508,65509,65510,65511,65512,65513,65514,65515,65516,65517,65518,65519,65520,65521,65522,65523,65524,2041,65525,65526,65527,65528,65529,65530,65531,65532,65533,65534],ac_length_luma:[4,2,2,3,4,5,7,8,10,16,16,4,5,7,9,11,16,16,16,16,16,5,8,10,12,16,16,16,16,16,16,6,9,12,16,16,16,16,16,16,16,6,10,16,16,16,16,16,16,16,16,7,11,16,16,16,16,16,16,16,16,7,12,16,16,16,16,16,16,16,16,8,12,16,16,16,16,16,16,16,16,9,15,16,16,16,16,16,16,16,16,9,16,16,16,16,16,16,16,16,16,9,16,16,16,16,16,16,16,16,16,10,16,16,16,16,16,16,16,16,16,10,16,16,16,16,16,16,16,16,16,11,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,11,16,16,16,16,16,16,16,16,16,16],pad_266:[0,0],ac_code_chroma:[0,1,4,10,24,25,56,120,500,1014,4084,11,57,246,501,2038,4085,65416,65417,65418,65419,26,247,1015,4086,32706,65420,65421,65422,65423,65424,27,248,1016,4087,65425,65426,65427,65428,65429,65430,58,502,65431,65432,65433,65434,65435,65436,65437,65438,59,1017,65439,65440,65441,65442,65443,65444,65445,65446,121,2039,65447,65448,65449,65450,65451,65452,65453,65454,122,2040,65455,65456,65457,65458,65459,65460,65461,65462,249,65463,65464,65465,65466,65467,65468,65469,65470,65471,503,65472,65473,65474,65475,65476,65477,65478,65479,65480,504,65481,65482,65483,65484,65485,65486,65487,65488,65489,505,65490,65491,65492,65493,65494,65495,65496,65497,65498,506,65499,65500,65501,65502,65503,65504,65505,65506,65507,2041,65508,65509,65510,65511,65512,65513,65514,65515,65516,16352,65517,65518,65519,65520,65521,65522,65523,65524,65525,1018,32707,65526,65527,65528,65529,65530,65531,65532,65533,65534],ac_length_chroma:[2,2,3,4,5,5,6,7,9,10,12,4,6,8,9,11,12,16,16,16,16,5,8,10,12,15,16,16,16,16,16,5,8,10,12,16,16,16,16,16,16,6,9,16,16,16,16,16,16,16,16,6,10,16,16,16,16,16,16,16,16,7,11,16,16,16,16,16,16,16,16,7,11,16,16,16,16,16,16,16,16,8,16,16,16,16,16,16,16,16,16,9,16,16,16,16,16,16,16,16,16,9,16,16,16,16,16,16,16,16,16,9,16,16,16,16,16,16,16,16,16,9,16,16,16,16,16,16,16,16,16,11,16,16,16,16,16,16,16,16,16,14,16,16,16,16,16,16,16,16,16,10,15,16,16,16,16,16,16,16,16,16],pad_44E:[0,0]};
pub static lbl_803B9670:JpegMetadata=JpegMetadata{comment:JpegComment{data:[72,65,76,32,76,97,98,111,114,97,116,111,114,121,44,32,73,110,99,46,0]},comment_pad:[0;3],huff_dc_luma:JpegHuffDc{data:[0,1,5,1,1,1,1,1,1,0,0,0,0,0,0,0,0,1,2,3,4,5,6,7,8,9,10,11]},huff_dc_chroma:JpegHuffDc{data:[0,3,1,1,1,1,1,1,1,1,1,0,0,0,0,0,0,1,2,3,4,5,6,7,8,9,10,11]},huff_ac_luma:JpegHuffAc{data:[0,2,1,3,3,2,4,3,5,5,4,4,0,0,1,125,1,2,3,0,4,17,5,18,33,49,65,6,19,81,97,7,34,113,20,50,129,145,161,8,35,66,177,193,21,82,209,240,36,51,98,114,130,9,10,22,23,24,25,26,37,38,39,40,41,42,52,53,54,55,56,57,58,67,68,69,70,71,72,73,74,83,84,85,86,87,88,89,90,99,100,101,102,103,104,105,106,115,116,117,118,119,120,121,122,131,132,133,134,135,136,137,138,146,147,148,149,150,151,152,153,154,162,163,164,165,166,167,168,169,170,178,179,180,181,182,183,184,185,186,194,195,196,197,198,199,200,201,202,210,211,212,213,214,215,216,217,218,225,226,227,228,229,230,231,232,233,234,241,242,243,244,245,246,247,248,249,250]},huff_ac_luma_pad:[0;2],huff_ac_chroma:JpegHuffAc{data:[0,2,1,2,4,4,3,4,7,5,4,4,0,1,2,119,0,1,2,3,17,4,5,33,49,6,18,65,81,7,97,113,19,34,50,129,8,20,66,145,161,177,193,9,35,51,82,240,21,98,114,209,10,22,36,52,225,37,241,23,24,25,26,38,39,40,41,42,53,54,55,56,57,58,67,68,69,70,71,72,73,74,83,84,85,86,87,88,89,90,99,100,101,102,103,104,105,106,115,116,117,118,119,120,121,122,130,131,132,133,134,135,136,137,138,146,147,148,149,150,151,152,153,154,162,163,164,165,166,167,168,169,170,178,179,180,181,182,183,184,185,186,194,195,196,197,198,199,200,201,202,210,211,212,213,214,215,216,217,218,226,227,228,229,230,231,232,233,234,242,243,244,245,246,247,248,249,250]}};
pub static mut hsd_804D2648:JpegWork=unsafe{core::mem::zeroed()};
pub static mut hsd_804D79A0:*mut u8=core::ptr::null_mut();
pub static mut hsd_804D79A4:*mut u8=core::ptr::null_mut();
pub static mut hsd_804D79A8:i32=0;
pub static mut hsd_804D79AC:i32=0;
pub static mut hsd_804D79B0:[u8;8]=[0;8];
pub static mut lbl_804D6398:i32=3;
const JPEG_EXHAUSTED:u64=0x4a5045475f454f46;
/// Replacement for the TU-local setjmp/longjmp boundary. Preserve all partial state.
pub fn jpeg_fail()->! { std::panic::resume_unwind(Box::new(JPEG_EXHAUSTED)) }
pub fn jpeg_try(f:impl FnOnce()->i32)->Option<i32> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v)=>Some(v),Err(e)=>if e.downcast_ref::<u64>()==Some(&JPEG_EXHAUSTED){None}else{std::panic::resume_unwind(e)},
    }
}
pub mod ext {
    pub static mut HSD_803B3344:Option<unsafe fn(u8)>=None;
    pub static mut HSD_803B3398:Option<unsafe fn(*mut core::ffi::c_void,usize)>=None;
    pub unsafe fn hsd_803B3344(v:u8) { (HSD_803B3344.expect("TODO hsd_3B33.c: hsd_803B3344"))(v) }
    pub unsafe fn hsd_803B3398(p:*mut core::ffi::c_void,n:usize) { (HSD_803B3398.expect("TODO hsd_3B33.c: hsd_803B3398"))(p,n) }
}
/// Deterministic native boundary implementation available for oracle hooks.
pub unsafe fn write_byte(byte:u8) {
    if hsd_804D79A0 < hsd_804D79A4.wrapping_offset(hsd_804D79A8 as isize) {
        *hsd_804D79A0=byte; hsd_804D79A0=hsd_804D79A0.add(1);
    } else {jpeg_fail()}
}
pub unsafe fn write_bulk(src:*mut core::ffi::c_void,n:usize) {
    if hsd_804D79A0 < hsd_804D79A4.wrapping_offset(hsd_804D79A8 as isize).wrapping_sub(n) {
        core::ptr::copy_nonoverlapping(src.cast::<u8>(),hsd_804D79A0,n);hsd_804D79A0=hsd_804D79A0.add(n);
    } else {jpeg_fail()}
}
pub unsafe fn jpegLumaAddress(dest:*mut *mut i32,work:*mut u8,offset:u32) {
    *dest=core::ptr::addr_of_mut!((*(work.cast::<JpegWork>())).x118).cast::<i32>().add((offset/4) as usize);
}
pub unsafe fn hsd_803B3408(image:*mut u8,x:i32,y:i32,width:i32,_height:i32) {
    let stride=((width+15)/16)*64;
    let image_offset=(x/4)*16+(y/4)*stride;
    let work=core::ptr::addr_of_mut!(hsd_804D2648);
    for tile_y in 0..2 {let mut src=image.cast::<u16>().offset((image_offset+tile_y*stride) as isize);
        for tile_x in 0..2 {
            let tile_offset=tile_y*16+tile_x*2;
            for cy in 0..4 {let dst_row=tile_offset+(cy&1)*4+(cy&2)*16;let src_row=(cy&1)*32+(cy&2)*stride;
                for cx in 0..4 {let ci=(cx&1)+(cx&2)*4;let p=*src.offset(((cx&1)*2+(cx&2)*4+src_row) as isize) as u32;
                    let r=((p>>8)&248) as f32;let g=((p>>3)&252) as f32;let b=((p*8)&248) as f32;
                    (*work).x518[(ci+dst_row) as usize]=fmadds(0.5,b,fmadds(-0.1687,r,-(0.3313*g))) as i32;
                    (*work).x618[(ci+dst_row) as usize]=fmadds(-0.0813,b,fmadds(0.5,r,-(0.4187*g))) as i32;
                }
            }
            for ly in 0..4 {for lx in 0..4 {
                let mut luma=core::ptr::null_mut();jpegLumaAddress(&mut luma,work.cast(),((tile_x*4+(tile_y*4+ly)*8+lx)*4) as u32);
                for pi in 0..4 {let p=*src.offset(((pi&1)*32+(pi&2)*stride) as isize) as u32;
                    let r=((p>>8)&248) as f32;let g=((p>>3)&252) as f32;let b=((p*8)&248) as f32;
                    *luma.add(pi as usize*64)=(fmadds(0.114,b,fmadds(0.299,r,0.587*g)) as i32).wrapping_sub(128);
                }src=src.add(1);
            }}
        }
    }
}
/// Each row and column uses exactly the f64 fmuladd sites in fn_803B376C.
pub unsafe fn fn_803B376C(arg:*mut u8) {
    let b=arg.cast::<i32>();
    for pass in 0..2 {for i in 0..8 {let stride=if pass==0{1}else{8};let p=b.add(if pass==0{i*8}else{i});
        let a=core::array::from_fn::<i32,8,_>(|j|*p.add(j*stride));
        let s0=a[0].wrapping_add(a[7]) as f32;let s1=a[1].wrapping_add(a[6]) as f32;
        let s2=a[2].wrapping_add(a[5]) as f32;let s3=a[3].wrapping_add(a[4]) as f32;
        let sum12=s1+s2;let sum03=s0+s3;let diff03=s0-s3;let diff12=s1-s2;
        let d3=a[3].wrapping_sub(a[4]) as f32;let d2=a[2].wrapping_sub(a[5]) as f32;
        let d1=a[1].wrapping_sub(a[6]) as f32;let d0=a[0].wrapping_sub(a[7]) as f32;
        *p=(0.707107f64*(sum03+sum12) as f64) as i32;
        *p.add(4*stride)=(0.707107f64*(sum03-sum12) as f64) as i32;
        *p.add(6*stride)=(-0.92388f64).mul_add(diff12 as f64,0.382683f64*diff03 as f64) as i32;
        *p.add(2*stride)=0.382683f64.mul_add(diff12 as f64,0.92388f64*diff03 as f64) as i32;
        let rot_diff=(0.707107f64*(d2-d1) as f64) as f32;let rot_sum=(0.707107f64*(d2+d1) as f64) as f32;
        let a=rot_sum+d0;let b=-d3+rot_diff;let c=-rot_sum+d0;let d=d3+rot_diff;
        *p.add(7*stride)=0.980785f64.mul_add(b as f64,0.19509f64*a as f64) as i32;
        *p.add(5*stride)=0.83147f64.mul_add(d as f64,0.55557f64*c as f64) as i32;
        *p.add(3*stride)=(-0.55557f64).mul_add(d as f64,0.83147f64*c as f64) as i32;
        *p.add(stride)=(-0.19509f64).mul_add(b as f64,0.980785f64*a as f64) as i32;
    }}
    for i in 0..64 {*b.add(i)>>=2;}
}
pub fn hsd_803B3CD8_bit_length(value:i32)->i32 {32-(value as u32).leading_zeros() as i32}
pub fn jpeg_run_bit_length(value:i32)->i32 {hsd_803B3CD8_bit_length(value.wrapping_add(1))}
pub unsafe fn hsd_803B3CD8_write_byte(byte:u8,_work:*mut *mut JpegWork) {write_byte(byte)}
pub unsafe fn hsd_803B3CD8_write_bits(value:i32,length:i32,work:*mut *mut JpegWork) {
    for bit in (0..length).rev() {
        hsd_804D79B0[0]=hsd_804D79B0[0].wrapping_shl(1);hsd_804D79AC=hsd_804D79AC.wrapping_add(1);
        if value as u32 & shl(1,bit as u32)!=0 {hsd_804D79B0[0]|=1;}
        if hsd_804D79AC==8 {hsd_803B3CD8_write_byte(hsd_804D79B0[0],work);if hsd_804D79B0[0]==255 {hsd_803B3CD8_write_byte(0,work);}hsd_804D79AC=0;hsd_804D79B0[0]=0;}
    }
}
pub unsafe fn jpeg_write_run_payload(run:i32,length:i32,work:*mut *mut JpegWork) {hsd_803B3CD8_write_bits(run.wrapping_add(1),length,work)}
pub unsafe fn jpeg_encode_component(component:i32,ac_value_out:*mut i32,run_out:*mut i32,value_out:*mut i32,index_out:*mut i32) {
    let mut work=core::ptr::addr_of_mut!(hsd_804D2648);let c=component as usize;
    let dc_code=if c==0{&lbl_80431678}else{&lbl_8043169C};let dc_length=if c==0{&lbl_80431690}else{&lbl_804316B4};
    let ac_code=if c==0{&TABLES.ac_code_luma}else{&TABLES.ac_code_chroma};let ac_length=if c==0{&TABLES.ac_length_luma}else{&TABLES.ac_length_chroma};
    *value_out=(*work).coef[0].wrapping_sub((*work).prev_dc[c]);*run_out=0;
    let len=hsd_803B3CD8_bit_length((*value_out).wrapping_abs());(*work).prev_dc[c]=(*work).coef[0];
    hsd_803B3CD8_write_bits(dc_code[len as usize] as i32,dc_length[len as usize] as i32,&mut work);
    if len!=0 {if *value_out<0 {*value_out=(*value_out).wrapping_sub(1);}hsd_803B3CD8_write_bits(*value_out,len,&mut work);}
    *index_out=1;while *index_out<64 {
        let coefficient=(*work).coef[lbl_80431638[*index_out as usize] as usize];*ac_value_out=coefficient;
        if coefficient!=0 {
            let len=jpeg_run_bit_length(*run_out);hsd_803B3CD8_write_bits(ac_code[len as usize] as i32,ac_length[len as usize] as i32,&mut work);jpeg_write_run_payload(*run_out,len,&mut work);
            let len=hsd_803B3CD8_bit_length((*ac_value_out).wrapping_abs());hsd_803B3CD8_write_bits(ac_code[len as usize] as i32,ac_length[len as usize] as i32,&mut work);
            if *ac_value_out<0 {*ac_value_out=(*ac_value_out).wrapping_sub(1);}hsd_803B3CD8_write_bits(*ac_value_out,len,&mut work);*run_out=0;
        }else{*run_out+=1;}*index_out+=1;
    }
    if *run_out!=0 {hsd_803B3CD8_write_bits(ac_code[0] as i32,ac_length[0] as i32,&mut work);}
}
pub unsafe fn hsd_803B3CD8(component:i32) {let(mut a,mut r,mut v,mut i)=(0,0,0,0);jpeg_encode_component(component,&mut a,&mut r,&mut v,&mut i);}
pub unsafe fn hsd_803B46D4() {
    for b in [255,224,0,16] {write_byte(b);}
    write_bulk(b"JFIF\0".as_ptr() as *mut _,5);
    for b in [1,2,1,0,72,0,72,0,0] {write_byte(b);}
}
pub unsafe fn hsd_803B4A2C() {
    for c in 0..2 {for b in [255,219,0,67,c] {write_byte(b);}let q=if c==0{&TABLES.quant_luma}else{&TABLES.quant_chroma};
        for i in 0..64 {write_byte(sdiv(q[lbl_80431638[i] as usize] as i32,lbl_804D6398) as u8);}
    }
}
pub unsafe fn hsd_803B4D64(width:u32,height:u32) {
    for b in [255,192,0,17,8,(height>>8) as u8,height as u8,(width>>8) as u8,width as u8,3,0,34,0,1,17,1,2,17,1] {write_byte(b);}
}
pub unsafe fn hsd_803B51C8_inline(image:*mut u8,image_height:i32,image_width:i32,output:*const i8,output_capacity:i32,metadata:*const JpegMetadata,comment:*mut JpegComment,huff_dc_luma:*mut JpegHuffDc,huff_dc_chroma:*mut JpegHuffDc,huff_ac_luma:*mut JpegHuffAc,huff_ac_chroma:*mut JpegHuffAc)->i32 {
    hsd_804D79A8=output_capacity;hsd_804D79AC=0;hsd_804D79A0=output as *mut u8;hsd_804D79A4=output as *mut u8;hsd_804D2648.prev_dc=[0;3];
    jpeg_try(||{
        write_byte(255);write_byte(216);hsd_803B46D4();
        *comment=(*metadata).comment;let n=(*comment).data.iter().position(|&b|b==0).unwrap()+1;
        for b in [255,254,((n+2)>>8) as u8,(n+2) as u8] {ext::hsd_803B3344(b);}ext::hsd_803B3398(comment.cast(),n);
        hsd_803B4A2C();
        for stage in 0..4 {
            let (p,n,c)=match stage {
                0=>{*huff_dc_luma=(*metadata).huff_dc_luma;(huff_dc_luma.cast(),28,0)},
                1=>{*huff_dc_chroma=(*metadata).huff_dc_chroma;(huff_dc_chroma.cast(),28,1)},
                2=>{*huff_ac_luma=(*metadata).huff_ac_luma;(huff_ac_luma.cast(),178,16)},
                _=>{*huff_ac_chroma=(*metadata).huff_ac_chroma;(huff_ac_chroma.cast(),178,17)},
            };
            for b in [255,196,0,(n+3) as u8,c] {ext::hsd_803B3344(b);}ext::hsd_803B3398(p,n);
        }
        hsd_803B4D64(image_width as u32,image_height as u32);
        for b in [255,218,0,12,3,0,0,1,17,2,17,0,63,0] {write_byte(b);}
        let work=core::ptr::addr_of_mut!(hsd_804D2648);
        for y in (0..image_height).step_by(16) {for x in (0..image_width).step_by(16) {
            hsd_803B3408(image,x,y,image_width,image_height);
            for block in 0..6 {
                let (src,quant,c)=if block<4 {(core::ptr::addr_of_mut!((*work).x118).cast::<i32>().add(block*64),&TABLES.quant_luma,0)}else if block==4 {(core::ptr::addr_of_mut!((*work).x518).cast(),&TABLES.quant_chroma,1)}else{(core::ptr::addr_of_mut!((*work).x618).cast(),&TABLES.quant_chroma,2)};
                fn_803B376C(src.cast());for i in 0..64 {(*work).coef[i]=sdiv(*src.add(i),sdiv(quant[i] as i32,lbl_804D6398));}hsd_803B3CD8(c);
            }
        }}
        if hsd_804D79AC!=0 {let bits=8-hsd_804D79AC;hsd_804D79B0[0]=shl(hsd_804D79B0[0] as u32,bits as u32) as u8;hsd_804D79B0[0]|=shl(1,bits as u32).wrapping_sub(1) as u8;write_byte(hsd_804D79B0[0]);hsd_804D79B0[0]=0;hsd_804D79AC=0;}
        write_byte(255);write_byte(217);hsd_804D79A0.offset_from(hsd_804D79A4) as i32
    }).unwrap_or(0)
}
/// The C integer image address becomes a native pointer (no 64-bit truncation).
pub unsafe fn hsd_803B51C8(image:*mut u8,width:i32,height:i32,out:*mut i8,capacity:i32)->i32 {
    hsd_803B51C8_inline(image,height,width,out,capacity,&lbl_803B9670,&mut JpegComment::zeroed(),&mut JpegHuffDc::zeroed(),&mut JpegHuffDc::zeroed(),&mut JpegHuffAc::zeroed(),&mut JpegHuffAc::zeroed())
}
pub unsafe fn hsd_803B5C2C(scale:i32) {lbl_804D6398=if (1..=10).contains(&scale){scale}else{3};}
