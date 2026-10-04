#include <assert.h>
#include <stdio.h>
#include <string.h>
static int reads, enabled=1;
static int bits(float f){int n;memcpy(&n,&f,4);return n;}
int gw_EchoCopyStyle(int port,int sub,int copy,int field){
 assert(port==0&&sub==1&&copy==2);++reads;
 if(!enabled)return -1;
 if(field==0)return 8;
 return bits(field==5?.8f:field==6?.2f:field==7?1.f:field==8?0.f:1.f);
}
#include "../platform/gw_script_echo_render.h"
int main(void){
 GwEchoCopyStyle s=gw_echo_copy_capture(0,1,2);float tint[4]={.5f,.5f,.5f,.5f},strength=2.f;
 assert(s.described&&s.age==8&&reads==6);enabled=0;
 gw_echo_copy_apply(&s,tint,&strength);assert(reads==6&&strength>1.59f&&strength<1.61f&&tint[0]==.2f&&tint[1]==1.f);
 s=gw_echo_copy_capture(0,1,2);assert(!s.described&&reads==7);
 strength=2.f;tint[0]=.5f;gw_echo_copy_apply(&s,tint,&strength);assert(strength==2.f&&tint[0]==.5f&&reads==7);
 puts("echo copy FIFO projection: captured game-thread values, no render-thread game reads PASS");return 0;
}
