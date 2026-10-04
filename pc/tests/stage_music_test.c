#include <assert.h>
#include <stdio.h>
#include <string.h>
enum{VOL_MAX=127};static char cur_hps_stem[128];static int stops,starts;
static void lbAudioAx_800236DC(void){++stops;}
static void fn_80023ED4(const char* s,int volume,int loop){assert(strstr(s,".hps") && volume==127 && loop==1);++starts;}
#include "stage_music_retail.inc"
int main(void){strcpy(cur_hps_stem,"audio/old.hps");
    assert(lbAudioAx_80023F28_helper1("audio/new.hps")==0 && stops==1 && starts==1);
    assert(!strcmp(cur_hps_stem,"audio/new.hps"));
    assert(lbAudioAx_80023F28_helper1("audio/new.hps")==1 && stops==1 && starts==1);
    puts("stage music: retail helper stops old, starts changed HPS and retains same track passed");return 0;}
