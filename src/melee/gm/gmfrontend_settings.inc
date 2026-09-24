/* gmfrontend_settings.inc - included by gmfrontend.c (TARGET_PC only), before the kit list.
 *
 * ---- SETTINGS: the port's own settings pages -----------------------------------------------
 *
 * MAIN MENU > SETTINGS lists these pages above the game's own options (Rumble, Screen Display,
 * Language, Erase Data stay native). Each page is a toolkit screen - a table of rows, each row a
 * get/set pair on the setting it shows - so rows are data: a page is one array, and a later
 * script API can append rows the same way. Every change applies at once and persists:
 *
 *   VIDEO     render scale, frame rate, vsync, FPS readout     video.cfg   (gw_Video_*)
 *   AUDIO     master volume; music/effects mix, stereo/mono     audio.cfg + the memory card
 *   CONTROLS  adapter status + recalibrate, the hotkeys        -
 *   ONLINE    player name, server, input delay, stage list     settings.cfg
 *   MODS      every mod in the folder, on/off                  mods/enabled.txt (gw_Mods_*)
 *   GAMEPLAY  unlock everything, the default match rules       settings.cfg + the memory card
 *
 * Environment variables still win over what is saved (MELEE_UNLOCK_ALL, MELEE_INPUT,
 * MELEE_NETPLAY_SERVER, MELEE_VOLUME, MELEE_RENDER_SCALE, MELEE_FPS), so tests stay scripted. */

extern float Video_RenderScale(void);
extern void Video_SetRenderScale(float scale);
extern int Video_FrameRate(void);
extern void Video_SetFrameRate(int fps);
extern int Video_Vsync(void);
extern void Video_SetVsync(int on);
extern int Video_ShowFps(void);
extern void Video_SetShowFps(int on);
extern int Audio_Volume(void);
extern void Audio_SetVolume(int percent);
extern int Settings_Str(const char* key, char* out, int cap, const char* dflt);
extern void Settings_SetStr(const char* key, const char* value);
extern int Settings_Int(const char* key, int dflt);
extern void Settings_SetInt(const char* key, int value);
extern int Settings_TextKeys(char* buf, int cap, int caret);
extern int Settings_TextCaret(void);
extern void PcUnlockAll_Reload(void);
extern int PcUnlockAll(void);
extern void Netplay_ReloadServer(void);
extern void Netplay_ServerName(char* out, int cap);
extern int gc_adapter_present(void);
extern void gc_adapter_recalibrate(void);
extern int Mods_Count(void);
extern const char* Mods_Name(int i);
extern const char* Mods_Description(int i);
extern const char* Mods_Kind(int i);
extern const char* Mods_StatusText(int i);
extern int Mods_IsEnabled(int i);
extern int Mods_SetEnabled(int i, int on);
extern int Mods_Save(void);
extern int Mods_RestartNeeded(void);
extern const char* Mods_Dir(void);
extern bool lbAudioAx_80024BD0(void);
extern bool lbAudioAx_80024C08(enum_t);

enum { FSP_VIDEO, FSP_AUDIO, FSP_CONTROLS, FSP_ONLINE, FSP_MODS, FSP_GAMEPLAY, FSP_COUNT };

/* ---- text entry (player name, server) ----------------------------------------------------------
 * A on the row starts editing in place: the keyboard types (Backspace, Enter done, Esc cancel);
 * a controller moves the caret with left/right and cycles the character with up/down, A done,
 * B cancel. The row's readout shows the text with the caret. */
static struct {
    bool on;
    char buf[48], before[48];
    int cap, caret;
    const char* key;
    void (*done)(void);
} fte;

static const char fte_set[] = " ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789.-_:";

static void fte_begin(const char* key, int cap, void (*done)(void))
{
    fte.on = true;
    fte.key = key;
    fte.cap = cap < (int) sizeof fte.buf ? cap : (int) sizeof fte.buf;
    fte.done = done;
    Settings_Str(key, fte.buf, fte.cap, "");
    fp_copy(fte.before, fte.buf, sizeof fte.before);
    fte.caret = (int) strlen(fte.buf);
    fe_ol_notice("Type it, or Up/Down + Left/Right. A or Enter: done. B or Esc: cancel.", false);
}

static void fte_fmt(const char* key, char* out)
{
    if (fte.on && fte.key == key) {
        int i, k = 0;
        for (i = 0; i <= (int) strlen(fte.buf) && k < FE_STR - 2; i++) {
            if (i == fte.caret) out[k++] = '|';
            if (fte.buf[i] != '\0') out[k++] = fte.buf[i];
        }
        out[k] = '\0';
    } else {
        Settings_Str(key, out, FE_STR, "");
        if (out[0] == '\0') sprintf(out, "%s", "(none)");
    }
}

/* One frame while editing. True when the edit ended this frame. */
static bool fte_frame(void)
{
    u32 in = mn_80229624(4);
    int n, k = Settings_TextKeys(fte.buf, fte.cap, fte.caret), end = 0;
    fte.caret = Settings_TextCaret();
    n = (int) strlen(fte.buf);
    if (in & MenuInput_Left) {
        if (fte.caret > 0) fte.caret--;
        sfxMove();
    } else if (in & MenuInput_Right) {
        if (fte.caret < n) fte.caret++;
        sfxMove();
    } else if (in & (MenuInput_Up | MenuInput_Down)) {
        int dir = (in & MenuInput_Up) ? 1 : -1, ns = (int) sizeof fte_set - 1, ci = 0;
        const char* p;
        if (fte.caret >= n) {
            if (n >= fte.cap - 1) return false;
            fte.buf[n] = ' ';
            fte.buf[n + 1] = '\0';
        }
        for (p = fte_set; *p != 0 && *p != fte.buf[fte.caret]; p++) {
        }
        if (*p == 0) p = NULL;
        ci = p != NULL ? (int) (p - fte_set) : 0;
        fte.buf[fte.caret] = fte_set[((ci + dir) % ns + ns) % ns];
        sfxMove();
    }
    if ((in & MenuInput_Confirm) || k == 2) {
        end = 1;
    } else if ((in & MenuInput_Back) || k == 3) {
        end = 2;
    }
    if (end) {
        /* trailing spaces off */
        n = (int) strlen(fte.buf);
        while (n > 0 && fte.buf[n - 1] == ' ') fte.buf[--n] = '\0';
        if (end == 1) {
            Settings_SetStr(fte.key, fte.buf);
            sfxForward();
            if (fte.done != NULL) fte.done();
            fe_ol_notice("Saved.", false);
        } else {
            sfxBack();
            fe_ol_notice("Not changed.", false);
        }
        fte.on = false;
        return true;
    }
    return false;
}

/* ---- VIDEO ---------------------------------------------------------------------------------- */
static const float fsv_scales[] = { 0.0F, 1.0F, 1.5F, 2.0F, 3.0F, 4.0F };
static const char* const fsv_scale_names[] = { "Auto (window)", "1x", "1.5x", "2x", "3x", "4x" };
static int fsv_get_scale(void)
{
    float s = Video_RenderScale();
    int i, best = 0;
    for (i = 0; i < 6; i++) {
        float d = fsv_scales[i] - s, db = fsv_scales[best] - s;
        if ((d < 0 ? -d : d) < (db < 0 ? -db : db)) best = i;
    }
    return best;
}
static void fsv_set_scale(int v) { Video_SetRenderScale(fsv_scales[v]); }
static const int fsv_rates[] = { 60, 120, 144, 0 };
static const char* const fsv_rate_names[] = { "60 (normal)", "120", "144", "Uncapped" };
static int fsv_get_rate(void)
{
    int r = Video_FrameRate(), i;
    for (i = 0; i < 4; i++) {
        if (fsv_rates[i] == r) return i;
    }
    return 0;
}
static void fsv_set_rate(int v) { Video_SetFrameRate(fsv_rates[v]); }
static int fsv_get_vsync(void) { return Video_Vsync(); }
static void fsv_set_vsync(int v) { Video_SetVsync(v); }
static int fsv_get_fps(void) { return Video_ShowFps(); }
static void fsv_set_fps(int v) { Video_SetShowFps(v); }

static const FrontendItem fe_items_set_video[] = {
    { FE_CHOICE, 0, "Render Scale", "The picture's internal size. Auto follows the window.",
      fsv_get_scale, fsv_set_scale, 0, 5, 1, fsv_scale_names },
    { FE_CHOICE, 0, "Frame Rate",
      "60 is Melee's own. Above it, frames in between are drawn (experimental, 1 frame behind).",
      fsv_get_rate, fsv_set_rate, 0, 3, 1, fsv_rate_names },
    { FE_TOGGLE, 0, "VSync", "Wait for the display. Off can tear but responds sooner.", fsv_get_vsync,
      fsv_set_vsync, 0, 1, 1 },
    { FE_TOGGLE, 0, "Show FPS", "A frame-rate readout in the corner.", fsv_get_fps, fsv_set_fps, 0, 1,
      1 },
};

/* ---- AUDIO ---------------------------------------------------------------------------------- */
static int fsa_get_vol(void) { return Audio_Volume(); }
static void fsa_set_vol(int v) { Audio_SetVolume(v); }
static void fsa_fmt_vol(int v, char* out) { sprintf(out, "%d%%", v); }
static int fsa_get_mix(void) { return (s8) gmMainLib_8015ED74(); }
static void fsa_set_mix(int v)
{
    gmMainLib_8015ED80((s8) v);
    gm_801602C0((s8) v);
}
static void fsa_fmt_mix(int v, char* out)
{
    if (v == 0) sprintf(out, "%s", "Even");
    else sprintf(out, "%+d", v);
}
static const char* const fsa_modes[] = { "Stereo", "Mono" };
static int fsa_get_mono(void) { return lbAudioAx_80024BD0() ? 1 : 0; }
static void fsa_set_mono(int v) { lbAudioAx_80024C08(v); }

static const FrontendItem fe_items_set_audio[] = {
    { FE_SLIDER, 0, "Master Volume", "Everything the game plays.", fsa_get_vol, fsa_set_vol, 0, 100,
      5, NULL, fsa_fmt_vol },
    { FE_SLIDER, 0, "Music / Effects", "The mix between music and sound effects (the old Sound screen's).",
      fsa_get_mix, fsa_set_mix, -100, 100, 5, NULL, fsa_fmt_mix },
    { FE_CHOICE, 0, "Output", "Stereo or mono.", fsa_get_mono, fsa_set_mono, 0, 1, 1, fsa_modes },
};

/* ---- CONTROLS ------------------------------------------------------------------------------- */
static void fsc_fmt_adapter(int v, char* out)
{
    (void) v;
    sprintf(out, "%s", gc_adapter_present() ? "Connected - A recalibrates" : "Not found");
}
static void fsc_recal(void)
{
    if (gc_adapter_present()) {
        gc_adapter_recalibrate();
        fe_ol_notice("Controllers recalibrated - hands off the sticks.", false);
    } else {
        fe_ol_notice("No GameCube adapter found.", true);
    }
}
static int fsc_zero(void) { return 0; }
#define FSC_KEY(name, text)                                                                       \
    static void fsc_k_##name(int v, char* out)                                                    \
    {                                                                                             \
        (void) v;                                                                                 \
        sprintf(out, "%s", text);                                                                 \
    }
FSC_KEY(panel, "F9")
FSC_KEY(reshow, "F10")
FSC_KEY(console, "Backquote (left of 1)")
FSC_KEY(mouse, "Point, click; right-click back")

/* The keyboard does not play: controllers play, the keyboard is hotkeys (and mods' keys). */
static const FrontendItem fe_items_set_controls[] = {
    { FE_ACTION, FE_DO_CALL, "GameCube Adapter", "A recalibrates every controller's sticks.", NULL, NULL,
      0, 0, 0, NULL, fsc_fmt_adapter, NULL, fsc_recal },
    { FE_SLIDER, 0, "Controller Panel", "Hotkey: what each port reads; with an adapter, also recalibrates.",
      fsc_zero, NULL, 0, 0, 0, NULL, fsc_k_panel },
    { FE_SLIDER, 0, "Overlays Back", "Hotkey: puts every overlay back on screen.", fsc_zero, NULL, 0,
      0, 0, NULL, fsc_k_reshow },
    { FE_SLIDER, 0, "Console", "Hotkey: the command console.", fsc_zero, NULL, 0, 0, 0, NULL,
      fsc_k_console },
    { FE_SLIDER, 0, "Mouse", "Menus take the mouse too (the wheel scrolls); matches never do.",
      fsc_zero, NULL, 0, 0, 0, NULL, fsc_k_mouse },
};

/* ---- ONLINE --------------------------------------------------------------------------------- */
static const char fso_key_name[] = "name";
static const char fso_key_server[] = "server";
static void fso_fmt_name(int v, char* out)
{
    (void) v;
    fte_fmt(fso_key_name, out);
}
static void fso_edit_name(void) { fte_begin(fso_key_name, 17, NULL); }
static void fso_server_done(void) { Netplay_ReloadServer(); }
static void fso_fmt_server(int v, char* out)
{
    (void) v;
    if (fte.on && fte.key == fso_key_server) {
        fte_fmt(fso_key_server, out);
    } else {
        Netplay_ServerName(out, FE_STR);
        if (out[0] == '\0') sprintf(out, "%s", "(none)");
    }
}
static void fso_edit_server(void) { fte_begin(fso_key_server, 48, fso_server_done); }
static int fso_get_delay(void) { return fe_np_delay; }
static void fso_set_delay(int v)
{
    fe_np_delay = v;
    Settings_SetInt("delay", v);
}
static int fso_get_stages(void) { return Netplay_StageMode(); }
static void fso_set_stages(int v)
{
    Netplay_SetStageMode(v);
    Settings_SetInt("stage_mode", v);
}
static const char* const fso_stage_modes[] = { "Competitive", "All Stages" };

static const FrontendItem fe_items_set_online[] = {
    { FE_ACTION, FE_DO_CALL, "Player Name", "Shown to your opponent in the lobby and before matches.",
      NULL, NULL, 0, 0, 0, NULL, fso_fmt_name, NULL, fso_edit_name },
    { FE_ACTION, FE_DO_CALL, "Server", "The matchmaking server (address or address:port).", NULL, NULL,
      0, 0, 0, NULL, fso_fmt_server, NULL, fso_edit_server },
    { FE_SLIDER, 0, "Input Delay", "Frames of delay in rooms you host. 2 suits most connections.",
      fso_get_delay, fso_set_delay, 0, 8, 1, NULL, fe_np_fmt_delay },
    { FE_CHOICE, 0, "Stage List", "Rooms you host: the legal six, or every stage you both have.",
      fso_get_stages, fso_set_stages, 0, 1, 1, fso_stage_modes },
};

/* ---- MODS: one toggle per mod in the folder --------------------------------------------------- */
#define FSM_MAX 40
static FrontendItem fe_items_set_mods[FSM_MAX + 1];
static int fsm_n;
static void fsm_fmt_state(int v, char* out)
{
    (void) v;
    if (Mods_RestartNeeded()) sprintf(out, "%s", "Restart to apply");
    else sprintf(out, "%d mod%s", fsm_n, fsm_n == 1 ? "" : "s");
}
static void fsm_toggle(int i, int v)
{
    if (i < 0 || i >= fsm_n) return;
    Mods_SetEnabled(i, v);
    Mods_Save();
    fe_ol_notice(Mods_RestartNeeded() ? "Saved. The change applies when the game restarts." : "Saved.",
                 false);
}
#define FSM_ROW(i)                                                                                \
    static int fsm_g##i(void) { return Mods_IsEnabled(i); }                                        \
    static void fsm_s##i(int v) { fsm_toggle(i, v); }                                             \
    static int fsm_v##i(void) { return (i) < fsm_n; }
FSM_ROW(0) FSM_ROW(1) FSM_ROW(2) FSM_ROW(3) FSM_ROW(4) FSM_ROW(5) FSM_ROW(6) FSM_ROW(7)
FSM_ROW(8) FSM_ROW(9) FSM_ROW(10) FSM_ROW(11) FSM_ROW(12) FSM_ROW(13) FSM_ROW(14) FSM_ROW(15)
FSM_ROW(16) FSM_ROW(17) FSM_ROW(18) FSM_ROW(19) FSM_ROW(20) FSM_ROW(21) FSM_ROW(22) FSM_ROW(23)
FSM_ROW(24) FSM_ROW(25) FSM_ROW(26) FSM_ROW(27) FSM_ROW(28) FSM_ROW(29) FSM_ROW(30) FSM_ROW(31)
FSM_ROW(32) FSM_ROW(33) FSM_ROW(34) FSM_ROW(35) FSM_ROW(36) FSM_ROW(37) FSM_ROW(38) FSM_ROW(39)
#define FSM_P(i) { fsm_g##i, fsm_s##i, fsm_v##i }
static const struct {
    int (*g)(void);
    void (*s)(int);
    int (*v)(void);
} fsm_fns[FSM_MAX] = {
    FSM_P(0), FSM_P(1), FSM_P(2), FSM_P(3), FSM_P(4), FSM_P(5), FSM_P(6), FSM_P(7), FSM_P(8),
    FSM_P(9), FSM_P(10), FSM_P(11), FSM_P(12), FSM_P(13), FSM_P(14), FSM_P(15), FSM_P(16),
    FSM_P(17), FSM_P(18), FSM_P(19), FSM_P(20), FSM_P(21), FSM_P(22), FSM_P(23), FSM_P(24),
    FSM_P(25), FSM_P(26), FSM_P(27), FSM_P(28), FSM_P(29), FSM_P(30), FSM_P(31), FSM_P(32),
    FSM_P(33), FSM_P(34), FSM_P(35), FSM_P(36), FSM_P(37), FSM_P(38), FSM_P(39),
};
static char fsm_help[FSM_MAX][FE_STR];

/* Append, bounded to a help line. */
static void fsm_cat(char* dst, const char* src)
{
    int n = (int) strlen(dst);
    while (src != NULL && *src != 0 && n < FE_STR - 1) {
        dst[n++] = *src++;
    }
    dst[n] = 0;
}

/* The rows, from the mods folder as it is now. */
static void fsm_fill(void)
{
    int i;
    FrontendItem* it = &fe_items_set_mods[0];
    fsm_n = Mods_Count() < FSM_MAX ? Mods_Count() : FSM_MAX;
    memset(fe_items_set_mods, 0, sizeof fe_items_set_mods);
    it->kind = FE_SLIDER;
    it->label = "Mods";
    it->help = "Turn mods on or off. Changes apply when the game restarts.";
    it->get = fsc_zero;
    it->format = fsm_fmt_state;
    for (i = 0; i < FSM_MAX; i++) {
        FrontendItem* r = &fe_items_set_mods[i + 1];
        r->kind = FE_TOGGLE;
        r->label = i < fsm_n ? Mods_Name(i) : "";
        if (i < fsm_n) {
            const char* d = Mods_Description(i);
            const char* st = Mods_StatusText(i);
            char* h = fsm_help[i];
            h[0] = 0;
            fsm_cat(h, Mods_Kind(i));
            if (st != NULL && st[0] != 0 && strcmp(st, "active") != 0) {
                fsm_cat(h, " (");
                fsm_cat(h, st);
                fsm_cat(h, ")");
            }
            if (d != NULL && d[0] != 0) {
                fsm_cat(h, " - ");
                fsm_cat(h, d);
            }
        }
        r->help = fsm_help[i];
        r->get = fsm_fns[i].g;
        r->set = fsm_fns[i].s;
        r->min = 0;
        r->max = 1;
        r->step = 1;
        r->visible = fsm_fns[i].v;
    }
}

/* ---- GAMEPLAY ------------------------------------------------------------------------------- */
static int fsg_get_unlock(void) { return PcUnlockAll(); }
static void fsg_set_unlock(int v)
{
    Settings_SetInt("unlock_all", v);
    PcUnlockAll_Reload();
    fe_ol_notice(v ? "Every fighter and stage is open. Your save data is not changed."
                   : "Back to what your save has unlocked.",
                 false);
}

static const FrontendItem fe_items_set_gameplay[] = {
    { FE_TOGGLE, 0, "Unlock Everything", "Every fighter, stage and option, without touching the save.",
      fsg_get_unlock, fsg_set_unlock, 0, 1, 1 },
    { FE_CHOICE, 0, "Mode", "How a VS match is won.", fe_get_mode, fe_set_mode, 0, 3, 1, fe_opt_mode },
    { FE_SLIDER, 0, "Stocks", "Lives each player starts with.", fe_get_stocks, fe_set_stocks, 1, 99, 1,
      NULL, NULL, fe_is_stock },
    { FE_SLIDER, 0, "Time Limit", "0 plays without a clock.", fe_get_time, fe_set_time, 0, 99, 1, NULL,
      fe_fmt_time },
    { FE_CHOICE, 0, "Items", "How often items appear.", fe_get_items, fe_set_items, -1, 4, 1,
      fe_opt_items },
    { FE_SLIDER, 0, "Damage Ratio", "How far hits send fighters.", fe_get_ratio, fe_set_ratio, 5, 20, 1,
      NULL, fe_fmt_ratio },
    { FE_CHOICE, 0, "Handicap", "Auto evens out winners and losers.", fe_get_handicap, fe_set_handicap,
      0, 2, 1, fe_opt_handicap },
    { FE_TOGGLE, 0, "Friendly Fire", "Team attacks hit teammates.", fe_get_ff, fe_set_ff, 0, 1, 1 },
    { FE_TOGGLE, 0, "Pause", "Allow pausing during the match.", fe_get_pause, fe_set_pause, 0, 1, 1 },
};

#define FSP_N(a) ((int) (sizeof a / sizeof a[0]))
static const FrontendScreen fe_screen_settings[FSP_COUNT] = {
    { "SETTINGS", "VIDEO", fe_items_set_video, FSP_N(fe_items_set_video) },
    { "SETTINGS", "AUDIO", fe_items_set_audio, FSP_N(fe_items_set_audio) },
    { "SETTINGS", "CONTROLS", fe_items_set_controls, FSP_N(fe_items_set_controls) },
    { "SETTINGS", "ONLINE", fe_items_set_online, FSP_N(fe_items_set_online) },
    { "SETTINGS", "MODS", fe_items_set_mods, FSM_MAX + 1 },
    { "SETTINGS", "GAMEPLAY", fe_items_set_gameplay, FSP_N(fe_items_set_gameplay) },
};

static bool fe_is_settings(const FrontendScreen* s)
{
    return s >= &fe_screen_settings[0] && s < &fe_screen_settings[FSP_COUNT];
}

/* The saved online defaults, into the ONLINE screen's state (once per boot is enough, but cheap). */
static void fe_settings_apply_online(void)
{
    fe_np_delay = Settings_Int("delay", fe_np_delay);
    Netplay_SetStageMode(Settings_Int("stage_mode", Netplay_StageMode()));
}
