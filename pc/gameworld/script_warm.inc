#include "../geno/geno_items.h"
static Article* script_warm_article(int kind)
{
    if(kind<0 || kind>=5000)return NULL;
    if(kind<It_Kind_Kuriboh)return it_804D6D24?it_804D6D24[kind]:NULL;
    if(kind<It_PKind_Start)return it_804D6D38?it_804D6D38[kind-It_Kind_Kuriboh]:NULL;
    if(kind<It_Kind_Old_Kuri)return it_804D6D30?it_804D6D30[kind-It_PKind_Start]:NULL;
    if(kind<It_Kind_Old_Kuri+30)return it_804A0F60[kind-It_Kind_Old_Kuri];
    if(kind>=GENO_ITEM_KIND_BASE && kind<GENO_ITEM_KIND_END) {
        extern void* Geno_ItemDesc(int);return Geno_ItemDesc(kind);
    }
    if(kind>=237) {
        extern int Mex_ItemReady(int);extern void* Mex_ItemCustomDesc(int);
        return Mex_ItemReady(kind)?Mex_ItemCustomDesc(kind):NULL;
    }
    return NULL;
}
int ScriptGame_WarmItemReady(int kind)
{
    Article* article=script_warm_article(kind);
    return article && article->x10_modelDesc && article->x10_modelDesc->x0_joint;
}
/* Pipeline discovery through real descriptor material traversal, without item
 * spawning, AI, hitboxes, or render-object registration. Aurora capture drops
 * the geometry after deriving the same GX shader/pipeline keys as a draw. */
int ScriptGame_WarmDraw(int category, int index)
{
    HSD_JObj* joint = NULL;
    Article* article = NULL;
    Mtx view;
    int owned = 0, kind = index;
    HSD_GObj* camera = Camera_80030A50();
    if (!camera || !camera->hsd_obj) return 0; /* stage not ready; retry */
    HSD_CObjGetViewingMtx(camera->hsd_obj, view);
    if (category == 0) {
        if (index < 0 || index >= SCRIPT_ENEMY_KINDS) return -1;
        if (!script_enemy_preload(index)) return -1;
        kind = script_enemy_kinds[index];
    }
    if (category == 2) {
        HSD_GObj* fighter;
        if (index < 0 || index >= 6) return -1;
        fighter = Player_GetEntityAtIndex(index, 0);
        if (!fighter) return -1;
        joint = GET_JOBJ(fighter);
    } else {
        article=script_warm_article(kind);
        if (!article || !article->x10_modelDesc || !article->x10_modelDesc->x0_joint) return -1;
        joint = HSD_JObjLoadJoint(article->x10_modelDesc->x0_joint);
        owned = 1;
    }
    if (!joint) return -1;
    HSD_StateInvalidate(-1);
    HSD_JObjDispAll(joint, view, HSD_GObj_80390EB8(0) | HSD_GObj_80390EB8(1) |
                   HSD_GObj_80390EB8(2), 0);
    GXFlush();
    { extern void Script_WarmSync(void); Script_WarmSync(); }
    if (owned) HSD_JObjRemoveAll(joint);
    HSD_StateInvalidate(-1);
    return 1;
}

/* Owned mesh art supplies a game-endian view to the host GXMS draw path. */
int ScriptGame_WarmModel(int model)
{
    Mtx view;
    HSD_GObj* camera = Camera_80030A50();
    extern void Script_WarmModelDraw(int, void*);
    if (!camera || !camera->hsd_obj) return 0;
    HSD_CObjGetViewingMtx(camera->hsd_obj, view);
    HSD_StateInvalidate(-1);
    Script_WarmModelDraw(model, view);
    GXFlush();
    HSD_StateInvalidate(-1);
    return 1;
}
