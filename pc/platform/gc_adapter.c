/* Raw GameCube controller adapter input (Nintendo WUP-028 and compatibles such as the Mayflash
 * in Wii U / Switch mode), read directly over WinUSB.
 *
 * Aurora can already see the adapter through SDL3's HIDAPI GameCube driver, but everything it
 * reports has passed through SDL's gamepad abstraction: deadzones, axis rescaling, trigger
 * emulation and a button mapping table. That mapping is wrong for this adapter in practice
 * (A/B/X/Y and the d-pad come through scrambled), and Melee is calibrated against the console's
 * own analog ranges, so the abstraction also costs precision exactly where it matters --
 * shield-drop angles, wavedash notches, lightshield depth. This reads the adapter's report bytes
 * and builds PADStatus the way the console does, with no intermediate mapping.
 *
 * Two transports are supported. The Windows HID class driver is the usual case and needs no
 * setup at all -- it hands back the adapter's own report bytes, which is how SDL can see the
 * device in the first place. WinUSB is tried first for machines where the adapter has been bound
 * with Zadig, as older Dolphin setups require. Both deliver identical reports, so the decode is
 * shared. If neither opens, every entry point reports "no adapter" and the caller falls back to
 * Aurora's SDL path.
 *
 * Protocol (as implemented by Dolphin's GCAdapter):
 *   - write a single 0x13 byte to start polling
 *   - interrupt IN delivers 37 bytes: a 0x21 tag followed by 4 ports x 9 bytes
 *   - per port: [0] status, [1] buttons low, [2] buttons high, [3] stickX, [4] stickY,
 *               [5] substickX, [6] substickY, [7] triggerLeft, [8] triggerRight
 *   - status high nibble: 1 = wired, 2 = wireless; 0 = nothing plugged into that adapter port
 *   - rumble: write {0x11, p0, p1, p2, p3}
 */
#include "gw.h"

#include <dolphin/pad.h>

#include <string.h>
#include <stdlib.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winusb.h>
#include <setupapi.h>
#include <hidsdi.h>
#include <ctype.h>

#define GC_ADAPTER_VID 0x057E
#define GC_ADAPTER_PID 0x0337

/* Endpoint addresses: read from the interface descriptor when the adapter opens (Dolphin does the
 * same), so a clone that numbers its pipes differently still works. These are the official
 * adapter's, kept as the fallback. */
static UCHAR gw_gc_ep_in = 0x81;
static UCHAR gw_gc_ep_out = 0x02;
#define GC_EP_IN gw_gc_ep_in
#define GC_EP_OUT gw_gc_ep_out

#define GC_PAYLOAD_SIZE 37
#define GC_PORTS 4

/* Adapter report bits. This is the adapter's own encoding, not the console's PADStatus bits. */
#define GC_BTN_A 0x01
#define GC_BTN_B 0x02
#define GC_BTN_X 0x04
#define GC_BTN_Y 0x08
#define GC_BTN_DLEFT 0x10
#define GC_BTN_DRIGHT 0x20
#define GC_BTN_DDOWN 0x40
#define GC_BTN_DUP 0x80

#define GC_BTN2_START 0x01
#define GC_BTN2_Z 0x02
#define GC_BTN2_R 0x04
#define GC_BTN2_L 0x08

/* {A5DCBF10-6530-11D2-901F-00C04FB951ED}: the device interface Windows exposes for a USB device
 * bound to WinUSB. Declared here so the shim does not need the SDK's usbiodef.h. */
static const GUID gw_guid_devinterface_usb_device = {
    0xA5DCBF10, 0x6530, 0x11D2, { 0x90, 0x1F, 0x00, 0xC0, 0x4F, 0xB9, 0x51, 0xED }
};

static HANDLE gw_gc_dev = INVALID_HANDLE_VALUE;
static WINUSB_INTERFACE_HANDLE gw_gc_usb;
static int gw_gc_ready;
static int gw_gc_tried;

/* Which transport is driving the adapter. The device is normally left on the Windows HID driver
 * (which is how SDL can see it at all); WinUSB only applies on machines where it has been bound
 * with Zadig, as older Dolphin setups require. Both deliver the same report bytes. */
enum { GW_GC_NONE = 0, GW_GC_WINUSB, GW_GC_HID };
static int gw_gc_backend;

/* HID transport state. Reads are overlapped so a frame is never blocked waiting on the device. */
static HANDLE gw_gc_hid = INVALID_HANDLE_VALUE;
static HANDLE gw_gc_hid_event;
static OVERLAPPED gw_gc_hid_ov;
static int gw_gc_hid_pending;
static unsigned char gw_gc_hid_buf[64];
static ULONG gw_gc_hid_in_len;
static ULONG gw_gc_hid_out_len;

/* Neutral stick readings captured the first time a port reports data. The console calibrates
 * against the origin the controller reports at power-on; the adapter gives raw 0..255, so the
 * first sample stands in for that origin. */
static unsigned char gw_gc_origin[GC_PORTS][4];
static int gw_gc_have_origin[GC_PORTS];

/* Resting trigger positions, and any button bits found held at rest. Worn or plugged triggers
 * sit well off zero -- one pinned at 255 reads as permanently holding L, which the game sees as
 * a held shield. Calibrating against the resting value and rescaling the travel that is left
 * makes such a controller usable; a trigger with no travel at all is reported as never pressed
 * rather than always pressed, which is the safer failure. The same sampling catches a switch
 * stuck closed and masks that bit. */
static unsigned char gw_gc_trig_rest[GC_PORTS][2];
static unsigned char gw_gc_btn_stuck[GC_PORTS][2];
static volatile LONG gw_gc_recal;

#define GW_GC_TRIG_DEAD 200 /* resting value above which a trigger has no usable travel left */

/* The adapter is read on its own thread. A blocking USB read on the game thread costs a frame:
 * the device only produces a packet when it has one, so a timeout-bounded read in the frame loop
 * stalls for the whole timeout whenever the adapter is idle, which shows up directly as input
 * lag. The reader thread parks in that blocking read instead, and the game thread only copies the
 * most recent packet out under the lock. */
static unsigned char gw_gc_payload[GC_PAYLOAD_SIZE];
static int gw_gc_payload_valid;

static CRITICAL_SECTION gw_gc_lock;
static int gw_gc_lock_ready;
static HANDLE gw_gc_thread;
static volatile LONG gw_gc_quit;
static volatile LONG gw_gc_lost;

void gw_gc_adapter_shutdown(void);
static DWORD WINAPI gw_gc_reader(LPVOID arg);

/* shim_pad.c: the level-1 pad log (rate-capped; MELEE_PAD_DIAG / settings pad_diag). */
extern void gw_pad_log(const char *fmt, ...);

/* Open/close are serialised: the hotplug scanner opens from its own thread, while focus loss
 * (gw_gc_adapter_suspend, game thread) and a lost device (gw_gc_poll, game thread) close. */
static SRWLOCK gw_gc_init_srw = SRWLOCK_INIT;

/* Suspended = the game window is in the background and has let go of the adapter so another
 * program (Dolphin, a second copy of the game) can claim it. The scanner leaves it alone until
 * gw_gc_adapter_resume. */
static volatile LONG gw_gc_suspended;
static volatile LONG gw_gc_resume_pending; /* the next scanner open is a resume: keep calibration */
static HANDLE gw_gc_scan_wake;             /* auto-reset: wakes the scanner early */
static volatile LONG gw_gc_reports_total;  /* every good report since start (never reset) */
static int (*gw_gc_test_open)(void);
static int gw_gc_open_quiet;               /* scanner retry: skip the per-attempt failure lines */

/* MELEE_INPUT_PROFILE (shim_vi.c): when each report arrived, and the spacing between reports.
 * The official adapter reports at 125 Hz (8 ms); an overclocked one (HIDUSBF / a patched
 * bInterval) at up to 1000 Hz. Whatever the rate, the game takes the newest report, so this is
 * what says how old that report can be. Written by the reader thread only. */
#define GW_GC_IVL_BUCKETS 9
static const double gw_gc_ivl_edges[GW_GC_IVL_BUCKETS] = { 0.75, 1.5, 3.0, 5.0, 7.0, 9.0, 12.0, 20.0, 1e9 };
static volatile LONGLONG gw_gc_report_qpc;
static LONGLONG gw_gc_prev_qpc;
static unsigned gw_gc_ivl_hist[GW_GC_IVL_BUCKETS];
static unsigned gw_gc_ivl_count, gw_gc_ivl_changed;
static double gw_gc_ivl_min = 1e9, gw_gc_ivl_max;
static unsigned char gw_gc_prev_payload[GC_PAYLOAD_SIZE];

/* The reader calls this, under the lock, for every good report. */
static void gw_gc_stamp(const unsigned char *payload) {
  static double freq;
  LARGE_INTEGER now;
  QueryPerformanceCounter(&now);
  if (freq == 0.0) {
    LARGE_INTEGER f;
    QueryPerformanceFrequency(&f);
    freq = (double)f.QuadPart;
  }
  if (gw_gc_prev_qpc != 0) {
    const double ms = (double)(now.QuadPart - gw_gc_prev_qpc) * 1000.0 / freq;
    int b;
    for (b = 0; b < GW_GC_IVL_BUCKETS; ++b) {
      if (ms < gw_gc_ivl_edges[b]) {
        ++gw_gc_ivl_hist[b];
        break;
      }
    }
    if (ms < gw_gc_ivl_min) gw_gc_ivl_min = ms;
    if (ms > gw_gc_ivl_max) gw_gc_ivl_max = ms;
    ++gw_gc_ivl_count;
    if (memcmp(payload, gw_gc_prev_payload, GC_PAYLOAD_SIZE) != 0) {
      ++gw_gc_ivl_changed;
    }
  }
  memcpy(gw_gc_prev_payload, payload, GC_PAYLOAD_SIZE);
  gw_gc_prev_qpc = now.QuadPart;
  InterlockedExchange64(&gw_gc_report_qpc, now.QuadPart);
  InterlockedIncrement(&gw_gc_reports_total);
}

/* Reports received so far (monotonic; the 10 s pad summary turns it into a rate without
 * disturbing MELEE_INPUT_PROFILE's resetting statistics). */
unsigned gw_gc_adapter_report_count(void) {
  return (unsigned)InterlockedCompareExchange(&gw_gc_reports_total, 0, 0);
}

/* QPC time the newest report arrived (0: none yet). */
long long gw_gc_adapter_report_qpc(void) {
  return (long long)InterlockedCompareExchange64(&gw_gc_report_qpc, 0, 0);
}

/* Copy out and reset the interval statistics. Racy against the reader by design: diagnostics. */
void gw_gc_adapter_take_stats(unsigned *hist9, unsigned *count, unsigned *changed, double *min_ms,
                              double *max_ms) {
  int b;
  for (b = 0; b < GW_GC_IVL_BUCKETS; ++b) {
    hist9[b] = gw_gc_ivl_hist[b];
    gw_gc_ivl_hist[b] = 0;
  }
  *count = gw_gc_ivl_count;
  *changed = gw_gc_ivl_changed;
  *min_ms = gw_gc_ivl_count ? gw_gc_ivl_min : 0.0;
  *max_ms = gw_gc_ivl_max;
  gw_gc_ivl_count = gw_gc_ivl_changed = 0;
  gw_gc_ivl_min = 1e9;
  gw_gc_ivl_max = 0.0;
}

static void gw_gc_close(void) {
  if (gw_gc_usb != NULL) {
    WinUsb_Free(gw_gc_usb);
    gw_gc_usb = NULL;
  }
  if (gw_gc_dev != INVALID_HANDLE_VALUE) {
    CloseHandle(gw_gc_dev);
    gw_gc_dev = INVALID_HANDLE_VALUE;
  }
  if (gw_gc_hid != INVALID_HANDLE_VALUE) {
    CloseHandle(gw_gc_hid);
    gw_gc_hid = INVALID_HANDLE_VALUE;
  }
  if (gw_gc_hid_event != NULL) {
    CloseHandle(gw_gc_hid_event);
    gw_gc_hid_event = NULL;
  }
  gw_gc_hid_pending = 0;
  gw_gc_backend = GW_GC_NONE;
  gw_gc_ready = 0;
}

/* Open the adapter through the Windows HID class driver. This is the path that works on a stock
 * machine with no Zadig step: the HID driver hands back the adapter's own report bytes, exactly
 * as SDL's HIDAPI GameCube driver reads them, so the decode below is unchanged. */
static int gw_gc_open_hid(void) {
  GUID hidGuid;
  HDEVINFO info;
  SP_DEVICE_INTERFACE_DATA ifdata;
  DWORD index;
  int found = 0;

  HidD_GetHidGuid(&hidGuid);
  info = SetupDiGetClassDevsA(&hidGuid, NULL, NULL, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE);
  if (info == INVALID_HANDLE_VALUE) {
    return 0;
  }

  ZeroMemory(&ifdata, sizeof(ifdata));
  ifdata.cbSize = sizeof(ifdata);

  for (index = 0; SetupDiEnumDeviceInterfaces(info, NULL, &hidGuid, index, &ifdata); ++index) {
    DWORD needed = 0;
    SP_DEVICE_INTERFACE_DETAIL_DATA_A *detail;
    HANDLE h;

    SetupDiGetDeviceInterfaceDetailA(info, &ifdata, NULL, 0, &needed, NULL);
    if (needed == 0) {
      continue;
    }
    detail = (SP_DEVICE_INTERFACE_DETAIL_DATA_A *)LocalAlloc(LMEM_FIXED, needed);
    if (detail == NULL) {
      continue;
    }
    detail->cbSize = sizeof(SP_DEVICE_INTERFACE_DETAIL_DATA_A);
    if (!SetupDiGetDeviceInterfaceDetailA(info, &ifdata, detail, needed, NULL, NULL)) {
      LocalFree(detail);
      continue;
    }

    h = CreateFileA(detail->DevicePath, GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED, NULL);
    if (h != INVALID_HANDLE_VALUE) {
      HIDD_ATTRIBUTES attr;
      ZeroMemory(&attr, sizeof(attr));
      attr.Size = sizeof(attr);
      if (HidD_GetAttributes(h, &attr) && attr.VendorID == GC_ADAPTER_VID &&
          attr.ProductID == GC_ADAPTER_PID) {
        PHIDP_PREPARSED_DATA pp = NULL;
        if (HidD_GetPreparsedData(h, &pp)) {
          HIDP_CAPS caps;
          if (HidP_GetCaps(pp, &caps) == HIDP_STATUS_SUCCESS) {
            gw_gc_hid_in_len = caps.InputReportByteLength;
            gw_gc_hid_out_len = caps.OutputReportByteLength;
            if (gw_gc_hid_in_len > 0 && gw_gc_hid_in_len <= sizeof(gw_gc_hid_buf)) {
              gw_gc_hid = h;
              found = 1;
            }
          }
          HidD_FreePreparsedData(pp);
        }
      }
      if (!found) {
        CloseHandle(h);
      }
    }
    LocalFree(detail);
    if (found) {
      break;
    }
  }

  SetupDiDestroyDeviceInfoList(info);
  return found;
}

/* Copy a report into the payload buffer. Windows prefixes HID reports with a report-ID byte, so
 * the 0x21 tag may sit at offset 0 or 1 depending on whether the device uses numbered reports. */
static void gw_gc_accept_report(const unsigned char *buf, ULONG len) {
  ULONG off;
  for (off = 0; off + GC_PAYLOAD_SIZE <= len; ++off) {
    if (buf[off] == 0x21) {
      memcpy(gw_gc_payload, buf + off, GC_PAYLOAD_SIZE);
      gw_gc_payload_valid = 1;
      return;
    }
  }
}

/* Non-blocking overlapped read: issue once, harvest whenever it completes. */
static int gw_gc_poll_hid(void) {
  int guard = 0;

  while (guard++ < 8) {
    DWORD got = 0;

    if (!gw_gc_hid_pending) {
      ZeroMemory(&gw_gc_hid_ov, sizeof(gw_gc_hid_ov));
      gw_gc_hid_ov.hEvent = gw_gc_hid_event;
      ResetEvent(gw_gc_hid_event);
      if (ReadFile(gw_gc_hid, gw_gc_hid_buf, gw_gc_hid_in_len, &got, &gw_gc_hid_ov)) {
        gw_gc_accept_report(gw_gc_hid_buf, got);
        continue;
      }
      if (GetLastError() != ERROR_IO_PENDING) {
        gw_log("gw: gc adapter: HID read failed (error %lu); reverting to the SDL pad path",
               (unsigned long)GetLastError());
        gw_gc_close();
        return 0;
      }
      gw_gc_hid_pending = 1;
    }

    if (WaitForSingleObject(gw_gc_hid_event, 0) != WAIT_OBJECT_0) {
      break;
    }
    if (GetOverlappedResult(gw_gc_hid, &gw_gc_hid_ov, &got, FALSE)) {
      gw_gc_accept_report(gw_gc_hid_buf, got);
    }
    gw_gc_hid_pending = 0;
  }

  return gw_gc_payload_valid;
}

/* Case-insensitive substring search; the device path's VID/PID casing is not guaranteed. */
static const char *gw_gc_stristr(const char *hay, const char *needle) {
  size_t n = strlen(needle);
  if (n == 0) {
    return hay;
  }
  for (; *hay != '\0'; ++hay) {
    size_t i = 0;
    while (i < n && hay[i] != '\0' &&
           (char)toupper((unsigned char)hay[i]) == (char)toupper((unsigned char)needle[i])) {
      ++i;
    }
    if (i == n) {
      return hay;
    }
  }
  return NULL;
}

/* Walk the WinUSB device interfaces looking for the adapter's VID/PID in the device path. */
static int gw_gc_open(void) {
  HDEVINFO info;
  SP_DEVICE_INTERFACE_DATA ifdata;
  DWORD index;
  char want[32];
  int found = 0;

  wsprintfA(want, "vid_%04x&pid_%04x", GC_ADAPTER_VID, GC_ADAPTER_PID);

  info = SetupDiGetClassDevsA(&gw_guid_devinterface_usb_device, NULL, NULL,
                              DIGCF_PRESENT | DIGCF_DEVICEINTERFACE);
  if (info == INVALID_HANDLE_VALUE) {
    return 0;
  }

  ZeroMemory(&ifdata, sizeof(ifdata));
  ifdata.cbSize = sizeof(ifdata);

  for (index = 0;
       SetupDiEnumDeviceInterfaces(info, NULL, &gw_guid_devinterface_usb_device, index, &ifdata);
       ++index) {
    DWORD needed = 0;
    SP_DEVICE_INTERFACE_DETAIL_DATA_A *detail;

    SetupDiGetDeviceInterfaceDetailA(info, &ifdata, NULL, 0, &needed, NULL);
    if (needed == 0) {
      continue;
    }
    detail = (SP_DEVICE_INTERFACE_DETAIL_DATA_A *)LocalAlloc(LMEM_FIXED, needed);
    if (detail == NULL) {
      continue;
    }
    detail->cbSize = sizeof(SP_DEVICE_INTERFACE_DETAIL_DATA_A);
    if (SetupDiGetDeviceInterfaceDetailA(info, &ifdata, detail, needed, NULL, NULL) &&
        gw_gc_stristr(detail->DevicePath, want) != NULL) {
      gw_gc_dev = CreateFileA(detail->DevicePath, GENERIC_READ | GENERIC_WRITE,
                              FILE_SHARE_READ | FILE_SHARE_WRITE, NULL, OPEN_EXISTING,
                              FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED, NULL);
      if (gw_gc_dev == INVALID_HANDLE_VALUE) {
        /* ERROR_ACCESS_DENIED here almost always means another process (typically SDL, which
         * opens this adapter through its own HIDAPI GameCube driver) already holds the device. */
        if (!gw_gc_open_quiet) gw_pad_log("gw: gc adapter: found %s but CreateFile failed (error %lu%s)",
                   detail->DevicePath, (unsigned long)GetLastError(),
                   GetLastError() == ERROR_ACCESS_DENIED ? ": another program holds it" : "");
      } else if (WinUsb_Initialize(gw_gc_dev, &gw_gc_usb)) {
        found = 1;
      } else {
        gw_log("gw: gc adapter: WinUsb_Initialize failed (error %lu)",
               (unsigned long)GetLastError());
        CloseHandle(gw_gc_dev);
        gw_gc_dev = INVALID_HANDLE_VALUE;
      }
    }
    LocalFree(detail);
    if (found) {
      break;
    }
  }

  SetupDiDestroyDeviceInfoList(info);
  return found;
}

/* Diagnostic: list every device-interface path that mentions this vendor, under both the USB
 * and HID interface classes. A Zadig-installed WinUSB device does not necessarily register under
 * GUID_DEVINTERFACE_USB_DEVICE -- it exposes whatever DeviceInterfaceGUIDs the generated INF set
 * -- so when the open fails this says which class the device is actually reachable through. */
static void gw_gc_dump_one_class(const GUID *guid, const char *label) {
  HDEVINFO info;
  SP_DEVICE_INTERFACE_DATA ifdata;
  DWORD index;
  int shown = 0;

  info = SetupDiGetClassDevsA(guid, NULL, NULL, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE);
  if (info == INVALID_HANDLE_VALUE) {
    gw_log("gw: gc adapter: %s enumeration unavailable (error %lu)", label,
           (unsigned long)GetLastError());
    return;
  }
  ZeroMemory(&ifdata, sizeof(ifdata));
  ifdata.cbSize = sizeof(ifdata);
  for (index = 0; SetupDiEnumDeviceInterfaces(info, NULL, guid, index, &ifdata); ++index) {
    DWORD needed = 0;
    SP_DEVICE_INTERFACE_DETAIL_DATA_A *detail;
    SetupDiGetDeviceInterfaceDetailA(info, &ifdata, NULL, 0, &needed, NULL);
    if (needed == 0) {
      continue;
    }
    detail = (SP_DEVICE_INTERFACE_DETAIL_DATA_A *)LocalAlloc(LMEM_FIXED, needed);
    if (detail == NULL) {
      continue;
    }
    detail->cbSize = sizeof(SP_DEVICE_INTERFACE_DETAIL_DATA_A);
    if (SetupDiGetDeviceInterfaceDetailA(info, &ifdata, detail, needed, NULL, NULL) &&
        gw_gc_stristr(detail->DevicePath, "vid_057e") != NULL && shown < 8) {
      ++shown;
      gw_log("gw: gc adapter: %s candidate: %s", label, detail->DevicePath);
    }
    LocalFree(detail);
  }
  SetupDiDestroyDeviceInfoList(info);
  if (shown == 0) {
    gw_log("gw: gc adapter: %s: nothing matching vid_057e (%lu interfaces scanned)", label,
           (unsigned long)index);
  }
}

static void gw_gc_dump_interfaces(void) {
  GUID hidGuid;
  gw_gc_dump_one_class(&gw_guid_devinterface_usb_device, "usb-class");
  HidD_GetHidGuid(&hidGuid);
  gw_gc_dump_one_class(&hidGuid, "hid-class");
}

static void gw_gc_start_reader(void) {
  DWORD tid = 0;
  if (!gw_gc_lock_ready) {
    InitializeCriticalSection(&gw_gc_lock);
    gw_gc_lock_ready = 1;
  }
  InterlockedExchange(&gw_gc_quit, 0);
  InterlockedExchange(&gw_gc_lost, 0);
  gw_gc_thread = CreateThread(NULL, 0, gw_gc_reader, NULL, 0, &tid);
  if (gw_gc_thread == NULL) {
    gw_log("gw: gc adapter: could not start the reader thread (error %lu)",
           (unsigned long)GetLastError());
  } else {
    /* Input latency matters more than throughput here; keep the reader ahead of the frame. */
    SetThreadPriority(gw_gc_thread, THREAD_PRIORITY_ABOVE_NORMAL);
  }
}

static void gw_gc_shutdown_locked(void) {
  InterlockedExchange(&gw_gc_quit, 1);
  if (gw_gc_thread != NULL) {
    /* The reader is parked in a timeout-bounded read, so it observes the flag promptly. */
    if (WaitForSingleObject(gw_gc_thread, 500) != WAIT_OBJECT_0) {
      if (gw_gc_backend == GW_GC_HID && gw_gc_hid != INVALID_HANDLE_VALUE) {
        CancelIoEx(gw_gc_hid, NULL);
      } else if (gw_gc_backend == GW_GC_WINUSB && gw_gc_usb != NULL) {
        WinUsb_AbortPipe(gw_gc_usb, GC_EP_IN);
      }
      WaitForSingleObject(gw_gc_thread, 500);
    }
    CloseHandle(gw_gc_thread);
    gw_gc_thread = NULL;
  }
  gw_gc_close();
  /* the last report belongs to the closed session; don't decode it again after a reopen */
  if (gw_gc_lock_ready) {
    EnterCriticalSection(&gw_gc_lock);
    gw_gc_payload_valid = 0;
    LeaveCriticalSection(&gw_gc_lock);
  }
}

void gw_gc_adapter_shutdown(void) {
  AcquireSRWLockExclusive(&gw_gc_init_srw);
  gw_gc_shutdown_locked();
  ReleaseSRWLockExclusive(&gw_gc_init_srw);
}

/* When no adapter is found: every USB VID:PID present, once per run, so a log from a player whose
 * adapter "isn't detected" says what IS plugged in (a clone with another PID, an adapter in PC
 * mode showing up as a generic HID gamepad, nothing at all). */
static void gw_gc_log_usb_ids_once(void) {
  static int done;
  HDEVINFO set;
  SP_DEVINFO_DATA info;
  char id[512];
  char seen[32][18];
  int nseen = 0;
  char line[32 * 19 + 1];
  DWORD i;
  if (done) {
    return;
  }
  done = 1;
  set = SetupDiGetClassDevsA(NULL, "USB", NULL, DIGCF_PRESENT | DIGCF_ALLCLASSES);
  if (set == INVALID_HANDLE_VALUE) {
    return;
  }
  info.cbSize = sizeof(info);
  for (i = 0; nseen < 32 && SetupDiEnumDeviceInfo(set, i, &info); ++i) {
    char *p, *v;
    int k;
    if (!SetupDiGetDeviceInstanceIdA(set, &info, id, sizeof(id), NULL)) {
      continue;
    }
    for (p = id; *p != '\0'; ++p) *p = (char)toupper((unsigned char)*p);
    v = strstr(id, "VID_");
    if (v == NULL || strlen(v) < 17 || strncmp(v + 8, "&PID_", 5) != 0) {
      continue;
    }
    for (k = 0; k < nseen; ++k) {
      if (strncmp(seen[k], v + 4, 4) == 0 && strncmp(seen[k] + 5, v + 13, 4) == 0) break;
    }
    if (k < nseen) {
      continue;
    }
    memcpy(seen[nseen], v + 4, 4);
    seen[nseen][4] = ':';
    memcpy(seen[nseen] + 5, v + 13, 4);
    seen[nseen][9] = '\0';
    ++nseen;
  }
  SetupDiDestroyDeviceInfoList(set);
  line[0] = '\0';
  for (i = 0; i < (DWORD)nseen; ++i) {
    strcat(line, i ? " " : "");
    strcat(line, seen[i]);
  }
  gw_log("gw: gc adapter: not found (057E:0337); USB devices present (VID:PID): %s",
         nseen ? line : "(none listed)");
}

int gw_gc_adapter_device_plugged(void);
static int gw_gc_init_locked(void);
int gw_gc_adapter_init(void) {
  int r;
  AcquireSRWLockExclusive(&gw_gc_init_srw);
  r = gw_gc_init_locked();
  ReleaseSRWLockExclusive(&gw_gc_init_srw);
  return r;
}

static int gw_gc_open_locked(void);
/* Open under gw_gc_init_srw. Whatever the caller, an open never survives a suspend: suspend sets
 * the flag under this same lock, so checking it again after the open (which can take a while -
 * device enumeration, the reader start) closes an adapter a background window must not hold. */
static int gw_gc_init_locked(void) {
  int ok = gw_gc_open_locked();
  if (ok && InterlockedCompareExchange(&gw_gc_suspended, 0, 0) != 0) {
    gw_log("gw: gc adapter: opened while suspended - closing it again");
    gw_gc_shutdown_locked();
    return 0;
  }
  return ok;
}

static int gw_gc_open_locked(void) {
  unsigned char start = 0x13;
  ULONG written = 0;
  ULONG timeout = 20;
  ULONG raw = 1;

  if (gw_gc_tried) {
    return gw_gc_ready;
  }
  gw_gc_tried = 1;
  if (InterlockedCompareExchange(&gw_gc_suspended, 0, 0) != 0) {
    return 0; /* in the background: leave the adapter to whoever wants it */
  }
  if (gw_gc_test_open != NULL) {
    return gw_gc_test_open(); /* tests: a stand-in device (gw_gc_adapter_tests_register) */
  }

  if (gw_gc_open()) {
    gw_gc_backend = GW_GC_WINUSB;
  } else if (gw_gc_open_hid()) {
    gw_gc_backend = GW_GC_HID;
  } else {
    static int dumped;
    if (!gw_gc_open_quiet) {
      gw_pad_log("gw: gc adapter: no WUP-028 adapter found on either WinUSB or HID; using the SDL "
                 "pad path instead.");
    }
    if (!dumped) {
      dumped = 1;
      gw_gc_dump_interfaces();
      if (!gw_gc_adapter_device_plugged()) {
        gw_gc_log_usb_ids_once();
      }
    }
    return 0;
  }

  if (gw_gc_backend == GW_GC_HID) {
    unsigned char out[64];
    DWORD written = 0;
    ULONG len = gw_gc_hid_out_len;

    gw_gc_hid_event = CreateEventA(NULL, TRUE, FALSE, NULL);
    if (gw_gc_hid_event == NULL) {
      gw_gc_close();
      return 0;
    }
    /* Start polling. The report is padded to the descriptor's output length, with the leading
     * byte reserved for the report ID (0 on this device). */
    if (len == 0 || len > sizeof(out)) {
      len = 2;
    }
    ZeroMemory(out, sizeof(out));
    out[0] = 0x00;
    out[1] = 0x13;
    if (!HidD_SetOutputReport(gw_gc_hid, out, len)) {
      OVERLAPPED ov;
      ZeroMemory(&ov, sizeof(ov));
      ov.hEvent = gw_gc_hid_event;
      ResetEvent(gw_gc_hid_event);
      if (!WriteFile(gw_gc_hid, out, len, &written, &ov) && GetLastError() != ERROR_IO_PENDING) {
        gw_log("gw: gc adapter: HID start command failed (error %lu)",
               (unsigned long)GetLastError());
      } else {
        WaitForSingleObject(gw_gc_hid_event, 50);
      }
    }
    gw_gc_ready = 1;
    gw_gc_start_reader();
    gw_log("gw: gc adapter: opened over HID (vid %04X pid %04X, report %lu bytes), reading raw "
           "reports", GC_ADAPTER_VID, GC_ADAPTER_PID, (unsigned long)gw_gc_hid_in_len);
    return 1;
  }

  /* Find the interrupt pipes from the descriptor instead of assuming the official adapter's. */
  {
    USB_INTERFACE_DESCRIPTOR ifd;
    if (WinUsb_QueryInterfaceSettings(gw_gc_usb, 0, &ifd)) {
      UCHAR i;
      for (i = 0; i < ifd.bNumEndpoints; ++i) {
        WINUSB_PIPE_INFORMATION pipe;
        if (WinUsb_QueryPipe(gw_gc_usb, 0, i, &pipe) && pipe.PipeType == UsbdPipeTypeInterrupt) {
          if (pipe.PipeId & 0x80) {
            gw_gc_ep_in = pipe.PipeId;
          } else {
            gw_gc_ep_out = pipe.PipeId;
          }
        }
      }
    }
  }

  /* A run that ended without closing the adapter (a crash, a kill) can leave a pipe halted or
   * mid-transfer: the next open succeeds but reads deliver nothing until the adapter is replugged.
   * Resetting both pipes clears the halt and the data toggle - melee-unlocked's gc_adapter.cpp
   * does the same with libusb_clear_halt (GPL-2.0-or-later, Hero88go). */
  if (!WinUsb_ResetPipe(gw_gc_usb, GC_EP_IN)) {
    gw_pad_log("gw: gc adapter: reset of IN pipe 0x%02X failed (error %lu)", GC_EP_IN,
               (unsigned long)GetLastError());
  }
  if (!WinUsb_ResetPipe(gw_gc_usb, GC_EP_OUT)) {
    gw_pad_log("gw: gc adapter: reset of OUT pipe 0x%02X failed (error %lu)", GC_EP_OUT,
               (unsigned long)GetLastError());
  }

  /* Short read timeout: the pad is polled once per frame from the game thread, and a stalled
   * read must never hold up the frame. RAW_IO keeps WinUSB from buffering partial packets. */
  WinUsb_SetPipePolicy(gw_gc_usb, GC_EP_IN, PIPE_TRANSFER_TIMEOUT, sizeof(timeout), &timeout);
  (void)raw;
  WinUsb_SetPipePolicy(gw_gc_usb, GC_EP_OUT, PIPE_TRANSFER_TIMEOUT, sizeof(timeout), &timeout);

  if (!WinUsb_WritePipe(gw_gc_usb, GC_EP_OUT, &start, 1, &written, NULL)) {
    gw_log("gw: gc adapter: found, but the start command failed (error %lu)",
           (unsigned long)GetLastError());
    gw_gc_close();
    return 0;
  }

  gw_gc_ready = 1;
  gw_gc_start_reader();
  gw_log("gw: gc adapter: opened over WinUSB (vid %04X pid %04X), reading raw reports",
         GC_ADAPTER_VID, GC_ADAPTER_PID);
  return 1;
}

int gw_gc_adapter_present(void) { return gw_gc_ready; }

/* Is a WUP-028 (057E:0337) plugged in right now, whatever driver it has? A SetupAPI walk of the
 * present USB devices - cheap, and it opens nothing, so it can't disturb another program. */
int gw_gc_adapter_device_plugged(void) {
  HDEVINFO set = SetupDiGetClassDevsA(NULL, "USB", NULL, DIGCF_PRESENT | DIGCF_ALLCLASSES);
  SP_DEVINFO_DATA info;
  char id[512];
  DWORD i;
  int found = 0;

  if (set == INVALID_HANDLE_VALUE) {
    return 0;
  }
  info.cbSize = sizeof(info);
  for (i = 0; !found && SetupDiEnumDeviceInfo(set, i, &info); ++i) {
    if (SetupDiGetDeviceInstanceIdA(set, &info, id, sizeof(id), NULL)) {
      char *p;
      for (p = id; *p != '\0'; ++p) *p = (char)toupper((unsigned char)*p);
      found = strstr(id, "VID_057E&PID_0337") != NULL;
    }
  }
  SetupDiDestroyDeviceInfoList(set);
  return found;
}

/* Hotplug, the way Dolphin handles it: while no adapter is open (never found at startup, or
 * unplugged), a low-priority thread checks once a second whether one is plugged in and opens it,
 * so plugging the adapter in or moving it to another USB port mid-session just works. The full
 * open (and its logging) only runs when the device is actually present; after a failed open
 * (another program holding it, say) it waits 30 s before trying again. */
static HANDLE gw_gc_scan_thread;
static volatile LONG gw_gc_resume_gen; /* bumped by every resume: restarts the reclaim chain */

/* The scanner thread is the only place that opens the adapter after startup, so there is only ever
 * one retry chain. A resume starts (or restarts) the fast chain: every 100 ms for the first 1.5 s
 * (the window that lost focus lets go within ~100 ms), then every second up to 15 s, then the
 * ordinary 30 s hotplug backoff. A suspend ends the chain at once. */
static DWORD WINAPI gw_gc_scanner(LPVOID arg) {
  DWORD retry_at = 0;
  DWORD chain_start = 0;
  LONG chain_gen = 0;
  int chain = 0;
  int attempt = 0;
  (void)arg;
  for (;;) {
    DWORD now;
    DWORD wait = 1000;
    int pending;
    int ok;
    if (chain) {
      const int d = (int)(retry_at - GetTickCount());
      wait = d <= 0 ? 0 : (d > 1000 ? 1000 : (DWORD)d);
    }
    if (gw_gc_scan_wake != NULL) {
      WaitForSingleObject(gw_gc_scan_wake, wait);
    } else {
      Sleep(wait);
    }
    now = GetTickCount();
    if (InterlockedCompareExchange(&gw_gc_suspended, 0, 0) != 0) {
      chain = 0;
      continue;
    }
    pending = InterlockedCompareExchange(&gw_gc_resume_pending, 0, 0) != 0;
    if (!pending) {
      chain = 0;
    } else {
      const LONG gen = InterlockedCompareExchange(&gw_gc_resume_gen, 0, 0);
      if (!chain || gen != chain_gen) {
        chain = 1;
        chain_gen = gen;
        chain_start = now;
        attempt = 0;
        retry_at = now;
      }
    }
    if (gw_gc_ready) {
      InterlockedExchange(&gw_gc_resume_pending, 0);
      chain = 0;
      attempt = 0;
      continue;
    }
    if ((int)(now - retry_at) < 0) {
      continue;
    }
    if (!gw_gc_adapter_device_plugged()) {
      if (chain) {
        gw_pad_log("gw: gc adapter: resume - no adapter plugged in, nothing to reclaim");
        InterlockedExchange(&gw_gc_resume_pending, 0);
        chain = 0;
      }
      attempt = 0;
      continue;
    }
    ++attempt;
    if (!chain) {
      if (attempt == 1) {
        gw_log("gw: gc adapter: an adapter is plugged in - opening it");
      } else {
        gw_pad_log("gw: gc adapter: scanner retry %d", attempt);
      }
    }
    AcquireSRWLockExclusive(&gw_gc_init_srw);
    /* a suspend may have landed between the checks above and taking the lock */
    if (InterlockedCompareExchange(&gw_gc_suspended, 0, 0) != 0) {
      ReleaseSRWLockExclusive(&gw_gc_init_srw);
      chain = 0;
      continue;
    }
    gw_gc_tried = 0;
    gw_gc_open_quiet = chain && attempt > 1; /* one "held by another program" line per chain */
    ok = gw_gc_init_locked();
    gw_gc_open_quiet = 0;
    now = GetTickCount();
    if (ok) {
      if (chain) {
        /* the controllers never moved: keep their calibration (origins, trigger rests) */
        gw_log("gw: gc adapter: reclaimed after %d attempt(s), %lu ms after focus; calibration kept",
               attempt, (unsigned long)(now - chain_start));
      } else {
        InterlockedExchange(&gw_gc_recal, 1); /* re-sample the resting sticks and triggers */
      }
      InterlockedExchange(&gw_gc_resume_pending, 0);
      chain = 0;
      attempt = 0;
    } else if (chain) {
      const DWORD el = now - chain_start;
      if (attempt == 1) {
        gw_pad_log("gw: gc adapter: reclaim - another program still holds it; retrying every "
                   "100 ms, then every second");
      }
      if (el < 1500u) {
        retry_at = now + 100;
      } else if (el < 15000u) {
        retry_at = now + 1000;
      } else {
        gw_log("gw: gc adapter: could not reclaim it in 15 s (%d attempts; another program still "
               "holds it?) - retrying every 30 s", attempt);
        InterlockedExchange(&gw_gc_resume_pending, 0);
        chain = 0;
        retry_at = now + 30000;
      }
    } else {
      retry_at = now + 30000;
    }
    ReleaseSRWLockExclusive(&gw_gc_init_srw);
  }
  return 0;
}

int gw_gc_adapter_suspended(void) {
  return InterlockedCompareExchange(&gw_gc_suspended, 0, 0) != 0;
}

/* Focus lost: let go of the adapter (reader joined, WinUSB/HID handles closed) and keep the
 * scanner off it, so another program can claim it. Calibration is kept for the resume. Returns
 * whether an open adapter was released. */
int gw_gc_adapter_suspend(void) {
  int was;
  AcquireSRWLockExclusive(&gw_gc_init_srw);
  InterlockedExchange(&gw_gc_suspended, 1);
  InterlockedExchange(&gw_gc_resume_pending, 0);
  was = gw_gc_ready;
  if (was) {
    gw_gc_shutdown_locked();
  }
  ReleaseSRWLockExclusive(&gw_gc_init_srw);
  gw_log("gw: gc adapter: suspended (window in the background) - %s",
             was ? "adapter released for other programs" : "no adapter was open; scanner paused");
  return was;
}

/* Focus back: the scanner reopens the adapter right away (off the game thread, so the frame
 * never waits on USB enumeration), retrying each second for 15 s if the other program is slow to
 * let go. */
void gw_gc_adapter_resume(void) {
  if (InterlockedExchange(&gw_gc_suspended, 0) == 0) {
    return;
  }
  InterlockedIncrement(&gw_gc_resume_gen);
  InterlockedExchange(&gw_gc_resume_pending, 1);
  gw_log("gw: gc adapter: resumed (window focused) - reclaiming");
  if (gw_gc_scan_wake != NULL) {
    SetEvent(gw_gc_scan_wake);
  }
}

void gw_gc_adapter_start_hotplug(void) {
  if (gw_gc_scan_wake == NULL) {
    gw_gc_scan_wake = CreateEventA(NULL, FALSE, FALSE, NULL);
  }
  if (gw_gc_scan_thread == NULL) {
    gw_gc_scan_thread = CreateThread(NULL, 0, gw_gc_scanner, NULL, 0, NULL);
    if (gw_gc_scan_thread != NULL) {
      SetThreadPriority(gw_gc_scan_thread, THREAD_PRIORITY_LOWEST);
    }
  }
}

/* Re-sample the resting state on the next read. Safe to call at any time; it only latches a
 * flag, and the capture itself happens on the game thread inside gw_gc_adapter_read. */
void gw_gc_adapter_recalibrate(void) {
  InterlockedExchange(&gw_gc_recal, 1);
  gw_log("gw: gc adapter: recalibrating on the next poll -- release everything now");
}

/* One blocking read on whichever transport is open. Returns 1 on a good packet, 0 on timeout,
 * -1 if the device has gone away. Runs on the reader thread only. */
static int gw_gc_read_blocking(void) {
  unsigned char buf[64];
  DWORD err;

  if (gw_gc_backend == GW_GC_WINUSB) {
    ULONG got = 0;
    if (WinUsb_ReadPipe(gw_gc_usb, GC_EP_IN, buf, GC_PAYLOAD_SIZE, &got, NULL)) {
      if (got == GC_PAYLOAD_SIZE && buf[0] == 0x21) {
        EnterCriticalSection(&gw_gc_lock);
        memcpy(gw_gc_payload, buf, GC_PAYLOAD_SIZE);
        gw_gc_payload_valid = 1;
        gw_gc_stamp(buf);
        LeaveCriticalSection(&gw_gc_lock);
        return 1;
      }
      return 0;
    }
    err = GetLastError();
    if (err == ERROR_SEM_TIMEOUT) {
      return 0;
    }
    return -1;
  }

  if (gw_gc_backend == GW_GC_HID) {
    DWORD got = 0;
    OVERLAPPED ov;
    ZeroMemory(&ov, sizeof(ov));
    ov.hEvent = gw_gc_hid_event;
    ResetEvent(gw_gc_hid_event);
    if (!ReadFile(gw_gc_hid, buf, gw_gc_hid_in_len, &got, &ov)) {
      if (GetLastError() != ERROR_IO_PENDING) {
        return -1;
      }
      if (WaitForSingleObject(gw_gc_hid_event, 20) != WAIT_OBJECT_0) {
        CancelIo(gw_gc_hid);
        WaitForSingleObject(gw_gc_hid_event, 50);
        return 0;
      }
      if (!GetOverlappedResult(gw_gc_hid, &ov, &got, FALSE)) {
        return -1;
      }
    }
    {
      ULONG off;
      for (off = 0; off + GC_PAYLOAD_SIZE <= got; ++off) {
        if (buf[off] == 0x21) {
          EnterCriticalSection(&gw_gc_lock);
          memcpy(gw_gc_payload, buf + off, GC_PAYLOAD_SIZE);
          gw_gc_payload_valid = 1;
          gw_gc_stamp(buf + off);
          LeaveCriticalSection(&gw_gc_lock);
          return 1;
        }
      }
    }
    return 0;
  }

  return -1;
}

static DWORD WINAPI gw_gc_reader(LPVOID arg) {
  (void)arg;
  while (InterlockedCompareExchange(&gw_gc_quit, 0, 0) == 0) {
    if (gw_gc_read_blocking() < 0) {
      InterlockedExchange(&gw_gc_lost, 1);
      break;
    }
  }
  return 0;
}

/* Game-thread side: never touches the device, only the last packet the reader stored. */
static int gw_gc_poll(void) {
  int valid;

  if (!gw_gc_ready) {
    return 0;
  }
  if (InterlockedCompareExchange(&gw_gc_lost, 0, 0) != 0) {
    gw_log("gw: gc adapter: device went away; reverting to the SDL pad path");
    gw_gc_adapter_shutdown();
    return 0;
  }
  EnterCriticalSection(&gw_gc_lock);
  valid = gw_gc_payload_valid;
  LeaveCriticalSection(&gw_gc_lock);
  return valid;
}

static int gw_gc_poll_unused(void) {
  unsigned char buf[GC_PAYLOAD_SIZE];
  ULONG got = 0;
  int guard = 0;

  if (!gw_gc_ready) {
    return 0;
  }
  if (gw_gc_backend == GW_GC_HID) {
    return gw_gc_poll_hid();
  }

  /* Drain whatever the adapter has queued and keep the newest packet: it reports far faster than
   * the game's frame rate, and acting on a stale packet would add input latency. The guard stops
   * a pathologically fast device from spinning the frame here. */
  while (guard++ < 16 && WinUsb_ReadPipe(gw_gc_usb, GC_EP_IN, buf, sizeof(buf), &got, NULL)) {
    if (got == GC_PAYLOAD_SIZE && buf[0] == 0x21) {
      memcpy(gw_gc_payload, buf, GC_PAYLOAD_SIZE);
      gw_gc_payload_valid = 1;
    }
    if (got == 0) {
      break;
    }
  }

  if (!gw_gc_payload_valid) {
    const DWORD err = GetLastError();
    if (err == ERROR_DEVICE_NOT_CONNECTED || err == ERROR_GEN_FAILURE ||
        err == ERROR_FILE_NOT_FOUND || err == ERROR_NO_SUCH_DEVICE) {
      gw_log("gw: gc adapter: lost the device (error %lu); reverting to the SDL pad path",
             (unsigned long)err);
      gw_gc_close();
      return 0;
    }
  }

  return gw_gc_payload_valid;
}

/* Rescale a trigger so its resting position reads 0 and full depression still reaches 255. A
 * trigger whose rest value leaves no usable travel is reported as never pressed. */
static u8 gw_gc_trigger(unsigned char raw, unsigned char rest) {
  int span;
  int v;

  if (rest >= GW_GC_TRIG_DEAD) {
    return 0;
  }
  span = 255 - (int)rest;
  v = ((int)raw - (int)rest) * 255 / span;
  if (v < 0) {
    v = 0;
  }
  if (v > 255) {
    v = 255;
  }
  return (u8)v;
}

/* Centre a raw 0..255 axis on its captured origin and clamp into the console's s8 range. */
static s8 gw_gc_axis(unsigned char raw, unsigned char origin) {
  int v = (int)raw - (int)origin;
  if (v > 127) {
    v = 127;
  }
  if (v < -128) {
    v = -128;
  }
  return (s8)v;
}

int gw_gc_adapter_read(void *status) {
  PADStatus *st = (PADStatus *)status;
  int chan;
  int any = 0;
  int recal;
  unsigned char snap[GC_PAYLOAD_SIZE];

  if (gw_gc_ready && InterlockedCompareExchange(&gw_gc_suspended, 0, 0) != 0) {
    /* can't happen (every open re-checks the flag under the lock) - but if it ever does, a
     * background window must not keep the adapter from the program in front */
    gw_log("gw: gc adapter: BUG open while suspended - closing it");
    gw_gc_adapter_shutdown();
    return 0;
  }
  if (!gw_gc_ready || st == NULL) {
    return 0;
  }
  if (!gw_gc_poll()) {
    return 0;
  }

  /* Take the newest report whole. The reader thread overwrites gw_gc_payload under the lock at
   * whatever rate the adapter reports (125 Hz stock, up to 1000 Hz overclocked); decoding it in
   * place, outside the lock, could mix two reports - one port's buttons from one and its stick
   * from the next. */
  EnterCriticalSection(&gw_gc_lock);
  memcpy(snap, gw_gc_payload, GC_PAYLOAD_SIZE);
  LeaveCriticalSection(&gw_gc_lock);

  recal = InterlockedExchange(&gw_gc_recal, 0) != 0;

  for (chan = 0; chan < GC_PORTS; ++chan) {
    const unsigned char *p = &snap[1 + chan * 9];
    const int type = p[0] >> 4;
    u16 btn = 0;
    unsigned char b1;
    unsigned char b2;

    if (type != 1 && type != 2) {
      /* Nothing plugged into this adapter port: leave the channel untouched so Aurora's own
       * devices and the keyboard overlay can still own it. */
      continue;
    }

    if (!gw_gc_have_origin[chan] || recal) {
      gw_gc_origin[chan][0] = p[3];
      gw_gc_origin[chan][1] = p[4];
      gw_gc_origin[chan][2] = p[5];
      gw_gc_origin[chan][3] = p[6];
      gw_gc_trig_rest[chan][0] = p[7];
      gw_gc_trig_rest[chan][1] = p[8];
      /* Anything held at the moment of calibration is treated as stuck and masked out. Hold
       * nothing while this runs; the hotkey exists so it can be repeated deliberately. */
      gw_gc_btn_stuck[chan][0] = p[1];
      gw_gc_btn_stuck[chan][1] = p[2];
      gw_gc_have_origin[chan] = 1;
      gw_log("gw: gc adapter: ch%d calibrated -- stick (%u,%u) c (%u,%u) triggers (%u,%u)%s%s",
             chan, p[3], p[4], p[5], p[6], p[7], p[8],
             p[7] >= GW_GC_TRIG_DEAD ? " [L has no travel, disabled]" : "",
             p[8] >= GW_GC_TRIG_DEAD ? " [R has no travel, disabled]" : "");
      if (p[1] != 0 || p[2] != 0) {
        gw_log("gw: gc adapter: ch%d has buttons held at rest (b1=%02X b2=%02X); masking them",
               chan, p[1], p[2]);
      }
    }

    b1 = (unsigned char)(p[1] & ~gw_gc_btn_stuck[chan][0]);
    b2 = (unsigned char)(p[2] & ~gw_gc_btn_stuck[chan][1]);

    if (b1 & GC_BTN_A) { btn |= PAD_BUTTON_A; }
    if (b1 & GC_BTN_B) { btn |= PAD_BUTTON_B; }
    if (b1 & GC_BTN_X) { btn |= PAD_BUTTON_X; }
    if (b1 & GC_BTN_Y) { btn |= PAD_BUTTON_Y; }
    if (b1 & GC_BTN_DLEFT) { btn |= PAD_BUTTON_LEFT; }
    if (b1 & GC_BTN_DRIGHT) { btn |= PAD_BUTTON_RIGHT; }
    if (b1 & GC_BTN_DDOWN) { btn |= PAD_BUTTON_DOWN; }
    if (b1 & GC_BTN_DUP) { btn |= PAD_BUTTON_UP; }
    if (b2 & GC_BTN2_START) { btn |= PAD_BUTTON_START; }
    if (b2 & GC_BTN2_Z) { btn |= PAD_TRIGGER_Z; }
    if (b2 & GC_BTN2_R) { btn |= PAD_TRIGGER_R; }
    if (b2 & GC_BTN2_L) { btn |= PAD_TRIGGER_L; }

    /* The status array lives in game memory, so the u16 button field is written big-endian;
     * every other field is a single byte and crosses unchanged. */
    gw_w16(&st[chan].button, btn);
    st[chan].stickX = gw_gc_axis(p[3], gw_gc_origin[chan][0]);
    st[chan].stickY = gw_gc_axis(p[4], gw_gc_origin[chan][1]);
    st[chan].substickX = gw_gc_axis(p[5], gw_gc_origin[chan][2]);
    st[chan].substickY = gw_gc_axis(p[6], gw_gc_origin[chan][3]);
    st[chan].triggerLeft = gw_gc_trigger(p[7], gw_gc_trig_rest[chan][0]);
    st[chan].triggerRight = gw_gc_trigger(p[8], gw_gc_trig_rest[chan][1]);
    st[chan].analogA = 0;
    st[chan].analogB = 0;
    st[chan].err = 0;
    any = 1;
  }

  return any;
}

void gw_gc_adapter_rumble(int chan, int on) {
  static unsigned char state[GC_PORTS];
  unsigned char cmd[5];
  ULONG written = 0;
  int i;

  if (!gw_gc_ready || chan < 0 || chan >= GC_PORTS) {
    return;
  }
  if (state[chan] == (unsigned char)(on ? 1 : 0)) {
    return;
  }
  state[chan] = (unsigned char)(on ? 1 : 0);

  cmd[0] = 0x11;
  for (i = 0; i < GC_PORTS; ++i) {
    cmd[1 + i] = state[i];
  }
  if (gw_gc_backend == GW_GC_HID) {
    unsigned char out[64];
    ULONG len = gw_gc_hid_out_len;
    if (len == 0 || len > sizeof(out)) {
      len = sizeof(cmd) + 1;
    }
    ZeroMemory(out, sizeof(out));
    out[0] = 0x00; /* report ID */
    memcpy(out + 1, cmd, sizeof(cmd));
    HidD_SetOutputReport(gw_gc_hid, out, len);
    return;
  }
  WinUsb_WritePipe(gw_gc_usb, GC_EP_OUT, cmd, sizeof(cmd), &written, NULL);
}

/* Dump the adapter's raw report bytes, so a mapping question can be settled against what was
 * actually pressed rather than by guessing. Verbose level only (MELEE_PAD_DIAG=2 / pad_diag=2). */
void gw_gc_adapter_diag(void) {
  extern int gw_pad_diag_level(void);
  static int frames;
  int chan;

  if (gw_pad_diag_level() < 2 || !gw_gc_ready || !gw_gc_payload_valid || (++frames % 30) != 0) {
    return;
  }
  for (chan = 0; chan < GC_PORTS; ++chan) {
    const unsigned char *p = &gw_gc_payload[1 + chan * 9];
    if ((p[0] >> 4) != 1 && (p[0] >> 4) != 2) {
      continue;
    }
    gw_log("gw: DIAG gcraw ch%d type=%d b1=%02X b2=%02X stick=(%u,%u) c=(%u,%u) trig=(%u,%u)",
           chan, p[0] >> 4, p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8]);
  }
}

/* ---- tests: suspend racing a slow open ---------------------------------------------------- */
#include "gw_test.h"

static volatile LONG gw_gc_t_opens;
static volatile LONG gw_gc_t_suspend_inside;
static int gw_gc_t_slow_open(void) {
  InterlockedIncrement(&gw_gc_t_opens);
  Sleep(150); /* an enumeration + WinUSB open that takes a while */
  if (gw_gc_t_suspend_inside) {
    InterlockedExchange(&gw_gc_suspended, 1); /* as if the flag flipped during the open */
  }
  gw_gc_ready = 1;
  gw_gc_backend = GW_GC_NONE;
  return 1;
}
static DWORD WINAPI gw_gc_t_opener(LPVOID arg) {
  (void)arg;
  AcquireSRWLockExclusive(&gw_gc_init_srw);
  gw_gc_tried = 0;
  gw_gc_init_locked();
  ReleaseSRWLockExclusive(&gw_gc_init_srw);
  return 0;
}

static int test_gc_suspend_beats_slow_open(void) {
  HANDLE t;
  int rc = 0;
  if (gw_gc_ready || gw_gc_scan_thread != NULL) {
    return 0; /* a real adapter session is live in this process: nothing to fake */
  }
  gw_gc_test_open = gw_gc_t_slow_open;
  InterlockedExchange(&gw_gc_suspended, 0);
  InterlockedExchange(&gw_gc_t_opens, 0);
  gw_gc_t_suspend_inside = 0;

  /* 1. suspend lands while the open is in flight: it waits for the open, then closes it */
  t = CreateThread(NULL, 0, gw_gc_t_opener, NULL, 0, NULL);
  Sleep(40);
  gw_gc_adapter_suspend();
  WaitForSingleObject(t, 2000);
  CloseHandle(t);
  if (gw_gc_ready) { gw_test_fail("adapter open after a suspend that raced the open"); rc = 1; }
  /* 2. while suspended nothing opens */
  if (!rc) {
    InterlockedExchange(&gw_gc_t_opens, 0);
    gw_gc_tried = 0;
    if (gw_gc_adapter_init() || gw_gc_ready || gw_gc_t_opens != 0) {
      gw_test_fail("opened while suspended"); rc = 1;
    }
  }
  /* 3. the flag flips during the open itself: the post-open check backs it out */
  if (!rc) {
    InterlockedExchange(&gw_gc_suspended, 0);
    gw_gc_t_suspend_inside = 1;
    gw_gc_tried = 0;
    if (gw_gc_adapter_init() || gw_gc_ready) { gw_test_fail("open survived a mid-open suspend"); rc = 1; }
  }
  /* 4. resume + open works again */
  if (!rc) {
    gw_gc_t_suspend_inside = 0;
    gw_gc_adapter_resume();
    gw_gc_tried = 0;
    if (!gw_gc_adapter_init() || !gw_gc_ready) { gw_test_fail("no open after resume"); rc = 1; }
  }
  gw_gc_adapter_shutdown();
  InterlockedExchange(&gw_gc_suspended, 0);
  InterlockedExchange(&gw_gc_resume_pending, 0);
  gw_gc_tried = 0;
  gw_gc_test_open = NULL;
  return rc;
}

void gw_gc_adapter_tests_register(void) {
  gw_test_register("gc_suspend_beats_slow_open", test_gc_suspend_beats_slow_open);
}
