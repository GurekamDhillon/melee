/* Harness for the night-3 round-4 oracle: a gobj whose user_data is a Ground stand-in (map_id at
 * 0x14), and recorders for the five callees that print their call and arguments. Pointers are
 * printed symbolically (G = the gobj, J = its hsd_obj, 0 = NULL, ? = anything else). */
#include <stdio.h>
#include <stdint.h>
#include <string.h>
typedef struct HSD_GObj { void* hsd_obj; void* user_data; } HSD_GObj;
typedef HSD_GObj HGObj;
static unsigned char h_ground[0x40];
static unsigned char h_jobj[0x10];
static HGObj h_gobj;
static void harness_set(int map_id)
{
    memset(h_ground, 0xA5, sizeof h_ground);
    memcpy(h_ground + 0x14, &map_id, 4);
    h_gobj.hsd_obj = h_jobj;
    h_gobj.user_data = h_ground;
}
static void* harness_gobj(void) { return &h_gobj; }
static const char* sym(const void* p)
{
    return p == &h_gobj ? "G" : p == h_jobj ? "J" : p == 0 ? "0" : "?";
}
void* grZakoGenerator_801CAE04(void* d) { printf(" zako(%s)", sym(d)); return 0; }
int Ground_UpdateMapColl(void* g) { printf(" umc(%s)", sym(g)); return 1; }
int Ground_InitMapColl(void* j, int m) { printf(" imc(%s,%d)", sym(j), m); return 1; }
void grAnime_801C8138(void* g, int m, int b) { printf(" anim(%s,%d,%d)", sym(g), m, b); }
void lb_800115F4(void) { printf(" lb()"); }
