#ifndef GENO_ANIM_BANK_H
#define GENO_ANIM_BANK_H
#include <stdint.h>
/* Native authoring API. One validated bank may await the next empty fighter
 * list; the game boundary copies bytes, never exposes a host pointer. */
int gw_Geno_ReloadAnimBank(const char* define, char* why, int cap);
int gw_Geno_AnimBankPhase(void); /* 0 idle, 1 pending, 2 ready, 3 refused */
const char* gw_Geno_AnimBankStatus(void);
void gw_Geno_AnimBankCancel(void);
void gw_Geno_AnimBankBeforeSpawn(void);
uint64_t gw_Geno_AnimBankDigest(void); /* LAB state compatibility only */
#endif
