/* TU: src/melee/ft/ftcoll.c */
/* C oracle for the world-level functions of src/tu/ftcoll.rs (the per-frame hit / hurt / shield / absorb / wind
 * passes): the whole TU included verbatim; every callee out of it is a recording stub (gen/mtu/stubs.py) except the
 * TARGET_PC echo layer, which is the identity (4 capsules = fp->x914) as in gen/cref/tu/ftcoll.c; the world is three
 * random fighters in the fighter plink, filled field by field (gen/mtu/fill.py).
 * Each line: op case ret state-hash trace. */
#include TU_SRC
/* the game links lbtrigf.c's atanf/atan2f (not the CRT's) */
#include <melee/lb/lbtrigf.c>
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/gobjproc.h>
#include "mtu_common.h"
void ProfBegin(int a, int b) { (void) a; (void) b; }
void ProfEnd(void) {}
void ProfCount(int a) { (void) a; }
int ProfActive(void) { return 0; }

#define NG 3
#define NJ 8
static HSD_GObj GO[NG];
static HSD_JObj JO[NJ];
static Fighter FT[NG];
static ftData FDATA[NG];
static FighterBone PARTS[NG][64];
static ftCommonData CD;
static ItemCommonData ICD;
static HSD_GObj* PLINKV[16];
static u8 PADB[72][2048];
static struct Fighter_804D6524_t D6524;

static HSD_GObj* pick_HSD_GObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &GO[(r >> 8) % NG]; }
static HSD_JObj* pick_HSD_JObj(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &JO[(r >> 8) % NJ]; }
static Fighter* pick_Fighter(void) { u32 r = rnd(); return (r % 6) == 0 ? 0 : &FT[(r >> 8) % NG]; }
static Item* pick_Item(void) { return 0; }
static HSD_DObj* pick_HSD_DObj(void) { return 0; }
static void cbev1(HSD_GObj* g) { T(7001); T(PI(g)); }
static void cbev2(HSD_GObj* g) { T(7002); T(PI(g)); }
static int cbp1(HSD_GObj* g) { T(7003); T(PI(g)); return (int) (fv() & 1); }
static int cbp2(HSD_GObj* g) { T(7004); T(PI(g)); return (int) (fv() & 1); }
static void cbev3(HSD_GObj* g, HSD_GObj* h) { T(7005); T(PI(g)); T(PI(h)); }
static void* pick_fn(int kind)
{
    u32 r = rnd();
    if ((r & 3) == 0) return 0;
    switch (kind) {
    case 1: return (r & 4) ? (void*) cbev1 : (void*) cbev2;
    case 2: return (r & 4) ? (void*) cbp1 : (void*) cbp2;
    case 3: return (void*) cbev3;
    }
    return 0;
}
static void* pick_cb(int kind) { void* f = pick_fn(kind); return f ? f : (kind == 2 ? (void*) cbp1 : kind == 3 ? (void*) cbev3 : (void*) cbev1); }

#include "ftcoll_stubs.h"
#include "ftcoll_state.h"

/* ---- TARGET_PC echo layer: the identity (4 capsules = fp->x914) ---- */
int ScriptGame_EchoCapsuleCount(Fighter* f) { (void) f; return 4; }
HitCapsule* ScriptGame_EchoCapsule(Fighter* f, int i) { return &f->x914[i]; }
int ScriptGame_EchoIsCapsule(HitCapsule* h) { (void) h; return 0; }
int ScriptGame_EchoBlocked(HitCapsule* h, Fighter* f) { (void) h; (void) f; return 0; }
int ScriptGame_EchoSameGroup(HitCapsule* a, HitCapsule* b) { return a->x4 == b->x4; }
int ScriptGame_EchoTargetAllowed(Fighter* s, Fighter* t) { (void) s; (void) t; return 0; }
void ScriptGame_EchoConnected(HitCapsule* h, Fighter* f) { (void) h; (void) f; T(2040); }
void ScriptGame_EchoPrepare(void) { T(2041); }
int ScriptGame_EchoHitIndex(Fighter* f, HitCapsule* h) { T(2042); return (int) (h - f->x914); }
int ScriptGame_EchoReportContext(Fighter* a, Fighter* v, HitCapsule* h) { (void) a; (void) v; (void) h; T(2043); return 1; }
int ScriptGame_EchoCredit(Fighter* a, Fighter* v, HitCapsule* h, float d) { (void) a; (void) v; (void) h; T(2044); T(B(d)); return (int) (fv() & 1); }
int ScriptGame_EchoSourceX(HitCapsule* h, int fb) { (void) h; return fb; }

void* efSync_Spawn(s32 id, HSD_GObj* g, ...) { (void) g; T(2050); T((u32) id); return 0; }

static void world(void)
{
    int i, b;
    zero(GO, sizeof GO); zero(JO, sizeof JO); zero(FT, sizeof FT); zero(FDATA, sizeof FDATA);
    zero(&CD, sizeof CD); zero(&ICD, sizeof ICD);
    for (i = 0; i < NG; i++) fill_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) fill_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) fill_HSD_GObj(&GO[i]);
    for (i = 0; i < NG; i++) {
        GO[i].user_data = &FT[i];
        GO[i].hsd_obj = &JO[i];
        GO[i].classifier = HSD_GOBJ_CLASS_FIGHTER;
        GO[i].next = i + 1 < NG ? &GO[i + 1] : 0;
        FT[i].gobj = &GO[i];
        FT[i].ft_data = &FDATA[i];
        FT[i].grab_cb = cbev1; FT[i].grabbed_cb = cbev3;
        FT[i].kind = (FighterKind) ((u32) FT[i].kind % 27u); FT[i].player_idx %= 4; FT[i].pad_port %= 4;
        FT[i].x166C %= 2;   /* x1670 is declared [1] and the C indexes past it into the filler, whose native layout differs from the Rust types' */
        for (b = 0; b < 64; b++) { zero(&PARTS[i][b], sizeof PARTS[i][b]); PARTS[i][b].joint = &JO[(u32) (i * 5 + b * 3) % NJ]; }
        FT[i].parts = PARTS[i];
        { int pk = i * 24; FDATA[i].x0 = (void*) PADB[pk++]; FDATA[i].ext_attr = (void*) PADB[pk++]; FDATA[i].x8 = (void*) PADB[pk++]; FDATA[i].xC = (void*) PADB[pk++]; FDATA[i].x10 = (void*) PADB[pk++]; FDATA[i].x14 = (void*) PADB[pk++]; FDATA[i].x18 = (void*) PADB[pk++]; FDATA[i].x1C = (void*) PADB[pk++]; FDATA[i].x20 = (void*) PADB[pk++]; FDATA[i].x24 = (void*) PADB[pk++]; FDATA[i].x28 = (void*) PADB[pk++]; FDATA[i].x2C = (void*) PADB[pk++]; FDATA[i].x30 = (void*) PADB[pk++]; FDATA[i].x34 = (void*) PADB[pk++]; FDATA[i].x38 = (void*) PADB[pk++]; FDATA[i].x3C = (void*) PADB[pk++]; FDATA[i].x40 = (void*) PADB[pk++]; FDATA[i].x44 = (void*) PADB[pk++]; FDATA[i].x48_items = (void*) PADB[pk++]; FDATA[i].x4C_sfx = (void*) PADB[pk++]; FDATA[i].x50 = (void*) PADB[pk++]; FDATA[i].x54 = (void*) PADB[pk++]; FDATA[i].x58 = (void*) PADB[pk++]; FDATA[i].x5C = (void*) PADB[pk++]; }
    }
    for (i = 0; i < NJ; i++) {
        JO[i].parent = i ? &JO[(i - 1) / 2] : 0;
        JO[i].child = 2 * i + 1 < NJ ? &JO[2 * i + 1] : 0;
        JO[i].next = (i & 1) && i + 1 < NJ ? &JO[i + 1] : 0;
    }
    fill_ftCommonData(&CD);
    p_ftCommonData = &CD;
    it_804D6D28 = &ICD;
    zero(&D6524, sizeof D6524); D6524.x0 = rg(); D6524.x4 = rg(); Fighter_804D6524 = &D6524;
    for (i = 0; i < 16; i++) PLINKV[i] = 0;
    PLINKV[HSD_GOBJ_PLINK_FIGHTER] = &GO[0];
    HSD_GObjPLinkHead = PLINKV;
    ScriptStageHostHidden = (int) (fv() & 1);
}
static void hworld(void)
{
    int i;
    for (i = 0; i < NG; i++) hash_Fighter(&FT[i]);
    for (i = 0; i < NJ; i++) hash_HSD_JObj(&JO[i]);
    for (i = 0; i < NG; i++) hash_HSD_GObj(&GO[i]);
}

int main(void)
{
    int op, k;
    preg(GO, sizeof GO); preg(JO, sizeof JO); preg(FT, sizeof FT); preg(FDATA, sizeof FDATA);
    freg((const void*) cbev1); freg((const void*) cbev2); freg((const void*) cbp1); freg((const void*) cbp2); freg((const void*) cbev3);
    for (op = 0; op < 15; op++) {
        for (k = 0; k < 500; k++) {
            u32 ret = 0;
            tu_case(op, k);
            world();
            ASRT_ARMED = 1;
            if (_setjmp(ASRT_JB, 0) != 0) { printf("%d %d ASSERT%c", op, k, 10); fflush(0); continue; }
            {
                Fighter_GObj* g = &GO[rnd() % NG];
                Fighter* fp = &FT[(u32) (g - GO)];
                switch (op) {
                case 0: ftColl_800764DC(g); break;
                case 1: ftColl_80078A2C(g); break;
                case 2: ftColl_80078C70(g); break;
                case 3: ftColl_8007925C(g); break;
                case 4: ftColl_8007AB48(g); break;
                case 5: ftColl_8007AB80(g); break;
                case 6: ftColl_8007AD18(fp, &fp->x914[rnd() % 4]); break;
                case 7: ftColl_8007AE80(g); break;
                case 8: ftColl_8007AF60(g); break;
                case 9: ftColl_8007B320(g); break;
                case 10: { Vec3 w; zero(&w, sizeof w); ftColl_GetWindOffsetVec(g, &w); HM(B(w.x)); HM(B(w.y)); HM(B(w.z)); break; }
                case 11: ftColl_8007BA0C(g); break;
                case 12: ftColl_8007BAC0(g); break;
                case 13: { union { float f; u32 u; } c; c.f = ftColl_8007BBCC(g); ret = c.u; break; }
                case 14: ftColl_8007BC90(g); break;
                }
            }
            hworld();
            ASRT_ARMED = 0;
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
            fflush(0);
        }
    }
    return 0;
}
