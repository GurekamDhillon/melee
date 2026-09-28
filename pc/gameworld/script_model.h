/* Scalar field protocol shared by the native Lua API and retargeted game TU. */
#ifndef SCRIPT_MODEL_H
#define SCRIPT_MODEL_H
#define SCRIPT_MESH_ASSETS 32
#define SCRIPT_MESH_INSTANCES 128
#define SCRIPT_MESH_LINES 32
enum { SM_X, SM_Y, SM_Z, SM_ROT, SM_SCALE, SM_LAYER, SM_VISIBLE, SM_TINT, SM_FIELDS };
/* Transform fields are float bits; layer, visible and RGBA tint are integers. */
typedef struct {
    int handle, asset, field[SM_FIELDS], count;
    struct { float x0, y0, x1, y1; int kind, flags, handle; } line[SCRIPT_MESH_LINES];
} ScriptMeshInstance;
#endif
