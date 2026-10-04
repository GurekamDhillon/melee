#include <assert.h>
#include <stdio.h>
#define TARGET_PC 1
static volatile int HSD_Synth_804D7778=1;
static int completions;
void wait_idle(void) {
    /* Header DVD -> hako header DVD -> hako data AR callback clears busy. */
    if(++completions==3)HSD_Synth_804D7778=0;
}
int main(void) {
#include "stage_synth_wait_retail.inc"
    assert(completions==3 && !HSD_Synth_804D7778);
    puts("retail stream-start wait: three deferred completion layers serviced passed");
    return 0;
}
