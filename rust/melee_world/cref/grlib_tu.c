/* grlib.c's PointInsideColl, cut verbatim by gen_grlib_ref.sh into grlib_body.inc, over a
 * stand-in CollData holding only the fields it reads. Compiled with the game flags. */
typedef float f32;
typedef struct { float x, y; } Vec2;
typedef struct { float x, y, z; } Vec3;
typedef struct { Vec2 top, bottom, right, left; } ftECB;
typedef struct { Vec3 cur_pos; ftECB ecb; } CollData;
typedef int bool;
#define true 1
#define false 0
#define ABS(x) ((x) < 0 ? -(x) : (x))
#include "grlib_body.inc"
bool ref_point_inside(CollData* cd, Vec3* p, float offset) { return PointInsideColl(cd, p, offset); }
