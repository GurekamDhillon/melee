/* C reference for the fighter-class half of src/melee/ft/ftparts.c (src/tu/ftparts_render.rs): ftParts_JObjInfoInit,
 * ftParts_IntpJObjInfoInit, ftParts_PObjInfoInit, ftParts_80073758 (hsdChangeClass), ftParts_IntpJObjLoad and
 * ftParts_JObjMakePositionMtx. The game TU is #included whole together with the real class.c, baselib/mtx.c and
 * pc/gameworld/mtx_pc.c (tu_run.py links with /FORCE:UNRESOLVED; the ops reach only what is defined here). The parent
 * classes hsdJObj / hsdPObj / hsdObj are synthetic (already initialised, registered callbacks in every method slot), so
 * the oracle covers the fighter-class functions and not the HSD JObj / PObj class initialisers behind them. Each case
 * prints a hash of the five class infos, the fighter JObj matrix scale and the trace of the parent callbacks.
 * tests/tu_ftparts_class.rs repeats the generator and the hash. Shipping-build contraction. */
#include "melee/ft/ftparts.c"
#include "sysdolphin/baselib/class.c"
#include "sysdolphin/baselib/mtx.c"
#include "gameworld/mtx_pc.c"

int printf(const char*, ...);
extern int fflush(void*);
typedef unsigned long long u64_t_;
HSD_JObjInfo hsdJObj;
HSD_PObjInfo hsdPObj;
HSD_ClassInfo hsdObj;
HSD_GObj* HSD_GObj_804D7814;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
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
    default: return (float) ((int) (rnd() % 41) - 20) / 7.0f;
    }
}
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 H, TR;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }
static void T(u32 v) { TR = TR * 31u + v; }
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }

/* the registered functions: the id of a function pointer is its position here */
static HSD_JObj JO[4];
static HSD_Joint JT[4];
static HSD_JObjInfo ftJObj0, ftIntpJObj0;
static HSD_PObjInfo ftPObj0;
static HSD_ClassInfo* CLS[5];
static int OPK;
static void cb0(void) { T(9000); }
static void cb1(void) { T(9001); }
static HSD_Class* cba(HSD_ClassInfo* c) { T(9002); T(0); (void) c; return 0; }
static int cbi(HSD_Class* c) { T(9003); (void) c; return 0; }
static void cbr(HSD_Class* c) { T(9004); (void) c; }
static void cbm(HSD_ClassInfo* c) { T(9005); (void) c; }
static s32 cbload(HSD_JObj* j, HSD_Joint* jt, HSD_JObj* p)
{
    int i, a = -1, b = -1, c = -1;
    for (i = 0; i < 4; i++) { if (j == &JO[i]) a = i; if (jt == &JT[i]) b = i; if (p == &JO[i]) c = i; }
    T(9006); T((u32) a); T((u32) b); T((u32) c); T(jt->u.dobjdesc == NULL);
    return (s32) rnd();
}
static void cbmk(HSD_JObj* j) { T(9007); (void) j; }
static void cbpm(HSD_JObj* j, Mtx m, Mtx r)
{
    int i, k;
    T(9008);
    for (i = 0; i < 3; i++) for (k = 0; k < 4; k++) { m[i][k] = rg(); r[i][k] = rg(); }
    (void) j;
}
static void cbd(HSD_JObj* j, Mtx v, Mtx p, HSD_TrspMask t, u32 r) { T(9009); (void) j; (void) v; (void) p; (void) t; (void) r; }
static void cbrc(HSD_JObj* j) { T(9010); (void) j; }
static void cbpd(HSD_PObj* p, Mtx v, Mtx m, u32 r) { T(9011); (void) p; (void) v; (void) m; (void) r; }
static void cbps(HSD_PObj* p, Mtx v, Mtx m, u32 r) { T(9012); (void) p; (void) v; (void) m; (void) r; }
static s32 cbpl(HSD_PObj* p, HSD_PObjDesc* d) { T(9013); (void) p; (void) d; return 0; }
s32 lbRefract_PObjLoad(HSD_PObj* pobj, HSD_PObjDesc* desc) { T(9014); (void) pobj; (void) desc; return 0; }
void HSD_JObjSetDefaultClass(HSD_ClassInfo* i);
HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* j);
static const void* FN[40];
static int NFN;
static void freg(const void* f) { FN[NFN++] = f; }
static u32 FI(const void* f)
{
    int i;
    if (f == 0) return 0;
    for (i = 0; i < NFN; i++) if (FN[i] == f) return (u32) (i + 1);
    return 0xFFFFFFFEu;
}
static u32 CI(const void* c)
{
    int i;
    if (c == 0) return 0;
    for (i = 0; i < 5; i++) if ((const void*) CLS[i] == c) return (u32) (i + 1);
    return 0xFFFFFFFEu;
}
void HSD_JObjSetDefaultClass(HSD_ClassInfo* i) { T(9020); T(CI(i)); }
HSD_JObj* HSD_JObjLoadJoint(HSD_Joint* jt)
{
    int a = -1, i;
    for (i = 0; i < 4; i++) if (jt == &JT[i]) a = i;
    T(9021); T((u32) a);
    return (rnd() & 1) ? &JO[rnd() % 4] : 0;
}
static u32 SH(const char* s)
{
    u32 h = 0x811C9DC5u;
    if (s == 0) return 0;
    while (*s) h = (h ^ (unsigned char) *s++) * 0x01000193u;
    return h;
}
static void hclass(HSD_ClassInfo* c, unsigned size)
{
    unsigned i;
    const uintptr_t* w = (const uintptr_t*) ((const char*) c + sizeof(HSD_ClassInfoHead));
    HM(FI((const void*) c->head.info_init)); HM(c->head.flags); HM(SH(c->head.library_name)); HM(SH(c->head.class_name));
    HM((u32) c->head.obj_size); HM((u32) c->head.info_size); HM(CI(c->head.parent)); HM(CI(c->head.next));
    HM(CI(c->head.child)); HM(c->head.nb_exist); HM(c->head.nb_peak);
    for (i = 0; i < (size - sizeof(HSD_ClassInfoHead)) / sizeof(uintptr_t); i++) HM(FI((const void*) w[i]));
}
static void hall(void)
{
    hclass((HSD_ClassInfo*) &hsdJObj, sizeof hsdJObj);
    hclass((HSD_ClassInfo*) &hsdPObj, sizeof hsdPObj);
    hclass((HSD_ClassInfo*) &ftJObj, sizeof ftJObj);
    hclass((HSD_ClassInfo*) &ftIntpJObj, sizeof ftIntpJObj);
    hclass((HSD_ClassInfo*) &ftPObj, sizeof ftPObj);
}
static void fillfn(HSD_ClassInfo* c, unsigned size, const void** pool, int npool)
{
    unsigned i;
    uintptr_t* w = (uintptr_t*) ((char*) c + sizeof(HSD_ClassInfoHead));
    for (i = 0; i < (size - sizeof(HSD_ClassInfoHead)) / sizeof(uintptr_t); i++) w[i] = (uintptr_t) pool[rnd() % (unsigned) npool];
}
static HSD_Class OBJ[4];
static Fighter FT;
static HSD_GObj GO;

int main(void)
{
    int k, op, i, j;
    static const void* poolA[8];
    CLS[0] = (HSD_ClassInfo*) &hsdJObj; CLS[1] = (HSD_ClassInfo*) &hsdPObj; CLS[2] = (HSD_ClassInfo*) &ftJObj;
    CLS[3] = (HSD_ClassInfo*) &ftIntpJObj; CLS[4] = (HSD_ClassInfo*) &ftPObj;
    ftJObj0 = ftJObj; ftIntpJObj0 = ftIntpJObj; ftPObj0 = ftPObj;
    /* function ids: callbacks first, then the fighter-class functions under test */
    freg(cb0); freg(cb1); freg(cba); freg(cbi); freg(cbr); freg(cbm); freg(cbload); freg(cbmk); freg(cbpm); freg(cbd);
    freg(cbrc); freg(cbpd); freg(cbps); freg(cbpl);
    freg(ftParts_JObjInfoInit); freg(ftParts_IntpJObjInfoInit); freg(ftParts_PObjInfoInit);
    freg(ftParts_JObjMakePositionMtx); freg(ftParts_IntpJObjLoad); freg(ftParts_PObjSetupMtx); freg(lbRefract_PObjLoad);
    poolA[0] = 0; poolA[1] = cba; poolA[2] = cbi; poolA[3] = cbr; poolA[4] = cbm; poolA[5] = cb0; poolA[6] = cb1; poolA[7] = cbmk;
    for (op = 0; op < 7; op++) {
        for (k = 0; k < 600; k++) {
            u32 ret = 0;
            rs = 0x2468Au + (u32) op * 7919u + (u32) k * 104729u;
            H = 0x811C9DC5u; TR = 0;
            ftJObj = ftJObj0; ftIntpJObj = ftIntpJObj0; ftPObj = ftPObj0;
            /* parents: already initialised, every method slot a registered function */
            zero(&hsdJObj, sizeof hsdJObj); zero(&hsdPObj, sizeof hsdPObj); zero(&hsdObj, sizeof hsdObj);
            hsdJObj.parent.parent.head.flags = 1; hsdJObj.parent.parent.head.obj_size = (s16) sizeof(HSD_JObj);
            hsdJObj.parent.parent.head.info_size = (s16) sizeof(HSD_JObjInfo);
            hsdJObj.parent.parent.head.library_name = "sysdolphin_base_library"; hsdJObj.parent.parent.head.class_name = "hsd_jobj";
            hsdJObj.parent.parent.head.nb_exist = rnd() % 5 + 1; hsdJObj.parent.parent.head.nb_peak = rnd() % 9;
            hsdJObj.parent.parent.head.child = (rnd() & 1) ? (HSD_ClassInfo*) &hsdPObj : 0;
            fillfn((HSD_ClassInfo*) &hsdJObj, sizeof hsdJObj, poolA, 8);
            hsdJObj.load = cbload; hsdJObj.make_pmtx = cbpm;
            hsdPObj.parent.head.flags = 1; hsdPObj.parent.head.obj_size = (s16) sizeof(HSD_PObj);
            hsdPObj.parent.head.info_size = (s16) sizeof(HSD_PObjInfo);
            hsdPObj.parent.head.library_name = "sysdolphin_base_library"; hsdPObj.parent.head.class_name = "hsd_pobj";
            fillfn((HSD_ClassInfo*) &hsdPObj, sizeof hsdPObj, poolA, 8);
            hsdPObj.setup_mtx = cbps; hsdPObj.load = cbpl;
            if (rnd() & 1) ftJObj.parent.parent.head.flags = 0;
            if (op == 0) {
                ftParts_JObjInfoInit();
            } else if (op == 1) {
                ftParts_IntpJObjInfoInit();
            } else if (op == 2) {
                ftParts_PObjInfoInit();
            } else if (op == 3) {
                /* a JObj of the base class changes onto the fighter class (the class initialises on first use) */
                zero(OBJ, sizeof OBJ);
                for (i = 0; i < 4; i++) OBJ[i].class_info = (HSD_ClassInfo*) &hsdJObj;
                j = (int) (rnd() % 4);
                ftParts_80073758((HSD_JObj*) &OBJ[j]);
                HM((u32) j); for (i = 0; i < 4; i++) HM(CI(OBJ[i].class_info));
            } else if (op == 4) {
                zero(JO, sizeof JO); zero(JT, sizeof JT);
                i = (int) (rnd() % 4); j = (int) (rnd() % 4);
                JT[j].u.dobjdesc = (HSD_DObjDesc*) (uintptr_t) (rnd() % 3 ? 0x1000 : 0);
                JT[j].flags = rnd();
                ret = (u32) ftParts_IntpJObjLoad(&JO[i], &JT[j], (rnd() & 1) ? &JO[(i + 1) % 4] : 0);
                HM(JT[j].u.dobjdesc != 0); HM(JT[j].flags);
            } else if (op == 5) {
                Mtx m, r;
                int a, b;
                zero(&FT, sizeof FT); zero(&GO, sizeof GO);
                GO.user_data = &FT; HSD_GObj_804D7814 = &GO;
                FT.x34_scale.z = (rnd() % 3 == 0) ? 1.0f : rg();
                zero(JO, sizeof JO);
                JO[0].scale.x = rg(); JO[0].scale.y = rg(); JO[0].scale.z = rg();
                if (rnd() % 4 == 0) { JO[0].scale.x = 0.0f; }
                for (a = 0; a < 3; a++) for (b = 0; b < 4; b++) { m[a][b] = 0.0f; r[a][b] = 0.0f; }
                ft_jobj_scale.has_z_scale = rnd() & 1;
                for (a = 0; a < 3; a++) for (b = 0; b < 4; b++) ft_jobj_scale.mtx[a][b] = rg();
                ftParts_JObjMakePositionMtx(&JO[0], m, r);
                for (a = 0; a < 3; a++) for (b = 0; b < 4; b++) { HM(B(m[a][b])); HM(B(r[a][b])); HM(B(ft_jobj_scale.mtx[a][b])); }
                HM(ft_jobj_scale.has_z_scale);
            } else {
                /* ftParts_8007482C: the default-class bracket around HSD_JObjLoadJoint (stand-ins record the calls) */
                zero(JO, sizeof JO); zero(JT, sizeof JT);
                i = (int) (rnd() % 4);
                {
                    HSD_JObj* rj = ftParts_8007482C((rnd() % 5) ? &JT[i] : 0);
                    ret = 0xFF;
                    for (j = 0; j < 4; j++) if (rj == &JO[j]) ret = (u32) j;
                }
            }
            hall();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
