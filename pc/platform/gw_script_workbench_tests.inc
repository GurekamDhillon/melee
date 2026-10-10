/* Aurora writes asynchronously. Catch publishing a header-only file, or a
 * complete-looking file whose writer is still open. No renderer/game needed. */
static int test_workbench_png_completion(void) {
#ifdef _WIN32
    static const unsigned char png[] = {
        137,80,78,71,13,10,26,10, 0,0,0,13,73,72,68,82,
        0,0,0,1,0,0,0,1,8,6,0,0,0,31,21,196,137,
        0,0,0,11,73,68,65,84,120,156,99,96,0,2,0,0,5,0,1,122,94,171,63,
        0,0,0,0,73,69,78,68,174,66,96,130
    };
    char dir[MAX_PATH], path[MAX_PATH];
    FILE *file;
    int w = 0, h = 0, rc = 0;
    DWORD len = GetTempPathA(sizeof dir, dir);
    if (!len || len >= sizeof dir || !GetTempFileNameA(dir, "wbr", 0, path)) {
        gw_test_fail("workbench PNG test: cannot create temporary path");
        return 1;
    }
    file = fopen(path, "wb");
    if (!file) { DeleteFileA(path); return 1; }
    if (fwrite(png, 1, 24, file) != 24) rc = 1;
    if (fclose(file) != 0) rc = 1;
    if (rc) { DeleteFileA(path); return rc; }
    if (gs_wb_png_size(path, &w, &h)) {
        gw_test_fail("workbench published a partial PNG header"); rc = 1;
    }
    file = fopen(path, "wb");
    if (!file) { DeleteFileA(path); return 1; }
    if (fwrite(png, 1, sizeof png, file) != sizeof png || fflush(file) != 0) rc = 1;
    if (gs_wb_png_size(path, &w, &h)) {
        gw_test_fail("workbench published while the PNG writer still owned the file"); rc = 1;
    }
    if (fclose(file) != 0) rc = 1;
    if (!gs_wb_png_size(path, &w, &h) || w != 1 || h != 1) {
        gw_test_fail("workbench did not read completed PNG dimensions"); rc = 1;
    }
    DeleteFileA(path);
    return rc;
#else
    return 0;
#endif
}
