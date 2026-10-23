MAPS = "5, 7, 17, 18, 19, 20, 8, 9, 10, 11, 12, 13, 14, 15, 16, 3, 6"
MAPS8 = "8, 9, 10, 11, 12, 13, 14, 15, 16"
NN = "NN = 1;"


def pre(c, rs):
    return dict(pre_c=c, pre_rs=rs)


def maps(lst):
    n = lst.count(",") + 1
    return (f"{{ static const int M_[] = {{ {lst} }}; int j_; for (j_ = 0; j_ < NGO; j_++) OGP[j_].map_id = M_[rnd() % {n}]; }}",
            f"{{ const M_: [u32; {n}] = [{lst}]; for j_ in 0..NGO {{ OGP[j_].map_id = M_[(rnd() % {n}) as usize]; }} }}")


def both(*parts):
    return ("".join(p[0] + " " for p in parts), "".join(p[1] + " " for p in parts))


def P(*parts):
    c, r = both(*parts)
    return dict(pre_c=c, pre_rs=r)


def raw(c, r):
    return (c, r)


NNs = raw("NN = 1;", "NN = 1;")
# the first word of Ground.u is a u32 slot (castle12.xC4[0]) and, in the same byte, the storage of the bitfield castle11.xC4;
# the C packs bitfields LSB-first, the Rust records MSB-first, so a stale word there would alias differently by design
Z0 = raw("{ int z_; for (z_ = 0; z_ < NGO; z_++) *(u32*) &OGP[z_].u = 0; }",
         "for z_ in 0..NGO { *(addr_of_mut!(OGP[z_].u) as *mut u32) = 0; }")
# gobj pointer in a u32 slot (nullable): path is a C/Rust field of OGP[0]
def gslot(path, p_null=3):
    return raw(f"OGP[0].{path} = (rnd() % {p_null}) ? (u32) (uintptr_t) &OGO[rnd() % NGO] : 0;",
               f"OGP[0].{path} = if rnd() % {p_null} != 0 {{ ptr_u32(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize])) }} else {{ 0 }};")


def gslot_nn(path):
    return raw(f"OGP[0].{path} = (u32) (uintptr_t) &OGO[rnd() % NGO];",
               f"OGP[0].{path} = ptr_u32(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]));")


def jslot_nn(path):
    return raw(f"OGP[0].{path} = (u32) (uintptr_t) &OJO[rnd() % NJO];",
               f"OGP[0].{path} = ptr_u32(addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]));")


def camslot(path):
    return raw(f"OGP[0].{path} = (rnd() % 2) ? (u32) (uintptr_t) OP_CmSubject[rnd() % 2] : 0;",
               f"OGP[0].{path} = if rnd() % 2 != 0 {{ ptr_u32(addr_of_mut!(OP_CmSubject[(rnd() % 2) as usize])) }} else {{ 0 }};")


def jpick(path, nullable=True):
    if nullable:
        return raw(f"OGP[0].{path} = (rnd() % 4) ? &OJO[rnd() % NJO] : (HSD_JObj*) 0;",
                   f"OGP[0].{path} = if rnd() % 4 != 0 {{ addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) }} else {{ null_mut() }};")
    return raw(f"OGP[0].{path} = &OJO[rnd() % NJO];", f"OGP[0].{path} = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]);")


# castle10 arrays: jobjs / effects set together (effect_b only with effect_a)
C10_SETUP = raw(
    """{ int k_; for (k_ = 0; k_ < 5; k_++) {
        OGP[0].u.castle10.jobjs[k_] = (rnd() % 4) ? &OJO[rnd() % NJO] : (HSD_JObj*) 0;
        if (rnd() % 2) { OGP[0].u.castle10.effect_a[k_] = &OJO[rnd() % NJO]; OGP[0].u.castle10.effect_b[k_] = &OJO[rnd() % NJO]; }
        else { OGP[0].u.castle10.effect_a[k_] = (HSD_JObj*) 0; OGP[0].u.castle10.effect_b[k_] = (HSD_JObj*) 0; }
        OGP[0].u.castle10.x10C[k_] = (rnd() % 2) ? (u32) (uintptr_t) &OGO[rnd() % NGO] : 0;
        OGP[0].u.castle10.x120[k_] = (int) (rnd() % 20);
        OGP[0].u.castle10.state[k_] = (u8) (rnd() % 3);
        OGP[0].u.castle10.idx[k_] = (u8) (rnd() % 6);
        OGP[0].u.castle10.baseY[k_] = rf();
    } OGP[0].u.castle10.xC4 = (s16) (rnd() % 5); }""",
    """for k_ in 0..5usize {
        OGP[0].u.castle10.jobjs[k_] = if rnd() % 4 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) } else { null_mut() };
        if rnd() % 2 != 0 { OGP[0].u.castle10.effect_a[k_] = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); OGP[0].u.castle10.effect_b[k_] = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); }
        else { OGP[0].u.castle10.effect_a[k_] = null_mut(); OGP[0].u.castle10.effect_b[k_] = null_mut(); }
        OGP[0].u.castle10.x10C[k_] = if rnd() % 2 != 0 { ptr_u32(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize])) } else { 0 };
        OGP[0].u.castle10.x120[k_] = (rnd() % 20) as i32;
        OGP[0].u.castle10.state[k_] = (rnd() % 3) as u8;
        OGP[0].u.castle10.idx[k_] = (rnd() % 6) as u8;
        OGP[0].u.castle10.baseY[k_] = rf();
    } OGP[0].u.castle10.xC4 = (rnd() % 5) as i16;""")
C10_SETUP_NN = raw(
    """{ int k_; for (k_ = 0; k_ < 5; k_++) {
        OGP[0].u.castle10.jobjs[k_] = &OJO[rnd() % NJO];
        if (rnd() % 2) { OGP[0].u.castle10.effect_a[k_] = &OJO[rnd() % NJO]; OGP[0].u.castle10.effect_b[k_] = &OJO[rnd() % NJO]; }
        else { OGP[0].u.castle10.effect_a[k_] = (HSD_JObj*) 0; OGP[0].u.castle10.effect_b[k_] = (HSD_JObj*) 0; }
        OGP[0].u.castle10.x10C[k_] = (rnd() % 2) ? (u32) (uintptr_t) &OGO[rnd() % NGO] : 0;
        OGP[0].u.castle10.x120[k_] = (int) (rnd() % 20);
        OGP[0].u.castle10.state[k_] = (u8) (rnd() % 3);
        OGP[0].u.castle10.idx[k_] = (u8) (rnd() % 6);
        OGP[0].u.castle10.baseY[k_] = rf();
    } OGP[0].u.castle10.xC4 = (s16) (rnd() % 5); }""",
    """for k_ in 0..5usize {
        OGP[0].u.castle10.jobjs[k_] = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]);
        if rnd() % 2 != 0 { OGP[0].u.castle10.effect_a[k_] = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); OGP[0].u.castle10.effect_b[k_] = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); }
        else { OGP[0].u.castle10.effect_a[k_] = null_mut(); OGP[0].u.castle10.effect_b[k_] = null_mut(); }
        OGP[0].u.castle10.x10C[k_] = if rnd() % 2 != 0 { ptr_u32(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize])) } else { 0 };
        OGP[0].u.castle10.x120[k_] = (rnd() % 20) as i32;
        OGP[0].u.castle10.state[k_] = (rnd() % 3) as u8;
        OGP[0].u.castle10.idx[k_] = (rnd() % 6) as u8;
        OGP[0].u.castle10.baseY[k_] = rf();
    } OGP[0].u.castle10.xC4 = (rnd() % 5) as i16;""")
STG_KIND = raw("stage_info.grkind = (rnd() % 2) ? 2 : (rnd() % 5);",
               "(*(addr_of_mut!(G_stage_info) as *mut StageInfo)).grkind = if rnd() % 2 != 0 { 2 } else { rnd() % 5 };")

C9_STATE = raw("OGP[0].u.castle9.xDE_b0 = (rnd() % 3) != 0; OGP[0].u.castle9.xD4 = (s16) ((int) (rnd() % 4) - 1); OGP[0].u.castle4.xD6 = (s16) (rnd() % 3); OGP[0].u.castle4.xD8 = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDA = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDC = (s16) ((int) (rnd() % 12) - 2);",
               "OGP[0].u.castle9.set_xDE_b0(if rnd() % 3 != 0 { 1 } else { 0 }); OGP[0].u.castle9.xD4 = ((rnd() % 4) as i32 - 1) as i16; OGP[0].u.castle4.xD6 = (rnd() % 3) as i16; OGP[0].u.castle4.xD8 = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDA = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDC = ((rnd() % 12) as i32 - 2) as i16;")

V = dict(
    c9=["castle9", "castle3", "castle4"],
    c8=["castle8"],
    c11=["castle11"],
    arw=["arwing"],
    cas=["castle"],
    c10=["castle10", "castle6"],
    c12=["castle12"],
    c3=["castle3"],
)

ALLU = V["c9"] + V["c8"] + V["c11"] + V["c10"] + V["arw"] + V["cas"] + ["castle12", "castle5"]
funcs = []


def F(name, u=None, args=None, pre_parts=None, n=None, rs_path=None, ov=None, bfs=None):
    d = dict(name=name)
    if bfs:
        d["bfs"] = bfs
    if u is not None:
        d["u"] = u
    if args:
        d["c_args"] = [a[0] for a in args]
        d["rs_args"] = [a[1] for a in args]
    if pre_parts:
        c, r = both(*pre_parts)
        d["pre_c"] = c
        d["pre_rs"] = r
    if n:
        d["n"] = n
    if rs_path:
        d["rs_path"] = rs_path
    funcs.append(d)


AI0 = ("AI[0]", "a.i[0] as _")
OGO0 = ("&OGO[0]", "g")
for nm in ("grCastle_801CD338",):
    F(nm, pre_parts=[NNs])
F("grCastle_801CD37C", bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], u=V["c9"] + V["c8"] + V["c11"] + V["c10"] + V["arw"] + V["cas"] + ["castle12"], n=300, pre_parts=[NNs, maps(MAPS)])
F("grCastle_801CD4A0")
F("grCastle_801CD4A4")
F("grCastle_801CD4C8")
F("grCastle_801CD4D0", bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], u=V["c9"] + V["c8"] + V["c11"] + V["c10"] + V["arw"] + V["cas"] + ["castle12"],
  args=[("(int) AI[0]", "a.i[0] as i32")], n=1000,
  pre_parts=[Z0, NNs, raw("AI[0] = rnd() % 21;", "a.i[0] = rnd() % 21;"),
             raw("{ static const int M_[] = { 5, 7, 17, 18, 19, 20 }; int j_; for (j_ = 0; j_ < NGO; j_++) OGP[j_].map_id = (AI[0] >= 8 && AI[0] <= 16) ? 8 + rnd() % 9 : M_[rnd() % 6]; }",
                 "{ const M_: [u32; 6] = [5, 7, 17, 18, 19, 20]; for j_ in 0..NGO { OGP[j_].map_id = if a.i[0] >= 8 && a.i[0] <= 16 { 8 + rnd() % 9 } else { M_[(rnd() % 6) as usize] }; } }")])
F("grCastle_801CD5BC")
F("grCastle_801CD600")
F("grCastle_801CD608")
F("grCastle_801CD60C")
F("grCastle_801CD610", u=ALLU, bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], n=300,
  pre_parts=[Z0, NNs, maps(MAPS)])
F("grCastle_801CD658", u=V["c9"], pre_parts=[NNs])
F("grCastle_801CD8A0")
F("grCastle_801CD8A8", u=ALLU, bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], n=300,
  pre_parts=[Z0, NNs, maps(MAPS8), C9_STATE, raw("{ int j_; for (j_ = 0; j_ < NGO; j_++) OGP[j_].u.castle5.xC6 = (s16) (rnd() % 9); }", "for j_ in 0..NGO { OGP[j_].u.castle5.xC6 = (rnd() % 9) as i16; }"),
             raw("{ int j_; for (j_ = 0; j_ < 3; j_++) { ODD[j_].desc.lb_unk0.jobj = &OJO[NGO + rnd() % NGO]; } for (j_ = 0; j_ < 12; j_++) OGP[0].u.castle3.x1C[j_].data = (rnd() % 2) ? &ODD[rnd() % 3] : (struct DynamicsData*) 0; OGP[0].u.castle12.xD0 = -1; OGP[0].u.castle9.xDE_b0 = (rnd() % 3) != 0; OGP[0].u.castle9.xD4 = (s16) ((int) (rnd() % 4) - 1); OGP[0].u.castle4.xD6 = (s16) (rnd() % 3); OGP[0].u.castle4.xD8 = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDA = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDC = (s16) ((int) (rnd() % 12) - 2); for (j_ = 0; j_ < 3; j_++) OGP[0].u.castle12.xC4[j_] = (u32) (uintptr_t) &OGO[1 + rnd() % (NGO - 1)]; OGP[0].u.castle12.xD2 = (s16) ((int) (rnd() % 3) - 1); }",
                 "{ for j_ in 0..3usize { ODD[j_].desc.lb_unk0.jobj = addr_of_mut!(OJO[NGO + (rnd() % NGO as u32) as usize]); } for j_ in 0..12usize { OGP[0].u.castle3.x1C[j_].data = if rnd() % 2 != 0 { addr_of_mut!(ODD[(rnd() % 3) as usize]) } else { null_mut() }; } OGP[0].u.castle12.xD0 = -1; OGP[0].u.castle9.set_xDE_b0(if rnd() % 3 != 0 { 1 } else { 0 }); OGP[0].u.castle9.xD4 = ((rnd() % 4) as i32 - 1) as i16; OGP[0].u.castle4.xD6 = (rnd() % 3) as i16; OGP[0].u.castle4.xD8 = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDA = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDC = ((rnd() % 12) as i32 - 2) as i16; for j_ in 0..3usize { OGP[0].u.castle12.xC4[j_] = ptr_u32(addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize])); } OGP[0].u.castle12.xD2 = ((rnd() % 3) as i32 - 1) as i16; }")])
F("grCastle_801CD960", u=V["c3"])
F("grCastle_801CD9B4")
F("grCastle_801CDA0C", u=V["c8"], pre_parts=[NNs])
F("grCastle_801CDC3C")
F("grCastle_801CDC44", u=V["c8"], n=600,
  pre_parts=[raw("{ int j_; for (j_ = 0; j_ < 2; j_++) { OGP[0].u.castle8.plat[j_].jobj = &OJO[rnd() % NJO]; OGP[0].u.castle8.plat[j_].state = (s16) (rnd() % 4); OGP[0].u.castle8.plat[j_].timer = (s16) (rnd() % 60); } }",
                  "{ for j_ in 0..2usize { OGP[0].u.castle8.plat[j_].jobj = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); OGP[0].u.castle8.plat[j_].state = (rnd() % 4) as i16; OGP[0].u.castle8.plat[j_].timer = (rnd() % 60) as i16; } }")])
F("grCastle_801CDF50")
F("grCastle_801CDF54", args=[("&AV3[0]", "&mut a.v3[0]")], pre_parts=[NNs, STG_KIND])
F("grCastle_801CDFD8", u=V["c9"])
F("grCastle_801CE054", u=["castle4"], pre_parts=[raw(
    "OGP[0].u.castle4.xD6 = (s16) (rnd() % 3); OGP[0].u.castle4.xD8 = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDA = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDC = (s16) ((int) (rnd() % 12) - 2);",
    "OGP[0].u.castle4.xD6 = (rnd() % 3) as i16; OGP[0].u.castle4.xD8 = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDA = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDC = ((rnd() % 12) as i32 - 2) as i16;")])
F("grCastle_801CE19C", u=V["c9"] + ["castle4", "castle11", "castle5"], n=500,
  pre_parts=[NNs, maps(MAPS8), raw("{ int j_; for (j_ = 0; j_ < NGO; j_++) OGP[j_].u.castle5.xC6 = (s16) (rnd() % 9); }", "for j_ in 0..NGO { OGP[j_].u.castle5.xC6 = (rnd() % 9) as i16; }"), raw("OGP[0].u.castle9.xDE_b0 = (rnd() % 3) != 0; OGP[0].u.castle9.xD4 = (s16) ((int) (rnd() % 4) - 1); OGP[0].u.castle4.xD6 = (s16) (rnd() % 3); OGP[0].u.castle4.xD8 = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDA = (s16) ((int) (rnd() % 12) - 2); OGP[0].u.castle4.xDC = (s16) ((int) (rnd() % 12) - 2);",
                                    "OGP[0].u.castle9.set_xDE_b0(if rnd() % 3 != 0 { 1 } else { 0 }); OGP[0].u.castle9.xD4 = ((rnd() % 4) as i32 - 1) as i16; OGP[0].u.castle4.xD6 = (rnd() % 3) as i16; OGP[0].u.castle4.xD8 = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDA = ((rnd() % 12) as i32 - 2) as i16; OGP[0].u.castle4.xDC = ((rnd() % 12) as i32 - 2) as i16;")])
F("grCastle_801CE260", u=V["c11"], n=300, pre_parts=[NNs, maps(MAPS8)])
F("fn_801CE3A0")
F("grCastle_801CE3A4")
for nm in ("grCastle_801CE3AC", "grCastle_801CE3AC_dontinline"):
    F(nm, u=V["c11"], n=300,
      pre_parts=[NNs, gslot("u.castle11.xCC"), raw("OGP[0].u.castle5.xC6 = (s16) (rnd() % 9);", "OGP[0].u.castle5.xC6 = (rnd() % 9) as i16;")])
C11_STATE = [gslot("u.castle11.xCC", 2), gslot_nn("u.castle11.xD4"), gslot("u.castle11.xD0", 2), camslot("u.castle11.xD8"),
             raw("OGP[0].u.castle11.xCA = (s16) ((int) (rnd() % 4) - 1); OGP[0].u.castle11.xC8 = (s16) ((int) (rnd() % 4) - 1);",
                 "OGP[0].u.castle11.xCA = ((rnd() % 4) as i32 - 1) as i16; OGP[0].u.castle11.xC8 = ((rnd() % 4) as i32 - 1) as i16;")]
F("grCastle_801CE578", bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], u=ALLU, n=500, pre_parts=[Z0, NNs, maps(MAPS8)] + C11_STATE + [raw("OGP[0].u.castle5.xC6 = (s16) (rnd() % 9);", "OGP[0].u.castle5.xC6 = (rnd() % 9) as i16;")])
F("grCastle_801CE7E4")
F("grCastle_801CE7E8", u=V["cas"], pre_parts=[NNs])
F("grCastle_801CE858")
F("grCastle_801CE860", u=V["cas"], pre_parts=[raw("OGP[0].u.castle.xC4 = rnd() % 4; OGP[0].u.castle.xC8 = (s16) ((int) (rnd() % 4) - 1);", "OGP[0].u.castle.xC4 = rnd() % 4; OGP[0].u.castle.xC8 = ((rnd() % 4) as i32 - 1) as i16;")])
F("grCastle_801CE8E4")
F("grCastle_801CE8E8", u=V["arw"], pre_parts=[NNs])
F("fn_801CE9DC")
F("grCastle_801CE9E0")
F("grCastle_801CE9E8", bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], u=V["arw"], pre_parts=[NNs, camslot("u.arwing.xC8"), gslot("u.arwing.xC4")])
F("grCastle_801CEAC8")
F("zero", u=V["c10"], args=[("&OGP[0]", "addr_of_mut!(OGP[0])"), ("(int) (AI[0] % 5)", "(a.i[0] % 5) as usize"), ("AF[0]", "a.f[0]")])
F("grCastle_801CEACC", u=V["c10"], n=300, pre_parts=[NNs, raw("{ static const int M_[] = { 17, 7, 5, 3 }; int j_; for (j_ = 0; j_ < NGO; j_++) OGP[j_].map_id = M_[rnd() % 4]; }",
                                                            "{ const M_: [u32; 4] = [17, 7, 5, 3]; for j_ in 0..NGO { OGP[j_].map_id = M_[(rnd() % 4) as usize]; } }")])
F("grCastle_801CEEFC")
F("grCastle_801CEF04", u=V["c10"], n=500, pre_parts=[NNs, C10_SETUP_NN, raw("OGP[0].u.castle6.xC4 = (s16) (rnd() % 5); OGP[0].u.castle6.xCC = (int) (rnd() % 4) - 1; OGP[0].u.castle6.xC8 = (s16) (rnd() % 2);",
                                                                        "OGP[0].u.castle6.xC4 = (rnd() % 5) as i16; OGP[0].u.castle6.xCC = (rnd() % 4) as i32 - 1; OGP[0].u.castle6.xC8 = (rnd() % 2) as i16;")])
F("grCastle_801CF0F0")
F("grCastle_801CF0F4", u=V["c11"], n=300, pre_parts=[NNs, raw("{ static const int M_[] = { 20, 19, 18, 3 }; int j_; for (j_ = 0; j_ < NGO; j_++) OGP[j_].map_id = M_[rnd() % 4]; }",
                                                            "{ const M_: [u32; 4] = [20, 19, 18, 3]; for j_ in 0..NGO { OGP[j_].map_id = M_[(rnd() % 4) as usize]; } }")])
F("grCastle_801CF300")
F("grCastle_801CF308", bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], u=ALLU, n=800,
  pre_parts=[Z0, NNs, gslot_nn("u.castle11.xD0"), jslot_nn("u.castle11.xD4"), camslot("u.castle11.xD8"),
             raw("OGP[0].u.castle5.xC4 = (s16) (rnd() % 6); OGP[0].u.castle11.xCA = (s16) ((int) (rnd() % 3) - 1); OGP[0].u.castle.xC8 = (s16) ((int) (rnd() % 37) - 1); *(s16*) &OGP[0].u.castle11.xCC = (s16) ((int) (rnd() % 3) - 1);",
                 "OGP[0].u.castle5.xC4 = (rnd() % 6) as i16; OGP[0].u.castle11.xCA = ((rnd() % 3) as i32 - 1) as i16; OGP[0].u.castle.xC8 = ((rnd() % 37) as i32 - 1) as i16; *(addr_of_mut!(OGP[0].u.castle11.xCC) as *mut i16) = ((rnd() % 3) as i32 - 1) as i16;"),
             raw("{ int j_; for (j_ = 0; j_ < NGO; j_++) ((Ground*) OGO[j_].user_data)->u.castle5.xC4 = (s16) (rnd() % 3); }",
                 "{ for j_ in 0..NGO { (*(OGO[j_].user_data as *mut Ground)).u.castle5.xC4 = (rnd() % 3) as i16; } }")])
F("grCastle_801CF74C")
F("grCastle_801CF750", u=V["c8"], args=[("(void*) &OGP[0]", "addr_of_mut!(OGP[0]) as *mut c_void"), ("(s32) ((AI[0] & 1) ? 4 : 5)", "if a.i[0] & 1 != 0 { 4 } else { 5 }"), ("&OCD[0]", "addr_of_mut!(OCD[0])"),
                                       ("(int) AI[1]", "a.i[1] as i32"), ("(mpLib_GroundEnum) 0", "0"), ("AF[0]", "a.f[0]")])
F("grCastle_801CF7B0", u=ALLU, bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], n=400, pre_parts=[Z0, NNs, maps(MAPS)])
F("grCastle_ResetSatelliteTimer", u=V["c12"], args=[("&OGP[0]", "addr_of_mut!(OGP[0])")])
F("grCastle_PickSatellite", u=V["c12"] + V["c11"], bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], n=500, args=[("&OGP[0]", "addr_of_mut!(OGP[0])"), ("OWP", "addr_of_mut!(OWP) as *mut i32")],
  pre_parts=[NNs, raw("{ int j_; for (j_ = 0; j_ < NGO; j_++) OGP[j_].map_id = (rnd() % 2) ? 3 : 6; for (j_ = 0; j_ < 3; j_++) OGP[0].u.castle12.xC4[j_] = (u32) (uintptr_t) &OGO[1 + rnd() % (NGO - 1)]; for (j_ = 0; j_ < 3; j_++) OWP[j_] = (int) (rnd() % 150); }",
                      "{ for j_ in 0..NGO { OGP[j_].map_id = if rnd() % 2 != 0 { 3 } else { 6 }; } for j_ in 0..3usize { OGP[0].u.castle12.xC4[j_] = ptr_u32(addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize])); } for j_ in 0..3usize { OWP[j_] = (rnd() % 150) as i32; } }")])
F("grCastle_801CF868", u=V["c12"] + V["c11"], bfs=["u.castle11.xC4", "u.castle9.xDE_b0"], n=600,
  pre_parts=[NNs, maps(MAPS),
             raw("{ int j_; int z_ = (rnd() % 4) == 0; for (j_ = 0; j_ < 3; j_++) OGP[0].u.castle12.xC4[j_] = !z_ ? (u32) (uintptr_t) &OGO[1 + rnd() % (NGO - 1)] : 0; OGP[0].u.castle12.xD0 = (s16) ((int) (rnd() % 4) - 1); OGP[0].u.castle12.xD2 = (s16) ((int) (rnd() % 3) - 1); for (j_ = 1; j_ < NGO; j_++) *(s16*) &OGP[j_].u.castle2.xC4 = (s16) ((rnd() % 3) ? 0 : 4); yakumono_param->x6 = (s16) (1 + rnd() % 5); }",
                 "{ let z_ = (rnd() % 4) == 0; for j_ in 0..3usize { OGP[0].u.castle12.xC4[j_] = if !z_ { ptr_u32(addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize])) } else { 0 }; } OGP[0].u.castle12.xD0 = ((rnd() % 4) as i32 - 1) as i16; OGP[0].u.castle12.xD2 = ((rnd() % 3) as i32 - 1) as i16; for j_ in 1..NGO { *(addr_of_mut!(OGP[j_].u.castle2.xC4) as *mut i16) = if rnd() % 3 != 0 { 0 } else { 4 }; } (*t::yakumono_param).x6 = (1 + rnd() % 5) as i16; }")])
F("fn_801CFAFC", args=[("&OGO[0]", "g"), ("&OGP[1]", "addr_of_mut!(OGP[1])"), ("&AV3[0]", "&mut a.v3[0]"), ("&OGO[2]", "addr_of_mut!(OGO[2])")])
F("fn_801CFB68", args=[("&OGO[0]", "g"), ("&OGP[1]", "addr_of_mut!(OGP[1])"), ("&OGO[2]", "addr_of_mut!(OGO[2])")])
for nm in ("grCastle_801CFBD4", "grCastle_801D0298"):
    F(nm, u=V["c10"], n=700, args=[("&OGO[0]", "g"), ("(s32) (AI[0] & 1)", "(a.i[0] & 1) as i32")],
      pre_parts=[NNs, STG_KIND, C10_SETUP])
F("grCastle_801D02B8", u=V["c10"], n=500, pre_parts=[NNs, C10_SETUP_NN])
F("grCastle_801D0520", u=V["cas"], args=[("&OGO[0]", "g"), ("(int) AI[0]", "a.i[0] as i32")])
for k, nm in enumerate(("grCastle_801D0550", "grCastle_801D059C", "grCastle_801D05E8", "grCastle_801D0634", "grCastle_801D0680")):
    F(nm, u=V["c10"], args=[("(void*) &OGO[0]", "addr_of_mut!(OGO[0]) as *mut c_void"), ("&OGP[AI[0] % 6]", "addr_of_mut!(OGP[(a.i[0] % 6) as usize])")],
      pre_parts=[raw("{ int j_; for (j_ = 0; j_ < NGO; j_++) { int k_; for (k_ = 0; k_ < 5; k_++) OGP[j_].u.castle10.x10C[k_] = (rnd() % 2) ? (u32) (uintptr_t) &OGO[rnd() % NGO] : 0; } }",
                      "{ for j_ in 0..NGO { for k_ in 0..5usize { OGP[j_].u.castle10.x10C[k_] = if rnd() % 2 != 0 { ptr_u32(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize])) } else { 0 }; } } }")])
for k, nm in enumerate(("grCastle_801D06CC", "grCastle_801D0744", "grCastle_801D07BC", "grCastle_801D0834", "grCastle_801D08AC")):
    F(nm, u=V["c10"], args=[("(void*) &OGO[0]", "addr_of_mut!(OGO[0]) as *mut c_void"), ("&OGP[AI[0] % 6]", "addr_of_mut!(OGP[(a.i[0] % 6) as usize])"), ("&OGO[AI[1] % 6]", "addr_of_mut!(OGO[(a.i[1] % 6) as usize])")],
      pre_parts=[raw("{ int j_; for (j_ = 0; j_ < NGO; j_++) { int k_; for (k_ = 0; k_ < 5; k_++) OGP[j_].u.castle10.x10C[k_] = (rnd() % 2) ? (u32) (uintptr_t) &OGO[rnd() % NGO] : 0; } }",
                      "{ for j_ in 0..NGO { for k_ in 0..5usize { OGP[j_].u.castle10.x10C[k_] = if rnd() % 2 != 0 { ptr_u32(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize])) } else { 0 }; } } }")])
F("fn_801D0924", u=V["c3"], args=[("&OGO[0]", "g"), ("(int) AI[0]", "a.i[0] as i32")], n=400,
  pre_parts=[raw("{ int j_; for (j_ = 0; j_ < 12; j_++) OGP[0].u.castle3.x1C[j_].data = (rnd() % 2) ? &ODD[rnd() % 3] : (struct DynamicsData*) 0; }",
                 "{ for j_ in 0..12usize { OGP[0].u.castle3.x1C[j_].data = if rnd() % 2 != 0 { addr_of_mut!(ODD[(rnd() % 3) as usize]) } else { null_mut() }; } }")])
F("grCastle_801D09B8", args=[("(void*) 0", "null_mut()"), ("&OGO[0]", "g"), ("&AV3[0]", "&mut a.v3[0]")],
  pre_parts=[raw("{ int j_; for (j_ = 0; j_ < 2; j_++) ((HSD_Generator*) OP_HSD_Generator[j_])->appsrt = &OAPP[0]; }",
                 "{ for j_ in 0..2usize { (*(addr_of_mut!(OP_HSD_Generator[j_]) as *mut HSD_Generator)).appsrt = addr_of_mut!(OAPP[0]); } }")])
F("grCastle_801D0A9C", args=[("&AV3[0]", "&mut a.v3[0]"), ("AF[0]", "a.f[0]")],
  pre_parts=[raw("{ int j_; for (j_ = 0; j_ < 2; j_++) ((HSD_Generator*) OP_HSD_Generator[j_])->appsrt = &OAPP[0]; }",
                 "{ for j_ in 0..2usize { (*(addr_of_mut!(OP_HSD_Generator[j_]) as *mut HSD_Generator)).appsrt = addr_of_mut!(OAPP[0]); } }")])
F("grCastle_801D0B04")
F("grCastle_801D0B0C", args=[("&AV3[0]", "&mut a.v3[0]"), ("(int) AI[0]", "a.i[0] as i32"), ("&OJO[0]", "addr_of_mut!(OJO[0])")])
F("grCastle_801D0BBC", n=600, pre_parts=[raw("grCs_804D6974->unk_angle_int = (rnd() % 2) ? 30 * (int) (rnd() % 20) : (int) (rnd() % 1000);",
                                            "(*t::grCs_804D6974).unk_angle_int = if rnd() % 2 != 0 { 30 * (rnd() % 20) as i32 } else { (rnd() % 1000) as i32 };")])
F("grCastle_801D0D24")
F("grCastle_801D0D84", n=300, args=[("(AI[0] & 1) ? &OJO[NGO + AI[1] % NGO] : (HSD_JObj*) 0", "if a.i[0] & 1 != 0 { addr_of_mut!(OJO[NGO + (a.i[1] % NGO as u32) as usize]) } else { null_mut() }")])
F("grCastle_801D0FF0")

CALLBACK_NAMES = ["grCastle_801CD338", "grCastle_801CD37C", "grCastle_801CD4A0", "grCastle_801CD4A4", "grCastle_801CD4C8",
                  "grCastle_801CD5BC", "grCastle_801CD600", "grCastle_801CD608", "grCastle_801CD60C", "grCastle_801CD610", "grCastle_801CD658",
                  "grCastle_801CD8A0", "grCastle_801CD8A8", "grCastle_801CD960", "grCastle_801CD9B4", "grCastle_801CDA0C", "grCastle_801CDC3C",
                  "grCastle_801CDC44", "grCastle_801CDF50", "grCastle_801CE260", "grCastle_801CE3A4", "grCastle_801CE578", "grCastle_801CE7E4",
                  "grCastle_801CE7E8", "grCastle_801CE858", "grCastle_801CE860", "grCastle_801CE8E4", "grCastle_801CE8E8", "grCastle_801CE9E0",
                  "grCastle_801CE9E8", "grCastle_801CEAC8", "grCastle_801CEACC", "grCastle_801CEEFC", "grCastle_801CEF04", "grCastle_801CF0F0",
                  "grCastle_801CF0F4", "grCastle_801CF300", "grCastle_801CF308", "grCastle_801CF74C", "grCastle_801CF750", "grCastle_801D0520",
                  "grCastle_801D0550", "grCastle_801D059C", "grCastle_801D05E8", "grCastle_801D0634", "grCastle_801D0680", "grCastle_801D06CC",
                  "grCastle_801D0744", "grCastle_801D07BC", "grCastle_801D0834", "grCastle_801D08AC", "fn_801D0924", "grCastle_801D09B8",
                  "grCastle_801D0B04", "grCastle_801D0B0C", "fn_801CFAFC", "fn_801CFB68", "fn_801CE3A0", "fn_801CE9DC"]

SPEC = dict(
    src="src/melee/gr/grcastle.c",
    rust_mod="grcastle",
    ext_mods=["grinl"],
    u=[],
    bf_skip=["StageCallbacks"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    variadic={"OSReport": "pii"},
    provides=["HSD_GObjPLinkHead"],
    u32ptr=["u.castle11.xCC", "u.castle11.xD0", "u.castle11.xD4", "u.castle11.xD8", "u.arwing.xC4", "u.arwing.xC8",
            "u.castle12.xC4[0]", "u.castle12.xC4[1]", "u.castle12.xC4[2]",
            "u.castle10.x10C[0]", "u.castle10.x10C[1]", "u.castle10.x10C[2]", "u.castle10.x10C[3]", "u.castle10.x10C[4]"],
    c_patches=[
        ("gp->u.castle2.xC4 = grCastle_801CD4D0(20);", "gp->u.castle12.xC4[0] = (u32) (uintptr_t) grCastle_801CD4D0(20);"),
        ("gp->u.castle2.xC8 = grCastle_801CD4D0(19);", "gp->u.castle12.xC4[1] = (u32) (uintptr_t) grCastle_801CD4D0(19);"),
        ("gp->u.castle2.xCC = grCastle_801CD4D0(18);", "gp->u.castle12.xC4[2] = (u32) (uintptr_t) grCastle_801CD4D0(18);"),
        ("gp->u.castle2.xD0 = -1;", "gp->u.castle12.xD0 = -1;"),
        ("gp->u.castle2.xD2 = x2 > x0", "gp->u.castle12.xD2 = x2 > x0"),
        ("sat_gp->u.castle7.xD4 =", "sat_gp->u.castle11.xD4 ="),
        ("((Ground*) (gp->u.castle7.xD0)->user_data)", "((Ground*) ((HSD_GObj*) (uintptr_t) gp->u.castle11.xD0)->user_data)", 2),
        ("gp->u.castle7.xD8 = 0;", "gp->u.castle11.xD8 = 0;"),
        ("gp->u.castle7.xD0 = grCastle_801CD4D0(0x11);", "gp->u.castle11.xD0 = (u32) (uintptr_t) grCastle_801CD4D0(0x11);"),
        ("gp->u.castle7.xD0 = grCastle_801CD4D0(7);", "gp->u.castle11.xD0 = (u32) (uintptr_t) grCastle_801CD4D0(7);"),
        ("gp->u.castle7.xD0 = grCastle_801CD4D0(5);", "gp->u.castle11.xD0 = (u32) (uintptr_t) grCastle_801CD4D0(5);"),
        ("if (gp->u.castle5.xCC != NULL) {", "if (gp->u.castle11.xCC != 0) {"),
        ("jobj = HSD_GObjGetHSDObj(gp->u.castle5.xCC);", "jobj = HSD_GObjGetHSDObj((HSD_GObj*) (uintptr_t) gp->u.castle11.xCC);"),
        ("val = ((s8*) &tbl)[gp->u.castle.xC8];", "{ int ix_ = gp->u.castle.xC8; val = (s8) (((u16*) &tbl)[ix_ >> 1] >> ((ix_ & 1) ? 0 : 8)); }"),
        ("gp->u.castle8.plat[0].state = yakumono_param->x54;", "*(s16*) &gp->u.castle11.xCC = yakumono_param->x54;"),
        ("s16 cnt = gp->u.castle8.plat[0].state;\n            gp->u.castle8.plat[0].state = cnt - 1;", "s16 cnt = *(s16*) &gp->u.castle11.xCC;\n            *(s16*) &gp->u.castle11.xCC = cnt - 1;"),
        ("    arg0->x134[i] = 1;\n    grMaterial_801C8CDC(arg0->x10C[i]);\n    arg0->x10C[i] = NULL;",
         "    ((Ground*) arg0)->u.castle10.state[i] = 1;\n    grMaterial_801C8CDC((HSD_GObj*) (uintptr_t) ((Ground*) arg0)->u.castle10.x10C[i]);\n    ((Ground*) arg0)->u.castle10.x10C[i] = 0;"),
        ("        arg0->x134[i] = 1;\n        grMaterial_801C8CDC(arg0->x10C[i]);\n        arg0->x10C[i] = NULL;",
         "        ((Ground*) arg0)->u.castle10.state[i] = 1;\n        grMaterial_801C8CDC((HSD_GObj*) (uintptr_t) ((Ground*) arg0)->u.castle10.x10C[i]);\n        ((Ground*) arg0)->u.castle10.x10C[i] = 0;"),
    ],
    data_c="""static int NN;
static HSD_GObj* OPLH[HSD_GOBJ_PLINK_GROUND + 1];
HSD_GObj** HSD_GObjPLinkHead = OPLH;
static s32 OWP[3];
static CollData OCD[1];
static struct DynamicsData ODD[3];
static HSD_psAppSRT OAPP[1];""",
    use_rs="""static mut NN: i32 = 0;
static mut OWP: [i32; 3] = [0; 3];
static mut OCD: [CollData; 1] = unsafe { core::mem::zeroed() };
static mut ODD: [DynamicsData; 3] = unsafe { core::mem::zeroed() };
static mut OAPP: [HSD_psAppSRT; 1] = unsafe { core::mem::zeroed() };
use core::ptr::null_mut;
use melee_world::tu::grinl::{ptr_u32, u32_ptr};""",
    extra_pools=["grCastle_YakumonoParam", "UnkArchiveStruct"],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grCs_804D6974", rs="t::grCs_804D6974", kind="p"),
        dict(c="grCs_804D45E0", rs="t::grCs_804D45E0", kind="f"),
        dict(c="grCs_804D45E4", rs="t::grCs_804D45E4", kind="C"),
        dict(c="OWP", rs="OWP", kind="i", n=3),
        dict(c="OCD", rs="OCD", ty="CollData", n=1, fill=True),
        dict(c="ODD", rs="ODD", ty="DynamicsData", n=3, fill=True),
        dict(c="OAPP", rs="OAPP", ty="HSD_psAppSRT", n=1, fill=True),
        dict(c="grCs_StageCallbacks", rs="t::grCs_StageCallbacks", ty="StageCallbacks", n=21),
        dict(c="grCs_StageData", rs="t::grCs_StageData", ty="StageData"),
        dict(c="grCs_803E0FE8", rs="t::grCs_803E0FE8", ty="GrJoint", n=2),
        dict(c="grCs_803E1198", rs="t::grCs_803E1198", kind="c", n=10, const=True),
        dict(c="grCs_803B7E9C", rs="t::grCs_803B7E9C", ty="Vec3"),
        dict(c="grCs_803B7EA8", rs="t::grCs_803B7EA8", ty="grCastle_DynEntries"),
        dict(c="grCs_803B7EB8", rs="t::grCs_803B7EB8", ty="Quaternion"),
        dict(c="grCs_803B7EC8", rs="t::grCs_803B7EC8", ty="grCastle_BlinkTable"),
        dict(c="grCs_803B7EF0", rs="t::grCs_803B7EF0", ty="grCastle_WeightTable"),
        dict(c="grCs_803B7EFC", rs="t::grCs_803B7EFC", ty="grCastle_TargetTable"),
        dict(c="grCs_803B7F28", rs="t::grCs_803B7F28", ty="grCastle_CallbackTable"),
        dict(c="grCs_803B7F3C", rs="t::grCs_803B7F3C", ty="grCastle_CallbackTable2"),
        dict(c="grCs_803B7F50", rs="t::grCs_803B7F50", ty="grCastle_YOffsets"),
    ],
    graph=[dict(kind="list", arr="OGO", n=6, field="next", z=0)],
    links_c="    NN = 0; yakumono_param = (void*) OP_grCastle_YakumonoParam[0]; grCs_804D6974 = (void*) OP_lb_80011A50_t[0]; OPLH[HSD_GOBJ_PLINK_GROUND] = &OGO[0];",
    links_rs="    NN = 0; t::yakumono_param = addr_of_mut!(OP_grCastle_YakumonoParam[0]) as *mut _; t::grCs_804D6974 = addr_of_mut!(OP_lb_80011A50_t[0]) as *mut _;",
    hooks_rs="""    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);
    melee_world::tu::grcastle::ext::GRDISPLAY_801C5DB0 = Some(|a0: *mut HSD_GObj, a1: i32| { lg_s("grDisplay_801C5DB0("); lg_ptr(a0 as *const u8 as usize); lg_int(a1 as u32); lg_s(")"); lg_c(';'); });
    melee_world::tu::grcastle::ext::HSD_GOBJPLINKHEAD = Some(|_l: i32| addr_of_mut!(OGO[0]));""",
    outs={"lb_8000B1CC": [2], "ftLib_80086644": [1], "ftLib_80086684": [1], "ftLib_80086BEC": [1], "lb_800103B8": [1], "PSVECCrossProduct": [2]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "Ground_801C3FA4": ("((NN || rnd() % 5) ? (void*) &OJO[rnd() % NJO] : (void*) 0)",
                            "(if NN != 0 || rnd() % 5 != 0 { addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _ } else { null_mut() })"),
        "Ground_GetStageGObj": ("((NN || rnd() % 4) ? (void*) &OGO[1 + rnd() % (NGO - 1)] : (void*) 0)",
                                "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[1 + (rnd() % (NGO as u32 - 1)) as usize]) as *mut _ } else { null_mut() })"),
        "Ground_GetMapGObj": ("((NN || rnd() % 4) ? (void*) &OGO[rnd() % NGO] : (void*) 0)",
                              "(if NN != 0 || rnd() % 4 != 0 { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _ } else { null_mut() })"),
        "grMaterial_801C8CFC": ("(rnd() % 4 ? (void*) &OGO[rnd() % NGO] : (void*) 0)",
                                "(if rnd() % 4 != 0 { addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _ } else { null_mut() })"),
        "grDatFiles_GetArchive": ("(void*) OP_UnkArchiveStruct[rnd() % 2]", "addr_of_mut!(OP_UnkArchiveStruct[(rnd() % 2) as usize]) as *mut c_void"),
        "itGetKind": ("(rnd() % 2 ? 160 : (int) (rnd() % 20))", "(if rnd() % 2 != 0 { 160 } else { (rnd() % 20) as i32 })"),
        "HSD_JObjGetFlags": ("rnd()", "rnd()"),
        "Ground_GetYakumonoParam": ("(void*) OP_grCastle_YakumonoParam[0]", "addr_of_mut!(OP_grCastle_YakumonoParam[0]) as *mut c_void"),
    },
    fnvals={**{n: f"t::{n}" for n in CALLBACK_NAMES}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0"},
    funcs=funcs,
)
