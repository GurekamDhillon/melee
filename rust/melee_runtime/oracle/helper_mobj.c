/* Oracle scenarios for the logic of mobj.c (build_mobj.sh). Compiled with the game flags;
 * includes mobj.c itself (statics reachable). Generated inputs only.
 *
 * Rows (hex words): <op> <inputs...> | <outputs...>; every op ends with the FNV hash of the
 * whole persistent MObj state (rendermode, tobj handle, current_mobj, material, PE).
 *   op 0xFFFFFFFF prefix: reset (rendermode, pe present, tobj handle, 4 material words, alpha,
 *     shininess, 12 pe bytes)
 *   op 0: SetFlags null f   op 1: ClearFlags null f   op 2: UpdateFunc type fv
 *   op 3: SetDiffuseColor r g b   op 4: SetAlpha a   op 5: GetTObj null -> idx; SetCurrent null */
#include "../../../src/sysdolphin/baselib/mobj.c"

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
    switch (drv_rand() % 8) {
    case 0: return 0.0F;
    case 1: return bf(drv_rword());
    case 2: return drv_randf(1e10F);
    case 3: return drv_randf(2.0F);
    case 4: return 1.0F;
    default: return drv_randf(0.5F) + 0.5F;
    }
}

static HSD_MObj m;
static HSD_Material mat;
static HSD_PEDesc pe;
static HSD_TObj tobjs[2];

static u32 col(GXColor c) { return ((u32) c.r << 24) | (c.g << 16) | (c.b << 8) | c.a; }
static u32 tidx(HSD_TObj* t) { return t == NULL ? 0xFFFFFFFFu : (u32) (t - tobjs); }
static void dump(void)
{
    u8* p = (u8*) &pe;
    int i;
    drv_hash_begin();
    drv_u32(m.rendermode); drv_u32(tidx(m.tobj)); drv_u32(current_mobj == &m);
    drv_u32(col(mat.ambient)); drv_u32(col(mat.diffuse)); drv_u32(col(mat.specular));
    drv_u32(fb(mat.alpha)); drv_u32(fb(mat.shininess));
    drv_u32(m.pe != NULL);
    for (i = 0; i < 12; i++) drv_u32(p[i]);
    drv_hash_end();
}

static void reset(void)
{
    u8* p = (u8*) &pe;
    int i;
    memset(&m, 0, sizeof m);
    m.rendermode = drv_rword();
    m.mat = &mat;
    m.pe = (drv_rand() & 1) ? &pe : NULL;
    m.tobj = (drv_rand() % 3) ? &tobjs[drv_rand() & 1] : NULL;
    current_mobj = NULL;
    *(u32*) &mat.ambient = drv_rword(); *(u32*) &mat.diffuse = drv_rword();
    *(u32*) &mat.specular = drv_rword();
    mat.alpha = rf(); mat.shininess = rf();
    for (i = 0; i < 12; i++) p[i] = (u8) drv_rand();
    drv_u32(0xFFFFFFFFu);
    drv_u32(m.rendermode); drv_u32(m.pe != NULL); drv_u32(tidx(m.tobj));
    drv_u32(col(mat.ambient)); drv_u32(col(mat.diffuse)); drv_u32(col(mat.specular));
    drv_u32(fb(mat.alpha)); drv_u32(fb(mat.shininess));
    for (i = 0; i < 12; i++) drv_u32(p[i]);
}

void orc_mobj_run(int nops)
{
    int n;
    for (n = 0; n < nops; n++) {
        u32 op = drv_rand() % 6, nul = drv_rand() % 8 == 0;
        if (n % 50 == 0) reset();
        drv_u32(op);
        if (op == 0 || op == 1) {
            u32 f = drv_rword();
            drv_u32(nul); drv_u32(f); drv_sep();
            if (op == 0) HSD_MObjSetFlags(nul ? NULL : &m, f);
            else HSD_MObjClearFlags(nul ? NULL : &m, f);
        } else if (op == 2) {
            u32 ty = drv_rand() % 16;
            f32 fv = rf();
            HSD_ObjData d;
            drv_u32(nul); drv_u32(ty); drv_u32(fb(fv)); drv_sep();
            d.fv = fv;
            MObjUpdateFunc(nul ? NULL : &m, ty, &d);
        } else if (op == 3) {
            u32 r = drv_rand() & 0xFF, g = drv_rand() & 0xFF, b = drv_rand() & 0xFF;
            drv_u32(r); drv_u32(g); drv_u32(b); drv_sep();
            HSD_MObjSetDiffuseColor(&m, r, g, b);
        } else if (op == 4) {
            f32 a = rf();
            drv_u32(fb(a)); drv_sep();
            HSD_MObjSetAlpha(&m, a);
        } else {
            drv_u32(nul); drv_sep();
            drv_u32(tidx(HSD_MObjGetTObj(nul ? NULL : &m)));
            HSD_MObjSetCurrent(nul ? NULL : &m);
        }
        dump();
        drv_eol();
    }
}
