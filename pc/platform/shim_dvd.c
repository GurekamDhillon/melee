#include "gw_profiler.h"
/* DVD shims: read the user's disc image directly.
 *
 * Aurora's DVD support is compiled out of this build (AURORA_ENABLE_DVD=OFF), and the port needs
 * the GameCube's own path/file semantics anyway, so this implements the small surface melee uses on
 * top of a plain ISO image.
 *
 * Disc layout used here: the boot block holds the DOL offset at 0x420 and the FST offset/size at
 * 0x424/0x428 (all big-endian). The FST begins with the root node -- { u8 kind; u24 name offset;
 * u32 offset; u32 length }, 12 bytes, kind 0 = file, 1 = directory. For a directory, `offset` is
 * the parent index and `length` is the index one past its children; for a file they are the
 * partition offset and byte size. The name table follows the last node, i.e. at root.length()*12,
 * and node name offsets are relative to it.
 *
 * Read completions are queued through gw_defer rather than called inline: devcom.c sets its
 * in-flight flag *after* the DVDReadAsyncPrio call, so a synchronous callback would observe a
 * half-updated request. gw_wait_idle (called from DVDGetDriveStatus) and the frame tick both drain
 * the queue.
 *
 * Mods folder: gw_mods.c reads mods/ (next to the executable, or MELEE_MODS_DIR), mods/enabled.txt
 * and each mod's mod.json, and says which mods mount and in what order (see gw_mods.h). Each file
 * in a mounting mod's payload folder (mods/<id>/files/, or mods/<id>/ itself for the legacy
 * layout) answers the disc path of its position inside it: mods/sonic/files/PlSn.dat is
 * /PlSn.dat, .../files/audio/us/sonic.ssm is /audio/us/sonic.ssm. A path that exists on the
 * disc is overridden (it keeps its disc entrynum, so nothing that cached the number notices); any
 * other path is added with a new entrynum past the disc FST. A later mod in mount order wins a
 * path both provide (logged). Names match case-insensitively. MELEE_MODS=0 mounts nothing. A
 * FastOpen'd mod file carries GW_MOD_OFFSET_FLAG | index in the disc-offset field (real disc
 * offsets stay below 2 GiB), and reads of it go to the host file instead of the image. */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "gw_hang.h"
#include "shim_vi.h"
#include "gw_overlay.h"
#include "gw_stageskin.h"
#include "gw_mods.h"

#define WIN32_LEAN_AND_MEAN
#ifdef _WIN32
#include <windows.h>
#else
#include "gw_compat_linux.h"
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define GW_DVD_FST_OFFSET_FIELD 0x424
#define GW_DVD_FST_SIZE_FIELD 0x428
#define GW_DVD_DISK_ID_SIZE 32
#define GW_DVD_STATE_END 0
#define GW_DVD_NODE_SIZE 12

static FILE *gw_iso;
static unsigned char *gw_fst;
static uint32_t gw_fst_size;
static uint32_t gw_fst_nodes;
static uint32_t gw_fst_names;
static unsigned char gw_disk_id[GW_DVD_DISK_ID_SIZE];
static bool gw_iso_attempted;

static uint32_t gw_be32(const unsigned char *p) {
  return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | (uint32_t)p[3];
}

static bool gw_iso_open(void) {
  if (gw_iso_attempted) {
    return gw_iso != NULL;
  }
  gw_iso_attempted = true;

  const char *path = gw_iso_path();
  if (path == NULL) {
    gw_log("gw: DVD used before a disc image was found");
    return false;
  }
  gw_iso = fopen(path, "rb");
  if (gw_iso == NULL) {
    gw_log("gw: cannot open disc image <disc>");
    return false;
  }

  unsigned char boot[GW_DVD_FST_SIZE_FIELD + 4];
  if (fread(boot, 1, sizeof boot, gw_iso) != sizeof boot) {
    gw_log("gw: disc image <disc> is too short");
    fclose(gw_iso);
    gw_iso = NULL;
    return false;
  }
  memcpy(gw_disk_id, boot, sizeof gw_disk_id);

  const uint32_t fst_offset = gw_be32(boot + GW_DVD_FST_OFFSET_FIELD);
  const uint32_t fst_size = gw_be32(boot + GW_DVD_FST_SIZE_FIELD);
  gw_fst = (unsigned char *)malloc(fst_size);
  if (gw_fst == NULL || fst_size < GW_DVD_NODE_SIZE) {
    gw_log("gw: cannot allocate %u bytes for the disc FST", fst_size);
    fclose(gw_iso);
    gw_iso = NULL;
    return false;
  }
  if (fseek(gw_iso, (long)fst_offset, SEEK_SET) != 0 ||
      fread(gw_fst, 1, fst_size, gw_iso) != fst_size) {
    gw_log("gw: cannot read the disc FST (%u bytes at %u)", fst_size, fst_offset);
    free(gw_fst);
    gw_fst = NULL;
    fclose(gw_iso);
    gw_iso = NULL;
    return false;
  }

  gw_fst_nodes = gw_be32(gw_fst + 8);
  gw_fst_names = gw_fst_nodes * GW_DVD_NODE_SIZE;
  gw_fst_size = fst_size;
  if (gw_fst_nodes == 0 || gw_fst_names + GW_DVD_NODE_SIZE > fst_size) {
    gw_log("gw: disc FST looks corrupt (root length %u, size %u)", gw_fst_nodes, fst_size);
    gw_fst_nodes = 0;
  }
  gw_log("gw: disc image <disc>, disk id %.6s, FST %u nodes", gw_disk_id, gw_fst_nodes);
  return true;
}

static int gw_fst_kind(uint32_t index) { return gw_fst[index * GW_DVD_NODE_SIZE]; }

static uint32_t gw_fst_name_offset(uint32_t index) {
  const unsigned char *p = gw_fst + index * GW_DVD_NODE_SIZE + 1;
  return ((uint32_t)p[0] << 16) | ((uint32_t)p[1] << 8) | (uint32_t)p[2];
}

static uint32_t gw_fst_offset(uint32_t index) {
  return gw_be32(gw_fst + index * GW_DVD_NODE_SIZE + 4);
}

static uint32_t gw_fst_length(uint32_t index) {
  return gw_be32(gw_fst + index * GW_DVD_NODE_SIZE + 8);
}

static const char *gw_fst_name(uint32_t index) {
  const uint32_t offset = gw_fst_name_offset(index);
  if (index >= gw_fst_nodes || gw_fst_names + offset >= gw_fst_size) {
    return "";
  }
  return (const char *)(gw_fst + gw_fst_names + offset);
}

/* Case-insensitive, as the SDK's DVDConvertPathToEntrynum is: disc names are lower case, and a
 * mod's (or the game's) path may not be. */
static bool gw_name_matches(const char *name, const char *component, size_t length) {
  return strlen(name) == length && _strnicmp(name, component, length) == 0;
}

/* ---- mods overlay ------------------------------------------------------------------------------ */

#define GW_MOD_OFFSET_FLAG 0x80000000u
#define GW_MOD_MAX_FILES 4096
#define GW_MOD_PATH_MAX 260

/* One mod's copy of a path. Every provider is kept (in mount order) so the stage select can serve a stage from a
 * chosen skin instead of the last mounted one (gw_stageskin.h); the file's own host/mod/mod_index/size are the
 * provider serving it now. */
#define GW_MOD_PROV_MAX 48
typedef struct gw_mod_prov {
  char host[MAX_PATH];
  char mod[64];
  int mod_index;
  uint32_t size;
} gw_mod_prov;

typedef struct gw_mod_file {
  char disc[GW_MOD_PATH_MAX]; /* disc path, '/'-separated, no leading slash */
  char host[MAX_PATH];
  char mod[64];
  int mod_index; /* gw_Mods_* index of the mod that serves this path */
  uint32_t size;
  int entrynum;
  gw_mod_prov *prov; /* all providers, mount order */
  int nprov;
  int on_disc;  /* the disc has this path too */
  int use_disc; /* serve the disc's copy (a stage's Original) */
} gw_mod_file;

static gw_mod_file *gw_mod_files;
static int gw_mod_count;
static bool gw_mods_loaded;

static int gw_iso_lookup(const char *path);

static int gw_path_ieq(const char *a, const char *b) {
  for (;; ++a, ++b) {
    char ca = *a == '\\' ? '/' : *a, cb = *b == '\\' ? '/' : *b;
    if (ca >= 'A' && ca <= 'Z') ca = (char)(ca - 'A' + 'a');
    if (cb >= 'A' && cb <= 'Z') cb = (char)(cb - 'A' + 'a');
    if (ca != cb) return 0;
    if (ca == '\0') return 1;
  }
}

static const char *gw_strip_slash(const char *p) {
  while (*p == '/' || *p == '\\') ++p;
  return p;
}

static gw_mod_file *gw_mod_by_path(const char *path) {
  int i;
  path = gw_strip_slash(path);
  for (i = 0; i < gw_mod_count; ++i) {
    if (gw_path_ieq(gw_mod_files[i].disc, path)) return gw_mod_files[i].use_disc ? NULL : &gw_mod_files[i];
  }
  return NULL;
}

static gw_mod_file *gw_mod_by_entrynum(int entrynum) {
  int i;
  for (i = 0; i < gw_mod_count; ++i) {
    if (gw_mod_files[i].entrynum == entrynum && !gw_mod_files[i].use_disc) return &gw_mod_files[i];
  }
  return NULL;
}

static void gw_mod_push_prov(gw_mod_file *m) {
  gw_mod_prov *p;
  if (m->nprov >= GW_MOD_PROV_MAX) return;
  p = (gw_mod_prov *)realloc(m->prov, (size_t)(m->nprov + 1) * sizeof *p);
  if (p == NULL) return;
  m->prov = p;
  p += m->nprov++;
  memset(p, 0, sizeof *p);
  strncpy(p->host, m->host, sizeof p->host - 1);
  strncpy(p->mod, m->mod, sizeof p->mod - 1);
  p->mod_index = m->mod_index;
  p->size = m->size;
}

static void gw_mods_scan(int mod_index, const char *mod, const char *host_dir, const char *rel,
                         int skip_meta) {
  char pattern[MAX_PATH];
  WIN32_FIND_DATAA fd;
  HANDLE h;
  snprintf(pattern, sizeof pattern, "%s\\*", host_dir);
  h = FindFirstFileA(pattern, &fd);
  if (h == INVALID_HANDLE_VALUE) return;
  do {
    char host[MAX_PATH], disc[GW_MOD_PATH_MAX];
    gw_mod_file *m;
    if (strcmp(fd.cFileName, ".") == 0 || strcmp(fd.cFileName, "..") == 0) continue;
    /* a legacy-layout mod (no files/ folder) keeps its mod.json beside its disc files */
    if (skip_meta && rel[0] == '\0' && _stricmp(fd.cFileName, "mod.json") == 0) continue;
    /* ...and its Lua scripts (gw_script.c runs <mod>/scripts/*.lua; they are not disc files) */
    if (skip_meta && rel[0] == '\0' && _stricmp(fd.cFileName, "scripts") == 0) continue;
    snprintf(host, sizeof host, "%s\\%s", host_dir, fd.cFileName);
    snprintf(disc, sizeof disc, "%s%s", rel, fd.cFileName);
    if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
      char sub[GW_MOD_PATH_MAX];
      snprintf(sub, sizeof sub, "%s/", disc);
      gw_mods_scan(mod_index, mod, host, sub, skip_meta);
      continue;
    }
    if (fd.nFileSizeHigh != 0) {
      gw_log("gw: mods: %s/%s is over 4 GiB - skipped", mod, disc);
      continue;
    }
    m = gw_mod_by_path(disc);
    if (m != NULL) {
      gw_log("gw: mods: %s overrides %s's /%s", mod, m->mod, disc);
    } else {
      if (gw_mod_count >= GW_MOD_MAX_FILES) {
        gw_log("gw: mods: more than %d files - /%s skipped", GW_MOD_MAX_FILES, disc);
        continue;
      }
      m = &gw_mod_files[gw_mod_count++];
      strncpy(m->disc, disc, sizeof m->disc - 1);
    }
    strncpy(m->host, host, sizeof m->host - 1);
    strncpy(m->mod, mod, sizeof m->mod - 1);
    m->mod_index = mod_index;
    m->size = fd.nFileSizeLow;
    gw_mod_push_prov(m);
  } while (FindNextFileA(h, &fd));
  FindClose(h);
}

/* The size a mod file contributes to the mods fingerprint (gw_Mods_NoteFile). A text file checked out or
 * packaged with CRLF line ends is the same mod as its LF copy, so its CR of every CR LF pair is not counted:
 * the 0.2.2-test1 Windows package (CRLF) and Linux package (LF) of one commit otherwise showed different
 * mod hashes (envoy#2d54 against envoy#ad28). Binary files count their exact size. The game itself still
 * reads the file as it is (m->size). */
static uint32_t gw_mod_fingerprint_size(const gw_mod_file *m) {
    static const char *const text[] = { ".lua", ".json", ".md", ".txt", ".wgsl", ".csv", ".cfg", ".toml", ".xml", ".glsl", ".yml", ".yaml", ".ini" };
    size_t n = strlen(m->host), i;
    FILE *f;
    unsigned char buf[4096];
    size_t got;
    uint32_t crlf = 0;
    int prev_cr = 0, is_text = 0;
    for (i = 0; i < sizeof text / sizeof text[0]; ++i) {
        size_t l = strlen(text[i]);
        if (n >= l && _stricmp(m->host + n - l, text[i]) == 0) is_text = 1;
    }
    if (!is_text) return m->size;
    f = fopen(m->host, "rb");
    if (f == NULL) return m->size;
    while ((got = fread(buf, 1, sizeof buf, f)) > 0) {
        for (i = 0; i < got; ++i) {
            if (buf[i] == '\n' && prev_cr) ++crlf;
            prev_cr = buf[i] == '\r';
        }
    }
    fclose(f);
    return m->size - crlf;
}

/* Mount the enabled mods (gw_mods.c decides which, and in what order) over the disc. */
static void gw_mods_load(void) {
  int n, k, i, next;

  if (gw_mods_loaded || !gw_iso_open()) return;
  gw_mods_loaded = true;
  n = gw_Mods_ActiveCount();
  if (n == 0) {
    if (gw_Mods_Count() > 0) gw_log("gw: mods: none of the %d installed mods is mounting", gw_Mods_Count());
    return;
  }

  gw_mod_files = (gw_mod_file *)calloc(GW_MOD_MAX_FILES, sizeof *gw_mod_files);
  if (gw_mod_files == NULL) return;
  for (k = 0; k < n; ++k) {
    int idx = gw_Mods_ActiveAt(k);
    int before = gw_mod_count;
    gw_mods_scan(idx, gw_Mods_Id(idx), gw_Mods_PayloadDir(idx), "", gw_Mods_PayloadIsModDir(idx));
    gw_log("gw: mods: %s (%d new paths)", gw_Mods_Id(idx), gw_mod_count - before);
  }

  /* Entrynums: an override keeps the disc's number; an addition gets one past the FST. */
  next = (int)gw_fst_nodes;
  for (i = 0; i < gw_mod_count; ++i) {
    gw_mod_file *m = &gw_mod_files[i];
    int e = gw_iso_lookup(m->disc);
    /* the netplay fingerprint covers exactly the files the game will read, and who won each */
    gw_Mods_NoteFile(m->mod_index, m->disc, gw_mod_fingerprint_size(m));
    if (e >= 0 && gw_fst_kind((uint32_t)e) == 0) {
      m->entrynum = e;
      m->on_disc = 1;
      gw_log("gw: mods:   /%s <- %s (overrides the disc file, %u bytes)", m->disc, m->mod, m->size);
    } else {
      m->entrynum = next++;
      gw_log("gw: mods:   /%s <- %s (new file, %u bytes)", m->disc, m->mod, m->size);
    }
  }
}

/* ---- stage skins (gw_stageskin.h): choose which mod's copy of a stage's files the game reads ---- */

/* The stage-kind mods (skins) that provide a file of retail stage `r`, in mount order. */
int gw_DVDStageProviders(int r, int *mods, int cap) {
  int n, k, out = 0, i, p;
  if (mods == NULL || cap <= 0 || !gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  n = gw_Mods_ActiveCount();
  for (k = 0; k < n && out < cap; ++k) {
    int idx = gw_Mods_ActiveAt(k), found = 0;
    if (strcmp(gw_Mods_Kind(idx), "stage") != 0) continue;
    for (i = 0; i < gw_mod_count && !found; ++i) {
      if (gw_Skin_FileRetail(gw_mod_files[i].disc) != r) continue;
      for (p = 0; p < gw_mod_files[i].nprov; ++p) found |= gw_mod_files[i].prov[p].mod_index == idx;
    }
    if (found) mods[out++] = idx;
  }
  return out;
}

/* The n-th file of retail stage `r` that mod `mod_index` provides (disc path and host path): 1, or 0 past the last. */
int gw_DVDStageFile(int r, int mod_index, int n, char *disc, int dcap, char *host, int hcap) {
  int i, p, seen = 0;
  if (!gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  for (i = 0; i < gw_mod_count; ++i) {
    if (gw_Skin_FileRetail(gw_mod_files[i].disc) != r) continue;
    for (p = 0; p < gw_mod_files[i].nprov; ++p) {
      if (gw_mod_files[i].prov[p].mod_index != mod_index) continue;
      if (seen++ != n) continue;
      if (disc != NULL && dcap > 0) snprintf(disc, (size_t)dcap, "%s", gw_mod_files[i].disc);
      if (host != NULL && hcap > 0) snprintf(host, (size_t)hcap, "%s", gw_mod_files[i].prov[p].host);
      return 1;
    }
  }
  return 0;
}

/* Serve every file of retail stage `r` from the provider `want` names (GW_SKIN_DEFAULT, GW_SKIN_ORIGINAL or a skin's
 * gw_Mods_* index). Called when a stage loads, never while its files are open, so one match sees one file. */
int gw_DVDSelectStage(int r, int want) {
  int i, switched = 0;
  if (!gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  for (i = 0; i < gw_mod_count; ++i) {
    gw_mod_file *m = &gw_mod_files[i];
    gw_skin_prov sp[GW_MOD_PROV_MAX];
    int p, k;
    if (gw_Skin_FileRetail(m->disc) != r || m->nprov == 0) continue;
    for (p = 0; p < m->nprov; ++p) {
      sp[p].mod_index = m->prov[p].mod_index;
      sp[p].is_skin = strcmp(gw_Mods_Kind(m->prov[p].mod_index), "stage") == 0;
    }
    k = gw_skin_select(sp, m->nprov, m->on_disc, want);
    if (k < 0) {
      switched += !m->use_disc;
      m->use_disc = 1;
      gw_log("gw: mods: /%s served from the disc", m->disc);
    } else {
      const gw_mod_prov *pv = &m->prov[k];
      switched += m->use_disc || strcmp(m->host, pv->host) != 0;
      m->use_disc = 0;
      strncpy(m->host, pv->host, sizeof m->host - 1);
      strncpy(m->mod, pv->mod, sizeof m->mod - 1);
      m->mod_index = pv->mod_index;
      m->size = pv->size;
      gw_log("gw: mods: /%s served from %s (%u bytes)", m->disc, m->mod, m->size);
    }
  }
  return switched;
}

/* ---- alternate stage folders (m-ex "dynamic alternate stages") ------------------------------------------------
 * A mod disc built with that m-ex stage project ships a tiny GrXx.dat that is only a LOADER: its PowerPC code lists a folder
 * named like the stage (GrNBa/) at stage load, picks one file in it and builds the stage from that file; the stub itself has
 * no real map. The port does not run that code, so the stub used to load as the stage (2 vertices, no models) and crash.
 * Here the overlay serves the picked alt file in place of the stub instead. The choice is made by gw_stageskin.inc.
 * Consulted, not copied: the behaviour is read from the file names and the strings of the stub the mod ships
 * (dynamicAlts.c), nothing of m-ex was copied. */
#define GW_ALT_MAX 160
#define GW_ALT_STUB_MAX (512u * 1024u)

static void gw_alt_stem(int r, char *stem, size_t cap) {
  const char *f = gw_Skin_RetailFile(r);
  char *dot;
  snprintf(stem, cap, "%s", f != NULL ? f : "");
  dot = strrchr(stem, '.');
  if (dot != NULL) *dot = '\0';
}

/* the stage's root file entry (GrNBa.dat / GrPs.usd), or -1 */
static int gw_alt_root(int r, const char *stem) {
  int i;
  for (i = 0; i < gw_mod_count; ++i) {
    const char *d = gw_mod_files[i].disc;
    size_t n = strlen(stem), l = strlen(d);
    if (gw_Skin_FileRetail(d) != r || l < n + 4 || strchr(d, '/') != NULL) continue;
    if (!gw_path_ieq(d + n, ".dat") && !gw_path_ieq(d + n, ".usd")) continue;
    if (l != n + 4) continue;
    return i;
  }
  return -1;
}

/* does this file carry the loader's strings? (a real stage is megabytes; the stub a few KB) */
static int gw_alt_is_stub(const gw_mod_file *m) {
  FILE *f;
  unsigned char *buf;
  size_t n, i;
  int hit = 0;
  if (m->size == 0 || m->size > GW_ALT_STUB_MAX) return 0;
  f = fopen(m->host, "rb");
  if (f == NULL) return 0;
  buf = (unsigned char *)malloc(m->size);
  if (buf == NULL) { fclose(f); return 0; }
  n = fread(buf, 1, m->size, f);
  fclose(f);
  for (i = 0; !hit && i + 13 <= n; ++i) {
    if (buf[i] == 'd' && memcmp(buf + i, "dynamicAlts.c", 13) == 0) hit = 1;
    else if (buf[i] == 'n' && i + 20 <= n && memcmp(buf + i, "no valid alts found ", 20) == 0) hit = 1;
  }
  free(buf);
  return hit;
}

/* "name (L).dat": the loader reads a button tag in the file name; those alts are for a held button, not for the random pool */
static int gw_alt_tagged(const char *name) {
  size_t l = strlen(name);
  const char *dot = strrchr(name, '.');
  if (dot != NULL) l = (size_t)(dot - name);
  return l >= 3 && name[l - 1] == ')' && name[l - 3] == '(' && strchr("LRZBXYlrzbxy", name[l - 2]) != NULL;
}

static int gw_alt_cmp(const void *a, const void *b) {
  const char *x = *(const char *const *)a, *y = *(const char *const *)b;
  for (;; ++x, ++y) {
    int cx = *x >= 'A' && *x <= 'Z' ? *x + 32 : *x, cy = *y >= 'A' && *y <= 'Z' ? *y + 32 : *y;
    if (cx != cy) return cx - cy;
    if (cx == 0) return 0;
  }
}

/* The alternates of retail stage `r`: names (sorted, case-insensitive), in names[]/idx[]. 0 when the stage file is not a
 * loader stub or its folder holds no file of the stage file's extension. */
static int gw_alt_collect(int r, int *root_out, const char **names, int *idx, int cap) {
  char stem[32];
  int root, i, n = 0;
  const char *ext;
  size_t sl;
  if (!gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  gw_alt_stem(r, stem, sizeof stem);
  if (stem[0] == '\0') return 0;
  root = gw_alt_root(r, stem);
  if (root < 0 || !gw_alt_is_stub(&gw_mod_files[root])) return 0;
  ext = gw_mod_files[root].disc + strlen(stem);
  sl = strlen(stem);
  for (i = 0; i < gw_mod_count && n < cap; ++i) {
    const char *d = gw_mod_files[i].disc, *base;
    size_t l = strlen(d);
    if (l <= sl + 1 || d[sl] != '/' || strchr(d + sl + 1, '/') != NULL) continue;
    { size_t k; int same = 1; for (k = 0; k < sl; ++k) { char a = d[k], b = stem[k]; if (a >= 'A' && a <= 'Z') a = (char)(a + 32); if (b >= 'A' && b <= 'Z') b = (char)(b + 32); if (a != b) same = 0; } if (!same) continue; }
    base = d + sl + 1;
    if (strlen(base) < 5 || !gw_path_ieq(base + strlen(base) - 4, ext)) continue;
    names[n] = base;
    idx[n] = i;
    ++n;
  }
  qsort(names, (size_t)n, sizeof names[0], gw_alt_cmp); /* idx must follow: rebuild it from the sorted names */
  for (i = 0; i < n; ++i) {
    int j;
    for (j = 0; j < gw_mod_count; ++j) {
      const char *d = gw_mod_files[j].disc;
      if (d + sl + 1 == names[i]) { idx[i] = j; break; }
    }
  }
  if (root_out != NULL) *root_out = root;
  return n;
}

/* k < 0: the number of alternates of the stage (0 = no loader stub, nothing to choose). k >= 0: the k-th name (sorted),
 * copied to name; *tagged = 1 for a button-tagged alternate. Returns 1 when k exists. */
int gw_DVDAltList(int r, int k, char *name, int cap, int *tagged) {
  const char *names[GW_ALT_MAX];
  int idx[GW_ALT_MAX], root, n = gw_alt_collect(r, &root, names, idx, GW_ALT_MAX);
  if (k < 0) return n;
  if (k >= n) return 0;
  if (name != NULL && cap > 0) snprintf(name, (size_t)cap, "%s", names[k]);
  if (tagged != NULL) *tagged = gw_alt_tagged(names[k]);
  return 1;
}

/* 1 when the stage file the overlay serves for retail stage `r` is a loader stub (an m-ex alternate-stage loader). */
int gw_DVDStageIsStub(int r) {
  char stem[32];
  int root;
  if (!gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  gw_alt_stem(r, stem, sizeof stem);
  if (stem[0] == 0) return 0;
  root = gw_alt_root(r, stem);
  return root >= 0 && !gw_mod_files[root].use_disc && gw_alt_is_stub(&gw_mod_files[root]);
}

/* A loader stub with nothing to load: serve the disc's own copy of the stage file instead (1 when the disc has one). */
int gw_DVDStageToDisc(int r) {
  char stem[32];
  int root;
  if (!gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  gw_alt_stem(r, stem, sizeof stem);
  root = stem[0] != 0 ? gw_alt_root(r, stem) : -1;
  if (root < 0 || !gw_mod_files[root].on_disc) return 0;
  gw_mod_files[root].use_disc = 1;
  gw_log("gw: mods: /%s served from the disc (its mod copy is a loader stub with nothing to load)", gw_mod_files[root].disc);
  return 1;
}

/* Serve alternate `name` (a file of the stage's folder) in place of the stage file. 1 when switched, 0 when not found. */
int gw_DVDAltSelect(int r, const char *name) {
  const char *names[GW_ALT_MAX];
  int idx[GW_ALT_MAX], root, k, n = gw_alt_collect(r, &root, names, idx, GW_ALT_MAX);
  if (name == NULL) return 0;
  for (k = 0; k < n; ++k) {
    if (gw_path_ieq(names[k], name)) {
      gw_mod_file *m = &gw_mod_files[root], *a = &gw_mod_files[idx[k]];
      m->use_disc = 0;
      strncpy(m->host, a->host, sizeof m->host - 1);
      strncpy(m->mod, a->mod, sizeof m->mod - 1);
      m->mod_index = a->mod_index;
      m->size = a->size;
      gw_log("gw: mods: /%s served from the alternate /%s (%u bytes, %s)", m->disc, a->disc, m->size, m->mod);
      return 1;
    }
  }
  return 0;
}

/* 1 when `path` exists on the disc or in a mounted mod. Unlike DVDConvertPathToEntrynum it does
 * not count as a load for the loading overlay, so presence checks can call it freely. */
int gw_DVDFileExists(const char *path) {
  if (path == NULL || !gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  if (gw_mod_by_path(path) != NULL) return 1;
  {
    int e = gw_iso_lookup(path);
    return e >= 0 && gw_fst_kind((uint32_t)e) == 0;
  }
}

/* Where a path is served from: 1 when a mounted mod provides it (gw_DVDFileFromMod), or when the
 * disc image itself has it, whatever the mods do (gw_DVDFileOnDisc). Presence checks only - no
 * load is counted. The results screen asks both to tell whether the disc's name art can belong
 * to a fighter a mod added (gw_Mex_ResultArtIsOwn). */
int gw_DVDFileFromMod(const char *path) {
  if (path == NULL || !gw_iso_open() || gw_fst_nodes == 0) return 0;
  gw_mods_load();
  return gw_mod_by_path(path) != NULL;
}

int gw_DVDFileOnDisc(const char *path) {
  int e;
  if (path == NULL || !gw_iso_open() || gw_fst_nodes == 0) return 0;
  e = gw_iso_lookup(path);
  return e >= 0 && gw_fst_kind((uint32_t)e) == 0;
}

/* Read `length` bytes at `offset` of mod file `m` into `dst`. Returns bytes read. */
static uint32_t gw_mod_read(const gw_mod_file *m, void *dst, uint32_t offset, uint32_t length) {
  FILE *f = fopen(m->host, "rb");
  uint32_t got = 0;
  if (f == NULL) {
    gw_log("gw: mods: cannot open %s", m->host);
    return 0;
  }
  if (fseek(f, (long)offset, SEEK_SET) == 0) {
    got = (uint32_t)fread(dst, 1, length, f);
  }
  fclose(f);
  return got;
}

void gw_DVDInit(void) {
  (void)gw_iso_open();
  gw_mods_load();
}

int gw_DVDGetDriveStatus(void) {
  gw_wait_idle();
  return GW_DVD_STATE_END;
}

int gw_DVDCheckDisk(void) { return 1; }

void *gw_DVDGetCurrentDiskID(void) { return gw_disk_id; }

int gw_DVDConvertPathToEntrynum(const char *path) {
  gw_mod_file *m;
  if (!gw_iso_open() || gw_fst_nodes == 0 || path == NULL) {
    return -1;
  }
  gw_mods_load();
  m = gw_mod_by_path(path);
  {
    /* MELEE_DVD_TRACE=1: log every path the game resolves (with its source), e.g. to find which
     * disc files a mod must carry. */
    static int trace = -1;
    if (trace < 0) {
      const char *v = getenv("MELEE_DVD_TRACE");
      trace = (v != NULL && v[0] == '1');
    }
    if (trace) {
      gw_log("gw: dvd: %s -> %s", path, m != NULL ? m->mod : "disc");
    }
  }
  /* Feeds the loading overlay's file counter and "currently loading" line. */
  gw_Overlay_NoteFile(path);
  if (m != NULL) {
    return m->entrynum;
  }
  return gw_iso_lookup(path);
}

static int gw_iso_lookup(const char *path) {

  const char *component = path;
  if (*component == '/') {
    ++component;
  }
  if (*component == '\0') {
    return 0;
  }

  uint32_t end = gw_fst_length(0);
  uint32_t index = 1;
  for (;;) {
    const char *slash = strchr(component, '/');
    const size_t length = slash != NULL ? (size_t)(slash - component) : strlen(component);
    bool found = false;

    while (index < end && index < gw_fst_nodes) {
      if (gw_name_matches(gw_fst_name(index), component, length)) {
        if (slash == NULL) {
          return (int)index;
        }
        if (gw_fst_kind(index) != 1) {
          return -1;
        }
        end = gw_fst_length(index);
        index = index + 1;
        found = true;
        break;
      }
      index = gw_fst_kind(index) == 1 ? gw_fst_length(index) : index + 1;
    }

    if (!found || slash == NULL) {
      return -1;
    }
    component = slash + 1;
  }
}

int gw_DVDFastOpen(int entrynum, void *file_info) {
  gw_mod_file *m;
  if (!gw_iso_open() || gw_fst_nodes == 0 || entrynum < 0) {
    return 0;
  }
  gw_mods_load();
  m = gw_mod_by_entrynum(entrynum);
  if (m != NULL) {
    unsigned char *info = (unsigned char *)file_info;
    gw_w32(info + 0x0C, GW_DVD_STATE_END);
    gw_w32(info + 0x10, 0);
    gw_w32(info + 0x14, m->size);
    gw_w32(info + 0x30, GW_MOD_OFFSET_FLAG | (uint32_t)(m - gw_mod_files));
    gw_w32(info + 0x34, m->size);
    return 1;
  }
  if ((uint32_t)entrynum >= gw_fst_nodes) {
    return 0;
  }
  if (gw_fst_kind((uint32_t)entrynum) != 0) {
    return 0;
  }

  const uint32_t offset = gw_fst_offset((uint32_t)entrynum);
  const uint32_t length = gw_fst_length((uint32_t)entrynum);
  unsigned char *info = (unsigned char *)file_info;
  gw_w32(info + 0x0C, GW_DVD_STATE_END);
  gw_w32(info + 0x10, 0);
  gw_w32(info + 0x14, length);
  gw_w32(info + 0x30, offset);
  gw_w32(info + 0x34, length);
  return 1;
}

int gw_DVDClose(void *file_info) {
  (void)file_info;
  return 1;
}

static void gw_dvd_complete(void *file_info, void *callback, uint32_t result) {
  if (callback != NULL) {
    ((void (*)(int, void *))callback)((int)(int32_t)result, file_info);
  }
}

/* Synchronous read of a whole disc file into a malloc'd buffer. Returns the buffer (caller
 * frees it) or NULL on failure (with *out_size set to 0). This is the headless path the m-ex
 * ftFunction loader uses to pull a fighter .dat (e.g. PlSn.dat) off the ISO without going
 * through the game's async DVD queue. */
void *gw_DVDReadFileAlloc(const char *path, uint32_t *out_size) {
  uint32_t length;
  uint32_t offset;
  void *buf;
  int entrynum;

  if (out_size != NULL) {
    *out_size = 0;
  }
  entrynum = gw_DVDConvertPathToEntrynum(path);
  {
    gw_mod_file *m = entrynum >= 0 ? gw_mod_by_entrynum(entrynum) : NULL;
    if (m != NULL) {
      buf = malloc(m->size != 0 ? m->size : 1);
      if (buf == NULL || gw_mod_read(m, buf, 0, m->size) != m->size) {
        gw_log("gw: DVDReadFileAlloc: cannot read mod file %s", m->host);
        free(buf);
        return NULL;
      }
      if (out_size != NULL) {
        *out_size = m->size;
      }
      return buf;
    }
  }
  if (entrynum < 0 || !gw_iso_open() || gw_fst_nodes == 0) {
    {
      /* A miss is a miss for the life of the disc: say it once per path (a small negative
       * cache of the paths already reported), not on every retry. */
      static char missed[8][96];
      static unsigned missed_n;
      const char *p = path != NULL ? path : "(null)";
      unsigned i, n = missed_n < 8 ? missed_n : 8;
      for (i = 0; i < n; ++i) {
        if (strcmp(missed[i], p) == 0) return NULL;
      }
      snprintf(missed[missed_n++ & 7], sizeof missed[0], "%s", p);
      gw_log("gw: DVDReadFileAlloc: %s not found on the disc image", p);
    }
    return NULL;
  }
  if (gw_fst_kind((uint32_t)entrynum) != 0) {
    return NULL;
  }
  offset = gw_fst_offset((uint32_t)entrynum);
  length = gw_fst_length((uint32_t)entrynum);
  buf = malloc(length);
  if (buf == NULL) {
    gw_log("gw: DVDReadFileAlloc: cannot allocate %u bytes for %s", length, path);
    return NULL;
  }
  if (fseek(gw_iso, (long)offset, SEEK_SET) != 0 || fread(buf, 1, length, gw_iso) != length) {
    gw_log("gw: DVDReadFileAlloc: short read of %s (%u bytes at %u)", path, length, offset);
    free(buf);
    return NULL;
  }
  if (out_size != NULL) {
    *out_size = length;
  }
  return buf;
}

/* Animation authoring needs the current host file, not the mounted size alone:
 * reject truncation/growth instead of reading a prefix of a newly rebuilt bank.
 * Keep the mounted metadata and netplay fingerprint untouched. */
void* gw_DVDReadModFileFresh(const char* path, uint32_t* out_size, char* why, int cap) {
  gw_mod_file* m;
  FILE* f;
  long length;
  void* data;
  *out_size = 0;
  gw_mods_load();
  m = path ? gw_mod_by_path(path) : NULL;
  if (!m) { snprintf(why, cap, "animation bank is not a mounted mod file"); return NULL; }
  f = fopen(m->host, "rb");
  if (!f) { snprintf(why, cap, "cannot open animation bank %s", path); return NULL; }
  if (fseek(f, 0, SEEK_END) != 0 || (length = ftell(f)) <= 0 ||
      (uint64_t)length != m->size || fseek(f, 0, SEEK_SET) != 0) {
    snprintf(why, cap, "animation bank size changed or is unreadable; restart required");
    fclose(f); return NULL;
  }
  data = malloc(m->size);
  if (!data || fread(data, 1, m->size, f) != m->size || fgetc(f) != EOF || ferror(f)) {
    snprintf(why, cap, "cannot read a complete animation bank %s", path);
    free(data); fclose(f); return NULL;
  }
  fclose(f); *out_size = m->size;
  return data;
}

/* Synchronous read of the first `length` bytes of a disc (or mod) file. Returns the bytes read,
 * 0 when the file does not exist. For headers - e.g. an .ssm's sample-data size. */
uint32_t gw_DVDReadPrefix(const char *path, void *dst, uint32_t length) {
  const int entrynum = gw_DVDConvertPathToEntrynum(path);
  gw_mod_file *m = entrynum >= 0 ? gw_mod_by_entrynum(entrynum) : NULL;
  uint32_t n;
  if (entrynum < 0) {
    return 0;
  }
  if (m != NULL) {
    return gw_mod_read(m, dst, 0, length < m->size ? length : m->size);
  }
  if ((uint32_t)entrynum >= gw_fst_nodes || gw_fst_kind((uint32_t)entrynum) != 0) {
    return 0;
  }
  n = gw_fst_length((uint32_t)entrynum);
  if (length > n) {
    length = n;
  }
  if (fseek(gw_iso, (long)gw_fst_offset((uint32_t)entrynum), SEEK_SET) != 0) {
    return 0;
  }
  return (uint32_t)fread(dst, 1, length, gw_iso);
}

int gw_DVDReadAsyncPrio_profile_body(void *file_info, void *addr, int length, int offset, void *callback,
                        int prio);
int gw_DVDReadAsyncPrio(void *file_info, void *addr, int length, int offset, void *callback,
                        int prio) {
    gw_hang_shim("DVDReadAsyncPrio");
    int result;
    gw_prof_begin(GW_PROF_FILE_READ, gw_r32((const unsigned char *)file_info + 0x30));
    result = gw_DVDReadAsyncPrio_profile_body(file_info, addr, length, offset, callback, prio);
    gw_prof_end();
    return result;
}

int gw_DVDReadAsyncPrio_profile_body(void *file_info, void *addr, int length, int offset, void *callback,
                        int prio) {
  (void)prio;

  uint32_t result = 0;
  if (gw_iso_open()) {
    const unsigned char *info = (const unsigned char *)file_info;
    const uint32_t file_offset = gw_r32(info + 0x30);
    const uint32_t file_length = gw_r32(info + 0x34);
    uint32_t want = (uint32_t)length;

    if (offset < 0 || (uint32_t)offset >= file_length) {
      want = 0;
    } else if (want > file_length - (uint32_t)offset) {
      want = file_length - (uint32_t)offset;
    }

    /* fread rejects a NULL buffer through the CRT's invalid-parameter path, which calls
     * __fastfail and takes the process down with STATUS_STACK_BUFFER_OVERRUN -- bypassing SEH, so
     * no crash handler runs and nothing is logged. A NULL destination here means a game
     * allocation failed upstream, which is worth reporting as itself rather than as a silent
     * disappearance. */
    gw_Overlay_NoteBytes(want);

    if (want != 0 && addr == NULL) {
      gw_log("gw: DVDReadAsyncPrio with a NULL destination (%u bytes at offset %d) - the caller's "
             "allocation failed",
             want, offset);
      want = 0;
      result = (uint32_t)-1;
    }

    if (want != 0 && (file_offset & GW_MOD_OFFSET_FLAG) != 0) {
      const uint32_t idx = file_offset & ~GW_MOD_OFFSET_FLAG;
      if ((int)idx >= gw_mod_count ||
          gw_mod_read(&gw_mod_files[idx], addr, (uint32_t)offset, want) != want) {
        result = (uint32_t)-1;
      }
    } else if (want != 0 &&
               (fseek(gw_iso, (long)(file_offset + (uint32_t)offset), SEEK_SET) != 0 ||
                fread(addr, 1, want, gw_iso) != want)) {
      result = (uint32_t)-1;
    }
  } else {
    result = (uint32_t)-1;
  }

  gw_defer(gw_dvd_complete, file_info, callback, result);
  return 1;
}
