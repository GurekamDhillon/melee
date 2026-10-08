/* gw_writewatch_linux.c - Windows' GetWriteWatch/ResetWriteWatch for MEM1, on Linux.
 *
 * WHY. gw_snap.c (rollback and the Lab's savestates) copies only the 4 KB pages written since the
 * last poll. Windows tracks that for free (VirtualAlloc MEM_WRITE_WATCH). Linux had no equivalent
 * here, so every snapshot SAVE compared and copied all of MEM1 (40 MB read twice, "full-copy mode"):
 * ~8 ms per tick on a laptop against 0.8 ms on Windows, enough to drop an online match below 60 fps.
 *
 * BACKENDS, tried in this order (MELEE_WRITEWATCH=auto|uffd|softdirty|off; default auto):
 *   uffd       userfaultfd write-protect in ASYNC mode + the PAGEMAP_SCAN ioctl (Linux 6.7+). The range
 *              is write-protected; the first write to a page is resolved by the kernel (no signal, no
 *              handler) and marks it "written"; one ioctl returns the written pages and re-protects
 *              them atomically. Only MEM1 is touched, so the cost is the MEM1 page faults.
 *   softdirty  /proc/self/pagemap bit 55 + writing 4 to /proc/self/clear_refs (CONFIG_MEM_SOFT_DIRTY,
 *              Linux 3.11+). clear_refs resets the whole process, not just MEM1, so it is slower.
 * Each backend proves itself at start with a self-test (a user write, a kernel write via read(2), a
 * second write after a reset, and "nothing reported when nothing was written"). If none passes the
 * caller gets NULL from VirtualAlloc(MEM_WRITE_WATCH) and gw_snap.c runs in full-copy mode, as before.
 * A page reported dirty that is not is harmless (the snapshot copies it); a page that is written and
 * NOT reported would corrupt a snapshot, which is why the self-test is strict and why
 * MELEE_SNAP_VERIFY=1 (memcmp of live against the slot after every dirty save) stays available.
 *
 * Only the thread that polls (the simulation thread) is expected to write MEM1 while it polls - the
 * same assumption the snapshot copy itself makes.
 */
#include "gw_compat_linux.h"

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

/* ---- kernel ABI, spelled out here because the build rootfs' headers predate it ---- */
#ifndef __NR_userfaultfd
#define __NR_userfaultfd 374 /* i386 */
#endif
#define GWWW_UFFD_API 0xAAull
#define GWWW_UFFD_USER_MODE_ONLY 1
#define GWWW_UFFD_FEATURE_WP_UNPOPULATED (1ull << 13)
#define GWWW_UFFD_FEATURE_WP_ASYNC (1ull << 15)
#define GWWW_UFFDIO_REGISTER_MODE_WP (1ull << 1)
struct gwww_uffdio_api {
    uint64_t api, features, ioctls;
};
struct gwww_uffdio_range {
    uint64_t start, len;
};
struct gwww_uffdio_register {
    struct gwww_uffdio_range range;
    uint64_t mode, ioctls;
};
#define GWWW_UFFDIO_API _IOWR(0xAA, 0x3F, struct gwww_uffdio_api)
#define GWWW_UFFDIO_REGISTER _IOWR(0xAA, 0x00, struct gwww_uffdio_register)

struct gwww_page_region {
    uint64_t start, end, categories;
};
struct gwww_pm_scan_arg {
    uint64_t size, flags, start, end, walk_end, vec, vec_len, max_pages, category_inverted, category_mask,
        category_anyof_mask, return_mask;
};
#define GWWW_PAGEMAP_SCAN _IOWR('f', 16, struct gwww_pm_scan_arg)
#define GWWW_PAGE_IS_WRITTEN (1ull << 1)
#define GWWW_PM_SCAN_WP_MATCHING (1ull << 0)
#define GWWW_PM_SCAN_CHECK_WPASYNC (1ull << 1)

#define GWWW_PAGE 4096u
#define GWWW_VEC 512

enum { WW_OFF = 0, WW_UFFD = 1, WW_SOFTDIRTY = 2 };

static struct {
    int mode;
    int pm_fd;     /* /proc/self/pagemap */
    int cr_fd;     /* /proc/self/clear_refs (softdirty) */
    int uffd;
    uintptr_t base;
    size_t size;
    uint64_t *sd_buf; /* softdirty: one pagemap entry per tracked page */
    struct gwww_page_region vec[GWWW_VEC];
    const char *why;  /* why the last backend failed, for the log */
} ww;

const char *gw_linux_writewatch_name(void) {
    return ww.mode == WW_UFFD ? "userfaultfd write-protect (PAGEMAP_SCAN)"
         : ww.mode == WW_SOFTDIRTY ? "soft-dirty (pagemap/clear_refs)" : "none";
}

/* ---- uffd backend ---- */
static int uffd_scan(uintptr_t start, uintptr_t end, int reset, PVOID *addrs, ULONG_PTR *count, ULONG_PTR cap) {
    uintptr_t cur = start;
    ULONG_PTR n = 0;
    while (cur < end) {
        struct gwww_pm_scan_arg a;
        long r;
        int i;
        memset(&a, 0, sizeof a);
        a.size = sizeof a;
        a.flags = reset ? (GWWW_PM_SCAN_WP_MATCHING | GWWW_PM_SCAN_CHECK_WPASYNC) : GWWW_PM_SCAN_CHECK_WPASYNC;
        a.start = cur;
        a.end = end;
        a.vec = (uint64_t) (uintptr_t) ww.vec;
        a.vec_len = GWWW_VEC;
        a.category_mask = GWWW_PAGE_IS_WRITTEN;
        a.return_mask = GWWW_PAGE_IS_WRITTEN;
        r = ioctl(ww.pm_fd, GWWW_PAGEMAP_SCAN, &a);
        if (r < 0) {
            return -1;
        }
        for (i = 0; i < (int) r; ++i) {
            uint64_t p;
            for (p = ww.vec[i].start; p < ww.vec[i].end; p += GWWW_PAGE) {
                if (addrs != NULL && n < cap) {
                    addrs[n] = (PVOID) (uintptr_t) p;
                }
                ++n;
            }
        }
        if (a.walk_end <= cur) {
            return -1; /* no progress: never loop forever */
        }
        cur = (uintptr_t) a.walk_end;
    }
    *count = n;
    return 0;
}

static int uffd_start(void) {
    struct gwww_uffdio_api api;
    struct gwww_uffdio_register reg;
    ww.uffd = (int) syscall(__NR_userfaultfd, O_CLOEXEC | O_NONBLOCK | GWWW_UFFD_USER_MODE_ONLY);
    if (ww.uffd < 0) {
        ww.why = "userfaultfd() refused (kernel without it, or vm.unprivileged_userfaultfd and no UFFD_USER_MODE_ONLY)";
        return -1;
    }
    memset(&api, 0, sizeof api);
    api.api = GWWW_UFFD_API;
    api.features = GWWW_UFFD_FEATURE_WP_ASYNC | GWWW_UFFD_FEATURE_WP_UNPOPULATED;
    if (ioctl(ww.uffd, GWWW_UFFDIO_API, &api) != 0) {
        memset(&api, 0, sizeof api);
        api.api = GWWW_UFFD_API;
        api.features = GWWW_UFFD_FEATURE_WP_ASYNC;
        if (ioctl(ww.uffd, GWWW_UFFDIO_API, &api) != 0) {
            ww.why = "kernel has no UFFD_FEATURE_WP_ASYNC (needs Linux 6.7)";
            return -1;
        }
    }
    memset(&reg, 0, sizeof reg);
    reg.range.start = ww.base;
    reg.range.len = ww.size;
    reg.mode = GWWW_UFFDIO_REGISTER_MODE_WP;
    if (ioctl(ww.uffd, GWWW_UFFDIO_REGISTER, &reg) != 0) {
        ww.why = "UFFDIO_REGISTER of MEM1 failed";
        return -1;
    }
    ww.pm_fd = open("/proc/self/pagemap", O_RDONLY | O_CLOEXEC);
    if (ww.pm_fd < 0) {
        ww.why = "cannot open /proc/self/pagemap";
        return -1;
    }
    {
        /* arm: the first scan write-protects whatever is populated; nothing to report on a fresh range */
        ULONG_PTR c = 0;
        if (uffd_scan(ww.base, ww.base + ww.size, 1, NULL, &c, 0) != 0) {
            ww.why = "PAGEMAP_SCAN not supported (needs Linux 6.7)";
            return -1;
        }
    }
    ww.mode = WW_UFFD;
    return 0;
}

/* ---- softdirty backend ---- */
static int sd_read(PVOID *addrs, ULONG_PTR *count, ULONG_PTR cap) {
    size_t npages = ww.size / GWWW_PAGE, i;
    ssize_t want = (ssize_t) (npages * 8), got;
    ULONG_PTR n = 0;
    got = pread(ww.pm_fd, ww.sd_buf, (size_t) want, (off_t) ((ww.base / GWWW_PAGE) * 8));
    if (got != want) {
        return -1;
    }
    for (i = 0; i < npages; ++i) {
        if ((ww.sd_buf[i] >> 55) & 1u) {
            if (addrs != NULL && n < cap) {
                addrs[n] = (PVOID) (ww.base + i * GWWW_PAGE);
            }
            ++n;
        }
    }
    *count = n;
    return 0;
}

static int sd_clear(void) {
    return write(ww.cr_fd, "4", 1) == 1 ? 0 : -1;
}

static int sd_start(void) {
    ww.pm_fd = open("/proc/self/pagemap", O_RDONLY | O_CLOEXEC);
    ww.cr_fd = open("/proc/self/clear_refs", O_WRONLY | O_CLOEXEC);
    if (ww.pm_fd < 0 || ww.cr_fd < 0) {
        ww.why = "cannot open /proc/self/pagemap or clear_refs";
        return -1;
    }
    ww.sd_buf = (uint64_t *) malloc(ww.size / GWWW_PAGE * 8);
    if (ww.sd_buf == NULL || sd_clear() != 0) {
        ww.why = "clear_refs refused (no CONFIG_MEM_SOFT_DIRTY?)";
        return -1;
    }
    ww.mode = WW_SOFTDIRTY;
    return 0;
}

/* ---- the Win32 surface ---- */
UINT GetWriteWatch(DWORD flags, PVOID base, size_t size, PVOID *addrs, ULONG_PTR *count, DWORD *granularity) {
    uintptr_t b = (uintptr_t) base, e = b + size;
    ULONG_PTR cap = count != NULL ? *count : 0, n = 0;
    int rc = -1;
    if (granularity != NULL) {
        *granularity = GWWW_PAGE;
    }
    if (ww.mode == WW_OFF || count == NULL || b < ww.base || e > ww.base + ww.size || (b % GWWW_PAGE) != 0) {
        return 1;
    }
    if (ww.mode == WW_UFFD) {
        rc = uffd_scan(b, e, (flags & WRITE_WATCH_FLAG_RESET) != 0, addrs, &n, cap);
    } else {
        /* the whole tracked range is read; the caller's sub-range is filtered by the address checks below */
        ULONG_PTR all = 0;
        rc = sd_read(NULL, &all, 0);
        if (rc == 0) {
            if (b == ww.base && e == ww.base + ww.size) {
                rc = sd_read(addrs, &n, cap);
            } else {
                size_t i0 = (b - ww.base) / GWWW_PAGE, i1 = (e - ww.base) / GWWW_PAGE, i;
                for (i = i0; i < i1; ++i) {
                    if ((ww.sd_buf[i] >> 55) & 1u) {
                        if (addrs != NULL && n < cap) {
                            addrs[n] = (PVOID) (ww.base + i * GWWW_PAGE);
                        }
                        ++n;
                    }
                }
            }
        }
        if (rc == 0 && (flags & WRITE_WATCH_FLAG_RESET) != 0) {
            rc = sd_clear();
        }
    }
    if (rc != 0 || n > cap) {
        return 1; /* "cannot tell": the caller then treats everything as written */
    }
    *count = n;
    return 0;
}

UINT ResetWriteWatch(PVOID base, size_t size) {
    ULONG_PTR c = 0;
    if (ww.mode == WW_UFFD) {
        return uffd_scan((uintptr_t) base, (uintptr_t) base + size, 1, NULL, &c, 0) == 0 ? 0 : 1;
    }
    if (ww.mode == WW_SOFTDIRTY) {
        return sd_clear() == 0 ? 0 : 1;
    }
    return 1;
}

/* ---- self-test: every claim the snapshot code leans on ---- */
static int ww_selftest(void) {
    volatile uint8_t *m = (volatile uint8_t *) ww.base;
    PVOID *got = (PVOID *) malloc(ww.size / GWWW_PAGE * sizeof(PVOID));
    ULONG_PTR cnt;
    DWORD gran;
    int ok = 0, fd;
    static const size_t pa = 5, pb = 100, pc = 9;
    uint8_t saved[3];
    if (got == NULL) {
        return 0;
    }
    saved[0] = m[pa * GWWW_PAGE + 17];
    saved[1] = m[pb * GWWW_PAGE + 33];
    saved[2] = m[pc * GWWW_PAGE + 1];
#define WWCHK(cond, msg) do { if (!(cond)) { ww.why = msg; goto done; } } while (0)
    /* 1. two user writes are reported, and only those */
    cnt = ww.size / GWWW_PAGE;
    WWCHK(GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) ww.base, ww.size, got, &cnt, &gran) == 0, "self-test: first poll failed");
    m[pa * GWWW_PAGE + 17] = (uint8_t) (saved[0] + 1);
    m[pb * GWWW_PAGE + 33] = (uint8_t) (saved[1] + 1);
    cnt = ww.size / GWWW_PAGE;
    WWCHK(GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) ww.base, ww.size, got, &cnt, &gran) == 0, "self-test: poll 2 failed");
    {
        int fa = 0, fb = 0;
        ULONG_PTR i;
        for (i = 0; i < cnt; ++i) {
            fa |= (uintptr_t) got[i] == ww.base + pa * GWWW_PAGE;
            fb |= (uintptr_t) got[i] == ww.base + pb * GWWW_PAGE;
        }
        WWCHK(fa && fb, "self-test: a written page was not reported");
        WWCHK(cnt <= 8, "self-test: far too many pages reported for two writes");
    }
    /* 2. nothing written since the reset: nothing reported */
    cnt = ww.size / GWWW_PAGE;
    WWCHK(GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) ww.base, ww.size, got, &cnt, &gran) == 0, "self-test: poll 3 failed");
    WWCHK(cnt == 0, "self-test: pages reported after a reset with no writes");
    /* 3. a page written before (so already populated) is reported again after the reset */
    m[pa * GWWW_PAGE + 17] = saved[0];
    cnt = ww.size / GWWW_PAGE;
    WWCHK(GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) ww.base, ww.size, got, &cnt, &gran) == 0, "self-test: poll 4 failed");
    WWCHK(cnt >= 1 && (uintptr_t) got[0] == ww.base + pa * GWWW_PAGE, "self-test: a rewritten page was not reported");
    m[pb * GWWW_PAGE + 33] = saved[1];
    /* 4. a kernel write (read(2) into the range, as the DVD shim does) is reported too */
    fd = open("/proc/self/exe", O_RDONLY | O_CLOEXEC);
    if (fd >= 0) {
        cnt = ww.size / GWWW_PAGE;
        (void) GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) ww.base, ww.size, got, &cnt, &gran);
        if (pread(fd, (void *) (ww.base + pc * GWWW_PAGE), 64, 0) == 64) {
            cnt = ww.size / GWWW_PAGE;
            WWCHK(GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) ww.base, ww.size, got, &cnt, &gran) == 0, "self-test: poll 5 failed");
            WWCHK(cnt >= 1 && (uintptr_t) got[0] == ww.base + pc * GWWW_PAGE, "self-test: a kernel write was not reported");
        }
        close(fd);
    }
    ok = 1;
done:
    m[pa * GWWW_PAGE + 17] = saved[0];
    m[pb * GWWW_PAGE + 33] = saved[1];
    m[pc * GWWW_PAGE + 1] = saved[2];
    memset((void *) (ww.base + pc * GWWW_PAGE), 0, 64);
    m[pc * GWWW_PAGE + 1] = saved[2];
    cnt = ww.size / GWWW_PAGE;
    (void) GetWriteWatch(WRITE_WATCH_FLAG_RESET, (PVOID) ww.base, ww.size, got, &cnt, &gran);
    free(got);
    return ok;
}

static void ww_close(void) {
    if (ww.pm_fd > 0) close(ww.pm_fd);
    if (ww.cr_fd > 0) close(ww.cr_fd);
    if (ww.uffd > 0) close(ww.uffd);
    free(ww.sd_buf);
    memset(&ww, 0, sizeof ww);
    ww.pm_fd = ww.cr_fd = ww.uffd = -1;
}

/* Called by VirtualAlloc on a fresh, untouched MEM_WRITE_WATCH mapping. Returns 1 if a backend works. */
int gw_linux_writewatch_start(void *base, size_t size) {
    const char *env = getenv("MELEE_WRITEWATCH");
    int want_uffd = 1, want_sd = 1;
    if (env != NULL && env[0] != '\0') {
        if (strcmp(env, "off") == 0 || strcmp(env, "0") == 0) {
            want_uffd = want_sd = 0;
        } else if (strcmp(env, "uffd") == 0) {
            want_sd = 0;
        } else if (strcmp(env, "softdirty") == 0) {
            want_uffd = 0;
        }
    }
    memset(&ww, 0, sizeof ww);
    ww.pm_fd = ww.cr_fd = ww.uffd = -1;
    ww.base = (uintptr_t) base;
    ww.size = size;
    if (want_uffd) {
        if (uffd_start() == 0 && ww_selftest()) {
            goto up;
        }
        fprintf(stderr, "melee-pc: write-watch: uffd backend unavailable (%s)\n", ww.why != NULL ? ww.why : "?");
        ww_close();
        ww.base = (uintptr_t) base;
        ww.size = size;
    }
    if (want_sd) {
        if (sd_start() == 0 && ww_selftest()) {
            goto up;
        }
        fprintf(stderr, "melee-pc: write-watch: soft-dirty backend unavailable (%s)\n", ww.why != NULL ? ww.why : "?");
        ww_close();
    }
    return 0;
up:
    return 1;
}
