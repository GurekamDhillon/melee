#ifndef GW_SURFACE_PATH_H
#define GW_SURFACE_PATH_H
#include <stddef.h>
#include <string.h>
static int gw_surface_path_valid(const char *path, size_t len)
{
    size_t i, start = 0;
    if (len < 14 || len >= 260 || strlen(path) != len ||
        strncmp(path, "shaders/", 8) || strcmp(path + len - 5, ".wgsl") || strstr(path, "..")) return 0;
    for (i = 0; i < len; ++i) {
        unsigned char c = (unsigned char)path[i];
        if (c < 32 || c == 127 || strchr("\\:*?\"<>|", c)) return 0;
        if (c == '/') {
            if (i == start || path[i - 1] == '.' || path[i - 1] == ' ') return 0;
            start = i + 1;
        }
    }
    return 1;
}
#endif
