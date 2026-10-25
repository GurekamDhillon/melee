"""initialize.c spec (see lbspecs.py): HSD start-up over a fake OS arena."""
from lbspecs import add

POOLS = [("GXRenderModeObj", "RMO", 2), ("HSD_ObjAllocData", "OAD", 2)]
OWN = ["FrameBuffer", "memReport", "hsd_heap_next_arena_lo", "hsd_heap_next_arena_hi", "current_render_pass", "DefaultFifoObj", "current_pix_fmt",
       "init_done", "shown", "current_heap", "rmode", "current_z_fmt", "iparam_fifo_size", "iparam_xfb_max_num", "iparam_heap_max_num", "iparam_audio_heap_size"]
C_DECL = r'''
static unsigned char ARN[4096] __attribute__((aligned(64)));
static int AZ, LOK, HIK;
'''
RS_EXTRA = r'''
#[repr(align(64))] struct Al([u8; 4096]);
static mut ARN: Al = Al([0; 4096]);
static mut AZ: bool = false; static mut LOK: usize = 0; static mut HIK: usize = 0;
'''
SETUP_C = r'''{ int q; AZ = (fv() & 3) == 0; LOK = fv() % 16; HIK = (fv() & 3) == 0 ? LOK : 32 + fv() % 16;
 memset(&memReport, 0, sizeof memReport); for (q = 0; q < 3; q++) FrameBuffer[q] = 0; for (q = 0; q < 2; q++) RMO[q].fbWidth = (u16) (fv() % 700), RMO[q].xfbHeight = (u16) (fv() % 500), RMO[q].viHeight = (u16) (fv() % 500), RMO[q].aa = (u8) (fv() & 1), RMO[q].field_rendering = (u8) (fv() & 1);
 hsd_heap_next_arena_lo = AZ ? 0 : ARN + 32 * (fv() % 16); hsd_heap_next_arena_hi = AZ ? 0 : ARN + 1024 + 32 * (fv() % 16);
 current_render_pass = fv() % 4; DefaultFifoObj = (fv() & 1) ? (GXFifoObj*) BUF : 0; current_pix_fmt = fv() % 8; init_done = fv() & 1; shown = fv() & 1; current_heap = (int) fv() - 1;
 rmode = (fv() & 1) ? &RMO[fv() % 2] : &GXNtsc480IntDf; current_z_fmt = fv() % 3; iparam_fifo_size = fv(); iparam_xfb_max_num = (int) fv(); iparam_heap_max_num = (int) fv() % 8; iparam_audio_heap_size = 32 * (fv() % 8);
}'''
SETUP_R = r'''{ AZ = fv() & 3 == 0; LOK = (fv() % 16) as usize; HIK = if fv() & 3 == 0 { LOK } else { 32 + (fv() % 16) as usize }; m::memReport = core::mem::zeroed(); for q in 0..3usize { m::FrameBuffer[q] = null_mut(); }
 for q in 0..2usize { RMO[q].fbWidth = (fv() % 700) as u16; RMO[q].xfbHeight = (fv() % 500) as u16; RMO[q].viHeight = (fv() % 500) as u16; RMO[q].aa = (fv() & 1) as u8; RMO[q].field_rendering = (fv() & 1) as u8; }
 m::hsd_heap_next_arena_lo = if !AZ { (&raw mut ARN as *mut u8).add(32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() }; m::hsd_heap_next_arena_hi = if !AZ { (&raw mut ARN as *mut u8).add(1024 + 32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() };
 m::current_render_pass = fv() % 4; m::DefaultFifoObj = if fv() & 1 != 0 { (&raw mut BUF) as *mut GXFifoObj } else { null_mut() }; m::current_pix_fmt = (fv() % 8) as i32; m::init_done = (fv() & 1) as i32; m::shown = (fv() & 1) as i32; m::current_heap = fv() as i32 - 1;
 m::rmode = if fv() & 1 != 0 { &raw mut RMO[(fv() % 2) as usize] } else { &raw mut m::ext::GXNtsc480IntDf }; m::current_z_fmt = (fv() % 3) as i32; m::iparam_fifo_size = fv(); m::iparam_xfb_max_num = fv() as i32; m::iparam_heap_max_num = fv() as i32 % 8; m::iparam_audio_heap_size = 32 * (fv() % 8);
}'''
add("initialize", "src/sysdolphin/baselib/initialize.c", dict(
    pub_only=True, pools=POOLS, ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA, setup=(SETUP_C, SETUP_R),
    pregs=("preg(ARN, sizeof ARN); preg(&GXNtsc480IntDf, sizeof GXNtsc480IntDf);", "m::use_ext_data = true; preg(&raw const ARN, 4096); preg(&raw const m::ext::GXNtsc480IntDf, core::mem::size_of::<GXRenderModeObj>()); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"HSD_SetInitParameterU32": {0: ("(HSD_InitParam) (fv() % 6)", "(fv() % 6) as _")}, "HSD_StartRender": {0: ("(HSD_RenderPass) (fv() % 4)", "fv() % 4")},
        "HSD_AllocateXFB": {0: ("(s32) (fv() % 4)", "(fv() % 4) as i32")},
        "HSD_AllocateFifo": {0: ("fv() % 3000", "fv() % 3000")},
        "HSD_CreateMainHeap": {0: ("(!AZ && (fv() & 1)) ? (void*) (ARN + 32 * (fv() % 16)) : 0", "if !AZ && fv() & 1 != 0 { (&raw mut ARN as *mut u8).add(32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() }"),
                               1: ("(!AZ && (fv() & 1)) ? (void*) (ARN + 1024 + 32 * (fv() % 16)) : 0", "if !AZ && fv() & 1 != 0 { (&raw mut ARN as *mut u8).add(1024 + 32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() }")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_hash=OWN, skip_stubs=["memcpy", "memset"],
    patch=[("#include <dolphin/os.h>\n#include <dolphin/vi.h>\n", "#include <dolphin/os.h>\n#include <dolphin/vi.h>\n#undef OSRoundUp32B\n#undef OSRoundDown32B\n#define OSRoundUp32B(x) (((unsigned long long) (x) + 32 - 1) & ~31ULL)\n#define OSRoundDown32B(x) (((unsigned long long) (x)) & ~31ULL)\n")],
    null_ret_ok=["OSAllocFromHeap", "OSGetArenaLo", "OSGetArenaHi", "OSInitAlloc", "HSD_VIGetRenderMode"],
    ret={"OSGetArenaLo": ("(AZ ? (void*) 0 : (void*) (ARN + 32 * LOK))", "if AZ { null_mut() } else { (&raw mut ARN as *mut u8).add(32 * LOK) as *mut c_void }"),
         "OSGetArenaHi": ("(AZ ? (void*) 0 : (void*) (ARN + 32 * HIK))", "if AZ { null_mut() } else { (&raw mut ARN as *mut u8).add(32 * HIK) as *mut c_void }"),
         "OSAllocFromHeap": ("((fv() & 3) ? (void*) (ARN + 2048 + 32 * (fv() % 16)) : 0)", "if fv() & 3 != 0 { (&raw mut ARN as *mut u8).add(2048 + 32 * (fv() % 16) as usize) as *mut c_void } else { null_mut() }"),
         "OSInitAlloc": ("(AZ ? (void*) 0 : (void*) (ARN + 32 * (fv() % 16)))", "if AZ { null_mut() } else { (&raw mut ARN as *mut u8).add(32 * (fv() % 16) as usize) as *mut c_void }"),
         "HSD_VIGetRenderMode": ("&RMO[fv() % 2]", "&raw mut RMO[(fv() % 2) as usize]")})
