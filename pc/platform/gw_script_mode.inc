/* Included after the gameplay guards in gw_script.c. Lua owns the blob format;
 * the game owns its lifetime. Never serialize pointers or the Lua heap. */
#include "../gameworld/script_mode.h"
extern int gw_ScriptGame_ModeRead(int slot, int word);
extern void gw_ScriptGame_ModeWrite(int slot, int word, int value);
extern int gw_ScriptGame_ModeActive(void);
static void gs_rw_stop(void);

static uint64_t gs_mode_session(void) {
    /* File compatibility metadata only, never a simulation input or blob byte.
     * Native mesh/atlas caches are freed at scene end and cannot be resurrected
     * by gw_snap_file_load in another process/scene. */
    static uint64_t cookie;
    if (!cookie) {
        FILETIME now;
        GetSystemTimeAsFileTime(&now);
        cookie = ((uint64_t)now.dwHighDateTime << 32) | now.dwLowDateTime;
        cookie ^= (uint64_t)GetCurrentProcessId() << 32;
        if (!cookie) cookie = 1;
    }
    return cookie;
}

static uint64_t gs_mode_identity(void) {
    uint64_t hashes[GS_MAX_SCRIPTS], h = 14695981039346656037ull;
    int i, j, n = 0;
    for (i = 0; i < gs.n; ++i) {
        GsScript *s = &gs.s[i];
        uint64_t v;
        if (i == gs.console || !s->used || s->disabled || !s->gameplay) continue;
        v = gs_fnv(s->src_hash, s->version, strlen(s->version));
        /* Canonical order: script slot order is not part of blob identity. */
        for (j = n; j > 0 && hashes[j - 1] > v; --j) hashes[j] = hashes[j - 1];
        hashes[j] = v; ++n;
    }
    h = gs_fnv(h, hashes, (size_t)n * sizeof hashes[0]);
    return h ? h : 1; /* zero means no mode state in GsSaveSlot */
}

static void gs_mode_no_history(void) {
    /* A keyframe made while the director runs cannot be re-simulated later,
     * even after the director stops. Retire that history, not just requests. */
    if (gs.rw_frames || gs.rw_began) {
        gs_rw_stop();
        gs.rw_frames = 0;
        gw_log("script mode: history disabled (Lua director is not replayed)");
    }
}

static int gs_mode_slot(uint64_t hash) {
    int i;
    for (i = 0; i < SCRIPT_MODE_SLOTS; ++i) {
        if (gw_ScriptGame_ModeRead(i, 0) &&
            (uint32_t)gw_ScriptGame_ModeRead(i, 1) == (uint32_t)hash &&
            (uint32_t)gw_ScriptGame_ModeRead(i, 2) == (uint32_t)(hash >> 32)) return i;
    }
    return -1;
}

static void gs_mode_clear(int slot) {
    int i;
    for (i = 0; i < SCRIPT_MODE_WORDS; ++i) gw_ScriptGame_ModeWrite(slot, i, 0);
}

static void gs_mode_drop(int script) {
    int slot = gs_mode_slot(gs.s[script].src_hash);
    if (slot >= 0) {
        gs_rw_branch();
        gs_mode_clear(slot);
        gw_log("script [%s] mode state released", gs.s[script].id);
    }
}

/* gd.mode_blob([bytes]): nil if absent; "" clears. Snapshot state is keyed by
 * source+id hash, so script load order cannot expose another script's data. */
static int l_mode_blob(lua_State *L) {
    GsScript *s = gs_cur_script();
    int slot, i, n, old_words = 0;
    char bytes[SCRIPT_MODE_BYTES];
    gs_require_offline(L, "mode_blob");
    if (!s || gs.cur == gs.console || !gs.match_active)
        return luaL_error(L, "gd.mode_blob requires a gameplay script in an active match");
    slot = gs_mode_slot(s->src_hash);
    if (!lua_isnone(L, 1)) {
        size_t len;
        const unsigned char *src = (const unsigned char *)luaL_checklstring(L, 1, &len);
        if (len > SCRIPT_MODE_BYTES) return luaL_error(L, "mode blob exceeds 8192 bytes");
        if (gs.rw_pending_on || gs.rw_resim || gs.rw_resim_run || gs.rwt_phase || gs.hot_phase)
            return luaL_error(L, "mode state cannot change during rewind/test/hot reload");
        if (!len) {
            if (slot >= 0) { gs_rw_branch(); gs_mode_clear(slot); }
            lua_pushliteral(L, ""); return 1;
        }
        if (slot >= 0) {
            int previous = gw_ScriptGame_ModeRead(slot, 0);
            if (previous < 2 || previous > SCRIPT_MODE_BYTES + 1)
                return luaL_error(L, "invalid mode state length");
            old_words = (previous + 2) / 4;
        }
        if (slot < 0) {
            for (i = 0; i < SCRIPT_MODE_SLOTS; ++i)
                if (!gw_ScriptGame_ModeRead(i, 0)) { slot = i; break; }
            if (slot < 0) return luaL_error(L, "mode state pool full (16 scripts)");
            gw_log("script [%s] mode state allocated; rewind resimulation disabled", s->id);
        }
        gs_rw_branch();
        gs_mode_no_history();
        /* Validate first; no Lua callbacks/errors during this scalar transaction.
         * Zero unused words so shrinking a blob has canonical snapshot bytes. */
        n = (int)((len + 3) / 4);
        if (old_words > n) n = old_words;
        for (i = 0; i < n; ++i) {
            uint32_t word = 0;
            int j;
            for (j = 0; j < 4; ++j)
                if ((size_t)(i * 4 + j) < len) word |= (uint32_t)src[i * 4 + j] << (j * 8);
            gw_ScriptGame_ModeWrite(slot, i + 3, (int)word);
        }
        gw_ScriptGame_ModeWrite(slot, 1, (int)(uint32_t)s->src_hash);
        gw_ScriptGame_ModeWrite(slot, 2, (int)(uint32_t)(s->src_hash >> 32));
        gw_ScriptGame_ModeWrite(slot, 0, (int)len + 1);
        lua_settop(L, 1); return 1;
    }
    if (slot < 0) { lua_pushnil(L); return 1; }
    n = gw_ScriptGame_ModeRead(slot, 0);
    if (n < 2 || n > SCRIPT_MODE_BYTES + 1) return luaL_error(L, "invalid mode state length");
    --n;
    for (i = 0; i < n; i += 4) {
        uint32_t word = (uint32_t)gw_ScriptGame_ModeRead(slot, 3 + i / 4);
        int j;
        for (j = 0; j < 4 && i + j < n; ++j) bytes[i + j] = (char)(word >> (j * 8));
    }
    lua_pushlstring(L, bytes, (size_t)n); return 1;
}
