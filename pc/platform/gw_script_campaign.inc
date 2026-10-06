/* campaign-save: host files only, never MEM1, snapshots or rewind history.
 * Included after gs_data_path/l_data_read. Temp and destination share a directory;
 * a flushed complete record replaces one slot while Lua retains the other slot. */
static int l_campaign_storage(lua_State *L) {
    char path[MAX_PATH], temp[MAX_PATH];
    size_t name_len, len;
    const char *name;
    const char *bytes;
    HANDLE file;
    DWORD written = 0, error = 0;
    gs_require_offline(L, "campaign_storage");
    if (lua_gettop(L) == 0) { lua_pushboolean(L, 1); return 1; }
    name = luaL_checklstring(L, 1, &name_len);
    if (strlen(name) != name_len)
        return luaL_error(L, "campaign file name contains NUL");
    /* gs_data_path's fixed buffer must not silently turn a long path into a
     * different destination. Leave room for the private staging suffix. */
    if (strlen(gs.data_dir) + strlen(gs_cur_script()->id) + name_len + 8 >= MAX_PATH)
        return luaL_error(L, "campaign path too long");
    if (lua_gettop(L) == 1) return l_data_read(L);
    bytes = luaL_checklstring(L, 2, &len);
    if (len > (1 << 20)) return luaL_error(L, "campaign files are at most 1 MB");
    gs_data_path(L, name, path, sizeof path);
    snprintf(temp, sizeof temp, "%s.tmp", path);
#ifdef _WIN32
    file = CreateFileA(temp, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS,
                       FILE_ATTRIBUTE_NORMAL, NULL);
    if (file == INVALID_HANDLE_VALUE) error = GetLastError();
    else {
        if (!WriteFile(file, bytes, (DWORD) len, &written, NULL)) error = GetLastError();
        else if (written != (DWORD) len) error = ERROR_WRITE_FAULT;
        if (!error && !FlushFileBuffers(file)) error = GetLastError();
        if (!CloseHandle(file) && !error) error = GetLastError();
        if (!error && !MoveFileExA(temp, path, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH))
            error = GetLastError();
    }
#else
    { /* the same temp-write, flush, atomic-replace sequence with POSIX calls (rename() replaces) */
        FILE *out = fopen(temp, "wb");
        (void) file;
        if (!out) error = (DWORD) (errno ? errno : EIO);
        else {
            if (fwrite(bytes, 1, len, out) != len) error = (DWORD) (errno ? errno : EIO);
            if (!error && (fflush(out) || fsync(fileno(out)))) error = (DWORD) (errno ? errno : EIO);
            if (fclose(out) && !error) error = (DWORD) (errno ? errno : EIO);
            if (!error && !MoveFileExA(temp, path, 0)) error = (DWORD) (errno ? errno : EIO);
        }
    }
#endif
    if (error) {
        DeleteFileA(temp);
        return luaL_error(L, "campaign atomic write failed (%d)", (int) error);
    }
    gw_log("[campaign-save] committed %s (%u bytes)\n", name, (unsigned) len);
    lua_pushboolean(L, 1);
    return 1;
}
