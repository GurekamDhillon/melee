/* One unity translation unit of the player / bonus code, so a state oracle can run plbonus.c and plbonuslib.c against the
 * real player.c / plattack.c accessors they call (the Rust ports call those directly, not through a hook). */
#include "melee/pl/player.c"
#include "melee/pl/plattack.c"
#include "melee/pl/plbonus.c"
#include "melee/pl/plbonuslib.c"
#include "melee/ft/ft_0892.c"

/* pltrick.c accessors the Rust ports call natively (crate::tu::pltrick, not through a hook): upstream made them real
 * functions in pltrick.c (363720e12 sync), so the unity carries their bodies instead of a recording stub. */
unsigned int pl_800386D8(plActionStats* arg0, ssize_t arg1)
{
    return arg0->x358.x0.by_attack_hi[arg1];
}

int pl_800386E8(pl_800386E8_arg0_t* arg0)
{
    int temp = arg0->unk_190 + arg0->unk_5AC;
    return temp + arg0->unk_5B0;
}
