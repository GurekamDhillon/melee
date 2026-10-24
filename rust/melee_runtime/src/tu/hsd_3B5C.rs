//! Native JPEG decoder; JPEG bytes remain serialized big-endian, pixels/state are native.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]
use melee_types::{JpegState,JpegOutput};
use melee_core::fp::fmadds;
use crate::cabi::shl;
use super::hsd_3B34::{jpeg_fail,jpeg_try};
pub static lbl_80431638:[u8;64]=[0,1,8,16,9,2,3,10,17,24,32,25,18,11,4,5,12,19,26,33,40,48,41,34,27,20,13,6,7,14,21,28,35,42,49,56,57,50,43,36,29,22,15,23,30,37,44,51,58,59,52,45,38,31,39,46,53,60,61,54,47,55,62,63];
pub static lbl_80431678:[u16;12]=[0,2,3,4,5,6,14,30,62,126,254,510];
pub static lbl_80431690:[u8;12]=[2,3,3,3,3,3,4,5,6,7,8,9];
pub static lbl_8043169C:[u16;12]=[0,1,2,6,14,30,62,126,254,510,1022,2046];
pub static lbl_804316B4:[u8;12]=[2,2,2,3,4,5,6,7,8,9,10,11];
pub static lbl_80431090:[u8;1448]=[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0,1,2,3,4,5,6,7,8,9,10,11,0,0,0,1,0,4,0,10,0,11,0,12,0,26,0,27,0,28,0,58,0,59,0,120,0,121,0,122,0,123,0,248,0,249,0,250,1,246,1,247,1,248,1,249,1,250,3,246,3,247,3,248,3,249,3,250,7,246,7,247,7,248,7,249,15,244,15,245,15,246,15,247,127,192,255,130,255,131,255,132,255,133,255,134,255,135,255,136,255,137,255,138,255,139,255,140,255,141,255,142,255,143,255,144,255,145,255,146,255,147,255,148,255,149,255,150,255,151,255,152,255,153,255,154,255,155,255,156,255,157,255,158,255,159,255,160,255,161,255,162,255,163,255,164,255,165,255,166,255,167,255,168,255,169,255,170,255,171,255,172,255,173,255,174,255,175,255,176,255,177,255,178,255,179,255,180,255,181,255,182,255,183,255,184,255,185,255,186,255,187,255,188,255,189,255,190,255,191,255,192,255,193,255,194,255,195,255,196,255,197,255,198,255,199,255,200,255,201,255,202,255,203,255,204,255,205,255,206,255,207,255,208,255,209,255,210,255,211,255,212,255,213,255,214,255,215,255,216,255,217,255,218,255,219,255,220,255,221,255,222,255,223,255,224,255,225,255,226,255,227,255,228,255,229,255,230,255,231,255,232,255,233,255,234,255,235,255,236,255,237,255,238,255,239,255,240,255,241,255,242,255,243,255,244,255,245,255,246,255,247,255,248,255,249,255,250,255,251,255,252,255,253,255,254,2,2,3,4,4,4,5,5,5,6,6,7,7,7,7,8,8,8,9,9,9,9,9,10,10,10,10,10,11,11,11,11,12,12,12,12,15,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,0,0,1,2,3,0,4,17,5,18,33,49,65,6,19,81,97,7,34,113,20,50,129,145,161,8,35,66,177,193,21,82,209,240,36,51,98,114,130,9,10,22,23,24,25,26,37,38,39,40,41,42,52,53,54,55,56,57,58,67,68,69,70,71,72,73,74,83,84,85,86,87,88,89,90,99,100,101,102,103,104,105,106,115,116,117,118,119,120,121,122,131,132,133,134,135,136,137,138,146,147,148,149,150,151,152,153,154,162,163,164,165,166,167,168,169,170,178,179,180,181,182,183,184,185,186,194,195,196,197,198,199,200,201,202,210,211,212,213,214,215,216,217,218,225,226,227,228,229,230,231,232,233,234,241,242,243,244,245,246,247,248,249,250,0,0,2,2,3,4,4,5,5,5,5,6,6,6,6,7,7,7,8,8,8,8,9,9,9,9,9,9,9,10,10,10,10,10,11,11,11,11,12,12,12,12,14,15,15,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,16,0,0,0,0,0,1,0,4,0,10,0,11,0,24,0,25,0,26,0,27,0,56,0,57,0,58,0,59,0,120,0,121,0,122,0,246,0,247,0,248,0,249,1,244,1,245,1,246,1,247,1,248,1,249,1,250,3,246,3,247,3,248,3,249,3,250,7,246,7,247,7,248,7,249,15,244,15,245,15,246,15,247,63,224,127,194,127,195,255,136,255,137,255,138,255,139,255,140,255,141,255,142,255,143,255,144,255,145,255,146,255,147,255,148,255,149,255,150,255,151,255,152,255,153,255,154,255,155,255,156,255,157,255,158,255,159,255,160,255,161,255,162,255,163,255,164,255,165,255,166,255,167,255,168,255,169,255,170,255,171,255,172,255,173,255,174,255,175,255,176,255,177,255,178,255,179,255,180,255,181,255,182,255,183,255,184,255,185,255,186,255,187,255,188,255,189,255,190,255,191,255,192,255,193,255,194,255,195,255,196,255,197,255,198,255,199,255,200,255,201,255,202,255,203,255,204,255,205,255,206,255,207,255,208,255,209,255,210,255,211,255,212,255,213,255,214,255,215,255,216,255,217,255,218,255,219,255,220,255,221,255,222,255,223,255,224,255,225,255,226,255,227,255,228,255,229,255,230,255,231,255,232,255,233,255,234,255,235,255,236,255,237,255,238,255,239,255,240,255,241,255,242,255,243,255,244,255,245,255,246,255,247,255,248,255,249,255,250,255,251,255,252,255,253,255,254,0,1,2,3,17,4,5,33,49,6,18,65,81,7,97,113,19,34,50,129,8,20,66,145,161,177,193,9,35,51,82,240,21,98,114,209,10,22,36,52,225,37,241,23,24,25,26,38,39,40,41,42,53,54,55,56,57,58,67,68,69,70,71,72,73,74,83,84,85,86,87,88,89,90,99,100,101,102,103,104,105,106,115,116,117,118,119,120,121,122,130,131,132,133,134,135,136,137,138,146,147,148,149,150,151,152,153,154,162,163,164,165,166,167,168,169,170,178,179,180,181,182,183,184,185,186,194,195,196,197,198,199,200,201,202,210,211,212,213,214,215,216,217,218,226,227,228,229,230,231,232,233,234,242,243,244,245,246,247,248,249,250,0,0,0,0,0,0];
pub static mut hsd_804D2E70:JpegState=unsafe{core::mem::zeroed()};
pub static mut QUANT:melee_types::JpegQuantTables=melee_types::JpegQuantTables{luma:[1;64],chroma:[1;64]};
pub static mut hsd_804D79B8:*mut u8=core::ptr::null_mut();
pub static mut hsd_804D79BC:*mut u8=core::ptr::null_mut();
pub static mut hsd_804D79C0:i32=0;
pub static mut hsd_804D79C4:i32=0;
pub static mut hsd_804D79C8:u8=0;
pub unsafe fn hsd_803B5C4C_read(mut bits:i32,mut bit_count:i32)->i32 {
    loop {
        if hsd_804D79C4==0 {
            hsd_804D79C4=8;
            if hsd_804D79B8>=hsd_804D79BC.wrapping_offset(hsd_804D79C0 as isize){jpeg_fail();}
            hsd_804D79C8=*hsd_804D79B8;hsd_804D79B8=hsd_804D79B8.add(1);
            if hsd_804D79C8==255 {
                // Source checks the stuffed byte before checking its end pointer.
                if *hsd_804D79B8!=0 {jpeg_fail();}
                if hsd_804D79B8>=hsd_804D79BC.wrapping_offset(hsd_804D79C0 as isize){jpeg_fail();}
                hsd_804D79B8=hsd_804D79B8.add(1);
            }
        }
        bits=bits.wrapping_mul(2);if hsd_804D79C8 as u32 & shl(1,(hsd_804D79C4-1) as u32)!=0 {bits|=1;}
        bit_count=bit_count.wrapping_sub(1);hsd_804D79C4-=1;if bit_count==0{return bits;}
    }
}
pub unsafe fn hsd_803B5C4C(bit_count:i32)->i32 {hsd_803B5C4C_read(0,bit_count)}
fn table_word(offset:usize,index:usize)->u16 {u16::from_be_bytes([lbl_80431090[offset+index*2],lbl_80431090[offset+index*2+1]])}
pub unsafe fn hsd_803B5D70(ac:i32,component:i32)->i32 {
    let mut index=0;let mut code=0i32;
    for bit_len in 1..=16 {
        code=code.wrapping_mul(2)|hsd_803B5C4C(1);
        loop {
            let (length,word,value)=if ac==0 {
                let words=if component==0{&lbl_80431678}else{&lbl_8043169C};let lengths=if component==0{&lbl_80431690}else{&lbl_804316B4};
                // C walks into subsequent storage for invalid codes; supported streams use these entries.
                if index>=12{return 0;}(lengths[index],words[index],lbl_80431090[0x80+index])
            }else {
                if index>=162{return 0;}
                let(co,lo,vo)=if component==0{(0x8c,0x1d0,0x274)}else{(0x3bc,0x318,0x500)};
                (lbl_80431090[lo+index],table_word(co,index),lbl_80431090[vo+index])
            };
            if bit_len!=length as i32 {break;}
            if code==word as i32 {return value as i32;}index+=1;
        }
    }0
}
pub unsafe fn hsd_803B5EA0(component:i32) {
    let work=core::ptr::addr_of_mut!(hsd_804D2E70.work);
    let value_bits=hsd_803B5D70(0,component);
    let dc=if value_bits>0 {let mut v=hsd_803B5C4C(value_bits);if v as u32 & shl(1,(value_bits-1) as u32)==0 {v=v.wrapping_sub(shl(1,value_bits as u32).wrapping_sub(1) as i32);}v}else{0};
    (*work).prev_dc[component as usize]=(*work).prev_dc[component as usize].wrapping_add(dc);(*work).coeff[0]=(*work).prev_dc[component as usize];
    let mut coefficient=1;
    while coefficient<64 {
        let run_bits=hsd_803B5D70(1,component);
        if run_bits==0 {while coefficient<64 {(*work).coeff[lbl_80431638[coefficient] as usize]=0;coefficient+=1;}return;}
        let zeros=hsd_803B5C4C(run_bits).wrapping_sub(1) as u32;
        for _ in 0..zeros {if coefficient>=64{jpeg_fail();}(*work).coeff[lbl_80431638[coefficient] as usize]=0;coefficient+=1;}
        let value_bits=hsd_803B5D70(1,component);let mut ac=hsd_803B5C4C(value_bits);
        if ac as u32 & shl(1,(value_bits-1) as u32)==0 {ac=ac.wrapping_sub(shl(1,value_bits as u32).wrapping_sub(1) as i32);}
        if coefficient>=64{jpeg_fail();}(*work).coeff[lbl_80431638[coefficient] as usize]=ac;coefficient+=1;
    }
}
pub unsafe fn fn_803B61B4(block:*mut i32) {
    for pass in 0..2 {for i in 0..8 {
        let stride=if pass==0{1}else{8};let p=block.add(if pass==0{i*8}else{i});let a=core::array::from_fn::<i32,8,_>(|j|*p.add(j*stride));
        let odd0=0.980785f64.mul_add(a[7] as f64,-(0.19509f64*a[1] as f64)) as f32;
        let odd1=0.83147f64.mul_add(a[5] as f64,-(0.55557f64*a[3] as f64)) as f32;
        let odd2=0.55557f64.mul_add(a[5] as f64,0.83147f64*a[3] as f64) as f32;
        let odd3=0.19509f64.mul_add(a[7] as f64,0.980785f64*a[1] as f64) as f32;
        let even_sum=(0.707107f64*a[0].wrapping_add(a[4]) as f64) as f32;
        let even_diff=(0.707107f64*a[0].wrapping_sub(a[4]) as f64) as f32;
        let odd_pair_sum=odd0+odd1;let odd_pair_diff=-odd2+odd3;let odd_sum=odd2+odd3;
        let even_rotation_diff=(-0.92388f64).mul_add(a[6] as f64,0.382683f64*a[2] as f64) as f32;
        let even_rotation_sum=0.382683f64.mul_add(a[6] as f64,0.92388f64*a[2] as f64) as f32;
        let odd_rotation_diff=(0.707107f64*(-odd_pair_sum+odd_pair_diff) as f64) as f32;
        let even0=even_sum+even_rotation_sum;let odd_diff=-odd0+odd1;
        let odd_rotation_sum=(0.707107f64*(odd_pair_sum+odd_pair_diff) as f64) as f32;
        let even1=even_diff+even_rotation_diff;let even2=even_diff-even_rotation_diff;let even3=even_sum-even_rotation_sum;
        let result=[even0+odd_sum,even1+odd_rotation_diff,even2+odd_rotation_sum,even3+odd_diff,even3-odd_diff,even2-odd_rotation_sum,even1-odd_rotation_diff,even0-odd_sum];
        for j in 0..8 {*p.add(j*stride)=result[j] as i32;}
    }}
    for i in 0..64 {*block.add(i)>>=2;}
}
pub fn jpeg_clamp(value:f32)->i32 {if value<0.0{0}else if 255.0<value{255}else{(value as i32 as u8) as i32}}
pub unsafe fn jpeg_store_rgb565(out:*mut JpegOutput,offset:i32,luminance:i32,cb:i32,cr:i32) {
    let red=jpeg_clamp(1.402f64.mul_add(cr as f64,luminance as f64) as f32) as u8;
    let green=jpeg_clamp(fmadds(-0.7139,cr as f32,fmadds(-0.3441,cb as f32,luminance as f32))) as u8;
    let blue=jpeg_clamp((-0.0012f64).mul_add(cr as f64,fmadds(1.7718,cb as f32,luminance as f32) as f64) as f32) as u8;
    *(*out).pixels.offset(offset as isize)=((red as u16*256)&0xf800)|((green as u16*8)&0x7e0)|(blue as u16>>3);
}
pub unsafe fn fn_803B6820(dst:*mut u8,x:i32,y:i32,width:i32,_unused_height:i32) {
    let work=core::ptr::addr_of_mut!(hsd_804D2E70.work);
    for i in 0..256 {(*work).luma[i]=(*work).luma[i].wrapping_add(128);}
    let aligned_width=((width+15)/16)*16;
    for gy in 0..2 {
        let mut out=dst.cast::<u16>().offset(((x/4)*16+(aligned_width*4)*((y/4)+gy)) as isize);
        for gx in 0..2 {
            let chroma_base=gy*16+gx*2;let luma_x=gx*4;
            for ty in 0..4 {let row_offset=(gy*4+ty)*8;
                for tx in 0..4 {let chroma_row=(tx>>1)+chroma_base+((ty&2)*4);
                    for block in 0..4 {
                        let lum=(*work).luma[(luma_x+row_offset+tx+block*64) as usize];
                        let c=(chroma_row+(block%2)*4+(block/2)*32) as usize;
                        let offset=((block&1)*32)+aligned_width*((block&2)*4);let mut pixel_out=JpegOutput{pixels:out};
                        jpeg_store_rgb565(&mut pixel_out,offset,lum,(*work).cb[c],(*work).cr[c]);
                    }out=out.add(1);
                }
            }
        }
    }
}
unsafe fn marker(wanted:u16) {
    let end=hsd_804D79BC.wrapping_offset(hsd_804D79C0 as isize);
    loop {let value=u16::from_be_bytes([*hsd_804D79B8,*hsd_804D79B8.add(1)]);if value==wanted{return;}
        hsd_804D79B8=hsd_804D79B8.add(1);if hsd_804D79B8>=end {jpeg_fail();}
    }
}
pub unsafe fn hsd_803B6BE4_inline(src:*mut i8,size:i32,dst:*mut core::ffi::c_void)->i32 {
    hsd_804D79C0=size;hsd_804D79B8=src.cast();hsd_804D79BC=src.cast();hsd_804D2E70.work.prev_dc=[0;3];hsd_804D79C4=0;
    jpeg_try(||{
        for c in 0..2 {marker(0xffdb);hsd_804D79B8=hsd_804D79B8.add(5);
            for i in 0..64 {let v=*hsd_804D79B8;hsd_804D79B8=hsd_804D79B8.add(1);if c==0{QUANT.luma[lbl_80431638[i] as usize]=v;}else{QUANT.chroma[lbl_80431638[i] as usize]=v;}}
        }
        marker(0xffc0);hsd_804D79B8=hsd_804D79B8.add(5);
        let height=u16::from_be_bytes([*hsd_804D79B8,*hsd_804D79B8.add(1)]) as i32;hsd_804D79B8=hsd_804D79B8.add(2);
        let width=u16::from_be_bytes([*hsd_804D79B8,*hsd_804D79B8.add(1)]) as i32;hsd_804D79B8=hsd_804D79B8.add(12);
        marker(0xffda);hsd_804D79B8=hsd_804D79B8.add(14);
        let work=core::ptr::addr_of_mut!(hsd_804D2E70.work);
        for y in (0..height).step_by(16) {for x in (0..width).step_by(16) {
            for block in 0..6 {
                let (out,q,c)=if block<4 {(core::ptr::addr_of_mut!((*work).luma).cast::<i32>().add(block*64),core::ptr::addr_of!(QUANT.luma),0)}else if block==4{(core::ptr::addr_of_mut!((*work).cb).cast(),core::ptr::addr_of!(QUANT.chroma),1)}else{(core::ptr::addr_of_mut!((*work).cr).cast(),core::ptr::addr_of!(QUANT.chroma),2)};
                hsd_803B5EA0(c);for i in 0..64 {*out.add(i)=(*work).coeff[i].wrapping_mul((*q)[i] as i32);}fn_803B61B4(out);
            }
            fn_803B6820(dst.cast(),x,y,width,height);
        }}width.wrapping_mul(height).wrapping_mul(2)
    }).unwrap_or(0)
}
pub unsafe fn hsd_803B6BE4(src:*mut i8,size:i32,dst:*mut core::ffi::c_void)->i32 {hsd_803B6BE4_inline(src,size,dst)}
