#ifndef GW_MOTION_H
#define GW_MOTION_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
enum { GW_MOTION_AFTERIMAGE=1,GW_MOTION_TRACER=2 };
enum { GW_MOTION_MAX_COPIES=12,GW_MOTION_HISTORY=64,GW_MOTION_MAX_AGE=60,
       GW_MOTION_STATS_COUNT=31 };
enum { GW_ANCHOR_JOINT=0,GW_ANCHOR_HITBOX=1,GW_ANCHOR_HELD=2,GW_ANCHOR_SWORD=3,GW_ANCHOR_ITEM=4,GW_ANCHOR_HITS=5 };
typedef struct {
    int kind,port,sub,anchor,index,item,copies,spacing,lifetime,length,smoothing;
    int blend,trigger,flag,follow,clear_on_respawn,surface,shader,depth;
    float speed,scale,curve,width,taper,intensity,params[4],offset[3];
    float tint[4],tail[4],edge[4];
} GwMotionOptions;
void gw_motion_defaults(GwMotionOptions*,int kind);
int gw_motion_add(unsigned owner,const GwMotionOptions*,const char** error);
int gw_motion_get(unsigned owner,int handle,GwMotionOptions*);
int gw_motion_set(unsigned owner,int handle,const GwMotionOptions*,const char** error);
int gw_motion_remove(unsigned owner,int handle);
void gw_motion_release(unsigned owner);
void gw_motion_frame(int frame,int replay);
void gw_motion_warm(int port,int begin);
void gw_motion_prepare_ribbons(void);
void gw_motion_intensity(float value);
void gw_motion_stats(uint64_t out[GW_MOTION_STATS_COUNT]);
const char* gw_motion_stat_name(unsigned index);
void gw_motion_register_tests(void);
// Scalar game boundary. Reads/copies the big-endian view/root, never writes it.
int gw_MotionFighterBegin(int port,int sub,int entity,int view,int root,int hitlag,int held,int costume,int falls,int speed);
void gw_MotionFighterEnd(void);
void gw_MotionWorldDraw(int view);
#ifdef __cplusplus
}
#endif
#endif
