/* TU: src/melee/gm/gm_1601.c */
/* HSFIX: MatchEnd.loser = (u32) (u8) ((p->loser >> 4) | (p->loser << 4)) */
/* HS: MatchEnd VsSceneState GameRules FighterData NameTagData HSD_Text VsModeData SSSData PlayerInitData StartMeleeRules datetime gm_801677C0_s lbl_8046B378_t lbl_8046B488_t OSCalendarTime Vec3 GmStats */
/* STUBS: lbAudioAx_80024614 lbAudioAx_800245F4 gmMainLib_8015ED74 Mex_VictoryThemeForPortCKind Mex_ResultFileForPortCKind:ret=str */
/* STUBS: Toy_8030813C:manual Geno_DefineBaseCK:ret=m1:3 Geno_DefineBaseKind:ret=m1:3 Mex_InternalForPortKind:ret=m1:30 Mex_FtBaseKind:ret=small:30 */
/* STUBS: Player_800325C8:ret=small:40 lbLang_IsSavedLanguageUS Player_GetPlayerSlotType:ret=small:5 Player_GetPlayerCharacter:ret=small:33 */
/* STUBS: Player_80036394:ret=small:40 HSD_SisLib_803A6B98:manual;proto=int(HSD_Text*,float,float,const~char*) HSD_SisLib_803A7548 */
/* STUBS: HSD_SisLib_803A70A0:manual;proto=s32(HSD_Text*,s32,char*) gmMainLib_8015EDBC:ret=pool:512 GetPersistentFighterData:manual;proto=struct~FighterData*(SelectableCharacterKind) */
/* STUBS: GetPersistentNameData:manual gmMainLib_8015CCF0:ret=pool:8 gmMainLib_8015CCFC:ret=pool:8 gmMainLib_8015CFB4:ret=pool:8 */
/* STUBS: gmMainLib_GetMatchResetCounter:ret=pool:8 gm_GetCurrentGameMode:manual gmMainLib_GetStaminaMatchTotal:ret=pool:8 gmMainLib_GetTimeMatchTotal:ret=pool:8 */
/* STUBS: gmMainLib_GetStockMatchTotal:ret=pool:8 gmMainLib_GetCoinMatchTotal:ret=pool:8 gmMainLib_GetBonusMatchTotal:ret=pool:8 */
/* STUBS: gmMainLib_8015ED98:ret=pool:64 gmMainLib_8015EDB0:ret=pool:64 gmMainLib_GetVsPlayContestants:ret=pool:8 gmMainLib_GetVsPlayTime:ret=pool:8 */
/* STUBS: gmMainLib_GetCombinedVSPlayTime:ret=pool:8 gmMainLib_GetSingleplayerTime:ret=pool:8 gmMainLib_8015CD80:ret=pool:8 gmMainLib_GetTotalDamage:ret=pool:8 */
/* STUBS: gmMainLib_GetKOTotal:ret=pool:8 gmMainLib_8015EDC8:ret=pool:16 gmMainLib_GetSelfDestructTotal:ret=pool:8 gmMainLib_GetPowerCount:ret=pool:8 */
/* STUBS: gmMainLib_8015D0C0:ret=pool:8:32 gmMainLib_8015D0D8:ret=bit gmMainLib_8015D194:ret=pool:8:32 gmMainLib_8015D1E8:ret=pool:8:32 gmMainLib_8015D200:ret=bit */
/* STUBS: gmMainLib_8015D2BC:ret=pool:8:32 gmMainLib_8015D310:ret=pool:8:32 gmMainLib_8015D328:ret=bit gmMainLib_8015D3E4:ret=pool:8:32 */
/* STUBS: gmMainLib_8015D06C:ret=pool:8:32 gmMainLib_8015D7EC:ret=pool:8:32 gmMainLib_8015D48C:ret=bit gmMainLib_8015D438:ret=pool:8:32 gmMainLib_8015D6BC:ret=bit */
/* STUBS: gmMainLib_8015D6A4:ret=pool:8:32 gmMainLib_8015D710:ret=bit gmMainLib_8015D6F8:ret=pool:8:32 gmMainLib_8015D764:ret=bit gmMainLib_8015D74C:ret=pool:8:32 */
/* STUBS: gmMainLib_8015D7A4:ret=pool:8:32 gmMainLib_8015D7BC:ret=pool:8:32 gmMainLib_8015D7D4:ret=pool:8:32 gmMainLib_8015CEFC:ret=bit */
/* STUBS: gmMainLib_GetGamePrefs:ret=pool:128 gmMainLib_8015EE44:ret=bit gmMainLib_8015EDA4:ret=pool:8 Stage_8022519C:ret=small:50 Mex_Enabled:ret=bit */
/* STUBS: gmMainLib_8015D818 gmMainLib_GetUnlockedCharactersBitmaskPtr:ret=pool:8 gmMainLib_8015D94C:ret=bit Ground_EnableMatchCamera */
/* STUBS: Camera_SetUpPauseCameraWithDefaultZoom Player_GetEntity:ret=ptrbit Camera_8002F760 Camera_8002F784 Camera_8002FC7C Camera_SetModeToFixed Camera_8002F9E4 */
/* STUBS: gm_801A4BB8 hsd_80398310:ret=ptrbit hsd_80392528:manual HSD_GObjFree pl_80039450 fn_8016FFD4 memzero:manual gm_GetFrameCount fn_8016B728 */
/* STUBS: Player_GetStocks Player_GetCostumeId Player_GetNametagSlotID Player_GetCoins Player_GetMatchFrameCount Player_GetSelfDestructs Player_GetFalls */
/* STUBS: Player_GetTeam:ret=small:5 Player_GetDamage Player_800353BC Player_GetMoreFlagsBit2 Player_GetRemainingHP Player_GetJoystickCountByIndex fn_8016B5B0:ret=fpos:9 */
/* STUBS: pl_80040948:ret=fpos:9 pl_80040900 pl_80040924 pl_80040B3C pl_80040B64 pl_80040BD8 pl_80040BFC pl_80040C24:ret=fpos:1000 pl_80040C48:ret=fpos:1000 */
/* STUBS: pl_80040C6C:ret=fpos:1000 pl_80040C90:ret=fpos:1000 Player_GetTotalCoins Player_GetUnk98 Player_GetUnk9C pl_80040CB4 pl_80040CD8 pl_80040A04 */
/* STUBS: pl_80040A30 pl_80040A54 pl_80040B18 pl_80040A78 pl_80040A9C pl_80040AF0 pl_8003E39C pl_80040D44:ret=fpos:1000 pl_80040D68:ret=fpos:1000 */
/* STUBS: pl_80040CFC pl_80040D20 pl_80040DB8 Player_GetKOsByPlayerIndex gmVs_GetSceneState:manual gm_GetStKind:ret=small:80 Stage_80224DC8 */
/* STUBS: Ground_801C5774:ret=small:30 Stage_80224E38:manual Ground_801C38BC Camera_8002F3AC gm_IsCurrently1PMode_inline Player_SetSpawnPlatformPos */
/* STUBS: Player_80032FA4 Player_SetSomePos Stage_GetCamBoundsTopOffset Player_80032768 Player_SetFacingDirection Player_SetHPByIndex Player_80032070:boolint */
/* STUBS: ifStatus_802F6508 Camera_800310E8 Player_GetFlagsBit1:ret=small:2 gm_8016A1F8 fn_80169444:boolint ifStatus_802F6788 Player_80031EBC Player_SetSlottype */
/* STUBS: gm_8016A164 gm_8016B094 gm_8016B0E8 gm_8016AC44 fn_80169550 gm_801B97C4:boolint gm_1601_GetUnkData:manual fn_80169574 fn_80169900 fn_8016989C */
/* STUBS: fn_8016A09C fn_80169434:manual GetRumbleSettingOfPort:ret=small:2 lb_80014574 gmMainLib_GetGameRules:manual gmMainLib_8015CE44:manual */
/* STUBS: gmMainLib_8015ED30:ret=small:3 mnSelStageRandom:ret=small:30 mnStageSel_8025BC08:ret=small:80 HSD_Randi:manual HSD_JObjAddAnimAll */
/* STUBS: Mex_InternalCount:ret=small:12 Geno_DefineHasArt Geno_DefineArtBaseCK:ret=m1:3 Mex_AnnouncerForPortCKind:ret=m1:5 lbAudioAx_800243F4 */
/* STUBS: Geno_DefineAudioPlay lbAudioAx_800237A8 lbAudioAx_80023870 lbAudioAx_80026F2C lbAudioAx_8002702C lbAudioAx_80027168 lbAudioAx_80027648 */
/* STUBS: Skins_CostumeCountCK:ret=m1:8 Skins255_TestCount:ret=small:2 ftData_MexInitKinds Geno_DefineCostumeCountCK:ret=small:3 Mex_CostumeInfo:ret=small:9 */
/* STUBS: Skins_TeamCostumeCK:ret=m1:4 Geno_DefineTeamCostume:ret=m1:4 lbTime_8000B028:manual Geno_DefineName Mex_FighterName:ret=str Mex_PortCKindToExt */
/* STUBS: Geno_RecordAdd Geno_RecordCommit lbDvd_8001819C:manual HSD_ArchiveGetPublicAddress:manual */
/* C oracle for src/tu/gm_1601.rs: the whole TU included verbatim (TARGET_PC build); every callee out of it is a recording stub, the
 * pointer-returning save-data getters hand out lazily random scratch memory (pool), and the game state the TU reads is synthetic.
 * One line per case: op case ret state-hash trace. The ops are generated from the OPS block below (opsgen.py). */
#include TU_SRC
#include "tu_common.h"
#include "hs_gm_1601.h"
#include "stubs_gm_1601.h"



/* OPS:
gm_801601C4(s8@-100..100) -> int
gm_80160244(s8@-100..100) -> int
gm_801601C4_inner(s8@-100..100) -> int
gm_801601C4_noinline(s8@-100..100) -> int
gm_80160244_inner(s8@-100..100) -> int
gm_80160244_noinline(s8@-100..100) -> int
gm_801602C0(s8@-100..100) -> void
gm_801603B0() -> void
fn_80160400(CharacterKind=ck) -> u32
gm_80160438(s32=ck) -> str
gm_80160474(CharacterKind=ck, GameModeKind=mode) -> bool
gm_801604DC(CharacterKind=ck, GameModeKind=mode) -> str
gm_80160564(CharacterKind=ck, GameModeKind=mode) -> str
gm_SelKindToUnlockIndex(s32%26) -> u8
gm_CKindToUnlockIndex(s32=ckp) -> s8
get_unlockable_selkind_by_bit_index(s32%13) -> u8
gm_GetCKindByUnlockIndex(s32%13) -> u8
fn_80160710(s32%13) -> u8
gm_GetChallengerStKind(s32%26) -> u32
fn_801607A8(s32%14) -> u8
fn_801607F4(s32%40) -> u8
fn_80160840(u8) -> u8
gm_80160854(u8%6, u8%6, u8%2, u8%5) -> u32
gm_MexVanillaKind(s32%140, s32%2) -> s32
gm_80160968(u32%9) -> gx
gm_80160980(u8=ck) -> str
fn_801609E0(u8=ck) -> str
gm_80160A60(s32%6) -> str
gm_80160B40(&TXT, u8=ck, u8%2) -> void [TXT]
gm_80160C90(&TXT, u8=ck, u8%2) -> void [TXT]
fn_80160DE8({(HSD_JObj*) &TXT|addr_of_mut!(TXT) as *mut HSD_JObj}, u8=ck, s32%7, u8%2, f32, f32) -> void [TXT]
fn_80160F58(u8=ck) -> f32
fn_80161004({&ME|&*addr_of_mut!(ME)}) -> s32 [ME]
fn_80161154(&ME) -> s32 [ME]
fn_80161C90(&ME, s32%6, &GST) -> void [ME GMS]
fn_80161C90_count_players({&ME|&*addr_of_mut!(ME)}) -> u32 [ME]
fn_80162068(&ME) -> void [ME FD]
fn_80162170(&ME) -> void [ME NTD]
gm_801623A4(&ME) -> void [ME FD NTD]
gm_801623D8() -> s32
gm_801623FC(s32) -> void
gm_8016247C(s32) -> s32
gm_80162574(u8=ckp, u8%10) -> void
gm_SetupHumanResultsScreen(u8%4, u8%10) -> void
gm_GetVsPlayMatchTotal() -> u32
gm_80162800(&ME) -> u32 [ME]
gm_SetupResultsScreenPlayTime(u32, u32) -> void
gm_80162968(u32) -> void
gm_801629B4(s32) -> void
gm_GetPlayTime() -> s32
gm_80162A4C(s32) -> void
gm_80162A98(s32) -> ptr
gm_RecordSelfDestructs(s32) -> void
gm_IncrementPowerCount() -> void
gm_80162BD8(u8) -> s32
fn_80162BFC(s8=ckp, s32) -> bool
gm_80162C48() -> s32
fn_80162CCC() -> bool
gm_80162D1C() -> bool
gm_80162D6C() -> u8
gm_80162DD4(u8) -> s32
fn_80162DF8(u8=ckp, u32) -> s32
gm_80162E44() -> s32
gm_80162EC8() -> bool
gm_80162F18() -> bool
gm_80162F68() -> u8
gm_80162FD0(u8) -> s32
fn_80162FF4(u8=ckp, u32) -> bool
gm_80163040() -> s32
fn_801630C4() -> bool
gm_80163114() -> bool
gm_80163164() -> u8
gm_801631CC(u8) -> u32
gm_801631F0() -> u32
gm_80163274(u8) -> u16
gm_80163298(s8=ckp, u16) -> bool
gm_8016332C(u8) -> s32
gm_80163374(u8, po0, po1, po2, po3) -> void [MISC]
gm_801634D4(po0, po1, po2, po3) -> bool [MISC]
gm_8016365C(u8) -> bool
gm_80163690(u8) -> s32
gm_801636D8(u8, po0, po1, po2, po3) -> void [MISC]
gm_80163838(po0, po1, po2, po3) -> bool [MISC]
gm_801639C0(u8) -> bool
gm_801639F4(u8) -> s32
gm_80163A3C(u8, po0, po1, po2, po3) -> void [MISC]
gm_80163B9C(po0, po1, po2, po3) -> bool [MISC]
fn_80163D24() -> bool
fn_80163D74() -> bool
gm_Get3MinMultimanHighscore(u8) -> u16
gm_Get3MinMultimanTotalHighscore() -> s32
gm_Get15MinMultimanHighscore(u8) -> u16
gm_Get15MinMultimanTotalHighscore() -> s32
gm_GetEndlessHighscore(u8) -> s32
gm_GetEndlessTotalHighscore() -> s32
gm_GetCruelHighscore(u8) -> s32
gm_GetCruelTotalHighscore() -> s32
fn_80163FA4(u8) -> s32
gm_SelKindToCKind(u8%25) -> u8
gm_CKindToSelKind(u8=ckp) -> u8
gm_8016403C(u8%64) -> bool
fn_801640B0({&ITEMMASK|addr_of_mut!(ITEMMASK)}) -> void [MISC]
fn_8016419C(u8@1..9) -> f32
fn_801641B4(u8@1..9) -> f32
gm_801641CC(u8%30) -> u16
gm_801641E4(u8%32, u8%2) -> void
gm_IsStageUnlocked(u16%32) -> bool
fn_801642A0() -> bool
gm_80164330(s32%30) -> bool
getStageUnlockIndex(u8) -> u8
getStageUnlockNotifyId(u8%13) -> u8
gm_80164430(u16) -> bool
gm_80164504(u16) -> void
gm_80164600() -> bool
gm_8016468C() -> void
gm_801647D0() -> void
gm_801647F8(u8@0..25) -> s32
gm_IsCKindUnlocked(u8=ckp) -> bool
gm_UnlockCKind(CharacterKind=ckp) -> void
gm_80164A0C(u8=ckp) -> void
gm_80164ABC() -> bool
is_character_unlocked(u8%14, {&MASK16|&*addr_of_mut!(MASK16)}) -> bool [MISC]
fn_80164B48() -> bool
gm_80164F18() -> void
gm_8016505C() -> void
gm_80165084() -> bool
fn_801650E8() -> void
gm_EnablePlayerPauseCamera(s32@-1..5, s32%6) -> void
fn_80165190(s32%6, s32%6) -> void
fn_801651FC(s32%6, s32%6) -> void
gm_80165268(s32) -> void
gm_80165290(s32) -> void
fn_801652B0(s32%8, s32%8) -> void
fn_801652D8() -> ptreq:lbl_8046B378 [PERF GLOB]
gm_80165388(u16, u8, u8, s32) -> ptr
gm_801653C8({(HSD_GObj*) &TXT|addr_of_mut!(TXT) as *mut HSD_GObj}) -> void
fn_801653E8(u8) -> f32
fn_80165418({&ME|&*addr_of_mut!(ME)}) -> s32 [ME]
fn_801654A0({&ME|&*addr_of_mut!(ME)}) -> s32 [ME]
fn_80165548({&ME|&*addr_of_mut!(ME)}, s32%6, s32%5) -> s32 [ME]
fn_801656A8({&ME|&*addr_of_mut!(ME)}, u32%6) -> u32 [ME]
fn_8016588C_clamp(s32@-20000000..20000000) -> s32
fn_8016588C(&ME, s32%6) -> s32 [ME]
fn_80165AC0(&ME) -> selfptr:ME [ME]
fn_80165D60(&ME) -> ptreq:ME [ME]
fn_80165E7C(&ME) -> void [ME]
fn_80165FA4(&ME) -> selfptr:ME [ME]
fn_801661E0(&ME) -> selfptr:ME [ME]
gm_80166378(&ME) -> void [ME]
gm_80166A98(&ME, u8%8, s8=ckn, u8%11, s8=ckn, u8%11, s8=ckn, u8%11, u8=ckn, u8%11) -> selfptr:ME [ME GLOB]
fn_80166CBC({&ME|&*addr_of_mut!(ME)}, s32%6) -> u32 [ME]
gm_80166CCC({&ME|&mut *addr_of_mut!(ME)}, {&ME2|&mut *addr_of_mut!(ME2)}) -> void [ME ME2]
gm_MatchHasMultipleWinners({&ME|&*addr_of_mut!(ME)}) -> bool [ME]
fn_80167194({&ME|&*addr_of_mut!(ME)}) -> s32 [ME]
fn_8016719C(s32%6, s32%3) -> void [VSS GLOB]
gm_80167320(s32%6, bool) -> void [VSS GLOB]
gm_80167470(s32, s32) -> void
gm_801674C4(s8, u8, s8, s8, {(GmRouteCallback) (size_t) (rnd() & 1)|if rnd() & 1 != 0 { Some(cbR as unsafe extern "C" fn(i32) -> i32) } else { None }}) -> void [LBL]
fn_8016758C() -> void [VSS]
fn_80167638(s32%6, &V3A, &V3B) -> s32 [VSS GLOB V3]
gm_801677C0({&G677|&mut *addr_of_mut!(G677)}) -> void [G677]
gm_801677E8(s8) -> void
gm_801677F0() -> u8
gm_RumbleEnabledForPlayer(s32%5, s32@0..120) -> bool [NTD]
gm_80167858(s32%5, s32@0..120, s32, s32) -> void [NTD]
gm_801678F8(s32%5, s32, s32) -> void [NTD]
gm_SetupPlayerDefaults({&PID[0]|addr_of_mut!(PID[0])}) -> void [PID]
gm_SetupAllPlayerDefaults({&PID[0]|PID.as_mut_ptr()}) -> void [PID]
gm_SetupRulesDefaults(&SMR) -> void [PID]
gm_InitVsMode(&VSM) -> void [VSM]
gm_80167BC8(&VSM) -> void [VSM RULES MISC]
gm_80167FC4(&SSS) -> void [SSS RULES GLOB]
gm_801685D4(u8%6, u8@0..130) -> s8 [RULES MISC]
gm_80168638(&ME) -> void [ME RULES MISC]
gm_80168710(&ME, &VSM) -> void [ME VSM]
gm_801688AC({&ME|&*addr_of_mut!(ME)}) -> s32 [ME]
gm_80168940({&ME|&*addr_of_mut!(ME)}) -> s32 [ME]
gm_8016895C({(HSD_JObj*) &TXT|addr_of_mut!(TXT) as *mut HSD_JObj}, &DMD, s32%4) -> void [DMD]
fn_801689E4({(HSD_JObj*) &TXT|addr_of_mut!(TXT) as *mut HSD_JObj}, &DMD, s32%4) -> void [DMD]
gm_MexStockMatAnim() -> ptr [MEX]
gm_MexStockFrame(s32%140, s32%6) -> f32 [MEX]
gm_80168B34(CharacterKind=ck, s32%140, s32@-2..8) -> f32
gm_GenoStockFrame(s32@-2..130, s32@-2..257) -> f32
gm_80168BF8(s32%6) -> f32 [MEX]
gm_80168C5C(u32=ck) -> void
fn_80168E54(s8=ck, s8@0..30, u8%4, u8%2) -> void
fn_80168F2C(s8) -> void
fn_80168F7C() -> void
gm_80168F88() -> void
gm_LoadAnnouncer() -> void
fn_80169000(&ME, {&BYTES4[0]|BYTES4.as_mut_ptr()}) -> void [ME MISC]
gm_GetNumCostumesForCKind(u8=ck) -> u8 [GLOB]
gm_80169264(u8=ck) -> u8
gm_80169290(u8=ck) -> u8
gm_801692BC(u8=ck) -> u8
gm_801692E8(u32, &DT) -> void [DT]
*/

#define NCASES 600

void OSReport(char* fmt, ...) { (void) fmt; }

/* ---- globals the TU reads (defined elsewhere in the game) */
HSD_PadStatus HSD_PadMasterStatus[4];
StageInfo stage_info;
HSD_VIInfo HSD_VIData;
struct UnkCostumeList CostumeListsForeachCharacter[Ft_Kind_Max];

/* ---- the synthetic game state; a case fills the objects its op names (G_*) and the hash covers the same ones */
#define G_ME 1u
#define G_ME2 2u
#define G_VSS 4u
#define G_RULES 8u
#define G_FD 16u
#define G_NTD 32u
#define G_LBL 64u
#define G_TXT 128u
#define G_VSM 256u
#define G_SSS 512u
#define G_PID 1024u
#define G_DT 2048u
#define G_MISC 4096u
#define G_GLOB 8192u
#define G_MEX 16384u
#define G_DMD 32768u
#define G_GMS 65536u
#define G_G677 131072u
#define G_PERF 262144u
#define G_V3 524288u

static MatchEnd ME, ME2;
static VsSceneState VSS;
static GameRules GRULES;
static struct FighterData FD[26];
static struct NameTagData NTD[4];
static struct lbl_8046B488_t LBL;
static struct ToyModelFile TOYF;
static HSD_Text TXT;
static VsModeData VSM;
static SSSData SSS;
static PlayerInitData PID[6];
static StartMeleeRules SMR;
static datetime DT;
static struct GmStats GST;
static struct gm_801677C0_s G677;
static Vec3 V3A, V3B;
static u8 OUT4[4];
static u8 BYTES4[4];
static u64 ITEMMASK;
static u16 MASK16;
static s8 HANDI[16];
static MexStcIcns STC;
static HSD_MatAnimJoint MAJ;
static HSD_MatAnim MA;
static HSD_TexAnim TA;
static HSD_AObjDesc AD;
static u8 ARCHD[4];
static DynamicModelDesc DMD;
static HSD_AnimJoint* ANIMS[4];
static HSD_MatAnimJoint* MATANIMS[4];
static HSD_ShapeAnimJoint* SHAPEANIMS[4];

static u32 shash(const char* s) { return s ? HBUF(s, (unsigned) __builtin_strlen(s)) + 1 : 0; }

/* ---- stubs the generator does not write (they hand out the synthetic state or have an exotic prototype) */
void memzero(void* m, ssize_t n) { T(2000); T((u32) n); zero(m, (unsigned) n); }
s32 HSD_Randi(s32 max) { T(2001); T(max); return max > 0 ? (s32) (rnd() % (u32) max) : 0; }
VsSceneState* gmVs_GetSceneState(void) { T(2002); return &VSS; }
GameRules* gmMainLib_GetGameRules(void) { T(2003); return &GRULES; }
struct FighterData* GetPersistentFighterData(SelectableCharacterKind sk) { T(2004); T((u32) sk); return &FD[(u32) sk % 26]; }
struct NameTagData* GetPersistentNameData(s32 n) { T(2005); T((u32) n); return &NTD[(u32) n % 4]; }
struct lbl_8046B488_t* gm_1601_GetUnkData(void) { T(2006); return &LBL; }
ToyModelFile* Toy_8030813C(int id) { T(2007); T((u32) id); return &TOYF; }
u8 gm_GetCurrentGameMode(void) { u32 r; T(2008); r = rnd(); return (r % 3 == 0) ? 31 : (u8) (rnd() % 45); }
void Stage_80224E38(Vec3* v, s32 id) { T(2009); T((u32) id); v->x = rg(); v->y = rg(); v->z = rg(); }
void lbTime_8000B028(OSCalendarTime* tm, unsigned secs) { T(2010); T(secs); fl_OSCalendarTime(tm); }
HSD_Archive* lbDvd_8001819C(const char* name) { T(2011); T(shash(name)); return (rnd() & 1) ? (HSD_Archive*) ARCHD : 0; }
void* HSD_ArchiveGetPublicAddress(HSD_Archive* a, const char* s) { T(2012); T(a != 0); T(shash(s)); return &STC; }
int HSD_SisLib_803A6B98(HSD_Text* t, float x, float y, const char* s, ...) { T(2013); T(t == &TXT); T(B(x)); T(B(y)); T(shash(s)); return (int) rnd(); }
s32 HSD_SisLib_803A70A0(HSD_Text* t, s32 e, char* f, ...) { T(2014); T(t == &TXT); T((u32) e); T(shash(f)); return (s32) rnd(); }
s8* gmMainLib_8015CE44(s32 a, s32 b) { T(2015); T((u32) a); T((u32) b); if (rnd() % 4 == 0) return 0; return &HANDI[(u32) a % 16]; }
void fn_80169434(GmRouteCallback fn) { T(2016); T(fn != 0); }
void hsd_80392528(Event e) { T(2017); T(e == (Event) fn_801652D8); }

/* ---- generators shared with the ops (mirrored in tests/tu_gm_1601.rs) */
static int ckgen(void) { u32 r = rnd(); return (r % 9 == 0) ? (int) (34 + rnd() % 94) : (int) (rnd() % 33); }
static int ckpgen(void) { u32 r = rnd(); return (r % 9 == 0) ? (int) (34 + rnd() % 94) : (int) (rnd() % 26); }
static int ckngen(void)
{
    u32 r = rnd();
    if (r % 5 == 0) return 33;
    if (r % 7 == 0) return 19;
    return (int) (rnd() % 26);
}
static int modegen(void) { return (int) (rnd() % 45); }

static u32 run_custom(int op);
#include "ops_gm_1601.h"

static void fix_ME(MatchEnd* me, int k)
{
    int i, j;
    fl_MatchEnd(me);
    me->is_teams = rnd() % 3 == 0;
    me->match_kind = rnd() % 4;
    me->sd_penalty = (s8) (rnd() % 5) - 2;
    me->outcome = rnd() % 10;
    me->n_winners = rnd() % 7;
    me->n_team_winners = rnd() % 6;
    for (i = 0; i < 6; i++) {
        MatchPlayerData* p = &me->player_standings[i];
        p->pkind = rnd() % 4;
        p->team = rnd() % 5;
        p->is_small_loser = rnd() % 4;
        p->is_big_loser = rnd() % 6;
        p->x4 = (rnd() % 8 == 0) ? 0x78 : rnd() % 120;
        p->ckind = ckpgen();
        if (k & 1) {
            p->x20 = rnd() % 5;
            p->x24 = rnd() % 5;
            p->x30 = rnd() % 7;
            p->score = (s32) (rnd() % 9) - 4;
            p->x44 = rnd() % 4;
            p->x50 = rnd() % 4;
            p->x28 = (rnd() % 3) ? rnd() % 3000 : 0;
            p->x1C = (s32) (rnd() % 40) - 20;
            p->x9 = rnd() % 5;
            p->stocks = (s8) (rnd() % 5) - 1;
            p->self_destructs = rnd() % 4;
            for (j = 0; j < 4; j++) p->kills[j] = rnd() % 4;
        }
    }
    for (i = 0; i < 5; i++) {
        struct MatchTeamData* t = &me->team_standings[i];
        t->active = rnd() % 2;
        t->is_big_loser = rnd() % 5;
        t->is_small_loser = rnd() % 5;
        if (k & 1) {
            t->score = (s32) (rnd() % 9) - 4;
            t->subscore = (s32) (rnd() % 7) - 2;
        }
    }
    me->player_standings[0].pkind = rnd() % 3;
}

static void setup(u32 g, int k)
{
    int i;
    if (g & G_ME) fix_ME(&ME, k);
    if (g & G_ME2) fix_ME(&ME2, k + 1);
    if (g & G_VSS) fl_VsSceneState(&VSS);
    if (g & G_RULES) {
        fl_GameRules(&GRULES);
        GRULES.mode = rnd() % 5;
        GRULES.handicap = rnd() % 4;
        GRULES.stage_sel = rnd() % 6;
    }
    if (g & G_FD) for (i = 0; i < 26; i++) fl_FighterData(&FD[i]);
    if (g & G_NTD) for (i = 0; i < 4; i++) fl_NameTagData(&NTD[i]);
    if (g & G_LBL) fl_lbl_8046B488_t(&LBL);
    if (g & G_TXT) fl_HSD_Text(&TXT);
    if (g & G_VSM) {
        fl_VsModeData(&VSM);
        for (i = 0; i < 6; i++) VSM.start.players[i].handicap = (s8) (1 + rnd() % 9);
    }
    if (g & G_SSS) {
        fl_SSSData(&SSS);
        SSS.vs.loser = (s8) (rnd() % 5) - 1;
        SSS.vs.winner = (rnd() % 3 == 0) ? -1 : (s8) (rnd() % 4);
        SSS.vs.ordered_stage_index = (s8) (rnd() % 31) - 1;
        for (i = 0; i < 6; i++) SSS.vs.start.players[i].slot_type = rnd() % 5;
    }
    if (g & G_PID) {
        for (i = 0; i < 6; i++) fl_PlayerInitData(&PID[i]);
        fl_StartMeleeRules(&SMR);
    }
    if (g & G_DT) fl_datetime(&DT);
    if (g & G_GMS) fl_GmStats(&GST);
    if (g & G_G677) fl_gm_801677C0_s(&G677);
    if (g & G_V3) { fl_Vec(&V3A); fl_Vec(&V3B); }
    if (g & G_MISC) {
        for (i = 0; i < 4; i++) OUT4[i] = rnd();
        for (i = 0; i < 4; i++) BYTES4[i] = 1 + rnd() % 9;
        { u32 lo = rnd(); u32 hi = rnd(); ITEMMASK = ((u64) hi << 32) | lo; }
        MASK16 = rnd();
        for (i = 0; i < 16; i++) HANDI[i] = (s8) (1 + rnd() % 9);
    }
    if (g & G_GLOB) {
        for (i = 0; i < 4; i++) HSD_PadMasterStatus[i].err = rnd() % 2;
        stage_info.unk8C.b4 = rnd() & 1;
        stage_info.unk8C.b5 = rnd() & 1;
        HSD_VIData.perf.frame_renew = (s32) rnd();
        CostumeListsForeachCharacter[Ft_Kind_Mario].numCostumes = (u8) rnd();
    }
    if (g & G_MEX) {
        STC.reserved = (u16) rnd();
        STC.stride = (u16) rnd();
        STC.matanim = (rnd() & 1) ? &MAJ : 0;
        MAJ.matanim = (rnd() & 1) ? &MA : 0;
        MA.texanim = (rnd() & 1) ? &TA : 0;
        TA.aobjdesc = (rnd() & 1) ? &AD : 0;
        AD.end_frame = rg();
    }
    if (g & G_DMD) {
        DMD.anims = (rnd() & 1) ? ANIMS : 0;
        DMD.matanims = (rnd() & 1) ? MATANIMS : 0;
        DMD.shapeanims = (rnd() & 1) ? SHAPEANIMS : 0;
        for (i = 0; i < 4; i++) {
            ANIMS[i] = (rnd() & 1) ? (HSD_AnimJoint*) (size_t) 1 : 0;
            MATANIMS[i] = (rnd() & 1) ? (HSD_MatAnimJoint*) (size_t) 1 : 0;
            SHAPEANIMS[i] = (rnd() & 1) ? (HSD_ShapeAnimJoint*) (size_t) 1 : 0;
        }
    }
    if (g & G_PERF) fl_lbl_8046B378_t(&lbl_8046B378);
}

static u32 state_hash(u32 g)
{
    u32 h = 0;
    int i;
    h = h * 31u + pool_hash();
    h = hm_(h, (u32) (s32) lbl_804D6598);
    h = hm_(h, (u32) lbl_804D659A);
    if (g & G_ME) h = hs_MatchEnd(&ME, h);
    if (g & G_ME2) h = hs_MatchEnd(&ME2, h);
    if (g & G_VSS) h = hs_VsSceneState(&VSS, h);
    if (g & G_RULES) h = hs_GameRules(&GRULES, h);
    if (g & G_FD) for (i = 0; i < 26; i++) h = hs_FighterData(&FD[i], h);
    if (g & G_NTD) for (i = 0; i < 4; i++) h = hs_NameTagData(&NTD[i], h);
    if (g & G_LBL) h = hs_lbl_8046B488_t(&LBL, h);
    if (g & G_TXT) h = hs_HSD_Text(&TXT, h);
    if (g & G_VSM) h = hs_VsModeData(&VSM, h);
    if (g & G_SSS) h = hs_SSSData(&SSS, h);
    if (g & G_PID) { for (i = 0; i < 6; i++) h = hs_PlayerInitData(&PID[i], h); h = hs_StartMeleeRules(&SMR, h); }
    if (g & G_DT) h = hs_datetime(&DT, h);
    if (g & G_GMS) h = hs_GmStats(&GST, h);
    if (g & G_G677) h = hs_gm_801677C0_s(&G677, h);
    if (g & G_V3) { h = hs_Vec(&V3A, h); h = hs_Vec(&V3B, h); }
    if (g & G_MISC) {
        for (i = 0; i < 4; i++) h = hm_(h, OUT4[i]);
        for (i = 0; i < 4; i++) h = hm_(h, BYTES4[i]);
        h = hm_(h, (u32) ITEMMASK);
        h = hm_(h, (u32) (ITEMMASK >> 32));
        h = hm_(h, MASK16);
        for (i = 0; i < 16; i++) h = hm_(h, (u32) (s32) HANDI[i]);
    }
    if (g & G_GLOB) {
        for (i = 0; i < 4; i++) h = hm_(h, (u32) (s32) HSD_PadMasterStatus[i].err);
        h = hm_(h, (u32) stage_info.unk8C.b4);
        h = hm_(h, (u32) stage_info.unk8C.b5);
        h = hm_(h, (u32) HSD_VIData.perf.frame_renew);
        h = hm_(h, CostumeListsForeachCharacter[Ft_Kind_Mario].numCostumes);
    }
    h = hs_lbl_8046B378_t(&lbl_8046B378, h);
    return h;
}

/* the TU's two local bitfield structs are laid out for the PowerPC (first field = most significant bit); the x86
 * compiler lays them out the other way round, so the C side sees the nibbles / bits swapped around each call */
static void swap_in(u32 g)
{
    int i;
    if (g & G_ME) ME.loser = (u8) ((ME.loser >> 4) | (ME.loser << 4));
    if (g & G_ME2) ME2.loser = (u8) ((ME2.loser >> 4) | (ME2.loser << 4));
    if (g & G_VSS) {
        VSS.x24C.loser = (u8) ((VSS.x24C.loser >> 4) | (VSS.x24C.loser << 4));
        for (i = 0; i < 6; i++) {
            u16 v = VSS.fighters[i].xC;
            VSS.fighters[i].xC = (u16) ((v & 0x7FFEu) | ((v >> 15) & 1u) | ((v & 1u) << 15));
        }
    }
}

/* (the C hash of MatchEnd.loser swaps the nibbles back itself, see HSFIX; only the fighters' xC bits are put back) */
static void swap_out(u32 g)
{
    int i;
    if (g & G_VSS) {
        for (i = 0; i < 6; i++) {
            u16 v = VSS.fighters[i].xC;
            VSS.fighters[i].xC = (u16) ((v & 0x7FFEu) | ((v >> 15) & 1u) | ((v & 1u) << 15));
        }
    }
}

static u32 run_custom(int op) { (void) op; return 0; }

void* __acrt_iob_func(unsigned);
int setvbuf(void*, char*, int, unsigned);
int main(void)
{
    int op, k;
    setvbuf(__acrt_iob_func(1), 0, 4, 0);
    TOYF.trophy_id = 5;
    __builtin_memcpy(TOYF.archive_name, "toyarch", 8);
    __builtin_memcpy(TOYF.symbol_name, "toysymbol", 10);
    for (op = 1; op <= NOPS; op++) {
        u32 g = op_groups(op);
        for (k = 0; k < NCASES; k++) {
            u32 ret;
            tu_case(op, k);
            pool_reset();
            lbl_804D6598 = (s8) rnd();
            lbl_804D659A = (u16) (rnd() % 5);
            setup(g, k);
            swap_in(g);
            ret = run_op(op);
            swap_out(g);
            printf("%d %d %08X %08X %08X\n", op, k, ret, state_hash(g), TR);
        }
    }
    return 0;
}
