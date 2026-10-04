/* Read-only mission files. Open first, validate the final handle path, then
 * read that same handle: lexical checks alone do not contain junctions. */
#define GS_MISSION_TEXT_MAX (1u << 20)
static int gs_mission_path_valid(const char *path, size_t len)
{
    size_t i, start = 0;
    if (len < 9 || len >= MAX_PATH || strlen(path) != len ||
        strncmp(path, "missions/", 9) || strstr(path, "..")) return 0;
    for (i = 0; i < len; ++i) {
        unsigned char c = (unsigned char)path[i];
        if (c < 32 || c == 127 || strchr("\\:*?\"<>|", c)) return 0;
        if (c == '/') {
            if (i == start || path[i - 1] == '.' || path[i - 1] == ' ') return 0;
            start = i + 1;
        }
    }
    return start == len || (path[len - 1] != '.' && path[len - 1] != ' ');
}

#ifdef _WIN32
static int gs_mission_inside(const char *root, const char *path)
{
    size_t n = strlen(root);
    return !_strnicmp(root, path, n) && path[n] == '\\';
}

/* Mounted roots win; the ancestor fallback supports standalone mod manifests
 * and headless fixtures. Never infer a root merely by taking two parents. */
static int gs_mission_script_root(char out[MAX_PATH])
{
    GsScript *s = gs_cur_script();
    char entry[MAX_PATH], root[MAX_PATH], manifest[MAX_PATH];
    DWORD n;
    int i;
    char *slash;
    if (!s || gs.cur == gs.console) return 0;
    n = GetFullPathNameA(s->entry, sizeof entry, entry, NULL);
    if (!n || n >= sizeof entry) return 0;
    for (i = 0; i < gw_Mods_Count(); ++i) {
        if (!gw_Mods_IsActive(i)) continue;
        if (snprintf(root, sizeof root, "%s\\%s", gw_Mods_Dir(), gw_Mods_Id(i)) >= sizeof root) continue;
        n = GetFullPathNameA(root, MAX_PATH, out, NULL);
        if (n && n < MAX_PATH && gs_mission_inside(out, entry)) return 1;
    }
    snprintf(root, sizeof root, "%s", entry);
    while ((slash = gw_path_separator(root)) != NULL) {
        *slash = 0;
        if (snprintf(manifest, sizeof manifest, "%s\\mod.json", root) >= sizeof manifest) return 0;
        n = GetFileAttributesA(manifest);
        if (n != INVALID_FILE_ATTRIBUTES && !(n & FILE_ATTRIBUTE_DIRECTORY)) {
            snprintf(out, MAX_PATH, "%s", root); return 1;
        }
    }
    return 0;
}

static HANDLE gs_mission_open_native(const char *root, const char *path, DWORD access,
                                     char final[MAX_PATH], const char **error)
{
    char canonical[MAX_PATH];
    DWORD n;
    HANDLE base, h;
    base = CreateFileA(root, FILE_READ_ATTRIBUTES, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                       NULL, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, NULL);
    if (base == INVALID_HANDLE_VALUE) { *error = "mod folder unavailable"; return INVALID_HANDLE_VALUE; }
    n = GetFinalPathNameByHandleA(base, canonical, sizeof canonical, FILE_NAME_NORMALIZED | VOLUME_NAME_DOS);
    CloseHandle(base);
    if (!n || n >= sizeof canonical) { *error = "mod folder path unavailable or too long"; return INVALID_HANDLE_VALUE; }
    while (n && canonical[n - 1] == '\\') canonical[--n] = 0;
    h = CreateFileA(path, access, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                   NULL, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS, NULL);
    if (h == INVALID_HANDLE_VALUE) { *error = "mission path unavailable"; return h; }
    n = GetFinalPathNameByHandleA(h, final, MAX_PATH, FILE_NAME_NORMALIZED | VOLUME_NAME_DOS);
    if (!n || n >= MAX_PATH || GetFileType(h) != FILE_TYPE_DISK || !gs_mission_inside(canonical, final)) {
        CloseHandle(h); *error = "mission path escapes the mod folder or is too long"; return INVALID_HANDLE_VALUE;
    }
    return h;
}

static HANDLE gs_mission_open(lua_State *L, DWORD access, char final[MAX_PATH], const char **error)
{
    size_t len;
    const char *rel = luaL_checklstring(L, 1, &len);
    char root[MAX_PATH], path[MAX_PATH];
    char *p;
    if (!gs_mission_path_valid(rel, len)) { *error = "path must start with missions/ and contain no traversal, drive or backslash"; return INVALID_HANDLE_VALUE; }
    if (!gs_mission_script_root(root)) { *error = "calling script has no mod folder"; return INVALID_HANDLE_VALUE; }
    if (snprintf(path, sizeof path, "%s\\%s", root, rel) >= sizeof path) {
        *error = "mission path too long"; return INVALID_HANDLE_VALUE;
    }
    for (p = path; *p; ++p) if (*p == '/') *p = '\\';
    len = strlen(path); if (len && path[len - 1] == '\\') path[len - 1] = 0;
    return gs_mission_open_native(root, path, access, final, error);
}

static int gs_mission_error(lua_State *L, const char *error)
{
    lua_pushnil(L); lua_pushstring(L, error); return 2;
}

static int l_mod_read(lua_State *L)
{
    char final[MAX_PATH], *text;
    const char *error = NULL;
    BY_HANDLE_FILE_INFORMATION info;
    DWORD got = 0, extra = 0;
    unsigned char tail;
    HANDLE h = gs_mission_open(L, GENERIC_READ, final, &error);
    if (h == INVALID_HANDLE_VALUE) return gs_mission_error(L, error);
    if (!GetFileInformationByHandle(h, &info) || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) ||
        info.nFileSizeHigh || info.nFileSizeLow > GS_MISSION_TEXT_MAX) {
        CloseHandle(h); return gs_mission_error(L, "mission text must be a file of at most 1 MiB");
    }
    text = malloc((size_t)info.nFileSizeLow + 1);
    if (!text) { CloseHandle(h); return gs_mission_error(L, "out of memory"); }
    if (!ReadFile(h, text, info.nFileSizeLow, &got, NULL) || got != info.nFileSizeLow ||
        !ReadFile(h, &tail, 1, &extra, NULL) || extra) {
        free(text); CloseHandle(h); return gs_mission_error(L, "mission file changed or could not be read completely");
    }
    CloseHandle(h);
    lua_pushlstring(L, text, got); free(text); return 1;
}

static unsigned char *gs_mission_archive_read(lua_State *L, const char *relative, char final[MAX_PATH],
                                             int *bytes, uint64_t *stamp)
{
    char root[MAX_PATH], path[MAX_PATH];
    const char *error = NULL;
    BY_HANDLE_FILE_INFORMATION info;
    HANDLE h;
    DWORD got, extra;
    unsigned char *data, tail;
    if (!gs_mission_script_root(root) || snprintf(path, sizeof path, "%s\\%s", root, relative) >= sizeof path) {
        luaL_error(L, "mission archive has no mod folder or path is too long"); return NULL;
    }
    h = gs_mission_open_native(root, path, GENERIC_READ, final, &error);
    if (h == INVALID_HANDLE_VALUE) { luaL_error(L, "%s", error); return NULL; }
    if (!GetFileInformationByHandle(h, &info) || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) ||
        info.nFileSizeHigh || info.nFileSizeLow < 32 || info.nFileSizeLow > 8u * 1024 * 1024) {
        CloseHandle(h); luaL_error(L, "mission DAT must be 32 bytes..8 MiB"); return NULL;
    }
    data = malloc(info.nFileSizeLow);
    if (!data) { CloseHandle(h); luaL_error(L, "out of memory reading mission DAT"); return NULL; }
    if (!ReadFile(h, data, info.nFileSizeLow, &got, NULL) || got != info.nFileSizeLow ||
        !ReadFile(h, &tail, 1, &extra, NULL) || extra) {
        free(data); CloseHandle(h); luaL_error(L, "mission DAT changed or could not be read completely"); return NULL;
    }
    CloseHandle(h);
    *bytes = (int)got;
    *stamp = gs_fnv(14695981039346656037ull, &info.nFileSizeLow, sizeof(DWORD));
    *stamp = gs_fnv(*stamp, &info.ftLastWriteTime.dwHighDateTime, sizeof(DWORD));
    *stamp = gs_fnv(*stamp, &info.ftLastWriteTime.dwLowDateTime, sizeof(DWORD));
    return data;
}

/* Presence participates in the stamp: deleting an optional sidecar/glow is a
 * content change too. Fields are hashed separately, never structure padding. */
static int gs_mission_file_stamp(const char *path, uint64_t *stamp)
{
    WIN32_FILE_ATTRIBUTE_DATA info;
    DWORD attr = GetFileAttributesA(path);
    uint64_t h = 14695981039346656037ull;
    if (attr == INVALID_FILE_ATTRIBUTES) {
        DWORD error = GetLastError();
        if (error != ERROR_FILE_NOT_FOUND && error != ERROR_PATH_NOT_FOUND) return 0;
        *stamp = 0; return 1;
    }
    if ((attr & FILE_ATTRIBUTE_DIRECTORY) || !GetFileAttributesExA(path, GetFileExInfoStandard, &info)) return 0;
    h = gs_fnv(h, &info.nFileSizeHigh, sizeof info.nFileSizeHigh);
    h = gs_fnv(h, &info.nFileSizeLow, sizeof info.nFileSizeLow);
    h = gs_fnv(h, &info.ftLastWriteTime.dwHighDateTime, sizeof(DWORD));
    h = gs_fnv(h, &info.ftLastWriteTime.dwLowDateTime, sizeof(DWORD));
    *stamp = h ? h : 1; return 1;
}

static int l_mod_stamp(lua_State *L)
{
    char final[MAX_PATH];
    const char *error = NULL;
    BY_HANDLE_FILE_INFORMATION info;
    uint64_t stamp;
    HANDLE h = gs_mission_open(L, FILE_READ_ATTRIBUTES, final, &error);
    if (h == INVALID_HANDLE_VALUE) { lua_pushnil(L); return 1; }
    if (!GetFileInformationByHandle(h, &info) || (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) ||
        info.nFileSizeHigh || info.nFileSizeLow > GS_MISSION_TEXT_MAX) {
        CloseHandle(h); lua_pushnil(L); return 1;
    }
    CloseHandle(h);
    /* Metadata from the validated handle, rather than reopening the path. */
    stamp = gs_fnv(14695981039346656037ull, &info.nFileSizeLow, sizeof(DWORD));
    stamp = gs_fnv(stamp, &info.ftLastWriteTime.dwHighDateTime, sizeof(DWORD));
    stamp = gs_fnv(stamp, &info.ftLastWriteTime.dwLowDateTime, sizeof(DWORD));
    lua_pushinteger(L, (lua_Integer)(stamp & 0x7FFFFFFFFFFFFFFFull)); return 1;
}

typedef struct { char name[MAX_PATH]; int dir; } GsMissionEntry;
static int gs_mission_entry_compare(const void *a, const void *b)
{
    return strcmp(((const GsMissionEntry *)a)->name, ((const GsMissionEntry *)b)->name);
}

static int l_mod_list(lua_State *L)
{
    char final[MAX_PATH];
    const char *error = NULL;
    BY_HANDLE_FILE_INFORMATION info;
    unsigned char *buffer;
    int first = 1;
    GsMissionEntry *rows = NULL;
    size_t count = 0, cap = 0, i;
    HANDLE h = gs_mission_open(L, FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES, final, &error);
    if (h == INVALID_HANDLE_VALUE) return gs_mission_error(L, error);
    if (!GetFileInformationByHandle(h, &info) || !(info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)) {
        CloseHandle(h); return gs_mission_error(L, "mission path is not a directory");
    }
    buffer = malloc(65536);
    if (!buffer) { CloseHandle(h); return gs_mission_error(L, "out of memory listing mission directory"); }
    /* Enumerate the validated directory handle, never reopen a mutable path. */
    while (GetFileInformationByHandleEx(h, first ? FileIdBothDirectoryRestartInfo : FileIdBothDirectoryInfo,
                                       buffer, 65536)) {
        FILE_ID_BOTH_DIR_INFO *data = (FILE_ID_BOTH_DIR_INFO *)buffer;
        first = 0;
        do {
            GsMissionEntry *grown;
            char name[MAX_PATH];
            int n = WideCharToMultiByte(CP_ACP, 0, data->FileName, (int)(data->FileNameLength / sizeof(WCHAR)),
                                        name, sizeof name - 1, NULL, NULL);
            if (!n) { error = "mission directory entry name too long or invalid"; break; }
            name[n] = 0;
            if (!strcmp(name, ".") || !strcmp(name, "..")) {
                if (!data->NextEntryOffset) break;
                data = (FILE_ID_BOTH_DIR_INFO *)((unsigned char *)data + data->NextEntryOffset); continue;
            }
            if (count == cap) {
                size_t next = cap ? cap * 2 : 32;
                if (next > INT_MAX / sizeof *rows || !(grown = realloc(rows, next * sizeof *rows))) {
                    error = "mission listing exceeds available memory"; break;
                }
                rows = grown; cap = next;
            }
            snprintf(rows[count].name, sizeof rows[count].name, "%s", name);
            rows[count++].dir = (data->FileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
            if (!data->NextEntryOffset) break;
            data = (FILE_ID_BOTH_DIR_INFO *)((unsigned char *)data + data->NextEntryOffset);
        } while (1);
        if (error) break;
    }
    if (!error && GetLastError() != ERROR_NO_MORE_FILES) error = "mission directory enumeration failed";
    free(buffer);
    CloseHandle(h);
    if (error) { free(rows); return gs_mission_error(L, error); }
    if (count > 1) qsort(rows, count, sizeof *rows, gs_mission_entry_compare);
    lua_createtable(L, (int)count, 0);
    for (i = 0; i < count; ++i) {
        lua_createtable(L, 0, 2);
        gs_setstr(L, "name", rows[i].name); gs_setbool(L, "dir", rows[i].dir);
        lua_rawseti(L, -2, (lua_Integer)i + 1);
    }
    free(rows); return 1;
}
#else
/* The approved containment protocol relies on Win32 final handle paths.
 * Preserve existing Linux scripts/model loading; new mission IO fails closed. */
#include <sys/stat.h>
static int gs_mission_script_root(char out[MAX_PATH]) { (void)out; return 0; }
static int gs_mission_unavailable(lua_State *L) {
    lua_pushnil(L); lua_pushliteral(L, "mission file access requires Windows"); return 2;
}
static int l_mod_read(lua_State *L) { return gs_mission_unavailable(L); }
static int l_mod_list(lua_State *L) { return gs_mission_unavailable(L); }
static int l_mod_stamp(lua_State *L) { lua_pushnil(L); return 1; }
static unsigned char *gs_mission_archive_read(lua_State *L, const char *relative, char final[MAX_PATH],
                                             int *bytes, uint64_t *stamp) {
    (void)relative; (void)final; (void)bytes; (void)stamp;
    luaL_error(L, "mission archive access requires Windows"); return NULL;
}
static int gs_mission_file_stamp(const char *path, uint64_t *stamp) {
    struct stat info;
    uint64_t h;
    char normalized[MAX_PATH], *p;
    snprintf(normalized, sizeof normalized, "%s", path);
    for (p = normalized; *p; ++p) if (*p == '\\') *p = '/';
    if (stat(normalized, &info)) {
        if (errno != ENOENT && errno != ENOTDIR) return 0;
        *stamp = 0; return 1;
    }
    if (!S_ISREG(info.st_mode)) return 0;
    h = gs_fnv(14695981039346656037ull, &info.st_size, sizeof info.st_size);
    h = gs_fnv(h, &info.st_mtim.tv_sec, sizeof info.st_mtim.tv_sec);
    h = gs_fnv(h, &info.st_mtim.tv_nsec, sizeof info.st_mtim.tv_nsec);
    *stamp = h ? h : 1; return 1;
}
#endif
