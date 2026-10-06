/* Fakes for the atlas-* native tests: a fixed-width text measure and (added in Task 7) a recording sink. */
#ifndef ATLAS_FAKE_H
#define ATLAS_FAKE_H
#include <string.h>
#include "../platform/gw_ui_layout.h"

/* every UTF-8 character is half the role's size wide */
static float fake_width(void *u, int role, const char *s)
{
    int n = 0;
    (void) u;
    for (; *s; s++) if (((unsigned char) *s & 0xC0) != 0x80) n++;
    return 0.5f * (float) at_role_size(role) * (float) n;
}
static const AtTextOps FAKE = { fake_width, 0 };
#endif
