/* Game-side TU for the night-3 round-4 oracle (gen_forward_r4_ref.sh). Compiled with the
 * game's flags (contraction on, then lower.py). forward_r4_body.inc holds the forwarders cut
 * verbatim from the stage TUs (gen_forward_r4.py, renamed <tu>__<name>); the four gr/inlines.h
 * inlines they call are cut verbatim from inlines.h into forward_r4_inl.inc. Stand-in types
 * carry only the members the bodies touch; the callees are the harness's recorders. */
#include <Runtime/platform.h>

typedef struct HSD_GObj HSD_GObj;
struct HSD_GObj { void* hsd_obj; void* user_data; };
typedef HSD_GObj Ground_GObj;
typedef struct HSD_JObj HSD_JObj;
typedef int enum_t;
typedef enum_t GrKind;
typedef struct Ground { u8 pad[0x14]; GrKind map_id; } Ground;
typedef struct grZakoGenerator_SpawnDesc grZakoGenerator_SpawnDesc;
static inline void* HSD_GObjGetUserData(HSD_GObj* gobj) { return gobj->user_data; }
static inline void* HSD_GObjGetHSDObj(HSD_GObj* gobj) { return gobj->hsd_obj; }
#define GET_GROUND(gobj) ((Ground*) HSD_GObjGetUserData(gobj))
#define GET_JOBJ(gobj) ((HSD_JObj*) HSD_GObjGetHSDObj(gobj))
HSD_GObj* grZakoGenerator_801CAE04(grZakoGenerator_SpawnDesc*);
bool Ground_UpdateMapColl(Ground_GObj*);
bool Ground_InitMapColl(HSD_JObj* jobj, s32 map_id);
void grAnime_801C8138(HSD_GObj*, enum_t, bool);
void lb_800115F4(void);
#include "forward_r4_inl.inc"
#include "forward_r4_body.inc"
double __frsqrte(double x) { return x; }
