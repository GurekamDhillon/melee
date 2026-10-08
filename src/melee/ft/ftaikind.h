#ifndef MELEE_FT_FTAIKIND_H
#define MELEE_FT_FTAIKIND_H

#include <melee/ft/forward.h>

/* The kind the CPU's AI reads a fighter as (Geno slice 6, geno 10). Everything in the CPU code that asks "which fighter is this?" (the per-kind
 * switches that choose a recovery, a special move or a ranged attack, and the per-kind AI tables) goes through this. A retail or m-ex fighter is
 * itself; a define that declared "ai": {"like": "<retail fighter>"} is that retail fighter, so its CPU gets that fighter's moves and tables.
 * Without the key a define stays itself, and every switch takes its default arm, as before. */
#if defined(TARGET_PC)
extern int Geno_DefineAiLike(int kind);
static inline int ftAi_Kind(int kind)
{
    int like;
    if (kind < Ft_Kind_Mex0) {
        return kind;
    }
    like = Geno_DefineAiLike(kind);
    return like >= 0 ? like : kind;
}
#define FTAI_KIND(fp) ftAi_Kind((int) (fp)->kind)
#else
#define FTAI_KIND(fp) ((fp)->kind)
#endif

#endif
