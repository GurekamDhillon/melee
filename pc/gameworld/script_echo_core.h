#ifndef SCRIPT_ECHO_CORE_H
#define SCRIPT_ECHO_CORE_H
#include "script_echo_limits.h"
typedef struct {
 unsigned generation, identity[ECHO_HITS];
 int hit_move[ECHO_HITS],hit_grounded[ECHO_HITS],hit_element[ECHO_HITS];
 int motion,move,airborne,hit_count,attack,attack_instance;
 float frame,position[3],facing;
 int hit_source[ECHO_HITS],hit_source_index[ECHO_HITS];
 ECHO_CAPSULE hit[ECHO_HITS];
} EchoRecord;
typedef struct { unsigned head,count; EchoRecord record[ECHO_DEPTH]; } EchoRing;
typedef struct { EchoRing ring[ECHO_ENTITIES]; } EchoState;
static const EchoRecord* echo_history(const EchoState* s,int entity,int age) {
 const EchoRing* r;
 if(entity<0 || entity>=ECHO_ENTITIES || age<0 || age>=ECHO_DEPTH)return 0;
 r=&s->ring[entity];if((unsigned)age>=r->count)return 0;
 return &r->record[(r->head+ECHO_DEPTH-1-(unsigned)age)%ECHO_DEPTH];
}
static void echo_record(EchoState* s,int entity,const EchoRecord* value) {
 EchoRing* r=&s->ring[entity];r->record[r->head]=*value;
 r->head=(r->head+1)%ECHO_DEPTH;if(r->count<ECHO_DEPTH)++r->count;
}
static void echo_entity_clear(EchoState* s,int entity) {
 memset(&s->ring[entity],0,sizeof s->ring[entity]);
}
#endif
