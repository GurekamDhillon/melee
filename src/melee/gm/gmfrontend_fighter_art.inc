/* Included by gmfrontend_atlas_select.inc after fas_art/fas_geno_art_said.
 * Keep these choices shared by grid cells and port-card faces: an own costume
 * list does not imply that an explicit retail template has no disc artwork. */
extern int Geno_DefineArtBaseCK(int ck);

static void fas_fighter_fallback_said(int ck, int what, int tex)
{
    static unsigned char said[128][2];
    int base;
    if (ck < FA_RETAIL_FIGHTERS || ck >= 128 || what < 0 || what > 1 ||
        Geno_DefineBaseCK(ck) < 0 || said[ck][what]) {
        return;
    }
    said[ck][what] = 1;
    base = Geno_DefineArtBaseCK(ck);
    OSReport("frontend: Geno art ck %d %s: %s (retail ck %d, texture %d)\n",
             ck, what ? "portrait" : "icon", tex >= 0 ? "retail" : "letters", base, tex);
}

static int fas_fighter_icon(int ck)
{
    char id[16];
    int slot, tex;
    if (ck < 0) return -1;
    tex = Ui_ArtGeno(ck, 0, 0);
    if (tex >= 0) {
        fas_geno_art_said(ck, 0, tex);
        return tex;
    }
    sprintf(id, "f%d", ck);
    for (slot = 0; slot < fcs.n; ++slot) {
        if (fcs.ck[slot] == ck) {
            tex = fas_art(id, fcs.icon[slot]);
            break;
        }
    }
    fas_fighter_fallback_said(ck, 0, tex);
    return tex;
}

static int fas_fighter_portrait(int ck, int costume)
{
    char id[32];
    FsArt a;
    int tex, base;
    if (ck < 0) return -1;
    tex = Ui_ArtGeno(ck, 1, costume);
    if (tex >= 0) {
        fas_geno_art_said(ck, 1, tex);
        return tex;
    }
    base = Geno_DefineBaseCK(ck) >= 0 ? Geno_DefineArtBaseCK(ck) : ck;
    if (base >= 0) {
        a.img = NULL;
        a.tl = NULL;
        mnCharSel_PcArtPortrait(base, costume, &a.img, &a.tl);
        sprintf(id, "p%d:%d", ck, costume);
        tex = fas_art(id, a);
    }
    fas_fighter_fallback_said(ck, 1, tex);
    return tex;
}
