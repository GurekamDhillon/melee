"""lbrefract.c spec: the distortion texture fill lbRefract_80021CE8 (writer callbacks, fused sqrt / fmodf math), the IA4 texel writer, and the
refract PObj load (display list rewrite)."""
from lbspecs import add

POOLS = [('lbRefract_CallbackData', 'CB', 2), ('HSD_PObj', 'P1', 2), ('HSD_PObjDesc', 'P2', 2), ('HSD_VtxDescList', 'VD', 6), ('HSD_PObjInfo', 'PI_', 1)]
ONLY = ["lbRefract_80021CE8", "lbRefract_WriteTexCoordIA4", "lbRefract_PObjLoad"]
C_DECL = r'''
static unsigned char RFB[4096];
HSD_Archive* lbArchive_LoadSymbols(const char* f, void* out, ...) { (void) f; (void) out; return 0; }
float fabsf(float x) { union { float f; u32 u; } c; c.f = x; c.u &= 0x7FFFFFFFu; return c.f; }
static struct { u8 x0; f32* x4; } RD[1];
static int cb_load(HSD_PObj* p, HSD_PObjDesc* d) { T(8208); T(PI(p)); T(PI(d)); return (fv() & 3) ? 0 : (int) (fv() % 3) - 1; }
'''
RS_EXTRA = r'''
static mut RFB: [u8; 4096] = [0; 4096];
static mut RD: [m::RefractData; 1] = [m::RefractData { x0: 0, x4: null_mut() }];
unsafe extern "C" fn cb_load(p: *mut HSD_PObj, d: *mut HSD_PObjDesc) -> i32 { t(8208); t(pi(p)); t(pi(d)); if fv() & 3 != 0 { 0 } else { (fv() % 3) as i32 - 1 } }
'''
_SETUP_C = ("{ int q; for (q = 0; q < 2; q++) { CB[q].buffer = RFB; CB[q].width = 1 + fv() % 32; CB[q].height = 1 + fv() % 32; CB[q].row_stride = 256; CB[q].buffer_size = 2048; CB[q].format = 3; "
            "CB[q].callback0 = (void*) lbRefract_WriteTexCoordIA4; CB[q].callback1 = 0; } for (q = 0; q < 4096; q++) RFB[q] = (unsigned char) rnd(); RD[0].x4 = FA; refract_data = (void*) &RD[0]; "
            "for (q = 0; q < 6; q++) { VD[q].attr = (fv() & 3) ? (fv() % 26) : 255; VD[q].attr_type = fv() % 4; VD[q].comp_type = fv() % 6; VD[q].comp_cnt = fv() % 2; } VD[5].attr = 255;"
            " for (q = 0; q < 2; q++) { P1[q].verts = (fv() & 3) ? &VD[fv() % 6] : 0; P1[q].display = RFB; P1[q].n_display = fv() % 4; P1[q].flags = (fv() & 1) ? 0x2000 : (fv() & 0xFFF); P1[q].parent.class_info = (HSD_ClassInfo*) &PI_[0]; }"
            " PI_[0].load = cb_load; hsdPObj.load = cb_load; }")
_SETUP_R = ("{ for q in 0..2usize { CB[q].buffer = (&raw mut RFB) as *mut c_void; CB[q].width = 1 + fv() % 32; CB[q].height = 1 + fv() % 32; CB[q].row_stride = 256; CB[q].buffer_size = 2048; CB[q].format = 3; "
            "CB[q].callback0 = (m::lbRefract_WriteTexCoordIA4 as unsafe extern \"C\" fn(*mut lbRefract_CallbackData, u32, u32, u32, u32, u32, u32)) as usize as *mut c_void; CB[q].callback1 = null_mut(); } for q in 0..4096usize { RFB[q] = rnd() as u8; } RD[0].x4 = (&raw mut FA) as *mut f32; m::refract_data = &raw mut RD[0]; "
            "for q in 0..6usize { VD[q].attr = if fv() & 3 != 0 { fv() % 26 } else { 255 }; VD[q].attr_type = fv() % 4; VD[q].comp_type = fv() % 6; VD[q].comp_cnt = fv() % 2; } VD[5].attr = 255;"
            " for q in 0..2usize { P1[q].verts = if fv() & 3 != 0 { &raw mut VD[(fv() % 6) as usize] } else { null_mut() }; P1[q].display = (&raw mut RFB) as *mut u8; P1[q].n_display = (fv() % 4) as u16; P1[q].flags = if fv() & 1 != 0 { 0x2000 } else { (fv() & 0xFFF) as u16 }; P1[q].parent.class_info = (&raw mut PI_[0]) as *mut HSD_ClassInfo; }"
            " PI_[0].load = Some(cb_load); m::ext::hsdPObj.load = Some(cb_load); }")
add("lbrefract", "src/melee/lb/lbrefract.c", dict(
    pub_only=True, pools=POOLS, only=ONLY, setup=(_SETUP_C, _SETUP_R), ncases=400, c_decl=C_DECL, rs_extra=RS_EXTRA,
    hash=("{ int i; for (i = 0; i < 4096; i++) HM(RFB[i]); }", "{ for i in 0..4096usize { hm(RFB[i] as u32); } }"),
    pregs=("preg(RFB, sizeof RFB); preg(RD, sizeof RD); freg((const void*) cb_load); freg((const void*) lbRefract_WriteTexCoordIA4);",
           "preg(&raw const RFB, 4096); preg(&raw const RD, 16); m::use_ext_data = true; freg(cb_load as usize); melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());"),
    ov={"lbRefract_80021CE8": {0: ("&CB[fv() & 1]", "&raw mut CB[(fv() & 1) as usize]"), 1: ("(s32) (fv() & 1)", "(fv() & 1) as i32")},
        "lbRefract_WriteTexCoordIA4": {0: ("&CB[fv() & 1]", "&raw mut CB[(fv() & 1) as usize]"), 1: ("rnd() % 32", "rnd() % 32"), 2: ("rnd() % 32", "rnd() % 32")}}),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=[], own_hash=["lbl_804336D0"], own_only=ONLY,
    skip_stubs=["memcpy", "memset", "sqrtf", "__frsqrte", "fabsf"],
    ret={})
