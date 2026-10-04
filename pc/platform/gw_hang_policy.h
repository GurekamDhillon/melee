/* Pure diagnostics policy, shared by native and standalone tests. */
#ifndef GW_HANG_POLICY_H
#define GW_HANG_POLICY_H
enum { GW_HANG_RUNNING, GW_HANG_MAIN, GW_HANG_LOGIC, GW_HANG_PRESENT,
       GW_HANG_PAUSED, GW_HANG_HIDDEN, GW_HANG_DEBUGGER, GW_HANG_MODAL, GW_HANG_DISABLED,
       GW_HANG_TRANSITION };
static inline int gw_hang_classify(double main_age, double logic_age, double present_age,
                                   double seconds, int paused, int debugger, int modal, int hidden) {
    if (seconds <= 0) return GW_HANG_DISABLED;
    if (debugger) return GW_HANG_DEBUGGER;
    if (modal) return GW_HANG_MODAL;
    /* A remembered pause is never evidence that a dead main loop is healthy. */
    if (main_age >= seconds) return GW_HANG_MAIN;
    if (hidden) return GW_HANG_HIDDEN;
    if (paused) return GW_HANG_PAUSED;
    if (logic_age >= seconds) return GW_HANG_LOGIC;
    if (present_age >= seconds) return GW_HANG_PRESENT;
    return GW_HANG_RUNNING;
}
static inline int gw_hang_expected(int state, int transitioning, double age) {
    /* A bounded loading grace never hides a dead main loop or debugger/modal state. */
    return transitioning && age >= 0 && age < 30 &&
        (state==GW_HANG_RUNNING || state==GW_HANG_LOGIC || state==GW_HANG_PRESENT)
        ? GW_HANG_TRANSITION : state;
}
typedef struct { double next, started, seconds; unsigned episodes; int paused; } GwHangPauseLog;
static inline int gw_hang_pause_log(GwHangPauseLog *p, int paused, double now) {
    int result=0;
    if (paused && !p->paused) {
        p->started=now; ++p->episodes;
        if (now>=p->next && p->episodes==1 && p->seconds==0) {result=1;p->next=now+30;}
    }
    if (p->paused) p->seconds+=now-p->started;
    if (paused) p->started=now;
    p->paused=paused;
    if (now>=p->next && (p->episodes || p->seconds>0)) result|=2;
    return result;
}
static inline int gw_deadline_expired(unsigned waited, unsigned limit, int reported) {
    return !reported && waited >= limit;
}
#endif
