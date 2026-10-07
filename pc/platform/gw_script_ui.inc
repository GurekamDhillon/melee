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
#include "gw_mods.h"
#include "gw_ui_policy.h"
#include "gw_ui_hud.h"
#include "gw_ui_registry.h"
#include "gw_ui_retail.h"
#include "gw_ui_render.h"
#include "gw_ui_stack.h"
#include "gw_ui_tokens.h"

extern int gw_Settings_Int(const char *key, int dflt); /* gw_settings.c:138; the Reduced Motion row writes "reduced_motion" */

#define GS_UI_SLOTS 16 /* Envoy alone holds up to nine (bag, bag.p2, reward, keystone, swap, setup, pause, results, the online pick) */
#define GS_UI_ENGINE (-2) /* GsUiSlot.owner for a native screen (the menus, the title, Credits): drawn from a record the game fills, no Lua */

typedef struct {
    int used, owner, explain_dirty, dialog_fn, warned, render_warned, ex_warned;
    int scene;              /* the scene kind that was current when it was pushed (gw_Ui_SceneExit closes it with that scene) */
    int rebuilds;           /* an engine screen: how many times its record was replaced */
    unsigned primed_frame;  /* an engine screen: the tick it became the top again (an intent in that same frame is dropped) */
    int self_close;         /* an engine screen the host runs alone (Credits): B closes it here, no event reaches the game */
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
static int gs_ui_roles_state; /* 0 not tried, 1 every Atlas role found and every page loaded, -1 missing */
static char gs_ui_why[160];   /* the reason when it is -1 */
static struct { double sum, max; int frames, quads; float canvas_w; } gs_ui_stat;
static int gs_ui_menu_held_by = -2;   /* gd.ui.hold_menu: the script index holding the native menu, -2 none */
static unsigned gs_ui_script_close_frame; /* the tick a script's screen last closed (+1): a press of that frame is not the menu's */
static unsigned gs_ui_frame; /* gs_ui_tick calls: the 'same frame' of an uncovered engine screen */
static int gs_ui_reduced; /* gw_Settings_Int("reduced_motion") read once per draw pass */
static AtRetail gs_ui_retail;                                    /* the retail takeover's mask (spec 6.8); empty by default */
static AtPause gs_ui_pause = { 0, -1, 0, 0 };                    /* the retail pause as Atlas sees it */
static int gs_ui_pause_changed;                                  /* a retail pause began or ended: the tick pushes or pops the pause screen */
static int gs_ui_pause_slot = -1;                                /* the screen gd.ui.pause_screen named (latest call wins) */
static int gs_ui_pause_pushed = -1;                              /* the screen the tick pushed for the running pause, -1 none */

/* ---- fonts: the Atlas roles by name, resolved once ------------------------------------------------ */
static int gs_ui_roles_ok(void) {
    int r;
    if (gs_ui_roles_state != 0) return gs_ui_roles_state > 0;
    if (!gw_Kit_Available()) {                                      /* not latched: the kit may come up later (the title can be the first scene) */
        snprintf(gs_ui_why, sizeof gs_ui_why, "%s", gw_Kit_Why());
        return 0;
    }
    gs_ui_roles_state = -1;
    gs_ui_why[0] = '\0';
    for (r = 0; r < AT_R_COUNT; r++) {
        const char *missing;
        gs_ui_role_idx[r] = gw_Kit_Role(at_role(r)->name);
        if (gs_ui_role_idx[r] < 0) {
            snprintf(gs_ui_why, sizeof gs_ui_why, "the font manifest has no Atlas text role \"%s\"", at_role(r)->name);
            gw_log("ui: the font manifest has no Atlas role \"%s\" (rebuild the kit fonts: menu/pipeline/font_atlas.py)", at_role(r)->name);
            return 0;
        }
        missing = gw_Kit_RoleMissingPage(gs_ui_role_idx[r]);          /* a manifest role whose page is absent draws nothing */
        if (missing != NULL) {
            snprintf(gs_ui_why, sizeof gs_ui_why, "the Atlas font page \"%s\" (role \"%s\") did not load", missing, at_role(r)->name);
            gw_log("ui: %s (deploy the Atlas font pages to ui/: menu/pipeline/font_atlas.py, then png2gx)", gs_ui_why);
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
    p.spin = (focused && !gs_ui_reduced) ? 120.0f : 0.0f;
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
        int n, len = (int) lua_rawlen(L, idx), i;
        if (!lua_checkstack(L, 6)) { gs_ui_arena.overflow = 1; return -1; }   /* a key, a value and the call's own slots, per level */
        n = atv_table(&gs_ui_arena);
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

/* A script that is unloaded or switched off (gs_report) runs nothing: not a handler, not a dialog. */
static int gs_ui_usable(int owner) { return owner == GS_UI_ENGINE || (owner >= 0 && owner < gs.n && gs.s[owner].used && !gs.s[owner].disabled); }

/* A screen belongs to the script that registered it; only that script (and the console) may drive it. */
/* An engine screen belongs to the engine alone: no script, and not the console, may open, close, feed or replace it. */
static int gs_ui_may_touch(int slot) { return gs_ui_slot[slot].owner != GS_UI_ENGINE && (gs.cur == gs.console || gs_ui_slot[slot].owner == gs.cur); }

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
    if (ref < 0 || !gs_may_run(owner) || !gs_ui_usable(owner)) return 0;
    lua_rawgeti(L, LUA_REGISTRYINDEX, ref);
    if (!lua_isfunction(L, -1)) { lua_pop(L, 1); return 0; }
    lua_pushstring(L, a != NULL ? a : "");
    lua_pushstring(L, b != NULL ? b : "");
    return gs_pcall(owner, 2, nres, "ui") == 0;
}

static void gs_ui_close_screen(int slot);
static int gs_ui_open_screen(int slot);

/* a handler may return {pop = true} or {push = "screen id"} */
/* A handler's result is judged by the slot owners, never by gs.cur: the tick calls handlers between hooks, where gs.cur is -1.
 * `owner` is the script whose handler returned it: it may pop only a top screen it owns and push only a screen it owns. */
static void gs_ui_apply_result(int owner) {
    lua_State *L = gs.L;
    if (lua_istable(L, -1)) {
        lua_pushliteral(L, "pop");                                 /* raw reads: a handler's table may carry a metatable, and this */
        lua_rawget(L, -2);                                         /* runs outside any protected call (from gs_ui_tick) */
        if (lua_toboolean(L, -1) && at_stack_top(&gs_ui_stack) >= 0 && gs_ui_slot[at_stack_top(&gs_ui_stack)].owner == owner) gs_ui_close_screen(at_stack_top(&gs_ui_stack));
        lua_pop(L, 1);
        lua_pushliteral(L, "push");
        lua_rawget(L, -2);
        if (lua_type(L, -1) == LUA_TSTRING) {
            int s = gs_ui_find(lua_tostring(L, -1));
            if (s >= 0 && gs_ui_slot[s].owner == owner) gs_ui_open_screen(s);
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
            if (!(u->ex_warned & 1)) { u->ex_warned |= 1; gw_log("ui: %s: the explainer provider returned too large a table; showing none", u->sc.id); }
            at_explainer_from_val(&gs_ui_arena, -1, &u->view.ex, err, sizeof err);
        } else {
            at_explainer_from_val(&gs_ui_arena, n, &u->view.ex, err, sizeof err);
        }
        gs_ui_unref_arena_except(L, NULL, 0);
        lua_pop(L, 1);
        if (u->view.ex.warn && !(u->ex_warned & 2)) { u->ex_warned |= 2; gw_log("ui: %s: an explainer string was cut to its field", u->sc.id); }
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

/* ---- what is held right now -------------------------------------------------------------------------- */
static unsigned gs_ui_key_mask(void) {
    unsigned m = 0;
    if (gs.key_now[VK_UP]) m |= AT_KEY_UP;
    if (gs.key_now[VK_DOWN]) m |= AT_KEY_DOWN;
    if (gs.key_now[VK_LEFT]) m |= AT_KEY_LEFT;
    if (gs.key_now[VK_RIGHT]) m |= AT_KEY_RIGHT;
    if (gs.key_now[VK_RETURN]) m |= AT_KEY_ENTER;
    if (gs.key_now[VK_ESCAPE]) m |= AT_KEY_ESC;
    if (gs.key_now[VK_TAB]) m |= AT_KEY_TAB;
    if (gs.key_now[VK_SHIFT]) m |= AT_KEY_SHIFT;
    return m;
}

static unsigned gs_ui_pad_read(const GsUiSlot *u, int *sx, int *sy) {
    unsigned b;
    int cx, cy, tl, tr;
    gw_script_pad_state(u->sc.port - 1, &b, sx, sy, &cx, &cy, &tl, &tr);
    return gw_script_pad_raw_buttons(u->sc.port - 1);
}

/* A screen that becomes the top one (opened, or uncovered when the one above it closed) starts from the buttons, keys
 * and mouse buttons held at that moment, so a press has to be released before it can fire on the new top: a held A that
 * opened a screen does not accept on it, and one held B closes one level, not the whole stack. The events these calls
 * would make are discarded (capacity 0); only the previous-state they leave behind is wanted. */
static void gs_ui_prime(int slot) {
    GsUiSlot *u = &gs_ui_slot[slot];
    AtEvent ev[1];
    float x, y, wheel;
    int buttons, sx = 0, sy = 0;
    double now = gs_now_ms();
    memset(&u->pad, 0, sizeof u->pad);
    memset(&u->keys, 0, sizeof u->keys);
    memset(&u->mouse, 0, sizeof u->mouse);
    if (u->owner == GS_UI_ENGINE) u->primed_frame = gs_ui_frame + 1;  /* an intent of this same frame is the press that closed the screen above */
    if (at_screen_wants_pad(&u->sc)) {
        unsigned b = gs_ui_pad_read(u, &sx, &sy);
        (void) at_pad_events(&u->pad, b, sx, sy, now, ev, 0);
    }
    (void) at_key_events(&u->keys, gs_ui_key_mask(), now, ev, 0);
    gw_Mouse_ScriptRead(&x, &y, &buttons, &wheel);
    (void) at_mouse_events(&u->mouse, x, y, buttons, 0, &u->hits, ev, 0);
}

/* ---- stack operations --------------------------------------------------------------------------------- */
static int gs_ui_open_screen(int slot) {
    GsUiSlot *u = &gs_ui_slot[slot];
    int owner = u->owner;
    AtStackEntry *top;
    if (!at_stack_push(&gs_ui_stack, slot)) return 0;
    top = at_stack_top_entry(&gs_ui_stack);
    if (top != NULL && top->focus.block >= 0 && u->view.focus.block < 0) u->view.focus = top->focus;
    u->view.opened_ms = gs_now_ms();
    u->view.port_rgba = 0;
    u->explain_dirty = 1;
    u->scene = gs.scene_kind;
    gs_ui_prime(slot);
    gw_log("ui: screen %s open (%d blocks, %d items)", u->sc.id, u->sc.n_blocks, u->sc.n_items);
    if (u->sc.fn_open >= 0 && gs_ui_call(slot, u->sc.fn_open, "", "", 1)) gs_ui_apply_result(owner);
    return 1;
}

static void gs_ui_close_screen(int slot) {
    GsUiSlot *u = &gs_ui_slot[slot];
    AtStackEntry *top = at_stack_top_entry(&gs_ui_stack);
    if (top != NULL && top->screen == slot) { top->focus = u->view.focus; top->scroll = u->view.scroll; }
    if (!at_stack_remove(&gs_ui_stack, slot)) return;
    if (u->owner != GS_UI_ENGINE) gs_ui_script_close_frame = gs_ui_frame + 1;
    if (u->sc.fn_close >= 0 && gs_ui_call(slot, u->sc.fn_close, "", "", 1)) lua_pop(gs.L, 1);
    gw_log("ui: screen %s closed", u->sc.id);
    if (at_stack_top(&gs_ui_stack) >= 0 && gs_ui_slot[at_stack_top(&gs_ui_stack)].used) gs_ui_prime(at_stack_top(&gs_ui_stack));
}

/* a script unloaded or switched off: its screens and their references go */
static void gs_ui_reg_release_script(int script);
static void gs_ui_hud_release(int script);
/* a slot's screen leaves the stack, its Lua references go back to the registry and the slot is free again */
static void gs_ui_free_slot(int i) {
    GsUiSlot *u = &gs_ui_slot[i];
    int was_top = at_stack_top(&gs_ui_stack) == i;
    while (at_stack_remove(&gs_ui_stack, i)) {}                    /* open pushes again, so a duplicate entry may exist */
    if (gs_ui_pause_slot == i) gs_ui_pause_slot = -1;
    if (gs_ui_pause_pushed == i) gs_ui_pause_pushed = -1;
    gs_ui_unref_screen(gs.L, &u->sc);
    if (u->dialog_fn >= 0) luaL_unref(gs.L, LUA_REGISTRYINDEX, u->dialog_fn);
    memset(u, 0, sizeof *u);
    u->dialog_fn = u->sc.fn_provide = u->sc.fn_accept = u->sc.fn_back = u->sc.fn_focus = u->sc.fn_change = u->sc.fn_open =
        u->sc.fn_close = u->sc.fn_counter = u->sc.fn_page = u->sc.fn_start = -1;
    u->sc.fn_alt[0] = u->sc.fn_alt[1] = u->sc.fn_alt[2] = -1;
    if (was_top && at_stack_top(&gs_ui_stack) >= 0 && gs_ui_slot[at_stack_top(&gs_ui_stack)].used) gs_ui_prime(at_stack_top(&gs_ui_stack));
}

static void gs_ui_release(int script) {
    int i;
    if (script < 0) return;                                        /* an engine screen is never released by a script's unload */
    if (gs_ui_menu_held_by == script) gs_ui_menu_held_by = -2;      /* a script that goes takes its hold with it */
    gs_ui_reg_release_script(script);                                        /* an engine screen is never released by a script's unload */
    gs_ui_hud_release(script);                                               /* its HUD and its claim on the retail mask go too */
    for (i = 0; i < GS_UI_SLOTS; i++) {
        GsUiSlot *u = &gs_ui_slot[i];
        if (!u->used || u->owner != script) continue;
        gs_ui_free_slot(i);
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
    if (u->dialog_fn >= 0) {                                       /* answered: the reference is taken now and dropped now */
        int ref = u->dialog_fn;
        char one[2];
        one[0] = (char) btn; one[1] = '\0';
        u->dialog_fn = -1;
        lua_rawgeti(L, LUA_REGISTRYINDEX, ref);
        luaL_unref(L, LUA_REGISTRYINDEX, ref);
        if (lua_isfunction(L, -1) && gs_may_run(u->owner) && gs_ui_usable(u->owner)) { lua_pushstring(L, one); (void) gs_pcall(u->owner, 1, 0, "ui:dialog"); } else lua_pop(L, 1);
    }
}

/* on.change(item id, value): a toggle gives its new state (true / false), a slider its new number, a choice the direction
 * it was stepped (-1 or +1: the options are the script's own). The engine updates what it shows for a toggle and a slider
 * at once; the script re-registers when it wants the row to read differently. Returns 1 when the event was a value event. */
static int gs_ui_value_event(int slot, const AtEvent *e) {
    GsUiSlot *u = &gs_ui_slot[slot];
    int owner = u->owner;
    AtItem *it;
    int idx = u->view.focus.index, left, right, accept, step, nv, is_bool = 0, dir = 0;
    char id[AT_ID];
    double val = 0.0;
    if (u->sc.primary != AT_PRIMARY_LIST || idx < 0 || idx >= u->sc.n_items) return 0;
    it = &u->sc.items[idx];
    if ((it->vkind != AT_VAL_TOGGLE && it->vkind != AT_VAL_CHOICE && it->vkind != AT_VAL_SLIDER) || (it->flags & AT_CELL_DISABLED)) return 0;
    left = e->type == AT_EV_MOVE && e->a == AT_DIR_LEFT;
    right = e->type == AT_EV_MOVE && e->a == AT_DIR_RIGHT;
    accept = e->type == AT_EV_ACCEPT;
    if (!left && !right && !accept) return 0;
    snprintf(id, sizeof id, "%s", it->id);
    if (it->vkind == AT_VAL_TOGGLE) {
        it->on = !it->on; is_bool = 1; val = it->on;
    } else if (it->vkind == AT_VAL_CHOICE) {
        dir = left ? -1 : 1; val = dir;
    } else {
        if (accept) return 0;                                     /* a slider is stepped, not accepted */
        step = (it->vmax - it->vmin) / 20; if (step < 1) step = 1;
        nv = it->vval + (left ? -step : step);
        if (nv < it->vmin) nv = it->vmin;
        if (nv > it->vmax) nv = it->vmax;
        if (nv == it->vval) return 1;                             /* at the end: nothing changed, nothing to report */
        it->vval = nv; val = nv;
    }
    if (u->sc.fn_change >= 0 && gs_may_run(u->owner) && gs_ui_usable(u->owner)) {
        lua_State *L = gs.L;
        lua_rawgeti(L, LUA_REGISTRYINDEX, u->sc.fn_change);
        if (lua_isfunction(L, -1)) {
            lua_pushstring(L, id);
            if (is_bool) lua_pushboolean(L, val != 0.0); else lua_pushinteger(L, (lua_Integer) val);
            if (gs_pcall(u->owner, 2, 1, "ui") == 0) gs_ui_apply_result(owner);
        } else lua_pop(L, 1);
    }
    return 1;
}

static void gs_ui_engine_event(int slot, const AtEvent *e);
static void gs_ui_fix_scroll(GsUiSlot *u);
static void gs_ui_overlay_try(void);

static void gs_ui_event(int slot, const AtEvent *e) {
    GsUiSlot *u = &gs_ui_slot[slot];
    int owner = u->owner;
    AtFocusBlock fb[AT_MAX_BLOCKS];
    AtFocusPos before = u->view.focus;
    const char *cid, *bid;
    char cell[AT_ID], block[AT_ID];
    int nb, alt;
    if (owner == GS_UI_ENGINE) { gs_ui_engine_event(slot, e); return; }
    if (u->view.dialog.open) {                                    /* a dialog takes every event until it is answered */
        if (e->type == AT_EV_ACCEPT) gs_ui_dialog_press(slot, 'A');
        else if (e->type == AT_EV_BACK) gs_ui_dialog_press(slot, 'B');
        else if (e->type == AT_EV_ALT) gs_ui_dialog_press(slot, e->a);
        return;
    }
    if (gs_ui_value_event(slot, e)) { u->explain_dirty = 1; return; }
    nb = at_screen_focus_blocks(&u->sc, fb);
    if (e->type == AT_EV_MOVE) u->view.focus = at_focus_move(fb, nb, u->view.focus, e->a, 1);
    else if (e->type == AT_EV_SCROLL && (u->sc.primary == AT_PRIMARY_LIST || u->sc.primary == AT_PRIMARY_TILES)) u->view.focus = at_focus_move(fb, nb, u->view.focus, e->a < 0 ? AT_DIR_UP : AT_DIR_DOWN, 0);
    else if (e->type == AT_EV_FOCUS) {
        AtFocusPos p;
        p.block = e->a; p.index = e->b;
        if (at_screen_cell_id(&u->sc, p) != NULL) u->view.focus = p;
    }
    gs_ui_fix_scroll(u);
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
        if (u->sc.fn_accept >= 0 && cid != NULL && at_cell_accepts(&u->sc, u->view.focus) && gs_ui_call(slot, u->sc.fn_accept, cid, bid, 1)) gs_ui_apply_result(owner);
    } else if (e->type == AT_EV_BACK) {
        if (u->sc.fn_back >= 0 && gs_ui_call(slot, u->sc.fn_back, cid, bid, 1)) gs_ui_apply_result(owner);
    } else if (e->type == AT_EV_ALT) {
        alt = e->a == 'X' ? 0 : e->a == 'Y' ? 1 : 2;
        if (u->sc.fn_alt[alt] >= 0 && gs_ui_call(slot, u->sc.fn_alt[alt], cid, bid, 1)) gs_ui_apply_result(owner);
    } else if (e->type == AT_EV_PAGE) {                           /* L / R, Tab / Shift+Tab: on.page(dir, cell id, block id), dir -1 or +1 */
        if (u->sc.fn_page >= 0 && gs_may_run(u->owner) && gs_ui_usable(u->owner)) {
            lua_State *L = gs.L;
            lua_rawgeti(L, LUA_REGISTRYINDEX, u->sc.fn_page);
            if (lua_isfunction(L, -1)) {
                lua_pushinteger(L, e->a < 0 ? -1 : 1);
                lua_pushstring(L, cid != NULL ? cid : "");
                lua_pushstring(L, bid != NULL ? bid : "");
                if (gs_pcall(u->owner, 3, 1, "ui") == 0) gs_ui_apply_result(owner);
            } else lua_pop(L, 1);
        }
    } else if (e->type == AT_EV_START) {                          /* START: on.start(cell id, block id) (the spec leaves it to the screen) */
        if (u->sc.fn_start >= 0 && gs_ui_call(slot, u->sc.fn_start, cid, bid, 1)) gs_ui_apply_result(owner);
    }
}

/* once per script tick, before on_tick: the top screen's input, then its explainer refresh */
static void gs_ui_tick(void) {
    int top = at_stack_top(&gs_ui_stack), n, i;
    AtEvent ev[16];
    double now = gs_now_ms();
    GsUiSlot *u;
    gs_ui_frame++;
    gs_ui_overlay_try();
    if (gs_ui_pause_changed) {                  /* a retail pause began or ended: push or pop the named pause screen (a push is presentation only) */
        gs_ui_pause_changed = 0;
        if (gs_ui_pause.paused && gs_ui_pause.takeover && gs_ui_pause_slot >= 0 && gs_ui_slot[gs_ui_pause_slot].used && gs_ui_slot[gs_ui_pause_slot].sc.pause &&
            gs_ui_usable(gs_ui_slot[gs_ui_pause_slot].owner)) {
            gs_ui_slot[gs_ui_pause_slot].sc.port = gs_ui_pause.pauser + 1;                       /* the pausing port drives it */
            if (gs_ui_open_screen(gs_ui_pause_slot)) gs_ui_pause_pushed = gs_ui_pause_slot;
        } else if (!gs_ui_pause.paused && gs_ui_pause_pushed >= 0) {
            int s = gs_ui_pause_pushed;
            gs_ui_pause_pushed = -1;
            if (gs_ui_slot[s].used) {
                if (at_stack_top(&gs_ui_stack) == s) gs_ui_close_screen(s);
                else at_stack_remove(&gs_ui_stack, s);
            }
        }
    }
    top = at_stack_top(&gs_ui_stack);
    if (top < 0) return;
    u = &gs_ui_slot[top];
    if (!u->used) {
        at_stack_remove(&gs_ui_stack, top);
        if (at_stack_top(&gs_ui_stack) >= 0 && gs_ui_slot[at_stack_top(&gs_ui_stack)].used) gs_ui_prime(at_stack_top(&gs_ui_stack));
        return;
    }
    if (!gs_ui_usable(u->owner)) { gs_ui_release(u->owner); return; }   /* switched off or unloaded: its screens go (never an engine screen) */
    if (!gs_may_run(u->owner)) return;
    if (at_screen_wants_pad(&u->sc)) {
        int sx, sy;
        unsigned b = gs_ui_pad_read(u, &sx, &sy);
        n = at_pad_events(&u->pad, b, sx, sy, now, ev, 16);
        for (i = 0; i < n && at_stack_top(&gs_ui_stack) == top; i++) gs_ui_event(top, &ev[i]);
    }
    if (at_stack_top(&gs_ui_stack) != top) return;
    {
        n = at_key_events(&u->keys, gs_ui_key_mask(), now, ev, 16);
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
    if (at_stack_top(&gs_ui_stack) == top && u->explain_dirty && u->owner != GS_UI_ENGINE) gs_ui_refresh_view(top);
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
    if (!u->used || !gs_ui_usable(u->owner)) return;
    if (u->owner == GS_UI_ENGINE && gs_ui_menu_held_by != -2) return;   /* held: a script's own menu is drawn over the frontend instead */
    if (gw_Kit_QuadRoom() < AT_SCREEN_QUAD_CAP) {
        if (!u->warned) { u->warned = 1; gw_log("ui: %s not drawn: only %d quads left this frame", u->sc.id, gw_Kit_QuadRoom()); }
        return;
    }
    old = gs.cur;
    if (u->owner != GS_UI_ENGINE) gs.cur = u->owner;                   /* an engine screen draws as whoever is running */
    QueryPerformanceCounter(&t0);
    ops.width = gs_ui_width; ops.user = NULL;
    sink.user = NULL; sink.poly = gs_ui_poly; sink.text = gs_ui_text; sink.model = gs_ui_model_draw;
    memset(&info, 0, sizeof info);
    first = gw_Kit_QuadCount();
    gs_ui_stat.canvas_w = gw_Console_ScriptWidth();
    gs_ui_reduced = gw_Settings_Int("reduced_motion", 0) != 0;
    at_render_ex(&u->sc, &u->view, gs_ui_stat.canvas_w, gs_now_ms(), gs_ui_reduced, &ops, &sink, &u->hits, &info);
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
        gw_log("ui: %s render budget: %d entries (warning line %d, cap %d)%s%s", u->sc.id, info.entries, AT_SCREEN_QUAD_WARN, AT_SCREEN_QUAD_CAP,
               info.capped ? "; the cap was reached, later entries were dropped" : "",
               info.hits_dropped > 0 ? "; some hit rectangles did not fit (the mouse cannot reach them)" : "");
    }
    gs.cur = old;
}

/* ---- engine-owned screens (the native menus, the title, Credits) ------------------------------------------------
 * A screen whose owner is GS_UI_ENGINE has no Lua in it. The game side (src/melee/gm/gmfrontend_atlas.inc) describes it each
 * frame through the gw_Ui_* shims (game code calls them as Ui_*): Begin fills a scratch record, Commit replaces the slot's record
 * only when it differs. Its events do not go to handlers: they are queued and the game polls them (gw_Ui_PollEvent). The cursor is
 * the game's: Commit puts the focus where the game says, and a focus event only reports where the engine moved it.
 *
 * Who may do what: the engine (these functions) opens, closes, feeds and replaces its screens; a script, another script and the
 * console are refused ("belongs to the engine", gs_ui_may_touch and the guards in the Lua functions); an unloaded script's
 * release never reaches an engine slot; gw_Ui_SceneExit closes every screen pushed during a scene that ends. */

static struct { int type, block, index; } gs_ui_q[16];
static int gs_ui_qn;
static AtExplainer gs_ui_scratch_ex;
static int gs_ui_scratch_live;        /* a gw_Ui_Begin without its Commit yet */
static int gs_ui_atlas_env = -1;      /* MELEE_ATLAS: absent or anything but "0" is on */

static int gs_ui_find_on_stack(int slot) {
    int i;
    for (i = 0; i < gs_ui_stack.n; i++) if (gs_ui_stack.e[i].screen == slot) return 1;
    return 0;
}

static int atlas_env_on(void) {
    if (gs_ui_atlas_env < 0) {
        const char *e = getenv("MELEE_ATLAS");
        gs_ui_atlas_env = !(e != NULL && strcmp(e, "0") == 0);
        gw_log("ui: Atlas menus %s (MELEE_ATLAS)", gs_ui_atlas_env ? "on" : "off");
    }
    return gs_ui_atlas_env;
}

static void gs_ui_q_push(int type, int block, int index) {
    if (gs_ui_qn >= 16) return;                                    /* the game drains it every frame; a full queue drops the newest */
    gs_ui_q[gs_ui_qn].type = type; gs_ui_q[gs_ui_qn].block = block; gs_ui_q[gs_ui_qn].index = index;
    gs_ui_qn++;
}

/* the scroll offset that keeps the focused cell in the pane (shared by script and engine screens) */
static void gs_ui_fix_scroll(GsUiSlot *u) {
    AtLayout pl;
    if (u->view.focus.index < 0) return;
    at_layout(640.0f, u->sc.preset, &pl);                          /* the pane's height does not depend on the width */
    if (u->sc.primary == AT_PRIMARY_LIST) u->view.scroll = at_list_scroll(u->view.focus.index, u->view.scroll, at_list_visible(&pl), u->sc.n_items);
    else if (u->sc.primary == AT_PRIMARY_TILES) u->view.scroll = at_tiles_scroll(&u->sc, &pl, u->view.focus.block == 0 ? u->view.focus.index : -1, u->view.scroll);
}

static void gs_ui_credits_view(GsUiSlot *u);
static void gs_ui_free_engine_slot(int slot);
static void gs_ui_after_close(void);

/* an engine screen's event: the view moves, the game is told through the queue */
static void gs_ui_engine_event(int slot, const AtEvent *e) {
    GsUiSlot *u = &gs_ui_slot[slot];
    AtFocusBlock fb[AT_MAX_BLOCKS];
    AtFocusPos before = u->view.focus;
    int nb;
    if (u->sc.primary == AT_PRIMARY_DISPLAY) return;                  /* the title takes no input, from anything: retail reads the pad */
    nb = at_screen_focus_blocks(&u->sc, fb);
    if (e->type == AT_EV_MOVE) u->view.focus = at_focus_move(fb, nb, u->view.focus, e->a, 1);
    else if (e->type == AT_EV_SCROLL) u->view.focus = at_focus_move(fb, nb, u->view.focus, e->a < 0 ? AT_DIR_UP : AT_DIR_DOWN, 0);
    else if (e->type == AT_EV_FOCUS) {
        AtFocusPos p;
        p.block = e->a; p.index = e->b;
        if (at_screen_cell_id(&u->sc, p) != NULL) u->view.focus = p;
    }
    gs_ui_fix_scroll(u);
    if (u->self_close) {                                           /* Credits: the host's own screen, the game is told nothing */
        if (e->type == AT_EV_BACK) { gs_ui_free_engine_slot(slot); gs_ui_after_close(); return; }
        if (u->view.focus.block != before.block || u->view.focus.index != before.index) gs_ui_credits_view(u);
        return;
    }
    if (u->view.focus.block != before.block || u->view.focus.index != before.index) {
        gs_ui_q_push(AT_EV_FOCUS, u->view.focus.block, u->view.focus.index);
        return;
    }
    if (e->type == AT_EV_ACCEPT) {
        if (at_cell_accepts(&u->sc, u->view.focus)) gs_ui_q_push(AT_EV_ACCEPT, u->view.focus.block, u->view.focus.index);
    } else if (e->type == AT_EV_BACK) {
        gs_ui_q_push(AT_EV_BACK, u->view.focus.block, u->view.focus.index);
    } else if (e->type == AT_EV_ALT) {
        gs_ui_q_push(AT_EV_ALT, e->a, u->view.focus.index);          /* block = the button: 'X' 'Y' 'Z' */
    }
}

static void gs_ui_free_engine_slot(int slot) {
    GsUiSlot *u = &gs_ui_slot[slot];
    at_stack_remove(&gs_ui_stack, slot);
    gw_log("ui: screen %s closed (engine)", u->sc.id);
    memset(u, 0, sizeof *u);
    u->dialog_fn = -1;
}

static void gs_ui_after_close(void) {
    int top = at_stack_top(&gs_ui_stack);
    gs_ui_qn = 0;                                                  /* events of the screen that went are stale */
    if (top >= 0 && gs_ui_slot[top].used) gs_ui_prime(top);
}

int gw_Ui_Ready(void) { return gs_ui_roles_ok() && atlas_env_on(); }

/* the scene policy for a scene kind (RETAIL 0, OVERLAY 1, REPLACE 2): the table, MELEE_ATLAS and MELEE_ATLAS_SCENES (gw_ui_policy.h) */
int gw_Ui_ScenePolicy(int scene_kind) { return at_policy_for(scene_kind, atlas_env_on(), getenv("MELEE_ATLAS_SCENES")); }

static void gs_ui_scratch_reset(void) {
    memset(&gs_ui_scratch, 0, sizeof gs_ui_scratch);
    memset(&gs_ui_scratch_ex, 0, sizeof gs_ui_scratch_ex);
    gs_ui_scratch_ex.media_model = gs_ui_scratch_ex.media_ring = AT_NO_MODEL;
    gs_ui_scratch.fn_provide = gs_ui_scratch.fn_accept = gs_ui_scratch.fn_back = gs_ui_scratch.fn_focus = gs_ui_scratch.fn_change =
        gs_ui_scratch.fn_open = gs_ui_scratch.fn_close = gs_ui_scratch.fn_counter = gs_ui_scratch.fn_page = gs_ui_scratch.fn_start = -1;
    gs_ui_scratch.fn_alt[0] = gs_ui_scratch.fn_alt[1] = gs_ui_scratch.fn_alt[2] = -1;
    gs_ui_scratch.preset = AT_PRESET_NORMAL;
    gs_ui_scratch.port = 1;
    gs_ui_scratch.input_feed = 1;                                  /* the game feeds the merged menu input of every port (gw_Ui_Intent) */
}

int gw_Ui_Begin(const char *id, int primary, int chapter, const char *title) {
    if (!gw_Ui_Ready() || id == NULL || id[0] == '\0') return 0;
    gs_ui_scratch_reset();
    snprintf(gs_ui_scratch.id, sizeof gs_ui_scratch.id, "%s", id);
    snprintf(gs_ui_scratch.title, sizeof gs_ui_scratch.title, "%s", title != NULL ? title : id);
    gs_ui_scratch.primary = primary;
    gs_ui_scratch.chapter = chapter < 0 ? 0 : (chapter > 5 ? 5 : chapter);
    gs_ui_scratch_live = 1;
    return 1;
}

/* the tile columns: 1 (the main menu's big rows) or 2 (hubs); 0 lets the count decide */
void gw_Ui_Cols(int cols) {
    if (gs_ui_scratch_live && cols >= 0 && cols <= 2) gs_ui_scratch.tile_cols = cols;
}

void gw_Ui_Trail(const char *parent) {
    if (!gs_ui_scratch_live || parent == NULL || gs_ui_scratch.n_parents >= 3) return;
    snprintf(gs_ui_scratch.parent[gs_ui_scratch.n_parents++], AT_STR, "%s", parent);
}

static void gs_ui_fill_item(AtItem *it, const char *id, const char *label, const char *tag, const char *numeral, int flags) {
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id != NULL ? id : "");
    snprintf(it->label, sizeof it->label, "%s", label != NULL ? label : "");
    snprintf(it->tag, sizeof it->tag, "%s", tag != NULL ? tag : "");
    snprintf(it->numeral, sizeof it->numeral, "%s", numeral != NULL ? numeral : "");
    it->flags = (unsigned) flags;
}

void gw_Ui_Tile(const char *id, const char *label, const char *tag, const char *numeral, int flags) {
    if (!gs_ui_scratch_live || gs_ui_scratch.n_items >= AT_MAX_ITEMS) return;
    gs_ui_fill_item(&gs_ui_scratch.items[gs_ui_scratch.n_items++], id, label, tag, numeral, flags);
}

void gw_Ui_More(const char *id, const char *label) {
    if (!gs_ui_scratch_live || gs_ui_scratch.n_more >= AT_MAX_MORE) return;
    gs_ui_fill_item(&gs_ui_scratch.more[gs_ui_scratch.n_more++], id, label, "", "", 0);
}

void gw_Ui_Explain(const char *kicker, const char *title, const char *what) {
    if (!gs_ui_scratch_live) return;
    gs_ui_scratch_ex.has = 1;
    snprintf(gs_ui_scratch_ex.kicker, sizeof gs_ui_scratch_ex.kicker, "%s", kicker != NULL ? kicker : "");
    snprintf(gs_ui_scratch_ex.title, sizeof gs_ui_scratch_ex.title, "%s", title != NULL ? title : "");
    snprintf(gs_ui_scratch_ex.what, sizeof gs_ui_scratch_ex.what, "%s", what != NULL ? what : "");
}

void gw_Ui_ExplainWith(const char *tag) {
    if (!gs_ui_scratch_live || tag == NULL || gs_ui_scratch_ex.n_with_text >= AT_MAX_WITH) return;
    snprintf(gs_ui_scratch_ex.with_text[gs_ui_scratch_ex.n_with_text++], sizeof gs_ui_scratch_ex.with_text[0], "%s", tag);
}

void gw_Ui_Key(int button, const char *label) {
    AtKey *k;
    if (!gs_ui_scratch_live || gs_ui_scratch.n_keys >= AT_MAX_KEYS || label == NULL) return;
    k = &gs_ui_scratch.keys[gs_ui_scratch.n_keys++];
    k->btn = (char) button;
    snprintf(k->label, sizeof k->label, "%s", label);
    k->fn_label = k->fn_when = -1;
}

/* a display screen (the title): no pane, no explainer, no hits, no pad */
void gw_Ui_Display(const char *hero, const char *prompt, const char *foot_left, const char *foot_right) {
    if (!gs_ui_scratch_live) return;
    gs_ui_scratch.primary = AT_PRIMARY_DISPLAY;
    gs_ui_scratch.preset = AT_PRESET_NONE;
    snprintf(gs_ui_scratch.hero, sizeof gs_ui_scratch.hero, "%s", hero != NULL ? hero : "");
    snprintf(gs_ui_scratch.prompt, sizeof gs_ui_scratch.prompt, "%s", prompt != NULL ? prompt : "");
    snprintf(gs_ui_scratch.foot_left, sizeof gs_ui_scratch.foot_left, "%s", foot_left != NULL ? foot_left : "");
    snprintf(gs_ui_scratch.foot_right, sizeof gs_ui_scratch.foot_right, "%s", foot_right != NULL ? foot_right : "");
}

/* the view's key hints and explainer follow the record: refreshed whenever the record or the explainer changes */
static void gs_ui_engine_view(GsUiSlot *u) {
    int i;
    memcpy(&u->view.ex, &gs_ui_scratch_ex, sizeof u->view.ex);
    for (i = 0; i < u->sc.n_keys; i++) {
        snprintf(u->view.key_label[i], AT_STR, "%s", u->sc.keys[i].label);
        u->view.key_shown[i] = u->sc.keys[i].label[0] != '\0';
    }
    u->explain_dirty = 0;
}

int gw_Ui_Commit(int focus_block, int focus_index) {
    int slot, i;
    GsUiSlot *u;
    if (!gs_ui_scratch_live) return 0;
    gs_ui_scratch_live = 0;
    slot = gs_ui_find(gs_ui_scratch.id);
    if (slot < 0) {
        for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) { slot = i; break; }
        if (slot < 0) { gw_log("ui: no free screen slot for the engine screen %s", gs_ui_scratch.id); return 0; }
        u = &gs_ui_slot[slot];
        memset(u, 0, sizeof *u);
        at_view_init(&u->view);
        u->dialog_fn = -1;
        u->used = 1;
        u->owner = GS_UI_ENGINE;
        u->sc = gs_ui_scratch;
        gs_ui_engine_view(u);
    } else {
        u = &gs_ui_slot[slot];
        if (u->owner != GS_UI_ENGINE) { gw_log("ui: the engine screen id %s is a script's screen; not replaced", gs_ui_scratch.id); return 0; }
        if (memcmp(&u->sc, &gs_ui_scratch, sizeof u->sc) != 0) {
            const char *bid = at_screen_block_id(&u->sc, u->view.focus.block), *cid = at_screen_cell_id(&u->sc, u->view.focus);
            char b[AT_ID], c[AT_ID];
            AtFocusPos old = u->view.focus;
            snprintf(b, sizeof b, "%s", bid != NULL ? bid : "");
            snprintf(c, sizeof c, "%s", cid != NULL ? cid : "");
            memcpy(&u->sc, &gs_ui_scratch, sizeof u->sc);
            u->view.focus = at_screen_refocus(&u->sc, b[0] ? b : NULL, c[0] ? c : NULL, old);
            u->rebuilds++;
        }
        if (!u->self_close && (memcmp(&u->view.ex, &gs_ui_scratch_ex, sizeof u->view.ex) != 0 || u->explain_dirty)) gs_ui_engine_view(u);
    }
    if (focus_block >= 0) {                                        /* the game's cursor is the truth */
        AtFocusPos p;
        p.block = focus_block; p.index = focus_index;
        if (at_screen_cell_id(&u->sc, p) != NULL) u->view.focus = p;
    }
    gs_ui_fix_scroll(u);
    u->scene = gs.scene_kind;
    if (!gs_ui_find_on_stack(slot)) {
        if (at_stack_push(&gs_ui_stack, slot)) {                   /* not gs_ui_open_screen: a screen the engine opens is not "uncovered" */
            u->view.opened_ms = gs_now_ms();
            gw_log("ui: screen %s open (engine; %d items)", u->sc.id, u->sc.n_items);
            memset(&u->keys, 0, sizeof u->keys);
            memset(&u->mouse, 0, sizeof u->mouse);
        }
    }
    return at_stack_top(&gs_ui_stack) == slot;
}

void gw_Ui_Intent(int type, int a) {
    int top = at_stack_top(&gs_ui_stack);
    AtEvent e;
    GsUiSlot *u;
    if (top < 0) return;
    u = &gs_ui_slot[top];
    if (!u->used || u->owner != GS_UI_ENGINE) return;              /* a script's screen on top: the menu takes no input */
    if (gs_ui_menu_held_by != -2) return;                          /* held by a script (gd.ui.hold_menu): no input either */
    if (u->primed_frame == gs_ui_frame + 1) return;                 /* the press that uncovered it is not its own */
    if (!gs_may_run(GS_UI_ENGINE)) return;
    memset(&e, 0, sizeof e);
    e.type = type; e.a = a;
    gs_ui_engine_event(top, &e);
}

/* The three out-parameters are the GAME's locals (guest memory, big-endian, the retargeted code reads them with byte swaps), so a plain
 * native store arrives swapped: FOCUS (2) read as 0x02000000, matched no event and every menu was dead. They are written big-endian. */
static void gs_ui_put_be32(int *p, int v) {
    unsigned u = (unsigned) v;
    u = (u >> 24) | ((u >> 8) & 0xFF00u) | ((u << 8) & 0xFF0000u) | (u << 24);
    memcpy(p, &u, sizeof u);
}

int gw_Ui_PollEvent(int *type, int *block, int *index) {
    int i;
    if (gs_ui_qn <= 0) return 0;
    gs_ui_put_be32(type, gs_ui_q[0].type); gs_ui_put_be32(block, gs_ui_q[0].block); gs_ui_put_be32(index, gs_ui_q[0].index);
    for (i = 1; i < gs_ui_qn; i++) gs_ui_q[i - 1] = gs_ui_q[i];
    gs_ui_qn--;
    return 1;
}

void gw_Ui_Close(const char *id) {
    int s = id != NULL ? gs_ui_find(id) : -1;
    if (s < 0 || gs_ui_slot[s].owner != GS_UI_ENGINE) return;
    gs_ui_free_engine_slot(s);
    gs_ui_after_close();
}

int gw_Ui_TopIsEngine(const char *id) {
    int top = at_stack_top(&gs_ui_stack);
    if (top < 0 || !gs_ui_slot[top].used || gs_ui_slot[top].owner != GS_UI_ENGINE || gs_ui_menu_held_by != -2) return 0;
    return id == NULL || strcmp(gs_ui_slot[top].sc.id, id) == 0;
}

/* a scene ends: every screen pushed during it goes (the menu's, a mod's opened from a menu entry) */
void gw_Ui_SceneExit(int scene_kind) {
    int i, any = 0;
    for (i = 0; i < GS_UI_SLOTS; i++) {
        GsUiSlot *u = &gs_ui_slot[i];
        if (!u->used || u->scene != scene_kind || !gs_ui_find_on_stack(i)) continue;
        any = 1;
        if (u->owner == GS_UI_ENGINE) gs_ui_free_engine_slot(i);
        else gs_ui_close_screen(i);                                 /* a mod's screen: on.close runs, the screen stays registered */
    }
    gs_ui_menu_held_by = -2;                                       /* a hold lasts for the scene that took it */
    if (any) gs_ui_after_close();
}

/* how many engine screens hold a slot (the front door uses at most two at once: the menu and the title) */
static int gs_ui_engine_slots(void) {
    int i, n = 0;
    for (i = 0; i < GS_UI_SLOTS; i++) if (gs_ui_slot[i].used && gs_ui_slot[i].owner == GS_UI_ENGINE) n++;
    return n;
}

/* ---- scene policy OVERLAY: the title (spec 6.8) ------------------------------------------------------------------------------
 * At scene begin, a scene the policy table marks OVERLAY gets its engine display screen pushed over the retail scene, which runs
 * unchanged (START, the attract loop and the debug path stay retail's). The screen has no input and is closed with the scene
 * (gw_Ui_SceneExit). If the Atlas roles are not up yet (the kit loads late) it is pushed on the first tick where they are, provided
 * that scene is still current; if they are missing (no ui/ pages) the scene stays retail, with one log line. */
static int gs_ui_overlay_kind = -1;

static void gs_ui_overlay_try(void) {
    const char *screen;
    if (gs_ui_overlay_kind < 0) return;
    if (gs_ui_overlay_kind != gs.scene_kind) { gs_ui_overlay_kind = -1; return; }          /* that scene is over */
    if (!atlas_env_on()) { gs_ui_overlay_kind = -1; return; }
    if (!gs_ui_roles_ok()) {
        if (gs_ui_roles_state < 0) {                                                      /* missing for good, not merely late */
            gw_log("ui: scene %d stays retail: the Atlas roles are missing (%s)", gs_ui_overlay_kind, gs_ui_why);
            gs_ui_overlay_kind = -1;
        }
        return;
    }
    screen = at_policy_screen(gs_ui_overlay_kind);
    gs_ui_overlay_kind = -1;
    if (screen == NULL || strcmp(screen, "title") != 0) return;
    if (!gw_Ui_Begin("title", AT_PRIMARY_DISPLAY, 0, "")) return;
    /* no version symbol exists in the host (grep GW_VERSION in pc/platform finds none): the foot texts are empty */
    gw_Ui_Display("GD'S MELEE", "PRESS START", "", "");
    gw_Ui_Commit(-1, -1);
}

void gw_Ui_SceneBegin(int scene_kind) {
    gs_ui_overlay_kind = (gw_Ui_ScenePolicy(scene_kind) == AT_POLICY_OVERLAY && at_policy_screen(scene_kind) != NULL) ? scene_kind : -1;
    gs_ui_overlay_try();
}

/* ---- the Credits screen (MAIN MENU > More > Credits): a host-native list, B closes it here ---------------------------------
 * The rows are CREDITS.md's: each outside project we use or learned from, with its author, licence and link. Nothing here is
 * invented; the last row sends the player to the whole file. A row's short line is the list's sub; the link and what we drew
 * on are the explainer's text for the focused row. */
typedef struct { const char *label, *sub, *what; } GsCredit;
static const GsCredit gs_credits[] = {
    { "doldecomp/melee", "the doldecomp contributors", "github.com/doldecomp/melee. The matching decompilation the port is built from." },
    { "Aurora", "encounter. MIT", "github.com/encounter/aurora. The GameCube SDK reimplementation the port renders through." },
    { "Project Slippi", "Fizzi and the Project Slippi team", "slippi.gg. The replay format, rollback practice and UCF codes the port follows." },
    { "m-ex", "akaneia and contributors", "github.com/akaneia/m-ex. The content mod the port's m-ex layer follows; consulted, never copied (it publishes no licence)." },
    { "Lua 5.4", "Lua.org, PUC-Rio", "lua.org. The scripting language embedded in the port." },
    { "Barlow Condensed", "Jeremy Tribby. SIL OFL 1.1", "github.com/jpt/barlow. The caps labels and titles of the Atlas menus." },
    { "Source Sans 3", "Adobe. SIL OFL 1.1", "github.com/adobe-fonts/source-sans. The menus' body text." },
    { "Hasklug", "Nerd Fonts. SIL OFL 1.1", "github.com/ryanoasis/nerd-fonts. The menus' numerals." },
    { "SDL", "the SDL contributors. zlib", "github.com/libsdl-org/SDL. Controller buttons, axes and device polling." },
    { "Dawn and Tint", "Google. BSD-3-Clause", "dawn.googlesource.com/dawn. The WebGPU backend and the shader validation." },
    { "Tracy", "Bartosz Taudul. BSD-3-Clause", "github.com/wolfpld/tracy. The optional development profiler client." },
    { "GGPO", "Tony Cannon", "ggpo.net. The rollback vocabulary; ideas only, the netcode is our own." },
    { "melee-pc", "999sian and contributors", "github.com/999sian/melee-pc. Rollback snapshot design; ideas only." },
    { "MeleeVS", "Joyastick", "github.com/Joyastick/melee-pc-TagFighter. Benching and reserve fighters; ideas only." },
    { "Project M and Project+", "their development teams", "github.com/Fracture17/ProjectMCodes. Turbo mode prior art; ideas only, our code is original." },
    { "Blender", "the Blender Foundation. GPL", "blender.org. Modelling and animating our original fighter art." },
    { "glTF 2.0", "Khronos", "khronos.org/gltf. The model interchange format of the authored-fighter converter." },
    { "HSDLib", "Ploaj", "github.com/Ploaj/HSDLib. Writes the converter's model files." },
    { "Playwright", "Microsoft. Apache 2.0", "playwright.dev. Renders the menu mockups and the README art." },
    { "Path of Exile", "Grinding Gear Games", "pathofexile.com. The model for Envoy's loot; ideas only." },
    { "Super Smash Bros. Ultimate", "Nintendo, Bandai Namco, Sora Ltd.", "Design reference for the companion and stage switching; ideas only." },
    { "and more", "see CREDITS.md", "Every project, finding and idea we use or learned from is in CREDITS.md." },
};

static void gs_ui_credits_view(GsUiSlot *u) {
    int i = u->view.focus.index;
    memset(&u->view.ex, 0, sizeof u->view.ex);
    u->view.ex.media_model = u->view.ex.media_ring = AT_NO_MODEL;
    if (i < 0 || i >= (int) (sizeof gs_credits / sizeof gs_credits[0])) return;
    u->view.ex.has = 1;
    snprintf(u->view.ex.kicker, sizeof u->view.ex.kicker, "CREDITS");
    snprintf(u->view.ex.title, sizeof u->view.ex.title, "%s", gs_credits[i].label);
    snprintf(u->view.ex.what, sizeof u->view.ex.what, "%s", gs_credits[i].what);
}

/* push the Credits screen over the main menu (the game's More > Credits calls this) */
int gw_Ui_OpenCredits(void) {
    int i, slot;
    if (!gw_Ui_Begin("more.credits", AT_PRIMARY_LIST, 0, "CREDITS")) return 0;
    gw_Ui_Trail("MAIN MENU");
    for (i = 0; i < (int) (sizeof gs_credits / sizeof gs_credits[0]) && i < AT_MAX_ITEMS; i++) {
        AtItem *it = &gs_ui_scratch.items[gs_ui_scratch.n_items++];
        memset(it, 0, sizeof *it);
        snprintf(it->id, sizeof it->id, "c%d", i);
        snprintf(it->label, sizeof it->label, "%s", gs_credits[i].label);
        snprintf(it->sub, sizeof it->sub, "%s", gs_credits[i].sub);
    }
    gw_Ui_Key('B', "Back");
    gs_ui_scratch_ex.has = 0;
    if (!gw_Ui_Commit(0, 0)) return 0;
    slot = gs_ui_find("more.credits");
    if (slot < 0) return 0;
    gs_ui_slot[slot].self_close = 1;
    gs_ui_credits_view(&gs_ui_slot[slot]);
    return 1;
}

/* ---- entries at run time: the registry, gd.ui.entry, on_entry ------------------------------------------------------
 * The registry (gw_ui_registry.c) holds the menu tree's entries: the built-ins the adapter registers (gw_Ui_EntryBuiltin) and what
 * each ACTIVE mod's mod.json "menus" declares (read once, on the first use). The native menu shows an entry as a MOD tile; choosing
 * it comes back here (gw_Ui_EntryActivate): an "opens" entry pushes the mod's own registered screen, a "script" entry runs the
 * mod's on_entry(id) as that mod (its budget, its owner). A switched-off or unloaded mod's entries leave with it. An entry is
 * never shown, nor activated, over a netplay session unless it says "online": true. */

static AtRegistry gs_ui_reg;
static int gs_ui_reg_booted;

static int gs_ui_netplay(void) { return gw_RB_Enabled() || gw_Netplay_Enabled(); }

static void gs_ui_reg_log(void) {
    int i;
    for (i = 0; i < gs_ui_reg.nlog; i++) gw_log("ui: registry: %s", gs_ui_reg.log[i]);
    gs_ui_reg.nlog = 0;
}

static void gs_ui_reg_boot(void) {
    int i, k;
    if (gs_ui_reg_booted) return;
    gs_ui_reg_booted = 1;
    at_reg_init(&gs_ui_reg);
    for (i = 0; i < gw_Mods_Count(); i++) {
        for (k = 0; k < gw_Mods_MenuCount(i); k++) {
            AtEntry e;
            memset(&e, 0, sizeof e);
            snprintf(e.mod, sizeof e.mod, "%s", gw_Mods_Id(i));
            snprintf(e.id, sizeof e.id, "%s", gw_Mods_MenuField(i, k, "id"));
            snprintf(e.parent, sizeof e.parent, "%s", gw_Mods_MenuField(i, k, "parent"));
            snprintf(e.label, sizeof e.label, "%s", gw_Mods_MenuField(i, k, "label"));
            snprintf(e.blurb, sizeof e.blurb, "%s", gw_Mods_MenuField(i, k, "blurb"));
            snprintf(e.icon, sizeof e.icon, "%s", gw_Mods_MenuField(i, k, "icon"));
            snprintf(e.after, sizeof e.after, "%s", gw_Mods_MenuField(i, k, "after"));
            snprintf(e.opens, sizeof e.opens, "%s", gw_Mods_MenuField(i, k, "opens"));
            e.action = strcmp(gw_Mods_MenuField(i, k, "action"), "script") == 0 ? AT_ENTRY_SCRIPT : AT_ENTRY_OPENS;
            e.online = strcmp(gw_Mods_MenuField(i, k, "online"), "true") == 0;
            e.visible = 1;
            at_reg_add(&gs_ui_reg, &e);
        }
    }
    gs_ui_reg_log();
}

/* ---- the retail takeover (spec 6.8): the element mask and the retail pause ---------------------------------------------------
 * Game code calls these unprefixed (Ui_RetailHidden, Ui_RetailPause, Ui_TakeUnpause; melee/CLAUDE.md "The shim boundary"). The mask is
 * EMPTY by default: with every source empty each guard's code path is retail's own. It reads 0 while netplay or rollback is on, so
 * nothing retail is ever hidden online, and a guard never skips a proc, only a render callback or a panel's visibility. */
static int gs_ui_retail_env_done;

static int gs_ui_online(void) { return gs_ui_netplay(); }

/* MELEE_ATLAS_RETAIL="hud.damage,hud.stock" (or "all"): read once, into the environment source; a bad name is logged and ignored */
static void gs_ui_retail_env(void) {
    const char *e;
    unsigned m = 0;
    char err[96];
    if (gs_ui_retail_env_done) return;
    gs_ui_retail_env_done = 1;
    e = getenv("MELEE_ATLAS_RETAIL");
    if (e == NULL || e[0] == '\0') return;
    if (!at_retail_parse(e, &m, err, sizeof err)) { gw_log("ui: MELEE_ATLAS_RETAIL ignored: %s", err); return; }
    at_retail_set(&gs_ui_retail, AT_RS_ENV, m);
    {
        char names[160];
        int id, n = 0;
        names[0] = '\0';
        for (id = 0; id < AT_RE_COUNT; id++)
            if (m >> id & 1u) n += snprintf(names + n, sizeof names - (size_t) n, "%s%s", n > 0 ? "," : "", at_retail_name(id));
        gw_log("ui: retail elements hidden by MELEE_ATLAS_RETAIL: %s", names);
    }
}

/* the Atlas pause takeover is off unless MELEE_ATLAS_PAUSE=1 or the setting atlas_pause is on */
static int gs_ui_pause_wanted(void) {
    static int env = -1;
    if (env < 0) {
        const char *e = getenv("MELEE_ATLAS_PAUSE");
        env = e != NULL && strcmp(e, "1") == 0;
        if (env) gw_log("ui: the Atlas pause takeover is on (MELEE_ATLAS_PAUSE=1)");
    }
    return env || gw_Settings_Int("atlas_pause", 0) != 0;
}

int gw_Ui_RetailHidden(int id) {
    gs_ui_retail_env();
    return at_retail_hidden(&gs_ui_retail, id, gs_ui_online());
}

void gw_Ui_RetailPause(int pauser, int on) {
    if (on) at_pause_on(&gs_ui_pause, pauser, gs_ui_pause_wanted(), gs_ui_online());
    else at_pause_off(&gs_ui_pause);
    gs_ui_pause_changed = 1;
}

int gw_Ui_TakeUnpause(void) { return gs_ui_online() ? -1 : at_pause_take_unpause(&gs_ui_pause); }

/* ---- the HUD layer: gd.ui.hud and gd.ui.toast (spec 6.8 part 3) ----------------------------------------------------------------
 * A HUD is a script's presentation description, held here and drawn every rendered frame UNDER the screen stack (gs_ui_hud_draw, called
 * before gs_ui_draw). It is never on the stack, never takes input and never calls Lua: its record changes only from script hooks (which
 * do not run on a resimulated frame), its timings are on the UI clock, and it draws online. One HUD per script, four in all. It belongs to
 * the script that set it: the unload, a switch-off and a scene change release it (gs_ui_release, gs_ui_scene_changed). The embedding file
 * defines gs_ui_port_read (a port card's name, percent and stocks come from the fighter readbacks gd.player uses). */
#define GS_UI_HUDS 4
static AtHud gs_ui_hud[GS_UI_HUDS];
static int gs_ui_hud_warned[GS_UI_HUDS];
static int gs_ui_keepout_show;                 /* console: atlas keepout on - the retail keep-out rectangles as 1 px rose outlines */
static int gs_ui_port_read(int port, char *name, int cap, int *percent, int *stocks, int *cpu);   /* 1 when that port has a fighter */

static AtHud *gs_ui_hud_find(int owner1) {
    int i;
    for (i = 0; i < GS_UI_HUDS; i++) if (gs_ui_hud[i].owner == owner1) return &gs_ui_hud[i];
    return NULL;
}

/* a script goes (unload, switch-off): its HUD and its claim on the retail mask go with it */
static void gs_ui_hud_release(int script) {
    int i;
    if (script < 0) return;
    for (i = 0; i < GS_UI_HUDS; i++) if (gs_ui_hud[i].owner == script + 1) { memset(&gs_ui_hud[i], 0, sizeof gs_ui_hud[i]); gs_ui_hud_warned[i] = 0; }
    at_retail_release_script(&gs_ui_retail, script + 1);
}

/* a scene ends: the script's mask claim is released with every toast and note (a HUD's strips and banners are re-described by the script) */
static void gs_ui_scene_changed(void) {
    int i, z, k;
    if (gs_ui_retail.script_owner > 0) at_retail_release_script(&gs_ui_retail, gs_ui_retail.script_owner);
    for (i = 0; i < GS_UI_HUDS; i++) {
        AtHud *h = &gs_ui_hud[i];
        if (h->owner == 0) continue;
        for (z = 0; z < AT_Z_COUNT; z++) {
            int w = 0;
            for (k = 0; k < h->n[z]; k++) {
                if (h->z[z][k].kind == AT_HP_TOAST || h->z[z][k].kind == AT_HP_NOTE) continue;
                if (w != k) h->z[z][w] = h->z[z][k];
                w++;
            }
            for (k = w; k < h->n[z]; k++) memset(&h->z[z][k], 0, sizeof h->z[z][k]);
            h->n[z] = w;
        }
    }
    at_pause_off(&gs_ui_pause);                                                   /* a scene that ends mid-pause leaves no pause behind */
    gs_ui_pause_changed = 1;
}

static int gs_ui_part_live(const AtHudPart *p, double now) { return !(p->until_ms > 0.0 && now >= p->until_ms); }

/* gd.ui.hud{ id = "mod.hud", zones = { top_left = { {kind="strip", ...} }, ... } } -> true. Replaces the caller's HUD; its live toasts stay
 * (a toast is set by gd.ui.toast and must not vanish when the strip is re-described) and a note keeps its clock while its text is the same. */
static int l_ui_hud(lua_State *L) {
    char ownerbuf[64], err[160], why[96];
    const char *om = gs_ui_owner_mod(ownerbuf, sizeof ownerbuf);
    int root, z, k, j, i, slot = -1;
    double now = gs_now_ms();
    AtHud nh, *old;
    luaL_checktype(L, 1, LUA_TTABLE);
    if (om == NULL) return luaL_error(L, "gd.ui.hud: a HUD belongs to a mod script (the console has none)");
    atv_init(&gs_ui_arena);
    root = gs_ui_copy(L, 1, 0);
    if (root < 0 || gs_ui_arena.overflow) { gs_ui_unref_arena_except(L, NULL, 0); return luaL_error(L, "gd.ui.hud: the description is too large or nested too deeply"); }
    k = at_hud_from_val(&gs_ui_arena, root, om, now, &nh, err, sizeof err);
    gs_ui_unref_arena_except(L, NULL, 0);                                         /* a HUD holds no function: any the table carried go */
    if (!k) return luaL_error(L, "%s", err);
    old = gs_ui_hud_find(gs.cur + 1);
    if (old != NULL) {
        for (z = 0; z < AT_Z_COUNT; z++) {
            int has_toast = 0;
            for (k = 0; k < nh.n[z]; k++) {
                if (nh.z[z][k].kind == AT_HP_TOAST) has_toast = 1;
                if (nh.z[z][k].kind != AT_HP_NOTE) continue;
                for (j = 0; j < old->n[z]; j++)                                    /* the same note keeps its clock: a rebuild does not restart it */
                    if (old->z[z][j].kind == AT_HP_NOTE && strcmp(old->z[z][j].text, nh.z[z][k].text) == 0 && gs_ui_part_live(&old->z[z][j], now)) {
                        nh.z[z][k].from_ms = old->z[z][j].from_ms; nh.z[z][k].until_ms = old->z[z][j].until_ms;
                    }
            }
            for (j = 0; j < old->n[z] && !has_toast; j++) {
                if (old->z[z][j].kind != AT_HP_TOAST || !gs_ui_part_live(&old->z[z][j], now) || nh.n[z] >= AT_HUD_PER_ZONE) continue;
                for (k = nh.n[z]; k > 0; k--) nh.z[z][k] = nh.z[z][k - 1];         /* a live toast stays on top of its zone */
                nh.z[z][0] = old->z[z][j];
                nh.n[z]++;
                has_toast = 1;
            }
        }
        if (!at_hud_cap_ok(&nh, why, sizeof why)) return luaL_error(L, "gd.ui.hud: %s", why);
        slot = (int) (old - gs_ui_hud);
    } else {
        for (i = 0; i < GS_UI_HUDS; i++) if (gs_ui_hud[i].owner == 0) { slot = i; break; }
        if (slot < 0) return luaL_error(L, "gd.ui.hud: too many HUDs (%d)", GS_UI_HUDS);
    }
    nh.owner = gs.cur + 1;
    gs_ui_hud[slot] = nh;
    gs_ui_hud_warned[slot] = 0;
    lua_pushboolean(L, 1);
    return 1;
}

static int l_ui_hud_clear(lua_State *L) {
    AtHud *h;
    char ownerbuf[64];
    const char *om = gs_ui_owner_mod(ownerbuf, sizeof ownerbuf);
    if (om == NULL) return luaL_error(L, "gd.ui.hud_clear: a HUD belongs to a mod script (the console has none)");
    h = gs_ui_hud_find(gs.cur + 1);
    if (h != NULL && lua_isstring(L, 1) && strcmp(lua_tostring(L, 1), h->id) != 0) h = NULL;
    if (h != NULL) { int i = (int) (h - gs_ui_hud); memset(h, 0, sizeof *h); gs_ui_hud_warned[i] = 0; }
    lua_pushboolean(L, h != NULL);
    return 1;
}

/* gd.ui.toast{ zone = "top_left" | "top_right", title =, text =, rgba =, seconds = }: one toast per zone of the caller's HUD (a new one
 * replaces the old), 0.5 to 15 s, 4 by default. A script with no HUD gets an empty one, "<mod>.toast". */
static int l_ui_toast(lua_State *L) {
    char ownerbuf[64], idbuf[AT_ID_HUD], why[96];
    const char *om = gs_ui_owner_mod(ownerbuf, sizeof ownerbuf), *zn, *s;
    int z, k, slot = -1, i;
    double secs, now = gs_now_ms();
    AtHud *h;
    AtHudPart p, *dst;
    luaL_checktype(L, 1, LUA_TTABLE);
    if (om == NULL) return luaL_error(L, "gd.ui.toast: a toast belongs to a mod script (the console has none)");
    lua_getfield(L, 1, "zone");
    zn = luaL_optstring(L, -1, "top_right");
    z = at_hud_zone_by_name(zn);
    if (z != AT_Z_TOP_LEFT && z != AT_Z_TOP_RIGHT) { lua_pop(L, 1); return luaL_error(L, "gd.ui.toast: zone is \"top_left\" or \"top_right\""); }
    lua_pop(L, 1);
    memset(&p, 0, sizeof p);
    p.kind = AT_HP_TOAST;
    lua_getfield(L, 1, "title"); snprintf(p.text, AT_STR, "%s", luaL_optstring(L, -1, "")); lua_pop(L, 1);
    lua_getfield(L, 1, "text"); snprintf(p.rule, AT_TEXT, "%s", luaL_optstring(L, -1, "")); lua_pop(L, 1);
    lua_getfield(L, 1, "rgba");
    if (lua_isnumber(L, -1)) { double d = lua_tonumber(L, -1); p.rgba = (d >= 0.0 && d <= 4294967295.0) ? (unsigned) d : 0u; }
    lua_pop(L, 1);
    secs = gs_kit_optnum(L, 1, "seconds", 4.0);
    if (secs != secs || secs < 0.5) secs = 0.5;
    if (secs > 15.0) secs = 15.0;
    p.from_ms = now; p.until_ms = now + secs * 1000.0;
    s = om;
    h = gs_ui_hud_find(gs.cur + 1);
    if (h == NULL) {
        for (i = 0; i < GS_UI_HUDS; i++) if (gs_ui_hud[i].owner == 0) { slot = i; break; }
        if (slot < 0) return luaL_error(L, "gd.ui.toast: too many HUDs (%d)", GS_UI_HUDS);
        h = &gs_ui_hud[slot];
        memset(h, 0, sizeof *h);
        snprintf(idbuf, sizeof idbuf, "%s.toast", s);
        snprintf(h->id, sizeof h->id, "%s", idbuf);
        h->owner = gs.cur + 1;
    }
    dst = NULL;
    for (k = 0; k < h->n[z]; k++) if (h->z[z][k].kind == AT_HP_TOAST) { dst = &h->z[z][k]; break; }   /* replaces, never queues */
    if (dst == NULL) {
        if (h->n[z] >= AT_HUD_PER_ZONE) return luaL_error(L, "gd.ui.toast: zone %s is full (%d parts)", zn, AT_HUD_PER_ZONE);
        for (k = h->n[z]; k > 0; k--) h->z[z][k] = h->z[z][k - 1];
        dst = &h->z[z][0];
        h->n[z]++;
    }
    *dst = p;
    if (!at_hud_cap_ok(h, why, sizeof why)) return luaL_error(L, "gd.ui.toast: %s", why);
    lua_pushboolean(L, 1);
    return 1;
}

/* a copy of the HUD for this frame: a port card's name, percent and stocks read from the fighter, a card for a port with none left out */
static void gs_ui_hud_fill_ports(AtHud *h) {
    int z, k;
    for (z = 0; z < AT_Z_COUNT; z++) {
        int w = 0;
        for (k = 0; k < h->n[z]; k++) {
            AtHudPart *p = &h->z[z][k];
            if (p->kind == AT_HP_PORT_CARD) {
                int pct = 0, st = 0, cpu = 0;
                char name[AT_STR];
                if (!gs_ui_port_read(p->port.port, name, sizeof name, &pct, &st, &cpu)) continue;
                snprintf(p->port.name, sizeof p->port.name, "%s", name);
                p->port.percent = pct; p->port.stocks = st; p->port.cpu = cpu;
            }
            if (w != k) h->z[z][w] = h->z[z][k];
            w++;
        }
        h->n[z] = w;
    }
}

static void gs_ui_hud_warn_once(int i, const char *msg) {
    if (gs_ui_hud_warned[i]) return;
    gs_ui_hud_warned[i] = 1;
    gw_log("ui: HUD %s: %s", gs_ui_hud[i].id, msg);
}

/* the HUD layer: every script's HUD, under the screen stack, every rendered frame. Never calls Lua. Draws in a match only. */
static void gs_ui_hud_draw(void) {
    int i, entries;
    double now;
    float w;
    AtKeepOut ko;
    AtHudLayout l;
    AtTextOps ops;
    AtSink sink;
    static AtHud frame_copy;
    if (!gs.match_active || !gs_ui_roles_ok()) return;
    now = gs_now_ms();
    w = gw_Console_ScriptWidth();
    gs_ui_reduced = gw_Settings_Int("reduced_motion", 0) != 0;
    at_hud_retail_keepouts(w, ~at_retail_effective(&gs_ui_retail, gs_ui_online()), &ko);
    ops.width = gs_ui_width; ops.user = NULL;
    sink.user = NULL; sink.poly = gs_ui_poly; sink.text = gs_ui_text; sink.model = gs_ui_model_draw;
    if (gs_ui_keepout_show) {
        for (i = 0; i < ko.n; i++) {                                              /* 1 px outlines, for measuring them against the retail HUD */
            AtRect r = ko.r[i];
            at_poly_rect(&sink, r.x, r.y, r.w, 1.0f, AT_C_ROSE); at_poly_rect(&sink, r.x, r.y + r.h - 1.0f, r.w, 1.0f, AT_C_ROSE);
            at_poly_rect(&sink, r.x, r.y, 1.0f, r.h, AT_C_ROSE); at_poly_rect(&sink, r.x + r.w - 1.0f, r.y, 1.0f, r.h, AT_C_ROSE);
        }
    }
    for (i = 0; i < GS_UI_HUDS; i++) {
        int first, quads;
        if (gs_ui_hud[i].owner <= 0 || !gs_ui_usable(gs_ui_hud[i].owner - 1)) continue;
        if (gw_Kit_QuadRoom() < AT_HUD_QUAD_CAP) { gs_ui_hud_warn_once(i, "not drawn: the kit quad list is nearly full"); continue; }
        frame_copy = gs_ui_hud[i];
        gs_ui_hud_fill_ports(&frame_copy);
        at_hud_layout(&frame_copy, w, &ko, now, &ops, &l);
        if (l.dropped) gs_ui_hud_warn_once(i, "a part did not fit and is not shown");
        entries = 0;
        first = gw_Kit_QuadCount();
        at_hud_render(&frame_copy, &l, now, gs_ui_reduced, &ops, &sink, &entries);
        quads = gw_Kit_QuadCount() - first;
        gs_kit_record(first, quads);
    }
    gw_Kit_SetTracking(0.0f);
}

/* ---- gd.ui.retail_hide and gd.ui.retail ---------------------------------------------------------------------------------------- */
/* gd.ui.retail_hide{ "hud.damage", ... } claims the script source of the retail mask (offline, a gameplay mod script, in a match); {} releases it.
 * The checks run in this order: online, console or not a gameplay script, no match, another script's claim, an id a mod may not hide. */
static int l_ui_retail_hide(lua_State *L) {
    int n, i;
    unsigned m = 0, add;
    char err[96];
    const GsScript *me;
    err[0] = 0;
    luaL_checktype(L, 1, LUA_TTABLE);
    if (gs_ui_online()) return luaL_error(L, "gd.ui.retail_hide: not available online (nothing retail is hidden online)");
    me = (gs.cur >= 0 && gs.cur < gs.n) ? &gs.s[gs.cur] : NULL;
    if (gs.cur == gs.console || me == NULL || !me->gameplay) return luaL_error(L, "gd.ui.retail_hide: needs a gameplay mod script");
    if (!gs.match_active) return luaL_error(L, "gd.ui.retail_hide: needs an active match");
    if (gs_ui_retail.script_owner != 0 && gs_ui_retail.script_owner != gs.cur + 1) return luaL_error(L, "gd.ui.retail_hide: the mask belongs to another script");
    n = (int) lua_rawlen(L, 1);
    for (i = 1; i <= n; i++) {
        lua_rawgeti(L, 1, i);
        if (lua_type(L, -1) != LUA_TSTRING) { lua_pop(L, 1); return luaL_error(L, "gd.ui.retail_hide: element names are strings"); }
        if (!at_retail_parse(lua_tostring(L, -1), &add, err, sizeof err) || strcmp(lua_tostring(L, -1), "all") == 0) {
            lua_pop(L, 1);
            return luaL_error(L, "gd.ui.retail_hide: %s", err[0] != '\0' ? err : "name the elements, not \"all\"");
        }
        if ((add & ~at_retail_script_allowed()) != 0u) { lua_pop(L, 1); return luaL_error(L, "gd.ui.retail_hide: hud.timer may not be hidden by a mod"); }
        m |= add;
        lua_pop(L, 1);
    }
    at_retail_set(&gs_ui_retail, AT_RS_SCRIPT, m);
    gs_ui_retail.script_owner = m != 0u ? gs.cur + 1 : 0;
    lua_pushboolean(L, 1);
    return 1;
}

/* gd.ui.retail() -> { hidden = {names}, paused = bool, pauser = n or nil, takeover = bool }: what is hidden now (all sources; empty online) */
static int l_ui_retail(lua_State *L) {
    int id, n = 0;
    unsigned m;
    gs_ui_retail_env();
    m = at_retail_effective(&gs_ui_retail, gs_ui_online());
    lua_newtable(L);
    lua_newtable(L);
    for (id = 0; id < AT_RE_COUNT; id++) if (m >> id & 1u) { lua_pushstring(L, at_retail_name(id)); lua_rawseti(L, -2, ++n); }
    lua_setfield(L, -2, "hidden");
    lua_pushboolean(L, gs_ui_pause.paused); lua_setfield(L, -2, "paused");
    if (gs_ui_pause.paused) { lua_pushinteger(L, gs_ui_pause.pauser); lua_setfield(L, -2, "pauser"); }
    lua_pushboolean(L, gs_ui_pause.takeover); lua_setfield(L, -2, "takeover");
    return 1;
}

/* the console command: atlas retail <ids|clear> | atlas hud | atlas keepout on|off. The reply goes into out. */
static int gs_ui_console(const char *arg, char *out, int cap) {
    char err[96];
    unsigned m;
    int i, z, k, n = 0;
    while (*arg == ' ') arg++;
    out[0] = '\0';
    if (strncmp(arg, "retail", 6) == 0 && (arg[6] == '\0' || arg[6] == ' ')) {
        const char *list = arg + 6;
        while (*list == ' ') list++;
        if (strcmp(list, "clear") == 0) m = 0u;
        else if (list[0] == '\0') { snprintf(out, (size_t) cap, "usage: atlas retail <ids|all|clear> (hud.damage hud.stock hud.timer hud.nametag hud.magnify hud.coin hud.prize hud.hazard pause.panel)"); return -1; }
        else if (!at_retail_parse(list, &m, err, sizeof err)) { snprintf(out, (size_t) cap, "%s", err); return -1; }
        at_retail_set(&gs_ui_retail, AT_RS_CONSOLE, m);
        gw_log("ui: console retail mask: %u (effective %u%s)", m, at_retail_effective(&gs_ui_retail, gs_ui_online()), gs_ui_online() ? "; online, nothing is hidden" : "");
        snprintf(out, (size_t) cap, "retail elements hidden: %u%s", at_retail_effective(&gs_ui_retail, gs_ui_online()), gs_ui_online() ? " (online: none)" : "");
        return 0;
    }
    if (strcmp(arg, "hud") == 0) {
        AtKeepOut ko;
        AtHudLayout l;
        AtTextOps ops;
        static AtHud copy;
        double now = gs_now_ms();
        float w = gw_Console_ScriptWidth();
        at_hud_retail_keepouts(w, ~at_retail_effective(&gs_ui_retail, gs_ui_online()), &ko);
        ops.width = gs_ui_width; ops.user = NULL;
        gw_log("ui: canvas width %.1f; %d keep-out rectangle(s)", (double) w, ko.n);
        for (i = 0; i < ko.n; i++) gw_log("ui: keep-out %d: %.1f %.1f %.1f %.1f", i, (double) ko.r[i].x, (double) ko.r[i].y, (double) ko.r[i].w, (double) ko.r[i].h);
        for (i = 0; i < GS_UI_HUDS; i++) {
            if (gs_ui_hud[i].owner <= 0) continue;
            copy = gs_ui_hud[i];
            gs_ui_hud_fill_ports(&copy);
            at_hud_layout(&copy, w, &ko, now, &ops, &l);
            for (z = 0; z < AT_Z_COUNT; z++) for (k = 0; k < copy.n[z]; k++) {
                gw_log("ui: HUD %s %s[%d] kind %d %s %.1f %.1f %.1f %.1f", copy.id, at_hud_zone_name(z), k, copy.z[z][k].kind, l.shown[z][k] ? "shown" : "hidden",
                       (double) l.rect[z][k].x, (double) l.rect[z][k].y, (double) l.rect[z][k].w, (double) l.rect[z][k].h);
                n++;
            }
        }
        snprintf(out, (size_t) cap, "%d HUD part(s) logged; canvas width %.1f", n, (double) w);
        return 0;
    }
    if (strncmp(arg, "keepout", 7) == 0) {
        const char *v = arg + 7;
        while (*v == ' ') v++;
        if (strcmp(v, "on") != 0 && strcmp(v, "off") != 0) { snprintf(out, (size_t) cap, "usage: atlas keepout on|off"); return -1; }
        gs_ui_keepout_show = strcmp(v, "on") == 0;
        snprintf(out, (size_t) cap, "retail keep-out rectangles %s", gs_ui_keepout_show ? "drawn" : "hidden");
        return 0;
    }
    snprintf(out, (size_t) cap, "usage: atlas retail <ids|clear> | atlas hud | atlas keepout on|off");
    return -1;
}

/* a built-in entry (the adapter registers the native tree's rows that mods place themselves relative to) */
int gw_Ui_EntryBuiltin(const char *parent, const char *id, const char *label, int native_arg) {
    AtEntry e;
    int ok;
    gs_ui_reg_boot();
    memset(&e, 0, sizeof e);
    snprintf(e.parent, sizeof e.parent, "%s", parent != NULL ? parent : "");
    snprintf(e.id, sizeof e.id, "%s", id != NULL ? id : "");
    snprintf(e.label, sizeof e.label, "%s", label != NULL ? label : "");
    e.action = AT_ENTRY_NATIVE; e.native_arg = native_arg; e.visible = 1; e.online = 1; e.builtin = 1;
    ok = at_reg_add(&gs_ui_reg, &e);
    gs_ui_reg_log();
    return ok;
}

int gw_Ui_EntryCount(const char *parent) {
    const AtEntry *c[AT_REG_VISIBLE_PER_PARENT];
    gs_ui_reg_boot();
    return parent != NULL ? at_reg_children(&gs_ui_reg, parent, gs_ui_netplay(), c, AT_REG_VISIBLE_PER_PARENT) : 0;
}

/* only mod entries are tiles the adapter adds after its own rows, so k counts the visible children as a whole */
const char *gw_Ui_EntryField(const char *parent, int k, const char *field) {
    const AtEntry *c[AT_REG_VISIBLE_PER_PARENT];
    int n;
    gs_ui_reg_boot();
    if (parent == NULL || field == NULL) return "";
    n = at_reg_children(&gs_ui_reg, parent, gs_ui_netplay(), c, AT_REG_VISIBLE_PER_PARENT);
    if (k < 0 || k >= n) return "";
    if (strcmp(field, "id") == 0) return c[k]->id;
    if (strcmp(field, "label") == 0) return c[k]->label;
    if (strcmp(field, "blurb") == 0) return c[k]->blurb;
    if (strcmp(field, "badge") == 0) return c[k]->badge;
    if (strcmp(field, "tag") == 0) return c[k]->builtin ? "" : "MOD";
    return "";
}

/* The live script of mod `mod` (its script id is "<mod>/<name>") that defines the hook `hook`, else the first live script of the mod, or -1.
 * A mod is every .lua of its scripts/ folder, each its own script (Envoy: app, atlas_bag, ... main), and a hook such as on_entry is a global of ONE
 * of them: taking just the first script found a mod whose on_entry lives in main.lua "has no on_entry". */
static int gs_ui_mod_script(const char *mod, const char *hook) {
    size_t ml = strlen(mod);
    int i, first = -1;
    if (ml == 0) return -1;
    for (i = 0; i < gs.n; i++) {
        if (gs.s[i].used && !gs.s[i].disabled && strncmp(gs.s[i].id, mod, ml) == 0 && gs.s[i].id[ml] == '/') {
            if (first < 0) first = i;
            if (hook != NULL && gs_get_hook(i, hook)) { lua_pop(gs.L, 1); return i; }
        }
    }
    return first;
}

static int gs_ui_script_in_mod(int script, const char *mod) {
    size_t ml = strlen(mod);
    return script >= 0 && script < gs.n && ml > 0 && strncmp(gs.s[script].id, mod, ml) == 0 && gs.s[script].id[ml] == '/';
}

int gw_Ui_EntryActivate(const char *parent, int k) {
    const AtEntry *c[AT_REG_VISIBLE_PER_PARENT];
    const AtEntry *e;
    int n, owner;
    gs_ui_reg_boot();
    if (parent == NULL) return 0;
    n = at_reg_children(&gs_ui_reg, parent, gs_ui_netplay(), c, AT_REG_VISIBLE_PER_PARENT);   /* the netplay rule is the list's own */
    if (k < 0 || k >= n) { gw_log("ui: entry %s/%d refused: not available now", parent, k); return 0; }
    e = c[k];
    if (e->builtin) return 0;                                       /* a native row: the adapter runs it itself */
    owner = gs_ui_mod_script(e->mod, e->action == AT_ENTRY_OPENS ? NULL : "on_entry");
    if (owner < 0) { gw_log("ui: entry %s refused: no running script of mod %s", e->id, e->mod); return 0; }
    if (e->action == AT_ENTRY_OPENS) {
        int s = gs_ui_find(e->opens);
        if (s < 0 || !gs_ui_script_in_mod(gs_ui_slot[s].owner, e->mod) || !gs_ui_usable(gs_ui_slot[s].owner)) {
            gw_log("ui: entry %s refused: mod %s has not registered the screen %s", e->id, e->mod, e->opens);
            return 0;
        }
        return gs_ui_open_screen(s);
    }
    if (!gs_may_run(owner) || !gs_get_hook(owner, "on_entry")) { gw_log("ui: entry %s refused: mod %s has no on_entry", e->id, e->mod); return 0; }
    lua_pushstring(gs.L, e->id);
    if (gs_pcall(owner, 1, 1, "on_entry") != 0) return 0;
    gs_ui_apply_result(owner);                                      /* {push = "<mod>.<screen>"} opens one of the mod's own screens */
    return 1;
}

/* a script unloaded: its mod's entries go too, unless another script of the mod is still running */
static void gs_ui_reg_release_script(int script) {
    char mod[64];
    char *slash;
    int i;
    if (script < 0 || script >= gs.n) return;
    snprintf(mod, sizeof mod, "%s", gs.s[script].id);
    slash = strchr(mod, '/');
    if (slash == NULL) return;
    *slash = '\0';
    for (i = 0; i < gs.n; i++) if (i != script && gs.s[i].used && !gs.s[i].disabled && gs_ui_script_in_mod(i, mod)) return;
    at_reg_drop_mod(&gs_ui_reg, mod);
}

/* gd.ui.entry(id, { visible = bool, badge = "NEW" or "" }): true when the calling script's mod owns that entry */
static int l_ui_entry(lua_State *L) {
    char owner[64];
    const char *mod = gs_ui_owner_mod(owner, sizeof owner), *badge = NULL;
    int visible = -1, ok;
    const char *id = luaL_checkstring(L, 1);
    gs_ui_reg_boot();
    if (lua_istable(L, 2)) {
        lua_getfield(L, 2, "visible");
        if (lua_isboolean(L, -1)) visible = lua_toboolean(L, -1);
        lua_pop(L, 1);
        lua_getfield(L, 2, "badge");
        if (lua_type(L, -1) == LUA_TSTRING) badge = lua_tostring(L, -1);
        lua_pop(L, 1);
    }
    ok = mod != NULL && at_reg_set(&gs_ui_reg, mod, id, visible, badge);
    lua_pushboolean(L, ok);
    return 1;
}

/* 1 when the native menu must take no input: a script's screen is on top, a script holds it (gd.ui.hold_menu), or a script's screen closed
 * in this very tick (its B is not the menu's). The legacy menus (MELEE_ATLAS=0) ask this; the Atlas adapter is covered by gw_Ui_Intent. */
int gw_Ui_MenuBlocked(void) {
    int top = at_stack_top(&gs_ui_stack);
    if (gs_ui_menu_held_by != -2 || gs_ui_script_close_frame == gs_ui_frame + 1) return 1;
    return top >= 0 && gs_ui_slot[top].used && gs_ui_slot[top].owner != GS_UI_ENGINE;
}

/* ---- the Lua functions -------------------------------------------------------------------------------- */
/* gd.ui.hold_menu(on): a script that draws its own menu over a native one (Envoy's legacy menu over the frontend) holds the native
 * menu: it takes no input and draws nothing while held. Only the holder (or the console) releases it; a hold ends when the script
 * unloads or the scene ends. Returns whether the menu is held. */
static int l_ui_hold_menu(lua_State *L) {
    if (lua_toboolean(L, 1)) {
        if (gs.cur >= 0) gs_ui_menu_held_by = gs.cur;
    } else if (gs_ui_menu_held_by == gs.cur || gs.cur == gs.console) {
        gs_ui_menu_held_by = -2;
    }
    lua_pushboolean(L, gs_ui_menu_held_by != -2);
    return 1;
}

static int l_ui_available(lua_State *L) {
    int ok = gs_ui_roles_ok();
    lua_pushboolean(L, ok);
    lua_pushstring(L, ok ? "" : gs_ui_why);
    return 2;
}

/* ids the engine's own screens use: no dot (main, solo, title, settings, versus), or under more. solo. versus. settings. */
static int gs_ui_reserved_id(const char *id) {
    return strchr(id, '.') == NULL || strncmp(id, "more.", 5) == 0 || strncmp(id, "solo.", 5) == 0 || strncmp(id, "versus.", 7) == 0 ||
           strncmp(id, "settings.", 9) == 0;
}

static int l_ui_screen(lua_State *L) {
    char err[200], owner[64];
    int root, slot, nkeep, keep[40], replace = 0, old_port = 0;
    const char *om;
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
    om = gs_ui_owner_mod(owner, sizeof owner);
    if (!at_screen_from_val(&gs_ui_arena, root, om, &gs_ui_scratch, err, sizeof err)) {
        gs_ui_unref_arena_except(L, NULL, 0);
        return luaL_error(L, "%s", err);
    }
    if (om == NULL && gs_ui_reserved_id(gs_ui_scratch.id)) {                 /* the console may not take an id the engine's menus use */
        gs_ui_unref_arena_except(L, NULL, 0);
        return luaL_error(L, "gd.ui.screen: \"%s\" belongs to the engine", gs_ui_scratch.id);
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
    if (u->used && u->owner == GS_UI_ENGINE) { gs_ui_unref_screen(L, &gs_ui_scratch); return luaL_error(L, "gd.ui.screen: \"%s\" belongs to the engine", u->sc.id); }
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
    old_port = u->sc.port;
    u->sc = gs_ui_scratch;
    u->used = 1;
    u->owner = gs.cur;
    if (replace) u->view.focus = at_screen_refocus(&u->sc, old_block[0] ? old_block : NULL, old_cell[0] ? old_cell : NULL, old);
    else { AtFocusBlock fb[AT_MAX_BLOCKS]; u->view.focus = at_focus_first(fb, at_screen_focus_blocks(&u->sc, fb)); }
    u->explain_dirty = 1;
    if (replace && old_port != u->sc.port && at_stack_top(&gs_ui_stack) == slot) gs_ui_prime(slot);   /* the new port's buttons held now must be released first */
    lua_pushboolean(L, 1);
    return 1;
}

static int gs_ui_slot_arg(lua_State *L, int idx) {
    int s = gs_ui_find(luaL_checkstring(L, idx));
    if (s < 0) luaL_error(L, "gd.ui: no screen \"%s\"", lua_tostring(L, idx));
    if (gs_ui_slot[s].owner == GS_UI_ENGINE) luaL_error(L, "gd.ui: screen \"%s\" belongs to the engine", lua_tostring(L, idx));
    if (!gs_ui_may_touch(s)) luaL_error(L, "gd.ui: screen \"%s\" belongs to another script", lua_tostring(L, idx));
    return s;
}

static int l_ui_open(lua_State *L) {
    int s = gs_ui_slot_arg(L, 1);
    if (gs_ui_slot[s].sc.pause && gs_ui_online()) { lua_pushboolean(L, 0); return 1; }          /* a pause screen is never opened online */
    lua_pushboolean(L, gs_ui_open_screen(s));
    return 1;
}

/* gd.ui.pause_screen(id | nil): the caller's own pause screen (kind = "pause") that the retail pause pushes while the takeover is on.
 * The latest call wins; nil clears the named screen when it is the caller's. With the takeover off (the default) nothing is pushed. */
static int l_ui_pause_screen(lua_State *L) {
    if (lua_isnoneornil(L, 1)) {
        if (gs_ui_pause_slot >= 0 && gs_ui_may_touch(gs_ui_pause_slot)) gs_ui_pause_slot = -1;
        lua_pushboolean(L, gs_ui_pause_slot < 0);
        return 1;
    } else {
        int s = gs_ui_slot_arg(L, 1);
        if (!gs_ui_slot[s].sc.pause) return luaL_error(L, "gd.ui.pause_screen: \"%s\" is not a pause screen (kind = \"pause\")", gs_ui_slot[s].sc.id);
        gs_ui_pause_slot = s;
        lua_pushboolean(L, 1);
        return 1;
    }
}

/* gd.ui.unpause() -> true when the request was accepted: paused, offline, none pending. The game side takes it at its next unpause check
 * (Ui_TakeUnpause) and runs retail's own unpause routine for the pauser. One-shot; never online. */
static int l_ui_unpause(lua_State *L) {
    lua_pushboolean(L, at_pause_request_unpause(&gs_ui_pause, gs_ui_online()));
    return 1;
}

/* gd.ui.forget(id): close it if it is on the stack (on.close runs first), release every Lua reference it holds, free its slot. */
static int l_ui_forget(lua_State *L) {
    int slot = gs_ui_slot_arg(L, 1);                               /* raises for an unknown id, an engine screen or another script's */
    if (at_stack_top(&gs_ui_stack) == slot) gs_ui_close_screen(slot);
    if (!gs_ui_slot[slot].used) { lua_pushboolean(L, 1); return 1; }   /* on.close forgot it itself */
    gs_ui_free_slot(slot);
    lua_pushboolean(L, 1);
    return 1;
}

static int l_ui_close(lua_State *L) {
    int s = lua_isstring(L, 1) ? gs_ui_slot_arg(L, 1) : at_stack_top(&gs_ui_stack);
    if (s >= 0 && !gs_ui_may_touch(s)) s = -1;                      /* close() with no id closes the top screen only if it is the caller's */
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
    else if (strcmp(k, "l") == 0) { e.type = AT_EV_PAGE; e.a = -1; }
    else if (strcmp(k, "r") == 0) { e.type = AT_EV_PAGE; e.a = 1; }
    else if (strcmp(k, "start") == 0) e.type = AT_EV_START;
    else return luaL_error(L, "gd.ui.feed: unknown intent \"%s\" (up down left right accept back x y z l r start)", k);
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
    if (top < 0 || !gs_ui_may_touch(top)) { lua_pushboolean(L, 0); return 1; }
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
    if (top < 0 || !gs_ui_may_touch(top)) { lua_pushboolean(L, 0); return 1; }
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
    {"dialog", l_ui_dialog}, {"state", l_ui_state}, {"entry", l_ui_entry}, {"hold_menu", l_ui_hold_menu}, {"forget", l_ui_forget}, {"hud", l_ui_hud}, {"hud_clear", l_ui_hud_clear}, {"toast", l_ui_toast},
    {"retail_hide", l_ui_retail_hide}, {"retail", l_ui_retail}, {"pause_screen", l_ui_pause_screen}, {"unpause", l_ui_unpause}, {NULL, NULL}};

static void gs_push_ui(lua_State *L) {
    lua_newtable(L);
    gs_prof_setfuncs(L, gs_ui_funcs, "gd.ui");
}
