#include <assert.h>
#include <stdio.h>
#include "../platform/gw_model_format.h"

int main(void)
{
    GmCollision c;
    const char *bad[] = {
        "{}", "{\"version\":2,\"lines\":[]}",
        "{\"version\":1,\"lines\":[[\"floor\",1,0,0,0,0]]}",
        "{\"version\":1,\"lines\":[[\"left_wall\",0,0,0,1,2]]}",
        "{\"version\":1,\"lines\":[],}",
        "{\"version\":1,\"version\":1,\"lines\":[]}",
        "{\"version\":1,\"atlas\":\"../escape\",\"lines\":[]}",
        "{\"version\":1,\"lines\":[[\"floor\",0,0,1e999,0,0]]}",
        "{\"version\":1,\"lines\":[]} garbage"
    };
    unsigned i;
    assert(gm_collision_parse("{\"version\":1,\"atlas\":\"kit\",\"lines\":["
           "[\"floor\",-10,0,10,5,3],[\"left_wall\",0,-10,0,0,0]]}", &c));
    assert(c.count == 2 && c.line[0].flags == 3 && c.line[1].kind == 4);
    assert(!strcmp(c.atlas, "kit"));
    for (i = 0; i < sizeof bad / sizeof *bad; ++i)
        assert(!gm_collision_parse(bad[i], &c));
    assert(gm_collision_parse("{\"lines\":[],\"version\":1}", &c));
    assert(gm_model_path("kit/models/deck"));
    assert(!gm_model_path("../deck") && !gm_model_path("C:/deck"));
    assert(!gm_model_path("kit//deck") && !gm_model_path("kit/./deck"));
    puts("model format validation passed");
    return 0;
}
