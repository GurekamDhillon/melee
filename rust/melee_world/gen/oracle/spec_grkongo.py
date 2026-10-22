import gro_util

CBS = ["grKongo_801D5238", "grKongo_801D523C", "grKongo_801D52F8", "grKongo_801D52FC", "grKongo_801D5338",
       "grKongo_801D5490", "grKongo_801D5574", "grKongo_801D557C", "grKongo_801D55D4", "grKongo_801D55D8",
       "grKongo_801D5774", "grKongo_801D577C", "grKongo_801D5FA4", "grKongo_801D5FA8", "grKongo_801D5FD4",
       "grKongo_801D5FDC", "grKongo_801D5FE0", "grKongo_801D5FE4", "grKongo_801D6028", "grKongo_801D6030",
       "grKongo_801D6034", "grKongo_801D6038", "grKongo_801D6064", "grKongo_801D606C", "grKongo_801D6070",
       "grKongo_801D6074", "grKongo_801D6190", "grKongo_801D6198", "grKongo_801D6378", "grKongo_801D637C",
       "grKongo_801D64B4", "grKongo_801D64BC", "grKongo_801D6518", "grKongo_801D651C", "grKongo_801D6660",
       "grKongo_801D6668", "grKongo_801D69AC", "grKongo_801D8444", "grKongo_801D844C",
       "fn_801D542C", "fn_801D7700", "fn_801D7E60", "fn_801D8134"]
YP = "grKongo_YakumonoParam"
UALL = ["kongo", "kongo2", "kongo3", "inishie2", "nobf"]

RET_OGO = ("((void*) &OGO[rnd() % NGO])", "(addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]) as *mut _)")
RET_OJO = ("((void*) &OJO[rnd() % NJO])", "(addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]) as *mut _)")

# a jobj pointer written over a union slot must be written last
JP_C = ("OGP[0].u.kongo3.xCC = &OJO[rnd() % NJO]; OGP[0].u.kongo3.xD0 = &OJO[rnd() % NJO]; "
        "OGP[0].u.kongo.xDC = &OJO[rnd() % NJO]; OGP[0].u.kongo.xE0 = &OJO[rnd() % NJO];")
JP_R = ("OGP[0].u.kongo3.xCC = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); OGP[0].u.kongo3.xD0 = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); "
        "OGP[0].u.kongo.xDC = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]); OGP[0].u.kongo.xE0 = addr_of_mut!(OJO[(rnd() % NJO as u32) as usize]);")
# the barrel (801D577C / 801D828C): state shorts, a kept gobj for the states that dereference it
KEEP_C = "OGP[0].u.kongo.u.taru.keep = &OGO[1 + AI[3] % 4];"
KEEP_R = "OGP[0].u.kongo.u.taru.keep = addr_of_mut!(OGO[1 + (a.i[3] % 4) as usize]) as *mut c_void;"
BARREL_C = ("OGP[0].u.kongo3.xC4 = AI[0] % 4; OGP[0].u.kongo3.xC8 = AI[1] % 4; OGP[0].u.kongo3.xC6 = AI[2] % 4; "
            "OGP[0].u.kongo3.xCA = (s16) (rnd() % 6) - 2; "
            "if (!(OGP[0].u.kongo3.xE0 > 0.0f || OGP[0].u.kongo3.xE0 < 0.0f)) OGP[0].u.kongo3.xE0 = rnd() % 2 ? -0.002f : 0.001f; " + KEEP_C)
BARREL_R = ("OGP[0].u.kongo3.xC4 = (a.i[0] % 4) as i16; OGP[0].u.kongo3.xC8 = (a.i[1] % 4) as i16; OGP[0].u.kongo3.xC6 = (a.i[2] % 4) as i16; "
            "OGP[0].u.kongo3.xCA = (rnd() % 6) as i16 - 2; "
            "if !(OGP[0].u.kongo3.xE0 > 0.0 || OGP[0].u.kongo3.xE0 < 0.0) { OGP[0].u.kongo3.xE0 = if rnd() % 2 != 0 { -0.002 } else { 0.001 }; } " + KEEP_R)
# the platforms (801D6198): state 0..3
PLAT_C = "OGP[0].u.kongo3.xC4 = AI[0] % 4; OGP[0].u.kongo3.xC6 = (s16) (rnd() % 8) - 2; OGP[0].u.kongo3.xC8 = (s16) (rnd() % 8) - 2; OGP[0].u.kongo3.xCA = (s16) (rnd() % 8) - 2;"
PLAT_R = "OGP[0].u.kongo3.xC4 = (a.i[0] % 4) as i16; OGP[0].u.kongo3.xC6 = (rnd() % 8) as i16 - 2; OGP[0].u.kongo3.xC8 = (rnd() % 8) as i16 - 2; OGP[0].u.kongo3.xCA = (rnd() % 8) as i16 - 2;"
# the spline platform (801D6668): progress in [0, 1]
SPL_C = "OGP[0].u.kongo2.xC4 = (HSD_Spline*) OP_HSD_Spline[0]; OGP[0].u.kongo.xCC = (float) (rnd() % 1001) / 1000.0f;"
SPL_R = "OGP[0].u.kongo2.xC4 = addr_of_mut!(OP_HSD_Spline[0]) as *mut HSD_Spline; OGP[0].u.kongo.xCC = (rnd() % 1001) as f32 / 1000.0;"
# the hazard roll (801D7BBC)
ROLL_C = "OGP[0].u.kongo.xE4 = (s16) (AI[0] % 4) - 1; OGP[0].u.kongo.xE6 = AI[1] % 4;"
ROLL_R = "OGP[0].u.kongo.xE4 = (a.i[0] % 4) as i16 - 1; OGP[0].u.kongo.xE6 = (a.i[1] % 4) as i16;"
HIT_C = "((CollData*) OP_CollData[0])->x34_flags.b1234 = AI[3];"
HIT_R = "(*(addr_of_mut!(OP_CollData[0]) as *mut CollData)).x34_flags.set_b1234(a.i[3] as u8);"

CB6 = ["&OGP[0]", "AI[0]", "(CollData*) OP_CollData[0]", "AI[1]", "AI[2] % 4", "AF[0]"]
CB6R = ["addr_of_mut!(OGP[0]) as *mut c_void", "a.i[0] as i32", "addr_of_mut!(OP_CollData[0]) as *mut CollData", "a.i[1] as i32", "(a.i[2] % 4) as _", "a.f[0]"]


def fn(name, u=None, pre=None, **kw):
    d = dict(name=name)
    if u is not None:
        d["u"] = u
    if pre:
        d["pre_c"], d["pre_rs"] = pre
    d.update(kw)
    return d


SPEC = dict(
    src="src/melee/gr/grkongo.c",
    rust_mod="grkongo",
    bf_skip=["StageCallbacks", "CollData"],
    u=UALL,
    ext_mods=["grinl", "grinl2"],
    globals=[dict(c="stage_info", ty="StageInfo")],
    provides=["HSD_GObjPLinkHead", "efSync_Spawn", "ftCo_8009EC70"],
    extra_pools=[YP, "CollData", "HSD_Spline"],
    patch=[
        # grKongo_801D77E0 steps a Ground* by GC +0x10 from xC4/xC8 to xD4/xD8 (the second pair); natively
        # that offset is another field, so the patched C walks a local copy of the two (angle, velocity) pairs
        ("void grKongo_801D77E0(HSD_GObj* gobj, s32 arg1)\n{",
         "typedef struct KQ { union { struct { f32 xC4; f32 xC8; } kongo; } u; char pad[8]; } KQ;\n"
         "void grKongo_801D77E0(HSD_GObj* gobj, s32 arg1)\n{"),
        ("        Ground* q = gp;\n        for (i = 2; i != 0; i--) {",
         "        KQ loc[2]; KQ* q = loc;\n"
         "        loc[0].u.kongo.xC4 = gp->u.kongo.xC4; loc[0].u.kongo.xC8 = gp->u.kongo.xC8;\n"
         "        loc[1].u.kongo.xC4 = gp->u.kongo.xD4; loc[1].u.kongo.xC8 = gp->u.kongo.xD8;\n"
         "        for (i = 2; i != 0; i--) {"),
        ("            q = (Ground*) ((u8*) q + 0x10);\n        }\n    }\n    HSD_JObjSetRotationZ(gp->u.kongo3.xCC,",
         "            q = (KQ*) ((u8*) q + 0x10);\n        }\n"
         "        gp->u.kongo.xC4 = loc[0].u.kongo.xC4; gp->u.kongo.xC8 = loc[0].u.kongo.xC8;\n"
         "        gp->u.kongo.xD4 = loc[1].u.kongo.xC4; gp->u.kongo.xD8 = loc[1].u.kongo.xC8;\n"
         "    }\n    HSD_JObjSetRotationZ(gp->u.kongo3.xCC,"),
    ],
    statics=[
        dict(c="yakumono_param", rs="t::yakumono_param", kind="p"),
        dict(c="grKg_803E16E0", rs="t::grKg_803E16E0", ty="GrJoint", n=6),
        dict(c="grKg_StageCallbacks", rs="t::grKg_StageCallbacks", ty="StageCallbacks", n=12),
        dict(c="grKg_StageData", rs="t::grKg_StageData", ty="StageData"),
        dict(c="grKg_803E188C", rs="t::grKg_803E188C", ty="_struct_grKg_803E188C_0x18", n=15),
        dict(c="grKg_804D6984.unk0", rs="t::grKg_804D6984.unk0", kind="p", fill=False),
        dict(c="grKg_804D6984.unk4", rs="t::grKg_804D6984.unk4", kind="p", fill=False),
        dict(c="HSD_GObjPLinkHead", rs="t::ext::HSD_GObjPLinkHead", kind="p"),
        dict(c="PL", rs="PL", kind="p", n=16),
    ] + gro_util.cb_flags("grKg_StageCallbacks", "t::grKg_StageCallbacks", 12),
    data_c=("HSD_GObj** HSD_GObjPLinkHead; static HSD_GObj* PL[16];\n"
            "void* efSync_Spawn(s32 a0, HSD_GObj* a1, ...)\n{\n    __builtin_va_list ap; Vec3* v; void* r;\n"
            "    __builtin_va_start(ap, a1); v = __builtin_va_arg(ap, Vec3*); __builtin_va_end(ap);\n"
            "    lg_s(\"efSync_Spawn(\"); lg_int((u32) a0); lg_ptr((const void*) a1); lg_v3((const Vec3*) v); lg_s(\")\");\n"
            "    r = gen_p(); lg_retp(r); lg_c(';'); return r;\n}\n"
            "void ftCo_8009EC70(Fighter_GObj* gobj, Vec3* pos, UNK_T arg2, float kb_angle)\n{\n"
            "    lbColl_80008D30_arg1* h = (lbColl_80008D30_arg1*) arg2;\n"
            "    lg_s(\"ftCo_8009EC70(\"); lg_ptr((const void*) gobj); lg_v3((const Vec3*) pos);\n"
            "    lg_int((u32) h->state); lg_int(h->damage); lg_int((u32) h->kb_angle); lg_int(h->unkC); lg_int(h->unk10); lg_int(h->unk14);\n"
            "    lg_int(h->element); lg_int((u32) h->sfx_severity); lg_int((u32) h->sfx_kind); lg_f(kb_angle); lg_s(\")\"); lg_c(';');\n}"),
    use_rs=("static mut PL: [*mut HSD_GObj; 16] = [core::ptr::null_mut(); 16];"),
    links_c="""    {
        int k;
        static const int mids[] = { 1, 2, 7, 8, 9, 11, 4, 5 };
        struct grKongo_YakumonoParam* yp = (struct grKongo_YakumonoParam*) OP_%s[0];
        yakumono_param = yp;
        yp->unk44 = rnd() %% 4; yp->unk46 = rnd() %% 4; yp->unk48 = rnd() %% 4; yp->unk4A = rnd() %% 4;
        yp->unk4C = rnd() %% 4; yp->unk4E = rnd() %% 4; yp->unk50 = rnd() %% 4; yp->unk52 = 1 + rnd() %% 3;
        yp->unk90 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unk94 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unk98 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unk9C = (float) (1 + rnd() %% 3000) / 977.0f; yp->unkA0 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unkA4 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unkA8 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unkAC = (float) (1 + rnd() %% 3000) / 977.0f; yp->unkB0 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unkB4 = (float) (1 + rnd() %% 3000) / 977.0f; yp->unkB8 = (float) (1 + rnd() %% 3000) / 977.0f; 
        for (k = 0; k < NGO; k++) {
            OGP[k].map_id = mids[rnd() %% 8];
            OGO[k].p_link = rnd() %% 3 == 0 ? 8 : rnd() %% 2 ? 9 : 3;
        }
        for (k = 0; k < 15; k++) {
            grKg_803E188C[k].unk4 = &OJO[rnd() %% NJO];
            grKg_803E188C[k].unk2 = rnd() %% 4 == 0;
            grKg_803E188C[k].unk10 = (float) ((int) (rnd() %% 2001) - 1000) / 997.0f;
        }
        grKg_804D6984.unk0 = &OJO[rnd() %% NJO]; grKg_804D6984.unk4 = &OJO[rnd() %% NJO];
        for (k = 0; k < 16; k++) PL[k] = 0;
        HSD_GObjPLinkHead = PL;
        PL[5] = &OGO[0]; OGO[0].next = &OGO[1]; OGO[1].next = rnd() %% 3 ? &OGO[2] : 0; OGO[2].next = 0;
        PL[9] = rnd() %% 4 ? &OGO[3] : 0; OGO[3].next = &OGO[4]; OGO[4].next = 0;
    }""" % YP,
    links_rs="""    {
        let mids: [i32; 8] = [1, 2, 7, 8, 9, 11, 4, 5];
        let yp = addr_of_mut!(OP_%s[0]) as *mut t::grKongo_YakumonoParam;
        t::yakumono_param = yp;
        (*yp).unk44 = (rnd() %% 4) as i16; (*yp).unk46 = (rnd() %% 4) as i16; (*yp).unk48 = (rnd() %% 4) as i16; (*yp).unk4A = (rnd() %% 4) as i16;
        (*yp).unk4C = (rnd() %% 4) as i16; (*yp).unk4E = (rnd() %% 4) as i16; (*yp).unk50 = (rnd() %% 4) as i16; (*yp).unk52 = (1 + rnd() %% 3) as i16;
        (*yp).unk90 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unk94 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unk98 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unk9C = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unkA0 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unkA4 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unkA8 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unkAC = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unkB0 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unkB4 = (1 + rnd() %% 3000) as f32 / 977.0; (*yp).unkB8 = (1 + rnd() %% 3000) as f32 / 977.0; 
        for k in 0..NGO {
            OGP[k].map_id = mids[(rnd() %% 8) as usize] as _;
            OGO[k].p_link = if rnd() %% 3 == 0 { 8 } else if rnd() %% 2 != 0 { 9 } else { 3 };
        }
        for k in 0..15usize {
            t::grKg_803E188C[k].unk4 = addr_of_mut!(OJO[(rnd() %% NJO as u32) as usize]);
            t::grKg_803E188C[k].unk2 = (rnd() %% 4 == 0) as i16;
            t::grKg_803E188C[k].unk10 = ((rnd() %% 2001) as i32 - 1000) as f32 / 997.0;
        }
        t::grKg_804D6984.unk0 = addr_of_mut!(OJO[(rnd() %% NJO as u32) as usize]); t::grKg_804D6984.unk4 = addr_of_mut!(OJO[(rnd() %% NJO as u32) as usize]);
        for k in 0..16 { PL[k] = core::ptr::null_mut(); }
        t::ext::HSD_GObjPLinkHead = addr_of_mut!(PL) as *mut *mut HSD_GObj;
        PL[5] = addr_of_mut!(OGO[0]); OGO[0].next = addr_of_mut!(OGO[1]);
        OGO[1].next = if rnd() %% 3 != 0 { addr_of_mut!(OGO[2]) } else { core::ptr::null_mut() }; OGO[2].next = core::ptr::null_mut();
        PL[9] = if rnd() %% 4 != 0 { addr_of_mut!(OGO[3]) } else { core::ptr::null_mut() }; OGO[3].next = addr_of_mut!(OGO[4]); OGO[4].next = core::ptr::null_mut();
    }""" % YP,
    hooks_rs=("""    melee_world::tu::grinl::ext::STAGE_INFO = Some(|| addr_of_mut!(G_stage_info) as *mut StageInfo);
    melee_world::tu::grkongo::ext::EFSYNC_SPAWN = Some(|a0, a1, a2| { lg_s("efSync_Spawn("); lg_int(a0 as u32); lg_ptr(a1 as usize); lg_v3(a2 as *const Vec3); lg_s(")"); let r = gen_p(); lg_retp(r as usize); lg_c(';'); r });
    melee_world::tu::grkongo::ext::FTCO_8009EC70 = Some(|gobj, pos, h, kb| { lg_s("ftCo_8009EC70("); lg_ptr(gobj as usize); lg_v3(pos as *const Vec3);
        lg_int((*h).state as u32); lg_int((*h).damage); lg_int((*h).kb_angle as u32); lg_int((*h).unkC); lg_int((*h).unk10); lg_int((*h).unk14);
        lg_int((*h).element); lg_int((*h).sfx_severity as u32); lg_int((*h).sfx_kind as u32); lg_f(kb); lg_s(")"); lg_c(';'); });"""),
    no_hook=["grDisplay_801C5DB0"],
    outs={"lb_8000B1CC": [2], "Ground_801C4DA0": [0, 1], "ftLib_80086644": [1], "it_8026B294": [1],
          "Ground_801C4B50": [1, 2], "splArcLengthPoint": [0], "mpVtxGetPos": [1, 2]},
    rets={
        "HSD_Randi": ("(int) (a0 > 0 ? rnd() % (u32) a0 : 0)", "(if a0 > 0 { rnd() % (a0 as u32) } else { 0 })"),
        "Ground_GetStageGObj": ("((rnd() % 4) ? (void*) &OGO[(u32) a0 % NGO] : (void*) 0)",
                                "(if rnd() % 4 != 0 { addr_of_mut!(OGO[(a0 as u32 % NGO as u32) as usize]) as *mut _ } else { core::ptr::null_mut() })"),
        "Ground_GetMapGObj": RET_OGO,
        "Ground_GetYakumonoParam": ("((void*) OP_%s[0])" % YP, "(addr_of_mut!(OP_%s[0]) as *mut _)" % YP),
        "Ground_801C3FA4": RET_OJO,
        "Stage_80225194": ("(rnd() % 2 ? 0x3D : (int) (rnd() % 100))", "(if rnd() % 2 != 0 { 0x3D } else { (rnd() % 100) as i32 })"),
        "gm_8016B238": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
        "it_8026D324": ("(int) (rnd() % 2)", "((rnd() % 2) as i32)"),
        "itGetKind": ("(rnd() % 2 ? 218 : (int) (rnd() % 300))", "(if rnd() % 2 != 0 { 218 } else { (rnd() % 300) as i32 })"),
    },
    fnvals={**{n: "t::" + n for n in CBS}, "grDisplay_801C5DB0": "t::ext::grDisplay_801C5DB0"},
    funcs=[
        fn("grKongo_801D5238", c_args=["AI[0] % 2"], rs_args=["(a.i[0] % 2) as _"]),
        fn("grKongo_801D523C"), fn("grKongo_801D52F8"), fn("grKongo_801D52FC"), fn("grKongo_801D5338"),
        fn("grKongo_801D5340", pre=("AI[0] = rnd() % 12;", "a.i[0] = rnd() % 12;"), c_args=["(int) (AI[0] % 12)"], rs_args=["(a.i[0] % 12) as i32"]),
        fn("fn_801D542C", n=4000),
        fn("grKongo_801D5490"), fn("grKongo_801D5574"), fn("grKongo_801D557C", n=4000, pre=(JP_C, JP_R)), fn("grKongo_801D55D4"),
        fn("grKongo_801D55D8"), fn("grKongo_801D5774"),
        fn("grKongo_801D577C", n=4000, pre=(BARREL_C, BARREL_R)),
        fn("grKongo_801D5FA4"), fn("grKongo_801D5FA8"), fn("grKongo_801D5FD4"), fn("grKongo_801D5FDC"),
        fn("grKongo_801D5FE0"), fn("grKongo_801D5FE4"), fn("grKongo_801D6028"), fn("grKongo_801D6030"),
        fn("grKongo_801D6034"), fn("grKongo_801D6038"), fn("grKongo_801D6064"), fn("grKongo_801D606C"),
        fn("grKongo_801D6070"), fn("grKongo_801D6074"), fn("grKongo_801D6190"),
        fn("grKongo_801D6198", pre=(PLAT_C, PLAT_R)),
        fn("grKongo_801D6378"), fn("grKongo_801D637C"), fn("grKongo_801D64B4"), fn("grKongo_801D64BC"),
        fn("grKongo_801D6518"), fn("grKongo_801D651C"), fn("grKongo_801D6660"),
        fn("grKongo_801D6668", pre=(SPL_C, SPL_R)),
        fn("grKongo_801D69AC"), fn("grKongo_801D69B0", pre=(JP_C, JP_R)), fn("grKongo_801D6AFC", n=4000),
        fn("grKongo_801D7134", n=4000, c_args=["&OGO[0]", "AI[0]"], rs_args=["g", "a.i[0] as _"]),
        fn("fn_801D7700", n=4000, pre=(HIT_C, HIT_R), c_args=CB6, rs_args=CB6R),
        fn("grKongo_801D77E0", n=4000, pre=(JP_C, JP_R), c_args=["&OGO[0]", "AI[0] % 2"], rs_args=["g", "(a.i[0] % 2) as _"]),
        fn("grKongo_801D7BBC", n=4000, pre=(ROLL_C, ROLL_R)),
        fn("fn_801D7E60", c_args=CB6, rs_args=CB6R),
        fn("grKongo_801D7E78", c_args=["&OGO[AI[0] % NGO]", "&AV3[0]"], rs_args=["addr_of_mut!(OGO[(a.i[0] % NGO as u32) as usize])", "&mut a.v3[0]"]),
        fn("grKongo_801D7F78"), fn("grKongo_801D8058"), fn("grKongo_801D8078"),
        fn("fn_801D8134", n=4000, pre=("OGP[0].u.kongo3.xC6 = AI[0] % 2;", "OGP[0].u.kongo3.xC6 = (a.i[0] % 2) as i16;"),
           c_args=["&OGO[0]", "&OGO[1]"], rs_args=["g", "addr_of_mut!(OGO[1])"]),
        fn("grKongo_801D8270"),
        fn("grKongo_801D828C", pre=("OGP[0].u.kongo3.xC6 = AI[0] % 3; " + KEEP_C, "OGP[0].u.kongo3.xC6 = (a.i[0] % 3) as i16; " + KEEP_R)),
        fn("grKongo_801D8314"), fn("grKongo_801D8444", c_args=["AI[0]"], rs_args=["a.i[0] as _"]),
        fn("grKongo_801D844C", c_args=["&AV3[0]", "AI[0]", "&OJO[0]"], rs_args=["&mut a.v3[0]", "a.i[0] as _", "addr_of_mut!(OJO[0])"]),
    ],
)
gro_util.merge(SPEC, gro_util.snap([
    dict(c="grKg_803E16E0", rs="t::grKg_803E16E0", ty="GrJoint", n=6),
    dict(c="grKg_StageCallbacks", rs="t::grKg_StageCallbacks", ty="StageCallbacks", n=12),
    dict(c="grKg_StageData", rs="t::grKg_StageData", ty="StageData"),
]))
