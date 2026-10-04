/* Pair policy fixture: native initialization is injected, never needs disc data. */
static int script_cpu_mode_test_calls;
static void script_cpu_mode_test_init(Fighter* fp, int kind, int level, int control)
{
    script_cpu_mode_test_calls++;
    fp->cpu.kind=fp->kind==Ft_Kind_Nana ? CpuKind_Nana : kind;
    fp->cpu.level=level; fp->cpu.x14=control;
}
static int script_cpu_mode_test_slot_kind;
static void script_cpu_mode_test_persist(int slot,int kind)
{
    script_cpu_mode_test_slot_kind=kind;
}
int ScriptGame_CpuModeTest(void)
{
    static Fighter zelda,sheik,nana;
    static CmSubject a,b;
    Fighter* pair[2]; ScriptBench saved=script_bench[0];
    int rc=0;
    memset(&zelda,0,sizeof zelda); memset(&sheik,0,sizeof sheik);
    memset(&nana,0,sizeof nana); memset(&a,0,sizeof a); memset(&b,0,sizeof b);
    memset(&script_bench[0],0,sizeof script_bench[0]);
    zelda.kind=Ft_Kind_Zelda; sheik.kind=Ft_Kind_Seak; nana.kind=Ft_Kind_Nana;
    zelda.x890_cameraBox=&a; sheik.x890_cameraBox=&b;
    zelda.cpu.kind=sheik.cpu.kind=CpuKind_4;
    zelda.cpu.level=7; sheik.cpu.level=3;
    zelda.cpu.x14=11; sheik.cpu.x14=22;
    sheik.motion_id=ftCo_MS_Sleep; sheik.x221F_b3=1; sheik.invisible=1;
    pair[0]=&zelda; pair[1]=&sheik; script_cpu_mode_test_calls=0;
    if (!script_cpu_mode_pair(pair,0,script_cpu_mode_test_init) ||
        script_cpu_mode_test_calls!=2 || zelda.cpu.kind!=CpuKind_0 || sheik.cpu.kind!=CpuKind_0 ||
        zelda.cpu.level!=7 || sheik.cpu.level!=3 || zelda.cpu.x14!=11 || sheik.cpu.x14!=22 ||
        sheik.motion_id!=ftCo_MS_Sleep || !sheik.x221F_b3 || !sheik.invisible) rc++;
    pair[0]=&sheik; pair[1]=&zelda; /* accessor order after a transform */
    if (pair[0]->cpu.kind!=CpuKind_0 ||
        !script_cpu_mode_pair(pair,1,script_cpu_mode_test_init) ||
        zelda.cpu.kind!=CpuKind_4 || sheik.cpu.kind!=CpuKind_4) rc++;
    pair[0]=&zelda; pair[1]=&nana; nana.is_sub_fighter=1;
    script_cpu_mode_test_calls=0;
    if (!script_cpu_mode_pair(pair,0,script_cpu_mode_test_init) ||
        script_cpu_mode_test_calls!=2 || zelda.cpu.kind!=CpuKind_0 || nana.cpu.kind!=CpuKind_Nana) rc++;
    pair[1]=&sheik; sheik.cpu.level=10; script_cpu_mode_test_calls=0;
    if (script_cpu_mode_pair(pair,1,script_cpu_mode_test_init) || script_cpu_mode_test_calls) rc++;
    sheik.cpu.level=3;
    script_bench[0].owner=1;
    script_bench_save(&script_bench[0].entity[0],&zelda,0);
    script_bench_save(&script_bench[0].entity[1],&sheik,1);
    if (!script_cpu_mode_pair(pair,0,script_cpu_mode_test_init) ||
        zelda.cpu.kind!=CpuKind_5 || sheik.cpu.kind!=CpuKind_5 ||
        script_bench[0].entity[0].cpu.kind!=CpuKind_0 ||
        script_bench[0].entity[1].cpu.kind!=CpuKind_0) rc++;
    script_bench_restore(&script_bench[0].entity[0],&zelda);
    script_bench_restore(&script_bench[0].entity[1],&sheik);
    if (zelda.cpu.kind!=CpuKind_0 || sheik.cpu.kind!=CpuKind_0 || !sheik.x221F_b3) rc++;
    memset(&script_bench[0],0,sizeof script_bench[0]);
    script_cpu_mode_test_slot_kind=CpuKind_4;
    if (!script_cpu_mode_persist(pair,0,0,script_cpu_mode_test_init,
                                  script_cpu_mode_test_persist)) rc++;
    /* Exact ordering from Fighter reset: initialize from player slot again. */
    script_cpu_mode_test_init(&zelda,script_cpu_mode_test_slot_kind,7,0);
    script_cpu_mode_test_init(&sheik,script_cpu_mode_test_slot_kind,3,0);
    if(zelda.cpu.kind!=CpuKind_0 || sheik.cpu.kind!=CpuKind_0) rc++;
    if (!script_cpu_mode_persist(pair,0,1,script_cpu_mode_test_init,
                                  script_cpu_mode_test_persist)) rc++;
    script_cpu_mode_test_init(&zelda,script_cpu_mode_test_slot_kind,7,0);
    if(zelda.cpu.kind!=CpuKind_4) rc++;
    sheik.cpu.level=10;
    if(script_cpu_mode_persist(pair,0,0,script_cpu_mode_test_init,
                                script_cpu_mode_test_persist) ||
       script_cpu_mode_test_slot_kind!=CpuKind_4) rc++;
    {
        int kinds[]={Ft_Kind_Zelda,Ft_Kind_Seak,Ft_Kind_Nana,Ft_Kind_Fox};
        int k,frame;
        for (k=0; k<4; ++k) {
            zelda.kind=kinds[k]; zelda.self_vel.x=3; zelda.cur_pos.y=10;
            zelda.motion_id=ftCo_MS_Wait; zelda.x221F_b3=0;
            for (frame=0; frame<900; ++frame) {
                zelda.cpu.buttons=HSD_PAD_B; zelda.cpu.lstick.y=-100;
                zelda.cpu.cstick.x=100; zelda.cpu.ltrigger=255; zelda.cpu.rtrigger=255;
                zelda.cpu.xF8_b5=1; zelda.cpu.csP=zelda.cpu.buffer;
                zelda.cpu.command_duration=20; zelda.cpu.xC8=zelda.cpu.xEC=2;
                zelda.input.held_buttons[0]=HSD_PAD_B; zelda.input.lstick[0].y=-1;
                if (!script_cpu_stand_quiet(&zelda,1) || zelda.cpu.buttons ||
                    zelda.cpu.lstick.y || zelda.cpu.cstick.x || zelda.cpu.ltrigger ||
                    zelda.cpu.rtrigger || zelda.cpu.xF8_b5 || zelda.cpu.csP ||
                    zelda.cpu.command_duration || zelda.cpu.xC8 || zelda.cpu.xEC ||
                    zelda.input.held_buttons[0] || zelda.input.lstick[0].y ||
                    zelda.self_vel.x!=3 || zelda.cur_pos.y!=10 || zelda.x221F_b3) rc++;
            }
        }
        zelda.cpu.buttons=HSD_PAD_B;
        if (script_cpu_stand_quiet(&zelda,0) || zelda.cpu.buttons!=HSD_PAD_B) rc++;
    }
    script_bench[0]=saved;
    return rc;
}
