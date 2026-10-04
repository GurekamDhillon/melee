/* Formula witness, not a game/KO replay. Public NTSC 1.02 values:
 * Mario FSmash sweetspot: FightCore /characters/236/mario/moves/849/fsmash/
 * Mario weight100 / Peach90: meleeframedata.com/mario and /peach.
 * FD +/-246 horizontal blast zones: libmelee stages.py.
 * Formula: retail ftColl_80079C70 (x28=0); original damage remains 18.
 * Conservative stated proxy: continuous no-DI launch momentum travels to the
 * horizontal blast line at angle44 with speed .03*KB and decay .051/frame.
 * Ignore drift, gravity, stage collisions, DI, stale moves, armor and hitstun
 * exits. The proxy calibrates a budget; it does not certify actual KO timing. */
#include <assert.h>
#include <math.h>
#include <stdio.h>
#include "../gameworld/script_hit_rules_core.h"
static double kb(int before,double current_hit,double raw,double weight) {
 double after=before+current_hit;
 return ((after/10+after*raw/20)*200/(weight+100)*1.4+18)*.95+25;
}
static int launch_percent(double weight,double current,double raw,double scale,double target) {
 int p;for(p=0;p<1000;++p)if(kb(p,current,raw,weight)*scale>=target)return p;
 return 1000;
}
int main(void) {
 HRRule outgoing[3],incoming[1];HRResult a,b;double scale,target;int i;
 for(i=0;i<3;++i)hr_defaults(&outgoing[i]);hr_defaults(&incoming[0]);
 outgoing[0].id=1001;outgoing[0].percent_damage=1.6f; /* Glass Core; no launch */
 outgoing[1].id=1002;outgoing[1].launch=1.075f; /* Heavy tier3 */
 outgoing[2].id=1003;outgoing[2].launch=1.12f;outgoing[2].move=4; /* Pyre tier3 smash */
 incoming[0].id=1004;incoming[0].incoming=1;incoming[0].launch=1.0375f; /* Curse tier3 */
 a=hr_evaluate(outgoing,3,4,1,1,0,3);b=hr_evaluate(incoming,1,4,1,1,0,4);
 scale=hr_family_cap(a.launch,1.3f)*hr_family_cap(b.launch,1.3f);
 assert(fabs(scale-1.2398125)<.000001);
 assert(hr_family_cap(a.percent_damage,1.6f)==1.6f);
 /* The on-screen percent bonus stays outside both current-hit formula inputs. */
 assert(fabs(kb(0,18,18,100)-66.04)<.00001);
 assert(fabs(kb(0,36,36,100)-133.072)<.00001); /* actual native hitbox2x */
 assert(fabs(kb(0,38.88,18,100)-93.8104)<.00001); /* logged dealt2.16, raw unscaled */
 assert(fabs(kb(50,18,18,100)-132.54)<.00001);
 assert(fabs(kb(50,38.88,18,100)-160.3104)<.00001);
 target=sqrt(246*2*.051/(.03*.03*cos(44*3.141592653589793/180)));
 for(i=0;i<2;++i) {
  double weight=i ? 90 : 100;
  int vanilla=launch_percent(weight,18,18,1,target);
  int balanced=launch_percent(weight,18,18,scale,target);
  int old_logged=launch_percent(weight,38.88,18,1,target);
  int old_hitbox=launch_percent(weight,36,36,1,target);
  double earlier=100.0*(vanilla-balanced)/vanilla;
  assert(earlier>=20 && earlier<=30);
  assert(vanilla==(i ? 93 : 99) && balanced==(i ? 66 : 70));
  assert(old_logged==(i ? 72 : 78) && old_hitbox==(i ? 23 : 26));
  printf("%s weight %.0f FD launch-distance proxy: vanilla %d balanced %d (%.2f%% earlier), old logged2.16 %d native hitbox2x %d; KB threshold %.4f\n",i ? "Peach" : "Mario",weight,vanilla,balanced,earlier,old_logged,old_hitbox,target);
 }
 puts("Envoy budget formula witness PASS (not live KO acceptance)");return 0;
}
