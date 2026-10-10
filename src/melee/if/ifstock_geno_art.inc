/* Package stock icons, then the explicit retail art donor, then initials.
 * gm_GenoStockFrame uses a sentinel so UI decisions never enter fighter state.
 * Descriptors and animation tables belong to the HUD scene heap. */
#define IFSTOCK_GENO_FRAME 1000000.0F
#define IFSTOCK_GENO_CACHE 12
#define IFSTOCK_GENO_BINDINGS 256
extern int Geno_DefineBaseCK(int ck);
extern int Geno_DefineArtBaseCK(int ck);
extern int Geno_DefineName(int ck, char* out, int cap);
extern int Geno_DefineArtOpen(int ck, int what, int costume);
extern int GxTex_Width(int h);
extern int GxTex_Height(int h);
extern int GxTex_Format(int h);
extern int GxTex_ImageSize(int h);
extern void GxTex_CopyImage(int h, void* dst);
extern void GxTex_Close(int h);
extern int UI_TextI4(const char* text, int w, int h, int style, void* dst);

static struct IfStockGenoArt {
    int ck, costume, tried;
    HSD_ImageDesc* desc;
} ifStock_GenoArt[IFSTOCK_GENO_CACHE];
static struct IfStockGenoBinding {
    HSD_TObj* tobj;
    HSD_ImageDesc* original_image;
    HSD_ImageDesc** original;
    HSD_ImageDesc** table;
    int count;
} ifStock_GenoBindings[IFSTOCK_GENO_BINDINGS];

static void ifStock_GenoReset(void)
{
    memset(ifStock_GenoArt, 0, sizeof ifStock_GenoArt);
    memset(ifStock_GenoBindings, 0, sizeof ifStock_GenoBindings);
}

static struct IfStockGenoArt* ifStock_GenoFind(int ck, int costume, HSD_ImageDesc* current)
{
    int i, handle, w = 0, ht = 0, size = 0, fmt = -1;
    struct IfStockGenoArt* a = NULL;
    u8* allocation;
    u8* pix;
    char name[48], abbr[3];
    for (i = 0; i < IFSTOCK_GENO_CACHE; ++i) {
        if (ifStock_GenoArt[i].tried && ifStock_GenoArt[i].ck == ck &&
            ifStock_GenoArt[i].costume == costume) return &ifStock_GenoArt[i];
        if (a == NULL && !ifStock_GenoArt[i].tried) a = &ifStock_GenoArt[i];
    }
    if (a == NULL) return NULL;
    a->tried = 1; a->ck = ck; a->costume = costume;
    handle = Geno_DefineArtOpen(ck, 2, costume);
    if (handle >= 0) {
        w = GxTex_Width(handle); ht = GxTex_Height(handle);
        size = GxTex_ImageSize(handle); fmt = GxTex_Format(handle);
        if (fmt < 0 || fmt >= 8 || w < 1 || ht < 1 || w > 1024 || ht > 1024 || size < 1) {
            GxTex_Close(handle); handle = -1;
        }
    }
    if (handle < 0) {
        if (Geno_DefineArtBaseCK(ck) >= 0 || current == NULL ||
            !Geno_DefineName(ck, name, sizeof name)) return a;
        w = current->width; ht = current->height; fmt = GX_TF_I4;
        if (w < 1 || ht < 1 || w > 1024 || ht > 1024) return a;
        size = ((w + 7) / 8) * ((ht + 7) / 8) * 32;
        genoArt_Initials(name, abbr);
    }
    allocation = HSD_MemAlloc(size + 32);
    a->desc = HSD_MemAlloc(sizeof(HSD_ImageDesc));
    if (allocation == NULL || a->desc == NULL) {
        if (allocation != NULL) HSD_Free(allocation);
        if (a->desc != NULL) HSD_Free(a->desc);
        a->desc = NULL;
        if (handle >= 0) GxTex_Close(handle);
        return a;
    }
    pix = (u8*) (((uintptr_t) allocation + 31) & ~(uintptr_t) 31);
    if (handle >= 0) {
        GxTex_CopyImage(handle, pix);
        GxTex_Close(handle);
    } else {
        memset(pix, 0, size);
        UI_TextI4(abbr, w, ht, 0, pix);
    }
    a->desc->image_ptr = pix;
    a->desc->width = (u16) w; a->desc->height = (u16) ht;
    a->desc->format = (GXTexFmt) fmt;
    a->desc->mipmap = 0; a->desc->minLOD = a->desc->maxLOD = 0.0F;
    OSReport("geno: stock of ck %d costume %d: %s %dx%d\n",
             ck, costume, handle >= 0 ? "package" : "letters", w, ht);
    return a;
}

/* Stock JObjs are reused (e.g. stock sharing). Restore the original atlas before
 * choosing a new fighter, and size substitutions from TIMG keys, not a fixed
 * 128-entry table which m-ex animations can index beyond. */
static struct IfStockGenoBinding* ifStock_GenoBinding(HSD_TObj* tobj, int create)
{
    int i, max_img;
    struct IfStockGenoBinding* b = NULL;
    for (i = 0; i < IFSTOCK_GENO_BINDINGS; ++i) {
        if (ifStock_GenoBindings[i].tobj == tobj) {
            b = &ifStock_GenoBindings[i]; break;
        }
        if (b == NULL && ifStock_GenoBindings[i].tobj == NULL)
            b = &ifStock_GenoBindings[i];
    }
    if (b == NULL || (!create && b->tobj == NULL)) return NULL;
    if (b->tobj == NULL) {
        b->tobj = tobj;
        b->original = tobj->imagetbl;
        b->original_image = tobj->imagedesc;
        genoArt_ImageKeyAt(tobj, -1, &max_img);
        if (max_img < 0) max_img = 0;
        if (max_img > 65535) return b;
        b->count = max_img + 1;
        b->table = HSD_MemAlloc((b->count + 1) * sizeof(HSD_ImageDesc*));
    }
    tobj->imagetbl = b->original;
    tobj->imagedesc = b->original_image;
    return b;
}

static void ifStock_ReqAnim(HSD_TObj* tobj, f32 frame)
{
    struct IfStockGenoBinding* b;
    if (tobj == NULL) return;
    b = ifStock_GenoBinding(tobj, frame >= IFSTOCK_GENO_FRAME);
    if (frame >= IFSTOCK_GENO_FRAME) {
        int i, v = (int) (frame - IFSTOCK_GENO_FRAME), ck = v / 256, costume = v % 256;
        int base = Geno_DefineBaseCK(ck) >= 0 ? Geno_DefineArtBaseCK(ck) : ck;
        struct IfStockGenoArt* a = ifStock_GenoFind(ck, costume, tobj->imagedesc);
        if (a != NULL && a->desc != NULL && b != NULL && b->table != NULL) {
            for (i = 0; i < b->count; ++i) b->table[i] = a->desc;
            b->table[b->count] = NULL;
            tobj->imagetbl = b->table;
            tobj->imagedesc = a->desc;
            frame = 0.0F;
        } else {
            frame = base >= 0 ? gm_80168B34((CharacterKind) base, 0, costume) : 0.0F;
        }
    }
    (HSD_TObjReqAnimAll)(tobj, frame);
}
#define HSD_TObjReqAnimAll(t, f) ifStock_ReqAnim((t), (f))
