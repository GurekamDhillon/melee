/* No retail resources required: deterministic distances/team filtering and
 * byte-exact restoration of the entire timed-value record. */
int ScriptGame_CapsTargetTest(void)
{
    static Fighter a,b,c;
    ScriptCapsStatus state, saved;
    int rc=0;
    memset(&a,0,sizeof a); memset(&b,0,sizeof b); memset(&c,0,sizeof c);
    a.player_id=0; b.player_id=1; c.player_id=2;
    a.team=0; b.team=0; c.team=1;
    b.cur_pos.x=3; b.cur_pos.y=4; c.cur_pos.x=-5;
    b.motion_id=c.motion_id=ftCo_MS_Wait;
    if (script_caps_opponent(&a,&a,0) || !script_caps_opponent(&a,&b,0) ||
        script_caps_opponent(&a,&b,1) || !script_caps_opponent(&a,&c,1)) rc++;
    if (script_caps_distance2(&a,&b)!=25 || script_caps_distance2(&a,&c)!=25) rc++;
    b.player_id=0;
    if (script_caps_opponent(&a,&b,0)) rc++; /* Nana shares a port */
    b.player_id=1; b.motion_id=ftCo_MS_DeadDown;
    if (script_caps_opponent(&a,&b,0)) rc++;
    memset(&state,0,sizeof state);
    if (!script_caps_status_set(&state,&a,1,2.5f,3) ||
        script_caps_status_set(&state,&a,2,1,3)) rc++;
    saved=state;
    script_caps_status_tick(&state,&a);
    if (state.frames!=2 || state.value!=2.5f || state.owner!=1) rc++;
    state=saved;
    if (memcmp(&saved,&state,sizeof state)) rc++;
    script_caps_status_tick(&state,&a); script_caps_status_tick(&state,&a);
    script_caps_status_tick(&state,&a);
    if (state.owner || state.frames || state.fighter || state.value) rc++;
    script_caps_status_set(&state,&a,1,3,3);
    script_caps_status_tick(&state,&b);
    if (state.owner) rc++; /* replaced entity cannot inherit effect */
    return rc;
}
