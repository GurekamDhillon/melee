/* Full lobj TU oracle. The original source is included verbatim; only cross-TU
 * dependencies are deterministic test doubles. All state words/call arguments
 * are emitted. */
#include "../../../src/sysdolphin/baselib/lobj.c"
void out(unsigned);
void* alloc(unsigned);
static unsigned calls[32];
static HSD_LObj* lights[32];
static int nl;
static HSD_WObj* worlds[128];
static int nw;
static HSD_AObj* anims[128];
static int na;
static HSD_ClassInfo parent_info;
HSD_ClassInfo hsdObj;
static unsigned bits(float x)
{
    return x != x ? 0x7fc00000 : *(unsigned*) &x;
}
static void vout(Vec3 v)
{
    out(bits(v.x));
    out(bits(v.y));
    out(bits(v.z));
}
static unsigned color(GXColor c)
{
    return (c.r << 24) | (c.g << 16) | (c.b << 8) | c.a;
}
static unsigned ptr(void* p)
{
    int i;
    if (!p) {
        return 0;
    }
    for (i = 0; i < nl; i++) {
        if (p == lights[i]) {
            return 100 + i;
        }
    }
    for (i = 0; i < nw; i++) {
        if (p == worlds[i]) {
            return 200 + i;
        }
    }
    for (i = 0; i < na; i++) {
        if (p == anims[i]) {
            return 400 + i;
        }
    }
    return 1;
}
static void event(int n, void* p)
{
    calls[n]++;
    out(0xe0000000 | n);
    out(ptr(p));
}
static void rel(HSD_Class* p)
{
    event(1, p);
}
static void des(HSD_Class* p)
{
    event(2, p);
}
static void amn(HSD_ClassInfo* p)
{
    event(3, p);
}
void hsdInitClassInfo(HSD_ClassInfo* c, HSD_ClassInfo* p, char* l, char* n,
                      int is, int os)
{
    event(4, c);
    out(is);
    out(os);
    c->head.parent = &parent_info;
}
bool hsdIsDescendantOf(void* a, void* b)
{
    event(5, a);
    return 1;
}
void* hsdNew(HSD_ClassInfo* c)
{
    HSD_LObj* l = alloc(sizeof(*l));
    l->parent.parent.class_info = c;
    lights[nl++] = l;
    event(6, l);
    return l;
}
HSD_ClassInfo* hsdSearchClassInfo(const char* n)
{
    event(7, (void*) n);
    return n[0] == 'x' ? NULL : (HSD_ClassInfo*) &hsdLObj;
}
void HSD_AObjRemove(HSD_AObj* a)
{
    event(8, a);
}
HSD_AObj* HSD_AObjLoadDesc(HSD_AObjDesc* d)
{
    HSD_AObj* a;
    event(9, d);
    if (!d) {
        return NULL;
    }
    a = alloc(sizeof(*a));
    anims[na++] = a;
    return a;
}
void HSD_AObjReqAnim(HSD_AObj* a, float f)
{
    event(10, a);
    out(bits(f));
}
void HSD_AObjInterpretAnim(HSD_AObj* a, void* o, HSD_ObjUpdateFunc cb)
{
    HSD_ObjData d;
    event(11, a);
    out(ptr(o));
    if (a) {
        d.fv = 0.25f;
        cb(o, 9, &d);
    }
}
HSD_WObj* HSD_WObjAlloc(void)
{
    HSD_WObj* w = alloc(sizeof(*w));
    worlds[nw++] = w;
    w->parent.parent.class_info = &parent_info;
    event(12, w);
    return w;
}
HSD_WObj* HSD_WObjLoadDesc(HSD_WObjDesc* d)
{
    HSD_WObj* w;
    event(13, d);
    if (!d) {
        return NULL;
    }
    w = HSD_WObjAlloc();
    w->pos = d->pos;
    return w;
}
void HSD_WObjSetPosition(HSD_WObj* w, Vec3* p)
{
    event(14, w);
    w->pos = *p;
    w->flags |= 2;
    vout(*p);
}
void HSD_WObjGetPosition(HSD_WObj* w, Vec3* p)
{
    event(15, w);
    *p = w->pos;
}
void HSD_WObjInterpretAnim(HSD_WObj* w)
{
    event(16, w);
}
void HSD_WObjReqAnim(HSD_WObj* w, float f)
{
    event(17, w);
    out(bits(f));
}
void HSD_WObjAddAnim(HSD_WObj* w, HSD_WObjAnim* a)
{
    event(18, w);
    out(a != NULL);
}
HSD_SList* HSD_SListAllocAndPrepend(HSD_SList* n, void* d)
{
    HSD_SList* s = alloc(sizeof(*s));
    s->next = n;
    s->data = d;
    return s;
}
HSD_SList* HSD_SListRemove(HSD_SList* s)
{
    return s->next;
}
void GXInitLightColor(GXLightObj* l, GXColor c)
{
    event(19, NULL);
    out(color(c));
    ((u32*) l)[0] = color(c);
}
void GXInitLightPos(GXLightObj* l, float x, float y, float z)
{
    event(20, NULL);
    ((u32*) l)[1] = bits(x);
    ((u32*) l)[2] = bits(y);
    ((u32*) l)[3] = bits(z);
    out(bits(x));
    out(bits(y));
    out(bits(z));
}
void GXInitLightDir(GXLightObj* l, float x, float y, float z)
{
    event(21, NULL);
    ((u32*) l)[4] = bits(x);
    ((u32*) l)[5] = bits(y);
    ((u32*) l)[6] = bits(z);
    out(bits(x));
    out(bits(y));
    out(bits(z));
}
void GXInitLightAttn(GXLightObj* l, float a, float b, float c, float d,
                     float e, float f)
{
    float v[6] = { a, b, c, d, e, f };
    int i;
    event(22, NULL);
    for (i = 0; i < 6; i++) {
        ((u32*) l)[7 + i] = bits(v[i]);
        out(bits(v[i]));
    }
}
void GXInitLightDistAttn(GXLightObj* l, float d, float b, GXDistAttnFn f)
{
    event(23, NULL);
    out(bits(d));
    out(bits(b));
    out(f);
    ((u32*) l)[13] = bits(d);
    ((u32*) l)[14] = bits(b);
    ((u32*) l)[15] = f;
}
void GXInitLightSpot(GXLightObj* l, float c, GXSpotFn f)
{
    event(24, NULL);
    out(bits(c));
    out(f);
    ((u32*) l)[7] = bits(c);
    ((u32*) l)[8] = f;
}
void GXLoadLightObjImm(GXLightObj* l, GXLightID id)
{
    int i;
    event(25, NULL);
    out(id);
    for (i = 0; i < 16; i++) {
        out(((u32*) l)[i]);
    }
}
static void scalar_dump(HSD_LObj* l)
{
    int k;
    out(l->flags);
    out(color(l->color));
    for (k = 0; k < 6; k++) {
        out(((u32*) &l->u)[k]);
    }
}
static void dump(void)
{
    int i, k;
    HSD_SList* s;
    out(nb_active_lights);
    for (i = 0; i < 9; i++) {
        out(ptr(active_lights[i]));
    }
    out(lightmask_diffuse);
    out(lightmask_specular);
    out(lightmask_attnfunc);
    out(lightmask_alpha);
    for (s = current_lights; s; s = s->next) {
        out(ptr(s->data));
    }
    out(0);
    for (i = 0; i < nl; i++) {
        HSD_LObj* l = lights[i];
        out(l->flags);
        out(l->priority);
        out(l->parent.ref_count);
        out(ptr(l->next));
        out(color(l->color));
        out(color(l->hw_color));
        out(ptr(l->position));
        out(ptr(l->interest));
        out(ptr(l->aobj));
        for (k = 0; k < 6; k++) {
            out(((u32*) &l->u)[k]);
        }
        out(bits(l->shininess));
        vout(l->lvec);
        out(l->id);
        out(l->spec_id);
        for (k = 0; k < 16; k++) {
            out(((u32*) &l->lightobj)[k]);
        }
        for (k = 0; k < 16; k++) {
            out(((u32*) &l->spec_lightobj)[k]);
        }
    }
    for (i = 0; i < nw; i++) {
        out(worlds[i]->parent.ref_count);
        out(worlds[i]->flags);
        vout(worlds[i]->pos);
    }
    for (i = 0; i < 32; i++) {
        out(calls[i]);
    }
}
/* The Rust replay performs these same public/static entry-point calls, and
 * compares the complete output stream after each scenario. Includes absent
 * descriptor/anim paths. */
void complete_case(unsigned seed)
{
    int i, k;
    HSD_LObj* top;
    HSD_LightDesc ds[12];
    HSD_WObjDesc wd[24];
    HSD_LightAttn at[12];
    HSD_LightSpotDesc sp[12];
    HSD_LightPointDesc pt[12];
    HSD_LightAnim la[12];
    HSD_AObjDesc ad;
    HSD_WObjAnim wa;
    HSD_CObj cam;
    HSD_ObjData val;
    Vec3 v;
    GXColor c;
    HSD_LObjInfo custom;
    nl = nw = na = 0;
    for (i = 0; i < 32; i++) {
        calls[i] = 0;
    }
    current_lights = NULL;
    default_class = NULL;
    HSD_LObjClearActive();
    parent_info.release = rel;
    parent_info.destroy = des;
    parent_info.amnesia = amn;
    LObjInfoInit();
    hsdLObj.parent.parent.destroy = des;
    out(HSD_LObjGetDefaultClass() == &hsdLObj);
    custom = hsdLObj;
    HSD_LObjSetDefaultClass(&custom);
    out(HSD_LObjGetDefaultClass() == &custom);
    LObjAmnesia((HSD_ClassInfo*) &custom);
    out(HSD_LObjGetDefaultClass() == &hsdLObj);
    HSD_LObjSetDefaultClass(NULL);
    for (i = 0; i < 12; i++) {
        ds[i].next = i == 11 ? NULL : &ds[i + 1];
        ds[i].class_name = i % 3 == 0 ? "hsd_lobj" : i % 3 == 1 ? "x" : "";
        ds[i].flags = (i % 4) | ((i / 4 + 1) << 2) | ((seed & 1) ? 16 : 0) |
                      ((seed & 2) && i == 1 ? 32 : 0);
        ds[i].color.r = i * 17;
        ds[i].color.g = seed * 7;
        ds[i].color.b = 255 - i;
        ds[i].color.a = 255;
        ds[i].attnflags = (seed + i) % 3;
        ds[i].position = &wd[2 * i];
        ds[i].interest = &wd[2 * i + 1];
        for (k = 0; k < 2; k++) {
            wd[2 * i + k].pos.x = (float) (i + 1);
            wd[2 * i + k].pos.y = (float) (seed + 1 + k);
            wd[2 * i + k].pos.z = (float) (3 - k);
        }
        at[i].a0 = .1f;
        at[i].a1 = .2f;
        at[i].a2 = .3f;
        at[i].k0 = .4f;
        at[i].k1 = .5f;
        at[i].k2 = .6f;
        sp[i].cutoff = 40;
        sp[i].spot_func = 2;
        sp[i].ref_br = .75f;
        sp[i].ref_dist = 20;
        sp[i].dist_func = 1;
        pt[i].ref_br = .625f;
        pt[i].ref_dist = 13;
        pt[i].dist_func = 2;
        ds[i].u.p = ds[i].attnflags
                        ? (void*) &at[i]
                        : (i % 4 == 2 ? (void*) &pt[i] : (void*) &sp[i]);
        la[i].next = i == 5 ? NULL : &la[i + 1];
        la[i].aobjdesc = &ad;
        la[i].position_anim = &wa;
        la[i].interest_anim = NULL;
    }
    top = HSD_LObjLoadDesc(ds);
    out(HSD_LObjLoadDesc(NULL) == NULL);
    for (i = 0; i < 12; i++) {
        lights[i]->priority = (seed + i * 3) % 7;
        lights[i]->parent.ref_count = 5;
    }
    HSD_LObj_803668EC(top);
    dump();
    for (i = 0; i < 3; i++) {
        for (k = 0; k < 4; k++) {
            cam.view_mtx[i][k] =
                (i == k ? 1.0f : ((float) ((int) (seed % 5) - 2)) * .125f);
        }
    }
    HSD_LObjSetupInit(&cam);
    dump();
    HSD_LObjSetupSpecularInit(cam.view_mtx);
    dump();
    for (i = 0; i < 12; i++) {
        HSD_LObj* l = lights[i];
        c = l->color;
        c.r += 1;
        HSD_LObjSetup(l, c, (float) (seed + 4));
    }
    dump();
    HSD_LObjSetCurrentAll(lights[9]);
    HSD_LObjSetupInit(&cam);
    dump();
    setup_spec_lightobj(lights[9], cam.view_mtx, 0);
    dump();
    HSD_LObj_803668EC(top);
    HSD_LObjAddAnimAll(top, la);
    HSD_LObjAddAnim(top, la);
    HSD_LObjAddAnim(NULL, la);
    HSD_LObjAddAnim(top, NULL);
    HSD_LObjReqAnimAll(top, (float) seed + .25f);
    HSD_LObjReqAnim(NULL, 1);
    HSD_LObjReqAnimAll(NULL, 1);
    HSD_LObjAnimAll(top);
    HSD_LObjAnim(NULL);
    HSD_LObjAnimAll(NULL);
    dump();
    for (i = 0; i < 12; i++) {
        HSD_LObj* l = lights[i];
        for (k = 0; k < 26; k++) {
            val.fv = (float) ((int) seed - 5) * .125f;
            LObjUpdateFunc(l, k, &val);
            scalar_dump(l);
        }
        HSD_LObjSetSpot(l, 33, 2);
        scalar_dump(l);
        HSD_LObjSetDistAttn(l, 12, .25f, 1);
        scalar_dump(l);
        HSD_LObjSetAttnA(l, 1, 2, 3);
        scalar_dump(l);
        HSD_LObjSetAttnK(l, 4, 5, 6);
        scalar_dump(l);
        HSD_LObjSetAttn(l, .1f, .2f, .3f, .4f, .5f, .6f);
        scalar_dump(l);
        HSD_LObjSetFlags(l, 0x8000);
        scalar_dump(l);
        HSD_LObjClearFlags(l, 0x8000);
        scalar_dump(l);
        out(HSD_LObjGetFlags(l));
        out(HSD_LObjGetType(l));
        c.r = i;
        c.g = 2;
        c.b = 3;
        c.a = 4;
        HSD_LObjSetColor(l, c);
        HSD_LObjGetColor(l, &c);
        out(color(c));
        v.x = i;
        v.y = 2;
        v.z = 3;
        HSD_LObjSetPosition(l, &v);
        v.z = 4;
        HSD_LObjSetInterest(l, &v);
        v.x = 99;
        out(HSD_LObjGetPosition(l, &v));
        vout(v);
        out(HSD_LObjGetInterest(l, &v));
        vout(v);
        HSD_LObjGetLightVector(l, &v);
        vout(v);
        out(ptr(HSD_LObjGetPositionWObj(l)));
        out(ptr(HSD_LObjGetInterestWObj(l)));
    }
    dump();
    out(LObjLoad(top, &ds[0]));
    HSD_LObjSetPositionWObj(top, top->position);
    HSD_LObjSetInterestWObj(top, worlds[0]);
    dump();
    HSD_LObjSetFlags(NULL, 1);
    HSD_LObjClearFlags(NULL, 1);
    out(HSD_LObjGetFlags(NULL));
    LObjUpdateFunc(NULL, 0, &val);
    HSD_LObjSetSpot(NULL, 1, 2);
    HSD_LObjSetDistAttn(NULL, 1, 2, 3);
    HSD_LObjSetAttn(NULL, 1, 2, 3, 4, 5, 6);
    HSD_LObjSetPositionWObj(NULL, NULL);
    HSD_LObjSetInterestWObj(NULL, NULL);
    v.x = 7;
    v.y = 8;
    v.z = 9;
    out(HSD_LObjGetPosition(NULL, &v));
    out(HSD_LObjGetInterest(NULL, &v));
    HSD_LObjGetLightVector(NULL, &v);
    vout(v);
    out(ptr(HSD_LObjGetPositionWObj(NULL)));
    out(ptr(HSD_LObjGetInterestWObj(NULL)));
    for (i = 0; i < 9; i++) {
        out(HSD_LightID2Index(1 << i));
        out(HSD_Index2LightID(i));
        out(ptr(HSD_LObjGetActiveByID(1 << i)));
    }
    out(HSD_Index2LightID(9));
    for (i = -1; i < 10; i++) {
        out(ptr(HSD_LObjGetActiveByIndex(i)));
    }
    out(HSD_LObjGetNbActive());
    out(HSD_LObjGetLightMaskDiffuse());
    out(HSD_LObjGetLightMaskSpecular());
    out(HSD_LObjGetLightMaskAttnFunc());
    out(HSD_LObjGetLightMaskAlpha());
    for (i = 0; i < 4; i++) {
        out(ptr(HSD_LObjGetCurrentByType(i)));
    }
    HSD_LObjAddCurrent(top);
    HSD_LObjAddCurrent(NULL);
    HSD_LObjDeleteCurrent(NULL);
    HSD_LObjDeleteCurrent(top);
    HSD_LObjDeleteCurrent(top);
    HSD_LObjDeleteCurrentAll(top);
    dump();
    HSD_LObjSetCurrentAll(top);
    dump();
    LObjReplaceAll(top);
    dump();
    LObjRemoveAll();
    dump();
    HSD_LObjSetCurrentAll(top);
    HSD_LObjDeleteCurrentAll(NULL);
    HSD_LObjUnrefThis(NULL);
    top->parent.ref_count = 0;
    HSD_LObjUnrefThis(top);
    top->parent.ref_count = 65535;
    HSD_LObjUnrefThis(top);
    HSD_LObjRemoveAll(top);
    dump();
    LObjRelease((HSD_Class*) top);
    LObjAmnesia((HSD_ClassInfo*) &hsdLObj);
    dump();
}
