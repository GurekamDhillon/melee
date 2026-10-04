/* Zone simulation core. Included in the snapshot-covered script_game TU and
 * portable fixtures. No host pointers, allocation, collision calls or clock. */
#ifndef SCRIPT_ZONES_CORE_H
#define SCRIPT_ZONES_CORE_H
#include <math.h>
#include <string.h>
#include <limits.h>
static int sz_finite(float f) { union {float f;unsigned int i;}u;u.f=f;return (u.i&0x7f800000u)!=0x7f800000u; }
#define SZ_CAP 64
#define SZ_VERTICES 16
#define SZ_ENTITIES 12 /* primary/sub for each of six owners */
#define SZ_EVENTS (SZ_ENTITIES*(2*SZ_CAP+2))
#define GENO_ZONE_SCHEMA_VERSION 1 /* reserved data schema; no loader */
enum { SZ_ENTER=1,SZ_EXIT=2,SZ_NONE=3,SZ_SOME=4 };
typedef struct {
    int handle,owner,live,model,n,ntags;
    char name[81],label[81],kind[32],tags[8][32];
    float x[SZ_VERTICES],y[SZ_VERTICES],dx,dy,area;
} SzDef;
typedef struct { int token;float x,y; } SzInput;
typedef struct {
    int token,id;float x,y;int handles[SZ_CAP],frames[SZ_CAP];
} SzEntity;
typedef struct {
    int what,entity,port,sub;float x,y;
    struct {int handle;char name[81],label[81],kind[32];} zone;
    int from[SZ_CAP];
} SzEvent;
typedef struct {
    SzDef z[SZ_CAP];SzEntity e[SZ_ENTITIES];SzEvent ev[SZ_EVENTS];
    int serial,entity_serial,nev,count,pending;
} SzState;
static int sz_validate(SzDef* z) {
    int i,j;float area=0;
    if(z->n<3 || z->n>SZ_VERTICES || !z->name[0] || !z->kind[0] || z->ntags<0 || z->ntags>8)return 0;
    for(i=0;i<z->n;i++) {
        j=(i+1)%z->n;
        if(!sz_finite(z->x[i]) || !sz_finite(z->y[i]) || fabsf(z->x[i])>49000 || fabsf(z->y[i])>49000)return 0;
        if(z->x[i]==z->x[j] && z->y[i]==z->y[j])return 0;
        area+=z->x[i]*z->y[j]-z->x[j]*z->y[i];
    }
    if(!sz_finite(area) || area==0)return 0;
    if(area<0)for(i=0;i<z->n/2;i++) {
        float f=z->x[i];z->x[i]=z->x[z->n-1-i];z->x[z->n-1-i]=f;
        f=z->y[i];z->y[i]=z->y[z->n-1-i];z->y[z->n-1-i]=f;
    }
    /* Every other vertex must be strictly inside each directed edge. This
     * refuses collinear/repeated vertices and self-intersecting input too. */
    for(i=0;i<z->n;i++)for(j=0;j<z->n;j++)if(j!=i && j!=(i+1)%z->n) {
        int k=(i+1)%z->n;
        float cross=(z->x[k]-z->x[i])*(z->y[j]-z->y[i])-(z->y[k]-z->y[i])*(z->x[j]-z->x[i]);
        if(!(cross>0))return 0;
    }
    z->area=fabsf(area)*0.5f;return 1;
}
static int sz_contains(const SzDef* z,float x,float y) {
    int i;x-=z->dx;y-=z->dy;
    if(z->live!=1 || !sz_finite(x) || !sz_finite(y))return 0;
    for(i=0;i<z->n;i++) {
        int j=(i+1)%z->n;float dx=z->x[j]-z->x[i],dy=z->y[j]-z->y[i];
        float c=dx*(y-z->y[i])-dy*(x-z->x[i]);
        /* CCW: bottom and left included, top and right excluded. */
        if(c<0 || (c==0 && !(dy<0 || (dy==0 && dx>0))))return 0;
    }return 1;
}
static int sz_slot(const SzState* s,int handle) {
    int i;for(i=0;i<SZ_CAP;i++)if(s->z[i].live && s->z[i].handle==handle && handle>0)return i;return -1;
}
static int sz_put(SzState* s,const SzDef* value,int handle) {
    SzDef z=*value;int i,at=handle ? sz_slot(s,handle):-1;
    if(!sz_validate(&z) || z.owner<=0)return -1;
    if(handle && (at<0 || s->z[at].live!=1 || s->z[at].owner!=z.owner))return -2;
    for(i=0;i<SZ_CAP;i++)if(s->z[i].live==1 && i!=at && !strcmp(s->z[i].name,z.name))return -3;
    if(!handle) {
        if(s->serial==INT_MAX)return -4;
        for(i=0;i<SZ_CAP;i++)if(!s->z[i].live){at=i;break;}
        if(at<0)return -4;
        handle=++s->serial;++s->count;
    }
    z.handle=handle;z.live=1;s->z[at]=z;return handle;
}
static int sz_remove(SzState* s,int handle,int owner) {
    int at=sz_slot(s,handle);
    if(at<0 || s->z[at].live!=1 || s->z[at].owner!=owner)return 0;
    s->z[at].live=2;--s->count;++s->pending;return 1; /* retain payload until terminal pass */
}
static void sz_emit(SzState* s,int what,int e,const SzDef* z,const SzEntity* entity,const int* from,int mask) {
    SzEvent* v;
    if(!(mask&(1<<(what-1))))return;
    /* Mathematical bound: at most 2*64+2 records for each of 12 entities. */
    v=&s->ev[s->nev++];memset(v,0,sizeof *v);v->what=what;
    v->entity=entity->id;v->port=e/2+1;v->sub=e%2;v->x=entity->x;v->y=entity->y;
    if(z){v->zone.handle=z->handle;memcpy(v->zone.name,z->name,sizeof v->zone.name);memcpy(v->zone.label,z->label,sizeof v->zone.label);memcpy(v->zone.kind,z->kind,sizeof v->zone.kind);}
    memcpy(v->from,from,sizeof v->from);
}
static void sz_step(SzState* s,const SzInput* input,int mask,int advance) {
    int e,i,j,order[SZ_CAP];s->nev=0;
    for(i=0;i<SZ_CAP;i++) {
        int k=i;order[i]=i;
        while(k>0 && s->z[order[k-1]].handle>s->z[order[k]].handle) {
            int t=order[k-1];order[k-1]=order[k];order[k]=t;--k;
        }
    }
    for(e=0;e<SZ_ENTITIES;e++) {
        SzEntity old=s->e[e],next=old;int from[SZ_CAP],before=0,after=0,newlife;
        memcpy(from,old.handles,sizeof from);
        newlife=old.token!=input[e].token;
        if(newlife){memset(&next,0,sizeof next);next.token=input[e].token;
            if(next.token && s->entity_serial<INT_MAX)next.id=++s->entity_serial;}
        if(input[e].token){next.x=input[e].x;next.y=input[e].y;}
        for(i=0;i<SZ_CAP;i++) {
            int inside=input[e].token && next.id && sz_contains(&s->z[i],next.x,next.y);
            if(old.handles[i])++before;
            next.handles[i]=inside ? s->z[i].handle:0;
            next.frames[i]=inside ? (!newlife && old.handles[i]==next.handles[i] ? old.frames[i]:0):0;
            if(inside){++after;if(advance && next.frames[i]<INT_MAX)++next.frames[i];}
        }
        for(j=0;j<SZ_CAP;j++){i=order[j];if(old.handles[i] && (newlife || old.handles[i]!=next.handles[i]))
            sz_emit(s,SZ_EXIT,e,&s->z[i],newlife ? &old:&next,from,mask);}
        if(before && (newlife || !after))sz_emit(s,SZ_NONE,e,0,newlife ? &old:&next,from,mask);
        for(j=0;j<SZ_CAP;j++){i=order[j];if(next.handles[i] && (newlife || old.handles[i]!=next.handles[i]))
            sz_emit(s,SZ_ENTER,e,&s->z[i],&next,from,mask);}
        if(after && (newlife || !before))sz_emit(s,SZ_SOME,e,0,&next,from,mask);
        s->e[e]=next;
    }
    for(i=0;i<SZ_CAP;i++)if(s->z[i].live==2)memset(&s->z[i],0,sizeof s->z[i]);
    s->pending=0;
}
#endif
