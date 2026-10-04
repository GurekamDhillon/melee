/* Run the production slot/switch implementation with deterministic game fakes.
 * No disc, renderer, game build or executable is linked into this test. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define STAGE_SLOT_FIXTURE
#include "stage_slots_fixture.h"
#include "../gameworld/script_stage_slots.inc"
static StDatLine* floor_lines(int n)
{
    StDatLine* p=calloc(n,sizeof *p);int i;
    for(i=0;i<n;++i){p[i].x0=-10;p[i].x1=10;p[i].kind=1;p[i].prev0=p[i].next0=p[i].prev1=p[i].next1=-1;}
    return p;
}
static ScriptStageSlot* slot(int at,int h,int owner,int n)
{
    ScriptStageSlot* s=&script_slots[at];memset(s,0,sizeof *s);
    s->handle=h;s->owner=owner;s->lines=floor_lines(n);s->nlines=n;s->music=-1;
    s->param=&fixture_param;
    s->camera.cam_bounds=(StageBlastZone){-20,20,30,-10};s->blast=(StageBlastZone){-50,50,50,-50};
    return s;
}
int main(void)
{
    int i,rc;ScriptStageSlot* a;Fighter* fp=&fixture_fighter;
    { StageData data={0};const char* expected="GrPs.dat";
      data.data1="/GrPs";stage_datas[1]=&data;
      assert(Ground_StageSlotDataFile(-1)==NULL);
      assert(Ground_StageSlotDataFile(221)==NULL);
      assert(Ground_StageSlotDataFile(2)==NULL);
      for(i=0;i<=9;++i)assert(ScriptGame_StageSlotFile(1,i)==expected[i]);
      data.data1="/GrNLa.dat";expected="GrNLa.dat";
      for(i=0;i<=9;++i)assert(ScriptGame_StageSlotFile(1,i)==expected[i]);
      assert(ScriptGame_StageSlotFile(1,-1)==0);
      assert(ScriptGame_StageSlotFile(1,260)==0);
      data.data1=NULL;assert(ScriptGame_StageSlotFile(1,0)==0);
      stage_datas[1]=NULL; }
    fixture_init();
    /* Full-pool refusal must precede any allocating/archive/model operation. */
    for(i=0;i<16;++i)slot(i,100+i,1,1);
    script_slots[1].ngroups=1;script_slots[1].models[0]=calloc(1,sizeof(HSD_GObj));
    script_slots[1].models[0]->hsd_obj=calloc(1,sizeof(HSD_JObj));
    assert(ScriptGame_StageSlotLoad(1,999,-1)==-3 && fixture_allocs==0);
    assert(ScriptGame_StageSlotFree(100,2)==0);
    assert(ScriptGame_StageSlotFree(100,1)==1 && !script_slots[0].handle);
    assert(ScriptGame_StageSlotFree(100,1)==0);
    ScriptGame_StageSlotsRelease(0);assert(!ScriptGame_StageSlotsActive());
    assert(fixture_frees==16);
    assert(fixture_model_frees==1); /* no stage GObj survives pool teardown */
    fixture_init();a=slot(0,1,1,3);
    assert(ScriptGame_StageSwitchCheck(1,2,0)==-5);
    fixture_online=1;assert(ScriptGame_StageSwitchCheck(1,1,0)==-1);fixture_online=0;
    fp->victim_gobj=&fixture_gobj;assert(ScriptGame_StageSwitchCheck(1,1,0)==-9);fp->victim_gobj=NULL;
    /* A removed tracked handle cannot be counted twice against free capacity. */
    script_stage.cap=3;script_switch.nlines=1;script_switch.lines[0]=900;
    assert(ScriptGame_StageSwitchCheck(1,1,0)==-7);script_stage.cap=8;script_switch.nlines=0;
    fixture_fighter.cur_pos=(Vec3){0,0,0};fixture_dirty_contacts(fp);
    a->music=5;
    rc=ScriptGame_StageSwitch(1,1,1000,0);assert(rc==0);
    assert(script_stage_music_pending==5); /* switch queues, never starts in freeze */
    assert(script_switch.active==1 && script_switch.nlines==3 && fixture_safety_active==0);
    {int la=script_stage.base_l+script_stage_seam_slot(script_switch.lines[0]);
      int lb=script_stage.base_l+script_stage_seam_slot(script_switch.lines[1]);
      assert(ScriptGame_StageSlotSameJoint(la,lb)==1);
      a->lines[1].joint=1;assert(ScriptGame_StageSlotSameJoint(la,lb)==0);
      assert(ScriptGame_StageSlotSameJoint(-1,lb)==-1);a->lines[1].joint=0;}
    assert(fp->coll_data.floor_skip==-1 && fp->coll_data.ledge_id_left==-1 && fp->coll_data.ledge_id_right==-1);
    assert(fp->coll_data.left_facing_wall.index==-1 && fp->coll_data.ceiling.index==-1);
    assert(fp->coll_data.env_flags==0 && fp->coll_data.prev_env_flags==0);
    assert(fp->coll_data.prev_pos.x==fp->cur_pos.x && fp->prev_pos.y==fp->cur_pos.y);
    assert(stage_info.blast_zone.left==-50 && ScriptGame_StageSlotFree(1,1)==0);
    a=slot(1,2,1,1);fixture_fighter.cur_pos=(Vec3){200,20,0};
    fp->cpu=(struct CpuFighter){0,7,0};
    assert(ScriptGame_StageSwitch(2,1,2000,0)==0);
    assert(fp->cpu.kind==0 && fp->cpu.level==7 && fp->cpu.output==0);
    {Fighter partner=*fp;HSD_GObj pg={0};int turn;
      pg.user_data=&partner;partner.gobj=&pg;fixture_gobj.next=&pg;
      for(turn=0;turn<10;++turn){fp->cur_pos=partner.cur_pos=(Vec3){200,20,0};
        fp->cpu=(struct CpuFighter){0,7,0};partner.cpu=(struct CpuFighter){0,8,0};
        assert(ScriptGame_StageSwitch(turn&1?1:2,1,5000+turn*1000,0)==0);
        assert(fp->cpu.kind==0 && fp->cpu.output==0 && partner.cpu.kind==0 && partner.cpu.level==8 && partner.cpu.output==0);}
      fixture_gobj.next=NULL;}
    assert(fp->cur_pos.x==10 && fp->cur_pos.y<1 && fp->ground_or_air==GA_Air);
    assert(fixture_stage_item_deleted && !fixture_portable_deleted);
    fp->cur_pos=(Vec3){200,20,0};assert(ScriptGame_StageSwitch(1,1,3000,1)==0);
    assert(fp->cur_pos.y<stage_info.blast_zone.bottom);
    /* Grounded outside a deliberately narrower blast zone still KOs; inside
     * stays put. The rule is independent of finding a floor underneath. */
    a->blast.right=5;fp->cur_pos=(Vec3){8,0,0};fp->ground_or_air=GA_Ground;
    assert(ScriptGame_StageSwitch(2,1,3500,1)==0 && fp->cur_pos.y<a->blast.bottom);
    fp->cur_pos=(Vec3){0,0,0};fp->ground_or_air=GA_Ground;
    assert(ScriptGame_StageSwitch(2,1,3600,1)==0 && fp->cur_pos.y==0);
    fixture_ledge_line=script_stage_seam_slot(script_switch.lines[0]);
    {MapLine* l=&script_stage.map->lines[script_stage.base_l+fixture_ledge_line];
      l->prev_id0=l->next_id0=script_stage.base_l+fixture_ledge_line;}
    fp->motion_id=ftCo_MS_CliffWait;fp->mv.co.cliff.ledge_id=25;fp->cur_pos=(Vec3){0,0,0};
    assert(ScriptGame_StageSwitch(2,1,4000,0)==0 && fp->motion_id==20 && fp->mv.co.cliff.ledge_id==77);
    assert(fixture_ledge_releases==1);fixture_ledge_line=-1;
    {GroundParam scaled=fixture_param;int at;
      scaled.y=0.8f;a=slot(2,3,1,1);a->param=&scaled;
      assert(ScriptGame_StageSwitch(3,1,18000,0)==0);
      at=script_stage_seam_slot(script_switch.lines[0]);
      assert(script_stage.line[at].x0==-8 && script_stage.line[at].x1==8);
      fixture_ledge_line=at;fp->motion_id=ftCo_MS_CliffWait;fp->coll_data.ledge_id_left=10;
      ScriptGame_StageSlotsRelease(1);assert(fixture_ledge_releases==2);fixture_ledge_line=-1;}
    assert(!ScriptGame_StageSlotsActive() && !script_switch.active && !fixture_isolated);
    assert(stage_info.blast_zone.left==-100 && stage_info.param==&fixture_param);
    assert(ScriptGame_StageSlotFree(2,1)==0);
    puts("stage slots: lifetime/full pool/ownership/online/budgets, atomic switch, placement, contacts, items and restore passed");
    return 0;
}
