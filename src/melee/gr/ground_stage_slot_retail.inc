/* TARGET_PC only: install an already parsed private visit, then run the exact
 * retail initialization/load/start functions. Never touches scene heaps/DVD. */
extern void grDatFiles_StageSlotInstall(HSD_Archive*);
void Ground_StageSlotRetailInstall(StageIdPair* pair,HSD_Archive* archive)
{
    StageData* stage=stage_datas[pair->grkind];
    Ground_801BFFB0();
    stage_info.grkind=pair->grkind;
    grDatFiles_StageSlotInstall(archive);
    Ground_801C28CC(&stage_info.xA0,pair->stkind);
    stage_info.on_touch_line=stage->on_touch_line;
    stage_info.on_check_shadow_render=stage->on_check_shadow_render;
    Ground_801C5878();
    Ground_801C0800(pair);
    Ground_StageSlotFighterLightingRebuild();
    Ground_OnLoad(pair);
    Ground_801C0FB8(pair);
}
