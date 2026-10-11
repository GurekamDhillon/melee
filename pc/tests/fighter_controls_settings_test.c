/* Repeated profile reclamation against the REAL fixed512-entry settings store.
 * The explicit test file is private; no user settings, game, or window. */
#define _CRT_SECURE_NO_WARNINGS
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../platform/gw_settings.c"
void gw_log(const char* fmt, ...) { (void)fmt; }
void gw_test_fail(const char* fmt, ...) { (void)fmt; }
void gw_test_register(const char* name, gw_test_fn fn) { (void)name; (void)fn; }
int gw_View_DefaultWide(void) { return 0; }
int gw_TextEntryUntil;
#include "../platform/gw_fighter_controls_profiles.inc"
#include "../platform/gw_fighter_controls_model.h"
const GwFighterControlsSchema* gw_Geno_ControlsSchema(int ck) { (void)ck; return NULL; }
int gw_MexId_FindFighter(int ck) { (void)ck; return -1; }
uint64_t gw_MexId_Hash(int index) { (void)index; return 0; }
static void ctl_fighter_profile(int port, int* device, int* profile)
{ *device=port; *profile=0; }
#include "../platform/gw_fighter_controls_runtime.inc"
/* Report failures without a CRT assertion dialog on the owner's desktop. */
#undef assert
#define assert(condition) do { if (!(condition)) { \
    fprintf(stderr,"FAIL line %d: %s\n",__LINE__,#condition); return 1; \
} } while (0)

int main(void)
{
    const char* path=".omo/evidence/fighter-controls-settings.cfg";
    char key[32], value[16];
    int i, cycle;
    memset(&st,0,sizeof st); st.loaded=1;
    snprintf(st.path,sizeof st.path,"%s",path);
    /* A failed preference update must remain consistent with the CSS's old
     * displayed choice; a failed new key must not consume table capacity. */
    assert(gw_FighterControls_Get(0,8)==GW_FC_TAP_ON);
    assert(gw_FighterControls_Set(0,8,GW_FC_TAP_OFF) && st.n==1);
    snprintf(st.path,sizeof st.path,".omo/evidence/nonexistent-profile-save-parent/settings.cfg");
    assert(!gw_FighterControls_Set(0,8,GW_FC_TAP_ON));
    assert(gw_FighterControls_Get(0,8)==GW_FC_TAP_OFF && st.n==1);
    assert(!gw_FighterControls_Set(1,8,GW_FC_TAP_OFF));
    assert(gw_FighterControls_Get(1,8)==GW_FC_TAP_ON && st.n==1);
    snprintf(st.path,sizeof st.path,"%s",path);
    gw_Settings_SetInt("transaction_probe",99);
    assert(gw_Settings_LastSaveOk());
    memset(&st,0,sizeof st); snprintf(st.path,sizeof st.path,"%s",path);
    assert(gw_FighterControls_Get(0,8)==GW_FC_TAP_OFF && st.n==2);
    assert(gw_FighterControls_Get(1,8)==GW_FC_TAP_ON && st.n==2);
    gw_FighterControls_MatchSet(0,8,gw_FighterControls_Get(0,8));
    assert(gw_FighterControls_Apply(0,0x800)==0x800 && gw_FighterControls_TapJumpOff(0));
    gw_FighterControls_MatchClear();
    assert(gw_Settings_RemovePrefix("fc_d") && gw_Settings_RemovePrefix("transaction_probe") && st.n==0);
    for(i=0;i<510;++i) {
        snprintf(key,sizeof key,"unrelated%03d",i);
        gw_Settings_SetInt(key,i); assert(gw_Settings_LastSaveOk());
    }
    gw_Settings_SetInt("fc_d0_p1_s0_000000000000aaaa",2);
    assert(st.n==511);
    for(cycle=0;cycle<1024;++cycle) {
        snprintf(key,sizeof key,"fc_d0_p2_s0_%016llx",(unsigned long long)(cycle+1));
        gw_Settings_SetInt(key,1); assert(gw_Settings_LastSaveOk() && st.n==512);
        if(cycle==0) {
            gw_Settings_SetInt("capacity_probe",99); assert(!gw_Settings_LastSaveOk());
        }
        assert(fc_profile_forget(0,2)); /* Actual removal, not a fresh namespace. */
        assert(st.n==511 && gw_Settings_Int(key,0)==0);
        assert(gw_Settings_Int("fc_d0_p1_s0_000000000000aaaa",0)==2);
        assert(gw_Settings_Int("unrelated509",-1)==509);
        if(!(cycle%128)) {
            memset(&st,0,sizeof st); snprintf(st.path,sizeof st.path,"%s",path);
            assert(gw_Settings_Int("unrelated509",-1)==509 && st.n==511);
            assert(!gw_Settings_Str(key,value,sizeof value,""));
        }
    }
    /* Failed deletion keeps the profile's preferences in memory and on the
     * next unrelated save, rather than silently committing the failed removal. */
    snprintf(st.path,sizeof st.path,".omo/evidence/nonexistent-profile-save-parent/settings.cfg");
    assert(!fc_profile_forget(0,1) && !gw_Settings_LastSaveOk());
    assert(st.n==511 && gw_Settings_Int("fc_d0_p1_s0_000000000000aaaa",0)==2);
    assert(gw_Settings_Int("unrelated509",-1)==509);
    snprintf(st.path,sizeof st.path,"%s",path);
    gw_Settings_SetInt("unrelated509",999);
    assert(gw_Settings_LastSaveOk() && st.n==511);
    memset(&st,0,sizeof st); snprintf(st.path,sizeof st.path,"%s",path);
    assert(gw_Settings_Int("fc_d0_p1_s0_000000000000aaaa",0)==2 && st.n==511);
    assert(gw_Settings_Int("unrelated509",-1)==999);
    gw_Settings_SetInt("after_cycles",7);
    assert(gw_Settings_LastSaveOk() && st.n==512);
    assert(gw_Settings_RemovePrefix("after_cycles") && st.n==511);
    assert(!gw_Settings_RemovePrefix(""));
    puts("fighter controls settings: real512-entry store,1024 delete/reuse cycles, reclaim/reload/isolation/failed deletion and preference rollback PASS");
    return 0;
}
