/* Win32-primitive compatibility shim for the Linux build of the port.
 *
 * The platform layer was written against a bounded, repeated set of Win32 primitives (timing,
 * threading, critical sections, module-path lookup, page mapping, directory enumeration, and a
 * standard BSD-socket subset) rather than anything Windows-specific in spirit. Each file keeps its
 * `#include <windows.h>` under `#ifdef _WIN32` and falls back to this header otherwise, so the same
 * call sites work on both platforms unchanged. Genuinely Windows-only mechanisms (the GC adapter's
 * HID access, SEH, PAGE_GUARD watchpoints, GDI text, WIC image decode, the CryptoAPI) are NOT
 * faked to full fidelity here - each of those gets its own `#ifdef _WIN32` block at its (few) call
 * sites with a documented fallback. See gw_compat_linux.c for the implementations. */
#ifndef GW_COMPAT_LINUX_H
#define GW_COMPAT_LINUX_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <pthread.h>
#include <string.h>

/* ---- sockets (gw_net.c / gw_netplay.c / gw_script.c's console server) ----
 * Those call sites use a standard BSD-socket subset; Windows spells it with Winsock names plus a
 * startup/cleanup pair and two per-connection error codes. send/recv/bind/connect/getaddrinfo are
 * already identical on both platforms, so only the names and the startup pair differ. This block
 * is outside extern "C" because it pulls in the system socket headers, which carry their own
 * linkage blocks. */
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netdb.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>
typedef int SOCKET;
#define INVALID_SOCKET (-1)
#define SOCKET_ERROR (-1)
#define closesocket(fd) close(fd)
typedef struct {
    int unused;
} WSADATA;
#define MAKEWORD(a, b) ((unsigned short)(((a) & 0xff) | (((b) & 0xff) << 8)))
static inline int WSAStartup(unsigned short v, WSADATA *d) {
    (void)v;
    (void)d;
    return 0;
}
static inline int WSACleanup(void) { return 0; }
#define WSAGetLastError() (errno)
#define WSAEWOULDBLOCK EWOULDBLOCK
#define WSAECONNRESET ECONNRESET
#define WSAEMSGSIZE EMSGSIZE
/* Windows' ioctlsocket(FIONBIO) is Linux fcntl(O_NONBLOCK); the arg is a u_long 1/0. */
#ifndef FIONBIO
#define FIONBIO 0x5421
#endif

#define ioctlsocket(s, cmd, argp) gw_compat_ioctlsocket((s), (cmd), (unsigned long *)(argp))

#ifdef __cplusplus
extern "C" {
#endif

int gw_compat_ioctlsocket(SOCKET s, long cmd, unsigned long *argp);

/* ---- basic Win32 typedefs actually used by this codebase ---- */
typedef unsigned int DWORD;
typedef int BOOL;
typedef long LONG;
typedef unsigned long ULONG_PTR;
typedef void *LPVOID;
typedef const char *LPCSTR;
typedef void *HANDLE;
typedef unsigned long long ULONGLONG;
typedef long long LONGLONG;
typedef unsigned char BYTE;
#define WINAPI /* __stdcall on Windows; empty here so the same signatures compile */
#define CALLBACK

/* A read/write compiler barrier, the one x86 intrinsic the platform layer uses directly
 * (shim_ax.c's sample-buffer handoff). A memory-clobber asm is the portable equivalent. */
#define _ReadWriteBarrier() __asm__ __volatile__("" ::: "memory")

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif

#define MAX_PATH 260

#define INVALID_HANDLE_VALUE ((HANDLE)(intptr_t)-1)

typedef union {
    long long QuadPart;
} LARGE_INTEGER;

/* ---- timing ---- */
void Sleep(DWORD ms);
DWORD GetTickCount(void);
uint64_t GetTickCount64(void);
BOOL QueryPerformanceCounter(LARGE_INTEGER *out);
BOOL QueryPerformanceFrequency(LARGE_INTEGER *out);
DWORD timeBeginPeriod(DWORD ms); /* no-op on Linux; kept so call sites need no #ifdef */

/* ---- errors ---- */
DWORD GetLastError(void);
void SetLastError(DWORD err);

/* ---- threading ---- */
typedef DWORD(*gw_thread_fn)(LPVOID);
HANDLE CreateThread(void *unused_sa, size_t unused_stack_size, gw_thread_fn start, LPVOID arg,
                    DWORD unused_flags, DWORD *unused_out_tid);
DWORD WaitForSingleObject(HANDLE h, DWORD timeout_ms);
BOOL CloseHandle(HANDLE h);
#define WAIT_OBJECT_0 0u
#define WAIT_TIMEOUT 258u
#define WAIT_FAILED 0xFFFFFFFFu

typedef struct {
    void *opaque; /* pthread_mutex_t*, allocated lazily so the struct's layout need not match glibc's */
} CRITICAL_SECTION;
void InitializeCriticalSection(CRITICAL_SECTION *cs);
void DeleteCriticalSection(CRITICAL_SECTION *cs);
void EnterCriticalSection(CRITICAL_SECTION *cs);
void LeaveCriticalSection(CRITICAL_SECTION *cs);

typedef struct { pthread_mutex_t mutex; pthread_cond_t cond; int state; void *context; } INIT_ONCE;
typedef INIT_ONCE *PINIT_ONCE;
typedef void *PVOID;
typedef BOOL (*PINIT_ONCE_FN)(PINIT_ONCE, PVOID, PVOID *);
#define INIT_ONCE_STATIC_INIT { PTHREAD_MUTEX_INITIALIZER, PTHREAD_COND_INITIALIZER, 0, NULL }
BOOL InitOnceExecuteOnce(PINIT_ONCE once, PINIT_ONCE_FN callback, PVOID param, PVOID *context);

/* Win32 slim reader/writer lock, exclusive-only (shim_pad.c's pad log uses just the exclusive
 * path). Same lazy-pthread_mutex shape as CRITICAL_SECTION so a static SRWLOCK_INIT (all-NULL) is
 * valid before any initializer runs. */
typedef struct {
    void *opaque;
} SRWLOCK;
#define SRWLOCK_INIT { NULL }
void AcquireSRWLockExclusive(SRWLOCK *lock);
void ReleaseSRWLockExclusive(SRWLOCK *lock);

/* ---- high-resolution waitable timer (frame pacing; see shim_vi.c) ---- */
#define CREATE_WAITABLE_TIMER_HIGH_RESOLUTION 0x00000002u
#define TIMER_ALL_ACCESS 0x1F0000u
HANDLE CreateWaitableTimerExW(void *unused_attrs, void *unused_name, DWORD flags,
                              DWORD unused_access);
/* due->QuadPart: 100ns units, negative = relative delay (the only form this codebase uses). */
BOOL SetWaitableTimer(HANDLE h, const LARGE_INTEGER *due, LONG unused_period,
                      void *unused_completion_routine, void *unused_completion_arg,
                      BOOL unused_resume);

/* Linux ELF and signal support. Callback must only inspect immutable tables/TLS. */
typedef uintptr_t (*gw_linux_exec_resolver)(uintptr_t address);
int gw_linux_is_native_code(uintptr_t address);
int gw_linux_signal_thread_init(void);
int gw_linux_install_signals(void);
void gw_linux_set_exec_resolver(gw_linux_exec_resolver resolver);

/* ---- memory ---- */
#define MEM_RESERVE 0x00002000u
#define MEM_COMMIT 0x00001000u
#define MEM_WRITE_WATCH 0x00200000u /* honored when a backend works (gw_writewatch_linux.c), else VirtualAlloc returns NULL */
#define PAGE_READWRITE 0x04u
void *VirtualAlloc(void *addr, size_t size, DWORD alloc_type, DWORD protect);

/* ---- write watch (gw_writewatch_linux.c): which pages of a MEM_WRITE_WATCH range were written ---- */
typedef void *PVOID;
typedef unsigned int UINT;
#define WRITE_WATCH_FLAG_RESET 1u
UINT GetWriteWatch(DWORD flags, PVOID base, size_t size, PVOID *addrs, ULONG_PTR *count, DWORD *granularity);
UINT ResetWriteWatch(PVOID base, size_t size);
int gw_linux_writewatch_start(void *base, size_t size);
const char *gw_linux_writewatch_name(void);

/* ---- module path ---- */
DWORD GetModuleFileNameA(HANDLE unused_module, char *out, DWORD cap);
DWORD GetFullPathNameA(const char *path, DWORD cap, char *out, char **file_part);

/* ---- directory enumeration (only the "<dir>\*" pattern this codebase issues) ---- */
typedef struct {
    DWORD dwFileAttributes;
    DWORD nFileSizeHigh;
    DWORD nFileSizeLow;
    char cFileName[MAX_PATH];
} WIN32_FIND_DATAA;
#define FILE_ATTRIBUTE_DIRECTORY 0x10u
HANDLE FindFirstFileA(const char *pattern, WIN32_FIND_DATAA *out);
BOOL FindNextFileA(HANDLE h, WIN32_FIND_DATAA *out);
BOOL FindClose(HANDLE h);

/* ---- file attributes / directory / file management (backslash-normalizing) ---- */
#define INVALID_FILE_ATTRIBUTES ((DWORD)-1)
typedef enum { GetFileExInfoStandard } GET_FILEEX_INFO_LEVELS;
typedef struct {
    DWORD dwLowDateTime;
    DWORD dwHighDateTime;
} FILETIME;
void GetSystemTimeAsFileTime(FILETIME *ft); /* 100 ns ticks since 1601, like the Win32 call */
typedef struct {
    DWORD dwFileAttributes;
    DWORD nFileSizeHigh;
    DWORD nFileSizeLow;
    FILETIME ftLastWriteTime;
} WIN32_FILE_ATTRIBUTE_DATA;
static inline int CompareFileTime(const FILETIME *a, const FILETIME *b) {
    uint64_t av = ((uint64_t)a->dwHighDateTime << 32) | a->dwLowDateTime;
    uint64_t bv = ((uint64_t)b->dwHighDateTime << 32) | b->dwLowDateTime;
    return av < bv ? -1 : av > bv ? 1 : 0;
}
DWORD GetFileAttributesA(const char *path);
BOOL GetFileAttributesExA(const char *path, GET_FILEEX_INFO_LEVELS level, void *out);
BOOL CreateDirectoryA(const char *path, void *unused_sa);
BOOL DeleteFileA(const char *path);
BOOL RemoveDirectoryA(const char *path);
#define MOVEFILE_REPLACE_EXISTING 1u
BOOL MoveFileExA(const char *from, const char *to, DWORD flags);

#include <stdlib.h> /* malloc/free/calloc/strtoull - several files reached these transitively
                      * through windows.h and don't include <stdlib.h> themselves */
#include <strings.h> /* strcasecmp/strncasecmp */
#define strtok_s strtok_r
#define _stricmp strcasecmp
#define _strnicmp strncasecmp
#define _strtoui64 strtoull
#define _putenv_s(name, value) setenv((name), (value), 1)

/* fopen()/remove() are called throughout the platform layer with backslash-joined paths built for
 * Windows; redirect them through wrappers that normalize separators first. Defined after
 * <stdio.h> would already have declared the real symbols, so this only rewrites call sites, not
 * the libc declaration itself. */
#include <stdio.h>
FILE *gw_compat_fopen(const char *path, const char *mode);
static inline void *SecureZeroMemory(void *p, size_t n) {
    volatile unsigned char *q = (volatile unsigned char *)p;
    while (n--) *q++ = 0;
    return p;
}
static inline int gw_compat_fopen_s(FILE **out, const char *path, const char *mode) {
    if (!out || !path || !mode) return EINVAL;
    *out = gw_compat_fopen(path, mode);
    return *out != NULL ? 0 : errno;
}
#define fopen_s gw_compat_fopen_s
FILE *gw_compat_fopen(const char *path, const char *mode);
int gw_compat_remove(const char *path);
/* gw_compat_linux.c itself defines gw_compat_fopen/gw_compat_remove in terms of the real fopen()/
 * remove() and #defines this to suppress its own recursion. */
#ifndef GW_COMPAT_LINUX_NO_STDIO_MACROS
#define fopen gw_compat_fopen
#define remove gw_compat_remove
#endif

/* ---- thread stack bounds (gw_ppc.c's interpreter stack-overflow guard) ---- */
void GetCurrentThreadStackLimits(uintptr_t *low, uintptr_t *high);

/* ---- misc ---- */
DWORD GetCurrentProcessId(void);
DWORD GetTempPathA(DWORD cap, char *out); /* returns a path WITH a trailing separator, like Win32 */
typedef struct {
    uint16_t wYear, wMonth, wDayOfWeek, wDay, wHour, wMinute, wSecond, wMilliseconds;
} SYSTEMTIME;
void GetLocalTime(SYSTEMTIME *out);
#define _putenv putenv

/* ---- keyboard polling (the port's primary input path: GetAsyncKeyState(VK_*), polled every
 * frame rather than event-driven - see shim_pad.c). Backed by SDL3's continuously-updated
 * keyboard-state array, which is an equivalent polling model. Only a SHORT with bit 0x8000 set
 * for "down" is ever read by call sites (`GetAsyncKeyState(vk) & 0x8000`), matching Win32. */
typedef short SHORT;
SHORT GetAsyncKeyState(int vk);
#define VK_SHIFT 0x10
#define VK_LSHIFT 0xA0
#define VK_CONTROL 0x11
#define VK_MENU 0x12
#define VK_ESCAPE 0x1B
#define VK_SPACE 0x20
#define VK_RETURN 0x0D
#define VK_TAB 0x09
#define VK_BACK 0x08
#define VK_DELETE 0x2E
#define VK_INSERT 0x2D
#define VK_HOME 0x24
#define VK_END 0x23
#define VK_PRIOR 0x21 /* Page Up */
#define VK_NEXT 0x22  /* Page Down */
#define VK_UP 0x26
#define VK_DOWN 0x28
#define VK_LEFT 0x25
#define VK_RIGHT 0x27
#define VK_F1 0x70
#define VK_F9 0x78
#define VK_F10 0x79
#define VK_NUMPAD0 0x60 /* ..VK_NUMPAD0+9: contiguous, matches gw_script.c's arithmetic use */
#define VK_OEM_1 0xBA      /* ;: */
#define VK_OEM_3 0xC0      /* `~ */
#define VK_OEM_MINUS 0xBD  /* -_ */
#define VK_OEM_PERIOD 0xBE /* .> */
#define VK_OEM_PLUS 0xBB   /* =+ */
#define VK_OEM_COMMA 0xBC  /* ,< */
#define VK_OEM_2 0xBF      /* /? */
#define VK_OEM_4 0xDB      /* [{ */
#define VK_OEM_5 0xDC      /* backslash and | */
#define VK_OEM_6 0xDD      /* ]} */
#define VK_OEM_7 0xDE      /* quote */

/* ---- window (see gw.h's gw_get_window()/gw_window_focused()/gw_set_window_title()) ----
 *
 * Several files ask "is this process's own window focused" via the idiom
 * `HWND fg = GetForegroundWindow(); if (fg) GetWindowThreadProcessId(fg,&pid); return pid ==
 * GetCurrentProcessId();`. Rather than editing every call site, these compose to the same answer:
 * GetForegroundWindow returns our window's handle only when it is actually focused (NULL
 * otherwise, a legitimate real value meaning "no foreground window"), and
 * GetWindowThreadProcessId only reports our own pid for that same handle - so an unfocused window
 * or any other handle correctly compares unequal to GetCurrentProcessId(). */
typedef void *HWND;
HWND GetForegroundWindow(void);
void GetWindowThreadProcessId(HWND h, DWORD *out_pid);

#ifdef __cplusplus
}
#endif

#endif /* GW_COMPAT_LINUX_H */
