#include <assert.h>
#include <stdio.h>
#include "../gameworld/script_stage_render_pass.inc"
int main(void)
{
    int mode,code;
    /* Ordinary kit parts draw once, in the first (mode 2) link-3 walk, before
     * fighters/items/effects; modes 1, 0 and 3 (the last runs after effects and
     * would overwrite non-depth-writing shields, lasers and dust) are skipped. */
    for(mode=0;mode<=3;++mode)
        for(code=0;code<=2;++code)
            assert(script_stage_render_pass(mode,code)==((mode==2 && code!=1) || (mode==3 && code==2)));
    assert(SCRIPT_BACKGROUND_GX_LINK < 5); /* fighter link */
    assert(SCRIPT_BACKGROUND_GX_LINK < 6); /* beam/item link */
    assert(script_background_render_pass(0,1,0));
    assert(!script_background_render_pass(1,1,0));
    assert(!script_background_render_pass(2,1,0));
    assert(!script_background_render_pass(0,0,0));
    for(mode=1;mode<=3;++mode) assert(!script_background_render_pass(0,1,mode));
    puts("background bucket pass and ordinary stage behavior passed");
    return 0;
}
