/* Fixed storage: ownership tracking cannot consume the stage heap. */
#ifndef SCRIPT_STAGE_OWNED_H
#define SCRIPT_STAGE_OWNED_H
#define ST_OWNED_MAX 512
typedef struct { void* object; unsigned char borrowed; } StOwnedEntry;
typedef struct { StOwnedEntry entry[ST_OWNED_MAX]; int count; } StOwned;
static int st_owned_find(const StOwned* o, const void* p)
{ int i; for(i=0;i<o->count;++i)if(o->entry[i].object==p)return i;return -1; }
static int st_owned_add(StOwned* o,void* p,int borrowed)
{
    int i=st_owned_find(o,p);
    if(!p)return 0;
    if(i>=0)return 1;
    if(o->count==ST_OWNED_MAX)return 0;
    o->entry[o->count].object=p;o->entry[o->count++].borrowed=borrowed!=0;return 1;
}
static void st_owned_remove(StOwned* o,const void* p)
{ int i=st_owned_find(o,p);if(i>=0)o->entry[i]=o->entry[--o->count]; }
#endif
