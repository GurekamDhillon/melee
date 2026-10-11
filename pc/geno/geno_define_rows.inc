/* Same TU as the callback registry. Ordinary motion ids keep native semantics. */
extern int Geno_DefineOverrideCount(int kind);
extern int Geno_DefineOverrideField(int kind, int row, int field);
extern MotionState ftData_MotionStateList[];
extern MotionState ftMr_Init_MotionStateTable[];
extern MotionState ftCa_Init_MotionStateTable[];
extern MotionState ftFx_Init_MotionStateTable[];
extern MotionState ftFc_Init_MotionStateTable[];
extern MotionState ftMs_Init_MotionStateTable[];
extern MotionState ftPr_Init_MotionStateTable[];
extern MotionState ftSk_Init_MotionStateTable[];
extern MotionState ftZd_Init_MotionStateTable[];
void GenoGame_DefineApplyRows(int kind, MotionState* common, MotionState* specific, int subactions)
{
    int i, n = Geno_DefineOverrideCount(kind);
    for (i = 0; i < n; ++i) {
        int motion = Geno_DefineOverrideField(kind, i, 0);
        int like = Geno_DefineOverrideField(kind, i, 1);
        MotionState* row = motion < ftCo_MS_Count ? &common[motion] : &specific[motion-ftCo_MS_Count];
        int field, value;
        if (like >= 0) {
            /* Inheritance always reads the original preset, never a prior override. */
            *row = like < ftCo_MS_Count ? ftData_MotionStateList[like] : (Geno_DefineBaseKind(kind) == Ft_Kind_Captain ? ftCa_Init_MotionStateTable : Geno_DefineBaseKind(kind) == Ft_Kind_Fox ? ftFx_Init_MotionStateTable : Geno_DefineBaseKind(kind) == Ft_Kind_Falco ? ftFc_Init_MotionStateTable : Geno_DefineBaseKind(kind) == Ft_Kind_Mars ? ftMs_Init_MotionStateTable : Geno_DefineBaseKind(kind) == Ft_Kind_Purin ? ftPr_Init_MotionStateTable : Geno_DefineBaseKind(kind) == Ft_Kind_Seak ? ftSk_Init_MotionStateTable : Geno_DefineBaseKind(kind) == Ft_Kind_Zelda ? ftZd_Init_MotionStateTable : ftMr_Init_MotionStateTable)[like-ftCo_MS_Count];
        }
        value = Geno_DefineOverrideField(kind, i, 2);
        HSD_ASSERTREPORT(__LINE__, value < subactions, "Geno native state references missing subaction\n");
        if (value >= 0) row->anim_id = value;
        value = Geno_DefineOverrideField(kind, i, 3); if (value >= 0) row->x4_flags = value;
        value = Geno_DefineOverrideField(kind, i, 4); if (value >= 0) row->move_id = (u8)value;
        for (field = 5; field < 9; ++field) {
            const GenoCbDef* cb;
            value = Geno_DefineOverrideField(kind, i, field);
            if (value < 0) continue;
            cb = geno_cb_def(field-5, value);
            HSD_ASSERTREPORT(__LINE__, cb != NULL, "Geno native state callback unavailable\n");
            switch (field) {
            case 5: row->anim_cb = cb->fn; break;
            case 6: row->input_cb = cb->fn; break;
            case 7: row->phys_cb = cb->fn; break;
            case 8: row->coll_cb = cb->fn; break;
            }
        }
    }
}
