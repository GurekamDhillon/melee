//! Native-layout port of `src/sysdolphin/baselib/video.c`.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs)]
use core::ffi::c_void;
use melee_types::*;
const GX_TRUE: u8 = 1; // GXEnum.h macro
pub static mut HSD_VIData: HSD_VIInfo = unsafe { core::mem::zeroed() };
pub static mut garbage: [u8; 5120] = [0;5120];
pub mod ext { use super::*;
pub static mut OSDisableInterrupts: unsafe fn()->i32 = || todo!("OSDisableInterrupts");
pub static mut OSRestoreInterrupts: unsafe fn(i32)->i32 = |_| todo!("OSRestoreInterrupts");
pub static mut VIWaitForRetrace: unsafe fn()->() = || todo!("VIWaitForRetrace");
pub static mut VISetNextFrameBuffer: unsafe fn(*mut c_void)->() = |_| todo!("VISetNextFrameBuffer");
pub static mut VIConfigure: unsafe fn(*mut GXRenderModeObj)->() = |_| todo!("VIConfigure");
pub static mut VISetBlack: unsafe fn(i32)->() = |_| todo!("VISetBlack");
pub static mut VIFlush: unsafe fn()->() = || todo!("VIFlush");
pub static mut VIInit: unsafe fn()->() = || todo!("VIInit");
pub static mut VIGetTvFormat: unsafe fn()->u32 = || todo!("VIGetTvFormat");
pub static mut VISetPreRetraceCallback: unsafe fn(HSD_VIRetraceCallback)->HSD_VIRetraceCallback = |_| todo!("VISetPreRetraceCallback");
pub static mut VISetPostRetraceCallback: unsafe fn(HSD_VIRetraceCallback)->HSD_VIRetraceCallback = |_| todo!("VISetPostRetraceCallback");
pub static mut GXSetDrawDoneCallback: unsafe fn(GXDrawDoneCallback)->GXDrawDoneCallback = |_| todo!("GXSetDrawDoneCallback");
pub static mut GXSetDispCopySrc: unsafe fn(u16, u16, u16, u16)->() = |_, _, _, _| todo!("GXSetDispCopySrc");
pub static mut GXSetDispCopyDst: unsafe fn(u16, u16)->() = |_, _| todo!("GXSetDispCopyDst");
pub static mut GXSetDispCopyYScale: unsafe fn(f32)->u32 = |_| todo!("GXSetDispCopyYScale");
pub static mut GXSetCopyFilter: unsafe fn(u8, *const [u8;2], u8, *const u8)->() = |_, _, _, _| todo!("GXSetCopyFilter");
pub static mut GXSetDispCopyGamma: unsafe fn(GXGamma)->() = |_| todo!("GXSetDispCopyGamma");
pub static mut HSD_StateSetColorUpdate: unsafe fn(i32)->() = |_| todo!("HSD_StateSetColorUpdate");
pub static mut HSD_StateSetAlphaUpdate: unsafe fn(i32)->() = |_| todo!("HSD_StateSetAlphaUpdate");
pub static mut HSD_StateSetZMode: unsafe fn(i32, i32, i32)->() = |_, _, _| todo!("HSD_StateSetZMode");
pub static mut GXSetCopyClear: unsafe fn(GXColor, u32)->() = |_, _| todo!("GXSetCopyClear");
pub static mut GXSetCopyClamp: unsafe fn(GXFBClamp)->() = |_| todo!("GXSetCopyClamp");
pub static mut GXCopyDisp: unsafe fn(*mut c_void, u8)->() = |_, _| todo!("GXCopyDisp");
pub static mut GXPixModeSync: unsafe fn()->() = || todo!("GXPixModeSync");
pub static mut GXSetDrawDone: unsafe fn()->() = || todo!("GXSetDrawDone");
pub static mut GXWaitDrawDone: unsafe fn()->() = || todo!("GXWaitDrawDone");
}

static mut vr_count: i32 = 0;
static mut renew_count: i32 = 0;

pub unsafe fn HSD_VISearchXFBByStatus(status: HSD_VIXFBDrawDispStatus) -> i32 {
    for i in 0..3 { if HSD_VIData.xfb[i].status == status { return i as i32; } }
    -1
}
pub unsafe fn HSD_VISetUserPreRetraceCallback(cb:HSD_VIRetraceCallback)->HSD_VIRetraceCallback {
    let old=HSD_VIData.pre_cb; let intr=(ext::OSDisableInterrupts)();
    HSD_VIData.pre_cb=cb; (ext::OSRestoreInterrupts)(intr); old
}
pub unsafe fn HSD_VISetUserPostRetraceCallback(cb:HSD_VIRetraceCallback)->HSD_VIRetraceCallback {
    let old=HSD_VIData.post_cb; let intr=(ext::OSDisableInterrupts)();
    HSD_VIData.post_cb=cb; (ext::OSRestoreInterrupts)(intr); old
}
pub unsafe fn HSD_VISetUserGXDrawDoneCallback(cb:HSD_VIGXDrawDoneCallback)->HSD_VIGXDrawDoneCallback {
    let old=HSD_VIData.drawdone.cb; let intr=(ext::OSDisableInterrupts)();
    HSD_VIData.drawdone.cb=cb; (ext::OSRestoreInterrupts)(intr); old
}
pub unsafe extern "C" fn HSD_VIPreRetraceCB(retraceCount:u32) {
    let mut flush=false; let mut renew=false;
    let mut idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_NEXT);
    if idx != -1 {
        (ext::VISetNextFrameBuffer)(HSD_VIData.xfb[idx as usize].buffer);
        if HSD_VIData.xfb[idx as usize].vi_all.chg_flag != 0 {
            (ext::VIConfigure)(&raw mut HSD_VIData.xfb[idx as usize].vi_all.vi.rmode);
            (ext::VISetBlack)(HSD_VIData.xfb[idx as usize].vi_all.vi.black);
        }
        flush=true; renew=true;
    } else if HSD_VIGetNbXFB()==1 && HSD_VIData.efb.status==HSD_VI_EFB_DRAWDONE {
        idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_DISPLAY);
        if idx == -1 {
            idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_FREE);
            assert!(idx != -1,"HSD_ASSERT(252)");
            (ext::VISetNextFrameBuffer)(HSD_VIData.xfb[idx as usize].buffer); flush=true;
        }
        HSD_VIData.xfb[idx as usize].status=HSD_VI_XFB_COPYEFB;
        if HSD_VIData.efb.vi_all.chg_flag != 0 {
            (ext::VIConfigure)(&raw mut HSD_VIData.efb.vi_all.vi.rmode);
            (ext::VISetBlack)(HSD_VIData.efb.vi_all.vi.black); flush=true;
        }
        renew=true;
    }
    if flush {(ext::VIFlush)();}
    if renew {renew_count=renew_count.wrapping_add(1);}
    vr_count=vr_count.wrapping_add(1);
    if vr_count>=HSD_VIData.perf.frame_period {
        HSD_VIData.perf.frame_renew=renew_count; vr_count=0; renew_count=0;
    }
    if let Some(cb)=HSD_VIData.pre_cb {cb(retraceCount);}
}
pub unsafe extern "C" fn HSD_VIPostRetraceCB(retraceCount:u32) {
    let next=HSD_VISearchXFBByStatus(HSD_VI_XFB_NEXT);
    if next != -1 {
        let idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_DISPLAY);
        if idx != -1 {HSD_VIData.xfb[idx as usize].status=HSD_VI_XFB_FREE;}
        HSD_VIData.xfb[next as usize].status=HSD_VI_XFB_DISPLAY;
        let idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_DRAWDONE);
        if idx != -1 {HSD_VIData.xfb[idx as usize].status=HSD_VI_XFB_NEXT;}
    } else {
        let idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_COPYEFB);
        if idx != -1 {
            HSD_VICopyEFB2XFBPtr(&raw mut HSD_VIData.efb.vi_all.vi,HSD_VIData.xfb[idx as usize].buffer,HSD_RP_SCREEN);
            HSD_VIData.xfb[idx as usize].status=HSD_VI_XFB_DISPLAY; HSD_VIData.efb.status=HSD_VI_EFB_FREE;
        }
    }
    if let Some(cb)=HSD_VIData.post_cb {cb(retraceCount);}
}
pub unsafe extern "C" fn HSD_VIGXDrawDoneCB() {
    HSD_VIData.drawdone.waiting=0;
    if let Some(cb)=HSD_VIData.drawdone.cb {cb(HSD_VIData.drawdone.arg);}
}
pub unsafe fn HSD_VIGetDrawDoneWaitingFlag()->i32 {HSD_VIData.drawdone.waiting}
pub unsafe fn HSD_VIGetXFBDrawEnable()->i32 {
    if HSD_VIGetNbXFB()<2 {return -1;}
    let intr=(ext::OSDisableInterrupts)();
    let mut idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_DRAWING);
    if idx == -1 {idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_FREE);if idx != -1 {HSD_VIData.xfb[idx as usize].status=HSD_VI_XFB_DRAWING;}}
    (ext::OSRestoreInterrupts)(intr);idx
}
pub unsafe fn HSD_VIWaitXFBDrawEnable()->i32 {
    if HSD_VIGetNbXFB()<2 {return -1;}
    loop {let idx=HSD_VIGetXFBDrawEnable();if idx != -1 {return idx;}(ext::VIWaitForRetrace)();}
}
pub unsafe fn HSD_VICopyEFB2XFBHiResoAA(rmode:*mut GXRenderModeObj) {
    (ext::GXSetDispCopySrc)(0,0,(*rmode).fbWidth,(*rmode).efbHeight.wrapping_sub(4));
    let lines=(ext::GXSetDispCopyYScale)(1.0);(ext::GXSetDispCopyDst)((*rmode).fbWidth,lines as u16);
}
pub unsafe fn HSD_VICopyEFB2XFBPtr(vi:*mut HSD_VIStatus,buffer:*mut c_void,rpass:HSD_RenderPass) {
    let rmode=&raw mut (*vi).rmode;
    (ext::GXSetCopyFilter)((*rmode).aa,(*rmode).sample_pattern.as_ptr(),(*vi).vf,(*rmode).vfilter.as_ptr());
    (ext::GXSetDispCopyGamma)((*vi).gamma);
    (ext::HSD_StateSetColorUpdate)((*vi).update_clr as i32);
    (ext::HSD_StateSetAlphaUpdate)((*vi).update_alpha as i32);
    (ext::HSD_StateSetZMode)((*vi).update_z as i32,GX_LEQUAL as i32,GX_TRUE as i32);
    (ext::GXSetCopyClear)((*vi).clear_clr,(*vi).clear_z);
    match rpass {
        HSD_RP_SCREEN=>{
            (ext::GXSetCopyClamp)(GX_CLAMP_TOP|GX_CLAMP_BOTTOM);
            (ext::GXSetDispCopySrc)(0,0,(*rmode).fbWidth,(*rmode).efbHeight);
            let lines=(ext::GXSetDispCopyYScale)((*rmode).xfbHeight as f32/(*rmode).efbHeight as f32);
            (ext::GXSetDispCopyDst)((*rmode).fbWidth,lines as u16);(ext::GXCopyDisp)(buffer,GX_TRUE as u8);
        }
        HSD_RP_TOPHALF=>{
            HSD_VICopyEFB2XFBHiResoAA(rmode);(ext::GXSetCopyClamp)(GX_CLAMP_TOP);
            let lines=(*rmode).efbHeight.wrapping_sub(4);
            (ext::GXSetDispCopySrc)(0,0,(*rmode).fbWidth,lines);(ext::GXCopyDisp)(buffer,GX_TRUE as u8);
            (ext::GXPixModeSync)();return;
        }
        HSD_RP_BOTTOMHALF=>{
            HSD_VICopyEFB2XFBHiResoAA(rmode);(ext::GXSetCopyClamp)(GX_CLAMP_BOTTOM);
            let lines=(*rmode).efbHeight.wrapping_sub(4);
            (ext::GXSetDispCopySrc)(0,4,(*rmode).fbWidth,lines);
            // Native ABI adaptation: offset arithmetic remains u32; framebuffer pointers are
            // native-width. The C oracle changes only this console pointer cast accordingly.
            let width=((*rmode).fbWidth as u32+15)&!15;
            let offset=width.wrapping_mul(lines as u32).wrapping_mul(2);
            (ext::GXCopyDisp)((buffer as usize).wrapping_add(offset as usize)as *mut c_void,GX_TRUE as u8);
            (ext::GXSetDispCopySrc)(0,0,(*rmode).fbWidth,4);(ext::GXSetCopyClamp)(GX_CLAMP_TOP|GX_CLAMP_BOTTOM);
            (ext::GXCopyDisp)(core::ptr::addr_of_mut!(garbage).cast(),GX_TRUE as u8);
        }
        _=>panic!("video.c:0x207: unexpected type of render pass"),
    }
    (ext::GXPixModeSync)();
}
pub unsafe fn HSD_VIGXSetDrawDone(arg:i32) {
    while HSD_VIGetDrawDoneWaitingFlag()!=0 {(ext::GXWaitDrawDone)();}
    HSD_VIData.drawdone.waiting=1;HSD_VIData.drawdone.arg=arg;(ext::GXSetDrawDone)();
}
pub unsafe fn HSD_VISetXFBWaitDone(idx:i32) {
    let intr=(ext::OSDisableInterrupts)();
    assert_eq!(HSD_VIData.xfb[idx as usize].status,HSD_VI_XFB_DRAWING,"HSD_ASSERT(590)");
    HSD_VIData.xfb[idx as usize].status=HSD_VI_XFB_WAITDONE;
    HSD_VIData.xfb[idx as usize].vi_all=HSD_VIData.current;HSD_VIData.current.chg_flag=0;
    (ext::OSRestoreInterrupts)(intr);
}
pub unsafe fn HSD_VICopyXFBAsync(rpass:HSD_RenderPass) {
    if HSD_VIGetNbXFB()<2 {return;}
    let idx=HSD_VIWaitXFBDrawEnable();HSD_VICopyEFB2XFBPtr(HSD_VIGetVIStatus(),HSD_VIGetXFBPtr(idx),rpass);
    HSD_VISetXFBWaitDone(idx);HSD_VIGXSetDrawDone(idx);
}
pub unsafe fn HSD_VIDrawDoneXFB(idx:i32) {
    let intr=(ext::OSDisableInterrupts)();
    assert_eq!(HSD_VIData.xfb[idx as usize].status,HSD_VI_XFB_WAITDONE,"HSD_ASSERT(722)");
    HSD_VIData.xfb[idx as usize].status=if HSD_VISearchXFBByStatus(HSD_VI_XFB_NEXT)!=-1 {HSD_VI_XFB_DRAWDONE}else{HSD_VI_XFB_NEXT};
    (ext::OSRestoreInterrupts)(intr);
}
pub unsafe fn HSD_VIWaitXFBFlush_sub()->i32 {
    let intr=(ext::OSDisableInterrupts)();
    let val=(HSD_VISearchXFBByStatus(HSD_VI_XFB_WAITDONE)!=-1||HSD_VISearchXFBByStatus(HSD_VI_XFB_DRAWDONE)!=-1||HSD_VISearchXFBByStatus(HSD_VI_XFB_NEXT)!=-1)as i32;
    (ext::OSRestoreInterrupts)(intr);val
}
pub unsafe fn HSD_VIWaitXFBFlush() {
    if HSD_VIGetNbXFB()<2 {return;}while HSD_VIWaitXFBFlush_sub()!=0 {(ext::VIWaitForRetrace)();}
}
pub unsafe fn HSD_VIWaitXFBFlushNoYield() {
    if HSD_VIGetNbXFB()<2 {return;}while HSD_VIWaitXFBFlush_sub()!=0 {}
}
pub unsafe fn HSD_VIGetXFBLastDrawDone()->i32 {
    let intr=(ext::OSDisableInterrupts)();let mut idx=-1;
    for status in [HSD_VI_XFB_WAITDONE,HSD_VI_XFB_DRAWDONE,HSD_VI_XFB_NEXT,HSD_VI_XFB_DISPLAY] {
        idx=HSD_VISearchXFBByStatus(status);if idx != -1 {break;}
    }
    (ext::OSRestoreInterrupts)(intr);idx
}
pub unsafe fn HSD_VISetConfigure(rmode:*mut GXRenderModeObj) {HSD_VIData.current.vi.rmode=*rmode;HSD_VIData.current.chg_flag=1;}
pub unsafe fn HSD_VISetBlack(black:i32) {HSD_VIData.current.vi.black=black;HSD_VIData.current.chg_flag=1;}
pub unsafe fn HSD_VIInit(vi:*mut HSD_VIStatus,xfb0:*mut c_void,xfb1:*mut c_void,xfb2:*mut c_void) {
    (ext::VIInit)();HSD_VIData.current.vi=*vi;HSD_VIData.current.chg_flag=0;
    HSD_VIData.xfb[0].buffer=xfb0;HSD_VIData.xfb[1].buffer=xfb1;HSD_VIData.xfb[2].buffer=xfb2;
    let mut fbnum=0;
    for i in 0..3 {
        HSD_VIData.xfb[i].vi_all=HSD_VIData.current;
        HSD_VIData.xfb[i].status=if !HSD_VIData.xfb[i].buffer.is_null(){fbnum+=1;HSD_VI_XFB_FREE}else{HSD_VI_XFB_NONE};
    }
    HSD_VIData.nb_xfb=fbnum;HSD_VIData.efb.status=HSD_VI_EFB_FREE;HSD_VIData.efb.vi_all=HSD_VIData.current;
    (ext::VISetPreRetraceCallback)(Some(HSD_VIPreRetraceCB));(ext::VISetPostRetraceCallback)(Some(HSD_VIPostRetraceCB));
    HSD_VIData.pre_cb=None;HSD_VIData.post_cb=None;HSD_VIData.drawdone.waiting=0;HSD_VIData.drawdone.arg=0;
    (ext::GXSetDrawDoneCallback)(Some(HSD_VIGXDrawDoneCB));HSD_VIData.drawdone.cb=None;
    HSD_VIData.perf.frame_period=if (ext::VIGetTvFormat)()==0{60}else{50};HSD_VIData.perf.frame_renew=0;
    (ext::VIConfigure)(&raw mut HSD_VIData.current.vi.rmode);(ext::VISetBlack)(HSD_VIData.current.vi.black);(ext::VIFlush)();
    let idx=HSD_VISearchXFBByStatus(HSD_VI_XFB_FREE);HSD_VICopyEFB2XFBPtr(HSD_VIGetVIStatus(),HSD_VIGetXFBPtr(idx),HSD_RP_SCREEN);
}
pub unsafe fn HSD_VIGetNbXFB()->i32 {HSD_VIData.nb_xfb}
pub unsafe fn HSD_VIGetXFBPtr(idx:i32)->*mut c_void {HSD_VIData.xfb[idx as usize].buffer}
pub unsafe fn HSD_VIGetVIStatus()->*mut HSD_VIStatus {&raw mut HSD_VIData.current.vi}
pub unsafe fn HSD_VIGetRenderMode()->*mut GXRenderModeObj {&raw mut HSD_VIData.current.vi.rmode}
