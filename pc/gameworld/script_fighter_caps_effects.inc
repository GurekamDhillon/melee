/* Fighter capability effects. Included after ScriptGame_CapsFighter.
 * Guest BSS is part of LAB/rollback memory; never retain native pointers here. */
#include "script_fighter_caps.h"
#include <melee/ft/ftcoll.h>
#include <melee/ft/ftcolanim.h>
#include <melee/ft/ftlib.h>
#include <melee/ft/ftmetal.h>
#include <melee/ft/ftchangeparam.h>
#include <melee/ft/ft_081B.h>
#include <melee/ft/kinds/ftCommon/ftpickupitem.h>
#include <melee/it/item.h>
#include <melee/it/itspawn.h>
#include <melee/it/it_3F14.h>
#include <melee/it/it_26B1.h>
#include <sysdolphin/baselib/random.h>

struct script_caps_effect {
    Fighter* fighter;
    int owner, frames, value_bits, baseline, last_timer, foreign;
    float scale;
};
static struct script_caps_effect script_caps_effects[12][4];
struct script_caps_armour { Fighter* fighter; int owner; float damage, kb; };
static struct script_caps_armour script_caps_armours[12];
static int script_caps_threshold(float damage, float kb, float d, float k)
{ return (d > 0 && damage < d) || (k > 0 && kb < k); }
static int script_caps_hit_threshold(Fighter* fp, float d, float k)
{
    union { int i; float f; } selected;
    /* ftColl_8007A06C writes DmgResult.damage at +0x28 relative to
     * &fp->dmg.facing_dir_1 (fp+1844): fp+186C, despite its int declaration.
     * This is the winning retail hit's scaled damage, not the frame percent sum. */
    selected.i = fp->dmg.x186c;
    return script_caps_threshold(selected.f, fp->dmg.kb_applied, d, k);
}
static int script_caps_effect_valid(int kind, float value, int frames)
{
    return kind >= 0 && kind < 4 && frames > 0 && frames <= 3600 &&
        value == value && (kind == 3 ? (value >= 0.25f && value <= 4.0f) : (value >= 0 && value <= 1));
}
/* Retail timers may pause in hitlag. Track observed decrements rather than
 * comparing them with our logic-frame duration, so expiry still removes our
 * protection in hitlag. An observed extension belongs to a newer retail effect. */
static void script_caps_effect_observe(struct script_caps_effect* s,int timer)
{
    int elapsed=s->last_timer-timer;
    if (elapsed<0) s->foreign=1;
    else if (s->baseline>0) {
        s->baseline-=elapsed;
        if (s->baseline<0) s->baseline=0;
    }
    s->last_timer=timer;
}
static void script_caps_effect_end(int entity, int kind)
{
    struct script_caps_effect* s = &script_caps_effects[entity-1][kind];
    Fighter* fp = ScriptGame_CapsFighter(entity);
    if (s->owner && fp && fp == s->fighter) {
        if (kind < 2) {
            int* timer = kind == 0 ? &fp->x1990 : &fp->x1994;
            script_caps_effect_observe(s,*timer);
            if (!s->foreign) {
                *timer = s->baseline;
                fp->x198C = (fp->x1990 || fp->x2221_b0) ? 2 : fp->x1994 ? 1 : 0;
                if (!fp->x198C) ftCo_800C0200(fp, 9);
            }
        } else if (kind == 2) {
            script_caps_effect_observe(s,fp->metal_timer);
            if (fp->is_metal && !s->foreign)
                ftCo_800C8540(fp->gobj);
        } else {
            union { int i; float f; } value;
            value.i = s->value_bits;
            if (fp->x34_scale.y == value.f) {
                ftLib_SetScale(fp->gobj, s->scale);
                ftCo_800D105C(fp->gobj);
                ft_80081C88(fp->gobj, fp->x34_scale.y);
            }
        }
    }
    memset(s, 0, sizeof *s);
}
int ScriptGame_CapsEffect(int entity, int owner, int kind, int value_bits, int frames)
{
    Fighter* fp = ScriptGame_CapsFighter(entity);
    struct script_caps_effect* s;
    union { int i; float f; } value;
    value.i = value_bits;
    if (!fp || owner <= 0 || kind < 0 || kind > 3) return 0;
    if (!frames) {
        s = &script_caps_effects[entity-1][kind];
        if (s->owner && s->owner != owner) return 0;
        script_caps_effect_end(entity,kind);
        OSReport("script fighter effect: cleared entity=%d kind=%d owner=%d\n",entity,kind,owner);
        return 1;
    }
    if (!script_caps_effect_valid(kind,value.f,frames)) return 0;
    s = &script_caps_effects[entity-1][kind];
    if (s->owner && s->owner != owner) return 0;
    /* Do not take ownership of an existing metal match/item state. */
    if (kind == 2 && fp->is_metal && !s->owner) return 0;
    if (s->owner) script_caps_effect_end(entity, kind);
    s->fighter = fp; s->owner = owner; s->frames = frames; s->value_bits = value_bits;
    if (kind == 0) { s->baseline = fp->x1990; ftColl_8007B760(fp->gobj, frames); }
    else if (kind == 1) { s->baseline = fp->x1994; ftColl_8007B7A4(fp->gobj, frames); }
    else if (kind == 2) {
        ftCo_800C8348(fp->gobj, frames, 0x7fffffff);
        ftCo_800D105C(fp->gobj);
        ft_80081C88(fp->gobj, fp->x34_scale.y);
    }
    else {
        s->scale = fp->x34_scale.y;
        ftLib_SetScale(fp->gobj, value.f);
        ftCo_800D105C(fp->gobj);
        ft_80081C88(fp->gobj, value.f);
    }
    s->last_timer=kind==0 ? fp->x1990 : kind==1 ? fp->x1994 : kind==2 ? fp->metal_timer : 0;
    OSReport("script fighter effect: entity=%d kind=%d frames=%d owner=%d\n",entity,kind,frames,owner);
    return 1;
}
int ScriptGame_CapsEffectRead(int entity, int kind, int field)
{
    struct script_caps_effect* s;
    if (entity < 1 || entity > 12 || kind < 0 || kind > 3) return 0;
    s = &script_caps_effects[entity-1][kind];
    if (s->fighter != ScriptGame_CapsFighter(entity)) return 0;
    return field == 0 ? s->frames : field == 1 ? s->value_bits : field == 2 ? s->owner : 0;
}
int ScriptGame_CapsArmour(int entity, int owner, int damage_bits, int kb_bits)
{
    Fighter* fp = ScriptGame_CapsFighter(entity);
    struct script_caps_armour* s;
    union { int i; float f; } d,k;
    d.i = damage_bits; k.i = kb_bits;
    if (!fp || owner <= 0 || !(d.f >= 0 && d.f <= 1000) || !(k.f >= 0 && k.f <= 1000)) return 0;
    s = &script_caps_armours[entity-1];
    if (s->owner && s->owner != owner) return 0;
    if (!d.f && !k.f) {
        memset(s,0,sizeof *s);
        OSReport("script fighter armour: cleared entity=%d owner=%d\n",entity,owner);
        return 1;
    }
    s->fighter = fp; s->owner = owner; s->damage = d.f; s->kb = k.f;
    OSReport("script fighter armour: entity=%d damage=%g knockback=%g owner=%d\n",entity,d.f,k.f,owner);
    return 1;
}
int ScriptGame_CapsArmourRead(int entity, int field)
{
    struct script_caps_armour* s;
    union { float f; int i; } value;
    if (entity < 1 || entity > 12) return 0;
    s = &script_caps_armours[entity-1];
    if (s->fighter != ScriptGame_CapsFighter(entity)) return 0;
    if (field == 0) return s->owner;
    value.f = field == 1 ? s->damage : field == 2 ? s->kb : 0;
    return value.i;
}
/* Read overlay: retail calls inlineB2, the existing no-reaction cleanup branch. */
int ScriptGame_CapsNoFlinch(Fighter* fp)
{
    int i;
    for (i=0;i<12;++i) {
        struct script_caps_armour* s = &script_caps_armours[i];
        if (s->owner && s->fighter == fp && ScriptGame_CapsFighter(i+1) == fp)
            return script_caps_hit_threshold(fp,s->damage,s->kb);
    }
    return 0;
}
void ScriptGame_CapsEffectsRelease(int owner)
{
    int i,k;
    ScriptGame_ArmorRelease(owner);
    for (i=0;i<12;++i) {
        for (k=0;k<4;++k) if (script_caps_effects[i][k].owner &&
            (!owner || script_caps_effects[i][k].owner == owner)) script_caps_effect_end(i+1,k);
        if (!owner || script_caps_armours[i].owner == owner)
            memset(&script_caps_armours[i],0,sizeof script_caps_armours[i]);
    }
}
void ScriptGame_CapsEffectsReset(void) { ScriptGame_CapsEffectsRelease(0); }
void ScriptGame_CapsEffectsFrame(void)
{
    int i,k;
    ScriptGame_ArmorFrame();
    for (i=0;i<12;++i) {
        for (k=0;k<4;++k) {
            struct script_caps_effect* s = &script_caps_effects[i][k];
            if (!s->owner) continue;
            if (ScriptGame_CapsFighter(i+1) != s->fighter) { memset(s,0,sizeof *s); continue; }
            if (k<3) {
                Fighter* fp=s->fighter;
                script_caps_effect_observe(s,k==0 ? fp->x1990 : k==1 ? fp->x1994 : fp->metal_timer);
            }
            if (--s->frames <= 0) script_caps_effect_end(i+1,k);
        }
        if (script_caps_armours[i].owner && ScriptGame_CapsFighter(i+1) != script_caps_armours[i].fighter)
            memset(&script_caps_armours[i],0,sizeof script_caps_armours[i]);
    }
}
static int script_caps_holdable(int kind)
{
    switch (kind) {
    case It_Kind_BombHei: case It_Kind_Dosei: case It_Kind_Bat: case It_Kind_Sword:
    case It_Kind_G_Shell: case It_Kind_R_Shell: case It_Kind_L_Gun: case It_Kind_Freeze:
    case It_Kind_MSBomb: case It_Kind_S_Scope: case It_Kind_StarRod: case It_Kind_LipStick:
    case It_Kind_Harisen: case It_Kind_F_Flower: case It_Kind_M_Ball: return 1;
    default: return 0;
    }
}
int ScriptGame_CapsGiveItem(int entity, int owner, int kind)
{
    Fighter* fp = ScriptGame_CapsFighter(entity);
    Item_GObj* item;
    SpawnItem spawn;
    /* Conservative portable holdable set; reject heavy/consumable/character articles. */
    if (!fp || owner <= 0 || fp->item_gobj || fp->x1978 ||
        fp->dmg.x195c_hitlag_frames > 0 || !Item_80266F3C()) return 0;
    /* Grant only in neutral locomotion, avoiding capture/death/get-item state assumptions. */
    switch (fp->motion_id) {
    case ftCo_MS_Wait: case ftCo_MS_Fall: case ftCo_MS_JumpF: case ftCo_MS_JumpB:
    case ftCo_MS_JumpAerialF: case ftCo_MS_JumpAerialB: break;
    default: return 0;
    }
    if (kind == -1) {
        ItemPickTable* t = &it_804A0E30.x4;
        int i, total = 0;
        kind = -1;
        if (!t->size || !t->x8 || !t->x4 || !t->xC) return 0;
        for (i=0;i<t->size;++i) {
            int candidate = t->x4[i];
            int weight = (i+1 < t->size ? t->xC[i+1] : t->x8) - t->xC[i];
            if (weight <= 0 || !script_caps_holdable(candidate) || !it_8026D324(candidate)) continue;
            total += weight;
            if (HSD_Randi(total) < weight) kind = candidate;
        }
        if (kind < 0) return 0;
    }
    if (!script_caps_holdable(kind)) return 0;
    if (!it_8026D324(kind) || Item_804A0C64.x60 >= Item_804A0C64.x64) return 0;
    memset(&spawn,0,sizeof spawn);
    spawn.kind = kind; spawn.pos = spawn.prev_pos = fp->cur_pos;
    {
        Vec4* offset = fp->ground_or_air == GA_Ground ? &fp->x294_itPickup.gr_light_offset : &fp->x294_itPickup.air_light_offset;
        spawn.pos.x += fp->facing_dir * offset->x;
        spawn.pos.y += offset->y;
    }
    spawn.prev_pos = spawn.pos; spawn.facing_dir = fp->facing_dir; spawn.x44_flag.b0 = 1;
    item = Item_80268B18(&spawn);
    if (!item) return 0;
    if (!ftpickupitem_80094150(fp->gobj,item)) { Item_8026A8EC(item); return 0; }
    ftpickupitem_800948A8(fp->gobj,item);
    if (fp->item_gobj == item) {
        OSReport("script fighter item: entity=%d kind=%d owner=%d\n",entity,kind,owner);
        return 1;
    }
    Item_8026A8EC(item);
    return 0;
}
#include "script_fighter_caps_effects_tests.inc"
