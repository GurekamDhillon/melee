/* Game-side TU for the night-3 round-3 oracle (gen_trivial_r3_ref.sh). Compiled with the
 * game's flags (contraction on, then lower.py) and two generated includes:
 *   trivial_r3_body.inc  the 568 constant-return / empty gr callbacks (gen_trivial_r3.py),
 *                        verbatim, renamed <tu>__<name> (several TUs share static names);
 *   eflib_r3_body.inc    efLib_SetFlags / PauseAll / ResumeAll cut verbatim from ef/eflib.c.
 * Stand-in types carry only the members the bodies touch. */
#include <Runtime/platform.h>

typedef struct HSD_GObj HSD_GObj;
struct HSD_GObj { HSD_GObj* next; void* user_data; };
typedef HSD_GObj Ground_GObj;
typedef HSD_GObj Item_GObj;
typedef struct HSD_JObj HSD_JObj;
typedef struct Ground Ground;
typedef struct Vec3 Vec3;
typedef struct DynamicsDesc DynamicsDesc;
typedef struct EF_Effect {
    struct EF_Effect* next;
    HSD_GObj* gobj;
    HSD_GObj* parent_gobj;
    void* attach_jobj;
    void* update;
    void* user_data;
    f32 params[3];
    u16 lifetime;
    s8 expire_flags;
    u8 scale_flags;
    u8 state_flags;
    u8 is_async;
    s8 x2A;
    s8 x2B;
} EF_Effect;
#define HSD_GOBJ_PLINK_EFFECT_UNK0 11
#define HSD_GOBJ_PLINK_EFFECT_UNK1 12
#define EF_STATE_ENTERING_PAUSE 0x01
#define EF_STATE_ASYNC 0x80
#define GET_EFFECT(gobj) ((EF_Effect*) (gobj)->user_data)
HSD_GObj* HSD_GObjPLinkHead[16];
#include "trivial_r3_body.inc"
#include "eflib_r3_body.inc"
/* math_ppc.h sqrtf references it; nothing here calls sqrtf. */
double __frsqrte(double x) { return x; }
