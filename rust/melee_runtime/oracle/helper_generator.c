/* Oracle scenarios for generator.c + psappsrt.c (build_generator.sh). Compiled with the game
 * flags; includes both TUs so their statics (hsd_804D78F8, lbl_804D6368, the app SRT pool
 * size) are reachable. Output goes through the native driver (drv_*). No float arithmetic here
 * (only copies), so contraction cannot change anything this file computes.
 *
 * Rows (hex words):
 *   S <setup words>                                   a new scenario (see orc_gen_run)
 *   O <op> <args...> | <ret> | <FNV-1a of log + state>
 * The pools hand out slot indices (per allocator, last freed first), which is what the Rust
 * arenas do, so generator / SRT pointers are written as slot indices. The exe is linked at a
 * low fixed base (the generator code keeps pointers in u32 globals) and every object comes
 * from a static pool. */
#include "../../../src/sysdolphin/baselib/generator.c"
#include "../../../src/sysdolphin/baselib/psappsrt.c"
#include "../../../src/sysdolphin/baselib/cobj.h"
#include "../../../src/sysdolphin/baselib/wobj.h"

unsigned drv_rand(void);
float drv_randf(float r);
unsigned drv_rword(void);
void drv_tag(const char* t);
void drv_u32(unsigned v);
void drv_eol(void);
void drv_hash_begin(void);
void drv_hash_end(void);

static u32 fbits(f32 f) { return *(u32*) &f; }
static void out_f(f32 f) { drv_u32(f != f ? 0x7FC00000u : fbits(f)); }

/* ---- the allocator (HSD_ObjAlloc stand-in) ---- */
#define NPOOL 4
#define NSLOT 512
static char arena[NPOOL][NSLOT][256];
static void* pool_key[NPOOL];
static int pool_n[NPOOL];
static int pool_free[NPOOL][NSLOT];
static int pool_nfree[NPOOL];
static int pool_of(void* key)
{
    int i;
    for (i = 0; i < NPOOL; i++) if (pool_key[i] == key) return i;
    for (i = 0; i < NPOOL; i++) if (pool_key[i] == NULL) { pool_key[i] = key; return i; }
    return -1;
}
void HSD_ObjAllocInit(HSD_ObjAllocData* d, size_t s, u32 a)
{
    int p = pool_of(d);
    (void) s; (void) a;
    pool_n[p] = 0;
    pool_nfree[p] = 0;
}
void* HSD_ObjAlloc(HSD_ObjAllocData* d)
{
    int p = pool_of(d), k;
    if (pool_nfree[p] > 0) k = pool_free[p][--pool_nfree[p]];
    else k = pool_n[p]++;
    memset(arena[p][k], 0xA5, 256); /* the C memsets; garbage shows a missed field */
    return arena[p][k];
}
void HSD_ObjFree(HSD_ObjAllocData* d, void* o)
{
    int p = pool_of(d);
    pool_free[p][pool_nfree[p]++] = (int) (((char*) o - arena[p][0]) / 256);
}
static int slot_of(void* d, void* o)
{
    int p;
    if (o == NULL) return -1;
    p = pool_of(d);
    return (int) (((char*) o - arena[p][0]) / 256);
}
static int slot_live(void* d, int k)
{
    int p = pool_of(d), i;
    if (k >= pool_n[p]) return 0;
    for (i = 0; i < pool_nfree[p]; i++) if (pool_free[p][i] == k) return 0;
    return 1;
}
#define GEN_D (&hsd_804D0F90.alloc_data)
#define SRT_D (&HSD_PSAppSrt_804D10B0)
static int gidx(HSD_Generator* g) { return slot_of(GEN_D, g); }
static int sidx(HSD_psAppSRT* s) { return slot_of(SRT_D, s); }

/* ---- particle.c globals and calls ---- */
#define NBANK 3
#define NREC 6
#define NTG 4
#define NJ 6
#define NPART 4
HSD_PSTexGroup** psTexGroupArray[65];
int psCmdListArray[65];
HSD_PSCmdList** ptclref_804D0E5C[65];
u16 hsd_804D78D8, hsd_804D78DA, hsd_804D78DE, hsd_804D78E0;
u32 hsd_804D78E8, hsd_804D78EC, hsd_804D78F4;
HSD_CObj* psCamera;
struct hsd_804D0F60_t hsd_804D0F60;
static HSD_PSCmdList recs[NBANK][NREC];
static HSD_PSCmdList* recp[NBANK][NREC];
static HSD_PSTexGroup tgs[NBANK][NTG];
static HSD_PSTexGroup* tgp[NBANK][NTG];
static HSD_JObj jobjs[NJ];
static HSD_Particle parts[NPART];
static HSD_CObj cam;
static HSD_WObj cam_eye;
static Vec3 cam_up;
static int hooks_on, nc_mode, pb_mode;

static int jidx(HSD_JObj* j) { return j == NULL ? -1 : (int) (j - jobjs); }
static int cmd_handle(u8* p)
{
    int b, r;
    if (p == NULL) return 0;
    for (b = 0; b < NBANK; b++)
        for (r = 0; r < NREC; r++)
            if (p == recs[b][r].cmdList) return b * 16 + r + 1;
    return -2;
}

void hsd_80398F0C(s32 link, s32 bank, s32 kind, u16 tex, s32 cmd, s32 life, s32 a6, s32 gen, f32 x, f32 y,
                  f32 z, f32 vx, f32 vy, f32 vz, f32 size, f32 grav, f32 fric)
{
    drv_u32(1); drv_u32(link); drv_u32(bank); drv_u32(kind); drv_u32(tex);
    drv_u32(cmd_handle((u8*) (uintptr_t) (u32) cmd)); drv_u32(life); drv_u32(a6);
    drv_u32(gidx((HSD_Generator*) (uintptr_t) (u32) gen));
    out_f(x); out_f(y); out_f(z); out_f(vx); out_f(vy); out_f(vz); out_f(size); out_f(grav); out_f(fric);
}
void hsd_8039D0A0(HSD_Generator* g) { drv_u32(2); drv_u32(gidx(g)); }
void HSD_JObjUnref(HSD_JObj* j) { drv_u32(3); drv_u32(jidx(j)); }
void HSD_JObjSetupMatrixSub(HSD_JObj* j) { drv_u32(4); drv_u32(jidx(j)); j->flags &= ~JOBJ_MTX_DIRTY; }
static void freefunc_stub(HSD_psAppSRT* s) { drv_u32(5); drv_u32(0x77); drv_u32(sidx(s)); }
static void shape_emit_stub(HSD_Generator* g, Mtx m)
{
    int i, j;
    drv_u32(6); drv_u32(gidx(g));
    for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) out_f(m[i][j]);
}
static void shape_init_stub(HSD_Generator* g) { drv_u32(7); drv_u32(gidx(g)); }
static void on_create_stub(HSD_Generator* g) { drv_u32(8); drv_u32(gidx(g)); }
int ScriptGame_StageSlotParticleBegin(int link)
{
    int r = (link & 1) ^ pb_mode;
    drv_u32(9); drv_u32(link); drv_u32(r);
    return r;
}
void ScriptGame_StageSlotProcEnd(void) { drv_u32(10); }
void HSD_StageSlotParticleLinkClear(void) { drv_u32(11); }
int ScriptGame_StageSlotNativeContext(void) { drv_u32(12); drv_u32(nc_mode); return nc_mode; }
int HSD_CObjGetUpVector(HSD_CObj* c, Vec3* up) { (void) c; *up = cam_up; return 0; }

static void set_globals(void)
{
    psCamera = &cam;
    cam.eyepos = &cam_eye;
    hsd_804D78E8 = hooks_on ? (u32) (uintptr_t) shape_emit_stub : 0;
    hsd_804D78EC = hooks_on ? (u32) (uintptr_t) shape_init_stub : 0;
    hsd_804D7900 = hooks_on ? on_create_stub : NULL;
}

/* ---- state dump ---- */
static void dump(void)
{
    int k, i;
    HSD_SList* l;
    drv_u32(*HSD_RandSeedPtr);
    drv_u32(hsd_804D78E0); drv_u32(hsd_804D78DA); drv_u32(lbl_804D6368);
    drv_u32(gidx(hsd_804D78FC)); drv_u32(gidx((HSD_Generator*) (uintptr_t) hsd_804D78F8));
    drv_u32(hsd_804D78DE); drv_u32(hsd_804D78D8);
    for (l = (HSD_SList*) (uintptr_t) hsd_804D78F4; l != NULL; l = l->next) drv_u32(gidx(l->data));
    drv_u32(0xEEEE);
    for (k = 0; k < pool_n[pool_of(GEN_D)]; k++) {
        HSD_Generator* g = (HSD_Generator*) arena[pool_of(GEN_D)][k];
        if (!slot_live(GEN_D, k)) { drv_u32(0xDEAD); continue; }
        drv_u32(gidx(g->next)); drv_u32(g->kind); out_f(g->random); out_f(g->count); drv_u32(jidx(g->jobj));
        drv_u32(g->genLife); drv_u32(g->type); drv_u32(g->bank); drv_u32(g->linkNo); drv_u32(g->texGroup);
        drv_u32(g->dummy); drv_u32(g->idnum); drv_u32(g->life); drv_u32(cmd_handle(g->cmdList));
        out_f(g->pos.x); out_f(g->pos.y); out_f(g->pos.z); out_f(g->vel.x); out_f(g->vel.y); out_f(g->vel.z);
        out_f(g->grav); out_f(g->fric); out_f(g->size); out_f(g->radius); out_f(g->angle);
        drv_u32(g->numChild); drv_u32(sidx(g->appsrt)); drv_u32((u32) (uintptr_t) g->userfunc);
        drv_u32((u32) (uintptr_t) g->callback);
        for (i = 0; i < 12; i++) out_f(((f32*) &g->aux)[i]);
        drv_u32(g->aux.rect.flag);
    }
    drv_u32(0xEEEF);
    for (k = 0; k < pool_n[pool_of(SRT_D)]; k++) {
        HSD_psAppSRT* s = (HSD_psAppSRT*) arena[pool_of(SRT_D)][k];
        if (!slot_live(SRT_D, k)) { drv_u32(0xDEAD); continue; }
        drv_u32(gidx(s->gp));
        out_f(s->translate.x); out_f(s->translate.y); out_f(s->translate.z);
        out_f(s->rot.x); out_f(s->rot.y); out_f(s->rot.z); out_f(s->rot.w);
        out_f(s->scale.x); out_f(s->scale.y); out_f(s->scale.z);
        drv_u32(s->status); drv_u32(s->frameNum); drv_u32(s->usedCount);
        drv_u32(s->freefunc != NULL ? 0x77 : 0); drv_u32(s->idnum); drv_u32(s->xA2);
    }
    drv_u32(0xEEF0);
    for (i = 0; i < NJ; i++) { drv_u32(jobjs[i].object.ref_count); drv_u32(jobjs[i].flags); }
    for (i = 0; i < NPART; i++) drv_u32(sidx(parts[i].appsrt));
}

static HSD_Generator* pick_gen(void)
{
    int n = pool_n[pool_of(GEN_D)], k, c = 0, live = 0;
    for (k = 0; k < n; k++) live += slot_live(GEN_D, k);
    if (live == 0) return NULL;
    c = (int) (drv_rand() % live);
    for (k = 0; k < n; k++) if (slot_live(GEN_D, k) && c-- == 0) return (HSD_Generator*) arena[pool_of(GEN_D)][k];
    return NULL;
}
static HSD_JObj* pick_jobj(void)
{
    unsigned r = drv_rand() % (NJ + 1);
    return r == NJ ? NULL : &jobjs[r];
}
static f32 rf(f32 r) { return drv_randf(r); }
static void rand_mtx(Mtx m)
{
    int i, j;
    for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) m[i][j] = rf(j == 3 ? 50.0F : 2.0F);
}

static void setup(void)
{
    int b, r, i;
    static const u16 types[] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    static const f32 specials[] = { 0.0F, 3.1415927F, 3.1405927F };
    drv_tag("S");
    *HSD_RandSeedPtr = drv_rword();
    drv_u32(*HSD_RandSeedPtr);
    hooks_on = (int) (drv_rand() % 4 != 0); nc_mode = (int) (drv_rand() % 5 == 0); pb_mode = (int) (drv_rand() & 1);
    drv_u32(hooks_on); drv_u32(nc_mode); drv_u32(pb_mode);
    for (b = 0; b < NBANK; b++) {
        psCmdListArray[b] = (int) (drv_rand() % (NREC + 1));
        ptclref_804D0E5C[b] = recp[b];
        psTexGroupArray[b] = tgp[b];
        drv_u32(psCmdListArray[b]);
        for (i = 0; i < NTG; i++) {
            tgs[b][i].palflag = (u16) (drv_rand() % 2);
            tgp[b][i] = drv_rand() % 4 == 0 ? NULL : &tgs[b][i];
            drv_u32(tgp[b][i] != NULL); drv_u32(tgs[b][i].palflag);
        }
        for (r = 0; r < NREC; r++) {
            HSD_PSCmdList* c = &recs[b][r];
            unsigned sh = drv_rand();
            recp[b][r] = drv_rand() % 6 == 0 ? NULL : c;
            c->type = types[sh % 10] | (u16) ((drv_rand() & 7) << 8); /* 0x100/0x200/0x400 */
            c->texGroup = (u16) (drv_rand() % NTG);
            c->genLife = (u16) (drv_rand() % 4);
            c->life = (u16) drv_rand();
            c->kind = (drv_rand() & 0x30910) | (drv_rand() % 3 == 0 ? 0x10000 : 0);
            c->grav = rf(1.0F); c->fric = rf(1.0F);
            c->vx = rf(3.0F); c->vy = rf(3.0F); c->vz = rf(3.0F);
            if (drv_rand() % 8 == 0) { c->vx = 0.0F; c->vz = 0.0F; }
            if (drv_rand() % 10 == 0) { c->vx = 0.0F; c->vy = 0.0F; c->vz = 0.0F; }
            c->radius = rf(5.0F); c->angle = rf(1.5F);
            c->random = drv_rand() % 3 == 0 ? rf(-2.0F) : rf(3.0F);
            if (drv_rand() % 4 == 0) c->random = -c->random;
            c->size = rf(4.0F);
            c->param1 = rf(3.0F); c->param2 = rf(3.0F); c->param3 = rf(3.0F);
            if (drv_rand() % 5 == 0) { c->param1 = 0.0F; c->param2 = 0.0F; }
            if (drv_rand() % 4 == 0) c->param3 = specials[drv_rand() % 3];
            if (r == 0) { /* a rect with every face flag (aux.rect.flag 7) */
                c->type = 5 | (c->type & 0x700);
                if (c->param1 > 0.0F) c->param1 = -c->param1;
                if (c->param2 > 0.0F) c->param2 = -c->param2;
                if (c->param3 >= 0.0F) c->param3 = -c->param3 - 0.25F;
            }
            if (r == 1) c->type = 5 | (c->type & 0x700);
            drv_u32(recp[b][r] != NULL); drv_u32(c->type); drv_u32(c->texGroup); drv_u32(c->genLife);
            drv_u32(c->life); drv_u32(c->kind);
            out_f(c->grav); out_f(c->fric); out_f(c->vx); out_f(c->vy); out_f(c->vz); out_f(c->radius);
            out_f(c->angle); out_f(c->random); out_f(c->size); out_f(c->param1); out_f(c->param2); out_f(c->param3);
        }
    }
    memset(jobjs, 0, sizeof jobjs);
    for (i = 0; i < NJ; i++) {
        HSD_JObj* j = &jobjs[i];
        j->flags = (drv_rand() & 1 ? JOBJ_MTX_DIRTY : 0) | (drv_rand() % 4 == 0 ? JOBJ_USER_DEF_MTX : 0) |
                   (drv_rand() % 4 == 0 ? JOBJ_INSTANCE : 0);
        rand_mtx(j->mtx);
        if (i > 0) {
            int p = (int) (drv_rand() % i);
            HSD_JObj** c = &jobjs[p].child;
            while (*c != NULL) c = &(*c)->next;
            *c = j;
            j->parent = &jobjs[p];
            drv_u32(p);
        } else drv_u32(0xFFFFFFFFu);
        drv_u32(j->flags);
        for (b = 0; b < 3; b++) for (r = 0; r < 4; r++) out_f(j->mtx[b][r]);
    }
    memset(parts, 0, sizeof parts);
    for (i = 0; i < NPART; i++) { parts[i].idnum = (u16) drv_rand(); drv_u32(parts[i].idnum); }
    cam_eye.pos.x = rf(100.0F); cam_eye.pos.y = rf(100.0F); cam_eye.pos.z = rf(100.0F);
    cam_up.x = rf(1.0F); cam_up.y = rf(1.0F); cam_up.z = rf(1.0F);
    out_f(cam_eye.pos.x); out_f(cam_eye.pos.y); out_f(cam_eye.pos.z);
    out_f(cam_up.x); out_f(cam_up.y); out_f(cam_up.z);
    drv_eol();
    hsd_8039D354(0);
    psInitAppSRT(16, sizeof(HSD_psAppSRT));
    set_globals();
}

static void op(void)
{
    unsigned o = drv_rand() % 110, a, b, c;
    HSD_Generator* g;
    HSD_JObj* j;
    int ret = 0;
    unsigned args[16];
    int na = 0, i;
    /* choose the op and its arguments first (written before the hash) */
    if (o >= 100) o = 65 + (o - 100) / 2; /* more field pokes */
    if (o < 12) { a = drv_rand() % 10; b = drv_rand() % 8 == 0 ? 0x41 : drv_rand() % NBANK; c = drv_rand() % (NREC + 1);
        args[na++] = 0; args[na++] = a; args[na++] = b; args[na++] = c; }
    else if (o < 18) { a = drv_rand() % 9; b = drv_rand() % NBANK; c = drv_rand() % NREC; j = pick_jobj();
        args[na++] = 1; args[na++] = a; args[na++] = b; args[na++] = c; args[na++] = jidx(j); }
    else if (o < 22) { a = drv_rand() % 9; b = drv_rand() % NBANK; c = drv_rand() % NREC; j = pick_jobj();
        args[na++] = 2; args[na++] = a; args[na++] = b; args[na++] = c; args[na++] = jidx(j); }
    else if (o < 50) { a = drv_rand() % 3 == 0 ? (drv_rand() & 0xFF) << 16 : 0; args[na++] = 3; args[na++] = a; }
    else if (o < 54) { g = pick_gen(); args[na++] = 4; args[na++] = gidx(g); }
    else if (o < 56) { j = pick_jobj(); args[na++] = 5; args[na++] = jidx(j); }
    else if (o < 58) { j = pick_jobj(); args[na++] = 6; args[na++] = jidx(j); }
    else if (o < 60) { j = pick_jobj(); args[na++] = 7; args[na++] = jidx(j); }
    else if (o < 62) { g = pick_gen(); args[na++] = 8; args[na++] = gidx(g); args[na++] = drv_rand() % 3; }
    else if (o < 63) { args[na++] = 9; }
    else if (o < 65) { args[na++] = 10; }
    else if (o < 75) { /* poke a generator field */
        g = pick_gen(); args[na++] = 11; args[na++] = gidx(g); a = drv_rand() % 11; args[na++] = a;
        switch (a) {
        case 0: args[na++] = drv_rand() % 2; break;                 /* numChild */
        case 1: args[na++] = fbits(rf(3.0F)); break;                /* random */
        case 2: args[na++] = fbits(rf(5.0F)); break;                /* radius */
        case 3: args[na++] = fbits(rf(1.5F)); break;                /* angle */
        case 4: args[na++] = (drv_rand() & 0x1F80); break;           /* type bits to toggle */
        case 5: args[na++] = (drv_rand() & 0x30800); break;          /* kind bits to toggle */
        case 6: args[na++] = fbits(rf(9.0F)); break;                 /* count */
        case 7: args[na++] = drv_rand() % 8; break;                  /* rect flag */
        case 9: for (i = 0; i < 6; i++) args[na++] = fbits(rf(2.0F)); break; /* rect off-diagonals */
        case 10: args[na++] = fbits(rf(30.0F)); args[na++] = fbits(rf(30.0F)); args[na++] = fbits(rf(30.0F)); break; /* pos */
        default: args[na++] = fbits(rf(4.0F)); break;                /* aux[k] */
        }
        if (a == 8) args[na++] = drv_rand() % 12;
    }
    else if (o < 79) { /* jobj: new matrix and dirty */
        args[na++] = 12; args[na++] = drv_rand() % NJ; args[na++] = drv_rand() & 1;
        for (i = 0; i < 12; i++) args[na++] = fbits(rf(i % 4 == 3 ? 50.0F : 2.0F));
    }
    else if (o < 82) { HSD_Generator* h = pick_gen(); g = pick_gen(); args[na++] = 13; args[na++] = gidx(g); args[na++] = h && h->appsrt ? sidx(h->appsrt) : -1; }
    else if (o < 84) { g = pick_gen(); args[na++] = 14; args[na++] = gidx(g); }
    else if (o < 86) { args[na++] = 15; args[na++] = drv_rand() % NPART; args[na++] = drv_rand() % 3; }
    else if (o < 88) { HSD_Generator* h = pick_gen(); args[na++] = 16; args[na++] = drv_rand() % NPART; args[na++] = h && h->appsrt ? sidx(h->appsrt) : -1; }
    else if (o < 90) { args[na++] = 17; args[na++] = drv_rand() % NPART; }
    else if (o < 91) { g = pick_gen(); args[na++] = 18; args[na++] = g && g->appsrt ? sidx(g->appsrt) : -1; }
    else if (o < 92) { args[na++] = 19; }
    else if (o < 93) { args[na++] = 20; }
    else if (o < 95) { g = pick_gen(); args[na++] = 21; args[na++] = gidx(g); }
    else if (o < 97) { g = pick_gen(); args[na++] = 22; args[na++] = gidx(g); }
    else { g = pick_gen(); args[na++] = 23; args[na++] = gidx(g); args[na++] = fbits(rf(6.0F)); }

    /* A generator queued by hsd_8039F6CC and freed before the next hsd_8039EE24 is a C
     * use-after-free (EE24 runs D71C on it); the port drops it from the queue instead. Keep the
     * oracle off that path: with a queue pending, the removal ops become an EE24. */
    if (hsd_804D78F4 != 0 && args[0] >= 4 && args[0] <= 7) {
        na = 0; args[na++] = 3; args[na++] = 0;
    }
    /* HSD_StageSlotParticlesClear does not move hsd_804D78F8: freeing the generator it names
     * leaves it dangling for the next hsd_8039D9C8 (a C use-after-free; the port drops the
     * cursor). Only run it when the cursor survives. */
    if (args[0] == 19 && hsd_804D78F8 != 0 && ((HSD_Generator*) (uintptr_t) hsd_804D78F8)->linkNo == 3) {
        na = 0; args[na++] = 9;
    }
    drv_tag("O");
    for (i = 0; i < na; i++) drv_u32(args[i]);
    drv_tag("|");
    drv_hash_begin();
#define GEN(k) ((int) (k) < 0 ? NULL : (HSD_Generator*) arena[pool_of(GEN_D)][(int) (k)])
#define JOB(k) ((int) (k) < 0 ? NULL : &jobjs[(int) (k)])
#define SRT(k) ((int) (k) < 0 ? NULL : (HSD_psAppSRT*) arena[pool_of(SRT_D)][(int) (k)])
    switch (args[0]) {
    case 0: ret = gidx(hsd_8039F05C(args[1], args[2], args[3])); break;
    case 1: ret = gidx(hsd_8039EFAC(args[1], args[2], args[3], JOB(args[4]))); break;
    case 2: ret = gidx(hsd_8039F6CC(args[1], args[2], args[3], JOB(args[4]))); break;
    case 3: hsd_8039EE24(args[1]); break;
    case 4: if (GEN(args[1])) hsd_8039D4DC(GEN(args[1])); break;
    case 5: hsd_8039D580(JOB(args[1])); break;
    case 6: hsd_8039D5DC(JOB(args[1])); break;
    case 7: hsd_8039D688(JOB(args[1]), NULL, 0); break;
    case 8: if (GEN(args[1])) hsd_8039D1E4(GEN(args[1]), args[2] ? (void*) (uintptr_t) args[2] : NULL); break;
    case 9: ret = hsd_8039D1EC(); break;
    case 10: ret = gidx(hsd_8039D9C8()); break;
    case 11:
        if ((g = GEN(args[1])) != NULL) {
            u32 v = args[3];
            f32 fv = *(f32*) &v;
            switch (args[2]) {
            case 0: g->numChild = v; break;
            case 1: g->random = fv; break;
            case 2: g->radius = fv; break;
            case 3: g->angle = fv; break;
            case 4: g->type ^= (u16) v; break;
            case 5: g->kind ^= v; break;
            case 6: g->count = fv; break;
            case 7: g->aux.rect.flag = (u16) v; break;
            case 9: { static const int od[6] = { 4, 5, 6, 8, 9, 10 };
                for (i = 0; i < 6; i++) { v = args[3 + i]; ((f32*) &g->aux)[od[i]] = *(f32*) &v; } } break;
            case 10: g->pos.x = fv; v = args[4]; g->pos.y = *(f32*) &v; v = args[5]; g->pos.z = *(f32*) &v; break;
            default: ((f32*) &g->aux)[args[4]] = fv; break;
            }
        }
        break;
    case 12:
        j = &jobjs[args[1]];
        if (args[2]) j->flags |= JOBJ_MTX_DIRTY;
        for (i = 0; i < 12; i++) { u32 v = args[3 + i]; j->mtx[i / 4][i % 4] = *(f32*) &v; }
        break;
    case 13: if (GEN(args[1])) ret = psAttachGeneratorAppSRT(GEN(args[1]), SRT(args[2])); else ret = -9; break;
    case 14: if (GEN(args[1])) ret = psRemoveGeneratorSRT(GEN(args[1])); else ret = -9; break;
    case 15: ret = sidx(psAddParticleAppSRT_begin(&parts[args[1]], args[2])); break;
    case 16: ret = psAttachParticleAppSRT(&parts[args[1]], SRT(args[2])); break;
    case 17: ret = psRemoveParticleAppSRT(&parts[args[1]]); break;
    case 18: if (SRT(args[1])) SRT(args[1])->freefunc = freefunc_stub; break;
    case 19: HSD_StageSlotParticlesClear(); break;
    case 20: hsd_8039D354(0); psInitAppSRT(16, sizeof(HSD_psAppSRT)); set_globals();
        for (i = 0; i < NPART; i++) parts[i].appsrt = NULL;
        break;
    case 21: if (GEN(args[1])) hsd_8039D214(GEN(args[1])); break;
    case 22: if (GEN(args[1])) hsd_8039D71C(GEN(args[1])); break;
    case 23: if ((g = GEN(args[1])) != NULL) { u32 v = args[2]; g->count = *(f32*) &v; out_f(hsd_8039DAD4(g)); } break;
    }
    drv_u32(ret);
    dump();
    drv_hash_end();
    drv_eol();
}

void orc_gen_run(int nscen, int nops)
{
    int s, k;
    for (s = 0; s < nscen; s++) {
        setup();
        for (k = 0; k < nops; k++) op();
    }
}
