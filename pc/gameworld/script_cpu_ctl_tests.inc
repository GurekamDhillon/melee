/* The CPU virtual controller on a fixture fighter: no match, no disc. */
int ScriptGame_CpuCtlTest(void)
{
    static Fighter fp, target;
    ScriptCpuCtl c;
    int rc = 0, i;
    static const s8 bytes[] = { CpuCmd_PressA, CpuCmd_WaitFor, 2, CpuCmd_ReleaseA, CpuCmd_SetLstickX, 100 };
    /* the numbers gd.cpu_script compiles are the retail enum's */
    if (CpuCmd_PressA != 1 || CpuCmd_ReleaseAll != 25 || CpuCmd_Done != 0x7F ||
        CpuCmd_SetLstickX != 0x80 || CpuCmd_SetLtrigger != 0x85 || CpuCmd_PressAFor != 0x86 ||
        CpuCmd_ReleaseYFor != 0x8D || CpuCmd_WaitFor != 0x8E || CpuCmd_LstickTowardDestination != 0x8F ||
        CpuCmd_LstickXTowardDestination != 0x90 || CpuCmd_LstickXForward != 0x91 ||
        CpuCmd_WaitIfMotionId != 0x92 || CpuCmd_Unk0x93 != 0x93 || CpuCmd_LstickTowardFighter != 0x94 ||
        CpuCmd_LstickXTowardFighter != 0x95 || CpuCmd_LstickTowardDestinationClamped != 0xC0 ||
        CpuCmd_LstickXTowardDestinationClamped != 0xC1 || CpuCmd_LstickForwardClamped != 0xC2) rc++;
    if (!script_cpu_idle_policy(0, 3, 0) || !script_cpu_idle_policy(1, 3, 3)) rc++;
    if (ScriptGame_CpuCtl(-1, 0, CPU_CTL_PAD_COMMIT, 1) != 0 || ScriptGame_CpuCtl(6, 0, CPU_CTL_G_ACTIVE, 0) != -1 ||
        ScriptGame_CpuCtl(0, 2, CPU_CTL_G_ACTIVE, 0) != -1 || ScriptGame_CpuAttr(-1, 0) != 0) rc++;
    /* pad hold: two frames of the sample, one neutral frame, then untouched */
    memset(&fp, 0, sizeof fp); memset(&target, 0, sizeof target); memset(&c, 0, sizeof c);
    c.pad[0] = HSD_PAD_A | HSD_PAD_R; c.pad[1] = -127; c.pad[2] = 40; c.pad[3] = 9; c.pad[4] = -9;
    c.pad[5] = 0; c.pad[6] = 255; c.pad_frames = 2;
    script_cpu_ctl_tick(&fp, &c, &target);
    if (fp.cpu.buttons != (HSD_PAD_A | HSD_PAD_R) || fp.cpu.lstick.x != -127 || fp.cpu.lstick.y != 40 ||
        fp.cpu.cstick.x != 9 || fp.cpu.cstick.y != -9 || fp.cpu.rtrigger != 255 || fp.cpu.x44 != &target) rc++;
    script_cpu_ctl_tick(&fp, &c, &target);
    if (fp.cpu.buttons == 0 || c.pad_frames != 0) rc++;
    script_cpu_ctl_tick(&fp, &c, &target);
    if (fp.cpu.buttons != 0 || fp.cpu.lstick.x != 0 || fp.cpu.rtrigger != 0) rc++;
    /* the retail interpreter, driven from the buffer: press A, hold 2, release, stick 100 */
    memset(&fp, 0, sizeof fp); memset(&c, 0, sizeof c);
    fp.cpu.write_pos = fp.cpu.buffer;
    for (i = 0; i < (int) sizeof bytes; ++i) *fp.cpu.write_pos++ = bytes[i];
    ftCo_800B49F4(&fp);
    c.running = 1;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (fp.cpu.buttons != HSD_PAD_A || !c.running) rc++;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (fp.cpu.buttons != HSD_PAD_A || !c.running) rc++;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (fp.cpu.buttons != 0 || fp.cpu.lstick.x != 100 || c.running || c.done_count != 1 || fp.cpu.csP != NULL) rc++;
    /* neutral at the end releases the stick as well */
    memset(&fp, 0, sizeof fp); memset(&c, 0, sizeof c);
    fp.cpu.write_pos = fp.cpu.buffer;
    for (i = 0; i < (int) sizeof bytes; ++i) *fp.cpu.write_pos++ = bytes[i];
    ftCo_800B49F4(&fp);
    c.running = 1; c.neutral_end = 1;
    for (i = 0; i < 3; ++i) script_cpu_ctl_tick(&fp, &c, NULL);
    if (fp.cpu.lstick.x != 0 || c.done_count != 1) rc++;
    /* steering commands read the destination and the target: both live in fp->cpu */
    memset(&fp, 0, sizeof fp); memset(&c, 0, sizeof c);
    fp.cpu.x54.x = 50.0f; fp.cpu.x54.y = 0.0f; fp.cur_pos.x = 10.0f; fp.facing_dir = 1.0f;
    fp.cpu.write_pos = fp.cpu.buffer;
    *fp.cpu.write_pos++ = CpuCmd_LstickXTowardDestination; *fp.cpu.write_pos++ = 90;
    ftCo_800B49F4(&fp); c.running = 1;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (fp.cpu.lstick.x != 90) rc++;
    /* cpu_goto's decisions that need no collision query */
    memset(&fp, 0, sizeof fp); memset(&c, 0, sizeof c);
    fp.cur_pos.x = 10.0f; fp.cur_pos.y = 0.0f; fp.ground_or_air = GA_Ground; fp.motion_id = ftCo_MS_Wait;
    c.go_state = 1; c.go_x = 11.0f; c.go_y = 1.0f; c.go_radius = 2.0f; c.go_timeout = 5;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (c.go_state != 2 || fp.cpu.lstick.x != 0) rc++;                 /* inside the radius, 1 above: arrived */
    memset(&c, 0, sizeof c); c.go_state = 1; c.go_x = 90.0f; c.go_radius = 2.0f; c.go_timeout = 1; c.go_ticks = 1;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (c.go_state != 3) rc++;                                         /* past the timeout: blocked */
    memset(&c, 0, sizeof c); c.go_state = 1; c.go_x = 90.0f; c.go_radius = 2.0f; c.go_timeout = 50;
    fp.motion_id = ftCo_MS_DeadLeft;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (c.go_state != 4) rc++;                                         /* a KO: fell */
    fp.motion_id = ftCo_MS_Wait;
    memset(&c, 0, sizeof c); c.go_state = 1; c.go_x = 90.0f; c.go_radius = 2.0f; c.go_timeout = 50; c.pad_frames = 1;
    c.pad[1] = 33;
    script_cpu_ctl_tick(&fp, &c, NULL);
    if (c.go_state != 1 || fp.cpu.lstick.x != 33) rc++;                /* a pad hold outranks the walk */
    if (script_goto_stick_for(40.0f, 0.0f, 0.1f, 2.0f) != 127 || script_goto_stick_for(-15.0f, 0.0f, 0.1f, 2.0f) != -64 ||
        script_goto_stick_for(0.5f, 0.0f, 0.1f, 2.0f) != 0 || script_goto_stick_for(20.0f, 2.0f, 0.1f, 2.0f) != 0) rc++;
    fp.co_attrs.gravity = 0.25f; fp.co_attrs.jump_v_initial_velocity = 3.0f; fp.co_attrs.air_jump_v_multiplier = 1.0f;
    if (script_goto_reach(&fp, 0) != 18.0f || script_goto_reach(&fp, 1) != 36.0f) rc++;
    return rc;
}
