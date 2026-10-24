//! Native port of `src/sysdolphin/baselib/initialize.c`: HSD start-up (OS heap + arena setup,
//! XFB / FIFO allocation, GX light preload, object-allocator init) and the init parameters.
//!
//! The `TARGET_PC` build is what ships, so `HSD_SetInitParameterU32` / `HSD_SetInitParameterPtr`
//! replace the variadic `HSD_SetInitParameter`. File-scope statics are `static mut`, as in C.
//! Every call into an unported TU lives in `mod ext` as a TODO stub named like the C function.
#![allow(non_snake_case, non_upper_case_globals)]
use core::ffi::c_void;
use core::ptr::null_mut;
use melee_types::*;

use super::id::_HSD_IDForgetMemory;
use super::id::{HSD_IDGetAllocData, HSD_IDInitAllocData, HSD_IDSetup};
use super::list::{HSD_DListGetAllocData, HSD_ListInitAllocData, HSD_SListGetAllocData};

static mut FrameBuffer: [*mut c_void; HSD_VI_XFB_MAX as usize] = [null_mut(); HSD_VI_XFB_MAX as usize];
static mut memReport: HSD_MemReport = unsafe { core::mem::zeroed() };
static mut hsd_heap_next_arena_lo: *mut c_void = null_mut();
static mut hsd_heap_next_arena_hi: *mut c_void = null_mut();
static mut current_render_pass: HSD_RenderPass = 0;
static mut DefaultFifoObj: *mut GXFifoObj = null_mut();
static mut current_pix_fmt: i32 = 0;
static mut init_done: i32 = 0;
static mut shown: i32 = 0;

static mut current_heap: OSHeapHandle = -1;
/// `rmode = &GXNtsc480IntDf`; NULL stands for the default object, resolved through `ext`.
static mut rmode: *mut GXRenderModeObj = null_mut();
static mut current_z_fmt: i32 = GX_ZC_MID as i32;
static mut iparam_fifo_size: u32 = HSD_DEFAULT_FIFO_SIZE as u32;
static mut iparam_xfb_max_num: i32 = HSD_DEFAULT_XFB_MAX_NUM;
static mut iparam_heap_max_num: i32 = 4;
static mut iparam_audio_heap_size: u32 = HSD_DEFAULT_AUDIO_SIZE as u32;
static HSD_Init_804D5E1C: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

unsafe fn current_rmode() -> *mut GXRenderModeObj {
    if rmode.is_null() { ext::GXNtsc480IntDf() } else { rmode }
}

#[inline]
fn round_up_32b(x: usize) -> usize {
    (x + 31) & !31usize
}
#[inline]
fn round_down_32b(x: usize) -> usize {
    x & !31usize
}

pub unsafe fn HSD_InitComponent() {
    HSD_OSInit();
    {
        let mut vi_status: HSD_VIStatus = HSD_VIStatus::zeroed();
        let black = GXColor { r: 0, g: 0, b: 0, a: 0 };

        vi_status.rmode = *current_rmode();
        vi_status.black = 1;
        vi_status.vf = 1;
        vi_status.gamma = GX_GM_1_0;
        vi_status.clear_clr = black;
        vi_status.clear_z = 0x00FF_FFFF; // GX_MAX_Z24
        vi_status.update_clr = 1;
        vi_status.update_alpha = 1;
        vi_status.update_z = 1;

        ext::HSD_VIInit(&mut vi_status, FrameBuffer[0], FrameBuffer[1], FrameBuffer[2]);
    }

    HSD_GXInit();
    HSD_DVDInit();
    HSD_IDSetup();
    ext::VIWaitForRetrace();
    HSD_ObjInit();
    ext::HSD_LogInit();
    init_done = 1;
}

pub unsafe fn HSD_GXSetFifoObj(fifo: *mut GXFifoObj) {
    memReport.gxfifo = iparam_fifo_size;
    DefaultFifoObj = fifo;
}

pub fn HSD_DVDInit() {}

pub unsafe fn HSD_AllocateXFB(nbuffer: i32, rm: *mut GXRenderModeObj) -> *mut *mut c_void {
    if rm.is_null() {
        return null_mut();
    }
    let fb_size: u32 = ((((*rm).fbWidth as u32).wrapping_add(0xF)) & 0xFFF0)
        .wrapping_mul((*rm).xfbHeight as u32)
        .wrapping_mul(2);
    let mut arena_lo: usize = round_up_32b(ext::OSGetArenaLo() as usize);
    let arena_hi: usize = round_down_32b(ext::OSGetArenaHi() as usize);
    memReport.xfb = fb_size.wrapping_mul(nbuffer as u32);
    if arena_lo == arena_hi {
        for i in 0..nbuffer {
            FrameBuffer[i as usize] = ext::OSAllocFromHeap(ext::__OSCurrHeap(), fb_size);
        }
    } else {
        for i in 0..nbuffer {
            FrameBuffer[i as usize] = arena_lo as *mut c_void;
            arena_lo = round_up_32b(arena_lo.wrapping_add(fb_size as usize));
        }
        if arena_lo > arena_hi {
            ext::HSD_Panic("initialize.c", 265, "No memory space remains for XFB.\n");
        }
        ext::OSSetArenaLo(arena_lo as *mut c_void);
    }
    for i in nbuffer..HSD_VI_XFB_MAX {
        FrameBuffer[i as usize] = null_mut();
    }
    (&raw mut FrameBuffer).cast::<*mut c_void>()
}

pub unsafe fn HSD_AllocateFifo(size: u32) -> *mut GXFifoObj {
    let arena_lo0: usize = round_up_32b(ext::OSGetArenaLo() as usize);
    let arena_hi: usize = round_down_32b(ext::OSGetArenaHi() as usize);
    let fifo: *mut GXFifoObj;
    if arena_lo0 == 0 && arena_hi == 0 {
        fifo = ext::OSAllocFromHeap(ext::__OSCurrHeap(), size).cast();
        if fifo.is_null() {
            ext::HSD_Panic("initialize.c", 295, "cannot allocate memory for gx fifo.\n");
        }
    } else {
        fifo = arena_lo0 as *mut GXFifoObj;
        let arena_lo = arena_lo0.wrapping_add(size as usize);
        if arena_lo > ext::OSGetArenaHi() as usize {
            ext::HSD_Panic("initialize.c", 302, "no space remains for gx fifo.\n");
        }
        ext::OSSetArenaLo(arena_lo as *mut c_void);
    }
    fifo
}

pub unsafe fn HSD_GXInit() {
    let mut lightobj: GXLightObj = GXLightObj::zeroed();

    ext::GXInitLightPos(&mut lightobj, 1.0, 0.0, 0.0);
    ext::GXInitLightDir(&mut lightobj, 1.0, 0.0, 0.0);
    ext::GXInitLightAttn(&mut lightobj, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    ext::GXInitLightColor(&mut lightobj, HSD_Init_804D5E1C);

    for i in 0..8 {
        ext::GXLoadLightObjImm(&mut lightobj, ext::HSD_Index2LightID(i));
    }
    ext::HSD_StateInvalidate(-1);
}

pub unsafe fn HSD_OSInit() {
    let old_arena_lo0 = ext::OSGetArenaLo() as usize;
    let old_arena_hi = ext::OSGetArenaHi() as usize;
    memReport.total = ext::OSGetPhysicalMemSize();
    memReport.system = (memReport.total as usize)
        .wrapping_sub(ext::OSGetArenaHi() as usize)
        .wrapping_add(ext::OSGetArenaLo() as usize)
        .wrapping_sub(memReport.xfb as usize)
        .wrapping_sub(memReport.gxfifo as usize) as u32;
    let old_arena_lo = ext::OSInitAlloc(
        old_arena_lo0 as *mut c_void,
        old_arena_hi as *mut c_void,
        iparam_heap_max_num,
    ) as usize;
    ext::OSSetArenaLo(old_arena_lo as *mut c_void);

    let mut new_arena_lo = round_up_32b(old_arena_lo);
    let new_arena_hi = round_down_32b(old_arena_hi);
    ext::set_HSD_Synth_804D6018(ext::OSCreateHeap(
        new_arena_lo as *mut c_void,
        (new_arena_lo + iparam_audio_heap_size as usize) as *mut c_void,
    ));
    new_arena_lo += iparam_audio_heap_size as usize;
    hsd_heap_next_arena_lo = new_arena_lo as *mut c_void;
    hsd_heap_next_arena_hi = new_arena_hi as *mut c_void;
    current_heap = ext::OSCreateHeap(new_arena_lo as *mut c_void, new_arena_hi as *mut c_void);
    ext::OSSetCurrentHeap(current_heap);
    memReport.heap = new_arena_hi.wrapping_sub(new_arena_lo) as u32;
    ext::HSD_ObjSetHeap(new_arena_hi.wrapping_sub(new_arena_lo) as u32, null_mut());
    ext::OSSetArenaLo(new_arena_hi as *mut c_void);
}

pub unsafe fn HSD_GetHeap() -> OSHeapHandle {
    current_heap
}

pub unsafe fn HSD_SetHeap(handle: OSHeapHandle) {
    current_heap = handle;
}

pub unsafe fn HSD_GetNextArena(lo: *mut *mut c_void, hi: *mut *mut c_void) {
    *lo = hsd_heap_next_arena_lo;
    *hi = hsd_heap_next_arena_hi;
}

pub unsafe fn HSD_CreateMainHeap(lo: *mut c_void, hi: *mut c_void) -> OSHeapHandle {
    let cb_table: [unsafe fn(*mut c_void, *mut c_void); 6] = [
        ext::_HSD_AObjForgetMemory,
        ext::_HSD_DispForgetMemory,
        _HSD_IDForgetMemory,
        ext::_HSD_ObjAllocForgetMemory,
        ext::_HSD_RandForgetMemory,
        ext::_HSD_RObjForgetMemory,
    ];
    ext::hsdForgetClassLibrary(c"sysdolphin_base_library".as_ptr());
    HSD_ObjInit();
    for cb in cb_table {
        cb(hsd_heap_next_arena_lo, hsd_heap_next_arena_hi);
    }
    if !lo.is_null() {
        hsd_heap_next_arena_lo = lo;
    }
    if !hi.is_null() {
        hsd_heap_next_arena_hi = hi;
    }
    ext::OSDestroyHeap(current_heap);
    current_heap = ext::OSCreateHeap(hsd_heap_next_arena_lo, hsd_heap_next_arena_hi);
    ext::OSSetCurrentHeap(current_heap);
    ext::HSD_ObjSetHeap(
        (hsd_heap_next_arena_hi as usize).wrapping_sub(hsd_heap_next_arena_lo as usize) as u32,
        null_mut(),
    );
    current_heap
}

pub unsafe fn HSD_GetCurrentRenderPass() -> HSD_RenderPass {
    current_render_pass
}

pub unsafe fn HSD_StartRender(pass: HSD_RenderPass) {
    let rm = ext::HSD_VIGetRenderMode();
    current_render_pass = pass;
    if (*rm).aa != 0 {
        ext::GXSetPixelFmt(GX_PF_RGB565_Z16 as i32, current_z_fmt);
    } else {
        ext::GXSetPixelFmt(current_pix_fmt, GX_ZC_LINEAR as i32);
    }
    ext::GXSetFieldMode(
        (*rm).field_rendering,
        ((*rm).xfbHeight < (*rm).viHeight) as u8,
    );
}

/// The C body is a comparison with no effect (it only forces instruction order when matching).
pub unsafe fn HSD_Init_803755A8() {}

pub unsafe fn HSD_ObjInit() {
    HSD_ListInitAllocData();
    ext::HSD_AObjInitAllocData();
    ext::HSD_FObjInitAllocData();
    HSD_IDInitAllocData();
    ext::HSD_VecInitAllocData();
    ext::HSD_MtxInitAllocData();
    ext::HSD_RObjInitAllocData();
    ext::HSD_RenderInitAllocData();
    ext::HSD_ShadowInitAllocData();
    ext::HSD_ZListInitAllocData();
}

pub unsafe fn HSD_ObjDumpStat() {
    let types: [(unsafe fn() -> *mut HSD_ObjAllocData, &str); 13] = [
        (ext::HSD_AObjGetAllocData, "aobj"),
        (ext::HSD_FObjGetAllocData, "fobj"),
        (HSD_IDGetAllocData, "id"),
        (HSD_SListGetAllocData, "slist"),
        (HSD_DListGetAllocData, "dlist"),
        (ext::HSD_VecGetAllocData, "vec"),
        (ext::HSD_MtxGetAllocData, "mtx"),
        (ext::HSD_RObjGetAllocData, "robj"),
        (ext::HSD_RvalueObjGetAllocData, "rval"),
        (ext::HSD_ShadowGetAllocData, "shadow"),
        (ext::HSD_RenderGetAllocData, "render"),
        (ext::HSD_ChanGetAllocData, "chan"),
        (ext::HSD_TevRegGetAllocData, "tevreg"),
    ];
    for (func, label) in types {
        ext::OSReport(&format!(
            "objalloc: {}\tusing {}\tfreed {}\tpeak {}\n",
            label,
            ext::HSD_ObjAllocGetUsing(func()),
            ext::HSD_ObjAllocGetFreed(func()),
            ext::HSD_ObjAllocGetPeak(func()),
        ));
    }
}

unsafe fn hsd_init_parameter_allowed() -> bool {
    if init_done != 0 {
        if shown == 0 {
            ext::OSReport("init parameter should be set before invoking HSD_Init().\n");
            shown = 1;
        }
        return false;
    }
    true
}

pub unsafe fn HSD_SetInitParameterU32(param: HSD_InitParam, value: u32) -> bool {
    if !hsd_init_parameter_allowed() {
        return false;
    }
    match param {
        HSD_INIT_FIFO_SIZE => {
            if value > 0 {
                iparam_fifo_size = value;
                return true;
            }
        }
        HSD_INIT_XFB_MAX_NUM => {
            if value > 0 {
                iparam_xfb_max_num = value as i32;
                return true;
            }
        }
        HSD_INIT_HEAP_MAX_NUM => {
            if value > 0 {
                iparam_heap_max_num = value as i32;
                return true;
            }
        }
        HSD_INIT_AUDIO_HEAP_SIZE => {
            if value > 0 {
                iparam_audio_heap_size = value;
                return true;
            }
        }
        _ => {}
    }
    false
}

pub unsafe fn HSD_SetInitParameterPtr(param: HSD_InitParam, value: *mut c_void) -> bool {
    if !hsd_init_parameter_allowed() {
        return false;
    }
    if param == HSD_INIT_RENDER_MODE_OBJ {
        rmode = value.cast();
        return true;
    }
    false
}

/// Callees in unported TUs; each is a TODO stub named like the C function.
pub mod ext {
    use super::*;
    macro_rules! todo_fns {
        ($( fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? ; )*) => {
            $( pub unsafe fn $name($($a: $t),*) $(-> $r)? { todo!(concat!("TODO ext: ", stringify!($name))) } )*
        };
    }
    todo_fns! {
        fn GXNtsc480IntDf() -> *mut GXRenderModeObj;
        fn HSD_VIInit(s: *mut HSD_VIStatus, a: *mut c_void, b: *mut c_void, c: *mut c_void);
        fn VIWaitForRetrace();
        fn HSD_LogInit();
        fn OSGetArenaLo() -> *mut c_void;
        fn OSGetArenaHi() -> *mut c_void;
        fn OSSetArenaLo(p: *mut c_void);
        fn OSAllocFromHeap(h: OSHeapHandle, size: u32) -> *mut c_void;
        fn __OSCurrHeap() -> OSHeapHandle;
        fn HSD_Panic(file: &str, line: i32, msg: &str);
        fn GXInitLightPos(l: *mut GXLightObj, x: f32, y: f32, z: f32);
        fn GXInitLightDir(l: *mut GXLightObj, x: f32, y: f32, z: f32);
        fn GXInitLightAttn(l: *mut GXLightObj, a0: f32, a1: f32, a2: f32, k0: f32, k1: f32, k2: f32);
        fn GXInitLightColor(l: *mut GXLightObj, c: GXColor);
        fn GXLoadLightObjImm(l: *mut GXLightObj, id: u32);
        fn HSD_Index2LightID(i: i32) -> u32;
        fn HSD_StateInvalidate(mask: i32);
        fn OSGetPhysicalMemSize() -> u32;
        fn OSInitAlloc(lo: *mut c_void, hi: *mut c_void, max: i32) -> *mut c_void;
        fn OSCreateHeap(lo: *mut c_void, hi: *mut c_void) -> OSHeapHandle;
        fn OSSetCurrentHeap(h: OSHeapHandle) -> OSHeapHandle;
        fn OSDestroyHeap(h: OSHeapHandle);
        fn HSD_ObjSetHeap(size: u32, p: *mut c_void);
        fn set_HSD_Synth_804D6018(h: OSHeapHandle);
        fn hsdForgetClassLibrary(name: *const i8);
        fn _HSD_AObjForgetMemory(lo: *mut c_void, hi: *mut c_void);
        fn _HSD_DispForgetMemory(lo: *mut c_void, hi: *mut c_void);
        fn _HSD_ObjAllocForgetMemory(lo: *mut c_void, hi: *mut c_void);
        fn _HSD_RandForgetMemory(lo: *mut c_void, hi: *mut c_void);
        fn _HSD_RObjForgetMemory(lo: *mut c_void, hi: *mut c_void);
        fn HSD_VIGetRenderMode() -> *mut GXRenderModeObj;
        fn GXSetPixelFmt(pix: i32, z: i32);
        fn GXSetFieldMode(field_rendering: u8, half_aspect: u8);
        fn HSD_AObjInitAllocData();
        fn HSD_FObjInitAllocData();
        fn HSD_VecInitAllocData();
        fn HSD_MtxInitAllocData();
        fn HSD_RObjInitAllocData();
        fn HSD_RenderInitAllocData();
        fn HSD_ShadowInitAllocData();
        fn HSD_ZListInitAllocData();
        fn HSD_AObjGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_FObjGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_VecGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_MtxGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_RObjGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_RvalueObjGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_ShadowGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_RenderGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_ChanGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_TevRegGetAllocData() -> *mut HSD_ObjAllocData;
        fn HSD_ObjAllocGetUsing(d: *mut HSD_ObjAllocData) -> u32;
        fn HSD_ObjAllocGetFreed(d: *mut HSD_ObjAllocData) -> u32;
        fn HSD_ObjAllocGetPeak(d: *mut HSD_ObjAllocData) -> u32;
    }
    pub fn OSReport(_s: &str) {
        todo!("TODO ext: OSReport (dolphin/os)")
    }
}
