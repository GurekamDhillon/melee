/* Headless scalar adapter fixture, no fighter spawn, assets or collision writes. */
int ScriptGame_ZonesAdapterTest(int owner) {
    ScriptMeshInstance saved_model;SzInput input[SZ_ENTITIES];
    int i,at=-1,h,rc=0,serial=script_zones.serial,mask=script_zones_mask;
    if(script_zones.count || script_zones.pending)return 1;
    for(i=0;i<SCRIPT_MESH_INSTANCES;i++)if(!script_stage.instance[i].handle){at=i;break;}
    if(at<0)return 2;
    saved_model=script_stage.instance[at];
    memset(&script_stage.instance[at],0,sizeof script_stage.instance[at]);
    script_stage.instance[at].handle=INT_MAX-2;
    script_stage.instance[at].field[SM_X]=script_zone_bits(10);
    script_stage.instance[at].field[SM_Y]=script_zone_bits(20);
    memset(&script_zone_draft,0,sizeof script_zone_draft);
    strcpy(script_zone_draft.name,"zone-adapter-fixture");strcpy(script_zone_draft.kind,"room");
    script_zone_draft.n=4;script_zone_draft.model=INT_MAX-2;
    script_zone_draft.x[0]=script_zone_draft.x[3]=-1;script_zone_draft.x[1]=script_zone_draft.x[2]=1;
    script_zone_draft.y[0]=script_zone_draft.y[1]=-1;script_zone_draft.y[2]=script_zone_draft.y[3]=1;
    h=ScriptGame_ZoneCommit(0,owner,100);i=sz_slot(&script_zones,h);
    if(h<=100 || i<0 || !sz_contains(&script_zones.z[i],10,20))rc=3;
    if(i>=0) {
        script_zone_draft=script_zones.z[i];script_zone_draft.dx=script_zone_draft.dy=0;
        strcpy(script_zone_draft.label,"changed");
        if(ScriptGame_ZoneCommit(h,owner,100)!=h || !sz_contains(&script_zones.z[i],10,20))rc=4;
        memset(input,0,sizeof input);input[0].token=44;input[0].x=10;input[0].y=20;
        sz_step(&script_zones,input,15,1);
        script_stage.instance[at].field[SM_X]=script_zone_bits(30);
        ScriptGame_ZonesFrame(2);
        if(!sz_contains(&script_zones.z[i],30,20) || sz_contains(&script_zones.z[i],10,20))rc=5;
        script_stage.instance[at].handle=0;ScriptGame_ZonesFrame(2);
        if(script_zones.count || script_zones.pending)rc=6;
    }
    /* Reconstruct a pre-creation snapshot's serial; the host allocation floor
     * must still refuse aliasing the old cached handle on a new write. */
    memset(&script_zones,0,sizeof script_zones);script_zones.serial=serial;
    memset(&script_zone_draft,0,sizeof script_zone_draft);
    strcpy(script_zone_draft.name,"zone-after-restore");strcpy(script_zone_draft.kind,"room");
    script_zone_draft.n=3;script_zone_draft.x[1]=2;script_zone_draft.y[2]=2;
    if(ScriptGame_ZoneCommit(0,owner,h)<=h || ScriptGame_ZoneRemove(h,owner))rc=7;
    memset(&script_zones,0,sizeof script_zones);script_zones.serial=serial;
    script_stage.instance[at]=saved_model;script_zones_mask=mask;return rc;
}
