/* gw_script_ui.inc - gd.ui: Atlas screens for scripts (docs/scripting.md "gd.ui"; the design is
 * docs/superpowers/specs/2026-10-06-menu-reunification-design.md).
 *
 * Included by gw_script.c after gs_prof_setfuncs is declared. The pure Atlas units (gw_ui_*.c) own layout, focus,
 * input events and the parts; this file is the adapter: Lua table -> screen record, pad/keyboard/mouse -> events,
 * events -> Lua handlers, the sink -> the gd.kit quad list.
 *
 * ONLINE. A screen is presentation: it draws and takes local input, nothing here writes the simulation. The pad is
 * never masked from here (masking stays the mod's own call, and is refused online). Handlers do not run on a resimulated
 * frame (gs_may_run). Mouse and keyboard stay local, as gd.mouse and gd.key do.
 *
 * TRACKING. The kit's letter-spacing (gw_Kit_SetTracking) is set only inside gs_ui_width and gs_ui_text, for the
 * roles the tokens mark as tracked, and set back to 0 straight after each call. gw_Kit_BeginFrame also clears it,
 * so a value can never reach another script's text or the legacy menus. The kit adds the spacing after the last
 * glyph too (a width is n x tracking wider), so gs_ui_width reports the visible width and gs_ui_text shifts a
 * centred or right-aligned run by half or all of one gap to put the visible glyphs where the parts asked. */
#include "gw_ui_render.h"
#include "gw_ui_stack.h"

extern int gw_Settings_Int(const char *key, int dflt); /* gw_settings.c:138; the Reduced Motion row writes "reduced_motion" */

#define GS_UI_SLOTS 8

typedef struct {
    int used, owner, explain_dirty, dialog_fn, warned, render_warned;
    AtScreen sc;
    AtView view;
    AtHits hits;
    AtPad pad;
    AtKeys keys;
    AtMouse mouse;
} GsUiSlot;

static GsUiSlot gs_ui_slot[GS_UI_SLOTS];
static AtStack gs_ui_stack;
static AtvArena gs_ui_arena;
static AtScreen gs_ui_scratch;
static int gs_ui_role_idx[AT_R_COUNT];
static int gs_ui_roles_state; /* 0 not tried, 1 every Atlas role found, -1 missing */
static struct { double sum, max; int frames, quads; float canvas_w; } gs_ui_stat;

/* ---- fonts: the Atlas roles by name, resolved once ------------------------------------------------ */
static int gs_ui_roles_ok(void) {
    int r;
    if (gs_ui_roles_state != 0) return gs_ui_roles_state > 0;
    gs_ui_roles_state = -1;
    if (!gw_Kit_Available()) return 0;
    for (r = 0; r < AT_R_COUNT; r++) {
        gs_ui_role_idx[r] = gw_Kit_Role(at_role(r)->name);
        if (gs_ui_role_idx[r] < 0) {
            gw_log("ui: the font manifest has no Atlas role \"%s\" (rebuild the kit fonts: menu/pipeline/font_atlas.py)", at_role(r)->name);
            return 0;
        }
    }
    gs_ui_roles_state = 1;
    gw_log("ui: Atlas text roles found (%d)", AT_R_COUNT);
    return 1;
}

static float gs_ui_track(int role) { return at_role(role)->tracked ? 0.10f * (float) at_role(role)->size : 0.0f; }

/* the visible width: the kit's own width counts the spacing after the last glyph as well */
static float gs_ui_width(void *u, int role, const char *s) {
    float w, t = gs_ui_track(role);
    (void) u;
    gw_Kit_SetTracking(t);
    w = gw_Kit_TextWidth(gs_ui_role_idx[role], s);
    gw_Kit_SetTracking(0.0f);
    if (t > 0.0f && s != NULL && s[0] != '\0' && w >= t) w -= t;
    return w;
}

static void gs_ui_poly(void *u, const float x[4], const float y[4], unsigned rgba) {
    (void) u;
    gw_Kit_DrawPoly4(x, y, rgba);
}

static void gs_ui_text(void *u, float x, float base, const char *s, int role, unsigned rgba, int align, float max_w) {
    float t = gs_ui_track(role);
    (void) u;
    (void) max_w;
    if (align == AT_ALIGN_CENTER) x += t * 0.5f;       /* the kit aligns on a width that includes one trailing gap */
    else if (align == AT_ALIGN_RIGHT) x += t;
    gw_Kit_SetTracking(t);
    gw_Kit_DrawText(x, base, s, gs_ui_role_idx[role], rgba, align, 0.0f, 0.0f, NULL);
    gw_Kit_SetTracking(0.0f);
}

/* a model handle (a gd.model_load token) to its asset slot, or -1 when it is stale: a cell then draws nothing */
static int gs_ui_model_asset(int token) {
    int i;
    for (i = 0; i < gs_stage_nmodels; ++i)
        if (gs_stage_models[i].token == token && gw_ScriptGame_ModelRefOwned(i, token, gw_Script_StageResourceOwner(), 0)) return i;
    return -1;
}

static void gs_ui_model_draw(void *u, int model, int ring, float x, float y, float w, float h, int focused, int dim) {
    GsmParams p;
    const char *why = "";
    int a;
    (void) u;
    memset(&p, 0, sizeof p);
    p.x = x; p.y = y; p.w = w; p.h = h;
    p.yaw = 18.0f; p.pitch = 12.0f;                             /* the bag's pose (envoy run_screen.lua S.model_opts) */
    p.spin = (focused && !gw_Settings_Int("reduced_motion", 0)) ? 120.0f : 0.0f;
    p.clock = (float) gs_ui_clock;
    p.margin = 0.02f; p.dim = dim ? 0.35f : 1.0f; p.alpha = 1.0f; p.tint = 0xFFFFFFFFu; p.cull = 0;
    if (ring >= 0 && (a = gs_ui_model_asset(ring)) >= 0) (void) gs_sm_emit(a, &p, 1, &why);   /* the rarity ring first: it passes behind the body */
    if (model >= 0 && (a = gs_ui_model_asset(model)) >= 0) (void) gs_sm_emit(a, &p, 1, &why);
}

/* ---- Lua table -> the value tree; functions become registry references ------------------------------ */
/* Every way this can lose part of a description sets gs_ui_arena.overflow (the arena's own constructors set it when
 * a node, an entry or the string pool is full; a table nested past the depth limit sets it here), and l_ui_screen
 * refuses the description when it is set: nothing is dropped silently. A value of a kind the tree has no node for
 * (nil, userdata, a thread) is not part of a description and is skipped, as a nil field always is. */
static int gs_ui_copy(lua_State *L, int idx, int depth) {
    int t;
    idx = lua_absindex(L, idx);
    t = lua_type(L, idx);
    if (depth > 8) { gs_ui_arena.overflow = 1; return -1; }
    if (t == LUA_TBOOLEAN) return atv_bool(&gs_ui_arena, lua_toboolean(L, idx));
    if (t == LUA_TNUMBER) return atv_num(&gs_ui_arena, lua_tonumber(L, idx));
    if (t == LUA_TSTRING) return atv_str(&gs_ui_arena, lua_tostring(L, idx));
    if (t == LUA_TFUNCTION) {
        int ref, n;
        lua_pushvalue(L, idx);
        ref = luaL_ref(L, LUA_REGISTRYINDEX);
        n = atv_fn(&gs_ui_arena, ref);
        if (n < 0) luaL_unref(L, LUA_REGISTRYINDEX, ref);
        return n;
    }
    if (t == LUA_TTABLE) {
        int n = atv_table(&gs_ui_arena), len = (int) lua_rawlen(L, idx), i;
        if (n < 0) return -1;
        for (i = 1; i <= len; i++) {
            int v;
            lua_rawgeti(L, idx, i);
            v = gs_ui_copy(L, -1, depth + 1);
            lua_pop(L, 1);
            if (v >= 0) atv_push(&gs_ui_arena, n, v);
            if (gs_ui_arena.overflow) return n;
        }
        lua_pushnil(L);
        while (lua_next(L, idx)) {
            if (lua_type(L, -2) == LUA_TSTRING) {
                int v = gs_ui_copy(L, -1, depth + 1);
                if (v >= 0) atv_set(&gs_ui_arena, n, lua_tostring(L, -2), v);
            }
            lua_pop(L, 1);
            if (gs_ui_arena.overflow) { lua_pop(L, 1); return n; }   /* drop the key: lua_next is abandoned */
        }
        return n;
    }
    return -1;
}

static void gs_ui_unref_arena_except(lua_State *L, const int *keep, int nkeep) {
    int i, k;
    for (i = 0; i < gs_ui_arena.nn; i++) {
        int ref, kept = 0;
        if (gs_ui_arena.node[i].kind != ATV_FN) continue;
        ref = gs_ui_arena.node[i].fn;
        for (k = 0; k < nkeep; k++) if (keep[k] == ref) kept = 1;
        if (!kept) luaL_unref(L, LUA_REGISTRYINDEX, ref);
    }
}

static void gs_ui_unref_screen(lua_State *L, const AtScreen *s) {
    int refs[40], n = at_screen_fn_refs(s, refs, 40), i;
    for (i = 0; i < n; i++) luaL_unref(L, LUA_REGISTRYINDEX, refs[i]);
}

static int gs_ui_find(const char *id) {
    int i;
    for (i = 0; i < GS_UI_SLOTS; i++) if (gs_ui_slot[i].used && strcmp(gs_ui_slot[i].sc.id, id) == 0) return i;
    return -1;
}

/* the mod id of the running script (the part of "envoy/main" before the slash); NULL for the console */
static const char *gs_ui_owner_mod(char *buf, int cap) {
    const GsScript *s = gs_cur_script();
    char *slash;
    if (s == NULL || gs.cur == gs.console) return NULL;
    snprintf(buf, (size_t) cap, "%s", s->id);
    slash = strchr(buf, '/');
    if (slash != NULL) *slash = '\0';
    return buf;
}

/* ---- calling back into Lua ------------------------------------------------------------------------- */
/* Call a stored function with (cell id, block id). Returns 1 when it ran; nres results are then on the stack. */
static int gs_ui_call(int slot, int ref, const char *a, const char *b, int nres) {
    lua_State *L = gs.L;
    int owner = gs_ui_slot[slot].owner;
    if (ref < 0 || !gs_may_run(owner)) return 0;
    lua_rawgeti(L, LUA_REGISTRYINDEX, ref);
    if (!lua_isfunction(L, -1)) { lua_pop(L, 1); return 0; }
    lua_pushstring(L, a != NULL ? a : "");
    lua_pushstring(L, b != NULL ? b : "");
    return gs_pcall(owner, 2, nres, "ui") == 0;
}

static void gs_ui_close_screen(int slot);
static int gs_ui_open_screen(int slot);

/* a handler may return {pop = true} or {push = "screen id"} */
static void gs_ui_apply_result(void) {
    lua_State *L = gs.L;
    if (lua_istable(L, -1)) {
        lua_getfield(L, -1, "pop");
        if (lua_toboolean(L, -1) && at_stack_top(&gs_ui_stack) >= 0) gs_ui_close_screen(at_stack_top(&gs_ui_stack));
        lua_pop(L, 1);
        lua_getfield(L, -1, "push");
        if (lua_type(L, -1) == LUA_TSTRING) {
            int s = gs_ui_find(lua_tostring(L, -1));
            if (s >= 0) gs_ui_open_screen(s);
        }
        lua_pop(L, 1);
    }
    lua_pop(L, 1);
}

/* re-run the explainer provider, the key label and `when` functions and the counter for the focused cell */
static void gs_ui_refresh_view(int slot) {
    GsUiSlot *u = &gs_ui_slot[slot];
    lua_State *L = gs.L;
    const char *cid = at_screen_cell_id(&u->sc, u->view.focus), *bid = at_screen_block_id(&u->sc, u->view.focus.block);
    char err[96], cell[AT_ID], block[AT_ID];
    int i;
    u->explain_dirty = 0;
    snprintf(cell, sizeof cell, "%s", cid != NULL ? cid : "");     /* a handler may re-register the screen and move these */
    snprintf(block, sizeof block, "%s", bid != NULL ? bid : "");
    cid = cell[0] ? cell : NULL;
    bid = block[0] ? block : NULL;
    if (u->sc.fn_provide >= 0 && cid != NULL && gs_ui_call(slot, u->sc.fn_provide, cid, bid, 1)) {
        int n;
        atv_init(&gs_ui_arena);
        n = gs_ui_copy(L, -1, 0);
        if (gs_ui_arena.overflow) {
            gw_log("ui: %s: the explainer provider returned too large a table; showing none", u->sc.id);
            at_explainer_from_val(&gs_ui_arena, -1, &u->view.ex, err, sizeof err);
        } else {
            at_explainer_from_val(&gs_ui_arena, n, &u->view.ex, err, sizeof err);
        }
        gs_ui_unref_arena_except(L, NULL, 0);
        lua_pop(L, 1);
        if (u->view.ex.warn) gw_log("ui: %s: an explainer string was cut to its field", u->sc.id);
    } else {
        atv_init(&gs_ui_arena);
        at_explainer_from_val(&gs_ui_arena, -1, &u->view.ex, err, sizeof err);
    }
    for (i = 0; i < u->sc.n_keys; i++) {
        AtKey *k = &u->sc.keys[i];
        int show = 1;
        snprintf(u->view.key_label[i], AT_STR, "%s", k->label);
        if (k->fn_when >= 0) {
            show = 0;
            if (gs_ui_call(slot, k->fn_when, cid, bid, 1)) { show = lua_toboolean(L, -1); lua_pop(L, 1); }
        }
        if (show && k->fn_label >= 0) {
            show = 0;
            if (gs_ui_call(slot, k->fn_label, cid, bid, 1)) {
                if (lua_type(L, -1) == LUA_TSTRING) { snprintf(u->view.key_label[i], AT_STR, "%s", lua_tostring(L, -1)); show = 1; }
                lua_pop(L, 1);
            }
        }
        u->view.key_shown[i] = (unsigned char) (show && u->view.key_label[i][0] != '\0');
    }
    if (u->sc.fn_counter >= 0) {
        u->view.counter[0] = '\0';
        if (gs_ui_call(slot, u->sc.fn_counter, cid, bid, 1)) {
            if (lua_type(L, -1) == LUA_TSTRING) snprintf(u->view.counter, AT_STR, "%s", lua_tostring(L, -1));
            lua_pop(L, 1);
        }
    } else {
        snprintf(u->view.counter, AT_STR, "%s", u->sc.counter);
    }
}

/* ---- stack operations --------------------------------------------------------------------------------- */
static int gs_ui_open_screen(int slot) {
    GsUiSlot *u = &gs_ui_slot[slot];
    AtStackEntry *top;
    if (!at_stack_push(&gs_ui_stack, slot)) return 0;
    top = at_stack_top_entry(&gs_ui_stack);
    if (top != NULL && top->focus.block >= 0 && u->view.focus.block < 0) u->view.focus = top->focus;
    u->view.opened_ms = gs_now_ms();
    u->view.port_rgba = 0;
    u->explain_dirty = 1;
    memset(&u->pad, 0, sizeof u->pad);
    memset(&u->keys, 0, sizeof u->keys);
    memset(&u->mouse, 0, sizeof u->mouse);
    gw_log("ui: screen %s open (%d blocks, %d items)", u->sc.id, u->sc.n_blocks, u->sc.n_items);
    if (u->sc.fn_open >= 0 && gs_ui_call(slot, u->sc.fn_open, "", "", 1)) gs_ui_apply_result();
    return 1;
}

static void gs_ui_close_screen(int slot) {
    GsUiSlot *u = &gs_ui_slot[slot];
    AtStackEntry *top = at_stack_top_entry(&gs_ui_stack);
    if (top != NULL && top->screen == slot) { top->focus = u->view.focus; top->scroll = u->view.scroll; }
    if (!at_stack_remove(&gs_ui_stack, slot)) return;
    if (u->sc.fn_close >= 0 && gs_ui_call(slot, u->sc.fn_close, "", "", 1)) lua_pop(gs.L, 1);
    gw_log("ui: screen %s closed", u->sc.id);
}

/* a script unloaded: its screens and their references go */
static void gs_ui_release(int script) {
    int i;
    for (i = 0; i < GS_UI_SLOTS; i++) {
        GsUiSlot *u = &gs_ui_slot[i];
        if (!u->used || u->owner != script) continue;
        at_stack_remove(&gs_ui_stack, i);
        gs_ui_unref_screen(gs.L, &u->sc);
        if (u->dialog_fn >= 0) luaL_unref(gs.L, LUA_REGISTRYINDEX, u->dialog_fn);
        memset(u, 0, sizeof *u);
    }
}

/* ---- events ----------------------------------------------------------------------------------------- */
static void gs_ui_dialog_press(int slot, int btn) {
    GsUiSlot *u = &gs_ui_slot[slot];
    lua_State *L = gs.L;
    int i, hit = 0;
    for (i = 0; i < u->view.dialog.n; i++) if (u->view.dialog.btn[i] == btn) hit = 1;
    if (!hit) return;
    u->view.dialog.open = 0;
    if (u->dialog_fn >= 0 && gs_may_run(u->owner)) {
        char one[2];
        one[0] = (char) btn; one[1] = '\0';
        lua_rawgeti(L, LUA_REGISTRYINDEX, u->dialog_fn);
        if (lua_isfunction(L, -1)) { lua_pushstring(L, one); (void) gs_pcall(u->owner, 1, 0, "ui:dialog"); } else lua_pop(L, 1);
    }
}

static void gs_ui_event(int slot, const AtEvent *e) {
    GsUiSlot *u = &gs_ui_slot[slot];
    AtFocusBlock fb[AT_MAX_BLOCKS];
    AtFocusPos before = u->view.focus;
    const char *cid, *bid;
    char cell[AT_ID], block[AT_ID];
    int nb, alt;
    if (u->view.dialog.open) {                                    /* a dialog takes every event until it is answered */
        if (e->type == AT_EV_ACCEPT) gs_ui_dialog_press(slot, 'A');
        else if (e->type == AT_EV_BACK) gs_ui_dialog_press(slot, 'B');
        else if (e->type == AT_EV_ALT) gs_ui_dialog_press(slot, e->a);
        return;
    }
    nb = at_screen_focus_blocks(&u->sc, fb);
    if (e->type == AT_EV_MOVE) u->view.focus = at_focus_move(fb, nb, u->view.focus, e->a, 1);
    else if (e->type == AT_EV_SCROLL && u->sc.primary == AT_PRIMARY_LIST) u->view.focus = at_focus_move(fb, nb, u->view.focus, e->a < 0 ? AT_DIR_UP : AT_DIR_DOWN, 0);
    else if (e->type == AT_EV_FOCUS) {
        AtFocusPos p;
        p.block = e->a; p.index = e->b;
        if (at_screen_cell_id(&u->sc, p) != NULL) u->view.focus = p;
    }
    if (u->sc.primary == AT_PRIMARY_LIST && u->view.focus.index >= 0) {
        int visible = (int) ((362.0f - 28.0f) / 39.0f);
        u->view.scroll = at_list_scroll(u->view.focus.index, u->view.scroll, visible, u->sc.n_items);
    }
    cid = at_screen_cell_id(&u->sc, u->view.focus);
    bid = at_screen_block_id(&u->sc, u->view.focus.block);
    snprintf(cell, sizeof cell, "%s", cid != NULL ? cid : "");
    snprintf(block, sizeof block, "%s", bid != NULL ? bid : "");
    cid = cell[0] ? cell : NULL;
    bid = block[0] ? block : NULL;
    if (u->view.focus.block != before.block || u->view.focus.index != before.index) {
        u->explain_dirty = 1;
        if (u->sc.fn_focus >= 0 && cid != NULL && gs_ui_call(slot, u->sc.fn_focus, cid, bid, 1)) lua_pop(gs.L, 1);
        return;
    }
    if (e->type == AT_EV_ACCEPT) {
        if (u->sc.fn_accept >= 0 && cid != NULL && at_cell_accepts(&u->sc, u->view.focus) && gs_ui_call(slot, u->sc.fn_accept, cid, bid, 1)) gs_ui_apply_result();
    } else if (e->type == AT_EV_BACK) {
        if (u->sc.fn_back >= 0 && gs_ui_call(slot, u->sc.fn_back, cid, bid, 1)) gs_ui_apply_result();
    } else if (e->type == AT_EV_ALT) {
        alt = e->a == 'X' ? 0 : e->a == 'Y' ? 1 : 2;
        if (u->sc.fn_alt[alt] >= 0 && gs_ui_call(slot, u->sc.fn_alt[alt], cid, bid, 1)) gs_ui_apply_result();
    }
}

/* once per script tick, before on_tick: the top screen's input, then its explainer refresh */
static void gs_ui_tick(void) {
    int top = at_stack_top(&gs_ui_stack), n, i;
    AtEvent ev[16];
    double now = gs_now_ms();
    GsUiSlot *u;
    if (top < 0) return;
    u = &gs_ui_slot[top];
    if (!u->used) { at_stack_remove(&gs_ui_stack, top); return; }
    if (!gs_may_run(u->owner)) return;
    if (at_screen_wants_pad(&u->sc)) {
        unsigned b;
        int sx, sy, cx, cy, tl, tr;
        gw_script_pad_state(u->sc.port - 1, &b, &sx, &sy, &cx, &cy, &tl, &tr);
        b = gw_script_pad_raw_buttons(u->sc.port - 1);
        n = at_pad_events(&u->pad, b, sx, sy, now, ev, 16);
        for (i = 0; i < n && at_stack_top(&gs_ui_stack) == top; i++) gs_ui_event(top, &ev[i]);
    }
    if (at_stack_top(&gs_ui_stack) != top) return;
    {
        unsigned m = 0;
        if (gs.key_now[VK_UP]) m |= AT_KEY_UP;
        if (gs.key_now[VK_DOWN]) m |= AT_KEY_DOWN;
        if (gs.key_now[VK_LEFT]) m |= AT_KEY_LEFT;
        if (gs.key_now[VK_RIGHT]) m |= AT_KEY_RIGHT;
        if (gs.key_now[VK_RETURN]) m |= AT_KEY_ENTER;
        if (gs.key_now[VK_ESCAPE]) m |= AT_KEY_ESC;
        if (gs.key_now[VK_TAB]) m |= AT_KEY_TAB;
        if (gs.key_now[VK_SHIFT]) m |= AT_KEY_SHIFT;
        n = at_key_events(&u->keys, m, now, ev, 16);
        for (i = 0; i < n && at_stack_top(&gs_ui_stack) == top; i++) gs_ui_event(top, &ev[i]);
    }
    if (at_stack_top(&gs_ui_stack) != top) return;
    {
        float x, y, wheel;
        int buttons;
        gw_Mouse_ScriptRead(&x, &y, &buttons, &wheel);
        n = at_mouse_events(&u->mouse, x, y, buttons, (int) wheel, &u->hits, ev, 16);
        for (i = 0; i < n && at_stack_top(&gs_ui_stack) == top; i++) gs_ui_event(top, &ev[i]);
    }
    if (at_stack_top(&gs_ui_stack) == top && u->explain_dirty) gs_ui_refresh_view(top);
}

/* the draw pass: called from gs_finish_draw after the scripts' on_draw hooks */
static void gs_ui_draw(void) {
    int top = at_stack_top(&gs_ui_stack), first, old, quads;
    LARGE_INTEGER t0, t1, fq;
    AtTextOps ops;
    AtSink sink;
    AtRenderInfo info;
    GsUiSlot *u;
    double ms;
    if (top < 0 || !gs_ui_roles_ok()) return;
    u = &gs_ui_slot[top];
    if (!u->used) return;
    if (gw_Kit_QuadRoom() < AT_SCREEN_QUAD_CAP) {
        if (!u->warned) { u->warned = 1; gw_log("ui: %s not drawn: only %d quads left this frame", u->sc.id, gw_Kit_QuadRoom()); }
        return;
    }
    old = gs.cur;
    gs.cur = u->owner;
    QueryPerformanceCounter(&t0);
    ops.width = gs_ui_width; ops.user = NULL;
    sink.user = NULL; sink.poly = gs_ui_poly; sink.text = gs_ui_text; sink.model = gs_ui_model_draw;
    memset(&info, 0, sizeof info);
    first = gw_Kit_QuadCount();
    gs_ui_stat.canvas_w = gw_Console_ScriptWidth();
    at_render_ex(&u->sc, &u->view, gs_ui_stat.canvas_w, gs_now_ms(), gw_Settings_Int("reduced_motion", 0) != 0, &ops, &sink, &u->hits, &info);
    gw_Kit_SetTracking(0.0f);                                     /* belt and braces: no draw leaves tracking set */
    quads = gw_Kit_QuadCount() - first;
    gs_kit_record(first, quads);
    QueryPerformanceCounter(&t1);
    QueryPerformanceFrequency(&fq);
    ms = 1000.0 * (double) (t1.QuadPart - t0.QuadPart) / (double) fq.QuadPart;
    gs_ui_stat.quads = quads;
    gs_ui_stat.sum += ms; gs_ui_stat.frames++;
    if (ms > gs_ui_stat.max) gs_ui_stat.max = ms;
    if ((info.warned || info.capped || info.hits_dropped) && !u->render_warned) {   /* once per screen, never per frame */
        u->render_warned = 1;
        gw_log("ui: %s render budget: %d entries (warning line %d, cap %d)%s%s%s", u->sc.id, info.entries, AT_SCREEN_QUAD_WARN, AT_SCREEN_QUAD_CAP,
               info.capped ? "; the cap was reached and later entries were dropped" : "",
               info.dropped > 0 ? "; entries dropped" : "",
               info.hits_dropped > 0 ? "; some hit rectangles did not fit (the mouse cannot reach them)" : "");
    }
    gs.cur = old;
}

/* ---- the Lua functions -------------------------------------------------------------------------------- */
static int l_ui_available(lua_State *L) {
    int ok = gs_ui_roles_ok();
    lua_pushboolean(L, ok);
    lua_pushstring(L, ok ? "" : (gw_Kit_Available() ? "the Atlas text roles are missing from the font manifest" : gw_Kit_Why()));
    return 2;
}

static int l_ui_screen(lua_State *L) {
    char err[200], owner[64];
    int root, slot, nkeep, keep[40], replace = 0;
    GsUiSlot *u;
    AtFocusPos old;
    char old_block[AT_ID], old_cell[AT_ID];
    luaL_checktype(L, 1, LUA_TTABLE);
    atv_init(&gs_ui_arena);
    root = gs_ui_copy(L, 1, 0);
    if (root < 0 || gs_ui_arena.overflow) {
        gs_ui_unref_arena_except(L, NULL, 0);
        return luaL_error(L, "gd.ui.screen: the description is too large or nested too deeply");
    }
    if (!at_screen_from_val(&gs_ui_arena, root, gs_ui_owner_mod(owner, sizeof owner), &gs_ui_scratch, err, sizeof err)) {
        gs_ui_unref_arena_except(L, NULL, 0);
        return luaL_error(L, "%s", err);
    }
    nkeep = at_screen_fn_refs(&gs_ui_scratch, keep, 40);
    gs_ui_unref_arena_except(L, keep, nkeep);                     /* every reference the screen does not keep goes */
    slot = gs_ui_find(gs_ui_scratch.id);
    if (slot < 0) {
        int i;
        for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) { slot = i; break; }
        if (slot < 0) {
            gs_ui_unref_screen(L, &gs_ui_scratch);
            return luaL_error(L, "gd.ui.screen: too many screens (%d)", GS_UI_SLOTS);
        }
    }
    u = &gs_ui_slot[slot];
    if (u->used) {
        if (u->owner != gs.cur) { gs_ui_unref_screen(L, &gs_ui_scratch); return luaL_error(L, "gd.ui.screen: \"%s\" belongs to another script", u->sc.id); }
        replace = 1;
        old = u->view.focus;
        snprintf(old_block, sizeof old_block, "%s", at_screen_block_id(&u->sc, old.block) != NULL ? at_screen_block_id(&u->sc, old.block) : "");
        snprintf(old_cell, sizeof old_cell, "%s", at_screen_cell_id(&u->sc, old) != NULL ? at_screen_cell_id(&u->sc, old) : "");
        gs_ui_unref_screen(L, &u->sc);
    } else {
        memset(u, 0, sizeof *u);
        at_view_init(&u->view);
        u->dialog_fn = -1;
    }
    u->sc = gs_ui_scratch;
    u->used = 1;
    u->owner = gs.cur;
    u->render_warned = 0;
    if (replace) u->view.focus = at_screen_refocus(&u->sc, old_block[0] ? old_block : NULL, old_cell[0] ? old_cell : NULL, old);
    else { AtFocusBlock fb[AT_MAX_BLOCKS]; u->view.focus = at_focus_first(fb, at_screen_focus_blocks(&u->sc, fb)); }
    u->explain_dirty = 1;
    lua_pushboolean(L, 1);
    return 1;
}

static int gs_ui_slot_arg(lua_State *L, int idx) {
    int s = gs_ui_find(luaL_checkstring(L, idx));
    if (s < 0) luaL_error(L, "gd.ui: no screen \"%s\"", lua_tostring(L, idx));
    return s;
}

static int l_ui_open(lua_State *L) { lua_pushboolean(L, gs_ui_open_screen(gs_ui_slot_arg(L, 1))); return 1; }

static int l_ui_close(lua_State *L) {
    int s = lua_isstring(L, 1) ? gs_ui_slot_arg(L, 1) : at_stack_top(&gs_ui_stack);
    if (s >= 0) gs_ui_close_screen(s);
    lua_pushboolean(L, s >= 0);
    return 1;
}

static int l_ui_feed(lua_State *L) {
    int s = gs_ui_slot_arg(L, 1);
    const char *k = luaL_checkstring(L, 2);
    AtEvent e;
    memset(&e, 0, sizeof e);
    if (strcmp(k, "up") == 0) { e.type = AT_EV_MOVE; e.a = AT_DIR_UP; }
    else if (strcmp(k, "down") == 0) { e.type = AT_EV_MOVE; e.a = AT_DIR_DOWN; }
    else if (strcmp(k, "left") == 0) { e.type = AT_EV_MOVE; e.a = AT_DIR_LEFT; }
    else if (strcmp(k, "right") == 0) { e.type = AT_EV_MOVE; e.a = AT_DIR_RIGHT; }
    else if (strcmp(k, "accept") == 0) e.type = AT_EV_ACCEPT;
    else if (strcmp(k, "back") == 0) e.type = AT_EV_BACK;
    else if (strcmp(k, "x") == 0 || strcmp(k, "y") == 0 || strcmp(k, "z") == 0) { e.type = AT_EV_ALT; e.a = k[0] - 'a' + 'A'; }
    else return luaL_error(L, "gd.ui.feed: unknown intent \"%s\" (up down left right accept back x y z)", k);
    gs_ui_event(s, &e);
    if (gs_ui_slot[s].used && gs_ui_slot[s].explain_dirty) gs_ui_refresh_view(s);
    lua_pushboolean(L, 1);
    return 1;
}

static int l_ui_focus(lua_State *L) {
    int s = gs_ui_slot_arg(L, 1);
    const GsUiSlot *u = &gs_ui_slot[s];
    const char *c = at_screen_cell_id(&u->sc, u->view.focus), *b = at_screen_block_id(&u->sc, u->view.focus.block);
    if (c == NULL) { lua_pushnil(L); return 1; }
    lua_pushstring(L, c);
    lua_pushstring(L, b != NULL ? b : "");
    return 2;
}

static int l_ui_set_focus(lua_State *L) {
    int s = gs_ui_slot_arg(L, 1);
    GsUiSlot *u = &gs_ui_slot[s];
    AtFocusPos p = at_screen_refocus(&u->sc, luaL_checkstring(L, 2), luaL_checkstring(L, 3), u->view.focus);
    const char *c = at_screen_cell_id(&u->sc, p);
    int ok = c != NULL && strcmp(c, lua_tostring(L, 3)) == 0;
    if (ok) { u->view.focus = p; u->explain_dirty = 1; }
    lua_pushboolean(L, ok);
    return 1;
}

static int l_ui_note(lua_State *L) {
    int top = at_stack_top(&gs_ui_stack);
    GsUiSlot *u;
    const char *kind;
    double secs;
    luaL_checktype(L, 1, LUA_TTABLE);
    if (top < 0) { lua_pushboolean(L, 0); return 1; }
    u = &gs_ui_slot[top];
    lua_getfield(L, 1, "text");
    snprintf(u->view.note.text, AT_STR, "%s", luaL_optstring(L, -1, ""));
    lua_pop(L, 1);
    lua_getfield(L, 1, "kind");
    kind = luaL_optstring(L, -1, "info");
    u->view.note.kind = strcmp(kind, "ok") == 0 ? AT_NOTE_OK : strcmp(kind, "warn") == 0 ? AT_NOTE_WARN : strcmp(kind, "err") == 0 ? AT_NOTE_ERR : AT_NOTE_INFO;
    lua_pop(L, 1);
    secs = gs_kit_optnum(L, 1, "seconds", 3.0);
    if (secs < 0.5) secs = 0.5;
    if (secs > 15.0) secs = 15.0;
    u->view.note.from_ms = gs_now_ms();
    u->view.note.until_ms = u->view.note.from_ms + secs * 1000.0;
    lua_pushboolean(L, 1);
    return 1;
}

static int gs_ui_btn_char(const char *s) { return (s != NULL && s[0] != '\0' && strchr("ABXYZ", s[0]) != NULL) ? s[0] : 'A'; }

static int l_ui_dialog(lua_State *L) {
    int top = at_stack_top(&gs_ui_stack), i, n;
    GsUiSlot *u;
    luaL_checktype(L, 1, LUA_TTABLE);
    if (top < 0) { lua_pushboolean(L, 0); return 1; }
    u = &gs_ui_slot[top];
    if (u->dialog_fn >= 0) { luaL_unref(L, LUA_REGISTRYINDEX, u->dialog_fn); u->dialog_fn = -1; }
    memset(&u->view.dialog, 0, sizeof u->view.dialog);
    lua_getfield(L, 1, "title");
    snprintf(u->view.dialog.title, AT_STR, "%s", luaL_optstring(L, -1, ""));
    lua_pop(L, 1);
    lua_getfield(L, 1, "text");
    snprintf(u->view.dialog.body, AT_TEXT, "%s", luaL_optstring(L, -1, ""));
    lua_pop(L, 1);
    lua_getfield(L, 1, "actions");
    n = lua_istable(L, -1) ? (int) lua_rawlen(L, -1) : 0;
    if (n > 2) n = 2;
    for (i = 1; i <= n; i++) {
        lua_rawgeti(L, -1, i);
        if (lua_istable(L, -1)) {
            lua_rawgeti(L, -1, 1);
            u->view.dialog.btn[i - 1] = (char) gs_ui_btn_char(luaL_optstring(L, -1, "A"));
            lua_pop(L, 1);
            lua_rawgeti(L, -1, 2);
            snprintf(u->view.dialog.label[i - 1], 24, "%s", luaL_optstring(L, -1, ""));
            lua_pop(L, 1);
        }
        lua_pop(L, 1);
    }
    lua_pop(L, 1);
    lua_getfield(L, 1, "on");
    if (lua_isfunction(L, -1)) u->dialog_fn = luaL_ref(L, LUA_REGISTRYINDEX); else lua_pop(L, 1);
    u->view.dialog.n = n;
    u->view.dialog.focus = 0;
    u->view.dialog.from_ms = gs_now_ms();
    u->view.dialog.open = n > 0;
    lua_pushboolean(L, n > 0);
    return 1;
}

static int l_ui_state(lua_State *L) {
    int top = at_stack_top(&gs_ui_stack);
    AtLayout lay;
    lua_newtable(L);
    lua_pushinteger(L, gs_ui_stack.n); lua_setfield(L, -2, "depth");
    if (top >= 0) { lua_pushstring(L, gs_ui_slot[top].sc.id); lua_setfield(L, -2, "top"); }
    lua_pushnumber(L, gs_ui_stat.canvas_w); lua_setfield(L, -2, "canvas_w");
    at_layout(gs_ui_stat.canvas_w > 0.0f ? gs_ui_stat.canvas_w : 640.0f, AT_PRESET_NORMAL, &lay);
    lua_pushboolean(L, lay.wide); lua_setfield(L, -2, "wide");
    lua_pushinteger(L, gs_ui_stat.quads); lua_setfield(L, -2, "quads");
    lua_pushnumber(L, gs_ui_stat.frames > 0 ? gs_ui_stat.sum / (double) gs_ui_stat.frames : 0.0); lua_setfield(L, -2, "cost_ms");
    lua_pushnumber(L, gs_ui_stat.max); lua_setfield(L, -2, "cost_max_ms");
    lua_pushboolean(L, gs_ui_roles_state > 0); lua_setfield(L, -2, "roles_ok");
    return 1;
}

static const luaL_Reg gs_ui_funcs[] = {
    {"available", l_ui_available}, {"screen", l_ui_screen}, {"open", l_ui_open}, {"close", l_ui_close},
    {"feed", l_ui_feed}, {"focus", l_ui_focus}, {"set_focus", l_ui_set_focus}, {"note", l_ui_note},
    {"dialog", l_ui_dialog}, {"state", l_ui_state}, {NULL, NULL}};

static void gs_push_ui(lua_State *L) {
    lua_newtable(L);
    gs_prof_setfuncs(L, gs_ui_funcs, "gd.ui");
}
