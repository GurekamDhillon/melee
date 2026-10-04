/* Any retail static-switch host is neutralised at the GObj proc dispatcher,
 * grDisplay and mpLib ranges. Special scene stages and m-ex callbacks remain
 * excluded until their non-stage-class hazards have an explicit teardown. */
int ScriptStageHostHidden;
static DynamicsDesc* (*script_host_touch)(int);
static bool (*script_host_shadow)(Vec3*,int,HSD_JObj*);
static int script_stage_host_allowed(void)
{
    /* Legal hosts are the audited starting set; hazardous adventure/scrolling
     * and arbitrary m-ex hosts cannot promise restoration by a stage-class gate. */
    switch(stage_info.grkind) {
    case Gr_Kind_Last:case Gr_Kind_Battle:case Gr_Kind_Story:
    case Gr_Kind_OldPupupu:case Gr_Kind_Izumi:case Gr_Kind_PStadium:return 1;
    default:return 0;
    }
}
static void script_stage_host_hide(int hide)
{
    if(hide && !ScriptStageHostHidden) {
        script_host_touch=stage_info.on_touch_line;script_host_shadow=stage_info.on_check_shadow_render;
        stage_info.on_touch_line=NULL;stage_info.on_check_shadow_render=NULL;
        ScriptStageHostHidden=1;
    } else if(!hide && ScriptStageHostHidden) {
        stage_info.on_touch_line=script_host_touch;stage_info.on_check_shadow_render=script_host_shadow;
        ScriptStageHostHidden=0;
    }
}
