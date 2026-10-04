/* Independent watchdog: progress counters are the only tick-path work. A small
 * native thread atomically rewrites heartbeat.json once a second, even if the
 * game spins without calling a shim. Diagnosis never calls gw_log: a stuck main
 * thread may own the logger/CRT locks. Direct Win32 IO leaves evidence first.
 * Only a missing main tick is fatal; pauses, loading, hidden windows and script
 * deadlines have separate states. Rendering count is Aurora's completed-present
 * count, not the number of submissions. The legacy PC sampler remains intact.
 */
#include "gw_hang.h"
#include "gw_disc_privacy.h"
#include "gw_hang_policy.h"
#include "gw_test.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#include <dbghelp.h>
#include <dwmapi.h>

extern uint32_t aurora_get_present_count(void);
static volatile LONG hb_tick, hb_logic, hb_pause, hb_scene, hb_instructions, hb_exit;
__declspec(align(8)) static volatile LONG64 hb_log_wall;
static const char *volatile hb_shim = "boot";
static SRWLOCK hb_lock = SRWLOCK_INIT;
static struct {
    char owner[128], callback[128], deadline[128];
    int active, pending, expired;
} hb_meta;
static HANDLE hb_main, hb_stop, hb_worker;
static DWORD hb_main_id;
static unsigned long long hb_created;
static double hb_start, hb_seconds = 10;
static char hb_label[256], hb_action[16];
static char *hb_map;
static char hb_private_discs[5][1024]; /* capture env before starting the worker */
static volatile LONG hb_transition;
__declspec(align(8)) static volatile LONG64 hb_transition_at;
typedef HRESULT (WINAPI *HbDwmAttr)(HWND,DWORD,PVOID,DWORD);
static HbDwmAttr hb_dwm_attr;
static HMODULE hb_dwm;

static void hb_load_map(void) {
    HANDLE f=CreateFileA("melee-pc.map",GENERIC_READ,FILE_SHARE_READ,NULL,OPEN_EXISTING,0,NULL);
    DWORD n,done;char *stamp;unsigned expected;
    const unsigned char *image=(const unsigned char *)GetModuleHandleW(NULL);
    const IMAGE_DOS_HEADER *dos=(const IMAGE_DOS_HEADER *)image;
    const IMAGE_NT_HEADERS *nt=(const IMAGE_NT_HEADERS *)(image+dos->e_lfanew);
    if(f==INVALID_HANDLE_VALUE)return;
    n=GetFileSize(f,NULL);
    if(n && n<32*1024*1024) {
        hb_map=(char *)HeapAlloc(GetProcessHeap(),0,(SIZE_T)n+1);
        if(hb_map && ReadFile(f,hb_map,n,&done,NULL) && done==n) {
            hb_map[n]=0;stamp=strstr(hb_map,"Timestamp is ");expected=nt->FileHeader.TimeDateStamp;
            if(!stamp || strtoul(stamp+13,NULL,16)!=expected){HeapFree(GetProcessHeap(),0,hb_map);hb_map=NULL;}
        } else if(hb_map){HeapFree(GetProcessHeap(),0,hb_map);hb_map=NULL;}
    }
    CloseHandle(f);
}
static void hb_symbol(char *where,size_t cap) {
    const char *rva=strstr(where,"rva 0x");char name[256]="",candidate[256];
    unsigned pc,best=0;char *line;
    if(!hb_map || !rva)return;
    pc=(unsigned)strtoul(rva+6,NULL,16);
    for(line=hb_map;line && *line;) {
        char *end=strchr(line,'\n');unsigned section,offset,address;
        if(end)*end=0;
        if(sscanf(line," %x:%x %255s %x",&section,&offset,candidate,&address)==4 && section==1 &&
            address<=pc && address>=best) {best=address;snprintf(name,sizeof name,"%s",candidate);}
        if(end){*end='\n';line=end+1;}else break;
    }
    if(best && pc-best<0x20000) {
        size_t n=strlen(where);snprintf(where+n,cap-n," %s+0x%X",name,pc-best);
    }
}

static double hb_wall(void) {
    FILETIME ft; ULARGE_INTEGER u;
    GetSystemTimeAsFileTime(&ft); u.LowPart=ft.dwLowDateTime; u.HighPart=ft.dwHighDateTime;
    return (u.QuadPart-116444736000000000ULL)/10000000.0;
}
static LONG hb_read(volatile LONG *p) { return InterlockedCompareExchange(p,0,0); }
void gw_hang_tick(void) { InterlockedIncrement(&hb_tick); }
void gw_hang_logic(void) {
    InterlockedIncrement(&hb_logic);
    InterlockedCompareExchange(&hb_transition,0,2);
}
void gw_hang_transition(int phase) {
    if (phase==1) InterlockedExchange64(&hb_transition_at,(LONG64)GetTickCount64());
    if (phase!=2 || hb_read(&hb_transition)) InterlockedExchange(&hb_transition,phase);
}
void gw_hang_pause(int flags) { InterlockedExchange(&hb_pause,flags); }
void gw_hang_scene(int scene) { InterlockedExchange(&hb_scene,scene); }
void gw_hang_shim(const char *name) { InterlockedExchangePointer((void *volatile *)&hb_shim,(void *)name); }
void gw_hang_instructions(void) { InterlockedExchangeAdd(&hb_instructions,1000); }
void gw_hang_log_time(void) { InterlockedExchange64(&hb_log_wall,(LONG64)(hb_wall()*1000)); }
void gw_hang_callback(const char *owner,const char *name,int active) {
    AcquireSRWLockExclusive(&hb_lock);
    snprintf(hb_meta.owner,sizeof hb_meta.owner,"%s",owner);
    snprintf(hb_meta.callback,sizeof hb_meta.callback,"%s",name);
    hb_meta.active=active;
    if(active)InterlockedExchange(&hb_instructions,0);
    ReleaseSRWLockExclusive(&hb_lock);
}
void gw_hang_script_state(int pending,int expired,const char *last) {
    AcquireSRWLockExclusive(&hb_lock);
    hb_meta.pending=pending;hb_meta.expired=expired;
    snprintf(hb_meta.deadline,sizeof hb_meta.deadline,"%s",last);
    ReleaseSRWLockExclusive(&hb_lock);
}
static void hb_quote(char *dst,size_t cap,const char *s) {
    size_t n=0; if(cap<3)return;
    dst[n++]='"';
    while(*s && n+8<cap) {
        unsigned char c=(unsigned char)*s++;
        if(c=='"'||c=='\\'){dst[n++]='\\';dst[n++]=(char)c;}
        else if(c<32){snprintf(dst+n,cap-n,"\\u%04x",c);n+=6;}
        else dst[n++]=(char)c;
    }
    dst[n++]='"';dst[n]=0;
}
static int hb_write(const char *path,const char *text,int append) {
    char private_text[8192],tmp[8192];unsigned i;
    gw_disc_redact(private_text,sizeof private_text,text,gw_iso_path());
    for(i=0;i<5;++i) {
        gw_disc_redact(tmp,sizeof tmp,private_text,hb_private_discs[i]);
        memcpy(private_text,tmp,strlen(tmp)+1);
    }
    text=private_text;
    HANDLE f=CreateFileA(path,append?FILE_APPEND_DATA:GENERIC_WRITE,
        FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,NULL,append?OPEN_ALWAYS:CREATE_ALWAYS,
        FILE_ATTRIBUTE_NORMAL,NULL);
    DWORD done;size_t n=strlen(text);int ok;
    if(f==INVALID_HANDLE_VALUE)return 0;
    ok=WriteFile(f,text,(DWORD)n,&done,NULL) && done==n;
    FlushFileBuffers(f);CloseHandle(f);return ok;
}
static void hb_atomic(const char *name,const char *text) {
    char tmp[128];snprintf(tmp,sizeof tmp,"%s.%lu.tmp",name,GetCurrentThreadId());
    if(hb_write(tmp,text,0))MoveFileExA(tmp,name,MOVEFILE_REPLACE_EXISTING|MOVEFILE_WRITE_THROUGH);
}
void gw_hang_final(const char *reason,unsigned code) {
    char line[512],json[768],q[384];
    if(InterlockedExchange(&hb_exit,1))return;
    if(hb_stop)SetEvent(hb_stop);
    snprintf(line,sizeof line,"gw: final exit reason=%s code=%u\r\n",reason,code);
    hb_write("melee-pc.log",line,1);
    hb_quote(q,sizeof q,reason);
    snprintf(json,sizeof json,"{\"pid\":%lu,\"process_created\":%llu,\"wall_clock\":%.3f,\"reason\":%s,\"code\":%u}\n",
        GetCurrentProcessId(),hb_created,hb_wall(),q,code);
    hb_atomic("exit.json",json);
}
static void hb_atexit(void) { gw_hang_final("CRT exit (code unavailable; supervisor records it)",0); }

typedef struct { HWND window; int hidden,minimized,occluded,modal; long long area; } HbWindow;
static BOOL CALLBACK hb_window(HWND w,LPARAM param) {
    HbWindow *s=(HbWindow *)param;DWORD pid;char cls[64];RECT rect;
    GetWindowThreadProcessId(w,&pid);
    if(pid!=GetCurrentProcessId())return TRUE;
    GetClassNameA(w,cls,sizeof cls);
    if(strcmp(cls,"#32770")==0 && IsWindowVisible(w))s->modal=1;
    if(GetWindow(w,GW_OWNER)==NULL && strcmp(cls,"#32770")!=0 && GetWindowRect(w,&rect)) {
        long long area=(long long)(rect.right-rect.left)*(rect.bottom-rect.top);
        /* SDL's hidden helper/IME windows must not mask the actual game window. */
        if(area>s->area){s->window=w;s->area=area;}
    }
    return TRUE;
}
static HbWindow hb_window_state(void) {
    HbWindow s={0};GUITHREADINFO gui={sizeof gui};
    EnumWindows(hb_window,(LPARAM)&s);
    if(s.window) {
        s.hidden=!IsWindowVisible(s.window);s.minimized=IsIconic(s.window);
        if(!IsWindowEnabled(s.window))s.modal=1;
        /* DWM cloak is observable; arbitrary covering by other windows is not. */
        {
            DWORD cloak=0;
            if(hb_dwm_attr && SUCCEEDED(hb_dwm_attr(s.window,14,&cloak,sizeof cloak)))s.occluded=cloak!=0;
        }
    }
    if(GetGUIThreadInfo(hb_main_id,&gui) && (gui.flags&(GUI_INMOVESIZE|GUI_INMENUMODE|GUI_SYSTEMMENUMODE)))s.modal=1;
    return s;
}
static uintptr_t hb_pc(void) {
    CONTEXT ctx;uintptr_t pc=0;memset(&ctx,0,sizeof ctx);ctx.ContextFlags=CONTEXT_CONTROL;
    if(SuspendThread(hb_main)==(DWORD)-1)return 0;
    if(GetThreadContext(hb_main,&ctx)) {
#ifdef _WIN64
        pc=(uintptr_t)ctx.Rip;
#else
        pc=(uintptr_t)ctx.Eip;
#endif
    }
    /* Never resolve or log while the game thread is suspended. */
    ResumeThread(hb_main);return pc;
}
static DWORD WINAPI hb_dump(void *unused) {
    (void)unused;
    HMODULE dll=LoadLibraryA("dbghelp.dll");
    typedef BOOL (WINAPI *Dump)(HANDLE,DWORD,HANDLE,MINIDUMP_TYPE,
        PMINIDUMP_EXCEPTION_INFORMATION,PMINIDUMP_USER_STREAM_INFORMATION,PMINIDUMP_CALLBACK_INFORMATION);
    Dump dump=dll?(Dump)GetProcAddress(dll,"MiniDumpWriteDump"):NULL;
    if(dump) {
        HANDLE f=CreateFileA("hang.dmp",GENERIC_WRITE,FILE_SHARE_READ,NULL,CREATE_ALWAYS,FILE_ATTRIBUTE_NORMAL,NULL);
        if(f!=INVALID_HANDLE_VALUE) {
            BOOL ok=dump(GetCurrentProcess(),GetCurrentProcessId(),f,MiniDumpWithThreadInfo,NULL,NULL,NULL);
            CloseHandle(f);
            hb_write("melee-pc.log",ok?"gw: watchdog: hang.dmp written\r\n":"gw: watchdog: MiniDumpWriteDump failed\r\n",1);
        } else hb_write("melee-pc.log","gw: watchdog: could not create hang.dmp\r\n",1);
    } else hb_write("melee-pc.log","gw: watchdog: MiniDumpWriteDump unavailable\r\n",1);
    if(dll)FreeLibrary(dll);
    return 0;
}
static void hb_dump_bounded(void) {
    HANDLE worker=CreateThread(NULL,0,hb_dump,NULL,0,NULL);
    if(worker) {
        if(WaitForSingleObject(worker,5000)==WAIT_TIMEOUT)
            hb_write("melee-pc.log","gw: watchdog: dump exceeded 5 seconds; exit policy still applies\r\n",1);
        CloseHandle(worker);
    }
}
static const char *hb_state_name(int state) {
    static const char *const names[]={"running","main-loop-stalled","logic-not-advancing","not-presenting",
        "intentionally-paused","hidden","debugger","modal-dialog","disabled","scene-transition"};
    return names[state];
}
static DWORD WINAPI hb_run(void *unused) {
    LONG main_prev=hb_read(&hb_tick),logic_prev=hb_read(&hb_logic);
    uint32_t present_prev=aurora_get_present_count();
    ULONGLONG now=GetTickCount64(),main_at=now,logic_at=now,present_at=now;
    int diagnosed=0,last_state=-1;
    GwHangPauseLog pause_log={0};
    /* Cache survives a main-thread stall while publishing diagnostic metadata. */
    char owner[128]="",callback[128]="",deadline[128]="";
    int active=0,pending=0,expired=0;
    (void)unused;
    while(WaitForSingleObject(hb_stop,1000)==WAIT_TIMEOUT) {
        LONG main=hb_read(&hb_tick),logic=hb_read(&hb_logic),pause=hb_read(&hb_pause);
        uint32_t present=aurora_get_present_count();
        HbWindow w=hb_window_state();int debugger=IsDebuggerPresent(),state;
        double ma,la,pa,transition_age;char json[4096],qlabel[1024],qowner[512],qcallback[512],qdeadline[512],qshim[256];
        now=GetTickCount64();
        if(main!=main_prev){main_prev=main;main_at=now;diagnosed=0;}
        if(logic!=logic_prev){logic_prev=logic;logic_at=now;}
        if(present!=present_prev){present_prev=present;present_at=now;}
        /* Give an attached debugger/system dialog a full interval after release. */
        if(debugger||w.modal){main_at=logic_at=present_at=now;diagnosed=0;}
        ma=(now-main_at)/1000.0;la=(now-logic_at)/1000.0;pa=(now-present_at)/1000.0;
        state=gw_hang_classify(ma,la,pa,hb_seconds,pause,debugger,w.modal,w.hidden||w.minimized||w.occluded);
        transition_age=(now-InterlockedCompareExchange64(&hb_transition_at,0,0))/1000.0;
        state=gw_hang_expected(state,hb_read(&hb_transition),transition_age);
        if(TryAcquireSRWLockShared(&hb_lock)) {
            memcpy(owner,hb_meta.owner,sizeof owner);memcpy(callback,hb_meta.callback,sizeof callback);
            memcpy(deadline,hb_meta.deadline,sizeof deadline);
            active=hb_meta.active;pending=hb_meta.pending;expired=hb_meta.expired;
            ReleaseSRWLockShared(&hb_lock);
        }
        hb_quote(qlabel,sizeof qlabel,hb_label);hb_quote(qowner,sizeof qowner,owner);
        hb_quote(qcallback,sizeof qcallback,callback);hb_quote(qdeadline,sizeof qdeadline,deadline);
        hb_quote(qshim,sizeof qshim,(const char *)InterlockedCompareExchangePointer((void *volatile *)&hb_shim,NULL,NULL));
        snprintf(json,sizeof json,
            "{\"pid\":%lu,\"process_created\":%llu,\"start_time\":%.3f,\"run_label\":%s,\"wall_clock\":%.3f,"
            "\"main_ticks\":%lu,\"presented_frames\":%u,\"logic_frames\":%lu,\"scene\":%ld,"
            "\"paused\":%s,\"pause_flags\":%ld,\"debugger\":%s,\"modal\":%s,\"hidden\":%s,\"minimized\":%s,\"occluded\":%s,\"occlusion_known\":false,"
            "\"last_log_time\":%.3f,\"main_age_seconds\":%.3f,\"logic_age_seconds\":%.3f,\"present_age_seconds\":%.3f,\"watchdog_seconds\":%.3f,\"state\":\"%s\","
            "\"transition_age_seconds\":%.3f,\"expected_operation\":\"%s\",\"last_shim\":%s,\"script_watchdog\":{\"owner\":%s,\"callback\":%s,\"active\":%s,\"instructions\":%lu,\"pending\":%d,\"expired\":%d,\"last_deadline\":%s}}\n",
            GetCurrentProcessId(),hb_created,hb_start,qlabel,hb_wall(),(unsigned long)main,present,(unsigned long)logic,hb_read(&hb_scene),
            pause?"true":"false",pause,debugger?"true":"false",w.modal?"true":"false",w.hidden?"true":"false",w.minimized?"true":"false",w.occluded?"true":"false",
            InterlockedCompareExchange64(&hb_log_wall,0,0)/1000.0,ma,la,pa,hb_seconds,hb_state_name(state),transition_age,
            hb_read(&hb_transition)?"gd.scene_launch":"",qshim,qowner,qcallback,active?"true":"false",
            (unsigned long)hb_read(&hb_instructions),pending,expired,qdeadline);
        hb_atomic("heartbeat.json",json);
        {
            int event=gw_hang_pause_log(&pause_log,state==GW_HANG_PAUSED,now/1000.0);
            char line[256];
            if(event&2) {
                snprintf(line,sizeof line,"gw: watchdog: pause summary episodes=%u paused_seconds=%.1f\r\n",pause_log.episodes,pause_log.seconds);
                hb_write("melee-pc.log",line,1);pause_log.episodes=0;pause_log.seconds=0;pause_log.next=now/1000.0+30;
            }
            if(event&1) hb_write("melee-pc.log","gw: watchdog: intentionally-paused (pause entries limited to one per 30 seconds)\r\n",1);
        }
        if(state!=last_state && state!=GW_HANG_MAIN && state!=GW_HANG_PAUSED &&
            !(last_state==GW_HANG_PAUSED && state==GW_HANG_RUNNING)) {
            char line[256];snprintf(line,sizeof line,"gw: watchdog: %s presenting_age=%.1f logic_age=%.1f pause_flags=%ld\r\n",hb_state_name(state),pa,la,pause);
            hb_write("melee-pc.log",line,1);
        }
        last_state=state;
        if(state==GW_HANG_MAIN && !diagnosed) {
            char where[512],line[2048];uintptr_t pc=hb_pc();
            diagnosed=1;last_state=state;
            gw_hang_describe_addr(pc,where,sizeof where);
            hb_symbol(where,sizeof where);
            snprintf(line,sizeof line,"gw: watchdog: HUNG main_thread=%lu pc=%s scene=%ld main_age=%.1f presenting=%u age=%.1f logic=%lu age=%.1f paused=%ld hidden=%d last_shim=%s script=%s callback=%s active=%d instructions=%lu deadline=%s action=%s\r\n",
                hb_main_id,where,hb_read(&hb_scene),ma,present,pa,(unsigned long)logic,la,pause,w.hidden||w.minimized,
                (const char *)InterlockedCompareExchangePointer((void *volatile *)&hb_shim,NULL,NULL),owner,callback,active,(unsigned long)hb_read(&hb_instructions),deadline,hb_action);
            hb_write("hang.txt",line,0);hb_write("melee-pc.log",line,1);
            if(strcmp(hb_action,"log")!=0)hb_dump_bounded();
            if(strcmp(hb_action,"exit")==0) {
                gw_hang_final("watchdog main-loop timeout",GW_HANG_EXIT);
                TerminateProcess(GetCurrentProcess(),GW_HANG_EXIT);
            }
        }
    }
    return 0;
}
void gw_hang_start(void) {
    FILETIME create,exit,kernel,user;ULARGE_INTEGER u;const char *v;
    static const char *const disc_keys[]={"MELEE_ISO","GW_ISO_VANILLA","GW_ISO_AKANEIA","GW_ISO_ACE","GW_ISO"};
    unsigned i;
    if(hb_stop)return;
    for(i=0;i<5;++i) {v=getenv(disc_keys[i]);snprintf(hb_private_discs[i],sizeof hb_private_discs[i],"%s",v?v:"");}
    hb_main_id=GetCurrentThreadId();hb_start=hb_wall();
    hb_load_map();
    hb_dwm=LoadLibraryA("dwmapi.dll");
    if(hb_dwm)hb_dwm_attr=(HbDwmAttr)GetProcAddress(hb_dwm,"DwmGetWindowAttribute");
    if(GetProcessTimes(GetCurrentProcess(),&create,&exit,&kernel,&user)) {
        u.LowPart=create.dwLowDateTime;u.HighPart=create.dwHighDateTime;hb_created=u.QuadPart;
    }
    v=getenv("MELEE_WATCHDOG_SECS");
    if(v){char *end;double n=strtod(v,&end);if(end!=v && !*end && n>=0 && n<=86400)hb_seconds=n;}
    v=getenv("MELEE_RUN_LABEL");snprintf(hb_label,sizeof hb_label,"%s",v?v:"interactive");
    v=getenv("MELEE_WATCHDOG_ACTION");
    if(!v || (strcmp(v,"log") && strcmp(v,"dump") && strcmp(v,"exit"))) {
        const char *unattended=getenv("MELEE_UNATTENDED");v=unattended && strcmp(unattended,"1")==0?"exit":"log";
    }
    snprintf(hb_action,sizeof hb_action,"%s",v);atexit(hb_atexit);
    if(!DuplicateHandle(GetCurrentProcess(),GetCurrentThread(),GetCurrentProcess(),&hb_main,0,FALSE,DUPLICATE_SAME_ACCESS))return;
    hb_stop=CreateEventA(NULL,TRUE,FALSE,NULL);
    if(hb_stop)hb_worker=CreateThread(NULL,0,hb_run,NULL,0,NULL);
    if(!hb_worker)hb_write("melee-pc.log","gw: watchdog: heartbeat thread unavailable\r\n",1);
}
void gw_hang_test_stall(unsigned seconds) { Sleep(seconds*1000); }

/* Real heartbeat thread exercised without a renderer or disc by the native
 * suite. Temporarily restart with action=log, stall this test's main loop, then
 * restore the production policy. Never change policy while a worker reads it. */
static int hb_thread_test(void) {
    double saved_seconds=hb_seconds;char saved_action[16],json[4096]={0},hang[2048]={0};
    LONG saved_pause=hb_read(&hb_pause);HANDLE f;DWORD done;int failed=0;
    if(!hb_worker || IsDebuggerPresent())return 0;
    snprintf(saved_action,sizeof saved_action,"%s",hb_action);
    SetEvent(hb_stop);WaitForSingleObject(hb_worker,5000);CloseHandle(hb_worker);
    ResetEvent(hb_stop);hb_seconds=1;snprintf(hb_action,sizeof hb_action,"log");
    gw_hang_pause(1);gw_hang_tick();
    hb_worker=CreateThread(NULL,0,hb_run,NULL,0,NULL);
    if(!hb_worker)failed=1;
    else {
        Sleep(2300);
        f=CreateFileA("heartbeat.json",GENERIC_READ,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,NULL,OPEN_EXISTING,0,NULL);
        if(f==INVALID_HANDLE_VALUE)failed=1;
        else {ReadFile(f,json,sizeof json-1,&done,NULL);CloseHandle(f);}
        f=CreateFileA("hang.txt",GENERIC_READ,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,NULL,OPEN_EXISTING,0,NULL);
        if(f==INVALID_HANDLE_VALUE)failed=1;
        else {ReadFile(f,hang,sizeof hang-1,&done,NULL);CloseHandle(f);}
        failed|=!strstr(json,"\"pid\":") || !strstr(json,"\"process_created\":") ||
            !strstr(json,"\"presented_frames\":") || !strstr(json,"\"logic_frames\":") ||
            !strstr(json,"\"script_watchdog\":") || !strstr(json,"\"paused\":true") ||
            !strstr(json,"main-loop-stalled") || !strstr(hang,"HUNG main_thread=") || !strstr(hang,"action=log");
        SetEvent(hb_stop);WaitForSingleObject(hb_worker,5000);CloseHandle(hb_worker);
    }
    hb_seconds=saved_seconds;snprintf(hb_action,sizeof hb_action,"%s",saved_action);
    gw_hang_pause(saved_pause);gw_hang_tick();ResetEvent(hb_stop);
    hb_worker=CreateThread(NULL,0,hb_run,NULL,0,NULL);
    if(failed)gw_test_fail("heartbeat fields/intent and log-only stalled-main diagnosis failed");
    return failed;
}
#else
void gw_hang_start(void) {}
void gw_hang_tick(void) {}
void gw_hang_logic(void) {}
void gw_hang_pause(int flags) { (void)flags; }
void gw_hang_scene(int scene) { (void)scene; }
void gw_hang_transition(int phase) { (void)phase; }
void gw_hang_shim(const char *name) { (void)name; }
void gw_hang_callback(const char *o,const char *n,int a) { (void)o;(void)n;(void)a; }
void gw_hang_instructions(void) {}
void gw_hang_script_state(int p,int e,const char *n) { (void)p;(void)e;(void)n; }
void gw_hang_log_time(void) {}
void gw_hang_final(const char *reason,unsigned code) { (void)reason;(void)code; }
void gw_hang_test_stall(unsigned seconds) { (void)seconds; }
#endif
static int hb_policy_test(void) {
    int failed=gw_hang_classify(20,20,20,10,0,0,0,0)!=GW_HANG_MAIN ||
        gw_hang_classify(0,20,20,10,1,0,0,0)!=GW_HANG_PAUSED ||
        gw_hang_classify(0,20,20,10,0,0,0,1)!=GW_HANG_HIDDEN ||
        gw_hang_classify(20,20,20,10,0,1,0,0)!=GW_HANG_DEBUGGER ||
        gw_hang_classify(20,20,20,10,0,0,1,0)!=GW_HANG_MODAL;
    GwHangPauseLog pauses={0};char clean[256];
    failed|=gw_hang_expected(GW_HANG_LOGIC,1,10)!=GW_HANG_TRANSITION ||
        gw_hang_expected(GW_HANG_MAIN,1,10)!=GW_HANG_MAIN ||
        gw_hang_expected(GW_HANG_LOGIC,1,30)!=GW_HANG_LOGIC;
    failed|=gw_hang_pause_log(&pauses,1,1)!=1;
    failed|=gw_hang_pause_log(&pauses,1,2)!=0;
    gw_hang_pause_log(&pauses,0,3);
    failed|=gw_hang_pause_log(&pauses,1,4)!=0;
    failed|=!(gw_hang_pause_log(&pauses,0,32)&2) || pauses.episodes!=2;
    gw_disc_redact(clean,sizeof clean,"C:/private/a.iso C:\\private\\a.iso C:\\\\private\\\\a.iso","C:\\private\\a.iso");
    failed|=strcmp(clean,"<disc> <disc> <disc>")!=0;
    if(failed)gw_test_fail("hang progress/intent/pause/privacy policy differs");return failed;
}
void gw_hang_tests_register(void) {
    gw_test_register("hang_progress_policy",hb_policy_test);
#ifdef _WIN32
    gw_test_register("hang_heartbeat_stalled_loop",hb_thread_test);
#endif
}
