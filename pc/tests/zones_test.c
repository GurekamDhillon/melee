/* Portable fixture: exercises exactly the core included by script_game.c. */
#ifndef SZ_TEST_ENTRY
#include <assert.h>
#endif
#include <stdio.h>
#include <string.h>
#include "../gameworld/script_zones_core.h"
static SzState s, saved, expected;
static SzInput in[SZ_ENTITIES];
static SzDef rect(const char* name,float x0,float x1) {
    SzDef z={0}; strcpy(z.name,name);strcpy(z.label,name);strcpy(z.kind,"room");
    z.owner=1;z.n=4;z.x[0]=z.x[3]=x0;z.x[1]=z.x[2]=x1;
    z.y[0]=z.y[1]=-10;z.y[2]=z.y[3]=10;return z;
}
#ifndef SZ_TEST_ENTRY
#define SZ_TEST_ENTRY main
#endif
int SZ_TEST_ENTRY(void) {
    SzDef a=rect("left",-10,0),b=rect("right",0,10);int h,i;
    assert(sz_put(&s,&a,0)==1);assert(sz_put(&s,&b,0)==2);
    assert(sz_contains(&s.z[0],-10,-10));assert(!sz_contains(&s.z[0],0,0));
    assert(sz_contains(&s.z[1],0,0));assert(!sz_contains(&s.z[1],10,10));
    assert(!sz_contains(&s.z[0],-10,10));assert(!sz_contains(&s.z[0],0,-10));
    in[0].token=7;in[0].x=-1;sz_step(&s,in,15,1);
    assert(s.nev==2 && s.ev[0].what==SZ_ENTER && s.ev[1].what==SZ_SOME);
    for(i=0;i<20;i++){sz_step(&s,in,15,1);assert(s.nev==0);}
    assert(s.e[0].frames[0]==21);
    in[0].x=-10;for(i=0;i<20;i++){sz_step(&s,in,15,1);assert(s.nev==0);}
    in[0].x=0;sz_step(&s,in,15,1);assert(s.nev==2);
    assert(s.ev[0].what==SZ_EXIT && s.ev[1].what==SZ_ENTER);
    for(i=0;i<10;i++){in[0].x=-1;sz_step(&s,in,15,1);assert(s.nev==2);
        in[0].x=1;sz_step(&s,in,15,1);assert(s.nev==2);}
    in[0].x=30;sz_step(&s,in,15,1);assert(s.nev==2 && s.ev[1].what==SZ_NONE);
    in[0].x=1;sz_step(&s,in,15,1);assert(s.nev==2 && s.ev[1].what==SZ_SOME);
    in[0].token=0;sz_step(&s,in,15,1);assert(s.nev==2);
    in[0].token=8;sz_step(&s,in,0,1);assert(!s.nev && s.e[0].frames[1]==1);
    saved=s;sz_step(&s,in,15,1);expected=s;s=saved;sz_step(&s,in,15,1);
    assert(!memcmp(&s,&expected,sizeof s)); /* all state bytes, including events */
    assert(!sz_remove(&s,2,99));
    sz_remove(&s,2,1);sz_step(&s,in,15,1);assert(s.nev==2 && s.ev[0].zone.handle==2);
    memset(&s,0,sizeof s);
    for(i=0;i<SZ_CAP;i++){char name[20];sprintf(name,"z%d",i);a=rect(name,-10,10);assert(sz_put(&s,&a,0)>0);}
    a=rect("overflow",-10,10);assert(sz_put(&s,&a,0)<0);
    saved=s;a=s.z[0];a.x[0]=0.0f/0.0f;assert(sz_put(&s,&a,a.handle)<0);assert(!memcmp(&s,&saved,sizeof s));
    memset(&s,0,sizeof s);a=rect("moving",-1,1);h=sz_put(&s,&a,0);
    s.z[0].dx=10;assert(sz_contains(&s.z[0],10,0));assert(!sz_contains(&s.z[0],0,0));
    for(i=0;i<SZ_ENTITIES;i++){in[i].token=i+1;in[i].x=10;}
    sz_step(&s,in,15,1);assert(s.nev==SZ_ENTITIES*2);
    sz_remove(&s,h,1);sz_step(&s,in,15,0);assert(s.nev==SZ_ENTITIES*2);
    memset(&s,0,sizeof s);a=rect("poly",-1,1);a.n=3;a.x[0]=0;a.y[0]=0;a.x[1]=2;a.y[1]=0;a.x[2]=0;a.y[2]=2;
    assert(sz_put(&s,&a,0)>0);assert(sz_contains(&s.z[0],0.5f,0.5f));assert(!sz_contains(&s.z[0],2,2));
    assert(sz_contains(&s.z[0],0,0));assert(!sz_contains(&s.z[0],1,1));
    b=a;for(i=0;i<3;i++){b.x[i]=a.x[2-i];b.y[i]=a.y[2-i];}strcpy(b.name,"reverse");
    assert(sz_put(&s,&b,0)>0);assert(sz_contains(&s.z[1],0.5f,0.5f));
    saved=s;a=s.z[0];a.x[1]=a.x[0];a.y[1]=a.y[0];assert(sz_put(&s,&a,a.handle)<0);assert(!memcmp(&s,&saved,sizeof s));
    memset(&s,0,sizeof s);memset(in,0,sizeof in);
    for(i=0;i<SZ_CAP;i++){char name[20];sprintf(name,"full%d",i);a=rect(name,-10,10);assert(sz_put(&s,&a,0)>0);}
    for(i=0;i<SZ_ENTITIES;i++){in[i].token=i+1;in[i].x=0;}
    sz_step(&s,in,15,1);assert(s.nev==SZ_ENTITIES*(SZ_CAP+1));
    for(i=0;i<SZ_ENTITIES;i++)in[i].token+=100;
    sz_step(&s,in,15,1);assert(s.nev==SZ_EVENTS); /* no drop at worst-case lifetime replacement */
    for(i=0;i<SZ_ENTITIES;i++)in[i].x=100;
    sz_step(&s,in,1,1);assert(s.nev==0); /* exit/none hooks unarmed */
    for(i=0;i<SZ_ENTITIES;i++)in[i].x=0;
    sz_step(&s,in,1,1);assert(s.nev==SZ_ENTITIES*SZ_CAP); /* enter only */
    puts("zones core: PASS (edges, transitions, lifetime, capacity, cleanup, snapshot bytes)");return 0;
}
