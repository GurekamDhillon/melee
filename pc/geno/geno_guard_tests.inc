/* Asset-free reproduction: the authored guard lowers the shield compared with
 * the costume's rest pose. A low smash must contact it before shield set-off
 * can run. Exercise the real guard pose and collision code, including an own
 * Guard row at a different index from retail's hard-coded row 38. */
#include <melee/ft/ftdata.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ftanim.h>
#include <melee/ft/kinds/ftCommon/forward.h>
#include <melee/lb/lbanim.h>
#include <melee/lb/lbcollision.h>
#include <sysdolphin/baselib/jobj.h>
#include <sysdolphin/baselib/initialize.h>
#include <sysdolphin/baselib/id.h>
#include <dolphin/os.h>
extern void ftCo_80091E78(Fighter_GObj*, float);
extern int Geno_DefineCKAt(int);
extern void HSD_AObjInitAllocData(void);
extern void HSD_FObjInitAllocData(void);
extern void HSD_VecInitAllocData(void);
extern void HSD_MtxInitAllocData(void);
extern void HSD_RObjInitAllocData(void);

static int test_geno_none_guard_contact(void)
{
    static HSD_Joint rest[4];
    static HSD_Joint guard[4];
    static HSD_Joint* guard_trees[3];
    static ftData_x20 guard_data;
    static FighterBone bones[4];
    static struct ftData_x8 x8;
    static ftCommonData common;
    static FighterPartsTable parts_table;
    static FighterPartsTable* parts_tables[Ft_Kind_Max];
    static Fighter_804D6540_t* anim_parts[Ft_Kind_Max];
    static UnkCostumeStruct costume;
    static Fighter_WaitAnimData rows[40];
    static FigaTrack tracks[2];
    static FigaTree tree;
    /* Headless does not boot HSD's heap. Keep this arena alive: HSD's class
     * pools retain their freed cells after the fixture's joints are removed. */
    static u8 arena[1 << 20] ATTRIBUTE_ALIGN(32);
    static s8 nodes[] = {0, 0, 1, 1, -1};
    /* Two linear s8 keys: Rot y=6, Throw y=-3. */
    static u8 rot_y[] = {0x12, 6, 1, 6, 1};
    static u8 throw_y[] = {0x12, 253, 1, 253, 1};
    HSD_JObj *live, *blend, *j, *b;
    FighterPartsTable** old_parts = ftPartsTable;
    Fighter_804D6540_t** old_anim_parts = Fighter_804D6540;
    struct UnkCostumeList old_costumes;
    ftCommonData* old_common = p_ftCommonData;
    struct ftData_x8* old_x8 = t_ftdata.x8;
    ftData_x20* old_x20 = t_ftdata.x20;
    int old_guard_row = t_rows[ftCo_MS_Guard].anim_id;
    HitCapsule hit;
    void* heap_start;
    int kind, i, rc = 0, old_heap = HSD_GetHeap(), heap;
    if (Geno_TestInstall("{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"guard-test\",\"name\":\"Guard test\",\"base\":\"none\",\"common\":\"melee.common.v1\",\"resources\":\"mod:files\"},\"fighter\":{\"plan\":\"plan.json\",\"animation\":\"bank.dat\",\"costumes\":[{\"file\":\"model.dat\",\"joint\":\"j\"}]}}]}") != 1) {
        TestFail("guard fixture registration"); Geno_TestRestore(); return 1;
    }
    kind = Geno_DefineCKAt(0) - 1;
    t_setup(); t_fp.kind = kind;
    memset(bones, 0, sizeof bones); t_fp.parts = bones;
    heap_start = OSInitAlloc(arena, arena + sizeof arena, 4);
    heap_start = (void*)(((uintptr_t)heap_start + 31) & ~(uintptr_t)31);
    heap = OSCreateHeap(heap_start, arena + sizeof arena);
    if (heap < 0) { TestFail("guard fixture heap"); Geno_TestRestore(); return 1; }
    HSD_SetHeap(heap);
    HSD_AObjInitAllocData(); HSD_FObjInitAllocData(); HSD_IDInitAllocData();
    HSD_VecInitAllocData(); HSD_MtxInitAllocData(); HSD_RObjInitAllocData(); HSD_IDSetup();
    memset(rest, 0, sizeof rest);
    for (i = 0; i < 4; ++i) {
        rest[i].scale.x = rest[i].scale.y = rest[i].scale.z = 1;
        if (i < 3) rest[i].child = &rest[i+1];
    }
    rest[2].position.y = 10;
    rest[3].position.y = -10; /* rest grab anchor on the ground */
    live = HSD_JObjLoadJoint(rest); blend = HSD_JObjLoadJoint(rest);
    j = live; b = blend;
    for (i = 0; i < 4; ++i) {
        t_fp.parts[i].joint = j; t_fp.parts[i].x4_jobj2 = b;
        t_fp.parts[i].flags_b1 = true;
        j = HSD_JObjGetChild(j); b = HSD_JObjGetChild(b);
    }
    ftPartsTable = parts_tables;
    Fighter_804D6540 = anim_parts;
    old_costumes = CostumeListsForeachCharacter[kind];
    parts_table.parts_num = 4; ftPartsTable[kind] = &parts_table;
    costume.joint = rest; CostumeListsForeachCharacter[kind].costume_list = &costume;
    memset(&common, 0, sizeof common); common.x260_startShieldHealth = 60;
    common.x2D4 = common.x2D8 = 1;
    p_ftCommonData = &common;
    memset(&x8, 0, sizeof x8); x8.x10 = 255; x8.x11 = 3;
    t_ftdata.x8 = &x8; t_fp.x108_costume_joint = rest;
    t_fp.x8AC_animSkeleton = blend;
    t_fp.x1C_actionStateList[ftCo_MS_Guard].anim_id = 39;
    memset(rows, 0, sizeof rows); rows[39].x14 = 1;
    t_fp.x24 = rows; t_fp.x58C = 40; t_fp.x5A8 = (void*)1;
    memset(tracks, 0, sizeof tracks);
    for (i = 0; i < 2; ++i) {
        tracks[i].length = 5; tracks[i].obj_type = 6;
        tracks[i].frac_value = tracks[i].frac_slope = 0x60;
    }
    tracks[0].ad_head = rot_y; tracks[1].ad_head = throw_y;
    tree.type = 1; tree.frames = 1; tree.nodes = nodes; tree.tracks = tracks;
    t_fp.x598 = &tree;
    t_fp.x221B_b0 = true; t_fp.shield_health = 60;
    t_fp.co_attrs.initial_shield_size = 2;
    ftCo_80091E78(&t_gobj, 1);
    t_fp.shield_hit.bone = t_fp.parts[3].joint; t_fp.shield_hit.size = 1;
    memset(&hit, 0, sizeof hit);
    hit.x58.x = hit.x4C.x = 2.5f; hit.x58.y = hit.x4C.y = 3;
    hit.scale = 1;
    if (!lbColl_80007BCC(&hit, &t_fp.shield_hit, NULL, 0, 1, 1, 0)) {
        TestFail("authored guard shield must contact the low smash (rest pose misses)"); rc = 1;
    }
    if (fabsf(t_fp.shield_hit.pos.y - 3) > 0.001f) {
        TestFail("guard uses own row 39 and preserves authored shield anchor"); rc = 1;
    }
    /* Guard-on blends from the current pose, and a partial stick tilt must
     * blend toward the same authored base rather than back toward rest. */
    ftAnim_8006FA58(&t_fp, FtPart_TransN, rest[0].child);
    ftCo_80091E78(&t_gobj, 0.5f);
    if (fabsf(bones[2].joint->translate.y + bones[3].joint->translate.y - 1.5f) > 0.001f) {
        TestFail("guard-on half blend retains the authored anchor"); rc = 1;
    }
    t_fp.mv.co.guard.x4 = 0.5f;
    ftCo_80091E78(&t_gobj, 1);
    if (fabsf(bones[2].joint->translate.y + bones[3].joint->translate.y - 3) > 0.001f) {
        TestFail("tilted guard keeps the authored base pose and own row"); rc = 1;
    }
    /* Same attributes and same guard pose through the old descriptor path:
     * retail and a donor definition must continue to use x20, without looking
     * for the own animation row. */
    memcpy(guard, rest, sizeof guard);
    for (i = 0; i < 3; ++i) guard[i].child = &guard[i+1];
    guard[2].position.y = 6; guard[3].position.y = -3;
    guard_trees[2] = &guard[1]; guard_data.x0 = guard_trees;
    t_ftdata.x20 = &guard_data; t_fp.mv.co.guard.x4 = 0;
    parts_tables[Ft_Kind_Mario] = &parts_table;
    for (i = 0; i < 2; ++i) {
        if (i == 0) t_fp.kind = Ft_Kind_Mario;
        else {
            if (Geno_TestInstall("{\"geno\":10,\"fighters\":[{\"define\":{\"key\":\"donor-guard-test\",\"name\":\"Donor guard test\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"}}]}") != 1) {
                TestFail("donor guard fixture registration"); rc = 1; break;
            }
            t_fp.kind = Geno_DefineCKAt(0) - 1;
        }
        ftCo_80091E78(&t_gobj, 1);
        t_fp.shield_hit.skip_update_pos = false;
        if (!lbColl_80007BCC(&hit, &t_fp.shield_hit, NULL, 0, 1, 1, 0) ||
            fabsf(t_fp.shield_hit.pos.y - 3) > 0.001f) {
            TestFail("retail/donor guard keeps the original x20 shield pose"); rc = 1;
        }
    }
    HSD_JObjRemoveAll(live); HSD_JObjRemoveAll(blend);
    HSD_SetHeap(old_heap);
    ftPartsTable = old_parts; CostumeListsForeachCharacter[kind] = old_costumes;
    Fighter_804D6540 = old_anim_parts;
    p_ftCommonData = old_common; t_ftdata.x8 = old_x8; t_ftdata.x20 = old_x20;
    t_rows[ftCo_MS_Guard].anim_id = old_guard_row;
    Geno_TestRestore(); t_setup();
    return rc;
}
