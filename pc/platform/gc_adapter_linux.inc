/* Linux transport. Only the worker owns the USB handle; the game copies reports under mutex. */
#include "gw.h"
#include "gw_compat_linux.h"
#include <dolphin/pad.h>
#include <libusb.h>
#include <pthread.h>
#include <string.h>
#include <time.h>
static unsigned char gw_gc_origin[4][4], gw_gc_trig_rest[4][2], gw_gc_btn_stuck[4][2];
static int gw_gc_have_origin[4];
#define GW_GC_TRIG_DEAD 200
#include "gc_adapter_decode.h"
static pthread_mutex_t gc_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t gc_wake = PTHREAD_COND_INITIALIZER;
static pthread_t gc_worker;
static int gc_started, gc_opening, gc_quit, gc_suspended, gc_ready, gc_plugged, gc_valid, gc_recal;
static unsigned char gc_report[37], gc_rumble[4];
static unsigned gc_reports, gc_hist[9], gc_count, gc_changed;
static uint64_t gc_qpc;
static double gc_min, gc_max;
static void gc_stamp(const unsigned char *report) {
    LARGE_INTEGER now;
    QueryPerformanceCounter(&now);
    if (gc_qpc) {
        static const double edges[9] = {.75,1.5,3,5,7,9,12,20,1e9};
        double ms = (now.QuadPart - gc_qpc) / 1e6;
        for (int i=0;i<9;++i) if (ms < edges[i]) { ++gc_hist[i]; break; }
        if (!gc_count || ms < gc_min) gc_min = ms;
        if (ms > gc_max) gc_max = ms;
        ++gc_count;
        if (memcmp(gc_report, report, 37)) ++gc_changed;
    }
    gc_qpc = now.QuadPart;
    memcpy(gc_report, report, 37);
    gc_valid = 1;
    ++gc_reports;
}
static libusb_device_handle *gc_open(libusb_context *ctx, int *iface, unsigned char *in, unsigned char *out) {
    libusb_device **devices;
    ssize_t count = libusb_get_device_list(ctx, &devices);
    libusb_device_handle *handle = NULL;
    int plugged = 0, error = 0;
    for (ssize_t i=0; i<count && !handle; ++i) {
        struct libusb_device_descriptor desc;
        if (libusb_get_device_descriptor(devices[i], &desc) || desc.idVendor != 0x057e || desc.idProduct != 0x0337) continue;
        plugged = 1;
        struct libusb_config_descriptor *config = NULL;
        if (libusb_get_active_config_descriptor(devices[i], &config)) continue;
        for (int j=0; j<config->bNumInterfaces && !handle; ++j) {
            const struct libusb_interface *intf = &config->interface[j];
            for (int k=0; k<intf->num_altsetting && !handle; ++k) {
                const struct libusb_interface_descriptor *alt = &intf->altsetting[k];
                unsigned char ep_in=0, ep_out=0;
                for (int e=0; e<alt->bNumEndpoints; ++e) {
                    const struct libusb_endpoint_descriptor *ep = &alt->endpoint[e];
                    if ((ep->bmAttributes & LIBUSB_TRANSFER_TYPE_MASK) != LIBUSB_TRANSFER_TYPE_INTERRUPT) continue;
                    if (ep->bEndpointAddress & LIBUSB_ENDPOINT_IN) ep_in=ep->bEndpointAddress;
                    else ep_out=ep->bEndpointAddress;
                }
                if (!ep_in || !ep_out) continue;
                error = libusb_open(devices[i], &handle);
                if (error) { handle=NULL; continue; }
                libusb_set_auto_detach_kernel_driver(handle, 1);
                error = libusb_claim_interface(handle, alt->bInterfaceNumber);
                if (error) { libusb_close(handle); handle=NULL; continue; }
                if (alt->bAlternateSetting && (error=libusb_set_interface_alt_setting(handle, alt->bInterfaceNumber, alt->bAlternateSetting))) {
                    libusb_release_interface(handle, alt->bInterfaceNumber); libusb_close(handle); handle=NULL; continue;
                }
                *iface=alt->bInterfaceNumber; *in=ep_in; *out=ep_out;
            }
        }
        libusb_free_config_descriptor(config);
    }
    if (count >= 0) libusb_free_device_list(devices, 1);
    pthread_mutex_lock(&gc_lock); gc_plugged=plugged; pthread_mutex_unlock(&gc_lock);
    static int previous_error;
    if (error && error != previous_error) gw_log("gw: GameCube USB adapter: %s%s", libusb_error_name(error),
        error == LIBUSB_ERROR_ACCESS ? " (install the supplied 057e:0337 udev rule)" : error == LIBUSB_ERROR_BUSY ? " (another process owns the interface)" : "");
    previous_error=error;
    return handle;
}
static void gc_close(libusb_device_handle **handle, int iface, unsigned char ep_out) {
    if (*handle) {
        unsigned char stop[5]={0x11,0,0,0,0}; int sent;
        libusb_interrupt_transfer(*handle, ep_out, stop, sizeof stop, &sent, 100);
        libusb_release_interface(*handle, iface); /* auto-detach reattaches the kernel driver */
        libusb_close(*handle); *handle=NULL;
    }
    pthread_mutex_lock(&gc_lock);
    gc_ready=gc_valid=gc_opening=0; gc_qpc=0;
    memset(gc_rumble,0,sizeof gc_rumble);
    pthread_cond_broadcast(&gc_wake);
    pthread_mutex_unlock(&gc_lock);
}
static void gc_wait(void) {
    struct timespec until; clock_gettime(CLOCK_REALTIME,&until); ++until.tv_sec;
    pthread_mutex_lock(&gc_lock);
    if (!gc_quit) pthread_cond_timedwait(&gc_wake,&gc_lock,&until);
    pthread_mutex_unlock(&gc_lock);
}
static void *gc_run(void *unused) {
    (void)unused;
    libusb_context *ctx=NULL;
    libusb_device_handle *handle=NULL;
    unsigned char ep_in=0,ep_out=0, sent_rumble[4]={0};
    int iface=0;
    if (!gw_linux_signal_thread_init()) return NULL;
    if (libusb_init(&ctx)) { gw_log("gw: libusb initialization failed"); return NULL; }
    for (;;) {
        pthread_mutex_lock(&gc_lock); int quit=gc_quit, suspended=gc_suspended; pthread_mutex_unlock(&gc_lock);
        if (quit) break;
        if (suspended) { gc_close(&handle,iface,ep_out); gc_wait(); continue; }
        if (!handle) {
            pthread_mutex_lock(&gc_lock);
            int skip=gc_suspended || gc_quit;
            if (!skip) gc_opening=1;
            pthread_mutex_unlock(&gc_lock);
            if (skip) continue;
            handle=gc_open(ctx,&iface,&ep_in,&ep_out);
            if (!handle) { gc_close(&handle,iface,ep_out); gc_wait(); continue; }
            unsigned char init=0x13; int sent=0;
            int rc=libusb_interrupt_transfer(handle,ep_out,&init,1,&sent,100);
            pthread_mutex_lock(&gc_lock);
            int cancel=gc_suspended || gc_quit;
            if (!rc && sent==1 && !cancel) { gc_ready=1; gc_recal=1; gc_opening=0; }
            pthread_mutex_unlock(&gc_lock);
            if (rc || sent!=1 || cancel) { gc_close(&handle,iface,ep_out); continue; }
            memset(sent_rumble,0,sizeof sent_rumble);
            gw_log("gw: GameCube adapter connected via libusb");
        }
        unsigned char report[37]; int size=0;
        int rc=libusb_interrupt_transfer(handle,ep_in,report,sizeof report,&size,50);
        if (rc && rc!=LIBUSB_ERROR_TIMEOUT) { gc_close(&handle,iface,ep_out); continue; }
        unsigned char rumble[5]={0x11};
        pthread_mutex_lock(&gc_lock);
        if (!rc && size==37 && report[0]==0x21 && !gc_suspended) gc_stamp(report);
        memcpy(rumble+1,gc_rumble,4);
        pthread_mutex_unlock(&gc_lock);
        if (memcmp(sent_rumble,rumble+1,4)) {
            int sent;
            rc=libusb_interrupt_transfer(handle,ep_out,rumble,5,&sent,100);
            if (rc || sent!=5) { gc_close(&handle,iface,ep_out); continue; }
            memcpy(sent_rumble,rumble+1,4);
        }
    }
    gc_close(&handle,iface,ep_out); libusb_exit(ctx); return NULL;
}
void gw_gc_adapter_start_hotplug(void) {
    const char *sdl=getenv("MELEE_SDL_GAMECUBE");
    if (sdl && *sdl=='1') return;
    pthread_mutex_lock(&gc_lock);
    if (!gc_started) {
        gc_quit=0;
        int rc=pthread_create(&gc_worker,NULL,gc_run,NULL);
        if (!rc) gc_started=1;
        else gw_log("gw: GameCube worker creation failed: %d",rc);
    }
    pthread_mutex_unlock(&gc_lock);
}
int gw_gc_adapter_present(void) { pthread_mutex_lock(&gc_lock); int r=gc_ready && !gc_suspended; pthread_mutex_unlock(&gc_lock); return r; }
int gw_gc_adapter_init(void) { gw_gc_adapter_start_hotplug(); return gw_gc_adapter_present(); }
int gw_gc_adapter_device_plugged(void) { pthread_mutex_lock(&gc_lock); int r=gc_plugged; pthread_mutex_unlock(&gc_lock); return r; }
int gw_gc_adapter_read(void *status) {
    if (!status) return 0;
    pthread_mutex_lock(&gc_lock);
    int r=0;
    if (gc_ready && gc_valid && !gc_suspended) { r=gw_gc_decode(gc_report,status,gc_recal); gc_recal=0; }
    pthread_mutex_unlock(&gc_lock); return r;
}
void gw_gc_adapter_rumble(int chan,int on) { if (chan<0 || chan>=4) return; pthread_mutex_lock(&gc_lock); gc_rumble[chan]=!!on; pthread_mutex_unlock(&gc_lock); }
void gw_gc_adapter_recalibrate(void) { pthread_mutex_lock(&gc_lock); gc_recal=1; pthread_mutex_unlock(&gc_lock); }
int gw_gc_adapter_suspend(void) {
    pthread_mutex_lock(&gc_lock); int was=gc_ready; gc_suspended=1;
    pthread_cond_broadcast(&gc_wake);
    while (gc_ready || gc_opening) pthread_cond_wait(&gc_wake,&gc_lock);
    pthread_mutex_unlock(&gc_lock); return was;
}
void gw_gc_adapter_resume(void) { pthread_mutex_lock(&gc_lock); gc_suspended=0; pthread_cond_broadcast(&gc_wake); pthread_mutex_unlock(&gc_lock); }
int gw_gc_adapter_suspended(void) { pthread_mutex_lock(&gc_lock); int r=gc_suspended; pthread_mutex_unlock(&gc_lock); return r; }
unsigned gw_gc_adapter_report_count(void) { pthread_mutex_lock(&gc_lock); unsigned r=gc_reports; pthread_mutex_unlock(&gc_lock); return r; }
long long gw_gc_adapter_report_qpc(void) { pthread_mutex_lock(&gc_lock); uint64_t r=gc_qpc; pthread_mutex_unlock(&gc_lock); return r; }
void gw_gc_adapter_take_stats(unsigned *hist,unsigned *count,unsigned *changed,double *min,double *max) {
    pthread_mutex_lock(&gc_lock);
    if(hist) memcpy(hist,gc_hist,sizeof gc_hist);
    if(count) *count=gc_count; if(changed) *changed=gc_changed; if(min) *min=gc_min; if(max) *max=gc_max;
    memset(gc_hist,0,sizeof gc_hist); gc_count=gc_changed=0; gc_min=gc_max=0;
    pthread_mutex_unlock(&gc_lock);
}
void gw_gc_adapter_shutdown(void) {
    pthread_mutex_lock(&gc_lock); int join=gc_started; gc_quit=1; pthread_cond_broadcast(&gc_wake); pthread_mutex_unlock(&gc_lock);
    if (join) pthread_join(gc_worker,NULL);
    pthread_mutex_lock(&gc_lock); gc_started=0; pthread_mutex_unlock(&gc_lock);
}
void gw_gc_adapter_diag(void) {}
void gw_gc_adapter_tests_register(void) {}
