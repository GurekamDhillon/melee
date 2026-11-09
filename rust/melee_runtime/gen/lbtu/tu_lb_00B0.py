"""Oracle spec + world of src/melee/lb/lb_00B0.c (imported by specs.py / cfg.py)."""
L = "src/melee/lb/"

_Q = lambda a: (f"if ({a}) {{ {a}->x = rw(); {a}->y = rw(); {a}->z = rw(); {a}->w = rw(); }}",
                f"if !{a}.is_null() {{ (*{a}).x = rw(); (*{a}).y = rw(); (*{a}).z = rw(); (*{a}).w = rw(); }}")
SPEC = dict(
    src=L + "lb_00B0.c",
    types=["HSD_JObj", "HSD_Joint", "HSD_AObj", "HSD_DObj", "HSD_PObj", "HSD_ShapeSet", "HSD_MObj", "HSD_TObj", "HSD_RObj",
           "HSD_AnimJoint", "HSD_AObjDesc", "HSD_TevDesc", "HSD_TExpTevDesc", "HSD_TExp", "HSD_LObj", "DynamicModelDesc", "Vec"],
    ret={"HSD_JObjGetFlags": ("a0->flags", "(*a0).flags"),
         "HSD_LObjGetFlags": ("(a0 == &LO[2] ? 0 : (fv() & 0x20))", "if a0 == &raw mut crate::LO[2] { 0 } else { fv() & 0x20 }"),
         "HSD_RObjAlloc": ("((fv() & 3) ? &RO[0] : 0)", "if fv() & 3 != 0 { &raw mut crate::RO[0] } else { core::ptr::null_mut() }"),
         "HSD_RObjGetByType": ("((fv() % 3) == 0 ? 0 : &RO[fv() % 3])", "if fv() % 3 == 0 { core::ptr::null_mut() } else { &raw mut crate::RO[(fv() % 3) as usize] }")},
    null_ret_ok=["HSD_RObjAlloc", "HSD_RObjGetByType"],
    effects={
        "EulerToQuat": _Q("a1"),
        "QuatSlerp": _Q("a2"),
        "PSMTXMultVec": ("if (a2) { a2->x = rw(); a2->y = rw(); a2->z = rw(); }", "if !a2.is_null() { (*a2).x = rw(); (*a2).y = rw(); (*a2).z = rw(); }"),
        "HSD_JObjClearFlags": ("a0->flags &= ~a1;", "(*a0).flags &= !a1;"),
        "HSD_JObjSetFlags": ("a0->flags |= a1;", "(*a0).flags |= a1;"),
        "HSD_JObjSetMtxDirtySub": ("a0->flags |= 0x40;", "(*a0).flags |= 0x40;"),
        "HSD_JObjSetupMatrixSub": (
            "{ int i, j; a0->flags &= ~0x40u; for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) a0->mtx[i][j] = rf(); }",
            "(*a0).flags &= !0x40u32; for i in 0..3 { for j in 0..4 { (*a0).mtx[i][j] = rf(); } }"),
        "HSD_JObjWalkTree": (
            "T(B(**a2)); if (a1) { a1(a0, a2, (int) fv()); }",
            "t((**a2).to_bits()); if let Some(f) = a1 { f(a0, a2, fv() as i32); }"),
    },
)

_I4 = lambda n: (f"(int) (fv() % {n})", f"(fv() % {n}) as i32")
CFG = dict(
    world=[("HSD_JObj", "JO", 4), ("HSD_Joint", "JT", 2), ("HSD_AObj", "AO", 4), ("HSD_DObj", "DO", 3), ("HSD_PObj", "PO", 3),
           ("HSD_ShapeSet", "SS", 2), ("HSD_MObj", "MO", 2), ("HSD_TObj", "TO", 3), ("HSD_RObj", "RO", 3),
           ("HSD_AnimJoint", "AJ", 4), ("HSD_AObjDesc", "AD", 2), ("HSD_TevDesc", "TD", 3), ("HSD_TExpTevDesc", "TED", 2),
           ("HSD_TExp", "TE", 3), ("HSD_LObj", "LO", 3), ("DynamicModelDesc", "DMD", 1), ("Vec", "V", 3),
           ("unsigned char", "MZ", 64)],
    rtype={"unsigned char": "u8"},
    c_decl="static float FL[4];\nstatic float* FPP[2];\nstatic HSD_AnimJoint* AJP[4];\nstatic HSD_MatAnimJoint* MAJP[4];\nstatic HSD_ShapeAnimJoint* SAJP[4];\nstatic unsigned char MAJB[4][16], SAJB[4][16];",
    rs_decl="static mut FL: [f32; 4] = [0.0; 4];\nstatic mut FPP: [*mut f32; 2] = [null_mut(); 2];\nstatic mut AJP: [*mut HSD_AnimJoint; 4] = [null_mut(); 4];\nstatic mut MAJP: [*mut HSD_MatAnimJoint; 4] = [null_mut(); 4];\nstatic mut SAJP: [*mut HSD_ShapeAnimJoint; 4] = [null_mut(); 4];\nstatic mut MAJB: [[u8; 16]; 4] = [[0; 16]; 4];\nstatic mut SAJB: [[u8; 16]; 4] = [[0; 16]; 4];",
    c_preg=["preg(FL, sizeof FL);", "preg(MAJB, sizeof MAJB);", "preg(SAJB, sizeof SAJB);", "preg(FPP, sizeof FPP);", "preg(AJP, sizeof AJP);", "preg(MAJP, sizeof MAJP);", "preg(SAJP, sizeof SAJP);"],
    rs_preg=["preg(&raw const FL, 16); preg(&raw const MAJB, 64); preg(&raw const SAJB, 64); preg(&raw const FPP, core::mem::size_of_val(&FPP)); preg(&raw const AJP, core::mem::size_of_val(&AJP)); preg(&raw const MAJP, core::mem::size_of_val(&MAJP)); preg(&raw const SAJP, core::mem::size_of_val(&SAJP));"],
    argmap={"HSD_JObj": [(f"&JO[{i}]", f"&raw mut JO[{i}]") for i in range(4)],
            "HSD_Joint": [(f"&JT[{i}]", f"&raw mut JT[{i}]") for i in range(2)],
            "HSD_DObj": [(f"&DO[{i}]", f"&raw mut DO[{i}]") for i in range(3)],
            "HSD_AnimJoint": [(f"&AJ[{i}]", f"&raw mut AJ[{i}]") for i in range(4)],
            "HSD_MObj": [(f"&MO[{i}]", f"&raw mut MO[{i}]") for i in range(2)],
            "HSD_TExp": [("&TE[0]", "&raw mut TE[0]")],
            "HSD_TevDesc": [("&TD[0]", "&raw mut TD[0]")], "_HSD_TevDesc": [("&TD[0]", "&raw mut TD[0]")],
            "HSD_LObj": [("&LO[0]", "&raw mut LO[0]")],
            "DynamicModelDesc": [("&DMD[0]", "&raw mut DMD[0]")],
            "Vec": [(f"&V[{i}]", f"&raw mut V[{i}]") for i in range(3)],
            "void": [("MZ", "MZ.as_mut_ptr() as *mut core::ffi::c_void")],
            "float *": [("&FPP[0]", "&raw mut FPP[0]")]},
    hints={"lb_8000C07C:arg3": ("((fv() & 1) ? AJP : 0)", "if fv() & 1 != 0 { AJP.as_mut_ptr() } else { null_mut() }"),
           "lb_8000C07C:arg4": ("((fv() & 1) ? MAJP : 0)", "if fv() & 1 != 0 { MAJP.as_mut_ptr() } else { null_mut() }"),
           "lb_8000C07C:arg5": ("((fv() & 1) ? SAJP : 0)", "if fv() & 1 != 0 { SAJP.as_mut_ptr() } else { null_mut() }"),
           "lb_8000C07C:i": _I4(4), "lb_8000C0E8:i": _I4(4),
           "memzero:size": ("(long long) (fv() % 65)", "(fv() % 65) as _"),
           "lbGetFreeColorRegImpl:i0": _I4(9), "lbGetFreeAlphaRegImpl:i0": _I4(9),
           "lbGetFreeColorRegister:i0": _I4(9), "lbGetFreeAlphaRegister:i0": _I4(9),
           "lb_8000CC8C:i": _I4(8), "lb_8000CCA4:i": _I4(4), "lb_8000CD90:i": _I4(8), "lb_8000CDA8:i": _I4(4),
           "lb_8000B9D8:arg2": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
           "lbGetFreeColorRegImpl:texp2": ("((fv() & 1) ? &TE[1] : 0)", "if fv() & 1 != 0 { &raw mut TE[1] } else { null_mut() }"),
           "lbGetFreeColorRegImpl:texp1": ("((fv() & 1) ? &TE[0] : 0)", "if fv() & 1 != 0 { &raw mut TE[0] } else { null_mut() }"),
           "lbGetFreeAlphaRegImpl:arg2": ("0", "null_mut()"), "lbGetFreeAlphaRegImpl:arg3": ("0", "null_mut()"),
           },
    setup=(
"""    for (k = 0; k < 4; k++) {
        JO[k].next = 0; JO[k].parent = 0; JO[k].child = 0;
        JO[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        JO[k].u.dobj = (rnd() & 1) ? &DO[rnd() % 3] : 0;
    }
    for (k = 1; k < 4; k++) {
        int p = (int) (rnd() % k);
        HSD_JObj* c = JO[p].child;
        if (!c) JO[p].child = &JO[k]; else { while (c->next) c = c->next; c->next = &JO[k]; }
        JO[k].parent = &JO[p];
    }
    for (k = 0; k < 2; k++) { JT[k].next = (rnd() & 1) ? &JT[1 - k] : 0; JT[k].child = 0; }
    JT[1].next = 0;
    JT[0].child = (rnd() & 1) ? &JT[1] : 0;
    for (k = 0; k < 3; k++) {
        DO[k].next = (k < 2 && (rnd() & 1)) ? &DO[k + 1] : 0;
        DO[k].pobj = (rnd() & 1) ? &PO[rnd() % 3] : 0;
        DO[k].mobj = (rnd() & 1) ? &MO[rnd() % 2] : 0;
        PO[k].next = (k < 2 && (rnd() & 1)) ? &PO[k + 1] : 0;
        PO[k].flags = (rnd() & 1) ? 0x1000 : (u16) fv();
        PO[k].u.shape_set = &SS[rnd() % 2];
        TO[k].next = (k < 2 && (rnd() & 1)) ? &TO[k + 1] : 0;
        TO[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        RO[k].next = (k < 2 && (rnd() & 1)) ? &RO[k + 1] : 0;
        LO[k].next = k < 2 ? &LO[k + 1] : 0;
        TD[k].next = (k < 2 && (rnd() & 1)) ? &TD[k + 1] : 0;
        TD[k].u.tevconf.clr_out_reg = fv() % 4; TD[k].u.tevconf.alpha_out_reg = fv() % 4;
        TE[k].cnst.next = (k < 2 && (rnd() & 1)) ? &TE[k + 1] : 0;
        TE[k].cnst.reg = (u8) (fv() % 10);
    }
    LO[2].flags &= ~3;
    for (k = 0; k < 2; k++) {
        SS[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        MO[k].aobj = (rnd() & 1) ? &AO[rnd() % 4] : 0;
        MO[k].tobj = (rnd() & 1) ? &TO[rnd() % 3] : 0;
        MO[k].tevdesc = &TED[k];
        MO[k].texp = (rnd() & 1) ? &TE[rnd() % 3] : 0;
        TED[k].desc.next = (rnd() & 1) ? &TD[rnd() % 3] : 0;
        TED[k].desc.u.tevconf.clr_out_reg = fv() % 4; TED[k].desc.u.tevconf.alpha_out_reg = fv() % 4;
    }
    for (k = 0; k < 4; k++) {
        AJ[k].child = (k < 3 && (rnd() & 1)) ? &AJ[k + 1] : 0;
        AJ[k].next = (k < 2 && (rnd() & 1)) ? &AJ[k + 2] : 0;
        AJ[k].aobjdesc = (rnd() & 1) ? &AD[rnd() % 2] : 0;
        AJP[k] = &AJ[k]; MAJP[k] = (HSD_MatAnimJoint*) MAJB[k]; SAJP[k] = (HSD_ShapeAnimJoint*) SAJB[k];
    }
    DMD[0].anims = (rnd() & 1) ? AJP : 0; DMD[0].matanims = (rnd() & 1) ? MAJP : 0; DMD[0].shapeanims = (rnd() & 1) ? SAJP : 0;
    for (k = 0; k < 4; k++) FL[k] = rf();
    FPP[0] = &FL[0]; FPP[1] = &FL[1];
    for (k = 0; k < 64; k++) MZ[k] = (unsigned char) (rnd() | 1);""",
"""    for k in 0..4 {
        JO[k].next = null_mut(); JO[k].parent = null_mut(); JO[k].child = null_mut();
        JO[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        JO[k].u.dobj = if rnd() & 1 != 0 { &raw mut DO[(rnd() % 3) as usize] } else { null_mut() };
    }
    for k in 1..4usize {
        let p = (rnd() % k as u32) as usize;
        let mut c = JO[p].child;
        if c.is_null() { JO[p].child = &raw mut JO[k]; } else { while !(*c).next.is_null() { c = (*c).next; } (*c).next = &raw mut JO[k]; }
        JO[k].parent = &raw mut JO[p];
    }
    for k in 0..2usize { JT[k].next = if rnd() & 1 != 0 { &raw mut JT[1 - k] } else { null_mut() }; JT[k].child = null_mut(); }
    JT[1].next = null_mut();
    JT[0].child = if rnd() & 1 != 0 { &raw mut JT[1] } else { null_mut() };
    for k in 0..3usize {
        DO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut DO[k + 1] } else { null_mut() };
        DO[k].pobj = if rnd() & 1 != 0 { &raw mut PO[(rnd() % 3) as usize] } else { null_mut() };
        DO[k].mobj = if rnd() & 1 != 0 { &raw mut MO[(rnd() % 2) as usize] } else { null_mut() };
        PO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut PO[k + 1] } else { null_mut() };
        PO[k].flags = if rnd() & 1 != 0 { 0x1000 } else { fv() as u16 };
        PO[k].u.shape_set = &raw mut SS[(rnd() % 2) as usize];
        TO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut TO[k + 1] } else { null_mut() };
        TO[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        RO[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut RO[k + 1] } else { null_mut() };
        LO[k].next = if k < 2 { &raw mut LO[k + 1] } else { null_mut() };
        TD[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut TD[k + 1] } else { null_mut() };
        TD[k].u.tevconf.clr_out_reg = fv() % 4; TD[k].u.tevconf.alpha_out_reg = fv() % 4;
        TE[k].cnst.next = if k < 2 && rnd() & 1 != 0 { &raw mut TE[k + 1] } else { null_mut() };
        TE[k].cnst.reg = (fv() % 10) as u8;
    }
    LO[2].flags &= !3;
    for k in 0..2usize {
        SS[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        MO[k].aobj = if rnd() & 1 != 0 { &raw mut AO[(rnd() % 4) as usize] } else { null_mut() };
        MO[k].tobj = if rnd() & 1 != 0 { &raw mut TO[(rnd() % 3) as usize] } else { null_mut() };
        MO[k].tevdesc = &raw mut TED[k];
        MO[k].texp = if rnd() & 1 != 0 { &raw mut TE[(rnd() % 3) as usize] } else { null_mut() };
        TED[k].desc.next = if rnd() & 1 != 0 { &raw mut TD[(rnd() % 3) as usize] } else { null_mut() };
        TED[k].desc.u.tevconf.clr_out_reg = fv() % 4; TED[k].desc.u.tevconf.alpha_out_reg = fv() % 4;
    }
    for k in 0..4usize {
        AJ[k].child = if k < 3 && rnd() & 1 != 0 { &raw mut AJ[k + 1] } else { null_mut() };
        AJ[k].next = if k < 2 && rnd() & 1 != 0 { &raw mut AJ[k + 2] } else { null_mut() };
        AJ[k].aobjdesc = if rnd() & 1 != 0 { &raw mut AD[(rnd() % 2) as usize] } else { null_mut() };
        AJP[k] = &raw mut AJ[k]; MAJP[k] = MAJB[k].as_mut_ptr() as *mut _; SAJP[k] = SAJB[k].as_mut_ptr() as *mut _;
    }
    DMD[0].anims = if rnd() & 1 != 0 { AJP.as_mut_ptr() } else { null_mut() };
    DMD[0].matanims = if rnd() & 1 != 0 { MAJP.as_mut_ptr() } else { null_mut() };
    DMD[0].shapeanims = if rnd() & 1 != 0 { SAJP.as_mut_ptr() } else { null_mut() };
    for k in 0..4 { FL[k] = rf(); }
    FPP[0] = &raw mut FL[0]; FPP[1] = &raw mut FL[1];
    for k in 0..64 { MZ[k] = (rnd() | 1) as u8; }"""),
    ptrs=(
"""    for (k = 0; k < 4; k++) { HM(PIB(JO[k].u.dobj)); JO[k].u.dobj = 0; }
    for (k = 0; k < 3; k++) { HM(PIB(PO[k].u.shape_set)); PO[k].u.shape_set = 0; HM(PIB(TE[k].cnst.next)); TE[k].cnst.next = 0; }""",
"""    for k in 0..4 { hm(pib(JO[k].u.dobj)); JO[k].u.dobj = null_mut(); }
    for k in 0..3 { hm(pib(PO[k].u.shape_set)); PO[k].u.shape_set = null_mut(); hm(pib(TE[k].cnst.next)); TE[k].cnst.next = null_mut(); }"""),
    hash=("    for (k = 0; k < 4; k++) HM(B(FL[k]));\n    for (k = 0; k < 64; k++) HM(MZ[k]);",
          "    for k in 0..4 { hm(FL[k].to_bits()); }\n    for k in 0..64 { hm(MZ[k] as u32); }"),
)
