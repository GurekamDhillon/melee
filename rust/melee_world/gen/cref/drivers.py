"""Driver descriptions for mk_tu.py (see there). Helpers build the common op shapes."""


def vg(name, usel=0):
    """void f(Ground_GObj*)"""
    return dict(name=name, c=f"{name}(&G);", r=f"m::{name}(&raw mut G);", usel=usel)


def pg(name, usel=0):
    """bool f(Ground_GObj*)"""
    return dict(name=name, c=f"ret = (u32) {name}(&G);", r=f"ret = m::{name}(&raw mut G) as u32;", usel=usel)


def v0(name):
    return dict(name=name, c=f"{name}();", r=f"m::{name}();")


def p0(name):
    return dict(name=name, c=f"ret = (u32) {name}();", r=f"ret = m::{name}() as u32;")


def f0(name):
    return dict(name=name, c=f"ret = B({name}());", r=f"ret = m::{name}().to_bits();")


DRV = {}

# the common Ground fixture: G (the stage GObj) -> GP (its Ground) and JO[0] (its joint)
GROUND_SETUP_C = """
            G.user_data = &GP; G.hsd_obj = &JO[0]; GP.gobj = &G;
            for (i = 0; i < 8; i++) { GS[i].user_data = &GP; GS[i].hsd_obj = &JO[i]; }
"""
GROUND_SETUP_R = """
                G.user_data = &raw mut GP as *mut c_void; G.hsd_obj = &raw mut JO[0] as *mut c_void; GP.gobj = &raw mut G;
                for i in 0..8 { GS[i].user_data = &raw mut GP as *mut c_void; GS[i].hsd_obj = &raw mut JO[i] as *mut c_void; }
"""

DRV["grgarden"] = dict(
    state=[("G", "HSD_GObj", 1), ("GP", "Ground", 1), ("JO", "HSD_JObj", 8), ("GS", "HSD_GObj", 8),
           ("YP", "struct grGarden_YakumonoParam", 1), ("GEN", "HSD_Generator", 1), ("SRT", "HSD_psAppSRT", 1),
           ("PG", "HSD_GObj", 1), ("VW", "Vec", 2), ("GI", "HSD_GObj", 1)],
    setup_c=GROUND_SETUP_C + """
            GEN.appsrt = &SRT; yakumono_param = &YP;
            GP.x8_callback = grGarden_80202D94; GP.xC_callback = grGarden_80202D98;
""",
    setup_r=GROUND_SETUP_R + """
                GEN.appsrt = &raw mut SRT; m::yakumono_param = &raw mut YP;
                GP.x8_callback = Some(m::grGarden_80202D94); GP.xC_callback = Some(m::grGarden_80202D98);
""",
    decl_r="use m::grGarden_YakumonoParam;",
    post_c="            HM(PI(yakumono_param));",
    post_r="                hm(pi(m::yakumono_param));",
    ops=[
        dict(name="grGarden_80202B6C", c="grGarden_80202B6C((bool) fv());", r="m::grGarden_80202B6C(fv() as i32);"),
        dict(name="grGarden_80202B70", usel=1, c="yakumono_param = 0; grGarden_80202B70();", r="m::yakumono_param = core::ptr::null_mut(); m::grGarden_80202B70();"),
        v0("grGarden_80202C24"), v0("grGarden_80202C4C"), p0("grGarden_80202C70"),
        dict(name="grGarden_80202C78", usel=1, pre_c="int a = (int) (fv() % 7);", pre_r="let a = (fv() % 7) as i32;",
             c="ret = PI(grGarden_80202C78(a));", r="ret = pi(m::grGarden_80202C78(a));"),
        vg("stageGObj0_OnInit"), pg("grGarden_80202D8C"), vg("grGarden_80202D94"), vg("grGarden_80202D98"),
        vg("grGarden_80202D9C"), vg("grGarden_80202DC8"), pg("grGarden_80202E40"), vg("grGarden_80202E48"),
        vg("grGarden_80202E68"), vg("grGarden_80202E6C"), pg("grGarden_80202F34"), vg("grGarden_80202F3C"),
        vg("grGarden_80202F40"), vg("grGarden_80202F44"), pg("grGarden_80202F94"), vg("grGarden_80202F9C"),
        vg("grGarden_80202FA0"), vg("grGarden_80202FA4"), pg("grGarden_80202FF4"), vg("grGarden_80202FFC"),
        vg("grGarden_80203000"), vg("grGarden_80203004", 0), pg("grGarden_80203090"), vg("grGarden_80203098", 0),
        vg("grGarden_802031A0"), vg("grGarden_802031A4", 1), pg("grGarden_80203248"), vg("grGarden_80203250", 1),
        vg("grGarden_80203494"),
        dict(name="grGarden_80203498", c="grGarden_80203498(&GI, &GP);", r="m::grGarden_80203498(&raw mut GI, &raw mut GP);"),
        dict(name="grGarden_8020349C", pre_c="u32 a = fv();", pre_r="let a = fv();",
             c="ret = (u32) grGarden_8020349C(a, &PG, &VW[1]);", r="ret = m::grGarden_8020349C(a, &raw mut PG, &raw mut VW[1]) as u32;"),
        dict(name="grGarden_80203560", pre_c="float a = rw();", pre_r="let a = rw();",
             c="grGarden_80203560(a, &VW[0]);", r="m::grGarden_80203560(a, &raw mut VW[0]);"),
        dict(name="grGarden_802035C8", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = PI(grGarden_802035C8(a));", r="ret = pi(m::grGarden_802035C8(a));"),
        dict(name="grGarden_802035D0", pre_c="int a = (int) fv();", pre_r="let a = fv() as i32;",
             c="ret = (u32) grGarden_802035D0(&VW[0], a, &JO[2]);", r="ret = m::grGarden_802035D0(&raw mut VW[0], a, &raw mut JO[2]) as u32;"),
        f0("grGarden_80203624"),
        dict(name="tables", cases=1,
             c="for (j = 0; j < 7; j++) hash_StageCallbacks(&grGd_StageCallbacks[j]); hash_StageData(&grGd_StageData); HM(SH(grGd_StageData.data1)); HM(PI(grGd_StageData.callbacks));",
             r="for j in 0..7 { g::hash_StageCallbacks(&m::grGd_StageCallbacks[j]); } g::hash_StageData(&m::grGd_StageData); hm(sh(m::grGd_StageData.data1 as *const u8)); hm(pi(m::grGd_StageData.callbacks));"),
    ],
)
