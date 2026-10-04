/* Expose only native FD's audited group-0 pair, never an m-ex-overridden table. */
StageCallbacks* Ground_StageSlotProofCallbacks(void)
{
    return &grLast_StageCallbacks[0];
}

/* Reproduce case 1 of grLast_8021B920 without its generator/controller,
 * camera globals, RNG or backdrop-cycle callbacks. Slots pause when inactive. */
void Ground_StageSlotFDBackdrop(void)
{
    int i;
    for(i=0;i<5;++i) {
        Ground_GObj* g=Ground_GetMapGObj(grNLa_803E8010[i][0]);
        if(!g)continue;
        grAnime_801C7FF8(g,grNLa_803E8010[i][2],7,grNLa_803E8010[i][3],0,1);
        do_anime(g,i,0);
    }
    if(Ground_GetMapGObj(4))grAnime_801C8098(Ground_GetMapGObj(4),0x10,7,0,0,1);
}
