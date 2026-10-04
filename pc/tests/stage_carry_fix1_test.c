#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <fcntl.h>
#include <io.h>
#include "../gameworld/script_stage_dat.h"
typedef unsigned u32;
typedef struct {float x,y,z;} Vec3;
struct CpuFighter {int value;};
typedef struct {struct {int index;u32 flags;Vec3 normal;}floor;u32 env_flags,prev_env_flags;Vec3 contact;}CollData;
typedef struct {Vec3 cur_pos,self_vel,x8c_kb_vel,x98_atk_shield_kb;float gr_vel;
    int ground_or_air,player_id,is_sub_fighter,motion_id;struct CpuFighter cpu;CollData coll_data;} Fighter;
typedef struct HSD_GObj {struct HSD_GObj* next;Fighter* user_data;} HSD_GObj;
#define GA_Ground 0
#define GA_Air 1
#define HSD_GOBJ_PLINK_FIGHTER 0
#define GET_FIGHTER(g) ((g)->user_data)
#define Collide_FloorHug 65536
#define Collide_LedgeGrabMask 1
enum {ftCo_MS_CliffCatch=252,ftCo_MS_CliffJumpQuick2=260};
static HSD_GObj* heads[1],**HSD_GObjPLinkHead=heads;
static struct {struct {float left,right,bottom,top;}blast_zone;struct {float cam_x_offset,cam_y_offset;}cam_info;}stage_info;
static struct {struct {int line_count;}*map;}script_stage;
static int falls,spawns,logs;
#define OSReport(...) (++logs)
static void ftCo_Fall_Enter(HSD_GObj* g) {++falls;g->user_data->ground_or_air=GA_Air;g->user_data->motion_id=29;g->user_data->self_vel.x=999;g->user_data->cpu.value=999;}
static void script_switch_contacts(Fighter* fp) {memset(&fp->coll_data,0,sizeof fp->coll_data);fp->coll_data.floor.index=-1;}
static void ftCamera_80076064(Fighter* fp) {(void)fp;}
static int Ground_801C2D24(int player,Vec3* p) {(void)player;++spawns;*p=(Vec3){0,30,0};return 1;}
static StDatLine lines[5][768];static int counts[5],destination;
static float scales[5];
static Vec3 marker_joints[4096],points[261];static int nj,has[261];
static float bounds[5][4];
static void markers(const StDatView* v,unsigned at,Vec3 parent,Vec3 parent_scale)
{
    while(at) {
        Vec3 p,s;int i;unsigned child,next;
        assert(nj<4096 && st_dat_span(at,1,64,v->bytes));
        for(i=0;i<3;++i)assert(fabsf(st_dat_float(v->data+at+20+4*i))<.0001f);
        p=(Vec3){parent.x+parent_scale.x*st_dat_float(v->data+at+44),parent.y+parent_scale.y*st_dat_float(v->data+at+48),parent.z+parent_scale.z*st_dat_float(v->data+at+52)};
        s=(Vec3){parent_scale.x*st_dat_float(v->data+at+32),parent_scale.y*st_dat_float(v->data+at+36),parent_scale.z*st_dat_float(v->data+at+40)};
        marker_joints[nj++]=p;child=st_dat_u32(v->data+at+8);next=st_dat_u32(v->data+at+12);markers(v,child,p,s);at=next;
    }
}
static int mpCheckFloor(float ax,float ay,float bx,float by,int ignored,Vec3* p,int* line,u32* flags,Vec3* n,int skip,int a,int b,void* c,void* d)
{
    int i,found=-1;float best=by-1;
    (void)bx;(void)ignored;(void)a;(void)b;(void)c;(void)d;
    for(i=0;i<counts[destination];++i) {
        StDatLine* l=&lines[destination][i];float y;
        if(l->kind!=1 || i==skip || l->x0==l->x1 || ax<fminf(l->x0,l->x1) || ax>fmaxf(l->x0,l->x1))continue;
        y=l->y0+(ax-l->x0)*(l->y1-l->y0)/(l->x1-l->x0);
        if(y<=ay && y>=by && y>best){best=y;found=i;}
    }
    if(found<0)return 0;
    *p=(Vec3){ax,best,0};*n=(Vec3){0,1,0};*line=found;*flags=1;return 1;
}
#include "stage_detach_fix1_retail.inc"
#include "stage_carry_fix1_retail.inc"
static Fighter initial(float x,float y,int air)
{
    Fighter fp={0};fp.cur_pos=(Vec3){x,y,2};fp.self_vel=(Vec3){3,4,5};fp.x8c_kb_vel=(Vec3){6,7,8};
    fp.x98_atk_shield_kb=(Vec3){9,10,11};fp.gr_vel=12;fp.ground_or_air=air;fp.motion_id=20;fp.cpu.value=42;return fp;
}
static void check(Fighter* fp,Fighter* old,int oldlogs)
{
    assert(logs==oldlogs+1 && fp->cpu.value==old->cpu.value);
    assert(!memcmp(&fp->self_vel,&old->self_vel,sizeof(Vec3)));
    assert(!memcmp(&fp->x8c_kb_vel,&old->x8c_kb_vel,sizeof(Vec3)));
    assert(!memcmp(&fp->x98_atk_shield_kb,&old->x98_atk_shield_kb,sizeof(Vec3)) && fp->gr_vel==old->gr_vel);
}
int main(void)
{
    unsigned char* raw=malloc(8*1024*1024);int src,dst,i,j;unsigned bytes;Fighter fp,old;HSD_GObj g={NULL,&fp};
    static struct {int line_count;} map;
    _setmode(_fileno(stdin),_O_BINARY);heads[0]=&g;script_stage.map=(void*)&map;
    for(i=0;i<5;++i) {
        StDatView v;unsigned count,param,head,rows,r;
        assert(fread(&bytes,sizeof bytes,1,stdin)==1 && bytes<8*1024*1024);
        assert(fread(raw,1,bytes,stdin)==bytes && st_dat_open(raw,bytes,&v)==0);
        assert(st_dat_collision(&v,767,&count)==0 && st_dat_symbol(&v,"grGroundParam",&param)==0);
        scales[i]=st_dat_float(v.data+param);counts[i]=count;
        for(j=0;j<(int)count;++j){assert(st_dat_line(&v,j,&lines[i][j])==0);lines[i][j].x0*=scales[i];lines[i][j].x1*=scales[i];lines[i][j].y0*=scales[i];lines[i][j].y1*=scales[i];}
        assert(st_dat_symbol(&v,"map_head",&head)==0);rows=st_dat_u32(v.data+head);count=st_dat_u32(v.data+head+4);memset(has,0,sizeof has);
        for(r=0;r<count;++r) {
            unsigned root=st_dat_u32(v.data+rows+12*r),pairs=st_dat_u32(v.data+rows+12*r+4),np=st_dat_u32(v.data+rows+12*r+8);int relevant=0;
            for(j=0;j<(int)np;++j)if(st_dat_s16(v.data+pairs+4*j+2)==148)relevant=1;
            if(!relevant)continue;nj=0;markers(&v,root,(Vec3){0,0,0},(Vec3){1,1,1});
            for(j=0;j<(int)np;++j){int index=st_dat_s16(v.data+pairs+4*j),id=st_dat_s16(v.data+pairs+4*j+2);assert(index>=0 && index<nj && id>=0 && id<261);points[id]=marker_joints[index];has[id]=1;}
        }
        assert(has[151] && has[152]);bounds[i][0]=points[151].x*scales[i];bounds[i][1]=points[152].x*scales[i];
        bounds[i][2]=points[152].y*scales[i];bounds[i][3]=points[151].y*scales[i];
    }
    /* Each source/destination pair: every authored source floor midpoint as
     * ground and open air; missing-floor column; blast clamp; detached ledge. */
    for(src=0;src<5;++src)for(dst=0;dst<5;++dst) {
        destination=dst;map.line_count=counts[dst];
        float l=bounds[dst][0],r=bounds[dst][1],b=bounds[dst][2],t=bounds[dst][3];
        stage_info.blast_zone.left=l;stage_info.blast_zone.right=r;
        stage_info.blast_zone.bottom=b;stage_info.blast_zone.top=t;
        for(i=0;i<counts[src];++i)if(lines[src][i].kind==1) {
            float x=(lines[src][i].x0+lines[src][i].x1)/2,y=(lines[src][i].y0+lines[src][i].y1)/2;
            Vec3 floor,n;u32 flags;int line,hit;
            if(x<=l+1 || x>=r-1 || y<=b+1 || y+5>=t-1)continue;
            fp=old=initial(x,y,GA_Ground);hit=script_retail_carry_floor(x,y,&floor,&line,&flags,&n);
            j=logs;script_retail_carry();check(&fp,&old,j);assert(spawns==0 && fp.cur_pos.x==x);
            if(hit && floor.y>b && floor.y<t){assert(fp.cur_pos.y==floor.y && fp.motion_id==old.motion_id && fp.ground_or_air==GA_Ground && fp.coll_data.floor.index==line);}
            else assert(fp.cur_pos.y==y && fp.ground_or_air==GA_Air);
            fp=old=initial(x,y+5,GA_Air);j=logs;script_retail_carry();check(&fp,&old,j);
            assert(!memcmp(&fp.cur_pos,&old.cur_pos,sizeof(Vec3)) && fp.motion_id==old.motion_id && spawns==0);
        }
        fp=old=initial(r-1,40,GA_Ground);j=logs;script_retail_carry();check(&fp,&old,j);
        assert(fp.cur_pos.x==r-1 && fp.cur_pos.y==40 && fp.ground_or_air==GA_Air && spawns==0);
        fp=old=initial(10000,-10000,GA_Air);j=logs;script_retail_carry();check(&fp,&old,j);
        assert(fp.cur_pos.x==r-1 && fp.cur_pos.y==b+1 && fp.motion_id==old.motion_id && spawns==0);
        fp=old=initial(-60,10,GA_Air);j=logs;script_retail_carry();check(&fp,&old,j);
        assert(!memcmp(&fp.cur_pos,&old.cur_pos,sizeof(Vec3)) && spawns==0);
        fp=old=initial(-60,10,GA_Air);fp.motion_id=ftCo_MS_CliffCatch;
        script_switch_detach();j=logs;script_retail_carry();check(&fp,&old,j);
        assert(!memcmp(&fp.cur_pos,&old.cur_pos,sizeof(Vec3)) && fp.motion_id==29 && spawns==0);
        fp=old=initial(r-.25f,b+.25f,GA_Air);j=logs;script_retail_carry();check(&fp,&old,j);
        assert(!memcmp(&fp.cur_pos,&old.cur_pos,sizeof(Vec3)) && spawns==0);
    }
    fp=old=initial(NAN,20,GA_Air);j=logs;script_retail_carry();check(&fp,&old,j);assert(spawns==1);
    puts("continuous carry: all 25 retail stage pairs, authored floor columns/open air/missing floor/ledge/blast clamp, exact velocity/action and rare nonfinite fallback passed");
    free(raw);return 0;
}
