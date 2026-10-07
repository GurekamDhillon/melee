/* Implementations for gw_compat_linux.h. See that header for what this deliberately does not
 * cover (winsock, the GC HID adapter, SEH, PAGE_GUARD watchpoints - each ported at its own call
 * sites instead of faked here). */
#define _GNU_SOURCE /* pthread_getattr_np */
#define GW_COMPAT_LINUX_NO_STDIO_MACROS
#include "gw_compat_linux.h"

#include <dirent.h>
#include <fnmatch.h>
#include <elf.h>
#include <signal.h>
#include <ucontext.h>
#include <limits.h>
#include <sys/ioctl.h>
#include <errno.h>
#include <poll.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/timerfd.h>
#include <sys/types.h>
#include <time.h>
#include <unistd.h>

#include <SDL3/SDL_clipboard.h>
#include <SDL3/SDL_keyboard.h>
#include <SDL3/SDL_scancode.h>
#include <SDL3/SDL_video.h>

/* ---- timing ---- */

void Sleep(DWORD ms) {
    struct timespec ts;
    ts.tv_sec = ms / 1000;
    ts.tv_nsec = (long)(ms % 1000) * 1000000L;
    while (nanosleep(&ts, &ts) != 0 && errno == EINTR) {
        /* retry with the remaining time */
    }
}

static uint64_t gw_now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

DWORD GetTickCount(void) { return (DWORD)(gw_now_ns() / 1000000ull); }
uint64_t GetTickCount64(void) { return gw_now_ns() / 1000000ull; }

BOOL QueryPerformanceCounter(LARGE_INTEGER *out) {
    out->QuadPart = (long long)gw_now_ns();
    return TRUE;
}
BOOL QueryPerformanceFrequency(LARGE_INTEGER *out) {
    out->QuadPart = 1000000000ll; /* gw_now_ns() ticks in nanoseconds */
    return TRUE;
}
DWORD timeBeginPeriod(DWORD ms) {
    (void)ms;
    return 0; /* TIMERR_NOERROR; Linux has no matching process-wide timer-resolution knob to raise */
}

/* ---- errors ---- */
DWORD GetLastError(void) { return (DWORD)errno; }
void SetLastError(DWORD err) { errno = (int)err; }

/* ---- threading ---- */
enum gw_handle_kind { GW_HANDLE_THREAD, GW_HANDLE_TIMER };
struct gw_handle {
    enum gw_handle_kind kind;
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    unsigned refs;
    int done;
    int fd;
    gw_thread_fn fn;
    LPVOID arg;
};
static struct gw_handle *gw_handle_new(enum gw_handle_kind kind) {
    struct gw_handle *h = calloc(1, sizeof *h);
    pthread_condattr_t attr;
    if (!h) return NULL;
    h->kind = kind;
    h->refs = 1;
    h->fd = -1;
    pthread_mutex_init(&h->mutex, NULL);
    pthread_condattr_init(&attr);
    pthread_condattr_setclock(&attr, CLOCK_MONOTONIC);
    pthread_cond_init(&h->cond, &attr);
    pthread_condattr_destroy(&attr);
    return h;
}
static void gw_handle_release(struct gw_handle *h) {
    unsigned refs;
    pthread_mutex_lock(&h->mutex);
    refs = --h->refs;
    pthread_mutex_unlock(&h->mutex);
    if (!refs) {
        if (h->fd >= 0) close(h->fd);
        pthread_cond_destroy(&h->cond);
        pthread_mutex_destroy(&h->mutex);
        free(h);
    }
}
static void *gw_thread_trampoline(void *raw) {
    struct gw_handle *h = raw;
    if (!gw_linux_signal_thread_init()) abort();
    h->fn(h->arg);
    pthread_mutex_lock(&h->mutex);
    h->done = 1;
    pthread_cond_broadcast(&h->cond);
    pthread_mutex_unlock(&h->mutex);
    gw_handle_release(h);
    return NULL;
}
HANDLE CreateThread(void *sa, size_t stack_size, gw_thread_fn start, LPVOID arg,
                    DWORD flags, DWORD *out_tid) {
    pthread_t tid;
    pthread_attr_t attr;
    struct gw_handle *h;
    int rc;
    (void)sa;
    if (!start || flags || out_tid) { errno = ENOTSUP; return NULL; }
    h = gw_handle_new(GW_HANDLE_THREAD);
    if (!h) return NULL;
    h->fn = start;
    h->arg = arg;
    h->refs = 2; /* caller and worker have independent lifetimes */
    pthread_attr_init(&attr);
    pthread_attr_setdetachstate(&attr, PTHREAD_CREATE_DETACHED);
    rc = stack_size ? pthread_attr_setstacksize(&attr, stack_size) : 0;
    if (!rc) rc = pthread_create(&tid, &attr, gw_thread_trampoline, h);
    pthread_attr_destroy(&attr);
    if (rc) {
        gw_handle_release(h);
        gw_handle_release(h);
        errno = rc;
        return NULL;
    }
    return h;
}
BOOL CloseHandle(HANDLE raw) {
    if (!raw || raw == INVALID_HANDLE_VALUE) { errno = EINVAL; return FALSE; }
    gw_handle_release(raw);
    return TRUE;
}

/* ---- critical sections ---- */
void InitializeCriticalSection(CRITICAL_SECTION *cs) {
    pthread_mutex_t *m = (pthread_mutex_t *)malloc(sizeof *m);
    pthread_mutexattr_t attr;
    if (!m) abort();
    pthread_mutexattr_init(&attr);
    pthread_mutexattr_settype(&attr, PTHREAD_MUTEX_RECURSIVE);
    if (pthread_mutex_init(m, &attr)) abort();
    pthread_mutexattr_destroy(&attr);
    cs->opaque = m;
}
void DeleteCriticalSection(CRITICAL_SECTION *cs) {
    if (cs->opaque != NULL) {
        pthread_mutex_destroy((pthread_mutex_t *)cs->opaque);
        free(cs->opaque);
        cs->opaque = NULL;
    }
}
void EnterCriticalSection(CRITICAL_SECTION *cs) { pthread_mutex_lock((pthread_mutex_t *)cs->opaque); }
void LeaveCriticalSection(CRITICAL_SECTION *cs) { pthread_mutex_unlock((pthread_mutex_t *)cs->opaque); }

BOOL InitOnceExecuteOnce(PINIT_ONCE once, PINIT_ONCE_FN callback, PVOID param, PVOID *context) {
    BOOL ok;
    void *result = NULL;
    pthread_mutex_lock(&once->mutex);
    while (once->state == 1) pthread_cond_wait(&once->cond, &once->mutex);
    if (once->state == 2) {
        if (context) *context = once->context;
        pthread_mutex_unlock(&once->mutex);
        return TRUE;
    }
    once->state = 1;
    pthread_mutex_unlock(&once->mutex);
    ok = callback(once, param, &result);
    pthread_mutex_lock(&once->mutex);
    once->state = ok ? 2 : 0;
    if (ok) once->context = result;
    if (ok && context) *context = result;
    pthread_cond_broadcast(&once->cond);
    pthread_mutex_unlock(&once->mutex);
    return ok;
}

/* Publication is serialized; every caller obtains the same mutex. */
static pthread_mutex_t gw_srw_init_mutex = PTHREAD_MUTEX_INITIALIZER;
static pthread_mutex_t *gw_srw_mutex(SRWLOCK *lock) {
    pthread_mutex_t *m;
    pthread_mutex_lock(&gw_srw_init_mutex);
    if (!lock->opaque) {
        m = malloc(sizeof *m);
        if (!m || pthread_mutex_init(m, NULL)) abort();
        lock->opaque = m;
    }
    m = lock->opaque;
    pthread_mutex_unlock(&gw_srw_init_mutex);
    return m;
}
void AcquireSRWLockExclusive(SRWLOCK *lock) { pthread_mutex_lock(gw_srw_mutex(lock)); }
void ReleaseSRWLockExclusive(SRWLOCK *lock) { pthread_mutex_unlock(gw_srw_mutex(lock)); }

/* ---- waitable timer (frame pacing) ----
 * Backed by timerfd: WaitForSingleObject polls the fd for readability, matching the "wait up to
 * this many ms" contract the pacer uses it for. */
HANDLE CreateWaitableTimerExW(void *attrs, void *name, DWORD flags, DWORD access) {
    struct gw_handle *h;
    (void)attrs; (void)access;
    if (name || (flags & ~CREATE_WAITABLE_TIMER_HIGH_RESOLUTION)) { errno = ENOTSUP; return NULL; }
    h = gw_handle_new(GW_HANDLE_TIMER);
    if (!h) return NULL;
    h->fd = timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK | TFD_CLOEXEC);
    if (h->fd < 0) { gw_handle_release(h); return NULL; }
    return h;
}
BOOL SetWaitableTimer(HANDLE raw, const LARGE_INTEGER *due, LONG period,
                      void *routine, void *arg, BOOL resume) {
    struct gw_handle *h = raw;
    struct itimerspec its = {0};
    uint64_t ticks;
    (void)arg;
    if (!h || h->kind != GW_HANDLE_TIMER || !due || due->QuadPart > 0 ||
        period < 0 || routine || resume) { errno = EINVAL; return FALSE; }
    /* Split before multiplication to avoid overflow for large relative intervals. */
    ticks = 0ull - (uint64_t)due->QuadPart;
    its.it_value.tv_sec = ticks / 10000000;
    its.it_value.tv_nsec = (ticks % 10000000) * 100;
    if (!ticks) its.it_value.tv_nsec = 1;
    its.it_interval.tv_sec = period / 1000;
    its.it_interval.tv_nsec = (period % 1000) * 1000000;
    return timerfd_settime(h->fd, 0, &its, NULL) == 0;
}
DWORD WaitForSingleObject(HANDLE raw, DWORD timeout_ms) {
    struct gw_handle *h = raw;
    struct timespec deadline;
    uint64_t end = gw_now_ns() + (uint64_t)timeout_ms * 1000000;
    int rc = 0;
    if (!h || raw == INVALID_HANDLE_VALUE) { errno = EINVAL; return WAIT_FAILED; }
    if (h->kind == GW_HANDLE_THREAD) {
        deadline.tv_sec = end / 1000000000;
        deadline.tv_nsec = end % 1000000000;
        pthread_mutex_lock(&h->mutex);
        while (!h->done && !rc) {
            rc = timeout_ms == UINT32_MAX ? pthread_cond_wait(&h->cond, &h->mutex) :
                pthread_cond_timedwait(&h->cond, &h->mutex, &deadline);
        }
        int done = h->done;
        pthread_mutex_unlock(&h->mutex);
        return done ? WAIT_OBJECT_0 : rc == ETIMEDOUT ? WAIT_TIMEOUT : WAIT_FAILED;
    }
    for (;;) {
        struct pollfd pfd = { h->fd, POLLIN, 0 };
        uint64_t now = gw_now_ns();
        uint64_t remaining = now < end ? (end - now + 999999) / 1000000 : 0;
        int ms = timeout_ms == UINT32_MAX ? -1 : remaining > INT_MAX ? INT_MAX : (int)remaining;
        rc = poll(&pfd, 1, ms);
        if (rc < 0 && errno == EINTR) continue;
        if (!rc) {
            if (gw_now_ns() < end) continue;
            return WAIT_TIMEOUT;
        }
        if (rc < 0 || !(pfd.revents & POLLIN)) return WAIT_FAILED;
        uint64_t count;
        if (read(h->fd, &count, sizeof count) == sizeof count) return WAIT_OBJECT_0;
        if (errno != EAGAIN && errno != EINTR) return WAIT_FAILED;
    }
}

/* ---- memory ---- */
void *VirtualAlloc(void *addr, size_t size, DWORD alloc_type, DWORD protect) {
    int flags = MAP_PRIVATE | MAP_ANONYMOUS;
    void *p;
    (void)protect; /* only PAGE_READWRITE is ever requested */
    if (alloc_type & MEM_WRITE_WATCH) {
        /* Not implemented on Linux (no equivalent dirty-page-tracking mmap flag); the caller
         * (gw_mem_init) treats this as an optional optimisation and retries without it. */
        return NULL;
    }
    if (addr != NULL) {
#ifdef MAP_FIXED_NOREPLACE
        flags |= MAP_FIXED_NOREPLACE;
#else
        /* Use a hint and verify it below; never replace an existing mapping. */
#endif
    }
    p = mmap(addr, size, PROT_READ | PROT_WRITE, flags, -1, 0);
    if (p == MAP_FAILED) return NULL;
    if (addr && p != addr) {
        munmap(p, size);
        errno = EEXIST;
        return NULL;
    }
    return p;
}

/* ---- module path ---- */
DWORD GetModuleFileNameA(HANDLE unused_module, char *out, DWORD cap) {
    ssize_t n;
    (void)unused_module;
    if (cap == 0) {
        return 0;
    }
    n = readlink("/proc/self/exe", out, (size_t)cap - 1);
    if (n < 0) {
        out[0] = '\0';
        return 0;
    }
    out[n] = '\0';
    return (DWORD)n;
}

static void gw_normalize_path(const char *in, char *out, size_t cap);
DWORD GetFullPathNameA(const char *path, DWORD cap, char *out, char **file_part) {
    char input[PATH_MAX], absolute[PATH_MAX], resolved[PATH_MAX];
    char *save, *token;
    size_t n = 1;
    if (file_part) *file_part = NULL;
    if (!path || !*path) { errno = EINVAL; return 0; }
    if (strlen(path) >= sizeof input) { errno = ENAMETOOLONG; return 0; }
    gw_normalize_path(path, input, sizeof input);
    if (*input == '/') strcpy(absolute, input);
    else {
        if (!getcwd(absolute, sizeof absolute)) return 0;
        size_t cwd_len = strlen(absolute);
        if (cwd_len + strlen(input) + 2 > sizeof absolute) { errno = ENAMETOOLONG; return 0; }
        absolute[cwd_len++] = '/';
        strcpy(absolute + cwd_len, input);
    }
    resolved[0] = '/';
    for (token = strtok_r(absolute, "/", &save); token; token = strtok_r(NULL, "/", &save)) {
        if (!strcmp(token, ".")) continue;
        if (!strcmp(token, "..")) {
            while (n > 1 && resolved[n - 1] != '/') --n;
            if (n > 1) --n;
            continue;
        }
        if (n > 1) resolved[n++] = '/';
        size_t len = strlen(token);
        memcpy(resolved + n, token, len);
        n += len;
    }
    resolved[n] = 0;
    if (n >= cap || !out) return (DWORD)(n + 1);
    memcpy(out, resolved, n + 1);
    if (file_part) *file_part = strrchr(out, '/') + 1;
    return (DWORD)n;
}

/* ---- thread stack bounds ---- */
void GetCurrentThreadStackLimits(uintptr_t *low, uintptr_t *high) {
    pthread_attr_t attr;
    void *stackaddr = NULL;
    size_t stacksize = 0;
    if (pthread_getattr_np(pthread_self(), &attr) == 0) {
        pthread_attr_getstack(&attr, &stackaddr, &stacksize);
        pthread_attr_destroy(&attr);
    }
    *low = (uintptr_t)stackaddr;
    *high = (uintptr_t)stackaddr + (uintptr_t)stacksize;
}

/* ---- path separator normalization -------------------------------------------------------------
 * The platform layer builds paths with literal "\\" joins throughout (a Windows-native
 * convention baked into the source, e.g. gw_mods.c/gw_script.c/shim_dvd.c/gw_runtime.c), so
 * every path-taking compat entry point below normalizes in place before touching the real
 * filesystem, rather than requiring each of those files to be rewritten. */
static void gw_normalize_path(const char *in, char *out, size_t cap) {
    size_t i;
    for (i = 0; i + 1 < cap && in[i] != '\0'; ++i) {
        out[i] = (in[i] == '\\') ? '/' : in[i];
    }
    out[i] = '\0';
}

FILE *gw_compat_fopen(const char *path, const char *mode) {
    char buf[1024];
    gw_normalize_path(path, buf, sizeof buf);
    return fopen(buf, mode); /* the real libc fopen: this TU suppresses the header's macro */
}
int gw_compat_remove(const char *path) {
    char buf[1024];
    gw_normalize_path(path, buf, sizeof buf);
    return remove(buf);
}

/* ---- directory enumeration ---- */
struct gw_find_handle {
    DIR *dir;
    char pattern[256];
    char base[1024]; /* directory part, normalized, no trailing slash */
};

/* pattern is always "<dir>\*" (verified: every call site in this codebase uses exactly this
 * form, never a more specific glob) - strip the trailing wildcard and normalize the rest. */
static BOOL gw_find_advance(struct gw_find_handle *fh, WIN32_FIND_DATAA *out) {
    struct dirent *de;
    do { de = readdir(fh->dir); }
    while (de && fnmatch(fh->pattern, de->d_name, 0) != 0);
    struct stat st;
    char full[2048];
    if (de == NULL) {
        return FALSE;
    }
    snprintf(out->cFileName, sizeof out->cFileName, "%s", de->d_name);
    snprintf(full, sizeof full, "%s/%s", fh->base, de->d_name);
    out->dwFileAttributes = 0;
    out->nFileSizeHigh = 0;
    out->nFileSizeLow = 0;
    if (stat(full, &st) == 0) {
        if (S_ISDIR(st.st_mode)) {
            out->dwFileAttributes |= FILE_ATTRIBUTE_DIRECTORY;
        } else {
            out->nFileSizeHigh = (DWORD)((uint64_t)st.st_size >> 32);
            out->nFileSizeLow = (DWORD)((uint64_t)st.st_size & 0xFFFFFFFFu);
        }
    }
    return TRUE;
}

HANDLE FindFirstFileA(const char *pattern, WIN32_FIND_DATAA *out) {
    struct gw_find_handle *fh = (struct gw_find_handle *)malloc(sizeof *fh);
    if (fh == NULL) {
        return INVALID_HANDLE_VALUE;
    }
    char normalized[1280];
    if (!pattern || strlen(pattern) >= sizeof normalized) { free(fh); errno=ENAMETOOLONG; return INVALID_HANDLE_VALUE; }
    gw_normalize_path(pattern, normalized, sizeof normalized);
    char *slash = strrchr(normalized, '/');
    const char *glob = slash ? slash + 1 : normalized;
    if (strlen(glob) >= sizeof fh->pattern) { free(fh); errno=ENAMETOOLONG; return INVALID_HANDLE_VALUE; }
    strcpy(fh->pattern, glob);
    if (slash) *slash = 0;
    snprintf(fh->base, sizeof fh->base, "%s", slash ? (*normalized ? normalized : "/") : ".");
    fh->dir = opendir(fh->base);
    if (fh->dir == NULL) {
        free(fh);
        return INVALID_HANDLE_VALUE;
    }
    if (!gw_find_advance(fh, out)) {
        closedir(fh->dir);
        free(fh);
        return INVALID_HANDLE_VALUE;
    }
    return (HANDLE)fh;
}
BOOL FindNextFileA(HANDLE h, WIN32_FIND_DATAA *out) {
    return gw_find_advance((struct gw_find_handle *)h, out);
}
BOOL FindClose(HANDLE h) {
    struct gw_find_handle *fh = (struct gw_find_handle *)h;
    if (fh == NULL || fh == INVALID_HANDLE_VALUE) {
        return FALSE;
    }
    closedir(fh->dir);
    free(fh);
    return TRUE;
}

/* ---- file attributes / directory / file management ---- */
DWORD GetFileAttributesA(const char *path) {
    char buf[1024];
    struct stat st;
    gw_normalize_path(path, buf, sizeof buf);
    if (stat(buf, &st) != 0) {
        return INVALID_FILE_ATTRIBUTES;
    }
    return S_ISDIR(st.st_mode) ? FILE_ATTRIBUTE_DIRECTORY : 0u;
}
BOOL GetFileAttributesExA(const char *path, GET_FILEEX_INFO_LEVELS level, void *out) {
    char buf[1024];
    struct stat st;
    WIN32_FILE_ATTRIBUTE_DATA *data = (WIN32_FILE_ATTRIBUTE_DATA *)out;
    (void)level;
    gw_normalize_path(path, buf, sizeof buf);
    if (stat(buf, &st) != 0) {
        return FALSE;
    }
    data->dwFileAttributes = S_ISDIR(st.st_mode) ? FILE_ATTRIBUTE_DIRECTORY : 0u;
    data->nFileSizeHigh = (DWORD)((uint64_t)st.st_size >> 32);
    data->nFileSizeLow = (DWORD)((uint64_t)st.st_size & 0xFFFFFFFFu);
    {
        uint64_t stamp = ((uint64_t)st.st_mtim.tv_sec + 11644473600ull) * 10000000ull +
                         (uint64_t)st.st_mtim.tv_nsec / 100;
        data->ftLastWriteTime.dwLowDateTime = (DWORD)stamp;
        data->ftLastWriteTime.dwHighDateTime = (DWORD)(stamp >> 32);
    }
    return TRUE;
}
BOOL CreateDirectoryA(const char *path, void *unused_sa) {
    char buf[1024];
    (void)unused_sa;
    gw_normalize_path(path, buf, sizeof buf);
    return (mkdir(buf, 0755) == 0 || errno == EEXIST) ? TRUE : FALSE;
}
BOOL DeleteFileA(const char *path) {
    char buf[1024];
    gw_normalize_path(path, buf, sizeof buf);
    return unlink(buf) == 0 ? TRUE : FALSE;
}
BOOL RemoveDirectoryA(const char *path) {
    char buf[1024];
    gw_normalize_path(path, buf, sizeof buf);
    return rmdir(buf) == 0 ? TRUE : FALSE;
}
BOOL MoveFileExA(const char *from, const char *to, DWORD flags) {
    char a[1024], b[1024];
    (void)flags; /* POSIX rename() always replaces an existing destination */
    gw_normalize_path(from, a, sizeof a);
    gw_normalize_path(to, b, sizeof b);
    return rename(a, b) == 0 ? TRUE : FALSE;
}

/* ---- misc ---- */
DWORD GetCurrentProcessId(void) { return (DWORD)getpid(); }
void GetSystemTimeAsFileTime(FILETIME *ft) {
    struct timespec ts;
    uint64_t ticks;
    clock_gettime(CLOCK_REALTIME, &ts);
    /* 1601-01-01 to 1970-01-01 is 11644473600 s */
    ticks = ((uint64_t)ts.tv_sec + 11644473600ull) * 10000000ull + (uint64_t)ts.tv_nsec / 100u;
    ft->dwLowDateTime = (DWORD)(ticks & 0xFFFFFFFFu);
    ft->dwHighDateTime = (DWORD)(ticks >> 32);
}

DWORD GetTempPathA(DWORD cap, char *out) {
    const char *dir = getenv("TMPDIR");
    size_t n;
    if (dir == NULL || dir[0] == '\0') {
        dir = "/tmp";
    }
    n = (size_t)snprintf(out, cap, "%s/", dir);
    return (n < cap) ? (DWORD)n : 0;
}

void GetLocalTime(SYSTEMTIME *out) {
    time_t now = time(NULL);
    struct tm tmv;
    localtime_r(&now, &tmv);
    out->wYear = (uint16_t)(tmv.tm_year + 1900);
    out->wMonth = (uint16_t)(tmv.tm_mon + 1);
    out->wDayOfWeek = (uint16_t)tmv.tm_wday;
    out->wDay = (uint16_t)tmv.tm_mday;
    out->wHour = (uint16_t)tmv.tm_hour;
    out->wMinute = (uint16_t)tmv.tm_min;
    out->wSecond = (uint16_t)tmv.tm_sec;
    out->wMilliseconds = 0;
}

/* ---- keyboard polling ---- */
static int gw_vk_to_scancode(int vk) {
    if (vk >= 'A' && vk <= 'Z') {
        return SDL_SCANCODE_A + (vk - 'A');
    }
    if (vk >= '0' && vk <= '9') {
        static const int digit_scancode[10] = {
            SDL_SCANCODE_0, SDL_SCANCODE_1, SDL_SCANCODE_2, SDL_SCANCODE_3, SDL_SCANCODE_4,
            SDL_SCANCODE_5, SDL_SCANCODE_6, SDL_SCANCODE_7, SDL_SCANCODE_8, SDL_SCANCODE_9};
        return digit_scancode[vk - '0'];
    }
    if (vk >= VK_NUMPAD0 && vk <= VK_NUMPAD0 + 9) {
        static const int kp_scancode[10] = {
            SDL_SCANCODE_KP_0, SDL_SCANCODE_KP_1, SDL_SCANCODE_KP_2, SDL_SCANCODE_KP_3,
            SDL_SCANCODE_KP_4, SDL_SCANCODE_KP_5, SDL_SCANCODE_KP_6, SDL_SCANCODE_KP_7,
            SDL_SCANCODE_KP_8, SDL_SCANCODE_KP_9};
        return kp_scancode[vk - VK_NUMPAD0];
    }
    switch (vk) {
    case VK_SHIFT:
    case VK_LSHIFT:
        return SDL_SCANCODE_LSHIFT;
    case VK_CONTROL:
        return SDL_SCANCODE_LCTRL;
    case VK_MENU:
        return SDL_SCANCODE_LALT;
    case VK_ESCAPE:
        return SDL_SCANCODE_ESCAPE;
    case VK_SPACE:
        return SDL_SCANCODE_SPACE;
    case VK_RETURN:
        return SDL_SCANCODE_RETURN;
    case VK_TAB:
        return SDL_SCANCODE_TAB;
    case VK_BACK:
        return SDL_SCANCODE_BACKSPACE;
    case VK_DELETE:
        return SDL_SCANCODE_DELETE;
    case VK_INSERT:
        return SDL_SCANCODE_INSERT;
    case VK_HOME:
        return SDL_SCANCODE_HOME;
    case VK_END:
        return SDL_SCANCODE_END;
    case VK_PRIOR:
        return SDL_SCANCODE_PAGEUP;
    case VK_NEXT:
        return SDL_SCANCODE_PAGEDOWN;
    case VK_UP:
        return SDL_SCANCODE_UP;
    case VK_DOWN:
        return SDL_SCANCODE_DOWN;
    case VK_LEFT:
        return SDL_SCANCODE_LEFT;
    case VK_RIGHT:
        return SDL_SCANCODE_RIGHT;
    case VK_F1:
        return SDL_SCANCODE_F1;
    case VK_F9:
        return SDL_SCANCODE_F9;
    case VK_F10:
        return SDL_SCANCODE_F10;
    case VK_OEM_1:
        return SDL_SCANCODE_SEMICOLON;
    case VK_OEM_3:
        return SDL_SCANCODE_GRAVE;
    case VK_OEM_MINUS:
        return SDL_SCANCODE_MINUS;
    case VK_OEM_PERIOD:
        return SDL_SCANCODE_PERIOD;
    case VK_OEM_PLUS:
        return SDL_SCANCODE_EQUALS;
    case VK_OEM_COMMA:
        return SDL_SCANCODE_COMMA;
    case VK_OEM_2:
        return SDL_SCANCODE_SLASH;
    case VK_OEM_4:
        return SDL_SCANCODE_LEFTBRACKET;
    case VK_OEM_5:
        return SDL_SCANCODE_BACKSLASH;
    case VK_OEM_6:
        return SDL_SCANCODE_RIGHTBRACKET;
    case VK_OEM_7:
        return SDL_SCANCODE_APOSTROPHE;
    default:
        return SDL_SCANCODE_UNKNOWN;
    }
}

SHORT GetAsyncKeyState(int vk) {
    int scancode = gw_vk_to_scancode(vk);
    int numkeys = 0;
    const bool *state;
    if (scancode == SDL_SCANCODE_UNKNOWN) {
        return 0;
    }
    state = SDL_GetKeyboardState(&numkeys);
    if (state == NULL || scancode >= numkeys) {
        return 0;
    }
    return state[scancode] ? (SHORT)0x8000 : 0;
}

/* ---- window ---- */
static SDL_Window *gw_window;
void gw_set_window(void *sdl_window) { gw_window = (SDL_Window *)sdl_window; }
void *gw_get_window(void) { return gw_window; }
bool gw_window_focused(void) { return gw_window != NULL && SDL_GetKeyboardFocus() == gw_window; }
void gw_set_window_title(const char *title) {
    if (gw_window != NULL) {
        SDL_SetWindowTitle(gw_window, title);
    }
}

HWND GetForegroundWindow(void) { return gw_window_focused() ? (HWND)gw_window : NULL; }
void GetWindowThreadProcessId(HWND h, DWORD *out_pid) {
    *out_pid = (h != NULL && h == (HWND)gw_window) ? GetCurrentProcessId() : 0;
}

int gw_compat_ioctlsocket(SOCKET s, long cmd, unsigned long *argp) {
    if (cmd != FIONBIO || !argp) { errno = EINVAL; return -1; }
    int flags = fcntl(s, F_GETFL);
    if (flags < 0) return -1;
    return fcntl(s, F_SETFL, *argp ? flags | O_NONBLOCK : flags & ~O_NONBLOCK);
}

/* The executable is ET_EXEC. ELF program headers describe its actual executable ranges. */
extern const unsigned char __executable_start[];
int gw_linux_is_native_code(uintptr_t address) {
    const Elf32_Ehdr *eh = (const Elf32_Ehdr *)__executable_start;
    const Elf32_Phdr *ph = (const Elf32_Phdr *)(__executable_start + eh->e_phoff);
    for (unsigned i = 0; i < eh->e_phnum; ++i) {
        if (ph[i].p_type == PT_LOAD && (ph[i].p_flags & PF_X) &&
            address >= ph[i].p_vaddr && address - ph[i].p_vaddr < ph[i].p_memsz) return 1;
    }
    return 0;
}
static _Thread_local unsigned char gw_signal_stack[65536] __attribute__((aligned(16)));
static gw_linux_exec_resolver gw_exec_resolver;
static pthread_once_t gw_signals_once = PTHREAD_ONCE_INIT;
static int gw_signals_ok;
int gw_linux_signal_thread_init(void) {
    stack_t old, stack = { .ss_sp = gw_signal_stack, .ss_size = sizeof gw_signal_stack };
    if (sigaltstack(NULL, &old)) return 0;
    if (!(old.ss_flags & SS_DISABLE)) return 1;
    return sigaltstack(&stack, NULL) == 0;
}
void gw_linux_set_exec_resolver(gw_linux_exec_resolver resolver) {
    __atomic_store_n(&gw_exec_resolver, resolver, __ATOMIC_RELEASE);
}
static void gw_linux_signal(int signal_number, siginfo_t *info, void *raw) {
    ucontext_t *ctx = raw;
    uintptr_t ip = (uintptr_t)ctx->uc_mcontext.gregs[REG_EIP];
    gw_linux_exec_resolver resolver = __atomic_load_n(&gw_exec_resolver, __ATOMIC_ACQUIRE);
    /* x86 page-fault error bit 4 identifies an instruction fetch. Never redirect data faults. */
    if (signal_number == SIGSEGV && info->si_code == SEGV_ACCERR &&
        (ctx->uc_mcontext.gregs[REG_ERR] & 16) && (uintptr_t)info->si_addr == ip && resolver) {
        uintptr_t target = resolver(ip);
        if (target && gw_linux_is_native_code(target)) {
            ctx->uc_mcontext.gregs[REG_EIP] = (greg_t)target;
            return;
        }
    }
    /* Only async-signal-safe operations here; the core contains the complete original context. */
    char message[] = "gw: fatal Linux signal at ELF address 0x00000000; inspect matching debug ELF/core\n";
    const char hex[] = "0123456789abcdef";
    for (unsigned i = 0; i < 8; ++i) message[sizeof("gw: fatal Linux signal at ELF address 0x") - 1 + i] = hex[(ip >> ((7 - i) * 4)) & 15];
    (void)write(STDERR_FILENO, message, sizeof message - 1);
    struct sigaction action = {0};
    action.sa_handler = SIG_DFL;
    sigemptyset(&action.sa_mask);
    sigaction(signal_number, &action, NULL);
    if (info->si_code <= 0) raise(signal_number);
    /* Returning retries a synchronous fault with the default disposition, preserving the core. */
}
static void gw_linux_signals_init(void) {
    struct sigaction action = {0};
    action.sa_sigaction = gw_linux_signal;
    action.sa_flags = SA_SIGINFO | SA_ONSTACK;
    sigemptyset(&action.sa_mask);
    gw_signals_ok = sigaction(SIGSEGV, &action, NULL) == 0 &&
                    sigaction(SIGBUS, &action, NULL) == 0 &&
                    sigaction(SIGILL, &action, NULL) == 0;
}
int gw_linux_install_signals(void) {
    if (!gw_linux_signal_thread_init()) return 0;
    pthread_once(&gw_signals_once, gw_linux_signals_init);
    return gw_signals_ok;
}

/* x87 PRECISION CONTROL. A Windows process starts every thread with the x87 control word at 0x027F
 * (53-bit significands, round to nearest); a Linux process starts at 0x037F (64-bit). Game code is
 * compiled for SSE2, but the 32-bit ABIs return float/double in ST0 and libm's i386 routines
 * (fma, fmod...) compute on the x87 stack, so the control word decides the last bit of a value the
 * simulation can see. Two builds of one commit must start from the same word or they drift (found by
 * comparing per-frame state digests of a Windows and a Linux run, tools/xplat). Threads inherit it. */
__attribute__((constructor(101))) static void gw_linux_fpu_init(void) {
    unsigned short cw = 0x027F;
    __asm__ volatile("fldcw %0" : : "m"(cw));
}
