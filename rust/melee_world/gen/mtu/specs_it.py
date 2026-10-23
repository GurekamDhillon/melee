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
