"""Oracle spec + driver of src/melee/gr/grgreens.c (see ../gen_tu.py, ../mk_tu.py)."""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from drvlib import *

V3 = ("{a}->x = rw(); {a}->y = rw(); {a}->z = rw();", "(*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw();")
V3T = ("T(B({a}->x)); T(B({a}->y)); T(B({a}->z));", "t((*{a}).x.to_bits()); t((*{a}).y.to_bits()); t((*{a}).z.to_bits());")

SPEC = dict(
    src="src/melee/gr/grgreens.c",
    types=["HSD_GObj", "Ground", "HSD_JObj", "StageCallbacks", "StageData", "CollData", "Vec", "grGreens_YakumonoParam",
           "grGreens_BlockVars"],
    # views of GroundVars: 0 = the map's block grid (greens), 1 = Whispy's state (greens2)
    unions={"GroundVars": ["greens", "greens2"]},
    # x0_flags is a union with bitfields (melee_types keeps them in GameCube bit order): b0 is compared by value
    skip=(".greens.x0_flags",),
    null_ret_ok=["HSD_GObj_SetupProc", "grZakoGenerator_801CAE04", "grLib_801C9A10", "it_802EE200", "lb_80011A50", "efSync_Spawn"],
    custom_stubs={
        "efSync_Spawn": dict(
            c="void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...) { __builtin_va_list ap; Vec* v; float* f = 0; __builtin_va_start(ap, a1); "
              "v = __builtin_va_arg(ap, Vec*); if (a0 == 1032) f = __builtin_va_arg(ap, float*); __builtin_va_end(ap); "
              "T(@ID@); T((u32) a0); T(PI(a1)); T(PI(v)); T(B(v->x)); T(B(v->y)); T(B(v->z)); if (f) T(B(*f)); return 0; }",
            ext="fn efSync_Spawn(a0: i32, a1: *mut HSD_GObj, a2: *mut Vec, a3: *mut f32) -> *mut c_void;",
            hook="|a0, a1, a2, a3| { t(@ID@); t(a0 as u32); t(pi(a1)); t(pi(a2)); unsafe { t((*a2).x.to_bits()); t((*a2).y.to_bits()); "
                 "t((*a2).z.to_bits()); if a0 == 1032 { t((*a3).to_bits()); } } core::ptr::null_mut() }"),
    },
    ret={
        "Ground_801C2CF4": ("&JO[a0 & 7]", "&raw mut super::JO[(a0 & 7) as usize]"),
        "Ground_801C3FA4": ("&JO[a1 % 24]", "&raw mut super::JO[(a1 % 24) as usize]"),
        "Ground_GetMapGObj": ("&GS[a0 % 7]", "&raw mut super::GS[(a0 % 7) as usize]"),
        "Ground_GetStageGObj": ("((NOK && a0 == NID && (fv() & 7) == 0) ? 0 : &GS[a0 % 7])",
                                "if super::NOK && a0 == super::NID && fv() & 7 == 0 { core::ptr::null_mut() } else { &raw mut super::GS[(a0 % 7) as usize] }"),
        "Ground_GetYakumonoParam": ("&YP", "&raw mut super::YP as *mut c_void"),
        "HSD_MemAlloc": ("(a0 == 360 ? (void*) VB : (void*) BB)", "if a0 == 360 { &raw mut super::VB as *mut c_void } else { &raw mut super::BB as *mut c_void }"),
        "grMaterial_801C8CFC": ("&IG[fv() % 30]", "&raw mut super::IG[(fv() % 30) as usize]"),
        "Stage_GetBlastZoneTopOffset": ("rw()", "rw()"),
    },
    effects={
        "ftLib_GetPos": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "lb_8000B1CC": (V3[0].format(a="a2"), V3[1].format(a="a2")),
        "grLib_801C9A70": (V3[0].format(a="a1"), V3[1].format(a="a1")),
        "it_802EE200": (V3T[0].format(a="a1"), V3T[1].format(a="a1")),
        "lb_80011A50": (V3T[0].format(a="a0"), V3T[1].format(a="a0")),
        "memzero": ("zero(a0, (unsigned) a1);", "core::ptr::write_bytes(a0 as *mut u8, 0, a1 as usize);"),
    })

VIEWC = "(i == 5 ? OPUSEL[op] : 0)"
VIEWR = "(if i == 5 { OPUSEL[op as usize] } else { 0 })"

SETUP_C = """
            for (i = 0; i < 24; i++) JO[i].child = &JO[(i + 1) % 24];
            for (i = 0; i < 30; i++) IG[i].hsd_obj = &JO[i % 24];
            for (i = 0; i < 7; i++) {
                GS[i].user_data = &GG[i]; GS[i].hsd_obj = &JO[8 + i];
                zero(&GG[i], sizeof GG[i]);
                USEL = @VIEW@;
                fill_Ground(&GG[i]);
                GG[i].gobj = &GS[i];
                GG[i].x8_callback = grGreens_802138B4; GG[i].xC_callback = grGreens_802138B8;
            }
            USEL = OPUSEL[op];
            yakumono_param = &YP;
            YP.x0_blockTimerMin = (int) (fv() % 20); YP.x4_blockTimerMax = (int) (fv() % 20); YP.x8_blockBombChance = 0;
            YP.xC = (int) (fv() % 6); YP.x10 = (int) (fv() % 6); YP.x14 = (int) (fv() % 6); YP.x18 = (int) (fv() % 6); YP.x1C = (int) (fv() % 6);
            YP.x20 = (int) (fv() % 4); YP.x24 = (int) (fv() % 100); YP.x28 = (int) (fv() % 100);
            YP.x34_windTimerMin = (int) (fv() % 30); YP.x38_windTimerMax = (int) (fv() % 30);
            YP.x5C = (int) (fv() % 6); YP.x60 = (int) (fv() % 6); YP.x64 = (int) (fv() % 4); YP.x68 = 1 + (int) (fv() % 5);
            YP.x50 = (float) (fv() % 4); YP.x54 = (float) (fv() % 6); YP.x58 = (float) (fv() % 4);
            YP.x2C = rw(); YP.x30 = rw(); YP.x6C = rw(); YP.x70 = rw();
            /* the map: a stacked grid (a column's status is non-zero up to its height), pointers into the pools */
            GG[6].u.greens.x4 = VB; GG[6].u.greens.x8_blocks = (struct grGreens_BlockVars (*)[6]) BB;
            GG[6].u.greens.xC = (int) (fv() % 6) - 2;
            for (j = 0; j < 6; j++) {
                int h = (int) (fv() % 6);
                for (i = 0; i < 5; i++) {
                    struct grGreens_BlockVars* b = &BB[i * 6 + j];
                    b->status = (i < h) ? 1 + fv() % 3 : 0; b->index = i * 6 + j;
                    b->xC = &GS[(i + j) % 7]; b->x10 = &IG[i * 6 + j]; b->x14 = &JO[(i * 6 + j) % 24];
                    b->x18 = (int) (fv() % 64); b->x1C = (fv() & 1) ? &GS[(i * 3 + j) % 7] : 0;
                }
            }
            for (i = 0; i < 30; i++) VB[i].z = 0.0f;
            GG[6].u.greens.x0_flags.b0 = fv() & 1;
            for (i = 0; i < 0x20; i++) grGr_8049F9E0[i] = (u8) (fv() % 3);
            grGr_804D6AAC = (u8) (fv() & 1); grGr_804D6AAD = (u8) (fv() & 1);
            NOK = 0; NID = 99;
"""

SETUP_R = """
                for i in 0..24usize { JO[i].child = &raw mut JO[(i + 1) % 24]; }
                for i in 0..30usize { IG[i].hsd_obj = &raw mut JO[i % 24] as *mut c_void; }
                for i in 0..7usize {
                    GS[i].user_data = &raw mut GG[i] as *mut c_void; GS[i].hsd_obj = &raw mut JO[8 + i] as *mut c_void;
                    GG[i] = core::mem::zeroed();
                    USEL = @VIEW@;
                    g::fill_Ground(&mut GG[i]);
                    GG[i].gobj = &raw mut GS[i];
                    GG[i].x8_callback = Some(m::grGreens_802138B4); GG[i].xC_callback = Some(m::grGreens_802138B8);
                }
                USEL = OPUSEL[op as usize];
                m::yakumono_param = &raw mut YP;
                YP.x0_blockTimerMin = (fv() % 20) as i32; YP.x4_blockTimerMax = (fv() % 20) as i32; YP.x8_blockBombChance = 0;
                YP.xC = (fv() % 6) as i32; YP.x10 = (fv() % 6) as i32; YP.x14 = (fv() % 6) as i32; YP.x18 = (fv() % 6) as i32; YP.x1C = (fv() % 6) as i32;
                YP.x20 = (fv() % 4) as i32; YP.x24 = (fv() % 100) as i32; YP.x28 = (fv() % 100) as i32;
                YP.x34_windTimerMin = (fv() % 30) as i32; YP.x38_windTimerMax = (fv() % 30) as i32;
                YP.x5C = (fv() % 6) as i32; YP.x60 = (fv() % 6) as i32; YP.x64 = (fv() % 4) as i32; YP.x68 = 1 + (fv() % 5) as i32;
                YP.x50 = (fv() % 4) as f32; YP.x54 = (fv() % 6) as f32; YP.x58 = (fv() % 4) as f32;
                YP.x2C = rw(); YP.x30 = rw(); YP.x6C = rw(); YP.x70 = rw();
                GG[6].u.greens.x4 = VB.as_mut_ptr(); GG[6].u.greens.x8_blocks = BB.as_mut_ptr() as *mut [grGreens_BlockVars; 6];
                GG[6].u.greens.xC = (fv() % 6) as i32 - 2;
                for j in 0..6usize {
                    let h = (fv() % 6) as usize;
                    for i in 0..5usize {
                        let b = &mut BB[i * 6 + j];
                        b.set_status(if i < h { 1 + fv() % 3 } else { 0 }); b.set_index((i * 6 + j) as u32);
                        b.xC = &raw mut GS[(i + j) % 7]; b.x10 = &raw mut IG[i * 6 + j]; b.x14 = &raw mut JO[(i * 6 + j) % 24];
                        b.x18 = (fv() % 64) as i32; b.x1C = if fv() & 1 != 0 { &raw mut GS[(i * 3 + j) % 7] } else { core::ptr::null_mut() };
                    }
                }
                for i in 0..30usize { VB[i].z = 0.0; }
                GG[6].u.greens.x0_flags.anon_0.set_b0((fv() & 1) as u8);
                for i in 0..0x20usize { m::grGr_8049F9E0[i] = (fv() % 3) as u8; }
                m::grGr_804D6AAC = (fv() & 1) as u8; m::grGr_804D6AAD = (fv() & 1) as u8;
                NOK = false; NID = 99;
"""

POST_C = """
            for (i = 0; i < 7; i++) { USEL = @VIEW@; hash_Ground(&GG[i]); }
            USEL = OPUSEL[op];
            HM((u32) GG[6].u.greens.x0_flags.b0);
            HM(PI(yakumono_param)); HM(grGr_804D6AAC); HM(grGr_804D6AAD); HM(RH(grGr_8049F9E0, 0x20));
"""

POST_R = """
                for i in 0..7usize { USEL = @VIEW@; g::hash_Ground(&GG[i]); }
                USEL = OPUSEL[op as usize];
                hm(GG[6].u.greens.x0_flags.anon_0.b0() as u32);
                hm(pi(m::yakumono_param)); hm(m::grGr_804D6AAC as u32); hm(m::grGr_804D6AAD as u32); hm(rh(&m::grGr_8049F9E0));
"""


def gs(name, k, usel=0, pre_c=None, pre_r=None):
    d = vg(name, usel, f"GS[{k}]")
    if pre_c:
        d["pre_c"], d["pre_r"] = pre_c, pre_r
    return d


def pgs(name, k):
    return pg(name, 0, f"GS[{k}]")


# a block of the grid by (row, col) chosen by the driver
RC_C = "int row = (int) (fv() % 5); int col = (int) (fv() % 6);"
RC_R = "let row = (fv() % 5) as i32; let col = (fv() % 6) as i32;"

# whispy: indices into the pattern tables are kept in range, the state is one of the three
WHISPY_PRE_C = ("GG[5].u.greens2.x0 = (int) (fv() % 10); GG[5].u.greens2.x4 = (int) (fv() % 3); GG[5].u.greens2.x8 = (int) (fv() % 7); "
                "GG[5].u.greens2.x10 = (int) (fv() & 1); GG[5].u.greens2.xC = (int) (fv() % 10) - 2; GG[5].u.greens2.x14 = (int) (fv() & 1); "
                "GG[5].u.greens2.x18 = (int) (fv() % 3); GG[5].u.greens2.x20 = (int) (fv() % 5) - 1; GG[5].u.greens2.x24 = (int) (fv() & 1);")
WHISPY_PRE_R = ("GG[5].u.greens2.x0 = (fv() % 10) as i32; GG[5].u.greens2.x4 = (fv() % 3) as i32; GG[5].u.greens2.x8 = (fv() % 7) as i32; "
                "GG[5].u.greens2.x10 = (fv() & 1) as i32; GG[5].u.greens2.xC = (fv() % 10) as i32 - 2; GG[5].u.greens2.x14 = (fv() & 1) as i32; "
                "GG[5].u.greens2.x18 = (fv() % 3) as i32; GG[5].u.greens2.x20 = (fv() % 5) as i32 - 1; GG[5].u.greens2.x24 = (fv() & 1) as i32;")

DRV = dict(
    state=[("GS", "HSD_GObj", 7), ("JO", "HSD_JObj", 24), ("IG", "HSD_GObj", 30), ("YP", "struct grGreens_YakumonoParam", 1),
           ("VB", "Vec", 30), ("BB", "struct grGreens_BlockVars", 30), ("CD", "CollData", 1), ("VW", "Vec", 2)],
    decl_c="static Ground GG[7]; static int NOK; static int NID;",
    decl_r=chr(10).join(["use m::grGreens_YakumonoParam;", "static mut GG: [Ground; 7] = unsafe { core::mem::zeroed() };",
                         "static mut NOK: bool = false;", "static mut NID: i32 = 0;"]),
    regs=[("GG", "sizeof GG", "&raw const GG", "core::mem::size_of_val(&GG)")],
    setup_c=SETUP_C.replace("@VIEW@", VIEWC),
    setup_r=SETUP_R.replace("@VIEW@", VIEWR),
    post_c=POST_C.replace("@VIEW@", VIEWC),
    post_r=POST_R.replace("@VIEW@", VIEWR),
    ops=[
        dict(name="grGreens_80213458", c="grGreens_80213458((bool) fv());", r="m::grGreens_80213458(fv() as i32);"),
        dict(name="grGreens_Init", pre_c="yakumono_param = 0;", pre_r="m::yakumono_param = core::ptr::null_mut();", c="grGreens_Init();", r="m::grGreens_Init();"),
        v0("grGreens_802134F4"), v0("grGreens_802134F8"), p0("grGreens_8021351C"),
        dict(name="grGreens_80213524", pre_c="int a = (int) (fv() % 7); NOK = 1; NID = a;", pre_r="let a = (fv() % 7) as i32; NOK = true; NID = a;",
             c="ret = PI(grGreens_80213524(a));", r="ret = pi(m::grGreens_80213524(a));"),
        gs("grGreens_8021360C", 0), pgs("grGreens_802138AC", 0), gs("grGreens_802138B4", 0), gs("grGreens_802138B8", 0),
        gs("grGreens_802138BC", 2), pgs("grGreens_80213900", 2), gs("grGreens_80213908", 2), gs("grGreens_8021390C", 2),
        gs("stageGObj1_OnInit", 1), pgs("grGreens_8021393C", 1), gs("grGreens_80213944", 1), gs("grGreens_80213948", 1),
        gs("stageGObj3_OnInit", 3), pgs("grGreens_80213978", 3), gs("grGreens_80213980", 3), gs("grGreens_80213984", 3),
        gs("grGreens_80213988", 4), pgs("grGreens_802139B4", 4), gs("grGreens_802139BC", 4), gs("grGreens_802139C0", 4),
        gs("grGreens_802139C4", 5, 0), pgs("grGreens_80213AAC", 5),
        dict(name="grGreens_80213AB4", cases=400, pre_c="VW[0].x = rw(); VW[0].y = rw(); float a = rw(); float b = rw(); float c = rw(); float d = rw();",
             pre_r="VW[0].x = rw(); VW[0].y = rw(); let a = rw(); let b = rw(); let c = rw(); let d = rw();",
             c="ret = (u32) grGreens_80213AB4(&VW[0], a, b, c, d);", r="ret = m::grGreens_80213AB4(&raw mut VW[0], a, b, c, d) as u32;"),
        dict(name="grGreens_80213B1C", usel=1, pre_c=WHISPY_PRE_C + " YP.x3C_windSpeed = rw(); YP.x40_left = rw(); YP.x44_right = rw(); YP.x48_top = rw(); YP.x4C_bottom = rw();",
             pre_r=WHISPY_PRE_R + " YP.x3C_windSpeed = rw(); YP.x40_left = rw(); YP.x44_right = rw(); YP.x48_top = rw(); YP.x4C_bottom = rw();",
             c="ret = (u32) grGreens_80213B1C(&GS[5], &GS[0], &VW[1]);", r="ret = m::grGreens_80213B1C(&raw mut GS[5], &raw mut GS[0], &raw mut VW[1]) as u32;"),
        dict(name="grGreens_80213C10", usel=1, cases=600, pre_c=WHISPY_PRE_C + " YP.x40_left = rw(); YP.x44_right = rw(); YP.x48_top = rw(); YP.x4C_bottom = rw();",
             pre_r=WHISPY_PRE_R + " YP.x40_left = rw(); YP.x44_right = rw(); YP.x48_top = rw(); YP.x4C_bottom = rw();",
             c="grGreens_80213C10(&GS[5]);", r="m::grGreens_80213C10(&raw mut GS[5]);"),
        gs("grGreens_80214654", 5), gs("fn_80214658", 6),
        gs("grGreens_80214674", 6, 0), pgs("grGreens_80214794", 6),
        dict(name="grGreens_8021479C", usel=0, cases=300, c="grGreens_8021479C(&GS[6]);", r="m::grGreens_8021479C(&raw mut GS[6]);"),
        gs("grGreens_80214804", 6), gs("grGreens_8021483C", 6),
        dict(name="BOMB", pre_c="int a = (int) (fv() % 24);", pre_r="let a = (fv() % 24) as i32;", c="BOMB(a);", r="m::BOMB(a);"),
        dict(name="grGreens_80214B58", c="grGreens_80214B58(&GS[6]);", r="m::grGreens_80214B58(&raw mut GS[6]);"),
        dict(name="grGreens_80214FA8", pre_c="for (i = 0; i < 30; i++) BB[i].status = 0;", pre_r="for i in 0..30usize { BB[i].set_status(0); }",
             c="grGreens_80214FA8(&GS[6]);", r="m::grGreens_80214FA8(&raw mut GS[6]);"),
        dict(name="grGreens_802150C4", pre_c="int a1 = (int) (fv() % 6); int a2 = (int) (fv() % 5); for (i = 0; i < 30; i++) if (fv() & 1) BB[i].status = 3;",
             pre_r="let a1 = (fv() % 6) as i32; let a2 = (fv() % 5) as i32; for i in 0..30usize { if fv() & 1 != 0 { BB[i].set_status(3); } }",
             c="grGreens_802150C4(&GS[6], a1, a2);", r="m::grGreens_802150C4(&raw mut GS[6], a1, a2);"),
        dict(name="get_block_material_params", pre_c="int a = (int) (fv() % 4); float d = 0.0f; float s = 0.0f;",
             pre_r="let a = (fv() % 4) as i32; let mut d = 0.0f32; let mut s = 0.0f32;",
             c="get_block_material_params(a, &d, &s); ret = B(d); HM(B(s));", r="m::get_block_material_params(a, &mut d, &mut s); ret = d.to_bits(); hm(s.to_bits());"),
        dict(name="grGreens_80215358", pre_c=RC_C + " int a3 = 1 + (int) (fv() % 2); int a4 = (int) (fv() % 4); BB[row * 6 + col].status = 0;",
             pre_r=RC_R + " let a3 = 1 + (fv() % 2) as i32; let a4 = (fv() % 4) as i32; BB[(row * 6 + col) as usize].set_status(0);",
             c="grGreens_80215358(&GS[6], col, row, a3, a4);", r="m::grGreens_80215358(&raw mut GS[6], col, row, a3, a4);"),
        dict(name="fn_802159B4", c="fn_802159B4(&IG[0], &GG[6]);", r="m::fn_802159B4(&raw mut IG[0], &raw mut GG[6]);"),
        dict(name="grGreens_802159B8", pre_c=RC_C + " BB[row * 6 + col].x1_7 = fv() & 1;", pre_r=RC_R + " BB[(row * 6 + col) as usize].set_x1_7(fv() & 1);",
             c="grGreens_802159B8(&GG[6], col, row, &GS[(fv() % 7)]);", r="m::grGreens_802159B8(&raw mut GG[6], col, row, &raw mut GS[(fv() % 7) as usize]);"),
        dict(name="find_block", pre_c="int row = -1; int col = -1; int k = (int) (fv() % 31);",
             pre_r="let mut row = -1i32; let mut col = -1i32; let k = (fv() % 31) as usize;",
             c="ret = (u32) find_block(&GG[6], k < 30 ? &IG[k] : &IG[0], &row, &col); HM((u32) row); HM((u32) col);",
             r="ret = m::find_block(&raw mut GG[6], if k < 30 { &raw mut IG[k] } else { &raw mut IG[0] }, &mut row, &mut col) as u32; hm(row as u32); hm(col as u32);"),
        dict(name="fn_80215B84", pre_c="BB[0].status = 1; float d = rw();", pre_r="BB[0].set_status(1); let d = rw();",
             c="fn_80215B84(&IG[0], &GG[6], &VW[0], &GS[(fv() % 7)], d);", r="m::fn_80215B84(&raw mut IG[0], &raw mut GG[6], &raw mut VW[0], &raw mut GS[(fv() % 7) as usize], d);"),
        dict(name="fn_80215D50", c="fn_80215D50(&IG[0], &GG[6], &GS[1]);", r="m::fn_80215D50(&raw mut IG[0], &raw mut GG[6], &raw mut GS[1]);"),
        dict(name="grGreens_80215D54", pre_c="int a1 = (int) (fv() % 6); int a2 = (int) fv();", pre_r="let a1 = (fv() % 6) as i32; let a2 = fv() as i32;",
             c="ret = PI((void*) (intptr_t) grGreens_80215D54(&GS[6], a1, a2));", r="ret = pi((m::grGreens_80215D54(&raw mut GS[6], a1, a2) as isize) as *const u8);"),
        dict(name="grGreens_80215ED8", cases=500, pre_c=RC_C, pre_r=RC_R,
             c="grGreens_80215ED8(&GS[6], col, row);", r="m::grGreens_80215ED8(&raw mut GS[6], col, row);"),
        dict(name="grGreens_802166C4", cases=500, c="grGreens_802166C4(&GS[6]);", r="m::grGreens_802166C4(&raw mut GS[6]);"),
        dict(name="grGreens_80216C20", c="grGreens_80216C20(&GS[6]);", r="m::grGreens_80216C20(&raw mut GS[6]);"),
        dict(name="fn_80216DE4", pre_c="int jid = (int) (fv() % 64); int jb = (int) fv(); u32 jk = fv() % 3; float jd = rw();",
             pre_r="let jid = (fv() % 64) as i32; let jb = fv() as i32; let jk = fv() % 3; let jd = rw();",
             c="fn_80216DE4(&GG[6], jid, &CD, jb, jk, jd);", r="m::fn_80216DE4(&raw mut GG[6] as *mut c_void, jid, &raw mut CD, jb, jk, jd);"),
        dict(name="grGreens_80216E64", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;", c="ret = PI(grGreens_80216E64(a));", r="ret = pi(m::grGreens_80216E64(a));"),
        dict(name="grGreens_80216E6C", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = (u32) grGreens_80216E6C(&VW[0], a, &JO[2]);", r="ret = m::grGreens_80216E6C(&raw mut VW[0], a, &raw mut JO[2]) as u32;"),
        dict(name="randrange", cases=300, pre_c="int a = (int) (fv() % 50) - 10; int b = (int) (fv() % 50) - 10;", pre_r="let a = (fv() % 50) as i32 - 10; let b = (fv() % 50) as i32 - 10;",
             c="ret = (u32) randrange(a, b);", r="ret = m::randrange(a, b) as u32;"),
        dict(name="getBlock", pre_c="int a = (int) (fv() % 5); int b = (int) (fv() % 6);", pre_r="let a = (fv() % 5) as i32; let b = (fv() % 6) as i32;",
             c="ret = PI(getBlock(&GG[6], a, b));", r="ret = pi(m::getBlock(&raw mut GG[6], a, b));"),
        dict(name="get_whispy_dir", c="ret = (u32) get_whispy_dir(&GS[5], &VW[1]);", r="ret = m::get_whispy_dir(&raw mut GS[5], &raw mut VW[1]) as u32;"),
        dict(name="tables", cases=1,
             c="for (j = 0; j < 7; j++) hash_StageCallbacks(&grGr_callbacks[j]); hash_StageData(&grGr_StageData); "
               "HM(SH(grGr_StageData.data1)); HM(PI(grGr_StageData.callbacks)); HM(PI(grGr_StageData.joints)); "
               "for (j = 0; j < 10; j++) HM((u32) grGr_803E7734[j]); "
               "for (j = 0; j < 8; j++) { HM((u32) grGr_803E775C[j]); HM((u32) grGr_803E77B4[j]); } "
               "for (j = 0; j < 7; j++) { HM((u32) grGr_803E777C[j][0]); HM((u32) grGr_803E777C[j][1]); HM((u32) grGr_803E77D4[j][0]); HM((u32) grGr_803E77D4[j][1]); } "
               "for (j = 0; j < 2; j++) { HM(B(grGr_803E780C[j].x)); HM(B(grGr_803E780C[j].y)); HM(B(grGr_803E780C[j].z)); } "
               "for (j = 0; j < 30; j++) { HM((u32) grGr_803E7840[j]); HM((u32) grGr_803E787C[j]); }",
             r="for j in 0..7 { g::hash_StageCallbacks(&m::grGr_callbacks[j]); } g::hash_StageData(&m::grGr_StageData); "
               "hm(sh(m::grGr_StageData.data1 as *const u8)); hm(pi(m::grGr_StageData.callbacks)); hm(pi(m::grGr_StageData.joints)); "
               "for j in 0..10 { hm(m::grGr_803E7734[j] as u32); } "
               "for j in 0..8 { hm(m::grGr_803E775C[j] as u32); hm(m::grGr_803E77B4[j] as u32); } "
               "for j in 0..7 { hm(m::grGr_803E777C[j][0] as u32); hm(m::grGr_803E777C[j][1] as u32); hm(m::grGr_803E77D4[j][0] as u32); hm(m::grGr_803E77D4[j][1] as u32); } "
               "for j in 0..2 { hm(m::grGr_803E780C[j].x.to_bits()); hm(m::grGr_803E780C[j].y.to_bits()); hm(m::grGr_803E780C[j].z.to_bits()); } "
               "for j in 0..30 { hm(m::grGr_803E7840[j] as i32 as u32); hm(m::grGr_803E787C[j] as i32 as u32); }"),
    ],
)
