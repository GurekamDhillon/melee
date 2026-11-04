/* Geno fighter Lua (melee_script::geno::lua, the port of pc/platform/geno_lua_core.h): the one part of the domain that
 * needs the interpreter's internals. A module function may not keep state: its bytecode, and that of every function
 * nested in it, must contain no OP_SETUPVAL (assigns a captured variable) and no OP_SETTABUP (assigns a global or a
 * captured table's field). Compiled with the vendored 5.4.7 sources, so the Proto layout is that version's. */
#include "lua.h"
#include "lobject.h"
#include "lopcodes.h"

/* 0 ok, 1 OP_SETUPVAL, 2 OP_SETTABUP. */
static int geno_scan_proto(const Proto* p)
{
    int i, r;
    for (i = 0; i < p->sizecode; i++) {
        OpCode op = GET_OPCODE(p->code[i]);
        if (op == OP_SETUPVAL) return 1;
        if (op == OP_SETTABUP) return 2;
    }
    for (i = 0; i < p->sizep; i++)
        if ((r = geno_scan_proto(p->p[i])) != 0) return r;
    return 0;
}

/* The Lua closure at idx (the caller has checked it is one): its pointer identity for the seen set via
 * lua_topointer, and its prototype scan. */
int geno_lua_scan_closure(lua_State* L, int idx)
{
    const LClosure* cl = (const LClosure*) lua_topointer(L, idx);
    return geno_scan_proto(cl->p);
}
