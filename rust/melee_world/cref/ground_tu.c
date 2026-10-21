/* Game-side TU for the ground.c oracle (gen_ground_ref.sh). Compiled with the game's flags
 * (contraction on, then lower.py) and ground_body.inc: Ground_801C4B50, Ground_801C5630,
 * Ground_801C5694, Ground_801C5AEC cut verbatim from src/melee/gr/ground.c. The callees
 * outside lbvector/lbtrigf/gekko_fp are stand-ins that feed or record the driver's values. */
#include <Runtime/platform.h>
#include <math.h>
#include "lb/lbvector.h"

typedef struct HSD_Spline HSD_Spline;
typedef struct { u8 pad[0x20]; s32 x20[8]; } Ground;
Vec3 ref_spline_point, ref_tangent;
int ref_last_id, ref_last_val;
void splGetSplinePoint(Vec3* out, HSD_Spline* s, f32 t) { (void) s; (void) t; *out = ref_spline_point; }
void lbShadow_8000E9F0(Vec3* out, HSD_Spline* s, f32 t) { (void) s; (void) t; *out = ref_tangent; }
void lbAudioAx_80024B58(int id, int v) { ref_last_id = id; ref_last_val = v; }
void lbAudioAx_80024B1C(int id, int v) { ref_last_id = id; ref_last_val = v; }
#include "ground_body.inc"
