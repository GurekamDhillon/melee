"""Generate a standalone test using actual production terminal/query functions."""
from pathlib import Path
import sys
root = Path(__file__).resolve().parents[1]
source = (root / 'gameworld/script_game.c').read_text()
def function(name):
    start = source.index(name)
    start = source.rfind('\n', 0, start) + 1
    end = source.index('{', start) + 1
    depth = 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]
fixture = r'''
#include <assert.h>
#include <stdio.h>
#define SCRIPT_STAGE_ENEMIES 2
#define PART_CTL_TINT 7
#define OSReport(...) ((void)0)
typedef struct { int handle,kind,active,defeated; void* gobj; } ScriptStageEnemy;
static struct { ScriptStageEnemy enemy[2]; } script_stage;
static int defeats,removes;
static int script_enemy_index(int kind) { return kind; }
void Script_EnemyDefeated(int kind,int handle) { assert(kind==3&&handle==9); ++defeats; }
void Script_EnemyRemoved(int kind,int handle,int reason) { ++removes; }
typedef void Item_GObj;
static int lab_part_colors_hi=4;
static struct { void* dobj; int slot,mode; } lab_part_colors[4];
'''
fixture += function('static void script_enemy_terminal') + '\n'
fixture += function('int ScriptGame_EnemyDefeated') + '\n'
fixture += function('int ScriptGame_EnemyDestroyed') + '\n'
fixture += (root / 'gameworld/script_tint_query.inc').read_text()
fixture += r'''
int main(void) {
    int object;
    ScriptStageEnemy* e=&script_stage.enemy[0];
    e->active=1;e->kind=3;e->handle=9;e->gobj=&object;
    assert(ScriptGame_EnemyDefeated(&object)==1);
    assert(ScriptGame_EnemyDefeated(&object)==2);
    ScriptGame_EnemyDestroyed(&object);
    assert(defeats==1 && removes==0);
    lab_part_colors[0].dobj=&object;lab_part_colors[0].slot=0;lab_part_colors[0].mode=PART_CTL_TINT;
    lab_part_colors[1]=lab_part_colors[0];lab_part_colors[1].slot=1;
    lab_part_colors[2]=lab_part_colors[0];lab_part_colors[2].mode=8;
    assert(ScriptGame_TintCount(0)==1 && ScriptGame_TintCount(1)==1);
    lab_part_colors[0].dobj=NULL;
    assert(ScriptGame_TintCount(0)==0);
    puts("real enemy terminal and tint registry fixture PASS");return 0;
}
'''
Path(sys.argv[1]).write_text(fixture)
