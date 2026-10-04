/* Private DAT stdin, current retail bounds functions; no asset files. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <fcntl.h>
#include <io.h>
#include "../gameworld/script_stage_dat.h"
typedef unsigned char u8;typedef float f32;typedef struct{float x,y,z;}Vec3;
typedef struct{float left,right,top,bottom;}Box;
static struct{int grkind;struct{Box cam_bounds;float cam_x_offset,cam_y_offset;}cam_info;Box blast_zone;}stage_info;
enum{Gr_Kind_Castle=2,Gr_Kind_Corneria=14,Gr_Kind_Unk26=26,Gr_Kind_Inishie2=25,Gr_Kind_RCruise=3,Gr_Kind_Yorster=11,Gr_Kind_MuteCity=18,Gr_Kind_Shrine=7};
static Vec3 points[261],joints[4096];static int has[261],nj;
static float wrapper;
static int Ground_801C2D24(int id,Vec3* p){if(!has[id])return 0;*p=points[id];p->x*=wrapper;p->y*=wrapper;p->z*=wrapper;return 1;}
#define OSReport(...) ((void)0)
#include "stage_scale_retail.inc"
static void walk(const StDatView* v,unsigned at,Vec3 parent){
    while(at){Vec3 p;unsigned child,next;int i;
        assert(nj<4096 && st_dat_span(at,1,64,v->bytes));
        for(i=0;i<3;++i){assert(fabsf(st_dat_float(v->data+at+20+4*i))<.0001f);assert(fabsf(st_dat_float(v->data+at+32+4*i)-1)<.0001f);}
        p=(Vec3){parent.x+st_dat_float(v->data+at+44),parent.y+st_dat_float(v->data+at+48),parent.z+st_dat_float(v->data+at+52)};
        joints[nj++]=p;child=st_dat_u32(v->data+at+8);next=st_dat_u32(v->data+at+12);walk(v,child,p);at=next;
    }
}
int main(int argc,char** argv){unsigned char* raw=malloc(8*1024*1024);StDatView v;unsigned n,head,param,rows,count,r,i;float platform=0;Vec3 p;
    Box expected_cam,expected_blast;int side_platform=0;
    _setmode(_fileno(stdin),_O_BINARY);assert(st_dat_open(raw,fread(raw,1,8*1024*1024,stdin),&v)==0);
    assert(argc==2 && st_dat_symbol(&v,"grGroundParam",&param)==0 && st_dat_symbol(&v,"map_head",&head)==0);
    wrapper=st_dat_float(v.data+param);assert(fabsf(wrapper-(argv[1][0]=='b'?.8f:.7f))<.0001f);
    rows=st_dat_u32(v.data+head);count=st_dat_u32(v.data+head+4);
    for(r=0;r<count;++r){unsigned root=st_dat_u32(v.data+rows+12*r),pairs=st_dat_u32(v.data+rows+12*r+4),np=st_dat_u32(v.data+rows+12*r+8);int relevant=0;
        for(i=0;i<np;++i)if(st_dat_s16(v.data+pairs+4*i+2)==148)relevant=1;
        if(!relevant)continue;nj=0;walk(&v,root,(Vec3){0,0,0});
        for(i=0;i<np;++i){int index=st_dat_s16(v.data+pairs+4*i),id=st_dat_s16(v.data+pairs+4*i+2);assert(index>=0 && index<nj && id>=0 && id<261);points[id]=joints[index];has[id]=1;}
    }
    /* Independent native expectation: native Ground's parent scale applied
     * to the archive's world markers, then camera-relative blast coordinates. */
    assert(has[148]&&has[149]&&has[150]&&has[151]&&has[152]);
    expected_cam=(Box){points[149].x*wrapper-points[148].x*wrapper,points[150].x*wrapper-points[148].x*wrapper,points[149].y*wrapper-points[148].y*wrapper,points[150].y*wrapper-points[148].y*wrapper};
    expected_blast=(Box){points[151].x*wrapper-points[148].x*wrapper,points[152].x*wrapper-points[148].x*wrapper,points[151].y*wrapper-points[148].y*wrapper,points[152].y*wrapper-points[148].y*wrapper};
    Ground_801C39C0();Ground_801C3BB4();
    assert(!memcmp(&expected_cam,&stage_info.cam_info.cam_bounds,sizeof(Box)) && !memcmp(&expected_blast,&stage_info.blast_zone,sizeof(Box)));
    assert(Ground_801C2D24(0,&p) && fabsf(p.x-points[0].x*wrapper)<.0001f && fabsf(p.y-points[0].y*wrapper)<.0001f);
    assert(st_dat_collision(&v,767,&n)==0);
    for(i=0;i<n;++i){StDatLine l;assert(st_dat_line(&v,i,&l)==0);if(l.kind==1 && l.y0*wrapper>platform)platform=l.y0*wrapper;
        if(l.kind==1 && fabsf(l.y0*wrapper-23.45f)<.001f)side_platform=1;}
    /* Heights from the tester's native measurements in the fix3 prompt. */
    if(argv[1][0]=='b')assert(fabsf(platform-54.4f)<.001f);
    else assert(side_platform);
    printf("scaled real DAT: floor top %.3f blast right %.3f; retail camera/blast and scaled spawn passed\n",platform,stage_info.blast_zone.right);free(raw);return 0;}
