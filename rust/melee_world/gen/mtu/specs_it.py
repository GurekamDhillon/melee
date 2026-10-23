"""Oracle specs of the `it/kinds` TUs of melee_items (shard 697-704); keys as in specs_if.py
(pool-driver of if_drv.py) plus walk2 / uv / uv_only (gen_if.py: union variants by value)."""
SPECS = {}

# the gobj -> item / jobj web every item function walks
ITEM_RULES = {
    "HSD_GObj.hsd_obj": ("ptr", "HSD_JObj", False),
    "HSD_GObj.user_data": ("ptr", "Item", False),
    "HSD_JObj.child": ("ptr", "HSD_JObj", False),
    "HSD_JObj.next": ("ptr", "HSD_JObj", True),
    "Item.xC4_article_data": ("ptr", "Article", False),
    "Item.xCC_item_attr": ("ptr", "ItemAttr", False),
}
# C-side bit-view of every UnkFlagStruct / flag32 union the item code reads as bitfields
FLAG_UV = {f"Item_x{n}_flag": "anon0" for n in ("DCE", "DCF", "DD0", "DD1", "DD2", "DD3")}
FLAG_UV.update({f"SpawnItem_x{n}_flag": "anon0" for n in (44, 45, 46, 47)})
FLAG_UV.update({"Item_anon_477_xDAA_flag": "anon0", "Item_xD90": "anon0"})
JOBJ_SETUP_C = "    for (n = 0; n < 6; n++) P_HSD_JObj[n].flags &= ~(1u << 17);"
JOBJ_SETUP_RS = "    for n in 0..6usize { g::P_HSD_JObj[n].flags &= !(1u32 << 17); }"

SPECS["itheart"] = dict(
    crate="melee_items", src="src/melee/it/kinds/itheart.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "HeartContainerAttr": 1, "Vec3": 2},
    types=["SpawnItem"],
    rules={**ITEM_RULES,
           "Article.x4_specialAttributes": ("ptr", "HeartContainerAttr", False),
           "itHeart_ItemVars.xDDC": ("range", 0, 3)},
    uv={**FLAG_UV, "Item_xDD4_itemVar": "heart", "HeartContainerAttr_x4": "flags"},
    uv_only={"Item_xDD4_itemVar": ["heart"]},
    statics=[dict(c="gm_80473A18", ext=True, ty="UnkAllstarData")],
    skip_data=["result"],
    ret={"Item_80268B18": ("pick_HSD_GObj(1)", "pick_HSD_GObj(1)")},
    effects={"Item_80268B18": ("hash_SpawnItem(a0);", "hash_SpawnItem(&*a0);")},
    setup_c=JOBJ_SETUP_C, setup_rs=JOBJ_SETUP_RS,
    ops=[dict(name="it_80283AE4", args=["pn:HSD_GObj", "p:Vec3", "v"])] + [
        n for n in ("it_80283BD4", "itHeart_Logic8_Spawned", "itHeart_Logic8_Destroyed", "it_80283C7C",
                    "itHeart_UnkMotion0_Anim", "itHeart_UnkMotion0_Phys", "itHeart_UnkMotion0_Coll", "it_80283DD4",
                    "itHeart_UnkMotion3_Anim", "itHeart_UnkMotion3_Phys", "itHeart_UnkMotion3_Coll",
                    "itHeart_Logic8_PickedUp", "itHeart_UnkMotion2_Anim", "itHeart_UnkMotion2_Phys",
                    "itHeart_Logic8_Dropped", "itHeart_Logic8_EnteredAir", "itHeart_UnkMotion4_Anim",
                    "itHeart_UnkMotion4_Phys", "itHeart_UnkMotion4_Coll", "itHeart_Logic8_EvtUnk")],
)

# lb_8000B1CC(jobj, in, out): reads *in (when given), writes *out
LB_B1CC = ("{ if (a1) { T(B(a1->x)); T(B(a1->y)); T(B(a1->z)); } a2->x = rg(); a2->y = rg(); a2->z = rg(); }",
           "{ if !a1.is_null() { t((*a1).x.to_bits()); t((*a1).y.to_bits()); t((*a1).z.to_bits()); } (*a2).x = rg(); (*a2).y = rg(); (*a2).z = rg(); }")
BONES_C = "    for (n = 0; n < 8; n++) P_DynamicBoneTable[0].bones[n] = &P_HSD_JObj[rnd() % 6u];"
BONES_RS = "    for n in 0..8usize { g::P_DynamicBoneTable[0].bones[n] = &raw mut g::P_HSD_JObj[(rnd() % 6) as usize]; }"
SPAWN_HASH = {"Item_80268B18": ("hash_SpawnItem(a0);", "hash_SpawnItem(&*a0);")}

SPECS["itgreatfoxlaser"] = dict(
    crate="melee_items", src="src/melee/it/kinds/itgreatfoxlaser.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "itGreatFoxLaser_Attrs": 1,
           "DynamicBoneTable": 1, "Vec3": 2},
    types=["SpawnItem"],
    rules={**ITEM_RULES,
           "Article.x4_specialAttributes": ("ptr", "itGreatFoxLaser_Attrs", False),
           "Item.xBBC_dynamicBoneTable": ("ptr", "DynamicBoneTable", False),
           "itGreatFoxLaser_ItemVars.x20": ("ptr", "HSD_GObj", False)},
    uv={**FLAG_UV, "Item_xDD4_itemVar": "greatfoxlaser"},
    uv_only={"Item_xDD4_itemVar": ["greatfoxlaser", "likelike"]},
    skip_data=["result"],
    ret={"Item_80268B18": ("pick_HSD_GObj(1)", "pick_HSD_GObj(1)")},
    effects={**SPAWN_HASH, "lb_8000B1CC": LB_B1CC,
             "ftLib_80086644": ("{ a1->x = rg(); a1->y = rg(); a1->z = rg(); }", "{ (*a1).x = rg(); (*a1).y = rg(); (*a1).z = rg(); }")},
    setup_c=JOBJ_SETUP_C + "\n" + BONES_C, setup_rs=JOBJ_SETUP_RS + "\n" + BONES_RS,
    ops=[dict(name="it_802EAF28", uv={"Item_xDD4_itemVar": "likelike"}),
         dict(name="it_802EAF34", args=["p:HSD_GObj", "p:Vec3", "r:0:1"]),
         "it_802EB1EC", "it_802EB268", "itGreatfoxlaser_UnkMotion1_Anim", "itGreatfoxlaser_UnkMotion1_Phys",
         "itGreatFoxLaser_Logic27_DmgDealt", "itGreatFoxLaser_Logic27_Clanked", "itGreatFoxLaser_Logic27_Absorbed",
         "it_2725_Logic27_Reflected", "it_802EB5A8"],
)

ICD_STATIC = dict(c="it_804D6D28", ext=True, kind="ptr", fill="p:ItemCommonData")
VEC_OUT = ("{ a1->x = rg(); a1->y = rg(); a1->z = rg(); }", "{ (*a1).x = rg(); (*a1).y = rg(); (*a1).z = rg(); }")

SPECS["ithammerhead"] = dict(
    crate="melee_items", src="src/melee/it/kinds/ithammerhead.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "itHammerheadAttributes": 1,
           "ItemCommonData": 1, "Vec3": 2},
    types=["SpawnItem"],
    rules={**ITEM_RULES,
           "Article.x4_specialAttributes": ("ptr", "itHammerheadAttributes", False)},
    uv=FLAG_UV, uv_only={"Item_xDD4_itemVar": []},
    statics=[ICD_STATIC],
    skip_data=["result"],
    ret={"Item_80268B18": ("pick_HSD_GObj(1)", "pick_HSD_GObj(1)")},
    effects={**SPAWN_HASH, "it_8026BB68": VEC_OUT},
    setup_c=JOBJ_SETUP_C, setup_rs=JOBJ_SETUP_RS,
    ops=[dict(name="it_80299C48", args=["pn:HSD_GObj", "p:Vec3", "p:Vec3", "f"])] + [
        n for n in ("itHammerHead_Logic40_Spawned", "it_80299D7C", "itHammerHead_Logic40_PickedUp",
                    "itHammerhead_UnkMotion1_Anim", "itHammerhead_UnkMotion1_Phys", "itHammerHead_Logic40_Dropped",
                    "itHammerHead_Logic40_Thrown", "itHammerhead_UnkMotion2_Anim", "itHammerhead_UnkMotion2_Phys",
                    "itHammerhead_UnkMotion2_Coll", "it_80299F94", "it_80299FB4", "itHammerhead_UnkMotion3_Anim",
                    "itHammerhead_UnkMotion3_Phys", "itHammerhead_UnkMotion3_Coll", "itHammerHead_Logic40_DmgDealt",
                    "itHammerHead_Logic40_Clanked", "itHammerHead_Logic40_HitShield", "itHammerHead_Logic40_Reflected",
                    "itHammerHead_Logic40_DmgReceived", "itHammerHead_Logic40_EvtUnk")],
)

RAND_RET = {
    "HSD_Randi": ("(a0 > 0 ? (int) (rnd() % (u32) a0) : 0)", "if a0 > 0 { (rnd() % a0 as u32) as i32 } else { 0 }"),
    "HSD_Randf": ("(float) (rnd() % 1000) / 1000.0f", "(rnd() % 1000) as f32 / 1000.0"),
}
# in/out Vec3 argument of lbVector_Normalize: traced, then replaced
VEC_INOUT = ("{ T(B(a0->x)); T(B(a0->y)); T(B(a0->z)); a0->x = rg(); a0->y = rg(); a0->z = rg(); }",
             "{ t((*a0).x.to_bits()); t((*a0).y.to_bits()); t((*a0).z.to_bits()); (*a0).x = rg(); (*a0).y = rg(); (*a0).z = rg(); }")

SPECS["ithammer"] = dict(
    crate="melee_items", src="src/melee/it/kinds/ithammer.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "itHammerData": 1, "Vec3": 2},
    rules={**ITEM_RULES,
           "Article.x4_specialAttributes": ("ptr", "itHammerData", False)},
    uv=FLAG_UV, uv_only={"Item_xDD4_itemVar": []},
    skip_data=["result"],
    ret={**RAND_RET, "it_80272CC0": ("pick_HSD_JObj(0)", "pick_HSD_JObj(0)")},
    effects={"lb_8000B1CC": LB_B1CC, "lbVector_Normalize": VEC_INOUT},
    deref={"it_80299C48": [1, 2]},
    setup_c=JOBJ_SETUP_C, setup_rs=JOBJ_SETUP_RS,
    ops=["it_80293D94", "it_80293DCC", "it_80293E34", "itHammer_Logic28_Spawned", "it_80293F84",
         "itHammer_UnkMotion0_Anim", "itHammer_UnkMotion0_Phys", "itHammer_UnkMotion0_Coll", "it_8029402C",
         "itHammer_UnkMotion4_Anim", "itHammer_UnkMotion1_Phys", "itHammer_UnkMotion1_Coll",
         "itHammer_Logic28_PickedUp", "itHammer_UnkMotion3_Anim", "it_802941A4", "itHammer_Logic28_Dropped",
         "itHammer_UnkMotion4_Phys", "itHammer_UnkMotion4_Coll", "itHammer_Logic28_EnteredAir",
         "itHammer_UnkMotion5_Anim", "itHammer_UnkMotion5_Phys", "itHammer_UnkMotion5_Coll",
         "itHammer_Logic28_EvtUnk"],
)

SPECS["itharisen"] = dict(
    crate="melee_items", src="src/melee/it/kinds/itharisen.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "itHarisen_DatAttrs": 1,
           "ItemCommonData": 1},
    rules={**ITEM_RULES,
           "Article.x4_specialAttributes": ("ptr", "itHarisen_DatAttrs", False)},
    uv=FLAG_UV, uv_only={"Item_xDD4_itemVar": []},
    statics=[ICD_STATIC],
    skip_data=["result"],
    setup_c=JOBJ_SETUP_C, setup_rs=JOBJ_SETUP_RS,
    ops=["it_802927E8", "it_8029282C", "itHarisen_Logic24_Spawned", "it_8029287C", "itHarisen_UnkMotion0_Anim",
         "itHarisen_UnkMotion0_Phys", "itHarisen_UnkMotion0_Coll", "it_8029290C", "itHarisen_UnkMotion8_Anim",
         "itHarisen_UnkMotion1_Phys", "itHarisen_UnkMotion1_Coll", "it_80292998", "it_802929C8", "it_802929F8",
         "it_80292A28", "itHarisen_Logic24_PickedUp", "itHarisen_UnkMotion6_Anim", "itHarisen_UnkMotion6_Phys",
         "itHarisen_Logic24_Dropped", "itHarisen_UnkMotion8_Coll", "itHarisen_Logic24_Thrown",
         "itHarisen_UnkMotion8_Phys", "itHarisen_UnkMotion7_Coll", "itHarisen_Logic24_DmgDealt",
         "itHarisen_Logic24_EnteredAir", "itHarisen_UnkMotion9_Anim", "itHarisen_UnkMotion9_Phys",
         "itHarisen_UnkMotion9_Coll", "itHarisen_Logic24_Clanked", "itHarisen_Logic24_Reflected",
         "itHarisen_Logic24_HitShield", "itHarisen_Logic24_ShieldBounced", "itHarisen_Logic24_EvtUnk"],
)

GSHELL_FNS = ["it_8028B8D8", "it_8028B988", "it_8028BA2C", "it_8028BAD8", "it_8028BC2C", "itGShell_Logic14_Spawned",
              "it_8028BE54", "itGshell_UnkMotion0_Anim", "itGshell_UnkMotion0_Phys", "itGshell_UnkMotion0_Coll",
              "it_8028C018", "itGshell_UnkMotion1_Anim", "itGshell_UnkMotion1_Phys", "itGshell_UnkMotion1_Coll",
              "itGShell_Logic14_PickedUp", "itGshell_UnkMotion2_Anim", "itGshell_UnkMotion2_Phys",
              "itGShell_Logic14_Thrown", "itGshell_UnkMotion3_Anim", "itGshell_UnkMotion3_Phys",
              "itGshell_UnkMotion3_Coll", "itGShell_Logic14_Dropped", "itGshell_UnkMotion4_Anim",
              "itGshell_UnkMotion4_Phys", "itGshell_UnkMotion4_Coll", "it_8028C3A8", "itGshell_UnkMotion6_Anim",
              "itGshell_UnkMotion6_Phys", "itGshell_UnkMotion6_Coll", "it_8028C898", "itGshell_UnkMotion8_Anim",
              "itGshell_UnkMotion8_Phys", "itGshell_UnkMotion8_Coll", "itGShell_Logic14_EnteredAir",
              "itGshell_UnkMotion9_Anim", "itGshell_UnkMotion9_Phys", "itGshell_UnkMotion9_Coll",
              "itGShell_Logic14_DmgDealt", "itGShell_Logic14_DmgReceived", "itGShell_Logic14_Reflected",
              "shellHit", "itGShell_Logic14_Clanked", "itGShell_Logic14_HitShield", "itGShell_Logic14_ShieldBounced",
              "it_8028CF68", "itGShell_Logic14_EvtUnk"]

SPECS["itgshell"] = dict(
    crate="melee_items", src="src/melee/it/kinds/itgshell.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "itGShell_Attrs": 1,
           "ItemCommonData": 1},
    rules={**ITEM_RULES,
           "Article.x4_specialAttributes": ("ptr", "itGShell_Attrs", False),
           "Item.msid": ("range", 0, 9)},
    uv={**FLAG_UV, "Item_xDD4_itemVar": "gshell"}, uv_only={"Item_xDD4_itemVar": ["gshell"]},
    statics=[ICD_STATIC],
    skip_data=["result"],
    ret=RAND_RET,
    null_ret_ok=["it_8027236C", "it_802723FC"],
    custom_stubs={"efAsync_Spawn": dict(
        c="void efAsync_Spawn(HSD_GObj* a0, void* a1, u32 a2, u32 a3, HSD_JObj* a4, ...) { __builtin_va_list ap; "
          "Vec3* v; __builtin_va_start(ap, a4); T(@ID@); T(PI(a0)); T(PI(a1)); T(a2); T(a3); T(PI(a4)); "
          "v = __builtin_va_arg(ap, Vec3*); T(B(v->x)); T(B(v->y)); T(B(v->z)); __builtin_va_end(ap); }",
        ext="fn efAsync_Spawn(a0: *mut HSD_GObj, a1: *mut c_void, a2: u32, a3: u32, a4: *mut HSD_JObj, a5: *mut Vec);",
        hook="|a0, a1, a2, a3, a4, a5| { t(@ID@); t(pi(a0)); t(pi(a1)); t(a2); t(a3); t(pi(a4)); "
             "unsafe { t((*a5).x.to_bits()); t((*a5).y.to_bits()); t((*a5).z.to_bits()); } }")},
    setup_c=JOBJ_SETUP_C, setup_rs=JOBJ_SETUP_RS,
    ops=GSHELL_FNS, ncases={"it_8028BAD8": 3000, "it_8028BC2C": 3000, "itGShell_Logic14_DmgReceived": 3000},
)

FG_SETUP_C = ("    for (n = 0; n < 3; n++) { FG[n].next = (n < 2) ? &FG[n + 1] : 0; FG[n].user_data = &FT[n]; "
              "FT[n].x221F_b3 = rnd() & 1; FT[n].team = rnd() % 3u; }\n"
              "    PLH[HSD_GOBJ_PLINK_FIGHTER] = &FG[0]; HSD_GObjPLinkHead = PLH;")
FG_SETUP_RS = ("    for n in 0..3usize { FG[n].next = if n < 2 { &raw mut FG[n + 1] } else { null_mut() }; "
               "FG[n].user_data = &raw mut FT[n] as *mut c_void; FT[n].set_x221F_b3((rnd() & 1) as _); FT[n].team = (rnd() % 3) as _; }\n"
               "    PLH[HSD_GOBJ_PLINK_FIGHTER as usize] = &raw mut FG[0]; m::ext::HSD_GObjPLinkHead = &raw mut PLH as *mut *mut HSD_GObj;")

SPECS["ithassam"] = dict(
    crate="melee_items", src="src/melee/it/kinds/ithassam.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "itHassam_ItemVars": 1,
           "DynamicBoneTable": 1, "Vec3": 2},
    rules={**ITEM_RULES,
           "Article.x4_specialAttributes": ("ptr", "itHassam_ItemVars", False),
           "Item.xBBC_dynamicBoneTable": ("ptr", "DynamicBoneTable", False),
           "Item.owner": ("expr", "&FG[rnd() % 3u]", "&raw mut super::FG[(rnd() % 3) as usize]"),
           "itHassam_ItemVars.x20": ("range", 1, 9)},
    uv={**FLAG_UV, "Item_xDD4_itemVar": "hassam"}, uv_only={"Item_xDD4_itemVar": ["hassam"]},
    statics=[dict(c="FG", rs="super::FG", n=3, region=True), dict(c="FT", rs="super::FT", n=3, region=True),
             dict(c="PLH", rs="super::PLH", n=32, region=True)],
    skip_data=["result", "item"],
    c_decl="static HSD_GObj FG[3];\nstatic Fighter FT[3];\nstatic HSD_GObj* PLH[32];\n",
    rs_decl="pub static mut FG: [HSD_GObj; 3] = unsafe { core::mem::zeroed() };\n"
            "pub static mut FT: [Fighter; 3] = unsafe { core::mem::zeroed() };\n"
            "pub static mut PLH: [*mut HSD_GObj; 32] = [null_mut(); 32];\n",
    ret={"ftLib_80086FD4": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "gm_8016B168": ("(int) (fv() & 1)", "(fv() & 1) as i32"),
         "ftLib_8008627C": ("((fv() & 1) ? (HSD_GObj*) 0 : &FG[rnd() % 3u])",
                            "if fv() & 1 != 0 { null_mut() } else { &raw mut super::FG[(rnd() % 3) as usize] }")},
    effects={"lbVector_Diff": ("{ T(B(a0->x)); T(B(a0->y)); T(B(a0->z)); T(B(a1->x)); T(B(a1->y)); T(B(a1->z)); "
                               "a2->x = rg(); a2->y = rg(); a2->z = rg(); }",
                               "{ t((*a0).x.to_bits()); t((*a0).y.to_bits()); t((*a0).z.to_bits()); t((*a1).x.to_bits()); "
                               "t((*a1).y.to_bits()); t((*a1).z.to_bits()); (*a2).x = rg(); (*a2).y = rg(); (*a2).z = rg(); }"),
             "lbVector_Rotate": ("{ T(B(a0->x)); T(B(a0->y)); T(B(a0->z)); a0->x = rg(); a0->y = rg(); a0->z = rg(); }",
                                 "{ t((*a0).x.to_bits()); t((*a0).y.to_bits()); t((*a0).z.to_bits()); (*a0).x = rg(); (*a0).y = rg(); (*a0).z = rg(); }"),
             "lbVector_Normalize": VEC_INOUT,
             "ftLib_800866DC": ("{ a1->x = rg(); a1->y = rg(); a1->z = rg(); }",
                                "{ (*a1).x = rg(); (*a1).y = rg(); (*a1).z = rg(); }"),
             "Camera_GetTransformPosition": ("{ a0->x = rg(); a0->y = rg(); a0->z = rg(); }",
                                             "{ (*a0).x = rg(); (*a0).y = rg(); (*a0).z = rg(); }")},
    defaults={"atan2f": "melee_runtime::lbtrigf::atan2f"},
    setup_c=JOBJ_SETUP_C + "\n" + BONES_C + "\n" + FG_SETUP_C, setup_rs=JOBJ_SETUP_RS + "\n" + BONES_RS + "\n" + FG_SETUP_RS,
    ops=["itHassam_802CDBE0", "itHassam_802CDC5C", "itHassam_802CDC60", "itHassam_802CDC80", "itHassam_802CDCB4",
         "itHassam_UnkMotion0_Anim", "itHassam_UnkMotion0_Phys", "itHassam_UnkMotion0_Coll",
         dict(name="itHassam_802CDE1C", args=["p:Vec3", ("raw", "((rnd() & 3) == 0 ? (HSD_GObj*) 0 : &FG[rnd() % 3u])",
                                                 "if rnd() & 3 == 0 { null_mut() } else { &raw mut FG[(rnd() % 3) as usize] }")]),
         "itHassam_802CDF28", "itHassam_802CE008", "itHassam_UnkMotion1_Anim", "itHassam_UnkMotion1_Phys",
         "itHassam_UnkMotion1_Coll", "itHassam_802CE400_sub", "itHassam_802CE400", "itHassam_UnkMotion2_Anim",
         "itHassam_UnkMotion2_Phys", "itHassam_UnkMotion2_Coll", "it_802CE640", "itHassam_UnkMotion3_Anim",
         "itHassam_UnkMotion3_Phys", "itHassam_UnkMotion3_Coll"],
)

SPECS["itheiho"] = dict(
    crate="melee_items", src="src/melee/it/kinds/itheiho.c", walk2=True,
    pools={"HSD_GObj": 4, "HSD_JObj": 6, "Item": 2, "Article": 1, "ItemAttr": 1, "DynamicBoneTable": 1, "Vec3": 2,
           "HSD_DObj": 2, "HSD_MObj": 2, "HSD_TObj": 2, "HSD_AObj": 2},
    rules={**ITEM_RULES,
           "HSD_JObj.u": ("uptr", "dobj", "HSD_DObj", False),
           "HSD_JObj.aobj": ("ptr", "HSD_AObj", False),
           "HSD_DObj.mobj": ("ptr", "HSD_MObj", False),
           "HSD_MObj.tobj": ("ptr", "HSD_TObj", False),
           "HSD_TObj.aobj": ("ptr", "HSD_AObj", False),
           "Article.x4_specialAttributes": ("expr", "HATTR", "&raw mut super::HATTR"),
           "Item.xBBC_dynamicBoneTable": ("ptr", "DynamicBoneTable", False),
           "itHeiho_ItemVars.x54": ("ptr", "HSD_GObj", True),
           "itHeiho_ItemVars.x21": ("range", 1, 5)},
    uv={**FLAG_UV, "Item_xDD4_itemVar": "heiho"}, uv_only={"Item_xDD4_itemVar": ["heiho"]},
    statics=[dict(c="HATTR", rs="super::HATTR", n=16, region=True), dict(c="HINT", rs="super::HINT", region=True)],
    skip_data=["result", "item", "zero", "pos"],
    c_decl="static unsigned HATTR[16];\nstatic int HINT;\n",
    rs_decl="pub static mut HATTR: [u64; 8] = [0; 8];\npub static mut HINT: i32 = 0;\n",
    ret={**RAND_RET, "it_8027B5B0": ("pick_HSD_GObj(1)", "pick_HSD_GObj(1)"),
         "it_8028FAF4": ("pick_HSD_GObj(1)", "pick_HSD_GObj(1)")},
    effects={"Stage_UnkSetVec3TCam_Offset": ("{ a0->x = rg(); a0->y = rg(); a0->z = rg(); }",
                                             "{ (*a0).x = rg(); (*a0).y = rg(); (*a0).z = rg(); }")},
    deref={"it_8028F9D8": [1]},
    setup_c=JOBJ_SETUP_C + "\n" + BONES_C + "\n"
            "    for (n = 2; n < 16; n++) HATTR[n] = B(rg());\n    HINT = (int) fv(); *(int**) HATTR = &HINT;",
    setup_rs=JOBJ_SETUP_RS + "\n" + BONES_RS + "\n"
             "    for n in 2..16usize { *(&raw mut HATTR as *mut u32).add(n) = rg().to_bits(); }\n    HINT = fv() as i32; *(&raw mut HATTR as *mut *mut i32) = &raw mut HINT;",
    ops=[dict(name="it_802D8618", args=["v", "p:Vec3", "v", "v"]), "it_802D8688", "it_802D8894",
         "itHeiho_UnkMotion0_Anim", "itHeiho_UnkMotion0_Phys", "itHeiho_UnkMotion0_Coll", "it_802D8918",
         dict(name="itHeiho_UnkMotion1_Anim_inline", args=[None, "i:6"]),
         "itHeiho_UnkMotion1_Anim", "itHeiho_UnkMotion1_Phys", "itHeiho_UnkMotion1_Coll",
         "itHeiho_UnkMotion2_Anim", "it_802D8EC8_inline", "itHeiho_UnkMotion2_Phys", "itHeiho_UnkMotion2_Coll",
         "itHeiho_UnkMotion3_Anim", "itHeiho_UnkMotion3_Phys", "itHeiho_UnkMotion3_Coll", "it_802D8EC8",
         "it_802D9168", "itHeiho_UnkMotion4_Anim", "itHeiho_UnkMotion4_Phys", "itHeiho_UnkMotion4_Coll",
         "it_802D96B0", "it_802D9714_inline", "it_802D9714", "it_802D98AC",
         dict(name="it_802D98C4", args=["p:HSD_JObj", ("raw", "&P_Item[rnd() % 2u]", "&raw mut g::P_Item[(rnd() % 2) as usize]")]),
         "it_802D9A0C"],
)
