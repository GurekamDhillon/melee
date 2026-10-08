/* Boot-time costume descriptors. Shared by m-ex kinds and retail extensions. */
static void ftData_PcInstallCostumes(int kind, int count, int have,
                                   const char* (*string_at)(int, int, int), int internal,
                                   void* (*allocate)(int))
{
    Fighter_CostumeStrings* strings;
    UnkCostumeStruct* lists;
    int c;
    HSD_ASSERTREPORT(__LINE__, count > 0 && count <= 255 && have <= count,
                     "invalid costume descriptor count\n");
    strings = allocate(count * sizeof *strings);
    lists = allocate(count * sizeof *lists);
    HSD_ASSERTREPORT(__LINE__, strings != NULL && lists != NULL,
                     "costume descriptor allocation exhausted\n");
    memset(lists, 0, count * sizeof *lists);
    for (c = 0; c < count; ++c) {
        if (c < have) {
            strings[c] = ftData_803C2360[kind][c];
            lists[c] = CostumeListsForeachCharacter[kind].costume_list[c];
        } else {
            strings[c].dat_filename = (char*) string_at(internal, c, 0);
            strings[c].joint_name = (char*) string_at(internal, c, 1);
            strings[c].matanim_joint_name = (char*) string_at(internal, c, 2);
        }
    }
    ftData_803C2360[kind] = strings;
    CostumeListsForeachCharacter[kind].costume_list = lists;
    CostumeListsForeachCharacter[kind].numCostumes = count;
}

#ifndef SKINS255_NATIVE_TEST
#include <sysdolphin/baselib/initialize.h>
#include <dolphin/os/OSAlloc.h>
/* Headless proof uses the production installer in retargeted guest memory. */
static char ftData_PcTestNames[255][3][24];
static const char* ftData_PcTestString(int internal, int c, int which)
{
    return ftData_PcTestNames[c][which];
}
int ftData_PcTestCostumes(void)
{
    static u8 arena[32768] ATTRIBUTE_ALIGN(32);
    void* start = OSInitAlloc(arena, arena + sizeof arena, 4);
    int heap = OSCreateHeap(start, arena + sizeof arena);
    int previous = HSD_GetHeap();
    int k, c, w, failed = 0;
    if (heap < 0) return 4;
    HSD_SetHeap(heap);
    int kinds[2] = { Ft_Kind_Max - 1, Ft_Kind_Mario };
    for (c = 0; c < 255; ++c) for (w = 0; w < 3; ++w)
        sprintf(ftData_PcTestNames[c][w], "synthetic_%d_%d", c, w);
    for (k = 0; k < 2; ++k) {
        int kind = kinds[k], count = k == 0 ? 64 : 255;
        struct UnkCostumeList saved = CostumeListsForeachCharacter[kind];
        Fighter_CostumeStrings* strings = ftData_803C2360[kind];
        int have = k == 0 ? 0 : saved.numCostumes;
        ftData_PcInstallCostumes(kind, count, have, ftData_PcTestString, 0, HSD_MemAlloc);
        if (CostumeListsForeachCharacter[kind].numCostumes != count) failed = 1;
        for (c = 0; c < count; ++c) {
            Fighter_CostumeStrings* row = &ftData_803C2360[kind][c];
            if (c < have) {
                if (memcmp(row, &strings[c], sizeof *row) ||
                    memcmp(&CostumeListsForeachCharacter[kind].costume_list[c],
                           &saved.costume_list[c], sizeof(UnkCostumeStruct))) failed = 2;
            } else if (row->dat_filename != ftData_PcTestNames[c][0] ||
                       row->joint_name != ftData_PcTestNames[c][1] ||
                       row->matanim_joint_name != ftData_PcTestNames[c][2]) failed = 3;
        }
        HSD_Free(ftData_803C2360[kind]);
        HSD_Free(CostumeListsForeachCharacter[kind].costume_list);
        ftData_803C2360[kind] = strings;
        CostumeListsForeachCharacter[kind] = saved;
    }
    HSD_SetHeap(previous);
    OSDestroyHeap(heap);
    OSReport("skins255: guest costume descriptors 64/255, retail preserved: %s\n", failed ? "FAIL" : "PASS");
    return failed;
}
#endif
