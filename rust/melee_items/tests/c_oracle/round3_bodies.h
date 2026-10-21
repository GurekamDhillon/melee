/* Verbatim copies of the decomp C bodies ported in src/itlib3.rs (round 3), extracted unchanged
 * from the files named below; #included by round3.c. */
/* ---- src/melee/it/it_2725.c */
Fighter* it_80272818(Item* item)
{
    return ((it_2728_DatAttrs*) item->xC4_article_data->x4_specialAttributes)
        ->fighter;
}

Item_GObj* it_80272828(ItemKind kind)
{
    Item_GObj* item_gobj_return;
    Item* new_var2;
    Item_GObj* item_gobj_check;
    void* new_var;
    item_gobj_return = NULL;
    item_gobj_check = (Item_GObj*) HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM];
    while (item_gobj_check != NULL) {
        new_var2 = (Item*) (new_var = item_gobj_check->user_data);
        if (new_var2->kind == kind) {
            item_gobj_return = item_gobj_check;
        }

        item_gobj_check = item_gobj_check->next;
    }
    return item_gobj_return;
}

void it_80272860(Item_GObj* item_gobj, f32 arg1, f32 arg2)
{
    Item* item;
    f32 var_f3;
    s32 var_r0;
    s32 var_r3;

    item = item_gobj->user_data;
    var_r0 = arg1 < 0.0f ? -1 : 1;
    var_f3 = item->x40_vel.y;
    var_r3 = var_f3 < 0.0f ? -1 : 1;

    if (var_r3 != var_r0) {
        if (var_f3 < 0.0f) {
            var_f3 = -var_f3;
        }
        if (var_f3 < arg2) {
        label_1:
            item->x40_vel.y -= arg1;
            return;
        }
        return;
    }
    // big dumb
    goto label_1;
}

bool it_80272D1C(Item_GObj* item_gobj)
{
    if ((item_gobj != NULL) && (item_gobj->classifier == HSD_GOBJ_CLASS_ITEM))
    {
        return true;
    }
    return false;
}

void itColl_BounceOffVictim(Item_GObj* gobj)
{
    Item* item = GET_ITEM(gobj);
    item->x40_vel.x *= it_804D6D28->x58_float;
    item->x40_vel.y =
        (item->x40_vel.y * it_804D6D28->x5C_float) + it_804D6D28->x60_float;
}

void it_80273500(Item_GObj* item_gobj, Vec3* arg1)
{
    Item* item = GET_ITEM(item_gobj);
    item->x40_vel.x = arg1->x;
    item->x40_vel.y = arg1->y;
    item->x40_vel.z = arg1->z;
    it_8027346C(item_gobj);
    it_80273484(item_gobj);
    it_8027349C(item_gobj);
    it_802734B4(item_gobj);
}

int it_80274658(Item_GObj* item_gobj, f32 arg1)
{
    Item* item;
    f32 spin_speed1;
    f32 spin_speed2;
    s32 dir1;
    s32 dir2;

    item = GET_ITEM(item_gobj);
    spin_speed2 = 0.0f;
    spin_speed1 = item->spin_spd;
    if (spin_speed1 != spin_speed2) {
        spin_speed2 = 0.01f * spin_speed1 * (0.017453292f * arg1);
    }
    item->xD3C_spinSpeed = spin_speed2;
    if (item->facing_dir < 0.0f) {
        dir1 = -1;
    } else {
        dir1 = 1;
    }
    if (item->x40_vel.x < 0.0f) {
        dir2 = -1;
    } else {
        dir2 = 1;
    }
    if (dir2 != dir1) {
        item->xD3C_spinSpeed = -item->xD3C_spinSpeed;
    }
    if (item->xDC8_word.flags.x19 != 1) {
        item->xD3C_spinSpeed *= -item->facing_dir;
    }
    return dir2;
}

void it_80274D04(Item_GObj* item_gobj, struct lb_80014638_arg1_t* arg1)
{
    Vec3 sp20;
    Vec3 sp14;
    PAD_STACK(4);

    it_8026BB88(item_gobj, &sp20);
    it_8026BBCC(item_gobj, &sp14);
    arg1->unk_x = sp20.x - sp14.x;
    arg1->unk_y = sp20.y - sp14.y;
}

void it_80275174(Item_GObj* item_gobj, f32 lifetime)
{
    ((Item*) item_gobj->user_data)->xD48_halfLifeTimer =
        lifetime * it_804D6D28->x4C_float;
}

void it_80275328(HSD_GObj* gobj, ItemLink* arg1)
{
    arg1->prev = NULL;
    arg1->next = NULL;
    arg1->vel.z = 0.0f;
    arg1->vel.y = 0.0f;
    arg1->vel.x = 0.0f;
    arg1->pos.z = 0.0f;
    arg1->pos.y = 0.0f;
    arg1->pos.x = 0.0f;
    arg1->x20.z = 0.0f;
    arg1->x20.y = 0.0f;
    arg1->x20.x = 0.0f;
    arg1->x2C_b0 = 0;
    arg1->x2C_b1 = 0;
    arg1->x2C_b2 = 0;
    arg1->x1CC = 0;
    arg1->gobj = gobj;
    arg1->jobj = NULL;
}

void it_80275390(Item_GObj* item_gobj)
{
    ((Item*) item_gobj->user_data)->owner = NULL;
    it_8026B3A8(item_gobj);
}

/* ---- src/melee/it/it_26B1.c */
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

void it_8026C3FC(void)
{
    HSD_GObj* cur;
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]; cur != NULL;
         cur = cur->next)
    {
        it_8026B724(cur);
    }
}

void it_8026C42C(void)
{
    HSD_GObj* cur;
    for (cur = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_ITEM]; cur != NULL;
         cur = cur->next)
    {
        Item* ip = GET_ITEM(cur);

        if (ip->xDC8_word.flags.x7) {
            ip->xDC8_word.flags.x5 = true;
        }

        if (ip->xDC8_word.flags.x3) {
            ip->xDC8_word.flags.x3 = false;
        }
    }
}

/* ---- src/melee/it/it_279C.c */
void it_8027A13C(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDD4_itemVar.pokemon_spawn.x10.z = 0.0f;
    item->xDD4_itemVar.pokemon_spawn.x10.y = 0.0f;
    item->xDD4_itemVar.pokemon_spawn.x10.x = 0.0f;
    item->xDD4_itemVar.pokemon_spawn.x4.z = 0.0f;
    item->xDD4_itemVar.pokemon_spawn.x4.y = 0.0f;
    item->xDD4_itemVar.pokemon_spawn.x4.x = 0.0f;
}

void it_8027A344(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->x40_vel.x = item->xDD4_itemVar.pokemon_spawn.x4.z * item->facing_dir;
    item->x40_vel.y = item->xDD4_itemVar.pokemon_spawn.x4.y;
}

s32 it_8027A364(Item* item)
{
    itPokemonSpawn_DatAttrs* attr =
        item->xC4_article_data->x4_specialAttributes;
    int ret_val = 1;
    int i;
    int start = It_PKind_Start;
    int end = It_PKind_Terminate;

    for (i = start; i < end; i++) {
        if (Item_804A0E24.last_kind != i && Item_804A0E24.previous_kind != i) {
            ret_val += attr->pokemon_spawn_weights[i - It_PKind_Start];
        }
    }

    return ret_val;
}

void it_8027AAA0(Item_GObj* item1_gobj, Item* item2, s32 arg2)
{
    Item* var_r6;

    Item* it = GET_ITEM(item1_gobj);
    itPokemonSpawn_DatAttrs* attr = it->xC4_article_data->x4_specialAttributes;
    if (arg2 == It_PKind_Marumine) {
        item2->xDD4_itemVar.pokemon_spawn.x1C = 0.0f;
        item2->xDD4_itemVar.pokemon_spawn.x20 = 0.0f;
    } else {
        item2->xDD4_itemVar.pokemon_spawn.x1C = attr->xC;
        item2->xDD4_itemVar.pokemon_spawn.x20 = attr->x10;
    }
    item2->xDD4_itemVar.pokemon_spawn.x24 = attr->x14;
    var_r6 = item2;
    item2->xDD4_itemVar.pokemon_spawn.x28 = attr->x18;
    item2->xDD4_itemVar.pokemon_spawn.x48 =
        (item2->xCC_item_attr->x60_scale -
         item2->xDD4_itemVar.pokemon_spawn.x4C) /
        item2->xDD4_itemVar.pokemon_spawn.x28;
    item2->xDD4_itemVar.pokemon_spawn.x44 = attr->x34;
    item2->xDD4_itemVar.pokemon_spawn.x54 = attr->x38;
    item2->xDD4_itemVar.pokemon_spawn.x5C = 0.0f;
    item2->xDD4_itemVar.pokemon_spawn.x58 = 0.0f;
    item2->xDD4_itemVar.pokemon_spawn.x50 = 0.0f;

    {
        int i;
        for (i = 0; i < attr->x38; i++) {
            item2->xDD4_itemVar.pokemon_spawn.x2C[i] =
                attr->x1C[i] * item2->xCC_item_attr->x60_scale;
        }
    }
}

/* ---- src/melee/it/itanimlist.c */
void it_802795EC(Item_GObj* item_gobj, CommandInfo* cmd)
{
    s32 idx = cmd->u->set_hitbox_scale.idx;
    Item* item = item_gobj->user_data;
    HitCapsule* hit = &item->x5D4_hitboxes[idx].hit;
    PAD_STACK(8);
    hit->scale = 0.003906f * cmd->u->set_hitbox_scale.value;
    item->x3C = hit->scale;
    it_80275594(item_gobj, idx, 1.0f / item->scl);
    ++cmd->u;
}

void it_802796FC(Item_GObj* item_gobj, CommandInfo* cmd)
{
    Item* it = GET_ITEM(item_gobj);
    it->xDAC_itcmd_var0 = cmd->u->set_throw_flags.hit_idx;
    ++cmd->u;
}

/* ---- src/melee/it/itcoll.c */
f32 it_80270CD8(Item* ip, HitCapsule* hit)
{
    ItemAttr* attr = ip->xCC_item_attr;
    f32 f0;
    f32 f1;

    if (hit->x28 != 0) {
        f1 = (0.01f * hit->x24 *
              ((it_804D6D28->x80_float[11] *
                (attr->x1C_damage_mul *
                 ((it_804D6D28->x80_float[10] * it_804D6D28->x80_float[8]) +
                  (it_804D6D28->x80_float[9] *
                   (it_804D6D28->x80_float[10] * hit->x28))))) +
               it_804D6D28->x80_float[12])) +
             hit->x2C;
    } else {
        f1 = ((0.01f * hit->x24) *
              ((it_804D6D28->x80_float[11] *
                (attr->x1C_damage_mul *
                 ((it_804D6D28->x80_float[8] * (ip->xC9C + (f32) ip->xCA0)) +
                  (it_804D6D28->x80_float[9] *
                   (hit->damage * (ip->xC9C + (f32) ip->xCA0)))))) +
               it_804D6D28->x80_float[12])) +
             hit->x2C;
    }
    if (f1 >= it_804D6D28->x80_float[7]) {
        f1 = it_804D6D28->x80_float[7];
    }
    return f1;
}

/* ---- src/melee/it/item.c */
void Item_80268E40(Item* item_data, struct ItemStateDesc* itemStateDesc)
{
    item_data->x524_cmd.u = itemStateDesc->xC_script;
    item_data->x524_cmd.loop_count = 0;
    item_data->x524_cmd.timer = 0.0f;
}

static void Item_80269CC4(HSD_GObj* gobj)
{
    Item* temp_item = gobj->user_data;

    temp_item->xCA0 = 0;
    temp_item->xCA4 = 0;
    temp_item->xC94 = 0;
    temp_item->xCC8_knockback = 0.0f;
    temp_item->xC34_damageDealt = 0;
    temp_item->xC48 = 0;
    temp_item->xC4C = 0;
    temp_item->xC50 = 0;
    temp_item->xCCC_incDamageDirection = 0.0f;
    temp_item->xCB8_outDamageDirection = 0.0f;
    temp_item->xC68 = 0.0f;
    temp_item->xCD0 = 0.0f;
    temp_item->xCC0 = 0.0f;
    temp_item->xDCF_flag.b6 = false;
    temp_item->xCC4 = 0;
    temp_item->xCEC_fighterGObj = 0;
    temp_item->xCF0_itemGObj = 0;
    temp_item->xCF4_fighterGObjUnk = 0;
    temp_item->toucher = 0;
    temp_item->xC64_reflectGObj = 0;
    temp_item->xC90_absorbGObj = 0;
    temp_item->xCFC = 0;
    temp_item->xCB4 = -1;
    temp_item->xC38 = -1;
    temp_item->xCE0.z = 0.0f;
    temp_item->xCE0.y = 0.0f;
    temp_item->xCE0.x = 0.0f;
    temp_item->xCD4.z = 0.0f;
    temp_item->xCD4.y = 0.0f;
    temp_item->xCD4.x = 0.0f;
    temp_item->xDCE_flag.b5 = false;
    temp_item->xDCE_flag.b4 = false;
    temp_item->xDCC_flag.b1 = false;
    temp_item->xC70 = 1.0f;
    temp_item->xC6C = 1.0f;
    temp_item->xDCC_flag.b2 = false;
    temp_item->xDC8_word.flags.xB = false;
    temp_item->xCA8 = 0;
    temp_item->xDC8_word.flags.x3 = false;
    temp_item->xDC8_word.flags.x4 = false;
    temp_item->xDCE_flag.b6 = false;
}

u32 Item_8026AE60(void)
{
    u32 result = it_804D6D14++;

    if (it_804D6D14 == 0) {
        it_804D6D14++;
    }

    return result;
}

bool Item_IsGrabbable(Item_GObj* gobj)
{
    Item* temp_item;

    temp_item = gobj->user_data;

    if ((temp_item->xDC8_word.flags.x15 != 0) &&
        (temp_item->xB8_itemLogicTable->picked_up != NULL))
    {
        return true;
    }

    return false;
}

/* ---- src/melee/it/ithitbox.c */
void it_802756E0(Item_GObj* item_gobj)
{
    ((Item*) item_gobj->user_data)->xD0C = 0;
    it_802714C0(item_gobj);
}

/* ---- src/melee/it/itmaplib.c */
void it_80276CEC(Item_GObj* item_gobj)
{
    CollData* coll;
    Item* item;
    u32 coll_index;

    item = item_gobj->user_data;
    coll = &item->x378_itemColl;
    if (coll->env_flags & Collide_LeftWallMask) {
        item->xAC_unk = coll->left_facing_wall.normal;
        coll_index = coll->left_facing_wall.index;
    }
    if (coll->env_flags & Collide_RightWallMask) {
        item->xAC_unk = coll->right_facing_wall.normal;
        coll_index = coll->right_facing_wall.index;
    }
    if (coll->env_flags & Collide_CeilingMask) {
        item->xAC_unk = coll->ceiling.normal;
        coll_index = coll->ceiling.index;
    }
    if (coll->env_flags & Collide_FloorMask) {
        item->xAC_unk = coll->floor.normal;
        coll_index = coll->floor.index;
    }
    item->xC2C = coll_index;
}

/* ---- src/melee/it/itspawn.c */
void it_8026CA4C(ItemPickTable* alloc, s32* arg1, u64 arg2, s32 arg3, f32 arg4)
{
    u64 mask = arg2;
    s32* p = arg1 + arg3;
    s32 i = arg3;
    s32 sum = 0;

    while (i < It_Kind_L_Gun_Ray) {
        if (mask & 1) {
            sum += arg4 * *p + 0.99f;
        }
        p++;
        i++;
        mask >>= 1;
    }
    alloc->x8 = sum;
}

/* ---- src/melee/it/itzako.c */
void it_8027B730(Item_GObj* item_gobj)
{
    Item* item;

    item = item_gobj->user_data;
    item->xDD4_itemVar.zako.jobj = 0;
    item->xDD4_itemVar.zako.idx = -1;
    item->xDD4_itemVar.zako.x8.z = 0.0f;
    item->xDD4_itemVar.zako.x8.x = 0.0f;
    item->xDD4_itemVar.zako.x8.y = 1.0f;
    item->xDD4_itemVar.zako.x14.x = 0;
    item->xDD4_itemVar.zako.x14.y = 0;
    item->xDD4_itemVar.zako.x14.z = 0;
    it_80275228(item_gobj);
    it_80275240(item_gobj);
}

f32 it_8027CBFC(Item_GObj* item_gobj)
{
    s32 ret_val;
    PAD_STACK(4);

    ret_val = 0U;
    switch (GET_ITEM(item_gobj)->xCC4) {
    case 6:
    case 7:
        ret_val = it_804D6D40->xC;
        break;
    case 9:
        ret_val = it_804D6D40->x10;
        break;
    case 4:
    case 5:
    case 10:
    case 11:
    case 12:
    case 13:
    case 14:
    case 15:
    case 16:
        break;
    }
    return (f32) ret_val;
}

