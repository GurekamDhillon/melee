/* Legacy GX frontend adapter. Package pixels cross the shim as copied bytes;
 * descriptors/palettes live in game memory. Released before the scene archive.
 * A bounded scene cache avoids reopening files on every cursor movement. */
extern int Geno_DefineArtOpen(int ck, int what, int costume);
extern int GxTex_Width(int h);
extern int GxTex_Height(int h);
extern int GxTex_Format(int h);
extern int GxTex_ImageSize(int h);
extern int GxTex_TlutSize(int h);
extern int GxTex_TlutFormat(int h);
extern int GxTex_TlutEntries(int h);
extern void GxTex_CopyImage(int h, void* dst);
extern void GxTex_CopyTlut(int h, void* dst);
extern void GxTex_Close(int h);

#define FS_GENO_CACHE 256
#define FS_GENO_BYTES (16 * 1024 * 1024)
static struct {
    int ck, what, costume;
    void* pixels;
    void* palette;
    HSD_ImageDesc image;
    HSD_Tlut tlut;
    FsArt art;
} fs_geno[FS_GENO_CACHE];
static int fs_geno_n, fs_geno_bytes;

static void fs_geno_clear(void)
{
    int i;
    for (i = 0; i < fs_geno_n; ++i) {
        if (fs_geno[i].pixels != NULL) HSD_Free(fs_geno[i].pixels);
        if (fs_geno[i].palette != NULL) HSD_Free(fs_geno[i].palette);
    }
    memzero(fs_geno, sizeof fs_geno);
    fs_geno_n = fs_geno_bytes = 0;
}

static FsArt fs_geno_art(int ck, int what, int costume)
{
    FsArt none = { NULL, NULL };
    int i, h, w, ht, fmt, size, pal, entries;
    if (ck < 0) return none;
    if (what == 0) costume = 0;
    for (i = 0; i < fs_geno_n; ++i)
        if (fs_geno[i].ck == ck && fs_geno[i].what == what &&
            fs_geno[i].costume == costume) return fs_geno[i].art;
    if (fs_geno_n == FS_GENO_CACHE) return none;
    i = fs_geno_n++;
    fs_geno[i].ck = ck;
    fs_geno[i].what = what;
    fs_geno[i].costume = costume;
    h = Geno_DefineArtOpen(ck, what, costume);
    if (h < 0) return none;
    w = GxTex_Width(h); ht = GxTex_Height(h); fmt = GxTex_Format(h);
    size = GxTex_ImageSize(h); pal = GxTex_TlutSize(h);
    entries = GxTex_TlutEntries(h);
    if (w < 1 || w > 1024 || ht < 1 || ht > 1024 || size < 1 ||
        size > FS_GENO_BYTES - fs_geno_bytes - 64 || pal < 0 ||
        pal > FS_GENO_BYTES - fs_geno_bytes - size - 64 ||
        ((fmt == 8 || fmt == 9 || fmt == 10) && (pal < 2 || entries < 1))) {
        GxTex_Close(h);
        OSReport("geno: legacy select art of ck %d kind %d unavailable\n", ck, what);
        return none;
    }
    fs_geno[i].pixels = HSD_MemAlloc(size + 32);
    if (pal) fs_geno[i].palette = HSD_MemAlloc(pal + 32);
    if (fs_geno[i].pixels == NULL || (pal && fs_geno[i].palette == NULL)) {
        if (fs_geno[i].pixels) HSD_Free(fs_geno[i].pixels);
        if (fs_geno[i].palette) HSD_Free(fs_geno[i].palette);
        fs_geno[i].pixels = fs_geno[i].palette = NULL;
        GxTex_Close(h);
        return none;
    }
    fs_geno[i].image.image_ptr = (void*) (((uintptr_t) fs_geno[i].pixels + 31) & ~(uintptr_t) 31);
    fs_geno[i].image.width = (u16) w;
    fs_geno[i].image.height = (u16) ht;
    fs_geno[i].image.format = (GXTexFmt) fmt;
    GxTex_CopyImage(h, fs_geno[i].image.image_ptr);
    fs_geno[i].art.img = &fs_geno[i].image;
    if (pal) {
        fs_geno[i].tlut.lut = (void*) (((uintptr_t) fs_geno[i].palette + 31) & ~(uintptr_t) 31);
        fs_geno[i].tlut.fmt = GxTex_TlutFormat(h);
        fs_geno[i].tlut.n_entries = entries;
        GxTex_CopyTlut(h, fs_geno[i].tlut.lut);
        fs_geno[i].art.tl = &fs_geno[i].tlut;
    }
    GxTex_Close(h);
    fs_geno_bytes += size + pal + 64;
    OSReport("geno: legacy select art of ck %d kind %d costume %d: package %dx%d\n", ck, what, costume, w, ht);
    return fs_geno[i].art;
}

static FsArt fs_fighter_icon(int ck, FsArt retail)
{
    FsArt own = fs_geno_art(ck, 0, 0);
    return own.img != NULL ? own : retail;
}

static FsArt fs_fighter_portrait(int ck, int costume)
{
    FsArt own = fs_geno_art(ck, 1, costume);
    int base = Geno_DefineBaseCK(ck) >= 0 ? Geno_DefineArtBaseCK(ck) : ck;
    if (own.img == NULL && base >= 0)
        mnCharSel_PcArtPortrait(base, costume, &own.img, &own.tl);
    return own;
}
