/* Complete lifecycle policy. Unusual/streaming stages deliberately fail closed. */
#ifndef SCRIPT_STAGE_SLOT_RETAIL_H
#define SCRIPT_STAGE_SLOT_RETAIL_H
static int st_retail_supported(int kind)
{
    return kind==Gr_Kind_Last || kind==Gr_Kind_Battle ||
           kind==Gr_Kind_Story || kind==Gr_Kind_OldPupupu || kind==Gr_Kind_Izumi;
}
static int st_retail_capacity(int nv,int nl,int nj,int cap)
{
    return nv>0 && nl>0 && nj>0 && cap>0 && nv+cap*2<=2048 &&
           nl+cap<=1536 && nj+cap<=SCRIPT_STAGE_JOINTS;
}
#endif
