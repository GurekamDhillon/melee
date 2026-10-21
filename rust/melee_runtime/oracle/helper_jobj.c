/* Oracle helper for jobj.c (build_jobj.sh): compiled with the game flags so it sees the real
 * HSD_JObj / HSD_RObj / HSD_AObj / HSD_Joint / HSD_DObj layouts; the plain-C driver only uses
 * these accessors. */
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/robj.h>
#include <sysdolphin/baselib/aobj.h>
#include <sysdolphin/baselib/dobj.h>

void* orc_alloc(unsigned n);

static int class_ready;
static void ready(void)
{
    if (!class_ready) {
        HSD_JOBJ_INFO(&hsdJObj)->make_mtx = HSD_JObjMakeMatrix;
        class_ready = 1;
    }
}
HSD_JObj* orc_jobj_new(void)
{
    HSD_JObj* j = orc_alloc(sizeof(HSD_JObj));
    ready();
    j->object.parent.class_info = HSD_CLASS_INFO(&hsdJObj);
    j->object.ref_count = 60000;
    /* JObjInit */
    j->flags = JOBJ_MTX_DIRTY;
    j->scale.x = 1.0F;
    j->scale.y = 1.0F;
    j->scale.z = 1.0F;
    return j;
}
HSD_JObj* orc_next(HSD_JObj* j) { return j->next; }
HSD_JObj* orc_parent(HSD_JObj* j) { return j->parent; }
HSD_JObj* orc_child(HSD_JObj* j) { return j->child; }
u32 orc_flags(HSD_JObj* j) { return j->flags; }
HSD_RObj* orc_robj(HSD_JObj* j) { return j->robj; }
HSD_DObj* orc_dobj_raw(HSD_JObj* j) { return j->u.dobj; }
/* rotate4 scale3 translate3 mtx12 sclflag scl3 = 26 words */
void orc_dump(HSD_JObj* j, u32* o)
{
    int i, k = 0;
    const f32* f = &j->rotate.x;
    for (i = 0; i < 10; i++) {
        o[k++] = *(const u32*) &f[i];
    }
    for (i = 0; i < 12; i++) {
        o[k++] = *(const u32*) &j->mtx[i / 4][i % 4];
    }
    o[k++] = j->scl != NULL;
    for (i = 0; i < 3; i++) {
        o[k++] = j->scl != NULL ? *(const u32*) &(&j->scl->x)[i] : 0;
    }
}
void orc_set_srt(HSD_JObj* j, const f32* v)
{
    f32* f = &j->rotate.x;
    int i;
    for (i = 0; i < 10; i++) {
        f[i] = v[i];
    }
}
void orc_set_aobj(HSD_JObj* j, HSD_JObj* target)
{
    HSD_AObj* a = orc_alloc(sizeof(HSD_AObj));
    a->hsd_obj = (HSD_Obj*) target;
    j->aobj = a;
}
HSD_RObj* orc_robj_new(u32 flags, f32 bone_length)
{
    HSD_RObj* r = orc_alloc(sizeof(HSD_RObj));
    r->flags = flags;
    r->u.ik_hint.bone_length = bone_length;
    return r;
}
HSD_RObj* orc_robj_next(HSD_RObj* r) { return r->next; }
HSD_DObj* orc_dobj_new(void) { return orc_alloc(sizeof(HSD_DObj)); }
HSD_DObj* orc_dobj_next(HSD_DObj* d) { return d->next; }
HSD_Joint* orc_joint_new(HSD_Joint* child, HSD_Joint* next, const f32* v)
{
    HSD_Joint* jt = orc_alloc(sizeof(HSD_Joint));
    jt->child = child;
    jt->next = next;
    jt->rotation.x = v[0];
    jt->rotation.y = v[1];
    jt->rotation.z = v[2];
    jt->scale.x = v[3];
    jt->scale.y = v[4];
    jt->scale.z = v[5];
    jt->position.x = v[6];
    jt->position.y = v[7];
    jt->position.z = v[8];
    return jt;
}
void orc_setup(HSD_JObj* j) { HSD_JObjSetupMatrix(j); }
void orc_set_dirty(HSD_JObj* j) { HSD_JObjSetMtxDirty(j); }
int orc_is_dirty(HSD_JObj* j) { return HSD_JObjMtxIsDirty(j); }
/* robj stubs, mirrored in tests/oracle_jobj.rs */
extern int orc_update_log(HSD_JObj* j);
HSD_RObj* HSD_RObjGetByType(HSD_RObj* robj, u32 type, u32 subtype)
{
    (void) subtype;
    if (type != REFTYPE_IKHINT) {
        return NULL;
    }
    for (; robj != NULL; robj = robj->next) {
        if (robj->flags & REFTYPE_IKHINT) {
            return robj;
        }
    }
    return NULL;
}
void HSD_RObjUpdateAll(HSD_RObj* robj, void* obj, void (*fn)(void*, enum_t, HSD_ObjData*))
{
    (void) robj;
    (void) fn;
    orc_update_log(obj);
}
