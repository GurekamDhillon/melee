/* Bounded descriptor census before HSD allocation. Runtime allowance is an
 * explicit conservative estimate; the heap delta after loading is measured.
 * Texture payload bytes remain in the DAT allocation, not a second game heap. */
#ifndef SCRIPT_STAGE_DAT_MODELS_H
#define SCRIPT_STAGE_DAT_MODELS_H
#include "script_stage_dat.h"
typedef struct {unsigned groups,joints,dobjs,pobjs,tobjs,texture_bytes,runtime_allowance,nimages;
                unsigned image[4096],image_size[4096];} StDatModels;
static int st_dat_object(const StDatView* v,unsigned off,unsigned bytes)
{ return off && !(off&3) && st_dat_span(off,1,bytes,v->bytes); }
static int st_dat_texture(const StDatView* v,unsigned off,StDatModels* m)
{
    const unsigned char* p;unsigned image,w,h,fmt,bw,bh,block,mip,level,size=0;
    float maxlod;
    if(!off)return 0;
    if(!st_dat_object(v,off,24))return ST_DAT_RANGE;
    p=v->data+off;image=st_dat_u32(p);w=st_dat_u16(p+4);h=st_dat_u16(p+6);fmt=st_dat_u32(p+8);
    mip=st_dat_u32(p+12);maxlod=st_dat_float(p+20);
    if(!w || !h || w>4096 || h>4096 || !(maxlod>=0 && maxlod<=12))return ST_DAT_RANGE;
    switch(fmt) {
    case 0:case 8:case 14:bw=8;bh=8;block=32;break;
    case 1:case 2:case 9:bw=8;bh=4;block=32;break;
    case 3:case 4:case 5:case 10:bw=4;bh=4;block=32;break;
    case 6:bw=4;bh=4;block=64;break;
    default:return ST_DAT_RANGE;
    }
    for(level=0;level<=(mip?(unsigned)maxlod:0);++level) {
        size+=((w+bw-1)/bw)*((h+bh-1)/bh)*block;
        w=w>1?w/2:1;h=h>1?h/2:1;
    }
    if(!st_dat_span(image,size,1,v->bytes))return ST_DAT_RANGE;
    for(level=0;level<m->nimages;++level)if(m->image[level]==image){
        if(size>m->image_size[level]){m->texture_bytes+=size-m->image_size[level];m->image_size[level]=size;}return 0;}
    if(m->nimages>=4096)return ST_DAT_BUDGET;
    m->image[m->nimages]=image;m->image_size[m->nimages++]=size;m->texture_bytes+=size;return 0;
}
static int st_dat_joint_tree(const StDatView* v,unsigned joint,unsigned* path,unsigned depth,StDatModels* m)
{
    const unsigned char* p;unsigned i,dobj;int rc;
    if(!joint)return 0;
    if(depth>=128 || ++m->joints>4096 || !st_dat_object(v,joint,64))return ST_DAT_RANGE;
    for(i=0;i<depth;++i)if(path[i]==joint)return ST_DAT_RANGE;
    path[depth]=joint;p=v->data+joint;
    if(st_dat_u32(p+4)&4096)return ST_DAT_RANGE; /* instance trees are not supported */
    /* A joint's union is a DObj only in the normal joint case. Splines and
     * particle markers are data, never treated as linked model descriptors. */
    dobj=(st_dat_u32(p+4)&((1u<<14)|(1u<<5)))?0:st_dat_ref(v,joint+16);
    while(dobj) {
        unsigned mat,pobj;const unsigned char* d;
        if(++m->dobjs>4096 || !st_dat_object(v,dobj,16))return ST_DAT_RANGE;
        d=v->data+dobj;mat=st_dat_ref(v,dobj+8);pobj=st_dat_ref(v,dobj+12);
        if(mat) {
            unsigned tobj;
            if(!st_dat_object(v,mat,24))return ST_DAT_RANGE;
            tobj=st_dat_ref(v,mat+8);
            while(tobj) {
                unsigned image;
                if(++m->tobjs>4096 || !st_dat_object(v,tobj,92))return ST_DAT_RANGE;
                image=st_dat_ref(v,tobj+76);
                rc=st_dat_texture(v,image,m);if(rc)return rc;
                tobj=st_dat_ref(v,tobj+4);
            }
        }
        while(pobj) {
            unsigned display,n;
            if(++m->pobjs>4096 || !st_dat_object(v,pobj,24))return ST_DAT_RANGE;
            display=st_dat_u32(v->data+pobj+16);n=st_dat_u16(v->data+pobj+14);
            if(n && !st_dat_span(display,n,32,v->bytes))return ST_DAT_RANGE;
            pobj=st_dat_ref(v,pobj+4);
        }
        dobj=st_dat_ref(v,dobj+4);
    }
    rc=st_dat_joint_tree(v,st_dat_ref(v,joint+8),path,depth+1,m);if(rc)return rc;
    return st_dat_joint_tree(v,st_dat_ref(v,joint+12),path,depth+1,m);
}
static int st_dat_models_mask(const StDatView* v,StDatModels* m,unsigned long long mask)
{
    unsigned head,groups,i,path[128];int rc;
    memset(m,0,sizeof *m);rc=st_dat_symbol(v,"map_head",&head);if(rc)return rc;
    if(!st_dat_span(head,1,48,v->bytes))return ST_DAT_RANGE;
    groups=st_dat_u32(v->data+head+8);m->groups=st_dat_u32(v->data+head+12);
    if(!m->groups || m->groups>64 || !st_dat_span(groups,m->groups,52,v->bytes))return ST_DAT_RANGE;
    for(i=0;i<m->groups;++i) {
        if(!(mask & (1ull<<i)))continue;
        rc=st_dat_joint_tree(v,st_dat_ref(v,groups+52*i),path,0,m);if(rc)return rc;
    }
    /* Includes object-pool chunk overhead, transforms, material expressions and
     * envelope bookkeeping. This is deliberately reported as an allowance. */
    m->runtime_allowance=65536+(unsigned)v->bytes+2*(m->joints*512+m->dobjs*256+m->pobjs*512+m->tobjs*1024);
    return 0;
}
static int st_dat_models(const StDatView* v,StDatModels* m)
{return st_dat_models_mask(v,m,~0ull);}
#endif
