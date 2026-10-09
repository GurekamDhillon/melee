/* gw_nucleus_http.inc - one blocking HTTPS GET for the Nucleus browser, used only from its worker threads (never the game thread, never a netplay
 * session: the engine refuses to start one online).
 *
 * Windows: WinHTTP, loaded by name at first use (no import library to add to the link; the system proxy and certificate store apply). Redirects are
 * followed and the FINAL address is checked against the allow-list. Other platforms: the system `curl` program (the game links no TLS library); when it
 * is missing the call fails with a message and the browser says it is offline.
 *
 * Only ssbmnucleus.net and its subdomains are ever contacted, over https. MELEE_NUCLEUS_API may name another API base for testing against a local
 * fixture server; its host (127.0.0.1 or localhost only) and plain http are then allowed too.
 *
 * Every request carries the User-Agent the API's owner asked for: GDMelee/<version> (+https://github.com/GurekamDhillon/melee). */
#ifndef GW_NUCLEUS_HTTP_INC
#define GW_NUCLEUS_HTTP_INC

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#include <winhttp.h>
#endif

#define NCH_APP_VERSION "0.2"

typedef struct { int status; uint8_t *data; size_t len; int retry_after; char err[120]; } nch_result;
/* progress(user, bytes so far, bytes expected or 0 when the server did not say): called after every read; a nonzero return abandons the download */
typedef int (*nch_progress_fn)(void *user, size_t got, size_t total);

static char nch_override_host[64];     /* set by nch_set_override: a loopback host that is allowed besides ssbmnucleus.net */

static const char *nch_user_agent(void) {
    static char ua[160];
    const char *v;
    if (ua[0]) return ua;
    v = getenv("MELEE_VERSION");
    snprintf(ua, sizeof ua, "GDMelee/%s (+https://github.com/GurekamDhillon/melee)", v && v[0] && strlen(v) < 24 ? v : NCH_APP_VERSION);
    return ua;
}

/* "https://host/path" -> host (no port), scheme-secure flag; returns 0 when not an absolute http(s) URL */
static int nch_split(const char *url, char *host, size_t hcap, int *secure) {
    const char *p;
    size_t n = 0;
    if (!strncmp(url, "https://", 8)) { *secure = 1; p = url + 8; }
    else if (!strncmp(url, "http://", 7)) { *secure = 0; p = url + 7; }
    else return 0;
    while (p[n] && p[n] != '/' && p[n] != ':' && p[n] != '?' && p[n] != '#') ++n;
    if (n == 0 || n >= hcap) return 0;
    memcpy(host, p, n);
    host[n] = '\0';
    return 1;
}

static int nch_host_ok(const char *host, int secure) {
    size_t n = strlen(host), s = strlen("ssbmnucleus.net");
    if (secure && (!strcmp(host, "ssbmnucleus.net") || (n > s && !strcmp(host + n - s, "ssbmnucleus.net") && host[n - s - 1] == '.'))) return 1;
    return nch_override_host[0] && !strcmp(host, nch_override_host) && (!strcmp(host, "127.0.0.1") || !strcmp(host, "localhost"));
}

static void nch_set_override(const char *base) {
    int secure;
    nch_override_host[0] = '\0';
    if (base && base[0] && nch_split(base, nch_override_host, sizeof nch_override_host, &secure)
        && strcmp(nch_override_host, "127.0.0.1") != 0 && strcmp(nch_override_host, "localhost") != 0) nch_override_host[0] = '\0';
}

#ifdef _WIN32

typedef HINTERNET (WINAPI *nch_Open)(LPCWSTR, DWORD, LPCWSTR, LPCWSTR, DWORD);
typedef HINTERNET (WINAPI *nch_Connect)(HINTERNET, LPCWSTR, INTERNET_PORT, DWORD);
typedef HINTERNET (WINAPI *nch_OpenRequest)(HINTERNET, LPCWSTR, LPCWSTR, LPCWSTR, LPCWSTR, LPCWSTR *, DWORD);
typedef BOOL (WINAPI *nch_Send)(HINTERNET, LPCWSTR, DWORD, LPVOID, DWORD, DWORD, DWORD_PTR);
typedef BOOL (WINAPI *nch_Recv)(HINTERNET, LPVOID);
typedef BOOL (WINAPI *nch_Query)(HINTERNET, DWORD, LPCWSTR, LPVOID, LPDWORD, LPDWORD);
typedef BOOL (WINAPI *nch_Read)(HINTERNET, LPVOID, DWORD, LPDWORD);
typedef BOOL (WINAPI *nch_Close)(HINTERNET);
typedef BOOL (WINAPI *nch_Timeouts)(HINTERNET, int, int, int, int);
typedef BOOL (WINAPI *nch_Crack)(LPCWSTR, DWORD, DWORD, LPURL_COMPONENTS);
typedef BOOL (WINAPI *nch_QueryOpt)(HINTERNET, DWORD, LPVOID, LPDWORD);
typedef BOOL (WINAPI *nch_SetOpt)(HINTERNET, DWORD, LPVOID, DWORD);

static struct {
    HMODULE dll; int tried;
    nch_Open open; nch_Connect connect; nch_OpenRequest request; nch_Send send; nch_Recv recv; nch_Query query; nch_Read read; nch_Close close;
    nch_Timeouts timeouts; nch_Crack crack; nch_QueryOpt qopt; nch_SetOpt sopt;
} nch_w;

static int nch_load(void) {
    if (nch_w.tried) return nch_w.dll != NULL && nch_w.read != NULL;
    nch_w.tried = 1;
    nch_w.dll = LoadLibraryA("winhttp.dll");
    if (!nch_w.dll) return 0;
#define NCH_SYM(f, name) nch_w.f = (void *) GetProcAddress(nch_w.dll, name)
    NCH_SYM(open, "WinHttpOpen"); NCH_SYM(connect, "WinHttpConnect"); NCH_SYM(request, "WinHttpOpenRequest"); NCH_SYM(send, "WinHttpSendRequest");
    NCH_SYM(recv, "WinHttpReceiveResponse"); NCH_SYM(query, "WinHttpQueryHeaders"); NCH_SYM(read, "WinHttpReadData"); NCH_SYM(close, "WinHttpCloseHandle");
    NCH_SYM(timeouts, "WinHttpSetTimeouts"); NCH_SYM(crack, "WinHttpCrackUrl"); NCH_SYM(qopt, "WinHttpQueryOption"); NCH_SYM(sopt, "WinHttpSetOption");
#undef NCH_SYM
    if (!nch_w.open || !nch_w.connect || !nch_w.request || !nch_w.send || !nch_w.recv || !nch_w.query || !nch_w.read || !nch_w.close || !nch_w.crack) {
        nch_w.read = NULL;
        return 0;
    }
    return 1;
}

/* GET url; follows redirects (final address re-checked); at most max_bytes of body; the result owns data (free it). 0 when an HTTP status came back. */
static int nch_get_p(const char *url, size_t max_bytes, nch_result *r, nch_progress_fn prog, void *pu) {
    wchar_t wurl[1200], wua[200], host[128], path[1100];
    URL_COMPONENTS uc;
    HINTERNET ses = NULL, con = NULL, req = NULL;
    char hostc[96];
    int secure = 0;
    DWORD flags, status = 0, sz, avail;
    size_t cap = 0, len = 0, expect = 0;
    uint8_t *buf = NULL;
    memset(r, 0, sizeof *r);
    if (!nch_split(url, hostc, sizeof hostc, &secure) || !nch_host_ok(hostc, secure)) { snprintf(r->err, sizeof r->err, "address not allowed"); return -1; }
    if (!nch_load()) { snprintf(r->err, sizeof r->err, "WinHTTP is not available"); return -1; }
    if (!MultiByteToWideChar(CP_UTF8, 0, url, -1, wurl, 1200) || !MultiByteToWideChar(CP_UTF8, 0, nch_user_agent(), -1, wua, 200)) { snprintf(r->err, sizeof r->err, "bad address"); return -1; }
    memset(&uc, 0, sizeof uc);
    uc.dwStructSize = sizeof uc;
    uc.lpszHostName = host; uc.dwHostNameLength = 128;
    uc.lpszUrlPath = path; uc.dwUrlPathLength = 1100;
    if (!nch_w.crack(wurl, 0, 0, &uc)) { snprintf(r->err, sizeof r->err, "bad address"); return -1; }
    ses = nch_w.open(wua, 4 /* WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY */, NULL, NULL, 0);
    if (!ses) ses = nch_w.open(wua, WINHTTP_ACCESS_TYPE_DEFAULT_PROXY, NULL, NULL, 0);
    if (!ses) { snprintf(r->err, sizeof r->err, "cannot start WinHTTP (%lu)", GetLastError()); return -1; }
    if (nch_w.timeouts) nch_w.timeouts(ses, 10000, 10000, 20000, 30000);
    if (nch_w.sopt) { DWORD dec = WINHTTP_DECOMPRESSION_FLAG_ALL; nch_w.sopt(ses, WINHTTP_OPTION_DECOMPRESSION, &dec, sizeof dec); }
    con = nch_w.connect(ses, host, uc.nPort, 0);
    flags = uc.nScheme == INTERNET_SCHEME_HTTPS ? WINHTTP_FLAG_SECURE : 0;
    if (con) req = nch_w.request(con, L"GET", path, NULL, WINHTTP_NO_REFERER, WINHTTP_DEFAULT_ACCEPT_TYPES, flags);
    if (!req) { snprintf(r->err, sizeof r->err, "cannot open the request (%lu)", GetLastError()); goto done; }
    if (!nch_w.send(req, WINHTTP_NO_ADDITIONAL_HEADERS, 0, WINHTTP_NO_REQUEST_DATA, 0, 0, 0) || !nch_w.recv(req, NULL)) {
        snprintf(r->err, sizeof r->err, "no answer (%lu)", GetLastError());
        goto done;
    }
    /* the final address after redirects must still be one we allow */
    if (nch_w.qopt) {
        wchar_t wf[1200];
        char fin[1200], fh[96];
        DWORD fl = sizeof wf;
        int fs = 0;
        if (nch_w.qopt(req, WINHTTP_OPTION_URL, wf, &fl) && WideCharToMultiByte(CP_UTF8, 0, wf, -1, fin, sizeof fin, NULL, NULL)
            && (!nch_split(fin, fh, sizeof fh, &fs) || !nch_host_ok(fh, fs))) {
            snprintf(r->err, sizeof r->err, "redirected to an address that is not allowed");
            goto done;
        }
    }
    sz = sizeof status;
    if (!nch_w.query(req, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_HEADER_NAME_BY_INDEX, &status, &sz, WINHTTP_NO_HEADER_INDEX)) {
        snprintf(r->err, sizeof r->err, "no status");
        goto done;
    }
    r->status = (int) status;
    {
        DWORD ra = 0;
        sz = sizeof ra;
        if (nch_w.query(req, 29 /* WINHTTP_QUERY_RETRY_AFTER */ | WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_HEADER_NAME_BY_INDEX, &ra, &sz, WINHTTP_NO_HEADER_INDEX)) r->retry_after = (int) ra;
    }
    {
        DWORD cl = 0;
        sz = sizeof cl;
        if (nch_w.query(req, 5 /* WINHTTP_QUERY_CONTENT_LENGTH */ | WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_HEADER_NAME_BY_INDEX, &cl, &sz, WINHTTP_NO_HEADER_INDEX)) expect = cl;
    }
    for (;;) {
        DWORD got = 0;
        avail = 65536;
        if (len + avail + 1 > cap) {
            size_t nc = cap ? cap * 2 : 65536;
            uint8_t *nb;
            while (nc < len + avail + 1) nc *= 2;
            nb = (uint8_t *) realloc(buf, nc);
            if (!nb) { snprintf(r->err, sizeof r->err, "out of memory"); free(buf); buf = NULL; goto done; }
            buf = nb; cap = nc;
        }
        if (!nch_w.read(req, buf + len, avail, &got)) { snprintf(r->err, sizeof r->err, "connection lost (%lu)", GetLastError()); free(buf); buf = NULL; r->status = 0; goto done; }
        if (got == 0) break;
        len += got;
        if (prog && prog(pu, len, expect)) { snprintf(r->err, sizeof r->err, "stopped"); free(buf); buf = NULL; r->status = 0; goto done; }
        if (len > max_bytes) { snprintf(r->err, sizeof r->err, "answer is larger than allowed"); free(buf); buf = NULL; r->status = 0; goto done; }
    }
    if (buf) buf[len] = '\0';
    r->data = buf; r->len = len;
done:
    if (req) nch_w.close(req);
    if (con) nch_w.close(con);
    if (ses) nch_w.close(ses);
    return r->status ? 0 : -1;
}

#else  /* the system curl */

static int nch_get_p(const char *url, size_t max_bytes, nch_result *r, nch_progress_fn prog, void *pu) {
    (void) prog; (void) pu;      /* curl runs to the end: no progress to report */
    char host[96], cmd[1500], body[400], hdr[400];
    int secure = 0, code = 0;
    FILE *p;
    memset(r, 0, sizeof r[0]);
    if (!nch_split(url, host, sizeof host, &secure) || !nch_host_ok(host, secure) || strchr(url, '\'') || strchr(url, '\n')) { snprintf(r->err, sizeof r->err, "address not allowed"); return -1; }
    snprintf(body, sizeof body, "/tmp/gw-nucleus-%d.body", (int) getpid());
    snprintf(hdr, sizeof hdr, "/tmp/gw-nucleus-%d.hdr", (int) getpid());
    snprintf(cmd, sizeof cmd, "curl -sS -L --max-time 120 --max-filesize %lu --proto =https,http -A '%s' -D '%s' -o '%s' --proto-redir =https -w '%%{http_code} %%{url_effective}' '%s' 2>/dev/null",
             (unsigned long) max_bytes, nch_user_agent(), hdr, body, url);
    p = popen(cmd, "r");
    if (!p) { snprintf(r->err, sizeof r->err, "curl is not available"); return -1; }
    {
        char fin[600], fh[96];
        int fs = 0;
        fin[0] = ' ';
        if (fscanf(p, "%d %599s", &code, fin) < 1) code = 0;
        if (code > 0 && fin[0] && (!nch_split(fin, fh, sizeof fh, &fs) || !nch_host_ok(fh, fs))) code = -1;   /* redirected somewhere we do not allow */
    }
    pclose(p);
    if (code < 0) { snprintf(r->err, sizeof r->err, "redirected to an address that is not allowed"); remove(body); remove(hdr); return -1; }
    if (code <= 0) { snprintf(r->err, sizeof r->err, "no answer (is curl installed?)"); remove(body); remove(hdr); return -1; }
    r->status = code;
    {
        FILE *h = fopen(hdr, "rb");
        char line[300];
        while (h && fgets(line, sizeof line, h)) if (!strncasecmp(line, "retry-after:", 12)) r->retry_after = atoi(line + 12);
        if (h) fclose(h);
        FILE *b = fopen(body, "rb");
        if (b) {
            long n;
            fseek(b, 0, SEEK_END); n = ftell(b); fseek(b, 0, SEEK_SET);
            if (n >= 0 && (size_t) n <= max_bytes && (r->data = (uint8_t *) malloc((size_t) n + 1)) != NULL) {
                r->len = fread(r->data, 1, (size_t) n, b);
                r->data[r->len] = '\0';
            }
            fclose(b);
        }
    }
    remove(body); remove(hdr);
    return 0;
}

#endif

static int nch_get(const char *url, size_t max_bytes, nch_result *r) { return nch_get_p(url, max_bytes, r, NULL, NULL); }

#endif
