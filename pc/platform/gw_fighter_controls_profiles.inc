/* Included by the physical-profile runtime and its standalone regression test.
 * Reusing a slot reclaims its old fighter keys from the finite settings table. */
#ifndef GW_FIGHTER_CONTROLS_PROFILES_INC
#define GW_FIGHTER_CONTROLS_PROFILES_INC
#include "gw_settings.h"
static int fc_profile_forget(int device, int profile)
{
    char prefix[32];
    if(device<0 || device>=16 || profile<0 || profile>=4) return 0;
    snprintf(prefix,sizeof prefix,"fc_d%d_p%d_s",device,profile);
    if(!gw_Settings_RemovePrefix(prefix)) return 0;
    gw_log("fighter controls: cleared device %d profile %d preferences",device,profile+1);
    return 1;
}
#endif
