/* Mid-match visits must not borrow scene preload entry 2001, whose heap also
 * contains live fighter data. Entry zero requests lb_800121FC's own allocation. */
static void grIzumi_StageSlotReflection(HSD_ImageDesc* image)
{
    extern int ScriptGame_StageSlotRetailLoading(void);
    extern void ScriptGame_StageSlotRetailBuffer(void*);
    int private_visit=ScriptGame_StageSlotRetailLoading();
    lb_800121FC(image,80,60,4,private_visit?0:2001);
    if(private_visit)ScriptGame_StageSlotRetailBuffer(image->image_ptr);
}
