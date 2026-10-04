/* Extra fighter inputs live in snapshotted game BSS, never in PADStatus[4].
 * Fields: owner, remaining logic frames, buttons, x,y,cx,cy,L,R. */
static int script_virtual_pad[2][9];
static int script_virtual_seen[2][7];
void ScriptGame_VirtualPadObserve(Fighter* fp)
{
    int* seen;
    if (fp->player_id<4 || fp->player_id>=6 || fp->is_sub_fighter) return;
    seen=script_virtual_seen[fp->player_id-4];
    seen[0]=fp->cpu.buttons;
    seen[1]=fp->cpu.lstick.x; seen[2]=fp->cpu.lstick.y;
    seen[3]=fp->cpu.cstick.x; seen[4]=fp->cpu.cstick.y;
    seen[5]=fp->cpu.ltrigger; seen[6]=fp->cpu.rtrigger;
}
int ScriptGame_VirtualPadSeen(int slot,int field)
{
    return slot>=4 && slot<6 && field>=0 && field<7 ? script_virtual_seen[slot-4][field] : 0;
}
int ScriptGame_VirtualPadField(int slot, int field, int value, int writing)
{
    if (slot<4 || slot>=6 || field<0 || field>=9) return 0;
    if (writing) script_virtual_pad[slot-4][field]=value;
    return script_virtual_pad[slot-4][field];
}
void ScriptGame_VirtualPadRelease(int slot, int owner)
{
    int i;
    if (slot<0 && !owner) memset(script_virtual_seen,0,sizeof script_virtual_seen);
    for (i=0; i<2; ++i)
        if ((slot<0 || slot==i+4) && (!owner || script_virtual_pad[i][0]==owner))
            memset(script_virtual_pad[i],0,sizeof script_virtual_pad[i]);
}
void ScriptGame_VirtualPadConsume(void)
{
    int i;
    for (i=0; i<2; ++i) if (script_virtual_pad[i][1]>0) --script_virtual_pad[i][1];
}
int ScriptGame_VirtualPadApply(Fighter* fp)
{
    int* v; int active;
    if (fp->player_id<4 || fp->player_id>=6) return 0;
    v=script_virtual_pad[fp->player_id-4];
    if (!v[0]) return 0;
    active=v[1]>0 && !ScriptGame_FighterBenched(fp->player_id);
    fp->cpu.buttons=active ? v[2] : 0;
    fp->cpu.lstick.x=active ? v[3] : 0;
    fp->cpu.lstick.y=active ? v[4] : 0;
    fp->cpu.cstick.x=active ? v[5] : 0;
    fp->cpu.cstick.y=active ? v[6] : 0;
    fp->cpu.ltrigger=active ? v[7] : 0;
    fp->cpu.rtrigger=active ? v[8] : 0;
    ScriptGame_VirtualPadObserve(fp);
    return 1; /* skip fighting AI while claimed, including connected neutral */
}

/* The ordinary time-mode death path retains a sleeping entity. Reinitialize
 * that entity through retail's respawn reset, then use reserve's existing floor,
 * ECB, input and camera placement. No new allocations or character-file loads.
 * Paired fighters/transforms are refused until their reset ordering is tested. */
static int script_recycle_policy(Fighter* fp, int slot, int character,
                                 int current, int paired, int cpu, int benched)
{
    if (!fp || !fp->x890_cameraBox || slot<0 || slot>=6) return 1;
    if (character>=0 && character!=current) return 2;
    if (paired) return 3;
    if (!cpu) return 4;
    if ((fp->motion_id!=ftCo_MS_Sleep && fp->motion_id!=ftCo_MS_Rebirth &&
         fp->motion_id!=ftCo_MS_RebirthWait) ||
        (fp->motion_id==ftCo_MS_Sleep && !fp->x221F_b3) || fp->victim_gobj || fp->x1A5C || fp->item_gobj || benched) return 5;
    if (fp->kind==Ft_Kind_MasterH || fp->kind==Ft_Kind_CrezyH) return 3;
    return 0;
}
static int script_recycle_check(Fighter* fp, int slot, int character)
{
    if (!fp || slot<0 || slot>=6) return 1;
    return script_recycle_policy(fp,slot,character,Player_GetPlayerCharacter(slot),
        Player_GetEntityAtIndex(slot,1)!=NULL,Player_GetPlayerSlotType(slot)==Gm_PKind_Cpu,
        script_bench[slot].owner!=0);
}
int ScriptGame_FighterRecycleCheck(int slot,int character)
{
    return script_recycle_check(script_fighter(slot),slot,character);
}
int ScriptGame_FighterRecycle(int slot,int character,int xb,int yb,int fb,int intangible)
{
    Fighter* fp=script_fighter(slot);
    float x=script_mesh_float(xb), y=script_mesh_float(yb), facing=script_mesh_float(fb);
    int rc=script_recycle_check(fp,slot,character);
    if (rc) return rc;
    if (!(x>=-49000 && x<=49000 && y>=-49000 && y<=49000) ||
        (facing!=1 && facing!=-1) || intangible<0 || intangible>600) return 6;
    /* The dead entity has already run death cleanup. Rebirth's reset refreshes
     * shield, jumps, hitboxes, Geno data and CPU initialization from the player.
     * Clear sleep exclusion after reset before entering Wait/Fall. */
    Fighter_UnkProcessDeath_80068354(fp->gobj);
    fp->x221F_b3=0; fp->x221F_b1=0;
    fp->x2219_b1=0; fp->x2219_b2=0;
    fp->x221E_b1=0; fp->x221E_b2=0; fp->invisible=0;
    fp->x890_cameraBox->force_inactive=0;
    Player_SetHUDDamage(slot,0);
    fp->dmg.x1830_percent=0;
    script_bench_place(fp,x,y,facing,intangible,&script_bench_ops);
    OSReport("script recycle: fresh enemy port=%d character=%d x=%g y=%g\n",
             slot+1,Player_GetPlayerCharacter(slot),x,y);
    return 0;
}
