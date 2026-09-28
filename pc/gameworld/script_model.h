/* Scalar field protocol shared by the native Lua API and retargeted game TU. */
#ifndef SCRIPT_MODEL_H
#define SCRIPT_MODEL_H
#define SCRIPT_MESH_ASSETS 32
#define SCRIPT_MESH_INSTANCES 256
#define SCRIPT_MESH_LINES 32
enum { SM_X, SM_Y, SM_Z, SM_ROT, SM_SCALE, SM_LAYER, SM_VISIBLE, SM_TINT,
       SM_SX, SM_SY, SM_SZ, SM_ALPHA, SM_FIELDS };
#define SM_FLOAT_FIELD(i) ((i) <= SM_SCALE || ((i) >= SM_SX && (i) <= SM_SZ))
#define SM_IS_ALPHA(m) ((m)->field[SM_ALPHA] || (((m)->field[SM_TINT] & 255) != 255))
/* Transform fields are float bits; layer, visible and RGBA tint are integers. */
typedef struct {
    int handle, asset, field[SM_FIELDS], count, dynamic;
    float center[3]; /* immutable mesh bounds centre, mirrored with its transform */
    int handle, asset, field[SM_FIELDS], count, area;
    struct { float x0, y0, x1, y1; int kind, flags, handle; } line[SCRIPT_MESH_LINES];
} ScriptMeshInstance;
#endif
