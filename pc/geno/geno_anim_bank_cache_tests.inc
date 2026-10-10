/* Driven by the native synthetic registry/candidate fixture. Exercises the
 * actual coordinator, game cache gate, byte copy and next-load consumption.
 * No DVD/heap or disc assets. All game globals are restored before returning. */
int ftData_AnimBankCacheTest(int kind, int mode, unsigned address)
{
    static ftData fixture;
    static Fighter_WaitAnimData main_rows[2], demo_rows[1], thrown_rows[2];
    unsigned char* bank = (unsigned char*)(uintptr_t)address;
    HSD_GObj fake_user;
    HSD_GObj* heads[HSD_GOBJ_PLINK_FIGHTER + 1];
    HSD_GObj** saved_heads = HSD_GObjPLinkHead;
    ftData* saved_fd = gFtDataList[kind];
    ftData_UnkCountStruct saved_bank = ftData_Table_Unk0[kind];
    ftData_UnkCountStruct saved_demo = ftData_UnkIntPairs[kind];
    Fighter_WaitAnimData* saved_thrown = ftData_ThrownRows[kind];
    void* saved_ready = ftData_AnimReloadData[kind];
    size_t saved_bytes = ftData_AnimBankBytes[kind];
    char* saved_name = ftData_803C23E4[kind];
    int rc = 0, phase;
    extern void Geno_AnimBankBeforeSpawn(void);
    extern int Geno_AnimBankPhase(void);
    memset(&fixture, 0, sizeof fixture);
    memset(main_rows, 0, sizeof main_rows);
    memset(demo_rows, 0, sizeof demo_rows);
    memset(thrown_rows, 0, sizeof thrown_rows);
    memset(heads, 0, sizeof heads);
    memset(bank, 0x55, 16);
    fixture.xC = main_rows; fixture.x14 = demo_rows;
    main_rows[0].x4 = 0; main_rows[0].x8 = 8; main_rows[0].x14 = 123;
    main_rows[1].x14 = 456; demo_rows[0].x14 = 789;
    gFtDataList[kind] = &fixture;
    ftData_803C23E4[kind] = "__synthetic_anim_bank__.dat";
    ftData_Table_Unk0[kind].count = 2;
    ftData_Table_Unk0[kind].data = mode == 2 ? (void*)0x7ffffff0u : bank;
    ftData_UnkIntPairs[kind].count = 1;
    ftData_UnkIntPairs[kind].data = (void*)demo_rows;
    ftData_ThrownRows[kind] = thrown_rows;
    ftData_AnimReloadData[kind] = NULL;
    ftData_AnimBankBytes[kind] = mode == 1 ? 17 : 16;
    HSD_GObjPLinkHead = heads;
    /* A missing resident/preload must refuse, never allocate an unused bank
     * from lbMemory_80014FC8's asserting heap. */
    if (ftData_InstallAnimBank(kind, NULL, 16) != -4 || main_rows[0].x14 != 123) rc = 1;
    if (mode == 0) heads[HSD_GOBJ_PLINK_FIGHTER] = &fake_user;
    Geno_AnimBankBeforeSpawn();
    phase = Geno_AnimBankPhase();
    if (mode != 3) {
        if (phase != (mode == 0 ? 1 : 3) || bank[9] != 0x55 ||
            main_rows[0].x14 != 123 || main_rows[1].x14 != 456 || demo_rows[0].x14 != 789 ||
            ftData_ThrownRows[kind] != thrown_rows || ftData_AnimReloadData[kind] != NULL ||
            ftData_Table_Unk0[kind].data != (mode == 2 ? (void*)0x7ffffff0u : bank) ||
            ftData_UnkIntPairs[kind].data != (void*)demo_rows) rc = 1;
    } else {
        if (phase != 2 || bank[9] != 17 ||
            ftData_Table_Unk0[kind].data || ftData_UnkIntPairs[kind].data ||
            main_rows[0].x14 || main_rows[1].x14 || demo_rows[0].x14 ||
            ftData_ThrownRows[kind] || ftData_AnimReloadData[kind] != bank ||
            main_rows[0].x4 != 0 || main_rows[0].x8 != 8 ||
            ftData_Table_Unk0[kind].count != 2) rc = 1;
        ftData_80085A14((FighterKind)kind);
        if (ftData_Table_Unk0[kind].data != bank || ftData_AnimReloadData[kind] ||
            main_rows[0].x14 != (uintptr_t)bank || main_rows[1].x14 ||
            bank[9] != 17) rc = 1;
        ftData_ClearAnimBank(kind, bank);
        if (main_rows[0].x14 || demo_rows[0].x14) rc = 1;
    }
    HSD_GObjPLinkHead = saved_heads;
    gFtDataList[kind] = saved_fd; ftData_Table_Unk0[kind] = saved_bank;
    ftData_UnkIntPairs[kind] = saved_demo; ftData_ThrownRows[kind] = saved_thrown;
    ftData_AnimReloadData[kind] = saved_ready; ftData_AnimBankBytes[kind] = saved_bytes;
    ftData_803C23E4[kind] = saved_name;
    return rc;
}
