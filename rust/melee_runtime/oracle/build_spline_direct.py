"""Direct native C oracle for all eleven spline.c definitions."""
from pathlib import Path
import subprocess,sys,os
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'rust/melee_world/gen/mtu'))
from run_tu import FLAGS,CLANG
os.environ['GW_NARROW_FPTOUI_AS_SI']='1'
OUT=ROOT/'rust/target/spline-direct';OUT.mkdir(parents=True,exist_ok=True)
def run(a):subprocess.run(a,cwd=ROOT,check=True)
lls=[]
for source in ['rust/melee_runtime/oracle/spline_driver.c','pc/gameworld/gekko_fp.c','rust/melee_runtime/oracle/wrap.c','rust/melee_runtime/oracle/stubs_hsd.c']:
    ll=OUT/(Path(source).stem+'.ll');lo=ll.with_suffix('.lo.ll')
    run([CLANG,*FLAGS,source,'-o',str(ll)])
    run([sys.executable,str(ROOT/'rust/melee_runtime/oracle/lower.py'),str(ll),str(lo)])
    lls.append(str(lo))
main=OUT/'main.c';main.write_text('#include <stdio.h>\nvoid emit(unsigned v){printf("%08x ",v);}\nvoid run_spline_case(int);\nint main(void){for(int s=1;s<=4096;s++){run_spline_case(s);puts("");}return 0;}\n')
exe=OUT/'spline.exe';run([CLANG,'-O0','-w',*lls,str(main),'-fuse-ld=lld','-o',str(exe)])
data=subprocess.run([str(exe)],check=True,capture_output=True).stdout
(ROOT/'rust/melee_runtime/tests/data/oracle_spline_direct.txt').write_bytes(data)
print('spline direct oracle:',len(data.splitlines()),'cases')
