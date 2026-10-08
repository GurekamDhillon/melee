/* Executes the production table installer with synthetic m-ex strings; no disc assets. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define Ft_Kind_Max 3
#define SKINS255_NATIVE_TEST 1
typedef struct { char *dat_filename, *joint_name, *matanim_joint_name; } Fighter_CostumeStrings;
typedef struct { void *joint, *x4, *pad_x8, *xC, *x10, *archive; } UnkCostumeStruct;
static Fighter_CostumeStrings *ftData_803C2360[Ft_Kind_Max];
static struct { UnkCostumeStruct *costume_list; unsigned char numCostumes; } CostumeListsForeachCharacter[Ft_Kind_Max];
static size_t allocated;
static void *HSD_MemAlloc(int n) { allocated += n; return malloc(n); }
#define HSD_ASSERTREPORT(line, condition, message) assert(condition)
#include "../../src/melee/ft/ftdata_costumes.inc"
static char names[255][3][32];
static const char *string_at(int kind, int c, int which) {
    assert(kind == 42 && c >= 0 && c < 255 && which >= 0 && which < 3);
    return names[c][which];
}
int main(void) {
    Fighter_CostumeStrings retail[5]; UnkCostumeStruct retail_lists[5];
    int c, w;
    for (c = 0; c < 255; ++c) for (w = 0; w < 3; ++w)
        sprintf(names[c][w], "synthetic_%d_%d", c, w);
    memset(retail_lists, 0x35, sizeof retail_lists);
    for (c = 0; c < 5; ++c) {
        retail[c].dat_filename = names[c][0]; retail[c].joint_name = names[c][1];
        retail[c].matanim_joint_name = names[c][2];
    }
    ftData_803C2360[0] = retail;
    CostumeListsForeachCharacter[0].costume_list = retail_lists;
    CostumeListsForeachCharacter[0].numCostumes = 5;
    ftData_PcInstallCostumes(1, 64, 0, string_at, 42, HSD_MemAlloc);
    assert(CostumeListsForeachCharacter[1].numCostumes == 64);
    for (c = 0; c < 64; ++c) {
        assert(ftData_803C2360[1][c].dat_filename == names[c][0]);
        assert(ftData_803C2360[1][c].joint_name == names[c][1]);
        assert(ftData_803C2360[1][c].matanim_joint_name == names[c][2]);
        assert(!memcmp(&CostumeListsForeachCharacter[1].costume_list[c], &(UnkCostumeStruct){0}, sizeof(UnkCostumeStruct)));
    }
    assert(ftData_803C2360[0] == retail && CostumeListsForeachCharacter[0].costume_list == retail_lists);
    ftData_PcInstallCostumes(0, 255, 5, string_at, 42, HSD_MemAlloc);
    assert(CostumeListsForeachCharacter[0].numCostumes == 255);
    assert(!memcmp(ftData_803C2360[0], retail, sizeof retail));
    assert(!memcmp(CostumeListsForeachCharacter[0].costume_list, retail_lists, sizeof retail_lists));
    for (c = 5; c < 255; ++c) {
        assert(ftData_803C2360[0][c].dat_filename == names[c][0]);
        assert(ftData_803C2360[0][c].joint_name == names[c][1]);
        assert(ftData_803C2360[0][c].matanim_joint_name == names[c][2]);
    }
    assert(allocated == (64 + 255) * (sizeof(Fighter_CostumeStrings) + sizeof(UnkCostumeStruct)));
    puts("skins255: 64 clone and 255 retail-extension costumes resolve; retail strings/runtime preserved; exact-size allocations PASS");
    return 0;
}
