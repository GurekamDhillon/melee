/* Read-only effect observation for the Lua console. Native data only. */
#ifndef GW_FX_QUERY_H
#define GW_FX_QUERY_H
typedef struct GwFxQuery {
    const char *package;
    int owner_kind; /* 0 unknown, 1 fighter, 2 article */
    int port;       /* 1-6, or 0 when unknown */
    int joint;      /* fighter binding joint; -1 for an article */
    float x, y, z;
    int facing, particles, emitters, has_bbox;
    float min_x, min_y, min_z, max_x, max_y, max_z;
} GwFxQuery;
int gw_Fx_Query(int index, GwFxQuery *out);
#endif
