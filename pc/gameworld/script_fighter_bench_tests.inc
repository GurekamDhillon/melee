/* Fake pair validates the atomic safety policy and reversible freeze flags. */
static void script_bench_cpu_changed(Fighter* fp);
static int script_bench_entity_read(Fighter* fp, int benched, int dormant, int field);
static int script_bench_test_floor_on;
static int script_bench_test_floor(float x,float y,Vec3* p,int* line,u32* flags,Vec3* n)
{
    p->x=x; p->y=y-0.25f; *line=3; *flags=0;
    n->x=n->z=0; n->y=1; return script_bench_test_floor_on;
}
static void script_bench_test_fall(Fighter* fp) { fp->ground_or_air=GA_Air; fp->motion_id=ftCo_MS_Fall; }
static void script_bench_test_ground(Fighter* fp) { fp->ground_or_air=GA_Ground; }
static void script_bench_test_wait(HSD_GObj* gobj) { GET_FIGHTER(gobj)->motion_id=ftCo_MS_Wait; }
static void script_bench_test_camera(Fighter* fp) { fp->x890_cameraBox->pos=fp->cur_pos; }
int ScriptGame_FighterBenchTest(void)
{
    static Fighter a,b;
    static CmSubject ca,cb;
    static HSD_GObj ga;
    static const ScriptBenchOps ops={script_bench_test_floor,script_bench_test_fall,
        script_bench_test_ground,script_bench_test_wait,script_bench_test_camera};
    ScriptBenchEntity sa,sb;
    ScriptBench saved=script_bench[0];
    int rc=0;
    memset(&a,0,sizeof a); memset(&b,0,sizeof b);
    memset(&ca,0,sizeof ca); memset(&cb,0,sizeof cb);
    a.motion_id=b.motion_id=ftCo_MS_Wait;
    a.x890_cameraBox=&ca; b.x890_cameraBox=&cb;
    a.cpu.kind=CpuKind_4; b.cpu.kind=CpuKind_5;
    /* Do not let the mocked entry hide an animation-helper/entry mix-up. */
    if (script_bench_ops.wait!=ft_8008A2BC) rc++;
    a.cpu.x18=4; a.cpu.x24=123;
    b.is_sub_fighter=1; b.invisible=1;
    if (script_bench_refused(&a,0,0) || script_bench_refused(&b,0,0)) rc++;
    b.motion_id=ftCo_MS_ThrownF;
    if (!script_bench_refused(&b,0,0)) rc++;
    b.motion_id=ftCo_MS_Wait;
    a.victim_gobj=(void*)&b;
    if (!script_bench_refused(&a,0,0)) rc++;
    a.victim_gobj=NULL; a.motion_id=ftCo_MS_RebirthWait;
    if (!script_bench_refused(&a,0,0)) rc++;
    a.motion_id=ftCo_MS_Wait; a.dmg.x195c_hitlag_frames=1;
    if (!script_bench_refused(&a,0,0)) rc++;
    a.dmg.x195c_hitlag_frames=0;
    a.motion_id=ftCo_MS_Attack11;
    if (script_bench_refused(&a,0,0)!=7) rc++;
    a.motion_id=ftCo_MS_Wait;
    script_bench_save(&sa,&a,0); script_bench_save(&sb,&b,0);
    script_bench_assert(&a); script_bench_assert(&b);
    if (!a.x221F_b3 || !a.x2219_b1 || !a.invisible ||
        ca.state!=CmSubjectState_Inactive || a.cpu.kind!=CpuKind_5) rc++;
    a.x221F_b3=0; script_bench_assert(&a); /* every-frame reassertion */
    a.cpu.x18=10; a.cpu.x24=0; /* disabled controller must not leak its brain */
    if (!a.x221F_b3) rc++;
    script_bench_restore(&sa,&a); script_bench_restore(&sb,&b);
    if (a.x221F_b3 || a.x2219_b1 || a.invisible || a.cpu.kind!=CpuKind_4 ||
        !b.invisible || b.cpu.kind!=CpuKind_5 || a.cpu.x18!=4 || a.cpu.x24!=123) rc++;
    script_bench[0].owner=1; script_bench[0].entity[0]=sa;
    a.player_id=0; a.cpu.kind=CpuKind_0; a.cpu.x18=4; a.cpu.x24=300;
    script_bench_cpu_changed(&a); /* cpu_mode selected while benched */
    if (a.cpu.kind!=CpuKind_5) rc++;
    script_bench_restore(&script_bench[0].entity[0],&a);
    if (a.cpu.kind!=CpuKind_0 || a.cpu.x18!=4 || a.cpu.x24!=300) rc++;
    a.cpu.kind=CpuKind_4; a.cpu.x18=10;
    script_bench_cpu_changed(&a);
    script_bench_restore(&script_bench[0].entity[0],&a);
    if (a.cpu.kind!=CpuKind_4 || a.cpu.x18!=10) rc++;
    script_bench[0]=saved;
    {
        HSD_GObj** old_heads=HSD_GObjPLinkHead;
        static HSD_GObj* heads[64];
        static HSD_GObj target;
        memset(heads,0,sizeof heads); memset(&target,0,sizeof target);
        target.user_data=&b; heads[HSD_GOBJ_PLINK_FIGHTER]=&target;
        HSD_GObjPLinkHead=heads;
        a.cpu.x44=&b; a.cpu.xF0=&b;
        a.cpu.x4C=(Item*)&target; a.cpu.xF4=(Item*)&target;
        a.cpu.xC8=a.cpu.xEC=2; a.cpu.command_duration=20;
        a.cpu.csP=a.cpu.buffer; a.cpu.write_pos=a.cpu.buffer+8;
        a.cpu.x444=&a.cpu.xFC[5]; a.cpu.x448=a.cpu.xFC;
        script_bench_save(&sa,&a,0);
        script_bench_restore(&sa,&a);
        if (a.cpu.x44!=&b || a.cpu.xF0!=&b || a.cpu.x4C || a.cpu.xF4) rc++;
        heads[HSD_GOBJ_PLINK_FIGHTER]=NULL; /* target despawned while benched */
        script_bench_restore(&sa,&a);
        if (a.cpu.x44 || a.cpu.xF0 || a.cpu.x4C || a.cpu.xF4 ||
            a.cpu.xC8 || a.cpu.xEC || a.cpu.command_duration || a.cpu.csP ||
            a.cpu.write_pos!=a.cpu.buffer || a.cpu.kind!=CpuKind_4 ||
            a.cpu.x444!=&a.cpu.xFC[5] || a.cpu.x448!=a.cpu.xFC) rc++;
        HSD_GObjPLinkHead=old_heads;
    }
    /* The dormant transform half stays frozen and invisible after restoration. */
    b.kind=Ft_Kind_Zelda; b.x221F_b3=1; b.motion_id=ftCo_MS_Sleep;
    if (script_bench_refused(&b,0,1)) rc++;
    script_bench_save(&sb,&b,1); script_bench_assert(&b); script_bench_restore(&sb,&b);
    if (!b.x221F_b3 || !b.invisible) rc++;
    a.kind=Ft_Kind_Zelda; a.motion_id=ftZd_MS_SpecialLw;
    if (script_bench_refused(&a,0,0)!=6) rc++;
    if (script_bench_entity_read(&a,0,0,2)!=ftZd_MS_SpecialLw ||
        script_bench_entity_read(&a,0,0,13)!=6 ||
        script_bench_entity_read(&b,1,1,11)!=1 ||
        script_bench_entity_read(&b,1,1,2)!=ftCo_MS_Sleep) rc++;
    b.kind=Ft_Kind_Nana; b.motion_id=ftCo_MS_Attack11; b.x221F_b3=0;
    if (script_bench_entity_read(&b,0,0,1)!=Ft_Kind_Nana ||
        script_bench_entity_read(&b,0,0,13)!=7) rc++;
    b.kind=Ft_Kind_Seak; b.motion_id=ftSk_MS_SpecialAirLw;
    if (script_bench_entity_read(&b,0,0,13)!=6 ||
        script_bench_entity_read(NULL,0,0,0)!=0) rc++;
    memset(&ga,0,sizeof ga); ga.user_data=&a; a.gobj=&ga;
    a.input.held_buttons[2]=1; a.input.lstick[2].x=1;
    a.active_timer.lstick.x=0; a.x67C=0; a.wall_jump_input_timer=0;
    a.fall_fast=1; a.self_vel.x=4; a.x8c_kb_vel.y=7;
    a.cpu.buttons=1; a.cpu.lstick.x=100; a.cpu.cstick.y=-100;
    a.xF8_playerNudgeVel.x=2;
    script_bench_test_floor_on=1;
    script_bench_place(&a,20,10,-1,0,&ops);
    if (a.motion_id!=ftCo_MS_Wait || a.ground_or_air!=GA_Ground ||
        a.cur_pos.y!=9.75f || a.coll_data.floor.index!=3 || a.facing_dir!=-1 ||
        a.self_vel.x || a.x8c_kb_vel.y || a.xF8_playerNudgeVel.x || a.fall_fast ||
        a.input.held_buttons[2] || a.input.lstick[2].x ||
        a.active_timer.lstick.x!=254 || a.x67C!=255 || a.wall_jump_input_timer!=254 ||
        a.cpu.buttons || a.cpu.lstick.x || a.cpu.cstick.y ||
        a.x890_cameraBox->pos.x!=20) rc++;
    {
        int cycle;
        for (cycle=0; cycle<20; ++cycle) {
            script_bench_place(&a,20,10,-1,0,&ops);
            if (a.motion_id!=ftCo_MS_Wait || a.ground_or_air!=GA_Ground ||
                script_bench_refused(&a,0,0)) rc++;
        }
    }
    script_bench_test_floor_on=0; a.fall_fast=1;
    script_bench_place(&a,30,40,1,0,&ops);
    if (a.motion_id!=ftCo_MS_Fall || a.ground_or_air!=GA_Air || a.cur_pos.y!=40 ||
        a.coll_data.cur_pos.x!=30 || a.fall_fast ||
        (a.coll_data.x130_flags & CollData_X130_Locked)) rc++;
    return rc;
}
