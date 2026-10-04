#include <assert.h>
#include <math.h>
#include <stdio.h>
#include "../platform/gw_stage_cover.h"
int main(void) {
    int i;float x;
    for(i=1;i<100;++i){x=i/100.0f;
        assert(gw_stage_cover_amount(0,0,x)==0);
        assert(gw_stage_cover_amount(0,.5f,x)==1);
        assert(gw_stage_cover_amount(0,1,x)==0);
    }
    assert(gw_stage_cover_amount(0,.25f,.25f)==1);
    assert(gw_stage_cover_amount(0,.25f,.75f)==0);
    assert(gw_stage_cover_amount(0,.75f,.25f)==0);
    assert(gw_stage_cover_amount(0,.75f,.75f)==1);
    assert(gw_stage_cover_amount(1,0,0)==0);
    assert(gw_stage_cover_amount(1,.5f,0)==1);
    assert(gw_stage_cover_amount(1,1,0)==0);
    assert(gw_stage_cover_amount(1,.25f,0)==.5f);
    puts("stage cover: directional entry/exit, opaque midpoint, flash envelope passed");
    return 0;
}
