/* Standalone real-helper fixture. No disc data, interpreter, or game linkage. */
#include <stdint.h>
#include <stdio.h>
#include <assert.h>
#include <string.h>
#define GW_MEXDT_OFF_ITEM 0x1Cu
#define GW_MEXDT_ITEM_OFF_CUSTOM 0x10u
#define GW_MEXDT_ITEM_OFF_RUNTIME_INDEX 0x14u
#define GW_MEX_CUSTOM_ITEM_START 237u
#define GW_MEX_ITEM_LOGIC_STRIDE 0x3Cu
static uint32_t gw_mexdt,gw_mexdt_base=0x80001000u,gw_mexdt_size=0x80000u;
static uint32_t gw_mem1_size=0x1800000u;
static uint32_t custom=0x80003000u,runtime=0x80002000u,descriptor,states,attributes,model,joint;
static int reads;
static int gw_mexdt_in(uint32_t a,uint32_t len) {
    return a>=gw_mexdt_base && (uint64_t)a+len<=(uint64_t)gw_mexdt_base+gw_mexdt_size;
}
static uint32_t gw_r32(const void* p) {
    uint32_t a=(uint32_t)(uintptr_t)p;
    ++reads; assert(a>=0x80000000u && a<=0x80000000u+gw_mem1_size-4);
    if (a==gw_mexdt+GW_MEXDT_OFF_ITEM) return 0x80001100u;
    if (a==0x80001100u+GW_MEXDT_ITEM_OFF_CUSTOM) return custom;
    if (a==0x80001100u+GW_MEXDT_ITEM_OFF_RUNTIME_INDEX) return runtime;
    if (a==runtime) return descriptor;
    if (a==custom) return states;
    if (a==descriptor) return attributes;
    if (a==descriptor+0x10) return model;
    if (a==model) return joint;
    return 0;
}
#include "../platform/gw_mex_items_query.inc"
int main(void) {
    assert(!gw_Mex_ItemReady(237) && !reads);
    assert(gw_Mex_ItemRangeAvailable(4608,64));
    assert(!gw_Mex_ItemRangeAvailable(2147483647,2));
    assert(!gw_Mex_ItemRangeAvailable(4608,0));
    gw_mexdt=gw_mexdt_base;
    assert(!gw_Mex_ItemReady(237));
    descriptor=0x80090000u; assert(!gw_Mex_ItemReady(237));
    states=0x80091000u; assert(!gw_Mex_ItemReady(237));
    attributes=0x80092000u;model=0x80093000u;joint=0x80094000u;
    assert(gw_Mex_ItemReady(237));
    assert(!gw_Mex_ItemReady(2147483647));
    assert(!gw_Mex_ItemRangeAvailable(4608,64)); /* present zero rows reserved */
    gw_mexdt_size=0x2000u;
    assert(!gw_Mex_ItemRangeAvailable(4608,64)); /* table base at exclusive end is malformed */
    gw_mexdt_size=0x2001u; /* both table bases valid; candidate rows beyond archive tail */
    assert(gw_Mex_ItemRangeAvailable(4608,64));
    custom=0xFFFFFFF0u;
    assert(!gw_Mex_ItemReady(238));
    assert(!gw_Mex_ItemRangeAvailable(4608,64)); /* malformed table conservative */
    assert(!strcmp(gw_Mex_ItemName(277),"mex:277"));
    assert(!gw_Mex_ItemName(236));
    puts("real m-ex item admission helper fixtures PASS"); return 0;
}
