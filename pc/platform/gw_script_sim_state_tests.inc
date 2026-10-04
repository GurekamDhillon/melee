/* Safe fixture tests; no fighter or scene required. The actual snapshot/logic
 * ordering is additionally checked by gd.rewind_test in the owner's LAB. */
/* Step 2 of the one-system spine: every capability writer is admitted by the journal. The fixture
 * parses each op with the same bounds as its gd.* setter, applies a commit, wipes the records the way
 * a restored keyframe would, replays the commit, and compares what the read-backs return. */
static int gs_sim_probe_cf(lua_State *L) { gs_sim_parse_caps(L,(GsSimOp*)lua_touserdata(L,1)); return 0; }
static int gs_sim_probe(int kind,const char *table,GsSimOp *o) {
    lua_State *L=gs.L; int top=lua_gettop(L),bad;
    memset(o,0,sizeof *o); o->kind=kind;
    if(luaL_loadstring(L,table)||lua_pcall(L,0,1,0)) { lua_settop(L,top); return 1; }
    lua_pushcfunction(L,gs_sim_probe_cf); lua_pushlightuserdata(L,o); lua_pushvalue(L,-3);
    bad=lua_pcall(L,2,0,0); lua_settop(L,top); return bad;
}
static void gs_sim_caps_readback(int entity,int out[16]) {
    int n=0,i,type;
    out[n++]=gw_ScriptGame_CapsMovementRead(entity,0); out[n++]=gw_ScriptGame_CapsMovementRead(entity,1); out[n++]=gw_ScriptGame_CapsMovementRead(entity,2);
    out[n++]=gw_ScriptGame_CapsEffectRead(entity,2,0); out[n++]=gw_ScriptGame_CapsEffectRead(entity,2,1);
    out[n++]=gw_ScriptGame_CapsArmourRead(entity,0); out[n++]=gw_ScriptGame_CapsArmourRead(entity,1); out[n++]=gw_ScriptGame_CapsArmourRead(entity,2);
    out[n++]=gw_ScriptGame_CapsStatusRead(entity,2,0); out[n++]=gw_ScriptGame_CapsStatusRead(entity,2,1);
    for(type=1,i=0;type<=2;++type) { out[n++]=gw_ScriptGame_ArmorRead(entity,type,0); out[n++]=gw_ScriptGame_ArmorRead(entity,type,1); out[n++]=gw_ScriptGame_ArmorRead(entity,type,3); (void)i; }
}
static int test_sim_capability_ops(void) {
    GsSimCommit *c; GsSimOp parsed; int before[16],after[16],i,rc=0;
    const int owner=1;
    c=(GsSimCommit*)calloc(1,sizeof *c); if(!c) return 1;
    c->script=0; c->count=0;
    /* parse: accepted and refused forms, per op */
    if(gs_sim_probe(GS_SIM_CAPS,"return {entity=7,values={air_jumps=3,run=true}}",&c->ops[0]) || c->ops[0].slot!=7 || c->ops[0].eff[0]!=3 || c->ops[0].eff[1]!=4) rc|=1;
    if(!gs_sim_probe(GS_SIM_CAPS,"return {entity=1,values={air_jumps=9}}",&parsed)) rc|=2;
    if(!gs_sim_probe(GS_SIM_CAPS,"return {entity=13}",&parsed)) rc|=2;
    if(gs_sim_probe(GS_SIM_EFFECT,"return {entity=7,effect='metal',value=1,frames=90}",&c->ops[1]) || c->ops[1].eff[0]!=2 || c->ops[1].eff[2]!=90) rc|=4;
    if(!gs_sim_probe(GS_SIM_EFFECT,"return {entity=1,effect='metal',value=1,frames=3601}",&parsed)) rc|=4;
    if(gs_sim_probe(GS_SIM_ARMOUR,"return {entity=7,damage=12,knockback=30}",&c->ops[2])) rc|=8;
    if(!gs_sim_probe(GS_SIM_ARMOUR,"return {entity=7,damage=-1}",&parsed)) rc|=8;
    if(gs_sim_probe(GS_SIM_TIMED,"return {entity=7,channel=2,value=0.5,frames=60}",&c->ops[3])) rc|=16;
    if(!gs_sim_probe(GS_SIM_TIMED,"return {entity=7,channel=5,value=1,frames=1}",&parsed)) rc|=16;
    if(gs_sim_probe(GS_SIM_ARMOR,"return {entity=7,type='knockback',value=2,frames=120}",&c->ops[4]) || c->ops[4].eff[0]!=1) rc|=32;
    if(!gs_sim_probe(GS_SIM_ARMOR,"return {entity=7,type='knockback',value=-2}",&parsed)) rc|=32;
    if(!gs_sim_probe(GS_SIM_ARMOR,"return {entity=7,type='typo',value=2}",&parsed)) rc|=32;
    if(!gs_sim_probe(GS_SIM_ARMOR,"return {entity=7,type='super',value=2,from=9,to=3}",&parsed)) rc|=32;
    for(i=0;i<5;++i) c->ops[i].kind=GS_SIM_CAPS+i;
    c->count=5;
    /* replay: apply, wipe as a keyframe restore would, apply again, compare */
    gs_caps_release(owner);
    gs_sim_apply_caps(c,&c->ops[0]);
    for(i=1;i<c->count;++i) gs_sim_apply_caps(c,&c->ops[i]);
    gs_sim_caps_readback(7,before);
    /* Only the movement record is BSS-only: effects, armour and timers need a live fighter, so their replay
       equality is a LAB check (see the spine report); here they prove the op is parsed and dispatched. */
    if(before[0]!=1 || before[1]!=3 || before[2]!=4) rc|=64; /* the movement write took effect */
    gs_caps_release(owner);
    gs_sim_caps_readback(7,after);
    if(!memcmp(before,after,sizeof before)) rc|=128; /* wiping really cleared them */
    for(i=0;i<c->count;++i) gs_sim_apply_caps(c,&c->ops[i]);
    gs_sim_caps_readback(7,after);
    if(memcmp(before,after,sizeof before)) rc|=256; /* zero difference after replay */
    gs_caps_release(owner); free(c);
    return rc;
}
static int test_script_sim_state(void) {
    static const unsigned char payload[]={0,1,255,'x'};
    char out[512]; int rc;
    gs_sim_input=payload; gs_sim_input_size=sizeof payload;
    if(!gw_ScriptGame_SimCommit(INT_MAX-1,sizeof payload) ||
       gw_ScriptGame_SimSize(INT_MAX-1)!=(int)sizeof payload ||
       gw_ScriptGame_SimByte(INT_MAX-1,2)!=255 ||
       gw_ScriptGame_SimSize(INT_MAX-2)!=-1 ||
       gw_ScriptGame_SimCommit(INT_MAX-1,GS_SIM_BYTES+1)) {
        gs_sim_input=NULL; gs_sim_input_size=0; gw_ScriptGame_SimRelease(INT_MAX-1);
        gw_test_fail("script simulation checkpoint bounds or generation isolation"); return 1;
    }
    gs_sim_input_size=1;
    gw_ScriptGame_SimCommit(INT_MAX-1,1);
    if(gw_ScriptGame_SimSize(INT_MAX-1)!=1 || gw_ScriptGame_SimByte(INT_MAX-1,2)!=0) {
        gs_sim_input=NULL; gs_sim_input_size=0; gw_ScriptGame_SimRelease(INT_MAX-1);
        gw_test_fail("script simulation checkpoint replacement"); return 1;
    }
    gs_sim_input=NULL; gs_sim_input_size=0; gw_ScriptGame_SimRelease(INT_MAX-1);
    rc=t_exec("assert(gd.sim_supported and type(gd.sim_read)=='function'); "
              "assert(type(gd.sim_replaying())=='boolean' and type(gd.sim_clear)=='function'); "
              "local ok,e=pcall(gd.sim_commit,'test',{}); "
              "assert(not ok and tostring(e):find('on_frame',1,true))",out,sizeof out);
    if(rc) { gw_test_fail("script simulation API frame guard: %s",out); return 1; }
    rc=test_sim_capability_ops();
    if(rc) { gw_test_fail("script simulation capability/armour/timer ops parse or replay mismatch: mask %d",rc); return 1; }
    return 0;
}
