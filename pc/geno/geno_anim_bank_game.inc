/* Included in ftdata.c under TARGET_PC. A bank can live in MEM1 or AllA's
 * ARAM preload heap; keep its allocation and replace only validated raw bytes.
 * No Fighter may retain x590/x598/x5A4/x5A8 or cached trees from the old bank.
 * Waiting for ALL fighters also covers throw victims using another kind's rows. */
extern int Geno_AnimBankCopy(int kind, unsigned address, unsigned size);
int ftData_AnimBankLimit(void) { return FT_ANIM_BUF_SIZE; }

static void ftData_ClearAnimBank(int kind, void* replacement)
{
    ftData* fd = gFtDataList[kind];
    int i;
    if (fd != NULL) {
        if (fd->xC != NULL) {
            for (i = 0; i < ftData_Table_Unk0[kind].count; ++i)
                fd->xC[i].x14 = 0;
        }
        if (fd->x14 != NULL) {
            for (i = 0; i < ftData_UnkIntPairs[kind].count; ++i)
                fd->x14[i].x14 = 0;
        }
    }
    ftData_Table_Unk0[kind].data = NULL;
    ftData_UnkIntPairs[kind].data = NULL;
    ftData_ThrownRows[kind] = NULL;
    ftData_AnimReloadData[kind] = replacement;
}
static int ftData_InstallAnimBank(int kind, void* bank, unsigned bytes)
{
    /* lbMemory_80014FC8 asserts on exhaustion: reload must never allocate
     * an arbitrary unused bank. A missing preload requires a normal restart. */
    if (bank == NULL) return -4;
    if (!Geno_AnimBankCopy(kind, (unsigned)(uintptr_t)bank, bytes)) return -5;
    ftData_ClearAnimBank(kind, bank);
    ftData_AnimBankBytes[kind] = bytes;
    OSReport("geno: anim bank caches cleared: kind %d, %u bytes; main/demo rows and thrown stand-in\n",
             kind, bytes);
    return 1;
}

/* 1 applied, 0 deferred (live fighter), <0 refused with the old cache intact.
 * Called by the native coordinator before Fighter_Create creates its GObj. */
int ftData_AnimBankReload(int kind, unsigned bytes)
{
    void* bank;
    int i;
    if (HSD_GObjPLinkHead != NULL && HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER] != NULL)
        return 0;
    if (kind < Ft_Kind_Mex0 || kind >= Ft_Kind_Max ||
        !Geno_DefineIsNone(kind) || !ftData_803C23E4[kind] || !bytes)
        return -1;
    bank = ftData_Table_Unk0[kind].data;
    if (bank == NULL) bank = ftData_AnimReloadData[kind];
    for (i = 0; i < Ft_Kind_Max; ++i) {
        if (i != kind && ((bank != NULL && (ftData_Table_Unk0[i].data == bank || ftData_AnimReloadData[i] == bank)) ||
            (ftData_803C23E4[i] != NULL && !strcmp(ftData_803C23E4[i], ftData_803C23E4[kind]))))
            return -2;
    }
    if (bank != NULL) {
        if (ftData_AnimBankBytes[kind] != bytes) return -3;
    } else {
        /* The DVD layer's mounted length is immutable this process. The fresh
         * reader required the same length, so any existing preload fits. */
        int entry;
        if (lbFileGetSize(ftData_803C23E4[kind]) != bytes) return -3;
        entry = DVDConvertPathToEntrynum(lbFileGetFullName(ftData_803C23E4[kind]));
        bank = lbDvd_GetPreloadedArchive(entry);
    }
    return ftData_InstallAnimBank(kind, bank, bytes);
}
