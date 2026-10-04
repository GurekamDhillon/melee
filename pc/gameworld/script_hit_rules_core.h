/* Shared pure evaluator. No Lua, allocation, pointers or mutable host state. */
#ifndef SCRIPT_HIT_RULES_CORE_H
#define SCRIPT_HIT_RULES_CORE_H
#define HR_MAX 32
#define HR_FIELDS 16
typedef struct {
 int id, move, grounded, original_element, status_bits, element;
 float damage, growth, base, percent_damage, launch;
 int shield, stun;
 float knockback_taken;
 int handle, incoming;
} HRRule;
typedef struct {
 int element, shield, stun, count, ids[HR_MAX];
 float damage, growth, base, knockback_taken, percent_damage, launch;
} HRResult;
/* Bit classification also works in the PPC retarget without libc math imports. */
static int hr_finite(float x) { union {float f;unsigned u;} v;v.f=x;return (v.u&0x7f800000u)!=0x7f800000u; }
static float hr_safe_float(double x) {
 if(x!=x)return 0;
 if(x>3.4028234663852886e38)return 3.4028234663852886e38f;
 if(x< -3.4028234663852886e38)return -3.4028234663852886e38f;
 return (float)x;
}
static int hr_ratio_valid(float x,float lo,float hi) {return hr_finite(x) && x>=lo && x<=hi;}
static float hr_cap(float x) { if(!hr_finite(x))return 1;return x<0.1f ? 0.1f : x>4 ? 4 : x; }
static float hr_family_cap(float x,float hi) {if(!hr_finite(x))return 1;return x<0.05f ? 0.05f : x>hi ? hi : x;}
static float hr_percent_cap(float x,int incoming) {if(!hr_finite(x))return 1;return x<(incoming ? .15f : .05f) ? (incoming ? .15f : .05f) : x>64 ? 64 : x;}
static int hr_ordinary(int e) { return e==0 || e==1 || e==2 || e==5 || e==13; }
static void hr_defaults(HRRule* r) {
 r->id=0; r->move=r->grounded=r->original_element=r->element=-1;
 r->status_bits=r->shield=r->stun=r->handle=r->incoming=0;
 r->damage=r->growth=r->base=r->knockback_taken=r->percent_damage=r->launch=1;
}
static HRResult hr_evaluate(const HRRule* rules,int count,int move,int grounded,int original,int status,int contact) {
 HRResult v; int i;double percent_damage=1,launch=1; v.element=original; v.shield=v.stun=v.count=0;
 v.damage=v.growth=v.base=v.knockback_taken=v.percent_damage=v.launch=1;
 for(i=0;i<HR_MAX;++i) v.ids[i]=0;
 if(!hr_ordinary(original)) return v;
 for(i=0;i<count && i<HR_MAX;++i) {
  const HRRule* r=&rules[i]; HRResult before=v;
  if((r->move>=0 && r->move!=move) || (r->grounded>=0 && r->grounded!=grounded) ||
     (r->original_element>=0 && r->original_element!=original) ||
     (contact ? (r->incoming!=(contact==2 || contact==4) || (contact==1 && !r->status_bits) || (status&r->status_bits)!=r->status_bits) : (r->status_bits!=0 || r->incoming))) continue;
  if(contact>=3) {
   if(r->percent_damage==1 && r->launch==1) continue;
   /* Signed encoded contributions retain excess until one final contact cap.
    * Double intermediates preserve small costs between opposite large terms. */
   percent_damage+=(double)r->percent_damage-1;launch+=(double)r->launch-1;
  } else if(contact) {
   if(r->damage==1 && r->knockback_taken==1) continue;
   v.damage=hr_cap(v.damage*r->damage); v.knockback_taken=hr_cap(v.knockback_taken*r->knockback_taken);
  } else {
   if(r->element<0 && r->damage==1 && r->growth==1 && r->base==1 && !r->shield && !r->stun) continue;
   if(r->element>=0) v.element=r->element;
   v.damage=hr_cap(v.damage*r->damage); v.growth=hr_cap(v.growth*r->growth); v.base=hr_cap(v.base*r->base);
   v.shield+=r->shield; if(v.shield>100) v.shield=100; if(v.shield< -100) v.shield=-100;
   v.stun+=r->stun; if(v.stun>120) v.stun=120;
  }
  if(contact>=3 || v.element!=before.element || v.damage!=before.damage || v.growth!=before.growth || v.base!=before.base || v.shield!=before.shield || v.stun!=before.stun || v.knockback_taken!=before.knockback_taken || v.percent_damage!=before.percent_damage || v.launch!=before.launch)
   v.ids[v.count++]=r->id ? r->id : r->handle;
 }
 v.percent_damage=(float)percent_damage;v.launch=(float)launch;
 return v;
}
#endif
