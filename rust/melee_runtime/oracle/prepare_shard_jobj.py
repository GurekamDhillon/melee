import re,pathlib
p=pathlib.Path(__file__).parent
s=(p/'stubs_jobj.c').read_text()
names=['hsdObj','hsdInitClassInfo','hsdIsDescendantOf','hsdSearchClassInfo','hsdNew','HSD_DObjLoadDesc','HSD_RObjLoadDesc','HSD_DObjResolveRefsAll','HSD_RObjResolveRefsAll','HSD_DObjRemoveAll','HSD_RObjRemoveAll','HSD_AObjRemove','HSD_IDInsertToTable','HSD_IDGetDataFromTable','HSD_IDRemoveByIDFromTable','HSD_CObjGetCurrent','HSD_JObjDisp','HSD_RObjGetGlobalPosition']
s='\n'.join(l for l in s.splitlines() if not any(re.search(r'\b'+n+r'\b',l) for n in names))+'\n'
(p/'stubs_shard_jobj.c').write_text(s)
s=(p/'build_jobj.sh').read_text().replace('g1-oracle-jobj','g1-oracle-shard-jobj').replace('src/sysdolphin/baselib/{jobj,mtx}.c','src/sysdolphin/baselib/mtx.c').replace('helper_jobj.c','helper_shard_jobj.c').replace('driver_jobj.c','driver_shard_jobj.c').replace('stubs_jobj.c','stubs_shard_jobj.c').replace('oracle_jobj','oracle_shard_jobj').replace('-fno-short-enums','-fno-short-enums -ffp-contract=on')
(p/'build_shard_jobj.sh').write_text(s)
