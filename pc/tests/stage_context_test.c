/* Compile the production scope begin/end functions against pointer-only fakes. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
typedef struct{unsigned char prefix[256];int grkind;void* param;void* yakumono_param;void* itemdata;void* map_gobjs[64];void* x280[261];unsigned char tail[128];}StageInfo;
typedef struct{int dynamic_kind;void* param;void* archive;void* models[64];void* markers[261];}ScriptStageSlot;
typedef struct{void* unk0;void* unk4;int other;}UnkArchiveStruct;
static StageInfo stage_info;static ScriptStageSlot* script_slot_callback_context;
static UnkArchiveStruct script_slot_archive_context;
static void* HSD_ArchiveGetPublicAddress(void* p,const char* s){(void)s;return p;}
#include "stage_context_retail.inc"
int main(void)
{
    StageInfo original,saved;ScriptStageSlot slot={0};int i;
    memset(&stage_info,0x59,sizeof stage_info);original=stage_info;
    slot.dynamic_kind=37;slot.param=&slot;slot.archive=&original;
    slot.models[0]=&saved;slot.markers[7]=&slot;
    for(i=0;i<100;++i){script_slot_context_begin(&slot,&saved);
      assert(script_slot_callback_context==&slot && stage_info.grkind==37 && stage_info.param==&slot);
      assert(stage_info.map_gobjs[0]==&saved && stage_info.x280[7]==&slot);
      assert(script_slot_archive_context.unk0==slot.archive);
      memset(&stage_info,0xAA,sizeof stage_info); /* callback writes */
      script_slot_context_end(&saved);
      assert(memcmp(&stage_info,&original,sizeof original)==0 && script_slot_callback_context==NULL);
      {UnkArchiveStruct zero={0};assert(memcmp(&script_slot_archive_context,&zero,sizeof zero)==0);}}
    puts("stage callback scope: 100 byte-exact StageInfo restores and archive-context clears passed");return 0;
}
