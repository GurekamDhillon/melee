#!/usr/bin/env python
"""Generator for the gr/grt*.c per-stage TU ports (shard 593-600).

usage: python gen_grt_stages.py <worktree root> <scratch dir>

Emits (all committed):
  rust/melee_world/src/gr/inlines.rs         the gr/inlines.h inlines these TUs expand
  rust/melee_world/src/tu/<stem>.rs          one port per C file
  rust/melee_world/tests/tu_grt_stages.rs    the state-oracle driver (Rust side)
and, into the scratch dir, the C side: prelude.h, stub include tree, the verbatim C copies and one
driver per TU (compiled and run by gen_grt_stages_ref.sh into tests/data/tu_<stem>_ref.txt).

The ports' bodies are matched by pattern from the C text (a body that matches no pattern aborts),
so the Rust is derived from the C, and the C reference is the unmodified game source.
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from grt_db import CALLEES, BY_NAME  # noqa: E402

ROOT = os.path.abspath(sys.argv[1])
OUT = os.path.abspath(sys.argv[2])
STEMS = ["grtdrmario", "grtemblem", "grtest", "grtfalco", "grtfox", "grtgamewatch", "grtganon", "grticeclimber"]
KIND = {"grtdrmario": "Gr_Kind_TDrmario", "grtemblem": "Gr_Kind_TEmblem", "grtest": "Gr_Kind_Test",
        "grtfalco": "Gr_Kind_TFalco", "grtfox": "Gr_Kind_TFox", "grtgamewatch": "Gr_Kind_TGamewatch",
        "grtganon": "Gr_Kind_TGanon", "grticeclimber": "Gr_Kind_TIceclimber"}
KINDNUM = {"Gr_Kind_TDrmario": 44, "Gr_Kind_TEmblem": 64, "Gr_Kind_Test": 1, "Gr_Kind_TFalco": 45,
           "Gr_Kind_TFox": 46, "Gr_Kind_TGamewatch": 63, "Gr_Kind_TGanon": 65, "Gr_Kind_TIceclimber": 47}
K = 20  # cases per function


def read(p):
    with open(p, encoding="utf-8", errors="replace", newline="") as f:
        return f.read().replace("\r\n", "\n")


def write(p, s):
    os.makedirs(os.path.dirname(p), exist_ok=True)
    with open(p, "w", encoding="utf-8", newline="\n") as f:
        f.write(s)


def strip_comments(t):
    t = re.sub(r"/\*.*?\*/", " ", t, flags=re.S)
    t = re.sub(r"//[^\n]*", " ", t)
    return t


def norm(s):
    return re.sub(r"\s+", " ", s).strip()


# ---------------------------------------------------------------------------------------------
# C parsing
# ---------------------------------------------------------------------------------------------
FN_RE = re.compile(r"(?m)^((?:static\s+)?(?:struct\s+)?[A-Za-z_]\w*\s*\**\s*)([A-Za-z_]\w*)\(([^;{}()]*)\)\s*\{")


def match_brace(t, i):
    d = 0
    j = i
    while True:
        if t[j] == "{":
            d += 1
        elif t[j] == "}":
            d -= 1
            if d == 0:
                return j
        j += 1


def parse_functions(text):
    t = strip_comments(text)
    fns = []
    for m in FN_RE.finditer(t):
        ret = norm(m.group(1)).replace("static ", "").replace("struct ", "")
        ret = ret.replace(" *", "*")
        name = m.group(2)
        pr = norm(m.group(3))
        params = []
        if pr and pr != "void":
            for p in pr.split(","):
                p = norm(p)
                mm = re.match(r"(.*?)(\w+)$", p)
                ty = norm(mm.group(1)).replace("struct ", "").replace(" *", "*")
                params.append((ty, mm.group(2)))
        i = m.end() - 1
        j = match_brace(t, i)
        body = norm(t[i + 1:j])
        fns.append(dict(ret=ret, name=name, params=params, body=body, static=m.group(1).strip().startswith("static")))
    return fns


RT = {"Ground_GObj*": "*mut HSD_GObj", "HSD_GObj*": "*mut HSD_GObj", "Item_GObj*": "*mut HSD_GObj",
      "Ground*": "*mut Ground", "Vec3*": "*mut Vec3", "HSD_JObj*": "*mut HSD_JObj", "bool": "i32",
      "int": "i32", "enum_t": "i32", "s32": "i32", "f32": "f32", "DynamicsDesc*": "*mut DynamicsDesc"}
KINDOF = {"Ground_GObj*": "G", "HSD_GObj*": "G", "Item_GObj*": "G", "Ground*": "R", "Vec3*": "P",
          "HSD_JObj*": "J", "bool": "b", "int": "i", "enum_t": "i", "s32": "i", "f32": "f"}


def rust_sig(f):
    ps = ", ".join("%s: %s" % (rn(n), RT[t]) for t, n in f["params"])
    r = f["ret"]
    rr = "" if r == "void" else " -> " + RT[r]
    return ps, rr


def rn(n):
    return n + "_" if n in ("unused",) else n


# ---------------------------------------------------------------------------------------------
# Rust body derivation
# ---------------------------------------------------------------------------------------------
class Ctx:
    pass


def gobj_param(f):
    return rn(f["params"][0][1])


def derive_body(stem, f, text_info):
    """Return (rust_body, pure) for a function, matched by pattern from the C body."""
    b = f["body"]
    n = f["name"]
    p = f["params"]
    pure = False
    if b in ("", "return;"):
        return "", True
    if b == "return false;":
        return "0", True
    if b == "return true;":
        return "1", True
    if b == "return NULL;":
        return "null_mut()", True
    if b == "grZakoGenerator_801CAE04(NULL);":
        return "ext::grZakoGenerator_801CAE04(null_mut());", False
    m = re.fullmatch(r"(Ground_StartMapAnim|Ground_InitMapCollAndAnim|Ground_UpdateWindAndMapColl)\((\w+)\);", b)
    if m:
        return "inlines::%s(%s);" % (m.group(1), rn(m.group(2))), False
    m = re.fullmatch(r"Ground_UpdateMapColl\((\w+)\);", b)
    if m:
        return "ext::Ground_UpdateMapColl(%s);" % rn(m.group(1)), False
    m = re.fullmatch(r"Ground\* gp = GET_GROUND\((\w+)\); grAnime_801C8138\(\1, gp->map_id, 0\);", b)
    if m:
        g = rn(m.group(1))
        return "let gp = inlines::GET_GROUND(%s);\n    ext::grAnime_801C8138(%s, (*gp).map_id as i32, 0);" % (g, g), False
    m = re.fullmatch(r"(?:(yakumono_param) = Ground_GetYakumonoParam\(\); )?Ground_InitTargetStage\((\w+)\);", b)
    if m:
        s = ""
        if m.group(1):
            s = "YAKUMONO_PARAM.store(ext::Ground_GetYakumonoParam().cast(), Ordering::Relaxed);\n    "
        return s + "inlines::Ground_InitTargetStage(%s);" % m.group(2), False
    m = re.fullmatch(r"(?:HSD_GObj|Ground_GObj)\* gobj; StageCallbacks\* callbacks = &(\w+)\[(\w+)\]; gobj = Ground_GetStageGObj\(\2\); "
                     r"if \(gobj != NULL\) \{ Ground_SetupStageCallbacks\(gobj, callbacks\); \} else \{ OSReport\(\"%s:%d: couldn t get gobj\(id=%d\)\\n\", __FILE__, (\w+), \2\); \} return gobj;", b)
    if m:
        idn = rn(m.group(2))
        line = int(m.group(3), 0)
        return ("let callbacks = stage_callbacks().add(%s as usize);\n"
                "    let gobj = ext::Ground_GetStageGObj(%s);\n"
                "    if !gobj.is_null() {\n        inlines::Ground_SetupStageCallbacks(gobj, callbacks);\n    } else {\n"
                "        ext::OSReport(c\"%%s:%%d: couldn t get gobj(id=%%d)\\n\", c\"src/melee/gr/%s.c\", %d, %s);\n    }\n    gobj"
                % (idn, idn, stem, line, idn)), False
    if "mpJointFromLine" in b:
        m = re.fullmatch(r"if \((\w+) != -1\) \{ enum_t i = mpJointFromLine\(\1\); if \(i != -1 && i == (\d)\) \{ i = mpLineGetKind\(\1\); (.*) return NULL; \} \} return NULL;", b)
        assert m, (stem, n, b)
        a = rn(m.group(1))
        joint = m.group(2)
        out = ("if %s != -1 {\n        let mut i = ext::mpJointFromLine(%s);\n        if i != -1 && i == %s {\n            i = ext::mpLineGetKind(%s);\n"
               % (a, a, joint, a))
        for kk, fld in re.findall(r"if \(i == (CollLine_\w+)\) \{ return yakumono_param->(\w+); \}", m.group(3)):
            out += "            if i == %s as i32 {\n                return (*YAKUMONO_PARAM.load(Ordering::Relaxed)).%s as *mut DynamicsDesc;\n            }\n" % (kk, fld)
        out += "            return null_mut();\n        }\n    }\n    null_mut()"
        return out, False
    return None, False


CUSTOM = {}

CUSTOM[("grtest", "grTest_80206E30")] = """\
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    grTest_80207044(0);
    let gobj = grTest_80207044(2);
    (*inlines::GET_GROUND(gobj)).x8_callback = Some(stageGObj2_X8Callback);
    let gobj = grTest_80207044(1);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    jobj_inline::HSD_JObjSetScaleX(jobj, 80.0);
    jobj_inline::HSD_JObjSetScaleY(jobj, 80.0);
    jobj_inline::HSD_JObjSetScaleZ(jobj, 80.0);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();"""

CUSTOM[("grtest", "grTest_802071C4")] = """\
    let pad = ext::HSD_PadMasterStatus();
    if (*pad.add(1)).trigger & HSD_PAD_A as u32 != 0 {
        let j = ext::Ground_801C3FA4(gobj, 0x10);
        if !j.is_null() {
            if ext::HSD_JObjGetFlags(j) & 0x10 != 0 {
                ext::HSD_JObjClearFlags(j, JOBJ_HIDDEN as u32);
            } else {
                ext::HSD_JObjSetFlags(j, JOBJ_HIDDEN as u32);
            }
            grTe_804D6A48 = 0.0;
        }
    }
    if (*pad.add(2)).trigger & HSD_PAD_A as u32 != 0 {
        let j = ext::Ground_801C3FA4(gobj, 0x11);
        if !j.is_null() {
            if ext::HSD_JObjGetFlags(j) & 0x10 != 0 {
                ext::HSD_JObjClearFlags(j, JOBJ_HIDDEN as u32);
            } else {
                ext::HSD_JObjSetFlags(j, JOBJ_HIDDEN as u32);
            }
            grTe_804D6A48 = 0.0;
        }
    }
    let button = (*pad.add(1)).button;
    if button & 0x400 != 0 && grTe_804D6A48 > -1.2217305f32 {
        grTe_804D6A48 -= 0.08726646f32;
    }
    if button & 0x800 != 0 && grTe_804D6A48 < 1.2217305f32 {
        grTe_804D6A48 += 0.08726646f32;
    }
    let j = ext::Ground_801C3FA4(gobj, 11);
    if !j.is_null() {
        jobj_inline::HSD_JObjSetRotationZ(j, grTe_804D6A48);
    }"""

CUSTOM[("grticeclimber", "grTIceClimber_80221208")] = """\
    let it = (*gobj).user_data as *mut Item;
    let mato_x4 = (*it).xDD4_itemVar.mato.x4;
    ext::HSD_JObjSetFlagsAll(mato_x4, JOBJ_HIDDEN as u32);
    let mut pos: Vec3 = core::mem::zeroed();
    ext::lb_8000B1CC(mato_x4, null(), &mut pos);
    ext::efSync_Spawn(0x445, gobj, &mut pos);
    ext::Camera_RequestQuake(QuakeKind_Small as i32, null_mut());
    ext::Ground_801C53EC(310);
    ext::grMaterial_801C8CDC(gobj);"""

CUSTOM[("grticeclimber", "grTIceClimber_80221288")] = """\
    let gp = inlines::GET_GROUND(ground_gobj);
    let mut i = 0usize;
    while grTIc_803E8B5C[i] != -1 {
        let result_0 = ext::Ground_801C33C0(2, grTIc_803E8B5C[i] as i32);
        if result_0 != -1 {
            let jo = ext::Ground_801C3FA4(ground_gobj, result_0);
            let result_1 = ext::grMaterial_801C8CFC(0, 0, gp, jo, null(), Some(grTIceClimber_80221208), null());
            if !result_1.is_null() {
                ext::grMaterial_801C8DE0(result_1, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 4.0);
                ext::grMaterial_801C8E08(result_1);
            }
        }
        i += 1;
    }"""

# stageGObj2_OnInit of the ice climbers is the only one with extra calls
CUSTOM[("grticeclimber", "stageGObj2_OnInit")] = """\
    inlines::Ground_InitMapCollAndAnim(gobj);
    ext::grAnime_801C7FF8(gobj, 69, 2, 1, 0.0, 1.0);
    grTIceClimber_80221288(gobj);"""


def first_param_fix(stem, f, body):
    """Parameter names are used by the body text; some C files name them arg0 etc."""
    return body


# ---------------------------------------------------------------------------------------------
# Table / data parsing
# ---------------------------------------------------------------------------------------------
def split_top(s):
    out, d, cur = [], 0, ""
    for ch in s:
        if ch in "{(":
            d += 1
        elif ch in "})":
            d -= 1
        if ch == "," and d == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def find_init(t, pat):
    m = re.search(pat, t)
    if not m:
        return None
    i = t.index("{", m.end() - 1)
    j = match_brace(t, i)
    return m, t[i + 1:j]


def evalc(e):
    e = e.strip()
    if e in ("0", "NULL"):
        return 0
    return eval(e.replace("(s16)", ""), {}, {}) & 0xFFFFFFFF


def parse_tables(stem, text):
    t = strip_comments(text)
    r = {}
    m = re.search(r"(?m)^(?:static\s+)?StageCallbacks\s+(\w+)\[(\w*)\]\s*=\s*", t)
    r["cb_name"] = m.group(1)
    r["cb_len"] = int(m.group(2)) if m.group(2) else None
    i = t.index("{", m.end() - 1)
    j = match_brace(t, i)
    items = split_top(t[i + 1:j])
    ents = []
    for it in items:
        assert it.startswith("{")
        inner = split_top(it[1:-1].strip())
        if len(inner) == 1 or all(x in ("0", "NULL") for x in inner[:4]) and len(inner) < 4:
            ents.append(None)
        else:
            assert len(inner) == 5, (stem, inner)
            ents.append(inner)
    r["cb_ents"] = ents
    if r["cb_len"] is None:
        r["cb_len"] = len(ents)
    m = re.search(r"(?m)^StageData\s+(\w+)\s*=\s*", t)
    r["sd_name"] = m.group(1)
    i = t.index("{", m.end() - 1)
    j = match_brace(t, i)
    r["sd_items"] = split_top(t[i + 1:j])
    # yakumono struct
    m = re.search(r"(?:typedef )?struct (\w+) \{([^}]*)\}", t)
    r["yak_fields"] = None
    for mm in re.finditer(r"struct (\w+) \{([^}]*)\}", t):
        if "Yakumono" in mm.group(1):
            flds = re.findall(r"(?:UNK_T|DynamicsDesc\*)\s+(\w+);", mm.group(2))
            r["yak_fields"] = flds
            r["yak_type"] = "UNK_T" if "UNK_T" in mm.group(2) else "dd"
    # joints table
    m = re.search(r"(?m)^GrJoint\s+(\w+)\[\]\s*=\s*", t)
    r["joints"] = None
    if m:
        i = t.index("{", m.end() - 1)
        j = match_brace(t, i)
        trip = re.findall(r"\{\s*(-?\d+),\s*(-?\d+),\s*(-?\d+)\s*\}", t[i + 1:j])
        r["joints"] = (m.group(1), [tuple(int(x) for x in tr) for tr in trip])
    m = re.search(r"(?m)^s16\s+(\w+)\[\]\s*=\s*", t)
    r["s16tab"] = None
    if m:
        i = t.index("{", m.end() - 1)
        j = match_brace(t, i)
        r["s16tab"] = (m.group(1), [int(x) for x in re.findall(r"-?\d+", t[i + 1:j])])
    r["float_global"] = bool(re.search(r"(?m)^float grTe_804D6A48;", t))
    return r


SD_FIELDS = ["grkind", "callbacks", "data1", "on_init", "on_demo_init", "on_load", "on_start", "callback4",
             "on_touch_line", "on_check_shadow_render", "flags2", "joints", "joint_count"]


# ---------------------------------------------------------------------------------------------
# ext module generation
# ---------------------------------------------------------------------------------------------
def gen_ext(names, reexport, header_note):
    own = [BY_NAME[n] for n in names]
    s = "pub mod ext {\n    use super::*;\n"
    if reexport:
        s += "    pub use crate::gr::inlines::ext::{%s};\n" % ", ".join(reexport)
    s += "\n    /// The callees of this TU (unported TUs). Each defaults to a TODO panic; a test or the game\n"
    s += "    /// wires the real function by assigning [`HOOKS`].\n"
    s += "    #[derive(Clone, Copy)]\n    pub struct Hooks {\n"
    for c in own:
        s += "        pub %s: unsafe fn(%s)%s,\n" % (c["name"], c["rp"], (" " + c["rr"]) if c["rr"] else "")
    s += "    }\n"
    for c in own:
        s += "    unsafe fn todo_%s(%s)%s { panic!(\"TODO(%s) %s\") }\n" % (
            c["name"], ", ".join("_" + x.strip() for x in split_params(c["rp"])), (" " + c["rr"]) if c["rr"] else "", c["tag"], c["name"])
    s += "    impl Hooks {\n        pub const fn todo() -> Hooks {\n            Hooks {\n"
    for c in own:
        s += "                %s: todo_%s,\n" % (c["name"], c["name"])
    s += "            }\n        }\n    }\n"
    s += "    pub static HOOKS: RwLock<Hooks> = RwLock::new(Hooks::todo());\n\n"
    for c in own:
        args = ", ".join(x.split(":")[0].strip() for x in split_params(c["rp"]))
        s += "    pub unsafe fn %s(%s)%s {\n        let hook = HOOKS.read().unwrap_or_else(|e| e.into_inner()).%s;\n        hook(%s)\n    }\n" % (
            c["name"], c["rp"], (" " + c["rr"]) if c["rr"] else "", c["name"], args)
    s += "}\n"
    return s


def split_params(rp):
    """split 'a: T, b: Option<fn(A, B)>' on top-level commas"""
    out, d, cur = [], 0, ""
    for ch in rp:
        if ch in "<([":
            d += 1
        elif ch in ">)]":
            d -= 1
        if ch == "," and d == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return [x.strip() for x in out]


# ---------------------------------------------------------------------------------------------
# gr/inlines.rs
# ---------------------------------------------------------------------------------------------
def gen_inlines():
    inl = [c["name"] for c in CALLEES if c["home"] == "inl"]
    s = '''//! Port of the `static inline` helpers of src/melee/gr/inlines.h that the per-stage gr TUs
//! expand (TARGET_PC branch): `Ground_StartMapAnim`, `Ground_InitMapCollAndAnim`,
//! `Ground_UpdateWindAndMapColl`, `Ground_SetupStageCallbacks`, `Ground_InitTargetStage`,
//! `GET_GROUND`. Generated by cref/gen_grt_stages.py (callee table in cref/grt_db.py); checked
//! against the C inlines (verbatim, compiled natively) through the TUs that expand them
//! (tests/tu_grt_stages.rs).
//!
//! Callees go through [`ext`]: TODO stubs named like the C functions, replaceable through
//! [`ext::HOOKS`]. Trap: `Ground_SetupStageCallbacks` binds the three callbacks with `Mex_GrBind`
//! on PC (an m-ex stage's table may hold guest addresses); the native pointer passes through.
#![allow(non_snake_case, non_upper_case_globals, clippy::missing_safety_doc)]

use core::ffi::c_void;
use core::mem::transmute;
use std::sync::RwLock;

use melee_types::{GObj_RenderFunc, Ground, HSD_GObj, HSD_GObjEvent, HSD_JObj, StageCallbacks, StageInfo};

'''
    s += gen_ext(inl, [], "")
    s = s.replace("    use super::*;\n", "    use super::*;\n", 1)
    s += '''
impl ext {}
'''
    return s


def gen_inlines_final():
    inl = [c["name"] for c in CALLEES if c["home"] == "inl"]
    head = gen_inlines().replace("\nimpl ext {}\n", "")
    head = head.replace("pub mod ext {\n    use super::*;\n",
                        "pub mod ext {\n    use super::*;\n\n"
                        "    /// grDisplay_801C5DB0 (gr/grdisplay.c): the render callback `Ground_SetupStageCallbacks` installs.\n"
                        "    /// Only its address is used here.\n"
                        "    pub unsafe extern \"C\" fn grDisplay_801C5DB0(_g: *mut HSD_GObj, _pass: i32) {\n"
                        "        panic!(\"TODO(group2: gr/grdisplay.c) grDisplay_801C5DB0\")\n    }\n", 1)
    body = '''
/// `GET_GROUND(gobj)`.
#[inline]
pub unsafe fn GET_GROUND(gobj: *mut HSD_GObj) -> *mut Ground {
    (*gobj).user_data.cast()
}

/// Resets the map's model to its base pose and starts animation set 0.
pub unsafe fn Ground_StartMapAnim(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// Binds the map's collision joints to its model, then starts animation set 0.
pub unsafe fn Ground_InitMapCollAndAnim(gobj: *mut HSD_GObj) {
    let gp = GET_GROUND(gobj);
    let jobj = (*gobj).hsd_obj as *mut HSD_JObj;
    ext::Ground_InitMapColl(jobj, (*gp).map_id as i32);
    ext::grAnime_801C8138(gobj, (*gp).map_id as i32, 0);
}

/// Advances the dynamics wind/force list, then re-syncs the map's collision joints.
pub unsafe fn Ground_UpdateWindAndMapColl(gobj: *mut HSD_GObj) {
    ext::lb_800115F4();
    ext::Ground_UpdateMapColl(gobj);
}

/// TARGET_PC `Ground_SetupStageCallbacks`.
pub unsafe fn Ground_SetupStageCallbacks(gobj: *mut HSD_GObj, callbacks: *mut StageCallbacks) {
    let gp = GET_GROUND(gobj);
    (*gp).x8_callback = None;
    (*gp).xC_callback = None;
    let render: GObj_RenderFunc = Some(ext::grDisplay_801C5DB0);
    ext::GObj_SetupGXLink(gobj, render, 3, 0);
    if (*callbacks).callback3.is_some() {
        let f = ext::Mex_GrBind((*callbacks).callback3.map_or(core::ptr::null_mut(), |f| f as *mut c_void));
        (*gp).x1C_callback = transmute::<*mut c_void, HSD_GObjEvent>(f);
    }
    if (*callbacks).on_init.is_some() {
        let f = ext::Mex_GrBind((*callbacks).on_init.map_or(core::ptr::null_mut(), |f| f as *mut c_void));
        if let Some(f) = transmute::<*mut c_void, HSD_GObjEvent>(f) {
            f(gobj);
        }
    }
    if (*callbacks).gobj_proc.is_some() {
        let f = ext::Mex_GrBind((*callbacks).gobj_proc.map_or(core::ptr::null_mut(), |f| f as *mut c_void));
        ext::HSD_GObj_SetupProc(gobj, transmute::<*mut c_void, HSD_GObjEvent>(f), 4);
    }
}

/// `Ground_InitTargetStage`: the common `OnInit` of the target-test stages.
pub unsafe fn Ground_InitTargetStage(create_gobj: unsafe extern "C" fn(i32) -> *mut HSD_GObj) {
    let si = ext::stage_info();
    (*si).unk8C.set_b4(0);
    (*si).unk8C.set_b5(1);
    create_gobj(0);
    create_gobj(1);
    create_gobj(2);
    ext::Ground_801C39C0();
    ext::Ground_801C3BB4();
    ext::Ground_801C4210();
    ext::Ground_801C42AC();
}
'''
    return head + body


# ---------------------------------------------------------------------------------------------
# TU generation
# ---------------------------------------------------------------------------------------------
def used_callees(stem, text):
    t = strip_comments(text)
    used = []
    for c in CALLEES:
        if re.search(r"\b%s\b" % c["name"], t):
            used.append(c["name"])
    if stem == "grtest":
        for x in ("HSD_JObjSetMtxDirtySub",):
            used.append(x)
    return used


def gen_tu(stem, trivial_names):
    src = read(os.path.join(ROOT, "src/melee/gr/%s.c" % stem))
    fns = parse_functions(src)
    tabs = parse_tables(stem, src)
    used = used_callees(stem, src)
    # the TU's ext: own callees; shared ones re-exported
    own = [n for n in used if BY_NAME[n]["home"] == "tu" or n == "OSReport"]
    own = [n for n in dict.fromkeys(own)]
    shared = [n for n in dict.fromkeys(used) if BY_NAME[n]["home"] == "inl"]
    # the setup fn uses Ground_GetStageGObj/OSReport; InitTargetStage inline: nothing extra
    for n in ("Ground_UpdateMapColl", "grAnime_801C8138"):
        pass
    if "Ground_UpdateMapColl" in src and "Ground_UpdateMapColl" not in shared:
        shared.append("Ground_UpdateMapColl")
    if re.search(r"\bgrAnime_801C8138\b", src) and "grAnime_801C8138" not in shared:
        shared.append("grAnime_801C8138")
    # the inlined Ground_StartMapAnim etc. are in `inlines`; the TU still names the direct calls
    if stem in ("grtest", "grtfalco", "grtfox", "grtgamewatch", "grtganon", "grticeclimber", "grtdrmario", "grtemblem"):
        pass
    has_yak = tabs["yak_fields"] is not None
    uses_hsd_assert_inline = stem == "grtest"

    s = "//! Port of src/melee/gr/%s.c (target-test stage `%s`).\n" % (stem, tabs["sd_name"])
    s += "//!\n"
    s += "//! Generated by cref/gen_grt_stages.py from the C text; every body is matched by pattern, so the\n"
    s += "//! Rust is derived from the C and verified against the unmodified C source compiled natively\n"
    s += "//! (state oracle: tests/tu_grt_stages.rs). Callees in unported TUs are TODO stubs in [`ext`]\n"
    s += "//! (replaceable through `ext::HOOKS`); the gr/inlines.h inlines live in [`crate::gr::inlines`].\n"
    s += "#![allow(non_snake_case, non_upper_case_globals, unused_imports, unused_unsafe, static_mut_refs, clippy::missing_safety_doc, clippy::all)]\n\n"
    s += "use core::ffi::{c_void, CStr};\nuse core::mem::MaybeUninit;\nuse core::ptr::{null, null_mut};\n"
    s += "use std::sync::atomic::{AtomicPtr, Ordering};\nuse std::sync::{Once, RwLock};\n\n"
    s += "use melee_types::{\n    DynamicsDesc, GrJoint, Ground, HSD_GObj, HSD_JObj, HSD_PadStatus, Item, StageCallbacks, StageCallbacks_anon_135,\n"
    s += "    StageData, StageInfo, Vec3, CollLine_Ceiling, CollLine_Floor, CollLine_LeftWall, CollLine_RightWall, HSD_PAD_A,\n"
    s += "    JOBJ_HIDDEN, JOBJ_MTX_DIRTY, JOBJ_MTX_INDEP_SRT, JOBJ_USER_DEF_MTX, QuakeKind_Small, %s,\n};\n" % KIND[stem]
    s += "use crate::gr::inlines;\n\n"
    s += gen_ext(own, shared, "") + "\n"
    if uses_hsd_assert_inline:
        s += JOBJ_INLINE + "\n"
    # yakumono
    if has_yak:
        flds = tabs["yak_fields"]
        s += "/// `struct %s_YakumonoParam` (UNK_T = void* fields).\n#[repr(C)]\npub struct YakumonoParam {\n" % ("grT")
        for fl in flds:
            s += "    pub %s: *mut c_void,\n" % fl
        s += "}\n\n"
        s += "static YAKUMONO_PARAM: AtomicPtr<YakumonoParam> = AtomicPtr::new(null_mut());\n\n"
        s += "/// The TU-static `yakumono_param` (set by this stage's `OnInit`).\npub fn yakumono_param() -> *mut YakumonoParam {\n    YAKUMONO_PARAM.load(Ordering::Relaxed)\n}\n\n"
        s += "#[cfg(test)]\npub fn set_yakumono_param(p: *mut YakumonoParam) {\n    YAKUMONO_PARAM.store(p, Ordering::Relaxed);\n}\n\n"
    if tabs["float_global"]:
        s += "/// `float grTe_804D6A48;` (the cursor-rotation accumulator).\npub static mut grTe_804D6A48: f32 = 0.0;\n\n"
    if tabs["joints"]:
        nm, tr = tabs["joints"]
        s += "pub static %s: [GrJoint; %d] = [\n" % (nm, len(tr))
        for x, y, z in tr:
            s += "    GrJoint { x: %d, y: %d, z: %d },\n" % (x, y, z)
        s += "];\n\n"
    if tabs["s16tab"]:
        nm, vals = tabs["s16tab"]
        s += "pub static %s: [i16; %d] = [%s];\n\n" % (nm, len(vals), ", ".join(str(v) for v in vals))

    # functions
    fn_names = [f["name"] for f in fns]
    reexp = []
    bodies = []
    for f in fns:
        key = (stem, f["name"])
        ps, rr = rust_sig(f)
        if key in CUSTOM:
            body, pure = CUSTOM[key], False
        else:
            body, pure = derive_body(stem, f, None)
        if body is None:
            raise SystemExit("no pattern for %s::%s: %s" % (stem, f["name"], f["body"]))
        if pure and f["name"] in trivial_names:
            reexp.append(f["name"])
            continue
        # unsafe extern "C": table entries need that ABI
        # C bool return -> i32 literal
        bodies.append("pub unsafe extern \"C\" fn %s(%s)%s {\n    %s\n}\n" % (f["name"], ps, rr, body.replace("\n    ", "\n    ") if body else ""))
    if reexp:
        s += "// Pure constant/empty callbacks already ported in gr::trivial (night 3 r3): re-exported.\n"
        s += "pub use crate::gr::trivial::%s::{%s};\n\n" % (stem, ", ".join(reexp))

    # tables
    s += gen_tables(stem, tabs, fns)
    s += "\n"
    # unused-parameter hygiene: allow unused variables
    s += "\n".join(bodies)
    s = s.replace("#![allow(non_snake_case,", "#![allow(unused_variables, unused_mut, non_snake_case,", 1)
    return s, fns, tabs, used


def gen_tables(stem, tabs, fns):
    def ref(x):
        x = x.strip()
        if x in ("0", "NULL"):
            return "None"
        return "Some(%s)" % x
    s = "static TABLES_INIT: Once = Once::new();\n"
    s += "static mut CALLBACKS: MaybeUninit<[StageCallbacks; %d]> = MaybeUninit::uninit();\n" % tabs["cb_len"]
    s += "static mut STAGE_DATA: MaybeUninit<StageData> = MaybeUninit::uninit();\n\n"
    s += "fn init_tables() {\n    TABLES_INIT.call_once(|| unsafe {\n"
    s += "        let mut t: [StageCallbacks; %d] = core::mem::zeroed();\n" % tabs["cb_len"]
    for i, e in enumerate(tabs["cb_ents"]):
        if e is None:
            continue
        s += "        t[%d] = StageCallbacks {\n            on_init: %s,\n            callback1: %s,\n            gobj_proc: %s,\n            callback3: %s,\n            anon_10: StageCallbacks_anon_135 { flags: 0x%08X },\n        };\n" % (
            i, ref(e[0]), ref(e[1]), ref(e[2]), ref(e[3]), evalc(e[4]))
    s += "        CALLBACKS.write(t);\n"
    items = tabs["sd_items"] + ["0"] * (len(SD_FIELDS) - len(tabs["sd_items"]))
    d = dict(zip(SD_FIELDS, items))
    s += "        let mut sd: StageData = core::mem::zeroed();\n"
    s += "        sd.grkind = %s;\n" % d["grkind"]
    s += "        sd.callbacks = CALLBACKS.as_mut_ptr().cast();\n"
    m = re.match(r'"(.*)"$', d["data1"])
    s += "        sd.data1 = c\"%s\".as_ptr() as *mut i8;\n" % m.group(1)
    for fld in ("on_init", "on_demo_init", "on_load", "on_start", "callback4", "on_touch_line", "on_check_shadow_render"):
        s += "        sd.%s = %s;\n" % (fld, ref(d[fld]))
    s += "        sd.flags2 = %s;\n" % evalc(d["flags2"])
    if d["joints"] not in ("0", "NULL"):
        s += "        sd.joints = %s.as_ptr() as *mut GrJoint;\n" % d["joints"]
    s += "        sd.joint_count = %s;\n" % evalc(d["joint_count"])
    s += "        STAGE_DATA.write(sd);\n    });\n}\n\n"
    s += "/// The TU's `StageCallbacks %s[%d]` (built on first use; stable address).\n" % (tabs["cb_name"], tabs["cb_len"])
    s += "pub fn stage_callbacks() -> *mut StageCallbacks {\n    init_tables();\n    unsafe { CALLBACKS.as_mut_ptr().cast() }\n}\n\n"
    s += "/// The TU's `StageData %s`.\npub fn stage_data() -> *mut StageData {\n    init_tables();\n    unsafe { STAGE_DATA.as_mut_ptr() }\n}\n" % tabs["sd_name"]
    s += "/// Entries of the callback table.\npub const STAGE_CALLBACKS_LEN: usize = %d;\n" % tabs["cb_len"]
    return s


JOBJ_INLINE = '''/// The jobj.h inlines grtest.c expands (`HSD_ASSERT` is the non-MUST_MATCH `__assert`).
pub mod jobj_inline {
    use super::*;

    fn hsd_assert(cond: bool, what: &str) {
        if !cond {
            panic!("__assert(jobj.h): {what}");
        }
    }
    /// `HSD_JObjMtxIsDirty`.
    unsafe fn HSD_JObjMtxIsDirty(j: *mut HSD_JObj) -> bool {
        hsd_assert(!j.is_null(), "jobj");
        let mut result = false;
        if (*j).flags & JOBJ_USER_DEF_MTX as u32 == 0 && (*j).flags & JOBJ_MTX_DIRTY as u32 != 0 {
            result = true;
        }
        result
    }
    /// The `HSD_JObjSetMtxDirty` macro.
    unsafe fn set_mtx_dirty(j: *mut HSD_JObj) {
        if !j.is_null() && !HSD_JObjMtxIsDirty(j) {
            ext::HSD_JObjSetMtxDirtySub(j);
        }
    }
    unsafe fn dirty(j: *mut HSD_JObj) {
        if (*j).flags & JOBJ_MTX_INDEP_SRT as u32 == 0 {
            set_mtx_dirty(j);
        }
    }
    pub unsafe fn HSD_JObjSetRotationZ(j: *mut HSD_JObj, z: f32) {
        hsd_assert(!j.is_null(), "jobj");
        hsd_assert((*j).flags & 131072 == 0, "!(jobj->flags & JOBJ_USE_QUATERNION)");
        (*j).rotate.z = z;
        dirty(j);
    }
    pub unsafe fn HSD_JObjSetScaleX(j: *mut HSD_JObj, v: f32) {
        hsd_assert(!j.is_null(), "jobj");
        (*j).scale.x = v;
        dirty(j);
    }
    pub unsafe fn HSD_JObjSetScaleY(j: *mut HSD_JObj, v: f32) {
        hsd_assert(!j.is_null(), "jobj");
        (*j).scale.y = v;
        dirty(j);
    }
    pub unsafe fn HSD_JObjSetScaleZ(j: *mut HSD_JObj, v: f32) {
        hsd_assert(!j.is_null(), "jobj");
        (*j).scale.z = v;
        dirty(j);
    }
}
'''


# ---------------------------------------------------------------------------------------------
# C side
# ---------------------------------------------------------------------------------------------
def extract_inline(t, name):
    m = re.search(r"static inline [\w\*\s]+?\b%s\(" % name, t)
    assert m, name
    i = t.index("{", m.end())
    j = match_brace(t, i)
    return t[m.start():j + 1]


def c_prelude():
    inl = read(os.path.join(ROOT, "src/melee/gr/inlines.h"))
    jh = read(os.path.join(ROOT, "src/sysdolphin/baselib/jobj.h"))
    inline_fns = "\n\n".join(extract_inline(inl, n) for n in
                             ("Ground_StartMapAnim", "Ground_InitMapCollAndAnim", "Ground_UpdateWindAndMapColl",
                              "Ground_SetupStageCallbacks", "Ground_InitTargetStage"))
    jfns = "\n\n".join(extract_inline(jh, n) for n in
                       ("HSD_JObjMtxIsDirty", "HSD_JObjSetRotationZ", "HSD_JObjSetScaleX", "HSD_JObjSetScaleY", "HSD_JObjSetScaleZ"))
    mm = re.search(r"#define HSD_JObjSetMtxDirty\(jobj\).*?\n    \}\n", jh, flags=re.S)
    mtxmacro = mm.group(0)
    # GET_* macros as in the game headers, minus the asserting accessors
    stubs = "\n".join(c["c"] for c in CALLEES if c["c"])
    p = r'''/* generated by gen_grt_stages.py: the types and recorders the unmodified gr/grt*.c compile against.
 * i686 build: only logical values are compared (no GameCube offsets are needed by these TUs). */
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <stddef.h>
#include <string.h>
#include <stdint.h>
typedef int s32; typedef unsigned int u32; typedef short s16; typedef unsigned short u16; typedef unsigned char u8; typedef signed char s8;
typedef float f32; typedef int bool; typedef int enum_t;
#define true 1
#define false 0
#define UNK_T void*
#define PAD_STACK(n)
#define HSD_ASSERT(line, cond) ((cond) ? ((void) 0) : __assert(__FILE__, line, #cond))
void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s %u %s\n", f, l, c); exit(3); }

#define JOBJ_HIDDEN (1 << 4)
#define JOBJ_MTX_DIRTY (1 << 6)
#define JOBJ_USER_DEF_MTX (1 << 23)
#define JOBJ_MTX_INDEP_SRT (1 << 25)
#define JOBJ_USE_QUATERNION (1 << 17)
#define HSD_PAD_A 0x0100
#define QuakeKind_Small 2
#define CollLine_Floor 1
#define CollLine_Ceiling 2
#define CollLine_RightWall 4
#define CollLine_LeftWall 8
#define Gr_Kind_TDrmario 44
#define Gr_Kind_TEmblem 64
#define Gr_Kind_Test 1
#define Gr_Kind_TFalco 45
#define Gr_Kind_TFox 46
#define Gr_Kind_TGamewatch 63
#define Gr_Kind_TGanon 65
#define Gr_Kind_TIceclimber 47

typedef struct HSD_GObj { void* hsd_obj; void* user_data; } HSD_GObj;
typedef HSD_GObj Ground_GObj;
typedef HSD_GObj Item_GObj;
typedef struct HSD_GObjProc { int x; } HSD_GObjProc;
typedef void (*HSD_GObjEvent)(HSD_GObj*);
typedef void (*GObj_RenderFunc)(HSD_GObj*, int);
typedef struct Vec3 { f32 x, y, z; } Vec3;
typedef struct HSD_JObj { u32 flags; struct { f32 x, y, z, w; } rotate; Vec3 scale; Vec3 translate; } HSD_JObj;
typedef struct DynamicsDesc { int x; } DynamicsDesc;
typedef struct Ground { int x0; void* gobj; HSD_GObjEvent x8_callback; HSD_GObjEvent xC_callback; int map_id; HSD_GObjEvent x1C_callback; } Ground;
typedef struct Item { char pad[0xDD4]; union { struct { HSD_JObj* x0; HSD_JObj* x4; } mato; } xDD4_itemVar; } Item;
typedef struct StageCallbacks { HSD_GObjEvent on_init; bool (*callback1)(HSD_GObj*); HSD_GObjEvent gobj_proc; HSD_GObjEvent callback3; u32 flags; } StageCallbacks;
typedef struct GrJoint { s16 x, y, z; } GrJoint;
typedef struct StageData {
    int grkind; StageCallbacks* callbacks; const char* data1; void (*on_init)(void); void (*on_demo_init)(bool);
    void (*on_load)(void); void (*on_start)(void); bool (*callback4)(void); DynamicsDesc* (*on_touch_line)(enum_t);
    bool (*on_check_shadow_render)(Vec3*, int, HSD_JObj*); u32 flags2; GrJoint* joints; int joint_count;
} StageData;
typedef struct HSD_PadStatus { u32 button, last_button, trigger; } HSD_PadStatus;
typedef struct StageInfo { struct { unsigned b0 : 1, b1 : 1, b2 : 1, b3 : 1, b4 : 1, b5 : 1, b6 : 1, b7 : 1; } unk8C; } StageInfo;

#define GET_JOBJ(gobj) ((HSD_JObj*) (gobj)->hsd_obj)
#define GET_ITEM(gobj) ((Item*) (gobj)->user_data)
#define HSD_GObjGetUserData(g) ((g)->user_data)
#define GET_GROUND(gobj) ((Ground*) HSD_GObjGetUserData(gobj))

/* ---- world ---- */
#define NCASE_ARGS 8
static int nonull;
static HSD_GObj fx_G[3]; static Ground fx_R[3]; static HSD_JObj fx_J[3];
static HSD_GObj fx_IG[2]; static Item fx_IT[2];
static Vec3 fx_P;
static void* fx_yak[4]; static DynamicsDesc fx_D[4];
StageInfo stage_info;
HSD_PadStatus HSD_PadMasterStatus[4];
static void sentinel_fn(HSD_GObj* g) { (void) g; }

/* ---- trace + registry ---- */
static char tbuf[1 << 16]; static int tlen; static int muted;
static void emit(const char* fmt, ...)
{
    va_list ap; int n;
    if (muted) return;
    va_start(ap, fmt);
    n = vsnprintf(tbuf + tlen, sizeof tbuf - tlen, fmt, ap);
    va_end(ap);
    tlen += n;
}
static uint32_t st = 1;
static uint32_t rnd(void) { st ^= st << 13; st ^= st >> 17; st ^= st << 5; return st; }
static uint32_t fb(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float rf(void) { return (float) ((int) (rnd() % 2000) - 1000) / 16.0f; }
typedef struct { const void* p; const char* n; } Reg;
static Reg reg[256]; static int nreg;
static void reg_add(const void* p, const char* n) { reg[nreg].p = p; reg[nreg].n = n; nreg++; }
static const char* sym(const void* p)
{
    int i;
    if (p == NULL) return "0";
    for (i = 0; i < nreg; i++) if (reg[i].p == p) return reg[i].n;
    return "?";
}
static const char* fname(const void* p) { return sym(p); }

/* ---- game inlines (verbatim) ---- */
''' + stubs + "\n\n" + "void grDisplay_801C5DB0(HSD_GObj* g, int pass) { (void) g; (void) pass; }\n"
    return p, inline_fns, jfns, mtxmacro


def make_stub_includes(stem, src, inc_dir, inline_fns, jfns, mtxmacro):
    for m in re.finditer(r'#include\s+[<"]([^>"]+)[>"]', src):
        name = m.group(1)
        pth = os.path.join(inc_dir, name)
        os.makedirs(os.path.dirname(pth), exist_ok=True)
        if name == "inlines.h":
            write(pth, "/* verbatim inlines of src/melee/gr/inlines.h (used by the TU) */\n" + inline_fns + "\n")
        elif name == stem + ".h":
            hd = read(os.path.join(ROOT, "src/melee/gr/%s.h" % stem))
            hd = chr(10).join(l for l in hd.split(chr(10)) if not l.startswith("#include"))
            write(pth, "/* the TU own header, includes dropped */" + chr(10) + hd + chr(10))
        elif not os.path.exists(pth):
            write(pth, "")


def driver(stem, fns, tabs, used):
    """C driver for one TU"""
    has_yak = tabs["yak_fields"] is not None
    prime = None
    for f in fns:
        if "yakumono_param =" in f["body"]:
            prime = f["name"]
    d = '#include "prelude.h"\n#include "jobj_inl.h"\n#include "src/melee/gr/%s.c"\n\n' % stem
    d += "static unsigned long long fnv(const char* s, int n) { unsigned long long h = 1469598103934665603ULL; int i; for (i = 0; i < n; i++) { h ^= (unsigned char) s[i]; h *= 1099511628211ULL; } return h; }\n"
    d += "static int verbose;\n"
    d += "static void begin_case(const char* name, int c, uint32_t seed) { tlen = 0; tbuf[0] = 0; st = seed; emit(\"%s #%d\", name, c); }\n"
    d += "static void end_case(void) { if (verbose) { printf(\"%s\\n\", tbuf); } else { char* sp = strchr(tbuf, ' '); (void) sp; printf(\"%.*s %016llx\\n\", (int) (strchr(tbuf, ' ') - tbuf), tbuf, fnv(tbuf, tlen)); } }\n"
    # the name token in begin_case is "<fn> #<c>": print "<fn> #<c> <hash>"
    d = d.replace('static void end_case(void) { if (verbose) { printf("%s\\n", tbuf); } else { char* sp = strchr(tbuf, \' \'); (void) sp; printf("%.*s %016llx\\n", (int) (strchr(tbuf, \' \') - tbuf), tbuf, fnv(tbuf, tlen)); } }\n',
                  'static void end_case(void) { if (verbose) { printf("%s\\n", tbuf); } else { char* sp = strchr(tbuf, \'#\'); sp = strchr(sp, \' \') ? strchr(sp, \' \') : tbuf + tlen; printf("%.*s %016llx\\n", (int) (sp - tbuf), tbuf, fnv(tbuf, tlen)); } }\n')
    # registry
    d += "static void build_registry(void)\n{\n"
    d += '    int i;\n    char* nm;\n'
    d += '    for (i = 0; i < 3; i++) { nm = malloc(4); sprintf(nm, "G%d", i); reg_add(&fx_G[i], nm); nm = malloc(4); sprintf(nm, "R%d", i); reg_add(&fx_R[i], nm); nm = malloc(4); sprintf(nm, "J%d", i); reg_add(&fx_J[i], nm); }\n'
    d += '    for (i = 0; i < 2; i++) { nm = malloc(4); sprintf(nm, "I%d", i); reg_add(&fx_IG[i], nm); }\n'
    d += '    for (i = 0; i < 4; i++) { nm = malloc(4); sprintf(nm, "D%d", i); reg_add(&fx_D[i], nm); }\n'
    d += '    reg_add(&fx_P, "P"); reg_add(fx_yak, "Y"); reg_add(sentinel_fn, "S"); reg_add(grDisplay_801C5DB0, "grDisplay_801C5DB0");\n'
    d += '    reg_add(&stage_info, "SI");\n'
    for f in fns:
        d += '    reg_add((const void*) %s, "%s");\n' % (f["name"], f["name"])
    d += "}\n\n"
    # reset world
    d += "static void reset_world(void)\n{\n    int i;\n"
    d += "    for (i = 0; i < 3; i++) {\n"
    d += "        fx_G[i].hsd_obj = &fx_J[i]; fx_G[i].user_data = &fx_R[i];\n"
    d += "        memset(&fx_R[i], 0, sizeof fx_R[i]); fx_R[i].map_id = (int) (rnd() % 64);\n"
    d += "        fx_R[i].x8_callback = sentinel_fn; fx_R[i].xC_callback = sentinel_fn; fx_R[i].x1C_callback = sentinel_fn;\n"
    d += "        memset(&fx_J[i], 0, sizeof fx_J[i]);\n"
    d += "        fx_J[i].flags = rnd() & (0x10u | 0x40u | 0x2000000u | 0x800000u | 0xFF00u);\n"
    d += "        fx_J[i].scale.x = rf(); fx_J[i].scale.y = rf(); fx_J[i].scale.z = rf(); fx_J[i].rotate.z = rf();\n"
    d += "        fx_J[i].translate.x = rf(); fx_J[i].translate.y = rf(); fx_J[i].translate.z = rf();\n"
    d += "    }\n"
    d += "    for (i = 0; i < 2; i++) {\n        memset(&fx_IT[i], 0, sizeof fx_IT[i]);\n        fx_IG[i].hsd_obj = &fx_J[i]; fx_IG[i].user_data = &fx_IT[i];\n"
    d += "        fx_IT[i].xDD4_itemVar.mato.x0 = &fx_J[(i + 1) % 3]; fx_IT[i].xDD4_itemVar.mato.x4 = &fx_J[i];\n    }\n"
    d += "    fx_P.x = rf(); fx_P.y = rf(); fx_P.z = rf();\n"
    d += "    for (i = 0; i < 4; i++) fx_yak[i] = &fx_D[i];\n"
    d += "    stage_info.unk8C.b4 = (int) (rnd() & 1); stage_info.unk8C.b5 = (int) (rnd() & 1);\n"
    d += "    for (i = 0; i < 4; i++) { HSD_PadMasterStatus[i].button = rnd() & 0xFFFF; HSD_PadMasterStatus[i].last_button = 0; HSD_PadMasterStatus[i].trigger = rnd() & 0xFFFF; }\n"
    if tabs["float_global"]:
        d += "    { static const float ft[] = { 1.2217305f, -1.2217305f, 1.1344640f, -1.1344640f, 0.0f }; unsigned r = rnd(); grTe_804D6A48 = (r & 1) ? ft[(r >> 1) % 5] : (float) ((int) ((r >> 4) % 2600) - 1300) / 1000.0f; }\n"
    d += "}\n\n"
    # snapshot
    d += "static void snapshot(void)\n{\n    int i;\n"
    d += '    for (i = 0; i < 3; i++) {\n'
    d += '        emit(" |G%d map=%u x8=%s xC=%s x1C=%s", i, (unsigned) fx_R[i].map_id, fname((void*) fx_R[i].x8_callback), fname((void*) fx_R[i].xC_callback), fname((void*) fx_R[i].x1C_callback));\n'
    d += '        emit(" J%d f=%x s=%08x,%08x,%08x rz=%08x t=%08x,%08x,%08x", i, fx_J[i].flags, fb(fx_J[i].scale.x), fb(fx_J[i].scale.y), fb(fx_J[i].scale.z), fb(fx_J[i].rotate.z), fb(fx_J[i].translate.x), fb(fx_J[i].translate.y), fb(fx_J[i].translate.z));\n'
    d += '    }\n'
    d += '    for (i = 0; i < 2; i++) emit(" I%d x0=%s x4=%s", i, sym(fx_IT[i].xDD4_itemVar.mato.x0), sym(fx_IT[i].xDD4_itemVar.mato.x4));\n'
    d += '    emit(" si=%d%d P=%08x,%08x,%08x", stage_info.unk8C.b4, stage_info.unk8C.b5, fb(fx_P.x), fb(fx_P.y), fb(fx_P.z));\n'
    if tabs["float_global"]:
        d += '    emit(" gt=%08x", fb(grTe_804D6A48));\n'
    if has_yak:
        d += '    emit(" yak=%s", sym(yakumono_param));\n'
    d += "}\n\n"
    # tables
    d += "static void dump_tables(void)\n{\n    int i;\n    begin_case(\"tables\", 0, 1);\n"
    d += '    for (i = 0; i < %d; i++) emit(" [%%d %%s %%s %%s %%s %%08x]", i, fname((void*) %s[i].on_init), fname((void*) %s[i].callback1), fname((void*) %s[i].gobj_proc), fname((void*) %s[i].callback3), %s[i].flags);\n' % (
        tabs["cb_len"], tabs["cb_name"], tabs["cb_name"], tabs["cb_name"], tabs["cb_name"], tabs["cb_name"])
    sd = tabs["sd_name"]
    d += '    emit(" SD kind=%%d cb=%%s data1=%%s", %s.grkind, %s.callbacks == %s ? "tbl" : "?", %s.data1);\n' % (sd, sd, tabs["cb_name"], sd)
    d += '    emit(" fns=%%s,%%s,%%s,%%s,%%s,%%s,%%s", fname((void*) %s.on_init), fname((void*) %s.on_demo_init), fname((void*) %s.on_load), fname((void*) %s.on_start), fname((void*) %s.callback4), fname((void*) %s.on_touch_line), fname((void*) %s.on_check_shadow_render));\n' % ((sd,) * 7)
    d += '    emit(" flags2=%%u count=%%d", %s.flags2, %s.joint_count);\n' % (sd, sd)
    d += '    for (i = 0; i < %s.joint_count; i++) emit(" j%%d=%%d,%%d,%%d", i, %s.joints[i].x, %s.joints[i].y, %s.joints[i].z);\n' % ((sd,) * 4)
    if tabs["s16tab"]:
        nm, vals = tabs["s16tab"]
        d += '    for (i = 0; i < %d; i++) emit(" s%%d", %s[i]);\n' % (len(vals), nm)
    d += "    end_case();\n}\n\n"
    # main
    d += "int main(int argc, char** argv)\n{\n    int c;\n    verbose = argc > 1;\n    build_registry();\n    dump_tables();\n"
    for f in fns:
        d += case_loop_c(f, prime if (has_yak and "yakumono_param" in f["body"] and "yakumono_param =" not in f["body"]) else None)
    d += "    return 0;\n}\n"
    return d


def argexpr_c(kind):
    return {
        "G": "&fx_G[rnd() % 3]",
        "R": "&fx_R[rnd() % 3]",
        "P": "((rnd() & 3) ? &fx_P : NULL)",
        "J": "((rnd() & 3) ? &fx_J[rnd() % 3] : NULL)",
        "b": "(int) (rnd() & 1)",
        "i": "ARGI",
        "f": "rf()",
    }[kind]


def case_loop_c(f, prime):
    n = f["name"]
    ps = f["params"]
    out = "    for (c = 0; c < %d; c++) {\n        uint32_t seed = 0x9E3779B9u * (uint32_t) (c + 1) + %d;\n" % (K, hash_name(n))
    out += "        begin_case(\"%s\", c, seed | 1);\n        reset_world();\n" % n
    out += "        nonull = %d;\n" % (1 if n == "grTest_80206E30" else 0)
    if prime:
        out += "        muted = 1; %s(); muted = 0;\n" % prime
    # args
    names = []
    fmt = []
    for i, (ty, pn) in enumerate(ps):
        kind = KINDOF[ty]
        if n == "grTIceClimber_80221208" and i == 0:
            expr = "&fx_IG[rnd() % 2]"
        elif kind == "i":
            if n.endswith("_OnInit") and False:
                expr = "0"
            expr = arg_int_c(f, i)
        else:
            expr = argexpr_c(kind)
        out += "        %s a%d = %s;\n" % (ctype(ty), i, expr)
        names.append("a%d" % i)
        fmt.append(kind)
    # print args
    for i, k in enumerate(fmt):
        out += "        emit(\" a%d=%s\", %s);\n" % (i, {"G": "%s", "R": "%s", "P": "%s", "J": "%s", "b": "%d", "i": "%d", "f": "%08x"}[k],
                                                 {"G": "sym(a%d)", "R": "sym(a%d)", "P": "sym(a%d)", "J": "sym(a%d)", "b": "a%d", "i": "a%d", "f": "fb(a%d)"}[k] % i)
    call = "%s(%s)" % (n, ", ".join(names))
    r = f["ret"]
    if r == "void":
        out += "        %s;\n" % call
    elif r in ("bool", "int", "s32", "enum_t"):
        out += "        { int rv = %s; emit(\" =>%%d\", rv); }\n" % call
    else:
        out += "        { void* rv = (void*) %s; emit(\" =>%%s\", sym(rv)); }\n" % call
    out += "        snapshot();\n        end_case();\n    }\n"
    return out


def hash_name(n):
    h = 0
    for ch in n:
        h = (h * 131 + ord(ch)) & 0xFFFFFFFF
    return h


def ctype(ty):
    return {"Ground_GObj*": "Ground_GObj*", "HSD_GObj*": "HSD_GObj*", "Item_GObj*": "Item_GObj*", "Ground*": "Ground*",
            "Vec3*": "Vec3*", "HSD_JObj*": "HSD_JObj*", "bool": "bool", "int": "int", "enum_t": "enum_t", "s32": "s32", "f32": "f32"}[ty]


def arg_int_c(f, i):
    n = f["name"]
    pn = f["params"][i][1]
    if "gobj_id" in pn or pn in ("id", "arg0") and f["ret"].startswith(("HSD_GObj", "Ground_GObj")):
        return "(int) (rnd() % 4)"
    if f["ret"].startswith("DynamicsDesc") or f["ret"].startswith("struct"):
        return "((rnd() % 6) == 0 ? -1 : (int) (rnd() % 1000))"
    return "(int) (rnd() % 40) - 8"


# ---------------------------------------------------------------------------------------------
# Rust test driver
# ---------------------------------------------------------------------------------------------
def rust_case(stem, f, prime, tabs):
    n = f["name"]
    ps = f["params"]
    out = "    for c in 0..%d {\n        let seed = 0x9E3779B9u32.wrapping_mul((c + 1) as u32).wrapping_add(%d);\n" % (K, hash_name(n))
    out += "        begin_case(\"%s\", c, seed | 1);\n        reset_world(%s);\n" % (n, "true" if tabs["float_global"] else "false")
    out += "        NONULL.store(%s, std::sync::atomic::Ordering::Relaxed);\n" % ("true" if n == "grTest_80206E30" else "false")
    if prime:
        out += "        set_muted(true);\n        melee_world::tu::%s::%s();\n        set_muted(false);\n" % (stem, prime)
    names = []
    kinds = []
    for i, (ty, pn) in enumerate(ps):
        kind = KINDOF[ty]
        if n == "grTIceClimber_80221208" and i == 0:
            expr = "w().ig.add((rnd() % 2) as usize)"
        elif kind == "i":
            expr = arg_int_r(f, i)
        else:
            expr = {
                "G": "w().g.add((rnd() % 3) as usize)",
                "R": "w().r.add((rnd() % 3) as usize)",
                "P": "if rnd() & 3 != 0 { addr_of_mut!(w().p) } else { null_mut() }",
                "J": "if rnd() & 3 != 0 { w().j.add((rnd() % 3) as usize) } else { null_mut() }",
                "b": "(rnd() & 1) as i32",
                "f": "rf()",
            }[kind]
        # C evaluates the two rnd() calls of J/G args in unspecified order?  Each is a single call in
        # one expression except J, which has two sequenced by the ?: operator (condition first).
        out += "        let a%d = %s;\n" % (i, expr)
        names.append("a%d" % i)
        kinds.append(kind)
    for i, k in enumerate(kinds):
        out += "        rec(&format!(\" a%d={}\", %s));\n" % (i, {"G": "sym(a%d as usize)", "R": "sym(a%d as usize)", "P": "sym(a%d as usize)", "J": "sym(a%d as usize)",
                                                              "b": "a%d", "i": "a%d", "f": "format!(\"{:08x}\", a%d.to_bits())"}[k] % i)
    call = "melee_world::tu::%s::%s(%s)" % (stem, n, ", ".join(names))
    r = f["ret"]
    if r == "void":
        out += "        %s;\n" % call
    elif r in ("bool", "int", "s32", "enum_t"):
        out += "        { let rv: i32 = %s; rec(&format!(\" =>{}\", rv)); }\n" % call
    else:
        out += "        { let rv = %s; rec(&format!(\" =>{}\", sym(rv as usize))); }\n" % call
    out += "        snapshot(%s, %s);\n        end_case(&mut out);\n    }\n" % (
        "true" if tabs["float_global"] else "false", ("Some(melee_world::tu::%s::yakumono_param() as usize)" % stem) if tabs["yak_fields"] else "None")
    return out


def arg_int_r(f, i):
    pn = f["params"][i][1]
    if "gobj_id" in pn or pn in ("id", "arg0") and f["ret"].startswith(("HSD_GObj", "Ground_GObj")):
        return "(rnd() % 4) as i32"
    if f["ret"].startswith("DynamicsDesc") or f["ret"].startswith("struct"):
        return "if rnd() % 6 == 0 { -1 } else { (rnd() % 1000) as i32 }"
    return "(rnd() % 40) as i32 - 8"


def gen_rust_test(info):
    s = '''//! State oracle for the gr/grt*.c per-stage TU ports (generated by cref/gen_grt_stages.py).
//!
//! The reference side is the UNMODIFIED game C (src/melee/gr/<stem>.c plus the verbatim
//! gr/inlines.h and jobj.h inlines) compiled natively with recorder stubs for every callee
//! (cref/gen_grt_stages_ref.sh -> tests/data/tu_<stem>_ref.txt, one FNV-1a hash per case).
//! This file replays the identical scripted world on the Rust port: a seeded fixture of three
//! gobj/Ground/JObj triples (+2 items, stage_info, pad status), every function called with seeded
//! arguments, the callee call sequence (with arguments and seeded return values) and the whole
//! fixture state afterwards (Ground callbacks, JObj flags/scale/rotation, stage_info bits, the
//! TU's own statics) hashed into the case line. Also the StageCallbacks/StageData tables.
#![allow(non_snake_case, non_upper_case_globals, static_mut_refs, unused_variables, unused_unsafe, clippy::all)]
use core::ffi::{c_void, CStr};
use core::ptr::{addr_of_mut, null_mut};
use std::cell::{Cell, RefCell};
use std::sync::Mutex;

use melee_types::*;
use melee_world::gr::inlines;

static LOCK: Mutex<()> = Mutex::new(());
static NONULL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub struct World {
    pub g: *mut HSD_GObj,
    pub r: *mut Ground,
    pub j: *mut HSD_JObj,
    pub ig: *mut HSD_GObj,
    pub it: *mut Item,
    pub p: Vec3,
    pub yak: *mut [*mut DynamicsDesc; 4],
    pub d: *mut DynamicsDesc,
    pub stage_info: *mut StageInfo,
    pub pad: *mut HSD_PadStatus,
}

thread_local! {
    static WP: Cell<*mut World> = const { Cell::new(null_mut()) };
    static TR: RefCell<String> = const { RefCell::new(String::new()) };
    static SEED: Cell<u32> = const { Cell::new(1) };
    static MUTED: Cell<bool> = const { Cell::new(false) };
    static REG: RefCell<std::vec::Vec<(usize, String)>> = const { RefCell::new(std::vec::Vec::new()) };
}

fn w() -> &'static mut World {
    WP.with(|p| {
        if p.get().is_null() {
            unsafe {
                let z = |n: usize| -> *mut u8 { Box::leak(vec![0u64; n].into_boxed_slice()).as_mut_ptr().cast() };
                let wd = World {
                    g: z(3 * core::mem::size_of::<HSD_GObj>() / 8 + 8).cast(),
                    r: z(3 * core::mem::size_of::<Ground>() / 8 + 8).cast(),
                    j: z(3 * core::mem::size_of::<HSD_JObj>() / 8 + 8).cast(),
                    ig: z(2 * core::mem::size_of::<HSD_GObj>() / 8 + 8).cast(),
                    it: z(2 * core::mem::size_of::<Item>() / 8 + 8).cast(),
                    p: core::mem::zeroed(),
                    yak: z(4).cast(),
                    d: z(4 * core::mem::size_of::<DynamicsDesc>() / 8 + 8).cast(),
                    stage_info: z(core::mem::size_of::<StageInfo>() / 8 + 8).cast(),
                    pad: z(4 * core::mem::size_of::<HSD_PadStatus>() / 8 + 8).cast(),
                };
                p.set(Box::leak(Box::new(wd)));
            }
        }
        unsafe { &mut *p.get() }
    })
}

fn rec(s: &str) {
    if !MUTED.with(|m| m.get()) {
        TR.with(|t| t.borrow_mut().push_str(s));
    }
}
fn set_muted(m: bool) {
    MUTED.with(|x| x.set(m));
}
fn rnd() -> u32 {
    SEED.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        s.set(x);
        x
    })
}
fn rf() -> f32 {
    (((rnd() % 2000) as i32) - 1000) as f32 / 16.0f32
}
fn sym(p: usize) -> String {
    if p == 0 {
        return "0".to_string();
    }
    REG.with(|r| r.borrow().iter().find(|e| e.0 == p).map(|e| e.1.clone()).unwrap_or_else(|| "?".to_string()))
}
fn fname(p: usize) -> String {
    sym(p)
}
fn fnv(s: &str) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    h
}
fn begin_case(name: &str, c: usize, seed: u32) {
    TR.with(|t| t.borrow_mut().clear());
    SEED.with(|s| s.set(seed));
    rec(&format!("{} #{}", name, c));
}
fn end_case(out: &mut String) {
    let tr = TR.with(|t| t.borrow().clone());
    let cut = tr.find('#').map(|i| tr[i..].find(' ').map_or(tr.len(), |k| i + k)).unwrap_or(tr.len());
    out.push_str(&format!("{} {:016x}\\n", &tr[..cut], fnv(&tr)));
    if std::env::var("GRT_VERBOSE").is_ok() {
        eprintln!("{}", tr);
    }
}
unsafe extern "C" fn sentinel_fn(_g: *mut HSD_GObj) {}

unsafe fn reset_world(float_global: bool, gt_setter: Option<&dyn Fn(f32)>) {
    let w = w();
    for i in 0..3 {
        let g = w.g.add(i);
        let r = w.r.add(i);
        let j = w.j.add(i);
        (*g).hsd_obj = j.cast();
        (*g).user_data = r.cast();
        core::ptr::write_bytes(r, 0, 1);
        (*r).map_id = rnd() % 64;
        (*r).x8_callback = Some(sentinel_fn);
        (*r).xC_callback = Some(sentinel_fn);
        (*r).x1C_callback = Some(sentinel_fn);
        core::ptr::write_bytes(j, 0, 1);
        (*j).flags = rnd() & (0x10u32 | 0x40 | 0x2000000 | 0x800000 | 0xFF00);
        (*j).scale.x = rf();
        (*j).scale.y = rf();
        (*j).scale.z = rf();
        (*j).rotate.z = rf();
        (*j).translate.x = rf();
        (*j).translate.y = rf();
        (*j).translate.z = rf();
    }
    for i in 0..2 {
        let it = w.it.add(i);
        core::ptr::write_bytes(it, 0, 1);
        let ig = w.ig.add(i);
        (*ig).hsd_obj = w.j.add(i).cast();
        (*ig).user_data = it.cast();
        (*it).xDD4_itemVar.mato.x0 = w.j.add((i + 1) % 3);
        (*it).xDD4_itemVar.mato.x4 = w.j.add(i);
    }
    w.p.x = rf();
    w.p.y = rf();
    w.p.z = rf();
    for i in 0..4 {
        (*w.yak)[i] = w.d.add(i);
    }
    (*w.stage_info).unk8C.set_b4((rnd() & 1) as u8);
    (*w.stage_info).unk8C.set_b5((rnd() & 1) as u8);
    for i in 0..4 {
        let p = w.pad.add(i);
        (*p).button = rnd() & 0xFFFF;
        (*p).last_button = 0;
        (*p).trigger = rnd() & 0xFFFF;
    }
    if float_global {
        const FT: [f32; 5] = [1.2217305f32, -1.2217305f32, 1.1344640f32, -1.1344640f32, 0.0f32];
        let r = rnd();
        let v = if r & 1 != 0 { FT[((r >> 1) % 5) as usize] } else { (((r >> 4) % 2600) as i32 - 1300) as f32 / 1000.0f32 };
        (gt_setter.unwrap())(v);
    }
}

unsafe fn snapshot_with(gt: Option<f32>, yak: Option<usize>) {
    let w = w();
    for i in 0..3 {
        let r = w.r.add(i);
        let j = w.j.add(i);
        rec(&format!(" |G{} map={} x8={} xC={} x1C={}", i, (*r).map_id, fname((*r).x8_callback.map_or(0, |f| f as usize)),
            fname((*r).xC_callback.map_or(0, |f| f as usize)), fname((*r).x1C_callback.map_or(0, |f| f as usize))));
        rec(&format!(" J{} f={:x} s={:08x},{:08x},{:08x} rz={:08x} t={:08x},{:08x},{:08x}", i, (*j).flags, (*j).scale.x.to_bits(),
            (*j).scale.y.to_bits(), (*j).scale.z.to_bits(), (*j).rotate.z.to_bits(), (*j).translate.x.to_bits(), (*j).translate.y.to_bits(), (*j).translate.z.to_bits()));
    }
    for i in 0..2 {
        let it = w.it.add(i);
        rec(&format!(" I{} x0={} x4={}", i, sym((*it).xDD4_itemVar.mato.x0 as usize), sym((*it).xDD4_itemVar.mato.x4 as usize)));
    }
    rec(&format!(" si={}{} P={:08x},{:08x},{:08x}", (*w.stage_info).unk8C.b4(), (*w.stage_info).unk8C.b5(), w.p.x.to_bits(), w.p.y.to_bits(), w.p.z.to_bits()));
    if let Some(g) = gt {
        rec(&format!(" gt={:08x}", g.to_bits()));
    }
    if let Some(y) = yak {
        rec(&format!(" yak={}", sym(y)));
    }
}

fn rf_for_hooks() -> f32 {
    rf()
}

/// Recorder hooks (the Rust twins of the C stubs in the generated prelude.h).
mod hooks {
    use super::*;
'''
    # hook fns
    for c in CALLEES:
        if not c["r"]:
            continue
        ps = split_params(c["rp"])
        rr = c["rr"]
        s += "    pub unsafe fn h_%s(%s)%s {\n        %s\n    }\n" % (c["name"], c["rp"], (" " + rr) if rr else "", c["r"].replace("W()", "w()"))
    s += "}\n\n"
    # install functions
    inl = [c for c in CALLEES if c["home"] == "inl"]
    s += "fn install_inlines() {\n    use melee_world::gr::inlines::ext::{Hooks, HOOKS};\n    *HOOKS.write().unwrap() = Hooks {\n"
    for c in inl:
        if c["name"] == "stage_info":
            s += "        stage_info: h_stage_info,\n"
        else:
            s += "        %s: hooks::h_%s,\n" % (c["name"], c["name"])
    s += "    };\n}\nunsafe fn h_stage_info() -> *mut StageInfo {\n    w().stage_info\n}\n\n"
    s += "unsafe fn h_HSD_PadMasterStatus() -> *mut HSD_PadStatus {\n    w().pad\n}\n\n"
    # fix: stage_info hook defined inline in hooks module with W() ; remove
    for stem in STEMS:
        _, fns, tabs, used = info[stem]
        own = [n for n in dict.fromkeys(used) if BY_NAME[n]["home"] == "tu" or n == "OSReport"]
        s += "fn install_%s() {\n    use melee_world::tu::%s::ext::{Hooks, HOOKS};\n    *HOOKS.write().unwrap() = Hooks {\n" % (stem, stem)
        for n in own:
            if n == "HSD_PadMasterStatus":
                s += "        HSD_PadMasterStatus: h_HSD_PadMasterStatus,\n"
            else:
                s += "        %s: hooks::h_%s,\n" % (n, n)
        s += "    };\n}\n\n"
    # per stem test
    for stem in STEMS:
        _, fns, tabs, used = info[stem]
        has_yak = tabs["yak_fields"] is not None
        prime = None
        for f in fns:
            if "yakumono_param =" in f["body"]:
                prime = f["name"]
        s += "fn run_%s() -> String {\n    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());\n    install_inlines();\n    install_%s();\n    let mut out = String::new();\n" % (stem, stem)
        s += "    REG.with(|r| r.borrow_mut().clear());\n    unsafe {\n"
        s += "    let wd = w();\n"
        s += "    let mut reg: std::vec::Vec<(usize, String)> = std::vec::Vec::new();\n"
        s += "    for i in 0..3 {\n        reg.push((wd.g.add(i) as usize, format!(\"G{}\", i)));\n        reg.push((wd.r.add(i) as usize, format!(\"R{}\", i)));\n        reg.push((wd.j.add(i) as usize, format!(\"J{}\", i)));\n    }\n"
        s += "    for i in 0..2 {\n        reg.push((wd.ig.add(i) as usize, format!(\"I{}\", i)));\n    }\n"
        s += "    for i in 0..4 {\n        reg.push((wd.d.add(i) as usize, format!(\"D{}\", i)));\n    }\n"
        s += "    reg.push((addr_of_mut!(wd.p) as usize, \"P\".to_string()));\n    reg.push((wd.yak as usize, \"Y\".to_string()));\n"
        s += "    reg.push((sentinel_fn as usize, \"S\".to_string()));\n"
        s += "    reg.push((inlines::ext::grDisplay_801C5DB0 as usize, \"grDisplay_801C5DB0\".to_string()));\n"
        s += "    reg.push((wd.stage_info as usize, \"SI\".to_string()));\n"
        for f in fns:
            s += "    reg.push((melee_world::tu::%s::%s as usize, \"%s\".to_string()));\n" % (stem, f["name"], f["name"])
        s += "    REG.with(|r| *r.borrow_mut() = reg);\n"
        # tables
        s += gen_rust_tables(stem, tabs)
        gts = "Some(&|v: f32| { melee_world::tu::%s::grTe_804D6A48 = v; })" % stem if tabs["float_global"] else "None"
        for f in fns:
            body = rust_case(stem, f, prime if (has_yak and "yakumono_param" in f["body"] and "yakumono_param =" not in f["body"]) else None, tabs)
            body = body.replace("reset_world(%s);" % ("true" if tabs["float_global"] else "false"), "reset_world(%s, %s);" % ("true" if tabs["float_global"] else "false", gts))
            gt_read = ("Some(melee_world::tu::%s::grTe_804D6A48)" % stem) if tabs["float_global"] else "None"
            body = body.replace("snapshot(%s, " % ("true" if tabs["float_global"] else "false"), "snapshot_with(%s, " % gt_read)
            s += body
        s += "    }\n    out\n}\n\n"
        s += "#[test]\nfn %s_matches_c() {\n    let got = run_%s();\n    let want = include_str!(\"data/tu_%s_ref.txt\").replace(\"\\r\\n\", \"\\n\");\n    compare(\"%s\", &got, &want);\n}\n\n" % (stem, stem, stem, stem)
    s += '''fn compare(name: &str, got: &str, want: &str) {
    let g: std::vec::Vec<&str> = got.lines().collect();
    let w: std::vec::Vec<&str> = want.lines().collect();
    for (i, (a, b)) in g.iter().zip(w.iter()).enumerate() {
        assert_eq!(a, b, "{name}: line {i} differs (rerun with GRT_VERBOSE=1 for the Rust trace)");
    }
    assert_eq!(g.len(), w.len(), "{name}: line count");
}
'''
    return s


def gen_rust_tables(stem, tabs):
    s = "    {\n        let m = melee_world::tu::%s::stage_callbacks();\n        begin_case(\"tables\", 0, 1);\n" % stem
    s += "        for i in 0..%d {\n            let e = &*m.add(i);\n" % tabs["cb_len"]
    s += "            rec(&format!(\" [{} {} {} {} {} {:08x}]\", i, fname(e.on_init.map_or(0, |f| f as usize)), fname(e.callback1.map_or(0, |f| f as usize)), fname(e.gobj_proc.map_or(0, |f| f as usize)), fname(e.callback3.map_or(0, |f| f as usize)), e.anon_10.flags));\n        }\n"
    s += "        let sd = &*melee_world::tu::%s::stage_data();\n" % stem
    s += "        rec(&format!(\" SD kind={} cb={} data1={}\", sd.grkind, if sd.callbacks == m { \"tbl\" } else { \"?\" }, CStr::from_ptr(sd.data1).to_str().unwrap()));\n"
    s += "        rec(&format!(\" fns={},{},{},{},{},{},{}\", fname(sd.on_init.map_or(0, |f| f as usize)), fname(sd.on_demo_init.map_or(0, |f| f as usize)), fname(sd.on_load.map_or(0, |f| f as usize)), fname(sd.on_start.map_or(0, |f| f as usize)), fname(sd.callback4.map_or(0, |f| f as usize)), fname(sd.on_touch_line.map_or(0, |f| f as usize)), fname(sd.on_check_shadow_render.map_or(0, |f| f as usize))));\n"
    s += "        rec(&format!(\" flags2={} count={}\", sd.flags2, sd.joint_count));\n"
    s += "        for i in 0..sd.joint_count {\n            let jt = &*sd.joints.add(i);\n            rec(&format!(\" j{}={},{},{}\", i, jt.x, jt.y, jt.z));\n        }\n"
    if tabs["s16tab"]:
        nm, vals = tabs["s16tab"]
        s += "        for i in 0..%d {\n            rec(&format!(\" s{}\", i16::from(melee_world::tu::%s::%s[i])));\n        }\n" % (len(vals), stem, nm)
    s += "        end_case(&mut out);\n    }\n"
    return s


# ---------------------------------------------------------------------------------------------
def main():
    triv = read(os.path.join(ROOT, "rust/melee_world/src/gr/trivial.rs"))
    trivial_by_stem = {}
    for stem in STEMS:
        m = re.search(r"pub mod %s \{(.*?)\n\}" % stem, triv, flags=re.S)
        names = set()
        if m:
            names = set(re.findall(r"pub extern \"C\" fn (\w+)\(", m.group(1)))
        trivial_by_stem[stem] = names
    global info
    info = {}
    for stem in STEMS:
        s, fns, tabs, used = gen_tu(stem, trivial_by_stem[stem])
        info[stem] = (s, fns, tabs, used)
        write(os.path.join(ROOT, "rust/melee_world/src/tu/%s.rs" % stem), s)
    write(os.path.join(ROOT, "rust/melee_world/src/gr/inlines.rs"), gen_inlines_final())
    write(os.path.join(ROOT, "rust/melee_world/tests/tu_grt_stages.rs"), gen_rust_test(info))
    # C side
    prelude, inline_fns, jfns, mtxmacro = c_prelude()
    write(os.path.join(OUT, "prelude.h"), prelude)
    write(os.path.join(OUT, "jobj_inl.h"), mtxmacro + "\n" + jfns + "\n")
    # jobj_inl.h needs HSD_JObjSetMtxDirtySub declared before use: it is defined in the prelude stubs.
    inc = os.path.join(OUT, "inc")
    for stem in STEMS:
        srcp = os.path.join(ROOT, "src/melee/gr/%s.c" % stem)
        src = read(srcp)
        make_stub_includes(stem, src, inc, inline_fns, jfns, mtxmacro)
        write(os.path.join(OUT, "src/melee/gr/%s.c" % stem), '#line 1 "src/melee/gr/%s.c"\n' % stem + src)
        write(os.path.join(OUT, "%s_drv.c" % stem), driver(stem, info[stem][1], info[stem][2], info[stem][3]))
    print("generated", len(STEMS), "TUs")


if __name__ == "__main__":
    main()
