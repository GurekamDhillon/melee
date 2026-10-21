/* Oracle helper for robj.c (build_robj.sh): compiled with the game flags so it sees the real
 * HSD_JObj / HSD_RObj / HSD_RObjDesc / HSD_Rvalue layouts; the plain-C driver only uses these
 * accessors. */
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/robj.h>

void* orc_alloc(unsigned n);

static int class_ready;
HSD_JObj* orc_jobj_new(void)
{
    HSD_JObj* j = orc_alloc(sizeof(HSD_JObj));
    if (!class_ready) {
        HSD_JOBJ_INFO(&hsdJObj)->make_mtx = HSD_JObjMakeMatrix;
        class_ready = 1;
    }
    j->object.parent.class_info = HSD_CLASS_INFO(&hsdJObj);
    j->object.ref_count = 60000;
    j->flags = JOBJ_MTX_DIRTY;
    j->scale.x = 1.0F;
    j->scale.y = 1.0F;
    j->scale.z = 1.0F;
    return j;
}
u32 orc_flags(HSD_JObj* j) { return j->flags; }
HSD_JObj* orc_parent(HSD_JObj* j) { return j->parent; }
/* rotate4 scale3 translate3 mtx12 sclflag scl3 = 26 words */
void orc_dump(HSD_JObj* j, u32* o)
{
    int i, k = 0;
    const f32* f = &j->rotate.x;
    for (i = 0; i < 10; i++) o[k++] = *(const u32*) &f[i];
    for (i = 0; i < 12; i++) o[k++] = *(const u32*) &j->mtx[i / 4][i % 4];
    o[k++] = j->scl != NULL;
    for (i = 0; i < 3; i++) o[k++] = j->scl != NULL ? *(const u32*) &(&j->scl->x)[i] : 0;
}
void orc_set_srt(HSD_JObj* j, const f32* v)
{
    f32* f = &j->rotate.x;
    int i;
    for (i = 0; i < 10; i++) f[i] = v[i];
}
void orc_set_scl(HSD_JObj* j, const f32* v)
{
    j->scl = orc_alloc(sizeof(Vec3));
    j->scl->x = v[0];
    j->scl->y = v[1];
    j->scl->z = v[2];
}
f32* orc_mtx(HSD_JObj* j) { return &j->mtx[0][0]; }
void orc_set_dirty(HSD_JObj* j) { HSD_JObjSetMtxDirty(j); }

HSD_RObjDesc* orc_desc_new(u32 flags, HSD_RObjDesc* prev)
{
    HSD_RObjDesc* d = orc_alloc(sizeof(HSD_RObjDesc));
    d->flags = flags;
    if (prev) prev->next = d;
    return d;
}
void orc_desc_joint(HSD_RObjDesc* d, u32 key) { d->u.joint = (HSD_Joint*) (uintptr_t) key; }
void orc_desc_limit(HSD_RObjDesc* d, f32 v) { d->u.limit = v; }
void orc_desc_ik(HSD_RObjDesc* d, f32 bl, f32 rx)
{
    d->u.ik_hint = orc_alloc(sizeof(HSD_IKHintDesc));
    d->u.ik_hint->bone_length = bl;
    d->u.ik_hint->rotate_x = rx;
}
HSD_RvalueList* orc_rvlist(int n) { return orc_alloc((n + 1) * sizeof(HSD_RvalueList)); }
void orc_rvlist_set(HSD_RvalueList* l, int i, u32 flags, u32 key)
{
    l[i].flags = flags;
    l[i].joint = (HSD_Joint*) (uintptr_t) key;
}
void orc_desc_exp(HSD_RObjDesc* d, f32 (*fn)(void*), HSD_RvalueList* l)
{
    d->u.exp = orc_alloc(sizeof(HSD_ExpDesc));
    d->u.exp->func = fn;
    d->u.exp->rvalue = l;
}
void orc_desc_bc(HSD_RObjDesc* d, u8* code, HSD_RvalueList* l)
{
    d->u.bcexp = orc_alloc(sizeof(HSD_ByteCodeExpDesc));
    d->u.bcexp->bytecode = code;
    d->u.bcexp->rvalue = l;
}
HSD_RObj* orc_robj_next(HSD_RObj* r) { return r->next; }
u32 orc_robj_flags(HSD_RObj* r) { return r->flags; }
u32 orc_robj_limit(HSD_RObj* r) { return *(u32*) &r->u.limit; }
u32 orc_robj_ik(HSD_RObj* r, int i) { return i ? *(u32*) &r->u.ik_hint.rotate_x : *(u32*) &r->u.ik_hint.bone_length; }
HSD_JObj* orc_robj_jobj(HSD_RObj* r) { return r->u.jobj; }
u32 orc_robj_nbargs(HSD_RObj* r) { return r->u.exp.nb_args; }
u32 orc_robj_isbc(HSD_RObj* r) { return r->u.exp.is_bytecode; }
HSD_Rvalue* orc_robj_rv(HSD_RObj* r) { return r->u.exp.rvalue; }
HSD_Rvalue* orc_rv_next(HSD_Rvalue* v) { return v->next; }
u32 orc_rv_flags(HSD_Rvalue* v) { return v->flags; }
HSD_JObj* orc_rv_jobj(HSD_Rvalue* v) { return v->jobj; }
void orc_setup(HSD_JObj* j) { HSD_JObjSetupMatrix(j); }
