/* gw_nucleus_json.h - a small JSON reader and writer for the Nucleus mod browser. Header-only, pure C, no game types.
 *
 * Reads what the SSBM Nucleus public API sends (and the browser's own cache files, which use the same shapes): objects, arrays, strings with
 * escapes and surrogate pairs (decoded to UTF-8), numbers, true/false/null. The document owns one node array and one string pool; indices are
 * stable until nj_free. Depth is limited (64) and so is the document (the caller bounds the byte count), so a hostile body cannot run away.
 * The writer appends to a growing buffer and escapes everything that is not printable ASCII as UTF-8 pass-through or \u00XX. */
#ifndef GW_NUCLEUS_JSON_H
#define GW_NUCLEUS_JSON_H

#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { NJ_NULL, NJ_BOOL, NJ_NUM, NJ_STR, NJ_ARR, NJ_OBJ };

typedef struct { int type; int first, next; int key; int str; double num; } nj_node;   /* key/str: offsets into the pool (-1: none) */
typedef struct { nj_node *n; int nn, cap; char *pool; size_t np, pcap; const char *err; } nj_doc;

static void nj_free(nj_doc *d) { free(d->n); free(d->pool); memset(d, 0, sizeof *d); }

static int nj_new(nj_doc *d, int type) {
    if (d->nn == d->cap) {
        int nc = d->cap ? d->cap * 2 : 256;
        nj_node *p = (nj_node *) realloc(d->n, (size_t) nc * sizeof *p);
        if (!p) return -1;
        d->n = p; d->cap = nc;
    }
    d->n[d->nn].type = type; d->n[d->nn].first = d->n[d->nn].next = -1; d->n[d->nn].key = d->n[d->nn].str = -1; d->n[d->nn].num = 0;
    return d->nn++;
}

static int nj_pool_add(nj_doc *d, const char *s, size_t len) {
    size_t at = d->np;
    if (d->np + len + 1 > d->pcap) {
        size_t nc = d->pcap ? d->pcap * 2 : 4096;
        char *p;
        while (nc < d->np + len + 1) nc *= 2;
        p = (char *) realloc(d->pool, nc);
        if (!p) return -1;
        d->pool = p; d->pcap = nc;
    }
    memcpy(d->pool + at, s, len);
    d->pool[at + len] = '\0';
    d->np += len + 1;
    return (int) at;
}

static const char *nj_ws(const char *p) { while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') ++p; return p; }

static int nj_hex4(const char *p) {
    int i, v = 0;
    for (i = 0; i < 4; ++i) {
        char c = p[i];
        v <<= 4;
        if (c >= '0' && c <= '9') v |= c - '0';
        else if (c >= 'a' && c <= 'f') v |= c - 'a' + 10;
        else if (c >= 'A' && c <= 'F') v |= c - 'A' + 10;
        else return -1;
    }
    return v;
}

static int nj_utf8(char *o, unsigned cp) {
    if (cp < 0x80) { o[0] = (char) cp; return 1; }
    if (cp < 0x800) { o[0] = (char) (0xC0 | (cp >> 6)); o[1] = (char) (0x80 | (cp & 63)); return 2; }
    if (cp < 0x10000) { o[0] = (char) (0xE0 | (cp >> 12)); o[1] = (char) (0x80 | ((cp >> 6) & 63)); o[2] = (char) (0x80 | (cp & 63)); return 3; }
    o[0] = (char) (0xF0 | (cp >> 18)); o[1] = (char) (0x80 | ((cp >> 12) & 63)); o[2] = (char) (0x80 | ((cp >> 6) & 63)); o[3] = (char) (0x80 | (cp & 63));
    return 4;
}

/* a string at *pp (on the opening quote): decoded into the pool; returns the pool offset or -1 */
static int nj_string(nj_doc *d, const char **pp) {
    const char *p = *pp + 1;
    size_t cap = 64, len = 0;
    char *buf = (char *) malloc(cap);
    int at;
    if (!buf) { d->err = "out of memory"; return -1; }
    for (;;) {
        unsigned char c = (unsigned char) *p;
        char tmp[4];
        int k, n = 0;
        if (c == '\0') { d->err = "unterminated string"; free(buf); return -1; }
        if (c == '"') { ++p; break; }
        if (c < 0x20) { d->err = "control character in string"; free(buf); return -1; }
        if (c == '\\') {
            char e = p[1];
            ++p;
            if (e == 'n') { tmp[0] = '\n'; n = 1; ++p; }
            else if (e == 't') { tmp[0] = '\t'; n = 1; ++p; }
            else if (e == 'r') { tmp[0] = '\r'; n = 1; ++p; }
            else if (e == 'b') { tmp[0] = '\b'; n = 1; ++p; }
            else if (e == 'f') { tmp[0] = '\f'; n = 1; ++p; }
            else if (e == '"' || e == '\\' || e == '/') { tmp[0] = e; n = 1; ++p; }
            else if (e == 'u') {
                int cp = nj_hex4(p + 1);
                if (cp < 0) { d->err = "bad \\u escape"; free(buf); return -1; }
                p += 5;
                if (cp >= 0xD800 && cp < 0xDC00 && p[0] == '\\' && p[1] == 'u') {
                    int lo = nj_hex4(p + 2);
                    if (lo >= 0xDC00 && lo < 0xE000) { cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00); p += 6; }
                }
                n = nj_utf8(tmp, (unsigned) cp);
            } else { d->err = "bad escape"; free(buf); return -1; }
        } else { tmp[0] = (char) c; n = 1; ++p; }
        if (len + (size_t) n + 1 > cap) {
            char *nb;
            cap *= 2;
            nb = (char *) realloc(buf, cap);
            if (!nb) { free(buf); d->err = "out of memory"; return -1; }
            buf = nb;
        }
        for (k = 0; k < n; ++k) buf[len++] = tmp[k];
    }
    buf[len] = '\0';
    at = nj_pool_add(d, buf, len);
    free(buf);
    *pp = p;
    if (at < 0) d->err = "out of memory";
    return at;
}

static int nj_value(nj_doc *d, const char **pp, int depth);

static int nj_container(nj_doc *d, const char **pp, int depth, int obj) {
    const char *p = nj_ws(*pp + 1);
    char close = obj ? '}' : ']';
    int self = nj_new(d, obj ? NJ_OBJ : NJ_ARR), last = -1;
    if (self < 0) { d->err = "out of memory"; return -1; }
    if (*p == close) { *pp = p + 1; return self; }
    for (;;) {
        int key = -1, v;
        if (obj) {
            if (*p != '"') { d->err = "object key expected"; return -1; }
            key = nj_string(d, &p);
            if (key < 0) return -1;
            p = nj_ws(p);
            if (*p != ':') { d->err = "':' expected"; return -1; }
            p = nj_ws(p + 1);
        }
        v = nj_value(d, &p, depth + 1);
        if (v < 0) return -1;
        d->n[v].key = key;
        if (last < 0) d->n[self].first = v; else d->n[last].next = v;
        last = v;
        p = nj_ws(p);
        if (*p == ',') { p = nj_ws(p + 1); continue; }
        if (*p == close) { *pp = p + 1; return self; }
        d->err = "',' or closing bracket expected";
        return -1;
    }
}

static int nj_value(nj_doc *d, const char **pp, int depth) {
    const char *p = nj_ws(*pp);
    int v;
    if (depth > 64) { d->err = "nested too deep"; return -1; }
    if (*p == '{') { v = nj_container(d, &p, depth, 1); }
    else if (*p == '[') { v = nj_container(d, &p, depth, 0); }
    else if (*p == '"') {
        int s = nj_string(d, &p);
        if (s < 0) return -1;
        v = nj_new(d, NJ_STR);
        if (v < 0) { d->err = "out of memory"; return -1; }
        d->n[v].str = s;
    } else if (!strncmp(p, "true", 4)) { v = nj_new(d, NJ_BOOL); if (v >= 0) d->n[v].num = 1; p += 4; }
    else if (!strncmp(p, "false", 5)) { v = nj_new(d, NJ_BOOL); p += 5; }
    else if (!strncmp(p, "null", 4)) { v = nj_new(d, NJ_NULL); p += 4; }
    else if (*p == '-' || (*p >= '0' && *p <= '9')) {
        char *end;
        double x = strtod(p, &end);
        if (end == p) { d->err = "bad number"; return -1; }
        v = nj_new(d, NJ_NUM);
        if (v >= 0) d->n[v].num = x;
        p = end;
    } else { d->err = "value expected"; return -1; }
    if (v < 0) { if (!d->err) d->err = "out of memory"; return -1; }
    *pp = p;
    return v;
}

/* Parse text (NUL-terminated); returns the root node or -1 (d->err says why). The doc is reset first. */
static int nj_parse(nj_doc *d, const char *text) {
    const char *p = text;
    int root;
    memset(d, 0, sizeof *d);
    if ((unsigned char) p[0] == 0xEF && (unsigned char) p[1] == 0xBB && (unsigned char) p[2] == 0xBF) p += 3;
    root = nj_value(d, &p, 0);
    if (root < 0) return -1;
    p = nj_ws(p);
    if (*p != '\0') { d->err = "trailing text"; return -1; }
    return root;
}

static const char *nj_str(const nj_doc *d, int n, const char *def) {
    return (n >= 0 && d->n[n].type == NJ_STR) ? d->pool + d->n[n].str : def;
}
static double nj_num(const nj_doc *d, int n, double def) { return (n >= 0 && d->n[n].type == NJ_NUM) ? d->n[n].num : def; }
static int nj_get(const nj_doc *d, int obj, const char *key) {
    int c;
    if (obj < 0 || d->n[obj].type != NJ_OBJ) return -1;
    for (c = d->n[obj].first; c >= 0; c = d->n[c].next) if (d->n[c].key >= 0 && !strcmp(d->pool + d->n[c].key, key)) return c;
    return -1;
}
static const char *nj_gstr(const nj_doc *d, int obj, const char *key, const char *def) { return nj_str(d, nj_get(d, obj, key), def); }
static double nj_gnum(const nj_doc *d, int obj, const char *key, double def) { return nj_num(d, nj_get(d, obj, key), def); }

/* ---- writer ------------------------------------------------------------------------------------------ */

typedef struct { char *s; size_t n, cap; int bad; } nj_buf;

static void nj_putn(nj_buf *b, const char *s, size_t len) {
    if (b->bad) return;
    if (b->n + len + 1 > b->cap) {
        size_t nc = b->cap ? b->cap * 2 : 1024;
        char *p;
        while (nc < b->n + len + 1) nc *= 2;
        p = (char *) realloc(b->s, nc);
        if (!p) { b->bad = 1; return; }
        b->s = p; b->cap = nc;
    }
    memcpy(b->s + b->n, s, len);
    b->n += len;
    b->s[b->n] = '\0';
}
static void nj_puts(nj_buf *b, const char *s) { nj_putn(b, s, strlen(s)); }
static void nj_printf(nj_buf *b, const char *fmt, ...) {
    char tmp[512];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(tmp, sizeof tmp, fmt, ap);
    va_end(ap);
    nj_puts(b, tmp);
}
/* "text" with the JSON escapes; bytes >= 0x80 pass through (the text is UTF-8 or ASCII) */
static void nj_qstr(nj_buf *b, const char *s) {
    nj_putn(b, "\"", 1);
    for (; *s; ++s) {
        unsigned char c = (unsigned char) *s;
        if (c == '"') nj_puts(b, "\\\"");
        else if (c == '\\') nj_puts(b, "\\\\");
        else if (c == '\n') nj_puts(b, "\\n");
        else if (c == '\r') nj_puts(b, "\\r");
        else if (c == '\t') nj_puts(b, "\\t");
        else if (c < 0x20) nj_printf(b, "\\u%04x", c);
        else nj_putn(b, (const char *) &c, 1);
    }
    nj_putn(b, "\"", 1);
}

#endif
