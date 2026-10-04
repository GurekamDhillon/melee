#include <assert.h>
#include <string.h>
#include "../gameworld/script_hit_rules_core.h"
int main(void) {
 HRRule r[8]; HRResult v; memset(r,0,sizeof r); hr_defaults(&r[0]); hr_defaults(&r[1]);
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
 return 0;
}

