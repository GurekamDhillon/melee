/* ftparts.c with the real HSD class machinery and the JObj / PObj class tables it chains to, so the fighter-class functions
 * (info inits, JObj / PObj matrix setup, the IntpJObj load) run against the code the Rust port reaches through melee_runtime. */
#include "melee/ft/ftparts.c"
#include "sysdolphin/baselib/class.c"
#define default_class jobj_default_class
#include "sysdolphin/baselib/jobj.c"
#undef default_class
#define default_class pobj_default_class
#include "sysdolphin/baselib/pobj.c"
#undef default_class
#include "gameworld/mtx_pc.c"
