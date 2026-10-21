/* Game-side TU for the night-3 round-5 oracle (gen_ground_r5_ref.sh). Compiled with the game's
 * flags (contraction on, then lower.py). ground_r5_body.inc holds, cut verbatim:
 * from gr/ground.c the Gr_CObj_Max line, Ground_MexParamStkind, Ground_801C2AD8, 801C2AE8,
 * GetMapGObj, 801C2BBC, 801C2BD4, 801C2C8C, 801C2CF4, 801C2D0C; grRCruise_80201918,
 * grBigBlue_801EF7D8, grIceMt_801FA728; Ground_801C20D0, 205C, 2090, 498C, 1D38. Stand-in types carry only the members the bodies touch
 * (layout does not matter: the driver feeds and reads them by name). OSReport / HSD_ASSERT
 * go to the driver: HSD_ASSERT sets a flag (it is the last statement), the OSReport + spin
 * case of 801C2AE8 is predicted by the driver and not run. */
#include <Runtime/platform.h>
typedef int ssize_t;
#define ARRAY_SIZE(a) (sizeof(a) / sizeof((a)[0]))
typedef struct HSD_GObj { u16 classifier; struct HSD_GObj* next; void* hsd_obj; void* user_data; } HSD_GObj;
typedef void (*HSD_GObjEvent)(HSD_GObj*);
typedef struct { u8 r, g, b, a; } GXColor;
typedef struct HSD_Fog { int type; GXColor color; } HSD_Fog;
#define GET_FOG(gobj) ((HSD_Fog*) (gobj)->hsd_obj)
#define HSD_GOBJ_PLINK_LIGHT 3
#define HSD_GOBJ_CLASS_GROUND 13
extern HSD_GObj* HSD_GObjPLinkHead[16];
typedef HSD_GObj Ground_GObj;
typedef struct HSD_JObj HSD_JObj;
typedef int StKind;
typedef struct { float x, y, z; } RVec3;
#define Vec3 RVec3
typedef struct StageParam { StKind stkind; s32 x4, x8; u32 xC, x10; s16 x14, x16, x18; } StageParam;
typedef struct GroundParam { s16 x68; StageParam* stage_params; s32 stage_param_count; } GroundParam;
typedef struct Ground {
    u8 pad[0xC]; HSD_GObjEvent xC_callback; int map_id;
    union { struct { u8 b; Vec3 x4, x10, x1C; } scroll; struct { s16 a[8]; f32 x10; } icemt; } u;
} Ground;
struct StageInfoR5 { struct { f32 cam_vertical_tilt; } cam_info; HSD_GObj* x12C; s32 xA0; GroundParam* param; Ground_GObj* map_gobjs[64]; HSD_JObj* x280[261]; void* x694[4]; };
struct StageInfoR5 stage_info;
#define GET_GROUND(gobj) ((Ground*) (gobj)->user_data)
void OSReport(const char*, ...);
void r5_assert(int line);
#define HSD_ASSERT(line, cond) ((cond) ? (void) 0 : r5_assert(line))
StKind Stage_80225194(void);
Ground_GObj* Ground_GetMapGObj(int map_id);
#include "ground_r5_body.inc"
/* entry points for the driver (the cut statics) */
StKind r5_mex(StKind s) { return Ground_MexParamStkind(s); }
void r5_2bbc(Ground_GObj* g, int i) { Ground_801C2BBC(g, i); }
void r5_2bd4(void* a) { Ground_801C2BD4(a); }
void r5_1d38(HSD_GObj* g) { Ground_801C1D38(g); }
double __frsqrte(double x) { return x; }
