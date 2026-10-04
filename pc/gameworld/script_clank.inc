/* Fetch the final engine result at dispatch, after collision think callbacks.
 * Walk live objects before dereferencing: projectiles may destroy themselves on
 * clank. Addresses are same-frame observation ids, never script item handles. */
int ScriptGame_ClankValue(int address,int is_item,int rebound)
{
    HSD_GObj* g;
    int link=is_item?HSD_GOBJ_PLINK_ITEM:HSD_GOBJ_PLINK_FIGHTER;
    /* The object table is absent before a match initializes GObjs. */
    if(!address || !HSD_GObjPLinkHead)return 0;
    for(g=HSD_GObjPLinkHead[link];g;g=g->next)if((int)g==address) {
        union { float f; int i; } u;
        if(!g->user_data)return 0;
        if(is_item) {Item* it=g->user_data;if(rebound)return 0;u.f=it->xCBC_hitlagFrames;}
        else {Fighter* fp=g->user_data;
            if(rebound)return fp->motion_id==ftCo_MS_ReboundStop || fp->motion_id==ftCo_MS_Rebound;
            u.f=fp->dmg.x195c_hitlag_frames;}
        return u.i;
    }
    return 0;
}
