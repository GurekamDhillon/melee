/* Standalone policy test: diagnostic decisions must not alter simulation. */
#include <assert.h>
#include "../platform/gw_hang_policy.h"
#include "../platform/gw_disc_privacy.h"
#include "../platform/gc_adapter_policy.h"
int main(void) {
    GwHangPauseLog pause={0};char clean[256];
    assert(gw_hang_expected(GW_HANG_LOGIC,1,10)==GW_HANG_TRANSITION);
    assert(gw_hang_expected(GW_HANG_MAIN,1,10)==GW_HANG_MAIN);
    assert(gw_hang_expected(GW_HANG_LOGIC,1,30)==GW_HANG_LOGIC);
    assert(gw_hang_pause_log(&pause,1,1)==1);
    assert(gw_hang_pause_log(&pause,1,2)==0);
    gw_hang_pause_log(&pause,0,3);
    assert(gw_hang_pause_log(&pause,1,4)==0);
    assert(gw_hang_pause_log(&pause,0,32)&2);
    assert(pause.episodes==2);
    gw_disc_redact(clean,sizeof clean,"C:/private/a.iso C:\\private\\a.iso C:\\\\private\\\\a.iso","C:\\private\\a.iso");
    assert(!strcmp(clean,"<disc> <disc> <disc>"));
    assert(!gw_gc_policy_allows(1,0,1));
    assert(!gw_gc_policy_allows(0,1,1));
    assert(!gw_gc_policy_allows(0,0,0));
    assert(gw_gc_policy_allows(0,0,1));
    assert(gw_hang_classify(20,20,20,10,0,0,0,0)==GW_HANG_MAIN);
    assert(gw_hang_classify(0,20,20,10,1,0,0,0)==GW_HANG_PAUSED);
    assert(gw_hang_classify(20,20,20,10,1,0,0,0)==GW_HANG_MAIN);
    assert(gw_hang_classify(20,20,20,10,0,1,0,0)==GW_HANG_DEBUGGER);
    assert(gw_hang_classify(20,20,20,10,0,0,1,0)==GW_HANG_MODAL);
    assert(gw_hang_classify(0,20,20,10,0,0,0,1)==GW_HANG_HIDDEN);
    assert(gw_hang_classify(0,20,0,10,0,0,0,0)==GW_HANG_LOGIC);
    assert(gw_hang_classify(0,0,20,10,0,0,0,0)==GW_HANG_PRESENT);
    assert(gw_hang_classify(20,20,20,0,0,0,0,0)==GW_HANG_DISABLED);
    assert(!gw_deadline_expired(4,5,0));
    assert(gw_deadline_expired(5,5,0));
    assert(!gw_deadline_expired(6,5,1));
    return 0;
}
