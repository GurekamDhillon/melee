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

use super::lbext::OsArg;

pub static mut FrameBuffer: [*mut c_void; HSD_VI_XFB_MAX as usize] = [null_mut(); HSD_VI_XFB_MAX as usize];
pub static mut memReport: HSD_MemReport = unsafe { core::mem::zeroed() };
pub static mut hsd_heap_next_arena_lo: *mut c_void = null_mut();
pub static mut hsd_heap_next_arena_hi: *mut c_void = null_mut();
pub static mut current_render_pass: HSD_RenderPass = 0;
pub static mut DefaultFifoObj: *mut GXFifoObj = null_mut();
pub static mut current_pix_fmt: i32 = 0;
pub static mut init_done: i32 = 0;
pub static mut shown: i32 = 0;

pub static mut current_heap: OSHeapHandle = -1;
/// `rmode = &GXNtsc480IntDf`; NULL stands for the default object, resolved through `ext`.
pub static mut rmode: *mut GXRenderModeObj = null_mut();
pub static mut current_z_fmt: i32 = GX_ZC_MID as i32;
pub static mut iparam_fifo_size: u32 = HSD_DEFAULT_FIFO_SIZE as u32;
pub static mut iparam_xfb_max_num: i32 = HSD_DEFAULT_XFB_MAX_NUM;
pub static mut iparam_heap_max_num: i32 = 4;
pub static mut iparam_audio_heap_size: u32 = HSD_DEFAULT_AUDIO_SIZE as u32;
static HSD_Init_804D5E1C: GXColor = GXColor { r: 0, g: 0, b: 0, a: 0 };

/// Tests that own their copy of other TUs' data set this; the game uses the owning TU's statics.
pub static mut use_ext_data: bool = false;
/// `GXNtsc480IntDf`, the default render mode (no Rust owner yet outside the oracle).
unsafe fn ntsc480_int_df() -> *mut GXRenderModeObj {
    if use_ext_data { core::ptr::addr_of_mut!(ext::GXNtsc480IntDf) } else { panic!("TODO: GXNtsc480IntDf is not ported") }
}
/// `__OSCurrHeap`.
unsafe fn os_curr_heap() -> OSHeapHandle {
    if use_ext_data { ext::__OSCurrHeap } else { panic!("TODO: __OSCurrHeap is not ported") }
}
unsafe fn current_rmode() -> *mut GXRenderModeObj {
    if rmode.is_null() { ntsc480_int_df() } else { rmode }
}
/// video.h `HSD_VIGetRenderMode` (inline): `&HSD_VIData.current.vi.rmode`.
unsafe fn vi_render_mode() -> *mut GXRenderModeObj {
    if use_ext_data { core::ptr::addr_of_mut!(ext::HSD_VIData.current.vi.rmode) } else { core::ptr::addr_of_mut!(super::video::HSD_VIData.current.vi.rmode) }
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
    melee_core::ct!("HSD_InitComponent");
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
    ext::HSD_IDSetup();
    ext::VIWaitForRetrace();
    HSD_ObjInit();
    ext::HSD_LogInit();
    init_done = 1;
}

pub unsafe fn HSD_GXSetFifoObj(fifo: *mut GXFifoObj) {
    melee_core::ct!("HSD_GXSetFifoObj");
    memReport.gxfifo = iparam_fifo_size;
    DefaultFifoObj = fifo;
}

pub fn HSD_DVDInit() { melee_core::ct!("HSD_DVDInit");}

pub unsafe fn HSD_AllocateXFB(nbuffer: i32, rm: *mut GXRenderModeObj) -> *mut *mut c_void {
    melee_core::ct!("HSD_AllocateXFB");
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
            FrameBuffer[i as usize] = ext::OSAllocFromHeap(os_curr_heap(), fb_size);
        }
    } else {
        for i in 0..nbuffer {
            FrameBuffer[i as usize] = arena_lo as *mut c_void;
            arena_lo = round_up_32b(arena_lo.wrapping_add(fb_size as usize));
        }
        if arena_lo > arena_hi {
            super::debug::HSD_Panic(c"initialize.c".as_ptr(), 265, c"No memory space remains for XFB.\n".as_ptr());
        }
        ext::OSSetArenaLo(arena_lo as *mut c_void);
    }
    for i in nbuffer..HSD_VI_XFB_MAX {
        FrameBuffer[i as usize] = null_mut();
    }
    (&raw mut FrameBuffer).cast::<*mut c_void>()
}

pub unsafe fn HSD_AllocateFifo(size: u32) -> *mut GXFifoObj {
    melee_core::ct!("HSD_AllocateFifo");
    let arena_lo0: usize = round_up_32b(ext::OSGetArenaLo() as usize);
    let arena_hi: usize = round_down_32b(ext::OSGetArenaHi() as usize);
    let fifo: *mut GXFifoObj;
    if arena_lo0 == 0 && arena_hi == 0 {
        fifo = ext::OSAllocFromHeap(os_curr_heap(), size).cast();
        if fifo.is_null() {
            super::debug::HSD_Panic(c"initialize.c".as_ptr(), 295, c"cannot allocate memory for gx fifo.\n".as_ptr());
        }
    } else {
        fifo = arena_lo0 as *mut GXFifoObj;
        let arena_lo = arena_lo0.wrapping_add(size as usize);
        if arena_lo > ext::OSGetArenaHi() as usize {
            super::debug::HSD_Panic(c"initialize.c".as_ptr(), 302, c"no space remains for gx fifo.\n".as_ptr());
        }
        ext::OSSetArenaLo(arena_lo as *mut c_void);
    }
    fifo
}

pub unsafe fn HSD_GXInit() {
    melee_core::ct!("HSD_GXInit");
    let mut lightobj: GXLightObj = GXLightObj::zeroed();

    ext::GXInitLightPos(&mut lightobj, 1.0, 0.0, 0.0);
    ext::GXInitLightDir(&mut lightobj, 1.0, 0.0, 0.0);
    ext::GXInitLightAttn(&mut lightobj, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    ext::GXInitLightColor(&mut lightobj, HSD_Init_804D5E1C);

    for i in 0..8 {
        ext::GXLoadLightObjImm(&mut lightobj, ext::HSD_Index2LightID(i) as u32);
    }
    ext::HSD_StateInvalidate(-1);
}

pub unsafe fn HSD_OSInit() {
    melee_core::ct!("HSD_OSInit");
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
    set_synth_heap(ext::OSCreateHeap(
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

unsafe fn set_synth_heap(h: OSHeapHandle) {
    if use_ext_data { ext::HSD_Synth_804D6018 = h } else { super::synth::HSD_Synth_804D6018 = h }
}

pub unsafe fn HSD_GetHeap() -> OSHeapHandle {
    melee_core::ct!("HSD_GetHeap");
    current_heap
}

pub unsafe fn HSD_SetHeap(handle: OSHeapHandle) {
    melee_core::ct!("HSD_SetHeap");
    current_heap = handle;
}

pub unsafe fn HSD_GetNextArena(lo: *mut *mut c_void, hi: *mut *mut c_void) {
    melee_core::ct!("HSD_GetNextArena");
    *lo = hsd_heap_next_arena_lo;
    *hi = hsd_heap_next_arena_hi;
}

pub unsafe fn HSD_CreateMainHeap(lo: *mut c_void, hi: *mut c_void) -> OSHeapHandle {
    melee_core::ct!("HSD_CreateMainHeap");
    let cb_table: [unsafe fn(*mut c_void, *mut c_void); 6] = [
        ext::_HSD_AObjForgetMemory,
        ext::_HSD_DispForgetMemory,
        ext::_HSD_IDForgetMemory,
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
    melee_core::ct!("HSD_GetCurrentRenderPass");
    current_render_pass
}

pub unsafe fn HSD_StartRender(pass: HSD_RenderPass) {
    melee_core::ct!("HSD_StartRender");
    let rm = vi_render_mode();
    current_render_pass = pass;
    if (*rm).aa != 0 {
        ext::GXSetPixelFmt(GX_PF_RGB565_Z16 as u32, current_z_fmt as u32);
    } else {
        ext::GXSetPixelFmt(current_pix_fmt as u32, GX_ZC_LINEAR as u32);
    }
    ext::GXSetFieldMode(
        (*rm).field_rendering,
        ((*rm).xfbHeight < (*rm).viHeight) as u8,
    );
}

/// The C body is a comparison with no effect (it only forces instruction order when matching).
pub unsafe fn HSD_Init_803755A8() { melee_core::ct!("HSD_Init_803755A8");}

pub unsafe fn HSD_ObjInit() {
    melee_core::ct!("HSD_ObjInit");
    ext::HSD_ListInitAllocData();
    ext::HSD_AObjInitAllocData();
    ext::HSD_FObjInitAllocData();
    ext::HSD_IDInitAllocData();
    ext::HSD_VecInitAllocData();
    ext::HSD_MtxInitAllocData();
    ext::HSD_RObjInitAllocData();
    ext::HSD_RenderInitAllocData();
    ext::HSD_ShadowInitAllocData();
    ext::HSD_ZListInitAllocData();
}

pub unsafe fn HSD_ObjDumpStat() {
    melee_core::ct!("HSD_ObjDumpStat");
    let types: [(unsafe fn() -> *mut HSD_ObjAllocData, &core::ffi::CStr); 13] = [
        (ext::HSD_AObjGetAllocData, c"aobj"),
        (ext::HSD_FObjGetAllocData, c"fobj"),
        (ext::HSD_IDGetAllocData, c"id"),
        (ext::HSD_SListGetAllocData, c"slist"),
        (ext::HSD_DListGetAllocData, c"dlist"),
        (ext::HSD_VecGetAllocData, c"vec"),
        (ext::HSD_MtxGetAllocData, c"mtx"),
        (ext::HSD_RObjGetAllocData, c"robj"),
        (ext::HSD_RvalueObjGetAllocData, c"rval"),
        (ext::HSD_ShadowGetAllocData, c"shadow"),
        (ext::HSD_RenderGetAllocData, c"render"),
        (ext::HSD_ChanGetAllocData, c"chan"),
        (ext::HSD_TevRegGetAllocData, c"tevreg"),
    ];
    for (func, label) in types {
        let using = melee_types::inline::HSD_ObjAllocGetUsing(&*(func() as *const _));
        let freed = melee_types::inline::HSD_ObjAllocGetFreed(&*(func() as *const _));
        let peak = melee_types::inline::HSD_ObjAllocGetPeak(&*(func() as *const _));
        super::lbext::OSReport(
            "objalloc: %s\tusing %d\tfreed %d\tpeak %d\n",
            &[OsArg::S(label.as_ptr()), OsArg::I(using as i32), OsArg::I(freed as i32), OsArg::I(peak as i32)],
        );
    }
}

unsafe fn hsd_init_parameter_allowed() -> bool {
    if init_done != 0 {
        if shown == 0 {
            super::lbext::OSReport("init parameter should be set before invoking HSD_Init().\n", &[]);
            shown = 1;
        }
        return false;
    }
    true
}

pub unsafe fn HSD_SetInitParameterU32(param: HSD_InitParam, value: u32) -> bool {
    melee_core::ct!("HSD_SetInitParameterU32");
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
    melee_core::ct!("HSD_SetInitParameterPtr");
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
// BEGIN GENERATED EXT (rust/melee_runtime/gen/mtu/gen_tu.py; the callees of the C TU)
pub mod ext {
    #![allow(non_upper_case_globals, non_snake_case, unused_imports, dead_code)]
    use super::*;
    use core::ffi::c_void;
    macro_rules! ext_fns {
        ($(fn $name:ident($($a:ident: $t:ty),*) $(-> $r:ty)? $(= $def:expr)?;)*) => {
            pub mod hook {
                use super::*;
                $( pub static mut $name: Option<fn($($t),*) $(-> $r)?> = None; )*
            }
            $( pub fn $name($($a: $t),*) $(-> $r)? {
                match unsafe { hook::$name } {
                    Some(f) => f($($a),*),
                    None => crate::tu::hookmac::fallback!(stringify!($name) $(, $def)?),
                }
            } )*
        };
    }
    ext_fns! {
    fn GXInitLightAttn(a0: *mut _GXLightObj, a1: f32, a2: f32, a3: f32, a4: f32, a5: f32, a6: f32);
    fn GXInitLightColor(a0: *mut _GXLightObj, a1: _GXColor);
    fn GXInitLightDir(a0: *mut _GXLightObj, a1: f32, a2: f32, a3: f32);
    fn GXInitLightPos(a0: *mut _GXLightObj, a1: f32, a2: f32, a3: f32);
    fn GXLoadLightObjImm(a0: *mut _GXLightObj, a1: u32);
    fn GXSetFieldMode(a0: u8, a1: u8);
    fn GXSetPixelFmt(a0: u32, a1: u32);
    fn HSD_AObjGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::aobj::HSD_AObjGetAllocData();
    fn HSD_AObjInitAllocData() = crate::tu::aobj::HSD_AObjInitAllocData();
    fn HSD_ChanGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::tev::HSD_ChanGetAllocData(crate::tu::globals::render());
    fn HSD_DListGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::list::HSD_DListGetAllocData();
    fn HSD_FObjGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::fobj::HSD_FObjGetAllocData();
    fn HSD_FObjInitAllocData() = crate::tu::fobj::HSD_FObjInitAllocData();
    fn HSD_IDGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::id::HSD_IDGetAllocData();
    fn HSD_IDInitAllocData() = crate::tu::id::HSD_IDInitAllocData();
    fn HSD_IDSetup() = crate::tu::id::HSD_IDSetup();
    fn HSD_Index2LightID(a0: u32) -> i32 = crate::lobj::index_to_light_id(a0) as i32;
    fn HSD_ListInitAllocData() = crate::tu::list::HSD_ListInitAllocData();
    fn HSD_LogInit() = crate::tu::debug::HSD_LogInit();
    fn HSD_MtxGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::mtx::HSD_MtxGetAllocData();
    fn HSD_MtxInitAllocData() = crate::tu::mtx::HSD_MtxInitAllocData();
    fn HSD_ObjSetHeap(a0: u32, a1: *mut c_void) = { let (size, p) = (a0, a1); crate::tu::objalloc::HSD_ObjSetHeap(size, p) };
    fn HSD_RObjGetAllocData() -> *mut _HSD_ObjAllocData;
    fn HSD_RObjInitAllocData();
    fn HSD_RenderGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::tev::HSD_RenderGetAllocData(crate::tu::globals::render());
    fn HSD_RenderInitAllocData() = crate::tu::tev::HSD_RenderInitAllocData(crate::tu::globals::render());
    fn HSD_RvalueObjGetAllocData() -> *mut _HSD_ObjAllocData;
    fn HSD_SListGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::list::HSD_SListGetAllocData();
    fn HSD_ShadowGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::shadow::HSD_ShadowGetAllocData();
    fn HSD_ShadowInitAllocData() = crate::tu::shadow::HSD_ShadowInitAllocData();
    fn HSD_StateInvalidate(a0: i32);
    fn HSD_TevRegGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::tev::HSD_TevRegGetAllocData(crate::tu::globals::render());
    fn HSD_VIInit(a0: *mut _HSD_VIStatus, a1: *mut c_void, a2: *mut c_void, a3: *mut c_void) = { let (s, a, b, c) = (a0, a1, a2, a3); crate::tu::video::HSD_VIInit(s, a, b, c) };
    fn HSD_VecGetAllocData() -> *mut _HSD_ObjAllocData = crate::tu::mtx::HSD_VecGetAllocData();
    fn HSD_VecInitAllocData() = crate::tu::mtx::HSD_VecInitAllocData();
    fn HSD_ZListInitAllocData();
    fn OSAllocFromHeap(a0: i32, a1: u32) -> *mut c_void;
    fn OSCreateHeap(a0: *mut c_void, a1: *mut c_void) -> i32;
    fn OSDestroyHeap(a0: i32);
    fn OSGetArenaHi() -> *mut c_void;
    fn OSGetArenaLo() -> *mut c_void;
    fn OSGetPhysicalMemSize() -> u32;
    fn OSInitAlloc(a0: *mut c_void, a1: *mut c_void, a2: i32) -> *mut c_void;
    fn OSSetArenaLo(a0: *mut c_void);
    fn OSSetCurrentHeap(a0: i32) -> i32;
    fn VIWaitForRetrace();
    fn _HSD_AObjForgetMemory(a0: *mut c_void, a1: *mut c_void) = { let (lo, hi) = (a0, a1); crate::tu::aobj::_HSD_AObjForgetMemory(lo, hi) };
    fn _HSD_DispForgetMemory(a0: *mut c_void, a1: *mut c_void);
    fn _HSD_IDForgetMemory(a0: *mut c_void, a1: *mut c_void) = crate::tu::id::_HSD_IDForgetMemory(a0, a1);
    fn _HSD_ObjAllocForgetMemory(a0: *mut c_void, a1: *mut c_void) = { let (lo, hi) = (a0, a1); crate::tu::objalloc::_HSD_ObjAllocForgetMemory(lo, hi) };
    fn _HSD_RObjForgetMemory(a0: *mut c_void, a1: *mut c_void);
    fn _HSD_RandForgetMemory(a0: *mut c_void, a1: *mut c_void) = { let (lo, hi) = (a0, a1); crate::tu::random::_HSD_RandForgetMemory(crate::tu::globals::rand(), lo as *const u8, hi as *const u8) };
    fn hsdForgetClassLibrary(a0: *const i8) = { let (name,) = (a0,); crate::tu::class::hsdForgetClassLibrary(name as _) };
    }
    /// Functions of other TUs whose address this TU stores (callbacks): C-ABI shims over the hooks.
    pub mod cb {
        use super::*;
        pub unsafe extern "C" fn HSD_AObjGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_AObjGetAllocData() }
        pub unsafe extern "C" fn HSD_ChanGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_ChanGetAllocData() }
        pub unsafe extern "C" fn HSD_DListGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_DListGetAllocData() }
        pub unsafe extern "C" fn HSD_FObjGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_FObjGetAllocData() }
        pub unsafe extern "C" fn HSD_IDGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_IDGetAllocData() }
        pub unsafe extern "C" fn HSD_MtxGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_MtxGetAllocData() }
        pub unsafe extern "C" fn HSD_RObjGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_RObjGetAllocData() }
        pub unsafe extern "C" fn HSD_RenderGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_RenderGetAllocData() }
        pub unsafe extern "C" fn HSD_RvalueObjGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_RvalueObjGetAllocData() }
        pub unsafe extern "C" fn HSD_SListGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_SListGetAllocData() }
        pub unsafe extern "C" fn HSD_ShadowGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_ShadowGetAllocData() }
        pub unsafe extern "C" fn HSD_TevRegGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_TevRegGetAllocData() }
        pub unsafe extern "C" fn HSD_VecGetAllocData() -> *mut _HSD_ObjAllocData { super::HSD_VecGetAllocData() }
        pub unsafe extern "C" fn OSGetArenaHi() -> *mut c_void { super::OSGetArenaHi() }
        pub unsafe extern "C" fn OSGetArenaLo() -> *mut c_void { super::OSGetArenaLo() }
        pub unsafe extern "C" fn _HSD_AObjForgetMemory(a0: *mut c_void, a1: *mut c_void) { super::_HSD_AObjForgetMemory(a0, a1) }
        pub unsafe extern "C" fn _HSD_DispForgetMemory(a0: *mut c_void, a1: *mut c_void) { super::_HSD_DispForgetMemory(a0, a1) }
        pub unsafe extern "C" fn _HSD_IDForgetMemory(a0: *mut c_void, a1: *mut c_void) { super::_HSD_IDForgetMemory(a0, a1) }
        pub unsafe extern "C" fn _HSD_ObjAllocForgetMemory(a0: *mut c_void, a1: *mut c_void) { super::_HSD_ObjAllocForgetMemory(a0, a1) }
        pub unsafe extern "C" fn _HSD_RObjForgetMemory(a0: *mut c_void, a1: *mut c_void) { super::_HSD_RObjForgetMemory(a0, a1) }
        pub unsafe extern "C" fn _HSD_RandForgetMemory(a0: *mut c_void, a1: *mut c_void) { super::_HSD_RandForgetMemory(a0, a1) }
    }
    // Data defined in other TUs (the tests set them).
    pub static mut GXNtsc480IntDf: GXRenderModeObj = unsafe { core::mem::zeroed() };
    pub use crate::tu::synth::HSD_Synth_804D6018;
    pub use crate::tu::video::HSD_VIData;
    pub static mut __OSCurrHeap: i32 = unsafe { core::mem::zeroed() };
}
// END GENERATED EXT
