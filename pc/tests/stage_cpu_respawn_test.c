/* Current production CPU pair/persistence helpers; reset order modeled here. */
#include <assert.h>
#include <stdio.h>
typedef struct {int is_sub_fighter;struct {int kind,level,x14,output;}cpu;}Fighter;
enum{CpuKind_0=0,CpuKind_4=4};
static int slot_kind=4,changed;
static void script_bench_cpu_changed(Fighter* fp){(void)fp;++changed;}
#include "stage_cpu_retail.inc"
static void init(Fighter* f,int kind,int level,int x14){f->cpu.kind=kind;f->cpu.level=level;f->cpu.x14=x14;f->cpu.output=0;}
static void persist(int slot,int kind){assert(slot==0);slot_kind=kind;}
int main(void)
{
    Fighter a={0},b={0};Fighter* pair[]={&a,&b};int i;
    a.cpu.level=7;b.cpu.level=8;a.cpu.x14=11;b.cpu.x14=22;
    assert(script_cpu_mode_persist(pair,0,0,init,persist));
    for(i=0;i<10;++i){ /* Fighter reset/rebirth reads persistent slot, not prior live kind */
      init(&a,slot_kind,a.cpu.level,a.cpu.x14);init(&b,slot_kind,b.cpu.level,b.cpu.x14);
      assert(a.cpu.kind==0 && b.cpu.kind==0 && a.cpu.level==7 && b.cpu.level==8 && !a.cpu.output && !b.cpu.output);}
    assert(changed==4);puts("production CPU persistence: both entities remain stood through ten modeled respawns");return 0;
}
