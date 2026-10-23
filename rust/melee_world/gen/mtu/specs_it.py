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
