/* Offline checkpoint + post-frame effect journal. The Lua VM is never replayed.
 * Records absolute percent, fighter overlays and bounded entity hit-rule/status outputs.
 * Journal allocations are sparse and bounded, not 16KB in every pad log entry. */
#define GS_SIM_BYTES 16384
#define GS_SIM_OPS 24
#define GS_SIM_BUDGET (128u*1024u*1024u)
extern int gw_ScriptGame_SimAvailable(int token);
extern int gw_ScriptGame_SimSize(int token);
extern int gw_ScriptGame_SimByte(int token,int at);
extern int gw_ScriptGame_SimCommit(int token,int size);
extern void gw_ScriptGame_SimRelease(int token);
static int gs_ring_now(void);
static int gs_rw_replaying(int tag);
static void gs_rw_stop(void);
/* kind: 1 fighter_mod, 2 damage, 3 hit_rules, 4 echoes, then the capability writers
 * 5 fighter_caps, 6 fighter_effect, 7 fighter_armour, 8 fighter_armor (typed), 9 timed_status.
 * eff[] holds the exact scalars the setter was called with (floats as bit patterns), so replay
 * calls the same native setter with the same arguments the live call used. */
enum { GS_SIM_CAPS=5, GS_SIM_EFFECT, GS_SIM_ARMOUR, GS_SIM_ARMOR, GS_SIM_TIMED };
/* eff shares storage with bits: an op is one or the other. */
typedef struct { int kind, slot, enabled; union { int bits[11]; int eff[9]; }; GsHitRules hit_rules; GsEchoRules echoes; } GsSimOp;
typedef struct GsSimCommit {
    struct GsSimCommit *next;
    int script, token, size, count;
    size_t allocated;
    unsigned char *blob; /* inside the same allocation, after the ops this commit really has */
    GsSimOp ops[GS_SIM_OPS];
} GsSimCommit;
typedef struct { int tag; GsSimCommit *first, *last; } GsSimFrame;
static GsSimFrame gs_sim_log[GS_LOG_N];
static size_t gs_sim_bytes;
static int gs_sim_replay_mode;
static int l_sim_replaying(lua_State *L) { lua_pushboolean(L,gs_sim_replay_mode); return 1; }
static int gs_sim_window, gs_sim_tag, gs_sim_done[GS_MAX_SCRIPTS];
static const unsigned char *gs_sim_input;
static int gs_sim_input_size;
int gw_Script_SimByte(int at) {
    return gs_sim_input && at>=0 && at<gs_sim_input_size ? gs_sim_input[at] : 0;
}
static void gs_sim_free(GsSimFrame *f) {
    GsSimCommit *p=f->first;
    while(p) { GsSimCommit *n=p->next; gs_sim_bytes-=p->allocated; free(p); p=n; }
    memset(f,0,sizeof *f);
}
static void gs_sim_reset(void) {
    int i;
    for(i=0;i<GS_LOG_N;++i) gs_sim_free(&gs_sim_log[i]);
    gs_sim_window=0;
}
static void gs_sim_release(int token) {
    /* Reload/unload changes the rule implementation: history may not replay a
     * different load generation. Invalidate it before releasing its BSS. */
    if(token>0) {
        int i;
        gs_rw_stop(); gs_sim_reset(); gw_ScriptGame_SimRelease(token);
        for(i=0;i<gs.n;++i) if(gs.s[i].stage_owner==token)
            { gw_ScriptGame_FighterModsRelease(i+1); gw_ScriptGame_HitRulesRelease(i+1); gw_ScriptGame_EchoClear(i+1); gs_echo_visual_release(i+1); }
    }
}
static int l_sim_clear(lua_State *L) {
    GsScript *s=gs_cur_script(); int i, owned=0;
    gs_require_offline(L,"sim_clear");
    if(!s || gs.cur==gs.console || s->stage_owner<=0 || gw_Snap_Resimulating() || gs_sim_replay_mode || gs.rw_resim>0 || gs.rw_resim_run>0)
        return luaL_error(L,"sim_clear requires a gameplay mod outside resimulation");
    /* An explicit edit can run while paused. Standard write branching discards
     * future history and forces the next boundary to snapshot this new state. */
    for(i=0;i<6;++i) if(gw_ScriptGame_FighterModOwner(i)==gs.cur+1) owned=1;
    for(i=0;i<12;++i) if(gw_ScriptGame_HitRulesOwner(i)==gs.cur+1) owned=1;
    for(i=0;i<12;++i) if(gw_ScriptGame_EchoOwner(i)==gs.cur+1) owned=1;
    if(owned || gw_ScriptGame_SimSize(s->stage_owner)>=0) gs_rw_branch();
    gw_ScriptGame_FighterModsRelease(gs.cur+1);
    gw_ScriptGame_HitRulesRelease(gs.cur+1);
    gw_ScriptGame_EchoClear(gs.cur+1);
    gw_ScriptGame_SimRelease(s->stage_owner);
    lua_pushboolean(L,1); return 1;
}
static void gs_sim_apply_caps(const GsSimCommit *c,const GsSimOp *o) {
    int owner=c->script+1, e=o->slot, ok=1;
    /* The same setters the Lua API calls: an ownership refusal is deterministic, so a replay refuses
       exactly where the live write did. */
    switch(o->kind) {
    case GS_SIM_CAPS: ok=gw_ScriptGame_CapsMovement(e,owner,o->eff[0],o->eff[1]); break;
    case GS_SIM_EFFECT: ok=gw_ScriptGame_CapsEffect(e,owner,o->eff[0],o->eff[1],o->eff[2]); break;
    case GS_SIM_ARMOUR: ok=gw_ScriptGame_CapsArmour(e,owner,o->eff[0],o->eff[1]); break;
    case GS_SIM_ARMOR:
        ok=o->eff[8] ? gw_ScriptGame_ArmorClear(e,owner,o->eff[0]) :
            gw_ScriptGame_ArmorSet(e,owner,o->eff[0],o->eff[1],o->eff[2],o->eff[3],o->eff[4],o->eff[5],o->eff[6]);
        break;
    case GS_SIM_TIMED: ok=gw_ScriptGame_CapsStatus(e,o->eff[0],owner,o->eff[1],o->eff[2]); break;
    }
    if(!ok) gw_log("script sim: capability op %d entity %d refused (owner conflict)",o->kind,e);
}
static void gs_sim_apply(GsSimCommit *c) {
    int i;
    for(i=0;i<c->count;++i) {
        GsSimOp *o=&c->ops[i];
        if(o->kind==1) {
            int owner=gw_ScriptGame_FighterModOwner(o->slot), j, same;
            same=o->enabled ? owner==c->script+1 : owner==0;
            if(o->enabled && same) for(j=0;j<11;++j)
                if(gw_ScriptGame_FighterModRead(o->slot,j)!=o->bits[j]) { same=0; break; }
            /* The base setter clamps shield and logs replacements. Stable
             * absolute overlays need neither repeated replacement nor logs. */
            if(same) continue;
            gw_ScriptGame_FighterMod(o->slot,c->script+1,o->enabled,
                o->bits[0],o->bits[1],o->bits[2],o->bits[3],o->bits[4],o->bits[5],o->bits[6],o->bits[7]);
            if(o->enabled) gw_ScriptGame_FighterModExtra(o->slot,c->script+1,o->bits[8],o->bits[9],o->bits[10]);
        } else if(o->kind==3) gs_hr_apply_targets(o->slot,o->enabled,c->script+1,&o->hit_rules);
        else if(o->kind==4) gs_echo_apply(o->slot*2+o->enabled,c->script+1,&o->echoes);
        else if(o->kind>=GS_SIM_CAPS) gs_sim_apply_caps(c,o);
        else gw_ScriptGame_SetPercent(o->slot,o->bits[0]);
    }
    gs_sim_input=c->blob; gs_sim_input_size=c->size;
    gw_ScriptGame_SimCommit(c->token,c->size);
    gs_sim_input=NULL; gs_sim_input_size=0;
}
static void gs_sim_replay(int tag) {
    GsSimFrame *f=&gs_sim_log[(unsigned)tag%GS_LOG_N]; GsSimCommit *c;
    if(f->tag!=tag) return;
    for(c=f->first;c;c=c->next) {
        /* Old snapshots cannot alias a newly loaded script in the same slot. */
        if(c->script>=0 && c->script<gs.n && gs.s[c->script].used &&
           !gs.s[c->script].disabled && gs.s[c->script].stage_owner==c->token)
            gs_sim_apply(c);
    }
}
static void gs_sim_live(int tag) {
    GsSimFrame *f=&gs_sim_log[(unsigned)tag%GS_LOG_N];
    gs_sim_free(f); f->tag=tag;
    gs_sim_tag=tag;
    memset(gs_sim_done,0,sizeof gs_sim_done);
}
static int l_sim_read(lua_State *L) {
    GsScript *s=gs_cur_script(); int n,i; char b[GS_SIM_BYTES];
    if(!s || s->stage_owner<=0 || (n=gw_ScriptGame_SimSize(s->stage_owner))<0) {
        lua_pushnil(L); return 1;
    }
    if(n>GS_SIM_BYTES) return luaL_error(L,"invalid simulation checkpoint size");
    for(i=0;i<n;++i) b[i]=(char)gw_ScriptGame_SimByte(s->stage_owner,i);
    lua_pushlstring(L,b,(size_t)n); return 1;
}
/* Capability writers take an entity 1..12 (not a port) and the bounds of their gd.* setters; the op
 * table is at the top of the stack. */
static int gs_sim_field(lua_State *L,const char *k) { lua_getfield(L,-1,k); return !lua_isnil(L,-1); }
static void gs_sim_parse_caps(lua_State *L,GsSimOp *o) {
    static const char *effect_kinds[]={"intangible","invincible","metal","size",NULL};
    static const char *caps_keys[]={"shield","air_dodge","run","grab","specials"};
    int i;
    lua_getfield(L,-1,"entity"); o->slot=gs_caps_entity(L,-1); lua_pop(L,1);
    if(o->kind==GS_SIM_CAPS) {
        int jumps=-1,flags=0;
        if(gs_sim_field(L,"values")) { /* absent values clear, like gd.fighter_caps(entity, nil) */
            luaL_checktype(L,-1,LUA_TTABLE); lua_pushnil(L);
            while(lua_next(L,-2)) {
                const char *key=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
                if(key && !strcmp(key,"air_jumps")) { lua_Integer n=luaL_checkinteger(L,-1); if(n<0||n>8) luaL_error(L,"air_jumps must be 0-8"); jumps=(int)n; }
                else { for(i=0;i<5;++i) if(key && !strcmp(key,caps_keys[i])) break;
                    if(i==5) luaL_error(L,"unknown fighter capability");
                    luaL_checktype(L,-1,LUA_TBOOLEAN); if(lua_toboolean(L,-1)) flags|=1<<i; }
                lua_pop(L,1);
            }
        }
        lua_pop(L,1); o->eff[0]=jumps; o->eff[1]=flags;
    } else if(o->kind==GS_SIM_EFFECT) {
        int kind; float value;
        lua_getfield(L,-1,"effect"); kind=luaL_checkoption(L,-1,NULL,effect_kinds); lua_pop(L,1);
        lua_getfield(L,-1,"value"); value=gs_caps_number(L,-1,kind==3 ? .25 : 0,kind==3 ? 4 : 1); lua_pop(L,1);
        lua_getfield(L,-1,"frames"); o->eff[2]=gs_caps_frames(L,-1); lua_pop(L,1);
        o->eff[0]=kind; o->eff[1]=gs_fbits(value);
    } else if(o->kind==GS_SIM_ARMOUR) {
        float damage=0,kb=0;
        if(gs_sim_field(L,"damage")) damage=gs_caps_number(L,-1,0,1000); lua_pop(L,1);
        if(gs_sim_field(L,"knockback")) kb=gs_caps_number(L,-1,0,1000); lua_pop(L,1);
        o->eff[0]=gs_fbits(damage); o->eff[1]=gs_fbits(kb);
    } else if(o->kind==GS_SIM_ARMOR) {
        double value=1; float from=-1,to=-1; int type,has_value=0,state=-1,frames=0,direction=0,clear=0;
        lua_getfield(L,-1,"type"); type=luaL_checkoption(L,-1,NULL,gs_armor_types)+1; lua_pop(L,1);
        if(gs_sim_field(L,"value")) { value=luaL_checknumber(L,-1); if(!isfinite(value)||value<0||value>1000) luaL_error(L,"armour value outside range"); has_value=1; } lua_pop(L,1);
        if(gs_sim_field(L,"frames")) frames=gs_armor_integer(L,-1,0,36000); lua_pop(L,1);
        if(gs_sim_field(L,"state")) state=gs_armor_integer(L,-1,-1,65535); lua_pop(L,1);
        if(gs_sim_field(L,"from")) from=gs_armor_bound(L,-1); lua_pop(L,1);
        if(gs_sim_field(L,"to")) to=gs_armor_bound(L,-1); lua_pop(L,1);
        if(gs_sim_field(L,"direction")) { static const char *const names[]={"any","front","back",NULL}; int n=luaL_checkoption(L,-1,NULL,names); direction=n==1?1:n==2?-1:0; } lua_pop(L,1);
        if(gs_sim_field(L,"clear")) { luaL_checktype(L,-1,LUA_TBOOLEAN); clear=lua_toboolean(L,-1); } lua_pop(L,1);
        if(from>=0 && to>=0 && from>to) luaL_error(L,"armour animation range is reversed");
        if(!clear) {
            if(!has_value && type!=4) luaL_error(L,"armour value required");
            if(type!=4 && value<=0) luaL_error(L,"armour value must be positive");
            if(type==5 && (value>255 || value!=(int)value)) luaL_error(L,"hit_count value must be integer 1-255");
        }
        o->eff[0]=type; o->eff[1]=gs_fbits((float)value); o->eff[2]=frames; o->eff[3]=state;
        o->eff[4]=gs_fbits(from); o->eff[5]=gs_fbits(to); o->eff[6]=direction; o->eff[8]=clear;
    } else { /* GS_SIM_TIMED */
        lua_Integer c; float value;
        lua_getfield(L,-1,"channel"); c=luaL_checkinteger(L,-1); lua_pop(L,1);
        if(c<1||c>4) luaL_error(L,"fighter status channel must be 1-4");
        lua_getfield(L,-1,"value"); value=gs_caps_number(L,-1,-100000,100000); lua_pop(L,1);
        lua_getfield(L,-1,"frames"); o->eff[2]=gs_caps_frames(L,-1); lua_pop(L,1);
        o->eff[0]=(int)c; o->eff[1]=gs_fbits(value);
    }
}
static int l_sim_commit(lua_State *L) {
    static const char *keys[11]={"damage_dealt","damage_taken","run_speed","air_speed","shield_max","jump_height","air_jump_height","knockback_taken","fall_speed","weight","shield_regen"};
    GsSimCommit draft,*c; GsSimFrame *f; GsScript *s=gs_cur_script();
    const char *blob; size_t size,allocated,n; int i,j,echo_next=gw_ScriptGame_EchoNextHandle();
    gs_require_offline(L,"sim_commit");
    if(!gs_sim_window || !gs.match_active || gw_Snap_Resimulating() || gs.cur==gs.console || !s || s->stage_owner<=0)
        return luaL_error(L,"sim_commit requires a gameplay mod on_frame in an offline match");
    if(gs_sim_done[gs.cur]) return luaL_error(L,"sim_commit allows one checkpoint per on_frame");
    blob=luaL_checklstring(L,1,&size);
    if(size>GS_SIM_BYTES) return luaL_error(L,"sim_commit checkpoint exceeds 16384 bytes");
    luaL_checktype(L,2,LUA_TTABLE); n=lua_rawlen(L,2);
    if(n>GS_SIM_OPS) return luaL_error(L,"sim_commit exceeds 24 operations");
    memset(&draft,0,sizeof draft); draft.script=gs.cur; draft.token=s->stage_owner;
    draft.size=(int)size; draft.count=(int)n;
    for(i=0;i<(int)n;++i) {
        GsSimOp *o=&draft.ops[i]; const char *name; int owner;
        lua_rawgeti(L,2,i+1); luaL_checktype(L,-1,LUA_TTABLE);
        lua_getfield(L,-1,"op"); name=luaL_checkstring(L,-1);
        o->kind=!strcmp(name,"fighter_mod") ? 1 : !strcmp(name,"damage") ? 2 : !strcmp(name,"hit_rules") ? 3 : !strcmp(name,"echoes") ? 4 :
            !strcmp(name,"fighter_caps") ? GS_SIM_CAPS : !strcmp(name,"fighter_effect") ? GS_SIM_EFFECT : !strcmp(name,"fighter_armour") ? GS_SIM_ARMOUR :
            !strcmp(name,"fighter_armor") ? GS_SIM_ARMOR : !strcmp(name,"timed_status") ? GS_SIM_TIMED : 0; lua_pop(L,1);
        if(!o->kind) return luaL_error(L,"sim_commit unsupported operation");
        if(o->kind>=GS_SIM_CAPS) { gs_sim_parse_caps(L,o); lua_pop(L,1); continue; }
        lua_getfield(L,-1,"port"); o->slot=gs_slot_arg(L,-1); lua_pop(L,1);
        if(o->kind==1) {
            owner=gw_ScriptGame_FighterModOwner(o->slot);
            if(owner && owner!=gs.cur+1) return luaL_error(L,"fighter modifiers owned by another script");
            lua_getfield(L,-1,"values"); o->enabled=!lua_isnil(L,-1);
            for(j=0;j<11;++j) o->bits[j]=gs_fbits(1.0f);
            if(o->enabled) {
                int idx=lua_absindex(L,-1); luaL_checktype(L,idx,LUA_TTABLE); lua_pushnil(L);
                while(lua_next(L,idx)) {
                    const char *key=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
                    double value=luaL_checknumber(L,-1);
                    for(j=0;j<11;++j) if(key && !strcmp(key,keys[j])) break;
                    if(j==11 || !isfinite(value) || value<0.1 || value>4)
                        return luaL_error(L,"sim_commit invalid fighter modifier");
                    o->bits[j]=gs_fbits((float)value); lua_pop(L,1);
                }
            }
            lua_pop(L,1);
        } else if(o->kind==3) {
            lua_getfield(L,-1,"sub");o->enabled=gs_hr_selector(L,-1);lua_pop(L,1);
            gs_hr_targets_owner(L,o->slot,o->enabled); gs_hr_parse(L,-1,&o->hit_rules);
        } else if(o->kind==4) {
            lua_getfield(L,-1,"sub");o->enabled=gs_echo_sub(L,-1);lua_pop(L,1);
            gs_echo_parse(L,-1,&o->echoes);
            gs_echo_prepare(L,o->slot*2+o->enabled,&o->echoes,&echo_next);
            for(j=0;j<i;++j) if(draft.ops[j].kind==4)
                gs_echo_pair(L,o->slot*2+o->enabled,&o->echoes,
                    draft.ops[j].slot*2+draft.ops[j].enabled,&draft.ops[j].echoes);
        } else {
            double value; lua_getfield(L,-1,"value"); value=luaL_checknumber(L,-1); lua_pop(L,1);
            if(!isfinite(value) || value<0 || value>999) return luaL_error(L,"sim_commit damage must be 0..999");
            o->bits[0]=(int)value;
        }
        lua_pop(L,1);
    }
    if(!gw_ScriptGame_SimAvailable(s->stage_owner)) return luaL_error(L,"sim_commit checkpoint owner capacity (8) exhausted");
    /* Only the ops a commit really carries are allocated (one GsSimOp is ~2.4 KB, mostly the hit-rule
     * table): the journal keeps GS_LOG_N frames, and the full 24-op array per frame overran the 128 MB
     * budget after about 30 seconds of continuous commits. */
    allocated=offsetof(GsSimCommit,ops)+(size_t)n*sizeof(GsSimOp)+size;
    if(allocated>GS_SIM_BUDGET-gs_sim_bytes) return luaL_error(L,"sim_commit journal memory budget exhausted");
    c=(GsSimCommit*)malloc(allocated);
    if(!c) return luaL_error(L,"sim_commit journal allocation failed");
    memcpy(c,&draft,offsetof(GsSimCommit,ops)+(size_t)n*sizeof(GsSimOp)); c->allocated=allocated;
    c->blob=(unsigned char*)c+offsetof(GsSimCommit,ops)+(size_t)n*sizeof(GsSimOp); memcpy(c->blob,blob,size);
    /* A live write after a seek forks future history, but ordinary post-frame
     * commits do not force a giant keyframe every logic frame. Internal replay
     * never calls this function or gs_rw_branch. */
    if(gs.rw_began && gs.rw_head>gs_ring_now()) gs_rw_branch();
    f=&gs_sim_log[(unsigned)gs_sim_tag%GS_LOG_N];
    if(f->last) f->last->next=c; else f->first=c; f->last=c;
    gs_sim_bytes+=allocated; gs_sim_done[gs.cur]=1;
    gs_sim_apply(c); lua_pushboolean(L,1); return 1;
}
