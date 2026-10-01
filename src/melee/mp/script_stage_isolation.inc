/* Included at the end of mplib.c on PC. Original ranges are disabled without
 * unlinking joints, changing material/callback state, or touching appended lines.
 * Keeping the linked-list order avoids a new floor priority after restoration. */
#include <string.h>
#include "../../../pc/gameworld/script_stage_isolation.h"

static struct {
    MapCollData* map;
    CollLine* lines;
    CollJoint* joints;
    int nlines, njoints;
    u32 line_flags[1536], joint_flags[256];
    struct MapLineRange ranges[256][MapLineGroup_Count];
} mp_script_isolation;

int mpScriptStageIsolationActive(void)
{
    return mp_script_isolation.map != NULL &&
        mp_script_isolation.map == mpLib_804D64B4 &&
        mp_script_isolation.lines == groundCollLine &&
        mp_script_isolation.joints == groundCollJoint;
}

int mpScriptStageIsolationSet(int lines, int joints, int isolate)
{
    int i, k;
    if (!isolate) {
        /* A replaced/freed scene cannot be restored through stale heap pointers. */
        if (mpScriptStageIsolationActive()) {
            for (i = 0; i < mp_script_isolation.nlines; ++i)
                groundCollLine[i].flags = mp_script_isolation.line_flags[i];
            for (i = 0; i < mp_script_isolation.njoints; ++i) {
                groundCollJoint[i].flags = mp_script_isolation.joint_flags[i];
                for (k = 0; k < MapLineGroup_Count; ++k)
                    groundCollJoint[i].inner->ranges[k] = mp_script_isolation.ranges[i][k];
            }
        }
        memset(&mp_script_isolation, 0, sizeof mp_script_isolation);
        return 1;
    }
    if (mp_script_isolation.map) return mpScriptStageIsolationActive();
    if (!mpLib_804D64B4 || !groundCollLine || !groundCollJoint ||
        lines <= 0 || joints <= 0 || lines > 1536 || joints > 256 ||
        lines > mpLib_804D64B4->line_count || joints > mpLib_804D64B4->joint_count)
        return 0;
    /* Validate every range before writing any state. A malformed original joint
     * must never hide or capture a script's reserved collision range. */
    for (i = 0; i < joints; ++i) {
        if (groundCollJoint[i].inner != &mpLib_804D64B4->joints[i]) return 0;
        for (k = 0; k < MapLineGroup_Count; ++k) {
            struct MapLineRange* r = &groundCollJoint[i].inner->ranges[k];
            if (r->count < 0 || (r->count && (r->start < 0 || r->start + r->count > lines))) return 0;
        }
    }
    mp_script_isolation.map = mpLib_804D64B4;
    mp_script_isolation.lines = groundCollLine;
    mp_script_isolation.joints = groundCollJoint;
    mp_script_isolation.nlines = lines;
    mp_script_isolation.njoints = joints;
    for (i = 0; i < lines; ++i) {
        mp_script_isolation.line_flags[i] = groundCollLine[i].flags;
        groundCollLine[i].flags &= ~LINE_FLAG_ENABLED;
    }
    for (i = 0; i < joints; ++i) {
        mp_script_isolation.joint_flags[i] = groundCollJoint[i].flags;
        for (k = 0; k < MapLineGroup_Count; ++k) {
            mp_script_isolation.ranges[i][k] = groundCollJoint[i].inner->ranges[k];
            groundCollJoint[i].inner->ranges[k].count = 0;
        }
    }
    return 1;
}

/* Headless fixture exercises the production transform and actual mpCheckFloor.
 * Every native pointer and bounding state is restored, even after failed checks. */
int mpScriptStageIsolationTest(void)
{
    MapCollData map = { 0 };
    MapJoint mj[2] = { 0 };
    MapLine ml[3] = { 0 };
    CollLine cl[3] = { 0 };
    CollJoint cj[2] = { 0 };
    CollVtx cv[6] = { 0 };
    MapCollData* saved_map = mpLib_804D64B4;
    CollLine* saved_lines = groundCollLine;
    CollJoint* saved_joints = groundCollJoint;
    CollVtx* saved_verts = groundCollVtx;
    CollJoint* saved_start = jointListStart;
    CollJoint* saved_end = jointListEnd;
    bool saved_bounds = didCheckBounding;
    struct MapLineRange ranges[MapLineGroup_Count];
    u32 lf[2], jf;
    int i, k, hit = -1, failed = 0;
    /* Never steal a live isolation snapshot just to execute a test. */
    if (mp_script_isolation.map) return 1;
    map.lines = ml; map.line_count = 3; map.joints = mj; map.joint_count = 2;
    for (i = 0; i < 3; ++i) {
        ml[i].v0_idx = i * 2; ml[i].v1_idx = i * 2 + 1;
        ml[i].prev_id0 = ml[i].prev_id1 = ml[i].next_id0 = ml[i].next_id1 = -1;
        cl[i].x0 = &ml[i]; cl[i].flags = CollLine_Floor | LINE_FLAG_ENABLED;
        cv[i * 2].pos.x = -10; cv[i * 2 + 1].pos.x = 10;
        cv[i * 2].pos.y = cv[i * 2 + 1].pos.y = i == 2 ? -5 : 0;
    }
    /* An intentionally disabled native ceiling must stay disabled after restore. */
    cl[1].flags = CollLine_Ceiling | LINE_FLAG_HIDDEN;
    mj[0].ranges[MapLineGroup_Floor].count = 1;
    mj[0].ranges[MapLineGroup_Ceiling].start = 1;
    mj[0].ranges[MapLineGroup_Ceiling].count = 1;
    mj[1].ranges[MapLineGroup_Floor].start = 2;
    mj[1].ranges[MapLineGroup_Floor].count = 1;
    for (i = 0; i < 2; ++i) { cj[i].inner = &mj[i]; cj[i].flags = CollJoint_Enabled | CollJoint_B11; }
    cj[0].next = &cj[1];
    for (k = 0; k < MapLineGroup_Count; ++k) ranges[k] = mj[0].ranges[k];
    lf[0] = cl[0].flags; lf[1] = cl[1].flags; jf = cj[0].flags;
    mpLib_804D64B4 = &map; groundCollLine = cl; groundCollJoint = cj;
    groundCollVtx = cv; jointListStart = cj; jointListEnd = &cj[1];
    didCheckBounding = true;
    if (!mpCheckFloor(0, 5, 0, -1, 0, NULL, &hit, NULL, NULL, -1, -1, -1, NULL, NULL) || hit != 0) failed++;
    if (mpScriptStageIsolationSet(4, 1, 1) || mpScriptStageIsolationSet(2, 3, 1)) failed++;
    mj[0].ranges[MapLineGroup_Ceiling].count = 2;
    if (mpScriptStageIsolationSet(2, 1, 1) || cl[0].flags != lf[0]) failed++;
    mj[0].ranges[MapLineGroup_Ceiling].count = 1;
    if (!mpScriptStageIsolationSet(2, 1, 1) || !mpScriptStageIsolationActive()) failed++;
    if (mpCheckFloor(0, 5, 0, -1, 0, NULL, &hit, NULL, NULL, -1, -1, -1, NULL, NULL)) failed++;
    if (!mpCheckFloor(0, 5, 0, -10, 0, NULL, &hit, NULL, NULL, -1, -1, -1, NULL, NULL) || hit != 2) failed++;
    if (cj[0].next != &cj[1] || jointListStart != cj || jointListEnd != &cj[1]) failed++;
    /* A custom line can change while isolated; its current flags survive restore. */
    cl[2].flags |= LINE_FLAG_PLATFORM;
    if (!mpScriptStageIsolationSet(2, 1, 1)) failed++;
    mpScriptStageIsolationSet(0, 0, 0);
    if (mpScriptStageIsolationActive() || cl[0].flags != lf[0] || cl[1].flags != lf[1] || cj[0].flags != jf) failed++;
    for (k = 0; k < MapLineGroup_Count; ++k)
        if (memcmp(&ranges[k], &mj[0].ranges[k], sizeof ranges[k])) failed++;
    if (!(cl[2].flags & LINE_FLAG_PLATFORM) || mj[1].ranges[MapLineGroup_Floor].count != 1) failed++;
    if (!mpCheckFloor(0, 5, 0, -1, 0, NULL, &hit, NULL, NULL, -1, -1, -1, NULL, NULL) || hit != 0) failed++;
    /* Repeated teardown and map replacement cannot write to an expired scene. */
    mpScriptStageIsolationSet(0, 0, 0);
    if (!mpScriptStageIsolationSet(2, 1, 1)) failed++;
    mpLib_804D64B4 = NULL;
    mpScriptStageIsolationSet(0, 0, 0);
    if (mp_script_isolation.map) failed++;
    mpLib_804D64B4 = saved_map; groundCollLine = saved_lines;
    groundCollJoint = saved_joints; groundCollVtx = saved_verts;
    jointListStart = saved_start; jointListEnd = saved_end; didCheckBounding = saved_bounds;
    return failed;
}
