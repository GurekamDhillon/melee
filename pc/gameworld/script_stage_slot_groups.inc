/* Return -1 unless both lines belong to the active slot. No pointer transport. */
int ScriptGame_StageSlotSameJoint(int a,int b)
{
    ScriptStageSlot* s;int i,ia=-1,ib=-1;
    if(!script_switch.active)return -1;
    s=script_slot(script_switch.active,0);
    if(!s || s->nmesh || a<0 || b<0)return -1;
    for(i=0;i<script_switch.nlines;++i) {
        int at=script_stage_seam_slot(script_switch.lines[i]);
        if(at<0)continue;
        if(a==script_stage.base_l+at)ia=i;
        if(b==script_stage.base_l+at)ib=i;
    }
    if(ia<0 || ib<0)return -1;
    return s->lines[ia].joint==s->lines[ib].joint;
}
