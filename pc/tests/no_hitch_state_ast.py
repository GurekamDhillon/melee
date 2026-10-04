"""Execute production scalar warm/launch C with deterministic host stubs.

This checks bookkeeping, not renderer execution, MEM1 rewind or timing.
The companion C++ pipeline_warm_test.cpp checks the renderer registry natively.
"""
from pathlib import Path
import re
import runpy
from pycparser import c_ast, c_parser

base = runpy.run_path(str(Path(__file__).with_name('area_streaming_ast.py')))


def function(source, name):
    start = re.search(r'^(?:static )?(?:int|void) ' + name + r'\(', source, re.M).start()
    brace = source.index('{', start)
    depth = 1
    end = brace + 1
    while depth:
        depth += (source[end] == '{') - (source[end] == '}')
        end += 1
    return source[start:end]


class State(base['Fixture']):
    def __init__(self):
        platform = Path(__file__).resolve().parents[1] / 'platform'
        warm = (platform / 'gw_script_warm.inc').read_text()
        launch = (platform / 'gw_script_launch.inc').read_text()
        code = 'typedef struct {int count, cursor, failed; unsigned captures[128];} GsWarmJob;\n'
        code += function(warm, 'gs_warm_remaining')
        for name in ('gs_launch_configured', 'gw_Script_LaunchPending', 'gs_launch_begin', 'gs_launch_retire'):
            code += '\n' + function(launch, name)
        tree = c_parser.CParser().parse(re.sub(r'/\*.*?\*/|//[^\n]*', '', code, flags=re.S))
        self.func = {n.decl.name: n for n in tree.ext if isinstance(n, c_ast.FuncDef)}
        self.globals = dict(gs_launch_revision=0, gs_launch_consumed=0, gs_launch_pending=0,
                            gs_launch_owner=0, gs_launch_delivered=0, gs_launch_warned=0)
        self.revision = 1
        self.mission = 'demo'
        self.size = 0
        self.gameplay = True
        self.pending = {}
        self.released = []

    def call(self, name, *args):
        if name == 'aurora_pipeline_warm_pending': return self.pending.get(args[0], -1)
        if name == 'aurora_pipeline_warm_release': self.released.append(args[0]); return 0
        if name == 'gw_SceneLaunch_Revision': return self.revision
        if name == 'gw_SceneLaunch_Mission': return self.mission
        if name == 'gw_SceneLaunch_MazeSize': return self.size
        if name == 'gw_ScriptGame_StageGameplayScene': return self.gameplay
        if name == 'gw_log': return 0
        return super().call(name, *args)

    def eval(self, n, env):
        if isinstance(n, c_ast.Assignment) and n.op == '+=':
            container, key = self.place(n.lvalue, env)
            container[key] += self.eval(n.rvalue, env)
            return container[key]
        if isinstance(n, c_ast.UnaryOp) and n.op == '*':
            value = self.eval(n.expr, env)
            return value[0] if value else 0
        return super().eval(n, env)


s = State()
job = dict(count=3, cursor=2, failed=0, captures=[10, 11, 0])
s.pending = {10: 0, 11: 4}
assert s.call('gs_warm_remaining', job) == 5
assert job['captures'] == [0, 11, 0] and s.released == [10]
assert s.call('gs_warm_remaining', job) == 5 and s.released == [10]
s.pending[11] = 0
job['cursor'] = 3
assert s.call('gs_warm_remaining', job) == 0 and s.released == [10, 11]
job['captures'][0] = 99
assert s.call('gs_warm_remaining', job) == -1 and job['failed'] == 1
job['captures'][0] = 0
assert s.call('gs_warm_remaining', job) == -1, 'failure must remain terminal'
assert s.call('gw_Script_LaunchPending') == 1
s.gameplay = False
s.call('gs_launch_begin')
assert s.globals['gs_launch_consumed'] == 0
s.gameplay = True
s.call('gs_launch_begin')
assert s.globals['gs_launch_pending'] == s.globals['gs_launch_consumed'] == 1
s.globals.update(gs_launch_owner=7, gs_launch_delivered=1, gs_launch_warned=1)
s.call('gs_launch_retire', 8)
assert s.globals['gs_launch_owner'] == 7
s.call('gs_launch_retire', 7)
assert s.globals['gs_launch_pending'] == 1 and s.globals['gs_launch_delivered'] == 0
s.call('gs_launch_begin')
assert s.globals['gs_launch_pending'] == 0, 'scene transition cancels consumed request'
assert s.call('gw_Script_LaunchPending') == 0
s.revision += 1
assert s.call('gw_Script_LaunchPending') == 1, 'new runtime request is not consumed'
s.mission = ''
s.size = 12
assert s.call('gw_Script_LaunchPending') == 1
s.size = 0
assert s.call('gw_Script_LaunchPending') == 0
print('PASS actual scalar C AST: warm progress/release/stale/failure; launch revision/consumption/owner replacement')
