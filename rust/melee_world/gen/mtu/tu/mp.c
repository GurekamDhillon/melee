/* TU: src/melee/mp/{mplib,mpcoll,mpisland}.c as one translation unit */
/* C oracle for the Rust ports of these TUs: the three files included verbatim; every callee out of
 * them is a recording stub. Written by gen/mtu/mk_mp.py. Each line: op case ret state-hash trace-hash. */
#include TU_SRC
#define __assert mtu_unused_assert
#include "mtu_common.h"
#undef __assert
extern void exit(int);
extern char* getenv(const char*);
extern void setbuf(void*, char*);
extern void* __acrt_iob_func(unsigned);
extern void RaiseException(unsigned long, unsigned long, unsigned long, const void*);
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; RaiseException(0xE0A55E87ul, 1, 0, 0); }
void* memset(void* d, int c, unsigned long n) { unsigned char* p = (unsigned char*) d; while (n--) *p++ = (unsigned char) c; return d; }
void memzero(void* d, int n) { unsigned char* p = (unsigned char*) d; while (n-- > 0) *p++ = 0; }
int __HI(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (c.u >> 32); }
int __LO(double x) { union { double d; u64_t_ u; } c; c.d = x; return (int) (u32) c.u; }
#define NLN 24
#define NVX 64
#define NJ 3
static MapCollData M;
static Vec2 VERTS[NVX];
static MapLine LINES[NLN];
static MapJoint JOINTS[NJ];
static CollVtx VTXA[2048];
static CollLine LINEA[1536];
static CollJoint JOINTA[8];
static mp_UnkStruct0 ISL[48];
static int ISLN;
static CollData CD;
static HSD_GObj GO;
static HSD_JObj JO[8];
static int OUTI[2];
static float OUTF[2];
static u32 OUTU[2];
static Vec3 V3[3];
int mtu_dbgp;
static int ASSN[256];
extern int fprintf(void*, const char*, ...);
static void fprintf_stub(int op) { fprintf(__acrt_iob_func(2), "ASSERTS %d %d%c", op, ASSN[op], 10); }
static int getenv_only;
static int NLN_;
static int NVX_;

static float RNN(void) { static const float k[4] = { 1.0f, 13.0f, 100.0f, 1000.0f }; float a = (float) ((int) (rnd() % 41) - 20) / 20.0f; return a * k[rnd() % 4]; }
#include "mp_gen.h"

static void jcbA(void* ud, int jid, CollData* c, int x50, mpLib_GroundEnum k, float dy) { T(0xCA); T(PI(ud)); T((u32) jid); T(PI(c)); T((u32) x50); T((u32) k); T(B(dy)); c->x50 += 1.0f; }
static void jcbB(void* ud, int jid, CollData* c, int x50, mpLib_GroundEnum k, float dy) { T(0xCB); T(PI(ud)); T((u32) jid); T(PI(c)); T((u32) x50); T((u32) k); T(B(dy)); c->x50 -= 0.5f; }
static bool cbPipe(CollData* c, u32 f) { T(0xCC); T(PI(c)); T(f); c->cur_pos.x += 0.5f; return (bool) (fv() & 1); }
static bool cbFloor(Fighter_GObj* g, int i) { T(0xCD); T(PI(g)); T((u32) i); return (bool) (fv() & 1); }
static HSD_JObj* JP(int i) { return (rnd() % 5 == 0) ? 0 : &JO[i]; }
static float RI(void) { if (rnd() % 8 == 0) return rg(); return (float) ((int) (rnd() % 21) - 10); }
static float RC(void) { return (float) ((int) (rnd() % 61) - 30) + ((rnd() & 1) ? 0.5f : 0.0f); }
static int RVID(void) { return (int) (rnd() % (u32) NLN_); }
static int RLID(void) { return (rnd() % 4 == 0) ? -1 : (int) (rnd() % (u32) NLN_); }

static void mkecb(ftECB* e, float w, float h)
{
    e->top.x = 0.0f; e->top.y = h; e->bottom.x = 0.0f; e->bottom.y = 0.0f;
    e->right.x = w; e->right.y = h * 0.5f; e->left.x = -w; e->left.y = h * 0.5f;
}

static void build_world(void)
{
    int g, i, j, k, n[5], st[5], tot = 0, slot = 0, joint_of[NLN], grp_of[NLN], pos_of[NLN];
    float p0x[NLN], p0y[NLN], p1x[NLN], p1y[NLN];
    n[0] = 2 + (int) (rnd() % 4); n[1] = 1 + (int) (rnd() % 3); n[2] = 1 + (int) (rnd() % 2); n[3] = 1 + (int) (rnd() % 2); n[4] = (int) (rnd() % 3);
    for (g = 0; g < 5; g++) { st[g] = tot; tot += n[g]; }
    for (g = 0; g < 5; g++) {
        float cx = (float) ((int) (rnd() % 61) - 30), cy = (float) ((int) (rnd() % 41) - 20);
        for (k = 0; k < n[g]; k++) {
            int len = 4 + (int) (rnd() % 12), d = (int) (rnd() % 9) - 4;
            i = st[g] + k; grp_of[i] = g; pos_of[i] = k;
            p0x[i] = cx; p0y[i] = cy;
            if (g == 0) { cx += (float) len; cy += (float) d; }
            else if (g == 1) { cx -= (float) len; cy += (float) d; }
            else if (g == 2) { cy -= (float) len; cx += (float) d; }
            else if (g == 3) { cy += (float) len; cx += (float) d; }
            else { int dx = (int) (rnd() % 21) - 10, dy = (int) (rnd() % 21) - 10; if (dx == 0 && dy == 0) dx = 1; cx += (float) dx; cy += (float) dy; }
            p1x[i] = cx; p1y[i] = cy;
        }
    }
    for (g = 0; g < 5; g++) for (k = 0; k < n[g]; k++) joint_of[st[g] + k] = (k * 3) / n[g];
    for (j = 0; j < NJ; j++) {
        int first = slot, cnt = 0;
        float lo_x = 1e9f, lo_y = 1e9f, hi_x = -1e9f, hi_y = -1e9f;
        for (g = 0; g < 5; g++) {
            int s = 0, c = 0;
            for (k = 0; k < n[g]; k++) if (joint_of[st[g] + k] == j) { if (c == 0) s = st[g] + k; c++; }
            JOINTS[j].ranges[g].start = (s16) (c ? s : st[g]); JOINTS[j].ranges[g].count = (s16) c;
        }
        for (i = 0; i < tot; i++) if (joint_of[i] == j) {
            LINES[i].v0_idx = (u16) slot; LINES[i].v1_idx = (u16) (slot + 1);
            VERTS[slot].x = p0x[i]; VERTS[slot].y = p0y[i]; VERTS[slot + 1].x = p1x[i]; VERTS[slot + 1].y = p1y[i];
            if (p0x[i] < lo_x) lo_x = p0x[i]; if (p1x[i] < lo_x) lo_x = p1x[i]; if (p0x[i] > hi_x) hi_x = p0x[i]; if (p1x[i] > hi_x) hi_x = p1x[i];
            if (p0y[i] < lo_y) lo_y = p0y[i]; if (p1y[i] < lo_y) lo_y = p1y[i]; if (p0y[i] > hi_y) hi_y = p0y[i]; if (p1y[i] > hi_y) hi_y = p1y[i];
            slot += 2; cnt += 2;
        }
        JOINTS[j].left_bound = lo_x - 2.0f; JOINTS[j].bottom_bound = lo_y - 2.0f; JOINTS[j].right_bound = hi_x + 2.0f; JOINTS[j].top_bound = hi_y + 2.0f;
        JOINTS[j].vtx_start = (s16) first; JOINTS[j].vtx_count = (s16) cnt;
    }
    for (i = 0; i < tot; i++) {
        int r;
        LINES[i].prev_id0 = (s16) (pos_of[i] > 0 ? i - 1 : -1);
        LINES[i].next_id0 = (s16) (pos_of[i] < n[grp_of[i]] - 1 ? i + 1 : -1);
        r = (int) rnd(); if (r % 8 == 0) LINES[i].prev_id0 = (s16) (i > 0 ? (int) (rnd() % (u32) i) : -1);
        r = (int) rnd(); if (r % 8 == 0) LINES[i].next_id0 = (s16) (tot - 1 - i > 0 ? i + 1 + (int) (rnd() % (u32) (tot - 1 - i)) : -1);
        LINES[i].prev_id1 = (s16) ((rnd() % 3 == 0) ? (i > 0 ? (int) (rnd() % (u32) i) : -1) : -1);
        LINES[i].next_id1 = (s16) ((rnd() % 3 == 0) ? (tot - 1 - i > 0 ? i + 1 + (int) (rnd() % (u32) (tot - 1 - i)) : -1) : -1);
        LINES[i].hi_flags = (u16) (((grp_of[i] < 4) ? (1 << grp_of[i]) : (1 << (rnd() % 4))) | ((rnd() & 1) ? 0x100 : 0) | ((rnd() & 1) ? 0x200 : 0) | ((rnd() % 4 == 0) ? 0x400 : 0) | (fv() & 0xF870));
        LINES[i].lo_flags = (u16) (fv() & 0xFFFF);
        if (rnd() % 3 == 0) LINES[i].lo_flags |= 0x400;
    }
    M.verts = VERTS; M.vert_count = slot; M.lines = LINES; M.line_count = tot; M.joints = JOINTS; M.joint_count = NJ;
    for (g = 0; g < 5; g++) { M.ranges[g].start = (s16) st[g]; M.ranges[g].count = (s16) n[g]; }
    NLN_ = tot; NVX_ = slot;
}

static void setup(void)
{
    int i, j;
    float w, h;
    zero(&M, sizeof M); zero(VERTS, sizeof VERTS); zero(LINES, sizeof LINES); zero(JOINTS, sizeof JOINTS);
    zero(VTXA, sizeof VTXA); zero(LINEA, sizeof LINEA); zero(JOINTA, sizeof JOINTA); zero(ISL, sizeof ISL); ISLN = 0;
    zero(&CD, sizeof CD); zero(&GO, sizeof GO); zero(JO, sizeof JO); zero(OUTI, sizeof OUTI); zero(OUTF, sizeof OUTF); zero(OUTU, sizeof OUTU); zero(V3, sizeof V3);
    stage_info.grkind = 0;
    zero(&mpColl_80458810, sizeof mpColl_80458810); mpColl_804D6488 = 0; mpColl_804D648C = 0; mpColl_804D6490_max_x = 0.0f; mpColl_804D6494_line_id = 0; mpColl_804D6498_flags = 0;
    mpColl_IsEcbTiny = 0; mpColl_804D64A0 = 0; mpColl_804D64A4 = 0; mpColl_804D64A8 = 0; mpColl_804D64AC = 0; mpColl_LedgeLevelOK = 0;
    mpLib_804D64CC = 0; mpLib_804D64D0 = 0; mpLib_804D64D4 = 0; mpLib_804D64D8 = 0; mpLib_804D64DC = 0; mpLib_804D64E0 = 0; mpLib_804D64E4 = 0;
    zero(mpLib_80458868, sizeof mpLib_80458868); scriptBatchDepth = 0; zero(&mp_script_isolation, sizeof mp_script_isolation);
    build_world();
    mpLibLoad(&M);
    for (j = 0; j < NJ; j++) {
        CollJoint* cj = &JOINTA[j];
        if (rnd() % 5 == 0) cj->flags &= ~CollJoint_Enabled;
        if (rnd() % 6 == 0) cj->flags |= CollJoint_Hidden;
        cj->flags |= (fv() & 0x1F00);
        cj->xE = (rnd() & 1);
        cj->x20 = (rnd() & 1) ? &JO[j] : 0;
        cj->cb_0 = (rnd() % 3) ? ((rnd() & 1) ? jcbA : jcbB) : 0; cj->cb_data_0 = (rnd() & 1) ? (Ground*) &GO : 0;
        cj->cb_1 = (rnd() % 3) ? ((rnd() & 1) ? jcbA : jcbB) : 0; cj->cb_data_1 = (rnd() & 1) ? (Ground*) &JO[1] : 0;
    }
    fill_HSD_GObj(&GO);
    for (i = 0; i < 8; i++) fill_HSD_JObj(&JO[i]);
    fill_CollData(&CD);
    CD.x0_gobj = (rnd() & 3) ? &GO : 0;
    CD.cur_pos.x = RC(); CD.cur_pos.y = RC(); CD.cur_pos.z = 0.0f;
    if (rnd() % 4 != 0) { /* near a line end, so the sweeps meet geometry */
        int li = (int) (rnd() % (u32) NLN_), vi = (int) (rnd() & 1);
        float ox = (float) ((int) (rnd() % 9) - 4) * 0.5f, oy = (float) ((int) (rnd() % 9) - 4) * 0.5f;
        Vec2* vp = &VERTS[vi ? LINES[li].v1_idx : LINES[li].v0_idx];
        CD.cur_pos.x = vp->x + ox; CD.cur_pos.y = vp->y + oy;
    }
    CD.prev_pos.x = CD.cur_pos.x + (float) ((int) (rnd() % 7) - 3) * 0.5f; CD.prev_pos.y = CD.cur_pos.y + (float) ((int) (rnd() % 7) - 3) * 0.5f; CD.prev_pos.z = 0.0f;
    CD.last_pos.x = CD.prev_pos.x + (float) ((int) (rnd() % 5) - 2) * 0.5f; CD.last_pos.y = CD.prev_pos.y + (float) ((int) (rnd() % 5) - 2) * 0.5f; CD.last_pos.z = 0.0f;
    CD.x28_vec = CD.prev_pos;
    w = (float) (3 + (int) (rnd() % 3)); h = (float) (8 + (int) (rnd() % 8));
    mkecb(&CD.ecb, w, h); mkecb(&CD.prev_ecb, w + (float) (rnd() % 2), h); mkecb(&CD.desired_ecb, w, h + (float) (rnd() % 3)); mkecb(&CD.x64_ecb, w, h); mkecb(&CD.xE4_ecb, w, h);
    CD.floor_skip = (rnd() % 4 == 0) ? (int) (rnd() % (u32) NLN_) : -1;
    CD.ledge_id_right = (rnd() % 3 == 0) ? (int) (rnd() % (u32) NLN_) : -1;
    CD.ledge_id_left = (rnd() % 3 == 0) ? (int) (rnd() % (u32) NLN_) : -1;
    CD.joint_id_skip = (rnd() % 6 == 0) ? (int) (rnd() % NJ) : -1;
    CD.joint_id_only = (rnd() % 6 == 0) ? (int) (rnd() % NJ) : -1;
    CD.x50 = rg(); CD.lstick_x = rg();
    CD.ledge_snap_x = (float) (2 + (int) (rnd() % 4)); CD.ledge_snap_y = (float) (rnd() % 8); CD.ledge_snap_height = (float) (4 + (int) (rnd() % 8));
    CD.facing_dir = (rnd() & 1) ? 1 : -1;
    CD.env_flags = (s32) rnd(); CD.prev_env_flags = (s32) rnd(); CD.x130_flags = rnd() & 0xFF; CD.x13C = (s32) rnd();
    if (rnd() & 1) {
        CD.ecb_source.kind = ECBSource_Fixed;
        CD.ecb_source.up = (float) (8 + (int) (rnd() % 8)); CD.ecb_source.down = (float) (rnd() % 3); CD.ecb_source.front = (float) (3 + (int) (rnd() % 3)); CD.ecb_source.back = (float) (3 + (int) (rnd() % 3));
        CD.ecb_source.angle = (float) ((int) (rnd() % 13) - 6) * 0.5f;
    } else {
        CD.ecb_source.kind = ECBSource_JObj;
        CD.ecb_source.x108_joint = &JO[0];
        for (i = 0; i < 6; i++) CD.ecb_source.x10C_joint[i] = &JO[i + 1];
    }
    CD.ecb_source.x124 = (float) ((int) (rnd() % 41) - 20) / 20.0f; CD.ecb_source.x128 = (float) (rnd() % 12); CD.ecb_source.x12C = (float) (rnd() % 12);
    CD.floor.index = RLID(); CD.floor.flags = fv(); CD.ceiling.index = RLID(); CD.ceiling.flags = fv();
    CD.left_facing_wall.index = RLID(); CD.left_facing_wall.flags = fv(); CD.right_facing_wall.index = RLID(); CD.right_facing_wall.flags = fv();
    CD.contact.x = rg(); CD.contact.y = rg(); CD.contact.z = 0.0f;
    if (rnd() % 2 == 0) { int li = (int) (rnd() % (u32) NLN_); if (LINES[li].hi_flags & 1) CD.floor.index = li; else if (LINES[li].hi_flags & 2) CD.ceiling.index = li; else if (LINES[li].hi_flags & 4) CD.right_facing_wall.index = li; else CD.left_facing_wall.index = li; }
}

static void hash_all(void)
{
    int i;
    hash_CollData(&CD);
    HM((u32) CD.ecb_source.kind);
    if (CD.ecb_source.kind == ECBSource_Fixed) {
        HM(B(CD.ecb_source.up)); HM(B(CD.ecb_source.down)); HM(B(CD.ecb_source.front)); HM(B(CD.ecb_source.back)); HM(B(CD.ecb_source.angle));
    } else {
        HM(PI(CD.ecb_source.x108_joint)); for (i = 0; i < 6; i++) HM(PI(CD.ecb_source.x10C_joint[i]));
    }
    HM(B(CD.ecb_source.x124)); HM(B(CD.ecb_source.x128)); HM(B(CD.ecb_source.x12C));
    hash_MapCollData(&M);
    for (i = 0; i < NVX; i++) { HM(B(VERTS[i].x)); HM(B(VERTS[i].y)); }
    for (i = 0; i < NLN; i++) hash_MapLine(&LINES[i]);
    for (i = 0; i < NJ; i++) hash_MapJoint(&JOINTS[i]);
    for (i = 0; i < NVX; i++) hash_CollVtx(&VTXA[i]);
    for (i = 0; i < NLN; i++) hash_CollLine(&LINEA[i]);
    for (i = 0; i < 8; i++) hash_CollJoint(&JOINTA[i]);
    for (i = 0; i < 48; i++) hash_mp_UnkStruct0(&ISL[i]);
    HM((u32) ISLN);
    hash_HSD_GObj(&GO);
    HM(PI(groundCollVtx)); HM(PI(groundCollLine)); HM(PI(groundCollJoint)); HM(PI(jointListStart)); HM(PI(jointListEnd)); HM(PI(mpLib_804D64B4));
    HM((u32) didCheckBounding); HM((u32) scriptBatchDepth);
    HM(PI(mp_script_isolation.map)); HM(PI(mp_script_isolation.lines)); HM(PI(mp_script_isolation.joints)); HM((u32) mp_script_isolation.nlines); HM((u32) mp_script_isolation.njoints);
    for (i = 0; i < NLN; i++) HM(mp_script_isolation.line_flags[i]);
    for (i = 0; i < 8; i++) HM(mp_script_isolation.joint_flags[i]);
    for (i = 0; i < 3; i++) { int kk; for (kk = 0; kk < 5; kk++) HM((u32) (u16) mp_script_isolation.ranges[i][kk].start | ((u32) (u16) mp_script_isolation.ranges[i][kk].count << 16)); }
    HM((u32) mpLib_804D64CC); HM((u32) mpLib_804D64D0); HM((u32) mpLib_804D64D4); HM((u32) mpLib_804D64D8); HM((u32) mpLib_804D64DC); HM((u32) mpLib_804D64E0); HM((u32) mpLib_804D64E4);
    for (i = 0; i < 2; i++) { HM(B(mpLib_80458868[i].top)); HM(B(mpLib_80458868[i].bottom)); HM(B(mpLib_80458868[i].left)); HM(B(mpLib_80458868[i].right)); }
    for (i = 0; i < 9; i++) { HM((u32) mpColl_80458810.right[i]); HM((u32) mpColl_80458810.left[i]); }
    HM(B(mpColl_80458810.normal.x)); HM(B(mpColl_80458810.normal.y)); HM(B(mpColl_80458810.normal.z));
    HM((u32) mpColl_804D6488); HM((u32) mpColl_804D648C); HM(B(mpColl_804D6490_max_x)); HM((u32) mpColl_804D6494_line_id); HM(mpColl_804D6498_flags); HM((u32) mpColl_IsEcbTiny);
    HM(FI((const void*) mpColl_804D64A0)); HM(PI(mpColl_804D64A4)); HM(FI((const void*) mpColl_804D64A8)); HM((u32) mpColl_804D64AC); HM((u32) mpColl_LedgeLevelOK);
    HM(PI(mpIsland_80458E88.next)); HM(PI(mpIsland_80458E88.x4)); HM(PI(mpIsland_80458E88.x8)); HM(PI(mpIsland_80458E88.xC)); HM(PI(mpIsland_80458E88.x10));
    HM(PI(mpIsland_80458E88.x14)); HM(PI(mpIsland_80458E88.x18)); HM(PI(mpIsland_80458E88.x1C)); HM(PI(mpIsland_80458E88.x20));
    HM((u32) stage_info.grkind);
    for (i = 0; i < 3; i++) { HM(B(V3[i].x)); HM(B(V3[i].y)); HM(B(V3[i].z)); }
    HM(OUTU[0]); HM(OUTU[1]); HM((u32) OUTI[0]); HM((u32) OUTI[1]); HM(B(OUTF[0])); HM(B(OUTF[1]));
}

int main(void)
{
    int op, k;
    setbuf(__acrt_iob_func(1), 0);
    getenv_only = getenv("MTU_ONLY") != 0;
    preg(&M, sizeof M); preg(VERTS, sizeof VERTS); preg(LINES, sizeof LINES); preg(JOINTS, sizeof JOINTS);
    preg(VTXA, sizeof VTXA); preg(LINEA, sizeof LINEA); preg(JOINTA, sizeof JOINTA); preg(ISL, sizeof ISL);
    preg(&CD, sizeof CD); preg(&GO, sizeof GO); preg(JO, sizeof JO); preg(OUTI, sizeof OUTI); preg(V3, sizeof V3);
    freg((const void*) jcbA); freg((const void*) jcbB); freg((const void*) cbPipe); freg((const void*) cbFloor);
    freg_all();
    for (op = 0; op < 95; op++) {
        for (k = 0; k < 600; k++) {
            u32 ret = 0;
            if (getenv_only && !(op == DBG_OP && k == DBG_K)) continue;
            tu_case(op, k);
            mtu_dbgp = (op == DBG_OP && k == DBG_K);
            setup();
            __try {
            switch (op) {
            case 0: { mpCollPrev(&CD); } break;
            case 1: { mpColl_80042384(&CD); } break;
            case 2: { ret = (u32) mpColl_80044E10_RightWall(&CD); } break;
            case 3: { ret = (u32) mpColl_80045B74_LeftWall(&CD); } break;
            case 4: { ret = (u32) mpColl_800478F4(&CD); } break;
            case 5: { ret = (u32) mpColl_800488F4(&CD); } break;
            case 6: { ret = (u32) mpColl_80048AB0_RightWall(&CD); } break;
            case 7: { ret = (u32) mpColl_80049778_LeftWall(&CD); } break;
            case 8: { ret = (u32) mpColl_8004B4B0(&CD); } break;
            case 9: { { if (CD.ecb_source.kind != ECBSource_Fixed) { CD.ecb_source.kind = ECBSource_Fixed; CD.ecb_source.up = 12.0f; CD.ecb_source.down = 1.0f; CD.ecb_source.front = 4.0f; CD.ecb_source.back = 3.0f; CD.ecb_source.angle = 0.5f; } mpColl_LoadECB_Fixed(&CD); } } break;
            case 10: { mpCollCheckBounding(&CD, fv()); } break;
            case 11: { { int a = (int) (fv() & 1); int b = (int) (fv() & 1); mpCollEnd(&CD, a, b); } } break;
            case 12: { mpCollInterpolateECB(&CD, (float) (rnd() % 11) / 10.0f); } break;
            case 13: { { u32 f = fv(); ret = (u32) mpColl_80043754(cbPipe, &CD, f); } } break;
            case 14: { { int lr = (int) (fv() & 1); ret = (u32) mpColl_80044628_Floor(&CD, ((rnd() & 1) ? cbFloor : 0), &GO, lr); } } break;
            case 15: { { int lr = (int) (fv() & 1); ret = (u32) mpColl_80044AD8_Ceiling(&CD, lr); } } break;
            case 16: { { u32 f = fv(); ret = (u32) mpColl_80046904(&CD, f); } } break;
            case 17: { mpColl_LoadECB_JObj(&CD, fv()); } break;
            case 18: { mpColl_SetECBSource_JObj(&CD, &GO, JP(0), JP(1), JP(2), JP(3), JP(4), JP(5), JP(6), rg()); } break;
            case 19: { { float a = rw(); float b = rw(); float c = rw(); mpColl_SetLedgeSnap(&CD, a, b, c); } } break;
            case 20: { ret = (u32) mpColl_80044164(&CD, &OUTI[0]); HM((u32) OUTI[0]); } break;
            case 21: { ret = (u32) mpColl_800443C4(&CD, &OUTI[0]); HM((u32) OUTI[0]); } break;
            case 22: { ret = (u32) mpColl_80043BBC(&CD, &OUTI[0]); HM((u32) OUTI[0]); } break;
            case 23: { ret = (u32) mpColl_80043E90(&CD, &OUTI[0]); HM((u32) OUTI[0]); } break;
            case 24: { { int b = (int) (fv() & 1); ret = (u32) mpColl_80044838_Floor(&CD, b); } } break;
            case 25: { { int id = RLID(); ret = (u32) mpColl_8004A908_Floor(&CD, id); } } break;
            case 26: { ret = (u32) mpColl_8004ACE4(&CD, (int) fv()); } break;
            case 27: { { int s; u32 f = fv(); mpCollPrev(&CD); for (s = 0; s < 4; s++) { ret = ret * 3 + (u32) mpColl_80043754(cbPipe, &CD, f); CD.cur_pos.x += (float) ((int) (rnd() % 9) - 4); CD.cur_pos.y += (float) ((int) (rnd() % 9) - 4); mpCollPrev(&CD); } mpCollEnd(&CD, (int) (fv() & 1), (int) (fv() & 1)); } } break;
            case 28: { { float l = rw(); float b = rw(); float r = rw(); float t = rw(); mpBoundingCheck(l, b, r, t); } } break;
            case 29: { { float l = RC(); float b = RC(); float r = RC(); float t = RC(); mpBoundingCheck(l, b, r, t); } } break;
            case 30: { ret = (u32) mpCheckedBounding(); } break;
            case 31: { { didCheckBounding = (int) (rnd() & 1); mpUncheckBounding(); } } break;
            case 32: { ret = PI(mpGetGroundCollVtx()); } break;
            case 33: { { int i = (int) (rnd() % 7); if (i < NLN_) LINES[i].v1_idx = LINES[i].v0_idx; mpPruneEmptyLines(&M); } } break;
            case 34: { { int j = (int) (rnd() % 3); mpJointUpdateDynamics(j); } } break;
            case 35: { ret = PI(mpLib_8004D164()); } break;
            case 36: { mpLib_80058614_Floor(); } break;
            case 37: { mpLib_800587FC(&GO); } break;
            case 38: { { int l = (int) (rnd() % (u32) (NLN_ + 4)) - 1; int j = (int) (rnd() % 6) - 1; int iso = (rnd() % 4) != 0; int bad = (int) (rnd() % 6); int bj = (int) (rnd() % NJ); int bk = (int) (rnd() % 5); int bc = (int) (rnd() % 5) - 1; if (bad == 0) JOINTS[bj].ranges[bk].count = (s16) bc; ret = (u32) mpScriptStageIsolationSet(l, j, iso); ret = ret * 3 + (u32) mpScriptStageIsolationActive(); } } break;
            case 39: { { int j = (int) (NJ); int a, b, c; a = mpScriptStageIsolationSet(NLN_, j, 1); if (rnd() & 1) { u32 li = rnd() % (u32) NLN_; u32 fx = fv() & 0xFF; LINEA[li].flags ^= fx; } if (rnd() % 4 == 0) { u32 ji = rnd() % NJ; u32 fx = fv() & 0xFF00; JOINTA[ji].flags ^= fx; } if (rnd() % 5 == 0) mpLib_804D64B4 = (rnd() & 1) ? 0 : (MapCollData*) &GO; b = mpScriptStageIsolationActive(); c = mpScriptStageIsolationSet(0, 0, 0); ret = (u32) (a * 4 + b * 2 + c); } } break;
            case 40: { { int a = (int) (rnd() % 20); stage_info.grkind = (int) (rnd() % 80); ret = (u32) mpLib_80056A54(a, &OUTI[0]); HM((u32) OUTI[0]); } } break;
            case 41: { { int a = (int) (rnd() % 20); stage_info.grkind = (int) (rnd() % 80); ret = (u32) mpLib_80056AC4(a, &OUTI[0]); HM((u32) OUTI[0]); } } break;
            case 42: { { int a = (int) (rnd() % 20); int* p; stage_info.grkind = (int) (rnd() % 80); p = mpLib_80056A8C(a, &OUTI[0]); HM((u32) OUTI[0]); HM((u32) p[0]); HM((u32) p[1]); HM((u32) p[2]); HM((u32) p[3]); } } break;
            case 43: { { float a = RI(), b = RI(), c = RI(), d = RI(), e = RI(), f = RI(), g = RI(), h = RI(); ret = (u32) mpLineIntersection(a, b, c, d, e, f, g, h, &OUTF[0], &OUTF[1]); HM(B(OUTF[0])); HM(B(OUTF[1])); } } break;
            case 44: { { float a = RI(), b = RI(), c = RI(), d = RI(), e = RI(), f = RI(), g = RI(); ret = (u32) mpLineIntersectionH(&OUTF[0], &OUTF[1], a, b, c, d, e, f, g); HM(B(OUTF[0])); HM(B(OUTF[1])); } } break;
            case 45: { { float a = RI(), b = RI(), c = RI(), d = RI(), e = RI(), f = RI(), g = RI(); ret = (u32) mpLineIntersectionV(&OUTF[0], &OUTF[1], a, b, c, d, e, f, g); HM(B(OUTF[0])); HM(B(OUTF[1])); } } break;
            case 46: { mpIsland_8005A728(); } break;
            case 47: { { int s = (int) (rnd() % 30); int c = (int) (rnd() % 12); int b = (int) (fv() & 1); mpIsland_8005AE1C(&mpIsland_80458E88.next, &mpIsland_80458E88.x18, s, c, b); } } break;
            case 48: { { int j = (int) (rnd() % 3); int k = (int) (1 + (rnd() & 1)); int s = (int) (rnd() % 30); int c = (int) (rnd() % 12); int b = (int) (fv() & 1); mpIsland_8005B004((k == 1) ? &mpIsland_80458E88.x10 : &mpIsland_80458E88.x14, &mpIsland_80458E88.x20, j, k, s, c, b); } } break;
            case 49: { { int j = (int) (rnd() % 3); int s = (int) (rnd() % 30); int c = (int) (rnd() % 12); int b = (int) (fv() & 1); mpIsland_8005B334(j, s, c, b); } } break;
            case 50: { { int id = RVID(); ret = (u32) mpLineGetNext(id); } } break;
            case 51: { { int id = RVID(); ret = (u32) mpLineGetPrev(id); } } break;
            case 52: { { int id = RVID(); ret = (u32) mpLineNextNonFloor(id); } } break;
            case 53: { { int id = RVID(); ret = (u32) mpLinePrevNonFloor(id); } } break;
            case 54: { { int id = RVID(); ret = (u32) mpLinePrevNonCeiling(id); } } break;
            case 55: { { int id = RVID(); ret = (u32) mpLineNextNonCeiling(id); } } break;
            case 56: { { int id = RVID(); ret = (u32) mpLineNextNonLeftWall(id); } } break;
            case 57: { { int id = RVID(); ret = (u32) mpLinePrevNonLeftWall(id); } } break;
            case 58: { { int id = RVID(); ret = (u32) mpLinePrevNonRightWall(id); } } break;
            case 59: { { int id = RVID(); ret = (u32) mpLineNextNonRightWall(id); } } break;
            case 60: { { int id = RVID(); ret = (u32) mpLib_80053394_Floor(id); } } break;
            case 61: { { int id = RVID(); ret = (u32) mpLib_80053448_Floor(id); } } break;
            case 62: { { int id = RVID(); ret = (u32) mpLib_800534FC_Floor(id); } } break;
            case 63: { { int id = RVID(); ret = (u32) mpLib_800536CC_Floor(id); } } break;
            case 64: { { int id = RVID(); ret = (u32) mpLib_8005389C_Ceiling(id); } } break;
            case 65: { { int id = RVID(); ret = (u32) mpLib_80053950_Ceiling(id); } } break;
            case 66: { { int id = RVID(); ret = (u32) mpLib_80053A04_Ceiling(id); } } break;
            case 67: { { int id = RVID(); ret = (u32) mpLib_80053BD4_Ceiling(id); } } break;
            case 68: { { int id = RVID(); ret = (u32) mpLineGetKind(id); } } break;
            case 69: { { int id = RVID(); ret = (u32) mpLineGetFlags(id); } } break;
            case 70: { { int id = RVID(); ret = (u32) mpLib_80054ED8(id); } } break;
            case 71: { { int id = RVID(); ret = (u32) mpJointFromLine(id); } } break;
            case 72: { { int id = RVID(); mpLib_80053DA4_Floor(id, &V3[0]); } } break;
            case 73: { { int id = RVID(); mpLib_80053ECC_Floor(id, &V3[0]); } } break;
            case 74: { { int id = RVID(); mpFloorGetRight(id, &V3[0]); } } break;
            case 75: { { int id = RVID(); mpFloorGetLeft(id, &V3[0]); } } break;
            case 76: { { int id = RVID(); mpCeilingGetRight(id, &V3[0]); } } break;
            case 77: { { int id = RVID(); mpCeilingGetLeft(id, &V3[0]); } } break;
            case 78: { { int id = RVID(); mpLeftWallGetTop(id, &V3[0]); } } break;
            case 79: { { int id = RVID(); mpLeftWallGetBottom(id, &V3[0]); } } break;
            case 80: { { int id = RVID(); mpRightWallGetTop(id, &V3[0]); } } break;
            case 81: { { int id = RVID(); mpRightWallGetBottom(id, &V3[0]); } } break;
            case 82: { { int id = RVID(); mpLineGetV1Pos(id, &V3[0]); } } break;
            case 83: { { int id = RVID(); mpLineGetV0Pos(id, &V3[0]); } } break;
            case 84: { { int id = RVID(); V3[0].x = RC(); V3[0].y = RC(); V3[0].z = 0.0f; ret = (u32) mpLib_8004DD90_Floor(id, &V3[0], &OUTF[0], &OUTU[0], &V3[1]); } } break;
            case 85: { { int id = RVID(); V3[0].x = RC(); V3[0].y = RC(); V3[0].z = 0.0f; ret = (u32) mpLib_8004E090_Ceiling(id, &V3[0], &OUTF[0], &OUTU[0], &V3[1]); } } break;
            case 86: { { int id = RVID(); V3[0].x = RC(); V3[0].y = RC(); V3[0].z = 0.0f; ret = (u32) mpLib_8004E398_LeftWall(id, &V3[0], &OUTF[0], &OUTU[0], &V3[1]); } } break;
            case 87: { { int id = RVID(); V3[0].x = RC(); V3[0].y = RC(); V3[0].z = 0.0f; ret = (u32) mpLib_8004E684_RightWall(id, &V3[0], &OUTF[0], &OUTU[0], &V3[1]); } } break;
            case 88: { { int a = RVID(), b = RVID(); ret = (u32) mpLinesConnected(a, b); } } break;
            case 89: { { int id = RVID(); V3[0].x = RC(); V3[0].y = RC(); V3[0].z = 0.0f; ret = (u32) mpGetSpeed(id, &V3[0], &V3[1]); } } break;
            case 90: { { int id = RVID(); ret = PI(mpLineGetNormal(id, &V3[0])); } } break;
            case 91: { { float a = RC(), b = RC(), c = RC(), d = RC(); int s = (int) (fv() & 3); ret = (u32) mpCheckFloor(a, b, c, d, 0.0f, &V3[0], &OUTI[0], &OUTU[0], &V3[1], (s == 0) ? RLID() : -1, (int) (rnd() % 4) - 1, (int) (rnd() % 4) - 1, 0, 0); } } break;
            case 92: { { float a = RC(), b = RC(), c = RC(), d = RC(); int js = (int) (rnd() % 4) - 1; int jo = (int) (rnd() % 4) - 1; ret = (u32) mpCheckCeiling(a, b, c, d, &V3[0], &OUTI[0], &OUTU[0], &V3[1], js, jo); } } break;
            case 93: { { float a = RC(), b = RC(), c = RC(), d = RC(); int js = (int) (rnd() % 4) - 1; int jo = (int) (rnd() % 4) - 1; ret = (u32) mpCheckLeftWall(a, b, c, d, &V3[0], &OUTI[0], &OUTU[0], &V3[1], js, jo); } } break;
            case 94: { { float a = RC(), b = RC(), c = RC(), d = RC(); int js = (int) (rnd() % 4) - 1; int jo = (int) (rnd() % 4) - 1; ret = (u32) mpCheckRightWall(a, b, c, d, &V3[0], &OUTI[0], &OUTU[0], &V3[1], js, jo); } } break;
            }
            } __except (_exception_code() == 0xE0A55E87ul) { HM(0xA55E87u); ASSN[op]++; }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    if (getenv("MTU_STAT")) { for (op = 0; op < 95; op++) fprintf_stub(op); }
    return 0;
}
