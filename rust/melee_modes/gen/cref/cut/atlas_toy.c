/* C reference for gmfrontend.c's Fad_ToyWindowKnown (gmfrontend_atlas_toy.inc, which the whole-TU
 * oracle of gmfrontend replaces by stand-ins): the three window rectangles, fat_window and the
 * function, cut verbatim. One line per kind 0..255: kind, result. */
#include <melee/gm/gmscene.h>
#include "mtu_common.h"
//CUT src/melee/gm/gmfrontend_atlas_toy.inc::^static const int FAT_GALLERY_WIN::^}$
//CUT src/melee/gm/gmfrontend_atlas_toy.inc::^int Fad_ToyWindowKnown::^}$
int main(void)
{
    int k;
    for (k = 0; k < 256; k++) printf("%d %d\n", k, Fad_ToyWindowKnown((u8) k));
    return 0;
}
