"""Helpers for the driver descriptions in drivers.py / tus/*.py (mk_tu.py): the common op shapes."""


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



# the common Ground fixture: G (the stage GObj) -> GP (its Ground) and JO[0] (its joint)
GROUND_SETUP_C = """
            G.user_data = &GP; G.hsd_obj = &JO[0]; GP.gobj = &G;
            for (i = 0; i < 8; i++) { GS[i].user_data = &GP; GS[i].hsd_obj = &JO[i]; }
"""
GROUND_SETUP_R = """
                G.user_data = &raw mut GP as *mut c_void; G.hsd_obj = &raw mut JO[0] as *mut c_void; GP.gobj = &raw mut G;
                for i in 0..8 { GS[i].user_data = &raw mut GP as *mut c_void; GS[i].hsd_obj = &raw mut JO[i] as *mut c_void; }
"""

