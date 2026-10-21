/* hand-written: data and port hooks declared without a header prototype */
struct Fighter_ShakeTable_t* Fighter_GrabMashShake;
HSD_GObj** HSD_GObjPLinkHead;
f32 Geno_FallLimit(Fighter* fp, f32 terminal) { abort(); { static f32 z; return z; } }
void Geno_GroundEdge(Fighter* fp, int landing) { abort(); }
void Script_GameEvent(int what, int a, int b, int c, int d) { abort(); }
float ScriptGame_ModValue(int slot, int field, float base) { abort(); { static float z; return z; } }
int ScriptGame_CapsMaxJumps(Fighter* fp) { abort(); { static int z; return z; } }
void (*ftData_OnItemDropExt[1])(HSD_GObj*, s32);
void (*ftData_OnItemDrop[1])(HSD_GObj*, s32);
void (*ftData_OnItemPickup[1])(HSD_GObj*, s32);
void Mex_OnItemDropExtDispatch(int kind, void* gobj, void* arg1, void* vanilla) { abort(); }
void Mex_OnItemDropDispatch(int kind, void* gobj, void* arg1, void* vanilla) { abort(); }
void Mex_OnItemPickup2Dispatch(int kind, void* gobj, void* arg1, void* vanilla) { abort(); }
s32 it_8028B08C(void* a, s32 b) { abort(); { static s32 z; return z; } }
void Mex_OnApplyHeadItemDispatch(int kind, void* gobj, void* vanilla) { abort(); }
void Mex_OnItemInvisibleDispatch(int kind, void* gobj, void* vanilla) { abort(); }
void Mex_OnItemVisibleDispatch(int kind, void* gobj, void* vanilla) { abort(); }
void Mex_OnKnockbackEnterDispatch(int kind, void* gobj, void* vanilla) { abort(); }
void Mex_OnKnockbackExitDispatch(int kind, void* gobj, void* vanilla) { abort(); }
void Mex_OnRemoveHeadItemDispatch(int kind, void* gobj, void* vanilla) { abort(); }
void (*ftData_OnItemInvisible[1])(HSD_GObj*);
void (*ftData_OnItemVisible[1])(HSD_GObj*);
void (*ftData_OnKnockbackEnter[1])(HSD_GObj*);
void (*ftData_OnKnockbackExit[1])(HSD_GObj*);
void (*ftData_UnkMotionStates1[1])(HSD_GObj*);
void (*ftData_UnkMotionStates2[1])(HSD_GObj*);
/* MSL float.c NaN (0x7FFFFFFF) / +inf words read by lbtrigf.c */
float MSL_TrigF_80400770[1] = { __builtin_nanf("0x3FFFFF") };
float MSL_TrigF_80400774[1] = { __builtin_inff() };
