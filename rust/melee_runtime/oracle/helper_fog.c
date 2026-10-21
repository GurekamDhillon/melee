/* Oracle scenarios for the logic of fog.c (build_fog.sh). Compiled with the game flags; includes
 * fog.c itself, over the real aobj.c / fobj.c. The GX calls are recorded here (GXGetViewportv and
 * GXGetProjectionv return what the op row gives).
 *
 * Rows (hex words): O <op> <inputs...> | <outputs...>
 * outputs = the GX log (count, then per call: id + args), the op's own result words, then the
 * fog state (see dump()). Generated inputs only. */
#include "../../../src/sysdolphin/baselib/fog.c"

unsigned drv_rand(void);
float drv_randf(float r);
unsigned drv_rword(void);
void drv_tag(const char* t);
void drv_u32(unsigned v);
void drv_eol(void);

static u32 fb(f32 f) { return f != f ? 0x7FC00000u : *(u32*) &f; }
static f32 bf(u32 u) { return *(f32*) &u; }

/* GX recording */
static u32 glog[256];
static int ng;
static f32 vp[6], pj[7];
static void lg(u32 w) { glog[ng++] = w; }
void GXSetFog(GXFogType t, f32 s, f32 e, f32 n, f32 f, GXColor c)
{
    lg(1); lg(t); lg(fb(s)); lg(fb(e)); lg(fb(n)); lg(fb(f));
    lg(((u32) c.r << 24) | (c.g << 16) | (c.b << 8) | c.a);
}
void GXGetViewportv(f32* v) { int i; lg(2); for (i = 0; i < 6; i++) v[i] = vp[i]; }
void GXGetProjectionv(f32* p) { int i; lg(3); for (i = 0; i < 7; i++) p[i] = pj[i]; }
void GXInitFogAdjTable(GXFogAdjTable* t, u16 w, f32 m[4][4])
{
    int i, j;
    lg(4); lg(w);
    for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) lg(fb(m[i][j]));
    t->r[0] = 0xABCD; t->r[1] = w; /* a marker the range-adj call logs */
}
void GXSetFogRangeAdj(GXBool en, u16 c, GXFogAdjTable* t)
{
    lg(5); lg(en); lg(c); lg(t != NULL);
    if (t) { lg(t->r[0]); lg(t->r[1]); }
}

static HSD_CObj fake_cobj;
static int have_cobj;
static f32 cnear, cfar;
HSD_CObj* HSD_CObjGetCurrent(void) { return have_cobj ? &fake_cobj : NULL; }
f32 HSD_CObjGetNear(HSD_CObj* c) { (void) c; return cnear; }
f32 HSD_CObjGetFar(HSD_CObj* c) { (void) c; return cfar; }

static HSD_Fog* fog;

static void dump(void)
{
    int i, j;
    drv_u32(fog != NULL);
    if (!fog) return;
    drv_u32(fog->type); drv_u32(fb(fog->start)); drv_u32(fb(fog->end));
    drv_u32(((u32) fog->color.r << 24) | (fog->color.g << 16) | (fog->color.b << 8) | fog->color.a);
    drv_u32(fog->fog_adj != NULL);
    if (fog->fog_adj) {
        drv_u32((u32) (s32) fog->fog_adj->center); drv_u32(fog->fog_adj->width);
        for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) drv_u32(fb(fog->fog_adj->mtx[i][j]));
    }
    drv_u32(fog->aobj != NULL);
    if (fog->aobj) {
        drv_u32(fog->aobj->flags); drv_u32(fb(fog->aobj->curr_frame)); drv_u32(fb(fog->aobj->rewind_frame));
        drv_u32(fb(fog->aobj->end_frame)); drv_u32(fb(fog->aobj->framerate));
    }
}

/* Interesting floats: wide ranges, halves, edges, NaN / inf. */
static f32 rfv(void)
{
    switch (drv_rand() % 10) {
    case 0: return (f32) ((s32) (drv_rand() % 9) - 4) * 0.5F;
    case 1: return drv_randf(1.5F);
    case 2: return drv_randf(70000.0F);
    case 3: return drv_randf(5e9F);
    case 4: return bf(drv_rword());
    case 5: return (drv_rand() & 1) ? 1.0F / 0.0F : -1.0F / 0.0F;
    case 6: return (f32) (drv_rand() % 1300) - 300.0F;
    case 7: return drv_randf(1.0F) + 1.0F;
    default: return drv_randf(800.0F);
    }
}
static u32 in_f(f32 f) { u32 u = *(u32*) &f; drv_u32(u); return u; }
static void rand_view(void)
{
    int i;
    for (i = 0; i < 6; i++) in_f(vp[i] = (i == 2 && drv_rand() % 3 == 0) ? (f32) (drv_rand() % 70000) : rfv());
    pj[0] = (drv_rand() % 4 == 0) ? rfv() : (f32) (drv_rand() % 3);
    in_f(pj[0]);
    for (i = 1; i < 7; i++) in_f(pj[i] = rfv());
}
static HSD_FogAdjDesc adesc;
static void rand_adesc(void)
{
    int i, j;
    adesc.center = (u16) drv_rand(); drv_u32(adesc.center);
    adesc.width = (drv_rand() % 3 == 0) ? 0 : (u16) drv_rand(); drv_u32(adesc.width);
    for (i = 0; i < 4; i++) for (j = 0; j < 4; j++) in_f(adesc.mtx[i][j] = rfv());
}
static HSD_AObjDesc aodesc;

void orc_fog_run(int nops)
{
    int k;
    for (k = 0; k < nops; k++) {
        u32 op = drv_rand() % 11, r = 0;
        int i;
        if (fog == NULL && op != 8 && op != 3) op = 8;
        ng = 0;
        drv_tag("O"); drv_u32(op);
        switch (op) {
        case 0: { u32 t = drv_rand() % 23; f32 v = rfv(); drv_u32(t); in_f(v);
            { HSD_ObjData d; d.fv = v; FogUpdateFunc(fog, t, &d); } break; }
        case 1: { u32 has = drv_rand() % 2; drv_u32(has); rand_view();
            if (has) { HSD_FogDesc d; d.type = drv_rand() % 8; d.start = rfv(); d.end = rfv(); d.fogadjdesc = NULL;
                d.color.r = drv_rand(); d.color.g = drv_rand(); d.color.b = drv_rand(); d.color.a = drv_rand();
                drv_u32(d.type); in_f(d.start); in_f(d.end);
                drv_u32(((u32) d.color.r << 24) | (d.color.g << 16) | (d.color.b << 8) | d.color.a);
                HSD_FogInit(fog, &d);
            } else HSD_FogInit(fog, NULL);
            break; }
        case 2: { u32 has = drv_rand() % 2; drv_u32(has); rand_view(); rand_adesc();
            HSD_FogAdjInit(fog->fog_adj, has ? &adesc : NULL); break; }
        case 3: { u32 nul = fog == NULL || drv_rand() % 8 == 0; drv_u32(nul); rand_view();
            have_cobj = 1; in_f(cnear = rfv()); in_f(cfar = rfv());
            HSD_FogSet(nul ? NULL : fog); break; }
        case 4: { u32 fl = drv_rword() & ((drv_rand() & 1) ? 0x200 : 0xFFFFFFFF); f32 fr = rfv(); drv_u32(fl); in_f(fr);
            HSD_FogReqAnimByFlags(fog, fl, fr); break; }
        case 5: { f32 fr = rfv(); in_f(fr); HSD_FogReqAnim(fog, fr); break; }
        case 6: { u32 n = 1 + drv_rand() % 4; drv_u32(n); for (i = 0; i < (int) n; i++) HSD_FogInterpretAnim(fog); break; }
        case 7: { u32 has = drv_rand() % 4 != 0; drv_u32(has);
            aodesc.flags = drv_rword() & 0x7C000000u; aodesc.end_frame = (drv_rand() & 1) ? (f32) (drv_rand() % 20) : rfv();
            aodesc.fobjdesc = NULL; aodesc.obj_id = 0; drv_u32(aodesc.flags); in_f(aodesc.end_frame);
            HSD_Fog_8037DE7C(fog, has ? &aodesc : NULL); break; }
        case 8: { HSD_FogDesc d; u32 hasadj = drv_rand() % 2; drv_u32(hasadj); rand_view(); rand_adesc();
            d.type = drv_rand() % 8; d.start = rfv(); d.end = rfv(); d.fogadjdesc = hasadj ? &adesc : NULL;
            d.color.r = drv_rand(); d.color.g = drv_rand(); d.color.b = drv_rand(); d.color.a = drv_rand();
            drv_u32(d.type); in_f(d.start); in_f(d.end);
            drv_u32(((u32) d.color.r << 24) | (d.color.g << 16) | (d.color.b << 8) | d.color.a);
            fog = HSD_FogLoadDesc(&d); break; }
        case 9: { u32 has = drv_rand() % 2; drv_u32(has); rand_view(); rand_adesc();
            fog->fog_adj = HSD_FogAdjLoadDesc(has ? &adesc : NULL); break; }
        case 10: { u32 v = drv_rand() % 4; drv_u32(v); /* ReqAnim with fog NULL: no-op */
            if (v == 0) { HSD_FogReqAnimByFlags(NULL, 0x200, 1.0F); HSD_FogReqAnim(NULL, 2.0F); HSD_FogInterpretAnim(NULL); }
            break; }
        }
        drv_tag("|");
        drv_u32(ng); for (i = 0; i < ng; i++) drv_u32(glog[i]);
        drv_u32(r);
        dump();
        drv_eol();
    }
}
