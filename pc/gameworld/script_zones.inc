/* All durable zone state belongs to script_game.c's captured .bss. */
#include "script_zones.h"
static SzState script_zones;
static SzDef script_zone_draft;
static int script_zones_mask;
static int script_zone_bits(float f) { union {float f;int i;}u;u.f=f;return u.i; }
static float script_zone_float(int b) { union {float f;int i;}u;u.i=b;return u.f; }
static int script_zone_field(const SzDef* z,int field) {
    switch(field) {
    case SZF_HANDLE:return z->handle;case SZF_OWNER:return z->owner;
    case SZF_LIVE:return z->live;case SZF_MODEL:return z->model;case SZF_N:return z->n;
    case SZF_TAGS:return z->ntags;case SZF_AREA:return script_zone_bits(z->area);
    case SZF_DX:return script_zone_bits(z->dx);case SZF_DY:return script_zone_bits(z->dy);
    }
    if(field>=SZF_X && field<SZF_X+SZ_VERTICES)return script_zone_bits(z->x[field-SZF_X]);
    if(field>=SZF_Y && field<SZF_Y+SZ_VERTICES)return script_zone_bits(z->y[field-SZF_Y]);
    return 0;
}
static char* script_zone_text(SzDef* z,int text,int byte) {
    if(byte<0)return NULL;
    if(text==SZT_NAME && byte<81)return &z->name[byte];
    if(text==SZT_LABEL && byte<81)return &z->label[byte];
    if(text==SZT_KIND && byte<32)return &z->kind[byte];
    if(text>=SZT_TAG && text<SZT_TAG+8 && byte<32)return &z->tags[text-SZT_TAG][byte];
    return NULL;
}
int ScriptGame_ZoneField(int slot,int field) {
    return slot>=0 && slot<SZ_CAP ? script_zone_field(&script_zones.z[slot],field):0;
}
int ScriptGame_ZoneText(int slot,int text,int byte) {
    char* c=slot>=0 && slot<SZ_CAP ? script_zone_text(&script_zones.z[slot],text,byte):NULL;
    return c ? (unsigned char)*c:0;
}
void ScriptGame_ZoneDraft(int field,int value) {
    if(field==-1){memset(&script_zone_draft,0,sizeof script_zone_draft);return;}
    if(field==SZF_MODEL)script_zone_draft.model=value;
    if(field==SZF_N)script_zone_draft.n=value;
    if(field==SZF_TAGS)script_zone_draft.ntags=value;
    if(field>=SZF_X && field<SZF_X+SZ_VERTICES)script_zone_draft.x[field-SZF_X]=script_zone_float(value);
    if(field>=SZF_Y && field<SZF_Y+SZ_VERTICES)script_zone_draft.y[field-SZF_Y]=script_zone_float(value);
}
void ScriptGame_ZoneDraftText(int text,int byte,int value) {
    char* c=script_zone_text(&script_zone_draft,text,byte);if(c)*c=(char)value;
}
int ScriptGame_ZoneCommit(int handle,int owner,int serial_floor) {
    int rc,old_serial=script_zones.serial,model=-1;
    script_zone_draft.owner=owner;
    if(script_zone_draft.model)model=ScriptGame_ModelSlot(script_zone_draft.model);
    if(script_zone_draft.model && model<0)rc=-5;
    else {
        if(model>=0){script_zone_draft.dx=script_zone_float(ScriptGame_ModelField(model,SM_X));script_zone_draft.dy=script_zone_float(ScriptGame_ModelField(model,SM_Y));}
        /* Native allocation watermark only influences an explicit offline write.
         * Restoring a snapshot cannot make a retained Lua handle alias a new zone. */
        if(!handle && serial_floor>script_zones.serial)script_zones.serial=serial_floor;
        rc=sz_put(&script_zones,&script_zone_draft,handle);
        if(rc<0)script_zones.serial=old_serial;
    }
    memset(&script_zone_draft,0,sizeof script_zone_draft);return rc;
}
int ScriptGame_ZoneRemove(int handle,int owner) { return sz_remove(&script_zones,handle,owner); }
void ScriptGame_ZonesRelease(int owner) {
    int i;for(i=0;i<SZ_CAP;i++)if(script_zones.z[i].live==1 && (!owner || script_zones.z[i].owner==owner))
        sz_remove(&script_zones,script_zones.z[i].handle,script_zones.z[i].owner);
}
void ScriptGame_ZonesArm(int mask) { script_zones_mask=mask&15; }
void ScriptGame_ZonesFrame(int terminal) {
    SzInput input[SZ_ENTITIES];int i;
    if(!script_zones.count && !script_zones.pending) { script_zones.nev=0;return; }
    memset(input,0,sizeof input);
    for(i=0;i<SZ_CAP;i++)if(script_zones.z[i].live==1) {
        SzDef* z=&script_zones.z[i];
        if(!Script_StageResourceOwnerAlive(z->owner)){sz_remove(&script_zones,z->handle,z->owner);continue;}
        if(z->model) {
            int at=ScriptGame_ModelSlot(z->model);
            if(at<0){sz_remove(&script_zones,z->handle,z->owner);continue;}
            z->dx=script_zone_float(ScriptGame_ModelField(at,SM_X));
            z->dy=script_zone_float(ScriptGame_ModelField(at,SM_Y));
        }
    }
    for(i=0;i<SZ_ENTITIES;i++) {
        HSD_GObj* gobj=Player_GetEntityAtIndex(i/2,i%2);Fighter* fp;
        if(terminal==1 || ScriptGame_FighterBenched(i/2) || !script_gobj_live(gobj))continue;
        fp=GET_FIGHTER(gobj);
        if(i%2 && !fp->is_sub_fighter)continue; /* dormant Zelda/Sheik is not Nana */
        if(fp->motion_id>=ftCo_MS_DeadDown && fp->motion_id<=ftCo_MS_DeadUpFallHitCameraIce)continue;
        input[i].token=(int)gobj;input[i].x=fp->cur_pos.x;input[i].y=fp->cur_pos.y;
    }
    sz_step(&script_zones,input,script_zones_mask,!terminal);
}
int ScriptGame_ZoneEntity(int entity,int field,int zone) {
    SzEntity* e;if(entity<0 || entity>=SZ_ENTITIES)return 0;e=&script_zones.e[entity];
    if(field==0)return e->token ? e->id:0;
    if(field==1)return script_zone_bits(e->x);if(field==2)return script_zone_bits(e->y);
    if(zone<0 || zone>=SZ_CAP)return 0;
    return field==3 ? e->handles[zone]:field==4 ? e->frames[zone]:0;
}
int ScriptGame_ZoneEventCount(void) { return script_zones.nev; }
int ScriptGame_ZoneEvent(int event,int field) {
    SzEvent* e;if(event<0 || event>=script_zones.nev)return 0;e=&script_zones.ev[event];
    switch(field) {
    case SZE_WHAT:return e->what;case SZE_ENTITY:return e->entity;case SZE_PORT:return e->port;
    case SZE_SUB:return e->sub;case SZE_X:return script_zone_bits(e->x);case SZE_Y:return script_zone_bits(e->y);
    case SZE_ZONE:return e->zone.handle;
    }
    if(field>=SZE_FROM && field<SZE_FROM+SZ_CAP)return e->from[field-SZE_FROM];return 0;
}
int ScriptGame_ZoneEventText(int event,int text,int byte) {
    char* c;if(event<0 || event>=script_zones.nev)return 0;
    if(byte<0)return 0;
    c=text==SZT_NAME && byte<81 ? &script_zones.ev[event].zone.name[byte]:
      text==SZT_LABEL && byte<81 ? &script_zones.ev[event].zone.label[byte]:
      text==SZT_KIND && byte<32 ? &script_zones.ev[event].zone.kind[byte]:NULL;
    return c ? (unsigned char)*c:0;
}
#include "script_zones_tests.inc"
