/* Oracle helper for wobj.c / dobj.c flags (build_wobj.sh). Compiled with the game flags; it
 * includes wobj.c itself so the static WObjUpdateFunc is reachable. */
#include "../../../src/sysdolphin/baselib/wobj.c"
#include <sysdolphin/baselib/dobj.h>
#include <sysdolphin/baselib/spline.h>

void* orc_alloc(unsigned n);

HSD_WObj* orc_wobj_new(int mode, HSD_JObj* target)
{
    HSD_WObj* w = orc_alloc(sizeof(HSD_WObj));
    if (mode != 0) {
        w->aobj = orc_alloc(sizeof(HSD_AObj));
        w->aobj->hsd_obj = mode == 2 ? (HSD_Obj*) target : NULL;
    }
    return w;
}
void orc_wobj_update(HSD_WObj* w, int type, u32 fvb)
{
    HSD_ObjData v;
    v.p.x = v.p.y = v.p.z = 0.0F;
    *(u32*) &v.fv = fvb;
    WObjUpdateFunc(w, type, &v);
}
void orc_wobj_set_position(HSD_WObj* w, const f32* p)
{
    Vec3 v;
    v.x = p[0];
    v.y = p[1];
    v.z = p[2];
    HSD_WObjSetPosition(w, &v);
}
void orc_wobj_get_position(HSD_WObj* w, f32* p)
{
    Vec3 v = { 0 };
    HSD_WObjGetPosition(w, &v);
    p[0] = v.x;
    p[1] = v.y;
    p[2] = v.z;
}
void orc_wobj_clear_flags(HSD_WObj* w, u32 f) { HSD_WObjClearFlags(w, f); }
void orc_wobj_dump(HSD_WObj* w, u32* o)
{
    o[0] = w->flags;
    o[1] = *(u32*) &w->pos.x;
    o[2] = *(u32*) &w->pos.y;
    o[3] = *(u32*) &w->pos.z;
}
void orc_jobj_set_spline(HSD_JObj* j, int type, int numcv, f32 tension, Vec3* cv, f32 total,
                         f32* seg, f32 (*poly)[5])
{
    HSD_Spline* s = orc_alloc(sizeof(HSD_Spline));
    s->type = type;
    s->numcv = numcv;
    s->tension = tension;
    s->cv = cv;
    s->totalLength = total;
    s->segLength = seg;
    s->segPoly = poly;
    j->u.spline = s;
}
const f32* orc_jobj_mtx(HSD_JObj* j) { return &j->mtx[0][0]; }
u32* orc_dobj_flags(HSD_DObj* d) { return &d->flags; }
