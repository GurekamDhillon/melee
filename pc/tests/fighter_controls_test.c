/* Real controls schema/model/runtime, synthetic controller identities/settings.
 * No game, disc, SDL, window, or persisted user settings are involved. */
#define _CRT_SECURE_NO_WARNINGS
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "../platform/gw_fighter_controls_model.h"
#include "geno_controls_json_reader.inc"
#include "../platform/geno_controls_schema.inc"

static GwFighterControlsSchema schemas[2];
static int controller_device[4] = {0, 1, -1, -1};
static int controller_profile[4];
static int fighters[2] = {34, 35};
static struct { char key[32], value[32]; } settings[128];
static int setting_count, save_ok = 1, settings_reads;
static void gw_log(const char* fmt, ...) { (void)fmt; }
const GwFighterControlsSchema* gw_Geno_ControlsSchema(int ck)
{
    int i;
    for (i=0; i<2; ++i) if (ck == fighters[i]) return &schemas[i];
    return NULL;
}
int gw_MexId_FindFighter(int ck)
{ int i; for(i=0;i<2;++i) if(ck==fighters[i]) return i; return -1; }
uint64_t gw_MexId_Hash(int index) { return (uint64_t)(0xabc0 + index); }
static void ctl_fighter_profile(int port, int* device, int* profile)
{ *device=controller_device[port]; *profile=controller_profile[port]; }
static int gw_Settings_Str(const char* key, char* out, int cap, const char* fallback)
{
    int i; ++settings_reads;
    for(i=0;i<setting_count;++i) if(!strcmp(settings[i].key,key)) {
        snprintf(out,(size_t)cap,"%s",settings[i].value); return 1;
    }
    snprintf(out,(size_t)cap,"%s",fallback); return 0;
}
static void gw_Settings_SetInt(const char* key, int value)
{
    int i;
    assert(strlen(key)<sizeof settings[0].key);
    for(i=0;i<setting_count;++i) if(!strcmp(settings[i].key,key)) break;
    assert(i<128);
    if(i==setting_count) ++setting_count;
    strcpy(settings[i].key,key);
    snprintf(settings[i].value,sizeof settings[i].value,"%d",value);
}
static int gw_Settings_LastSaveOk(void) { return save_ok; }
int gw_Settings_RemovePrefix(const char* prefix)
{
    int i, out=0; size_t length=strlen(prefix);
    for(i=0;i<setting_count;++i) if(strncmp(settings[i].key,prefix,length)) {
        if(out!=i) settings[out]=settings[i]; ++out;
    }
    setting_count=out; return save_ok;
}
#include "../platform/gw_fighter_controls_profiles.inc"
#include "../platform/gw_fighter_controls_runtime.inc"
#include "../platform/gw_fighter_controls_tests.inc"

static void schema_parse(void)
{
    static jdoc d;
    GwFighterControlsSchema s;
    int root;
    const char* invalid[] = {
        "[]", "{\"layouts\":true}",
        "{\"layouts\":[{\"name\":\"\",\"buttons\":{\"X\":\"A\"}}]}",
        "{\"layouts\":[{\"name\":\"Bad\",\"buttons\":{\"W\":\"A\"}}]}",
        "{\"extras\":[{\"name\":\"Bad\",\"buttons\":{\"Y\":\"boost\"}}]}",
        "{\"extras\":[{\"name\":\"Bad\",\"buttons\":{\"Y\":[]}}]}",
        "{\"extras\":[{\"name\":\"Bad\",\"buttons\":{\"Y\":[\"A\",\"A\"]}}]}",
        "{\"extras\":[{\"name\":\"Same\",\"buttons\":{\"Y\":\"A\"}},{\"name\":\"Same\",\"buttons\":{\"X\":\"B\"}}]}",
        "{\"extras\":[{\"name\":\"Bad\\nName\",\"buttons\":{\"Y\":\"A\"}}]}",
        "{\"unknown\":1}", "{\"extras\":[],\"extras\":[]}"
    };
    unsigned i;
    root=jd_parse(&d,"{\"layouts\":[{\"name\":\"Swap attacks\",\"buttons\":{\"A\":\"B\",\"B\":\"A\",\"Y\":\"B\"}},{\"name\":\"Standard\",\"buttons\":{\"A\":\"A\",\"B\":\"B\"}}],\"extras\":[{\"name\":\"Y boost\",\"buttons\":{\"Y\":[\"A\",\"B\"]}}]}");
    assert(root>=0 && gn_controls_parse(&s,&d,root));
    assert(s.layouts==2 && s.extras==1);
    assert(!strcmp(s.layout[0].name,"Swap attacks"));
    assert(!strcmp(s.layout[1].name,"Standard"));
    schemas[0]=s;
    for(i=0;i<sizeof invalid/sizeof invalid[0];++i) {
        root=jd_parse(&d,invalid[i]); assert(root>=0);
        memset(&s,0xff,sizeof s);
        assert(!gn_controls_parse(&s,&d,root));
        assert(s.layouts==0 && s.extras==0);
    }
    root=jd_parse(&d,"{\"extras\":[{\"name\":\"Consume Y\",\"buttons\":{\"Y\":\"none\"}}]}");
    assert(root>=0 && gn_controls_parse(&s,&d,root));
    assert(s.layouts==0 && s.extras==1 && s.extra[0].dest[3]==0);
    /* The checked-in fixture is parseable and exposes a nonidentity layout0. */
    {
        FILE* f=fopen("pc/tests/fixtures/controls-fighter/geno.json","rb");
        char text[4096]; size_t length; int array, entry, controls;
        assert(f); length=fread(text,1,sizeof text-1,f); fclose(f); text[length]=0;
        root=jd_parse(&d,text); assert(root>=0);
        array=jd_get(&d,root,"fighters"); assert(array>=0);
        entry=d.n[array].first; controls=jd_get(&d,entry,"controls");
        assert(gn_controls_parse(&s,&d,controls));
        assert(s.layouts==2 && s.extras==1 && s.layout[0].dest[0]==0x200);
    }
    /* The packed word accepts all 16 layouts and 8 extras, never silently
     * truncates a schema which could give a saved bit a different meaning. */
    {
        char text[4096]; int count;
        for(count=16;count<=17;++count) {
            int used=snprintf(text,sizeof text,"{\"layouts\":[");
            for(i=0;i<(unsigned)count;++i)
                used+=snprintf(text+used,sizeof text-(size_t)used,"%s{\"name\":\"L%u\",\"buttons\":{\"A\":\"B\"}}",i ? "," : "",i);
            snprintf(text+used,sizeof text-(size_t)used,"]}");
            root=jd_parse(&d,text); assert(root>=0);
            assert(gn_controls_parse(&s,&d,root)==(count==16));
            assert(s.layouts==(count==16 ? 16 : 0));
        }
        for(count=8;count<=9;++count) {
            int used=snprintf(text,sizeof text,"{\"extras\":[");
            for(i=0;i<(unsigned)count;++i)
                used+=snprintf(text+used,sizeof text-(size_t)used,"%s{\"name\":\"E%u\",\"buttons\":{\"Y\":\"A\"}}",i ? "," : "",i);
            snprintf(text+used,sizeof text-(size_t)used,"]}");
            root=jd_parse(&d,text); assert(root>=0);
            assert(gn_controls_parse(&s,&d,root)==(count==8));
            assert(s.extras==(count==8 ? 8 : 0));
        }
    }
}
static void tap_and_explicit_maps(void)
{
    unsigned short map[GW_FC_BUTTONS];
    unsigned bits;
    assert(gw_FighterControls_Count(8,0)==0 && gw_FighterControls_Count(8,1)==0);
    /* A fighter without a `controls` block has exactly one option: Tap jump (bit 0). */
    assert(gw_FighterControls_Normalize(8,0xffff)==GW_FC_TAP_OFF);
    assert(gw_FighterControls_Normalize(8,2)==0);
    for(bits=0;bits<65536u;++bits) {
        /* Tap jump is not a button map: no input is ever changed by it (the game-side gate reads it). */
        gw_FighterControls_MatchSet(0,8,GW_FC_TAP_ON);
        assert(gw_FighterControls_Apply(0,(int)bits)==(int)bits && !gw_FighterControls_TapJumpOff(0));
        gw_FighterControls_MatchSet(0,8,GW_FC_TAP_OFF);
        assert(gw_FighterControls_Apply(0,(int)bits)==(int)bits && gw_FighterControls_TapJumpOff(0));
    }
    assert(!gw_FighterControls_TapJumpOff(-1) && !gw_FighterControls_TapJumpOff(4));
    gw_fc_build(&schemas[0],0u,map); /* First named layout is the fighter default. */
    assert(gw_fc_apply(map,0x100)==0x200);
    gw_fc_build(&schemas[0],1u,map); /* Tap jump off changes no button: layout 0 as before. */
    assert(gw_fc_apply(map,0x800)==0x200);
    assert(gw_fc_apply(map,0x100)==0x200);
    assert(gw_fc_apply(map,0x200)==0x100);
    assert(gw_fc_apply(map,0x400)==0x400);
    gw_fc_build(&schemas[0],1u|4u,map); /* Standard layout: Y stays Y. */
    assert(gw_fc_apply(map,0x800)==0x800);
    gw_fc_build(&schemas[0],1u|0x100u,map);
    assert(gw_fc_apply(map,0x800)==0x300);
    /* Maps read original input once: X->Y,Y->A never cascades. */
    schemas[1]=schemas[0];
    schemas[1].layout[0].dest[2]=0x800;
    schemas[1].layout[0].dest[3]=0x100;
    schemas[1].layout[0].specified|=(1u<<2)|(1u<<3);
    gw_fc_build(&schemas[1],0u,map);
    assert(gw_fc_apply(map,0x400)==0x800);
    assert(gw_fc_apply(map,0xc00)==0x900);
    schemas[1].layout[0].dest[0]=0x400; /* Explicit fighter A->jump. */
    gw_fc_build(&schemas[1],0u,map);
    assert(gw_fc_apply(map,0x100)==0x400);
    assert(gw_FighterControls_Normalize(34,0xffffffffu)==0x101u);
    schemas[1].extras=2;
    schemas[1].extra[1]=schemas[1].extra[0];
    schemas[1].extra[1].dest[3]=0x10;
    gw_fc_build(&schemas[1],0x300u,map);
    assert(gw_fc_apply(map,0x800)==0x10); /* Last enabled extra wins. */
}
static void preference_and_freeze(void)
{
    int before, a=fighters[0], b=fighters[1];
    char label[8];
    unsigned char state[512], broken[512];
    int n=gw_FighterControls_StateSize();
    assert(n>0 && n<=(int)sizeof state);
    /* Tap jump belongs to the player: every fighter, with or without options, shares it. */
    assert(gw_FighterControls_Set(0,a,1));
    assert(gw_FighterControls_Get(0,a)==1 && gw_FighterControls_Get(0,b)==1 && gw_FighterControls_Get(0,8)==1);
    {
        char* value=settings[setting_count-1].value;
        strcpy(value,"bad"); assert(gw_FighterControls_Get(0,a)==0);
        strcpy(value,"42949672960"); assert(gw_FighterControls_Get(0,a)==0);
        strcpy(value,"-1"); assert(gw_FighterControls_Get(0,a)==0);
        strcpy(value,"1 trailing"); assert(gw_FighterControls_Get(0,a)==0);
        strcpy(value,"1");
    }
    /* A layout is the fighter's own: saved per player + fighter. */
    assert(gw_FighterControls_Set(0,a,1u|4u));
    assert(gw_FighterControls_Get(0,a)==(1u|4u) && gw_FighterControls_Get(0,b)==1u && gw_FighterControls_Get(0,8)==1u);
    assert(gw_FighterControls_Set(0,8,1u|4u)); /* No options on ck 8: the layout bits are dropped, tap kept. */
    assert(gw_FighterControls_Get(0,8)==1u);
    assert(gw_FighterControls_Get(1,a)==0 && gw_FighterControls_Get(0,255)==0);
    controller_device[1]=controller_device[0]; /* Identical hardware: slot still isolates. */
    assert(gw_FighterControls_Get(1,a)==0);
    controller_profile[0]=1;
    assert(gw_FighterControls_Get(0,a)==0);
    assert(gw_FighterControls_Set(0,a,4));
    assert(gw_FighterControls_Get(0,a)==4);
    controller_profile[0]=0;
    assert(gw_FighterControls_Get(0,a)==(1u|4u));
    /* Delete a populated profile and reuse its slot: old choices (tap jump and
     * fighter keys alike) may never reappear. New profiles copy physical mapping only. */
    controller_profile[0]=2;
    assert(gw_FighterControls_Set(0,a,1u|0x100u));
    assert(gw_FighterControls_Get(0,a)==(1u|0x100u));
    assert(fc_profile_forget(controller_device[0],2));
    assert(gw_FighterControls_Get(0,a)==0);
    assert(fc_profile_forget(controller_device[0],2)); /* Creation reclaims any unowned old slot. */
    assert(gw_FighterControls_Get(0,a)==0);
    assert(gw_FighterControls_Set(0,a,4));
    assert(gw_FighterControls_Get(0,a)==4);
    controller_profile[0]=0;
    assert(gw_FighterControls_Get(0,a)==(1u|4u)); /* Other profile remains isolated. */
    controller_device[0]=5;
    assert(gw_FighterControls_Get(0,a)==0);
    controller_device[0]=0;
    fighters[0]=b; fighters[1]=a; /* Same fighters at swapped resident aliases: the layout follows identity. */
    assert(gw_FighterControls_Get(0,b)==(1u|4u) && gw_FighterControls_Get(0,a)==1u);
    fighters[0]=a; fighters[1]=b;
    assert(gw_FighterControls_Set(2,8,1)); /* Disconnected/scripted fallback. */
    assert(gw_FighterControls_Get(2,8)==1 && gw_FighterControls_Get(3,8)==0);
    assert(!gw_FighterControls_Set(-1,a,1) && !gw_FighterControls_Set(0,255,1));
    gw_FighterControls_Name(a,0,0,label,sizeof label);
    assert(!strcmp(label,"Swap at"));
    gw_FighterControls_Name(a,0,50,label,sizeof label); assert(!label[0]);
    gw_FighterControls_MatchSet(0,a,1u|0x100u);
    assert(gw_FighterControls_MatchGet(0)==(1u|0x100u) && gw_FighterControls_TapJumpOff(0));
    assert(gw_FighterControls_MatchCK(0)==a);
    assert(gw_FighterControls_StateSave(state,n));
    assert(!gw_FighterControls_StateSave(state,n-1));
    before=settings_reads;
    memset(&schemas[0],0,sizeof schemas[0]); /* Registry reload cannot alter active mapping. */
    controller_profile[0]=2;
    assert(gw_FighterControls_Apply(0,0x800)==0x300);
    assert(settings_reads==before);
    gw_FighterControls_MatchClear();
    assert(gw_FighterControls_Apply(0,0x800)==0x800 && !gw_FighterControls_TapJumpOff(0));
    assert(gw_FighterControls_MatchCK(0)==-1);
    assert(gw_FighterControls_StateLoad(state,n));
    assert(gw_FighterControls_Apply(0,0x800)==0x300 && gw_FighterControls_TapJumpOff(0));
    memcpy(broken,state,(size_t)n); memset(broken,0xff,8);
    assert(!gw_FighterControls_StateLoad(broken,n));
    assert(gw_FighterControls_Apply(0,0x800)==0x300);
    {
        GwFighterControlsFrozen f;
        memcpy(&f,state+8,sizeof f); f.choice=3; /* bit 1 is reserved */
        memcpy(broken,state,(size_t)n); memcpy(broken+8,&f,sizeof f);
        assert(!gw_FighterControls_StateLoad(broken,n));
        memcpy(&f,state+8,sizeof f); f.map[0]=0x80;
        memcpy(broken+8,&f,sizeof f); assert(!gw_FighterControls_StateLoad(broken,n));
        memcpy(&f,state+8,sizeof f); f.ck=33;
        memcpy(broken+8,&f,sizeof f); assert(!gw_FighterControls_StateLoad(broken,n));
        assert(gw_FighterControls_Apply(0,0x800)==0x300);
    }
    assert(!gw_FighterControls_StateLoad(state,n-1));
    save_ok=0; assert(!gw_FighterControls_Set(0,8,1));
}
int main(void)
{
    assert(!fighter_controls_model_check());
    schema_parse(); tap_and_explicit_maps(); preference_and_freeze();
    puts("fighter controls schema/mapping/profile/frozen snapshot: PASS");
    return 0;
}
