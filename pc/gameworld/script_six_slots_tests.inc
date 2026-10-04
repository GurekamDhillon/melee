int ScriptGame_SixSlotsTest(void)
{
    static Fighter fp;
    int saved[2][9], seen[2][7], i,k, rc=0;
    memcpy(saved,script_virtual_pad,sizeof saved);
    memcpy(seen,script_virtual_seen,sizeof seen);
    memset(&fp,0,sizeof fp); fp.player_id=5;
    ScriptGame_VirtualPadRelease(-1,0);
    ScriptGame_VirtualPadField(5,0,123,1);
    ScriptGame_VirtualPadField(5,1,2,1);
    ScriptGame_VirtualPadField(5,2,0x100,1);
    ScriptGame_VirtualPadField(5,3,-127,1);
    if (!ScriptGame_VirtualPadApply(&fp) || fp.cpu.buttons!=0x100 || fp.cpu.lstick.x!=-127) rc++;
    if (ScriptGame_VirtualPadSeen(5,0)!=0x100 || ScriptGame_VirtualPadSeen(5,1)!=-127) rc++;
    ScriptGame_VirtualPadConsume();
    if (ScriptGame_VirtualPadField(5,1,0,0)!=1) rc++;
    ScriptGame_VirtualPadRelease(5,999);
    if (!ScriptGame_VirtualPadApply(&fp)) rc++;
    ScriptGame_VirtualPadConsume();
    if (!ScriptGame_VirtualPadApply(&fp) || fp.cpu.buttons || fp.cpu.lstick.x) rc++;
    ScriptGame_VirtualPadRelease(5,123);
    if (ScriptGame_VirtualPadApply(&fp)) rc++;
    /* Out-of-range writes cannot alter either virtual fighter. */
    for (i=-1; i<8; ++i) if (i!=4 && i!=5)
        for (k=-1; k<10; ++k) ScriptGame_VirtualPadField(i,k,999,1);
    for (i=0; i<2; ++i) for (k=0;k<9;++k)
        if (script_virtual_pad[i][k]) rc++;
    memcpy(script_virtual_pad,saved,sizeof saved);
    memcpy(script_virtual_seen,seen,sizeof seen);
    {
        static CmSubject camera;
        fp.x890_cameraBox=&camera; fp.kind=Ft_Kind_Fox;
        fp.motion_id=ftCo_MS_Sleep; fp.x221F_b3=1;
        if (script_recycle_policy(&fp,5,-1,2,0,1,0)) rc++;
        if (script_recycle_policy(&fp,5,9,2,0,1,0)!=2) rc++;
        if (script_recycle_policy(&fp,5,2,2,1,1,0)!=3) rc++;
        if (script_recycle_policy(&fp,5,2,2,0,0,0)!=4) rc++;
        if (script_recycle_policy(&fp,5,2,2,0,1,1)!=5) rc++;
        fp.motion_id=ftCo_MS_Rebirth;
        if (script_recycle_policy(&fp,5,2,2,0,1,0)) rc++;
        fp.motion_id=ftCo_MS_RebirthWait;
        if (script_recycle_policy(&fp,5,2,2,0,1,0)) rc++;
        fp.motion_id=ftCo_MS_DeadDown;
        if (script_recycle_policy(&fp,5,2,2,0,1,0)!=5) rc++;
        fp.motion_id=ftCo_MS_Wait;
        if (script_recycle_policy(&fp,5,2,2,0,1,0)!=5) rc++;
        if (script_recycle_policy(&fp,6,2,2,0,1,0)!=1) rc++;
    }
    return rc;
}
