/* Included by shim_pad.c after the physical-profile implementation. Persistent
 * menu preferences and frozen match mappings are deliberately separate: Apply
 * only reads this snapshotted table, never current settings/devices/registry. */
#include "gw_fighter_controls_model.h"
#include "gw_mexid.h"
#include "gw_fighter_controls_profiles.inc"
#include <errno.h>
extern void gw_Settings_SetInt(const char*, int);
extern int gw_Settings_LastSaveOk(void);

typedef struct GwFighterControlsFrozen {
    int active, ck;
    unsigned choice;
    unsigned short map[GW_FC_BUTTONS];
} GwFighterControlsFrozen;
static struct {
    unsigned magic, version;
    GwFighterControlsFrozen port[4];
} fc_match={0x4643544cu,1u,{{0}}};

static int fc_ck_valid(int ck) { return ck>=0 && ck<128 && ck!=33; }
static int fc_preference_key(int port, int ck, char key[32])
{
    int device, profile, index;
    char identity[20];
    if(port<0 || port>=4 || !fc_ck_valid(ck)) return 0;
    index=gw_MexId_FindFighter(ck);
    if(index>=0) snprintf(identity,sizeof identity,"%016llx",(unsigned long long)gw_MexId_Hash(index));
    else if(ck<33) snprintf(identity,sizeof identity,"retail%02d",ck);
    else return 0; /* Never save a roster-order-dependent custom alias. */
    ctl_fighter_profile(port,&device,&profile);
    if(device>=0) snprintf(key,32,"fc_d%d_p%d_s%d_%s",device,profile,port,identity);
    else snprintf(key,32,"fc_f%d_p%d_s%d_%s",port,profile,port,identity);
    return 1;
}
unsigned gw_FighterControls_Normalize(int ck, unsigned choice)
{ return fc_ck_valid(ck) ? gw_fc_normalize(gw_Geno_ControlsSchema(ck),choice) : 0; }
unsigned gw_FighterControls_Get(int port, int ck)
{
    char key[32], value[32], *end;
    unsigned long saved;
    if(!fc_preference_key(port,ck,key)) return 0;
    gw_Settings_Str(key,value,sizeof value,"");
    if(!value[0] || value[0]=='-' || value[0]=='+') return 0;
    errno=0; saved=strtoul(value,&end,10);
    if(errno || *end || saved>0xfffffffful) return 0;
    return gw_FighterControls_Normalize(ck,(unsigned)saved);
}
int gw_FighterControls_Set(int port, int ck, unsigned choice)
{
    char key[32];
    if(!fc_preference_key(port,ck,key)) return 0;
    choice=gw_FighterControls_Normalize(ck,choice);
    gw_Settings_SetInt(key,(int)choice);
    if(!gw_Settings_LastSaveOk()) { gw_log("fighter controls: P%d preference save failed",port+1); return 0; }
    gw_log("fighter controls: saved P%d ck %d choice %04x",port+1,ck,choice);
    return 1;
}
int gw_FighterControls_Count(int ck, int field)
{
    const GwFighterControlsSchema* s=fc_ck_valid(ck) ? gw_Geno_ControlsSchema(ck) : NULL;
    return !s ? 0 : field==GW_FC_LAYOUTS ? s->layouts : field==GW_FC_EXTRAS ? s->extras : 0;
}
void gw_FighterControls_Name(int ck, int field, int index, char* out, int cap)
{
    const GwFighterControlsSchema* s=fc_ck_valid(ck) ? gw_Geno_ControlsSchema(ck) : NULL;
    const char* name="";
    if(!out || cap<=0) return;
    if(s && index>=0) {
        if(field==GW_FC_LAYOUTS && index<s->layouts) name=s->layout[index].name;
        if(field==GW_FC_EXTRAS && index<s->extras) name=s->extra[index].name;
    }
    snprintf(out,(size_t)cap,"%s",name);
}
void gw_FighterControls_MatchSet(int port, int ck, unsigned choice)
{
    GwFighterControlsFrozen* f;
    const GwFighterControlsSchema* schema;
    if(port<0 || port>=4) return;
    f=&fc_match.port[port]; memset(f,0,sizeof *f); f->ck=-1;
    if(!fc_ck_valid(ck)) return;
    schema=gw_Geno_ControlsSchema(ck);
    f->active=1; f->ck=ck; f->choice=gw_fc_normalize(schema,choice);
    gw_fc_build(schema,f->choice,f->map);
    gw_log("fighter controls: frozen P%d ck %d choice %04x",port+1,ck,f->choice);
}
unsigned gw_FighterControls_MatchGet(int port)
{ return port>=0 && port<4 && fc_match.port[port].active ? fc_match.port[port].choice : 0; }
int gw_FighterControls_MatchCK(int port)
{ return port>=0 && port<4 && fc_match.port[port].active ? fc_match.port[port].ck : -1; }
void gw_FighterControls_MatchClear(void)
{
    int i;
    for(i=0;i<4;++i) { memset(&fc_match.port[i],0,sizeof fc_match.port[i]); fc_match.port[i].ck=-1; }
    gw_log("fighter controls: match mapping cleared");
}
int gw_FighterControls_Apply(int port, int buttons)
{
    if(port<0 || port>=4 || !fc_match.port[port].active) return buttons;
    return gw_fc_apply(fc_match.port[port].map,buttons);
}
int gw_FighterControls_StateSize(void) { return (int)sizeof fc_match; }
int gw_FighterControls_StateSave(void* out, int size)
{
    if(!out || size!=(int)sizeof fc_match) return 0;
    memcpy(out,&fc_match,sizeof fc_match); return 1;
}
int gw_FighterControls_StateLoad(const void* in, int size)
{
    unsigned magic, version;
    GwFighterControlsFrozen ports[4];
    int p,i;
    if(!in || size!=(int)sizeof fc_match) return 0;
    memcpy(&magic,in,4); memcpy(&version,(const unsigned char*)in+4,4);
    if(magic!=fc_match.magic || version!=fc_match.version) return 0;
    memcpy(ports,(const unsigned char*)in+8,sizeof ports);
    for(p=0;p<4;++p) {
        const GwFighterControlsFrozen* f=&ports[p];
        if(f->active!=0 && f->active!=1) return 0;
        if(f->active && (!fc_ck_valid(f->ck) || (f->choice&~GW_FC_CHOICE_MASK) ||
                        (f->choice&GW_FC_JUMP_MASK)>GW_FC_JUMP_Y)) return 0;
        for(i=0;i<GW_FC_BUTTONS;++i) if(f->map[i]&~GW_FC_BUTTON_MASK) return 0;
    }
    memcpy(&fc_match,in,sizeof fc_match); return 1;
}
