/* Geno art on the results card. Included only in the TARGET_PC section of
 * gmresultplayer.c; the native regression includes the same code with synthetic
 * HSD objects. fn_80177748 must handle a define before the m-ex CK range test.
 * what: 1 CSP (legacy), 2 stock, 3 emblem, 4 results override with CSP fallback. */
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

static HSD_TObj* gmRst_GenoTObjAt(HSD_JObj* jobj, int index)
{
    HSD_DObj* dobj;
    if (jobj == NULL || (dobj = jobj->u.dobj) == NULL) return NULL;
    for (; index > 0 && dobj != NULL; index--) dobj = dobj->next;
    return dobj != NULL && dobj->mobj != NULL ? dobj->mobj->tobj : NULL;
}

/* As ifStock_ReqAnim: every texture-image key selects our scene-owned image.
 * Keep the original TObj untouched on any allocation/decode failure. */
static bool gmRst_GenoSetImage(HSD_TObj* tobj, HSD_ImageDesc* desc)
{
    HSD_ImageDesc** tbl;
    int i, max_img;
    if (tobj == NULL || desc == NULL) return false;
    genoArt_ImageKeyAt(tobj, -1, &max_img);
    if (max_img < 0) max_img = 0;
    if (max_img > 65535) return false;
    tbl = HSD_MemAlloc((max_img + 2) * sizeof(HSD_ImageDesc*));
    if (tbl == NULL) return false;
    for (i = 0; i <= max_img; ++i) tbl[i] = desc;
    tbl[max_img + 1] = NULL;
    tobj->imagetbl = tbl;
    tobj->imagedesc = desc;
    return true;
}

static bool gmRst_GenoApplyArt(HSD_TObj* tobj, int ck, int what, int costume)
{
    HSD_ImageDesc* desc;
    u8* allocation;
    u8* pix;
    int h, w, ht, size, fmt;
    if (tobj == NULL || (h = Geno_DefineArtOpen(ck, what, costume)) < 0) return false;
    w = GxTex_Width(h);
    ht = GxTex_Height(h);
    size = GxTex_ImageSize(h);
    fmt = GxTex_Format(h);
    if (fmt >= 8 || w < 1 || ht < 1 || size < 1) {
        OSReport("geno: results art of ck %d (kind %d): unusable format %d\n", ck, what, fmt);
        GxTex_Close(h);
        return false;
    }
    allocation = HSD_MemAlloc(size + 32);
    desc = HSD_MemAlloc(sizeof(HSD_ImageDesc));
    if (allocation == NULL || desc == NULL) {
        if (allocation != NULL) HSD_Free(allocation);
        if (desc != NULL) HSD_Free(desc);
        GxTex_Close(h);
        return false;
    }
    pix = (u8*) (((uintptr_t) allocation + 31) & ~(uintptr_t) 31);
    GxTex_CopyImage(h, pix);
    GxTex_Close(h);
    desc->image_ptr = pix;
    desc->width = (u16) w;
    desc->height = (u16) ht;
    desc->format = (GXTexFmt) fmt;
    desc->mipmap = 0;
    desc->minLOD = desc->maxLOD = 0.0F;
    if (!gmRst_GenoSetImage(tobj, desc)) {
        HSD_Free(allocation);
        HSD_Free(desc);
        return false;
    }
    OSReport("geno: results %s of ck %d: %dx%d format %d from its package\n",
             what == 2 ? "stock icon" : what == 3 ? "emblem" : "card picture", ck, w, ht, fmt);
    return true;
}

/* No visual donor: draw the define's initials with the existing I4 name-strip
 * renderer. Only authored text and a generated bitmap, never another face. */
static bool gmRst_GenoLetters(HSD_TObj* tobj, int ck)
{
    char name[48], abbr[3];
    HSD_ImageDesc* desc;
    HSD_ImageDesc* cur;
    u8* allocation;
    u8* pix;
    int w, h, bytes;
    if (tobj == NULL || (cur = tobj->imagedesc) == NULL ||
        !Geno_DefineName(ck, name, sizeof name)) return false;
    genoArt_Initials(name, abbr);
    w = cur->width; h = cur->height;
    if (w < 1 || h < 1 || w > 1024 || h > 1024) return false;
    bytes = ((w + 7) / 8) * ((h + 7) / 8) * 32;
    allocation = HSD_MemAlloc(bytes + 32);
    desc = HSD_MemAlloc(sizeof(HSD_ImageDesc));
    if (allocation == NULL || desc == NULL) {
        if (allocation != NULL) HSD_Free(allocation);
        if (desc != NULL) HSD_Free(desc);
        return false;
    }
    pix = (u8*) (((uintptr_t) allocation + 31) & ~(uintptr_t) 31);
    memset(pix, 0, bytes);
    UI_TextI4(abbr, w, h, 0, pix);
    desc->image_ptr = pix;
    desc->width = (u16) w;
    desc->height = (u16) h;
    desc->format = GX_TF_I4;
    desc->mipmap = 0;
    desc->minLOD = desc->maxLOD = 0.0F;
    if (!gmRst_GenoSetImage(tobj, desc)) {
        HSD_Free(allocation); HSD_Free(desc);
        return false;
    }
    OSReport("geno: results letters of ck %d: %s\n", ck, abbr);
    return true;
}

/* Own the disc archive for the whole results scene. The card picture is STATIC:
 * inline0 selects its emblem, not its fighter. Reuse the CSS reader to obtain
 * the requested donor's actual image (including a palette on an m-ex disc). */
static HSD_Archive* gmRst_GenoChrArt;
static bool gmRst_GenoChrTried;
extern int mnCharSel_PcArtOpen(HSD_Archive* archive);
extern void mnCharSel_PcArtClose(void);
extern int mnCharSel_PcArtPortrait(int ck, int costume, HSD_ImageDesc** img, HSD_Tlut** tlut);

void gmRst_GenoArtReset(void)
{
    gmRst_GenoChrArt = NULL;
    gmRst_GenoChrTried = false;
}
void gmRst_GenoArtClose(void)
{
    if (gmRst_GenoChrArt != NULL) {
        mnCharSel_PcArtClose();
        lbArchive_80016EFC(gmRst_GenoChrArt);
    }
    gmRst_GenoArtReset();
}

static bool gmRst_GenoRetailPortrait(HSD_TObj* tobj, int base, int costume)
{
    HSD_ImageDesc* image = NULL;
    HSD_Tlut* palette = NULL;
    HSD_Tlut* copy = NULL;
    HSD_TlutDesc desc;
    int i;
    if (tobj == NULL || base < 0) return false;
    if (!gmRst_GenoChrTried) {
        gmRst_GenoChrTried = true;
        gmRst_GenoChrArt = lbArchive_LoadArchive(lbLang_IsSavedLanguageUS() ? "MnSlChr.usd" : "MnSlChr.dat");
        mnCharSel_PcArtOpen(gmRst_GenoChrArt);
    }
    if (gmRst_GenoChrArt == NULL ||
        !mnCharSel_PcArtPortrait(base, costume, &image, &palette) ||
        image == NULL || image->image_ptr == NULL) return false;
    if (image->format == 8 || image->format == 9 || image->format == 10) {
        if (palette == NULL || palette->lut == NULL) return false;
        desc.lut = palette->lut;
        desc.fmt = palette->fmt;
        desc.tlut_name = palette->tlut_name;
        desc.n_entries = palette->n_entries;
        copy = HSD_TlutLoadDesc(&desc);
        if (copy == NULL) return false;
    }
    if (!gmRst_GenoSetImage(tobj, image)) {
        HSD_TlutRemove(copy);
        return false;
    }
    /* TObjRelease owns its TLUT objects: never borrow the CSS TObj's object.
     * The palette bytes are borrowed from our scene-owned archive. */
    HSD_TlutRemove(tobj->tlut);
    if (tobj->tluttbl != NULL) {
        for (i = 0; tobj->tluttbl[i] != NULL; ++i) HSD_TlutRemove(tobj->tluttbl[i]);
        HSD_Free(tobj->tluttbl);
    }
    tobj->tlut = copy;
    tobj->tluttbl = NULL;
    tobj->tlut_no = TOBJ_TLUT_NONE;
    return true;
}

/* Returns true for a Geno define, even if it has no loadable package art.
 * Retail/m-ex names and identity are deliberately kept out of this decision. */
static bool gmRst_GenoCard(HSD_JObj* card, int ck, int costume)
{
    HSD_TObj* picture;
    int base;
    bool own;
    if (Geno_DefineBaseCK(ck) < 0) return false;
    base = Geno_DefineArtBaseCK(ck);
    HSD_JObjClearFlagsAll(card, JOBJ_HIDDEN);
    inline0(card, base >= 0 ? gm_80168B34(base, 0, 0) : 0.0F);
    picture = gmRst_GenoTObjAt(card, 1);
    own = gmRst_GenoApplyArt(picture, ck, 4, costume);
    if (!own) {
        if (gmRst_GenoRetailPortrait(picture, base, costume)) {
            OSReport("geno: results card of ck %d: retail ck %d\n", ck, base);
        } else if (!gmRst_GenoLetters(picture, ck)) {
            HSD_JObjSetFlagsAll(card, JOBJ_HIDDEN);
        }
    }
    gmRst_GenoApplyArt(gmRst_GenoTObjAt(card, 0), ck, 3, 0);
    return true;
}
