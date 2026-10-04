#include <assert.h>
#include <stdio.h>
typedef struct{int id;}Ground_GObj;
static Ground_GObj objs[10];static int clips[10][20],modes[10][20],clears[10][20];
static Ground_GObj* Ground_GetMapGObj(int id){assert(id>=4 && id<=8);objs[id].id=id;return &objs[id];}
static void grAnime_801C7FF8(Ground_GObj* g,int joint,int flags,int clip,float frame,float speed){assert(flags==7 && frame==0 && speed==1);clips[g->id][joint]=clip+1;modes[g->id][joint]=1;}
static void grAnime_801C8098(Ground_GObj* g,int joint,int flags,int clip,float frame,float speed){assert(flags==7 && frame==0 && speed==1);clips[g->id][joint]=clip+1;modes[g->id][joint]=2;}
static void grAnime_801C7980(Ground_GObj* g,int joint,int flags){assert(flags==7);clears[g->id][joint]=1;}
/* Actual retail descriptors and do_anime, plus production slot initializer. */
#include "stage_fd_backdrop_retail.inc"
int main(void){int g;Ground_StageSlotFDBackdrop();
    for(g=4;g<=8;++g){assert(clips[g][1]==1 && modes[g][1]==2 && clears[g][1]==1);assert(clips[g][2]==(g==4?1:(g-3)*2+1));}
    assert(clips[4][16]==1);puts("FD backdrop: native case-1 clips for groups 4..8 plus starfield joint 16 passed");return 0;}
