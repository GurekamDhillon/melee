#ifndef SCRIPT_SPINE_REFS_H
#define SCRIPT_SPINE_REFS_H
/* Canonical class1 fighter (interleaved0..11),2 item serial,3 echo handle,
 *4 scripted stage handle,5 retail stage map actor. Scalar fields:0 live,1 epoch,2 incarnation,3 definition,
 *4 owning fighter interleaved+1,5 role bits (CPU1, Geno article2). */
int ScriptGame_EntityField(int kind,int id,int field);
int ScriptGame_EntityEpoch(void);
int ScriptGame_EntityAddressField(int kind,int address,int field);
void ScriptGame_RefsSceneEnd(void);
void ScriptGame_RefsGroundBorn(int map_id,int address);
#endif
