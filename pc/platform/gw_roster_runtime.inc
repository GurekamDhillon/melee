/* Included by the m-ex roster TU, avoiding a new object/link-file dependency.
 * No mutable native match map is activated here: snapshot/scene hooks must own it.
 */
#include "gw_roster_catalog.h"
#include "gw_roster_runtime.h"
#include "gw_mods.h"

static GwrCatalog gwr_boot_catalog;
static int gwr_boot_loaded;

static uint64_t gwr_disc_hash(const char* path)
{
    extern void* gw_DVDReadFileAlloc(const char*, uint32_t*);
    uint32_t size = 0;
    void* data = path ? gw_DVDReadFileAlloc(path, &size) : NULL;
    uint64_t hash = data ? gwr_bytes_hash(data, size) : 0;
    free(data);
    return hash;
}

static unsigned char* gwr_manifest_read(const char* path, size_t* size, int* present)
{
    FILE* f = fopen(path, "rb");
    unsigned char* bytes;
    long n;
    if (!f) return NULL;
    *present = 1;
    if (fseek(f, 0, SEEK_END) || (n = ftell(f)) < GWR_HEADER_BYTES ||
        n > (long) (GWR_HEADER_BYTES + GWR_NATIVE_MAX * GWR_ROW_BYTES) ||
        fseek(f, 0, SEEK_SET)) { fclose(f); return NULL; }
    bytes = (unsigned char*) malloc((size_t) n);
    if (!bytes) { fclose(f); return NULL; }
    if (fread(bytes, 1, (size_t) n, f) != (size_t) n) {
        free(bytes); fclose(f); return NULL;
    }
    fclose(f); *size = (size_t) n;
    return bytes;
}

static void gwr_boot_load(void)
{
    int i, active;
    uint64_t table_hash = 0;
    if (gwr_boot_loaded) return;
    gwr_boot_loaded = 1;
    active = gw_Mods_ActiveCount();
    for (i = 0; i < active; ++i) {
        int mod = gw_Mods_ActiveAt(i), valid = 1, present = 0;
        const char* id = gw_Mods_Id(mod);
        char path[MAX_PATH];
        unsigned char* bytes;
        size_t size = 0;
        uint32_t j, count;
        GwrCatalog candidate = {0};
        if (snprintf(path, sizeof path, "%s/%s/roster.gwr", gw_Mods_Dir(), id) >= sizeof path) {
            gw_log("roster: %s: manifest location is too long", id); continue;
        }
        bytes = gwr_manifest_read(path, &size, &present);
        if (!bytes) {
            if (present) gw_log("roster: %s: native manifest is truncated, oversized, or unreadable", id);
            continue;
        }
        if (!gwr_catalog_import(&candidate, id, bytes, size)) {
            gw_log("roster: %s: invalid version, rows, or registry capacity", id);
            free(bytes); continue;
        }
        if (!table_hash) table_hash = gwr_disc_hash("MxDt.dat");
        count = candidate.count;
        for (j = 0; j < count && valid; ++j) {
            const GwrRow* row = &candidate.rows[j];
            int k = row->source_internal, e = row->source_external;
            if (!table_hash || row->table_hash != table_hash || k < 27 ||
                k >= gw_Mex_InternalCount() - 6 || gw_Mex_InternalForExt(e) != k ||
                gw_Mex_ExtToPortCKind(e) < 0) { valid = 0; break; }
            /* Identical source rows within one manifest hash their large files once. */
            if (j && candidate.rows[j-1].source_internal == k &&
                candidate.rows[j-1].fighter_hash == row->fighter_hash &&
                candidate.rows[j-1].animation_hash == row->animation_hash) continue;
            if (row->fighter_hash != gwr_disc_hash(gw_Mex_FtPlFile(k)) ||
                row->animation_hash != gwr_disc_hash(gw_Mex_FtAnimFile(k))) valid = 0;
        }
        if (!valid || !gwr_catalog_import(&gwr_boot_catalog, id, bytes, size)) {
            gw_log("roster: %s: source content, duplicate key, or capacity differs; rows refused", id);
        } else {
            gw_log("roster: %s: %u native identities registered (wide ids, legacy tables unchanged)", id, count);
        }
        gwr_catalog_free(&candidate); free(bytes);
    }
    if (gwr_boot_catalog.count) {
        gw_log("roster: %u native rows; digest %016llx; gameplay adapter required",
               gwr_boot_catalog.count, (unsigned long long) gwr_boot_catalog.digest);
    }
}

int gw_Roster_NativeCount(void)
{ gwr_boot_load(); return (int) gwr_boot_catalog.count; }

int gw_Roster_IdAt(int index)
{
    gwr_boot_load();
    return index >= 0 && (uint32_t) index < gwr_boot_catalog.count ? 128 + index : -1;
}

const char* gw_Roster_Name(int id)
{
    const GwrRow* row;
    gwr_boot_load();
    row = id >= 128 && id < GWR_NONE ? gwr_catalog_at(&gwr_boot_catalog, (uint16_t) id) : NULL;
    return row ? row->name : "";
}

int gw_Roster_SourceCK(int id)
{
    const GwrRow* row;
    gwr_boot_load();
    row = id >= 128 && id < GWR_NONE ? gwr_catalog_at(&gwr_boot_catalog, (uint16_t) id) : NULL;
    return row ? gw_Mex_ExtToPortCKind(row->source_external) : -1;
}

int gw_Roster_SourceInternal(int id)
{
    const GwrRow* row;
    gwr_boot_load();
    row = id >= 128 && id < GWR_NONE ? gwr_catalog_at(&gwr_boot_catalog, (uint16_t) id) : NULL;
    return row ? row->source_internal : -1;
}

uint64_t gw_Roster_Hash(void)
{ gwr_boot_load(); return gwr_boot_catalog.digest; }
