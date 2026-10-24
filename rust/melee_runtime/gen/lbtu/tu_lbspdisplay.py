"""Whole-TU display oracle: generated native layouts, object trees and GX call transcripts."""
SPEC = dict(
    src="src/melee/lb/lbspdisplay.c",
    types=["HSD_JObj", "HSD_DObj", "HSD_MObj", "HSD_ImageDesc", "GXTexObj", "HSD_GObj",
           "HSD_CObj", "CameraBlurData", "LightList", "HSD_LObj", "HSD_LightDesc", "HSD_Chan", "HSD_CameraDescPerspective"],
    # Use the host va_arg implementation, not MSL's PPC ABI layout.
    args=["-I."],
    ret={
        "HSD_LObjLoadDesc": ("&LO[a0 - LD]", "&raw mut crate::LO[a0.offset_from(crate::LD.as_ptr()) as usize]"),
        "HSD_MemAlloc": ("a0 == sizeof(CameraBlurData) ? (void*) BD : (void*) MEM", "if a0 as usize == core::mem::size_of::<CameraBlurData>() { crate::BD.as_mut_ptr() as *mut _ } else { crate::MEM.as_mut_ptr() as *mut _ }"),
        "lbDvd_GetPreloadedArchive": ("(fv() & 1) ? MEM : 0", "if fv() & 1 != 0 { crate::MEM.as_mut_ptr() as *mut _ } else { core::ptr::null_mut() }"),
        "GObj_Create": ("GO", "crate::GO.as_mut_ptr()"),
        "HSD_CObjAlloc": ("CO", "crate::CO.as_mut_ptr()"),
        "HSD_CObjLoadDesc": ("CO", "crate::CO.as_mut_ptr()"),
        "HSD_CObjGetCurrent": ("CO", "crate::CO.as_mut_ptr()"),
        "HSD_CObjGetProjectionType": ("(fv() % 3) + 1", "((fv() % 3) + 1) as u32"),
        "HSD_CObjGetAuthoredAspect": ("(fv() & 1) ? 1.18f : rf()", "if fv() & 1 != 0 { 1.18 } else { rf() }"),
        "GXGetTexObjWidth": ("a0->dummy[0]", "(*a0).dummy[0] as u16"),
        "GXGetTexObjHeight": ("a0->dummy[1]", "(*a0).dummy[1] as u16"),
    },
    effects={
        "GXInitTexObj": ("a0->dummy[0] = a2; a0->dummy[1] = a3; a0->dummy[2] = a4;", "(*a0).dummy[0] = a2 as u32; (*a0).dummy[1] = a3 as u32; (*a0).dummy[2] = a4;"),
        "HSD_GObjObject_80390A70": ("a0->hsd_obj = a2;", "(*a0).hsd_obj = a2;"),
        "GObj_InitUserData": ("a0->user_data = a3; a0->user_data_remove_func = a2;", "(*a0).user_data = a3; (*a0).user_data_remove_func = a2;"),
        "HSD_CObjGetViewingMtx": ("{ int i,j; for(i=0;i<3;i++) for(j=0;j<4;j++) a1[i][j]=rf(); }", "for i in 0..3 { for j in 0..4 { (*a1.add(i))[j] = rf(); } }"),
        "GXLoadPosMtxImm": ("{ int i,j; for(i=0;i<3;i++) for(j=0;j<4;j++) T(B(a0[i][j])); }", "for i in 0..3 { for j in 0..4 { t((*a0.add(i))[j].to_bits()); } }"),
        "HSD_CObjGetScissor": ("a1->left=fv(); a1->right=fv()%1000; a1->top=fv(); a1->bottom=fv()%800;", "(*a1).left=fv() as u16; (*a1).right=(fv()%1000) as u16; (*a1).top=fv() as u16; (*a1).bottom=(fv()%800) as u16;"),
        "HSD_CObjSetScissor": ("T(a1->left); T(a1->right); T(a1->top); T(a1->bottom); a0->scissor = *a1;", "t((*a1).left as u32); t((*a1).right as u32); t((*a1).top as u32); t((*a1).bottom as u32); (*a0).scissor = *a1;"),
        "HSD_CObjSetViewport": ("T(a1->xmin); T(a1->xmax); T(a1->ymin); T(a1->ymax);", "t((*a1).xmin as u32); t((*a1).xmax as u32); t((*a1).ymin as u32); t((*a1).ymax as u32);"),
        "HSD_SetupChannel": ("T(a0->chan); T(a0->flags); T(RH(&a0->amb_color, 4)); T(RH(&a0->mat_color, 4)); T(a0->enable); T(a0->amb_src); T(a0->mat_src); T(a0->light_mask); T(a0->diff_fn); T(a0->attn_fn);", "t((*a0).chan); t((*a0).flags); t(rh(&(*a0).amb_color)); t(rh(&(*a0).mat_color)); t((*a0).enable as u32); t((*a0).amb_src); t((*a0).mat_src); t((*a0).light_mask); t((*a0).diff_fn); t((*a0).attn_fn);"),
    },
)

_P = lambda n: (f"&{n}[0]", f"&raw mut {n}[0]")
CFG = dict(
    world=[("HSD_JObj", "JO", 9), ("HSD_DObj", "DO", 9), ("HSD_MObj", "MO", 9),
           ("HSD_ImageDesc", "IMG", 1), ("GXTexObj", "TEX", 1), ("HSD_GObj", "GO", 1),
           ("HSD_CObj", "CO", 1), ("CameraBlurData", "BD", 1), ("LightList", "LL", 4),
           ("HSD_LObj", "LO", 4), ("HSD_LightDesc", "LD", 4), ("HSD_CameraDescPerspective", "CD", 1),
           ("unsigned char", "MEM", 256), ("HSD_JObj*", "OUT", 6), ("u16", "IX", 6), ("LightList*", "LS", 5)],
    rtype={"unsigned char": "u8", "HSD_JObj*": "*mut HSD_JObj", "LightList*": "*mut LightList"},
    c_decl="typedef struct CameraBlurData CameraBlurData;\nstatic HSD_LightAnim* AN[1];\nstatic void evt(HSD_GObj* p) { T(9001); p->p_priority ^= 7; }\nstatic void draw(HSD_GObj* p, int pass) { T(9002); T(pass); }",
    rs_decl="unsafe extern \"C\" fn evt(p: *mut HSD_GObj) { t(9001); (*p).p_priority ^= 7; }\nunsafe extern \"C\" fn draw(_: *mut HSD_GObj, pass: i32) { t(9002); t(pass as u32); }",
    argmap={"HSD_JObj": [_P("JO")], "HSD_DObj": [_P("DO")], "HSD_ImageDesc": [_P("IMG")],
            "_GXTexObj": [_P("TEX")], "HSD_GObj": [_P("GO")], "HSD_CameraDescPerspective": [_P("CD")],
            "void": [("MEM", "MEM.as_mut_ptr() as *mut core::ffi::c_void")], "struct LightList *": [("LS", "LS.as_mut_ptr()")],
            "struct HSD_JObj *": [("OUT", "OUT.as_mut_ptr()")], "u16": [("IX", "IX.as_mut_ptr()")], "unsigned short": [("IX", "IX.as_mut_ptr()")],
    },
    hints={"count": ("(int)(fv()%7)-1", "(fv()%7) as i32 - 1"),
           "blur_callback": ("(fv()&1)?evt:0", "if fv()&1 != 0 { Some(evt) } else { None }"),
           "render_func": ("(fv()&1)?draw:0", "if fv()&1 != 0 { Some(draw) } else { None }")},
    # C's non-void function falls off its end; callers ignore the result. Compare its complete effects.
    noret=["lb_800138EC"],
    pre={"lb_800121FC": ("IMG[0].image_ptr=0;", "IMG[0].image_ptr=null_mut();")},
    over={
        "checkJObjFlags": ("ret=checkJObjFlags(&JO[0]);", "ret=m::checkJObjFlags(&raw mut JO[0]) as u32;"),
        "setTevAlpha": ("setTevAlpha((u8)fv());", "m::setTevAlpha(fv() as u8);"),
        "lb_80011E24": ("ret=lb_80011E24(JO, OUT, (int)IX[0], (int)IX[1], (int)IX[2], (int)IX[3], (int)IX[4], (int)IX[5], -1);",
                        "ret=m::lb_80011E24(JO.as_mut_ptr(),OUT.as_mut_ptr(),&[IX[0] as i32,IX[1] as i32,IX[2] as i32,IX[3] as i32,IX[4] as i32,IX[5] as i32,-1]) as u32;"),
    },
    setup=(
"""    int n = rnd()%5;
    for(k=0;k<4;k++) { LL[k].desc=&LD[k]; LL[k].anims=(rnd()&1)?AN:0; LS[k]=k<n?&LL[k]:0; }
    for(k=0;k<9;k++) { JO[k].flags &= ~((1u<<5)|(1u<<14)|(1u<<12)); JO[k].flags |= (rnd()%8==0)?(1u<<12):0; JO[k].u.dobj=(rnd()&1)?&DO[k]:0; DO[k].mobj=&MO[k]; DO[k].next=(k<8 && (rnd()&1))?&DO[k+1]:0; }
    JO[0].child=&JO[1]; JO[1].parent=JO[2].parent=JO[3].parent=JO;
    JO[1].next=&JO[2]; JO[2].next=&JO[3]; JO[1].child=&JO[4]; JO[4].next=&JO[5]; JO[4].parent=JO[5].parent=&JO[1];
    JO[2].child=&JO[6]; JO[6].parent=&JO[2]; JO[6].child=&JO[7]; JO[7].parent=&JO[6]; JO[0].next=&JO[8];
    for(k=0;k<6;k++) IX[k]=rnd()%14;
    IMG[0].width=(rnd()%641); IMG[0].height=(rnd()%481); IMG[0].image_ptr=MEM;
    TEX[0].dummy[0]=IMG[0].width; TEX[0].dummy[1]=IMG[0].height;
    BD[0].mode=rnd()%3; BD[0].efb_copy=IMG; BD[0].callback=(rnd()&1)?evt:0;
    GO[0].user_data=BD; GO[0].hsd_obj=CO;
""",
"""    let n=rnd()%5;
    for k in 0..4 { LL[k].desc=&raw mut LD[k]; LL[k].anims=if rnd()&1 !=0 { &raw mut AN as *mut _ } else { null_mut() }; LS[k]=if (k as u32)<n { &raw mut LL[k] } else { null_mut() }; }
    for k in 0..9 { JO[k].flags &= !((1u32<<5)|(1u32<<14)|(1u32<<12)); JO[k].flags |= if rnd()%8==0 {1u32<<12} else {0}; JO[k].u.dobj=if rnd()&1!=0 { &raw mut DO[k] } else { null_mut() }; DO[k].mobj=&raw mut MO[k]; DO[k].next=if k<8 && rnd()&1!=0 { &raw mut DO[k+1] } else { null_mut() }; }
    JO[0].child=&raw mut JO[1]; JO[1].parent=&raw mut JO[0]; JO[2].parent=&raw mut JO[0]; JO[3].parent=&raw mut JO[0];
    JO[1].next=&raw mut JO[2]; JO[2].next=&raw mut JO[3]; JO[1].child=&raw mut JO[4]; JO[4].next=&raw mut JO[5]; JO[4].parent=&raw mut JO[1]; JO[5].parent=&raw mut JO[1];
    JO[2].child=&raw mut JO[6]; JO[6].parent=&raw mut JO[2]; JO[6].child=&raw mut JO[7]; JO[7].parent=&raw mut JO[6]; JO[0].next=&raw mut JO[8];
    for k in 0..6 { IX[k]=(rnd()%14) as u16; }
    IMG[0].width=(rnd()%641) as u16; IMG[0].height=(rnd()%481) as u16; IMG[0].image_ptr=MEM.as_mut_ptr() as *mut _;
    TEX[0].dummy[0]=IMG[0].width as u32; TEX[0].dummy[1]=IMG[0].height as u32;
    BD[0].mode=(rnd()%3) as u8; BD[0].efb_copy=&raw mut IMG[0]; BD[0].callback=if rnd()&1!=0 { Some(evt) } else { None };
    GO[0].user_data=&raw mut BD[0] as *mut _; GO[0].hsd_obj=&raw mut CO[0] as *mut _;
"""),
    ptrs=("for(k=0;k<9;k++) { HM(PI(JO[k].u.dobj)); JO[k].u.dobj=0; }", "for k in 0..9 { hm(pi(JO[k].u.dobj)); JO[k].u.dobj=null_mut(); }"),
    hash=("for(k=0;k<6;k++) HM(PI(OUT[k]));", "for k in 0..6 { hm(pi(OUT[k])); }"),
    main_init=("freg(evt); freg(draw);", "freg(evt as *const () as usize); freg(draw as *const () as usize);"),
)
# The pointer-array has a stable null element, used to exercise non-null animation arrays.
CFG["rs_decl"] += "\nstatic mut AN: [*mut HSD_LightAnim;1] = [core::ptr::null_mut()];"
# Names declared only with `struct` need aliases before world declarations.
SPEC["patch"] = [("#include \"lbspdisplay.h\"", "#include \"lbspdisplay.h\"\ntypedef struct CameraBlurData CameraBlurData;\n"),
                 ("#include <stdarg.h>", '#include "tools/rust-port/include/stdarg.h"')]
