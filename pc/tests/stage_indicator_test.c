#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "../platform/gw_stage_queue.h"
enum{GW_SDRAW_TEXT=1};
typedef struct{int kind;float size,x,y;unsigned rgba;char text[256];}GwScriptDraw;
static int gs_stage_transition_owner;
static struct{GwStageQueueEntry request;}gs_stage_director[2];
static struct{int ndraw[2],build;GwScriptDraw draw[2][4];}gs;
static float gw_Console_ScriptWidth(void){return 640;}
#include "stage_indicator_retail.inc"
int main(void){int i;const GwScriptDraw* d;
    gs.ndraw[1]=1;gs_stage_transition_owner=1;gs_stage_director[0].request.indicator=60;
    for(i=0;i<100;++i){assert(gw_Script_DrawCount()==2);d=gw_Script_DrawAt(1);
        assert(d && d->kind==GW_SDRAW_TEXT && d->size>0 && strstr(d->text,"1.0s"));}
    gs_stage_director[0].request.indicator=0;assert(gw_Script_DrawCount()==1 && !gw_Script_DrawAt(1));
    puts("stage indicator: 100 render queries without any logic draw-list rebuild passed");return 0;}
