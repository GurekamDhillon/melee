/* Synthetic data only. Runs the same reader used by stage slots. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "../gameworld/script_stage_dat.h"
static unsigned char b[512];
static void w32(int p, unsigned v) { b[p]=v>>24; b[p+1]=v>>16; b[p+2]=v>>8; b[p+3]=v; }
static void w16(int p, unsigned v) { b[p]=v>>8; b[p+1]=v; }
static void fixture(void)
{
    memset(b,0,sizeof b);
    w32(0,512); w32(4,400); w32(12,1);
    w32(432,0); w32(436,0); memcpy(b+440,"coll_data",10);
    w32(32,48); w32(36,2); w32(40,64); w32(44,1);
    w32(68,80); w32(72,1);
    w32(80,0xc1200000); w32(84,0); w32(88,0x41200000); w32(92,0);
    w16(96,0); w16(98,1);
    w16(100,65535); w16(102,65535); w16(104,65535); w16(106,65535);
    w16(108,1); w16(110,0x60);
    w16(112,0); w16(114,1); w16(148,0); w16(150,2);
}
int main(void)
{
    StDatView v; StDatLine l; unsigned c;
    { const unsigned char s[]={1,2,0};
      assert(!st_dat_has_nul(s,0));assert(!st_dat_has_nul(s,2));
      assert(st_dat_has_nul(s,3)); }
    fixture();memset(b+440,1,sizeof b-440);
    assert(st_dat_open(b,sizeof b,&v)==ST_DAT_RANGE);
    b[sizeof b-1]=0;assert(st_dat_open(b,sizeof b,&v)==0);
    fixture(); assert(st_dat_open(b,sizeof b,&v)==0);
    assert(st_dat_collision(&v,767,&c)==0 && c==1);
    assert(st_dat_line(&v,0,&l)==0 && l.kind==1 && l.x0==-10 && l.x1==10);
    assert(l.lo==0x60 && l.joint==0 && l.prev0==-1);
    assert(st_dat_collision(&v,0,&c)==ST_DAT_BUDGET);
    w16(98,2); assert(st_dat_collision(&v,767,&c)==ST_DAT_INDEX);
    fixture(); st_dat_open(b,sizeof b,&v); w16(108,2);
    assert(st_dat_collision(&v,767,&c)==0); st_dat_line(&v,0,&l); assert(l.kind==2);
    w16(108,4); st_dat_line(&v,0,&l); assert(l.kind==3);
    w16(108,8); st_dat_line(&v,0,&l); assert(l.kind==4);
    w16(108,3); assert(st_dat_collision(&v,767,&c)==ST_DAT_KIND);
    fixture(); w32(68,399); st_dat_open(b,sizeof b,&v);
    assert(st_dat_collision(&v,767,&c)==ST_DAT_RANGE);
    fixture(); w32(4,0xffffffff); assert(st_dat_open(b,sizeof b,&v)==ST_DAT_RANGE);
    fixture(); w32(8,0xffffffff); assert(st_dat_open(b,sizeof b,&v)==ST_DAT_RANGE);
    fixture(); w32(436,10000); assert(st_dat_open(b,sizeof b,&v)==ST_DAT_RANGE);
    fixture(); w32(80,0x7fc00000); st_dat_open(b,sizeof b,&v);
    assert(st_dat_collision(&v,767,&c)==ST_DAT_COORD);
    fixture(); w16(100,1); st_dat_open(b,sizeof b,&v);
    assert(st_dat_collision(&v,767,&c)==ST_DAT_INDEX);
    fixture(); w16(114,2); st_dat_open(b,sizeof b,&v);
    assert(st_dat_collision(&v,767,&c)==ST_DAT_RANGE);
    fixture(); assert(st_dat_open(b,31,&v)==ST_DAT_RANGE);
    puts("stage DAT: flags, kinds, joints, indices, NaN, truncated data and budget refusal passed");
    return 0;
}
