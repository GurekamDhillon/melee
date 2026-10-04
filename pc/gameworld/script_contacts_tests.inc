/* Existing stage records are preserved; fixture changes never reach a real fighter. */
int ScriptGame_ContactsTest(void)
{
    static Fighter fp;
    static ScriptMeshInstance saved_instance;
    ScriptStageLine saved_line=script_stage.line[0];
    MapCollData* saved_map=script_stage.map;
    int saved_base=script_stage.base_l,saved_cap=script_stage.cap;
    MapCollData fake_map;
    int failed=0,part;
    float cx,cy;
    if(!script_contact_sweep(-10,0,10,0,-2,-4,2,4,&cx,&cy) || cx!=0 || cy!=0)++failed;
    if(!script_contact_sweep(0,0,10,10,2,0,8,10,&cx,&cy) || cx!=5 || cy!=5)++failed;
    if(script_contact_sweep(-10,0,10,0,20,-4,20,4,&cx,&cy))++failed;
    if(script_contact_sweep(-10,0,10,0,0,0,0,4,&cx,&cy))++failed;
    if(script_contact_sweep(-10,0,10,0,0,4,0,-4,&cx,&cy))++failed;
    saved_instance=script_stage.instance[0];
    memset(&fp,0,sizeof fp);memset(&fake_map,0,sizeof fake_map);
    memset(&script_stage.instance[0],0,sizeof script_stage.instance[0]);
    script_stage.map=&fake_map;script_stage.base_l=20;script_stage.cap=1;
    script_stage.line[0].active=1;script_stage.line[0].handle=12345;
    script_stage.instance[0].handle=54321;script_stage.instance[0].owner=9;
    script_stage.instance[0].count=1;script_stage.instance[0].line[0].handle=12345;
    if(script_contact_owner(20,&part)!=54321 || part!=0)++failed;
    if(script_contact_owner(3,&part)!=0 || part!=-1)++failed;
    script_stage.instance[0].count=0;
    if(script_contact_owner(20,&part)!=12345 || part!=-1)++failed;
    fp.coll_data.floor.index=20;fp.ground_or_air=GA_Ground;fp.coll_data.env_flags=Collide_FloorHug;
    if(script_contact_line(&fp,SC_FLOOR)!=20)++failed;
    fp.ground_or_air=GA_Air;if(script_contact_line(&fp,SC_FLOOR)!=-1)++failed;
    fp.coll_data.env_flags=Collide_RightWallPush;fp.coll_data.right_facing_wall.index=21;
    if(script_contact_line(&fp,SC_LEFT)!=21 || script_contact_line(&fp,SC_RIGHT)!=-1)++failed;
    fp.coll_data.env_flags=Collide_LeftWallHug;fp.coll_data.left_facing_wall.index=22;
    if(script_contact_line(&fp,SC_RIGHT)!=22)++failed;
    fp.coll_data.env_flags=Collide_CeilingPush;fp.coll_data.ceiling.index=23;
    if(script_contact_line(&fp,SC_CEILING)!=23)++failed;
    fp.motion_id=ftCo_MS_CliffWait;fp.mv.co.cliff.ledge_id=24;
    if(script_contact_line(&fp,SC_LEDGE)!=24)++failed;
    fp.motion_id=ftCo_MS_JumpF;if(!(script_contact_flags(&fp)&1))++failed;
    fp.motion_id=ftCo_MS_Pass;if(!(script_contact_flags(&fp)&2))++failed;
    fp.motion_id=ftCo_MS_DamageFlyRoll;if(!(script_contact_flags(&fp)&4))++failed;
    fp.motion_id=ftCo_MS_DeadDown;if(!(script_contact_flags(&fp)&8))++failed;
    fp.motion_id=ftCo_MS_RebirthWait;if(!(script_contact_flags(&fp)&16))++failed;
    fp.motion_id=ftCo_MS_Wait;if(script_contact_flags(&fp))++failed;
    script_stage.instance[0]=saved_instance;script_stage.line[0]=saved_line;
    script_stage.map=saved_map;script_stage.base_l=saved_base;script_stage.cap=saved_cap;
    return failed;
}
