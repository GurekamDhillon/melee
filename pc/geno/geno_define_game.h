#ifndef GENO_DEFINE_GAME_H
#define GENO_DEFINE_GAME_H
#include <melee/ft/forward.h>
/* Scalar shims; definitions are fixed for this process and offline only. */
extern int Geno_DefineCount(void);
extern int Geno_DefineCKAt(int index);
extern int Geno_DefineBaseKind(int kind);
extern int Geno_DefineBaseCK(int ck);
extern int Geno_DefineName(int ck, char* out, int cap);
extern int Geno_DefineIdentityWord(int kind, int high);
/* slice 2 (D8): digest of a defined fighter's GenoState for RB_GameHash; 0 for any other fighter */
extern u32 GenoDefine_StateDigest(Fighter* fp);
void GenoDefine_InitKinds(void);
void GenoDefine_CaptureMario(void);
void GenoDefine_ResetDescriptors(void);
void GenoDefine_Load(int kind, int base);
void GenoDefine_BindFighter(Fighter* fp);
void GenoGame_DefineApplyRows(int kind, MotionState* common, MotionState* specific, int subactions);
void GenoGame_ClearProfileStorage(void);
#endif
