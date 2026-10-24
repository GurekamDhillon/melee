ItemKind itGetKind(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    return temp_item->kind;
}

void it_8026B294(HSD_GObj* gobj, Vec3* pos)
{
    Item* temp_item;

    temp_item = gobj->user_data;
    *pos = temp_item->pos;
}

void it_8026B718(HSD_GObj* gobj, f32 hitlagFrames)
{
    Item* ip = gobj->user_data;
    ip->xCBC_hitlagFrames = hitlagFrames;
}

bool it_8026C1E8(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    if (ip->grab_victim != NULL ||
        (ip->xDC8_word.flags.x13 && ip->owner != NULL))
    {
        return false;
    }

    return true;
}

int it_8026B3C0(ItemKind kind)
{
    Item* temp_item;
    HSD_GObj* unkItemGObj;

    int i = 0;
    unkItemGObj = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM];

    while (unkItemGObj != NULL) {
        temp_item = unkItemGObj->user_data;

        if (temp_item->kind == kind) {
            ++i;
        }

        unkItemGObj = unkItemGObj->next;
    }
    return i;
}

void it_ResetStageArticleTable(void)
{
    int i;
    for (i=0;i<30;++i) it_804A0F60[i]=NULL;
}

void it_8026B40C(Article* article, s32 kind)
{
#if defined(TARGET_PC)
    /* Ported from m-ex (https://github.com/akaneia/m-ex): its C2 hook on this function, read out
     * of the shipped codes.gct. A custom stage registers its own articles through this same call,
     * passing 5000 + article index instead of an item kind; the index is resolved against
     * MexData.stage's item_lookup for the CURRENT stage and the descriptor goes into
     * item.RuntimeIndex, which is where Item_80267978 reads it back for kinds >= 237.
     *
     * Not an optimisation: without this branch `kind - It_Kind_Old_Kuri` is 4792 for article 0,
     * and the store below lands far outside the 30-entry table. */
    if (kind >= 5000) {
        extern void Mex_IndexStageItem(void* article, int article_index);
        Mex_IndexStageItem(article, kind - 5000);
        return;
    }
#endif
    it_804A0F60[kind - It_Kind_Old_Kuri] = article;
}

void it_80275414(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCD_flag.b7 = 1;
}

void it_802756D0(Item_GObj* item_gobj)
{
    ((Item*) item_gobj->user_data)->xD0C = 2;
}

void it_802756E0(Item_GObj* item_gobj)
{
    ((Item*) item_gobj->user_data)->xD0C = 0;
    it_802714C0(item_gobj);
}

void it_802714C0(Item_GObj* item_gobj)
{
    u32 index;
    Item* item;
    HurtCapsule* hurt;

    item = item_gobj->user_data;
    index = 0U;
    while (index < item->xAC8_hurtboxNum) {
        hurt = &item->xACC_itemHurtbox[index];
        hurt->skip_update_pos = 0;
        index++;
    }
    item->xDAA_flag.b1 = 1;
}

void it_80274C60(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.xC = 0;
}

void it_802F2014(Item_GObj* gobj, int arg1)
{
    GET_ITEM(gobj)->x378_itemColl.joint_id_only = arg1;
}

int it_802F2020(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);

    return ip->xDD4_itemVar.coin.x0;
}

int it_802ECC8C(Item_GObj* arg0)
{
    return GET_ITEM(arg0)->xDD4_itemVar.tincle.x20;
}

void it_802ECC98(Item_GObj* arg0, f32 arg1)
{
    GET_ITEM(arg0)->xDD4_itemVar.tincle.x54 = arg1;
}

void it_802ECCA4(Item_GObj* gobj, s32* kind, Vec3* pos)
{
    Item* ip = GET_ITEM(gobj);
    *pos = ip->pos;
    pos->y += ip->xDD4_itemVar.tincle.x50;
    *kind = ip->msid;
}

void it_802ECD1C(Item_GObj* gobj, Vec3* v)
{
    *v = GET_ITEM(gobj)->pos;
}

