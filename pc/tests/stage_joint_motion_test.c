/* Extracted current retail joint update; a fake animated matrix drives it. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
typedef unsigned u32;typedef unsigned char u8;typedef int s32;typedef int bool;
typedef float Mtx[3][4];typedef float (*MtxPtr)[4];
typedef struct{float x,y,z;}Vec3;
typedef struct HSD_JObj{Mtx mtx;unsigned flags;struct HSD_JObj* child;}HSD_JObj;
typedef struct{float x0,x4;struct{float x,y;}pos;float x10,x14;}CollVtx;
typedef struct{int vtx_start,vtx_count;float left_bound,right_bound,bottom_bound,top_bound;}MapJoint;
typedef struct{MapJoint* inner;HSD_JObj* x20;unsigned flags;int xE;Vec3 bounding_min,bounding_max;}CollJoint;
#define true 1
#define false 0
#define PAD_STACK(x)
#define JOBJ_HIDDEN 16
#define CollJoint_B8 256
#define CollJoint_B9 512
#define CollJoint_B10 1024
#define CollJoint_B11 2048
#define CollJoint_Enabled 65536
#define CollJoint_Hidden 262144
static CollJoint groundCollJoint[16];static CollVtx groundCollVtx[32];
static int mpColl_804D64AC,mpLib_804D64CC,updates;
static unsigned HSD_JObjGetFlags(HSD_JObj* j){return j->flags;}
static void HSD_JObjSetupMatrix(HSD_JObj* j){(void)j;}
static MtxPtr HSD_JObjGetMtxPtr(HSD_JObj* j){return j->mtx;}
static void PSMTXMultVec(Mtx m,Vec3* p,Vec3* out){Vec3 v=*p;out->x=m[0][0]*v.x+m[0][1]*v.y+m[0][2]*v.z+m[0][3];out->y=m[1][0]*v.x+m[1][1]*v.y+m[1][2]*v.z+m[1][3];out->z=0;}
static void mpJointHide(int id){groundCollJoint[id].flags|=CollJoint_Hidden;}
static void mpJointUnhide(int id){groundCollJoint[id].flags&=~CollJoint_Hidden;}
static void mpJointUpdateDynamics(int id){(void)id;++updates;}
static void mpIsland_8005B334(int a,int b,int c,int d){(void)a;(void)b;(void)c;(void)d;}
#include "stage_joint_retail.inc"
int main(void)
{
    MapJoint map={0,2,-10,10,0,0};HSD_JObj joint={0};
    joint.mtx[0][0]=joint.mtx[1][1]=joint.mtx[2][2]=1;
    groundCollJoint[0]=(CollJoint){&map,&joint,CollJoint_Enabled};
    groundCollVtx[0].x0=-10;groundCollVtx[1].x0=10;
    groundCollVtx[0].pos.x=-10;groundCollVtx[1].pos.x=10;
    joint.mtx[0][3]=3;joint.mtx[1][3]=5;mpLib_80055E9C(0);
    assert(groundCollVtx[0].pos.x==-7 && groundCollVtx[1].pos.x==13 && groundCollVtx[0].pos.y==5);
    assert(groundCollVtx[0].x10==-10 && groundCollVtx[0].x14==0); /* carry delta */
    mpLib_80055E9C(0);assert(groundCollVtx[0].x10==-7 && groundCollVtx[0].x14==5); /* next frame, zero carry */
    joint.mtx[0][0]=joint.mtx[1][1]=0;joint.mtx[0][1]=-1;joint.mtx[1][0]=1;
    mpLib_80055E9C(0);assert(groundCollVtx[0].pos.x==3 && groundCollVtx[0].pos.y==-5);
    assert(groundCollVtx[1].pos.y==15 && (groundCollJoint[0].flags&CollJoint_B9));
    joint.flags=JOBJ_HIDDEN;mpLib_80055E9C(0);assert(groundCollJoint[0].flags&CollJoint_Hidden);
    joint.flags=0;mpLib_80055E9C(0);assert(!(groundCollJoint[0].flags&CollJoint_Hidden));
    assert(updates>=4);puts("retail animated joint: translation/rotation/carry history/hide/unhide passed");return 0;
}
