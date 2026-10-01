#ifndef SCRIPT_PARTS_H
#define SCRIPT_PARTS_H

/* Offline diagnostic render overrides. No material data is modified. */
struct HSD_DObj;
int ScriptParts_Draw(struct HSD_DObj* dobj);
int ScriptParts_Tint(struct HSD_DObj* dobj);
void ScriptParts_Forget(struct HSD_DObj* dobj);
void ScriptGame_PartsReset(int owner);

enum {
    PART_I_REFRESH = 0,
    PART_I_COUNT,
    PART_I_OWNER_JOINT,
    PART_I_MATERIAL,
    PART_I_MATERIAL_DATA,
    PART_I_TRIANGLES,
    PART_I_STATUS,
    PART_I_HIDDEN,
    PART_I_SOURCE,
    PART_I_ITEM_KIND,
    PART_I_ITEM_ORDINAL,
    PART_I_PATH,
    PART_I_HASH_LO,
    PART_I_HASH_HI,
    PART_I_FLAGS,
    PART_I_MODELS,
    PART_I_VIS_MODEL,
    PART_I_VIS_STATE,
    PART_I_BODY_JOINT
};
enum {
    PART_F_AREA = 0,
    PART_F_MIN_X,
    PART_F_MIN_Y,
    PART_F_MIN_Z,
    PART_F_MAX_X,
    PART_F_MAX_Y,
    PART_F_MAX_Z,
    PART_F_REGION = 16
};
#define PART_REGIONS 12
enum {
    PART_CTL_COLOR = 1,
    PART_CTL_CLEAR,
    PART_CTL_IDS,
    PART_CTL_IDS_OFF,
    PART_CTL_TINT
};
/* status bits: exact decoder support; measurements never silently substitute
 * vertex counts. */
#define PART_STATUS_DECODE 1
#define PART_STATUS_SHAPE 2
#define PART_STATUS_MAPPING 4
#define PART_STATUS_ALPHA 8
#define PART_STATUS_PALETTE 16

#endif
