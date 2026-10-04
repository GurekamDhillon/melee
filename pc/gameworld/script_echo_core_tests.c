#include <assert.h>
#include <stdio.h>
#include <string.h>
typedef struct { unsigned words[78]; } EchoTestCapsule;
#define ECHO_CAPSULE EchoTestCapsule
#include "script_echo_core.h"
static EchoState state, saved;
int main(void) {
 EchoRecord r; const EchoRecord* old; int i;
 memset(&r,0,sizeof r); r.generation=7;r.hit_count=1;r.hit[0].words[0]=42;
 for(i=0;i<80;i++){r.position[0]=(float)i;echo_record(&state,0,&r);}
 assert(echo_history(&state,0,0)->position[0]==79);
 old=echo_history(&state,0,60);assert(old && old->position[0]==19);
 assert(old->hit[0].words[0]==42);
 assert(!echo_history(&state,0,61));assert(!echo_history(&state,1,0));
 saved=state;r.position[0]=100;echo_record(&state,0,&r);state=saved;
 assert(memcmp(&state,&saved,sizeof state)==0);
 assert(echo_history(&state,0,0)->position[0]==79);
 echo_entity_clear(&state,0);assert(!echo_history(&state,0,0));
 puts("echo production ring: wrap age60 identity snapshot 0 differing bytes cleanup PASS");
 return 0;
}
