/* Parser fixtures: no disc or scene execution required. */
static int test_scene_six_slots(void) {
  const GwSceneConfig *c;
  gw_SceneLaunch_LoadForTest("mode=vs;p1=fox;teams=1;p5=marth/cpu0/team1/stocks3;p6=falco/cpu9/team1");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (!c->configured || c->errors || c->p[4].ckind != 9 ||
      c->p[5].ckind != 20 || c->p[4].team != 1 || c->p[4].stocks != 3 ||
      gw_SceneLaunch_PlayerSlotType(5) != GW_SL_PK_CPU ||
      gw_SceneLaunch_PlayerCKind(6) != -1) return 1;
  gw_SceneLaunch_LoadForTest("mode=lab;p1=fox;p6=marth");
  if (!gw_SceneLaunch_Active() || gw_SceneLaunch_PlayerSlotType(5) != GW_SL_PK_CPU) return 1;
  gw_SceneLaunch_LoadForTest("mode=vs;p7=fox");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->errors != 1 || c->configured || !c->invalid_slot) return 1;
  gw_SceneLaunch_LoadForTest("p7=fox;mode=vs;p1=fox");
  if (gw_SceneLaunch_Active()) return 1;
  gw_SceneLaunch_LoadForTest("mode=vs;teams=1;enemy_team_colors=1;p6=fox");
  if (!gw_SceneLaunch_Active() || !gw_SceneLaunch_EnemyTeamColors()) return 1;
  gw_SceneLaunch_LoadForTest("mode=vs;p1=fox;p5=random;p6=random;preload=marth/falco");
  if (!gw_SceneLaunch_Active() || gw_SceneLaunch_PreloadCKind(0)!=9 ||
      gw_SceneLaunch_PreloadCKind(1)!=20 || gw_SceneLaunch_PreloadCKind(2)!=-1 ||
      gw_SceneLaunch_PlayerCKind(4)<0 || gw_SceneLaunch_PlayerCKind(5)<0) return 1;
  gw_SceneLaunch_LoadForTest("mode=vs;preload=fox/marth/falco");
  c=(const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->errors!=1 || c->preload[0]!=-1) return 1;
  gw_SceneLaunch_LoadForTest(NULL);
  return 0;
}
static int test_scene_six_refusals(void) {
  static const char *bad[] = {
    "mode=vs;at=css;p5=fox", "p6=fox;mode=vs;at=sss",
    "mode=training;p5=fox", "mode=classic;p6=fox",
    "mode=vs;p5=fox/hu", "mode=lab;p6=fox/demo"
  };
  unsigned i;
  for (i=0; i<sizeof bad/sizeof bad[0]; ++i) {
    const GwSceneConfig *c;
    gw_SceneLaunch_LoadForTest(bad[i]);
    c=(const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
    if (c->configured || !c->errors) return 1;
  }
  gw_SceneLaunch_LoadForTest("mode=vs;at=css;p5=none;p6=none");
  if (!gw_SceneLaunch_Active()) return 1;
  gw_SceneLaunch_LoadForTest(NULL);
  return 0;
}

extern int gw_GenoLab_SixSeedTest(int six);
static int test_scene_six_seed(void) {
  int rc;
  gw_SceneLaunch_LoadForTest("mode=vs;p1=fox/hu/team0/stocks1;p2=fox/cpu0/team1/stocks2;p3=fox/cpu0/team1/stocks3;p4=fox/cpu0/team1/stocks4;p5=fox/cpu0/team1/stocks5;p6=fox/cpu0/team1/stocks6");
  rc=gw_GenoLab_SixSeedTest(1);
  gw_SceneLaunch_LoadForTest("mode=vs;p1=fox");
  rc+=gw_GenoLab_SixSeedTest(0);
  gw_SceneLaunch_LoadForTest(NULL);
  return rc;
}
#include "../../src/melee/lb/lbsixbudget.h"
extern int gw_lbDvd_SixBudgetTest(void);
extern int gw_SceneLaunch_SixPresentationTest(int force);
static int test_scene_six_fix1(void) {
  int rc;
  gw_SceneLaunch_LoadForTest("mode=vs;teams=1;enemy_team_colors=1;p1=fox;p2=fox/cpu0/team1;p3=fox/cpu0/team1;p4=fox/cpu0/team1;p5=fox/cpu0/team1;p6=fox/cpu0/team1");
  rc=gw_SceneLaunch_SixPresentationTest(1);
  gw_SceneLaunch_LoadForTest("mode=vs;teams=1;enemy_team_colors=0;p1=fox;p6=fox/cpu0/team1");
  rc|=gw_SceneLaunch_SixPresentationTest(0);
  rc|=gw_lbDvd_SixBudgetTest();
  gw_SceneLaunch_LoadForTest(NULL);
  return rc;
}
static int test_scene_six_budget(void) {
  unsigned used=0;
  if (!lbSixBudget_Add(&used,256,1,1) || used!=128) return 1;
  if (!lbSixBudget_Add(&used,256,64,0) || used!=256) return 1;
  if (lbSixBudget_Add(&used,256,1,0) || used!=256) return 1;
  if (lbSixBudget_Add(&used,256,0,0) || lbSixBudget_Add(&used,256,0xFFFFFFFFU,0)) return 1;
  used=257;
  return lbSixBudget_Add(&used,256,1,0) ? 1 : 0;
}
