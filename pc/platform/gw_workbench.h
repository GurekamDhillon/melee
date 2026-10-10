/* Workbench diagnostics: native gate/payload writer and a pointer-free frame schema.
 * Native code assembles this schema from read-only game scalar accessors.
 * Nothing here is read by fighter simulation or rollback. */
#ifndef GW_WORKBENCH_H
#define GW_WORKBENCH_H
#include <stddef.h>

int gw_Wb_Enabled(void);
void gw_Wb_SetEnabled(int on);
const char *gw_Wb_Dir(void);
int gw_Wb_WritePayload(const char *name, const char *data, size_t n,
                      char *path_out, size_t cap);

typedef struct { int joint, parent; float x, y, z; } GwWbBone;
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
    float ecb_bx, ecb_by, ecb_tx, ecb_ty, floor_y, lowest_y;
} GwWbFrame;
int gw_Wb_FrameJson(const GwWbFrame *f, const char *row, char *out, size_t cap);
/* Host-only orbit math: degrees, +Z front, +X side; target -1 centers the bone
 * bounds, otherwise it is a joint id (not an index in the compact bone array).
 * Returns 0 on success, -1 for an invalid target/pose; never writes the frame. */
int gw_Wb_CameraOrbit(const GwWbFrame *f, int target_joint, float yaw, float pitch,
                     float distance, float eye[3], float interest[3]);
#endif
