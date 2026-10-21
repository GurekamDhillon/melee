/* Verbatim copies of the decomp C bodies ported in src/itlib.rs (round 2), extracted unchanged
 * from the files named below; #included by round2.c. */
/* ---- src/melee/it/it_26B1.c */
void it_8026B294(HSD_GObj* gobj, Vec3* pos)
{
    Item* temp_item;

    temp_item = gobj->user_data;
    *pos = temp_item->pos;
}

bool itIsHeavy(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;

    if (temp_item->xCC_item_attr->x0_is_heavy) {
        return true;
    }

    return false;
}

bool it_8026B2D8(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    if (temp_item->xCC_item_attr->x0_is_heavy == 1) {
        return true;
    }
    return false;
}

ItemKind itGetKind(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    return temp_item->kind;
}

enum_t it_8026B30C(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    return ip->xCC_item_attr->x0_78;
}

Item_HoldKinds itGetHoldKind(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    return temp_item->xCC_item_attr->x0_hold_kind;
}

f32 itGetDamageMultiplier(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    return temp_item->xCC_item_attr->x1C_damage_mul;
}

void it_8026B344(HSD_GObj* gobj, Vec3* pos)
{
    Item* ip;

    ip = gobj->user_data;
    pos->x = ((ip->facing_dir * ip->xBCC_unk.x) + ip->pos.x);
    pos->y = (ip->pos.y + ip->xBCC_unk.y);
    pos->z = ip->pos.z;
}

f32 itGetGrabRangeX(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    return temp_item->xBD4_grabRange.x;
}

f32 itGetGrabRangeY(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    return temp_item->xBD4_grabRange.y;
}

void it_8026B390(HSD_GObj* gobj)
{
    Item* temp_item;

    temp_item = gobj->user_data;
    temp_item->xDC8_word.flags.x15 = 1;
}

void it_8026B3A8(HSD_GObj* gobj)
{
    Item* temp_item;

    temp_item = gobj->user_data;
    temp_item->xDC8_word.flags.x15 = 0;
}

float it_8026B424(int damage)
{
    ItemCommonData* itCommonData = it_804D6D28;
    return (int) ((damage * itCommonData->xB8) + itCommonData->xBC);
}

s32 it_8026B47C(HSD_GObj* gobj)
{
    s32 kind;
    Item* temp_item;

    temp_item = gobj->user_data;
    kind = temp_item->kind;
    switch (kind) {
    case It_Kind_Heart:
        return temp_item->xDD4_itemVar.heart.xDD4_heal;
    case It_Kind_Tomato:
        return temp_item->xDD4_itemVar.tomato.heal_amount;
    case It_Kind_Foods:
        return temp_item->xDD4_itemVar.foods.heal_amount;
    case It_Kind_Lucky_Egg:
        return temp_item->xDD4_itemVar.egg.heal_amount;
    case It_Kind_WhispyHealApple:
        return temp_item->xDD4_itemVar.whispyapple.xDD4_heal;
    default:
        return 0;
    }
}

bool it_8026B4F0(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    ItemKind kind = temp_item->kind;

    switch (kind) {
    case It_Kind_Heart:
    case It_Kind_Tomato:
    case It_Kind_Foods:
    case It_Kind_Coin:
    case It_Kind_Lucky_Egg:
    case It_Kind_WhispyHealApple:
        return true;
    default:
        return false;
    }
}

float it_8026B54C(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    itUnkAttributes* unk_attr =
        temp_item->xC4_article_data->x4_specialAttributes;

    return unk_attr->x0_float;
}

float it_8026B560(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    itUnkAttributes* unk_attr =
        temp_item->xC4_article_data->x4_specialAttributes;
    return unk_attr->x0_float;
}

float it_8026B574(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;
    itUnkAttributes* unk_attr =
        temp_item->xC4_article_data->x4_specialAttributes;
    return unk_attr->x4_float;
}

s32 it_8026B588(void)
{
    return it_804D6D28->xDC;
}

bool it_8026B594(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    ItemKind kind = ip->kind;

    switch (kind) {
    case It_Kind_L_Gun:
    case It_Kind_S_Scope:
    case It_Kind_StarRod:
    case It_Kind_LipStick:
    case It_Kind_F_Flower:
        if (ip->xD4C <= 0) {
            return true;
        }

    default:
    case It_Kind_Harisen: // Why though
        return false;
    }
}

bool it_8026B6C8(HSD_GObj* gobj)
{
    s32 kind;
    s32 itemID_2;
    s32 itemID_3;
    Item* ip;

    ip = gobj->user_data;
    kind = ip->kind;
    if (((kind >= It_Kind_Kuriboh) && (kind < It_Kind_Octarock_Stone)) ||
        ((itemID_2 = ip->kind, ((itemID_2 < It_Kind_Old_Kuri) == false)) &&
         (itemID_2 < It_Kind_Arwing_Laser)))
    {
        itemID_3 = ip->kind;
        if ((itemID_3 != It_Kind_Kyasarin_Egg) &&
            (itemID_3 != It_Kind_WhispyApple))
        {
            return true;
        }
    }
    return false;
}

void it_8026B718(HSD_GObj* gobj, f32 hitlagFrames)
{
    Item* ip = gobj->user_data;
    ip->xCBC_hitlagFrames = hitlagFrames;
}

void it_8026B724(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    ip->xDC8_word.flags.x3 = true;
}

void it_8026B73C(HSD_GObj* gobj)
{
    Item* ip;

    ip = gobj->user_data;
    if (ip->xDC8_word.flags.x7 != 0) {
        ip->xDC8_word.flags.x5 = 1;
    }
    if (ip->xDC8_word.flags.x3 != 0) {
        ip->xDC8_word.flags.x3 = 0;
    }
}

bool it_8026B774(HSD_GObj* gobj, u8 arg1)
{
    Item* ip = gobj->user_data;

    if (ip->xDC8_word.flags.xF & (1 << arg1)) {
        return true;
    }
    return false;
}

s32 itGetMotionId(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    return ip->msid;
}

u8 itGetTeamId(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    return ip->x20_team_id;
}

s32 it_8026B7BC(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    return ip->xDC8_word.flags.x14;
}

s32 it_8026B7CC(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    return ip->x1C;
}

s32 it_8026B7E8(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    return ip->xDC8_word.flags.x1;
}

bool it_8026B894(HSD_GObj* gobj, HSD_GObj* ref_gobj)
{
    Item* ip = GET_ITEM(gobj);
    bool result = false;

    if (ip->owner == ref_gobj) {
        ip->owner = NULL;
        result = true;
    }

    if (ip->xC64_reflectGObj == ref_gobj) {
        ip->xC64_reflectGObj = NULL;
    }

    if (ip->xC90_absorbGObj == ref_gobj) {
        ip->xC90_absorbGObj = NULL;
    }

    if (ip->xCEC_fighterGObj == ref_gobj) {
        ip->xCEC_fighterGObj = NULL;
        ip->xCB0_source_ply = 6;
    }

    if (ip->xCF4_fighterGObjUnk == ref_gobj) {
        ip->xCF4_fighterGObjUnk = NULL;
    }

    if (ip->toucher == ref_gobj) {
        ip->toucher = NULL;
    }

    return result;
}

s32 it_8026B924(HSD_GObj* gobj)
{
    s32 result;
    Item* ip = GET_ITEM(gobj);
    enum_t kind = ip->kind;
    result = -1;
    switch (kind) {
    case It_Kind_Freeze:
    case It_Kind_Foods:
    case It_Kind_MSBomb:
    case It_Kind_Flipper:
    case It_Kind_LipStick:
    case It_Kind_Harisen:
        break;
    case It_Kind_S_Scope:
    case It_Kind_StarRod:
    case It_Kind_L_Gun:
    case It_Kind_F_Flower:
        result = ip->xD4C;
    }
    return result;
}

f32 it_8026B960(HSD_GObj* gobj)
{
    s32 kind;
    f32 unk_timer = -1.0f;
    Item* ip;

    ip = gobj->user_data;
    kind = ip->kind;
    switch (kind) {
    case It_Kind_BombHei:
        if ((s32) ip->msid != 0xB) {
            unk_timer = ip->xDD4_itemVar.bombhei.xDEC;
        }
        break;
    case It_Kind_Link_Bomb:
        if ((s32) ip->msid != 5) {
            unk_timer = ip->xD44_lifeTimer;
        }
    }
    return unk_timer;
}

void it_8026BB88(HSD_GObj* gobj, Vec3* pos)
{
    Item* ip = GET_ITEM(gobj);
    ftECB* ecb = &ip->x378_itemColl.ecb;

    /// @todo Why is this always zero? Stripped something?
    f32 offset_xz = 0.0f;

    f32 offset_y = 0.5f * (ecb->top.y + ecb->bottom.y);

    pos->x = ip->pos.x + offset_xz;
    pos->y = ip->pos.y + offset_y;
    pos->z = ip->pos.z + offset_xz;
}

void it_8026BBCC(HSD_GObj* gobj, Vec3* pos)
{
    Item* ip = gobj->user_data;
    CollData* coll_data = &ip->x378_itemColl;
    ftECB* ecb = &coll_data->xE4_ecb;

    /// @todo Why is this always zero? Stripped something?
    f32 offset_xz = 0.0f;

    f32 offset_y = 0.5f * (ecb->top.y + ecb->bottom.y);

    pos->x = coll_data->last_pos.x + offset_xz;
    pos->y = coll_data->last_pos.y + offset_y;
    pos->z = coll_data->last_pos.z + offset_xz;
}

bool it_8026BC68(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    return ip->xDD0_flag.b0;
}

s32 itGetAttackId(HSD_GObj* gobj)
{
    Item* ip = gobj->user_data;
    return ip->xD88_attackID;
}

void it_8026BCF4(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDCD_flag.b2 = false;
}

void it_8026BD0C(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDCD_flag.b2 = true;
}

void it_8026BD24(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDD0_flag.b3 = true;
}

void it_8026BD3C(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDCC_flag.b3 = true;
}

void it_8026BD54(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDCC_flag.b3 = 0;
}

void it_8026BD6C(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDCD_flag.b3 = true;
}

void it_8026BD84(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDCD_flag.b4 = true;
}

void it_8026BD9C(HSD_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xDC8_word.flags.x1A = true;
}

void it_8026BDCC(HSD_GObj* gobj)
{
    /// @todo Each of these is an inlined function. Some are already defined.
    {
        Item* ip = GET_ITEM(gobj);
        ip->xDD0_flag.b3 = false;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDCC_flag.b3 = true;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDCD_flag.b3 = false;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDCD_flag.b4 = false;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDC8_word.flags.x1A = true;
    }
}

void it_8026BE28(HSD_GObj* gobj)
{
    /// @todo Each of these is an inlined function. Some are already defined.
    {
        Item* ip = GET_ITEM(gobj);
        ip->xDD0_flag.b3 = true;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDCC_flag.b3 = false;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDCD_flag.b3 = true;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDCD_flag.b4 = true;
    }

    {
        Item* ip = GET_ITEM(gobj);
        ip->xDC8_word.flags.x1A = false;
    }
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

void it_8026C334(HSD_GObj* gobj, Vec3* pos)
{
    Item* ip = GET_ITEM(gobj);
    f32 offset_xz = 0.0f;
    f32 offset_y = ip->x378_itemColl.ecb.bottom.y;

    pos->x = ip->pos.x + offset_xz;
    pos->y = ip->pos.y + offset_y;
    pos->z = ip->pos.z + offset_xz;
}

/* ---- src/melee/it/it_2725.c */
bool it_80273030(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->x40_vel.x = -item->x40_vel.x * item->xC70;
    item->x40_vel.y = -item->x40_vel.y * item->xC70;
    item->facing_dir = -item->facing_dir;
    item->xD44_lifeTimer = item->xD48_halfLifeTimer;
    return false;
}

bool it_80273130(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xD44_lifeTimer--;
    if (item->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

void it_8027346C(Item_GObj* item_gobj)
{
    Item* item = item_gobj->user_data;

    item->x64_vec_unk2.x = item->x64_vec_unk2.y = item->x64_vec_unk2.z = 0.0f;
}

void it_80273484(Item_GObj* item_gobj)
{
    Item* item = item_gobj->user_data;

    item->x58_vec_unk.x = item->x58_vec_unk.y = item->x58_vec_unk.z = 0.0f;
}

void it_8027349C(Item_GObj* item_gobj)
{
    Item* item = item_gobj->user_data;

    item->x70_nudge.x = item->x70_nudge.y = item->x70_nudge.z = 0.0f;
}

void it_802734B4(Item_GObj* item_gobj)
{
    Item* item = GET_ITEM(item_gobj);
    item->x7C.x = item->x7C.y = item->x7C.z = 0.0f;
    item->x88.x = item->x88.y = item->x88.z = 0.0f;
    item->x94.x = item->x94.y = item->x94.z = 0.0f;
    item->xA0.x = item->xA0.y = item->xA0.z = 0.0f;
    item->xAC_unk.x = item->xAC_unk.z = 0.0f;
    item->xAC_unk.y = 1.0f;
}

void it_80274C60(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.xC = 0;
}

bool it_80274C78(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    return item->xDC8_word.flags.xC;
}

void it_80274C88(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDD0_flag.b0 = 0;
    item->xDAA_flag.b3 = 0;
}

void it_80274D6C(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    f32 right_x = ip->x378_itemColl.ecb.right.x;
    f32 right_y = ip->x378_itemColl.ecb.right.y;
    f32 left_x = ip->x378_itemColl.ecb.left.x;
    f32 top_y = ip->x378_itemColl.ecb.top.y;

    ip->xB54.x8.y = right_x + ip->pos.x;
    ip->xB54.x8.x = left_x + ip->pos.x;
    ip->xB54.x8.z = right_y + ip->pos.y + top_y;
}

void it_80274DAC(Item_GObj* gobj)
{
    Item* ip = GET_ITEM(gobj);
    ip->xB54.unk_x = 0.0f;
    ip->xB54.unk_y = 0.0f;
    {
        Item* ip = GET_ITEM(gobj);
        CollData* cd = &ip->x378_itemColl;
        f32 right_x = cd->ecb.right.x;
        f32 right_y = cd->ecb.right.y;
        f32 left_x = cd->ecb.left.x;
        f32 top_y = cd->ecb.top.y;
        ip->xB54.x8.y = right_x + ip->pos.x;
        ip->xB54.x8.x = left_x + ip->pos.x;
        ip->xB54.x8.z = right_y + ip->pos.y + top_y;
    }
}

void it_80274DFC(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xBCC_unk.x *= item->scl;
    item->xBCC_unk.y *= item->scl;
    item->xBD4_grabRange.x *= item->scl;
    item->xBD4_grabRange.y *= item->scl;
}

void it_80274E44(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xBEC.top *= item->scl;
    item->xBEC.bottom *= item->scl;
    item->xBEC.right *= item->scl;
    item->xBEC.left *= item->scl;
    item->xBDC.top *= item->scl;
    item->xBDC.bottom *= item->scl;
    item->xBDC.right *= item->scl;
    item->xBDC.left *= item->scl;
}

void it_80274ECC(Item_GObj* item_gobj, bool arg1)
{
    ((Item*) item_gobj->user_data)->xD09 = arg1;
}

void it_80274EF8(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.x2 = 0;
}

void it_80274F10(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.x2 = 1;
}

bool it_802750E8(Item_GObj* item_gobj, s32 mask)
{
    Item* item = GET_ITEM(item_gobj);
    return item->xDC0 & mask;
}

bool it_802751D8(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xD44_lifeTimer--;
    if (item->xD44_lifeTimer <= 0.0f) {
        return true;
    }
    return false;
}

void it_80275210(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.x1C = 1;
}

void it_80275228(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.x1C = 0;
}

void it_80275240(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.x1E = 0;
}

void it_80275258(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDD0_flag.b4 = 0;
}

void it_80275270(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDD0_flag.b4 = 1;
}

void it_802753BC(Item_GObj* item_gobj, s16 arg1)
{
    Item* item;

    item = item_gobj->user_data;
    item->xC94 = 0;
    item->xC9C = arg1;
    item->xCA0 = 0;
    item->xCA4 = 0;
}

/* ---- src/melee/it/ithitbox.c */
void it_802753DC(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xC34_damageDealt = 0;
    item->xC40 = 1.0f;
    item->xC44 = 1.0f;
    item->xC3C = 1.0f;
    item->xC48 = 0;
    item->xC4C = 0;
    item->xC50 = 0;
    item->xDCE_flag.b6 = 0;
}

void it_80275414(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCD_flag.b7 = 1;
}

void it_8027542C(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCD_flag.b7 = 0;
}

void it_80275444(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCD_flag.b5 = 1;
    item->xDCD_flag.b7 = 1;
    item->xDCD_flag.b6 = 1;
}

void it_80275474(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCD_flag.b5 = 0;
    item->xDCD_flag.b7 = 0;
    item->xDCD_flag.b6 = 0;
}

void it_802754A4(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCE_flag.b2 = 1;
}

void it_802754BC(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCE_flag.b2 = 0;
}

void it_802754D4(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCE_flag.b0 = 1;
    item->xDCE_flag.b2 = 1;
    item->xDCE_flag.b1 = 1;
}

void it_80275504(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDCE_flag.b0 = 0;
    item->xDCE_flag.b2 = 0;
    item->xDCE_flag.b1 = 0;
}

void it_80275534(Item_GObj* item_gobj, f32 scale)
{
    Item* item;
    HitCapsule* hitcapsule;
    u32 var_ctr;

    item = item_gobj->user_data;
    for (var_ctr = 0U; var_ctr < 4U; var_ctr++) {
        hitcapsule = &item->x5D4_hitboxes[var_ctr].hit;
        if (hitcapsule->state != HitCapsule_Disabled) {
            hitcapsule->scale = scale;
        }
    }
}

void it_80275594(Item_GObj* item_gobj, s32 idx, f32 mult)
{
    Item* item;
    HitCapsule* hitcapsule;

    item = item_gobj->user_data;
    hitcapsule = &item->x5D4_hitboxes[idx].hit;
    if (hitcapsule->state != HitCapsule_Disabled) {
        hitcapsule->scale *= mult;
    }
}

void it_802755C0(Item_GObj* item_gobj, f32 scale)
{
    Item* item;
    HitCapsule* hitcapsule;
    u32 var_ctr;

    item = item_gobj->user_data;
    for (var_ctr = 0U; var_ctr < 4U; var_ctr++) {
        hitcapsule = &item->x5D4_hitboxes[var_ctr].hit;
        if (hitcapsule->state != HitCapsule_Disabled) {
            hitcapsule->scale *= scale;
        }
    }
}

void it_802756D0(Item_GObj* item_gobj)
{
    ((Item*) item_gobj->user_data)->xD0C = 2;
}

void it_8027570C(Item_GObj* item_gobj, s32 idx)
{
    Item* item;

    item = item_gobj->user_data;
    item->x5D4_hitboxes[idx].hit.x40_b0 = 0;
}

void it_8027572C(Item_GObj* item_gobj, s32 idx)
{
    Item* item;

    item = item_gobj->user_data;
    item->x5D4_hitboxes[idx].hit.x40_b0 = 1;
}

void it_80275788(Item_GObj* item_gobj)
{
    Item* item;
    HitCapsule* hitcapsule;
    HitCapsuleState state;
    u32 var_ctr;

    item = item_gobj->user_data;
    state = HitCapsule_Unk4;
    for (var_ctr = 0U; var_ctr < 4U; var_ctr++) {
        hitcapsule = (0, &item->x5D4_hitboxes[var_ctr].hit);
        if (hitcapsule->state != HitCapsule_Disabled) {
            hitcapsule->state = state;
            item->xDAA_flag.b2 = 1;
        }
    }
}

void it_80275820(Item_GObj* item_gobj, Vec3* arg1, Vec3* arg2, s32 idx)
{
    Item* item;
    HitCapsule* hitcapsule;

    item = item_gobj->user_data;
    hitcapsule = &item->x5D4_hitboxes[idx].hit;
    if (hitcapsule->state != HitCapsule_Disabled) {
        hitcapsule->x58 = *arg2;
        hitcapsule->x4C = *arg1;
    }
}

bool it_80275870(Item_GObj* item_gobj)
{
    Item* item;
    HitCapsule* hitcapsule;
    u32 var_ctr;

    item = item_gobj->user_data;
    for (var_ctr = 0U; var_ctr < 4U; var_ctr++) {
        hitcapsule = &item->x5D4_hitboxes[var_ctr].hit;
        if (hitcapsule->state != HitCapsule_Disabled) {
            return true;
        }
    }
    return false;
}

f32 it_802758D4(Item_GObj* item_gobj)
{
    Item* item;
    HitCapsule* hitcapsule;
    u32 var_ctr;
    f32 damage;
    bool disable_chk;

    item = item_gobj->user_data;
    damage = 0.0f;
    for (var_ctr = 0U; var_ctr < 4U; var_ctr++) {
        hitcapsule = &item->x5D4_hitboxes[var_ctr].hit;
        if (hitcapsule->state != HitCapsule_Disabled) {
            disable_chk = true;
            break;
        } else {
            disable_chk = false;
        }
    }

    if (disable_chk) {
        for (var_ctr = 0U; var_ctr < 4U; var_ctr++) {
            hitcapsule = &item->x5D4_hitboxes[var_ctr].hit;
            if (hitcapsule->state != HitCapsule_Disabled) {
                if (damage <= hitcapsule->damage) {
                    damage = hitcapsule->damage;
                }
            }
        }
    }
    return damage;
}

/* ---- src/melee/it/itmaplib.c */
bool it_802762D8(Item_GObj* item_gobj)
{
    CollData* coll;
    bool chk;

    chk = false;
    coll = &((Item*) item_gobj->user_data)->x378_itemColl;
    if (coll->env_flags & Collide_LeftEdge) {
        chk = true;
    }
    if (coll->env_flags & Collide_RightEdge) {
        return true;
    }
    return chk;
}

s32 it_80276308(Item_GObj* item_gobj)
{
    Item* item;
    CollData* coll;
    s32 ret_val;

    item = item_gobj->user_data;
    ret_val = 0;
    coll = &item->x378_itemColl;
    if (item->x378_itemColl.env_flags & Collide_LeftWallMask) {
        ret_val = 8;
        item->xC30 = coll->left_facing_wall.index;
    }
    if (coll->env_flags & Collide_RightWallMask) {
        item->xC30 = coll->right_facing_wall.index;
        ret_val = 4;
    }
    return ret_val;
}

s32 it_80276348(Item_GObj* item_gobj, Vec3* vec)
{
    s32 ret_val;
    Item* item;
    CollData* coll;

    item = item_gobj->user_data;
    ret_val = 0;
    coll = &item->x378_itemColl;
    if (coll->env_flags & Collide_LeftWallMask) {
        ret_val = 8;
        item->xC30 = coll->left_facing_wall.index;
        *vec = coll->left_facing_wall.normal;
    }
    if (coll->env_flags & Collide_RightWallMask) {
        item->xC30 = coll->right_facing_wall.index;
        *vec = coll->right_facing_wall.normal;
        ret_val = 4;
    }
    return ret_val;
}

bool it_802763B8(Item_GObj* item_gobj)
{
    Item* item;
    CollData* coll;
    bool chk = false;

    item = item_gobj->user_data;
    coll = &item->x378_itemColl;
    if (coll->env_flags & Collide_FloorMask) {
        item->xC30 = coll->floor.index;
        chk = true;
    }
    return chk;
}

void it_80276408(Item_GObj* item_gobj, CollData* coll, Vec3* vec)
{
    if (coll->env_flags & Collide_LeftWallMask) {
        *vec = coll->left_facing_wall.normal;
    }
    if (coll->env_flags & Collide_RightWallMask) {
        *vec = coll->right_facing_wall.normal;
    }
    if (coll->env_flags & Collide_CeilingMask) {
        *vec = coll->ceiling.normal;
    }
    if (coll->env_flags & Collide_FloorMask) {
        *vec = coll->floor.normal;
    }
}

/* ---- src/melee/it/itcoll.c */
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

void it_80271508(Item_GObj* item_gobj, HurtCapsuleState state)
{
    u32 index;
    Item* item;
    HurtCapsule* hurt;

    item = item_gobj->user_data;
    index = 0U;
    while (index < item->xAC8_hurtboxNum) {
        hurt = &item->xACC_itemHurtbox[index];
        hurt->state = state;
        index++;
    }
}

void it_80271534(Item_GObj* item_gobj, s32 index, HurtCapsule* arg_hurt)
{
    HurtCapsule* hurt;
    Item* item;

    item = item_gobj->user_data;
    if (item->xC4_article_data->x8_hurtbones != NULL) {
        hurt = &item->xACC_itemHurtbox[index];
        hurt->a_offset = arg_hurt->a_offset;
        hurt->b_offset = arg_hurt->b_offset;
        hurt->scale = arg_hurt->scale;
    }
}

void it_80271590(Item_GObj* item_gobj, s32 index, HurtCapsule* arg_hurt)
{
    u8 _[8];
    Vec3 sp18;
    HurtCapsule* hurt;
    Item* item;

    item = GET_ITEM(item_gobj);
    if (item->xC4_article_data->x8_hurtbones != NULL) {
        hurt = &item->xACC_itemHurtbox[index];
        arg_hurt->a_offset = hurt->a_offset;
        arg_hurt->b_offset = hurt->b_offset;
        arg_hurt->scale = hurt->scale;
        return;
    }
    sp18.x = sp18.y = sp18.z = 0.0f;
    arg_hurt->a_offset = sp18;
    arg_hurt->b_offset = sp18;
    arg_hurt->scale = 0.0f;
}

void it_80272280(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.x1B = 0;
}

void it_80272298(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDC8_word.flags.x1B = 1;
}

