/* C reference for src/tu/ft_0819.rs: src/melee/ft/ft_0819.c cut verbatim into
 * tu_ft_0819_extract.inc by gen/cref/tu_run_s57.py; it_8026BC68 / lb_80014638 / ftCo_800C703C
 * are logging stubs over a random item list. tests/tu_ft_0819.rs repeats the fill order and
 * compares the log, the bitfields and hashes of the native Fighter and Items. */
#include "ft/types.h"
#include "ft/forward.h"
#include "ft/fighter.h"
#include "ft/inlines.h"
#include "ft/ft_0819.h"
#include <melee/it/types.h>
#include <melee/it/inlines.h>
#include <sysdolphin/baselib/gobj.h>

int printf(const char*, ...);
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static HSD_GObj* heads[16];
HSD_GObj** HSD_GObjPLinkHead = heads;
static int g_flag[4], g_hit[4];
static Item items[4];
static HSD_GObj igo[4];
static int idx_of(Item* ip) { return (int) (ip - items); }
bool it_8026BC68(Item_GObj* g) { int i = idx_of(g->user_data); printf("BC68 %d\n", i); return g_flag[i]; }
bool lb_80014638(struct lb_80014638_arg0_t* a, struct lb_80014638_arg1_t* b)
{
    int i = (int) (((u8*) b - (u8*) items) / (int) sizeof(Item));
    printf("L %d %08X %08X %08X %08X %08X %08X\n", i, B(a->x0.x), B(a->x0.y), B(a->x0.z), B(a->xC.x), B(a->xC.y), B(a->xC.z));
    return g_hit[i];
}
void ftCo_800C703C(Fighter_GObj* g) { (void) g; printf("C703C\n"); }

#include "tu_ft_0819_extract.inc"

static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rf(void)
{
    u32 k = rnd() % 16;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    return (float) ((int) (rnd() % 2001) - 1000) / 7.0f;
}
static u32 fnv(const void* p, unsigned n)
{ const unsigned char* b = p; u32 h = 2166136261u; while (n--) { h ^= *b++; h *= 16777619u; } return h; }

static Fighter fp;

int main(void)
{
    int op, k, i;
    HSD_GObj g;
    zero(&g, sizeof g);
    g.user_data = &fp;
    for (op = 0; op < 3; op++) {
        for (k = 0; k < 2000; k++) {
            int ret = 0, n;
            zero(&fp, sizeof fp);
            zero(items, sizeof items);
            zero(igo, sizeof igo);
            rs = 0x81938u + (u32) op * 7919u + (u32) k * 104729u;
            fp.coll_data.cur_pos.x = rf(); fp.coll_data.cur_pos.y = rf();
            fp.coll_data.ecb.left.x = rf(); fp.coll_data.ecb.right.x = rf();
            fp.cur_pos.x = rf(); fp.cur_pos.y = rf();
            fp.co_attrs.x17C = rf();
            fp.dmg.x1930.x0.x = rf(); fp.dmg.x1930.xC.z = rf();
            fp.x2219_b1 = (rnd() % 4) == 0;
            fp.x2223_b4 = (rnd() % 4) == 0;
            fp.player_id = (u8) (rnd() % 6);
            fp.dmg.x1950 = (rnd() % 8) == 0;
            n = (int) (rnd() % 5);
            for (i = 0; i < 4; i++) {
                igo[i].user_data = &items[i];
                igo[i].next = i + 1 < n ? &igo[i + 1] : NULL;
                items[i].pos.x = rf();
                items[i].xCD0 = rf();
                items[i].xCB0_source_ply = (int) (rnd() % 9);
                g_flag[i] = rnd() & 1;
                g_hit[i] = rnd() & 1;
            }
            heads[HSD_GOBJ_PLINK_ITEM] = n ? &igo[0] : NULL;
            printf("OP %d %d\n", op, k);
            switch (op) {
            case 0: ft_80081938(&g); break;
            case 1: ft_800819A8(&g); break;
            case 2: ret = ft_80081A00(&g); break;
            }
            {
                int b1 = fp.x2219_b1, b4 = fp.x2223_b4, b5 = fp.x2223_b5;
                fp.x2219_b1 = 0; fp.x2223_b4 = 0; fp.x2223_b5 = 0;
                printf("R %d %d %d %d %08X\n", ret, b1, b4, b5, fnv(&fp, sizeof fp));
            }
            for (i = 0; i < 4; i++) {
                int own = items[i].xCFC == &g;
                items[i].xCFC = NULL;
                printf("I %d %d %08X\n", i, own, fnv(&items[i], sizeof items[i]));
            }
        }
    }
    return 0;
}
