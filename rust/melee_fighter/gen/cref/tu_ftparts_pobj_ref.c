/* C reference for ftParts_PObjSetupMtx (src/melee/ft/ftparts.c; src/tu/ftparts_render.rs): the z-scale PObj matrix setup
 * (ftPartsSetupRigidMtx / SharedVtxMtx / EnvelopeMtx and their inlines). The game TU is #included with the real class.c,
 * baselib/mtx.c, pobj.c (the matrix marks) and pc/gameworld/mtx_pc.c, plus HSD_Index2PosNrmMtx / HSD_Index2TexMtx cut
 * verbatim (run.py EXTRACTS tu_ftparts_pobj). The GX loads, perf counters, HSD_TObjGetCurrentByType, the envelope model
 * node matrix, HSD_JObjGetCurrent and HSD_JObjSetupMatrixSub are recording stand-ins (tests/tu_ftparts_pobj.rs installs the
 * same as melee_runtime hooks). Every case prints the trace (matrix contents included), the JObj matrices and the marks. */
#include "melee/ft/ftparts.c"
#include "sysdolphin/baselib/class.c"
#include "sysdolphin/baselib/mtx.c"
#include "sysdolphin/baselib/pobj.c"
#include "gameworld/mtx_pc.c"
#include "tu_ftparts_pobj_extract.inc"

int printf(const char*, ...);
HSD_GObj* HSD_GObj_804D7814;
HSD_PerfStat HSD_PerfCurrentStat;
extern int fflush(void*);
extern int _setjmp(void*, void*);
extern void longjmp(void*, int);
static char ASRT_JB[512] __attribute__((aligned(16)));
static int ASRT_ARMED;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; if (ASRT_ARMED) { ASRT_ARMED = 0; longjmp(ASRT_JB, 1); } for (;;) {} }
void OSReport(char* f, ...) { (void) f; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rg(void)
{
    switch (rnd() % 12) {
    case 0: return 0.0f;
    case 1: return 1.0f;
    case 2: return -1.0f;
    case 3: return 2.0f;
    case 4: return 0.5f;
    default: return (float) ((int) (rnd() % 41) - 20) / 7.0f;
    }
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 H, TR;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }
static void T(u32 v) { TR = TR * 31u + v; }
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

#define NJ 6
static HSD_JObj JO[NJ];
static Mtx ENVM[NJ];
static HSD_TObj TOB;
static HSD_PObj PO;
static HSD_SList SL[4];
static HSD_Envelope EV[6];
static HSD_JObj* CUR;
static void TM(const float m[3][4]) { int i, k; for (i = 0; i < 3; i++) for (k = 0; k < 4; k++) T(B(m[i][k])); }
static u32 JI(const void* p) { int i; if (p == 0) return 0; for (i = 0; i < NJ; i++) if (p == &JO[i]) return (u32) (i + 1); return 0xFFFFFFFEu; }

/* recording stand-ins for the GX / HSD externals */
void GXLoadPosMtxImm(Mtx m, u32 id) { T(9100); TM((const float (*)[4]) m); T(id); }
void GXLoadNrmMtxImm(Mtx m, u32 id) { T(9101); TM((const float (*)[4]) m); T(id); }
void GXLoadTexMtxImm(Mtx m, u32 id, GXTexMtxType type) { T(9102); TM((const float (*)[4]) m); T(id); T((u32) type); }
void GXSetCurrentMtx(u32 id) { T(9103); T(id); }
void HSD_PerfCountEnvelopeBlending(s32 n) { T(9105); T((u32) n); }
HSD_TObj* _HSD_TObjGetCurrentByType(HSD_TObj* from, u32 mapping)
{
    T(9106); T(from != 0); T(mapping);
    return (rnd() & 1) ? &TOB : 0;
}
MtxPtr _HSD_mkEnvelopeModelNodeMtx(HSD_JObj* m, MtxPtr out)
{
    int i, k;
    T(9107); T(JI(m));
    if (rnd() & 1) {
        for (i = 0; i < 3; i++) for (k = 0; k < 4; k++) out[i][k] = rg();
        return out;
    }
    return 0;
}
HSD_JObj* HSD_JObjGetCurrent(void) { T(9108); return CUR; }
void HSD_JObjSetupMatrixSub(HSD_JObj* j)
{
    int i, k;
    T(9109); T(JI(j));
    for (i = 0; i < 3; i++) for (k = 0; k < 4; k++) j->mtx[i][k] = rg();
    j->flags &= ~JOBJ_MTX_DIRTY;
}
static void cbps(HSD_PObj* p, Mtx v, Mtx m, u32 r)
{
    T(9112); T(p == &PO); TM((const float (*)[4]) v); TM((const float (*)[4]) m); T(r);
}

int main(void)
{
    int k, i, j;
    for (k = 0; k < 4000; k++) {
        Mtx vm, pm;
        u32 rm;
        int a, b, nenv, depth;
        rs = 0x2468Au + (u32) k * 104729u;
        H = 0x811C9DC5u; TR = 0;
        zero(JO, sizeof JO); zero(ENVM, sizeof ENVM); zero(&TOB, sizeof TOB); zero(&PO, sizeof PO);
        zero(SL, sizeof SL); zero(EV, sizeof EV);
        hsdPObj.setup_mtx = cbps;
        for (i = 0; i < NJ; i++) {
            for (a = 0; a < 3; a++) for (b = 0; b < 4; b++) { JO[i].mtx[a][b] = rg(); ENVM[i][a * 1][b] = 0; }
            for (a = 0; a < 3; a++) for (b = 0; b < 4; b++) ENVM[i][a][b] = rg();
            JO[i].envelopemtx = ENVM[i];
            JO[i].flags = ((rnd() & 1) ? JOBJ_LIGHTING : 0) | ((rnd() % 3 == 0) ? JOBJ_MTX_DIRTY : 0);
        }
        CUR = &JO[rnd() % NJ];
        for (a = 0; a < 3; a++) for (b = 0; b < 4; b++) { vm[a][b] = rg(); pm[a][b] = rg(); }
        rm = rnd();
        if (rnd() % 3 == 0) rm &= ~RENDER_SHADOW;
        ft_jobj_scale.has_z_scale = (rnd() % 4) != 0;
        for (a = 0; a < 3; a++) for (b = 0; b < 4; b++) ft_jobj_scale.mtx[a][b] = rg();
        /* the marks: cleared to a random object / mark pair (HSD_PObjSetMtxMark is a no-op for the two slots in the game code) */
        { void* mo = (rnd() & 1) ? (void*) CUR : (void*) &JO[rnd() % NJ]; u32 mm = (rnd() & 1) ? HSD_MTX_RIGID : HSD_MTX_ENVELOPE; HSD_PObjClearMtxMark(mo, mm); }
        switch (rnd() % 3) {
        case 0: PO.flags = 0; PO.u.jobj = (rnd() & 1) ? &JO[rnd() % NJ] : 0; break;
        case 1: PO.flags = 0x1000; PO.u.jobj = (rnd() & 1) ? &JO[rnd() % NJ] : 0; break;
        default:
            PO.flags = 0x2000;
            nenv = (int) (rnd() % 4);
            for (i = 0; i < nenv && i < 4; i++) {
                HSD_Envelope* e = &EV[i];
                depth = (int) (rnd() % 3);
                SL[i].data = e;
                SL[i].next = i + 1 < nenv ? &SL[i + 1] : 0;
                e->weight = (rnd() % 2) ? 1.0f : (float) (rnd() % 4) / 4.0f;
                e->jobj = &JO[rnd() % NJ];
                e->next = depth ? &EV[4 + (i & 1)] : 0;
                if (depth) { EV[4 + (i & 1)].weight = (float) (rnd() % 4) / 4.0f; EV[4 + (i & 1)].jobj = &JO[rnd() % NJ]; EV[4 + (i & 1)].next = 0; }
            }
            PO.u.envelope_list = nenv ? &SL[0] : 0;
            break;
        }
        ASRT_ARMED = 1;
        if (_setjmp(ASRT_JB, 0) != 0) { printf("%d ASSERT\n", k); continue; }
        ftParts_PObjSetupMtx(&PO, vm, pm, rm);
        ASRT_ARMED = 0;
        for (i = 0; i < NJ; i++) { TM((const float (*)[4]) JO[i].mtx); HM(JO[i].flags); }
        {
            void* o; u32 mk;
            for (j = 0; j < 2; j++) { HSD_PObjGetMtxMark(j, &o, &mk); HM(JI(o)); HM(o == (void*) 0); HM(mk); }
        }
        HM(HSD_PerfCurrentStat.nb_mtx_load); HSD_PerfCurrentStat.nb_mtx_load = 0;
        printf("%d %08X %08X\n", k, H, TR);
    }
    return 0;
}
