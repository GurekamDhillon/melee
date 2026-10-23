NG = "NGO"


def jo(i="AI[0]"):
    return f"&OJO[({i}) % NGO]", f"addr_of_mut!(OJO[(({i.replace('AI[', 'a.i[')}) % NGO as u32) as usize])"


def fn_c(ty, k):
    return f"({ty}) OSCR[{k}]"


FN4 = "void (*)(Item_GObj*, Ground*)"
FN5 = "void (*)(Item_GObj*, Ground*, Vec3*, HSD_GObj*, f32)"
FN6 = "void (*)(Item_GObj*, Ground*, HSD_GObj*)"
RFN4 = "Option<unsafe extern \"C\" fn(*mut HSD_GObj, *mut Ground)>"
RFN5 = "Option<unsafe extern \"C\" fn(*mut HSD_GObj, *mut Ground, *mut Vec3, *mut HSD_GObj, f32)>"
RFN6 = "Option<unsafe extern \"C\" fn(*mut HSD_GObj, *mut Ground, *mut HSD_GObj)>"


def fnp(cty, rty, k, sel):
    # a fake callback (a pointer into OSCR) or NULL
    return (f"(AI[{sel}] % 2 ? ({cty}) OSCR[{k}] : 0)",
            f"(if a.i[{sel}] % 2 != 0 {{ fpp::<{rty}>({k}) }} else {{ None }})")


def pair(*items):
    return [i[0] for i in items], [i[1] for i in items]


def j(i):
    return jo(i)


ITEM_PRE_C = "OGO[0].user_data = OP_Item[AI[3] % 2];"
ITEM_PRE_R = "OGO[0].user_data = addr_of_mut!(OP_Item[(a.i[3] % 2) as usize]) as *mut c_void;"
CMD_C = "(CommandInfo*) OP_CommandInfo[0]"
CMD_R = "addr_of_mut!(OP_CommandInfo[0]) as *mut CommandInfo"

SPEC = dict(
    src="src/melee/gr/grmaterial.c",
    rust_mod="grmaterial",
    ext_mods=["grinl"],
    u=[],
    bf_skip=["Item", "HSD_MObj"],
    globals=[],
    provides=["hsdMObj", "tobj_shadows", "tobj_toon", "HSD_GObj_804D7814", "HSD_IDInsertToTable", "HSD_SetMaterialColor", "lb_80014498"],
    extra_pools=["HSD_DObj", "HSD_MObj", "HSD_Material", "HSD_TObj", "HSD_PEDesc", "HSD_TExpTevDesc", "HSD_TExp",
                 "HSD_ImageDesc", "CommandInfo", "Item"],
    statics=[
        dict(c="grMaterial_803E0A20", rs="t::grMaterial_803E0A20", ty="grMaterial_MObjInfo", fill=True),
        dict(c="grMaterial_804D456C", rs="t::grMaterial_804D456C", kind="p", n=1),
        dict(c="hsdMObj", rs="t::ext::hsdMObj", ty="HSD_MObjInfo", fill=True),
        dict(c="tobj_shadows", rs="t::ext::tobj_shadows", kind="p"),
        dict(c="tobj_toon", rs="t::ext::tobj_toon", kind="p"),
        dict(c="HSD_GObj_804D7814", rs="t::ext::HSD_GObj_804D7814", kind="p"),
    ],
    data_c="""HSD_MObjInfo hsdMObj; HSD_TObj* tobj_shadows; HSD_TObj* tobj_toon; HSD_GObj* HSD_GObj_804D7814;
void HSD_IDInsertToTable(HSD_IDTable* a0, HSD_IDKey a1, void* a2)
{
    lg_s("HSD_IDInsertToTable("); lg_ptr((const void*) a0); lg_int((u32) (a1 == (HSD_IDKey) a2)); lg_ptr((const void*) a2); lg_s(")"); lg_c(';');
}
void HSD_SetMaterialColor(GXColor a0, GXColor a1, GXColor a2, f32 a3)
{
    lg_s("HSD_SetMaterialColor("); lg_int(*(u32*) &a0); lg_int(*(u32*) &a1); lg_int(*(u32*) &a2); lg_f(a3); lg_s(")"); lg_c(';');
}
/* by pointer only: the overlay's bitfield byte is laid out differently in the Rust record (the world hash covers its value) */
void lb_80014498(ColorOverlay* a0) { lg_s("lb_80014498("); lg_ptr((const void*) a0); lg_s(")"); lg_c(';'); }
#define POOL(T, i) ((T*) OP_##T[i])""",
    use_rs="""unsafe fn fpp<F: Copy>(i: usize) -> F { let p = addr_of_mut!(OSCR[i]) as usize; core::mem::transmute_copy::<usize, F>(&p) }""",
    links_c="""    {
        int k;
        grMaterial_804D456C[0] = grMaterial_801C9470;
        hsdMObj.parent.release = (void*) OSCR[0]; hsdMObj.parent.amnesia = (void*) OSCR[1];
        hsdMObj.load = (void*) OSCR[2]; hsdMObj.make_texp = (void*) OSCR[3];
        HSD_GObj_804D7814 = &OGO[rnd() % NGO];
        POOL(HSD_DObj, 0)->next = POOL(HSD_DObj, 1); POOL(HSD_DObj, 1)->next = 0;
        POOL(HSD_DObj, 0)->mobj = rnd() % 4 ? POOL(HSD_MObj, 0) : 0;
        POOL(HSD_DObj, 1)->mobj = rnd() % 4 ? POOL(HSD_MObj, 1) : 0;
        for (k = 0; k < 2; k++) {
            POOL(HSD_MObj, k)->mat = POOL(HSD_Material, k);
            POOL(HSD_MObj, k)->tevdesc = POOL(HSD_TExpTevDesc, k);
            POOL(HSD_MObj, k)->texp = POOL(HSD_TExp, k);
            POOL(HSD_MObj, k)->pe = POOL(HSD_PEDesc, k);
            POOL(HSD_MObj, k)->tobj = rnd() % 3 ? POOL(HSD_TObj, 0) : 0;
            POOL(HSD_MObj, k)->rendermode = rnd();
        }
        POOL(HSD_TObj, 0)->next = 0; POOL(HSD_TObj, 1)->next = 0;
        tobj_shadows = rnd() % 2 ? POOL(HSD_TObj, 1) : 0;
        tobj_toon = rnd() % 2 ? POOL(HSD_TObj, 1) : 0;
        POOL(HSD_TObj, 1)->imagedesc = rnd() % 2 ? POOL(HSD_ImageDesc, 0) : 0;
        POOL(CommandInfo, 0)->ptr[0] = (u32*) OSCR[1];
        *(u16*) OSCR[1] = (u16) rnd();
    }""",
    links_rs="""    {
        t::grMaterial_804D456C[0] = Some(t::grMaterial_801C9470);
        t::ext::hsdMObj.parent.release = fpp(0); t::ext::hsdMObj.parent.amnesia = fpp(1);
        t::ext::hsdMObj.load = fpp(2); t::ext::hsdMObj.make_texp = fpp(3);
        t::ext::HSD_GObj_804D7814 = addr_of_mut!(OGO[(rnd() % NGO as u32) as usize]);
        let d0 = addr_of_mut!(OP_HSD_DObj[0]) as *mut HSD_DObj; let d1 = addr_of_mut!(OP_HSD_DObj[1]) as *mut HSD_DObj;
        (*d0).next = d1; (*d1).next = core::ptr::null_mut();
        let m0 = addr_of_mut!(OP_HSD_MObj[0]) as *mut HSD_MObj; let m1 = addr_of_mut!(OP_HSD_MObj[1]) as *mut HSD_MObj;
        (*d0).mobj = if rnd() % 4 != 0 { m0 } else { core::ptr::null_mut() };
        (*d1).mobj = if rnd() % 4 != 0 { m1 } else { core::ptr::null_mut() };
        for k in 0..2usize {
            let m = addr_of_mut!(OP_HSD_MObj[k]) as *mut HSD_MObj;
            (*m).mat = addr_of_mut!(OP_HSD_Material[k]) as *mut HSD_Material;
            (*m).tevdesc = addr_of_mut!(OP_HSD_TExpTevDesc[k]) as *mut HSD_TExpTevDesc;
            (*m).texp = addr_of_mut!(OP_HSD_TExp[k]) as *mut HSD_TExp;
            (*m).pe = addr_of_mut!(OP_HSD_PEDesc[k]) as *mut HSD_PEDesc;
            (*m).tobj = if rnd() % 3 != 0 { addr_of_mut!(OP_HSD_TObj[0]) as *mut HSD_TObj } else { core::ptr::null_mut() };
            (*m).rendermode = rnd();
        }
        let o0 = addr_of_mut!(OP_HSD_TObj[0]) as *mut HSD_TObj; let o1 = addr_of_mut!(OP_HSD_TObj[1]) as *mut HSD_TObj;
        (*o0).next = core::ptr::null_mut(); (*o1).next = core::ptr::null_mut();
        t::ext::tobj_shadows = if rnd() % 2 != 0 { o1 } else { core::ptr::null_mut() };
        t::ext::tobj_toon = if rnd() % 2 != 0 { o1 } else { core::ptr::null_mut() };
        (*o1).imagedesc = if rnd() % 2 != 0 { addr_of_mut!(OP_HSD_ImageDesc[0]) as *mut HSD_ImageDesc } else { core::ptr::null_mut() };
        (*(addr_of_mut!(OP_CommandInfo[0]) as *mut CommandInfo)).anon_8.ptr[0] = (addr_of_mut!(OSCR[1]) as *mut u8) as *mut u32;
        *((addr_of_mut!(OSCR[1]) as *mut u8) as *mut u16) = rnd() as u16;
    }""",
    hooks_rs="""    melee_world::tu::grmaterial::ext::HSD_IDINSERTTOTABLE = Some(|a0, a1, a2| { lg_s("HSD_IDInsertToTable("); lg_ptr(a0 as usize); lg_int((a1 == a2 as usize as u32) as u32); lg_ptr(a2 as usize); lg_s(")"); lg_c(';'); });
    melee_world::tu::grmaterial::ext::LB_80014498 = Some(|a0| { lg_s("lb_80014498("); lg_ptr(a0 as usize); lg_s(")"); lg_c(';'); });
    melee_world::tu::grmaterial::ext::HSD_SETMATERIALCOLOR = Some(|a0, a1, a2, a3| { lg_s("HSD_SetMaterialColor("); for c in [a0, a1, a2] { lg_int(u32::from_le_bytes([c.r, c.g, c.b, c.a])); } lg_f(a3); lg_s(")"); lg_c(';'); });""",
    fnvals={
        "grMaterial_801C9470": "t::grMaterial_801C9470", "fn_801C9664": "t::fn_801C9664",
        "grMaterial_801C8E74": "t::grMaterial_801C8E74", "fn_801C8EF8": "t::fn_801C8EF8",
    },
    rets={
        "lbGetFreeColorRegister": ("(int) (rnd() % 8)", "((rnd() % 8) as i32)"),
        "lbGetFreeAlphaRegister": ("(int) (rnd() % 8)", "((rnd() % 8) as i32)"),
    },
    funcs=[
        dict(name="grMaterial_801C87D0", c_args=[jo()[0], "AI[1]"], rs_args=[jo()[1], "a.i[1]"]),
        dict(name="grMaterial_801C8858", c_args=[jo()[0], "AI[1]"], rs_args=[jo()[1], "a.i[1]"]),
        dict(name="grMaterial_801C897C", c_args=[jo()[0], "AI[1]"], rs_args=[jo()[1], "a.i[1]"]),
        dict(name="grMaterial_801C8A04", c_args=[jo()[0], "AI[1]"], rs_args=[jo()[1], "a.i[1]"]),
        dict(name="grMaterial_801C8B28", c_args=["&OJO[NGO + AI[0] % NGO]"], rs_args=["addr_of_mut!(OJO[NGO + (a.i[0] % NGO as u32) as usize])"]),
        dict(name="grMaterial_801C8B68", c_args=[jo()[0], "AI[1] % 2"], rs_args=[jo()[1], "(a.i[1] % 2) as i32"]),
        dict(name="grMaterial_801C8CDC"),
        dict(name="grMaterial_801C8CFC",
             c_args=["AI[0]", "AI[1]", "&OGP[0]", "&OJO[1]", fnp(FN4, RFN4, 1, 2)[0], fnp(FN5, RFN5, 2, 3)[0], fnp(FN6, RFN6, 3, 0)[0]],
             rs_args=["a.i[0] as _", "a.i[1] as _", "addr_of_mut!(OGP[0])", "addr_of_mut!(OJO[1])", fnp(FN4, RFN4, 1, 2)[1], fnp(FN5, RFN5, 2, 3)[1], fnp(FN6, RFN6, 3, 0)[1]]),
        dict(name="grMaterial_801C8D44",
             c_args=["AI[0]", "AI[1]", "&OGP[1]", "&AV3[0]", "AI[3]", fnp(FN4, RFN4, 1, 2)[0], fnp(FN5, RFN5, 2, 3)[0], fnp(FN6, RFN6, 3, 0)[0]],
             rs_args=["a.i[0] as _", "a.i[1] as _", "addr_of_mut!(OGP[1])", "&mut a.v3[0]", "a.i[3] as _", fnp(FN4, RFN4, 1, 2)[1], fnp(FN5, RFN5, 2, 3)[1], fnp(FN6, RFN6, 3, 0)[1]]),
        dict(name="grMaterial_801C8D98"),
        dict(name="grMaterial_801C8DE0", pre_c=ITEM_PRE_C, pre_rs=ITEM_PRE_R,
             c_args=["&OGO[0]", "AF[0]", "AF[1]", "AV3[0].x", "AV3[0].y", "AV3[0].z", "AV3[1].x", "AV3[1].y"],
             rs_args=["g", "a.f[0]", "a.f[1]", "a.v3[0].x", "a.v3[0].y", "a.v3[0].z", "a.v3[1].x", "a.v3[1].y"]),
        dict(name="grMaterial_801C8E08"),
        dict(name="grMaterial_801C8E28"),
        dict(name="grMaterial_801C8E48",
             pre_c=ITEM_PRE_C + " ((Item*) OP_Item[0])->x524_cmd.u = AI[1] % 2 ? (CmdUnion*) OSCR[1] : 0; ((Item*) OP_Item[1])->x524_cmd.u = AI[2] % 2 ? (CmdUnion*) OSCR[2] : 0;",
             pre_rs=ITEM_PRE_R + " (*(addr_of_mut!(OP_Item[0]) as *mut Item)).x524_cmd.anon_8.u = if a.i[1] % 2 != 0 { addr_of_mut!(OSCR[1]) as *mut CmdUnion } else { core::ptr::null_mut() }; (*(addr_of_mut!(OP_Item[1]) as *mut Item)).x524_cmd.anon_8.u = if a.i[2] % 2 != 0 { addr_of_mut!(OSCR[2]) as *mut CmdUnion } else { core::ptr::null_mut() };"),
        dict(name="grMaterial_801C8E68", pre_c=ITEM_PRE_C, pre_rs=ITEM_PRE_R),
        dict(name="grMaterial_801C8E74"),
        dict(name="fn_801C8EF8", c_args=["POOL(HSD_MObj, AI[0] % 2)", "rnd()"], rs_args=["addr_of_mut!(OP_HSD_MObj[(a.i[0] % 2) as usize]) as *mut HSD_MObj", "rnd()"],
             rs_path="t::fn_801C8EF8"),
        dict(name="grMaterial_801C92C0", c_args=[jo()[0]], rs_args=[jo()[1]]),
        dict(name="grMaterial_801C94D8", c_args=[jo()[0]], rs_args=[jo()[1] + " as *mut c_void"]),
        dict(name="grMaterial_801C9470", c_args=["&OGO[0]", CMD_C], rs_args=["g", CMD_R]),
        dict(name="grMaterial_801C9490", c_args=["&OGO[0]", CMD_C], rs_args=["g", CMD_R]),
        dict(name="grMaterial_801C95C4"),
        dict(name="grMaterial_801C9604", c_args=["&OGO[0]", "(void*) OSCR[2]", "AI[1] % 2"], rs_args=["g", "addr_of_mut!(OSCR[2]) as *mut c_void", "a.i[1] % 2 != 0"]),
        dict(name="fn_801C9664", c_args=["&OGO[0]", CMD_C, "21"], rs_args=["g", CMD_R, "21"]),
        dict(name="grMaterial_801C9698"),
    ],
)
