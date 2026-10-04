/* Read-only admission checks. Never use the panic-producing creation getters.
 * MxDt has no explicit Custom/RuntimeIndex row counts: archive containment is
 * deliberately conservative, including zero rows and possible trailing data. */
static int gw_mex_items_tables(uint32_t* custom, uint32_t* runtime)
{
    uint32_t item;
    if (!gw_mexdt || !gw_mexdt_in(gw_mexdt, GW_MEXDT_OFF_ITEM+4u)) return 0;
    item=gw_r32((const void*)(uintptr_t)(gw_mexdt+GW_MEXDT_OFF_ITEM));
    if (!item || !gw_mexdt_in(item,GW_MEXDT_ITEM_OFF_RUNTIME_INDEX+4u)) return 0;
    *custom=gw_r32((const void*)(uintptr_t)(item+GW_MEXDT_ITEM_OFF_CUSTOM));
    *runtime=gw_r32((const void*)(uintptr_t)(item+GW_MEXDT_ITEM_OFF_RUNTIME_INDEX));
    return 1;
}
static int gw_mex_items_row(uint32_t table,uint32_t index,uint32_t stride,uint32_t* row)
{
    uint64_t at=(uint64_t)table+(uint64_t)index*stride;
    if (!table || at>0xFFFFFFFFu || !gw_mexdt_in((uint32_t)at,stride)) return 0;
    *row=(uint32_t)at; return 1;
}
static int gw_mex_items_guest(uint32_t at,uint32_t size)
{
    return at>=0x80000000u && (uint64_t)at+size <= (uint64_t)0x80000000u+gw_mem1_size;
}
int gw_Mex_ItemReady(int kind)
{
    uint32_t custom,runtime,logic,slot,desc,states,index,attributes,model,joint;
    if (kind<(int)GW_MEX_CUSTOM_ITEM_START || !gw_mex_items_tables(&custom,&runtime)) return 0;
    index=(uint32_t)kind-GW_MEX_CUSTOM_ITEM_START;
    if (!gw_mex_items_row(custom,index,GW_MEX_ITEM_LOGIC_STRIDE,&logic) ||
        !gw_mex_items_row(runtime,index,4u,&slot)) return 0;
    desc=gw_r32((const void*)(uintptr_t)slot);
    states=gw_r32((const void*)(uintptr_t)logic);
    if(!gw_mex_items_guest(desc,0x18u)||!gw_mex_items_guest(states,0x10u)) return 0;
    attributes=gw_r32((const void*)(uintptr_t)desc);
    model=gw_r32((const void*)(uintptr_t)(desc+0x10u));
    if(!gw_mex_items_guest(attributes,0x84u)||!gw_mex_items_guest(model,0x10u)) return 0;
    joint=gw_r32((const void*)(uintptr_t)model);
    return gw_mex_items_guest(joint,0x40u);
}
int gw_Mex_ItemRangeAvailable(int base,int count)
{
    uint32_t custom,runtime,logic,slot,index;
    uint64_t last;
    if (base<(int)GW_MEX_CUSTOM_ITEM_START || count<=0) return 0;
    last=(uint64_t)(unsigned)base+(unsigned)count-1u;
    if (last>0x7FFFFFFFu) return 0;
    if (!gw_mexdt) return 1;
    if (!gw_mex_items_tables(&custom,&runtime)) return 0;
    /* Table capacity inferred from its archive tail is contiguous. If the
     * first candidate row fits in both arrays it is reserved, even all-zero.
     * A later row cannot fit if this first row already lies beyond a tail. */
    if ((custom && !gw_mexdt_in(custom,1u)) ||
        (runtime && !gw_mexdt_in(runtime,1u))) return 0;
    index=(uint32_t)base-GW_MEX_CUSTOM_ITEM_START;
    return !(gw_mex_items_row(custom,index,GW_MEX_ITEM_LOGIC_STRIDE,&logic) &&
             gw_mex_items_row(runtime,index,4u,&slot));
}
const char* gw_Mex_ItemName(int kind)
{
    static char alias[32];
    if (kind<(int)GW_MEX_CUSTOM_ITEM_START) return NULL;
    /* No item names table is known in this MxDt layout. This is an explicit
     * numeric alias, never a guessed fighter/article display name. */
    snprintf(alias,sizeof alias,"mex:%d",kind);
    return alias;
}
