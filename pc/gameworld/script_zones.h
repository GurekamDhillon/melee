/* Scalar-only PPC/native zone protocol. Floats cross as integer bit patterns. */
#ifndef SCRIPT_ZONES_H
#define SCRIPT_ZONES_H
#include "script_zones_core.h"
enum { SZF_HANDLE,SZF_OWNER,SZF_LIVE,SZF_MODEL,SZF_N,SZF_TAGS,
       SZF_AREA,SZF_DX,SZF_DY,SZF_X=16,SZF_Y=32 };
enum { SZT_NAME,SZT_LABEL,SZT_KIND,SZT_TAG=3 };
enum { SZE_WHAT,SZE_ENTITY,SZE_PORT,SZE_SUB,SZE_X,SZE_Y,SZE_ZONE,SZE_FROM=16 };
int ScriptGame_ZoneField(int slot,int field);
int ScriptGame_ZoneText(int slot,int text,int byte);
void ScriptGame_ZoneDraft(int field,int value);
void ScriptGame_ZoneDraftText(int text,int byte,int value);
int ScriptGame_ZoneCommit(int handle,int owner,int serial_floor);
int ScriptGame_ZoneRemove(int handle,int owner);
void ScriptGame_ZonesRelease(int owner);
void ScriptGame_ZonesArm(int mask);
void ScriptGame_ZonesFrame(int terminal);
int ScriptGame_ZoneEntity(int entity,int field,int zone);
int ScriptGame_ZoneEvent(int event,int field);
int ScriptGame_ZoneEventText(int event,int text,int byte);
int ScriptGame_ZoneEventCount(void);
#endif
