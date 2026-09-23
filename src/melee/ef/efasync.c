#include "efasync.h"

#include <math.h>
#include <stdarg.h>

#include "efdata.h"
#include "eflib.h"
#include "efsync.h"
#include "types.h"
#include <melee/cm/camera.h>
#include <melee/lb/lb_00B0.h>
#include <melee/lb/lbarchive.h>
#include <melee/lb/lbdvd.h>
#include <sysdolphin/baselib/generator.h>
#include <sysdolphin/baselib/gobj.h>
#include <sysdolphin/baselib/gobjproc.h>
#include <sysdolphin/baselib/particle.h>
#include <sysdolphin/baselib/psstructs.h>
#include <sysdolphin/baselib/random.h>

HSD_ObjAllocData efAsync_AllocData;

#if defined(TARGET_PC)
static void efAsync_MexNoteArchive(int bank, HSD_Archive* archive);
static void efAsync_MexProcessDeferred(HSD_GObj* gobj, EF_QueuedEffect* q);
#endif

static inline HSD_JObj* efAsync_GetEffectJObj(EF_Effect* effect)
{
    return GET_JOBJ(effect->gobj);
}

static inline void efAsync_SetEffectRotationZFromPtr(EF_Effect* effect,
                                                     const f32* rotation)
{
    HSD_JObjSetRotationZ(GET_JOBJ(effect->gobj), *rotation);
}

static inline void efAsync_SetEffectRandomRotationZ(EF_Effect* effect)
{
    HSD_JObjSetRotationZ(GET_JOBJ(effect->gobj), M_TAU * HSD_Randf());
}

static inline void efAsync_SetEffectScaleXYZ(EF_Effect* effect, f32 scale)
{
    HSD_JObj* jobj;

    jobj = effect->gobj->hsd_obj;
    (void) jobj->scale.x;
    jobj = GET_JOBJ(effect->gobj);
    HSD_JObjSetScaleX(jobj, scale);
    jobj = GET_JOBJ(effect->gobj);
    HSD_JObjSetScaleY(jobj, scale);
    jobj = GET_JOBJ(effect->gobj);
    HSD_JObjSetScaleZ(jobj, scale);
}

static inline void efAsync_SetEffectScale(EF_Effect* effect, Vec3* scale)
{
    HSD_JObjSetScale(GET_JOBJ(effect->gobj), scale);
}

static inline void efAsync_SetEffectFacingDir(EF_Effect* effect,
                                              f32 facing_dir)
{
    f64 rotation;

    if (facing_dir < 0.0f) {
        rotation = -M_PI_2;
    } else {
        rotation = M_PI_2;
    }
    HSD_JObjSetRotationY(GET_JOBJ(effect->gobj), rotation);
}

void* efAsync_Dispatch(s32 gfx_id, HSD_GObj* gobj, va_list vlist)
{
    Vec3 translate;
    Vec3 scale;
    HSD_Generator* generator;
    HSD_psAppSRT* psAppSRT;
    EF_Effect* effect;
    void* va_pos;
    void* va_jobj;
    void* ret_obj;
    f64 rot_y;
    f32 f32_2;
    f32 f32_1;
    s32 u32_2;
    s32 u32_1;
    u32 color;
    HSD_JObj* jobj_2;
    HSD_JObj* jobj_1;
    HSD_JObj* jobj_3;
    Vec3* va_vec3;
    s32 count;
    struct {
        HSD_Generator* generator;
    } state;

    ret_obj = NULL;
    switch (gfx_id) {
    case 0x3E8:
        if (HSD_Randi(8) == 0) {
            ret_obj = efLib_Create_Attach_Pos(9, gobj, va_arg(vlist, Vec3*));
        } else {
            ret_obj = efLib_Create_Attach_Pos(0xA, gobj, va_arg(vlist, Vec3*));
        }
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            f32_1 = (0.04f * (*va_arg(vlist, f32*))) + 0.3f;
            if (f32_1 < 0.3f) {
                f32_1 = 0.3f;
            }
            if (f32_1 > 1.5f) {
                f32_1 = 1.5f;
            }
            scale.x = scale.y = scale.z = f32_1;
            HSD_JObjSetScale(GET_JOBJ(((EF_Effect*) ret_obj)->gobj), &scale);
        }
        break;
    case 0x3E9:
        ret_obj = efLib_CreateGenerator(0xC, va_arg(vlist, Vec3*));
        break;
    case 0x3EA:
        ret_obj = efLib_CreateGenerator(0x14, va_arg(vlist, Vec3*));
        break;
    case 0x3EB:
        ret_obj = efLib_Create_Attach_Pos(7, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
        }
        break;
    case 0x3EC:
        ret_obj = efLib_Create_Attach_Pos(8, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectRandomRotationZ(ret_obj);
        }
        break;
    case 0x3ED:
        va_vec3 = va_arg(vlist, Vec3*);
        f32_1 = va_vec3->x;
        translate = *va_vec3;
        ret_obj = efLib_CreateGenerator(0x50, &translate);
        if (ret_obj != NULL) {
            generator = efLib_CreateGenerator_AddAppSRT(0x54);
            if (generator != NULL) {
                psAppSRT = generator->appsrt;
                psAppSRT->translate = translate;
                if (*va_arg(vlist, f32*) < 0.0f) {
                    rot_y = 0.0;
                } else {
                    rot_y = -M_PI;
                }
                psAppSRT = generator->appsrt;
                psAppSRT->rot.y = rot_y;
            }
        }
        break;
    case 0x3EE:
        ret_obj = efLib_Create_Attach_Pos(0x27, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            efAsync_SetEffectRandomRotationZ(ret_obj);
        }
        break;
    case 0x3EF:
    case 0x3F0:
        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        if (gfx_id == 0x3F0) {
            f32_1 = *va_arg(vlist, f32*);
        } else {
            f32_1 = -*va_arg(vlist, f32*);
        }
        ret_obj =
            efLib_CreateGenerator_Translate_FacingDir(0x42, &translate, f32_1);
        break;
    case 0x3F1:
    case 0x3F2:
        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        if (gfx_id == 0x3F2) {
            f32_1 = *va_arg(vlist, f32*);
        } else {
            f32_1 = -*va_arg(vlist, f32*);
        }
        ret_obj = efLib_CreateGenerator_Translate_FacingDir(0x14B, &translate,
                                                            f32_1);
        break;
    case 0x3F3:
        ret_obj = efLib_CreateGenerator(0xB, va_arg(vlist, Vec3*));
        break;
    case 0x3F4:
        ret_obj = efLib_CreateGenerator(0x48, va_arg(vlist, Vec3*));
        break;
    case 0x3F5:
        ret_obj = efLib_Create_Attach_Pos(0x10, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectFacingDir(ret_obj, *va_arg(vlist, f32*));
        }
        break;
    case 0x3F6:
        ret_obj = efLib_Create_Attach_Pos(0x11, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
        }
        break;
    case 0x3F7:
        ret_obj = efLib_Create_Attach_Pos(0x12, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectFacingDir(ret_obj, *va_arg(vlist, f32*));
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x3F8:
        ret_obj = efLib_Create_Attach_Pos(0x13, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectFacingDir(ret_obj, *va_arg(vlist, f32*));
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x3F9:
        ret_obj = efLib_Create_Attach_Pos(0x14, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectFacingDir(ret_obj, *va_arg(vlist, f32*));
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x3FA:
        ret_obj = efLib_Create_Attach_Pos(0x15, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
        }
        break;
    case 0x3FB:
        ret_obj = efLib_Create_Attach_Pos(0x16, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
        }
        break;
    case 0x3FC:
        ret_obj = efLib_Create_Attach_Pos(0x17, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
        }
        break;
    case 0x3FD:
        ret_obj = efLib_Create_Attach_Pos(3, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectFacingDir(ret_obj, *va_arg(vlist, f32*));
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x3FE:
        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        f32_1 = *va_arg(vlist, f32*);
        ret_obj = efLib_CreateGenerator_Translate_FacingDir(0x107, &translate,
                                                            f32_1);
        break;
    case 0x3FF:
        ret_obj = efLib_Create_Attach_Pos(5, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectFacingDir(ret_obj, *va_arg(vlist, f32*));
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x400:
    case 0x401:
        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        if (gfx_id == 0x401) {
            f32_1 = *va_arg(vlist, f32*);
        } else {
            f32_1 = -*va_arg(vlist, f32*);
        }
        ret_obj =
            efLib_CreateGenerator_Translate_FacingDir(0x5A, &translate, f32_1);
        break;
    case 0x402:
        ret_obj = hsd_8039EFAC(0, 0, 0x59, va_arg(vlist, HSD_JObj*));
        break;
    case 0x403:
        ret_obj = hsd_8039EFAC(0, 0, 0x5E, va_arg(vlist, HSD_JObj*));
        break;
    case 0x404:
        ret_obj = efLib_Create_Attach_Pos(0x18, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x405:
        ret_obj = efLib_CreateGenerator(0x2C, va_arg(vlist, Vec3*));
        break;
    case 0x406:
        ret_obj = efLib_Create_Attach_Pos(4, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x407:
        ret_obj = efLib_CreateGenerator(0x3C, va_arg(vlist, Vec3*));
        break;
    case 0x408: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x3E);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->rot.z = f32_1;
        }
        break;
    }
    case 0x409:
        ret_obj = hsd_8039EFAC(0, 0, 0xE2, va_arg(vlist, HSD_JObj*));
        break;
    case 0x40A: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x241);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->rot.z = f32_1;
        }
        break;
    }
    case 0x40B: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x242);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->rot.z = f32_1;
        }
        break;
    }
    case 0x40C:
        ret_obj = efLib_CreateGenerator(0x19, va_arg(vlist, Vec3*));
        break;
    case 0x40D: {
        state.generator = efLib_CreateGenerator_AddAppSRT(0x19);
        if (state.generator != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = state.generator->appsrt;
            psAppSRT->translate = *va_vec3;
            jobj_1 = GET_JOBJ(gobj);
            (void) jobj_1;
            HSD_JObjGetScale(jobj_1, &scale);
            ret_obj = state.generator;
            state.generator->appsrt->scale.x =
                state.generator->appsrt->scale.y =
                    state.generator->appsrt->scale.z = scale.y;
        }
        break;
    }
    case 0x40E:
        ret_obj = efLib_CreateGenerator(0x43, va_arg(vlist, Vec3*));
        break;
    case 0x40F:
        ret_obj = efLib_CreateGenerator(0xE3, va_arg(vlist, Vec3*));
        break;
    case 0x410:
        ret_obj = efLib_CreateGenerator(0x22A, va_arg(vlist, Vec3*));
        break;
    case 0x411:
        ret_obj = efLib_CreateGenerator(0x4B, va_arg(vlist, Vec3*));
        break;
    case 0x412:
        ret_obj = hsd_8039EFAC(0, 0, 0x13, va_arg(vlist, HSD_JObj*));
        break;
    case 0x413:
        ret_obj = hsd_8039EFAC(0, 0, 0x37, va_arg(vlist, HSD_JObj*));
        break;
    case 0x414:
        ret_obj = hsd_8039EFAC(0, 0, 0xE1, va_arg(vlist, HSD_JObj*));
        break;
    case 0x415:
        ret_obj =
            efLib_Create_AttachChild(0x25, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
            f32_1 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_1;
            efAsync_SetEffectScale(ret_obj, &scale);
        }
        break;
    case 0x416:
        ret_obj = efLib_CreateGenerator(0x196, va_arg(vlist, Vec3*));
        break;
    case 0x417:
        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach(0xB, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            jobj_1 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            u32_1 = va_arg(vlist, u32);
            if (jobj_1 == NULL) {
                jobj_2 = NULL;
            } else {
                jobj_2 = jobj_1->child;
            }
            if ((u32) (u32_1 + 0xFFA00000) == 0x6060U) {
                u32_2 = 0x808080;
            } else {
                u32_2 = 0xFFFFFF;
            }
            efLib_SetTevKonstColor(jobj_2, 1, u32_2, u32_1);
            ((EF_Effect*) ret_obj)->scale_flags |= EF_SCALE_INHERIT;
            efLib_SetParamGfxId(gobj, gfx_id);
            ((EF_Effect*) ret_obj)->update = efLib_Cb_ApplyStoredAlpha;
        }
        break;
    case 0x418:
        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach(0xC, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            jobj_1 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            u32_1 = va_arg(vlist, u32);
            if (jobj_1 == NULL) {
                jobj_2 = NULL;
            } else {
                jobj_2 = jobj_1->child;
            }
            if ((u32) (u32_1 + 0xFFA00000) == 0x6060U) {
                u32_2 = 0x808080;
            } else {
                u32_2 = 0xFFFFFF;
            }
            efLib_SetTevKonstColor(jobj_2, 0, u32_2, u32_1);
            ((EF_Effect*) ret_obj)->scale_flags |= EF_SCALE_INHERIT;
            efLib_SetParamGfxId(gobj, gfx_id);
            ((EF_Effect*) ret_obj)->update = efLib_Cb_ApplyStoredAlpha;
        }
        break;
    case 0x419: {
        HSD_JObj* child;

        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach(0xD, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            jobj_1 = efAsync_GetEffectJObj(ret_obj);
            u32_1 = va_arg(vlist, u32);
            if ((u32) (u32_1 + 0xFFA00000) == 0x6060U) {
                color = 0x808080;
            } else {
                color = 0xFFFFFF;
            }
            u32_2 = color;
            child = HSD_JObjGetChild(jobj_1);
            efLib_SetTevKonstColor(child, 0, u32_2, u32_1);
            jobj_1 = HSD_JObjGetNext(child);
            efLib_SetTevKonstColor(jobj_1, 0, u32_2, u32_1);
            jobj_1 = HSD_JObjGetNext(jobj_1);
            efLib_SetTevKonstColor(jobj_1, 0, u32_2, u32_1);
            jobj_1 = HSD_JObjGetNext(jobj_1);
            efLib_SetTevKonstColor(jobj_1, 0, u32_2, u32_1);
            jobj_1 = HSD_JObjGetNext(jobj_1);
            efLib_SetTevKonstColor(jobj_1, 0, u32_2, u32_1);
            jobj_1 = HSD_JObjGetNext(jobj_1);
            efLib_SetTevKonstColor(jobj_1, 0, u32_2, u32_1);
            jobj_1 = HSD_JObjGetNext(jobj_1);
            efLib_SetTevKonstColor(jobj_1, 0, u32_2, u32_1);
            ((EF_Effect*) ret_obj)->scale_flags |= EF_SCALE_INHERIT;
            efLib_SetParamGfxId(gobj, gfx_id);
            ((EF_Effect*) ret_obj)->update = efLib_Cb_ApplyStoredAlpha;
        }
        break;
    }
    case 0x41A:
        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach(0xE, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            jobj_1 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            u32_1 = va_arg(vlist, u32);
            if (jobj_1 == NULL) {
                jobj_2 = NULL;
            } else {
                jobj_2 = jobj_1->child;
            }
            if ((u32) (u32_1 + 0xFFA00000) == 0x6060U) {
                u32_2 = 0x808080;
            } else {
                u32_2 = 0xFFFFFF;
            }
            efLib_SetTevKonstColor(jobj_2, 0, u32_2, u32_1);
            ((EF_Effect*) ret_obj)->scale_flags |= EF_SCALE_INHERIT;
            efLib_SetParamGfxId(gobj, gfx_id);
            ((EF_Effect*) ret_obj)->update = efLib_Cb_ApplyStoredAlpha;
        }
        break;
    case 0x41B: {
        HSD_Generator* result;
        HSD_JObj* jobj;

        efLib_LoadKind = EF_LOADKIND_SYNC;
        result = efLib_CreateGenerator_AddAppSRT(0x31);
        if (result != NULL) {
            jobj = va_arg(vlist, HSD_JObj*);
            lb_8000B1CC(jobj, NULL, &result->appsrt->translate);
            HSD_JObjGetScale(jobj, &scale);
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = scale.y;
        }
        break;
    }
    case 0x41C:
        ret_obj = efLib_CreateGenerator(0x5D, va_arg(vlist, Vec3*));
        break;
    case 0x41D:
        ret_obj = efLib_Create_Attach_Pos(0xF, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->state_flags |= EF_STATE_ASYNC;
        }
        break;
    case 0x41E:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x55, vlist);
        break;
    case 0x41F:
        ret_obj = efLib_CreateGenerator(0x5C, va_arg(vlist, Vec3*));
        break;
    case 0x420:
        ret_obj = efLib_CreateGenerator(0x159, va_arg(vlist, Vec3*));
        break;
    case 0x421:
        ret_obj = efLib_CreateGenerator(0x3F, va_arg(vlist, Vec3*));
        break;
    case 0x422:
        ret_obj = hsd_8039EFAC(0, 0, 0x5B, va_arg(vlist, HSD_JObj*));
        break;
    case 0x423:
        ret_obj = efLib_Create_Attach(1, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x424:
        ret_obj = efLib_Create_Attach(2, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            efAsync_SetEffectRotationZFromPtr(ret_obj, va_arg(vlist, f32*));
        }
        break;
    case 0x425:
        ret_obj = efLib_CreateGenerator(0x7E, va_arg(vlist, Vec3*));
        break;
    case 0x426:
        ret_obj = efLib_CreateGenerator(0x7F, va_arg(vlist, Vec3*));
        break;
    case 0x427: {
        EF_Effect* node;
        s32 i;

        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        for (i = 0; i < 6; i++) {
            node = efLib_Create_Attach_Pos(0x1B, gobj, &translate);
            if (node == NULL) {
                break;
            }
            node->update = efLib_Cb_Fall_FromParamY;
            node->lifetime = 0x28;
            node->params.y = (0.4f * HSD_Randf()) + 2.8f;
            f32_1 = (M_TAU * HSD_Randf());
            jobj_1 = GET_JOBJ(node->gobj);
            HSD_JObjSetRotationY(jobj_1, f32_1);
            if (i != 0) {
                effect->next = node;
                effect = effect->next;
            } else {
                ret_obj = effect = node;
            }
        }
        break;
    }
    case 0x428:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0xCA, vlist);
        break;
    case 0x429:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0xCE, vlist);
        break;
    case 0x42A:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0xCF, vlist);
        break;
    case 0x42B: {
        HSD_JObj* setter_jobj;

        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach_Pos(0x19, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            jobj_1 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            setter_jobj = jobj_1;
            f32_2 = *va_arg(vlist, f32*);
            HSD_JObjSetRotationZ(setter_jobj, f32_2);
            f32_2 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_2;
            HSD_JObjSetScale(setter_jobj, &scale);
            if (jobj_1 == NULL) {
                jobj_2 = NULL;
            } else {
                jobj_2 = jobj_1->child;
            }
            efLib_SetTevKonstColor(jobj_2, 0, va_arg(vlist, u32),
                                   va_arg(vlist, u32));
        }
        break;
    }
    case 0x42C:
        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach_Pos(0x1A, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            f32_2 = *va_arg(vlist, f32*);
            HSD_JObjSetRotationZ(jobj_2, f32_2);
        }
        break;
    case 0x42D:
        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_CreateGenerator(0x121, va_arg(vlist, Vec3*));
        break;
    case 0x42E:
        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        f32_1 = *va_arg(vlist, f32*);
        ret_obj = efLib_CreateGenerator_Translate_FacingDir(0x13C, &translate,
                                                            f32_1);
        break;
    case 0x42F:
        ret_obj = efLib_Create_Attach_Pos(0x20, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            if (*va_arg(vlist, f32*) < 0.0f) {
                rot_y = -M_PI_2;
            } else {
                rot_y = M_PI_2;
            }
            f32_2 = rot_y;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetRotationY(jobj_2, f32_2);
        }
        break;
    case 0x430:
        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        f32_1 = *va_arg(vlist, f32*);
        ret_obj = efLib_CreateGenerator_Translate_FacingDir(0x140, &translate,
                                                            f32_1);
        break;
    case 0x431:
        ret_obj = efLib_Create_Attach_Pos(0x21, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            if (*va_arg(vlist, f32*) < 0.0f) {
                rot_y = -M_PI_2;
            } else {
                rot_y = M_PI_2;
            }
            f32_2 = rot_y;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetRotationY(jobj_2, f32_2);
        }
        break;
    case 0x432: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x145);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            f32_1 = *va_arg(vlist, f32*);
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x433:
        ret_obj = hsd_8039EFAC(0, 0, 0x115, va_arg(vlist, HSD_JObj*));
        break;
    case 0x434:
        ret_obj = efLib_CreateGenerator(0x14D, va_arg(vlist, Vec3*));
        break;
    case 0x435:
        u32_1 = 0x14E;
        goto block_515;
    case 0x436:
        u32_1 = 0x153;
        goto block_515;
    case 0x437:
        u32_1 = 0x156;
    block_515: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(u32_1);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            result->appsrt->rot.y = M_PI_2;
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->rot.z = f32_1;
        }
        break;
    }
    case 0x438:
        PAD_STACK(4);
        ret_obj = efLib_Create_Attach(0x22, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            jobj_3 = GET_JOBJ(gobj);
            (void) jobj_3;
            HSD_JObjGetRotationY(jobj_3);
            f32_2 = jobj_3->rotate.y;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetRotationY(jobj_2, f32_2);
            ((EF_Effect*) ret_obj)->update = efLib_Cb_SetScale_FromParamX;
            ((EF_Effect*) ret_obj)->params.x = *va_arg(vlist, f32*);
        }
        break;
    case 0x439:
        ret_obj = efLib_Create_Attach(0x23, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_2 = *va_arg(vlist, f32*);
            efAsync_SetEffectScaleXYZ(ret_obj, f32_2);
        }
        break;
    case 0x43A:
        ret_obj = efLib_CreateGenerator(0x193, va_arg(vlist, Vec3*));
        break;
    case 0x43B:
        ret_obj = efLib_CreateGenerator(0x192, va_arg(vlist, Vec3*));
        break;
    case 0x43C:
        ret_obj = efLib_CreateGenerator(0x1A0, va_arg(vlist, Vec3*));
        break;
    case 0x43D:
        ret_obj = hsd_8039EFAC(0, 0, 0x1AF, va_arg(vlist, HSD_JObj*));
        break;
    case 0x43E:
        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach(0x24, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_2 = *va_arg(vlist, f32*);
            efAsync_SetEffectScaleXYZ(ret_obj, f32_2);
        }
        break;
    case 0x43F: {
        HSD_Generator* result;

        efLib_LoadKind = EF_LOADKIND_SYNC;
        result = efLib_CreateGenerator_AddAppSRT(0xCA);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            f32_1 = *va_arg(vlist, f32*);
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x440:
        va_vec3 = va_arg(vlist, Vec3*);
        translate = *va_vec3;
        f32_1 = *va_arg(vlist, f32*);
        ret_obj = efLib_CreateGenerator_Translate_FacingDir(0x1D8, &translate,
                                                            f32_1);
        break;
    case 0x441:
        ret_obj = efLib_CreateGenerator(0x1FB, va_arg(vlist, Vec3*));
        break;
    case 0x442:
        ret_obj = efLib_CreateGenerator(0x1DC, va_arg(vlist, Vec3*));
        break;
    case 0x443: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x1F1);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            f32_1 = *va_arg(vlist, f32*);
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x444:
        ret_obj = efLib_CreateGenerator(0x1FF, va_arg(vlist, Vec3*));
        break;
    case 0x445:
        ret_obj = efLib_CreateGenerator(0x209, va_arg(vlist, Vec3*));
        break;
    case 0x446: {
        HSD_JObj* source_jobj = va_arg(vlist, HSD_JObj*);

        va_vec3 = va_arg(vlist, Vec3*);
        ret_obj = efLib_CreateGenerator_AppSRT_SetPos(0x1B, gobj, source_jobj,
                                                      va_vec3);
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->lifetime = 0x16;
        }
        break;
    }
    case 0x447:
        efLib_LoadKind = EF_LOADKIND_SYNC;
        ret_obj = efLib_Create_Attach_Pos(0x28, gobj, va_arg(vlist, Vec3*));
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->update = efLib_Cb_ftCo_Bury;
            if (gfx_id == 0x447) {
                f32_1 = *va_arg(vlist, f32*);
                scale.x = scale.y = scale.z = f32_1;
                jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
                HSD_JObjSetScale(jobj_2, &scale);
            }
        }
        break;
    case 0x448:
        va_jobj = va_arg(vlist, void*);
        va_pos = va_arg(vlist, void*);
        ret_obj =
            efLib_CreateGenerator_AppSRT_SetPos(0x92, gobj, va_jobj, va_pos);
        if (ret_obj != NULL) {
            ((EF_Effect*) ret_obj)->lifetime = 0xB4;
        }
        break;
    case 0x449:
        ret_obj = efLib_Create_Attach(0x26, gobj, va_arg(vlist, HSD_JObj*));
        break;
    case 0x44A:
        f32_2 = 0.5f;
        goto block_636;
    case 0x44B:
        f32_2 = 1.0f;
        goto block_636;
    case 0x44C:
        f32_2 = 2.0f;
    block_636: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x237);
        if (result != NULL) {
            va_vec3 = va_arg(vlist, Vec3*);
            psAppSRT = result->appsrt;
            psAppSRT->translate = *va_vec3;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_2;
        }
        break;
    }
    case 0x44D:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x48, vlist);
        break;
    case 0x44E:
    case 0x457:
        ret_obj = efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDA, vlist);
        break;
    case 0x44F:
    case 0x458:
        ret_obj = efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDB, vlist);
        break;
    case 0x450:
    case 0x459:
        ret_obj = efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDC, vlist);
        break;
    case 0x451:
    case 0x45A:
        ret_obj = efLib_CreateGenerator_AppSRT_SetFacingDirScale(0xDD, vlist);
        break;
    case 0x453: {
        HSD_Generator* result;

        lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL, &translate);
        result = efLib_CreateGenerator_Translate_FacingDir(
            0x234, &translate, *va_arg(vlist, f32*));
        if (result != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x454: {
        HSD_Generator* result;

        lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL, &translate);
        result = efLib_CreateGenerator_Translate_FacingDir(
            0x235, &translate, *va_arg(vlist, f32*));
        if (result != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x455: {
        HSD_Generator* result;

        lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL, &translate);
        result = efLib_CreateGenerator_Translate_FacingDir(
            0x236, &translate, *va_arg(vlist, f32*));
        if (result != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x456: {
        HSD_Generator* result;

        lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL, &translate);
        result = efLib_CreateGenerator_Translate_FacingDir(
            0x23D, &translate, *va_arg(vlist, f32*));
        if (result != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x452: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x21E);
        if (result != NULL) {
            lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL,
                        &result->appsrt->translate);
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x45B:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x8C, vlist);
        break;
    case 0x45C:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x8D, vlist);
        break;
    case 0x45D: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x23F);
        if (result != NULL) {
            lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL,
                        &result->appsrt->translate);
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x45E: {
        HSD_Generator* result;

        result = efLib_CreateGenerator_AddAppSRT(0x240);
        if (result != NULL) {
            lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL,
                        &result->appsrt->translate);
            f32_1 = *va_arg(vlist, f32*);
            ret_obj = result;
            result->appsrt->scale.x = result->appsrt->scale.y =
                result->appsrt->scale.z = f32_1;
        }
        break;
    }
    case 0x45F:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x8E, vlist);
        break;
    case 0x460:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x99, vlist);
        break;
    case 0x461:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x95, vlist);
        break;
    case 0x462:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x219, vlist);
        break;
    case 0x463:
        ret_obj = efLib_Create_Attach(0x29, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_1;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetScale(jobj_2, &scale);
        }
        break;
    case 0x464:
        ret_obj = efLib_Create_Attach(0x2A, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_1;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetScale(jobj_2, &scale);
        }
        break;
    case 0x465:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x88, vlist);
        break;
    case 0x466:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x89, vlist);
        break;
    case 0x467:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x8A, vlist);
        break;
    case 0x468:
        ret_obj = efLib_Create_Attach(0x2C, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_1;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetScale(jobj_2, &scale);
        }
        break;
    case 0x469:
        ret_obj = efLib_Create_Attach(0x2E, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_1;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetScale(jobj_2, &scale);
        }
        break;
    case 0x46A:
        generator = efLib_CreateGenerator_AddAppSRT(0xAC);
        if (generator != NULL) {
            lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL,
                        &generator->appsrt->translate);
            f32_1 = *va_arg(vlist, f32*);
            generator->appsrt->scale.x = generator->appsrt->scale.y =
                generator->appsrt->scale.z = f32_1;
            jobj_3 = GET_JOBJ(gobj);
            (void) jobj_3;
            f32_2 = HSD_JObjGetRotationY(jobj_3);
            ret_obj = generator;
            generator->appsrt->rot.y = jobj_3->rotate.y;
        }
        break;
    case 0x46B:
        ret_obj = efLib_Create_Attach(0x2B, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_1;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetScale(jobj_2, &scale);
        }
        break;
    case 0x46C:
        generator = efLib_CreateGenerator_AddAppSRT(0x9E);
        if (generator != NULL) {
            lb_8000B1CC(va_arg(vlist, HSD_JObj*), NULL,
                        &generator->appsrt->translate);
            f32_1 = *va_arg(vlist, f32*);
            generator->appsrt->scale.x = generator->appsrt->scale.y =
                generator->appsrt->scale.z = f32_1;
            jobj_3 = gobj->hsd_obj;
            (void) jobj_3;
            f32_2 = HSD_JObjGetRotationY(jobj_3);
            ret_obj = generator;
            generator->appsrt->rot.y = jobj_3->rotate.y;
        }
        break;
    case 0x46D:
        ret_obj = efLib_Create_Attach(0x2D, gobj, va_arg(vlist, HSD_JObj*));
        if (ret_obj != NULL) {
            f32_1 = *va_arg(vlist, f32*);
            scale.x = scale.y = scale.z = f32_1;
            jobj_2 = GET_JOBJ(((EF_Effect*) ret_obj)->gobj);
            HSD_JObjSetScale(jobj_2, &scale);
        }
        break;
    case 0x46E:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0xAE, vlist);
        break;
    case 0x46F:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0xA0, vlist);
        break;
    case 0x470:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x21B, vlist);
        break;
    case 0x471:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x220, vlist);
        break;
    case 0x472:
        ret_obj = efLib_CreateGenerator_AppSRT_SetScale(0x131, vlist);
        break;
    case 0x473:
        ret_obj = hsd_8039EFAC(0, 0, 0x82, va_arg(vlist, HSD_JObj*));
        break;
    case 0x477:
        ret_obj = efLib_CreateGenerator(0x7918, va_arg(vlist, Vec3*));
        break;
    case 0x474:
        ret_obj = efLib_CreateGenerator(0xF7, va_arg(vlist, Vec3*));
        break;
    case 0x475:
        ret_obj = efLib_CreateGenerator(0xFC, va_arg(vlist, Vec3*));
        break;
    case 0x476:
        ret_obj = efLib_CreateGenerator(0xFF, va_arg(vlist, Vec3*));
        break;
    }
    while (efLib_AnimCount != 0) {
        count = efLib_AnimCount - 1;
        efLib_AnimCount = count;
        HSD_JObjAnimAll(((HSD_JObj**) efLib_AnimQueue)[count]);
    }
#if 1
#else
    va_end(vlist);
#endif
    return ret_obj;
}

static char efAsync_803BFD68[] = "EfCoData.dat";
static char efAsync_803BFD78[] = "effCommonDataTable";
static char efAsync_803BFD8C[] = "EfMrData.dat";
static char efAsync_803BFD9C[] = "effMarioDataTable";
static char efAsync_803BFDB0[] = "EfSsData.dat";
static char efAsync_803BFDC0[] = "effSamusDataTable";
static char efAsync_803BFDD4[] = "EfFxData.dat";
static char efAsync_803BFDE4[] = "effFoxDataTable";
static char efAsync_803BFDF4[] = "EfCaData.dat";
static char efAsync_803BFE04[] = "effCaptainDataTable";
static char efAsync_803BFE18[] = "EfKbData.dat";
static char efAsync_803BFE28[] = "effKirbyDataTable";
static char efAsync_803BFE3C[] = "EfLkData.dat";
static char efAsync_803BFE4C[] = "effLinkDataTable";
static char efAsync_803BFE60[] = "EfPkData.dat";
static char efAsync_803BFE70[] = "effPikachuDataTable";
static char efAsync_803BFE84[] = "EfDkData.dat";
static char efAsync_803BFE94[] = "effDonkeyDataTable";
static char efAsync_803BFEA8[] = "EfYsData.dat";
static char efAsync_803BFEB8[] = "effYoshiDataTable";
static char efAsync_803BFECC[] = "EfNsData.dat";
static char efAsync_803BFEDC[] = "effNessDataTable";
static char efAsync_803BFEF0[] = "EfPrData.dat";
static char efAsync_803BFF00[] = "effPurinDataTable";
static char efAsync_803BFF14[] = "EfKpData.dat";
static char efAsync_803BFF24[] = "effKoopaDataTable";
static char efAsync_803BFF38[] = "EfMtData.dat";
static char efAsync_803BFF48[] = "effMewtwoDataTable";
static char efAsync_803BFF5C[] = "EfIcData.dat";
static char efAsync_803BFF6C[] = "effIceclimberDataTable";
static char efAsync_803BFF84[] = "EfPeData.dat";
static char efAsync_803BFF94[] = "effPeachDataTable";
static char efAsync_803BFFA8[] = "EfMsData.dat";
static char efAsync_803BFFB8[] = "effMarsDataTable";
static char efAsync_803BFFCC[] = "EfZdData.dat";
static char efAsync_803BFFDC[] = "effZeldaDataTable";
static char efAsync_803BFFF0[] = "EfLgData.dat";
static char efAsync_803C0000[] = "effLuigiDataTable";
static char efAsync_803C0014[] = "EfGnData.dat";
static char efAsync_803C0024[] = "effGanonDataTable";
static char efAsync_803C0038[] = "EfKbMs.dat";
static char efAsync_803C0044[] = "effKirbyMarsDataTable";
static char efAsync_803C005C[] = "EfKbZd.dat";
static char efAsync_803C0068[] = "effKirbyZeldaDataTable";
static char efAsync_803C0080[] = "EfMnData.dat";
static char efAsync_803C0090[] = "effMenuDataTable";
static char efAsync_803C00A4[] = "EfKbMr.dat";
static char efAsync_803C00B0[] = "effKirbyMarioDataTable";
static char efAsync_803C00C8[] = "EfKbFx.dat";
static char efAsync_803C00D4[] = "effKirbyFoxDataTable";
static char efAsync_803C00EC[] = "EfKbSs.dat";
static char efAsync_803C00F8[] = "effKirbySamusDataTable";
static char efAsync_803C0110[] = "EfKbPk.dat";
static char efAsync_803C011C[] = "effKirbyPikachuDataTable";
static char efAsync_803C0138[] = "EfKbLg.dat";
static char efAsync_803C0144[] = "effKirbyLuigiDataTable";
static char efAsync_803C015C[] = "EfKbCa.dat";
static char efAsync_803C0168[] = "effKirbyCaptainDataTable";
static char efAsync_803C0184[] = "EfKbDk.dat";
static char efAsync_803C0190[] = "effKirbyDonkeyDataTable";
static char efAsync_803C01A8[] = "EfKbKp.dat";
static char efAsync_803C01B4[] = "effKirbyKoopaDataTable";
static char efAsync_803C01CC[] = "EfKbIc.dat";
static char efAsync_803C01D8[] = "effKirbyIceDataTable";
static char efAsync_803C01F0[] = "EfKbGn.dat";
static char efAsync_803C01FC[] = "effKirbyGanonDataTable";
static char efAsync_803C0214[] = "EfKbFe.dat";
static char efAsync_803C0220[] = "effKirbyEmblemDataTable";
static char efAsync_803C0238[] = "EfFeData.dat";
static char efAsync_803C0248[] = "effEmblemDataTable";

#if defined(TARGET_PC)
/* 3C025C */ EF_DAT_Entry efAsync_DatEntries[EF_BANK_MAX] = {
#else
/* 3C025C */ EF_DAT_Entry efAsync_DatEntries[51] = {
#endif
    { efAsync_803BFD68, efAsync_803BFD78, NULL },
    { efAsync_803BFD8C, efAsync_803BFD9C, NULL },
    { efAsync_803BFDB0, efAsync_803BFDC0, NULL },
    { efAsync_803BFDD4, efAsync_803BFDE4, NULL },
    { efAsync_803BFDF4, efAsync_803BFE04, NULL },
    { efAsync_803BFE18, efAsync_803BFE28, NULL },
    { efAsync_803BFE3C, efAsync_803BFE4C, NULL },
    { efAsync_803BFE60, efAsync_803BFE70, NULL },
    { efAsync_803BFE84, efAsync_803BFE94, NULL },
    { efAsync_803BFEA8, efAsync_803BFEB8, NULL },
    { efAsync_803BFECC, efAsync_803BFEDC, NULL },
    { efAsync_803BFEF0, efAsync_803BFF00, NULL },
    { efAsync_803BFF14, efAsync_803BFF24, NULL },
    { efAsync_803BFF38, efAsync_803BFF48, NULL },
    { efAsync_803BFF5C, efAsync_803BFF6C, NULL },
    { efAsync_803BFF84, efAsync_803BFF94, NULL },
    { efAsync_803BFFA8, efAsync_803BFFB8, NULL },
    { efAsync_803BFFCC, efAsync_803BFFDC, NULL },
    { efAsync_803BFFF0, efAsync_803C0000, NULL },
    { efAsync_803C0014, efAsync_803C0024, NULL },
    { efAsync_803C0038, efAsync_803C0044, NULL },
    { efAsync_803C005C, efAsync_803C0068, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { efAsync_803C0080, efAsync_803C0090, NULL },
    { efAsync_803C00A4, efAsync_803C00B0, NULL },
    { efAsync_803C00C8, efAsync_803C00D4, NULL },
    { efAsync_803C00EC, efAsync_803C00F8, NULL },
    { NULL, NULL, NULL },
    { efAsync_803C0110, efAsync_803C011C, NULL },
    { efAsync_803C0138, efAsync_803C0144, NULL },
    { efAsync_803C015C, efAsync_803C0168, NULL },
    { efAsync_803C0184, efAsync_803C0190, NULL },
    { NULL, NULL, NULL },
    { efAsync_803C01A8, efAsync_803C01B4, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { NULL, NULL, NULL },
    { efAsync_803C01CC, efAsync_803C01D8, NULL },
    { efAsync_803C01F0, efAsync_803C01FC, NULL },
    { efAsync_803C0214, efAsync_803C0220, NULL },
    { efAsync_803C0238, efAsync_803C0248, NULL },
    { NULL, NULL, NULL },
};

void efAsync_LoadAsync(int index)
{
    EF_DAT_Entry* entry = &efAsync_DatEntries[index];
#if defined(TARGET_PC)
    efAsync_MexInit();
    if (index >= EF_BANK_MAX || index < 0) {
        return;
    }
#else
    if (index >= 50 || index < 0) {
        return;
    }
#endif

    if (entry->ef_DAT_file == NULL) {
        return;
    }

    lbDvd_800178E8(3, entry->ef_DAT_file, 4, 4, 0, 1U, 4, 4U, index);
}

void efAsync_OnLoad(HSD_Archive* archive, u8* data, u32 length, int index)
{
    EF_DAT_Entry* result;

    lbArchive_InitializeDAT(archive, data, length);
    result = HSD_ArchiveGetPublicAddress(
        archive, efAsync_DatEntries[index].effDataTable_name);
#if defined(TARGET_PC)
    efAsync_MexNoteArchive(index, archive);
#endif
    if ((u32) result->ef_DAT_file | (u32) result->effDataTable_name) {
        psInitDataBankLocate((HSD_Archive*) result->ef_DAT_file,
                             (HSD_Archive*) result->effDataTable_name, NULL);
    }
}

void efAsync_LoadSync(int idx)
{
    EF_DAT_Entry* spC;
    EF_DAT_Entry* lookup;
    lookup = &efAsync_DatEntries[idx];

#if defined(TARGET_PC)
    efAsync_MexInit();
    if (idx >= EF_BANK_MAX || idx < 0) {
        return;
    }
#else
    if (idx >= 50 || idx < 0) {
        return;
    }
#endif
    if (!lookup->ef_DAT_file) {
        return;
    }
    if (lookup->data) {
        return;
    }
    {
#if defined(TARGET_PC)
        HSD_Archive* ef_archive = NULL;
        bool chk = lbArchive_80017040(&ef_archive, lookup->ef_DAT_file, &spC,
                                      lookup->effDataTable_name, 0);
        efAsync_MexNoteArchive(idx, ef_archive);
#else
        bool chk = lbArchive_80017040(NULL, lookup->ef_DAT_file, &spC,
                                      lookup->effDataTable_name, 0);
#endif
        if ((u32) spC->ef_DAT_file | (u32) spC->effDataTable_name) {
            if (chk) {
                psInitDataBankLoad(idx, (void*) spC->ef_DAT_file,
                                   (void*) spC->effDataTable_name, NULL, NULL);
            } else {
                psInitDataBank(idx, (void*) spC->ef_DAT_file,
                               (void*) spC->effDataTable_name, NULL, NULL);
            }
        }
        lookup->data = &spC->data;
    }
}

void efAsync_QueueProcessDeferred(HSD_GObj* gobj,
                                  EF_QueuedEffect* queued_effect)
{
    Vec3 sp4C;
    Vec3 sp40;
    Vec3 sp34;
    Vec3 sp28;
    Vec3 sp1C;
    Vec3 sp10;
    HSD_JObj* jobj;
    u32 gfx_id;
    u8 spawn_kind;

    spawn_kind = queued_effect->spawn_kind;
    gfx_id = queued_effect->gfx_id;
    jobj = queued_effect->jobj;
    switch (spawn_kind) {
    case EF_SPAWN_ATTACH:
        efSync_Spawn(gfx_id, gobj, jobj);
        break;
    case EF_SPAWN_POS:
        lb_8000B1CC(jobj, NULL, &sp4C);
        efSync_Spawn(gfx_id, gobj, &sp4C);
        break;
    case EF_SPAWN_POS_OFFSET:
        lb_8000B1CC(jobj, &queued_effect->params, &sp40);
        efSync_Spawn(gfx_id, gobj, &sp40);
        break;
    case EF_SPAWN_ATTACH_PARAM:
        efSync_Spawn(gfx_id, gobj, jobj, &queued_effect->params);
        break;
    case EF_SPAWN_POS_PARAM:
        lb_8000B1CC(jobj, NULL, &sp34);
        efSync_Spawn(gfx_id, gobj, &sp34, &queued_effect->params);
        break;
    case EF_SPAWN_POS_OFFSET_PARAM:
        lb_8000B1CC(jobj, &queued_effect->params, &sp28);
        efSync_Spawn(gfx_id, gobj, &sp28, &queued_effect->extra1);
        break;
    case EF_SPAWN_POS_OFFSET_PARAM2:
        lb_8000B1CC(jobj, &queued_effect->params, &sp1C);
        efSync_Spawn(gfx_id, gobj, &sp1C, &queued_effect->extra1,
                     &queued_effect->extra2);
        break;
    case EF_SPAWN_ATTACH_OFFSET:
        efSync_Spawn(gfx_id, gobj, jobj, &queued_effect->params);
        break;
    case EF_SPAWN_CAMERA_SHAKE:
        lb_8000B1CC(jobj, &queued_effect->params, &sp10);
        Camera_RequestQuake(gfx_id, &sp10);
        break;
#if defined(TARGET_PC)
    case EF_SPAWN_MEX:
        efAsync_MexProcessDeferred(gobj, queued_effect);
        break;
#endif
    default:
        HSD_ASSERTREPORT(0x7CU, 0, "[EfASync] unknown type %d\n", spawn_kind,
                         jobj);
        break;
    }
    HSD_ObjFree(&efAsync_AllocData, queued_effect);
}

void efAsync_QueueFlush(HSD_GObj* gobj, void* arg_struct)
{
    EF_QueuedEffect* temp_r31;
    EF_QueuedEffect* var_r4;

    var_r4 = ((EF_QueuedEffect*) arg_struct)->next;
    while (var_r4 != NULL) {
        temp_r31 = var_r4->next;
        efAsync_QueueProcessDeferred(gobj, var_r4);
        var_r4 = temp_r31;
    }
    ((EF_QueuedEffect*) arg_struct)->next = NULL;
}

void efAsync_QueueClear(void* arg_struct)
{
    EF_QueuedEffect* temp_r30;
    EF_QueuedEffect* var_r4;

    var_r4 = ((EF_QueuedEffect*) arg_struct)->next;
    while (var_r4 != NULL) {
        temp_r30 = var_r4->next;
        HSD_ObjFree(&efAsync_AllocData, var_r4);
        var_r4 = temp_r30;
    }
    ((EF_QueuedEffect*) arg_struct)->next = NULL;
}

void efAsync_Spawn(HSD_GObj* gobj, void* queue_head, u32 spawn_kind,
                   u32 gfx_id, HSD_JObj* jobj, ...)
{
    va_list vlist;
    Vec3* va_vec3;
    f32* extra1;
    f32* extra2;
    EF_QueuedEffect* queued;
    PAD_STACK(0x4);

    va_start(vlist, jobj);
    queued = HSD_ObjAlloc(&efAsync_AllocData);
    queued->spawn_kind = spawn_kind;
    queued->gfx_id = gfx_id;
    queued->jobj = jobj;
    switch (spawn_kind) {
    case EF_SPAWN_ATTACH:
    case EF_SPAWN_POS:
        break;
    case EF_SPAWN_POS_OFFSET:
        queued->params = *va_arg(vlist, Vec3*);
        break;
    case EF_SPAWN_ATTACH_PARAM:
        queued->params.x = *va_arg(vlist, f32*);
        break;
    case EF_SPAWN_POS_PARAM:
        queued->params.x = *va_arg(vlist, f32*);
        break;
    case EF_SPAWN_POS_OFFSET_PARAM:
        va_vec3 = va_arg(vlist, Vec3*);
        extra2 = va_arg(vlist, f32*);
        queued->params = *va_vec3;
        queued->extra1 = *extra2;
        break;
    case EF_SPAWN_POS_OFFSET_PARAM2:
        va_vec3 = va_arg(vlist, Vec3*);
        extra1 = va_arg(vlist, f32*);
        extra2 = va_arg(vlist, f32*);
        queued->params = *va_vec3;
        queued->extra1 = *extra1;
        queued->extra2 = *extra2;
        break;
    case EF_SPAWN_ATTACH_OFFSET:
        queued->params = *va_arg(vlist, Vec3*);
        break;
    case EF_SPAWN_CAMERA_SHAKE:
        queued->params = *va_arg(vlist, Vec3*);
        break;
    default:
        HSD_ASSERTREPORT(0xF6U, 0, "[EfASync] unknown type %d\n", spawn_kind);
        break;
    }
    va_end(vlist);
    if ((HSD_GObj_CurrentInvokedProc != NULL) &&
        (HSD_GObj_CurrentInvokedProc->s_link < 9U))
    {
        queued->next = ((EF_QueuedEffect*) queue_head)->next;
        ((EF_QueuedEffect*) queue_head)->next = queued;
        return;
    }
    efAsync_QueueProcessDeferred(gobj, queued);
}

void efAsync_QueueInit(void)
{
    HSD_ObjAllocInit(&efAsync_AllocData, sizeof(EF_QueuedEffect),
                     sizeof(EF_QueuedEffect*));
}

#if defined(TARGET_PC)
#include <melee/ft/fighter.h>
#include <melee/ft/types.h>
#include <melee/it/types.h>

extern int Mex_EffectCount(void);
extern const char* Mex_EffectString(int i, int which);
extern u8 ftData_UnkBytePerCharacter[];

/* Ported from m-ex (https://github.com/akaneia/m-ex).
 * Source patches: asm/m-ex/Effect Expansion/{SyncEffect,AsyncEffect,AsyncToSync,Item/AsyncEffect}.asm,
 * asm/m-ex/Standalone Functions/Effect_CreateAsyncObject.asm,
 * asm/m-ex/MnSlChrData - Effect File Names/ (incl. Index effBehaviorTable).
 * Behaviour: effect ids 5000..8999 are a fighter's OWN effects, relative to its effect bank. */

/// effBehaviorTable, the public symbol an m-ex effect archive carries next to its data table:
/// how each of its models / particle generators is placed (the "behaviour type").
typedef struct EfMexBehavior {
    /* +0 */ s32 mdl_num;
    /* +4 */ u8* mdl_type;
    /* +8 */ s32 ptcl_num;
    /* +C */ u8* ptcl_type;
} EfMexBehavior;

static EfMexBehavior* efAsync_MexBhv[EF_BANK_MAX];
static bool efAsync_MexReady;

/// Fill the effect bank table from MxDt.dat (m-ex's table replaces the game's). Retail rows sit at
/// their retail indices on both known builds; a row MxDt leaves empty keeps the retail entry.
void efAsync_MexInit(void)
{
    int n, i;
    if (efAsync_MexReady) {
        return;
    }
    efAsync_MexReady = true;
    n = Mex_EffectCount();
    if (n > EF_BANK_MAX) {
        OSReport("mexeffect: %d effect banks, the particle system holds %d - the rest are "
                 "left out\n",
                 n, EF_BANK_MAX);
        n = EF_BANK_MAX;
    }
    for (i = 0; i < n; i++) {
        char* file = (char*) Mex_EffectString(i, 0);
        char* sym = (char*) Mex_EffectString(i, 1);
        if (file == NULL || sym == NULL) {
            continue;
        }
        efAsync_DatEntries[i].ef_DAT_file = file;
        efAsync_DatEntries[i].effDataTable_name = sym;
    }
    if (n > 0) {
        OSReport("mexeffect: %d effect banks from MxDt.dat\n", n);
    }
}

/// An effect archive just loaded: remember its effBehaviorTable (m-ex's "Index effBehaviorTable").
static void efAsync_MexNoteArchive(int bank, HSD_Archive* archive)
{
    EfMexBehavior* bhv;
    if (bank < 0 || bank >= EF_BANK_MAX || archive == NULL) {
        return;
    }
    bhv = HSD_ArchiveGetPublicAddress(archive, "effBehaviorTable");
    if (bhv != efAsync_MexBhv[bank] && bhv != NULL) {
        OSReport("mexeffect: bank %d (%s): %d model(s), %d particle generator(s) with a "
                 "behaviour\n",
                 bank, efAsync_DatEntries[bank].ef_DAT_file, bhv->mdl_num,
                 bhv->ptcl_num);
    }
    efAsync_MexBhv[bank] = bhv;
}

/// Say a miss once per (fighter kind, id): which fighter wanted which effect and why it was not
/// drawn is the whole point of the log.
static void efAsync_MexMiss(int kind, s32 gfx_id, const char* why)
{
    static s32 seen[256];
    static int nseen;
    s32 key = (kind * 10000 + gfx_id % 100000) * 2 + (gfx_id >= 100000);
    int i;
    for (i = 0; i < nseen; i++) {
        if (seen[i] == key) {
            return;
        }
    }
    if (nseen < (int) ARRAY_SIZE(seen)) {
        seen[nseen++] = key;
    }
    if (gfx_id >= 100000) {
        OSReport("mexeffect: fighter kind %d effect %d %s\n", kind, gfx_id - 100000, why);
        return;
    }
    OSReport("mexeffect: fighter kind %d effect %d not drawn - %s\n", kind, gfx_id, why);
}

int efAsync_MexResolve(HSD_GObj* gobj, s32 gfx_id, s32* final_id, int* is_ptcl,
                       HSD_GObj** owner)
{
    Fighter* fp;
    int kind, src_kind, bank, int_id, copy, type;
    EfMexBehavior* bhv;

    if (gobj == NULL || !EF_MEX_IS_CUSTOM(gfx_id)) {
        return -1;
    }
    /* an item's effect belongs to the fighter that ORIGINALLY owned it */
    if (gobj->classifier == HSD_GOBJ_CLASS_ITEM) {
        gobj = ((Item*) gobj->user_data)->mex_original_owner;
        if (gobj == NULL || gobj->classifier != HSD_GOBJ_CLASS_FIGHTER) {
            return -1;
        }
    }
    if (gobj->classifier != HSD_GOBJ_CLASS_FIGHTER) {
        return -1;
    }
    fp = GET_FIGHTER(gobj);
    kind = fp->kind;
    /* An m-ex Kirby clone with an effect bank of its own (MxDt effect_index is not Kirby's, e.g. the Brawl
     * Meta Knight slot) spawns its OWN effects with 5xxx / 6xxx, as every other m-ex fighter does; only
     * Kirby and clones that share Kirby's bank read 5xxx / 6xxx as the copied fighter's. */
    if (FTKB_IS_KIRBY(kind) &&
        (kind == Ft_Kind_Kirby ||
         ftData_UnkBytePerCharacter[kind] == ftData_UnkBytePerCharacter[Ft_Kind_Kirby]))
    {
        /* Kirby's own ids are the copied fighter's: 5xxx model, 6xxx generator */
        if (gfx_id >= EF_MEX_CPMDL_START) {
            return -1;
        }
        copy = true;
        *is_ptcl = gfx_id >= EF_MEX_PTCL_START;
        int_id = gfx_id - (*is_ptcl ? EF_MEX_PTCL_START : EF_MEX_MDL_START);
    } else {
        copy = gfx_id >= EF_MEX_CPMDL_START;
        *is_ptcl = (gfx_id >= EF_MEX_PTCL_START && gfx_id < EF_MEX_CPMDL_START) ||
                   gfx_id >= EF_MEX_CPPTCL_START;
        int_id = gfx_id % 1000;
    }
    src_kind = copy ? (int) fp->u.kb.hat.kind : kind;
    if (src_kind < 0 || src_kind >= Ft_Kind_Max) {
        return -1;
    }
    bank = ftData_UnkBytePerCharacter[src_kind];
    if (bank < 0 || bank >= EF_BANK_MAX || bank == 0xFF) {
        efAsync_MexMiss(kind, gfx_id, "no effect bank");
        return -1;
    }
    bhv = efAsync_MexBhv[bank];
    if (bhv == NULL || efAsync_DatEntries[bank].data == NULL) {
        if (!FTKB_IS_KIRBY(kind)) {
            efAsync_MexMiss(kind, gfx_id, "its effect bank has no effBehaviorTable (not loaded?)");
        }
        return -1;
    }
    if (int_id >= (*is_ptcl ? bhv->ptcl_num : bhv->mdl_num)) {
        efAsync_MexMiss(kind, gfx_id, "past the bank's behaviour table");
        return -1;
    }
    type = (*is_ptcl ? bhv->ptcl_type : bhv->mdl_type)[int_id];
    *final_id = bank * 1000 + int_id;
    *owner = gobj;
    {
        /* first use of each (fighter, id): which effect it became - the trace that says a
           custom effect ran at all */
        static char buf[128];
        sprintf(buf, "drawn as bank %d (%s) %s %d, behaviour %d, in motion %d", bank,
                efAsync_DatEntries[bank].ef_DAT_file, *is_ptcl ? "generator" : "model", int_id,
                type, (int) fp->motion_id);
        efAsync_MexMiss(kind, gfx_id + 100000, buf);
    }
    return type;
}

/// Effect_CreateAsyncObject + the fighter-command hook: a custom id from a subaction queues one
/// EF_SPAWN_MEX entry on the fighter's effect queue, or spawns it at once outside the fighter
/// procs - exactly like efAsync_Spawn.
void efAsync_MexSpawn(HSD_GObj* gobj, void* queue_head, s32 gfx_id, HSD_JObj* jobj,
                      Vec3* offset, f32 facing, f32 orientation)
{
    EF_QueuedEffect* q;
    s32 final_id;
    int is_ptcl;
    HSD_GObj* owner;
    if (efAsync_MexResolve(gobj, gfx_id, &final_id, &is_ptcl, &owner) < 0) {
        return;
    }
    q = HSD_ObjAlloc(&efAsync_AllocData);
    if (q == NULL) {
        return;
    }
    q->spawn_kind = EF_SPAWN_MEX;
    q->gfx_id = gfx_id;
    q->jobj = jobj;
    q->params = *offset;
    q->extra1 = facing;
    q->extra2 = orientation;
    if (HSD_GObj_CurrentInvokedProc != NULL && HSD_GObj_CurrentInvokedProc->s_link < 9U) {
        q->next = ((EF_QueuedEffect*) queue_head)->next;
        ((EF_QueuedEffect*) queue_head)->next = q;
        return;
    }
    efAsync_QueueProcessDeferred(gobj, q);
}

/// AsyncToSync: turn the queued entry into the efSync_Spawn call its behaviour type expects.
/// Types m-ex leaves unimplemented from the async path spawn nothing, as there.
static void efAsync_MexProcessDeferred(HSD_GObj* gobj, EF_QueuedEffect* q)
{
    Vec3 pos;
    s32 final_id;
    int is_ptcl, type;
    HSD_GObj* owner;
    s32 id = q->gfx_id;

    type = efAsync_MexResolve(gobj, id, &final_id, &is_ptcl, &owner);
    if (type < 0) {
        return;
    }
    if (!is_ptcl) {
        switch (type) {
        case 0: /* DefinePosRot   } all placed at the bone + offset,  */
        case 1: /* UseJointPos    } with facing direction and ground  */
        case 2: /* ..._Ground     } orientation for the sync side     */
        case 3: /* UseJointPosRot */
        case 4: /* UseJointPosFtDir */
            lb_8000B1CC(q->jobj, &q->params, &pos);
            efSync_Spawn(id, gobj, &pos, &q->extra1, &q->extra2);
            break;
        case 5: /* FollowJointPos */
        case 6: /* FollowJointPosRot */
            efSync_Spawn(id, gobj, q->jobj);
            break;
        default:
            break;
        }
        return;
    }
    switch (type) {
    case 0: /* UseJointPos */
        lb_8000B1CC(q->jobj, &q->params, &pos);
        efSync_Spawn(id, gobj, &pos, &q->extra1, &q->extra2);
        break;
    case 3: /* UseJointPosFtDir */
        lb_8000B1CC(q->jobj, &q->params, &pos);
        efSync_Spawn(id, gobj, &pos, &q->extra1);
        break;
    case 5: /* FollowJointPos */
    case 7: /* FollowJointPos_CopyGObjScale */
        efSync_Spawn(id, gobj, q->jobj);
        break;
    case 6: /* FollowJointPos_FtDir */
        efSync_Spawn(id, gobj, q->jobj, &q->extra1);
        break;
    default: /* 1, 2, 4: nothing from the async path in m-ex either */
        break;
    }
}
#endif
