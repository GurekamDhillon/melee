/* Effect IR custom body adapter; shader ids live in immutable packages, not fx_state. */
#include "gw_shader.h"
static int fx_custom_shader(fjdoc *d, int emitter, const char *path)
{
    int shader = fj_path(d, emitter, "material", "shader");
    int params = fj_get(d, shader, "params"), node, first = 1, used = 1;
    char json[8192] = "{";
    const char *fragment = fj_s(d, fj_get(d, shader, "fragment"));
    const char *vertex = fj_s(d, fj_get(d, shader, "vertex"));
    if (strcmp(fj_s(d, fj_get(d, shader, "type")), "custom") && !fragment[0]) return 0;
    if (!fragment[0]) { gw_log("shader: %s: custom effect needs fragment", path); return -1; }
    if (params >= 0 && d->n[params].type != FJ_OBJ) return -1;
    for (node = params >= 0 ? d->n[params].first : -1; node >= 0; node = d->n[node].next) {
        const char *name = d->n[node].key;
        int n, j;
        if (!name || strchr(name, '"') || strchr(name, '\\') || used > 7000) return -1;
        n = snprintf(json + used, sizeof json - (size_t)used, "%s\"%s\":", first ? "" : ",", name);
        if (n < 0 || n >= (int)sizeof json - used) return -1;
        used += n; first = 0;
        if (d->n[node].type == FJ_NUM) {
            n = snprintf(json + used, sizeof json - (size_t)used, "%.9g", d->n[node].num);
        } else if (d->n[node].type == FJ_ARR) {
            float v[4];
            for (j = 0; j < 4; ++j) {
                int at = fj_at(d, node, j);
                if (at < 0 || d->n[at].type != FJ_NUM) return -1;
                v[j] = (float)d->n[at].num;
            }
            if (fj_at(d, node, 4) >= 0) return -1;
            n = snprintf(json + used, sizeof json - (size_t)used, "[%.9g,%.9g,%.9g,%.9g]", v[0], v[1], v[2], v[3]);
        } else return -1;
        if (n < 0 || n >= (int)sizeof json - used) return -1;
        used += n;
    }
    if (used + 2 >= (int)sizeof json) return -1;
    json[used++] = '}'; json[used] = 0;
    return gw_Shader_FxLoad(path, fragment, vertex, json);
}
