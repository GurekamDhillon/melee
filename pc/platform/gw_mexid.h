/* gw_mexid.h - content identities for netplay: which fighters and stages two installs have IN
 * COMMON, whatever their mods, and whether the global game data that the whole simulation reads
 * agrees.
 *
 * WHY: two players may run different mods. Refusing every mismatch would make mods poison online
 * play, so instead:
 *   - every fighter and every selectable stage gets a CONTENT IDENTITY, a 64-bit hash over the
 *     files that define it (read through the file layer, so a stage baked into a mod ISO and the
 *     same stage as a loose mod hash the same - file NAMES and the mod/ISO it came from are not
 *     part of it; see below for exactly which files);
 *   - both sides exchange their lists (kind, identity, local id) and play only the INTERSECTION,
 *     mapped to each side's own local ids (CharacterKinds and external stage ids differ between
 *     installs);
 *   - match setup names m-ex content by identity ("id:<16 hex>" in the scene string), so each side
 *     resolves it to its own local id;
 *   - only GLOBAL data (common fighter physics in PlCo.dat, common items in ItCo.dat, the port's
 *     m-ex feature flags) must match exactly - that alone refuses a connection.
 *
 * IDENTITY INPUTS (content hashes, in a fixed order; a missing file hashes as "absent"):
 *   fighter  Pl<xx>.dat (attributes, subactions, the m-ex ftFunction code, article data) and
 *            Pl<xx>AJ.dat (animations: they move the hitboxes), plus the partner's pair for
 *            Zelda/Sheik and Ice Climbers, plus the Kirby copy file for an m-ex fighter.
 *            NOT included (cosmetic): costume files, effect banks, sound banks, CSS art.
 *   stage    its Gr file(s): the .dat, and for a prefix name ("/GrPs") every <prefix>.dat and
 *            <prefix>1..9.dat present. NOT included: music, SSS art, .usd language variants.
 *   global   PlCo.dat's 21 global tables (all but the two per-kind tables, see gw_mexid.c) at
 *            their vanilla sizes; ItCo.dat whole; MELEE_MEX + <exe>/mods/mex.txt.
 *
 * All ints and const char* so gwtool-compiled game code (alpha's online CSS/SSS) can call these
 * without the gw_ prefix, e.g. `extern int MexId_OnlineFighter(int ck);`.
 */
#ifndef GW_MEXID_H
#define GW_MEXID_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define GW_MEXID_FIGHTER 1
#define GW_MEXID_STAGE 2

/* ---- this install ----------------------------------------------------------------------------
 * The table is built once, on first use (it reads every fighter's and stage's files: about a
 * second or two for a full m-ex roster, less once the OS has the files cached). */
int gw_MexId_Count(void);                  /* fighters + stages */
int gw_MexId_Kind(int i);                  /* GW_MEXID_FIGHTER / GW_MEXID_STAGE, 0 = bad index */
int gw_MexId_LocalId(int i);               /* CharacterKind (fighter) / external stage id (stage) */
const char *gw_MexId_Name(int i);          /* display name */
const char *gw_MexId_HashHex(int i);       /* 16 hex digits */
uint64_t gw_MexId_Hash(int i);
int gw_MexId_FindFighter(int ck);          /* table index of CharacterKind ck, or -1 */
int gw_MexId_FindStage(int ext);           /* table index of external stage id, or -1 */
/* Scene-string token for a fighter / stage: "id:<16 hex>" when it has an identity, else the plain
 * "ck:N" / "ext:N". The string is valid until the next call of the same function. */
const char *gw_MexId_TokenForCk(int ck);
const char *gw_MexId_TokenForExt(int ext);
/* "id:<16 hex>" -> this install's CharacterKind / external stage id, or -1 (not installed). */
int gw_MexId_CkForHex(const char *hex16);
int gw_MexId_ExtForHex(const char *hex16);
/* How many "id:" fighters/stages in a scene string this install cannot resolve (0 = playable);
 * `why` lists them. A guest checks the host's agreed match with this before loading it. */
int gw_MexId_SceneCheck(const char *scene, char *why, int cap);

/* ---- global game data --------------------------------------------------------------------- */
uint64_t gw_MexId_GlobalHash(void);        /* 0 is never returned */
const char *gw_MexId_GlobalDescribe(void); /* "plco#hhhh,itco#hhhh,mexflags#hhhh" */
/* Name what differs between the peer's GlobalDescribe and ours. Returns the number of differing
 * components (0 = same). */
int gw_MexId_GlobalDiff(const char *peer_desc, char *out, int cap);

/* ---- the peer (online) ---------------------------------------------------------------------
 * Wire: the list goes over gw_net's lobby channel in chunks of <= GW_NET_LOBBY_MAX bytes, each
 * starting "MXE". gw_netplay.c pumps WireNext into gw_net_lobby_send and hands every received
 * lobby message to WireFeed first. */
void gw_MexId_PeerReset(void);             /* new connection: forget the peer, restart sending */
int gw_MexId_WireNext(uint8_t *out, int cap); /* next chunk to send (bytes), 0 = all sent */
void gw_MexId_WireSent(void);              /* the chunk WireNext produced was accepted */
int gw_MexId_WireFeed(const uint8_t *msg, int len); /* 1 = it was ours (consumed), 0 = not ours */
int gw_MexId_PeerReady(void);              /* 1 = the peer's whole list has arrived */
int gw_MexId_PeerCount(void);              /* entries the peer sent */

/* Online availability: 1 = both have it (same identity), 0 = not common, -1 = peer list not here
 * yet. Vanilla content is common whenever its files are unmodified on both sides. */
int gw_MexId_OnlineFighter(int ck);
int gw_MexId_OnlineStage(int ext);
/* Mapping between the two installs' local ids, -1 when not common or not known yet. */
int gw_MexId_PeerCkForLocal(int ck);
int gw_MexId_LocalCkForPeer(int peer_ck);
int gw_MexId_PeerExtForLocal(int ext);
int gw_MexId_LocalExtForPeer(int peer_ext);
int gw_MexId_CommonFighterCount(void);     /* -1 until the peer list is here */
int gw_MexId_CommonStageCount(void);

void gw_mexid_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_MEXID_H */
