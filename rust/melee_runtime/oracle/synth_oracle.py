"""Generate and run synth.c's native, shipping-contraction state oracle.

Native-layout normalizations (the original C remains the reference algorithm):
* AX packed high/low address words are joined/split by value, not host byte order.
* The group's AXVPB aliases use the real SfxLoadStreamNode fields; native pointers
  invalidate their GC-only offset coincidence.
* A native sample entry has a 12-byte pointer/id prefix (GC: 8), and its allocation
  and stride round up to pointer alignment. Per-voice payload stride remains 64.
* Stream position reads pb.addr.currentAddress rather than a GC AXVPB byte offset.
All source replacements below are asserted. No C implementation is re-created.
"""
import os, re, subprocess, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
BUILD=ROOT/'rust/target/synth-oracle'
sys.path.insert(0,str(ROOT/'rust/melee_types/gen'))
import gen_types as gt
CL=r'C:/Program Files/LLVM/bin/clang.exe'
SRC=(ROOT/'src/sysdolphin/baselib/synth.c').read_text()
RS=(ROOT/'rust/melee_runtime/src/tu/synth.rs').read_text()
GEN=(ROOT/'rust/melee_types/src/gen.rs').read_text()
BUILD.mkdir(parents=True,exist_ok=True)

def sub(a,b):
    global SRC
    assert a in SRC,a
    SRC=SRC.replace(a,b)
def body(name):
    p=SRC.index('{',re.search(r'\b'+name+r'\([^;]*?\)\s*\{',SRC,re.S).start())
    depth=1;q=p+1
    while depth: depth+=(SRC[q]=='{')-(SRC[q]=='}');q+=1
    return p,q
def scoped(name,pairs):
    global SRC
    a,b=body(name);s=SRC[a:b]
    for x,y in pairs:
        assert x in s,(name,x)
        s=s.replace(x,y)
    SRC=SRC[:a]+s+SRC[b:]

foo=re.search(r'struct foo \{.*?\n\};',SRC,re.S).group()
sub(foo,'')
sub('struct SfxLoadStreamNode {',foo+'\nstruct SfxLoadStreamNode {')
prefix='''
static unsigned native_pair(const u16* p) { return ((u32)p[0]<<16)|p[1]; }
static void native_set_pair(u16* p,u32 v) { p[0]=v>>16;p[1]=v; }
static void native_add_pair(void* p,u32 v) { native_set_pair(p,native_pair(p)+v); }
#define ENTRY_SIZE(n) (((n)*64+__builtin_offsetof(struct foo,unk8)+8+7)&~7)
'''
sub('static inline s32 SfxLoadStreamDataSize',prefix+'\nstatic inline s32 SfxLoadStreamDataSize')
sub('hsd_SynthSFXLoadBuf[2] * 8 + sizeof(struct SfxLoadStreamNode)','hsd_SynthSFXLoadBuf[2] * 16 + sizeof(struct SfxLoadStreamNode)')
sub('memcpy((u8*) HSD_Synth_804D7730 + 8,','memcpy((u8*) HSD_Synth_804D7730 + __builtin_offsetof(struct foo,unk8),')
sub('*(u32*) (e + 0x14) += hsd_SynthSFXBank[bankID] * 2;','native_add_pair(e + __builtin_offsetof(struct foo,x10) + 4, hsd_SynthSFXBank[bankID] * 2);')
sub('*(u32*) ((u8*) HSD_Synth_804D7730 + k * 0x40 + 0x18) +=\n                    hsd_SynthSFXBank[bankID] * 2;','native_add_pair((u8*) HSD_Synth_804D7730 + k * 0x40 + __builtin_offsetof(struct foo,x10) + 8, hsd_SynthSFXBank[bankID] * 2);')
sub('*(u32*) ((u8*) HSD_Synth_804D7730 + k * 0x40 + 0x1C) +=\n                    hsd_SynthSFXBank[bankID] * 2;','native_add_pair((u8*) HSD_Synth_804D7730 + k * 0x40 + __builtin_offsetof(struct foo,x10) + 12, hsd_SynthSFXBank[bankID] * 2);')
sub('((u32) ((n << 6) + 0x10) >> 2)','((u32) ENTRY_SIZE(n) >> 2)')
sub('((int*) cur)[1]','((struct foo*)cur)->unk4')
scoped('HSD_SynthSFXGroupDataUnlink',[('vpb->priority','((struct SfxLoadStreamNode*)vpb)->xC'),('(int) vpb->next1','((struct SfxLoadStreamNode*)vpb)->x8')])
scoped('HSD_SynthSFXGroupDataRemove',[('(int) cur->prev','((struct SfxLoadStreamNode*)cur)->x4')])
scoped('HSD_SynthSFXGroupDataReaddress',[
 ('p = (u8*) arg0 + 0x18;','p = (u8*) ((struct SfxLoadStreamNode*)arg0 + 1);'),
 ('(uintptr_t) arg0->callback','(uintptr_t) ((struct SfxLoadStreamNode*)arg0)->x10'),
 ('arg0->userContext','((struct SfxLoadStreamNode*)arg0)->x14'),
 ('((u8*) callback - (u8*) arg0->callback)','((uintptr_t) callback - (uintptr_t) ((struct SfxLoadStreamNode*)arg0)->x10)'),
 ('arg0->priority','((struct SfxLoadStreamNode*)arg0)->xC'),
 ('*(int*) (p + 8)','((struct foo*)p)->unk8'),
 ('*(u16*) (q + 0x10)','((struct foo*)q)->x10.loopFlag'),
 ('*(u32*) (q + 0x14) += delta;','native_add_pair(&((struct foo*)q)->x10.loopAddressHi,delta);'),
 ('*(u32*) (q + 0x18) += delta;','native_add_pair(&((struct foo*)q)->x10.endAddressHi,delta);'),
 ('*(u32*) (q + 0x1C) += delta;','native_add_pair(&((struct foo*)q)->x10.currentAddressHi,delta);'),
 ('p = (u8*) ((count << 6) + (uintptr_t) p);\n        p += 0x10;','p += ENTRY_SIZE(count);'),
 ('arg0->callback = (void (*)(void*)) callback;','((struct SfxLoadStreamNode*)arg0)->x10 = (int)(uintptr_t)callback;')])
scoped('HSD_SynthSFXBankDeflag',[('(intptr_t) vpb->callback','((struct SfxLoadStreamNode*)vpb)->x10'),('vpb->userContext','((struct SfxLoadStreamNode*)vpb)->x14')])
sub('*(size_t*) &hsd_SynthSFXNodes[i]\n                        .voice[0]\n                        ->pb.addr.currentAddressHi','native_pair(&hsd_SynthSFXNodes[i].voice[0]->pb.addr.currentAddressHi)')
sub('*(u32*) ((u8*) node->voice[0] + 0x1B2)','native_pair(&node->voice[0]->pb.addr.currentAddressHi)')
sub('(size_t) hsd_SynthSFXLoadBuf','(uintptr_t) hsd_SynthSFXLoadBuf')
sub('(u32) HSD_Synth_804D7730','(uintptr_t) HSD_Synth_804D7730')
SRC,n=re.subn(r'\*\(u32\*\) &HSD_Synth_80407FD8.ratioHi\s*=\s*([^;]+);',r'native_set_pair(&HSD_Synth_80407FD8.ratioHi, \1);',SRC)
assert n==4,n
(BUILD/'synth_native.c').write_text(SRC)

# Generate native field walkers from the shared generated structs. No pointer or
# padding bytes are hashed; pointer identities are normalized to fixture regions.
records={n:re.findall(r'pub (\w+): ([^\n]+),',b) for n,b in re.findall(r'pub struct (\w+) \{(.*?)\n\}',GEN,re.S)}
aliases=dict(re.findall(r'^pub type (\w+) = (\w+);',GEN,re.M))
C=[];R=[]
def emit(c,r=None): C.append(c);R.append(c if r is None else r)
def hash_expr(ce,rex,t,depth=0):
    if t in aliases:return hash_expr(ce,rex,aliases[t],depth)
    if t.startswith('['):
        ty,num=re.fullmatch(r'\[(.*);\s*(\d+)\]',t).groups();i='j'+str(depth)
        emit(f'for(int {i}=0;{i}<{num};{i}++){{',f'for {i} in 0..{num} {{');hash_expr(f'{ce}[{i}]',f'{rex}[{i}]',ty,depth+1);emit('}')
    elif t in records:
        for f,ty in records[t]: hash_expr(f'{ce}.{f}',f'{rex}.{f}',ty,depth)
    elif t=='HSD_SynthSFXNode_user_vol_anon_37':emit(f'h(fb({ce.rsplit(".",1)[0]}.x8_float));',f'h({rex}.x8_float.to_bits());')
    elif t.startswith('*'):emit(f'h(pi((void*)({ce})));',f'h(pi(({rex}) as usize));')
    elif t.startswith('Option'):emit(f'h(fi((void*)({ce})));',f'h(fi(({rex}).map_or(0,|f|f as usize)));')
    elif t=='f32':emit(f'h(fb({ce}));',f'h(({rex}).to_bits());')
    else:emit(f'h((u32)({ce}));',f'h(({rex}) as u32);')
types=['AXVPB','HSD_SynthSFXNode','AXPBMIX','AXPBVE','AXPBSRC','AXPBADDR','AXPBADPCM','AXPBADPCMLOOP','SfxLoadStreamNode','HSD_SynthVoiceEntry']
cnames={'HSD_SynthSFXNode':'struct HSD_SynthSFXNode','SfxLoadStreamNode':'struct SfxLoadStreamNode','HSD_SynthVoiceEntry':'struct foo'}
walk_c=[];walk_r=[]
for ty in types:
    C=[];R=[];hash_expr('(*p)','(*p)',ty)
    walk_c.append(f'static void hash_{ty}(const {cnames.get(ty,ty)}*p){{\n'+ '\n'.join(C)+'\n}')
    walk_r.append(f'unsafe fn hash_{ty}(p:*const {ty}){{\n'+ '\n'.join(R)+'\n}')

# C/Rust hooks have identical call IDs and semantic argument transcripts. AX
# hardware hooks record calls without inventing writes; direct TU writes still
# appear in the full state hash. This isolates the TU from unported AX services.
sigbody=re.search(r'    hooks! \{(.*?)\n    \}',RS,re.S).group(1)
hooks=[]
for name,args,ret in re.findall(r'fn (\w+)\((.*?)\)(?:->([^;]+))?;',sigbody):
    args=re.findall(r'(\w+):((?:Option<.*?>)|[^,]+)(?:,|$)',args)
    hooks.append((name,args,ret or '()'))
idx=gt.ci.Index.create();tu=idx.parse('src/sysdolphin/baselib/synth.c',args=gt.NAT)
decls={c.spelling:c for c in tu.cursor.walk_preorder() if c.kind==gt.K.FUNCTION_DECL}
def cdecl(t,n):
    s=t.spelling
    if '(*)' in s:return s.replace('(*)',f'(*{n})')
    return s+' '+n
cbnames=['HSD_SynthSFXHeaderLoadCallback','HSD_SynthSFXSampleLoadCallback','HSD_SynthSFXGroupDataReaddressCallback','dropcallback','HSD_SynthCallback','HSD_SynthResetStreamCounters','HSD_SynthPStreamHakoHeaderCallback','HSD_SynthPStreamFirstHakoDataCallback','HSD_SynthPStreamFirstHakoHeaderCallback','HSD_SynthPStreamHeaderCallback','cb1','cb2','cb3','load_cb','wait_cb']
stubs_c=[];stubs_r=[];install=[]
for id,(name,args,ret) in enumerate(hooks,1):
    if name=='OSReport':
        stubs_c.append('void OSReport(char*fmt,...){}');stubs_r.append('fn stub_OSReport(_m:&str,_v:i32){}');install.append('ext::hook::OSReport=Some(stub_OSReport);');continue
    d=decls[name];ca=list(d.get_arguments());assert len(ca)==len(args),(name,ca,args)
    cc=[];rr=[]
    for j,((a,t),arg) in enumerate(zip(args,ca)):
        if name=='DVDConvertPathToEntrynum':
            cc.append(f'for(const unsigned char*q=(const unsigned char*){a};*q;q++)t(*q);');rr.append(f'let mut q={a}.cast::<u8>();while *q!=0{{t(*q as u32);q=q.add(1);}}')
        elif t.startswith('Option') or t=='DevCallback':cc.append(f't(fi((void*){a}));');rr.append(f't(fi({a}.map_or(0,|f|f as usize)));')
        elif t.startswith('*'):
            if name.startswith('AXSetVoice') and j==1:
                ty=t.replace('*mut ','');cc.append(f'hash_{ty}({a});');rr.append(f'hash_{ty}({a});')
            else:cc.append(f't(pi((void*){a}));');rr.append(f't(pi({a} as usize));')
        elif t=='f32':cc.append(f't(fb({a}));');rr.append(f't({a}.to_bits());')
        elif t=='usize' and name=='HSD_DevComRequest' and a in ('src','dst','args'):
            cc.append(f't(pi((void*){a}));');rr.append(f't(pi({a}));')
        else:cc.append(f't((u32){a});');rr.append(f't({a} as u32);')
    # Hash pointee parameters on the trace channel, not state channel.
    cc=[s.replace('hash_','trace_') for s in cc];rr=[s.replace('hash_','trace_') for s in rr]
    ctail='';rtail=''
    if name=='OSAllocFromHeap':ctail='return heap;';rtail='heap.as_mut_ptr().cast()'
    elif name=='AXAcquireVoice':
        ctail='u32 call=acq++;if(acquire_mode==1||(acquire_mode==2&&call==1))return 0;return &voices[8+(acquire_mode==3?0:call%2)];'
        rtail='let call=acq;acq+=1;if acquire_mode==1||(acquire_mode==2&&call==1){core::ptr::null_mut()}else{&raw mut voices[8+if acquire_mode==3{0}else{(call%2) as usize}]} '
    elif name=='wait_idle':ctail='HSD_Synth_804D7778=0;';rtail='HSD_Synth_804D7778=0;'
    elif ret!='()':ctail='return rnd();';rtail='rnd() as _'
    stubs_c.append(cdecl(d.result_type,name)+'('+','.join(cdecl(a.type,n) for a,(n,_) in zip(ca,args))+'){t('+str(id)+');'+''.join(cc)+ctail+'}')
    stubs_r.append('fn stub_'+name+'('+','.join(a+':'+t for a,t in args)+')->'+ret+'{unsafe{t('+str(id)+');'+''.join(rr)+rtail+'}}')
    install.append('ext::hook::'+name+'=Some(stub_'+name+');')

globals=[]
for name,ty in re.findall(r'(\w+):([^,\n]+(?:\([^\n]*?\)>)?)',re.search(r'zero_statics! \{(.*?)\n\}',RS,re.S).group(1)):
    if name.startswith('unsafe'):continue
    globals.append((name,ty.strip()))
# Parse globals robustly from generated Rust declaration expansion grammar.
globals=[]
block=re.search(r'zero_statics! \{(.*?)\n\}',RS,re.S).group(1)
for m in re.finditer(r'(\w+):\s*(\[[^\]]+\]|Option<[^>]+>|\*mut \w+|[iu]\d+|\w+)',block):globals.append(m.groups())
globals += [('HSD_Synth_804D6018','i32'),('HSD_Synth_804D6030','f32'),('HSD_Synth_80407FD8','AXPBSRC')]
statec=[];stater=[]
for name,ty in globals:
    # Anonymous static C records have exactly these native generated fields.
    C=[];R=[];hash_expr(name,name,ty)
    statec+=C;stater+=R
statec+=['for(int i=0;i<64;i++)hash_AXVPB(&voices[i]);','hash_HSD_SynthVoiceEntry((struct foo*)sample);','hash_SfxLoadStreamNode((struct SfxLoadStreamNode*)group);','hash_HSD_SynthVoiceEntry((struct foo*)((struct SfxLoadStreamNode*)group+1));']
stater+=['for i in 0..64{hash_AXVPB(&raw const voices[i]);}','hash_HSD_SynthVoiceEntry(sample.as_ptr().cast());','hash_SfxLoadStreamNode(group.as_ptr().cast());','hash_HSD_SynthVoiceEntry(group.as_ptr().cast::<SfxLoadStreamNode>().add(1).cast());']

def run(cmd):
    r=subprocess.run([str(v) for v in cmd],cwd=ROOT,capture_output=True,text=True)
    if r.returncode:raise RuntimeError(r.stdout+r.stderr)
    return r.stdout

# The remainder (fixtures + per-function calls) is kept in a small companion
# description so a failed case prints the function name and independent seed.
exec((HERE/'synth_cases.py').read_text())
