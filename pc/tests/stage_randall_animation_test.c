/* Stream the private DAT into the retail FObj/AObj interpreter. No graphics. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <stdint.h>
#include <fcntl.h>
#include <io.h>
#include "../gameworld/script_stage_dat.h"
typedef unsigned char u8;typedef signed char s8;typedef uint32_t u32;typedef int32_t s32;typedef int16_t s16;typedef uint16_t u16;typedef float f32;
typedef struct {float x,y,z;}Vec3;
typedef struct HSD_FObj HSD_FObj;
typedef union HSD_ObjData HSD_ObjData;
typedef void (*HSD_ObjUpdateFunc)(void*,u32,HSD_ObjData*);
#define HSD_ASSERT(line,test) assert(test)
#define ABS(x) fabsf(x)
#define sqrtf__Ff sqrtf
static int HSD_AObj_804D762C,HSD_AObj_804D7630;
#include "stage_animation_types_retail.inc"
#include "stage_animation_interpreter_retail.inc"
static HSD_FObj fobjs[512];static int nf;
typedef struct {HSD_AObj aobj;float values[32];float first[32];HSD_Spline spline;Vec3 cv[256];float segments[256],poly[256][5];Vec3 point;}Node;
static Node nodes[128];static int nn;
static void update(void* object,u32 type,HSD_ObjData* d){Node* n=object;assert(type<32);n->values[type]=d->fv;if(type==4 && n->spline.numcv)splArcLengthPoint(&n->point,&n->spline,d->fv);}
static HSD_FObj* load_fobj(StDatView* v,unsigned at)
{
    HSD_FObj* f;unsigned raw;
    if(!at)return NULL;
    assert(nf<512);f=&fobjs[nf++];
    f->length=st_dat_u32(v->data+at+4);f->startframe=(s16)st_dat_float(v->data+at+8);
    f->obj_type=v->data[at+12];f->frac_value=v->data[at+13];f->frac_slope=v->data[at+14];raw=st_dat_u32(v->data+at+16);
    assert(st_dat_span(raw,f->length,1,v->bytes));f->ad_head=(u8*)v->data+raw;
    f->next=load_fobj(v,st_dat_u32(v->data+at));return f;
}
static void load_anim(StDatView* v,unsigned at)
{
    unsigned desc;if(!at)return;desc=st_dat_u32(v->data+at+8);
    if(desc){Node* n;assert(nn<128);n=&nodes[nn++];n->aobj.flags=st_dat_u32(v->data+desc);
      n->aobj.end_frame=st_dat_float(v->data+desc+4);n->aobj.framerate=1;
      {unsigned id=st_dat_u32(v->data+desc+12);if(id && (st_dat_u32(v->data+id+4)&(1<<14))) {
        unsigned sp=st_dat_u32(v->data+id+16),cv=st_dat_u32(v->data+sp+8),segments=st_dat_u32(v->data+sp+16),poly=st_dat_u32(v->data+sp+20);int i,k,count;
        n->spline.type=v->data[sp];n->spline.numcv=st_dat_s16(v->data+sp+2);n->spline.tension=st_dat_float(v->data+sp+4);n->spline.totalLength=st_dat_float(v->data+sp+12);
        count=n->spline.type==1?3*n->spline.numcv:n->spline.numcv+2;assert(count<256);
        for(i=0;i<count;++i)n->cv[i]=(Vec3){st_dat_float(v->data+cv+12*i),st_dat_float(v->data+cv+12*i+4),st_dat_float(v->data+cv+12*i+8)};
        for(i=0;i<n->spline.numcv;++i){n->segments[i]=st_dat_float(v->data+segments+4*i);if(poly)for(k=0;k<5;++k)n->poly[i][k]=st_dat_float(v->data+poly+20*i+4*k);}
        n->spline.cv=n->cv;n->spline.segLength=n->segments;n->spline.segPoly=n->poly;
      }}
      n->aobj.fobj=load_fobj(v,st_dat_u32(v->data+desc+8));HSD_AObjReqAnim(&n->aobj,0);}
    load_anim(v,st_dat_u32(v->data+at));load_anim(v,st_dat_u32(v->data+at+4));
}
int main(void)
{
    unsigned char* raw=malloc(8*1024*1024);StDatView v;unsigned head,group,tree;int frame,i,k,loops=0;
    _setmode(_fileno(stdin),_O_BINARY);assert(!st_dat_open(raw,fread(raw,1,8*1024*1024,stdin),&v));
    assert(!st_dat_symbol(&v,"map_head",&head));group=st_dat_u32(v.data+head+8)+2*52;
    tree=st_dat_u32(v.data+st_dat_u32(v.data+group+4));load_anim(&v,tree);
    {unsigned count,i,bindings=st_dat_u32(v.data+group+32);int source=st_dat_s16(v.data+bindings);
      assert(!st_dat_collision(&v,768,&count));
      for(i=0;i<count;++i){StDatLine line;assert(!st_dat_line(&v,i,&line));
        if(line.joint==source)printf("Randall source joint=%d depth=%d local line (%g,%g)->(%g,%g) kind=%d\n",source,st_dat_s16(v.data+bindings+4),line.x0,line.y0,line.x1,line.y1,line.kind);}}
    for(frame=0;frame<=2400;++frame)for(i=0;i<nn;++i){Node* n=&nodes[i];
      HSD_AObjInterpretAnim(&n->aobj,n,update);
      if(n->aobj.end_frame!=1200)continue;
      assert(n->aobj.flags&AOBJ_LOOP);
      if(frame==0)memcpy(n->first,n->values,sizeof n->first);
      if(frame==1200 || frame==2400){assert(n->aobj.curr_frame==0);for(k=0;k<32;++k)assert(fabsf(n->first[k]-n->values[k])<0.0001f);++loops;}
      if(frame%120==0 && frame<1200)printf("Randall node=%d frame=%d path joint x=%g y=%g z=%g path=%g nodevis=%g branchvis=%g\n",i,frame,n->point.x*.7f,n->point.y*.7f,n->point.z*.7f,n->values[4],n->values[11],n->values[12]);
    }
    assert(loops>0);printf("Randall retail FObj/AObj: %d loop samples match at 1200/2400 frames passed\n",loops);free(raw);return 0;
}
