#include <assert.h>
#include <stdio.h>
#include "../platform/gw_presentation_state.h"
int main(void) {
    GwPresentationTimer t={0};
    assert(!gw_presentation_live(&t,100));
    gw_presentation_start(&t,7,100,71);
    assert(gw_presentation_live(&t,100) && gw_presentation_live(&t,1283));
    assert(!gw_presentation_live(&t,1284));
    assert(gw_presentation_progress(&t,100)==0);
    assert(gw_presentation_progress(&t,2000)==1);
    assert(!gw_presentation_cancel(&t,8));
    assert(gw_presentation_cancel(&t,7));
    assert(!gw_presentation_live(&t,100));
    puts("presentation timer: PASS");return 0;
}
