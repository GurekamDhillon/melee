/* gmfrontend_menus.inc - included by gmfrontend.c (TARGET_PC only); see the build note in
 * gmfrontend_player.inc.
 *
 * ---- THE MAIN-MENU TREE, AS FRONTEND SCREENS ------------------------------------------------
 *
 * Every screen of Melee's GM_MENU tree that is only a list of choices - Main, 1P (Solo), Regular
 * Match, Stadium, Multi-Man Melee, VS, Special Melee, Trophies (Collection), Options, Data and
 * Records - is a frontend screen here: a HUB (tiles, hub_layout rules) or a LIST (the same
 * system with equal rows). Each screen stands for one MenuKind and each item for one of that
 * menu's selection indices, so a position is always the pair vanilla uses: (MenuKind,
 * selection). That is what lets all of vanilla's own positioning keep working unchanged:
 *
 *   - arriving from a game mode, the screen and cursor come from the same table gmmenumode.c's
 *     onEnter uses (the previous mode -> (kind, selection));
 *   - a native screen backing out calls mn_80229894(kind, selection, ...) - mnmain.c hands that
 *     position to gmFrontend_NativeReturn, which re-enters the frontend there;
 *   - L+R+START, the language change and force_main_menu go through GM_MENU exactly as before
 *     and are routed here.
 *
 * WHAT EACH ITEM DOES is written out per item below from the corresponding mnmain.c think
 * (mn_8022DB10 main, mn_8022D7F4 1P, mn_8022D594 VS, mn_8022D34C trophies, mn_8022D104
 * settings, mn_8022CE6C data, mn_8022CC28 regular, mn_8022C7CC stadium, mn_8022C4F4 special,
 * mn_8022CA54 records, mnHyaku_8024C68C multi-man): the same game mode, the same
 * gm_801677E8(port) call where vanilla makes it, the same sounds (sfxForward on confirm except
 * Sound Test, sfxBack on B, sfxMove on a cursor move), B to the same parent item. Locked items
 * use vanilla's own predicate, mn_80229938, and are hidden as vanilla hides them.
 *
 * NATIVE SCREENS. Anything that is not a list of choices (Rules, Name Entry, the Event list,
 * Options' sub-screens, Snapshots, Movies, Sound Test, Special Messages, the three Records
 * tables) still runs natively inside GM_MENU: the frontend leaves for GM_MENU with a request
 * (parent kind, selection); gmmenumode.c positions the menu there and mnmain.c opens that
 * screen exactly as the parent's think would have (mn_PcOpenNative). Its own back-out returns
 * through mn_80229894 to the frontend.
 *
 * VS > Melee goes through the MATCH SETUP screen before the CSS, as it did from the native menu.
 */

enum { FM_HUB, FM_LIST };
enum { FA_SUB, FA_MODE, FA_NATIVE, FA_MATCH_SETUP, FA_ONLINE, FA_PAGE };
#define SEL_VS_ONLINE 0x40 ///< not a vanilla selection: VERSUS > ONLINE is the port's
#define SEL_1P_LAB 0x42    ///< not a vanilla selection: SOLO > LAB (the Geno Lab, private build)
#define FMF_PORT 1  ///< gm_801677E8(the port that pressed A) before acting, as vanilla does
#define FMF_NOSFX 2 ///< no sfxForward (Data > Sound Test stops the music instead)

typedef struct FeMenuItem {
    const char* label;
    const char* desc;
    const char* icon; ///< texture name (brief section 2), NULL = none
    u8 sel;           ///< the vanilla selection index this item is
    u8 act;           ///< FA_*
    u8 arg;           ///< FA_SUB: MenuKind; FA_MODE: GameModeKind
    u8 flags;
} FeMenuItem;

typedef struct FeMenu {
    u8 kind;  ///< MenuKind
    u8 style; ///< FM_HUB / FM_LIST
    u8 hero;  ///< for a hub: the selection that gets the hero tile
    u8 back_kind, back_sel; ///< where B goes; back_kind 0xFF = GM_TITLE
    u8 port_all;            ///< vanilla calls gm_801677E8 for every confirm on this menu
    const char* title;
    const FeMenuItem* items;
    int n;
} FeMenu;

#define FM_N(a) ((int) (sizeof a / sizeof a[0]))

static const FeMenuItem fm_main[] = {
    { "SOLO", "Classic, Adventure, events, training and more.", "ico_solo", SEL_MAIN_1P, FA_SUB,
      MENU_KIND_1P },
    { "VERSUS", "Up to four players. Your rules, your stage.", "ico_versus", SEL_MAIN_VS, FA_SUB,
      MENU_KIND_VS },
    { "COLLECTION", "Trophies: gallery, lottery and your collection.", "ico_collection",
      SEL_MAIN_TOY, FA_SUB, MENU_KIND_TOY },
    { "SETTINGS", "Video, audio, controls, online, mods and the game's options.", "ico_options",
      SEL_MAIN_SETTINGS, FA_SUB, MENU_KIND_SETTINGS },
    { "DATA", "Snapshots, movies, sound test, records, messages.", "ico_data", SEL_MAIN_DATA,
      FA_SUB, MENU_KIND_DATA },
};

static const FeMenuItem fm_1p[] = {
    { "REGULAR MATCH", "Classic, Adventure and All-Star.", "ico_regular", SEL_1P_REG, FA_SUB,
      MENU_KIND_REG },
    { "EVENT MATCH", "Special battles with their own rules.", "ico_event", SEL_1P_EVENT,
      FA_NATIVE },
    { "STADIUM", "Target Test, Home-Run Contest and Multi-Man.", "ico_stadium", SEL_1P_STADIUM,
      FA_SUB, MENU_KIND_STADIUM },
    { "TRAINING", "Practice against a dummy, any stage, any speed.", "ico_training",
      SEL_1P_TRAINING, FA_MODE, GM_TRAINING },
    { "LAB", "The Geno Lab: any fighters, no timer, frame-step, hitboxes, timelines.", "ico_lab",
      SEL_1P_LAB, FA_MODE, GM_LAB },
};

static const FeMenuItem fm_reg[] = {
    { "CLASSIC", "A series of battles, then the final boss.", "ico_classic", SEL_REG_CLASSIC,
      FA_MODE, GM_CLASSIC },
    { "ADVENTURE", "Side-scrolling stages between the battles.", "ico_adventure",
      SEL_REG_ADVENTURE, FA_MODE, GM_ADVENTURE },
    { "ALL-STAR", "Every fighter, one after another.", "ico_allstar", SEL_REG_ALLSTAR, FA_MODE,
      GM_ALLSTAR },
};

static const FeMenuItem fm_stadium[] = {
    { "TARGET TEST", "Break every target as fast as you can.", "ico_target", SEL_STADIUM_TARGET,
      FA_MODE, GM_TARGET_TEST, FMF_PORT },
    { "HOME-RUN CONTEST", "Launch the Sandbag as far as it will go.", "ico_homerun",
      SEL_STADIUM_HOMERUN, FA_MODE, GM_HOME_RUN_CONTEST, FMF_PORT },
    { "MULTI-MAN MELEE", "Take on a whole team of fighters.", "ico_multiman",
      SEL_STADIUM_MULTIMAN, FA_SUB, MENU_KIND_MULTI_VS },
};

static const FeMenuItem fm_multi[] = {
    { "10-MAN MELEE", "KO ten fighters.", NULL, SEL_MULTI_VS_10MAN, FA_MODE, GM_10MAN_VS,
      FMF_PORT },
    { "100-MAN MELEE", "KO one hundred fighters.", NULL, SEL_MULTI_VS_100MAN, FA_MODE,
      GM_100MAN_VS, FMF_PORT },
    { "3-MINUTE MELEE", "As many KOs as you can in three minutes.", NULL, SEL_MULTI_VS_3MIN,
      FA_MODE, GM_3MIN_VS, FMF_PORT },
    { "15-MINUTE MELEE", "Survive fifteen minutes.", NULL, SEL_MULTI_VS_15MIN, FA_MODE,
      GM_15MIN_VS, FMF_PORT },
    { "ENDLESS MELEE", "Fight until you fall.", NULL, SEL_MULTI_VS_ENDLESS, FA_MODE,
      GM_ENDLESS_VS, FMF_PORT },
    { "CRUEL MELEE", "Tough opponents. Good luck.", NULL, SEL_MULTI_VS_CRUEL, FA_MODE,
      GM_CRUEL_VS, FMF_PORT },
};

static const FeMenuItem fm_vs[] = {
    { "MELEE", "Up to four players. Your rules, your stage.", "ico_melee", SEL_VS_MELEE,
      FA_MATCH_SETUP, GM_VS },
    { "ONLINE", "Play someone over the internet: host a room or join one.", "ico_online",
      SEL_VS_ONLINE, FA_ONLINE, GM_VS },
    { "TOURNAMENT", "A bracket for up to 64 players.", "ico_tournament", SEL_VS_TOURNAMENT,
      FA_MODE, GM_TOURNAMENT },
    { "SPECIAL MELEE", "Melee with a twist.", "ico_special", SEL_VS_SPECIAL, FA_SUB,
      MENU_KIND_SPECIAL },
    { "RULES", "Stock, time, items and stages.", "ico_rules", SEL_VS_RULES, FA_NATIVE },
    { "NAME ENTRY", "Make a name tag.", "ico_names", SEL_VS_NAME, FA_NATIVE },
};

static const FeMenuItem fm_special[] = {
    { "CAMERA MODE", "Take pictures during the match.", NULL, SEL_SPECIAL_VS_CAMERA, FA_MODE,
      GM_CAMERA_MODE },
    { "STAMINA MODE", "Knock out an opponent's health.", NULL, SEL_SPECIAL_VS_STAMINA,
      FA_MODE, GM_STAMINA_VS },
    { "SUPER SUDDEN DEATH", "Everyone starts at 300%.", NULL, SEL_SPECIAL_VS_SUDDEN_DEATH,
      FA_MODE, GM_SUPER_SUDDEN_DEATH_VS },
    { "GIANT MELEE", "Everyone is huge.", NULL, SEL_SPECIAL_VS_GIANT, FA_MODE, GM_GIANT_VS },
    { "TINY MELEE", "Everyone is tiny.", NULL, SEL_SPECIAL_VS_TINY, FA_MODE, GM_TINY_VS },
    { "INVISIBLE MELEE", "Now you see them...", NULL, SEL_SPECIAL_VS_INVISIBLE, FA_MODE,
      GM_INVISIBLE_VS },
    { "FIXED-CAMERA MODE", "The camera does not follow.", NULL, SEL_SPECIAL_VS_FIXED_CAMERA,
      FA_MODE, GM_CAMERA_VS },
    { "SINGLE-BUTTON MODE", "One attack button.", NULL, SEL_SPECIAL_VS_SINGLE_BUTTON, FA_MODE,
      GM_SINGLE_BUTTON_VS },
    { "LIGHTNING MELEE", "Everything moves faster.", NULL, SEL_SPECIAL_VS_LIGHTNING, FA_MODE,
      GM_LIGHTNING_VS },
    { "SLO-MO MELEE", "Everything moves slower.", NULL, SEL_SPECIAL_VS_SLOMO, FA_MODE,
      GM_SLOMO_VS },
};

static const FeMenuItem fm_toy[] = {
    { "GALLERY", "Look at the trophies you have.", "ico_gallery", SEL_TOY_GALLERY, FA_MODE,
      GM_TOY_GALLERY },
    { "LOTTERY", "Spend coins on new trophies.", "ico_lottery", SEL_TOY_LOTTERY, FA_MODE,
      GM_TOY_LOTTERY },
    { "COLLECTION", "Arrange trophies on a stand.", "ico_trophies", SEL_TOY_COLLECTION, FA_MODE,
      GM_TOY_COLLECTION },
};

static const FeMenuItem fm_settings[] = {
    /* the port's pages (gmfrontend_settings.inc); the game's own options follow */
    { "VIDEO", "Render scale, frame rate, vsync, FPS readout.", NULL, 0x41, FA_PAGE, 0 },
    { "AUDIO", "Master volume, music and effects mix, stereo or mono.", NULL, 0x42, FA_PAGE, 1 },
    { "CONTROLS", "Adapter recalibration, the hotkeys and the mouse.", NULL, 0x43, FA_PAGE, 2 },
    { "ONLINE", "Your player name, the server, input delay, stage list.", NULL, 0x44, FA_PAGE, 3 },
    { "MODS", "Turn mods on or off.", NULL, 0x45, FA_PAGE, 4 },
    { "GAMEPLAY", "Unlock everything, the default match rules.", NULL, 0x46, FA_PAGE, 5 },
    { "RUMBLE", "Turn controller rumble on or off.", NULL, SEL_SETTINGS_RUMBLE, FA_NATIVE },
    { "SCREEN DISPLAY", "Flicker filter.", NULL, SEL_SETTINGS_DISPLAY, FA_NATIVE },
    { "LANGUAGE", "English or Japanese.", NULL, SEL_SETTINGS_LANG, FA_NATIVE },
    { "ERASE DATA", "Delete save data.", NULL, SEL_SETTINGS_ERASE, FA_NATIVE },
};

static const FeMenuItem fm_data[] = {
    { "SNAPSHOTS", "Pictures saved to the memory card.", "ico_snapshots", SEL_DATA_SNAP,
      FA_NATIVE },
    { "MOVIES", "Watch the movies you have seen.", "ico_movies", SEL_DATA_ARCHIVES, FA_NATIVE },
    { "SOUND TEST", "Listen to music, effects and voices.", "ico_soundtest", SEL_DATA_SOUND,
      FA_NATIVE, 0, FMF_NOSFX },
    { "RECORDS", "VS, bonus and other records.", "ico_records", SEL_DATA_RECORDS, FA_SUB,
      MENU_KIND_RECORDS },
    { "SPECIAL MESSAGES", "Messages you have unlocked.", "ico_messages", SEL_DATA_SPECIAL,
      FA_NATIVE },
};

static const FeMenuItem fm_records[] = {
    { "VS. RECORDS", "Records from VS. matches.", NULL, SEL_RECORDS_VS, FA_NATIVE },
    { "BONUS RECORDS", "Bonuses you have earned.", NULL, SEL_RECORDS_BONUS, FA_NATIVE },
    { "MISC. RECORDS", "Everything else the game counts.", NULL, SEL_RECORDS_MISC, FA_NATIVE },
};

static const FeMenu fm_menus[] = {
    { MENU_KIND_MAIN, FM_HUB, SEL_MAIN_VS, 0xFF, 0, 0, "MAIN MENU", fm_main, FM_N(fm_main) },
    { MENU_KIND_1P, FM_HUB, SEL_1P_REG, MENU_KIND_MAIN, SEL_MAIN_1P, 1, "SOLO", fm_1p,
      FM_N(fm_1p) },
    { MENU_KIND_REG, FM_HUB, SEL_REG_CLASSIC, MENU_KIND_1P, SEL_1P_REG, 1, "REGULAR MATCH",
      fm_reg, FM_N(fm_reg) },
    { MENU_KIND_STADIUM, FM_HUB, SEL_STADIUM_TARGET, MENU_KIND_1P, SEL_1P_STADIUM, 0,
      "STADIUM", fm_stadium, FM_N(fm_stadium) },
    { MENU_KIND_MULTI_VS, FM_LIST, 0, MENU_KIND_STADIUM, SEL_STADIUM_MULTIMAN, 0,
      "MULTI-MAN MELEE", fm_multi, FM_N(fm_multi) },
    { MENU_KIND_VS, FM_HUB, SEL_VS_MELEE, MENU_KIND_MAIN, SEL_MAIN_VS, 0, "VERSUS", fm_vs,
      FM_N(fm_vs) },
    { MENU_KIND_SPECIAL, FM_LIST, 0, MENU_KIND_VS, SEL_VS_SPECIAL, 0, "SPECIAL MELEE",
      fm_special, FM_N(fm_special) },
    { MENU_KIND_TOY, FM_HUB, SEL_TOY_COLLECTION, MENU_KIND_MAIN, SEL_MAIN_TOY, 1,
      "COLLECTION", fm_toy, FM_N(fm_toy) },
    { MENU_KIND_SETTINGS, FM_LIST, 0, MENU_KIND_MAIN, SEL_MAIN_SETTINGS, 0, "SETTINGS",
      fm_settings, FM_N(fm_settings) },
    { MENU_KIND_DATA, FM_HUB, SEL_DATA_RECORDS, MENU_KIND_MAIN, SEL_MAIN_DATA, 0, "DATA",
      fm_data, FM_N(fm_data) },
    { MENU_KIND_RECORDS, FM_LIST, 0, MENU_KIND_DATA, SEL_DATA_RECORDS, 0, "RECORDS",
      fm_records, FM_N(fm_records) },
};

static const FeMenu* fm_find(int kind)
{
    int i;
    for (i = 0; i < FM_N(fm_menus); i++) {
        if (fm_menus[i].kind == kind) {
            return &fm_menus[i];
        }
    }
    return NULL;
}

/* A menu the frontend draws itself (so a native screen returning to it comes here instead). */
static bool fm_replaced(int kind)
{
    return fm_find(kind) != NULL;
}

/* Vanilla's lock/visibility rule for an item. The multi-man list is its own screen in vanilla
 * (mnhyaku.c) and cycles through all six with no check. */
static bool fm_visible(const FeMenu* m, const FeMenuItem* it)
{
    if (m->kind == MENU_KIND_MULTI_VS) {
        return true;
    }
    if (it->sel == SEL_1P_LAB) {
        /* only when the Geno Lab script mod is loaded (gw_script.c) */
        extern int Script_LabAvailable(void);
        return Script_LabAvailable() != 0;
    }
    return mn_80229938((MenuKind) m->kind, it->sel) != false;
}

/* gmmenumode.c onEnter's table: where GM_MENU puts the cursor, by the mode the player came
 * from. Kept identical so the frontend lands where the native menu would have. */
static void fm_position_for(u8 previous_mode, u8* kind, u8* sel)
{
    static const u8 map[][3] = {
        { GM_CLASSIC, MENU_KIND_REG, SEL_REG_CLASSIC },
        { GM_ADVENTURE, MENU_KIND_REG, SEL_REG_ADVENTURE },
        { GM_ALLSTAR, MENU_KIND_REG, SEL_REG_ALLSTAR },
        { GM_CLASSIC_GOVER, MENU_KIND_REG, SEL_REG_CLASSIC },
        { GM_ADVENTURE_GOVER, MENU_KIND_REG, SEL_REG_ADVENTURE },
        { GM_ALLSTAR_GOVER, MENU_KIND_REG, SEL_REG_ALLSTAR },
        { GM_EVENT, MENU_KIND_EVENT, 0 },
        { GM_TARGET_TEST, MENU_KIND_STADIUM, SEL_STADIUM_TARGET },
        { GM_10MAN_VS, MENU_KIND_MULTI_VS, SEL_MULTI_VS_10MAN },
        { GM_100MAN_VS, MENU_KIND_MULTI_VS, SEL_MULTI_VS_100MAN },
        { GM_3MIN_VS, MENU_KIND_MULTI_VS, SEL_MULTI_VS_3MIN },
        { GM_15MIN_VS, MENU_KIND_MULTI_VS, SEL_MULTI_VS_15MIN },
        { GM_ENDLESS_VS, MENU_KIND_MULTI_VS, SEL_MULTI_VS_ENDLESS },
        { GM_CRUEL_VS, MENU_KIND_MULTI_VS, SEL_MULTI_VS_CRUEL },
        { GM_HOME_RUN_CONTEST, MENU_KIND_STADIUM, SEL_STADIUM_HOMERUN },
        { GM_TRAINING, MENU_KIND_1P, SEL_1P_TRAINING },
        { GM_VS, MENU_KIND_VS, SEL_VS_MELEE },
        { GM_TOURNAMENT, MENU_KIND_VS, SEL_VS_TOURNAMENT },
        { GM_CAMERA_MODE, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_CAMERA },
        { GM_STAMINA_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_STAMINA },
        { GM_SUPER_SUDDEN_DEATH_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_SUDDEN_DEATH },
        { GM_GIANT_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_GIANT },
        { GM_TINY_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_TINY },
        { GM_INVISIBLE_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_INVISIBLE },
        { GM_SLOMO_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_SLOMO },
        { GM_LIGHTNING_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_LIGHTNING },
        { GM_CAMERA_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_FIXED_CAMERA },
        { GM_SINGLE_BUTTON_VS, MENU_KIND_SPECIAL, SEL_SPECIAL_VS_SINGLE_BUTTON },
        { GM_TOY_GALLERY, MENU_KIND_TOY, SEL_TOY_GALLERY },
        { GM_TOY_LOTTERY, MENU_KIND_TOY, SEL_TOY_LOTTERY },
        { GM_TOY_COLLECTION, MENU_KIND_TOY, SEL_TOY_COLLECTION },
        { GM_MENU, MENU_KIND_SETTINGS, SEL_SETTINGS_LANG },
        { GM_LAB, MENU_KIND_1P, SEL_1P_LAB },
    };
    int i;
    GameRules* rules = gmMainLib_GetGameRules();
    if (fm_back_kind >= 0) {
        /* backing out of a SETTINGS page: the list, on its item */
        *kind = (u8) fm_back_kind;
        *sel = (u8) fm_back_sel;
        fm_back_kind = -1;
        if (rules->force_main_menu == 0) {
            return;
        }
    }
    if (fm_back_to_online_item) {
        /* backing out of ONLINE (or its rooms): the VS hub, on ONLINE */
        fm_back_to_online_item = false;
        if (previous_mode == GM_VS && rules->force_main_menu == 0) {
            *kind = MENU_KIND_VS;
            *sel = SEL_VS_ONLINE;
            return;
        }
    }
    if (rules->force_main_menu != 0) {
        rules->force_main_menu = 0;
        *kind = MENU_KIND_MAIN;
        *sel = SEL_MAIN_1P;
        return;
    }
    if (previous_mode == GM_CHALLENGER_APPROACH) {
        previous_mode = gm_801737D8();
    }
    for (i = 0; i < FM_N(map); i++) {
        if (map[i][0] == previous_mode) {
            *kind = map[i][1];
            *sel = map[i][2];
            return;
        }
    }
    *kind = MENU_KIND_MAIN;
    *sel = SEL_MAIN_1P;
}

/* ---- the menus scene state ---------------------------------------------------------------- */

#define FM_MAX_VIS 12
#define FM_FADE 8

static struct {
    bool active;       ///< this GS_FRONTEND scene is the menu tree
    u8 start_kind, start_sel; ///< where the next menus scene opens
    bool native_req;          ///< leaving for GM_MENU to run a native screen
    u8 native_kind, native_sel;
    int demo;                 ///< MELEE_FE_HUBDEMO frame, -1 off
    const FeMenu* menu;
    int vis[FM_MAX_VIS];      ///< item indices in the visible (unlocked) order
    int nvis;
    int tile_of[FM_MAX_VIS];  ///< item slot -> fp tile index
    int cursor;               ///< slot in vis
    int frames;               ///< frames since this screen opened
    int ready_at;             ///< input accepted from this frame
    int leave_at;             ///< the pending action happens at this fp frame, 0 none
    int pend_kind;            ///< 0 none, 1 open menu, 2 game mode, 3 native, 4 match setup
    int pend_a, pend_b;
    int fade;                 ///< scene fade, FM_FADE = black
    bool fading_out;
    u8 leave_mode;
    int canvas_frames;
    char fit[FM_MAX_VIS + 2][40];
} fm;

/* ---- generated layouts (the kit's chrome and list template, the hub's tiles) -------------- */

/* SisLib fallback metrics (only when the atlas is missing): 640x480 units per unit of
 * font_size - capitals are ~22 units tall at font_size 1. */
#define FM_SIS_CAP 22.0F
#define FM_SIS_ADV 0.78F

static int fm_section_of(int kind)
{
    switch (kind) {
    case MENU_KIND_1P:
    case MENU_KIND_REG:
    case MENU_KIND_STADIUM:
    case MENU_KIND_MULTI_VS:
    case MENU_KIND_EVENT:
        return KS_SOLO;
    case MENU_KIND_TOY:
        return KS_COLLECTION;
    case MENU_KIND_SETTINGS:
        return KS_OPTIONS;
    case MENU_KIND_DATA:
    case MENU_KIND_RECORDS:
        return KS_DATA;
    default:
        return KS_VERSUS; /* VS, Special Melee, and the main menu's own backdrop */
    }
}

static void fm_shear_rect(float x0, float y0, float x1, float y1, float v[4][2])
{
    float s = fp.shear_s, y = fp.shear_y0;
    v[0][0] = x0 + (y - y0) * s; v[0][1] = y0;
    v[1][0] = x1 + (y - y0) * s; v[1][1] = y0;
    v[2][0] = x1 + (y - y1) * s; v[2][1] = y1;
    v[3][0] = x0 + (y - y1) * s; v[3][1] = y1;
}

static void fm_rgb(float* dst, const float* src)
{
    dst[0] = src[0];
    dst[1] = src[1];
    dst[2] = src[2];
}

static void fm_tok(const char* tok, int section, float* rgb)
{
    if (!kit_colour(tok, section, rgb)) {
        rgb[0] = rgb[1] = rgb[2] = 1.0F;
    }
}

static int fm_flat(const char* id, u8 role, int tile, float x0, float y0, float x1, float y1,
                   const float* rgb)
{
    int q = fp_add_quad(id, FPK_FLAT, role, tile);
    if (q < 0) {
        return -1;
    }
    fm_shear_rect(x0, y0, x1, y1, fp.q[q].v);
    if (rgb != NULL) {
        fp.q[q].has_colour = true;
        fm_rgb(fp.q[q].rgb, rgb);
    }
    return q;
}

static int fm_textured(const char* id, u8 role, int tile, const char* tex, float x0, float y0,
                       float x1, float y1, const float* rgb)
{
    int t = fp_tex(tex), q;
    if (t < 0) {
        return -1;
    }
    q = fp_add_quad(id, FPK_TEX, role, tile);
    if (q < 0) {
        return -1;
    }
    fm_shear_rect(x0, y0, x1, y1, fp.q[q].v);
    fp.q[q].tex = (s8) t;
    fp.q[q].has_colour = true;
    fm_rgb(fp.q[q].rgb, rgb);
    return q;
}

/* A text quad: role from the kit's type scale, left-aligned at an unsheared baseline anchor,
 * fitted to max_w by the manifest's rule. Returns the quad; its advance is fm.last_w. */
static float fm_last_w;

static int fm_text(const char* id, u8 role, int tile, const char* ff_name, float ax, float ay,
                   const char* s, const float* rgb, float max_w)
{
    int q = fp_add_quad(id, FPK_TEXT, role, tile);
    FpQuad* d;
    float w, asc, desc;
    if (q < 0) {
        return -1;
    }
    d = &fp.q[q];
    d->has_colour = true;
    fm_rgb(d->rgb, rgb);
    d->ax = ax;
    d->ay = ay;
    d->ff_role = (s8) ff_role(ff_name);
    if (d->ff_role >= 0) {
        d->ff_role = (s8) ff_fit(d->ff_role, s, max_w, d->textbuf, (int) sizeof d->textbuf);
        d->text = d->textbuf;
        w = ff_width(d->ff_role, d->text);
        asc = ff.r[d->ff_role].ascent;
        desc = ff.r[d->ff_role].descent;
        d->text_h = ff.r[d->ff_role].cap;
    } else {
        /* SisLib: its size from the role's nominal size, shrunk to fit */
        float h = 0.66F * (strcmp(ff_name, "hero") == 0    ? 44.0F
                           : strcmp(ff_name, "title") == 0 ? 24.0F
                           : strcmp(ff_name, "label") == 0 ? 20.0F
                           : strcmp(ff_name, "row") == 0   ? 16.0F
                                                           : 14.0F);
        fp_copy(d->textbuf, s, (int) sizeof d->textbuf);
        d->text = d->textbuf;
        w = (float) strlen(s) * h * FM_SIS_ADV;
        if (max_w > 0.0F && w > max_w) {
            h *= max_w / w;
            w = max_w;
        }
        d->text_h = h;
        asc = h;
        desc = h * 0.3F;
    }
    fm_shear_rect(ax, ay - asc, ax + w, ay + desc, d->v);
    fm_last_w = w;
    return q;
}

static void fm_default_states(int section)
{
    float gold[3], gold_dk[3], ink[3], bone[3];
    fm_tok("gold", section, gold);
    fm_tok("gold_dk", section, gold_dk);
    fm_tok("ink", section, ink);
    fm_tok("bone", section, bone);
    fm_rgb(fp.sel_face, gold);
    fm_rgb(fp.sel_icon, gold_dk);
    fm_rgb(fp.sel_label, ink);
    fm_rgb(fp.sel_plate, gold_dk);
    fm_rgb(fp.ng_label, bone);
    fm_tok("@face", section, fp.ng_face);
    fm_tok("@face_hi", section, fp.ng_icon);
    fm_tok("@bg", section, fp.clear);
    fp.sel_offset[0] = fp.sel_offset[1] = -5.0F;
}

/* One tile or row: plate, face, icon, label, and its description in the strip. */
static void fm_tile(int slot, const FeMenuItem* it, int section, int rank, float x0, float y0,
                    float x1, float y1, bool hero, bool row)
{
    FpTile* t = &fp.tile[fp.nt];
    char id[FP_ID], qid[FP_ID];
    float h = y1 - y0, w = x1 - x0;
    float pad = hero ? 16.0F : 10.0F, icon = 0.0F, ink[3], bone[3];
    const char* role;
    fm_tok("ink", section, ink);
    fm_tok("bone", section, bone);
    sprintf(id, "t%d", slot);
    fp_copy(t->id, id, FP_ID);
    t->nextra = 0;
    t->rank = rank;
    fm_tok("@face", section, t->ng_face);
    fm_tok("@face_hi", section, t->ng_icon);
    fm.tile_of[slot] = fp.nt;
    sprintf(qid, "%s_plate", id);
    t->q_plate = fm_flat(qid, FPR_PLATE, fp.nt, x0, y0, x1, y1, NULL);
    sprintf(qid, "%s_face", id);
    t->q_face = fm_flat(qid, FPR_FACE, fp.nt, x0, y0, x1, y1, t->ng_face);
    t->q_icon = -1;
    if (!row && it->icon != NULL) {
        icon = hero ? 128.0F : 64.0F;
        if (icon > h - 2 * pad - (hero ? 40.0F : 0.0F)) {
            icon = h - 2 * pad - (hero ? 40.0F : 0.0F);
        }
        if (icon > 8.0F) {
            sprintf(qid, "%s_icon", id);
            t->q_icon = fm_textured(qid, FPR_ICON, fp.nt, it->icon, x1 - pad - icon, y0 + pad,
                                    x1 - pad, y0 + pad + icon, t->ng_icon);
        }
        if (t->q_icon < 0) {
            icon = 0.0F; /* missing art: the label takes the room (logged by fp_tex) */
        }
    }
    sprintf(qid, "%s_label", id);
    if (row) {
        float max_w = kit.row_label_max > w - 2 * kit.row_label_x ? kit.row_label_max
                                                                   : w - 2 * kit.row_label_x;
        t->q_label = fm_text(qid, FPR_LABEL, fp.nt, kit.row_role, x0 + kit.row_label_x,
                             y0 + kit.row_label_base, it->label, bone, max_w);
    } else {
        /* the hero's label in the hero role; the others in label, or row on a short tile */
        role = hero ? "hero" : h >= 48.0F ? "label" : "row";
        t->q_label =
            fm_text(qid, FPR_LABEL, fp.nt, role, x0 + pad + 2, y1 - pad, it->label, bone,
                    w - 2 * pad - 4 - (h < 2 * pad + icon + 28.0F ? icon + 6.0F : 0.0F));
    }
    sprintf(qid, "%s_desc", id);
    t->q_desc = fm_text(qid, FPR_DESC, fp.nt, kit.desc_role, kit.desc_anchor[0],
                        kit.desc_anchor[1], it->desc, bone, kit.desc_max);
    fp.nt++;
}

/* The chrome (chrome_layout.json): the section's backdrop and band rhythm, the header slab
 * sized by its title, the breadcrumb (`crumbs`, root first) with the section icon at its root,
 * the description strip, and the footer hints (glyph + text; A in `ok`, B in `danger`, the
 * kit's suggested tints, anything else bone). */
static void fm_chrome_with(const char* title, const char* const* crumbs, int ncrumbs,
                           int section, int nhints, const char* const* hglyph,
                           const char* const* htext)
{
    const KitSection* ks = &kit.sec[section];
    float ink[3], gold[3], muted[3], bone[3], ok[3], danger[3];
    int i, q;
    float x, slab_x1;
    char id[FP_ID];
    fm_tok("ink", section, ink);
    fm_tok("gold", section, gold);
    fm_tok("muted", section, muted);
    fm_tok("bone", section, bone);
    fm_tok("ok", section, ok);
    fm_tok("danger", section, danger);
    for (i = 0; i < ks->nbands; i++) {
        sprintf(id, "band_%d", i);
        fm_flat(id, FPR_DECOR, -1, ks->band_x[i], -40, ks->band_x[i] + ks->band_w[i], 520,
                ks->band);
    }
    /* header: the title decides the slab's width */
    q = fm_text("header_title", FPR_TITLE, -1, kit.title_role, kit.title_anchor[0],
                kit.title_anchor[1], title, ink, kit.title_max);
    slab_x1 = kit.title_anchor[0] + fm_last_w + 20.0F;
    fm_flat("header_rule", FPR_CHROME, -1, slab_x1 - 4, kit.rule[1], kit.rule[2], kit.rule[3],
            ink);
    fm_flat("header_slab", FPR_CHROME, -1, kit.slab[0], kit.slab[1], slab_x1, kit.slab[3], gold);
    /* the title draws over its slab */
    if (q >= 0) {
        FpQuad tmp = fp.q[q];
        int n = fp.nq - 1;
        for (i = q; i < n; i++) {
            fp.q[i] = fp.q[i + 1];
        }
        fp.q[n] = tmp;
        fp.q[n].text = fp.q[n].textbuf; /* it pointed into its old slot */
    }
    {
        char icon[FP_ID];
        sprintf(icon, "ico_%s", kit_section_names[section]);
        fm_textured("crumb_icon", FPR_CHROME, -1, icon, kit.crumb_icon[0], kit.crumb_icon[1],
                    kit.crumb_icon[2], kit.crumb_icon[3], ks->face_hi);
    }
    x = kit.crumb_anchor[0];
    for (i = 0; i < ncrumbs; i++) {
        sprintf(id, "crumb_%d", i);
        fm_text(id, FPR_CHROME, -1, kit.crumb_role, x, kit.crumb_anchor[1], crumbs[i],
                i == ncrumbs - 1 ? bone : muted, kit.crumb_anchor[0] + kit.crumb_max - x);
        x += fm_last_w;
        if (i < ncrumbs - 1) {
            sprintf(id, "crumb_bar_%d", i);
            fm_flat(id, FPR_CHROME, -1, x + 6, kit.crumb_anchor[1] - 10, x + 8,
                    kit.crumb_anchor[1], gold);
            x += 14;
        }
    }
    fm_flat("desc_strip", FPR_CHROME, -1, kit.desc_strip[0], kit.desc_strip[1], kit.desc_strip[2],
            kit.desc_strip[3], ink);
    fm_flat("footer_strip", FPR_CHROME, -1, kit.footer[0], kit.footer[1], kit.footer[2],
            kit.footer[3], ink);
    fm_flat("footer_mark", FPR_CHROME, -1, kit.footer_mark[0], kit.footer_mark[1],
            kit.footer_mark[2], kit.footer_mark[3], gold);
    /* hints: glyph, 4px, text, 12px (each one a button for the mouse) */
    x = kit.footer[0] + 24;
    fms.nhint = 0;
    for (i = 0; i < nhints; i++) {
        float hx0 = x - 4;
        int gq;
        int t = fp_tex(hglyph[i]);
        float gw = kit.hint_glyph_h;
        const float* tint = strcmp(hglyph[i], "glyph_a") == 0   ? ok
                            : strcmp(hglyph[i], "glyph_b") == 0 ? danger
                                                                : bone;
        if (t >= 0 && fp.tex[t].t.h > 0) {
            gw = kit.hint_glyph_h * (float) fp.tex[t].t.w / (float) fp.tex[t].t.h;
        }
        sprintf(id, "hint_%d_glyph", i);
        gq = fm_textured(id, FPR_CHROME, -1, hglyph[i], x, kit.hint_glyph_y0, x + gw,
                         kit.hint_glyph_y0 + kit.hint_glyph_h, tint);
        x += gw + 4;
        sprintf(id, "hint_%d_text", i);
        fm_text(id, FPR_CHROME, -1, kit.hint_role, x, kit.hint_anchor_y, htext[i], bone,
                kit.hint_max);
        x += fm_last_w + 12;
        fms_hint_add(gq, hglyph[i], hx0, x - 6, kit.footer[1], kit.footer[3]);
    }
}

static void fm_chrome(const FeMenu* m, int section)
{
    static const char* const hg[2] = { "glyph_a", "glyph_b" };
    static const char* const ht[2] = { "Select", "Back" };
    const FeMenu* path[6];
    const char* crumbs[6];
    int np = 0, i;
    for (path[np++] = m; np < 6 && path[np - 1]->back_kind != 0xFF; np++) {
        path[np] = fm_find(path[np - 1]->back_kind);
        if (path[np] == NULL || path[np]->kind == MENU_KIND_MAIN) {
            break;
        }
    }
    for (i = 0; i < np; i++) {
        crumbs[i] = path[np - 1 - i]->title;
    }
    fm_chrome_with(m->title, crumbs, np, section, 2, hg, ht);
}

/* THE HUB LAYOUT RULE (the engine's stand-in until brief section 2 delivers the art's): one hero
 * tile the full height of the cluster, the others stacked in a column beside it with heights
 * stepping down by 0.76 each (the prototype's 144 -> 110 -> 82 for three is this rule), 8px
 * gutters; the cluster sits between the kit's breadcrumb and description strip. LISTS are the
 * kit's list template: rows at its pitch from rows_top, `visible` at a time, scrolling. */
static void fm_build(const FeMenu* m, int section)
{
    int n = fm.nvis, k, i;
    float X0 = 90, X1 = 556, Y0 = 82, Y1 = 390, HX1 = 352, SX0 = 360, G = 8;
    int order[FM_MAX_VIS];
    float ink[3];
    fm_tok("ink", section, ink);
    fm_chrome(m, section);
    if (m->style == FM_LIST) {
        for (i = 0; i < n; i++) {
            float y = kit.rows_top + i * kit.pitch;
            fm_tile(i, &m->items[fm.vis[i]], section, i, kit.row_x0, y, kit.row_x1,
                    y + kit.row_h, false, true);
        }
        return;
    }
    fm_flat("backplate", FPR_CHROME, -1, X0 - 6, Y0 - 6, X1 + 6, Y1 + 6, ink);
    k = 0;
    for (i = 0; i < n; i++) {
        if (m->items[fm.vis[i]].sel == m->hero) {
            order[k++] = i;
        }
    }
    if (k == 0 && n > 0) {
        order[k++] = 0;
    }
    for (i = 0; i < n; i++) {
        if (i != order[0]) {
            order[k++] = i;
        }
    }
#define FM_TILE_SECTION(slot)                                                                 \
    (m->kind == MENU_KIND_MAIN ? fm_section_of(m->items[fm.vis[slot]].arg) : section)
    if (n == 1) {
        fm_tile(order[0], &m->items[fm.vis[order[0]]], FM_TILE_SECTION(order[0]), 0, X0, Y0, X1,
                Y1, true, false);
        return;
    }
    fm_tile(order[0], &m->items[fm.vis[order[0]]], FM_TILE_SECTION(order[0]), 0, X0, Y0, HX1, Y1,
            true, false);
    {
        int s = n - 1;
        float avail = Y1 - Y0 - G * (s - 1), wsum = 0.0F, wgt = 1.0F, y = Y0;
        for (i = 0; i < s; i++, wgt *= 0.76F) {
            wsum += wgt;
        }
        wgt = 1.0F;
        for (i = 0; i < s; i++, wgt *= 0.76F) {
            float h = (float) (int) (avail * wgt / wsum + 0.5F);
            if (i == s - 1) {
                h = Y1 - y;
            }
            fm_tile(order[i + 1], &m->items[fm.vis[order[i + 1]]], FM_TILE_SECTION(order[i + 1]),
                    i + 1, SX0, y, X1, y + h, false, false);
            y += h + G;
        }
    }
#undef FM_TILE_SECTION
}

/* ---- SisLib text, only when the atlas is missing ------------------------------------------ */

static void fm_texts_create(void)
{
    int i;
    for (i = 0; i < fp.nq; i++) {
        FpQuad* q = &fp.q[i];
        GXColor c;
        if (q->kind != FPK_TEXT || q->ff_role >= 0) {
            continue;
        }
        c.r = fp_u8(q->rgb[0]);
        c.g = fp_u8(q->rgb[1]);
        c.b = fp_u8(q->rgb[2]);
        c.a = 255;
        q->sis = HSD_SisLib_803A6754(0, fe.canvas);
        if (q->sis == NULL) {
            continue;
        }
        q->sis->pos_x = q->v[3][0];
        q->sis->pos_y = q->v[0][1];
        q->sis->pos_z = 0.0F;
        q->sis->font_size.x = q->sis->font_size.y = q->text_h / FM_SIS_CAP;
        q->sis->default_kerning = 1;
        q->sis->default_alignment = 0;
        q->sis_entry = HSD_SisLib_803A6B98(q->sis, 0.0F, 0.0F, q->text);
        HSD_SisLib_803A74F0(q->sis, q->sis_entry, &c);
        q->sis_col = c;
        q->sis->hidden = 1;
    }
}

static void fm_texts_update(void)
{
    int i;
    for (i = 0; i < fp.nq; i++) {
        FpQuad* q = &fp.q[i];
        GXColor c;
        if (q->kind != FPK_TEXT || q->sis == NULL) {
            continue;
        }
        q->sis->hidden = q->wa < 0.02F;
        q->sis->pos_x = (q->w[0][0] + q->w[3][0]) * 0.5F;
        q->sis->pos_y = q->w[0][1] - q->text_h * 0.18F * q->wscale;
        q->sis->font_size.x = q->sis->font_size.y = q->text_h * q->wscale / FM_SIS_CAP;
        c.r = fp_u8(q->wrgb[0]);
        c.g = fp_u8(q->wrgb[1]);
        c.b = fp_u8(q->wrgb[2]);
        c.a = fp_u8(q->wa);
        if (c.r != q->sis_col.r || c.g != q->sis_col.g || c.b != q->sis_col.b) {
            HSD_SisLib_803A74F0(q->sis, q->sis_entry, &c);
        }
        q->sis_col = c;
        q->sis->text_color.a = c.a;
        q->sis->active_color.a = c.a;
    }
}

static void fm_texts_destroy(void)
{
    int i;
    for (i = 0; i < fp.nq; i++) {
        if (fp.q[i].kind == FPK_TEXT && fp.q[i].sis != NULL) {
            HSD_SisLib_803A5CC4(fp.q[i].sis);
            fp.q[i].sis = NULL;
        }
    }
}

/* ---- opening and closing a screen --------------------------------------------------------- */

static void fm_close(void)
{
    fm_texts_destroy();
    fp_free_textures();
    fp.ok = false;
}

static int fm_rows_joint = -1;
static int fm_scroll;

static void fm_open(const FeMenu* m, int sel)
{
    void *lt, *lp, *mt, *mp;
    const FjNode* L;
    const FjNode* M;
    int i, section = fm_section_of(m->kind);
    kit_load();
    ff_load();
    fm.menu = m;
    fm.nvis = 0;
    for (i = 0; i < m->n && fm.nvis < FM_MAX_VIS; i++) {
        if (fm_visible(m, &m->items[i])) {
            fm.vis[fm.nvis++] = i;
        }
    }
    fm.cursor = 0;
    for (i = 0; i < fm.nvis; i++) {
        if (m->items[fm.vis[i]].sel == sel) {
            fm.cursor = i;
        }
    }
    fp_reset();
    L = fj_load("hub_layout.json", &lt, &lp);
    if (L != NULL) {
        fp_read_states(L); /* the shear, the selection offset and the hub's ng numbers */
    }
    if (lt != NULL) HSD_Free(lt);
    if (lp != NULL) HSD_Free(lp);
    fm_default_states(section);
    if (m->style == FM_LIST) {
        fp.sel_offset[0] = kit.row_sel_offset[0];
        fp.sel_offset[1] = kit.row_sel_offset[1];
        fm_rows_joint = fp_add_joint("rows", -1, NULL); /* scrolls the rows; first in order */
    } else {
        fm_rows_joint = -1;
    }
    fp.recolour = true;
    fm_build(m, section);
    M = fj_load("hub_motion.json", &mt, &mp);
    if (M != NULL) {
        fp_read_motion(M, true);
    }
    if (mt != NULL) HSD_Free(mt);
    if (mp != NULL) HSD_Free(mp);
    if (m->style == FM_LIST) {
        kit_read_motion(section);
    }
    if (M == NULL) {
        /* no motion file: plain joints so the selection still reads (lift + colours) */
        for (i = 0; i < fp.nt; i++) {
            char nm[FP_ID];
            FpTile* t = &fp.tile[i];
            sprintf(nm, "lift_%s", t->id);
            t->j_lift = fp_add_joint(nm, -1, NULL);
            t->j_tile = -1;
            fp_joint_draws(t->j_lift, fp.q[t->q_face].id);
            if (t->q_icon >= 0) fp_joint_draws(t->j_lift, fp.q[t->q_icon].id);
            fp_joint_draws(t->j_lift, fp.q[t->q_label].id);
            sprintf(nm, "plate_%s", t->id);
            t->j_plate = fp_add_joint(nm, -1, NULL);
            fp_joint_draws(t->j_plate, fp.q[t->q_plate].id);
            sprintf(nm, "desc_%s", t->id);
            t->j_desc = fp_add_joint(nm, -1, NULL);
            fp_joint_draws(t->j_desc, fp.q[t->q_desc].id);
        }
    }
    /* the kit's chrome rides the header and footer joints the hub motion slides */
    {
        int hj = fp_find_joint("header"), fj2 = fp_find_joint("footer");
        for (i = 0; i < fp.nq; i++) {
            const char* id = fp.q[i].id;
            if (fp.q[i].tile >= 0 || fp.q[i].joint >= 0) {
                continue;
            }
            if (strncmp(id, "crumb", 5) == 0 || strncmp(id, "header", 6) == 0) {
                fp.q[i].joint = (s8) hj;
            } else if (strncmp(id, "hint", 4) == 0 || strncmp(id, "desc_strip", 10) == 0 ||
                       strncmp(id, "footer", 6) == 0)
            {
                fp.q[i].joint = (s8) fj2;
            }
        }
    }
    if (fm_rows_joint >= 0) {
        for (i = 0; i < fp.nt; i++) {
            if (fp.tile[i].j_tile >= 0) {
                fp.j[fp.tile[i].j_tile].parent = (s8) fm_rows_joint;
            }
        }
    }
    /* the atlas pages this screen sets text in, loaded now rather than mid-draw */
    for (i = 0; i < fp.nq; i++) {
        if (fp.q[i].kind == FPK_TEXT && fp.q[i].ff_role >= 0) {
            int p;
            for (p = 0; p < ff.r[fp.q[i].ff_role].npages; p++) {
                fp_tex(ff.r[fp.q[i].ff_role].page[p]);
            }
        }
    }
    fp_reset_channels();
    fp.ok = true;
    fp.frame = 0;
    fm_scroll = 0;
    if (m->style == FM_LIST && fm.cursor >= kit.visible - 1) {
        fm_scroll = fm.cursor - (kit.visible - 2);
        if (fm_scroll > fm.nvis - kit.visible) {
            fm_scroll = fm.nvis - kit.visible;
        }
        fp_set(fp_jch(fm_rows_joint, FJC_TRA_Y), 0, -kit.pitch * fm_scroll);
    }
    if (fm.nvis > 0) {
        fp_select_initial(fm.tile_of[fm.cursor], 0);
    }
    fp_screen_in(0);
    fm_texts_create();
    fm.frames = 0;
    fm.ready_at = 8;
    fm.leave_at = 0;
    fm.pend_kind = 0;
    OSReport("frontend: menu %s (kind %d, %s), %d of %d items shown, cursor on \"%s\"%s%s\n",
             m->title, m->kind, m->style == FM_HUB ? "hub" : "list", fm.nvis, m->n,
             fm.nvis > 0 ? m->items[fm.vis[fm.cursor]].label : "-",
             fp.motion_ok ? "" : " - no hub_motion.json, static",
             ff.state == 1 ? "" : " - no font atlas, SisLib text");
}

/* Lists: keep the cursor at least one row from either edge of the visible window, sliding the
 * rows as the kit's scroll event does (7 frames, spline). Rows outside the window are hidden
 * after evaluation (fm_clip_rows). */
static void fm_list_scroll(void)
{
    int want = fm_scroll;
    FpKey k[2];
    if (fm.menu->style != FM_LIST || fm.nvis <= kit.visible || fm_rows_joint < 0) {
        return;
    }
    if (fm.cursor < want + 1) {
        want = fm.cursor - 1;
    }
    if (fm.cursor > want + kit.visible - 2) {
        want = fm.cursor - (kit.visible - 2);
    }
    if (want < 0) {
        want = 0;
    }
    if (want > fm.nvis - kit.visible) {
        want = fm.nvis - kit.visible;
    }
    if (want == fm_scroll) {
        return;
    }
    fm_scroll = want;
    k[0].frame = 0;
    k[0].interp = FPI_SPL;
    k[0].from_current = 1;
    k[0].v = 0.0F;
    k[0].slope = 0.0F;
    k[1] = k[0];
    k[1].frame = 6;
    k[1].from_current = 0;
    k[1].v = -kit.pitch * (float) fm_scroll;
    fp_start(fp_jch(fm_rows_joint, FJC_TRA_Y), fp.frame, k, 2);
}

static void fm_clip_rows(void)
{
    int i;
    float top = kit.rows_top - kit.pitch * 0.5F;
    float bot = kit.rows_top + (kit.visible - 1) * kit.pitch + kit.pitch * 0.5F;
    if (fm.menu == NULL || fm.menu->style != FM_LIST || fm.nvis <= kit.visible) {
        return;
    }
    for (i = 0; i < fp.nt; i++) {
        FpTile* t = &fp.tile[i];
        float y = fm_rows_joint >= 0 ? fp.q[t->q_plate].v[0][1] +
                                           fp_value(fp_jch(fm_rows_joint, FJC_TRA_Y), fp.frame)
                                     : 0.0F;
        if (y < top || y > bot) {
            fp.q[t->q_plate].wa = fp.q[t->q_face].wa = 0.0F;
            if (t->q_label >= 0) {
                fp.q[t->q_label].wa = 0.0F;
            }
        }
    }
}

/* ---- the hub demo (MELEE_FE_HUBDEMO): out_hub's own screen and pipeline/hub_motion.py's
 * preview script, so captures line up with the preview sheets frame for frame ---- */

static int fm_demo_tile(const char* id)
{
    int i;
    for (i = 0; i < fp.nt; i++) {
        if (strcmp(fp.tile[i].id, id) == 0) {
            return i;
        }
    }
    return -1;
}

static void fm_demo_open(void)
{
    void *lt, *lp, *mt, *mp;
    const FjNode* L;
    const FjNode* M;
    fp_reset();
    L = fj_load("hub_layout.json", &lt, &lp);
    if (L != NULL) {
        fp_build_from_layout(L);
    }
    M = fj_load("hub_motion.json", &mt, &mp);
    if (M != NULL) {
        fp_read_motion(M, false);
    }
    if (lt != NULL) HSD_Free(lt);
    if (lp != NULL) HSD_Free(lp);
    if (mt != NULL) HSD_Free(mt);
    if (mp != NULL) HSD_Free(mp);
    fp_reset_channels();
    fp.ok = L != NULL;
    fm.menu = NULL;
    fm.frames = 0;
    OSReport("frontend: hub demo - %d quads, %d tiles, %d joints, %d events, freeze at %d\n",
             fp.nq, fp.nt, fp.nj, fp.nev, fm.demo);
}

/* hub_motion.py main(): select versus + screen_in at 0; solo 40, collection 74, options 108,
 * versus 142; confirm 176; screen_out 190; the demo ends at 224. Replayed from scratch each
 * loop so a frozen frame is exactly the script's. */
static void fm_demo_frame(int f)
{
    static const struct { int f; const char* id; } script[] = {
        { 40, "solo" }, { 74, "collection" }, { 108, "options" }, { 142, "versus" },
    };
    int i;
    if (!fp.ok) {
        return;
    }
    fp_reset_channels();
    fp.sel = fp.prev = -1;
    fp_select_initial(fm_demo_tile("versus"), 0);
    fp_screen_in(0);
    for (i = 0; i < 4; i++) {
        if (script[i].f <= f) {
            fp_cursor_move(fm_demo_tile(script[i].id), script[i].f);
        }
    }
    if (176 <= f) {
        fp_event(fp.confirm_event, fp.sel, 176, -1);
    }
    if (190 <= f) {
        fp_screen_out(190);
    }
    fp_evaluate(f);
}

/* ---- actions ------------------------------------------------------------------------------ */

static void fm_mirror_flow(void)
{
    /* MenuFlow mirrors the frontend's position, so the scene trace ("scene: cursor menu") and
       anything that reads it see the same (kind, selection) the native menu would have. */
    if (fm.menu != NULL && fm.nvis > 0) {
        mn_804A04F0.cur_menu = fm.menu->kind;
        mn_804A04F0.hovered_selection = fm.menu->items[fm.vis[fm.cursor]].sel;
    }
}

static void fm_leave_scene(u8 mode)
{
    fm.leave_mode = mode;
    fm.fading_out = true;
}

/* The first boot opens SETTINGS > CONTROLS once, as its menu row would (page 2, FSP_CONTROLS):
 * a controller to plug in and test, the dead zone, and how to play online. Onboard_Pending says
 * when (settings.cfg has no onboarded=1, and nobody automated the run); true = it was queued. */
extern int Onboard_Pending(void);
extern void Onboard_Done(void);
static bool fm_first_boot(void)
{
    static bool done;
    if (done || !Onboard_Pending()) {
        return false;
    }
    done = true;
    Onboard_Done();
    OSReport("frontend: first boot -> SETTINGS > CONTROLS\n");
    fm.pend_kind = 6;
    fm.pend_a = 2; /* FSP_CONTROLS (gmfrontend_settings.inc, included after this file) */
    fm.leave_at = fp.frame + 1;
    return true;
}

static void fm_do_pending(void)
{
    const FeMenu* m;
    switch (fm.pend_kind) {
    case 1:
        m = fm_find(fm.pend_a);
        if (m != NULL) {
            fm_close();
            fm_open(m, fm.pend_b);
            fm_mirror_flow();
        }
        return;
    case 2:
        OSReport("frontend: -> game mode %d\n", fm.pend_a);
        fe.reported = GM_MENU;
        fm_leave_scene((u8) fm.pend_a);
        return;
    case 3:
        OSReport("frontend: -> native screen (kind %d, selection %d) in GM_MENU\n", fm.pend_a,
                 fm.pend_b);
        fm.native_req = true;
        fm.native_kind = (u8) fm.pend_a;
        fm.native_sel = (u8) fm.pend_b;
        fe.reported = GM_MENU;
        fm_leave_scene(GM_MENU);
        return;
    case 4:
        OSReport("frontend: -> MATCH SETUP, then GM_VS\n");
        fe_match_setup_from_menus();
        return;
    case 5:
        OSReport("frontend: -> ONLINE\n");
        fe_online_from_menus();
        return;
    case 6:
        OSReport("frontend: -> SETTINGS page %d\n", fm.pend_a);
        fe_settings_from_menus(fm.pend_a);
        return;
    }
}

static int fm_port_pressed(void)
{
    return mn_802295AC(); /* the port that pressed A, as mn_8022C7CC_inline finds it */
}

static void fm_confirm(void)
{
    const FeMenu* m = fm.menu;
    const FeMenuItem* it = &m->items[fm.vis[fm.cursor]];
    if (!(it->flags & FMF_NOSFX)) {
        sfxForward();
    }
    if (m->port_all || (it->flags & FMF_PORT)) {
        gm_801677E8((s8) fm_port_pressed());
    }
    if (it->act == FA_NATIVE && m->kind == MENU_KIND_DATA && it->sel == SEL_DATA_SOUND) {
        /* the Sound Test think's own first two calls: stop the effects and the music */
        lbAudioAx_80023694();
        lbAudioAx_800236DC();
    }
    OSReport("frontend: confirm \"%s\" on %s\n", it->label, m->title);
    switch (it->act) {
    case FA_SUB:
        fm.pend_kind = 1;
        fm.pend_a = it->arg;
        fm.pend_b = 0; /* vanilla enters every submenu on its first item */
        if (it->arg == MENU_KIND_SETTINGS) {
            fm.pend_b = 0x41; /* ours starts on the port's first page (VIDEO) */
        }
        break;
    case FA_MODE:
        fm.pend_kind = 2;
        fm.pend_a = it->arg;
        if (it->sel == SEL_1P_LAB) {
            /* Training, with the Lab asked for: the Lab script turns itself on in the match */
            extern void Script_LabRequest(void);
            Script_LabRequest();
        }
        break;
    case FA_NATIVE:
        fm.pend_kind = 3;
        fm.pend_a = m->kind;
        fm.pend_b = it->sel;
        break;
    case FA_MATCH_SETUP:
        fm.pend_kind = fe_rules_enabled() ? 4 : 2;
        fm.pend_a = it->arg;
        break;
    case FA_ONLINE:
        fm.pend_kind = 5;
        break;
    case FA_PAGE:
        fm.pend_kind = 6;
        fm.pend_a = it->arg;
        break;
    }
    if (fp.motion_ok) {
        fp_event(fp.confirm_event, fp.sel, fp.frame, -1);
        fp_screen_out(fp.frame + fp.screen_out_at);
        fm.leave_at = fp.frame + fp.screen_out_at + fp_screen_out_length();
    } else {
        fm.leave_at = fp.frame + 1;
    }
}

static void fm_back(void)
{
    const FeMenu* m = fm.menu;
    sfxBack();
    if (m->back_kind == 0xFF) {
        fm.pend_kind = 2;
        fm.pend_a = GM_TITLE;
    } else {
        fm.pend_kind = 1;
        fm.pend_a = m->back_kind;
        fm.pend_b = m->back_sel;
    }
    if (fp.motion_ok) {
        fp_screen_out(fp.frame);
        fm.leave_at = fp.frame + fp_screen_out_length();
    } else {
        fm.leave_at = fp.frame + 1;
    }
}

/* The slot a direction moves to. Up/down: vanilla's item order, wrapping. Hubs: left/right
 * cross between the hero and the column. */
static int fm_move(u32 in)
{
    int n = fm.nvis, i, r = -1, rank_of[FM_MAX_VIS], by_rank[FM_MAX_VIS];
    if (n <= 1) {
        return fm.cursor;
    }
    for (i = 0; i < n; i++) {
        rank_of[i] = fp.tile[fm.tile_of[i]].rank;
        by_rank[rank_of[i]] = i;
    }
    r = rank_of[fm.cursor];
    /* up/down in vanilla's own order, wrapping, as its menus do (and as players' hands and the
       netplay pad scripts expect) */
    if (in & MenuInput_Up) {
        return (fm.cursor + n - 1) % n;
    }
    if (in & MenuInput_Down) {
        return (fm.cursor + 1) % n;
    }
    if (fm.menu->style == FM_HUB) {
        if ((in & MenuInput_Left) && r > 0) {
            return by_rank[0];
        }
        if ((in & MenuInput_Right) && r == 0) {
            return by_rank[1];
        }
    }
    return fm.cursor;
}

/* The cursor to slot `to`, as a d-pad press moves it: the sound, the list's scroll, the kit's
 * row_select / row_deselect or the hub's cursor move. */
static void fm_go(int to)
{
    if (to == fm.cursor || to < 0 || to >= fm.nvis) {
        return;
    }
    sfxMove();
    fm.cursor = to;
    fm_list_scroll();
    if (fp.motion_ok && fm.menu->style == FM_LIST && fp_find_event("row_select") >= 0) {
        /* the kit's list_cursor_move: row_deselect / row_select, with the hub's
           description fade for the strip */
        int old = fp.sel;
        fp.prev = old;
        fp.sel = fm.tile_of[to];
        fp_event("row_deselect", old, fp.frame, -1);
        fp_event("desc_out", old, fp.frame, -1);
        fp_event("row_select", fp.sel, fp.frame, -1);
        fp_event("desc_in", fp.sel, fp.frame, -1);
    } else if (fp.motion_ok) {
        fp_cursor_move(fm.tile_of[to], fp.frame);
    } else {
        fp_reset_channels();
        fp_select_initial(fm.tile_of[to], fp.frame);
    }
}

/* The slot under the pointer (a tile's face as drawn), or -1. */
static int fm_mouse_slot(void)
{
    int i;
    for (i = 0; i < fm.nvis; i++) {
        const FpTile* t = &fp.tile[fm.tile_of[i]];
        if (fms_in_quad(t->q_face) || fms_in_quad(t->q_plate)) {
            return i;
        }
    }
    return -1;
}

/* ---- the scene ---------------------------------------------------------------------------- */

static void fm_scene_enter(void)
{
    extern int PcFeHubDemo(void);
    const FeMenu* m;
    fm.active = true;
    fm.fade = FM_FADE;
    fm.fading_out = false;
    fm.native_req = false;
    fm.demo = PcFeHubDemo();
    /* what mnMain_Scene_OnEnter does on every arrival: the menu music and saving the card */
    lbAudioAx_80023F28(gmMainLib_8015ECB0());
    lbCardGame_SaveChanges();
    if (fm.demo >= 0) {
        fm_demo_open();
        return;
    }
    m = fm_find(fm.start_kind);
    if (m == NULL) {
        m = fm_find(MENU_KIND_MAIN);
    }
    fm_open(m, fm.start_sel);
    fm_mirror_flow();
}

static void fm_scene_frame(void)
{
    u32 in;
    fm.frames++;
    if (fm.fading_out) {
        if (++fm.fade >= FM_FADE) {
            gm_ChangeGameModeAfterCurrentScene(fm.leave_mode);
            gm_801A4B60();
        }
    } else if (fm.fade > 0) {
        fm.fade--;
    }
    if (fm.demo >= 0) {
        int f = fm.frames - 1;
        if (fm.demo < 100000) {
            f = f < fm.demo ? f : fm.demo;
        } else {
            f %= 250;
        }
        fm_demo_frame(f);
        fp.frame = f;
        return;
    }
    fp.frame++;
    if (fm.leave_at != 0 && fp.frame >= fm.leave_at) {
        fm.leave_at = 0;
        fm_do_pending();
    } else if (fm.leave_at == 0 && !fm.fading_out && fm.frames >= fm.ready_at && fm.nvis > 0 &&
               fm_first_boot()) {
        /* the first boot: SETTINGS > CONTROLS once (plug in, test, how to play online) */
    } else if (fm.leave_at == 0 && !fm.fading_out && fm.frames >= fm.ready_at && fm.nvis > 0) {
        in = mn_80229624(4) | fms_menu_bits();
        {
            /* the mouse: pointing moves the cursor, a click selects, the wheel scrolls a list */
            int hov = fm_mouse_slot();
            if (hov >= 0 && (fms.moved || fms.click) && !(in & MenuInput_Confirm)) {
                fm_go(hov);
            }
            if (fms.click && hov >= 0) {
                in |= MenuInput_Confirm;
            }
            if (fms.wheel != 0 && fm.menu->style == FM_LIST) {
                /* the wheel stops at the ends (the d-pad wraps) */
                int to = fm.cursor - fms.wheel;
                fm_go(to < 0 ? 0 : to >= fm.nvis ? fm.nvis - 1 : to);
            }
        }
        /* L+R+START anywhere but Main: back to the main menu, as mnMain_Scene_OnFrame does
           (it reloads GM_MENU with force_main_menu set, which routes here) */
        if (mn_8022F218() && fm.menu->kind != MENU_KIND_MAIN) {
            lbAudioAx_80023694();
            sfxBack();
            mn_8022F268();
            fe.reported = GM_MENU;
            fm_leave_scene(GM_MENU);
        } else if (in & MenuInput_Confirm) {
            fm_confirm();
        } else if (in & MenuInput_Back) {
            fm_back();
        } else if (in & (MenuInput_Up | MenuInput_Down | MenuInput_Left | MenuInput_Right)) {
            fm_go(fm_move(in));
        }
    }
    fm_mirror_flow();
    fp_evaluate(fp.frame);
    fm_clip_rows();
    fm_texts_update();
}

static void fm_scene_exit(void)
{
    fm_close();
    fm.active = false;
}
