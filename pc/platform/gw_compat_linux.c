/* Implementations for gw_compat_linux.h. See that header for what this deliberately does not
 * cover (winsock, the GC HID adapter, SEH, PAGE_GUARD watchpoints - each ported at its own call
 * sites instead of faked here). */
#define _GNU_SOURCE /* pthread_getattr_np */
#define GW_COMPAT_LINUX_NO_STDIO_MACROS
#include "gw_compat_linux.h"

#include <dirent.h>
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
struct gw_thread_trampoline_arg {
    gw_thread_fn fn;
    LPVOID arg;
};
static void *gw_thread_trampoline(void *raw) {
    struct gw_thread_trampoline_arg *a = (struct gw_thread_trampoline_arg *)raw;
    gw_thread_fn fn = a->fn;
    LPVOID arg = a->arg;
    free(a);
    fn(arg);
    return NULL;
}

/* Neither call site in this codebase ever waits on the handle CreateThread returns (both do
 * `CloseHandle(CreateThread(...))` immediately - see gw_start_watchdog, gw_test_run_all) - they
 * are fire-and-forget background threads, so this detaches immediately rather than emulating a
 * joinable Win32 handle nothing ever joins. */
HANDLE CreateThread(void *unused_sa, size_t unused_stack_size, gw_thread_fn start, LPVOID arg,
                    DWORD unused_flags, DWORD *unused_out_tid) {
    pthread_t tid;
    struct gw_thread_trampoline_arg *a =
        (struct gw_thread_trampoline_arg *)malloc(sizeof *a);
    (void)unused_sa;
    (void)unused_stack_size;
    (void)unused_flags;
    (void)unused_out_tid;
    if (a == NULL) {
        return NULL;
    }
    a->fn = start;
    a->arg = arg;
    if (pthread_create(&tid, NULL, gw_thread_trampoline, a) != 0) {
        free(a);
        return NULL;
    }
    pthread_detach(tid);
    return (HANDLE)(intptr_t)1; /* opaque non-NULL "started ok"; nothing dereferences this */
}

BOOL CloseHandle(HANDLE h) {
    (void)h;
    return TRUE;
}

/* ---- critical sections ---- */
void InitializeCriticalSection(CRITICAL_SECTION *cs) {
    pthread_mutex_t *m = (pthread_mutex_t *)malloc(sizeof *m);
    pthread_mutex_init(m, NULL);
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

/* ---- waitable timer (frame pacing) ----
 * Backed by timerfd: WaitForSingleObject polls the fd for readability, matching the "wait up to
 * this many ms" contract the pacer uses it for. */
HANDLE CreateWaitableTimerExW(void *unused_attrs, void *unused_name, DWORD flags,
                              DWORD unused_access) {
    int fd;
    (void)unused_attrs;
    (void)unused_name;
    (void)flags;
    (void)unused_access;
    fd = timerfd_create(CLOCK_MONOTONIC, TFD_NONBLOCK);
    if (fd < 0) {
        return NULL;
    }
    return (HANDLE)(intptr_t)(fd + 1); /* +1 so fd 0 never collides with NULL */
}

BOOL SetWaitableTimer(HANDLE h, const LARGE_INTEGER *due, LONG unused_period,
                      void *unused_completion_routine, void *unused_completion_arg,
                      BOOL unused_resume) {
    int fd = (int)(intptr_t)h - 1;
    struct itimerspec its;
    long long ns = -due->QuadPart * 100ll; /* due->QuadPart: negative 100ns units -> ns */
    (void)unused_period;
    (void)unused_completion_routine;
    (void)unused_completion_arg;
    (void)unused_resume;
    if (ns < 1) {
        ns = 1;
    }
    its.it_value.tv_sec = ns / 1000000000ll;
    its.it_value.tv_nsec = ns % 1000000000ll;
    its.it_interval.tv_sec = 0;
    its.it_interval.tv_nsec = 0;
    return timerfd_settime(fd, 0 /* relative */, &its, NULL) == 0;
}

DWORD WaitForSingleObject(HANDLE h, DWORD timeout_ms) {
    int fd = (int)(intptr_t)h - 1;
    struct pollfd pfd;
    int rc;
    if (fd < 0) {
        return WAIT_FAILED;
    }
    pfd.fd = fd;
    pfd.events = POLLIN;
    pfd.revents = 0;
    rc = poll(&pfd, 1, (int)timeout_ms);
    if (rc > 0 && (pfd.revents & POLLIN)) {
        uint64_t n;
        ssize_t r = read(fd, &n, sizeof n); /* drain the timerfd expiration count */
        (void)r;
        return WAIT_OBJECT_0;
    }
    return (rc == 0) ? WAIT_TIMEOUT : WAIT_FAILED;
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
        flags |= MAP_FIXED;
#endif
    }
    p = mmap(addr, size, PROT_READ | PROT_WRITE, flags, -1, 0);
    if (p == MAP_FAILED) {
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
    char base[1024]; /* directory part, normalized, no trailing slash */
};

/* pattern is always "<dir>\*" (verified: every call site in this codebase uses exactly this
 * form, never a more specific glob) - strip the trailing wildcard and normalize the rest. */
static void gw_find_split_dir(const char *pattern, char *out, size_t cap) {
    char norm[1024];
    size_t n;
    gw_normalize_path(pattern, norm, sizeof norm);
    n = strlen(norm);
    if (n >= 2 && norm[n - 1] == '*' && norm[n - 2] == '/') {
        norm[n - 2] = '\0';
    }
    if (norm[0] == '\0') {
        snprintf(out, cap, ".");
    } else {
        snprintf(out, cap, "%s", norm);
    }
}

static BOOL gw_find_advance(struct gw_find_handle *fh, WIN32_FIND_DATAA *out) {
    struct dirent *de = readdir(fh->dir);
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
    gw_find_split_dir(pattern, fh->base, sizeof fh->base);
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
