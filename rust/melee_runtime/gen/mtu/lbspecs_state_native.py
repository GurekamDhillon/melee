"""state.c spec: the cached GX state, the channel mode per render mode, PE descriptors, the invalidate table.
The lights (HSD_LObj) come from the light-list stubs; tev.c entry points are stubs (traced)."""
from lbspecs import add

POOLS = [("HSD_LObj", "L", 3), ("HSD_PEDesc", "PE", 2)]
STATE = ["state_line_width", "state_line_tex_offsets", "state_point_size", "HSD_State_804D75BC", "state_cull_mode", "state_blend_type", "state_src_factor",
         "state_dst_factor", "state_logic_op", "state_z_enable", "state_z_func", "state_z_update", "state_alpha_comp0", "state_alpha_ref0", "state_alpha_op",
         "state_alpha_comp1", "state_alpha_ref1", "state_color_update", "state_alpha_update", "state_enable_dst_alpha", "state_dst_alpha", "state_before_tex", "state_dither"]
CHANS = ["HSD_State_80405A38", "HSD_State_80405A38_30", "HSD_State_80405A38_60", "HSD_State_80405A38_90", "HSD_State_80405A38_C0", "HSD_State_80405A38_F0"]
C_DECL = r'''
static GXColor pick_col(void) { GXColor c; c.r = (u8) fv(); c.g = (u8) fv(); c.b = (u8) fv(); c.a = (u8) fv(); return c; }
'''
RS_EXTRA = r'''
fn pick_col() -> GXColor { let r = fv() as u8; let g = fv() as u8; let b = fv() as u8; let a = fv() as u8; GXColor { r, g, b, a } }
'''
COL = ("pick_col()", "pick_col()")
add("state_native", "src/sysdolphin/baselib/state.c", dict(
    pub_only=True, pools=POOLS, ncases=300, c_decl=C_DECL, rs_extra=RS_EXTRA, ov={"HSD_SetMaterialColor": {0: COL, 1: COL, 2: COL}},
    pregs=("", "melee_runtime::tu::lbext::H_OSReport = Some(|_f, _a| ());")),
    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS], own_fill=STATE, own_hash=STATE + CHANS + ["matstate", "dark_matter"],
    skip_stubs=["memcpy", "memset"],
    ret={"HSD_LObjGetActiveByIndex": ("picko_HSD_LObj()", "picko_HSD_LObj()"), "HSD_LObjGetActiveByID": ("picko_HSD_LObj()", "picko_HSD_LObj()"),
         "HSD_LObjGetNbActive": ("(int) (fv() % 4)", "(fv() % 4) as i32"), "HSD_StateGetNumTevStages": ("(int) (fv() % 3)", "(fv() % 3) as i32"),
         "HSD_StateAssignTev": ("(int) (fv() % 8)", "(fv() % 8) as i32")})
