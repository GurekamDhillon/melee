/* Pure shared test, usable in the native binary and the headless suite. */
#include "gw_fighter_controls_model.h"
static int fighter_controls_model_check(void)
{
    GwFighterControlsSchema s;
    unsigned short map[GW_FC_BUTTONS];
    unsigned raw;
#define FC_CHECK(c) do { if(!(c)) return __LINE__; } while(0)
    for(raw=0;raw<65536u;++raw) {
        gw_fc_build(NULL,0,map); FC_CHECK(gw_fc_apply(map,(int)raw)==(int)raw);
        gw_fc_build(NULL,1,map); FC_CHECK(gw_fc_apply(map,(int)raw)==(int)raw); /* tap jump off maps nothing */
    }
    memset(&s,0,sizeof s); s.layouts=2; s.extras=1;
    s.layout[0].specified=(1u<<0)|(1u<<1)|(1u<<3);
    s.layout[0].dest[0]=0x200; s.layout[0].dest[1]=0x100; s.layout[0].dest[3]=0x200;
    s.extra[0].specified=1u<<3; s.extra[0].dest[3]=0x300;
    gw_fc_build(&s,0u,map); FC_CHECK(gw_fc_apply(map,0x100)==0x200);
    gw_fc_build(&s,1u|4u,map); FC_CHECK(gw_fc_apply(map,0x800)==0x800);
    gw_fc_build(&s,1u,map); FC_CHECK(gw_fc_apply(map,0x800)==0x200);
    FC_CHECK(gw_fc_apply(map,0x100)==0x200 && gw_fc_apply(map,0x200)==0x100);
    gw_fc_build(&s,1u|0x100u,map); FC_CHECK(gw_fc_apply(map,0x800)==0x300);
    FC_CHECK(gw_fc_normalize(&s,0xffffffffu)==0x101u);
    FC_CHECK(gw_fc_normalize(NULL,0xffffffffu)==1u && gw_fc_normalize(NULL,2u)==0);
    FC_CHECK(gw_fc_apply(map,0x80000000u)==(int)0x80000000u);
#undef FC_CHECK
    return 0;
}
