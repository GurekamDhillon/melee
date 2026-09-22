/* gw_lua.c - Lua 5.4.7 as one native translation unit (pc/third_party/lua-5.4.7, MIT).
 *
 * One object instead of thirty keeps the link lists short; the Lua sources are vendored
 * unmodified and included here in the order Lua's own onelua.c uses. Only the libraries the
 * script sandbox (gw_script.c) opens are compiled in: no io, os, package/loadlib or debug
 * library, so no script can reach the file system, the network or native code through Lua
 * itself - that is enforced by their absence, not only by not registering them.
 *
 * Lua errors unwind with longjmp; every call into Lua from the port goes through lua_pcall.
 */
#define LUA_CORE
#define LUA_LIB
#define LUA_USE_WINDOWS /* use the Windows code paths in luaconf.h (still C89-flavoured) */
#if defined(__clang__)
#pragma clang diagnostic ignored "-Wimplicit-fallthrough"
#endif

/* core */
#include "../third_party/lua-5.4.7/src/lzio.c"
#include "../third_party/lua-5.4.7/src/lctype.c"
#include "../third_party/lua-5.4.7/src/lopcodes.c"
#include "../third_party/lua-5.4.7/src/lmem.c"
#include "../third_party/lua-5.4.7/src/lundump.c"
#include "../third_party/lua-5.4.7/src/ldump.c"
#include "../third_party/lua-5.4.7/src/lstate.c"
#include "../third_party/lua-5.4.7/src/lgc.c"
#include "../third_party/lua-5.4.7/src/llex.c"
#include "../third_party/lua-5.4.7/src/lcode.c"
#include "../third_party/lua-5.4.7/src/lparser.c"
#include "../third_party/lua-5.4.7/src/ldebug.c"
#include "../third_party/lua-5.4.7/src/lfunc.c"
#include "../third_party/lua-5.4.7/src/lobject.c"
#include "../third_party/lua-5.4.7/src/ltm.c"
#include "../third_party/lua-5.4.7/src/lstring.c"
#include "../third_party/lua-5.4.7/src/ltable.c"
#include "../third_party/lua-5.4.7/src/ldo.c"
#include "../third_party/lua-5.4.7/src/lvm.c"
#include "../third_party/lua-5.4.7/src/lapi.c"

/* the libraries the sandbox opens (base, coroutine, table, string, utf8, math) */
#include "../third_party/lua-5.4.7/src/lauxlib.c"
#include "../third_party/lua-5.4.7/src/lbaselib.c"
#include "../third_party/lua-5.4.7/src/lcorolib.c"
#include "../third_party/lua-5.4.7/src/ltablib.c"
#include "../third_party/lua-5.4.7/src/lstrlib.c"
#include "../third_party/lua-5.4.7/src/lutf8lib.c"
#include "../third_party/lua-5.4.7/src/lmathlib.c"
