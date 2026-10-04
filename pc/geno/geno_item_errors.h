#ifndef GENO_ITEM_ERRORS_H
#define GENO_ITEM_ERRORS_H
/* Negative spawn results; positive values are handles. Shared with Lua adapter. */
enum GenoItemSpawnError {
 GENO_ITEM_ERR_UNSUPPORTED=-1, GENO_ITEM_ERR_UNSAFE=-2,
 GENO_ITEM_ERR_UNINITIALIZED=-3, GENO_ITEM_ERR_POOL=-4,
 GENO_ITEM_ERR_CAP=-5, GENO_ITEM_ERR_CONFLICT=-6
};
#endif
