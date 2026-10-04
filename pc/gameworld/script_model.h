/* Scalar field protocol shared by the native Lua API and retargeted game TU. */
#ifndef SCRIPT_MODEL_H
#define SCRIPT_MODEL_H
#include <math.h>
#define SCRIPT_MESH_ASSETS 128
#define SCRIPT_MESH_INSTANCES 256
#define SCRIPT_MESH_LINES 64
#define SCRIPT_MESH_OWNERS 64
enum { SM_X, SM_Y, SM_Z, SM_ROT, SM_SCALE, SM_LAYER, SM_VISIBLE, SM_TINT,
       SM_SX, SM_SY, SM_SZ, SM_ALPHA, SM_BACKGROUND, SM_RX, SM_RY, SM_FIELDS };
#define SM_FLOAT_FIELD(i) ((i) <= SM_SCALE || ((i) >= SM_SX && (i) <= SM_SZ) || (i) == SM_RX || (i) == SM_RY)
#define SM_IS_ALPHA(m) ((m)->field[SM_ALPHA] || (((m)->field[SM_TINT] & 255) != 255))
/* Transform fields are float bits; layer, visible and RGBA tint are integers. */
typedef struct {
    int handle, asset, field[SM_FIELDS], count, dynamic, area, owner; /* area: named-map-area slot; owner: refcounted resource token */
    float center[3]; /* immutable mesh bounds centre, mirrored with its transform */
    struct { float x0, y0, x1, y1; int kind, flags, handle; } line[SCRIPT_MESH_LINES];
} ScriptMeshInstance;
/* Right-handed Euler degrees: scale, Rx, Ry, then legacy Z rotation.
 * Kept shared so CPU batches, replay matrices and alpha centres agree. */
static void script_model_rotation(float rx, float ry, float rz, float r[3][3])
{
    float d = 0.017453292519943295f;
    float cx = cosf(rx*d), sx = sinf(rx*d);
    float cy = cosf(ry*d), sy = sinf(ry*d);
    float cz = cosf(rz*d), sz = sinf(rz*d);
    r[0][0] = cz*cy; r[0][1] = cz*sy*sx-sz*cx; r[0][2] = cz*sy*cx+sz*sx;
    r[1][0] = sz*cy; r[1][1] = sz*sy*sx+cz*cx; r[1][2] = sz*sy*cx-cz*sx;
    r[2][0] = -sy; r[2][1] = cy*sx; r[2][2] = cy*cx;
}
#endif
