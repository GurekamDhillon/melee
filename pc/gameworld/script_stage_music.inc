/* Stage changes happen between frozen logic frames. Never start a DVD stream
 * there or from Lua; the normal native loop consumes this latest-only request. */
static int script_stage_music_pending=-1;
void ScriptGame_StageMusicRequest(int id)
{
    /* An unspecified destination/unknown original track still cancels an old
     * queued destination. Otherwise unload before thaw can play stale music. */
    script_stage_music_pending=id>=0?id:-1;
}
void ScriptGame_StageMusicReset(void)
{script_stage_music_pending=-1;}
void ScriptGame_StageMusicTick(int ready)
{
    int id,unchanged;
    if(!ready || script_stage_music_pending<0)return;
    id=script_stage_music_pending;script_stage_music_pending=-1;
    unchanged=lbAudioAx_80023F28(id);
    OSReport("stage switch: deferred music requested=%d current=%d restarted=%d\n",
             id,lbAudioAx_StageSlotMusic(),!unchanged);
}
