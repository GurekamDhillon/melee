/* Redact before a diagnostic reaches stdout, disk, or the crash-report buffer. */
#ifndef GW_DISC_PRIVACY_H
#define GW_DISC_PRIVACY_H
#include <ctype.h>
#include <stdlib.h>
#include <string.h>
static inline void gw_disc_redact(char *out, size_t cap, const char *in, const char *secret) {
    size_t n=0;
    while (*in && n+1<cap) {
        const char *a=in, *b=secret;
        if (b && *b) {
            while (*a && *b) {
                int x=tolower((unsigned char)*a), y=tolower((unsigned char)*b);
                if (x=='\\') x='/';
                if (y=='\\') y='/';
                if (x!=y) break;
                /* JSON's escaped backslashes represent one separator. */
                if (*a=='\\' && a[1]=='\\') ++a;
                ++a; ++b;
            }
            if (!*b) {
                const char *r="<disc>";
                while (*r && n+1<cap) out[n++]=*r++;
                in=a; continue;
            }
        }
        out[n++]=*in++;
    }
    if (cap) out[n]=0;
}
extern const char *gw_iso_path(void);
static inline void gw_disc_clean(char *out, size_t cap, const char *in) {
    static const char *const keys[]={"MELEE_ISO","GW_ISO_VANILLA","GW_ISO_AKANEIA","GW_ISO_ACE","GW_ISO"};
    char tmp[8192]; size_t i;
    gw_disc_redact(out,cap,in,gw_iso_path());
    for (i=0;i<sizeof keys/sizeof keys[0];++i) {
        gw_disc_redact(tmp,sizeof tmp,out,getenv(keys[i]));
        if (cap) {strncpy(out,tmp,cap-1);out[cap-1]=0;}
    }
}
#endif
