/* Host UI tests replace the persistent controls service, not the CSS model. */
#ifndef ATLAS_FIGHTER_CONTROLS_FAKE_H
#define ATLAS_FIGHTER_CONTROLS_FAKE_H
static unsigned ui_controls_saved[4][256];
unsigned gw_FighterControls_Get(int port, int ck) { return port >= 0 && port < 4 && ck >= 0 && ck < 256 ? ui_controls_saved[port][ck] : 0; }
int gw_FighterControls_Set(int port, int ck, unsigned choice) {
    if (port < 0 || port > 3 || ck < 0 || ck > 255) return 0;
    ui_controls_saved[port][ck] = choice; return 1;
}
int gw_FighterControls_Count(int ck, int field) { return ck == 3 ? (field == 0 ? 3 : 2) : 0; }
void gw_FighterControls_Name(int ck, int field, int index, char *out, int cap) {
    (void) ck; snprintf(out, (size_t) cap, "%s %d", field == 0 ? "Layout" : "Extra", index + 1);
}
#endif
