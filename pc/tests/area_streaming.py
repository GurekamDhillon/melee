"""Compile the real area implementation against deterministic scalar fixtures.

No game executable, disc data, retargeter, or game build is involved.
"""
import pathlib
import subprocess
import sys

root = pathlib.Path(__file__).resolve().parents[2]
work = root.parent / '_build' / 'tmp' / 'area-streaming-fixture'
work.mkdir(parents=True, exist_ok=True)
source = (root / 'pc/gameworld/script_largemap.inc').read_text()
body = source[source.index('int ScriptGame_AreaFind'):source.index('int ScriptGame_StageStat')]
gates = (root / 'pc/gameworld/script_game.c').read_text()
gates = gates[gates.index('int ScriptGame_AreaVisible'):gates.index('static int script_area_retiring')]
preamble = r'''
#include <assert.h>
#include <stdio.h>
#include <string.h>
#define SCRIPT_STAGE_AREAS 64
#define SCRIPT_MESH_INSTANCES 256
#define SCRIPT_STAGE_TARGETS 128
static struct {
 struct { int owner, handle, state, drain; char name[48]; } area[64];
 struct { int handle, area; } instance[256];
 struct { int handle, active, area; } line[768], target[128];
 int cap, loading_area;
} script_stage;
static char input[48];
static int deletes, flushes, batches;
static int Script_AreaNameByte(int i) { return input[i]; }
static void OSReport(const char *fmt, ...) { (void)fmt; }
static void mpScriptBatchBegin(void) { ++batches; }
static void mpScriptBatchEnd(void) { ++flushes; }
static void mpScriptInvalidateBounding(void) {}
static int ScriptGame_ModelDespawn(int h) {
 int i; for(i=0;i<256;i++) if(script_stage.instance[i].handle==h) {
 script_stage.instance[i].handle=0; ++deletes; return 1; } return 0;
}
static int ScriptGame_StageRemove(int h) {
 int i; for(i=0;i<script_stage.cap;i++) if(script_stage.line[i].handle==h && script_stage.line[i].active) {
 script_stage.line[i].active=0; ++deletes; return 1; } return 0;
}
'''
checks = r'''
int main(void) {
 int area, i, before; typeof(script_stage) saved;
 script_stage.cap=768; strcpy(input,"room");
 area=ScriptGame_AreaPrepare(7,101); assert(area>0);
 script_stage.line[0].active=1;script_stage.line[0].handle=33;script_stage.line[0].area=area;
 script_stage.instance[0].handle=44;script_stage.instance[0].area=area;
 assert(!ScriptGame_AreaVisible(area)); ScriptGame_AreaEnd(1);
 assert(ScriptGame_AreaStatus(7,101)==1); saved=script_stage;
 before=deletes; assert(ScriptGame_AreaActivate(8,101)==0);
 assert(ScriptGame_AreaActivate(7,101)==1 && ScriptGame_AreaVisible(area));
 assert(deletes==before && ScriptGame_AreaStatus(7,101)==2);
 assert(ScriptGame_AreaUnload(7)==1); assert(!ScriptGame_AreaVisible(area));
 assert(deletes==before && ScriptGame_AreaStatus(7,101)==3);
 ScriptGame_AreaDrain(); assert(deletes>before);
 for(i=0;i<1024 && ScriptGame_AreaStatus(7,101);i++) ScriptGame_AreaDrain();
 assert(ScriptGame_AreaStatus(7,101)==0);
 script_stage=saved; assert(ScriptGame_AreaStatus(7,101)==1);
 assert(script_stage.line[0].active && script_stage.instance[0].handle==44);
 assert(ScriptGame_AreaActivate(7,101));
 assert(ScriptGame_AreaBegin(7)==0); /* compatible load remains idempotent */
 strcpy(input,"failed"); area=ScriptGame_AreaPrepare(7,102); assert(area>0);
 ScriptGame_AreaEnd(0); assert(ScriptGame_AreaStatus(7,102)==3);
 assert(!ScriptGame_AreaActivate(7,102));
 assert(batches==flushes);
 puts("PASS area prepare/activate/owner/refusal/deferred drain/snapshot/failure/idempotence");
 return 0;
}
'''
unit = work / 'fixture.c'
unit.write_text(preamble + gates + body + checks)
clang = pathlib.Path(sys.argv[1]) if len(sys.argv)>1 else root.parent / '_toolchains/llvm/bin/clang.exe'
exe = work / 'fixture.exe'
subprocess.run([str(clang), '-std=gnu11', str(unit), '-o', str(exe)], check=True)
subprocess.run([str(exe)], check=True)
