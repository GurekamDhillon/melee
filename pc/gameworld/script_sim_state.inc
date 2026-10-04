/* Opaque script checkpoints live in the game snapshot, never in native Lua.
 * Generation tokens are never reused by the native loader. Scalars only cross
 * the retarget boundary; bytes are copied synchronously through a shim. */
#define SCRIPT_SIM_OWNERS 8
#define SCRIPT_SIM_BYTES 16384
typedef struct { int token, size; unsigned char blob[SCRIPT_SIM_BYTES]; } ScriptSimState;
static ScriptSimState script_sim_state[SCRIPT_SIM_OWNERS];
extern int Script_SimByte(int at);
static int script_sim_slot(int token, int create)
{
    int i, empty=-1;
    if (token<=0) return -1;
    for(i=0;i<SCRIPT_SIM_OWNERS;++i) {
        if(script_sim_state[i].token==token) return i;
        if(!script_sim_state[i].token && empty<0) empty=i;
    }
    return create ? empty : -1;
}
int ScriptGame_SimAvailable(int token) { return script_sim_slot(token,1)>=0; }
int ScriptGame_SimSize(int token)
{
    int i=script_sim_slot(token,0);
    return i<0 ? -1 : script_sim_state[i].size;
}
int ScriptGame_SimByte(int token,int at)
{
    int i=script_sim_slot(token,0);
    return i>=0 && at>=0 && at<script_sim_state[i].size ? script_sim_state[i].blob[at] : 0;
}
int ScriptGame_SimCommit(int token,int size)
{
    int i=script_sim_slot(token,1), j;
    if(i<0 || size<0 || size>SCRIPT_SIM_BYTES) return 0;
    script_sim_state[i].token=token;
    script_sim_state[i].size=size;
    for(j=0;j<size;++j) script_sim_state[i].blob[j]=(unsigned char)Script_SimByte(j);
    /* Canonical tail: a shorter checkpoint cannot leak old bytes into hashes. */
    memset(script_sim_state[i].blob+size,0,SCRIPT_SIM_BYTES-size);
    return 1;
}
void ScriptGame_SimRelease(int token)
{
    int i;
    for(i=0;i<SCRIPT_SIM_OWNERS;++i)
        if(!token || script_sim_state[i].token==token) memset(&script_sim_state[i],0,sizeof script_sim_state[i]);
}
