/* Oracle scenarios for the logic of tobj.c (build_tobj.sh). Compiled with the game flags; includes
 * tobj.c itself (statics reachable) over the real mtx.c / mtx_pc.c / gekko_fp.c / lbtrigf.c.
 *
 * Rows (hex words): <op> <inputs...> | <outputs...>
 *   op 0: index map k, input -> result
 *   op 1: DifferentTluts n0 n1 -> result
 *   op 2: MakeTextureMtx (scale xyz, rotate xyz, translate xyz, repeat s t, wrap_t) -> 12 mtx words
 *   op 3: TObjUpdateFunc on a persistent tobj: type fv -> FNV hash of the whole tobj state
 * Generated inputs only. */
#include "../../../src/sysdolphin/baselib/tobj.c"

unsigned drv_rand(void);
float drv_randf(float r);
unsigned drv_rword(void);
void drv_u32(unsigned v);
void drv_sep(void);
void drv_eol(void);
void drv_hash_begin(void);
void drv_hash_end(void);

static u32 fb(f32 f) { return f != f ? 0x7FC00000u : *(u32*) &f; }
static f32 bf(u32 u) { return *(f32*) &u; }

static f32 rf(void)
{
    switch (drv_rand() % 10) {
    case 0: return 0.0F;
    case 1: return bf(drv_rword());
    case 2: return drv_randf(1e-9F);
    case 3: return drv_randf(1000.0F);
    default: return drv_randf(4.0F);
    }
}

static u32 valid_in(int k)
{
    static const u32 mt[11] = { 30, 33, 36, 39, 42, 45, 48, 51, 54, 57, 60 };
    switch (k) {
    case 0: return drv_rand() % 8;           /* TexMapID2PTTexMtx */
    case 1: return 4 + drv_rand() % 8;       /* TGTex2Index */
    case 2: return drv_rand() % 7;           /* TexCoordID2TexGenSrc */
    case 3: case 4: case 6: case 7: return drv_rand() % 8;
    case 5: return mt[drv_rand() % 11];      /* TexMtx2Index */
    default: return drv_rand() % 11;         /* Index2TexMtx */
    }
}
static u32 call_map(int k, u32 v)
{
    switch (k) {
    case 0: return HSD_TexMapID2PTTexMtx(v);
    case 1: return HSD_TGTex2Index(v);
    case 2: return HSD_TexCoordID2TexGenSrc(v);
    case 3: return HSD_TexCoord2Index(v);
    case 4: return HSD_Index2TexCoord(v);
    case 5: return HSD_TexMtx2Index(v);
    case 6: return HSD_Index2TexMap(v);
    case 7: return HSD_TexMap2Index(v);
    default: return HSD_Index2TexMtx(v);
    }
}

static HSD_TObj t;
static HSD_ImageDesc imgs[5];
static HSD_ImageDesc* tbl[4];
static HSD_Tlut* tlt[1];
static HSD_TexLODDesc lod;
static HSD_TObjTev tev;

static u32 col(GXColor c) { return ((u32) c.r << 24) | (c.g << 16) | (c.b << 8) | c.a; }
static u32 imgidx(HSD_ImageDesc* d) { return d == NULL ? 0xFFFFFFFFu : (u32) (d - imgs); }
static void dump(void)
{
    drv_hash_begin();
    drv_u32(t.flags);
    drv_u32(fb(t.rotate.x)); drv_u32(fb(t.rotate.y)); drv_u32(fb(t.rotate.z)); drv_u32(fb(t.rotate.w));
    drv_u32(fb(t.scale.x)); drv_u32(fb(t.scale.y)); drv_u32(fb(t.scale.z));
    drv_u32(fb(t.translate.x)); drv_u32(fb(t.translate.y)); drv_u32(fb(t.translate.z));
    drv_u32(fb(t.blending)); drv_u32(imgidx(t.imagedesc)); drv_u32((u32) t.tlut_no);
    drv_u32(fb(lod.LODBias)); drv_u32(col(tev.konst)); drv_u32(col(tev.tev0)); drv_u32(col(tev.tev1));
    drv_hash_end();
}

static void reset(void)
{
    int i;
    memset(&t, 0, sizeof t);
    memset(&lod, 0, sizeof lod);
    memset(&tev, 0, sizeof tev);
    for (i = 0; i < 4; i++) tbl[i] = (drv_rand() % 3) ? &imgs[1 + i] : NULL;
    t.imagedesc = (drv_rand() & 1) ? &imgs[0] : NULL;
    t.imagetbl = tbl;
    t.tluttbl = (drv_rand() & 1) ? tlt : NULL;
    t.tlut_no = (s32) drv_rword();
    t.lod = &lod;
    t.tev = &tev;
    t.flags = drv_rword();
    drv_u32(0xFFFFFFFFu); /* marker: reset; then the setup */
    for (i = 0; i < 4; i++) drv_u32(tbl[i] != NULL);
    drv_u32(t.imagedesc != NULL); drv_u32(t.tluttbl != NULL); drv_u32((u32) t.tlut_no); drv_u32(t.flags);
}

void orc_tobj_run(int nops)
{
    int n;
    for (n = 0; n < nops; n++) {
        u32 op = drv_rand() % 4;
        drv_u32(op);
        if (op == 0) {
            int k = drv_rand() % 9;
            u32 v = valid_in(k);
            drv_u32(k); drv_u32(v); drv_sep(); drv_u32(call_map(k, v));
        } else if (op == 1) {
            HSD_Tlut a, b;
            a.lut = (void*) (size_t) drv_rword(); b.lut = (void*) (size_t) drv_rword();
            a.n_entries = (u16) drv_rand(); b.n_entries = (drv_rand() & 1) ? a.n_entries : (u16) drv_rand();
            drv_u32(a.n_entries); drv_u32(b.n_entries); drv_sep(); drv_u32(DifferentTluts(&a, &b));
        } else if (op == 2) {
            HSD_TObj m;
            int i, j;
            memset(&m, 0, sizeof m);
            m.scale.x = rf(); m.scale.y = rf(); m.scale.z = rf();
            m.rotate.x = rf(); m.rotate.y = rf(); m.rotate.z = rf();
            m.translate.x = rf(); m.translate.y = rf(); m.translate.z = rf();
            m.repeat_s = 1 + drv_rand() % 255; m.repeat_t = 1 + drv_rand() % 255;
            m.wrap_t = drv_rand() % 3;
            drv_u32(fb(m.scale.x)); drv_u32(fb(m.scale.y)); drv_u32(fb(m.scale.z));
            drv_u32(fb(m.rotate.x)); drv_u32(fb(m.rotate.y)); drv_u32(fb(m.rotate.z));
            drv_u32(fb(m.translate.x)); drv_u32(fb(m.translate.y)); drv_u32(fb(m.translate.z));
            drv_u32(m.repeat_s); drv_u32(m.repeat_t); drv_u32(m.wrap_t);
            drv_sep();
            MakeTextureMtx(&m);
            for (i = 0; i < 3; i++) for (j = 0; j < 4; j++) drv_u32(fb(m.mtx[i][j]));
        } else {
            u32 ty;
            f32 fv;
            HSD_ObjData d;
            if (n == 0 || drv_rand() % 50 == 0) reset();
            ty = drv_rand() % 27;
            switch (drv_rand() % 6) {
            case 0: fv = drv_randf(2.0F) + 2.0F; break;   /* image index 0..4 incl. negatives */
            case 1: fv = bf(drv_rword()); break;
            case 2: fv = drv_randf(1.5F); break;
            case 3: fv = drv_randf(1e10F); break;
            default: fv = drv_randf(0.5F) + 0.5F; break;
            }
            if (ty == HSD_A_T_TIMG && !(fv >= 0.0F && fv < 4.0F)) fv = 1.5F; /* C indexes imagetbl[(int)fv] */
            drv_u32(ty); drv_u32(fb(fv)); drv_sep();
            d.fv = fv;
            TObjUpdateFunc(&t, ty, &d);
            dump();
        }
        drv_eol();
    }
}
