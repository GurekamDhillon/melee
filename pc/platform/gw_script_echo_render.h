#ifndef GW_SCRIPT_ECHO_RENDER_H
#define GW_SCRIPT_ECHO_RENDER_H
#include <string.h>
#ifdef __cplusplus
extern "C" {
#endif
int gw_EchoCopyStyle(int port,int sub,int copy,int field);
#ifdef __cplusplus
}
#endif
/* Capture only on the game thread, before copying the motion FIFO payload.
 * The GX recording thread projects this value and never calls a game helper. */
typedef struct {int described,age;float brightness,tint[4];} GwEchoCopyStyle;
static inline GwEchoCopyStyle gw_echo_copy_capture(int port,int sub,int copy){
 GwEchoCopyStyle s={0};int i,b;s.age=gw_EchoCopyStyle(port,sub,copy,0);
 if(s.age<0)return s;
 s.described=1;b=gw_EchoCopyStyle(port,sub,copy,5);memcpy(&s.brightness,&b,4);
 for(i=0;i<4;++i){b=gw_EchoCopyStyle(port,sub,copy,6+i);memcpy(&s.tint[i],&b,4);}
 return s;
}
static inline void gw_echo_copy_apply(const GwEchoCopyStyle* s,float tint[4],float* strength){
 if(s->described){*strength*=s->brightness;memcpy(tint,s->tint,sizeof s->tint);}
}
#endif
