#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <limits.h>
static int music_calls, music_id=-1, callback, frozen, ticks;
static int lbAudioAx_80023F28(int id) {
    assert(!callback && !frozen); ++music_calls; music_id=id; return 0;
}
static int lbAudioAx_StageSlotMusic(void) {return music_id;}
#define OSReport(...) ((void)0)
#include "stage_music_queue_retail.inc"
int main(void) {
    frozen=callback=1;
    ScriptGame_StageMusicRequest(5); assert(!music_calls);
    callback=0;
    for(ticks=0;ticks<60;++ticks) {
        ScriptGame_StageMusicTick(!frozen); assert(!music_calls);
    }
    frozen=0; ScriptGame_StageMusicTick(1);
    assert(music_calls==1 && music_id==5);
    ScriptGame_StageMusicTick(1); assert(music_calls==1);
    ScriptGame_StageMusicRequest(6); ScriptGame_StageMusicRequest(7);
    ScriptGame_StageMusicTick(1); assert(music_calls==2 && music_id==7);
    ScriptGame_StageMusicRequest(8); ScriptGame_StageMusicReset();
    ScriptGame_StageMusicTick(1); assert(music_calls==2);
    ScriptGame_StageMusicRequest(9); ScriptGame_StageMusicRequest(-1);
    ScriptGame_StageMusicTick(1); assert(music_calls==2);
    puts("switch music: 60 frozen main-loop ticks, thaw-only start, latest request, scene cancellation passed");
    return 0;
}
