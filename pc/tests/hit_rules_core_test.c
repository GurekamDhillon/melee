#include <assert.h>
#include <string.h>
#include <math.h>
#include <stdio.h>
#include "../gameworld/script_hit_rules_core.h"
int main(void) {
 HRRule r[32]; HRResult v; int i; memset(r,0,sizeof r); hr_defaults(&r[0]); hr_defaults(&r[1]);
 r[0].element=1; r[0].damage=2; r[0].id=11;
 r[1].element=2; r[1].damage=3; r[1].id=12;
 v=hr_evaluate(r,2,4,1,0,0,0);
 assert(v.element==2 && v.damage==4 && v.count==2);
 v=hr_evaluate(r,2,4,1,7,0,0); assert(v.count==0 && v.damage==1 && v.element==7);
 r[0].status_bits=8; r[0].element=-1; r[0].knockback_taken=1.5f;
 v=hr_evaluate(r,1,4,1,0,8,1); assert(v.damage==2 && v.knockback_taken==1.5f);
 v=hr_evaluate(r,1,4,1,0,0,1); assert(v.damage==1 && v.count==0);
 r[0].move=5; v=hr_evaluate(r,1,4,1,0,8,1); assert(v.count==0);
 r[0].move=-1; r[0].original_element=1; v=hr_evaluate(r,1,4,1,0,8,1); assert(v.count==0);
 hr_defaults(&r[0]);r[0].element=0;v=hr_evaluate(r,1,4,1,0,0,0);assert(v.count==0);
 /* Every one of 30 pool IDs plus 2 aggregate rules survives evaluation. */
 for(i=0;i<32;++i){hr_defaults(&r[i]);r[i].id=300+i;r[i].percent_damage=1.25f;r[i].launch=1.125f;}
 v=hr_evaluate(r,32,4,1,0,0,3);
 assert(v.count==32 && v.ids[31]==331 && v.percent_damage==9 && v.launch==5);
 assert(hr_family_cap(v.percent_damage,64)==9 && hr_family_cap(v.launch,4)==4);
 for(i=0;i<32;++i){r[i].percent_damage=64;r[i].launch=4;}
 v=hr_evaluate(r,32,4,1,0,0,3);
 assert(hr_family_cap(v.percent_damage,64)==64 && hr_family_cap(v.launch,4)==4);
 for(i=0;i<32;++i){r[i].percent_damage=.05f;r[i].launch=.05f;}
 v=hr_evaluate(r,32,4,1,0,0,3);
 assert(hr_family_cap(v.percent_damage,64)==.05f && hr_family_cap(v.launch,4)==.05f);
 r[0].percent_damage=NAN;r[0].launch=INFINITY;
 v=hr_evaluate(r,1,4,1,0,0,3);
 assert(isfinite(hr_family_cap(v.percent_damage,64)) && isfinite(hr_family_cap(v.launch,4)));
 {
  static const int order[6][3]={{0,1,2},{0,2,1},{1,0,2},{1,2,0},{2,0,1},{2,1,0}};
  static const float values[]={1000000000.f,-1000000000.f,2.5f};int j;
  for(i=0;i<6;++i){for(j=0;j<3;++j){hr_defaults(&r[j]);r[j].id=800+j;r[j].percent_damage=r[j].launch=values[order[i][j]];}
   v=hr_evaluate(r,3,4,1,0,0,3);assert(v.count==3 && v.percent_damage==.5f && v.launch==.5f);}
 }
 puts("native hit-rule core: 32 rules, additive progression and finite family safety PASS");
 return 0;
}

