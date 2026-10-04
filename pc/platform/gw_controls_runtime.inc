/* Included in shim_pad.c: public Aurora/SDL input access, no Aurora edits.
 * Defaults remain Aurora's device layout; our profile is applied only to the
 * physical sample, before script pads and the game's input/netplay pipeline. */
#include <SDL3/SDL_gamepad.h>
#include <SDL3/SDL_joystick.h>
#include "gw_controls_model.h"
extern void gw_Settings_SetStr(const char *, const char *);
extern int gw_Settings_LastSaveOk(void);
extern void gw_PADControlMotor(int, int);

enum { CTL_PROFILES = 4, CTL_DEVICES = 16 };
static struct {
    int connected, source, device, profile;
    char identity[192], name[64];
    GcMap map, defaults;
    unsigned char raw[GC_PHYSICAL];
    GcIntent preview;
    unsigned restore_since;
} ctl[4];
static unsigned ctl_menu_until;
static int ctl_editor;
/* GameModeKind in src/melee/gm/forward.h; GM_FRONTEND = GM_COUNT+1.
 * Scalar game accessor: no guest pointer or multibyte memory is read here. */
extern unsigned char gw_gm_GetCurrentGameMode(void);
static int ctl_input_changed(int c) {
    GcMap a = ctl[c].map, b = ctl[c].defaults;
    /* Rumble is an output preference, not an input transform. */
    a.rumble = b.rumble;
    return memcmp(&a, &b, sizeof a) != 0;
}
static int ctl_edit;
static int ctl_capture = -1, ctl_target, ctl_also, ctl_result;
static int ctl_pending = -1, ctl_pending_port;
static unsigned ctl_deadline;
static unsigned char ctl_before[4][GC_PHYSICAL];
#include "gw_controls_presets.inc"
static void ctl_key(char *key, int device, int profile, const char *field) {
    snprintf(key, 48, "ctl%02d_p%d_%s", device, profile, field);
}
static int ctl_device(const char *identity) {
    int i, empty = -1;
    char key[48], saved[192];
    for (i = 0; i < CTL_DEVICES; ++i) {
        snprintf(key, sizeof key, "ctl%02d_id", i);
        gw_Settings_Str(key, saved, sizeof saved, "");
        if (!strcmp(saved, identity)) return i;
        if (!saved[0] && empty < 0) empty = i;
    }
    if (empty >= 0) {
        snprintf(key, sizeof key, "ctl%02d_id", empty);
        gw_Settings_SetStr(key, identity);
        if (!gw_Settings_LastSaveOk()) return -1;
    }
    return empty;
}
static void ctl_load(int c) {
    char key[48], text[512];
    int p;
    ctl[c].map = ctl[c].defaults;
    if (ctl[c].device < 0) return;
    snprintf(key, sizeof key, "ctl%02d_active", ctl[c].device);
    p = gw_Settings_Int(key, 0);
    ctl[c].profile = p >= 0 && p < CTL_PROFILES ? p : 0;
    ctl_key(key, ctl[c].device, ctl[c].profile, "map");
    gw_Settings_Str(key, text, sizeof text, "");
    if (text[0] && !gc_map_decode(&ctl[c].map, text))
        gw_log("controls: invalid profile on P%d; using defaults", c+1);
}
static int ctl_save(int c) {
    char key[48], text[512];
    int i;
    if (!ctl[c].connected || ctl[c].device < 0) { ctl_result = 10; return 0; }
    ctl_key(key, ctl[c].device, ctl[c].profile, "map");
    if (gc_map_encode(&ctl[c].map, text, sizeof text)) gw_Settings_SetStr(key, text);
    else { ctl_result = 10; return 0; }
    if (!gw_Settings_LastSaveOk()) { ctl_result = 10; return 0; }
    /* Identical devices share profiles; keep their caches coherent. */
    for (i = 0; i < 4; ++i)
        if (i != c && ctl[i].connected && ctl[i].device == ctl[c].device &&
            ctl[i].profile == ctl[c].profile) ctl[i].map = ctl[c].map;
    gw_log("controls: saved P%d profile %d", c+1, ctl[c].profile+1);
    return 1;
}
static void ctl_reset(int c) {
    ctl[c].map = ctl[c].defaults;
    if (c == ctl_edit) { ctl_capture = -1; ctl_result = 5; }
    ctl_save(c);
    gw_Overlay_Toast("controller defaults restored");
}
/* Called each frontend frame, never during matches. A short lease avoids a
 * retained menu flag surviving scene teardown or a different major mode. */
void gw_Controls_Menu(int editor) {
    ctl_menu_until = GetTickCount() + 100;
    ctl_editor = editor >= 0 && editor < 4;
}
int gw_Controls_Port(void) { return ctl_edit >= 0 ? ctl_edit : 0; }
void gw_Controls_Select(int c) {
    if (c >= 0 && c < 4) {
        ctl_edit = c; ctl_capture = -1; ctl_result = 0;
        gw_log("controls: editing P%d", c+1);
    }
}
int gw_Controls_Connected(int c) { return c >= 0 && c < 4 && ctl[c].connected; }
int gw_Controls_Profile(void) { return ctl_edit >= 0 ? ctl[ctl_edit].profile : 0; }
void gw_Controls_SetProfile(int p) {
    char key[48];
    int c = ctl_edit, i;
    if (c < 0 || !ctl[c].connected || ctl[c].device < 0 || p < 0 || p >= CTL_PROFILES) return;
    snprintf(key, sizeof key, "ctl%02d_active", ctl[c].device);
    gw_Settings_SetInt(key, p);
    if (!gw_Settings_LastSaveOk()) ctl_result = 10;
    for (i = 0; i < 4; ++i)
        if (ctl[i].connected && ctl[i].device == ctl[c].device) ctl_load(i);
    ctl_capture = -1;
}
void gw_Controls_ProfileName(char *out, int cap) {
    char key[48], fallback[24];
    int c = ctl_edit;
    if (c < 0 || !ctl[c].connected) { snprintf(out, (size_t)cap, "Press a controller button"); return; }
    ctl_key(key, ctl[c].device, ctl[c].profile, "name");
    snprintf(fallback, sizeof fallback, "Profile %d", ctl[c].profile+1);
    gw_Settings_Str(key, out, cap, fallback);
}
/* Controller-only names are deliberately presets, avoiding keyboard reliance. */
void gw_Controls_ProfileAction(int action) {
    char key[48], value[32];
    int c = ctl_edit, p, names;
    static const char *const labels[] = { "Default", "Main", "Practice", "Tournament", "Friends" };
    if (c < 0 || !ctl[c].connected || ctl[c].device < 0) { ctl_result = 10; return; }
    if (action == 0) { /* new: first unused slot, copy active */
        GcMap copy = ctl[c].map;
        for (p = 0; p < CTL_PROFILES; ++p) {
            ctl_key(key, ctl[c].device, p, "map");
            gw_Settings_Str(key, value, sizeof value, "");
            if (!value[0] && p != ctl[c].profile) {
                gw_Controls_SetProfile(p); ctl[c].map = copy; if (ctl_save(c)) ctl_result = 6; return;
            }
        }
        ctl_result = 7;
    } else if (action == 1) {
        ctl_key(key, ctl[c].device, ctl[c].profile, "name");
        gw_Settings_Str(key, value, sizeof value, "");
        for (names = 0; names < 5 && strcmp(value, labels[names]); ++names) {}
        gw_Settings_SetStr(key, labels[(names+1)%5]);
        if (!gw_Settings_LastSaveOk()) ctl_result = 10;
    } else if (action == 2) {
        ctl_key(key, ctl[c].device, ctl[c].profile, "map"); gw_Settings_SetStr(key, "");
        ctl_key(key, ctl[c].device, ctl[c].profile, "name"); gw_Settings_SetStr(key, "");
        gw_Controls_SetProfile(0); ctl_load(c);
    } else if (action == 3) ctl_reset(c);
}
int gw_Controls_Assign(int port) {
    int c = ctl_edit;
    GcMap copy;
    if (c < 0 || port < 0 || port >= 4 || !ctl[c].connected || !ctl[port].connected ||
        ctl[c].source != ctl[port].source || ctl[port].device < 0) return 0;
    copy = ctl[c].map;
    ctl[port].map = copy;
    return ctl_save(port);
}
int gw_Controls_Option(int what) {
    GcMap *m;
    if (ctl_edit < 0) return 0;
    m = &ctl[ctl_edit].map;
    switch (what) {
    case 0: return m->swap; case 1: return m->analog_off;
    case 2: return m->shield; case 3: return m->dz[0];
    case 4: return m->dz[1]; case 5: return m->rumble;
    }
    return 0;
}
void gw_Controls_SetOption(int what, int value) {
    GcMap *m;
    if (ctl_edit < 0 || !ctl[ctl_edit].connected) return;
    m = &ctl[ctl_edit].map;
    switch (what) {
    case 0: m->swap = value != 0; break;
    case 1: m->analog_off = value != 0; break;
    case 2: m->shield = value < 0 ? 0 : value > 140 ? 140 : value; break;
    case 3: case 4: m->dz[what-3] = value < -1 ? -1 : value > 50 ? 50 : value; break;
    case 5: m->rumble = value != 0; if (!m->rumble) gw_PADControlMotor(ctl_edit, 0); break;
    default: return;
    }
    ctl_save(ctl_edit);
}
void gw_Controls_Preset(int preset) {
    int c = ctl_edit, i;
    if (c < 0 || !ctl[c].connected || preset < 0 || preset > 2) return;
    ctl[c].map = ctl[c].defaults;
    for (i = 0; i < ctl_presets[preset].count; ++i) {
        int physical = ctl[c].source == GW_SRC_ADAPTER ?
            ctl_presets[preset].binding[i].adapter : ctl_presets[preset].binding[i].sdl;
        if (physical >= 0) ctl[c].map.dest[physical] = 1u << ctl_presets[preset].binding[i].target;
    }
    ctl_save(c);
    gw_log("controls: preset %s: %s", ctl_presets[preset].name, ctl_presets[preset].description);
}
void gw_Controls_Capture(int target, int also) {
    int c = ctl_edit, i;
    if (c < 0 || target < -1 || target >= GC_TARGETS) return;
    if (target >= 0 && !ctl[c].connected) return;
    if (target >= 0 && ctl[c].device < 0) { ctl_result = 10; return; }
    ctl_capture = target < 0 ? -2 : c; ctl_target = target; ctl_also = also != 0;
    ctl_pending = -1;
    ctl_deadline = GetTickCount()+8000; ctl_result = 1;
    for (i = 0; i < 4; ++i) memcpy(ctl_before[i], ctl[i].raw, sizeof ctl_before[i]);
}
void gw_Controls_Cancel(void) { ctl_capture = -1; ctl_pending = -1; ctl_result = 4; }
int gw_Controls_Capturing(void) { return ctl_capture != -1; }
int gw_Controls_Result(void) { return ctl_result; }
int gw_Controls_Held(void) {
    int c = ctl_edit, i;
    if (c < 0 || !ctl[c].connected) return 0;
    for (i = 0; i < GC_PHYSICAL; ++i) {
        int threshold = i < 26 ? 1 : i < 34 ? (ctl[c].source == GW_SRC_SDL ? 64 : 40) : 140;
        if (ctl[c].raw[i] >= threshold) return 1;
    }
    return 0;
}
void gw_Controls_Binding(int target, char *out, int cap) {
    int c = ctl_edit, i, n = 0;
    out[0] = 0;
    if (c < 0 || target < 0 || target >= GC_TARGETS) return;
    for (i = 0; i < GC_PHYSICAL; ++i) if (ctl[c].map.dest[i] & (1u << target)) {
        const char *label;
        char tmp[32];
        if (i >= 26) label = gc_labels[12+i-26];
        else if (ctl[c].source == GW_SRC_ADAPTER) label = i < 12 ? gc_labels[i] : "Unused";
        else { label = PADGetNativeButtonName((u32)i); if (!label) label = "Button"; }
        snprintf(tmp, sizeof tmp, "%s%s", n ? " + " : "", label);
        if (n+(int)strlen(tmp) >= cap-1) break;
        strcpy(out+n, tmp); n += (int)strlen(tmp);
    }
    if (!n) snprintf(out, (size_t)cap, "Unbound");
}
void gw_Controls_Tester(char *out, int cap) {
    GcIntent *p;
    static const char *const button_names[12] = { "A","B","X","Y","Z","START","L","R","Up","Down","Left","Right" };
    char held[48] = "";
    int i, n = 0;
    if (ctl_edit < 0 || !ctl[ctl_edit].connected) { snprintf(out, (size_t)cap, "No controller"); return; }
    p = &ctl[ctl_edit].preview;
    for (i = 0; i < 12 && n < (int)sizeof held-8; ++i)
        if (p->buttons & gc_bits[i]) n += snprintf(held+n, sizeof held-(size_t)n, "%s%s", n ? "+" : "", button_names[i]);
    snprintf(out, (size_t)cap, "%s  S%+d,%+d C%+d,%+d L%d R%d", held[0] ? held : "--",
        p->sx, p->sy, p->cx, p->cy, p->l, p->r);
}
static void ctl_defaults(int c, int source) {
    u32 count = 0;
    int i, t;
    PADButtonMapping *b;
    gc_map_default(&ctl[c].defaults);
    if (source != GW_SRC_SDL) return;
    PADRestoreDefaultMapping((u32)c);
    memset(ctl[c].defaults.dest, 0, sizeof ctl[c].defaults.dest);
    b = PADGetButtonMappings((u32)c, &count);
    for (i = 0; b && i < (int)count; ++i)
        if (b[i].nativeButton < 26)
            for (t = 0; t < 12; ++t)
                if (b[i].padButton & gc_bits[t]) ctl[c].defaults.dest[b[i].nativeButton] |= 1u << t;
    for (i = 0; i < 10; ++i) ctl[c].defaults.dest[26+i] = 1u << (12+i);
    /* SDL trigger pulls also produce the default digital clicks. */
    ctl[c].defaults.dest[34] |= 1u << 6;
    ctl[c].defaults.dest[35] |= 1u << 7;
}
static void ctl_poll(PADStatus *st, const int *src) {
    int c, i, major = gw_gm_GetCurrentGameMode();
    int menu = major == 0 || major == 1 || major == 8 || major == 9 ||
               major == 0x0B || major == 0x0C || major == 0x0D || major == 0x2E;
    int editor = major == 0x2E && (int)(ctl_menu_until-GetTickCount()) > 0 && ctl_editor;
    unsigned now = GetTickCount();
    if (ctl_capture != -1 && (!editor || (int)(now-ctl_deadline) >= 0)) {
        ctl_capture = -1; ctl_result = editor ? 3 : 4;
    }
    for (c = 0; c < 4; ++c) {
        SDL_Gamepad *pad = NULL;
        char identity[192];
        int index;
        if (src[c] != GW_SRC_ADAPTER && src[c] != GW_SRC_SDL) {
            ctl[c].connected = 0; ctl[c].restore_since = 0;
            if (ctl_capture == c) { ctl_capture = -1; ctl_result = 4; }
            continue;
        }
        if (src[c] == GW_SRC_ADAPTER) gc_device_identity(identity, sizeof identity, 1, c, NULL, NULL, 0, 0);
        else {
            char guid[33];
            u32 vid = 0, pid = 0;
            const char *name = PADGetName((u32)c);
            index = PADGetIndexForPort((u32)c);
            if (index >= 0) pad = PADGetSDLGamepadForIndex((u32)index);
            if (!pad) continue;
            SDL_GUIDToString(SDL_GetJoystickGUID(SDL_GetGamepadJoystick(pad)), guid, sizeof guid);
            PADGetVidPid((u32)c, &vid, &pid);
            gc_device_identity(identity, sizeof identity, 0, c, guid, name, vid, pid);
        }
        if (!ctl[c].connected || strcmp(identity, ctl[c].identity)) {
            /* A capture belongs to a physical identity, not whichever device
             * happens to occupy that game port at the next poll. */
            if (ctl_capture == c || (ctl_pending >= 0 && ctl_pending_port == c)) gw_Controls_Cancel();
            ctl[c].connected = 1; ctl[c].source = src[c]; ctl[c].profile = 0;
            snprintf(ctl[c].identity, sizeof ctl[c].identity, "%s", identity);
            ctl[c].device = ctl_device(identity);
            ctl_defaults(c, src[c]); ctl_load(c);
            ctl[c].restore_since = 0;
            gw_log("controls: P%d device %d profile %d", c+1, ctl[c].device, ctl[c].profile+1);
        }
        memset(ctl[c].raw, 0, sizeof ctl[c].raw);
        if (pad) {
            for (i = 0; i < 26 && i < SDL_GAMEPAD_BUTTON_COUNT; ++i)
                ctl[c].raw[i] = SDL_GetGamepadButton(pad, (SDL_GamepadButton)i) ? 255 : 0;
            for (i = 0; i < 4; ++i) {
                int v = SDL_GetGamepadAxis(pad, (SDL_GamepadAxis)i);
                if (i & 1) v = -v;
                v /= 256;
                ctl[c].raw[26+i*2+(v < 0)] = (unsigned char)(abs(v) > 127 ? 127 : abs(v));
            }
            for (i = 0; i < 2; ++i) {
                int v = SDL_GetGamepadAxis(pad, (SDL_GamepadAxis)(4+i));
                ctl[c].raw[34+i] = (unsigned char)(v > 0 ? v/128 : 0);
            }
        } else {
            unsigned b = gw_r16(&st[c].button);
            int v[4] = {st[c].stickX, st[c].stickY, st[c].substickX, st[c].substickY};
            for (i = 0; i < 12; ++i) ctl[c].raw[i] = (b & gc_bits[i]) ? 255 : 0;
            for (i = 0; i < 4; ++i) ctl[c].raw[26+i*2+(v[i]<0)] = (unsigned char)abs(v[i]);
            ctl[c].raw[34] = st[c].triggerLeft; ctl[c].raw[35] = st[c].triggerRight;
        }
        {
            GcMap effective = ctl[c].map;
            /* Integrate the existing global SDL deadzone as the inherited value;
             * adapter inheritance is zero (its hardware calibration remains). */
            for (i = 0; i < 2; ++i) if (effective.dz[i] < 0) {
                int legacy = gw_Settings_Int("stick_deadzone", -1);
                effective.dz[i] = pad ? (legacy < 0 ? 24 : legacy) : 0;
            }
            gc_map_apply_range(&effective, ctl[c].raw, &ctl[c].preview, pad ? 127 : 80);
            if (!ctl_input_changed(c)) {
                GcIntent *p = &ctl[c].preview;
                p->buttons = gw_r16(&st[c].button);
                p->sx = st[c].stickX; p->sy = st[c].stickY;
                p->cx = st[c].substickX; p->cy = st[c].substickY;
                p->l = st[c].triggerLeft; p->r = st[c].triggerRight;
            }
        }
        if (gc_map_restore_hold(menu && (gw_r16(&st[c].button) & 0x1200) == 0x1200,
                                now, &ctl[c].restore_since)) ctl_reset(c);
        if ((ctl_capture == c || ctl_capture == -2) &&
            (gw_r16(&st[c].button) & 0x1200) == 0x1200) gw_Controls_Cancel();
        if (ctl_capture == c || ctl_capture == -2) {
            if ((int)(now-ctl_deadline) >= 0) { ctl_capture = -1; ctl_result = 3; }
            else for (i = 0; i < GC_PHYSICAL; ++i) {
                int threshold = i < 26 ? 1 : i < 34 ? (pad ? 64 : 40) : 140;
                /* Commit on release, so rolling START+B cancels reliably and
                 * the captured button cannot immediately activate another row. */
                if (ctl_pending == i && ctl_pending_port == c && ctl[c].raw[i] < threshold) {
                    if (ctl_target >= 0) {
                        int conflict = gc_map_bind(&ctl[c].map, i, ctl_target, ctl_also);
                        if (ctl_save(c)) ctl_result = conflict ? (ctl_also ? 9 : 8) : 2;
                    } else { ctl_edit = c; ctl_result = 2; }
                    ctl_capture = -1; ctl_pending = -1; break;
                }
                if (ctl_pending < 0 && ctl[c].raw[i] >= threshold && ctl_before[c][i] < threshold) {
                    ctl_pending = i; ctl_pending_port = c;
                }
                ctl_before[c][i] = ctl[c].raw[i];
            }
        }
        /* During editor navigation retain Aurora/adapter defaults. Tester uses
         * preview, so it still shows mapped bytes while navigation is protected.
         * Untouched profiles retain the original calibration and full ranges. */
        if (!editor && ctl_input_changed(c)) {
            GcIntent *p = &ctl[c].preview;
            gw_w16(&st[c].button, p->buttons);
            st[c].stickX = p->sx; st[c].stickY = p->sy;
            st[c].substickX = p->cx; st[c].substickY = p->cy;
            st[c].triggerLeft = p->l; st[c].triggerRight = p->r;
            st[c].analogA = st[c].analogB = 0;
        }
    }
}
