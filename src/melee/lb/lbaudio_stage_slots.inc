/* Stage slots preserve the current primary BGM identity without re-running
 * Ground's random track selection, and use its existing validated HPS table. */
#if defined(TARGET_PC)
int lbAudioAx_StageSlotMusic(void)
{
    int i;
    for(i=0;i<LBAX_BGM_N;++i) if(hps_files[i] &&
        !strcmp(cur_hps_stem+7,hps_files[i]) && !strncmp(cur_hps_stem,"/audio/",7)) return i;
    return -1;
}
#endif
