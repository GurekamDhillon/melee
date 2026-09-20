/* gw_overlay.h - host-side loading overlay.
 *
 * The game clears to black and presents frames for several seconds during boot while it pulls
 * ~84 sound banks off the disc (measured: window up at 0.6 s, banks 0.6-3.8 s, first frame with
 * geometry at 4.6 s). The render loop is alive that whole time, so this draws on top of those
 * otherwise-empty frames through Aurora's ImGui pass.
 *
 * It reports what is actually happening - file count, bytes, the file in flight - rather than a
 * fake percentage, because the total is not known until a boot has been observed once.
 *
 * MELEE_OVERLAY=0 disables it.
 */
#ifndef GW_OVERLAY_H
#define GW_OVERLAY_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* The DVD layer resolved a path. Cheap; called for every lookup, hit or miss. */
void gw_Overlay_NoteFile(const char *path);

/* The DVD layer transferred `bytes` bytes. */
void gw_Overlay_NoteBytes(uint32_t bytes);

/* The game submitted real geometry this frame: boot is over and the overlay stands down.
 * `prims` is the running GX primitive count. */
void gw_Overlay_NoteContent(uint32_t prims);

/* A short-lived message in the corner - for things the player did, not for load progress.
 * Safe to call from the game thread at any time, and a no-op in the headless test run. */
void gw_Overlay_Toast(const char *msg);

/* F9 opens a panel with what the port knows about the current run: scene, disc, fighters, the
 * frame counter and the pad state. It is the thing you want when something looks wrong on
 * screen and the log is 4000 lines away. */
void gw_Overlay_NotePad(int chan, unsigned buttons, int sx, int sy);
/* Once per frame, between aurora_begin_frame() and aurora_end_frame(). */
void gw_Overlay_Draw(void);

/* The F9 panel and the toast. Also once per frame, AFTER gw_Overlay_Draw so it layers on
 * top, and unconditional - it has to be reachable during a match, not just a load. */
void gw_Overlay_DrawPanel(void);

#ifdef __cplusplus
}
#endif

#endif /* GW_OVERLAY_H */
