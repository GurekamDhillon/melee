/* geno_define_online.h - what netplay needs to know about a native define (Geno slice 7).
 *
 * gw_mexid.c gives each define an ONLINE IDENTITY: the entry's content id (the whole entry, overlay words, Lua module)
 * mixed with the bytes of the resource files the simulation reads. The registry says which files those are; mexid hashes
 * them through the file layer (the same way it hashes a fighter's Pl file). Plain ints and strings only. */
#ifndef GENO_DEFINE_ONLINE_H
#define GENO_DEFINE_ONLINE_H

#include <stdint.h>

#define GENO_ONLINE_MAX_FILES 24

typedef struct GenoDefineOnline {
    int ck;                       /* the resident alias on this install (CharacterKind) */
    uint64_t id;                  /* the entry's content id (geno_registry.c), not the alias */
    char key[40];
    char name[48];
    int none;                     /* base "none": the files below are the package's own */
    int nfiles;
    char files[GENO_ONLINE_MAX_FILES][64]; /* resource files whose bytes join the identity, in a fixed order */
} GenoDefineOnline;

/* 1 and *out filled for define number `index` (0..gw_Geno_DefineCount()-1), 0 otherwise. */
int gw_Geno_DefineOnlineInfo(int index, GenoDefineOnline* out);

#endif
