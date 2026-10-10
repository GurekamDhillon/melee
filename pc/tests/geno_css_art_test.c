/* Production select/results adapters against synthetic images. No disc, GPU or window.
 * Regressions: own art loses to the donor; template costumes suppress disc art;
 * missing package files hide a retail results card; base-none shows Mario.
 * HSD/GX stand in for scene allocation/animation, not the selection code. */
#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
typedef unsigned char u8;
typedef unsigned short u16;
typedef unsigned int u32;
typedef signed short s16;
typedef signed char s8;
typedef int CharacterKind;
typedef struct FObj { struct FObj* next; u8* ad_head; u32 length; u8 obj_type, frac_value, frac_slope; s16 startframe; } HSD_FObj;
typedef struct { HSD_FObj* fobj; } HSD_AObj;
enum { HSD_A_T_TIMG=1, HSD_A_OP_CON=1, HSD_A_OP_LIN=2, HSD_A_OP_SPL0=3, HSD_A_OP_SPL=4, HSD_A_OP_SLP=5, HSD_A_OP_KEY=6 };
static u8 image_keys[] = {HSD_A_OP_KEY, 1, 44}; /* synthetic TIMG index300 */
typedef float f32;
typedef int GXTexFmt;
typedef struct { void* image_ptr; u16 width, height; int format, mipmap; float minLOD, maxLOD; } HSD_ImageDesc;
typedef struct { void* lut; int fmt, n_entries, tlut_name; } HSD_Tlut;
typedef HSD_Tlut HSD_TlutDesc;
typedef struct { HSD_ImageDesc* imagedesc; HSD_ImageDesc** imagetbl; HSD_Tlut* tlut; HSD_Tlut** tluttbl; int tlut_no; HSD_AObj* aobj; } HSD_TObj;
typedef struct { int loaded; } HSD_Archive;
typedef struct { HSD_TObj* tobj; } HSD_MObj;
typedef struct DObj { struct DObj* next; HSD_MObj* mobj; } HSD_DObj;
typedef struct { struct { HSD_DObj* dobj; } u; unsigned flags; } HSD_JObj;
typedef struct { HSD_ImageDesc* img; HSD_Tlut* tl; } FsArt;
enum { JOBJ_HIDDEN = 1, GX_TF_I4 = 0, FA_RETAIL_FIGHTERS = 33, TOBJ_TLUT_NONE = -1 };
static unsigned char disc_pixels[32], own_pixels[32];
static HSD_ImageDesc disc = { disc_pixels, 4, 4, 5, 0, 0, 0 };
static HSD_ImageDesc icon = { own_pixels, 4, 4, 5, 0, 0, 0 };
static struct { int n; int ck[4]; FsArt icon[4]; } fcs;
static int own_tex[5], own_handle[5], donor = 0, defined = 1;
static int last_portrait_ck, last_portrait_cos, calls_open, closed, alloc_fail;
static int malformed, disc_missing, max_image = 300;
static HSD_Archive archive;
static HSD_Tlut palette;
static void* allocations[128]; static int nalloc;
static void* HSD_MemAlloc(int size)
{
    void* p;
    if (alloc_fail) return NULL;
    assert(nalloc < 128);
    p = calloc(1, (size_t)size); assert(p);
    allocations[nalloc++] = p; return p;
}
static void HSD_Free(void* p) { (void)p; } /* freed after each case */
static void memzero(void* p, size_t n) { memset(p, 0, n); }
static void OSReport(const char* fmt, ...) { (void)fmt; }
static void HSD_JObjClearFlagsAll(HSD_JObj* j, unsigned f) { j->flags &= ~f; }
static void HSD_JObjSetFlagsAll(HSD_JObj* j, unsigned f) { j->flags |= f; }
static int Geno_DefineBaseCK(int ck) { return ck == 127 && defined ? 0 : -1; }
static int Geno_DefineArtBaseCK(int ck) { return ck == 127 && defined ? donor : -1; }
static int Geno_DefineName(int ck, char* out, int cap)
{ if (Geno_DefineBaseCK(ck) < 0) return 0; snprintf(out, (size_t)cap, "Geno Falcon"); return 1; }
static int Ui_ArtGeno(int ck, int what, int costume)
{ assert(ck >= 0 && what >= 0 && what < 5); (void)costume; return own_tex[what]; }
static void fas_geno_art_said(int ck, int what, int tex) { (void)ck; (void)what; (void)tex; }
static int fas_art(const char* key, FsArt a)
{ (void)key; return a.img && a.img->image_ptr ? (a.img == &icon ? 91 : 81) : -1; }
static int mnCharSel_PcArtPortrait(int ck, int costume, HSD_ImageDesc** img, HSD_Tlut** tl)
{ last_portrait_ck = ck; last_portrait_cos = costume; *img = !disc_missing && (ck == 0 || ck == 8) ? &disc : NULL; *tl = disc.format == 9 ? &palette : NULL; return *img != NULL; }
static HSD_Archive* lbArchive_LoadArchive(const char* name) { (void)name; return &archive; }
static void lbArchive_80016EFC(HSD_Archive* a) { assert(a == &archive); }
static int lbLang_IsSavedLanguageUS(void) { return 1; }
static int mnCharSel_PcArtOpen(HSD_Archive* a) { assert(a == &archive); return 25; }
static void mnCharSel_PcArtClose(void) {}
static float last_stock_frame;
static void HSD_TObjReqAnimAll(HSD_TObj* t, float frame)
{ last_stock_frame = frame; if (t->imagetbl) t->imagedesc = t->imagetbl[max_image]; }
static HSD_Tlut* HSD_TlutLoadDesc(HSD_TlutDesc* d)
{ HSD_Tlut* t = HSD_MemAlloc(sizeof *t); if(t) *t=*d; return t; }
static void HSD_TlutRemove(HSD_Tlut* t) { (void)t; }
static f32 gm_80168B34(int ck, int fk, int costume)
{ (void)fk; assert(ck == 0 || ck == 8); return (float)(ck + costume * 30); }
static void inline0(HSD_JObj* j, float frame)
{
    HSD_DObj* d;
    assert(frame == 0 || frame == 8);
    for (d = j->u.dobj; d; d = d->next) {
        HSD_TObj* t = d->mobj->tobj;
        /* A static results image never changes just because the card animates. */
        if (t->imagetbl != NULL) t->imagedesc = t->imagetbl[max_image];
    }
}
static int Geno_DefineArtOpen(int ck, int what, int costume)
{ assert((ck == 127 || ck == 8) && what >= 0 && what < 5); (void)costume; ++calls_open; return own_handle[what]; }
static int GxTex_Width(int h) { assert(h == 7); return 4; }
static int GxTex_Height(int h) { assert(h == 7); return 4; }
static int GxTex_Format(int h) { assert(h == 7); return malformed ? 9 : 5; }
static int GxTex_ImageSize(int h) { assert(h == 7); return 32; }
static void GxTex_CopyImage(int h, void* dst) { assert(h == 7); memcpy(dst, own_pixels, 32); }
static void GxTex_Close(int h) { assert(h == 7); ++closed; }
static int GxTex_TlutSize(int h) { assert(h == 7); return malformed ? 32 : 0; }
static int GxTex_TlutFormat(int h) { assert(h == 7); return malformed ? 2 : -1; }
static int GxTex_TlutEntries(int h) { assert(h == 7); return malformed ? 16 : 0; }
static void GxTex_CopyTlut(int h, void* dst) { assert(h == 7); memcpy(dst, own_pixels, 32); }
static int UI_TextI4(const char* text, int w, int h, int style, void* dst)
{ assert(!strcmp(text, "GF")); (void)style; memset(dst, 0xff, (size_t)(((w+7)/8)*((h+7)/8)*32)); return 1; }

#include "../geno/geno_art_text.h"
#include "../geno/geno_art_anim.inc"
#include "../../src/melee/gm/gmfrontend_package_art.inc"
#include "../../src/melee/gm/gmfrontend_fighter_art.inc"
#include "../../src/melee/gm/gmresultplayer_geno_art.inc"
#include "../../src/melee/if/ifstock_geno_art.inc"
#undef HSD_TObjReqAnimAll

static void reset(void)
{
    int i;
    ifStock_GenoReset();
    fs_geno_clear();
    gmRst_GenoArtClose();
    for (i=0; i<nalloc; ++i) free(allocations[i]);
    nalloc = alloc_fail = malformed = calls_open = closed = disc_missing = 0;
    disc.format = 5; max_image = 300;
    palette.lut = disc_pixels; palette.n_entries = 16;
    memset(disc_pixels, 0x83, sizeof disc_pixels); /* invented RGB5A3 */
    memset(own_pixels, 0xfc, sizeof own_pixels);
    for (i=0; i<5; ++i) own_tex[i] = own_handle[i] = -1;
    donor = 0; defined = 1; last_portrait_ck = last_portrait_cos = -1;
    fcs.n=2; fcs.ck[0]=0; fcs.ck[1]=127;
    fcs.icon[0].img = fcs.icon[1].img = &disc;
    fcs.icon[0].tl = fcs.icon[1].tl = NULL;
}
static void select_art(void)
{
    reset();
    own_tex[0]=91; own_tex[1]=92;
    assert(fas_fighter_icon(127) == 91);
    assert(fas_fighter_portrait(127, 3) == 92 && last_portrait_ck == -1);
    own_tex[0] = own_tex[1] = -1;            /* absent or decoder-refused */
    assert(fas_fighter_icon(127) == 81);      /* template has own costumes */
    assert(fas_fighter_portrait(127, 3) == 81);
    assert(last_portrait_ck == 0 && last_portrait_cos == 3);
    donor = -1; fcs.icon[1].img = NULL;
    assert(fas_fighter_icon(127) == -1 && fas_fighter_portrait(127, 0) == -1);
    own_tex[0]=91; own_tex[1]=92;
    assert(fas_fighter_icon(127) == 91 && fas_fighter_portrait(127, 0) == 92);
    own_tex[0]=own_tex[1]=-1;
    assert(fas_fighter_icon(0) == 81 && fas_fighter_portrait(0, 0) == 81);
    assert(fas_fighter_icon(-1) == -1 && fas_fighter_portrait(-1, 0) == -1);
}

static void legacy_art(void)
{
    FsArt empty = {NULL, NULL}, got;
    reset();
    fs_geno_clear();
    own_handle[0] = own_handle[1] = 7;
    got = fs_fighter_icon(127, fcs.icon[1]);
    assert(got.img != &disc && !memcmp(got.img->image_ptr, own_pixels, 32));
    assert(fs_fighter_icon(127, empty).img == got.img); /* scene cache */
    got = fs_fighter_portrait(127, 3);
    assert(got.img && got.img != &disc && last_portrait_ck == -1);
    fs_geno_clear(); own_handle[0] = own_handle[1] = -1;
    assert(fs_fighter_icon(127, fcs.icon[1]).img == &disc);
    assert(fs_fighter_portrait(127, 3).img == &disc && last_portrait_ck == 0);
    donor = -1; fs_geno_clear();
    assert(fs_fighter_icon(127, empty).img == NULL);
    assert(fs_fighter_portrait(127, 0).img == NULL);
    own_handle[1] = 7; malformed = 1; fs_geno_clear();
    /* A CI image retains its copied palette in the legacy renderer. */
    got = fs_fighter_portrait(127, 0);
    assert(got.img && got.img->format == 9 && got.tl && got.tl->lut);
    fs_geno_clear(); malformed = 0;
    alloc_fail = 1;
    assert(fs_fighter_portrait(127, 2).img == NULL);
    alloc_fail = 0; fs_geno_clear();
}

static void results_card(void)
{
    HSD_TObj emblem = {0}, picture = {0};
    HSD_FObj track = {NULL, image_keys, sizeof image_keys, HSD_A_T_TIMG, 0x40, 0, 0};
    HSD_AObj animation = {&track};
    picture.aobj = &animation;
    HSD_MObj em = {&emblem}, pm = {&picture};
    HSD_DObj pd = {NULL,&pm}, ed = {&pd,&em};
    HSD_JObj card = {{&ed}, JOBJ_HIDDEN};
    reset();
    picture.imagedesc = &icon; /* unrelated STATIC card texture; not a donor */
    assert(gmRst_GenoCard(&card,127,0));
    assert(!(card.flags & JOBJ_HIDDEN));
    assert(picture.imagedesc == &disc && picture.imagedesc->image_ptr);
    assert(!memcmp(picture.imagedesc->image_ptr, disc_pixels, 32));
    assert(picture.imagetbl[300] == &disc && picture.imagetbl[301] == NULL);
    assert(last_portrait_ck == 0);
    own_handle[4]=7;
    assert(gmRst_GenoCard(&card,127,2));
    assert(picture.imagedesc != &disc && !memcmp(picture.imagedesc->image_ptr,own_pixels,32));
    assert(closed == 1 && !(card.flags & JOBJ_HIDDEN));
    own_handle[4]=-1;
    assert(gmRst_GenoCard(&card,127,2) && picture.imagedesc == &disc);
    own_handle[4]=7; malformed=1;
    assert(gmRst_GenoCard(&card,127,2) && picture.imagedesc == &disc);
    malformed=0; own_handle[4]=-1;
    disc.format=9;
    assert(gmRst_GenoCard(&card,127,2) && picture.imagedesc == &disc);
    assert(picture.tlut != NULL && picture.tlut != &palette && picture.tlut_no == TOBJ_TLUT_NONE);
    disc.format=5;
    donor=-1;
    assert(gmRst_GenoCard(&card,127,0));
    assert(!(card.flags & JOBJ_HIDDEN) && picture.imagedesc->format == GX_TF_I4);
    assert(((u8*)picture.imagedesc->image_ptr)[0] == 0xff);
    donor=0; disc_missing=1;
    assert(gmRst_GenoCard(&card,127,0) && picture.imagedesc->format == GX_TF_I4);
    defined=0; calls_open=0;
    assert(!gmRst_GenoCard(&card,127,0) && calls_open == 0);
    assert(!gmRst_GenoCard(&card,0,0) && calls_open == 0);
    reset();
}

static void stock_art(void)
{
    HSD_ImageDesc* atlas[302] = {NULL};
    HSD_FObj track = {NULL, image_keys, sizeof image_keys, HSD_A_T_TIMG, 0x40, 0, 0};
    HSD_AObj animation = {&track};
    HSD_TObj stock = {0};
    reset();
    for (int i=0; i<301; ++i) atlas[i] = &disc;
    stock.imagedesc = &disc; stock.imagetbl = atlas; stock.aobj = &animation;
    own_handle[2] = 7;
    ifStock_ReqAnim(&stock, 1000000.0F + 127*256 + 2);
    assert(stock.imagedesc != &disc && !memcmp(stock.imagedesc->image_ptr, own_pixels, 32));
    assert(stock.imagetbl[300] == stock.imagedesc && stock.imagetbl[301] == NULL);
    /* A TObj reused by a different fighter must recover its retail atlas. */
    ifStock_ReqAnim(&stock, 0.0F);
    assert(stock.imagetbl == atlas && stock.imagedesc == &disc);
    own_handle[2] = -1;
    ifStock_ReqAnim(&stock, 1000000.0F + 127*256 + 3);
    assert(stock.imagetbl == atlas && stock.imagedesc == &disc);
    assert(last_stock_frame == 90.0F); /* Captain CK0, costume3 */
    ifStock_ReqAnim(&stock, 1000000.0F + 8*256 + 5);
    assert(stock.imagedesc == &disc && last_stock_frame == 158.0F); /* retail skin failure stays Mario */
    donor = -1;
    ifStock_ReqAnim(&stock, 1000000.0F + 127*256 + 4);
    assert(stock.imagedesc && stock.imagedesc->format == GX_TF_I4);
    assert(((u8*)stock.imagedesc->image_ptr)[0] == 0xff);
    reset();
}

int main(void)
{
    char abbr[3];
    genoArt_Initials(" Geno Falcon", abbr); assert(!strcmp(abbr,"GF"));
    genoArt_Initials("  Falcon", abbr); assert(!strcmp(abbr,"FA"));
    select_art(); legacy_art(); results_card(); stock_art();
    puts("geno-css-art: own art, retail fallback, letters, visible results card PASS");
    return 0;
}
