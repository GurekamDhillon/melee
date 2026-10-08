/* gmfrontend_atlas_online.inc - included by gmfrontend.c (TARGET_PC only) after gmfrontend_online.inc.
 *
 * The Atlas drawing of the online room screens (docs/superpowers/plans/2026-10-06-atlas-step6-online-room.md). DRAWING ONLY.
 * Every rule about what a screen shows or accepts is still a gmfrontend_online.inc predicate (fl_phase, fl_my_stage_turn,
 * fl_can_pick_char, fl_reconnecting, fl_card_portrait, fl_stage_open, fl_instruction, fl_phase_title): this file asks them and
 * copies the answers into the host's room view by name (pc/platform/gw_script_ui_room.inc). It READS netplay state (the Netplay_
 * calls below are the read-only set tools/port/check_atlas_online.sh allows); every write stays in gmfrontend_online.inc. Pad input is
 * not touched: fl_frame ORs the retail pad read with fa_room_bits() where it used to OR fms_menu_bits(), and fa_room_bits() turns the
 * mouse and keyboard intents into the same MenuInput_ bits a pad press has.
 *
 * "ON" MEANS ATTACHED. fa_room_on() is true only while the host has a room screen attached: with MELEE_ATLAS=0, missing Atlas fonts, another
 * native screen up or no free slot, Ui_RoomBegin refuses and every legacy path below runs exactly as it did. */

extern int Ui_RoomOpen(void);
extern int Ui_RoomBegin(const char* kind);
extern void Ui_RoomEnd(void);
extern int Ui_RoomSetInt(const char* key, int v);
extern int Ui_RoomSetStr(const char* key, const char* s);
extern void Ui_RoomPlayer(int who, int present, int locked, int ready, int score);
extern void Ui_RoomPlayerName(int who, const char* name, const char* fighter);
extern void Ui_RoomStageCount(int n);
extern void Ui_RoomStage(int i, const char* name, const char* state, int starter, int open);
extern int Ui_RoomGridFor(int n, int starters);
extern int Ui_RoomGridCell(int i, int* col, int* row, int* page);
extern void Ui_RoomFrame(void);
extern void Ui_RoomFreeze(int on);
extern int Ui_RoomPollName(char* out, int cap, int* arg);

_Static_assert(LB_MAX_UI_STAGES == 32, "pc/platform/gw_ui_room.h AT_ROOM_STAGES must be LB_MAX_UI_STAGES");

static bool fa_room_on(void) { return Ui_RoomOpen() != 0; }

/* The one way a room screen is detached: the host's screen goes, and a code-slot click that was never applied goes with it. */
static void fa_room_end(void)
{
    fa_code_slot_req = -1;
    Ui_RoomEnd();
}

static void fa_room_begin(int art)
{
    Ui_RoomBegin(art == FL_CODE ? "code" : art == FL_WAIT ? "wait" : "lobby");
}

static const char* fa_phase_name(int p)
{
    switch (p) {
    case LBP_CHAR_BLIND: return "char_blind";
    case LBP_STRIKE: return "strike";
    case LBP_BAN: return "ban";
    case LBP_PICK: return "pick";
    case LBP_CHAR_WINNER: return "char_winner";
    case LBP_CHAR_LOSER: return "char_loser";
    case LBP_READY: return "ready";
    case LBP_GO: return "go";
    default: return "off";
    }
}

static const char* fa_stage_state(int s)
{
    switch (s) {
    case LBS_P1: return "p1";
    case LBS_P2: return "p2";
    case LBS_BANNED: return "banned";
    case LBS_PICKED: return "picked";
    default: return "free";
    }
}

/* The grid model for the cursor code: the legacy cursor functions (fl_step_row, fl_step_open, fl_step_page) are unchanged and read flg;
 * under Atlas flg comes from the same function that draws the tiles. A tile is an 80 x 80 box on a 100 px pitch, so "the nearest row
 * that way" means the nearest row of the grid on screen. flg.n and flg.starters are already set by the caller. */
static void fa_room_grid(int n, int starters)
{
    int i, col, row, page;
    if (flg.n > LB_MAX_UI_STAGES) {
        flg.n = LB_MAX_UI_STAGES; /* the host's list stops there */
        n = flg.n;
    }
    flg.kind = FL_GRID_FLOW;
    flg.pages = Ui_RoomGridFor(n, starters);
    flg.nhead = 0;
    for (i = 0; i < n; i++) {
        if (!Ui_RoomGridCell(i, &col, &row, &page)) {
            continue;
        }
        flg.page[i] = page;
        flg.r[i][0] = (float) (col * 100);
        flg.r[i][1] = (float) (row * 100);
        flg.r[i][2] = flg.r[i][0] + 80.0F;
        flg.r[i][3] = flg.r[i][1] + 80.0F;
    }
}

/* What a card may say about a fighter. The legacy card decided with fl_card_portrait; so does this. The opponent's blind pick
 * is never read here until it is "shown" (check_atlas_online.sh guard 7). */
static void fa_room_fighter(int w, char* out, int cap)
{
    const char* pt = fl_card_portrait(w);
    if (strcmp(pt, "shown") != 0) {
        sprintf(out, "%s", strcmp(pt, "hidden") == 0 ? "Locked in" : "Choosing...");
        return;
    }
    Netplay_FighterName(fl_ck(w), out, cap);
}

static void fa_room_players(bool lobby)
{
    char nm[FE_STR], fi[FE_STR];
    int w, me = Netplay_LobbyMe(), found = fe_np_phase == FE_NP_LOBBY || fe_np_phase == FE_NP_CONNECTED;
    for (w = 0; w < 2; w++) {
        Netplay_PlayerName(w, nm, 18);
        fi[0] = '\0';
        if (lobby) {
            fa_room_fighter(w, fi, 24);
        }
        Ui_RoomPlayerName(w, nm, fi);
        Ui_RoomPlayer(w, w == me || found, Netplay_LobbyPlayer(w, 2) != 0, Netplay_LobbyPlayer(w, 3) != 0, Netplay_LobbyInfo(2 + w));
    }
}

static void fa_room_common(void)
{
    char b[FE_STR];
    const char* t = fl_toast_now();
    bool pref = Netplay_IsHost() && fe_np_phase != FE_NP_LOBBY;
    Netplay_RoomCode(b, sizeof b);
    Ui_RoomSetStr("code", b);
    Ui_RoomSetInt("host", Netplay_IsHost() != 0);
    Ui_RoomSetInt("me", Netplay_LobbyMe());
    Ui_RoomSetInt("ping", fl_reconnecting() ? -1 : Netplay_Ping());
    Netplay_MenuStatus(b, FE_STR);
    if (Netplay_EnvoyBlock()[0] != '\0' && fe_np_phase == FE_NP_LOBBY) { /* Envoy cannot start: the reason stays on the status line */
        fp_copy(b, "Envoy: ", FE_STR);
        fp_copy(b + 7, Netplay_EnvoyBlock(), FE_STR - 7);
    }
    Ui_RoomSetStr("status", b);
    Ui_RoomSetInt("turbo", pref ? Netplay_TurboPref() != 0 : Netplay_Turbo() != 0);
    Ui_RoomSetInt("envoy", pref ? Netplay_EnvoyPref() != 0 : Netplay_Envoy() != 0);
    Ui_RoomSetInt("stage_all", Netplay_StageMode() != 0);
    Ui_RoomSetInt("stocks", fe_np_stocks);
    Ui_RoomSetInt("minutes", fe_np_minutes);
    Ui_RoomSetInt("delay", fe_np_delay);
    Ui_RoomSetInt("rules_known", Netplay_IsHost() != 0); /* a guest sees the rules in the lobby, not before */
    Ui_RoomSetStr("toast", t != NULL ? t : "");
    Ui_RoomSetInt("toast_bad", fl.toast_bad);
    Ui_RoomSetInt("reconnecting", fl_reconnecting());
}

/* The whole room view, once a frame. The host replaces its screen record from it (Ui_RoomFrame). */
static void fa_room_submit(void)
{
    char b[FE_STR];
    int i, n, s = 0;
    if (!fa_room_on()) {
        fa_room_begin(fl.art); /* fl_open attached it; this only retries after a refusal (a re-open clears the host's ring, so it is never called per frame) */
        if (!fa_room_on()) {
            return;
        }
    }
    fa_room_common();
    if (fl.art == FL_CODE) {
        for (i = 0; i < 4; i++) {
            b[i] = (char) Netplay_CodeChar(i);
        }
        b[4] = '\0';
        Ui_RoomSetStr("code_chars", b);
        Ui_RoomSetInt("code_slot", Netplay_CodeSlot());
        Ui_RoomSetInt("code_invalid", fl.invalid);
        Ui_RoomSetStr("code_msg", fl.code_msg);
        Ui_RoomSetInt("code_msg_bad", fl.code_msg_bad);
    } else if (fl.art == FL_WAIT) {
        fa_room_players(false);
        Ui_RoomSetInt("random", Netplay_RandomStatus() == 1);
        Ui_RoomSetInt("random_secs", Netplay_RandomSeconds());
        Ui_RoomSetInt("copied", fl.frames < fl.copied_until);
        Ui_RoomSetInt("found", fe_np_phase == FE_NP_LOBBY);
    } else if (fl.art == FL_LOBBY) {
        int p = fl_phase(), me = Netplay_LobbyMe();
        fa_room_players(true);
        fl_grid_layout(); /* under Atlas: the host's grid model; also sets flg.n and flg.starters */
        n = flg.n;
        s = flg.starters;
        Ui_RoomSetStr("phase", fa_phase_name(p));
        Ui_RoomSetStr("phase_title", fl_phase_title());
        fl_instruction(b);
        Ui_RoomSetStr("instruction", b);
        Ui_RoomSetInt("game", Netplay_LobbyInfo(0));
        Ui_RoomSetInt("turn", Netplay_LobbyInfo(4));
        Ui_RoomSetInt("left", Netplay_LobbyInfo(5));
        Ui_RoomSetInt("first", Netplay_LobbyInfo(6));
        Ui_RoomSetInt("countdown", Netplay_LobbyInfo(8));
        Ui_RoomSetInt("coin", fl.frames < fl.coin_until);
        Ui_RoomSetInt("reward_open", Netplay_LobbyInfo(13) != 0);
        Ui_RoomSetInt("reward_left", (Netplay_LobbyInfo(14) + 59) / 60);
        Ui_RoomSetInt("leave_armed", fl.frames < fl.leave_until);
        Ui_RoomSetInt("my_stage_turn", fl_my_stage_turn());
        Ui_RoomSetInt("can_pick", fl_can_pick_char());
        Ui_RoomSetInt("can_ready", p == LBP_READY);
        Ui_RoomSetInt("ready_mine", Netplay_LobbyPlayer(me, 3) != 0);
        Ui_RoomSetInt("cursor", fl.cur);
        Ui_RoomStageCount(n);
        for (i = 0; i < n && i < LB_MAX_UI_STAGES; i++) {
            Ui_RoomStage(i, fl_stage_name(i), fa_stage_state(Netplay_LobbyStage(i)), Netplay_LobbyStageGroup(i) == 0, fl_stage_open(i));
        }
        Ui_RoomGridFor(n, s);
    }
    Ui_RoomSetInt("fade_out", 0);
    Ui_RoomFrame();
}

/* A screen that is leaving: the legacy fade is drawn by the game under the host's quads, so the host fades its own screen out. */
static void fa_room_leaving(void)
{
    Ui_RoomFreeze(1); /* a leaving screen takes no input: the host drops its ring */
    Ui_RoomSetInt("fade_out", FE_FADE_FRAMES > 0 ? (1000 * fe.fade) / FE_FADE_FRAMES : 1000);
    Ui_RoomFrame();
}

/* Mouse and keyboard intents become the bits a pad press has, or a local cursor move the legacy mouse code already made. The
 * pad itself never comes through here. One line per intent, so check_atlas_online.sh can read the table. */
static u32 fa_room_bits(void)
{
    char n[16];
    int arg;
    u32 bits = 0;
    while (Ui_RoomPollName(n, sizeof n, &arg)) {
        if (strcmp(n, "up") == 0) bits |= MenuInput_Up;
        else if (strcmp(n, "down") == 0) bits |= MenuInput_Down;
        else if (strcmp(n, "left") == 0) bits |= MenuInput_Left;
        else if (strcmp(n, "right") == 0) bits |= MenuInput_Right;
        else if (strcmp(n, "accept") == 0) bits |= MenuInput_Confirm | MenuInput_AButton; /* what the retail read sets for A */
        else if (strcmp(n, "back") == 0) bits |= MenuInput_Back;
        else if (strcmp(n, "start") == 0) bits |= MenuInput_StartButton;
        else if (strcmp(n, "copy") == 0) bits |= MenuInput_XButton;
        else if (strcmp(n, "paste") == 0) bits |= MenuInput_YButton;
        else if (strcmp(n, "page_l") == 0) bits |= MenuInput_LTrigger;
        else if (strcmp(n, "page_r") == 0) bits |= MenuInput_RTrigger;
        else if (strcmp(n, "stage_at") == 0) {
            if (fl.art == FL_LOBBY && fl_my_stage_turn() && arg >= 0 && arg < Netplay_LobbyInfo(9) && fl_stage_open(arg)) {
                if (fl.cur != arg) {
                    sfxMove();
                }
                fl.cur = arg; /* the legacy mouse set the cursor the same way, then confirmed */
            }
        } else if (strcmp(n, "stage_click") == 0) {
            /* a click: the cursor there AND the confirm, applied together or not at all. Not on my stage turn (it ended this frame), or a stage that is not
               open, it does nothing: the legacy cursor would move to another stage and the confirm would strike THAT one. */
            if (fl.art == FL_LOBBY && fl_my_stage_turn() && arg >= 0 && arg < Netplay_LobbyInfo(9) && fl_stage_open(arg)) {
                if (fl.cur != arg) {
                    sfxMove();
                }
                fl.cur = arg;
                bits |= MenuInput_Confirm;
            }
        } else if (strcmp(n, "code_slot") == 0) {
            if (fl.art == FL_CODE) {
                fa_code_slot_req = arg; /* acted on in fl_frame_code, where the legacy arrows move the slot: the adapter writes nothing */
            }
        }
    }
    return bits;
}
