/* Helpers shared by the gr TU state oracles (tu_gr*_ref.c); mirrored token for token by
 * rust/melee_world/tests/common/gr.rs. Included after the game TU, oracle.h and before the
 * generated stubs. */
#ifndef TU_GR_COMMON_H
#define TU_GR_COMMON_H
#include <stdarg.h>

void __assert(const char* f, u32 l, const char* c) { printf("ASSERT %s", c); exit(7); }

static HSD_JObj* mkjobj(void)
{
    HSD_JObj* j = (HSD_JObj*) AZ(sizeof(HSD_JObj));
    j->flags = rnd();
    j->translate.x = rf(); j->translate.y = rf(); j->translate.z = rf();
    j->scale.x = rf(); j->scale.y = rf(); j->scale.z = rf();
    return j;
}
/* Every Ground made since gr_reset(). The bitfield bytes of Ground (x10_flags, x11_flags) hold the
 * C compiler's bit order (LSB first natively, MSB first in melee_types), so the state hash sees
 * them only through gnd_canon_begin/end: the fields by value, the byte zeroed while hashing. */
static Ground* GPS[16];
static int NGP;
static void gr_reset(void)
{
    arena_reset();
    NGP = 0;
}
static Ground* mkground(void)
{
    Ground* g = (Ground*) AZ(sizeof(Ground));
    g->map_id = rnd() % 60;
    GPS[NGP++] = g;
    return g;
}
static unsigned char GSV[16][2];
static void gnd_canon_begin(void)
{
    int i;
    for (i = 0; i < NGP; i++) {
        Ground* g = GPS[i];
        unsigned char* p10 = (unsigned char*) &g->x10_flags;
        unsigned char* p11 = (unsigned char*) &g->x11_flags;
        printf("g%d x10 %d%d%d%d%d%d%d%d x11 %d %d %d %d %d %d\n", i, (int) g->x10_flags.b0, (int) g->x10_flags.b1,
               (int) g->x10_flags.b2, (int) g->x10_flags.b3, (int) g->x10_flags.b4, (int) g->x10_flags.b5,
               (int) g->x10_flags.b6, (int) g->x10_flags.b7, (int) g->x11_flags.b012, (int) g->x11_flags.b3,
               (int) g->x11_flags.b4, (int) g->x11_flags.b5, (int) g->x11_flags.b6, (int) g->x11_flags.b7);
        GSV[i][0] = *p10;
        GSV[i][1] = *p11;
        *p10 = 0;
        *p11 = 0;
    }
}
static void gnd_canon_end(void)
{
    int i;
    for (i = 0; i < NGP; i++) {
        *(unsigned char*) &GPS[i]->x10_flags = GSV[i][0];
        *(unsigned char*) &GPS[i]->x11_flags = GSV[i][1];
    }
}
static HSD_GObj* mkgobj(void)
{
    HSD_GObj* g = (HSD_GObj*) AZ(sizeof(HSD_GObj));
    g->hsd_obj = mkjobj();
    g->user_data = mkground();
    return g;
}

/* trace of a string argument that is a __FILE__: only the file name */
static void trfile(const char* s)
{
    const char* b = s;
    const char* p;
    if (s == 0) { printf(" null"); return; }
    for (p = s; *p; p++) if (*p == 47 || *p == 92) b = p + 1;
    printf(" \"%s\"", b);
}
/* variadic OSReport: the gr TUs pass %s (a __FILE__), %d values */
void OSReport(char* fmt, ...)
{
    va_list ap;
    const char* p;
    printf("OSReport");
    TRS(fmt);
    va_start(ap, fmt);
    for (p = fmt; *p; p++) {
        if (*p != '%') continue;
        p++;
        if (*p == 's') trfile(va_arg(ap, const char*));
        else if (*p == 'd') TRI(va_arg(ap, int));
    }
    va_end(ap);
    printf("\n");
}
/* variadic grAnime_801C752C(jobj, a, b, fn, n, flags) */
void grAnime_801C752C(HSD_JObj* j, s32 a, s32 b, void* f, u32 n, ...)
{
    va_list ap;
    u32 flags;
    va_start(ap, n);
    flags = va_arg(ap, u32);
    va_end(ap);
    printf("grAnime_801C752C");
    TRP(j); TRI(a); TRI(b); TRP(f); TRU(n); TRU(flags);
    printf("\n");
}
#endif
