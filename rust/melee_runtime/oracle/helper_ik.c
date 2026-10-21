/* Oracle helper for the jobj.c IK / JObjUpdateFunc round (build_ik.sh), compiled with the game
 * flags next to helper_robj.c. */
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/robj.h>

void orc_set_robj(HSD_JObj* j, HSD_RObj* r) { j->robj = r; }
HSD_RObj* orc_get_robj(HSD_JObj* j) { return j->robj; }
void orc_update(HSD_JObj* j, int type, u32* w) { JObjUpdateFunc(j, type, (HSD_ObjData*) w); }
