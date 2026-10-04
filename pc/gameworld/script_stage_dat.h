/* Portable, bounded reader of UNRELOCATED HSD stage archives. No game/OS state.
 * Layout: baselib/archive.h, mp/types.h. Never infer collision from models.
 * This stays usable by native fixtures and the game-world scalar transport. */
#ifndef SCRIPT_STAGE_DAT_H
#define SCRIPT_STAGE_DAT_H
#include <stddef.h>
#include <string.h>
enum { ST_DAT_OK, ST_DAT_RANGE, ST_DAT_SYMBOL, ST_DAT_BUDGET,
       ST_DAT_INDEX, ST_DAT_KIND, ST_DAT_COORD, ST_DAT_EXTERN };
typedef struct {
    const unsigned char *raw, *data;
    size_t size, bytes, publics, symbols;
    unsigned npublic, coll, verts, lines, joints, nv, nl, nj;
} StDatView;
typedef struct {
    float x0,y0,x1,y1;
    unsigned hi,lo;
    int kind,joint,prev0,next0,prev1,next1;
} StDatLine;
static unsigned st_dat_u32(const unsigned char* p)
{ return ((unsigned)p[0]<<24)|((unsigned)p[1]<<16)|((unsigned)p[2]<<8)|p[3]; }
static unsigned st_dat_u16(const unsigned char* p) { return ((unsigned)p[0]<<8)|p[1]; }
static int st_dat_s16(const unsigned char* p)
{ unsigned v=st_dat_u16(p); return v>=32768 ? (int)v-65536 : (int)v; }
static float st_dat_float(const unsigned char* p)
{ union { unsigned i; float f; } v; v.i=st_dat_u32(p); return v.f; }
static int st_dat_span(size_t at,size_t count,size_t stride,size_t end)
{ return at<=end && (!stride || count<=(end-at)/stride); }
/* The game libc shim has no byte-search export. Never scan beyond the archive. */
static int st_dat_has_nul(const unsigned char* p,size_t n)
{ size_t i;for(i=0;i<n;++i)if(!p[i])return 1;return 0; }
static int st_dat_symbol(const StDatView* v,const char* name,unsigned* out)
{
    unsigned i;
    for(i=0;i<v->npublic;++i) {
        const unsigned char* p=v->raw+v->publics+8*i;
        size_t off=st_dat_u32(p+4), n;
        if(off>=v->size-v->symbols) return ST_DAT_RANGE;
        n=v->size-v->symbols-off;
        if(!st_dat_has_nul(v->raw+v->symbols+off,n)) return ST_DAT_RANGE;
        if(!strcmp((const char*)v->raw+v->symbols+off,name)) {
            *out=st_dat_u32(p); return *out<v->bytes ? 0 : ST_DAT_RANGE;
        }
    }
    return ST_DAT_SYMBOL;
}
static int st_dat_open(const void* raw,size_t size,StDatView* v)
{
    const unsigned char* b=raw; unsigned nr,ne,i; size_t at;
    memset(v,0,sizeof *v);
    if(!b || size<32 || st_dat_u32(b)!=size) return ST_DAT_RANGE;
    v->raw=b; v->data=b+32; v->size=size; v->bytes=st_dat_u32(b+4);
    nr=st_dat_u32(b+8); v->npublic=st_dat_u32(b+12); ne=st_dat_u32(b+16);
    if(!st_dat_span(32,v->bytes,1,size)) return ST_DAT_RANGE;
    at=32+v->bytes;
    if(!st_dat_span(at,nr,4,size)) return ST_DAT_RANGE;
    for(i=0;i<nr;++i) {
        unsigned off=st_dat_u32(b+at+4*i);
        if((off&3) || !st_dat_span(off,1,4,v->bytes) ||
           st_dat_u32(v->data+off)>=v->bytes) return ST_DAT_RANGE;
    }
    at+=4*(size_t)nr; v->publics=at;
    if(!st_dat_span(at,v->npublic,8,size)) return ST_DAT_RANGE;
    at+=8*(size_t)v->npublic;
    if(!st_dat_span(at,ne,8,size)) return ST_DAT_RANGE;
    v->symbols=at+8*(size_t)ne;
    /* Retail Yoshi/PS archives have extern chains for secondary art/animations.
     * Slots resolve them to NULL, exactly like script_stage_archive. Validate
     * the linked patch list before HSD follows it; never execute stage code. */
    for(i=0;i<ne;++i) {
        unsigned off=st_dat_u32(b+at+8*i), steps=0;
        size_t sym=st_dat_u32(b+at+8*i+4);
        if(sym>=size-v->symbols || !st_dat_has_nul(b+v->symbols+sym,size-v->symbols-sym)) return ST_DAT_RANGE;
        while(off!=0xffffffffu) {
            if((off&3) || !st_dat_span(off,1,4,v->bytes) || ++steps>v->bytes/4) return ST_DAT_EXTERN;
            off=st_dat_u32(v->data+off);
        }
    }
    for(i=0;i<v->npublic;++i) {
        unsigned dummy; const unsigned char* p=b+v->publics+8*i;
        size_t off=st_dat_u32(p+4);
        if(st_dat_u32(p)>=v->bytes || off>=size-v->symbols ||
           !st_dat_has_nul(b+v->symbols+off,size-v->symbols-off)) return ST_DAT_RANGE;
        (void)dummy;
    }
    return 0;
}
/* Read a pointer before relocation, treating retail external patch sites as
 * NULL (the static loader never imports their secondary archives). */
static unsigned st_dat_ref(const StDatView* v,unsigned field)
{
    unsigned ne=st_dat_u32(v->raw+16),i;size_t table=v->publics+8*(size_t)v->npublic;
    for(i=0;i<ne;++i){unsigned off=st_dat_u32(v->raw+table+8*i),steps=0;
        while(off!=0xffffffffu && ++steps<=v->bytes/4){if(off==field)return 0;off=st_dat_u32(v->data+off);}}
    return st_dat_u32(v->data+field);
}
static int st_dat_line(const StDatView* v,unsigned i,StDatLine* l)
{
    const unsigned char *p,*a,*b; unsigned x,y,k; int j;
    if(i>=v->nl) return ST_DAT_INDEX;
    p=v->data+v->lines+16*i; x=st_dat_u16(p); y=st_dat_u16(p+2);
    if(x>=v->nv || y>=v->nv) return ST_DAT_INDEX;
    a=v->data+v->verts+8*x; b=v->data+v->verts+8*y;
    l->x0=st_dat_float(a); l->y0=st_dat_float(a+4);
    l->x1=st_dat_float(b); l->y1=st_dat_float(b+4);
    if(!(l->x0>=-49000 && l->x0<=49000 && l->y0>=-49000 && l->y0<=49000 &&
         l->x1>=-49000 && l->x1<=49000 && l->y1>=-49000 && l->y1<=49000) ||
         (l->x0==l->x1 && l->y0==l->y1)) return ST_DAT_COORD;
    l->hi=st_dat_u16(p+12); l->lo=st_dat_u16(p+14); k=l->hi&15;
    l->kind=k==1?1:k==2?2:k==4?3:k==8?4:0;
    if(!l->kind) return ST_DAT_KIND;
    l->prev0=st_dat_s16(p+4); l->next0=st_dat_s16(p+6);
    l->prev1=st_dat_s16(p+8); l->next1=st_dat_s16(p+10);
    if(l->prev0< -1 || l->next0< -1 || l->prev1< -1 || l->next1< -1 ||
       l->prev0>=(int)v->nl || l->next0>=(int)v->nl ||
       l->prev1>=(int)v->nl || l->next1>=(int)v->nl) return ST_DAT_INDEX;
    l->joint=-1;
    for(j=0;j<(int)v->nj;++j) for(k=0;k<5;++k) {
        p=v->data+v->joints+40*j+4*k;
        x=st_dat_u16(p); y=st_dat_u16(p+2);
        if(i>=x && i-x<y) {
            if(l->joint>=0 && l->joint!=j) return ST_DAT_INDEX;
            l->joint=j;
        }
    }
    return l->joint<0 ? ST_DAT_INDEX : 0;
}
static int st_dat_collision(StDatView* v,unsigned budget,unsigned* count)
{
    unsigned i,k; int rc; StDatLine l; const unsigned char* p;
    rc=st_dat_symbol(v,"coll_data",&v->coll); if(rc) return rc;
    if(!st_dat_span(v->coll,1,48,v->bytes)) return ST_DAT_RANGE;
    p=v->data+v->coll;
    v->verts=st_dat_u32(p); v->nv=st_dat_u32(p+4);
    v->lines=st_dat_u32(p+8); v->nl=st_dat_u32(p+12);
    v->joints=st_dat_u32(p+36); v->nj=st_dat_u32(p+40);
    if(v->nl>budget || v->nv>2048 || v->nl>1536 || v->nj>256) return ST_DAT_BUDGET;
    if(!v->nl || !v->nv || !v->nj || !st_dat_span(v->verts,v->nv,8,v->bytes) ||
       !st_dat_span(v->lines,v->nl,16,v->bytes) || !st_dat_span(v->joints,v->nj,40,v->bytes))
        return ST_DAT_RANGE;
    for(i=0;i<v->nj;++i) {
        p=v->data+v->joints+40*i;
        for(k=0;k<5;++k) {
            int start=st_dat_s16(p+4*k), n=st_dat_s16(p+4*k+2);
            if(n<0 || (n && (start<0 || (unsigned)start>v->nl || (unsigned)n>v->nl-start)))
                return ST_DAT_RANGE;
        }
        if(!st_dat_span(st_dat_u16(p+36),st_dat_u16(p+38),1,v->nv)) return ST_DAT_RANGE;
    }
    for(i=0;i<v->nl;++i) { rc=st_dat_line(v,i,&l); if(rc) return rc; }
    *count=v->nl; return 0;
}
#endif
