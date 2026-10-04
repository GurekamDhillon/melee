/* Read-only legal-stage audit; stdin is private disc data, stdout is counts only. */
#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <io.h>
#include "../gameworld/script_stage_dat.h"
#include "../gameworld/script_stage_dat_models.h"
int main(void)
{
    StDatView v;StDatModels models; unsigned count=0,head=0; int rc,mrc;
    unsigned char* b=malloc(8*1024*1024+1);
    size_t n;
    _setmode(_fileno(stdin),_O_BINARY);
    n=fread(b,1,8*1024*1024+1,stdin);
    rc=st_dat_open(b,n,&v);
    if(!rc) rc=st_dat_collision(&v,767,&count);
    mrc=st_dat_models(&v,&models);
    st_dat_symbol(&v,"map_head",&head);
    printf("%d %u %u %u %u %u %u %d %u %u %u\n",rc,(unsigned)n,v.nv,v.nl,v.nj,
           head && st_dat_span(head,1,16,v.bytes)?st_dat_u32(v.data+head+12):0,
           count*(unsigned)sizeof(StDatLine),mrc,models.joints,models.texture_bytes,models.runtime_allowance);
    free(b); return 0;
}
