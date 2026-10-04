"""Exercise extracted production collision-log writers, without a game build."""
from pathlib import Path
import re

root = Path(__file__).resolve().parents[2]
source = (root / 'src/melee/ft/ftcoll.c').read_text()
limits = root / 'pc/gameworld/script_echo_limits.h'

def function(name):
    start = source.index(name + '(')
    begin = source.index('{', start)
    depth = 1
    end = begin + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[source.rfind('\n', 0, start) + 1:end]

# Fail against retail20 arrays and the two inline literal20 guards before fix.
assert source.count('[ECHO_COLLISION_LOG_CAPACITY]') == 2, 'virtual contact log capacity missing'
assert 'dmg_log0_idx < 20U' not in source, 'inline contact retains retail20 guard'
assert 'ftColl_804D6560[ECHO_COLLISION_CAPS]' in source
out = root.parent / '_build/tmp/em5-log-fixture.c'
out.write_text('''#include <assert.h>
#include <stddef.h>
#include "script_echo_limits.h"
#define ARRAY_SIZE(a) (sizeof(a)/sizeof((a)[0]))
#define HSD_ASSERTREPORT(line,condition,...) assert(condition)
typedef int enum_t;
typedef void Fighter_GObj;
typedef void HSD_GObj;
typedef struct {int count;} DynamicsDesc;
typedef struct {float x,y,z;} Vec3;
typedef struct {Vec3 cur_pos;} Fighter;
typedef struct {int unused;} FighterHurtCapsule;
typedef struct {Vec3 hurt_coll_pos;} HitCapsule;
typedef struct {int x0,kind;void *gobj;union{DynamicsDesc *unk_anim0;HitCapsule *hit0;};union{FighterHurtCapsule *hurt1;HitCapsule *hit1;};Vec3 pos;float x20;int size_of_xC;} DmgLogEntry;
static DmgLogEntry dmg_log0[ECHO_COLLISION_LOG_CAPACITY],dmg_log1[ECHO_COLLISION_LOG_CAPACITY];
static int dmg_log0_idx,dmg_log1_idx;
''' + function('ftColl_80076764') + '\n' + function('tiplog') + '''
int main(void){int i;Fighter fp={{1,2,3}};DynamicsDesc d={7};HitCapsule h={{4,5,6}};
assert(ECHO_COLLISION_LOG_CAPACITY>=20+12*44);
for(i=0;i<ECHO_COLLISION_LOG_CAPACITY;++i){ftColl_80076764(i,i,0,&d,&fp,0);tiplog(i,0,&h,0,i,(float)i);}
assert(dmg_log0_idx==ECHO_COLLISION_LOG_CAPACITY&&dmg_log1_idx==ECHO_COLLISION_LOG_CAPACITY);
for(i=0;i<ECHO_COLLISION_LOG_CAPACITY;++i){assert(dmg_log0[i].kind==i&&dmg_log0[i].size_of_xC==7);assert(dmg_log1[i].kind==i&&dmg_log1[i].x20==(float)i);}
return 0;}
''')
print('production log writers extracted; all inline log guards use array capacity')
