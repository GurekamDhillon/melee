#include <assert.h>
#include <stdio.h>
#include "../gameworld/script_stage_owned.h"
int main(void)
{
    StOwned o={0};int object[ST_OWNED_MAX+1],visit,i;
    for(visit=0;visit<100;++visit){
        for(i=0;i<ST_OWNED_MAX;++i)assert(st_owned_add(&o,&object[i],i<4));
        assert(!st_owned_add(&o,&object[ST_OWNED_MAX],0));
        assert(st_owned_add(&o,&object[0],1));assert(o.count==ST_OWNED_MAX);
        st_owned_remove(&o,&object[7]);assert(st_owned_find(&o,&object[7])<0);
        while(o.count)st_owned_remove(&o,o.entry[o.count-1].object);
        assert(o.count==0);
    }
    puts("stage ownership: capacity, duplicate, natural destruction, 100 clean visits passed");return 0;
}
