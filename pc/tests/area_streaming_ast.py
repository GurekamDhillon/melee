"""Execute scalar area C through pycparser for sandboxed behavioral checks.

This is a deterministic host fixture for the actual C functions, not a native
compile, MEM1 snapshot test or timing measurement. Native companion:
area_streaming.py. Run from any directory with Python and pycparser installed.
"""
from copy import deepcopy
from pathlib import Path
import re
from pycparser import c_ast, c_parser


class Return(Exception):
    def __init__(self, value): self.value = value


class Break(Exception): pass
class Continue(Exception): pass


class Fixture:
    def __init__(self):
        game = Path(__file__).resolve().parents[2] / 'pc/gameworld'
        code = (game / 'script_largemap.inc').read_text()
        code = code[code.index('int ScriptGame_AreaFind'):code.index('int ScriptGame_StageStat')]
        gates = (game / 'script_game.c').read_text()
        gates = gates[gates.index('int ScriptGame_AreaVisible'):gates.index('static int script_area_retiring')]
        code = re.sub(r'/\*.*?\*/', '', gates + code, flags=re.S)
        tree = c_parser.CParser().parse(code)
        self.func = {n.decl.name: n for n in tree.ext if isinstance(n, c_ast.FuncDef)}
        self.state = {
            'area': [dict(owner=0, handle=0, state=0, drain=0, name=[0]*48) for _ in range(64)],
            'instance': [dict(handle=0, area=0) for _ in range(256)],
            'line': [dict(handle=0, active=0, area=0) for _ in range(768)],
            'target': [dict(handle=0, active=0, area=0) for _ in range(128)],
            'cap': 768, 'loading_area': 0,
        }
        self.globals = dict(script_stage=self.state, SCRIPT_STAGE_AREAS=64,
                            SCRIPT_MESH_INSTANCES=256, SCRIPT_STAGE_TARGETS=128)
        self.name = 'room'
        self.deletes = self.begin = self.end = 0
        self.refuse = False

    def call(self, name, *args):
        if name == 'Script_AreaNameByte':
            return ord(self.name[args[0]]) if args[0] < len(self.name) else 0
        if name == 'OSReport' or name == 'mpScriptInvalidateBounding': return 0
        if name == 'mpScriptBatchBegin': self.begin += 1; return 0
        if name == 'mpScriptBatchEnd': self.end += 1; return 0
        if name == 'memset':
            for k, v in args[0].items(): args[0][k] = [0]*len(v) if isinstance(v, list) else 0
            return 0
        if name in ('ScriptGame_ModelDespawn', 'ScriptGame_StageRemove'):
            if self.refuse: return 0
            pools = ['instance'] if name.endswith('Despawn') else ['line', 'target']
            for pool in pools:
                for row in self.state[pool]:
                    if row['handle'] == args[0] and (row.get('active', 1)):
                        row['handle'] = 0
                        if 'active' in row: row['active'] = 0
                        self.deletes += 1
                        return 1
            return 0
        fn = self.func[name]
        env = {p.name: v for p, v in zip(fn.decl.type.args.params or [], args)}
        try: self.exec(fn.body, env)
        except Return as r: return r.value
        return 0

    def place(self, n, env):
        if isinstance(n, c_ast.ID): return (env if n.name in env else self.globals), n.name
        if isinstance(n, c_ast.StructRef): return self.eval(n.name, env), n.field.name
        if isinstance(n, c_ast.ArrayRef): return self.eval(n.name, env), self.eval(n.subscript, env)
        raise TypeError(type(n))

    def eval(self, n, env):
        if n is None: return 0
        if isinstance(n, (c_ast.ID, c_ast.StructRef, c_ast.ArrayRef)):
            container, key = self.place(n, env); return container[key]
        if isinstance(n, c_ast.Constant):
            return n.value if n.type == 'string' else int(n.value, 0)
        if isinstance(n, c_ast.Cast): return self.eval(n.expr, env)
        if isinstance(n, c_ast.BinaryOp):
            a = self.eval(n.left, env)
            if n.op == '&&': return int(bool(a) and bool(self.eval(n.right, env)))
            if n.op == '||': return int(bool(a) or bool(self.eval(n.right, env)))
            b = self.eval(n.right, env)
            return {'+': lambda: a+b, '-': lambda: a-b, '<': lambda: int(a<b),
                    '>': lambda: int(a>b), '<=': lambda: int(a<=b), '>=': lambda: int(a>=b),
                    '==': lambda: int(a==b), '!=': lambda: int(a!=b)}[n.op]()
        if isinstance(n, c_ast.UnaryOp):
            if n.op == 'sizeof': return 1
            if n.op == '&': return self.eval(n.expr, env)
            if n.op == '!': return int(not self.eval(n.expr, env))
            if n.op == '-': return -self.eval(n.expr, env)
            if n.op in ('++', 'p++'):
                c, k = self.place(n.expr, env); old = c[k]; c[k] += 1
                return old if n.op == 'p++' else c[k]
        if isinstance(n, c_ast.Assignment):
            c, k = self.place(n.lvalue, env); value = self.eval(n.rvalue, env)
            if n.op == '-=': value = c[k] - value
            c[k] = value; return value
        if isinstance(n, c_ast.FuncCall):
            return self.call(n.name.name, *(self.eval(x, env) for x in (n.args.exprs if n.args else [])))
        if isinstance(n, c_ast.TernaryOp):
            return self.eval(n.iftrue if self.eval(n.cond, env) else n.iffalse, env)
        raise TypeError(f'Unsupported C expression {type(n).__name__}')

    def exec(self, n, env):
        if n is None: return
        if isinstance(n, c_ast.Compound):
            for x in n.block_items or []: self.exec(x, env)
        elif isinstance(n, c_ast.Decl): env[n.name] = self.eval(n.init, env) if n.init else 0
        elif isinstance(n, c_ast.If): self.exec(n.iftrue if self.eval(n.cond, env) else n.iffalse, env)
        elif isinstance(n, c_ast.Return): raise Return(self.eval(n.expr, env))
        elif isinstance(n, c_ast.Break): raise Break()
        elif isinstance(n, c_ast.Continue): raise Continue()
        elif isinstance(n, (c_ast.For, c_ast.While)):
            if isinstance(n, c_ast.For): self.exec(n.init, env)
            guard = 0
            while self.eval(n.cond, env):
                guard += 1
                assert guard <= 4096, 'nonterminating fixture loop'
                try: self.exec(n.stmt, env)
                except Break: break
                except Continue: pass
                if isinstance(n, c_ast.For): self.eval(n.next, env)
        elif isinstance(n, c_ast.EmptyStatement): pass
        else: self.eval(n, env)


f = Fixture()
area = f.call('ScriptGame_AreaPrepare', 7, 101)
assert area > 0 and not f.call('ScriptGame_AreaVisible', area)
f.state['line'][0].update(handle=33, active=1, area=area)
f.state['instance'][0].update(handle=44, area=area)
assert f.call('ScriptGame_AreaPrepare', 7, 102) == -1, 'nesting refused'
f.call('ScriptGame_AreaEnd', 1)
saved = deepcopy(f.state)
assert f.call('ScriptGame_AreaStatus', 7, 101) == 1
assert not f.call('ScriptGame_AreaActivate', 8, 101), 'ownership checked'
assert f.call('ScriptGame_AreaActivate', 7, 101)
assert f.call('ScriptGame_AreaVisible', area) and f.deletes == 0
assert f.call('ScriptGame_AreaBegin', 7) == 0, 'legacy idempotence'
assert f.call('ScriptGame_AreaUnload', 7) == 1
assert not f.call('ScriptGame_AreaVisible', area) and f.deletes == 0
assert f.call('ScriptGame_AreaStatus', 7, 101) == 3
f.refuse = True
f.call('ScriptGame_AreaDrain')
assert f.call('ScriptGame_AreaStatus', 7, 101) == 3 and f.state['line'][0]['active']
f.refuse = False
f.call('ScriptGame_AreaDrain')
assert f.deletes == 2
for _ in range(1024):
    if not f.call('ScriptGame_AreaStatus', 7, 101): break
    before = f.deletes
    f.call('ScriptGame_AreaDrain')
    assert f.deletes - before <= 2, 'bounded drain'
assert f.call('ScriptGame_AreaStatus', 7, 101) == 0
f.state = deepcopy(saved); f.globals['script_stage'] = f.state
assert f.call('ScriptGame_AreaStatus', 7, 101) == 1
assert f.state['line'][0]['active'] and f.state['instance'][0]['handle'] == 44
assert f.call('ScriptGame_AreaActivate', 7, 101)
f.name = 'failed'; assert f.call('ScriptGame_AreaPrepare', 7, 102) > 0
f.call('ScriptGame_AreaEnd', 0)
assert f.call('ScriptGame_AreaStatus', 7, 102) == 3
assert not f.call('ScriptGame_AreaActivate', 7, 102)
assert f.begin == f.end
f = Fixture()
area = f.call('ScriptGame_AreaPrepare', 7, 201)
for i in range(64): f.state['line'][i].update(handle=1000+i, active=1, area=area)
f.state['instance'][0].update(handle=2000, area=area)
f.call('ScriptGame_AreaEnd', 1)
assert f.call('ScriptGame_AreaActivate', 7, 201)
f.call('ScriptGame_AreaUnload', 7)
for frame in range(33):
    before = f.deletes
    f.call('ScriptGame_AreaDrain')
    assert f.deletes - before <= 2
    if frame < 32: assert f.state['instance'][0]['handle'] == 2000
assert f.deletes == 65 and f.call('ScriptGame_AreaStatus', 7, 201) == 0
# Retiring slots cannot alias a new handle, even for the same owner/name.
assert f.call('ScriptGame_AreaPrepare', 7, 202) > 0
f.call('ScriptGame_AreaEnd', 1)
assert not f.call('ScriptGame_AreaActivate', 7, 201)
assert f.call('ScriptGame_AreaActivate', 7, 202)
print('PASS actual scalar C AST: prepare/activate/ownership/nesting/idempotence/refused cleanup/bounded drain/snapshot/failure')
