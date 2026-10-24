"""Rebuild the field-by-field seeded texp C oracle with shipping FP lowering."""
from pathlib import Path
import subprocess, sys, os
os.environ["GW_NARROW_FPTOUI_AS_SI"]="1"
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'rust/melee_world/gen/mtu'))
from run_tu import FLAGS, CLANG
OUT=ROOT/'rust/target/texp-oracle'
OUT.mkdir(parents=True,exist_ok=True)
def run(a):
    subprocess.run(a,cwd=ROOT,check=True)
ll=OUT/'texp.ll'; lo=OUT/'texp.lo.ll'
run([CLANG,*FLAGS,'rust/melee_runtime/oracle/texp_driver.c','-o',str(ll)])
run([sys.executable,str(ROOT/'rust/melee_runtime/oracle/lower.py'),str(ll),str(lo)])
main=OUT/'main.c'
main.write_text('#include <stdio.h>\n#include <stdlib.h>\nvoid emit(unsigned v){printf("%08x ",v);}\nvoid oracle_abort(void){fflush(stdout);exit(2);}\nvoid run_case(int,int);\nint main(void){for(int op=0;op<14;op++)for(int s=1;s<=256;s++){run_case(op,s);puts("");}return 0;}\n')
exe=OUT/'texp.exe'
run([CLANG,'-O0','-w',str(lo),str(main),'-fuse-ld=lld','-o',str(exe)])
p=subprocess.run([str(exe)],cwd=ROOT,capture_output=True); data=p.stdout
if p.returncode: print(data.decode()[-2000:]); sys.exit(p.returncode)
(ROOT/'rust/melee_runtime/tests/data/oracle_texp.txt').write_bytes(data)
print('texp oracle:',len(data.splitlines()),'cases;',ll.read_text().count('call float @llvm.fmuladd'),'f32 fmuladd sites')
