/* wb.render, the wb.camera presets and view.json are FROZEN (debug only). The viewer is the
 * Blender add-on (docs/superpowers/specs/2026-10-09-geno-workbench-design.md, Revision 1).
 * Do not extend them.
 * Workbench diagnostics: native gate/payload writer and a pointer-free frame schema.
 * Native code assembles this schema from read-only game scalar accessors.
 * Nothing here is read by fighter simulation or rollback. */
#ifndef GW_WORKBENCH_H
#define GW_WORKBENCH_H
#include <stddef.h>

int gw_Wb_Enabled(void);
void gw_Wb_SetEnabled(int on);
const char *gw_Wb_Dir(void);
/* Headless data engine: MELEE_WB_HEADLESS=1 + workbench scene. PreScan runs before Aurora
 * starts (main.c); Headless() is the gate checked after the scene gate is published. */
void gw_Wb_PreScan(int argc, char **argv);
int gw_Wb_HeadlessRequested(void);
int gw_Wb_Headless(void);
int gw_Wb_WritePayload(const char *name, const char *data, size_t n,
                      char *path_out, size_t cap);

typedef struct { int joint, parent; float x, y, z; float m[12]; } GwWbBone; /* m: row-major 3x4 world matrix */
typedef struct {
    int slot, joint, active;
    float ox, oy, oz, radius, wx, wy, wz;
} GwWbHit;
typedef struct { int joint; float ax, ay, az, bx, by, bz, radius; } GwWbHurt;
typedef struct {
    int motion, frame;
    int nbones, nhits, nhurts;
    GwWbBone bones[160];
    GwWbHit hits[8];
    GwWbHurt hurts[24];
    float ecb_bx, ecb_by, ecb_tx, ecb_ty, ecb_lx, ecb_ly, ecb_rx, ecb_ry, floor_y, lowest_y;
    int root_joint, facing; /* first parentless joint; facing +1/-1 */
    float root_m[12];
} GwWbFrame;
int gw_Wb_FrameJson(const GwWbFrame *f, const char *row, char *out, size_t cap);
/* Host-only orbit math: degrees, +Z front, +X side; target -1 centers the bone
 * bounds, otherwise it is a joint id (not an index in the compact bone array).
 * Returns 0 on success, -1 for an invalid target/pose; never writes the frame. */
int gw_Wb_CameraOrbit(const GwWbFrame *f, int target_joint, float yaw, float pitch,
                     float distance, float eye[3], float interest[3]);
/* Widen an automatic lens to include the floor below a nearby target (<=60
 * units), retaining a margin for stage geometry. Never narrows an existing lens. */
float gw_Wb_CameraFloorFov(float floor_y, const float eye[3],
                         const float interest[3], float fov);
#endif
