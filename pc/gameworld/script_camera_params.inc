/* Engine batch 2: snapshot-owned stage parameter overrides. Standard FOV is
 * seeded from the misnamed cam_vertical_tilt into cm_803BCCA0.x40 at match
 * start; hook its target read as well so a live stage edit takes effect. */
typedef struct {
    int owner, mask;
    float saved[10], value[10];
} ScriptCameraParams;
static ScriptCameraParams script_camera_params;

static float* script_camera_field(int field)
{
    StageCameraInfo* c = &stage_info.cam_info;
    switch (field) {
    case 0: return &c->cam_zoom_rate; case 1: return &c->cam_max_depth;
    case 2: return &c->cam_vertical_tilt;
    case 3: return &c->cam_fixed_zoom; case 4: return &c->cam_track_smooth;
    case 5: return &c->cam_track_ratio; case 6: return &c->cam_vertical_tilt;
    case 7: return &c->cam_pan_degrees; case 8: return &c->x20;
    case 9: return &c->x24;
    }
    return NULL;
}
float ScriptGame_CameraFov(float fallback)
{
    return script_camera_params.mask & (1 << 2) ? script_camera_params.value[2] : fallback;
}
int ScriptGame_CameraParamRead(int field)
{
    float* p = script_camera_field(field);
    return script_mesh_bits(p ? *p : 0);
}
int ScriptGame_CameraParamOwner(void) { return script_camera_params.owner; }
static void script_camera_params_release(int owner)
{
    int i;
    if (!script_camera_params.owner || (owner && script_camera_params.owner != owner)) return;
    for (i = 0; i < 10; ++i) {
        float* p = script_camera_field(i);
        if (p && (script_camera_params.mask & (1 << i))) *p = script_camera_params.saved[i];
    }
    OSReport("script camera params: restored owner=%d\n", script_camera_params.owner);
    memset(&script_camera_params, 0, sizeof script_camera_params);
}
void ScriptGame_CameraParamsRelease(int owner) { script_camera_params_release(owner); }
int ScriptGame_CameraParams(int owner, int mask, int a, int b, int c, int d, int e,
                            int f, int g, int h, int j, int k)
{
    int bits[10], i;
    float v[10];
    static const float lo[10] = {1,1,1,0,0,0,1,-180,0,0};
    static const float hi[10] = {49000,49000,89,10,10,10,89,180,10,10};
    if (owner <= 0 || mask < 0 || mask > 1023 ||
        (script_camera_params.owner && script_camera_params.owner != owner)) return 0;
    if (!mask) { script_camera_params_release(owner); return 1; }
    bits[0]=a; bits[1]=b; bits[2]=c; bits[3]=d; bits[4]=e;
    bits[5]=f; bits[6]=g; bits[7]=h; bits[8]=j; bits[9]=k;
    /* Retail calls this field 'vertical tilt', but gmvs seeds standard FOV
     * from Ground_801C20D0. tilt is a legacy alias, not a second angle. */
    if (mask & (1 << 6)) {
        if ((mask & (1 << 2)) && bits[2]!=bits[6]) return 0;
        bits[2]=bits[6]; mask=(mask & ~(1 << 6)) | (1 << 2);
    }
    for (i = 0; i < 10; ++i) {
        float* p = script_camera_field(i);
        v[i] = mask & (1 << i) ? script_mesh_float(bits[i]) : p ? *p : 30;
        if ((mask & (1 << i)) && !(v[i] >= lo[i] && v[i] <= hi[i])) return 0;
    }
    if (v[0] > v[1]) return 0;
    script_camera_params.owner = owner;
    for (i = 0; i < 10; ++i) {
        float* p = script_camera_field(i);
        if (!(mask & (1 << i))) continue;
        if (!(script_camera_params.mask & (1 << i)))
            script_camera_params.saved[i] = p ? *p : 0;
        script_camera_params.value[i] = v[i];
        if (p) *p = v[i];
    }
    script_camera_params.mask |= mask;
    OSReport("script camera params: owner=%d mask=%d min=%g max=%g yaw=%g pitch=%g\n",
             owner, mask, v[0], v[1], v[8], v[9]);
    return 1;
}
#include "script_camera_params_tests.inc"
