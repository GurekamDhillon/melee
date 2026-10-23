import gro_util

CBS = ["stageGObj0_OnInit", "grInishie2_801FCDF4", "grInishie2_801FCDFC", "grInishie2_801FCE00",
       "grInishie2_801FDE8C", "grInishie2_801FDED0", "grInishie2_801FDED8", "grInishie2_801FDFE4",
       "grInishie2_801FCE04", "grInishie2_801FCF24", "grInishie2_801FCFCC", "grInishie2_801FD014",
       "grInishie2_801FD198", "grInishie2_801FD21C", "grInishie2_801FD224", "grInishie2_801FD364",
       "grInishie2_801FD4F0", "grInishie2_801FD64C", "grInishie2_801FD654", "grInishie2_801FD740",
       "grInishie2_801FCBC0", "grInishie2_801FCBC4", "grInishie2_801FCCAC", "grInishie2_801FCCB0",
       "grInishie2_801FCCD4", "grInishie2_801FDFE8", "grInishie2_801FDFF0"]
YP = "grInishie2_YakumonoParam"
# Ground unions: inishie2 (flags/shorts), inishie22 (three gobj pointers), inishie23 (flags/shorts/vecs)
U2 = ["inishie2"]
U22 = ["inishie22"]
U23 = ["inishie23"]
U2_22 = ["inishie2", "inishie22"]
MIX = ["inishie2", "inishie22", "inishie23", "nobf"]  # a pointer written over the flags byte: bits are not compared
P22 = ["inishie2", "inishie22", "nobf"]  # same, without inishie23 (its xC4 short would overlay the flags byte)


def flags_pre(bits):
    c = " ".join(f"OGP[0].u.inishie2.xC4_flags.b{b} = (AI[1] >> {b}) & 1;" for b in bits)
    r = " ".join(f"OGP[0].u.inishie2.xC4_flags.set_b{b}(((a.i[1] >> {b}) & 1) as u8);" for b in bits)
    return (c, r)


# every stage Ground's map_id is 3 or 4 (it indexes yakumono_param->unk10[])
MAPID_C = "{ int k; for (k = 0; k < NGO; k++) OGP[k].map_id = 3 + rnd() % 2; }"
MAPID_R = "for k in 0..NGO { OGP[k].map_id = (3 + rnd() % 2) as _; }"
# OGP[0].u.inishie22.<field> = another gobj (or NULL)
def ptr_pre(fld, nonnull=False):
    if nonnull:
        return (f"OGP[0].u.inishie22.{fld} = &OGO[1 + AI[2] % 4];",
                f"OGP[0].u.inishie22.{fld} = addr_of_mut!(OGO[1 + (a.i[2] % 4) as usize]);")
    return (f"OGP[0].u.inishie22.{fld} = AI[1] % 3 ? &OGO[1 + AI[2] % 4] : 0;",
            f"OGP[0].u.inishie22.{fld} = if a.i[1] % 3 != 0 {{ addr_of_mut!(OGO[1 + (a.i[2] % 4) as usize]) }} else {{ core::ptr::null_mut() }};")


def fn(name, u=None, pre=None, **kw):
    d = dict(name=name)
    if u is not None:
        d["u"] = u
    if pre:
        d["pre_c"], d["pre_rs"] = pre
    d.update(kw)
    return d


SPEC = dict(
    src="src/melee/gr/grinishie2.c",
    rust_mod="grinishie2",
    bf_skip=["StageCallbacks"],
    u=U2,
    ext_mods=["grinl", "grinl2"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    extra_pools=[YP],
    patch=[
        # six calls in one argument list: the shipping PPC build evaluates left to right, this
        # x86-64 MSVC-target oracle right to left
        ("    Ground_801C4E70(Ground_801C3FA4(gobj, 9), Ground_801C3FA4(gobj, 0xC),\n"
         "                    Ground_801C3FA4(gobj, 8), Ground_801C3FA4(gobj, 0xB),\n"
         "                    Ground_801C3FA4(gobj, 0xA), Ground_801C3FA4(gobj, 0xD));",
         "    { HSD_JObj* a0_ = Ground_801C3FA4(gobj, 9); HSD_JObj* a1_ = Ground_801C3FA4(gobj, 0xC);\n"
         "      HSD_JObj* a2_ = Ground_801C3FA4(gobj, 8); HSD_JObj* a3_ = Ground_801C3FA4(gobj, 0xB);\n"
         "      HSD_JObj* a4_ = Ground_801C3FA4(gobj, 0xA); HSD_JObj* a5_ = Ground_801C3FA4(gobj, 0xD);\n"
         "      Ground_801C4E70(a0_, a1_, a2_, a3_, a4_, a5_); }"),
    ],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grI2_803E4A60", rs="t::grI2_803E4A60", ty="GrJoint", n=14),
        dict(c="grI2_StageCallbacks", rs="t::grI2_StageCallbacks", ty="StageCallbacks", n=16),
        dict(c="grI2_StageData", rs="t::grI2_StageData", ty="StageData"),
    ] + gro_util.cb_flags("grI2_StageCallbacks", "t::grI2_StageCallbacks", 16),
    links_c="    yakumono_param = (void*) OP_%s[0];\n    %s" % (YP, MAPID_C),
    links_rs="    t::yakumono_param = addr_of_mut!(OP_%s[0]) as *mut _;\n    %s" % (YP, MAPID_R),
    hooks_rs="    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);",
    outs={"GetMatchTimer": [0], "it_802ECD1C": [1], "lb_8000B1CC": [2]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "Ground_GetYakumonoParam": ("((void*) OP_%s[0])" % YP, "(addr_of_mut!(OP_%s[0]) as *mut _)" % YP),
        "Ground_801C3FA4": ("((void*) &OJO[rnd() % NJO])", "(addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _)"),
        "Ground_801C5A94": ("(rnd() % 3 == 0 ? 0x29 : rnd() % 3 == 0 ? 0x2B : (int) (rnd() % 100))",
                            "(if rnd() % 3 == 0 { 0x29 } else if rnd() % 3 == 0 { 0x2B } else { (rnd() % 100) as i32 })"),
    },
    fnvals={**{n: "t::" + n for n in CBS}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0"},
    funcs=[
        fn("grInishie2_801FCBC0"), fn("grInishie2_801FCBC4"), fn("grInishie2_801FCCAC"), fn("grInishie2_801FCCB0"),
        fn("grInishie2_801FCCD4"),
        fn("grInishie2_801FCCDC", u=MIX, c_args=["((int[]) { 0, 1, 2, 3, 4, 15 })[AI[0] % 6]"],
           rs_args=["[0i32, 1, 2, 3, 4, 15][(a.i[0] % 6) as usize]"]),
        fn("stageGObj0_OnInit"),
        fn("grInishie2_801FCDF4"), fn("grInishie2_801FCDFC"), fn("grInishie2_801FCE00"),
        fn("grInishie2_801FCE04"),
        fn("grInishie2_801FCF24"), fn("grInishie2_801FCF2C"),
        fn("grInishie2_801FCFCC", u=P22, pre=flags_pre([0, 1, 2, 3, 4])),
        fn("grInishie2_801FD014"),
        fn("grInishie2_801FD018"), fn("grInishie2_801FD0B4", u=U2_22),
        fn("grInishie2_801FD198"), fn("grInishie2_801FD21C"),
        fn("grInishie2_801FD224", u=U2_22, pre=ptr_pre("xCC", True)),
        fn("grInishie2_801FD364"),
        fn("grInishie2_801FD368"), fn("grInishie2_801FD3CC", u=U2_22),
        fn("grInishie2_801FD448", u=U2_22, pre=ptr_pre("xC8", True)),
        fn("grInishie2_801FD4CC"),
        fn("grInishie2_801FD4F0", u=U22),
        fn("grInishie2_801FD64C"),
        fn("grInishie2_801FD654", u=U22, pre=ptr_pre("xC4")),
        fn("grInishie2_801FD740"),
        fn("grInishie2_801FD744"), fn("grInishie2_801FD7A8", u=P22, pre=flags_pre([4])),
        fn("grInishie2_801FD824", u=U23), fn("grInishie2_801FD9EC", u=U23),
        fn("grInishie2_801FDE8C", u=U23),
        fn("grInishie2_801FDED0"),
        # the walker's `inishie22.xC4` pointer shares its first 8 bytes with `inishie23.xC4/xC6/xC8_flags`
        # natively (4 bytes on the GC): the branch behind `xC8_flags.b0` is not reachable here
        fn("grInishie2_801FDED8", u=U23, pre=(
            "OGP[0].u.inishie23.xC8_flags.b0 = 0; OGP[0].u.inishie23.xC8_flags.b1 = 0; OGP[0].u.inishie23.xC8_flags.b2 = 0; OGP[0].u.inishie23.xCA = AI[1] % 2;",
            "OGP[0].u.inishie23.xC8_flags.set_b0(0); OGP[0].u.inishie23.xC8_flags.set_b1(0); OGP[0].u.inishie23.xC8_flags.set_b2(0); OGP[0].u.inishie23.xCA = (a.i[1] % 2) as i16;")),
        fn("grInishie2_801FDFB4"), fn("grInishie2_801FDFE4"), fn("grInishie2_801FDFE8"),
        fn("grInishie2_801FDFF0", c_args=["&AV3[0]", "AI[0]", "&OJO[0]"], rs_args=["&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[0])"]),
    ],
)
gro_util.merge(SPEC, gro_util.snap([
    dict(c="grI2_803E4A60", rs="t::grI2_803E4A60", ty="GrJoint", n=14),
    dict(c="grI2_StageCallbacks", rs="t::grI2_StageCallbacks", ty="StageCallbacks", n=16),
    dict(c="grI2_StageData", rs="t::grI2_StageData", ty="StageData"),
]))
