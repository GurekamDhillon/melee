/* Game-side TU for the night-3 round-2 leaf oracle (gen_leaves_r2_ref.sh). Compiled with the
 * game's flags (contraction on, then lower.py) and leaves_r2_body.inc: the bodies below cut
 * verbatim from src/melee/gr and src/melee/if, with `static` / `static inline` dropped so the
 * driver can call them. Stand-in types carry only the members the bodies touch. */
#include <Runtime/platform.h>

typedef struct { f32 x, y, z; } Vec;
typedef Vec Vec3;
typedef struct { u8 pad[0x28]; void* user_data; } HSD_GObj;
typedef HSD_GObj Ground_GObj;
typedef struct {
    u8 pad[0xC4];
    union {
        struct { u16 xC4; s16 xC6; } inishie2;
        struct { s16 xC4; s16 xC6; } taru;
    } u;
} Ground;
#define GET_GROUND(g) ((Ground*) (g)->user_data)
s8 ifStatus_804D6D61;
#include "leaves_r2_body.inc"
