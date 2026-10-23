# (name, C type spelling, count, Rust element type)
ARR = [
    ("OPO", "HSD_PObj", 4, "HSD_PObj"), ("OSS", "HSD_ShapeSet", 4, "HSD_ShapeSet"), ("OSA", "HSD_ShapeAnim", 4, "HSD_ShapeAnim"),
    ("OTO", "HSD_TObj", 4, "HSD_TObj"), ("OTA", "HSD_TexAnim", 4, "HSD_TexAnim"), ("OMO", "HSD_MObj", 2, "HSD_MObj"),
    ("OMA", "HSD_MatAnim", 3, "HSD_MatAnim"), ("ODO", "HSD_DObj", 3, "HSD_DObj"),
    ("OSDO", "HSD_ShapeAnimDObj", 3, "HSD_ShapeAnimDObj"), ("ORO", "HSD_RObj", 3, "HSD_RObj"),
    ("ORA", "HSD_RObjAnimJoint", 3, "HSD_RObjAnimJoint"), ("OAD", "HSD_AObjDesc", 4, "HSD_AObjDesc"),
    ("OAO", "HSD_AObj", 6, "HSD_AObj"), ("OFO", "HSD_FObj", 5, "HSD_FObj"), ("OAJ", "HSD_AnimJoint", 8, "HSD_AnimJoint"),
    ("OMJ", "HSD_MatAnimJoint", 8, "HSD_MatAnimJoint"), ("OSJ", "HSD_ShapeAnimJoint", 8, "HSD_ShapeAnimJoint"),
    ("OJT", "HSD_JObj", 8, "HSD_JObj"), ("OJNT", "HSD_Joint", 6, "HSD_Joint"),
    ("OX8T", "struct UnkStageDat_x8_t", 4, "UnkStageDat_x8_t"), ("OUSD", "UnkStageDat", 1, "UnkStageDat"),
    ("OAR", "UnkArchiveStruct", 2, "UnkArchiveStruct"),
]
PTR = [("OIT", "HSD_ImageDesc*", 2, "*mut HSD_ImageDesc"), ("OAJP", "HSD_AnimJoint*", 4, "*mut HSD_AnimJoint"),
       ("OMJP", "HSD_MatAnimJoint*", 4, "*mut HSD_MatAnimJoint"), ("OSJP", "HSD_ShapeAnimJoint*", 4, "*mut HSD_ShapeAnimJoint"),
       ("OSLOT", "HSD_AObj*", 2, "*mut HSD_AObj")]

data_c = "\n".join(f"static {t} {n}[{c}];" for n, t, c, _ in ARR)
data_c += "\n" + "\n".join(f"static {t} {n}[{c}];" for n, t, c, _ in PTR)
data_c += "\nstatic u8 OEF[8];\nstatic u32 OPAR;"
data_c += r"""
/* callbacks of the five shapes grAnime_801C6F50 calls through */
static void tcE(HSD_AObj* a) { lg_s("tcE("); lg_ptr(a); lg_s(");"); }
static void tc1(HSD_AObj* a, HSD_TObj* o, u32 f, float p) { lg_s("tc1("); lg_ptr(a); lg_ptr(o); lg_int(f); lg_f(p); lg_s(");"); }
static void tc2(HSD_AObj* a, int p) { lg_s("tc2("); lg_ptr(a); lg_int((u32) p); lg_s(");"); }
static void tc3(HSD_AObj* a, HSD_TObj* o, int p) { lg_s("tc3("); lg_ptr(a); lg_ptr(o); lg_int((u32) p); lg_s(");"); }
static void tc4(HSD_AObj* a, HSD_TObj* o, u32 f, int p) { lg_s("tc4("); lg_ptr(a); lg_ptr(o); lg_int(f); lg_int((u32) p); lg_s(");"); }
/* HSD_AObjSetRate is only ever a callback here: it takes the Callback1 shape (aobj, obj, flags, rate) */
void HSD_AObjSetRate_cbstub(HSD_AObj* a, HSD_TObj* o, u32 f, float p) { (void) o; (void) f; lg_s("HSD_AObjSetRate("); lg_ptr(a); lg_f(p); lg_s(")"); lg_c(';'); }
/* HSD_ForeachAnim: logs, then calls the callback once (the C longjmps out of the first call) */
void HSD_ForeachAnim(void* obj, HSD_Type type, HSD_TypeMask mask, void* func, AObj_Arg_Type arg_type, ...)
{
    va_list ap;
    void* p;
    lg_s("HSD_ForeachAnim("); lg_ptr(obj); lg_int((u32) type); lg_int((u32) mask); lg_ptr(func); lg_int((u32) arg_type); lg_s(")"); lg_c(';');
    va_start(ap, arg_type);
    p = va_arg(ap, void*);
    va_end(ap);
    if (rnd() % 3) ((void (*)(void*, void*)) func)((void*) &OAO[rnd() % 6], p);
}"""
use_rs = "\n".join(f"static mut {n}: [{rt}; {c}] = unsafe {{ core::mem::zeroed() }};" for n, t, c, rt in ARR)
use_rs += "\n" + "\n".join(f"static mut {n}: [{rt}; {c}] = [core::ptr::null_mut(); {c}];" for n, t, c, rt in PTR)
use_rs += """
static mut OEF: [u8; 8] = [0; 8];
static mut OPAR: u32 = 0;
use core::ptr::null_mut;
use std::panic::{catch_unwind, AssertUnwindSafe};
unsafe extern "C" fn tcE(a: *mut HSD_AObj, _o: *mut c_void, _f: u32, _p: u32) { lg_s("tcE("); lg_ptr(a as usize); lg_s(");"); }
unsafe extern "C" fn tc1(a: *mut HSD_AObj, o: *mut c_void, f: u32, p: u32) { lg_s("tc1("); lg_ptr(a as usize); lg_ptr(o as usize); lg_int(f); lg_f(f32::from_bits(p)); lg_s(");"); }
unsafe extern "C" fn tc2(a: *mut HSD_AObj, _o: *mut c_void, _f: u32, p: u32) { lg_s("tc2("); lg_ptr(a as usize); lg_int(p); lg_s(");"); }
unsafe extern "C" fn tc3(a: *mut HSD_AObj, o: *mut c_void, _f: u32, p: u32) { lg_s("tc3("); lg_ptr(a as usize); lg_ptr(o as usize); lg_int(p); lg_s(");"); }
unsafe extern "C" fn tc4(a: *mut HSD_AObj, o: *mut c_void, f: u32, p: u32) { lg_s("tc4("); lg_ptr(a as usize); lg_ptr(o as usize); lg_int(f); lg_int(p); lg_s(");"); }
/// fn_801C82E8 unwinds (the C longjmps): the test calls it under a catch.
unsafe fn call_82e8(a: *mut HSD_AObj, p: *mut *mut HSD_AObj) { let _ = catch_unwind(AssertUnwindSafe(|| t::fn_801C82E8(a, p))); }
"""

statics = [dict(c=n, rs=n, ty=t.replace("struct ", ""), n=c, fill=True) for n, t, c, _ in ARR]
statics += [dict(c=n, rs=n, kind="p", n=c) for n, t, c, _ in PTR]
statics += [dict(c="OEF", rs="OEF", kind="C", n=8),
            dict(c="grAnime_804D6958", rs="t::grAnime_804D6958", kind="f"),
            dict(c="grAnime_804D695C", rs="t::grAnime_804D695C", kind="f")]


def lst(arr, n, field, z=4):
    return dict(kind="list", arr=arr, n=n, field=field, z=z)


def pk(arr, n, field, src, m, z=4):
    return dict(kind="pick", arr=arr, n=n, field=field, src=src, m=m, z=z)


def fwd(arr, n, field, z=4):
    return dict(kind="pickfwd", arr=arr, n=n, field=field, z=z)


graph = [
    lst("OPO", 4, "next", 3), pk("OPO", 4, "u.shape_set", "OSS", 4, 0),
    dict(kind="set", arr="OPO", n=4, field="flags", c="(OPO[i].flags & 0xCFFF) | 0x1000", rs="(OPO[i].flags & 0xCFFF) | 0x1000"),
    pk("OSS", 4, "aobj", "OAO", 6, 3),
    lst("OSA", 4, "next", 3), pk("OSA", 4, "aobjdesc", "OAD", 4, 3),
    lst("OTO", 4, "next", 3), pk("OTO", 4, "aobj", "OAO", 6, 3),
    dict(kind="set", arr="OTO", n=4, field="id", c="rnd() % 3", rs="rnd() % 3"),
    lst("OTA", 4, "next", 3), pk("OTA", 4, "aobjdesc", "OAD", 4, 3), pk("OTA", 4, "imagetbl", "OIT", 2, 3),
    dict(kind="set", arr="OTA", n=4, field="id", c="rnd() % 3", rs="rnd() % 3"),
    pk("OMO", 2, "aobj", "OAO", 6, 3), pk("OMO", 2, "tobj", "OTO", 4, 3),
    lst("OMA", 3, "next", 3), pk("OMA", 3, "aobjdesc", "OAD", 4, 3), pk("OMA", 3, "texanim", "OTA", 4, 3),
    lst("ODO", 3, "next", 3), pk("ODO", 3, "pobj", "OPO", 4, 3), pk("ODO", 3, "mobj", "OMO", 2, 3), pk("ODO", 3, "aobj", "OAO", 6, 3),
    lst("OSDO", 3, "next", 3), pk("OSDO", 3, "shapeanim", "OSA", 4, 3),
    lst("ORO", 3, "next", 3), pk("ORO", 3, "aobj", "OAO", 6, 3),
    lst("ORA", 3, "next", 3), pk("ORA", 3, "aobjdesc", "OAD", 4, 3),
    pk("OAO", 6, "fobj", "OFO", 5, 3),
    lst("OFO", 5, "next", 3),
    dict(kind="set", arr="OFO", n=5, field="obj_type", c="(rnd() % 3 == 0) ? 0xC : (rnd() % 20)", rs="if rnd() % 3 == 0 { 0xC } else { (rnd() % 20) as u8 }"),
    fwd("OAJ", 8, "child", 3), fwd("OAJ", 8, "next", 3), pk("OAJ", 8, "aobjdesc", "OAD", 4, 3), pk("OAJ", 8, "robj_anim", "ORA", 3, 3),
    fwd("OMJ", 8, "child", 3), fwd("OMJ", 8, "next", 3), pk("OMJ", 8, "matanim", "OMA", 3, 3),
    fwd("OSJ", 8, "child", 3), fwd("OSJ", 8, "next", 3), pk("OSJ", 8, "shapeanimdobj", "OSDO", 3, 3),
    fwd("OJT", 8, "child", 3), fwd("OJT", 8, "next", 3), pk("OJT", 8, "aobj", "OAO", 6, 3), pk("OJT", 8, "robj", "ORO", 3, 3),
    pk("OJT", 8, "u.dobj", "ODO", 3, 2),
    fwd("OJNT", 6, "child", 3), fwd("OJNT", 6, "next", 3),
    dict(kind="set", arr="OAJP", n=4, field="__self", c="0", rs="0"),
]
graph = [g for g in graph if g.get("field") != "__self"]
# pointer arrays (kind p): slots -> elements of the joint arrays, or NULL
POINTER_SLOTS_C = """
    for (i = 0; i < 4; i++) { OAJP[i] = (rnd() % 4) ? &OAJ[i] : (HSD_AnimJoint*) 0; OMJP[i] = (rnd() % 4) ? &OMJ[i] : (HSD_MatAnimJoint*) 0; OSJP[i] = (rnd() % 4) ? &OSJ[i] : (HSD_ShapeAnimJoint*) 0; }
    for (i = 0; i < 2; i++) OIT[i] = (HSD_ImageDesc*) ((rnd() % 3) ? (void*) &OAD[i] : (void*) 0);
    for (i = 0; i < 4; i++) {
        OX8T[i].unk0 = (rnd() % 4) ? &OJNT[rnd() % 6] : (HSD_Joint*) 0;
        OX8T[i].unk4 = OAJP; OX8T[i].unk8 = OMJP; OX8T[i].unkC = OSJP;
        OX8T[i].x28 = (rnd() % 4) ? (void*) OEF : (void*) 0;
    }
    OUSD[0].unk8 = OX8T; OUSD[0].unkC = 4;
    for (i = 0; i < 2; i++) { OAR[i].unk4 = &OUSD[0]; OAR[i].unk0 = (rnd() % 3) ? (rnd() % 2 ? (HSD_Archive*) &OGO[0] : (HSD_Archive*) -1) : (HSD_Archive*) 0; }
"""
POINTER_SLOTS_RS = """
    for i in 0..4 {
        OAJP[i] = if rnd() % 4 != 0 { addr_of_mut!(OAJ[i]) } else { null_mut() };
        OMJP[i] = if rnd() % 4 != 0 { addr_of_mut!(OMJ[i]) } else { null_mut() };
        OSJP[i] = if rnd() % 4 != 0 { addr_of_mut!(OSJ[i]) } else { null_mut() };
    }
    for i in 0..2 { OIT[i] = (if rnd() % 3 != 0 { addr_of_mut!(OAD[i]) as *mut c_void } else { null_mut() }) as *mut HSD_ImageDesc; }
    for i in 0..4 {
        OX8T[i].unk0 = if rnd() % 4 != 0 { addr_of_mut!(OJNT[(rnd() % 6) as usize]) } else { null_mut() };
        OX8T[i].unk4 = addr_of_mut!(OAJP) as *mut _; OX8T[i].unk8 = addr_of_mut!(OMJP) as *mut _; OX8T[i].unkC = addr_of_mut!(OSJP) as *mut _;
        OX8T[i].x28 = if rnd() % 4 != 0 { addr_of_mut!(OEF) as *mut c_void } else { null_mut() };
    }
    OUSD[0].unk8 = addr_of_mut!(OX8T) as *mut _; OUSD[0].unkC = 4;
    for i in 0..2 { OAR[i].unk4 = addr_of_mut!(OUSD[0]); OAR[i].unk0 = if rnd() % 3 != 0 { if rnd() % 2 != 0 { addr_of_mut!(OGO[0]) as *mut _ } else { usize::MAX as *mut _ } } else { null_mut() }; }
"""

T0 = [(0, "tcE"), (1, "tc1"), (3, "tc2"), (6, "tc3"), (11, "tc4")]
FNV = {"fn_801C6EE4": "t::fn_801C6EE4", "fn_801C6F2C": "t::fn_801C6F2C", "fn_801C82E8": "t::fn_801C82E8",
       "HSD_AObjClearFlags": "t::ext::HSD_AObjClearFlags_cb", "HSD_AObjSetFlags": "t::ext::HSD_AObjSetFlags_cb",
       "HSD_AObjSetRate_cbstub": "t::ext::HSD_AObjSetRate_cb",
       "tcE": "tcE", "tc1": "tc1", "tc2": "tc2", "tc3": "tc3", "tc4": "tc4"}


def P(arr, mod, null_when=None):
    """C / Rust expressions for a pointer to arr[AI % mod], or NULL when AI == null_when"""
    c = f"&{arr}[AI[%d] %% {mod}]"
    r = f"addr_of_mut!({arr}[(a.i[%d] % {mod}) as usize])"
    return c, r


def ptr(arr, mod, idx, nullable=False, ty=None):
    c = f"&{arr}[AI[{idx}] % {mod}]"
    r = f"addr_of_mut!({arr}[(a.i[{idx}] % {mod}) as usize])"
    if nullable:
        nc = f"(AI[{(idx + 1) % 4}] & 1)"
        c = f"({nc} ? {c} : ({ty}*) 0)"
        r = f"if a.i[{(idx + 1) % 4}] & 1 != 0 {{ {r} }} else {{ null_mut() }}"
    return c, r


def entry(name, args, rs_path=None, **kw):
    d = dict(name=name, c_args=[a[0] for a in args], rs_args=[a[1] for a in args])
    if rs_path:
        d["rs_path"] = rs_path
    d.update(kw)
    return d


def cexp(c, r):
    return (c, r)


fl = ("(s32) (rnd() % 0x800)", "(rnd() % 0x800) as i32")      # call flags
funcs = []
funcs.append(entry("grAnime_801C65B0", [ptr("OAR", 2, 0, True, "UnkArchiveStruct")]))
funcs.append(entry("grAnime_801C6620", [ptr("OPO", 4, 0, True, "HSD_PObj"), ptr("OSA", 4, 2, True, "HSD_ShapeAnim")]))
funcs.append(entry("HSD_TexAnimFindById", [ptr("OTA", 4, 0, True, "HSD_TexAnim"), ("(int) (AI[1] % 3)", "(a.i[1] % 3) as i32")]))
for nm in ("grAnime_801C6710", "grAnime_801C6710_all"):
    funcs.append(entry(nm, [ptr("OTO", 4, 0, True, "HSD_TObj"), ptr("OTA", 4, 2, True, "HSD_TexAnim")]))
for nm in ("grAnime_801C67A8", "grAnime_801C67A8_inner", "grAnime_801C67A8_noinline"):
    funcs.append(entry(nm, [ptr("OMO", 2, 0, True, "HSD_MObj"), ptr("OMA", 3, 2, True, "HSD_MatAnim")]))
funcs.append(entry("grAnime_801C683C", [ptr("ODO", 3, 0, True, "HSD_DObj"), ptr("OMA", 3, 2, True, "HSD_MatAnim"), ptr("OSDO", 3, 3, True, "HSD_ShapeAnimDObj")]))
funcs.append(entry("grAnime_801C68F4", [ptr("ORO", 3, 0, True, "HSD_RObj"), ptr("ORA", 3, 2, True, "HSD_RObjAnimJoint")]))
funcs.append(entry("grAnime_801C6960", [ptr("ORO", 3, 0, True, "HSD_RObj"), ptr("ORA", 3, 2, True, "HSD_RObjAnimJoint")]))
for nm in ("grAnime_801C69FC", "grAnime_801C69FC_for_jobj", "grAnime_801C69FC_inner", "grAnime_801C69FC_noinline"):
    funcs.append(entry(nm, [ptr("OAO", 6, 0, True, "HSD_AObj")]))
funcs.append(entry("grAnime_GetAObj", [ptr("OJT", 8, 0)]))
J4 = lambda: [ptr("OJT", 8, 0, True, "HSD_JObj"), ptr("OAJ", 4, 1, True, "HSD_AnimJoint"), ptr("OMJ", 4, 2, True, "HSD_MatAnimJoint"), ptr("OSJ", 4, 3, True, "HSD_ShapeAnimJoint")]
for nm in ("grAnime_801C6A54", "grAnime_801C6A54_inner", "grAnime_801C6A54_noinline", "grAnime_801C6C0C_inner", "grAnime_801C6C0C"):
    funcs.append(entry(nm, J4(), n=300))
funcs.append(dict(name="fn_801C6EE4", c_args=["&OAO[AI[0] % 6]"], rs_args=["addr_of_mut!(OAO[(a.i[0] % 6) as usize])", "null_mut()", "0", "0"]))
funcs.append(dict(name="fn_801C6F2C", c_args=["&OAO[AI[0] % 6]"], rs_args=["addr_of_mut!(OAO[(a.i[0] % 6) as usize])", "null_mut()", "0", "0"]))
PRE_PAR = ("OPAR = rnd();", "OPAR = rnd();")
for t, cb in [(0, "tcE"), (1, "tc1"), (2, "tc2"), (3, "tc2"), (4, "tcE"), (5, "tc1"), (6, "tc3"), (7, "tc3"), (8, "tcE"), (9, "tc1"), (10, "tc4"), (11, "tc4"), (12, "tcE")]:
    funcs.append(dict(name="grAnime_801C6F50", c_args=["&OAO[AI[0] % 6]", "(void*) &OJT[AI[1] % 8]", "(u32) (rnd() % 0x10)", f"(void*) {cb}", str(t), "(void*) &OPAR"],
                      rs_args=["addr_of_mut!(OAO[(a.i[0] % 6) as usize])", "addr_of_mut!(OJT[(a.i[1] % 8) as usize]) as *mut c_void", "rnd() % 0x10", f"{cb} as *mut c_void", str(t), "addr_of_mut!(OPAR) as *mut c_void"],
                      pre_c=PRE_PAR[0], pre_rs=PRE_PAR[1], n=60))
WALK = [("grAnime_801C706C", "OTO", 4, "HSD_TObj"), ("grAnime_MObjForeachAnim", "OMO", 2, "HSD_MObj"), ("grAnime_PObjForeachAnim", "OPO", 4, "HSD_PObj"),
        ("grAnime_801C70E0", "ODO", 3, "HSD_DObj"), ("grAnime_801C70E0_inner", "ODO", 3, "HSD_DObj"), ("grAnime_801C70E0_noinline", "ODO", 3, "HSD_DObj"),
        ("grAnime_RObjForeachAnim", "ORO", 3, "HSD_RObj")]
cyc = 0
for nm, arr, m, ty in WALK:
    for t, cb in (T0[cyc % 5], T0[(cyc + 2) % 5]):
        c, r = ptr(arr, m, 0, True, ty)
        funcs.append(dict(name=nm, c_args=[c, "(s32) (rnd() % 0x800)", f"(void*) {cb}", str(t), "(void*) &OPAR"],
                          rs_args=[r, "(rnd() % 0x800) as i32", f"{cb} as *mut c_void", str(t), "addr_of_mut!(OPAR) as *mut c_void"],
                          pre_c=PRE_PAR[0], pre_rs=PRE_PAR[1], n=300))
    cyc += 1
for nm in ("grAnime_801C6F50_wrapped", "grAnime_801C7228", "grAnime_801C7228_inner", "grAnime_801C7228_noinline"):
    for t, cb in (T0[cyc % 5], T0[(cyc + 3) % 5]):
        extra_c = ["(s32) AI[2]"] if nm != "grAnime_801C6F50_wrapped" else []
        extra_r = ["a.i[2] as i32"] if nm != "grAnime_801C6F50_wrapped" else []
        funcs.append(dict(name=nm, c_args=["&OJT[AI[0] % 8]", "(s32) (rnd() % 0x8000)", f"(void*) {cb}", str(t), "(void*) &OPAR"] + extra_c,
                          rs_args=["addr_of_mut!(OJT[(a.i[0] % 8) as usize])", "(rnd() % 0x8000) as i32", f"{cb} as *mut c_void", str(t), "addr_of_mut!(OPAR) as *mut c_void"] + extra_r,
                          pre_c=PRE_PAR[0], pre_rs=PRE_PAR[1], n=300))
    cyc += 1
# grAnime_801C752C(obj, arg1, flags, func, type, ...)
for t, cb, va_c, va_r in [(0, "tcE", "", ""), (1, "tc1", ", (double) AF[0]", "&[(a.f[0] as f64).to_bits()]"), (3, "tc2", ", (u32) (rnd() % 100)", "&[(rnd() % 100) as u64]"),
                          (5, "tc1", ", (double) AF[1]", "&[(a.f[1] as f64).to_bits()]"), (7, "tc3", ", (u32) (rnd() % 100)", "&[(rnd() % 100) as u64]")]:
    funcs.append(dict(name="grAnime_801C752C", c_args=["&OJT[AI[0] % 8]", "(s32) (AI[1] & 1)", "(s32) (rnd() % 0x8000)", f"(void*) {cb}", str(t)] + ([va_c[2:]] if va_c else []),
                      rs_args=["addr_of_mut!(OJT[(a.i[0] % 8) as usize])", "(a.i[1] & 1) as i32", "(rnd() % 0x8000) as i32", f"{cb} as *mut c_void", str(t), va_r or "&[]"],
                      n=300))
for nm in ("grAnime_801C77FC", "grAnime_801C787C", "grAnime_801C78FC", "grAnime_801C7980"):
    funcs.append(dict(name=nm, c_args=["&OGO[0]", "(int) AI[0]", "(u32) (rnd() % 8)"], rs_args=["g", "a.i[0] as i32", "rnd() % 8"], n=300))
funcs.append(dict(name="grAnime_801C775C", c_args=["&OGO[0]", "(int) AI[0]", "(u32) (rnd() % 8)", "AF[0]", "AF[1]"], rs_args=["g", "a.i[0] as i32", "rnd() % 8", "a.f[0]", "a.f[1]"], n=300))
for nm in ("grAnime_801C7A04", "grAnime_801C7A94", "grAnime_801C7B24", "grAnime_801C7BA0"):
    funcs.append(dict(name=nm, c_args=["&OGO[0]", "(int) AI[0]", "(u32) (rnd() % 8)", "AF[0]"], rs_args=["g", "a.i[0] as i32", "rnd() % 8", "a.f[0]"], n=300))
funcs.append(dict(name="grAnime_801C7C1C", c_args=["&OJT[AI[0] % 8]", "(s32) AI[1]", "(s32) AI[2]", "(s32) (rnd() % 8)", "(s32) AI[3]", "(int) (AI[0] & 1)", "AF[0]", "AF[1]"],
                  rs_args=["addr_of_mut!(OJT[(a.i[0] % 8) as usize])", "a.i[1] as i32", "a.i[2] as i32", "(rnd() % 8) as i32", "a.i[3] as i32", "(a.i[0] & 1) as i32", "a.f[0]", "a.f[1]"],
                  n=500))
SETMAP_C = "OGP[0].map_id = rnd() % 4;"
SETMAP_RS = "OGP[0].map_id = rnd() % 4;"
for nm, f5 in (("grAnime_801C7FF8", 1), ("grAnime_801C8098", 0)):
    funcs.append(dict(name=nm, c_args=["&OGO[0]", "(int) AI[0]", "(int) (rnd() % 8)", "(int) AI[1] - 1", "AF[0]", "AF[1]"],
                      rs_args=["g", "a.i[0] as i32", "(rnd() % 8) as i32", "a.i[1] as i32 - 1", "a.f[0]", "a.f[1]"], pre_c=SETMAP_C, pre_rs=SETMAP_RS, n=500))
funcs.append(dict(name="grAnime_801C8138", c_args=["&OGO[0]", "(enum_t) AI[0]", "(int) AI[1] - 1"], rs_args=["g", "a.i[0] as i32", "a.i[1] as i32 - 1"], n=500))
funcs.append(dict(name="fn_801C82E8", c_args=["&OAO[AI[0] % 6]", "&OSLOT[0]"], rs_args=["addr_of_mut!(OAO[(a.i[0] % 6) as usize])", "addr_of_mut!(OSLOT[0])"], rs_path="call_82e8"))
funcs += [dict(name="grAnime_801C8318", n=300), dict(name="grAnime_801C83D0", n=300), dict(name="grAnime_801C84A4", n=300)]
funcs.append(dict(name="grAnime_801C8578", c_args=["&OJNT[AI[0] % 6]", "&API[0]"], rs_args=["addr_of_mut!(OJNT[(a.i[0] % 6) as usize])", "&mut a.pi[0]"]))
funcs.append(dict(name="grAnime_801C8578_noinline", c_args=["&OJNT[AI[0] % 6]", "&API[0]"], rs_args=["addr_of_mut!(OJNT[(a.i[0] % 6) as usize])", "&mut a.pi[0]"]))
funcs.append(dict(name="grAnime_801C86D4", c_args=["(s32) AI[0]", "&OGO[0]", "(s32) AI[1]"], rs_args=["a.i[0] as i32", "g", "a.i[1] as i32"],
                  pre_c="{ int j; for (j = 0; j < 4; j++) if (!OX8T[j].unk0) OX8T[j].unk0 = &OJNT[0]; }",
                  pre_rs="{ for j in 0..4 { if OX8T[j].unk0.is_null() { OX8T[j].unk0 = addr_of_mut!(OJNT[0]); } } }"))
funcs.append(dict(name="grAnime_801C8780", c_args=["&OGO[0]", "AI[0]", "AI[1]", "AF[0]", "AF[1]"], rs_args=["g", "a.i[0]", "a.i[1]", "a.f[0]", "a.f[1]"]))

SPEC = dict(
    src="src/melee/gr/granime.c",
    rust_mod="granime",
    ext_mods=["grinl"],
    u=[],
    provides=["HSD_ForeachAnim", "HSD_AObjSetRate_cbstub"],
    pre_c="void HSD_AObjSetRate_cbstub(HSD_AObj*, HSD_TObj*, u32, float);\n#define HSD_AObjSetRate HSD_AObjSetRate_cbstub",
    variadic={"OSReport": "pii"},
    globals=[],
    data_c=data_c,
    use_rs=use_rs,
    statics=statics,
    graph=graph,
    links_c=POINTER_SLOTS_C,
    links_rs=POINTER_SLOTS_RS,
    c_patches=[
        ("((Event) func)();", "((void (*)(HSD_AObj*)) func)(aobj);", 3),
        ("(uintptr_t) arg0->unk0 != -1U", "(uintptr_t) arg0->unk0 != (uintptr_t) -1"),
        ("static void fn_801C82E8(int arg0, int* arg1);", "static void fn_801C82E8(HSD_AObj* arg0, HSD_AObj** arg1);"),
        ("void fn_801C82E8(int arg0, int* arg1)\n{\n    *arg1 = arg0;\n    longjmp(grAnime_8049EE40, 1);\n}", "void fn_801C82E8(HSD_AObj* arg0, HSD_AObj** arg1)\n{\n    *arg1 = arg0;\n}"),
        ("if (setjmp(grAnime_8049EE40) == 0) {", "if (1) {"),
    ],
    rets={
        "Ground_801C3FA4": ("(rnd() % 5 ? (void*) &OJT[rnd() % 8] : (void*) 0)", "(if rnd() % 5 != 0 { addr_of_mut!(OJT[(rnd() % 8) as usize]) as *mut _ } else { null_mut() })"),
        "grMaterial_801C8B28": ("(void*) &OJT[rnd() % 8]", "addr_of_mut!(OJT[(rnd() % 8) as usize]) as *mut _"),
        "grDatFiles_801C6330": ("(void*) &OAR[rnd() % 2]", "addr_of_mut!(OAR[(rnd() % 2) as usize]) as *mut _"),
        "HSD_AObjGetFlags": ("rnd()", "rnd()"),
    },
    fnvals=FNV,
    hooks_rs="""    melee_world::tu::granime::ext::HSD_AOBJSETRATE = Some(|a0: *mut HSD_AObj, a1: f32| { lg_s("HSD_AObjSetRate("); lg_ptr(a0 as usize); lg_f(a1); lg_s(")"); lg_c(';'); });
    melee_world::tu::granime::ext::HSD_FOREACHANIM = Some(|obj: *mut c_void, ty: u32, mask: u32, func: *mut c_void, at: u32, va: &[usize]| {
        lg_s("HSD_ForeachAnim("); lg_ptr(obj as usize); lg_int(ty); lg_int(mask); lg_ptr(func as usize); lg_int(at); lg_s(")"); lg_c(';');
        if rnd() % 3 != 0 {
            let a = addr_of_mut!(OAO[(rnd() % 6) as usize]);
            let f: unsafe extern "C-unwind" fn(*mut HSD_AObj, *mut c_void) = core::mem::transmute(func);
            f(a, va[0] as *mut c_void);
        }
    });""",
    funcs=funcs,
)
