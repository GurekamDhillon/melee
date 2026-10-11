/* gw_nucleus_kinds.h - what kind of file a Nucleus post's file is, read from its file type and NAME (the catalog's half; the installer then checks the CONTENT,
 * see gw_nucleus_loose.h). Pure C, header-only. Included by gw_nucleus_catalog.h.
 *
 * The tabs of the browser (COSTUMES / STAGES / EFFECTS / MENUS / AUDIO / OTHER) and the categories a file falls in, with the plain-words reason a file or a
 * post cannot be installed. The names that are listed here (retail stage files, the effect files, the four supported menu files) are file NAMES the game's own
 * loader asks for; nothing disc-derived is stored. */
#ifndef GW_NUCLEUS_KINDS_H
#define GW_NUCLEUS_KINDS_H

#include "gw_nucleus_core.h"

enum { NC_TAB_COSTUMES, NC_TAB_STAGES, NC_TAB_EFFECTS, NC_TAB_MENUS, NC_TAB_AUDIO, NC_TAB_OTHER, NC_TAB_N };
static const char *const nc_tab_label[NC_TAB_N] = { "Costumes", "Stages", "Effects", "Menus", "Audio", "Other" };

static int nc_has_ext(const char *fn, const char *ext) {
    size_t a = strlen(fn), b = strlen(ext);
    return a >= b && nc_ieq(fn + a - b, ext);
}

/* a file's category */
enum { NC_FC_OTHER, NC_FC_COSTUME, NC_FC_STAGE, NC_FC_EFFECT, NC_FC_FIGHTER, NC_FC_MENU, NC_FC_AUDIO, NC_FC_IMAGE, NC_FC_PATCH, NC_FC_ARCHIVE, NC_FC_N };

/* the versus stages' files (longest first where one is the start of another: GrPs1 before GrPs). GrPs1..4 are Pokemon Stadium's transformations. */
static const char *const nc_stage_stems[] = {
    "GrPs1", "GrPs2", "GrPs3", "GrPs4", "GrNBa", "GrNLa", "GrBb", "GrCn", "GrCs", "GrFs", "GrFz", "GrGb", "GrGd", "GrGr", "GrI1", "GrI2", "GrIm", "GrIz", "GrKg",
    "GrKr", "GrMc", "GrOk", "GrOp", "GrOt", "GrOy", "GrPs", "GrPu", "GrRc", "GrSh", "GrSt", "GrVe", "GrYt", "GrZe",
};
#define NC_N_STAGE_STEMS ((int) (sizeof nc_stage_stems / sizeof nc_stage_stems[0]))
/* the stages whose file the game asks for by PREFIX: it reads <prefix>.usd on an American disc, so a skin has to be there as well */
static const char *const nc_stage_usd_stems[] = { "GrPs", "GrCn", "GrOt", "GrVe" };

/* the retail stages the tabs and messages name */
static const struct { const char *stem, *name; } nc_stage_names[] = {
    { "GrNBa", "Battlefield" }, { "GrNLa", "Final Destination" }, { "GrSt", "Yoshi's Story" }, { "GrOp", "Dream Land" }, { "GrPs", "Pokemon Stadium" },
    { "GrIz", "Fountain of Dreams" }, { "GrCs", "Peach's Castle" }, { "GrKg", "Kongo Jungle" }, { "GrZe", "Brinstar" }, { "GrCn", "Corneria" },
    { "GrOt", "Onett" }, { "GrMc", "Mute City" }, { "GrRc", "Rainbow Cruise" }, { "GrGd", "Jungle Japes" }, { "GrGb", "Great Bay" }, { "GrSh", "Temple" },
    { "GrKr", "Brinstar Depths" }, { "GrYt", "Yoshi's Island" }, { "GrGr", "Green Greens" }, { "GrFs", "Fourside" }, { "GrI1", "Mushroom Kingdom" },
    { "GrI2", "Mushroom Kingdom II" }, { "GrVe", "Venom" }, { "GrPu", "Poke Floats" }, { "GrBb", "Big Blue" }, { "GrIm", "Icicle Mountain" },
    { "GrFz", "Flat Zone" }, { "GrOy", "Yoshi's Island (64)" }, { "GrOk", "Kongo Jungle (64)" },
    { "GrPs1", "Pokemon Stadium (fire)" }, { "GrPs2", "Pokemon Stadium (grass)" }, { "GrPs3", "Pokemon Stadium (water)" }, { "GrPs4", "Pokemon Stadium (rock)" },
};
static const char *nc_stage_name(const char *stem) {
    size_t i;
    for (i = 0; i < sizeof nc_stage_names / sizeof nc_stage_names[0]; ++i) if (!strcmp(nc_stage_names[i].stem, stem)) return nc_stage_names[i].name;
    return stem;
}
/* "GrPs3" -> the family stem the mod id and the conflicts use ("GrPs"); any other stem is its own family */
static void nc_stage_family(const char *stem, char *out, size_t cap) {
    if (!strncmp(stem, "GrPs", 4) && stem[4] >= '1' && stem[4] <= '4') snprintf(out, cap, "GrPs");
    else snprintf(out, cap, "%s", stem);
}
static int nc_stage_has_usd(const char *stem) {
    size_t i;
    for (i = 0; i < sizeof nc_stage_usd_stems / sizeof nc_stage_usd_stems[0]; ++i) if (!strcmp(stem, nc_stage_usd_stems[i])) return 1;
    return 0;
}

/* the four menu files that load as plain overrides (global: one active per file) */
static const char *const nc_menu_files[] = { "MnSlChr", "MnSlMap", "IfAll", "GmPause" };

/* the base name of a path (the part after the last / or \), as a view */
static const char *nc_basename(const char *p) {
    const char *b = p, *q;
    for (q = p; *q; ++q) if (*q == '/' || *q == '\\') b = q + 1;
    return b;
}

static int nc_word_boundary_before(const char *start, const char *p) { return p == start || !isalnum((unsigned char) p[-1]); }

/* ext of the form ".dat" / ".usd" right after position p? */
static int nc_dat_ext_at(const char *p) { return (p[0] == '.' && (nc_ieqn(p + 1, "dat", 3) || nc_ieqn(p + 1, "usd", 3)) && !isalnum((unsigned char) p[4])); }

/* A versus stage's file stem in a messy file name ("Pumpkin Hill - GrOp.dat" -> "GrOp", "GrPs3.dat" -> "GrPs3"). 1 when there is one. */
static int nc_stage_stem_in(const char *fn, char *stem, size_t cap) {
    int i;
    const char *p;
    for (p = fn; *p; ++p) {
        if (p[0] != 'G' || p[1] != 'r' || !nc_word_boundary_before(fn, p)) continue;
        for (i = 0; i < NC_N_STAGE_STEMS; ++i) {
            size_t n = strlen(nc_stage_stems[i]);
            if (strncmp(p, nc_stage_stems[i], n) == 0 && !isalnum((unsigned char) p[n])) { snprintf(stem, cap, "%s", nc_stage_stems[i]); return 1; }
        }
    }
    stem[0] = '\0';
    return 0;
}

/* An effect file name in a messy file name: EfFxData.dat, EfKbCa.dat (Kirby's copy effects). out: "EfFxData" (no extension). 1 when there is one. */
static int nc_effect_stem_in(const char *fn, char *out, size_t cap) {
    const char *p;
    for (p = fn; *p; ++p) {
        size_t n = 0;
        if (p[0] != 'E' || p[1] != 'f' || !isupper((unsigned char) p[2]) || !islower((unsigned char) p[3]) || !nc_word_boundary_before(fn, p)) continue;
        if (!strncmp(p + 4, "Data", 4)) n = 8;
        else if (!strncmp(p, "EfKb", 4) && isupper((unsigned char) p[4]) && islower((unsigned char) p[5])) n = 6;
        else continue;
        if (!nc_dat_ext_at(p + n)) continue;
        snprintf(out, cap, "%.*s", (int) n, p);
        return 1;
    }
    out[0] = '\0';
    return 0;
}

/* A menu / UI file (Mn*, If*, Gm* + a short name) in a file name: out is "MnSlChr" etc. supported: whether it is one of the four we load. */
static int nc_menu_stem_in(const char *fn, char *out, size_t cap, int *supported) {
    const char *p;
    size_t i;
    for (p = fn; *p; ++p) {
        size_t n = 2;
        if (!((p[0] == 'M' && p[1] == 'n') || (p[0] == 'I' && p[1] == 'f') || (p[0] == 'G' && p[1] == 'm')) || !nc_word_boundary_before(fn, p)) continue;
        while (isalnum((unsigned char) p[n]) && n < 12) ++n;
        if (n < 5 || !nc_dat_ext_at(p + n)) continue;
        snprintf(out, cap, "%.*s", (int) n, p);
        *supported = 0;
        for (i = 0; i < sizeof nc_menu_files / sizeof nc_menu_files[0]; ++i) if (!strcmp(out, nc_menu_files[i])) *supported = 1;
        return 1;
    }
    out[0] = '\0';
    *supported = 0;
    return 0;
}

/* the file category from the API's file type and the file's name. stem: the target the name points at (see nc_file) or "". */
static int nc_file_category(const char *file_type, const char *filename, char *stem, size_t cap, int *supported) {
    const char *bn = nc_basename(filename);
    char tmp[24];
    int sup = 0;
    stem[0] = '\0';
    *supported = 0;
    if (nc_has_ext(filename, ".hps") || nc_has_ext(filename, ".ssm")) { *supported = 1; return NC_FC_AUDIO; }
    if (nc_has_ext(filename, ".png") || nc_has_ext(filename, ".jpg") || nc_has_ext(filename, ".tpl") || nc_has_ext(filename, ".webp") || nc_has_ext(filename, ".bmp")) return NC_FC_IMAGE;
    if (nc_has_ext(filename, ".xdelta") || nc_has_ext(filename, ".pat") || nc_has_ext(filename, ".ips") || nc_has_ext(filename, ".bps")) return NC_FC_PATCH;
    if (nc_has_ext(filename, ".zip") || nc_has_ext(filename, ".7z") || nc_has_ext(filename, ".rar")) return NC_FC_ARCHIVE;
    if (nc_stage_stem_in(bn, tmp, sizeof tmp) || (!strcmp(file_type, "stage_dat") && nc_stage_stem_in(filename, tmp, sizeof tmp))) {
        const char *ext = nc_has_ext(filename, ".usd") ? ".usd" : ".dat";
        if (nc_has_ext(filename, ".dat") || nc_has_ext(filename, ".usd")) { snprintf(stem, cap, "%s%s", tmp, ext); *supported = 1; }
        return NC_FC_STAGE;
    }
    if (!strcmp(file_type, "stage_dat")) return NC_FC_STAGE;                          /* a stage file we cannot name: shown, not installed */
    if (nc_effect_stem_in(bn, tmp, sizeof tmp)) { snprintf(stem, cap, "%s%s", tmp, nc_has_ext(filename, ".usd") ? ".usd" : ".dat"); *supported = 1; return NC_FC_EFFECT; }
    if (nc_menu_stem_in(bn, tmp, sizeof tmp, &sup)) { snprintf(stem, cap, "%s%s", tmp, nc_has_ext(filename, ".usd") ? ".usd" : ".dat"); *supported = sup; return NC_FC_MENU; }
    return NC_FC_OTHER;       /* costumes and fighter files are told apart by the caller (they need the fighter lookup) */
}

/* why a file of this category cannot be installed (stem / supported from nc_file_category) */
static const char *nc_cat_why_not(int cat, int supported) {
    switch (cat) {
    case NC_FC_STAGE: return supported ? "" : "Not a retail versus stage file name: needs conversion.";
    case NC_FC_EFFECT: case NC_FC_AUDIO: return "";
    case NC_FC_MENU: return supported ? "" : "This menu file is not supported yet (only MnSlChr, MnSlMap, IfAll and GmPause).";
    case NC_FC_FIGHTER: return "Ships a whole fighter file (PlXx.dat): it would replace the fighter itself, not just its look.";
    case NC_FC_IMAGE: return "Image / texture files: not installable yet.";
    case NC_FC_PATCH: return "Patch file (xdelta / pat): not a mod, it patches the whole disc.";
    case NC_FC_ARCHIVE: return "Packed archive (zip / 7z / rar): not unpacked yet.";
    case NC_FC_COSTUME: return "";
    default: return "Not a file GD can load as a loose override.";
    }
}

/* the tab a post is shown in, from its API type and the categories of its files (cmask: bit per NC_FC_*) */
static int nc_tab_of(int type, unsigned cmask) {
    if (type == NC_T_COSTUME) return NC_TAB_COSTUMES;
    if (type == NC_T_STAGE_SKIN || type == NC_T_CUSTOM_STAGE) return NC_TAB_STAGES;
    if (type == NC_T_EFFECTS) return NC_TAB_EFFECTS;
    if (cmask & (1u << NC_FC_STAGE)) return NC_TAB_STAGES;
    if (cmask & (1u << NC_FC_MENU)) return NC_TAB_MENUS;
    if (cmask & (1u << NC_FC_AUDIO)) return NC_TAB_AUDIO;
    if (cmask & (1u << NC_FC_EFFECT)) return NC_TAB_EFFECTS;
    return NC_TAB_OTHER;
}

#endif
