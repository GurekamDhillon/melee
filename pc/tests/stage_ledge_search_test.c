/* Private archives arrive on stdin; no extracted/disc fixture is persisted.
 * The harness extracts the CURRENT retail search/grab functions into an ignored
 * include, so this checks the game algorithm rather than a copied approximation. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <float.h>
#include <fcntl.h>
#include <io.h>
#include "../gameworld/script_stage_dat.h"
typedef unsigned u32;typedef unsigned char u8;typedef int bool;
typedef struct {float x,y,z;} Vec3;
typedef struct {int start,count;} Range;
typedef struct {Range ranges[5];} MapJoint;
typedef struct {unsigned lo_flags;int v0_idx,v1_idx;} MapLine;
typedef struct {MapLine* x0;unsigned flags;} CollLine;
typedef struct {struct {float x,y;} pos;} CollVtx;
typedef struct CollJoint {struct CollJoint* next;MapJoint* inner;unsigned flags;} CollJoint;
typedef struct {Vec3 prev_pos,cur_pos,contact;float ledge_snap_x,ledge_snap_y,ledge_snap_height;
 int floor_skip,joint_id_skip,joint_id_only;struct{Vec3 right,left,bottom,top;}ecb;} CollData;
#define true 1
#define false 0
#define TARGET_PC
#define F32_MAX FLT_MAX
#define ABS(x) ((x)<0?-(x):(x))
#define HSD_ASSERT(a,b) assert(b)
#define CollLine_Floor 1
#define LINE_FLAG_ENABLED 65536
#define LINE_FLAG_EMPTY 128
#define LINE_FLAG_LEDGE 512
#define CollJoint_TooFar 4096
#define MapLineGroup_Floor 0
#define MapLineGroup_Dynamic 4
static CollLine groundCollLine[768];static CollVtx groundCollVtx[1536];
static CollJoint groundCollJoint[768],*jointListStart;
static MapLine maplines[768];static MapJoint mapjoints[768];static StDatLine lines[768];
static int checked,alias,wall;
static bool mpCheckedBounding(void){return checked;}
static void mpBoundingCheck(float a,float b,float c,float d){(void)a;(void)b;(void)c;(void)d;checked=1;}
static void mpUncheckBounding(void){checked=0;}
static int mpJointFromLine(int line){return line;}
static int ScriptGame_StageSlotSameJoint(int a,int b){return alias?lines[a].joint==lines[b].joint:-1;}
#include "../../src/melee/mp/mpcoll_stage_slots.inc"
static void mpFloorGetLeft(int id,Vec3* p){p->x=lines[id].x0;p->y=lines[id].y0;p->z=0;}
static void mpFloorGetRight(int id,Vec3* p){p->x=lines[id].x1;p->y=lines[id].y1;p->z=0;}
/* Force the adjacent wall occlusion branch of retail grab validation. Search
 * and flag/position selection below are the actual production functions. */
static int mpCheckMultiple(float a,float b,float c,float d,void* e,int* id,void* f,void* g,int mask,int skip,int only)
{(void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;(void)mask;(void)skip;(void)only;*id=wall;return 1;}
#include "stage_ledge_retail.inc"
int main(void)
{
    unsigned char* raw=malloc(8*1024*1024);StDatView v;unsigned n,param;size_t size;int i,ledges=0;float scale;
    _setmode(_fileno(stdin),_O_BINARY);size=fread(raw,1,8*1024*1024,stdin);
    assert(st_dat_open(raw,size,&v)==0 && st_dat_collision(&v,767,&n)==0);
    assert(st_dat_symbol(&v,"grGroundParam",&param)==0);scale=st_dat_float(v.data+param);
    for(i=0;i<(int)n;++i){StDatLine* l=&lines[i];assert(st_dat_line(&v,i,l)==0);
      l->x0*=scale;l->y0*=scale;l->x1*=scale;l->y1*=scale;
      maplines[i]=(MapLine){l->lo,2*i,2*i+1};groundCollLine[i]=(CollLine){&maplines[i],(1u<<(l->kind-1))|LINE_FLAG_ENABLED};
      groundCollVtx[2*i].pos.x=l->x0;groundCollVtx[2*i].pos.y=l->y0;
      groundCollVtx[2*i+1].pos.x=l->x1;groundCollVtx[2*i+1].pos.y=l->y1;
      mapjoints[i].ranges[4]=(Range){i,1};groundCollJoint[i]=(CollJoint){i+1<(int)n?&groundCollJoint[i+1]:NULL,&mapjoints[i],0};}
    jointListStart=groundCollJoint;
    for(i=0;i<(int)n;++i)if(lines[i].kind==1 && (lines[i].lo&LINE_FLAG_LEDGE)){
      int j,id;Vec3 p;CollData cd={0};StDatLine* l=&lines[i];++ledges;
      assert(mpLib_80051BA8_Floor(&p,-1,-1,-1,1,l->x0-2,l->y0-2,l->x0+2,l->y0+2)==i);
      assert(mpLib_80051BA8_Floor(&p,-1,-1,-1,-1,l->x1-2,l->y1-2,l->x1+2,l->y1+2)==i);
      for(j=0;j<(int)n;++j)if(lines[j].joint==l->joint && lines[j].kind>=3)break;
      assert(j<(int)n);wall=j;
      cd.cur_pos=cd.prev_pos=(Vec3){l->x0-1,l->y0-1,0};cd.ledge_snap_x=4;cd.ledge_snap_y=0;cd.ledge_snap_height=4;
      cd.floor_skip=cd.joint_id_skip=cd.joint_id_only=-1;cd.ecb.top.y=2;
      alias=0;assert(!mpColl_80044164(&cd,&id));
      alias=1;assert(mpColl_80044164(&cd,&id) && id==i);
      cd.cur_pos=cd.prev_pos=(Vec3){l->x1+1,l->y1-1,0};
      alias=0;assert(!mpColl_800443C4(&cd,&id));
      alias=1;assert(mpColl_800443C4(&cd,&id) && id==i);
    }
    assert(ledges>=2);printf("real DAT: %d ledge lines; retail search and wall-group grab regression passed\n",ledges);free(raw);return 0;
}
