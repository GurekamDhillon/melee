/* Wide roster foundation, deliberately NOT connected to the live game yet.
 * Native identities never go into a guest byte field. A future loader binds
 * at most six unique identities to resident fighter slots before match boot.
 * Disc m-ex callbacks continue to see their original signed-byte source id.
 * This header has no allocation, host pointers or mutable global state.
 */
#ifndef GW_ROSTER_REGISTRY_H
#define GW_ROSTER_REGISTRY_H

#include <stdint.h>

#define GW_ROSTER_CAPACITY 256
#define GW_ROSTER_MATCH_CAPACITY 6
#define GW_ROSTER_NONE UINT16_C(0xFFFF)

typedef struct GwRosterRow {
    uint64_t identity; /* Hash of stable mod id + row inputs, not just Pl bytes. */
    uint16_t source_internal; /* Original m-ex id; NEVER a wide guest id. */
} GwRosterRow;

typedef struct GwRosterRegistry {
    GwRosterRow rows[GW_ROSTER_CAPACITY];
    uint16_t count;
} GwRosterRegistry;

typedef struct GwRosterMatch {
    uint16_t ids[GW_ROSTER_MATCH_CAPACITY];
    uint16_t count;
} GwRosterMatch;

static inline uint16_t gw_roster_add(GwRosterRegistry* r, uint64_t identity,
                                     uint16_t source_internal)
{
    uint16_t id;
    if (!r || r->count >= GW_ROSTER_CAPACITY || source_internal > 127 || !identity) {
        return GW_ROSTER_NONE;
    }
    id = r->count++;
    r->rows[id].identity = identity;
    r->rows[id].source_internal = source_internal;
    return id;
}

static inline void gw_roster_id_write(unsigned char out[2], uint16_t id)
{
    out[0] = (unsigned char) (id >> 8);
    out[1] = (unsigned char) id;
}

static inline uint16_t gw_roster_id_read(const unsigned char in[2])
{
    return (uint16_t) (((uint16_t) in[0] << 8) | in[1]);
}

static inline int gw_roster_match_slot(const GwRosterMatch* m, uint16_t id)
{
    int i;
    if (!m || id == GW_ROSTER_NONE || m->count > GW_ROSTER_MATCH_CAPACITY) {
        return -1;
    }
    for (i = 0; i < m->count; ++i) {
        if (m->ids[i] == id) return i;
    }
    return -1;
}

static inline uint16_t gw_roster_match_id(const GwRosterMatch* m, int slot)
{
    return m && m->count <= GW_ROSTER_MATCH_CAPACITY && slot >= 0 && slot < m->count
               ? m->ids[slot] : GW_ROSTER_NONE;
}

static inline int gw_roster_bind(const GwRosterRegistry* r, GwRosterMatch* m,
                                 const uint16_t* ids, int count)
{
    GwRosterMatch next = {{GW_ROSTER_NONE, GW_ROSTER_NONE, GW_ROSTER_NONE,
                            GW_ROSTER_NONE, GW_ROSTER_NONE, GW_ROSTER_NONE}, 0};
    int i;
    if (!r || !m || !ids || r->count > GW_ROSTER_CAPACITY || count < 0 ||
        count > GW_ROSTER_MATCH_CAPACITY) return 0;
    for (i = 0; i < count; ++i) {
        if (ids[i] >= r->count) return 0;
        if (gw_roster_match_slot(&next, ids[i]) < 0) next.ids[next.count++] = ids[i];
    }
    *m = next;
    return 1;
}

static inline int gw_roster_pages(int count, int per)
{
    return count > 0 && count <= GW_ROSTER_CAPACITY && per > 0
               ? 1 + (count - 1) / per : 0;
}

static inline int gw_roster_page_count(int count, int per, int page)
{
    int start;
    if (page < 0 || page >= gw_roster_pages(count, per)) return 0;
    start = page * per;
    return count - start < per ? count - start : per;
}

#endif
