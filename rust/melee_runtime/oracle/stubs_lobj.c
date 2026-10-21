/* Link stubs for the lobj oracle (build_lobj.sh). */
#include <stdlib.h>
#include <stdio.h>
float fabsf(float x) { union { float f; unsigned u; } v; v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }
void* HSD_ObjAlloc(void* d) { (void) d; return calloc(1, 256); }
void HSD_ObjFree(void* d, void* p) { (void) d; (void) p; }
void HSD_ObjAllocInit(void* d, unsigned s, unsigned a) { (void) d; (void) s; (void) a; }
void HSD_Panic(const char* f, int l, const char* m) { fprintf(stderr, "panic %s:%d %s\n", f, l, m); exit(4); }
void __assert(const char* f, int l, const char* m) { fprintf(stderr, "assert %s:%d %s\n", f, l, m); exit(4); }
void OSReport(const char* m, ...) { (void) m; }
/* Unreached by the driver: abort if called. */
void HSD_AObjInitEndCallBack(void) { fprintf(stderr, "unexpected HSD_AObjInitEndCallBack\n"); exit(3); }
void HSD_AObjInterpretAnim(void) { fprintf(stderr, "unexpected HSD_AObjInterpretAnim\n"); exit(3); }
void HSD_AObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_AObjLoadDesc\n"); exit(3); }
void HSD_AObjRemove(void) { fprintf(stderr, "unexpected HSD_AObjRemove\n"); exit(3); }
void HSD_AObjReqAnim(void) { fprintf(stderr, "unexpected HSD_AObjReqAnim\n"); exit(3); }
void HSD_JObjDispSub(void) { fprintf(stderr, "unexpected HSD_JObjDispSub\n"); exit(3); }
void HSD_JObjMakePositionMtx(void) { fprintf(stderr, "unexpected HSD_JObjMakePositionMtx\n"); exit(3); }
void HSD_RObjAddAnimAll(void) { fprintf(stderr, "unexpected HSD_RObjAddAnimAll\n"); exit(3); }
void HSD_RObjAnimAll(void) { fprintf(stderr, "unexpected HSD_RObjAnimAll\n"); exit(3); }
void HSD_RObjRemoveAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_RObjRemoveAnimAllByFlags\n"); exit(3); }
void HSD_RObjReqAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_RObjReqAnimAllByFlags\n"); exit(3); }
void hsdInitClassInfo(void) { fprintf(stderr, "unexpected hsdInitClassInfo\n"); exit(3); }
void ProfBegin(void) {}
int ProfActive(void) { return 0; }
char hsdObj[64];
void HSD_AObjInvokeCallBacks(void) { fprintf(stderr, "unexpected HSD_AObjInvokeCallBacks\n"); exit(3); }
void HSD_IDGetDataFromTable(void) { fprintf(stderr, "unexpected HSD_IDGetDataFromTable\n"); exit(3); }
void HSD_IDInsertToTable(void) { fprintf(stderr, "unexpected HSD_IDInsertToTable\n"); exit(3); }
void HSD_IDRemoveByIDFromTable(void) { fprintf(stderr, "unexpected HSD_IDRemoveByIDFromTable\n"); exit(3); }
void HSD_JObjDisp(void) { fprintf(stderr, "unexpected HSD_JObjDisp\n"); exit(3); }
void HSD_RObjGetGlobalPosition(void) { fprintf(stderr, "unexpected HSD_RObjGetGlobalPosition\n"); exit(3); }
void HSD_RObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_RObjLoadDesc\n"); exit(3); }
void HSD_RObjRemoveAll(void) { fprintf(stderr, "unexpected HSD_RObjRemoveAll\n"); exit(3); }
void HSD_RObjResolveRefsAll(void) { fprintf(stderr, "unexpected HSD_RObjResolveRefsAll\n"); exit(3); }
void hsdIsDescendantOf(void) { fprintf(stderr, "unexpected hsdIsDescendantOf\n"); exit(3); }
void* hsdNew(void* c) { (void) c; return calloc(1, 256); }
void hsdSearchClassInfo(void) { fprintf(stderr, "unexpected hsdSearchClassInfo\n"); exit(3); }
void ProfEnd(void) {}
void diag_jobj_mtx(const float* m, int s, int b) { (void) m; (void) s; (void) b; }
int MSL_TrigF_80400770[] = { 0x7FFFFFFF }; /* src/MSL/float.c */
int MSL_TrigF_80400774[] = { 0x7F800000 };
void HSD_MObjAddAnim(void) { fprintf(stderr, "unexpected HSD_MObjAddAnim\n"); exit(3); }
void HSD_MObjAnim(void) { fprintf(stderr, "unexpected HSD_MObjAnim\n"); exit(3); }
void HSD_MObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_MObjLoadDesc\n"); exit(3); }
void HSD_MObjRemove(void) { fprintf(stderr, "unexpected HSD_MObjRemove\n"); exit(3); }
void HSD_MObjRemoveAnimByFlags(void) { fprintf(stderr, "unexpected HSD_MObjRemoveAnimByFlags\n"); exit(3); }
void HSD_MObjReqAnimByFlags(void) { fprintf(stderr, "unexpected HSD_MObjReqAnimByFlags\n"); exit(3); }
void HSD_MObjSetCurrent(void) { fprintf(stderr, "unexpected HSD_MObjSetCurrent\n"); exit(3); }
void HSD_PObjAddAnimAll(void) { fprintf(stderr, "unexpected HSD_PObjAddAnimAll\n"); exit(3); }
void HSD_PObjAnimAll(void) { fprintf(stderr, "unexpected HSD_PObjAnimAll\n"); exit(3); }
void HSD_PObjLoadDesc(void) { fprintf(stderr, "unexpected HSD_PObjLoadDesc\n"); exit(3); }
void HSD_PObjRemoveAll(void) { fprintf(stderr, "unexpected HSD_PObjRemoveAll\n"); exit(3); }
void HSD_PObjRemoveAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_PObjRemoveAnimAllByFlags\n"); exit(3); }
void HSD_PObjReqAnimAllByFlags(void) { fprintf(stderr, "unexpected HSD_PObjReqAnimAllByFlags\n"); exit(3); }
void HSD_PObjResolveRefsAll(void) { fprintf(stderr, "unexpected HSD_PObjResolveRefsAll\n"); exit(3); }
void HSD_RObjRemoveAnimAll(void) { fprintf(stderr, "unexpected HSD_RObjRemoveAnimAll\n"); exit(3); }
void HSD_StateInvalidate(void) { fprintf(stderr, "unexpected HSD_StateInvalidate\n"); exit(3); }
void ScriptParts_Draw(void) { fprintf(stderr, "unexpected ScriptParts_Draw\n"); exit(3); }
void ScriptParts_Forget(void) { fprintf(stderr, "unexpected ScriptParts_Forget\n"); exit(3); }
void ScriptParts_Tint(void) { fprintf(stderr, "unexpected ScriptParts_Tint\n"); exit(3); }
void hsdClass(void) { fprintf(stderr, "unexpected hsdClass\n"); exit(3); }
void HSD_RObjReqAnimAll(void) { fprintf(stderr, "unexpected HSD_RObjReqAnimAll\n"); exit(3); }
void GXSetProjection(void) { fprintf(stderr, "unexpected GXSetProjection\n"); exit(3); }
void GXSetScissor(void) { fprintf(stderr, "unexpected GXSetScissor\n"); exit(3); }
void GXSetViewport(void) { fprintf(stderr, "unexpected GXSetViewport\n"); exit(3); }
void GXSetViewportJitter(void) { fprintf(stderr, "unexpected GXSetViewportJitter\n"); exit(3); }
void HSD_EraseRect(void) { fprintf(stderr, "unexpected HSD_EraseRect\n"); exit(3); }
void HSD_GetCurrentRenderPass(void) { fprintf(stderr, "unexpected HSD_GetCurrentRenderPass\n"); exit(3); }
void VIGetNextField(void) { fprintf(stderr, "unexpected VIGetNextField\n"); exit(3); }
void _HSD_ZListClear(void) { fprintf(stderr, "unexpected _HSD_ZListClear\n"); exit(3); }
void _HSD_ZListDisp(void) { fprintf(stderr, "unexpected _HSD_ZListDisp\n"); exit(3); }
void _HSD_ZListSort(void) { fprintf(stderr, "unexpected _HSD_ZListSort\n"); exit(3); }
char HSD_VIData[4096];
int orc_update_log(void* j) { (void) j; return 0; }
void GXInitLightAttn(void) { fprintf(stderr, "unexpected GXInitLightAttn\n"); exit(3); }
void GXInitLightColor(void) { fprintf(stderr, "unexpected GXInitLightColor\n"); exit(3); }
void GXInitLightDir(void) { fprintf(stderr, "unexpected GXInitLightDir\n"); exit(3); }
void GXInitLightDistAttn(void) { fprintf(stderr, "unexpected GXInitLightDistAttn\n"); exit(3); }
void GXInitLightPos(void) { fprintf(stderr, "unexpected GXInitLightPos\n"); exit(3); }
void GXInitLightSpot(void) { fprintf(stderr, "unexpected GXInitLightSpot\n"); exit(3); }
void GXLoadLightObjImm(void) { fprintf(stderr, "unexpected GXLoadLightObjImm\n"); exit(3); }
void HSD_CObjGetCurrent(void) { fprintf(stderr, "unexpected HSD_CObjGetCurrent\n"); exit(3); }
