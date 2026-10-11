/* gw_nucleus_fighter.h - installing a MexManager FIGHTER zip (a Nucleus custom-fighter post) as a mod that adds a NEW fighter on a vanilla disc.
 * Pure C, header-only (static), after gw_nucleus_loose.h and gw_nucleus_stage.h. No game types.
 *
 * The zip: `fighter.json` + the fighter file `PlXx.dat` + its animation file `PlXxAJ.dat` (+ the effect bank, `sound.zip`, a zip per costume). The installer
 * checks the fighter file's CONTENT (an HSD archive with the json's data symbol), gives every file a PRIVATE name (the json's own names are usually retail
 * names: PlGk.dat is retail Giga Bowser, and writing it would REPLACE a retail fighter), and writes
 *
 *   mods/nucleus-<post>-fighter/mod.json         kind "fighter", source "nucleus", the post link, the verdict note the browser shows
 *   mods/nucleus-<post>-fighter/mexfighter.txt   what the runtime reads (pc/platform/gw_mex_modfighter.inc)
 *   mods/nucleus-<post>-fighter/files/PlNxxxx*.dat   the fighter, animation and costume files under their private names
 *
 * THE VERDICT (the three words fighter_compat.py also prints), computed here from the json at install time:
 *   works       every callback is 0, a small constant, or an address in the retail dol (a clone of a retail fighter); the files are all in the zip
 *   partial     it installs and runs, but something is dropped (own sound bank / sound.zip, announcer call, own effect bank, Kirby copy hat, per-costume
 *               icons/CSPs): the note says which
 *   needs-base  refused: callbacks point outside the retail dol (built for another clone base: ACE / Akaneia), custom items, a file missing from the zip
 * Callback ADDRESSES that are inside the retail range but have no native counterpart are only known to the runtime (the bridge); it logs each and keeps the
 * clone base's callback, never jumping to it. */
#ifndef GW_NUCLEUS_FIGHTER_H
#define GW_NUCLEUS_FIGHTER_H

#include "gw_nucleus_stage.h"

#define NF_NCB 46
/* fighter.json "functions" key -> Arch_FighterFunc slot (the m-ex Header order, pc/platform/gw_mex_ftfunction.c slot_names). -1 = not a callback the port reads */
static const struct { const char *key; int slot; } nf_cb_keys[] = {
    { "onLoad", 0 }, { "onRespawn", 1 }, { "onDestroy", 2 }, { "moveLogicPointer", 3 },
    { "specialN", 4 }, { "specialNAir", 5 }, { "specialS", 6 }, { "specialSAir", 7 }, { "specialHi", 8 }, { "specialHiAir", 9 }, { "specialLw", 10 }, { "specialLwAir", 11 },
    { "onAbsorb", 12 }, { "onItemPickup", 13 }, { "onMakeItemInvisible", 14 }, { "onMakeItemVisible", 15 }, { "onItemDrop", 16 }, { "onItemCatch", 17 },
    { "onUnknownItemRelated", 18 }, { "onApplyHeadItem", 19 }, { "onRemoveHeadItem", 20 }, { "eyeTextureDamaged", 21 }, { "eyeTextureNormal", 22 },
    { "onFrame", 23 }, { "onActionStateChange", 24 }, { "resetAttribute", 25 }, { "onModelRender", 26 }, { "onShadowRender", 27 }, { "onUnknownMultijump", 28 },
    { "onActionStateChangeWhileEyeTextureIsChanged1", 29 }, { "onTwoEntryTable1", 30 }, { "enterFloat", 31 }, { "enterDoubleJump", 32 }, { "enterTether", 33 },
    { "onLanding", 34 }, { "onSmashF", 35 }, { "onSmashHi", 36 }, { "onSmashLw", 37 }, { "onExtRstAnim", 38 }, { "onIndexExtRstAnim", 39 },
    { "demoMoveLogicPointer", 40 }, { "onIntroL", 41 }, { "onIntroR", 42 }, { "onAppeal", 43 }, { "onCatch", 44 }, { "getSwordTrail", 45 },
};
#define NF_N_CB_KEYS ((int) (sizeof nf_cb_keys / sizeof nf_cb_keys[0]))

/* fighter.json boneDefinitions.lookup key -> the engine's Fighter_Part index (src/melee/ft/forward.h). MexManager's order is not the engine's: it has waistB, and lHave / rHave sit
 * before the thumbs. Matching is by NAME. */
static const char *const nf_bone_names[53] = {
    "top", "trans", "xRot", "yRot", "hip", "waist", "lLegA", "lLeg", "lKnee", "lFootA", "lFoot", "rLegA", "rLeg", "rKnee", "rFootA", "rFoot",
    "bust", "lClavicle", "lShoulderA", "lShoulder", "lArm", "lHand", "lIndex1", "lIndex2", "lMiddle1", "lMiddle2", "lRing1", "lRing2", "lPinky1", "lPinky2", "lThumb1", "lThumb2", "lHave",
    "neckN", "headN", "rClavicle", "rShoulderA", "rShoulder", "rArm", "rHand", "rIndex1", "rIndex2", "rMiddle1", "rMiddle2", "rRing1", "rRing2", "rPinky1", "rPinky2", "rThumb1", "rThumb2", "rHave",
    "throw", "extra" };

typedef struct {
    char name[48], sym[64], pl[64], anim[64], effect[64], post_verdict[16];
    int animcount, sound_bank, announcer, victory, music[2], ncost, items, kirby, have_sound_zip;
    char cfile[32][64], cjoint[32][64], cmat[32][64];
    int cvis[32];
    char demo[4][64];
    uint32_t cb[NF_NCB];
    int ncb, bone_count, nbones, bones[64];
    char needs[300], partial[300], compat[500]; /* needs: why it is refused; partial: what is dropped */
    char verdict[12];                            /* "works" | "partial" | "needs-base" */
} nf_fighter;

/* A callback value is acceptable when it is 0, a small constant (the json stores flags and counts in some slots), or a retail-dol address. */
static int nf_cb_ok(double v) { return v == 0.0 || (v > 0.0 && v < 1048576.0) || (v >= 2147483648.0 && v < 2172649472.0); }

static void nf_add(char *dst, size_t cap, const char *part) {
    size_t l = strlen(dst);
    if (l + 4 < cap) snprintf(dst + l, cap - l, "%s%s", l ? "; " : "", part);
}

/* Parse and check fighter.json. 0 ok (verdict set, maybe needs-base: the caller refuses), -1 with err for a file that is not a fighter package. */
static int nf_parse_json(const char *text, nf_fighter *o, char *err, size_t ecap) {
    nj_doc d;
    int root, fl, fn, cn, e, k, kb;
    char part[200];
    memset(o, 0, sizeof *o);
    root = nj_parse(&d, text);
    if (root < 0 || d.n[root].type != NJ_OBJ) { snprintf(err, ecap, "fighter.json is not a JSON object"); nj_free(&d); return -1; }
    nc_copy(o->name, sizeof o->name, nj_gstr(&d, root, "name", ""));
    if (!o->name[0]) { snprintf(err, ecap, "fighter.json has no name"); nj_free(&d); return -1; }
    fl = nj_get(&d, root, "files");
    if (fl < 0) { snprintf(err, ecap, "fighter.json has no files section"); nj_free(&d); return -1; }
    nc_copy(o->pl, sizeof o->pl, nc_basename(nj_gstr(&d, fl, "fighterDataPath", "")));
    nc_copy(o->sym, sizeof o->sym, nj_gstr(&d, fl, "fighterDataSymbol", ""));
    nc_copy(o->anim, sizeof o->anim, nc_basename(nj_gstr(&d, fl, "animFile", "")));
    nc_copy(o->effect, sizeof o->effect, nc_basename(nj_gstr(&d, fl, "effectFile", "")));
    o->animcount = (int) nj_gnum(&d, fl, "animCount", 0);
    if (strlen(o->pl) < 6 || strncmp(o->pl, "Pl", 2) != 0 || !nc_has_ext(o->pl, ".dat") || !o->sym[0] || !o->anim[0] || o->animcount <= 0 || o->animcount > 1024) {
        snprintf(err, ecap, "fighter.json does not name a fighter file (PlXx.dat), its data symbol and its animation file"); nj_free(&d); return -1;
    }
    o->sound_bank = (int) nj_gnum(&d, root, "soundBank", 55);
    o->announcer = (int) nj_gnum(&d, root, "announcerCall", -1);
    o->victory = (int) nj_gnum(&d, root, "victoryTheme", -1);
    o->music[0] = (int) nj_gnum(&d, root, "fighterMusic1", 0xFFFF);
    o->music[1] = (int) nj_gnum(&d, root, "fighterMusic2", 0xFFFF);
    if (o->music[0] < 0 || o->music[0] > 98) o->music[0] = 0xFFFF;
    if (o->music[1] < 0 || o->music[1] > 98) o->music[1] = 0xFFFF;
    snprintf(o->demo[0], 64, "%s", nj_gstr(&d, fl, "demoResult", ""));
    snprintf(o->demo[1], 64, "%s", nj_gstr(&d, fl, "demoIntro", ""));
    snprintf(o->demo[2], 64, "%s", nj_gstr(&d, fl, "demoEnding", ""));
    snprintf(o->demo[3], 64, "%s", nj_gstr(&d, fl, "demoWait", ""));
    kb = nj_get(&d, root, "kirbyCostumes");
    o->kirby = (nj_get(&d, root, "hasKirbyCostumes") >= 0 && nj_gnum(&d, root, "hasKirbyCostumes", 0) != 0.0) || (kb >= 0 && d.n[kb].type == NJ_ARR && d.n[kb].first >= 0) ||
               nj_gstr(&d, fl, "kirbyCapFileName", "")[0] != '\0';
    e = nj_get(&d, root, "items");
    o->items = (e >= 0 && d.n[e].type == NJ_ARR && d.n[e].first >= 0);
    cn = nj_get(&d, root, "costumes");
    if (cn >= 0 && d.n[cn].type == NJ_ARR) {
        for (e = d.n[cn].first; e >= 0 && o->ncost < 32; e = d.n[e].next) {
            int f = nj_get(&d, e, "file");
            if (f < 0) continue;
            nc_copy(o->cfile[o->ncost], 64, nc_basename(nj_gstr(&d, f, "fileName", "")));
            nc_copy(o->cjoint[o->ncost], 64, nj_gstr(&d, f, "jointSymbol", ""));
            nc_copy(o->cmat[o->ncost], 64, nj_gstr(&d, f, "materialSymbol", ""));
            o->cvis[o->ncost] = (int) nj_gnum(&d, f, "visibilityIndex", 0);
            if (o->cfile[o->ncost][0] && o->cjoint[o->ncost][0]) ++o->ncost;
        }
    }
    if (o->ncost == 0) { snprintf(err, ecap, "fighter.json lists no costume with a file and symbols"); nj_free(&d); return -1; }
    {   /* boneDefinitions.lookup: role -> joint index in the fighter's own skeleton (255 = none), in the engine's FtPart order */
        int bd = nj_get(&d, root, "boneDefinitions"), lk = bd >= 0 ? nj_get(&d, bd, "lookup") : -1, c, found = 0;
        memset(o->bones, 0xFF, sizeof o->bones);
        if (lk >= 0 && d.n[lk].type == NJ_OBJ) {
            for (c = d.n[lk].first; c >= 0; c = d.n[c].next) {
                const char *key = d.n[c].key >= 0 ? d.pool + d.n[c].key : "";
                if (!strcmp(key, "boneCount")) { o->bone_count = (int) d.n[c].num; continue; }
                if (!strcmp(key, "trimmedSize")) continue;
                if (d.n[c].type == NJ_NUM) {
                    int q;
                    for (q = 0; q < 53; ++q) if (!strcmp(key, nf_bone_names[q])) { o->bones[q] = (int) d.n[c].num; ++found; break; }
                }
            }
            o->nbones = found == 53 ? 53 : 0;
            if (o->bone_count < 1 || o->bone_count > 255 || o->nbones != 53) o->bone_count = o->nbones = 0;
        }
    }
    fn = nj_get(&d, root, "functions");
    if (fn < 0 || d.n[fn].type != NJ_OBJ) { snprintf(err, ecap, "fighter.json has no functions section"); nj_free(&d); return -1; }
    {
        int bad = 0;
        char first[64] = "";
        for (k = 0; k < NF_N_CB_KEYS; ++k) {
            double v;
            if (nj_get(&d, fn, nf_cb_keys[k].key) < 0) continue;
            v = nj_gnum(&d, fn, nf_cb_keys[k].key, 0.0);
            if (!nf_cb_ok(v)) { if (!bad) snprintf(first, sizeof first, "%s", nf_cb_keys[k].key); ++bad; continue; }
            o->cb[nf_cb_keys[k].slot] = (uint32_t) v;
            if (v != 0.0) ++o->ncb;
        }
        if (bad) {
            snprintf(part, sizeof part, "%d callback(s) point outside the retail code (%s...): built for another clone base (ACE / Akaneia)", bad, first);
            nf_add(o->needs, sizeof o->needs, part);
        }
    }
    o->have_sound_zip = 0;
    nj_free(&d);
    return 0;
}

/* The verdict words and the plain-language note. Call after the zip's contents are known (have_sound_zip). */
static void nf_verdict(nf_fighter *o) {
    if (o->items) nf_add(o->needs, sizeof o->needs, "it owns custom items, which need the item tables of an m-ex disc");
    if (o->needs[0]) {
        snprintf(o->verdict, sizeof o->verdict, "needs-base");
        snprintf(o->compat, sizeof o->compat, "Needs another base: %s.", o->needs);
        return;
    }
    if (o->sound_bank > 54) nf_add(o->partial, sizeof o->partial, "its own sound bank is not loaded on a vanilla disc (its sound effects are silent)");
    else if (o->have_sound_zip) nf_add(o->partial, sizeof o->partial, "sound.zip is not installed (the fighter uses the retail sound bank)");
    if (o->announcer > 0) nf_add(o->partial, sizeof o->partial, "the announcer call is the clone base's");
    if (o->kirby) nf_add(o->partial, sizeof o->partial, "Kirby cannot copy this fighter's hat");
    if (!(strlen(o->effect) == 12 && !strncmp(o->effect, "Ef", 2) && !strcmp(o->effect + 4, "Data.dat")) && o->effect[0]) nf_add(o->partial, sizeof o->partial, "its own effect bank is not loaded (the clone base's effects are used)");
    nf_add(o->partial, sizeof o->partial, "the CSS shows the clone base's icon and stock art");
    if (o->partial[0] && strcmp(o->partial, "the CSS shows the clone base's icon and stock art") != 0) {
        snprintf(o->verdict, sizeof o->verdict, "partial");
        snprintf(o->compat, sizeof o->compat, "Works, partial: %s.", o->partial);
    } else {
        snprintf(o->verdict, sizeof o->verdict, "works");
        snprintf(o->compat, sizeof o->compat, "Works: new fighter with its own file, animations and costumes (%s).", o->partial);
    }
}

/* mod-private file stem for a post: PlN + 4 hex digits of the post id. Never a retail name (retail stems are two letters). */
static void nf_stem(int post_id, char *out, size_t cap) { snprintf(out, cap, "PlN%04X", (unsigned) post_id & 0xFFFFu); }

/* Installs the post's custom fighter(s). Same contract as nc_stage_install: 0 when at least one mod was written. */
static int nc_fighter_install(const char *mods_dir, const nc_post *post, const nc_in_file *in, int nin, const nc_loose_env *env, nc_loose_result *res,
                              char *msg, size_t mcap, void (*logf)(const char *line)) {
    int i;
    char line[400];
    (void) env;
    memset(res, 0, sizeof *res);
    for (i = 0; i < nin; ++i) {
        uint8_t *js = NULL, *dat = NULL, *aj = NULL, *cz = NULL, *cdat[32];
        size_t jl = 0, dl = 0, al = 0, czl = 0, cl[32];
        char err[200], mod_id[96], stem[16], stage_dir[620], final_dir[620], tmp[760], conf[16][64], target[64], tname[40][64];
        nf_fighter f;
        nc_symset cs;
        nj_buf b, t;
        int c, k, ncf, ntarget = 0, wrote_err = 0;
        uint8_t *probe = NULL;
        size_t probel = 0;
        memset(cdat, 0, sizeof cdat);
        if (in[i].cat != NC_FC_ARCHIVE || !nc_has_ext(in[i].filename, ".zip")) continue;
        if (nc_zip_extract(in[i].data, in[i].len, "fighter.json", &js, &jl, 1u << 20, err, sizeof err) < 0) {
            snprintf(res->first_err, sizeof res->first_err, "No fighter.json in %s (%s).", nc_basename(in[i].filename), err); ++res->refused; continue;
        }
        js = (uint8_t *) realloc(js, jl + 1); js[jl] = '\0';
        if (nf_parse_json((const char *) js, &f, err, sizeof err) < 0) { snprintf(res->first_err, sizeof res->first_err, "%s.", err); ++res->refused; free(js); continue; }
        free(js);
        if (nc_zip_extract(in[i].data, in[i].len, "sound.zip", &probe, &probel, 64u * 1024 * 1024, err, sizeof err) == 0) { f.have_sound_zip = 1; free(probe); }
        nf_verdict(&f);
        if (!strcmp(f.verdict, "needs-base")) {
            snprintf(res->first_err, sizeof res->first_err, "%s", f.compat); ++res->refused; continue;
        }
        if (nc_zip_extract(in[i].data, in[i].len, f.pl, &dat, &dl, 24u * 1024 * 1024, err, sizeof err) < 0) {
            snprintf(res->first_err, sizeof res->first_err, "The package has no %s (%s).", f.pl, err); ++res->refused; continue;
        }
        if (nc_dat_symbols(dat, dl, &cs, err, sizeof err) < 0 || !nc_symset_has(&cs, nc_fnv32(f.sym))) {
            snprintf(res->first_err, sizeof res->first_err, "%s is not a fighter file (%s).", f.pl, cs.h ? "its data symbol is missing" : err); ++res->refused; free(dat);
            if (cs.h) nc_symset_free(&cs);
            continue;
        }
        nc_symset_free(&cs);
        if (nc_zip_extract(in[i].data, in[i].len, f.anim, &aj, &al, 64u * 1024 * 1024, err, sizeof err) < 0) {
            snprintf(res->first_err, sizeof res->first_err, "The package has no %s (%s).", f.anim, err); ++res->refused; free(dat); continue;
        }
        for (c = 0; c < f.ncost; ++c) { /* a costume file is a .dat in the zip, or a zip of that .dat */
            char zn[80];
            snprintf(zn, sizeof zn, "%.*s.zip", (int) (strlen(f.cfile[c]) - (nc_has_ext(f.cfile[c], ".dat") ? 4 : 0)), f.cfile[c]);
            if (nc_zip_extract(in[i].data, in[i].len, f.cfile[c], &cdat[c], &cl[c], 24u * 1024 * 1024, err, sizeof err) < 0) {
                cdat[c] = NULL;
                if (nc_zip_extract(in[i].data, in[i].len, zn, &cz, &czl, 24u * 1024 * 1024, err, sizeof err) == 0) {
                    if (nc_zip_extract(cz, czl, f.cfile[c], &cdat[c], &cl[c], 24u * 1024 * 1024, err, sizeof err) < 0) cdat[c] = NULL;
                    free(cz); cz = NULL;
                }
            }
            if (cdat[c] == NULL) {
                snprintf(res->first_err, sizeof res->first_err, "The package has no costume file %s.", f.cfile[c]); ++res->refused; wrote_err = 1; break;
            }
        }
        if (wrote_err) { for (c = 0; c < 32; ++c) free(cdat[c]); free(dat); free(aj); continue; }
        nf_stem(post->id, stem, sizeof stem);
        snprintf(mod_id, sizeof mod_id, "nucleus-%d-fighter", post->id);
        snprintf(stage_dir, sizeof stage_dir, "%s/.staging-%s", mods_dir, mod_id);
        snprintf(final_dir, sizeof final_dir, "%s/%s", mods_dir, mod_id);
        nc_fix_slashes(stage_dir); nc_fix_slashes(final_dir);
        nc_rmtree(stage_dir);
        snprintf(target, sizeof target, "%s.dat", stem);
        snprintf(tname[ntarget++], 64, "%s", target);
        k = 0;
        snprintf(tmp, sizeof tmp, "%s/files/%s", stage_dir, target); nc_fix_slashes(tmp);
        { char dd[760], *sl = strrchr(tmp, '\\'); if (!sl) sl = strrchr(tmp, '/'); if (sl) { snprintf(dd, sizeof dd, "%.*s", (int) (sl - tmp), tmp); nc_mkdirs(dd); } }
        k |= nc_write_file(tmp, dat, dl) != 0;
        snprintf(tname[ntarget++], 64, "%sAJ.dat", stem);
        snprintf(tmp, sizeof tmp, "%s/files/%sAJ.dat", stage_dir, stem); nc_fix_slashes(tmp);
        k |= nc_write_file(tmp, aj, al) != 0;
        for (c = 0; c < f.ncost && ntarget < 40; ++c) {
            snprintf(tname[ntarget], 64, "%sC%d.dat", stem, c);
            snprintf(tmp, sizeof tmp, "%s/files/%s", stage_dir, tname[ntarget]); nc_fix_slashes(tmp);
            k |= nc_write_file(tmp, cdat[c], cl[c]) != 0;
            ++ntarget;
        }
        for (c = 0; c < 32; ++c) free(cdat[c]);
        free(dat); free(aj);
        if (k) { snprintf(res->first_err, sizeof res->first_err, "Cannot write to the mods folder."); nc_rmtree(stage_dir); continue; }
        memset(&t, 0, sizeof t);
        nj_printf(&t, "name=%s\npl=%s\nplsym=%s\nanim=%sAJ.dat\nanimcount=%d\npost=https://%s/post/%d\n", f.name, target, f.sym, stem, f.animcount, NC_SITE, post->id);
        for (c = 0; c < f.ncost; ++c) nj_printf(&t, "c%d=%sC%d.dat|%s|%s|%d\n", c, stem, c, f.cjoint[c], f.cmat[c], f.cvis[c]);
        if (f.demo[0][0] || f.demo[1][0] || f.demo[2][0] || f.demo[3][0]) nj_printf(&t, "demo=%s|%s|%s|%s\n", f.demo[0], f.demo[1], f.demo[2], f.demo[3]);
        if (f.victory >= 0 && f.victory <= 98) nj_printf(&t, "victory=%d\n", f.victory);
        if (f.music[0] != 0xFFFF || f.music[1] != 0xFFFF) nj_printf(&t, "bgm=%d,%d\n", f.music[0], f.music[1]);
        for (c = 0; c < NF_NCB; ++c) if (f.cb[c]) nj_printf(&t, "cb%d=0x%08X\n", c, (unsigned) f.cb[c]);
        if (f.bone_count > 0) { nj_printf(&t, "bonecount=%d\nbones=", f.bone_count); for (c = 0; c < f.nbones; ++c) nj_printf(&t, "%s%d", c ? "," : "", f.bones[c]); nj_puts(&t, "\n"); }
        snprintf(tmp, sizeof tmp, "%s/mexfighter.txt", stage_dir); nc_fix_slashes(tmp);
        k = t.bad || nc_write_file(tmp, t.s, t.n) != 0;
        free(t.s);
        if (k) { snprintf(res->first_err, sizeof res->first_err, "Cannot write mexfighter.txt."); nc_rmtree(stage_dir); continue; }
        ncf = nc_find_conflicts(mods_dir, mod_id, (char (*)[64]) tname, ntarget, conf, 16);
        memset(&b, 0, sizeof b);
        nj_puts(&b, "{\n  \"id\": "); nj_qstr(&b, mod_id);
        nj_puts(&b, ",\n  \"name\": "); nj_qstr(&b, f.name);
        nj_puts(&b, ",\n  \"version\": "); nj_qstr(&b, post->updated);
        nj_puts(&b, ",\n  \"kind\": \"fighter\",\n  \"authors\": "); nj_qstr(&b, post->author);
        nj_printf(&b, ",\n  \"description\": \"New fighter from SSBM Nucleus: %s/post/%d. Online, both players need it. %s\"", NC_SITE, post->id, f.compat);
        nj_puts(&b, ",\n  \"conflicts\": [");
        for (k = 0; k < ncf; ++k) { if (k) nj_puts(&b, ", "); nj_qstr(&b, conf[k]); }
        nj_puts(&b, "],\n  \"source\": \"nucleus\",\n  \"nucleus\": {\n    \"id\": ");
        nj_printf(&b, "%d,\n    \"author\": ", post->id); nj_qstr(&b, post->author);
        nj_puts(&b, ",\n    \"page_url\": "); nj_qstr(&b, post->page);
        nj_puts(&b, ",\n    \"updated_at\": "); nj_qstr(&b, post->updated);
        nj_puts(&b, ",\n    \"installed_at\": "); nj_qstr(&b, post->installed_at);
        nj_puts(&b, ",\n    \"category\": \"cfighter\",\n    \"fighter\": "); nj_qstr(&b, f.name);
        nj_puts(&b, ",\n    \"verdict\": "); nj_qstr(&b, f.verdict);
        nj_puts(&b, ",\n    \"compat\": "); nj_qstr(&b, f.compat);
        nj_puts(&b, ",\n    \"targets\": [");
        for (k = 0; k < ntarget; ++k) { if (k) nj_puts(&b, ", "); nj_qstr(&b, tname[k]); }
        nj_printf(&b, "],\n    \"files\": [{\"id\": %d, \"size\": %u, \"fnv64\": \"%016llx\"}]\n  }\n}\n", in[i].file_id, (unsigned) in[i].len, (unsigned long long) nc_fnv64(in[i].data, in[i].len));
        snprintf(tmp, sizeof tmp, "%s/mod.json", stage_dir); nc_fix_slashes(tmp);
        k = b.bad || nc_write_file(tmp, b.s, b.n) != 0;
        free(b.s);
        if (k) { snprintf(res->first_err, sizeof res->first_err, "Cannot write mod.json."); nc_rmtree(stage_dir); continue; }
        if (nc_swap_dir(stage_dir, final_dir) != 0) {
            nc_rmtree(stage_dir);
            snprintf(res->first_err, sizeof res->first_err, "Could not replace %s (the game may be using it). Restart and try again.", mod_id);
            continue;
        }
        { char off[8][64]; nc_loose_set_enabled(mods_dir, mod_id, 1, off, 8); }
        for (k = 0; k < ncf && res->nturned_off < 4; ++k) snprintf(res->turned_off[res->nturned_off++], 64, "%s", conf[k]);
        if (res->nmods < 8) snprintf(res->mods[res->nmods++], 64, "%s", mod_id);
        ++res->installed; res->files += ntarget;
        if (logf) { snprintf(line, sizeof line, "install %d: %s: custom fighter \"%s\" (%s, %d callbacks) verdict %s. %s", post->id, mod_id, f.name, target, f.ncb, f.verdict, f.compat); logf(line); }
        snprintf(res->first_err, sizeof res->first_err, "%s", f.compat);
    }
    if (res->installed > 0) {
        snprintf(msg, mcap, "Installed the fighter. Restart the game to use it. %s", res->first_err);
        return 0;
    }
    snprintf(msg, mcap, "%s", res->first_err[0] ? res->first_err : "No fighter package (.zip with a fighter.json) in this post.");
    return -1;
}

#endif
