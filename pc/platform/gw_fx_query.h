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
int gw_Fx_LabAttach(const char *package, int jobj, int script, int port, int joint, int frame, int facing,
                    float x, float y, float z, float scale);
void gw_Fx_LabStop(int script);
typedef struct GwFxLabQuery {
    int age, emitters, particles, emitting, refused, refused_emitters;
} GwFxLabQuery;
int gw_Fx_LabPlay(const char *package, int jobj, int script, int port, int joint,
                 int frame, int facing, float x, float y, float z, float scale, unsigned seed);
int gw_Fx_LabControl(int handle, int script, int emitter, const float values[7], int frames);
int gw_Fx_LabEnd(int handle, int script, int fade_frames);
int gw_Fx_LabQuery(int handle, int script, GwFxLabQuery *out);
#endif
