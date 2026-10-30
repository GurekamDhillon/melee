/* One unity translation unit of the player / bonus code, so a state oracle can run plbonus.c and plbonuslib.c against the
 * real player.c / plattack.c accessors they call (the Rust ports call those directly, not through a hook). */
#include "melee/pl/player.c"
#include "melee/pl/plattack.c"
#include "melee/pl/plbonus.c"
#include "melee/pl/plbonuslib.c"
#include "melee/ft/ft_0892.c"
