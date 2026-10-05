/* Slot raw/decoded buffers must fail normally rather than HSD_MemAlloc's
 * assertion. A sum-of-free counter alone does not guarantee a contiguous block. */
#if defined(TARGET_PC)
void* lbHeap_StageSlotTryAlloc(unsigned size)
{
    struct Heap* p=&lbHeap_80431FA0.heap_array[0];
    if(!size || p->status!=LbHeapStatus_Create || p->type!=0) return NULL;
    return OSAllocFromHeap(p->id,size);
}
/* Diagnostic: dump heap 0's allocated/free cells (OSDumpHeap) to the log. */
void lbHeap_StageSlotDump(int mode)
{
    struct Heap* p=&lbHeap_80431FA0.heap_array[0];
    if(p->status!=LbHeapStatus_Create || p->type!=0) return;
    extern void HSD_ObjAllocReport(void);
    extern void OSAllocTrack(int);
    if(mode==2){OSAllocTrack(1);return;}
    OSDumpHeap(p->id);
    HSD_ObjAllocReport();
}
#endif
