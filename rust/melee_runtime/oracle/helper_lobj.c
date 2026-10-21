/* Oracle scenarios for the logic half of lobj.c (build_lobj.sh). Compiled with the game flags;
 * includes lobj.c itself so its statics (current_lights, active_lights, ...) are reachable.
 * Output goes through the native driver (drv_*).
 *
 * Rows (all hex words):
 *   S <state>                                a new scenario (8 lights built by the driver)
 *   O <op> <isnull> <idx> <w0> <a0..a5> <v x3> | <results...> | <FNV-1a of the state>
 * state  = nb_active, active[9] (light index or ffffffff), current count, 8 current slots
 *          (index or ffffffff), the four light masks, release count, then 8 lights
 * light  = alive, ref_count, flags, priority, next, color, hw_color, position present, flags,
 *          pos x3, interest present, flags, pos x3, u[6], shininess, lvec x3, id, spec_id (29)
 * (a destroyed light is alive 0 and 28 zero words). */
#include "../../../src/sysdolphin/baselib/lobj.c"

unsigned drv_rand(void);
float drv_randf(float r);
unsigned drv_rword(void);
void drv_tag(const char* t);
void drv_u32(unsigned v);
void drv_eol(void);
void drv_hash_begin(void);
void drv_hash_end(void);
void* orc_alloc(unsigned n);

#define NL 8
static HSD_LObj* L[NL];
static int alive[NL];
static int nrelease;
static HSD_ClassInfo info;

static u32 fbits(f32 f) { return *(u32*) &f; }
/* NaN is written as 7fc00000 (sign not stable across the x86 C and the port; group-runtime.md). */
static void out_f(f32 f) { drv_u32(f != f ? 0x7FC00000u : fbits(f)); }
static void out_v(Vec3* v) { out_f(v->x); out_f(v->y); out_f(v->z); }
static unsigned idx_of(HSD_LObj* l)
{
    int i;
    if (l == NULL) return 0xFFFFFFFFu;
    for (i = 0; i < NL; i++) if (L[i] == l) return i;
    return 0xEEEEEEEEu;
}
static unsigned col(GXColor c) { return ((u32) c.r << 24) | (c.g << 16) | (c.b << 8) | c.a; }

/* The class methods HSD_LObjUnrefThis calls: count releases, mark destroyed (memory is kept). */
static void rel(HSD_Class* o) { (void) o; nrelease++; }
static void des(HSD_Class* o) { alive[idx_of((HSD_LObj*) o)] = 0; }

static void dump_wobj(HSD_WObj* w)
{
    drv_u32(w != NULL);
    drv_u32(w ? w->flags : 0);
    if (w) out_v(&w->pos); else { drv_u32(0); drv_u32(0); drv_u32(0); }
}
static void dump_light(int i)
{
    HSD_LObj* l = L[i];
    int k;
    drv_u32(alive[i]);
    if (!alive[i]) { for (k = 0; k < 28; k++) drv_u32(0); return; }
    drv_u32(l->parent.ref_count); drv_u32(l->flags); drv_u32(l->priority); drv_u32(idx_of(l->next));
    drv_u32(col(l->color)); drv_u32(col(l->hw_color));
    dump_wobj(l->position); dump_wobj(l->interest);
    for (k = 0; k < 6; k++) drv_u32(((u32*) &l->u)[k]);
    out_f(l->shininess); out_v(&l->lvec); drv_u32(l->id); drv_u32(l->spec_id);
}
static void dump(void)
{
    int i, n = 0;
    HSD_SList* s;
    drv_u32(nb_active_lights);
    for (i = 0; i < MAX_GXLIGHT; i++) drv_u32(idx_of(active_lights[i]));
    for (s = current_lights; s; s = s->next) n++;
    drv_u32(n);
    s = current_lights;
    for (i = 0; i < NL; i++) { drv_u32(s ? idx_of(s->data) : 0xFFFFFFFFu); if (s) s = s->next; }
    drv_u32(lightmask_diffuse); drv_u32(lightmask_specular); drv_u32(lightmask_attnfunc); drv_u32(lightmask_alpha);
    drv_u32(nrelease);
    for (i = 0; i < NL; i++) dump_light(i);
}

static f32 rcoord(void)
{
    switch (drv_rand() % 6) {
    case 0: return 0.0F;
    case 1: return (f32) ((int) (drv_rand() % 5) - 2);
    default: return drv_randf(100.0F);
    }
}
static void rvec(Vec3* v) { v->x = rcoord(); v->y = rcoord(); v->z = rcoord(); }
static HSD_WObj* new_wobj(void)
{
    HSD_WObj* w = orc_alloc(sizeof(HSD_WObj));
    w->flags = drv_rand() % 4;
    rvec(&w->pos);
    return w;
}
static GXColor rcol(void)
{
    u32 w = drv_rword();
    GXColor c;
    c.r = w >> 24; c.g = w >> 16; c.b = w >> 8; c.a = w;
    return c;
}

static void new_scenario(void)
{
    int i, k;
    info.release = rel;
    info.destroy = des;
    HSD_LObjClearActive();
    current_lights = NULL;
    nrelease = 0;
    for (i = 0; i < NL; i++) {
        HSD_LObj* l = orc_alloc(sizeof(HSD_LObj));
        L[i] = l;
        alive[i] = 1;
        l->parent.parent.class_info = (void*) &info;
        l->parent.ref_count = drv_rand() % 3;
        l->flags = drv_rword() & 0x1FF;
        l->priority = drv_rand() % 4 == 0 ? (drv_rword() & 0x3FF) : drv_rand() % 4;
        l->color = rcol(); l->hw_color = rcol();
        l->position = drv_rand() % 3 ? new_wobj() : NULL;
        l->interest = drv_rand() % 3 ? new_wobj() : NULL;
        for (k = 0; k < 6; k++) ((u32*) &l->u)[k] = drv_rword();
        l->shininess = drv_randf(50.F);
        rvec(&l->lvec);
        l->id = drv_rword() & 0x1FF;
        l->spec_id = drv_rword() & 0x1FF;
    }
    for (i = 0; i < NL; i++) {
        int j = i + 1 + (int) (drv_rand() % NL);
        L[i]->next = (j < NL && drv_rand() % 2) ? L[j] : NULL;
    }
}

static int chain_ok(HSD_LObj* l)
{
    for (; l; l = l->next) if (!alive[idx_of(l)]) return 0;
    return 1;
}

static const u32 ids[9] = { 1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80, 0x100 };
#define NOPS 32
void orc_lobj_run(int nscen, int nops)
{
    int s, k;
    for (s = 0; s < nscen; s++) {
        new_scenario();
        drv_tag("S"); dump(); drv_eol();
        for (k = 0; k < nops; k++) {
            int op = drv_rand() % NOPS;
            int idx = drv_rand() % NL;
            int isnull = drv_rand() % 12 == 0;
            HSD_LObj *c, *cn;
            Vec3 v, r;
            f32 a[6];
            u32 w0 = drv_rword();
            int i;
            for (i = 0; i < 6; i++) a[i] = drv_randf(100.F);
            for (i = 0; i < NL && !alive[idx]; i++) idx = (idx + 1) % NL;
            if (!alive[idx]) break;
            c = L[idx];
            /* ops that dereference the light unconditionally */
            if (op == 4 || op == 5 || op == 13 || op == 14 || op == 19 || op == 21) isnull = 0;
            if (op == 5 && HSD_LObjGetType(c) != LOBJ_AMBIENT && nb_active_lights >= MAX_GXLIGHT) op = 6;
            if ((op == 28 || op == 30 || op == 31) && !chain_ok(c)) op = 0;
            cn = isnull ? NULL : c;
            rvec(&v);
            if (op == 9) {
                switch (w0 % 4) {
                case 0: a[0] = drv_randf(1.5F) + 0.5F; break; /* colours, incl. < 0 and > 1 */
                case 1: a[0] = (f32) (w0 % 3) * 0.5F; break;  /* VIS at 0.5 exactly */
                case 2: a[0] = drv_randf(1e10F); break;
                default: break;
                }
            }
            drv_tag("O"); drv_u32(op); drv_u32(isnull); drv_u32(idx); drv_u32(w0);
            for (i = 0; i < 6; i++) out_f(a[i]);
            out_v(&v);
            drv_tag("|");
            switch (op) {
            case 0: drv_u32(HSD_LObjGetFlags(cn)); break;
            case 1: HSD_LObjSetFlags(cn, w0); break;
            case 2: HSD_LObjClearFlags(cn, w0); break;
            case 3: drv_u32(HSD_LObjGetLightMaskDiffuse()); drv_u32(HSD_LObjGetLightMaskAttnFunc());
                    drv_u32(HSD_LObjGetLightMaskAlpha()); drv_u32(HSD_LObjGetLightMaskSpecular()); break;
            case 4: drv_u32(HSD_LObjGetType(c)); break;
            case 5: HSD_LObjSetActive(c); break;
            case 6: drv_u32(HSD_LObjGetNbActive()); break;
            case 7: drv_u32(idx_of(HSD_LObjGetActiveByID(ids[w0 % 9]))); break;
            case 8: drv_u32(idx_of(HSD_LObjGetActiveByIndex((int) (w0 % 13) - 2))); break;
            case 9: { HSD_ObjData d; d.fv = a[0]; LObjUpdateFunc(cn, w0 % 26, &d); break; }
            case 10: r.x = r.y = r.z = 7.0F; HSD_LObjGetLightVector(cn, &r); out_v(&r); break;
            case 11: drv_u32(HSD_LightID2Index(ids[w0 % 9])); break;
            case 12: drv_u32(HSD_Index2LightID(w0 % 11)); break;
            case 13: { GXColor cc; cc.r = w0 >> 24; cc.g = w0 >> 16; cc.b = w0 >> 8; cc.a = w0; HSD_LObjSetColor(c, cc); break; }
            case 14: { GXColor cc; HSD_LObjGetColor(c, &cc); drv_u32(col(cc)); break; }
            case 15: HSD_LObjSetSpot(cn, a[0], (s32) w0); break;
            case 16: HSD_LObjSetDistAttn(cn, a[0], a[1], (s32) w0); break;
            case 17: HSD_LObjSetAttnA(cn, a[0], a[1], a[2]); break;
            case 18: HSD_LObjSetAttnK(cn, a[0], a[1], a[2]); break;
            case 19: HSD_LObjSetPosition(c, &v); break;
            case 20: r.x = r.y = r.z = 7.0F; drv_u32(HSD_LObjGetPosition(cn, &r)); out_v(&r); break;
            case 21: HSD_LObjSetInterest(c, &v); break;
            case 22: r.x = r.y = r.z = 7.0F; drv_u32(HSD_LObjGetInterest(cn, &r)); out_v(&r); break;
            case 23: drv_u32(HSD_LObjGetPositionWObj(cn) != NULL); drv_u32(HSD_LObjGetInterestWObj(cn) != NULL); break;
            case 24: HSD_LObjAddCurrent(cn); break;
            case 25: HSD_LObjDeleteCurrent(cn); break;
            case 26: HSD_LObjSetAttn(cn, a[0], a[1], a[2], a[3], a[4], a[5]); break;
            case 27: drv_u32(idx_of(HSD_LObjGetCurrentByType((u16) w0))); break;
            case 28: HSD_LObjDeleteCurrentAll(cn); break;
            case 29: HSD_LObjSetCurrentAll(cn); break;
            case 30: HSD_LObj_803668EC(cn); break;
            case 31: HSD_LObjRemoveAll(cn); break;
            }
            drv_tag("|"); drv_hash_begin(); dump(); drv_hash_end(); drv_eol();
        }
    }
}
