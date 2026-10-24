from pathlib import Path
import re
p=Path('rust/melee_runtime/oracle')
s=(p/'stubs_jobj.c').read_text();names=['HSD_AObjInitEndCallBack','HSD_AObjInterpretAnim','HSD_AObjLoadDesc','HSD_AObjRemove','HSD_AObjReqAnim','HSD_AObjInvokeCallBacks','HSD_DObjAddAnimAll','HSD_DObjAnimAll','HSD_DObjRemoveAnimAllByFlags','HSD_DObjReqAnimAllByFlags','HSD_RObjAddAnimAll','HSD_RObjAnimAll','HSD_RObjRemoveAnimAllByFlags','HSD_RObjReqAnimAllByFlags','HSD_RObjGetGlobalPosition','splArcLengthPoint']
s='\n'.join(l for l in s.splitlines() if not any(re.search(r'\b'+n+r'\b',l) for n in names))+'\nint ppc_is_guest_code(unsigned a){return 0;}\nvoid* guest_scratch(unsigned s){return 0;}\nvoid* __va_arg(void*a,int t){abort();return 0;}\n'
(p/'stubs_shard_jobj_anim.c').write_text(s)
s=(p/'build_shard_jobj.sh').read_text().replace('src/sysdolphin/baselib/mtx.c','src/sysdolphin/baselib/{mtx,aobj,fobj,list,spline}.c').replace('helper_shard_jobj.c','helper_shard_jobj_anim.c').replace('stubs_shard_jobj.c','stubs_shard_jobj_anim.c').replace('g1-oracle-shard-jobj','g1-oracle-shard-jobj-anim').replace('oracle_shard_jobj.exe','oracle_shard_jobj_anim.exe').replace('data/oracle_shard_jobj.txt','data/oracle_shard_jobj_anim.txt');(p/'build_shard_jobj_anim.sh').write_text(s)
