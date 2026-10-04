/* Standalone discovery/content-validation/UI test against a synthetic mod folder. */
#include <assert.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define MAX_PATH 1024
static const char* test_dir;
static int wrong_content;
static int log_rows;
typedef struct GwUiFighter { int external_id, kind, has_file; } GwUiFighter;
int gw_Mods_ActiveCount(void) { return 1; }
int gw_Mods_ActiveAt(int n) { return n ? -1 : 0; }
const char* gw_Mods_Id(int n) { (void) n; return "sonic-test"; }
const char* gw_Mods_Dir(void) { return test_dir; }
void gw_log(const char* format, ...)
{
    if (strstr(format, "native identities registered")) ++log_rows;
}
void* gw_DVDReadFileAlloc(const char* path, uint32_t* size)
{
    const char* src = NULL;
    void* out;
    if (path && !strcmp(path, "MxDt.dat")) src = "table";
    if (path && !strcmp(path, "PlSn.dat")) src = wrong_content ? "wrong" : "fighter";
    if (path && !strcmp(path, "PlSnAJ.dat")) src = "animation";
    if (!src) return NULL;
    *size = (uint32_t) strlen(src); out = malloc(*size); assert(out);
    memcpy(out, src, *size); return out;
}
int gw_Mex_InternalCount(void) { return 65; }
int gw_Mex_InternalForExt(int e) { return e == 30 ? 31 : -1; }
int gw_Mex_ExtToPortCKind(int e) { return e == 30 ? 38 : -1; }
const char* gw_Mex_FtPlFile(int k) { return k == 31 ? "PlSn.dat" : NULL; }
const char* gw_Mex_FtAnimFile(int k) { return k == 31 ? "PlSnAJ.dat" : NULL; }
int gw_UI_FighterCount(void) { return 56; }
int gw_UI_FighterAt(int i, GwUiFighter* row)
{
    if (i < 0 || i >= 56) return 0;
    row->external_id = i; row->kind = i == 30 ? 38 : i; row->has_file = 1;
    return 1;
}
#include "../platform/gw_roster_runtime.inc"
#include "../platform/gw_roster_ui.inc"

int main(int argc, char** argv)
{
    assert(argc == 2); test_dir = argv[1];
    assert(gw_Roster_NativeCount() == 100 && log_rows == 1);
    assert(gw_UI_RosterCount() == 156);
    assert(gw_UI_RosterIdAt(56) == 128);
    assert(gw_UI_RosterIdAt(155) == 227);
    assert(gw_UI_RosterIdAt(93) == 165); /* 94th raw catalogue entry. */
    assert(gw_UI_RosterIdAt(94) == 166); /* 95th raw catalogue entry. */
    assert(!strcmp(gw_Roster_Name(163), "Sonic 036")); /* Wide id, not ordinal. */
    assert(gw_UI_RosterIdAt(162) == -1); /* The 163rd ordinal is absent on this base. */
    assert(gw_UI_RosterIdAt(156) == -1);
    assert(gw_UI_RosterIdAt(-1) == -1);
    assert(gw_UI_RosterSourceIconAt(56) == 30);
    assert(!strcmp(gw_Roster_Name(128), "Sonic 001"));
    assert(!strcmp(gw_Roster_Name(227), "Sonic 100"));
    assert(gw_Roster_SourceCK(200) == 38);
    assert(gw_Roster_SourceInternal(200) == 31);
    assert(gw_Roster_SourceInternal(65536) == -1);
    assert(gw_Roster_SourceCK(127) == -1);
    assert(!strcmp(gw_Roster_Name(-1), ""));
    assert(gw_Roster_Hash() != 0);
    gwr_catalog_free(&gwr_boot_catalog); gwr_boot_loaded = 0; wrong_content = 1;
    assert(gw_Roster_NativeCount() == 0); /* An incompatible source refuses every row. */
    assert(gw_UI_RosterCount() == 56); /* Legacy enumeration remains usable. */
    gwr_catalog_free(&gwr_boot_catalog);
    puts("roster runtime: PASS (100 mounted rows, wide UI, original art, content refusal, legacy fallback)");
    return 0;
}
