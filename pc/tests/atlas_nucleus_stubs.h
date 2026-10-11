/* atlas_nucleus_stubs.h - stand-ins for the three game functions the Nucleus engine (gw_nucleus_engine.inc, included through gw_script_ui_mods.inc)
 * calls and that the host tests do not otherwise define. Include once per test that includes gw_script_ui.inc. The browser is never opened in those
 * tests, so the bodies only have to link. */
#ifndef ATLAS_NUCLEUS_STUBS_H
#define ATLAS_NUCLEUS_STUBS_H
#include <stddef.h>
#include <stdint.h>
static const char *at_nuc_mods_dir = "."; /* the local zip worker test supplies a scratch directory */
const char *gw_Mods_Dir(void) { return at_nuc_mods_dir; }
int gw_DVDFileExists(const char *path) { (void) path; return 0; }
void *gw_DVDReadFileAlloc(const char *path, uint32_t *out_size) { (void) path; if (out_size) *out_size = 0; return 0; }
int gw_Kit_TexHsdFind(const char *key) { (void) key; return -1; }
int gw_Kit_TexAddGxtex(const char *key, const uint8_t *blob, size_t len) { (void) key; (void) blob; (void) len; return -1; }
#endif
