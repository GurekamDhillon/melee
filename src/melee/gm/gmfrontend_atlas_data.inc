/* gmfrontend_atlas_data.inc - the Atlas data screens (spec 13.8, Atlas step 8): Event Match, Name Entry's tag list, Sound Test, Special Messages, Bonus
 * Records, Misc. Records and VS. Records, and the Results stand-in. Included by gmfrontend.c after gmfrontend_atlas_set.inc.
 *
 * HOW A SCREEN OPENS. Each is opened from the FA_NATIVE row that opens its retail screen today: fm_confirm asks fad_open_for(kind, sel) first and, when a data
 * screen takes the row, the frontend scene stays loaded (no scene change), the menu keeps its cursor and B returns to it. So the position protocol
 * (MenuKind, selection), the SEL_* numbers and gmFrontend_NativeReturn are untouched, and no table row changed. MELEE_ATLAS=0, a netplay session, or a
 * screen left out of MELEE_ATLAS_DATA (a word list: events,name,sound,messages,bonus,misc,vsrec; empty or "all" is every one; "none" is none) runs the retail
 * screen exactly as before. The word "notext" never opens the disc's text archive: every row keeps its authored label. On its own it means every screen, without text; with screen words it
 * narrows nothing (MELEE_ATLAS_DATA=sound,notext is the Sound Test without text).
 *
 * ONE DOOR. The screens are drawn by the host's data door (pc/platform/gw_script_ui_data.inc, over the settings door): one engine-owned native list screen with
 * a handle. THE ADAPTER OWNS THE CURSOR AND THE WINDOW: lists are longer than the host's window (up to 120 tags), so up, down and the wheel arrive as MOVE
 * events and are NOT applied by the host; this file moves its absolute cursor (up and down wrap over the whole list, not the window), slides the window with
 * Ui_DataFirst, submits the window's rows (ids are the absolute row number) and puts the host's focus on the cursor's row by id. A hover or a click arrives as
 * FOCUS or ACCEPT with a window-relative slot; the window's first row is added here. An accept on a locked row is dropped by the host; this file checks again.
 *
 * READ-ONLY. A data screen writes nothing to the save by looking at it. Events write the selection only when the player starts one (gm_801BEB74, as the retail
 * list does); Sound Test sets no volume (it reads the saved ones and applies them to the live mixer, as retail does) and stores the heard-track flag
 * (gmMainLib_8015ED68) for a track lbAudioAx_80023090 flags, exactly as the retail screen does (toy.c reads it for the Trophy Gallery music); Records, Messages,
 * Bonus and Name Entry's list write nothing. tools/port/test_fe_atlas_data.py allow-lists the game calls that write.
 *
 * RETAIL TEXT. Words that live on the disc (event names and descriptions, message bodies, sound names, bonus lines) are SIS streams. They are decoded at run
 * time, per string, into a bounded buffer (fad_sis_text), drawn, and never stored in a file, a log or the repo; a string with any glyph the Latin decoder does not
 * know keeps the row's AUTHORED label ("EVENT 7", "TRACK 12"): the gate's verdict is PARTIAL (tools/port/sis_probe.py). Logs carry counts only
 * (tools/port/check_no_disc_text.py). */

#include <melee/db/db.h>
#include <melee/gm/gmresult.h>
#include <melee/gm/gmvs.h>
#include <melee/lb/lb_013B.h>

#define FAD_TEXT_ROOM 160  /* the explainer takes 159 characters and a terminator (AT_TEXT) */
#define FAD_TEXT_PROBE 512
#define FAD_ID_ROOM 24     /* AT_ID */

extern int Ui_DataOpen(const char* id, const char* t0, const char* t1, const char* title, int chapter);
extern void Ui_DataRow(int h, int slot, int abs, const char* label, const char* value, int flags);
extern void Ui_DataSub(int h, int slot, const char* sub);
extern void Ui_DataCounter(int h, const char* text);
extern int Ui_DataWindow(void);
extern int Ui_DataFirst(int total, int per, int focus, int first);
extern const char* Ui_EnvData(void);
extern int Ui_EnvProbe(void);
extern int Ui_NetplayActive(void);
extern void Ui_RowMisc(int h, int slot, int row, unsigned value, const char* fighter);
extern const char* Ui_MiscLabel(int row);
extern int Ui_EventLastOpen(int first_bound, int debug);
extern void Ui_RowEvent(int h, int slot, int event, int flags, unsigned best, const char* name);
extern void Ui_RowMsg(int h, int slot, int k, int ymd);
extern void Ui_RowBonus(int h, int slot, int k);
extern void Ui_MsgSet(int id, int unlocked, unsigned date);
extern int Ui_MsgOrder(int n);
extern int Ui_MsgAt(int k);
extern void Ui_RowSound(int h, int slot, int i, const char* name, int playing);
extern void Ui_RowSfx(int h, int slot, int group, int sub, int count);
extern int Ui_SoundToggle(int playing, int row);
extern void Ui_RowStat(int h, int slot, int rank, const char* name, int statflags, unsigned value);
extern int Ui_StatKind(int stat);
extern const char* Ui_StatLabel(int stat);
extern const char* Ui_StatText(int statflags, unsigned value);
extern void Ui_RankSet(int i, unsigned value);
extern int Ui_RankBuild(int n);
extern int Ui_RankAt(int k);
extern void Ui_RowTag(int h, int slot, int tag, const unsigned char* bytes, int index);
extern const char* Ui_TagText(const unsigned char* bytes);
extern void Ui_SisTables(const u8* glyph, const u8* sjis);
extern int Ui_SisDecode(const u8* sis, int len, char* out, int cap, int* st5);
extern void Ui_ResBegin(int outcome, int is_teams, int canceled);
extern void Ui_ResPlayer(int port, int pkind, int ckind, int stocks, int kos, int falls);
extern void Ui_ResPlayer2(int port, int percent, int team, int winner, const char* name);
extern void Ui_ResCommit(void);
extern void Ui_ResConfirm(int port, int kind);
extern int Ui_ResDone(void);
extern int Ui_ResFill(int h);

/* retail functions the sources read: declared here so this file does not pull whole headers (and their enum names) into the frontend's translation unit */
extern s32 mnInfo_80251A08(s32 id);                      /* mninfo.c: message id -> unlocked */
extern void un_802FE3F8(int id, int base, s16* idx, s16* idx2); /* ifprize.c: message id -> its SIS index */
extern int mnInfoBonus_PcKind(int i);                    /* mninfobonus.c (TARGET_PC): the retail bonus table, 0x100 at its end */
extern int gmResult_PcWinner(MatchEnd* me);         /* gmresult.c (TARGET_PC): fn_801771C0's winner index (data->x6) for a summary, no scene state touched */
extern s32 mnEvent_8024CE74(void);                       /* mnevent.c: the last page start of the event list (a scroll bound) */
extern unsigned int mnCount_GetRowValue_Number(int row); /* mncount.c */
extern int mnCount_GetRowValue_Character(int row);       /* mncount.c: a fighter, or SELKIND_COUNT for nobody */
extern int mnDiagram2_GetStatValue(u8 is_name_mode, u8 stat_type, u8 entity_idx);
extern u8 mnDiagram_GetFighterByIndex(int idx);
extern u8 mnDiagram_GetNameByIndex(int idx);
extern u32 mnDiagram_ConvertDistanceForDisplay(u32 distance);
extern bool mnDiagram_IsDistanceOverflow(u32 distance);
extern char* GetNameText(int slot);
extern int GetNameCount(void);
extern bool IsNameListFull(void);
extern u8 gm_801BEB80(void);
extern void gm_801BEB74(u8 slot);
extern bool gm_801BEB8C(u8 slot);
extern u8 gm_801BEBA8(u8 slot);

/* mnsoundtest.c's tables (globals there): 80 tracks (text id, music id) and 30 sound groups */
typedef struct FadSoundEntry {
    u16 text_id;
    u32 idx;
} FadSoundEntry;
extern u8 text_ids[];
extern FadSoundEntry data_2[];
extern u8 data_3[];

enum { FD_NONE, FD_EVENTS, FD_NAME, FD_SOUND, FD_MESSAGES, FD_BONUS, FD_MISC, FD_VSREC, FD_COUNT };
/* the host's event numbers (GS_SETEV_*, gw_script_ui_set.inc); the host test pins them */
enum { FAD_EV_ACCEPT = 2, FAD_EV_BACK = 3, FAD_EV_TAB = 4, FAD_EV_FOCUS = 5, FAD_EV_ALT = 6, FAD_EV_MOVE = 7, FAD_EV_PAGE = 8 };
enum { FAD_DR_DONE = 1, FAD_DR_LOCKED = 2, FAD_DR_NEW = 4 };   /* AT_DR_* */
enum { FAD_EVENTS = 0x33, FAD_MSGS = 0x42, FAD_TAG_SLOTS = 0x78, FAD_SND_TRACKS = 80, FAD_SND_GROUPS = 30, FAD_STATS = 21, FAD_BONUS_MAX = 128 };

typedef struct FadScreen {
    int id;
    const char* atlas_id;
    const char* title;
    u8 kind, sel;           ///< the (MenuKind, SEL_*) the retail entry is, and where B returns: the menu that opened it, untouched
    u8 chapter;             ///< the host's chapter colour: 0 the More side, 1 Solo, 2 Versus
    const char* parent[2];  ///< the trail above the title
    int (*total)(void);
    void (*row)(int h, int slot, int i);   ///< fills window slot `slot` with absolute row i
    void (*explain)(int h, int i);         ///< the explainer of the focused row (-1: no row)
    bool (*accept)(int i);                 ///< A on row i; true when the screen is leaving
    bool (*event)(int h, int type, int a); ///< MOVE left or right, PAGE, ALT, TAB: true when handled
    void (*open)(void);
    void (*close)(void);
    const char* (*keys)(void);
    const char* word;       ///< the word in MELEE_ATLAS_DATA
} FadScreen;

static struct {
    int id;          ///< FD_*, FD_NONE when no data screen is up
    int h;           ///< the host's handle, -1 none
    int focus;       ///< the focused row, absolute
    int first;       ///< the window's first row
    int total;
    bool env_read;
    u32 enabled;     ///< a bit per FD_*, from MELEE_ATLAS_DATA
    bool leaving;    ///< an accept started a scene change: no more input, the screen stays up until the scene ends
    bool reopen;     ///< coming back from an event: the event list opens again on the saved selection
    bool ev_return;  ///< an event was started from the list: the next open focuses its selection (retail opens on 0 the first time)
    bool probed;
} fad = { FD_NONE, -1 };

static const FadScreen* fad_find(int id);
static const FadScreen* fad_for(u8 kind, u8 sel);

/* ---- retail text: the SIS archive the retail menu scene loads, opened for the life of one data screen --------------------------------------------------- */

static HSD_Archive* fad_sis_arc;
static u8** fad_sis_tab;

static bool fad_has(const char* hay, const char* needle);

static int fad_sis_n;        ///< the table's slot count (see fad_sis_open); every index is checked against it
static bool fad_sis_failed;  ///< the archive or its symbol is missing: remembered, so it is not reopened for every string

/* the string at slot idx, and how many bytes of the archive's data lie from its start to the end: NULL when the slot is empty or its pointer is not inside the data */
static const u8* fad_sis_stream(int idx, int* len)
{
    const u8* p;
    const u8* base;
    int size;
    if (fad_sis_tab == NULL || idx < 0 || idx >= fad_sis_n) {
        return NULL;
    }
    base = fad_sis_arc->data;
    size = (int) fad_sis_arc->header.data_size;
    p = (const u8*) fad_sis_tab[idx];
    if (p == NULL || p < base || p >= base + size) {
        return NULL;
    }
    *len = (int) ((base + size) - p);
    return p;
}

static bool fad_sis_open(void)
{
    const u8* base;
    int size, k, n;
    const u8* lowest = NULL;
    if (fad_sis_tab != NULL) {
        return true;
    }
    if (fad_sis_failed || fad_has(Ui_EnvData(), "notext")) {
        return false; /* MELEE_ATLAS_DATA=...,notext: never open the disc's text archive; every row keeps its authored label */
    }
    fad_sis_arc = HSD_SisLib_803A945C(lbLang_IsSavedLanguageUS() ? "SdMenu.usd" : "SdMenu.dat"); /* what mnmain.c:3019 opens for the menu scene */
    if (fad_sis_arc == NULL) {
        fad_sis_failed = true;
        return false;
    }
    fad_sis_tab = (u8**) HSD_ArchiveGetPublicAddress(fad_sis_arc, "SIS_MenuData");
    base = fad_sis_arc->data;
    size = (int) fad_sis_arc->header.data_size;
    n = 0;
    if (fad_sis_tab != NULL) {
        /* the table is an array of pointers and the first string starts where it ends: its length is the distance to the lowest pointer among the first slots */
        for (k = 0; k < 64; k++) {
            const u8* p = (const u8*) fad_sis_tab[k];
            if (p != NULL && p >= base && p < base + size && (lowest == NULL || p < lowest)) {
                lowest = p;
            }
        }
        if (lowest != NULL && lowest > (const u8*) fad_sis_tab) {
            n = (int) ((lowest - (const u8*) fad_sis_tab) / 4);
        }
    }
    if (fad_sis_tab == NULL || n <= 0 || n > size / 4) {
        HSD_SisLib_803A947C(fad_sis_arc); /* the symbol is missing or the table has no readable end: close the archive and do not try again */
        fad_sis_arc = NULL;
        fad_sis_tab = NULL;
        fad_sis_failed = true;
        OSReport("frontend: Atlas data: the text archive has no usable SIS table; rows keep their authored labels\n");
        return false;
    }
    fad_sis_n = n;
    Ui_SisTables(HSD_SisLib_8040C680, lbl_8040C8C0);
    return true;
}

static void fad_sis_close(void)
{
    if (fad_sis_arc != NULL) {
        HSD_SisLib_803A947C(fad_sis_arc);
    }
    fad_sis_arc = NULL;
    fad_sis_tab = NULL;
    fad_sis_n = 0;
}

/* 1 when string sis_idx decoded with no unknown glyph, no jump and no bad opcode; 0 when partly or not decodable (the caller then uses its authored label).
 * out always holds what came out (never logged). A newline becomes a space: a row and the explainer wrap by themselves. A string longer than the room is cut
 * with "..." (and then still counts as decoded). */
static int fad_sis_text(int sis_idx, char* out, int cap)
{
    int st[5];
    int i, len = 0, n;
    const u8* p;
    out[0] = '\0';
    if (!fad_sis_open() || (p = fad_sis_stream(sis_idx, &len)) == NULL) {
        return 0;
    }
    st[0] = st[1] = st[2] = st[3] = st[4] = 0;
    n = Ui_SisDecode(p, len, out, cap, st);
    for (i = 0; out[i] != '\0'; i++) {
        if (out[i] == '\n') {
            out[i] = ' ';
        }
    }
    if (st[0] > n && n >= 4) { /* more characters than fit: the cut is shown */
        out[n - 3] = '.';
        out[n - 2] = '.';
        out[n - 1] = '.';
    }
    return st[1] == 0 && st[3] == 0 && st[4] == 0 && out[0] != '\0';
}

/* The self-check that needs no disc text: encode a known string with the retail encoder, decode it back. */
static void fad_sis_selfcheck(void)
{
    static const char probe[] = "Ab 09:.-";
    u8 buf[0x100];
    char back[32];
    int st[5];
    int n;
    st[0] = st[1] = st[2] = st[3] = st[4] = 0;
    n = HSD_SisLib_803A67EC(buf, (u8*) probe);
    Ui_SisTables(HSD_SisLib_8040C680, lbl_8040C8C0);
    Ui_SisDecode(buf, n + 1, back, sizeof back, st);
    OSReport("frontend: sis round trip %s (chars %d, unknown %d, bad %d)\n", strcmp(back, probe) == 0 ? "ok" : "FAILED", st[0], st[1], st[4]); /* OK-NOTEXT */
}

/* MELEE_ATLAS_TEXTPROBE=1: per source, how many strings, characters and glyphs that do not decode. COUNTS ONLY. tools/port/sis_probe.py does the same from a disc
 * image outside the game; this is the run-time check in the real archive and language. */
static void fad_probe_range(const char* source, int first, int count, int step)
{
    char buf[FAD_TEXT_PROBE];
    int st[5];
    int n = 0, c = 0, u = 0, k;
    for (k = 0; k < count; k++) {
        int len = 0;
        const u8* p = fad_sis_stream(first + k * step, &len);
        if (p == NULL) {
            continue;
        }
        st[0] = st[1] = st[2] = st[3] = st[4] = 0;
        Ui_SisDecode(p, len, buf, sizeof buf, st);
        n++;
        c += st[0];
        u += st[1];
    }
    OSReport("frontend: sis probe %s: %d strings, %d chars, %d unknown glyphs\n", source, n, c, u); /* OK-NOTEXT */
}

static void fad_text_probe(void)
{
    if (fad.probed || !Ui_EnvProbe()) {
        return;
    }
    fad.probed = true;
    fad_sis_selfcheck();
    if (!fad_sis_open()) {
        OSReport("frontend: sis probe: the text archive did not open\n");
        return;
    }
    fad_probe_range("event names", 0x154, FAD_EVENTS, 2);
    fad_probe_range("event descriptions", 0x155, FAD_EVENTS, 2);
    fad_probe_range("misc labels", 0xC9, 30, 1);
    fad_probe_range("sound names", 0xE7, 87, 1);
    fad_probe_range("bonus lines", 0x1BA, 90, 1);
    fad_probe_range("message bodies", 0x4BD, 69, 1);
    fad_sis_close();
}

/* a copy of text that fits cap, with "..." where it was cut (the host clamps again with its own field and does the same) */
static const char* fad_clip(const char* text, char* out, int cap)
{
    int n = (int) strlen(text);
    if (n < cap) {
        strcpy(out, text);
        return out;
    }
    strncpy(out, text, (size_t) (cap - 1));
    out[cap - 1] = '\0';
    if (cap > 4) {
        out[cap - 4] = '.';
        out[cap - 3] = '.';
        out[cap - 2] = '.';
    }
    return out;
}

/* ---- common row and explainer helpers --------------------------------------------------------------------------------------------------------------- */

static void fad_explain(int h, const char* kicker, const char* title, const char* what)
{
    Ui_SetExplain(h, kicker, title, what, "", "");
}

/* ---- Misc. Records: the 30 retail getters, our own labels (no retail text) ---------------------------------------------------------------------------- */

static int fad_misc_total(void)
{
    return 30; /* mnCount_row: POWER_COUNT .. DISASTER_MASTER */
}

static void fad_misc_row(int h, int slot, int i)
{
    char name[24];
    unsigned v = 0;
    name[0] = '\0';
    if (i >= 20) { /* LONGEST_TIME and up: a fighter, or nobody */
        int k = mnCount_GetRowValue_Character(i);
        if (k != SELKIND_COUNT) {
            fas_fighter_name(gm_SelKindToCKind((SelectableCharacterKind) k), name, sizeof name);
        }
    } else {
        v = mnCount_GetRowValue_Number(i);
    }
    Ui_RowMisc(h, slot, i, v, name);
}

static void fad_misc_explain(int h, int i)
{
    fad_explain(h, "MISC. RECORDS", i >= 0 ? Ui_MiscLabel(i) : "NOTHING HERE", "Counted from your save, as the retail screen does.");
}

static const char* fad_misc_keys(void) { return "B:Back"; }

/* ---- Event Match ---------------------------------------------------------------------------------------------------------------------------------- */

static char fad_ev_name[FAD_EVENTS][40];
static char fad_ev_desc[FAD_EVENTS][FAD_TEXT_ROOM];
static bool fad_ev_text_ok;
static int fad_ev_last;

static int fad_ev_total(void)
{
    return FAD_EVENTS;
}

static void fad_ev_open(void)
{
    int i;
    /* retail reaches display index (the page start bound) + 8; a debug build reaches all (mnEvent_8024CE74, mnevent.c) */
    fad_ev_last = Ui_EventLastOpen(mnEvent_8024CE74(), DbLevel > DbLKind_DebugDevelop);
    fad_ev_text_ok = fad_sis_open();
    for (i = 0; i < FAD_EVENTS; i++) {
        int ev = gm_801BEBA8((u8) i); /* the event at display index i: its name is SIS 0x154 + 2 * ev and its description the next one (mnevent.c) */
        fad_ev_name[i][0] = '\0';
        fad_ev_desc[i][0] = '\0';
        if (fad_ev_text_ok && i <= fad_ev_last) { /* a locked row never shows its name */
            if (!fad_sis_text(ev * 2 + 0x154, fad_ev_name[i], sizeof fad_ev_name[i])) {
                fad_ev_name[i][0] = '\0'; /* a name with a glyph we cannot show: the authored "EVENT n" */
            }
            if (!fad_sis_text(ev * 2 + 0x155, fad_ev_desc[i], sizeof fad_ev_desc[i])) {
                fad_ev_desc[i][0] = '\0';
            }
        }
    }
    /* the first entry opens on event 1, as retail; coming back from an event, on the one that was played (retail reopens its list there) */
    if (fad.ev_return || fad.reopen) {
        int sel = gm_801BEB80();
        fad.focus = sel >= 0 && sel < FAD_EVENTS ? sel : 0;
    } else {
        fad.focus = 0;
    }
    fad.ev_return = false;
    fad.reopen = false;
}

static void fad_ev_row(int h, int slot, int i)
{
    int flags = 0;
    unsigned best = 0;
    if (i > fad_ev_last) {
        flags = 2; /* locked */
    } else {
        if (gmMainLib_8015CEFC(i) != 0) {
            flags |= 1;
        }
        if (gm_801BEB8C((u8) i)) {
            flags |= 4;
        }
        best = (unsigned) gmMainLib_8015CF5C(i);
    }
    Ui_RowEvent(h, slot, i, flags, best, fad_ev_name[i]);
}

static void fad_ev_explain(int h, int i)
{
    char title[48];
    if (i < 0) {
        fad_explain(h, "EVENT MATCH", "NOTHING HERE", "");
    } else if (i > fad_ev_last) {
        sprintf(title, "EVENT %d", i + 1);
        fad_explain(h, "EVENT MATCH", title, "Clear more events to unlock this one.");
    } else {
        sprintf(title, "EVENT %d", i + 1);
        fad_explain(h, "EVENT MATCH", fad_ev_name[i][0] != '\0' ? fad_ev_name[i] : title,
                    fad_ev_desc[i][0] != '\0' ? fad_ev_desc[i] : "Pick an event and press A to start it.");
    }
}

/* A: the retail start sequence (mnEvent_8024D864): store the selection, say which port pressed A, and go to GM_EVENT. The screen stays up until the scene ends
 * (the menu does not flash back in for the frames the fade takes). */
static bool fad_ev_accept(int i)
{
    if (i < 0 || i >= FAD_EVENTS || i > fad_ev_last) {
        return false;
    }
    gm_801BEB74((u8) i);
    gm_801677E8((s8) fm_port_pressed());
    sfxForward();
    OSReport("frontend: Atlas event %d -> game mode %d\n", i + 1, GM_EVENT);
    fad.ev_return = true;
    fm.pend_kind = 2;
    fm.pend_a = GM_EVENT;
    fm.leave_at = fp.frame + 1;
    fad.leaving = true;
    return false; /* the screen is not closed here: the scene exit closes it */
}

static const char* fad_ev_keys(void) { return "A:Start,B:Back"; }

/* ---- Special Messages ----------------------------------------------------------------------------------------------------------------------------- */

static int fad_msg_n;
static char fad_msg_body[FAD_MSGS][FAD_TEXT_ROOM];
static bool fad_msg_text_ok;

static int fad_msg_total(void)
{
    return fad_msg_n;
}

static void fad_msg_open(void)
{
    int id;
    for (id = 0; id < FAD_MSGS; id++) {
        u32 date = 0;
        int un = mnInfo_80251A08(id) != 0;
        if (un) {
            date = *gmMainLib_8015D804(id);
        }
        Ui_MsgSet(id, un, date);
    }
    fad_msg_n = Ui_MsgOrder(FAD_MSGS);
    fad_msg_text_ok = fad_sis_open();
}

/* the body of the message at position k, decoded once, or "" when it cannot be shown */
static const char* fad_msg_text(int k)
{
    int id = Ui_MsgAt(k);
    s16 idx = 0;
    if (id < 0 || id >= FAD_MSGS || !fad_msg_text_ok) {
        return "";
    }
    if (fad_msg_body[id][0] == '\0') {
        un_802FE3F8(id, 0x4BD, &idx, NULL);
        if (!fad_sis_text((int) (u16) idx, fad_msg_body[id], sizeof fad_msg_body[id])) {
            fad_msg_body[id][0] = '\0';
            strncpy(fad_msg_body[id], "\1", 2); /* a marker: tried, cannot be shown */
        }
    }
    return fad_msg_body[id][0] == '\1' ? "" : fad_msg_body[id];
}

static void fad_msg_row(int h, int slot, int i)
{
    int id = Ui_MsgAt(i);
    int ymd = 0;
    if (id >= 0) {
        u32 secs = *gmMainLib_8015D804(id);
        if (secs != 0) {
            datetime dt;
            gm_801692E8(secs, &dt);
            ymd = dt.year * 10000 + dt.month * 100 + dt.day;
        }
    }
    Ui_RowMsg(h, slot, i, ymd);
}

static void fad_msg_explain(int h, int i)
{
    char title[24];
    const char* body;
    if (i < 0) {
        fad_explain(h, "SPECIAL MESSAGES", "NO MESSAGES YET", "Messages you unlock turn up here.");
        return;
    }
    sprintf(title, "MESSAGE %d", i + 1);
    body = fad_msg_text(i);
    fad_explain(h, "SPECIAL MESSAGES", title, body[0] != '\0' ? body : "This message cannot be shown here yet.");
}

static const char* fad_msg_keys(void) { return "B:Back"; }

/* ---- Bonus Records -------------------------------------------------------------------------------------------------------------------------------- */

static int fad_bonus_kind[FAD_BONUS_MAX];
static int fad_bonus_n;
static bool fad_bonus_text_ok;

static int fad_bonus_total(void)
{
    return fad_bonus_n;
}

static void fad_bonus_open(void)
{
    int i;
    fad_bonus_n = 0;
    for (i = 0; i < 0x100 && fad_bonus_n < FAD_BONUS_MAX; i++) {
        int kind = mnInfoBonus_PcKind(i);
        if (kind == 0x100) {
            break;
        }
        if (gm_8016F120(kind) != 0) { /* earned: the retail screen lists exactly these */
            fad_bonus_kind[fad_bonus_n++] = kind;
        }
    }
    fad_bonus_text_ok = fad_sis_open();
}

static void fad_bonus_row(int h, int slot, int i)
{
    Ui_RowBonus(h, slot, i);
}

static void fad_bonus_explain(int h, int i)
{
    char title[24];
    char what[FAD_TEXT_ROOM];
    char line[FAD_TEXT_ROOM];
    int base, k, used = 0;
    if (i < 0) {
        fad_explain(h, "BONUS RECORDS", "NO BONUSES YET", "Bonuses you earn turn up here.");
        return;
    }
    sprintf(title, "BONUS %d", i + 1);
    what[0] = '\0';
    base = ((u16) gm_8016F208(fad_bonus_kind[i]) - 2) * 3 + 0x1BA; /* the three lines the retail screen shows for this bonus (mninfobonus.c) */
    for (k = 0; k < 3 && fad_bonus_text_ok; k++) {
        if (fad_sis_text(base + k, line, sizeof line)) {
            if (used + (int) strlen(line) + 3 < (int) sizeof what) {
                strcat(what, used > 0 ? " - " : "");
                strcat(what, line);
                used = (int) strlen(what);
            }
        }
    }
    fad_explain(h, "BONUS RECORDS", title, what[0] != '\0' ? what : "A bonus you have earned.");
}

static const char* fad_bonus_keys(void) { return "B:Back"; }

/* ---- Sound Test: MUSIC (80 tracks) and EFFECTS (30 groups, left and right choose the sound) ------------------------------------------------------------ */

static struct {
    int tab;                      ///< 0 music, 1 effects
    int playing;                  ///< the music row that plays, -1 none
    int focus[2];                 ///< the focused row of each tab
    u8 sub[FAD_SND_GROUPS];       ///< the sound of each group the next A plays
    char name[FAD_SND_TRACKS][40];
    bool text_ok;
} fad_snd;

static int fad_snd_total(void)
{
    return fad_snd.tab == 0 ? FAD_SND_TRACKS : FAD_SND_GROUPS;
}

static int fad_snd_count(int group)
{
    return lbAudioAx_80023968((int) data_3[group]);
}

static void fad_snd_open(void)
{
    int i;
    lbAudioAx_80023694(); /* the retail think's first two calls: stop the effects and the music (fm_confirm no longer does it for this row) */
    lbAudioAx_800236DC();
    memset(&fad_snd, 0, sizeof fad_snd);
    fad_snd.playing = -1;
    fad_snd.text_ok = fad_sis_open();
    for (i = 0; i < FAD_SND_TRACKS; i++) {
        if (!fad_snd.text_ok || !fad_sis_text((int) data_2[text_ids[i]].text_id, fad_snd.name[i], sizeof fad_snd.name[i])) {
            fad_snd.name[i][0] = '\0';
        }
    }
}

/* B and a scene that ends: what the retail screen does on its way out (mnsoundtest.c fn_8024B2B0), then the menu music comes back */
static void fad_snd_close(void)
{
    lbAudioAx_80023694();
    gm_801603B0();
    lbAudioAx_80023F28(gmMainLib_8015ECB0());
}

static void fad_snd_row(int h, int slot, int i)
{
    if (fad_snd.tab == 0) {
        Ui_RowSound(h, slot, i, fad_snd.name[i], fad_snd.playing == i);
    } else {
        Ui_RowSfx(h, slot, i, fad_snd.sub[i], fad_snd_count(i));
    }
}

static void fad_snd_explain(int h, int i)
{
    if (fad_snd.tab == 0) {
        fad_explain(h, "SOUND TEST: MUSIC", i >= 0 && fad_snd.name[i][0] != '\0' ? fad_snd.name[i] : "MUSIC", "A plays the track, A again stops it. B leaves and the menu music returns.");
    } else {
        fad_explain(h, "SOUND TEST: EFFECTS", "SOUND GROUP", "Left and right choose the sound in the group, A plays it.");
    }
}

static bool fad_snd_accept(int i)
{
    if (fad_snd.tab == 0) {
        int act = Ui_SoundToggle(fad_snd.playing, i);
        if (act == 2) { /* AT_SND_STOP */
            lbAudioAx_800236DC();
            fad_snd.playing = -1;
        } else if (act == 1 || act == 3) { /* play, or switch to this row */
            lbAudioAx_80024614(gm_801601C4(gmMainLib_8015ED74())); /* the saved music volume, read not written */
            lbAudioAx_80023F28((s32) data_2[text_ids[i]].idx);
            if (lbAudioAx_80023090((int) data_2[text_ids[i]].idx) != 0) {
                gmMainLib_8015ED68((ssize_t) data_2[text_ids[i]].idx); /* as mnSoundTest_PlaySampleAnim does: the save remembers this track as heard (toy.c reads it for the Trophy Gallery music) */
            }
            fad_snd.playing = i;
        }
    } else if (i >= 0 && i < FAD_SND_GROUPS) {
        int kind = data_3[i];
        int sub = fad_snd.sub[i];
        int audio_id;
        lbAudioAx_800245F4(gm_80160244(gmMainLib_8015ED74())); /* the saved effects volume */
        audio_id = lbAudioAx_80023A44(kind, sub);
        lbAudioAx_80023694();
        lbAudioAx_80023B24(audio_id);
    }
    return false;
}

static void fad_snd_switch(int h, int tab)
{
    fad_snd.focus[fad_snd.tab] = fad.focus;
    fad_snd.tab = tab != 0;
    fad.focus = fad_snd.focus[fad_snd.tab];
    fad.first = 0;
    (void) h;
}

static bool fad_snd_event(int h, int type, int a)
{
    if (type == FAD_EV_ALT && a == 'X') {
        fad_snd_switch(h, !fad_snd.tab);
        sfxMove();
        return true;
    }
    if (type == FAD_EV_TAB) {
        fad_snd_switch(h, a);
        sfxMove();
        return true;
    }
    if (type == FAD_EV_MOVE && fad_snd.tab == 1 && (a == FSS_DIR_LEFT || a == FSS_DIR_RIGHT) && fad.focus >= 0 && fad.focus < FAD_SND_GROUPS) {
        int n = fad_snd_count(fad.focus), s = fad_snd.sub[fad.focus];
        if (n > 0) {
            s = a == FSS_DIR_RIGHT ? (s + 1) % n : (s + n - 1) % n;
            fad_snd.sub[fad.focus] = (u8) s;
            sfxMove();
        }
        return true;
    }
    return false;
}

static const char* fad_snd_keys(void)
{
    return fad_snd.tab == 0 ? "A:Play,X:Effects,B:Back" : "A:Play,X:Music,B:Back";
}

/* ---- VS. Records: rankings by fighter and by name tag, one stat at a time ----------------------------------------------------------------------------------- */

static struct {
    int mode;                       ///< 0 fighters, 1 tags
    int stat;                       ///< 0 .. FAD_STATS - 1
    int n;                          ///< ranked entries (non-zero value)
    u8 ent[FAD_TAG_SLOTS + 8];      ///< the entity (a fighter or a tag slot) at each rank
    u32 raw[FAD_TAG_SLOTS + 8];     ///< its raw value (the retail ranking's number)
    bool dirty;
} fad_vs;

static void fad_vs_rebuild(void)
{
    u8 who[FAD_TAG_SLOTS + 8];
    u32 val[FAD_TAG_SLOTS + 8];
    int i, m = 0, cnt;
    if (fad_vs.mode == 0) {
        for (i = 0; i < SELKIND_COUNT && m < FAD_TAG_SLOTS; i++) {
            u8 sk = mnDiagram_GetFighterByIndex(i);
            if (mn_IsFighterUnlocked(sk)) {
                int v = mnDiagram2_GetStatValue(0, (u8) fad_vs.stat, sk);
                who[m] = sk;
                val[m] = v > 0 ? (u32) v : 0;
                Ui_RankSet(m, val[m]);
                m++;
            }
        }
    } else {
        cnt = GetNameCount();
        for (i = 0; i < cnt && i < FAD_TAG_SLOTS; i++) {
            u8 slot = mnDiagram_GetNameByIndex(i);
            if (slot < FAD_TAG_SLOTS && GetNameText(slot) != NULL) {
                int v = mnDiagram2_GetStatValue(1, (u8) fad_vs.stat, slot);
                who[m] = slot;
                val[m] = v > 0 ? (u32) v : 0;
                Ui_RankSet(m, val[m]);
                m++;
            }
        }
    }
    fad_vs.n = Ui_RankBuild(m);
    for (i = 0; i < fad_vs.n; i++) {
        int e = Ui_RankAt(i);
        fad_vs.ent[i] = who[e];
        fad_vs.raw[i] = val[e];
    }
    fad_vs.dirty = false;
}

static int fad_vs_total(void)
{
    if (fad_vs.dirty) {
        fad_vs_rebuild();
    }
    return fad_vs.n;
}

static void fad_vs_open(void)
{
    fad_vs.mode = 0;
    fad_vs.stat = 0;
    fad_vs.dirty = true;
}

/* the name of the entity at rank k: a fighter's, a Latin tag's, or "TAG n" */
static void fad_vs_name(int k, char* out, int cap)
{
    out[0] = '\0';
    if (fad_vs.mode == 0) {
        fas_fighter_name(gm_SelKindToCKind((SelectableCharacterKind) fad_vs.ent[k]), out, cap);
    } else {
        const char* t = Ui_TagText((const unsigned char*) GetNameText(fad_vs.ent[k]));
        if (t[0] != '\0') {
            fad_clip(t, out, cap);
        } else {
            sprintf(out, "TAG %d", fad_vs.ent[k] + 1);
        }
    }
}

/* the number a stat shows for a raw value, and the flags the host's formatter needs (distance: converted, US units, the overflow mark) */
static void fad_vs_value(int stat, u32 raw, int* statflags, u32* shown)
{
    *statflags = stat;
    *shown = raw;
    if (Ui_StatKind(stat) == 4) { /* AT_STAT_DISTANCE */
        *shown = mnDiagram_ConvertDistanceForDisplay(raw);
        *statflags |= ((lbLang_IsSavedLanguageUS() ? 1 : 0) | (mnDiagram_IsDistanceOverflow(raw) ? 2 : 0)) << 8;
    }
}

static void fad_vs_row(int h, int slot, int i)
{
    char name[24];
    int sf;
    u32 shown;
    fad_vs_name(i, name, sizeof name);
    fad_vs_value(fad_vs.stat, fad_vs.raw[i], &sf, &shown);
    Ui_RowStat(h, slot, i, name, sf, shown);
}

static void fad_vs_explain(int h, int i)
{
    char kicker[48];
    char name[24];
    char what[FAD_TEXT_ROOM];
    sprintf(kicker, "%s: %s", fad_vs.mode == 0 ? "FIGHTERS" : "TAGS", Ui_StatLabel(fad_vs.stat));
    if (i < 0) {
        fad_explain(h, kicker, "NOTHING RANKED YET", "Play some matches and the rankings fill in. L and R change the stat.");
        return;
    }
    fad_vs_name(i, name, sizeof name);
    {
        int sf, sf2, sf3;
        u32 v1, v2, v3;
        u8 e = fad_vs.ent[i];
        int is_name = fad_vs.mode;
        int m = mnDiagram2_GetStatValue((u8) is_name, 8, e), w = mnDiagram2_GetStatValue((u8) is_name, 9, e), k = mnDiagram2_GetStatValue((u8) is_name, 0, e);
        fad_vs_value(8, m > 0 ? (u32) m : 0, &sf, &v1);
        fad_vs_value(9, w > 0 ? (u32) w : 0, &sf2, &v2);
        fad_vs_value(0, k > 0 ? (u32) k : 0, &sf3, &v3);
        sprintf(what, "Matches %s, wins %s, KOs %s.", Ui_StatText(sf, v1), Ui_StatText(sf2, v2), Ui_StatText(sf3, v3));
    }
    fad_explain(h, kicker, name, what);
}

static bool fad_vs_event(int h, int type, int a)
{
    if (type == FAD_EV_PAGE) {
        fad_vs.stat = (fad_vs.stat + FAD_STATS + (a < 0 ? -1 : 1)) % FAD_STATS;
        fad_vs.dirty = true;
        fad.focus = 0;
        fad.first = 0;
        sfxMove();
        return true;
    }
    if ((type == FAD_EV_ALT && a == 'X') || type == FAD_EV_TAB) {
        fad_vs.mode = type == FAD_EV_TAB ? (a != 0) : !fad_vs.mode;
        fad_vs.dirty = true;
        fad.focus = 0;
        fad.first = 0;
        sfxMove();
        return true;
    }
    (void) h;
    return false;
}

static const char* fad_vs_keys(void)
{
    return fad_vs.mode == 0 ? "L:Stat,R:Stat,X:Tags,B:Back" : "L:Stat,R:Stat,X:Fighters,B:Back";
}

/* ---- Name Entry: the tag list (the editor stays retail) --------------------------------------------------------------------------------------------------- */

static u8 fad_name_slot[FAD_TAG_SLOTS];
static int fad_name_n;   ///< the tags
static bool fad_name_new; ///< a "NEW TAG" row leads (the list is not full)

static int fad_name_total(void)
{
    return fad_name_n + (fad_name_new ? 1 : 0);
}

static void fad_name_open(void)
{
    int s;
    fad_name_n = 0;
    for (s = 0; s < FAD_TAG_SLOTS; s++) {
        if (GetNameText(s) != NULL) {
            fad_name_slot[fad_name_n++] = (u8) s;
        }
    }
    fad_name_new = !IsNameListFull();
}

static void fad_name_row(int h, int slot, int i)
{
    int k = i - (fad_name_new ? 1 : 0);
    if (k < 0) {
        Ui_RowTag(h, slot, -1, NULL, i);
    } else {
        Ui_RowTag(h, slot, fad_name_slot[k], (const unsigned char*) GetNameText(fad_name_slot[k]), i);
    }
}

static void fad_name_explain(int h, int i)
{
    char what[FAD_TEXT_ROOM];
    sprintf(what, "%d of %d name tags. A opens the game's own Name Entry to make, rename or delete one.", fad_name_n, FAD_TAG_SLOTS);
    fad_explain(h, "NAME ENTRY", i >= 0 && fad_name_new && i == 0 ? "NEW TAG" : "NAME TAGS", what);
}

/* A hands over to the retail Name Entry (the same native request the FA_NATIVE row makes): it makes, edits and deletes tags, and backs out to the menu */
static bool fad_name_accept(int i)
{
    (void) i;
    fm.pend_kind = 3;
    fm.pend_a = MENU_KIND_VS;
    fm.pend_b = SEL_VS_NAME;
    fm.leave_at = fp.frame + 1;
    fad.leaving = true;
    return false;
}

static const char* fad_name_keys(void) { return "A:Open Name Entry,B:Back"; }

/* ---- the table ------------------------------------------------------------------------------------------------------------------------------------ */

static const FadScreen fad_screens[] = {
    /* id         atlas id          title                kind               sel                    ch parents                          total          row          explain          accept          event         open          close        keys           word */
    { FD_EVENTS,   "data.events",   "EVENT MATCH",     MENU_KIND_1P,      SEL_1P_EVENT,          1, { "MAIN MENU", "SOLO" },       fad_ev_total,  fad_ev_row,  fad_ev_explain,  fad_ev_accept,  NULL,         fad_ev_open,  NULL,        fad_ev_keys,   "events" },
    { FD_NAME,     "data.name",     "NAME ENTRY",      MENU_KIND_VS,      SEL_VS_NAME,           2, { "MAIN MENU", "VERSUS" },     fad_name_total, fad_name_row, fad_name_explain, fad_name_accept, NULL,       fad_name_open, NULL,       fad_name_keys, "name" },
    { FD_SOUND,    "data.sound",    "SOUND TEST",      MENU_KIND_DATA,    SEL_DATA_SOUND,        0, { "MAIN MENU", "DATA" },       fad_snd_total, fad_snd_row, fad_snd_explain, fad_snd_accept, fad_snd_event, fad_snd_open, fad_snd_close, fad_snd_keys, "sound" },
    { FD_MESSAGES, "data.messages", "SPECIAL MESSAGES", MENU_KIND_DATA,   SEL_DATA_SPECIAL,      0, { "MAIN MENU", "DATA" },       fad_msg_total, fad_msg_row, fad_msg_explain, NULL,           NULL,         fad_msg_open, NULL,        fad_msg_keys,  "messages" },
    { FD_BONUS,    "data.bonus",    "BONUS RECORDS",   MENU_KIND_RECORDS, SEL_RECORDS_BONUS,     0, { "DATA", "RECORDS" },         fad_bonus_total, fad_bonus_row, fad_bonus_explain, NULL,       NULL,         fad_bonus_open, NULL,      fad_bonus_keys, "bonus" },
    { FD_MISC,     "data.misc",     "MISC. RECORDS",   MENU_KIND_RECORDS, SEL_RECORDS_MISC,      0, { "DATA", "RECORDS" },         fad_misc_total, fad_misc_row, fad_misc_explain, NULL,          NULL,         NULL,         NULL,        fad_misc_keys, "misc" },
    { FD_VSREC,    "data.vsrec",    "VS. RECORDS",     MENU_KIND_RECORDS, SEL_RECORDS_VS,        0, { "DATA", "RECORDS" },         fad_vs_total,  fad_vs_row,  fad_vs_explain,  NULL,           fad_vs_event, fad_vs_open,  NULL,        fad_vs_keys,   "vsrec" },
};

static const FadScreen* fad_find(int id)
{
    int i;
    for (i = 0; i < (int) (sizeof fad_screens / sizeof fad_screens[0]); i++) {
        if (fad_screens[i].id == id) {
            return &fad_screens[i];
        }
    }
    return NULL;
}

static const FadScreen* fad_for(u8 kind, u8 sel)
{
    int i;
    for (i = 0; i < (int) (sizeof fad_screens / sizeof fad_screens[0]); i++) {
        if (fad_screens[i].kind == kind && fad_screens[i].sel == sel) {
            return &fad_screens[i];
        }
    }
    return NULL;
}

/* a plain substring test (the MSL string.h here has no strstr) */
static bool fad_has(const char* hay, const char* needle)
{
    int n = (int) strlen(needle), i;
    for (i = 0; hay[i] != 0; i++) {
        if (strncmp(hay + i, needle, (size_t) n) == 0) {
            return true;
        }
    }
    return false;
}

static void fad_read_env(void)
{
    const char* e;
    int i;
    bool any = false;
    if (fad.env_read) {
        return;
    }
    fad.env_read = true;
    e = Ui_EnvData(); /* the string of MELEE_ATLAS_DATA, "" when unset */
    for (i = 1; i < FD_COUNT; i++) { /* a list with no screen word in it (empty, "all", or only "notext") means every screen */
        const FadScreen* s = fad_find(i);
        if (s != NULL && fad_has(e, s->word)) {
            any = true;
        }
    }
    for (i = 1; i < FD_COUNT; i++) {
        const FadScreen* s = fad_find(i);
        if (s != NULL && (!any || fad_has(e, "all") || fad_has(e, s->word)) && !fad_has(e, "none")) {
            fad.enabled |= 1u << i;
        }
    }
}

/* a data screen takes this (kind, selection) when Atlas is on, no netplay session is up and the screen is on */
static bool fad_wants(u8 kind, u8 sel)
{
    const FadScreen* s;
    if (!Ui_Ready() || Ui_NetplayActive()) {
        return false;
    }
    fad_read_env();
    s = fad_for(kind, sel);
    return s != NULL && (fad.enabled & (1u << s->id)) != 0;
}

/* the Solo menu's EVENT MATCH row is on: coming back from an event reopens the list (gmfrontend_menus.inc fm_position_for) */
static bool fad_events_wanted(void)
{
    if (!fad_wants(MENU_KIND_1P, SEL_1P_EVENT)) {
        return false;
    }
    fad.reopen = true;
    return true;
}

static void fad_close_now(void)
{
    const FadScreen* s = fad_find(fad.id);
    if (s != NULL && s->close != NULL) {
        s->close();
    }
    if (fad.h >= 0) {
        Ui_SetClose(fad.h);
    }
    fad_sis_close();
    fad.id = FD_NONE;
    fad.h = -1;
    fad.leaving = false;
}

/* fm_confirm calls this before it runs an FA_NATIVE row: true when a data screen took the row */
static bool fad_open_for(u8 kind, u8 sel)
{
    const FadScreen* s;
    int h;
    if (fad.id != FD_NONE || !fad_wants(kind, sel)) {
        return false;
    }
    s = fad_for(kind, sel);
    h = Ui_DataOpen(s->atlas_id, s->parent[0], s->parent[1], s->title, s->chapter);
    if (h < 0) {
        return false; /* the retail screen runs: no roles, another native screen up, no slot */
    }
    fad.id = s->id;
    fad.h = h;
    fad.focus = 0;
    fad.first = 0;
    fad.leaving = false;
    fad_text_probe();
    if (s->open != NULL) {
        s->open();
    }
    if (fat.shown != NULL) { /* the data screen replaces the menu's host screen; the menu's own comes back when this closes */
        Ui_Close(fat.shown);
        fat.shown = NULL;
    }
    OSReport("frontend: Atlas data screen %s\n", s->atlas_id); /* OK-NOTEXT */
    return true;
}

/* B, or a screen that is done: back to the menu, on the item that opened it (the menu never changed) */
static void fad_back(void)
{
    sfxBack();
    fad_close_now();
}

/* every frame the screen is up and takes input: the game's menu input as the host's intents, then the host's events */
static void fad_feed_pad(u32 in)
{
    if (in & MenuInput_Up) {
        Ui_SetIntent(fad.h, FSS_IN_MOVE, FSS_DIR_UP);
    }
    if (in & MenuInput_Down) {
        Ui_SetIntent(fad.h, FSS_IN_MOVE, FSS_DIR_DOWN);
    }
    if (in & MenuInput_Left) {
        Ui_SetIntent(fad.h, FSS_IN_MOVE, FSS_DIR_LEFT);
    }
    if (in & MenuInput_Right) {
        Ui_SetIntent(fad.h, FSS_IN_MOVE, FSS_DIR_RIGHT);
    }
    if (in & MenuInput_Confirm) {
        Ui_SetIntent(fad.h, FSS_IN_ACCEPT, 0);
    }
    if (in & MenuInput_Back) {
        Ui_SetIntent(fad.h, FSS_IN_BACK, 0);
    }
    if (in & MenuInput_LTrigger) {
        Ui_SetIntent(fad.h, FSS_IN_PAGE, -1);
    }
    if (in & MenuInput_RTrigger) {
        Ui_SetIntent(fad.h, FSS_IN_PAGE, 1);
    }
    if (in & MenuInput_XButton) {
        Ui_SetIntent(fad.h, FSS_IN_ALT, 'X');
    }
    if (in & MenuInput_YButton) {
        Ui_SetIntent(fad.h, FSS_IN_ALT, 'Y');
    }
}

static void fad_event(const FadScreen* s, int type, int slot, int arg)
{
    switch (type) {
    case FAD_EV_FOCUS:
        if (slot >= 0) {
            int to = fad.first + slot;
            if (to != fad.focus && to >= 0 && to < fad.total) {
                fad.focus = to;
                sfxMove();
            }
        }
        break;
    case FAD_EV_MOVE:
        if (s->event != NULL && s->event(fad.h, type, arg)) {
            break;
        }
        if (fad.total > 0 && (arg == FSS_DIR_UP || arg == FSS_DIR_DOWN)) {
            if (slot == 1) { /* the mouse wheel (the host sends slot 1; keys and the pad send -1): it stops at the ends, as the other lists' wheel does */
                int to = arg == FSS_DIR_UP ? fad.focus - 1 : fad.focus + 1;
                if (to >= 0 && to < fad.total) {
                    fad.focus = to;
                    sfxMove();
                }
            } else {
                fad.focus = arg == FSS_DIR_UP ? (fad.focus > 0 ? fad.focus - 1 : fad.total - 1) : (fad.focus < fad.total - 1 ? fad.focus + 1 : 0); /* the pad and keys wrap over the whole list */
                sfxMove();
            }
        }
        break;
    case FAD_EV_ACCEPT: {
        int row = fad.first + slot;
        if (slot >= 0 && row >= 0 && row < fad.total) {
            fad.focus = row;
            if (s->accept != NULL && s->accept(row)) {
                fad_close_now();
            }
        }
        break;
    }
    case FAD_EV_BACK:
        fad_back();
        break;
    default:
        if (s->event != NULL) {
            (void) s->event(fad.h, type, arg);
        }
        break;
    }
}

/* Returns true when a data screen is up and took this frame's input (the legacy pad block is skipped). `pad` is the game's menu input of every port, merged. */
static bool fad_frame(u32 pad)
{
    const FadScreen* s;
    int type, slot, arg, guard = 0;
    if (fad.id == FD_NONE) {
        return false;
    }
    s = fad_find(fad.id);
    if (s == NULL || fad.h < 0) {
        fad_close_now();
        return false;
    }
    if (fad.leaving) {
        return true; /* a scene change is under way: the screen waits for it */
    }
    fad_feed_pad(pad);
    while (fad.id != FD_NONE && !fad.leaving && guard++ < 16 && (type = Ui_SetPoll(fad.h, &slot, &arg)) != 0) {
        fad_event(s, type, slot, arg);
        if (type == FAD_EV_ACCEPT || type == FAD_EV_BACK) {
            break; /* one action per frame, as the legacy path; what is left in the ring is stale */
        }
    }
    return true;
}

/* the window and its rows, the focus, the counter, the explainer and the key hints, every frame the screen is up. Returns true when it is: the menu's own
 * host screen is not submitted then. Also opens the event list again when the player is back from an event. */
static bool fad_sync(void)
{
    const FadScreen* s;
    int n, i;
    char ctr[24];
    char id[FAD_ID_ROOM];
    if (fad.id == FD_NONE) {
        if (fad.reopen && fm.menu != NULL && fm.menu->kind != MENU_KIND_1P) {
            fad.reopen = false; /* the position went elsewhere: never open the list later from a stale request */
        }
        if (fad.reopen && fm.menu != NULL && fm.menu->kind == MENU_KIND_1P && fm.leave_at == 0 && !fm.fading_out && fm.frames >= fm.ready_at && fm.nvis > 0) {
            if (!fad_open_for(MENU_KIND_1P, SEL_1P_EVENT)) {
                fad.reopen = false; /* could not open: the Solo menu stays */
            }
        }
        if (fad.id == FD_NONE) {
            return false;
        }
    }
    s = fad_find(fad.id);
    if (s == NULL || fad.h < 0) {
        fad_close_now();
        return false;
    }
    fad.total = s->total();
    if (fad.focus >= fad.total) {
        fad.focus = fad.total > 0 ? fad.total - 1 : 0;
    }
    if (fad.focus < 0) {
        fad.focus = 0;
    }
    fad.first = Ui_DataFirst(fad.total, Ui_DataWindow(), fad.focus, fad.first);
    n = fad.total - fad.first;
    if (n > Ui_DataWindow()) {
        n = Ui_DataWindow();
    }
    if (n < 0) {
        n = 0;
    }
    Ui_SetRows(fad.h, n);
    for (i = 0; i < n; i++) {
        s->row(fad.h, i, fad.first + i);
    }
    if (n > 0) {
        sprintf(id, "r%d", fad.focus);
        Ui_SetFocus(fad.h, id);
    }
    if (fad.total > 0) {
        sprintf(ctr, "%d / %d", fad.focus + 1, fad.total);
    } else {
        ctr[0] = '\0';
    }
    Ui_DataCounter(fad.h, ctr);
    if (s->id == FD_VSREC) {
        Ui_SetTabs(fad.h, 2, "FIGHTERS,TAGS", fad_vs.mode);
    } else if (s->id == FD_SOUND) {
        Ui_SetTabs(fad.h, 2, "MUSIC,EFFECTS", fad_snd.tab);
    }
    s->explain(fad.h, fad.total > 0 ? fad.focus : -1);
    Ui_SetKeys(fad.h, s->keys());
    return true;
}

static void fad_scene_exit(void)
{
    if (fad.id != FD_NONE) {
        const FadScreen* s = fad_find(fad.id);
        if (fad.id == FD_SOUND) {
            lbAudioAx_80023694(); /* the scene is ending (L+R+START, a mode): stop the effects as the retail path does; the next scene chooses its own music */
        } else if (s != NULL && s->close != NULL) {
            s->close(); /* the host closes the screen with the scene (Ui_SceneExit) */
        }
        fad_sis_close();
    }
    fad.id = FD_NONE;
    fad.h = -1;
    fad.leaving = false;
}

/* ---- Results (a REPLACE stand-in, off unless MELEE_ATLAS_SCENES=5:replace) --------------------------------------------------------------------------------- */

/* THE SIDE-EFFECT TABLE of gm_Scene_Results_OnEnter and what runs it (gmresult.c, gmresultplayer.c), read before this was written. tools/port/test_fe_atlas_results.py
 * reads the next comment block and checks that every audio and state name occurs below.
 *
 * RESULTS SIDE EFFECTS:
 *   lbAudioAx_80023F28 audio     the winner's victory theme at enter (not when cancelled): fn_80160400(ckind)
 *   lbAudioAx_800237A8 audio     frame 2: 0xC355 and 0x144 (cancelled: 0x148); frame 0xA2: 0xB5 (not when cancelled); frame 0x9A when cancelled: 0xC350
 *   fn_80168E54 audio            frame 0x9A, not cancelled: the winner's announcer call
 *   lb_80014574 state            rumble on the winner's pad: a human winner, not cancelled, x3_b6 set
 *   gm_SetDbPauseFlag state      frame 0, when the results were entered from a paused debug match (ResultsMatchInfo x0_0): the pause bit is cleared
 *   gm_801A4B60 state            the exit, once every human has confirmed and the countdown ran (the mode's next state decides what follows)
 *   fn_801701AC state            gm_Scene_Results_OnExit: clears the 1P results flag (lbl_804D65A0.x0); runs from the scene's on_exit
 *   un_802FF1B4 presentation     loads IfCoGet.dat for the KO tally effect; its only readers are un_802FF128 (the tally) and gmregclear.c: dropped
 *   fn_80176A6C presentation     the results camera;  fn_80176F60 presentation: panels;  fn_8017AA78, fn_8017A004, fn_8017A9B4, fn_8017A67C, fn_8017A318: the 3D winner scene
 *   fn_801771C0 presentation     computes the winner index for the retail pages; the stand-in reads winners[] and team_winners[] itself
 */
static int far_frames;
static int far_wait;
static int far_card_frames; ///< frames since the card appeared: retail starts the exit countdown only after the card's frame 50 (fn_80178050)
static int far_h = -1;
static bool far_started;
static bool far_cancel;
static int far_winner = -1;
static s8 far_win_ck, far_win_ft;
static u8 far_win_team;
static u8 far_is_teams;
static bool far_pause_bit;
static bool far_rumble;

static bool far_is_winner(const MatchEnd* me, int i)
{
    int k;
    for (k = 0; k < me->n_winners && k < GM_MAX_PLAYERS; k++) {
        if (me->winners[k] == i) {
            return true;
        }
    }
    if (me->is_teams == 1) {
        for (k = 0; k < me->n_team_winners && k < GM_MAX_TEAMS; k++) {
            if (me->team_winners[k] == me->player_standings[i].team) {
                return true;
            }
        }
    }
    return false;
}

/* the scene's on_enter: what the retail enter does that is not presentation, then the card model from the summary. The host screen opens on the first frame,
 * inside the scene loop (the host closes a screen with the scene it was opened in; on_enter runs before that scene is current). */
static void far_enter(void* arg)
{
    ResultsMatchInfo* mi = arg;
    const MatchEnd* me = &mi->match_end;
    int i, j;
    far_frames = 0;
    far_wait = 0;
    far_card_frames = 0;
    far_h = -1;
    far_started = false;
    far_winner = -1;
    far_cancel = gm_WasMatchCanceled(me->outcome) != 0;
    far_is_teams = me->is_teams;
    far_pause_bit = mi->x0_0 != 0;
    far_rumble = false;
    Ui_ResBegin(me->outcome, me->is_teams == 1, far_cancel);
    for (i = 0; i < 4; i++) {
        const MatchPlayerData* p = &me->player_standings[i];
        int falls = 0;
        char name[24];
        if (p->pkind == 3) {
            continue;
        }
        for (j = 0; j < 4; j++) {
            if (me->player_standings[j].pkind != 3) {
                falls += me->player_standings[j].kills[i]; /* how often i was KO'd (a self-destruct is kills[i][i]) */
            }
        }
        name[0] = '\0';
        fas_fighter_name(p->ckind, name, sizeof name);
        Ui_ResPlayer(i, p->pkind, p->ckind, p->stocks, (int) p->x20, falls);
        Ui_ResPlayer2(i, p->percent, p->team, far_is_winner(me, i) ? 1 : 0, name);
    }
    far_winner = gmResult_PcWinner((MatchEnd*) me); /* the retail winner (x6): the announcer, the victory theme and the rumble follow it, as in gm_Scene_Results_OnEnter */
    if (far_winner >= 0 && far_winner < 4 && me->player_standings[far_winner].pkind != 3) {
        const MatchPlayerData* w = &me->player_standings[far_winner];
        far_win_ck = w->ckind;
        far_win_ft = w->ftkind;
        far_win_team = w->team;
        far_rumble = w->pkind == Gm_PKind_Human && !far_cancel && w->x3_b6;
    } else {
        far_winner = -1;
    }
    Ui_ResCommit();
    if (!far_cancel && far_winner >= 0) {
        lbAudioAx_80023F28(fn_80160400(far_win_ck)); /* the winner's victory theme */
    }
    if (far_rumble) {
        lb_80014574((u8) far_winner, 3, 0x20, 0);
    }
}

/* the scene's frame: the retail timeline's audio at the same frame numbers, the per-human confirm, the exit */
static void far_frame(void)
{
    int i;
    if (far_pause_bit && far_frames == 0) {
        gm_SetDbPauseFlag(0);
    }
    if (far_frames == 0x2) {
        if (!far_cancel) {
            lbAudioAx_800237A8(0xC355, 0x7F, 0x40);
            lbAudioAx_800237A8(0x144, 0x7F, 0x40);
        } else {
            lbAudioAx_800237A8(0x148, 0x7F, 0x40);
        }
    } else if (far_frames == 0x9A) {
        if (far_cancel) {
            lbAudioAx_800237A8(0xC350, 0x7F, 0x40);
        } else if (far_winner >= 0) {
            fn_80168E54(far_win_ck, far_win_ft, far_win_team, far_is_teams == 1);
        }
    } else if (far_frames == 0xA2 && !far_cancel) {
        lbAudioAx_800237A8(0xB5, 0x7F, 0x40);
    }
    far_frames++;
    if (!far_started) {
        far_started = true;
        far_h = Ui_DataOpen("data.results", "", "", "RESULTS", 1);
        if (far_h < 0) {
            OSReport("frontend: Atlas results could not open its screen: leaving at once\n");
            gm_801A4B60();
            return;
        }
        Ui_SetKeys(far_h, "S:Ready");
    }
    Ui_ResFill(far_h);
    /* retail: START per human player, or a pad error counts as confirmed (fn_80178050); a cancelled match leaves on any human's START (fn_801791E4).
       The first 90 frames ignore START unless the match was cancelled: a button still held from the match must not skip the card. */
    if (far_cancel || far_frames > 90) {
        for (i = 0; i < 4; i++) {
            if (HSD_PadCopyStatus[i].err != 0) {
                Ui_ResConfirm(i, 2);
            } else if (HSD_PadCopyStatus[i].trigger & PAD_BUTTON_START) {
                Ui_ResConfirm(i, 1);
            }
        }
    }
    {
        int done = Ui_ResDone();
        far_card_frames++;
        if (done >= 0 && far_card_frames > 50) {
            if (far_wait++ >= done) {
                gm_801A4B60(); /* the retail exit: the mode's next state decides what follows */
            }
        }
    }
    Ui_SetExplain(far_h, "RESULTS", far_winner >= 0 ? "MATCH OVER" : "NO WINNER", far_cancel ? "The match was cancelled. START leaves." : "Every player presses START to continue.", "", "");
}

static void far_exit(void* unused)
{
    (void) unused;
    if (far_h >= 0) {
        Ui_SetClose(far_h);
        far_h = -1;
    }
    fn_801701AC(); /* gm_Scene_Results_OnExit */
}

static GameScene fad_results_scene = { GS_RESULTS, far_frame, far_enter, far_exit, NULL };

/* the REPLACE stand-in for the results scene: only when Atlas is on and no netplay session is up (a netplay match's results stay retail) */
static GameScene* fad_results_standin(void)
{
    if (!Ui_Ready() || Ui_NetplayActive()) {
        return NULL;
    }
    return &fad_results_scene;
}
