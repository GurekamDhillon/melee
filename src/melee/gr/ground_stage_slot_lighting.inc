/* TARGET_PC: the fighter light is created by Fighter_800679B0, outside
 * Ground's visit ledger. Its descriptors/animations belong to the stage and
 * ftCo_09F4 also keeps two private pointers into the cloned light chain. */
#include <melee/ft/kinds/ftCommon/ftCo_09F4.h>
#include <sysdolphin/baselib/state.h>

void Ground_StageSlotFighterLightingClear(void)
{
    HSD_GObj* gobj;
    HSD_GObj* next;
    /* Drop current-list references before freeing either light chain, and
     * disable the outgoing visit's GX fog until the retail camera binds it. */
    HSD_LObjDeleteCurrentAll(NULL);
    HSD_FogSet(NULL);
    HSD_StateInvalidate(-1);
    if (HSD_GObjPLinkHead == NULL) {
        return;
    }
    for (gobj = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_LIGHT]; gobj != NULL;
         gobj = next)
    {
        next = gobj->next;
        /* Exact retail identity, not all lights on this link: retain the
         * stage and unrelated effect lights for their respective owners. */
        if (gobj->classifier == 12 && gobj->obj_kind == HSD_GObj_LightKind) {
            HSD_GObjFree(gobj);
        }
    }
}

void Ground_StageSlotFighterLightingRebuild(void)
{
    extern void ScriptGame_StageSlotRetailCaptureResume(void);
    extern void ScriptGame_StageSlotRetailCaptureEnd(void);
    /* Ground init has installed map_plit and stage scale. Use the retail
     * constructor: refreshes BOTH private caches, coordinate scaling,
     * animation request, draw link and process. Must precede on_load: FoD
     * grIzumi_OnLoad sets AOBJ_LOOP on this very fighter light chain. */
    ScriptGame_StageSlotRetailCaptureResume();
    ftCo_8009F4A4();
    ScriptGame_StageSlotRetailCaptureEnd();
    HSD_StateInvalidate(-1);
    OSReport("stage switch: destination fighter lighting rebuilt\n");
}
