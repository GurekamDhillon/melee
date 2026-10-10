#ifndef GENO_ANIM_BANK_CORE_H
#define GENO_ANIM_BANK_CORE_H
/* Read-only validation of the concatenated HSD subarchives ftData_80085E50
 * copies using the LOADED plan. Never relocate untrusted bytes in-game. */
#include "../geno/geno_plan.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static uint32_t gn_anim_u32(const unsigned char* p)
{
    return (uint32_t)p[0] << 24 | (uint32_t)p[1] << 16 |
           (uint32_t)p[2] << 8 | p[3];
}
static int gn_anim_reload_policy(int netplay, int rollback, int resimulating)
{
    return !netplay && !rollback && !resimulating;
}
static int gn_anim_overlap(size_t a, size_t an, size_t b, size_t bn)
{
    return an && bn && a < b + bn && b < a + an;
}
static int gn_anim_has_reloc(const unsigned char* bits, size_t at)
{
    return (bits[at / 32] & (1u << ((at / 4) & 7))) != 0;
}
/* fobj.c's parseWait/parsePackInfo do not check stream bounds themselves. */
static int gn_anim_varint(const unsigned char* p, size_t len, size_t* pos, unsigned* value)
{
    unsigned shift = 0, v = 0, b;
    do {
        if (*pos >= len || shift > 28) return 0;
        b = p[(*pos)++];
        if (shift == 28 && (b & 127) > 7) return 0;
        v |= (b & 127) << shift;
        shift += 7;
    } while (b & 128);
    *value = v;
    return 1;
}
static int gn_anim_value(const unsigned char* p, size_t len, size_t* pos, unsigned frac)
{
    unsigned type = frac >> 5, bytes;
    if (!frac) bytes = 4;
    else if (type == 1 || type == 2) bytes = 2;
    else if (type == 3 || type == 4) bytes = 1;
    else return 0;
    if (bytes > len - *pos) return 0;
    /* Key floats are little-endian even in a big-endian HSD archive. */
    if (!frac && (p[*pos + 3] & 127) == 127 && (p[*pos + 2] & 128)) return 0;
    *pos += bytes;
    return 1;
}
static int gn_anim_stream(const unsigned char* p, size_t len, unsigned fv, unsigned fs)
{
    size_t pos = 0;
    if (!len) return 0;
    while (pos < len) {
        unsigned b = p[pos++], op = b & 15, count = ((b >> 4) & 7) + 1, more;
        if (op < 1 || op > 6) return 0;
        if (b & 128) {
            if (!gn_anim_varint(p, len, &pos, &more) || more > (UINT32_MAX - count) / 8) return 0;
            count += more * 8;
        }
        if (count > len - pos) return 0;
        while (count--) {
            if (op != 5 && !gn_anim_value(p, len, &pos, fv)) return 0;
            if ((op == 4 || op == 5) && !gn_anim_value(p, len, &pos, fs)) return 0;
            if (op != 5 && pos < len && !gn_anim_varint(p, len, &pos, &more)) return 0;
        }
    }
    return 1;
}
static int gn_anim_tree(const GenoPlan* pl, const unsigned char* d, size_t data,
                        size_t root, size_t nr, const unsigned char* relocated)
{
    size_t nodes, tracks, ntracks = 0, j, node_bytes = (size_t)pl->joints + 1;
    uint32_t frames;
    if ((root & 3) || data < 20 || root > data - 20 ||
        !gn_anim_has_reloc(relocated, root + 12) ||
        !gn_anim_has_reloc(relocated, root + 16)) return 0;
    frames = gn_anim_u32(d + root + 8);
    if ((frames & 0x80000000u) || (frames & 0x7f800000u) == 0x7f800000u) return 0;
    nodes = gn_anim_u32(d + root + 12); tracks = gn_anim_u32(d + root + 16);
    if (nodes > data || node_bytes > data - nodes || tracks > data || (tracks & 3) ||
        gn_anim_overlap(root, 20, nodes, node_bytes)) return 0;
    /* ftAnim_8006F4C8 walks signed node counts until -1, advancing one joint
     * per entry. A changed skeleton must wait for a full package restart. */
    for (j = 0; j < (size_t)pl->joints; ++j) {
        if (d[nodes + j] > 127) return 0;
        ntracks += d[nodes + j];
    }
    if (d[nodes + pl->joints] != 255 || ntracks > (data - tracks) / 12 ||
        nr != ntracks + 2 || gn_anim_overlap(root, 20, tracks, ntracks * 12) ||
        gn_anim_overlap(nodes, node_bytes, tracks, ntracks * 12)) return 0;
    for (j = 0; j < ntracks; ++j) {
        size_t at = tracks + j * 12, ad = gn_anim_u32(d + at + 8);
        size_t length = (unsigned)d[at] << 8 | d[at + 1];
        if (!gn_anim_has_reloc(relocated, at + 8) || ad > data || length > data - ad ||
            d[at + 4] < 1 || d[at + 4] > 10 ||
            gn_anim_overlap(ad, length, root, 20) ||
            gn_anim_overlap(ad, length, nodes, node_bytes) ||
            gn_anim_overlap(ad, length, tracks, ntracks * 12) ||
            !gn_anim_stream(d + ad, length, d[at + 5], d[at + 6])) return 0;
    }
    /* Exactly these pointers are relocated: no scalar, node or key byte may
     * be changed by HSD_ArchiveParse's Locate after validation. */
    return 1;
}
static int gn_anim_bank_validate(const GenoPlan* pl, const unsigned char* bank,
                                 size_t length, unsigned clip_limit,
                                 char* why, size_t cap)
{
    int i;
    size_t bank_end = 0;
    unsigned char* relocated = NULL;
    const char* reason = "unusable loaded plan";
    if (!pl || !pl->ok || pl->joints < 1 || pl->joints > GPL_MAX_JOINTS ||
        pl->nclips < 1 || pl->nclips > GPL_MAX_CLIPS || !bank) goto bad_plan;
    for (i = 0; i < pl->nclips; ++i) {
        size_t off = pl->clip_offset[i], bytes = pl->clip_bytes[i];
        size_t data, nr, np, ne, rel, pub, symbols, j, root = 0;
        const unsigned char* arc;
        int found = 0;
        reason = "clip offset/length does not fit the loaded plan or animation buffer";
        if ((off & 31) || bytes < 32 || bytes > clip_limit ||
            off > length || bytes > length - off) goto bad_clip;
        if (off + bytes > bank_end) bank_end = off + bytes;
        arc = bank + off;
        reason = "HSD header size differs from the loaded plan";
        if (gn_anim_u32(arc) != bytes) goto bad_clip;
        data = gn_anim_u32(arc + 4); nr = gn_anim_u32(arc + 8);
        np = gn_anim_u32(arc + 12); ne = gn_anim_u32(arc + 16);
        reason = "HSD tables exceed the clip or require external symbols";
        if (!data || (data & 3) || data > bytes - 32 || ne) goto bad_clip;
        rel = 32 + data;
        if (nr > (bytes - rel) / 4) goto bad_clip;
        pub = rel + nr * 4;
        if (!np || np > (bytes - pub) / 8) goto bad_clip;
        symbols = pub + np * 8;
        reason = "cannot allocate relocation validation scratch";
        relocated = (unsigned char*)calloc((data + 31) / 32, 1);
        if (!relocated) goto bad_clip;
        for (j = 0; j < nr; ++j) {
            size_t at = gn_anim_u32(arc + rel + j * 4);
            reason = "HSD relocation is outside the data block or repeated";
            if ((at & 3) || data < 4 || at > data - 4 ||
                gn_anim_u32(arc + 32 + at) >= data || gn_anim_has_reloc(relocated, at)) goto bad_clip;
            relocated[at / 32] |= (unsigned char)(1u << ((at / 4) & 7));
        }
        for (j = 0; j < np; ++j) {
            size_t at = gn_anim_u32(arc + pub + j * 8);
            size_t name = gn_anim_u32(arc + pub + j * 8 + 4);
            const char* s;
            reason = "HSD public symbol is outside the clip";
            if (at >= data || name >= bytes - symbols) goto bad_clip;
            s = (const char*)arc + symbols + name;
            if (!memchr(s, 0, bytes - symbols - name)) goto bad_clip;
            if (!strcmp(s, pl->clip_symbol[i])) { ++found; root = at; }
        }
        reason = "clip does not export exactly one loaded-plan figatree symbol";
        if (found != 1) goto bad_clip;
        reason = "figatree joints, tracks or key streams are incompatible or malformed";
        if (!gn_anim_tree(pl, arc + 32, data, root, nr, relocated)) goto bad_clip;
        free(relocated); relocated = NULL;
    }
    reason = "bank length differs from the loaded plan (restart required)";
    if (bank_end > SIZE_MAX - 31 || ((bank_end + 31) & ~(size_t)31) != length) goto bad_plan;
    if (cap) why[0] = 0;
    return 1;
bad_clip:
    free(relocated);
    snprintf(why, cap, "clip %s: %s", pl->clip_name[i], reason);
    return 0;
bad_plan:
    snprintf(why, cap, "%s", reason);
    return 0;
}
#endif
