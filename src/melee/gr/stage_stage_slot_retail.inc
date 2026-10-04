/* Keep the external identity used by item scale/music queries in agreement
 * with the active internal stage, without invoking DVD/scene initialization. */
int Stage_StageSlotSelect(int kind)
{
    int i;
    for(i=0;i<(int)ARRAY_SIZE(stage_id_map);++i)if(stage_id_map[i].grkind==kind) {
        selected_stage.stkind=(StKind)i;selected_stage.entry=&stage_id_map[i];
        return i;
    }
    return -1;
}
