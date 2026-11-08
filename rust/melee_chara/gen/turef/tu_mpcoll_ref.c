/* C reference for melee_world::tu::mpcoll::mpColl_8004730C: the three game TUs mplib.c, mpisland.c and
 * mpcoll.c are #included verbatim (shipping contraction via run.py); the stage is a synthetic MapCollData
 * (floors with slopes and flat runs, two platforms, a ceiling, two walls) loaded through mpLibLoad;
 * the callees outside the three TUs are stand-ins both sides share. Every case seeds a CollData and a
 * ftCollisionBox from a xorshift stream (quarter-unit grid so no float arithmetic is done by the
 * driver) and prints the result plus every CollData field after mpColl_8004730C.
 * PSVECNormalize / lbVector_* come from the game sources through tu_pre.py's EXTRACTS. */
#include "mp/mplib.c"
#include "mp/mpisland.c"
#include "mp/mpcoll.c"
#include "tu_mpcoll_extract.inc"
#include <stdio.h>

void __assert(const char* f, u32 l, const char* c) { (void) f; (void) l; (void) c; printf("ASSERT\n"); for (;;) {} }
int ScriptGame_StageJointActive(int j) { (void) j; return 1; }
int ScriptGame_StageLineActive(int l) { (void) l; return 1; }
int ScriptGame_StageJoint(int j) { (void) j; return 0; }
int ScriptGame_StageJointCapacity(void) { return 256; }
int ScriptGame_StageSlotJointUpdate(int j) { (void) j; return 0; }
int ScriptGame_StageSlotSameJoint(int a, int b) { return a == b; }
float Ground_801C0498(void) { return 1.0f; }
void grDynamicAttr_801CA0B4(void) {}
void grDynamicAttr_801CA224(void) {}
int grDynamicAttr_801CA284(Vec3* v, int i) { (void) v; return (i % 3 == 1) ? 0x35 + i : 0; }
int Mex_GrLineTypeRow(int k) { return k; }
int ftLib_IsFighter(HSD_GObj* g) { (void) g; printf("UNEXPECTED ftLib\n"); return 0; }
u8 ftLib_GetPlayerIndex(HSD_GObj* g) { (void) g; return 0; }
int ftLib_IsSubFighter(HSD_GObj* g) { (void) g; return 0; }
CollData* ftLib_GetCollData(HSD_GObj* g) { (void) g; return NULL; }
int itGetKind(HSD_GObj* g) { (void) g; return 0; }
void OSReport(char* fmt, ...) { (void) fmt; printf("OSREPORT\n"); }
DbLKind DbLevel;
StageInfo stage_info;

/* Render / camera / stage-bounds callees are not reached by mpColl; /alternatename points them at a stub that says so. */
void unexpected_stub(void) { printf("UNEXPECTED CALL\n"); for (;;) {} }
#pragma comment(linker, "/alternatename:Camera_8002928C=unexpected_stub")
#pragma comment(linker, "/alternatename:Camera_80030B50=unexpected_stub")
#pragma comment(linker, "/alternatename:Camera_80030B7C=unexpected_stub")
#pragma comment(linker, "/alternatename:GXBegin=unexpected_stub")
#pragma comment(linker, "/alternatename:GXClearVtxDesc=unexpected_stub")
#pragma comment(linker, "/alternatename:GXLoadPosMtxImm=unexpected_stub")
#pragma comment(linker, "/alternatename:GXPosition3f32=unexpected_stub")
#pragma comment(linker, "/alternatename:GXSetCullMode=unexpected_stub")
#pragma comment(linker, "/alternatename:GXSetCurrentMtx=unexpected_stub")
#pragma comment(linker, "/alternatename:GXSetLineWidth=unexpected_stub")
#pragma comment(linker, "/alternatename:GXSetVtxAttrFmt=unexpected_stub")
#pragma comment(linker, "/alternatename:GXSetVtxDesc=unexpected_stub")
#pragma comment(linker, "/alternatename:GXSetZMode=unexpected_stub")
#pragma comment(linker, "/alternatename:Ground_801C2D24=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_CObjGetCurrent=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_CObjGetViewingMtx=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_GObj_SetupProc=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_LObjSetupInit=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_SetTevRegAll=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_SetupChannel=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_SetupPEMode=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_SetupTevStage=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_StateAssignTev=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_StateInitTev=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_StateInvalidate=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_StateSetCullMode=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_StateSetNumChans=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_StateSetNumTevStages=unexpected_stub")
#pragma comment(linker, "/alternatename:HSD_StateSetNumTexGens=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetBlastZoneBottomOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetBlastZoneLeftOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetBlastZoneRightOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetBlastZoneTopOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetCamBoundsBottomOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetCamBoundsLeftOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetCamBoundsRightOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:Stage_GetCamBoundsTopOffset=unexpected_stub")
#pragma comment(linker, "/alternatename:lb_8000B1CC=unexpected_stub")
HSD_GObj** HSD_GObjPLinkHead;
CmSubject* cm_804D6468;

static u8 HEAP[1 << 20];
static size_t HEAP_TOP;
void* HSD_MemAlloc(int n)
{
    void* p = HEAP + HEAP_TOP;
    HEAP_TOP += ((size_t) n + 15) & ~(size_t) 15;
    return p;
}
void HSD_Free(void* p) { (void) p; }
void memzero(void* p, ssize_t n) { unsigned char* b = p; while (n--) *b++ = 0; }
HSD_GObj* GObj_Create(u16 a, u8 b, u8 c) { (void) a; (void) b; (void) c; return NULL; }
void PSMTXMultVec(Mtx m, Vec* a, Vec* b) { (void) m; (void) a; (void) b; printf("UNEXPECTED MultVec\n"); }
u32 HSD_JObjGetFlags(HSD_JObj* j) { (void) j; return 0; }
void HSD_JObjSetupMatrixSub(HSD_JObj* j) { (void) j; }

static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 s = 0x8004730Cu;
static u32 rnd(void) { s ^= s << 13; s ^= s >> 17; s ^= s << 5; return s; }
/* quarter-unit grid value n/4 (exact) */
static float qf(int n) { return (float) n * 0.25f; }

/* joint callbacks (mpJointSetCb1 / mpJointSetCb2) record what mpCollEnd hands them */
typedef struct { int tag; unsigned ud; int joint; int x50; int kind; u32 dy; } CBREC;
static CBREC CBS[32];
static int NCB;
static void cb_a(void* ud, int joint, CollData* c, int x50, mpLib_GroundEnum kind, float dy)
{
    CBREC* r = &CBS[NCB++];
    (void) c;
    r->tag = 1; r->ud = (unsigned) (size_t) ud; r->joint = joint; r->x50 = x50; r->kind = (int) kind; r->dy = B(dy);
}
static void cb_b(void* ud, int joint, CollData* c, int x50, mpLib_GroundEnum kind, float dy)
{
    CBREC* r = &CBS[NCB++];
    (void) c;
    r->tag = 2; r->ud = (unsigned) (size_t) ud; r->joint = joint; r->x50 = x50; r->kind = (int) kind; r->dy = B(dy);
}

static Vec2 VERTS[32];
static MapLine LINES[32];
static MapJoint JOINTS[1];
static MapCollData MCD;
static int NV, NL;

static void vtx(float x, float y) { VERTS[NV].x = x; VERTS[NV].y = y; NV++; }
static void line(int v0, int v1, int prev, int next, u16 hi, u16 lo)
{
    MapLine* l = &LINES[NL++];
    l->v0_idx = v0; l->v1_idx = v1;
    l->prev_id0 = prev; l->next_id0 = next; l->prev_id1 = -1; l->next_id1 = -1;
    l->hi_flags = hi; l->lo_flags = lo;
}

static void build_world(void)
{
    int i;
    float y = 0.0f;
    /* floor chain: points 0..8, lines 0..7 */
    for (i = 0; i <= 8; i++) {
        if (i == 0 || (rnd() % 3) != 0) {
            y = qf((int) (rnd() % 33) - 16);
        }
        vtx(qf(-320 + 80 * i), y);
    }
    for (i = 0; i < 8; i++) {
        line(i, i + 1, i == 0 ? -1 : i - 1, i == 7 ? -1 : i + 1,
             CollLine_Floor | ((i == 0 || i == 7) ? LINE_FLAG_LEDGE : 0), rnd() % 16);
    }
    /* two platforms: points 9..12, lines 8..9 */
    vtx(-40.0f, 18.0f); vtx(-10.0f, 18.0f); vtx(10.0f, 30.0f); vtx(40.0f, 30.0f);
    line(9, 10, -1, -1, CollLine_Floor | LINE_FLAG_PLATFORM, 1);
    line(11, 12, -1, -1, CollLine_Floor | LINE_FLAG_PLATFORM, 2);
    /* ceiling: points 13..15, lines 10..11 (right to left) */
    vtx(80.0f, 70.0f); vtx(0.0f, 60.0f); vtx(-80.0f, 70.0f);
    line(13, 14, -1, 11, CollLine_Ceiling, 3);
    line(14, 15, 10, -1, CollLine_Ceiling, 3);
    /* walls: points 16..17; the LeftWall group has normal -x (hit moving right): line 12 at x=+80 going up; line 13 the RightWall group at x=-80 going down */
    vtx(80.0f, 40.0f); vtx(-80.0f, 40.0f);
    line(8, 16, -1, -1, CollLine_LeftWall, 4);
    line(17, 0, -1, -1, CollLine_RightWall, 5);

    JOINTS[0].ranges[MapLineGroup_Floor].start = 0;
    JOINTS[0].ranges[MapLineGroup_Floor].count = 10;
    JOINTS[0].ranges[MapLineGroup_Ceiling].start = 10;
    JOINTS[0].ranges[MapLineGroup_Ceiling].count = 2;
    JOINTS[0].ranges[MapLineGroup_LeftWall].start = 12;
    JOINTS[0].ranges[MapLineGroup_LeftWall].count = 1;
    JOINTS[0].ranges[MapLineGroup_RightWall].start = 13;
    JOINTS[0].ranges[MapLineGroup_RightWall].count = 1;
    JOINTS[0].ranges[MapLineGroup_Dynamic].start = 14;
    JOINTS[0].ranges[MapLineGroup_Dynamic].count = 0;
    JOINTS[0].left_bound = -100.0f; JOINTS[0].bottom_bound = -50.0f;
    JOINTS[0].right_bound = 100.0f; JOINTS[0].top_bound = 80.0f;
    JOINTS[0].vtx_start = 0; JOINTS[0].vtx_count = NV;
    MCD.verts = VERTS; MCD.vert_count = NV; MCD.lines = LINES; MCD.line_count = NL;
    for (i = 0; i < MapLineGroup_Count; i++) {
        MCD.ranges[i] = JOINTS[0].ranges[i];
    }
    MCD.joints = JOINTS; MCD.joint_count = 1;
}

static void pecb(const ftECB* e)
{
    printf(" %08x %08x %08x %08x %08x %08x %08x %08x", B(e->top.x), B(e->top.y), B(e->bottom.x), B(e->bottom.y),
           B(e->right.x), B(e->right.y), B(e->left.x), B(e->left.y));
}
static void psurf(const SurfaceData* d)
{
    printf(" %d %u %08x %08x %08x", d->index, (unsigned) d->flags, B(d->normal.x), B(d->normal.y), B(d->normal.z));
}
static void pvec(const Vec3* v) { printf(" %08x %08x %08x", B(v->x), B(v->y), B(v->z)); }

static void dump(int i, int r, CollData* c)
{
    printf("%d %d", i, r);
    pvec(&c->cur_pos); pvec(&c->prev_pos); pvec(&c->last_pos); pvec(&c->x28_vec);
    printf(" f %d %d %d %d %d %d %d %d %d %d", c->x34_flags.b0, c->x34_flags.b1234, c->x34_flags.b5, c->x34_flags.b6,
           c->x34_flags.b7, c->x35_flags.b0, c->x35_flags.b1234, c->x35_flags.b5, c->x35_flags.b6, c->x35_flags.b7);
    printf(" %d %d %d %d %d %d %d", (int) c->facing_dir, c->x38, c->floor_skip, c->ledge_id_right, c->ledge_id_left,
           c->joint_id_skip, c->joint_id_only);
    printf(" %08x %08x %08x %08x %08x", B(c->x50), B(c->ledge_snap_x), B(c->ledge_snap_y), B(c->ledge_snap_height),
           B(c->lstick_x));
    pecb(&c->x64_ecb); pecb(&c->desired_ecb); pecb(&c->ecb); pecb(&c->prev_ecb); pecb(&c->xE4_ecb);
    printf(" %u %d %d %d", (unsigned) c->x130_flags, c->env_flags, c->prev_env_flags, c->x13C);
    pvec(&c->contact); psurf(&c->floor); psurf(&c->left_facing_wall); psurf(&c->right_facing_wall);
    psurf(&c->ceiling);
    printf(" cb %d", NCB);
    {
        int k;
        for (k = 0; k < NCB; k++) {
            printf(" %d %u %d %d %d %08x", CBS[k].tag, CBS[k].ud, CBS[k].joint, CBS[k].x50, CBS[k].kind, CBS[k].dy);
        }
    }
    printf("\n");
}

static void rbits(ECBFlagStruct* f)
{
    f->b0 = rnd() & 1; f->b1234 = rnd() % 16; f->b5 = rnd() & 1; f->b6 = rnd() & 1; f->b7 = rnd() & 1;
}
static void recb(ftECB* e)
{
    float w = qf(12 + (int) (rnd() % 12));
    e->top.x = 0.0f; e->top.y = qf(30 + (int) (rnd() % 30));
    e->bottom.x = 0.0f; e->bottom.y = qf((int) (rnd() % 9) - 4);
    e->right.x = w; e->right.y = qf(24 + (int) (rnd() % 20));
    e->left.x = -qf(12 + (int) (rnd() % 12)); e->left.y = e->right.y;
}

int main(void)
{
    static CollData C;
    static ftCollisionBox BOX;
    int i, r;
    build_world();
    mpLibLoad(&MCD);
    mpJointSetCb1(0, (void*) (size_t) 0x11, cb_a);
    mpJointSetCb2(0, (void*) (size_t) 0x22, cb_b);
#ifdef DBG
    { Vec3 o, n; int lid; u32 fl; int k;
      for (k = 0; k < 10; k++) { printf("line %d flags %x\n", k, groundCollLine[k].flags); }
      printf("joint flags %x\n", jointListStart->flags);
      printf("vtx0 %f %f\n", groundCollVtx[0].pos.x, groundCollVtx[0].pos.y);
      printf("floor hit %d\n", mpCheckFloor(0.0f, 50.0f, 0.0f, -50.0f, 0.0f, &o, &lid, &fl, &n, -1, -1, -1, NULL, NULL));
      printf("%f %f lid %d\n", o.x, o.y, lid);
      printf("LW l->r %d r->l %d\n", mpCheckLeftWall(-90.0f, 20.0f, -70.0f, 20.0f, &o, &lid, &fl, &n, -1, -1), mpCheckLeftWall(-70.0f, 20.0f, -90.0f, 20.0f, &o, &lid, &fl, &n, -1, -1));
      printf("RW l->r %d r->l %d\n", mpCheckRightWall(70.0f, 20.0f, 90.0f, 20.0f, &o, &lid, &fl, &n, -1, -1), mpCheckRightWall(90.0f, 20.0f, 70.0f, 20.0f, &o, &lid, &fl, &n, -1, -1));
      printf("LW@R l->r %d r->l %d\n", mpCheckLeftWall(70.0f, 20.0f, 90.0f, 20.0f, &o, &lid, &fl, &n, -1, -1), mpCheckLeftWall(90.0f, 20.0f, 70.0f, 20.0f, &o, &lid, &fl, &n, -1, -1));
      printf("RW@L l->r %d r->l %d\n", mpCheckRightWall(-90.0f, 20.0f, -70.0f, 20.0f, &o, &lid, &fl, &n, -1, -1), mpCheckRightWall(-70.0f, 20.0f, -90.0f, 20.0f, &o, &lid, &fl, &n, -1, -1));
    }
#endif
    for (i = 0; i < 6000; i++) {
        int dx, dy;
        {
            int mode = (int) (rnd() % 10);
            int v;
            if (mode == 0) {
                C.cur_pos.x = qf((int) (rnd() % 761) - 380);
                C.cur_pos.y = qf((int) (rnd() % 421) - 120);
                dx = (int) (rnd() % 65) - 32; dy = (int) (rnd() % 65) - 32;
            } else if (mode <= 3) {
                /* a floor vertex (or a platform end), arriving from above */
                v = (rnd() & 1) ? (int) (rnd() % 9) : 9 + (int) (rnd() % 4);
                C.cur_pos.x = VERTS[v].x + qf((int) (rnd() % 9) - 4);
                C.cur_pos.y = VERTS[v].y + qf((int) (rnd() % 9) - 4);
                dx = (int) (rnd() % 9) - 4; dy = -(int) (rnd() % 25);
            } else if (mode <= 5) {
                /* the stage walls (x = +-80) and the floor ends, moving sideways */
                v = (int) (rnd() % 4);
                v = v == 0 ? 8 : v == 1 ? 0 : v == 2 ? 16 : 17;
                C.cur_pos.x = VERTS[v].x + qf((int) (rnd() % 49) - 24);
                C.cur_pos.y = VERTS[v].y + qf((int) (rnd() % 81) - 8);
                dx = (int) (rnd() % 49) - 24; dy = (int) (rnd() % 9) - 4;
            } else if (mode <= 7) {
                /* under the ceiling, moving up */
                v = 13 + (int) (rnd() % 3);
                C.cur_pos.x = VERTS[v].x + qf((int) (rnd() % 49) - 24);
                C.cur_pos.y = VERTS[v].y - qf((int) (rnd() % 49));
                dx = (int) (rnd() % 9) - 4; dy = (int) (rnd() % 25);
            } else {
                v = (int) (rnd() % NV);
                C.cur_pos.x = VERTS[v].x + qf((int) (rnd() % 49) - 24);
                C.cur_pos.y = VERTS[v].y + qf((int) (rnd() % 49) - 24);
                if (rnd() & 1) {
                    dx = (int) (rnd() % 65) - 32; dy = (int) (rnd() % 65) - 32;
                } else {
                    dx = (int) (rnd() % 13) - 6; dy = (int) (rnd() % 13) - 6;
                }
            }
        }
        C.cur_pos.z = 0.0f;
        C.last_pos.x = C.cur_pos.x - qf(dx);
        C.last_pos.y = C.cur_pos.y - qf(dy);
        C.last_pos.z = 0.0f;
        C.prev_pos.x = C.last_pos.x - qf((int) (rnd() % 25) - 12);
        C.prev_pos.y = C.last_pos.y - qf((int) (rnd() % 25) - 12);
        C.prev_pos.z = 0.0f;
        C.x28_vec.x = qf((int) (rnd() % 41) - 20); C.x28_vec.y = qf((int) (rnd() % 41) - 20); C.x28_vec.z = 0.0f;
        rbits(&C.x34_flags); rbits(&C.x35_flags);
        C.facing_dir = (rnd() & 1) ? 1 : -1;
        C.x38 = (int) (rnd() % 5);
        C.floor_skip = (rnd() & 1) ? -1 : (int) (rnd() % 14);
        C.ledge_id_right = (rnd() & 1) ? -1 : (int) (rnd() % 14);
        C.ledge_id_left = (rnd() & 1) ? -1 : (int) (rnd() % 14);
        C.joint_id_skip = (rnd() % 4 == 0) ? 0 : -1;
        C.joint_id_only = (rnd() % 4 == 0) ? 0 : -1;
        C.x50 = qf((int) (rnd() % 17) - 8);
        C.ledge_snap_x = qf((int) (rnd() % 17) - 8);
        C.ledge_snap_y = qf((int) (rnd() % 17) - 8);
        C.ledge_snap_height = qf((int) (rnd() % 17) - 8);
        C.lstick_x = qf((int) (rnd() % 9) - 4);
        recb(&C.x64_ecb); recb(&C.desired_ecb); recb(&C.ecb); recb(&C.prev_ecb); recb(&C.xE4_ecb);
        if (rnd() & 1) {
            C.prev_ecb = C.ecb;
        }
        C.x130_flags = (rnd() % 4 == 0 ? CollData_X130_Clear : 0) | (rnd() % 4 == 0 ? CollData_X130_Locked : 0) |
                       (rnd() & 0xF);
        C.env_flags = (int) (rnd() & 0xFFFF);
        C.prev_env_flags = (int) (rnd() & 0xFFFF);
        C.x13C = (int) (rnd() % 3);
        C.contact.x = qf((int) (rnd() % 17) - 8); C.contact.y = qf((int) (rnd() % 17) - 8); C.contact.z = 0.0f;
        C.floor.index = (int) (rnd() % 14) - 1; C.floor.flags = rnd() & 0xFF;
        C.floor.normal.x = qf((int) (rnd() % 5) - 2); C.floor.normal.y = qf((int) (rnd() % 5) - 2); C.floor.normal.z = 0.0f;
        C.left_facing_wall = C.floor; C.right_facing_wall = C.floor; C.ceiling = C.floor;
        C.left_facing_wall.index = (int) (rnd() % 14) - 1;
        C.right_facing_wall.index = (int) (rnd() % 14) - 1;
        C.ceiling.index = (int) (rnd() % 14) - 1;
        BOX.top = qf(30 + (int) (rnd() % 30)); BOX.bottom = (rnd() & 1) ? 0.0f : qf((int) (rnd() % 9) - 4);
        BOX.left.x = -qf(12 + (int) (rnd() % 12)); BOX.left.y = qf(24 + (int) (rnd() % 20));
        BOX.right.x = qf(12 + (int) (rnd() % 12)); BOX.right.y = qf(24 + (int) (rnd() % 20));
        NCB = 0;
        r = mpColl_8004730C(&C, &BOX);
        dump(i, r, &C);
    }
    return 0;
}
